//! HTTPS downloads through Windows' own TLS (system certificates), with progress, retries and
//! SHA-256 checks.

use crate::ui::Ui;
use std::path::{Path, PathBuf};

#[cfg_attr(not(windows), allow(dead_code))]
pub fn partial(dest: &Path) -> PathBuf {
    let mut name = dest.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    dest.with_file_name(name)
}

#[cfg_attr(not(windows), allow(dead_code))]
pub fn progress(
    label: &str,
    got: u64,
    total: Option<u64>,
    speed: f64,
) -> String {
    use crate::util::mb;
    let speed = format!("{:.1} МБ/с", speed / 1_048_576.0);
    match total {
        Some(total) if total > 0 => format!(
            "      {label}: {} из {} ({}%), {speed}",
            mb(got),
            mb(total),
            got * 100 / total
        ),
        _ => format!("      {label}: {}, {speed}", mb(got)),
    }
}

#[cfg(windows)]
mod imp {
    use super::{partial, progress};
    use crate::ui::Ui;
    use sha2::{Digest, Sha256};
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::Path;
    use std::sync::{Arc, OnceLock};
    use std::time::{Duration, Instant};

    fn agent() -> Result<&'static ureq::Agent, String> {
        static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
        if let Some(agent) = AGENT.get() {
            return Ok(agent);
        }
        let tls =
            native_tls::TlsConnector::new().map_err(|e| format!("TLS: {e}"))?;
        let agent = ureq::AgentBuilder::new()
            .tls_connector(Arc::new(tls))
            .timeout_connect(Duration::from_secs(30))
            .timeout_read(Duration::from_secs(120))
            .user_agent(concat!("U3drinthBuilder/", env!("CARGO_PKG_VERSION")))
            .try_proxy_from_env(true)
            .build();
        Ok(AGENT.get_or_init(|| agent))
    }

    fn failure(url: &str, e: ureq::Error) -> String {
        match e {
            ureq::Error::Status(code, _) => {
                format!("{url}: сервер ответил {code}")
            }
            ureq::Error::Transport(t) => format!("{url}: {t}"),
        }
    }

    pub fn fetch_text(url: &str) -> Result<String, String> {
        let response = agent()?.get(url).call().map_err(|e| failure(url, e))?;
        response.into_string().map_err(|e| format!("{url}: {e}"))
    }

    pub fn download(
        ui: &Ui,
        url: &str,
        dest: &Path,
        sha256: Option<&str>,
        label: &str,
    ) -> Result<(), String> {
        let mut pause = 5;
        for attempt in 1..=3 {
            match once(ui, url, dest, sha256, label) {
                Ok(()) => return Ok(()),
                Err(e) if attempt < 3 => {
                    ui.warn(&format!("{label}: {e}. Повтор через {pause} с"));
                    std::thread::sleep(Duration::from_secs(pause));
                    pause *= 3;
                }
                Err(e) => return Err(format!("{label}: {e}")),
            }
        }
        unreachable!()
    }

    fn once(
        ui: &Ui,
        url: &str,
        dest: &Path,
        sha256: Option<&str>,
        label: &str,
    ) -> Result<(), String> {
        let response = agent()?.get(url).call().map_err(|e| failure(url, e))?;
        let total: Option<u64> = response
            .header("Content-Length")
            .and_then(|v| v.trim().parse().ok());
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let part = partial(dest);
        let mut file = File::create(&part)
            .map_err(|e| format!("{}: {e}", part.display()))?;
        let mut reader = response.into_reader();
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 256 * 1024];
        let mut got = 0u64;
        let started = Instant::now();
        let mut drawn = Instant::now() - Duration::from_secs(1);
        loop {
            let n = match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    drop(file);
                    let _ = fs::remove_file(&part);
                    return Err(format!("загрузка прервалась: {e}"));
                }
            };
            file.write_all(&buf[..n])
                .map_err(|e| format!("{}: {e}", part.display()))?;
            hasher.update(&buf[..n]);
            got += n as u64;
            if drawn.elapsed() >= Duration::from_millis(200) {
                drawn = Instant::now();
                let speed =
                    got as f64 / started.elapsed().as_secs_f64().max(0.001);
                ui.status(&progress(label, got, total, speed));
            }
        }
        file.flush()
            .map_err(|e| format!("{}: {e}", part.display()))?;
        drop(file);
        ui.clear_status();
        if let Some(total) = total {
            if got != total {
                let _ = fs::remove_file(&part);
                return Err(format!(
                    "получено {} из {}",
                    crate::util::mb(got),
                    crate::util::mb(total)
                ));
            }
        }
        let digest = format!("{:x}", hasher.finalize());
        if let Some(want) = sha256 {
            if !digest.eq_ignore_ascii_case(want.trim()) {
                let _ = fs::remove_file(&part);
                return Err(format!("контрольная сумма не совпала: ожидалась {want}, получена {digest}"));
            }
        }
        let _ = fs::remove_file(dest);
        fs::rename(&part, dest)
            .map_err(|e| format!("{}: {e}", dest.display()))?;
        ui.log_line(&format!(
            "загружено {url} -> {} ({got} байт, sha256 {digest})",
            dest.display()
        ));
        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::ui::Ui;
    use std::path::Path;

    pub fn fetch_text(url: &str) -> Result<String, String> {
        Err(format!("{url}: загрузки работают только в Windows"))
    }

    pub fn download(
        _ui: &Ui,
        url: &str,
        _dest: &Path,
        _sha256: Option<&str>,
        _label: &str,
    ) -> Result<(), String> {
        Err(format!("{url}: загрузки работают только в Windows"))
    }
}

pub fn fetch_text(url: &str) -> Result<String, String> {
    imp::fetch_text(url)
}

pub fn download(
    ui: &Ui,
    url: &str,
    dest: &Path,
    sha256: Option<&str>,
    label: &str,
) -> Result<(), String> {
    imp::download(ui, url, dest, sha256, label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_files_sit_next_to_the_download() {
        assert_eq!(
            partial(Path::new("d/node.zip")),
            Path::new("d/node.zip.part")
        );
    }

    #[test]
    fn progress_reads_with_and_without_a_size() {
        assert_eq!(
            progress("Node.js", 1_048_576, Some(4_194_304), 2_097_152.0),
            "      Node.js: 1.0 МБ из 4.0 МБ (25%), 2.0 МБ/с"
        );
        assert_eq!(
            progress("pnpm", 0, None, 0.0),
            "      pnpm: 0.0 МБ, 0.0 МБ/с"
        );
    }
}
