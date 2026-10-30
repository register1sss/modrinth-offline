//! Explicit, user-initiated installation only. No model-generated commands or paths.
use super::assistant::{Result, MODEL};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, OnceLock,
};

#[derive(Clone, Serialize)]
pub struct SetupProgress {
    pub running: bool,
    pub stage: String,
    pub completed: u64,
    pub total: u64,
    pub cancellable: bool,
    pub error: Option<String>,
}
impl Default for SetupProgress {
    fn default() -> Self {
        Self {
            running: false,
            stage: "idle".into(),
            completed: 0,
            total: 0,
            cancellable: false,
            error: None,
        }
    }
}
fn progress() -> &'static Mutex<SetupProgress> {
    static P: OnceLock<Mutex<SetupProgress>> = OnceLock::new();
    P.get_or_init(|| Mutex::new(SetupProgress::default()))
}
static CANCEL: AtomicBool = AtomicBool::new(false);
pub fn get_progress() -> Result<SetupProgress> {
    progress()
        .lock()
        .map(|p| p.clone())
        .map_err(|e| e.to_string())
}
pub fn cancel() -> Result<SetupProgress> {
    if get_progress()?.cancellable {
        CANCEL.store(true, Ordering::SeqCst);
    }
    get_progress()
}
fn update(stage: &str, completed: u64, total: u64, cancellable: bool) {
    if let Ok(mut p) = progress().lock() {
        p.stage = stage.into();
        p.completed = completed;
        p.total = total;
        p.cancellable = cancellable;
    }
}
fn check_cancel() -> Result<()> {
    if CANCEL.load(Ordering::SeqCst) {
        Err("Installation cancelled".into())
    } else {
        Ok(())
    }
}
pub fn begin() -> Result<SetupProgress> {
    #[cfg(not(windows))]
    {
        return Err(
            "Automatic AI installation is available on Windows x64".into()
        );
    }
    #[cfg(windows)]
    {
        let mut p = progress().lock().map_err(|e| e.to_string())?;
        if p.running {
            return Ok(p.clone());
        }
        CANCEL.store(false, Ordering::SeqCst);
        *p = SetupProgress {
            running: true,
            stage: "checking".into(),
            cancellable: true,
            ..Default::default()
        };
        let result = p.clone();
        drop(p);
        tokio::spawn(async {
            let result = install().await;
            if let Ok(mut p) = progress().lock() {
                p.running = false;
                p.cancellable = false;
                match result {
                    Ok(()) => {
                        p.stage = "ready".into();
                        p.error = None;
                    }
                    Err(e) => {
                        p.stage = if CANCEL.load(Ordering::SeqCst) {
                            "cancelled"
                        } else {
                            "error"
                        }
                        .into();
                        p.error = Some(e);
                    }
                }
            }
        });
        Ok(result)
    }
}

#[cfg(windows)]
async fn install() -> Result<()> {
    use serde_json::{json, Value};
    use std::{path::PathBuf, time::Duration};
    use tokio::{io::AsyncWriteExt, process::Command};
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let error = |e: reqwest::Error| e.to_string();
    let local = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(3))
        .build()
        .map_err(error)?;
    let reachable = || async {
        local
            .get("http://127.0.0.1:11434/api/tags")
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    };
    let runtime = PathBuf::from(
        std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is missing")?,
    )
    .join("Programs/Ollama/ollama.exe");
    if !reachable().await {
        // Use the official signed per-user installer, not a script from model output.
        // Redirections are HTTPS-only and restricted to official release hosts.
        update("download_runtime", 0, 0, true);
        let download = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(1800))
            .redirect(reqwest::redirect::Policy::custom(|a| {
                if a.previous().len() > 5 || !trusted_download_url(a.url()) {
                    a.error("Untrusted installer redirect")
                } else {
                    a.follow()
                }
            }))
            .build()
            .map_err(error)?;
        let mut response = download
            .get("https://ollama.com/download/OllamaSetup.exe")
            .send()
            .await
            .map_err(error)?
            .error_for_status()
            .map_err(error)?;
        let total = response.content_length().unwrap_or(0);
        const MAX: u64 = 4 * 1024 * 1024 * 1024;
        if total > MAX {
            return Err("Installer is too large".into());
        }
        let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let setup = temp.path().join("OllamaSetup.exe");
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&setup)
            .await
            .map_err(|e| e.to_string())?;
        let mut completed = 0;
        loop {
            check_cancel()?;
            let chunk = tokio::select! {
                c=response.chunk()=>c.map_err(error)?,
                _=tokio::time::sleep(Duration::from_millis(250))=>{check_cancel()?;continue;}
            };
            let Some(chunk) = chunk else { break };
            completed += chunk.len() as u64;
            if completed > MAX {
                return Err("Installer is too large".into());
            }
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            update("download_runtime", completed, total, true);
        }
        file.flush().await.map_err(|e| e.to_string())?;
        drop(file);
        if total > 0 && completed != total {
            return Err("Incomplete installer download".into());
        }
        check_cancel()?;
        update("verify_runtime", 0, 0, false);
        // Authenticode chain and exact publisher name are checked before execution.
        // The path is passed through an environment variable, never interpolated as code.
        let verify=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command",
            "$ErrorActionPreference='Stop'; $s=Get-AuthenticodeSignature -LiteralPath $env:U3_SETUP_VERIFY; if($s.Status -ne 'Valid'){exit 2}; $n=$s.SignerCertificate.GetNameInfo([System.Security.Cryptography.X509Certificates.X509NameType]::SimpleName,$false); if($n -notmatch '^Ollama(?:,? Inc\\.?)?$'){exit 3}; exit 0"])
            .env("U3_SETUP_VERIFY",&setup).creation_flags(CREATE_NO_WINDOW).kill_on_drop(true).status();
        let verified = tokio::time::timeout(Duration::from_secs(120), verify)
            .await
            .map_err(|_| "Signature verification timed out")?
            .map_err(|e| e.to_string())?;
        if !verified.success() {
            return Err("Ollama publisher/signature could not be verified. Nothing was executed".into());
        }
        check_cancel()?;
        update("install_runtime", 0, 0, false);
        let code = Command::new(&setup)
            .args([
                "/VERYSILENT",
                "/SUPPRESSMSGBOXES",
                "/NORESTART",
                "/SP-",
                "/NOICONS",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await
            .map_err(|e| e.to_string())?;
        if !code.success() {
            return Err(format!("Ollama installer exited with {code}"));
        }
        update("start_runtime", 0, 0, false);
        if !reachable().await {
            // No PATH lookup and no shell. Fixed executable produced by the verified installer.
            Command::new(&runtime)
                .arg("serve")
                .creation_flags(CREATE_NO_WINDOW)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        let mut ready = false;
        for _ in 0..120 {
            if reachable().await {
                ready = true;
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        if !ready {
            return Err("Ollama was installed but did not start. Restart U3drinth and retry".into());
        }
    }
    check_cancel()?;
    update("download_model", 0, 0, true);
    let mut response = local
        .post("http://127.0.0.1:11434/api/pull")
        .json(&json!({"model":MODEL,"stream":true}))
        .timeout(Duration::from_secs(7200))
        .send()
        .await
        .map_err(error)?
        .error_for_status()
        .map_err(error)?;
    let mut buffer = Vec::new();
    let mut success = false;
    loop {
        check_cancel()?;
        let chunk = tokio::select! {c=response.chunk()=>c.map_err(error)?,_=tokio::time::sleep(Duration::from_millis(250))=>{check_cancel()?;continue;}};
        let Some(chunk) = chunk else { break };
        buffer.extend_from_slice(&chunk);
        if buffer.len() > 1_000_000 {
            return Err("Model progress response too large".into());
        }
        while let Some(i) = buffer.iter().position(|b| *b == b'\n') {
            let line: Vec<_> = buffer.drain(..=i).collect();
            if line.iter().all(|b| b.is_ascii_whitespace()) {
                continue;
            }
            let v: Value =
                serde_json::from_slice(&line).map_err(|e| e.to_string())?;
            if let Some(e) = v["error"].as_str() {
                return Err(e.into());
            }
            if v["status"] == "success" {
                success = true;
            }
            update(
                "download_model",
                v["completed"].as_u64().unwrap_or(0),
                v["total"].as_u64().unwrap_or(0),
                true,
            );
        }
    }
    if !success {
        return Err(
            "Model download did not finish. Click Install AI to retry".into()
        );
    }
    let state = crate::assistant::status().await?;
    if state["ready"] != true {
        return Err("Model verification failed. Retry installation".into());
    }
    Ok(())
}

#[cfg(any(windows, test))]
fn trusted_download_url(url: &url::Url) -> bool {
    url.scheme() == "https"
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
        && matches!(
            url.host_str(),
            Some(
                "ollama.com"
                    | "github.com"
                    | "objects.githubusercontent.com"
                    | "release-assets.githubusercontent.com"
            )
        )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_untrusted_runtime_downloads() {
        for u in [
            "http://ollama.com/a",
            "https://ollama.com.evil/a",
            "https://evil/a",
            "https://github.com:444/a",
            "https://evil@github.com/a",
        ] {
            assert!(!trusted_download_url(&url::Url::parse(u).unwrap()), "{u}");
        }
        assert!(trusted_download_url(
            &url::Url::parse(
                "https://release-assets.githubusercontent.com/release/a"
            )
            .unwrap()
        ));
    }
}
