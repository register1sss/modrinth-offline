//! What the build needs, each installed once: Visual Studio Build Tools (Microsoft's installer,
//! system-wide) and, in the builder's own folder, Rust, Node.js, pnpm and Python.

use crate::net;
use crate::proc;
use crate::source;
use crate::ui::Ui;
use crate::util;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const NODE: &str = "20.20.2";
const NODE_SHA256: &str =
    "dc3700fdd57a63eedb8fd7e3c7baaa32e6a740a1b904167ff4204bc68ed8bf77";
pub const PNPM: &str = "9.4.0";
const PNPM_SHA256: &str =
    "2126925905594d8be3ae61d8433becd969c872ddc46181474d0b41c6e6ac130c";
pub const PYTHON: &str = "3.13.17";
const PYTHON_SHA256: &str =
    "9e35319bffedab8b36ad44d8f377cb4f1733bcec257a0b6d04d94f01486ddb38";
const RUSTUP_INIT: &str = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe";
const VS_BUILD_TOOLS: &str = "https://aka.ms/vs/17/release/vs_BuildTools.exe";
const VC_TOOLS: &str = "Microsoft.VisualStudio.Component.VC.Tools.x86.x64";

pub struct Tools {
    home: PathBuf,
    root: PathBuf,
    rustup_home: PathBuf,
    cargo_home: PathBuf,
    node: PathBuf,
    pnpm_dir: PathBuf,
    pnpm_version: String,
    python_dir: PathBuf,
}

impl Tools {
    pub fn new(home: &Path, pnpm: Option<&str>) -> Tools {
        let root = home.join("tools");
        let pnpm_version = pnpm.unwrap_or(PNPM).to_string();
        Tools {
            home: home.to_path_buf(),
            rustup_home: root.join("rustup"),
            cargo_home: root.join("cargo"),
            node: root.join(format!("node-v{NODE}-win-x64")),
            pnpm_dir: root.join(format!("pnpm-{pnpm_version}")),
            pnpm_version,
            python_dir: root.join(format!("python-{PYTHON}")),
            root,
        }
    }

    fn downloads(&self) -> PathBuf {
        self.home.join("downloads")
    }

    /// pnpm's package store. Every pnpm command gets it: `pnpm rebuild` refuses dependencies
    /// linked from another store than the one it would use.
    pub fn pnpm_store(&self) -> PathBuf {
        self.home.join("pnpm-store")
    }

    pub fn pnpm(&self) -> PathBuf {
        self.pnpm_dir.join("pnpm.exe")
    }

    pub fn python(&self) -> PathBuf {
        self.python_dir.join("python.exe")
    }

    fn cargo_bin(&self) -> PathBuf {
        self.cargo_home.join("bin")
    }

    pub fn cargo(&self) -> PathBuf {
        self.cargo_bin().join("cargo.exe")
    }

    /// A command that finds these tools first and is not steered by the user's own Rust or
    /// Node settings.
    pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let mut cmd = Command::new(program);
        let mut path = OsString::new();
        for dir in [
            &self.node,
            &self.pnpm_dir,
            &self.cargo_bin(),
            &self.python_dir,
        ] {
            path.push(dir.as_os_str());
            path.push(";");
        }
        if let Some(old) = std::env::var_os("PATH") {
            path.push(old);
        }
        cmd.env("PATH", path)
            .env("RUSTUP_HOME", &self.rustup_home)
            .env("CARGO_HOME", &self.cargo_home)
            .env("npm_config_store_dir", self.pnpm_store())
            .env("CI", "true")
            .env("TURBO_TELEMETRY_DISABLED", "1")
            .env("DO_NOT_TRACK", "1")
            .env("PYTHONUTF8", "1")
            .env("PYTHONUNBUFFERED", "1");
        for var in [
            "RUSTFLAGS",
            "CARGO_ENCODED_RUSTFLAGS",
            "CARGO_BUILD_TARGET",
            "CARGO_TARGET_DIR",
            "CARGO_BUILD_JOBS",
            "RUSTUP_TOOLCHAIN",
            "RUSTC",
            "RUSTC_WRAPPER",
            "NODE_OPTIONS",
        ] {
            cmd.env_remove(var);
        }
        cmd
    }

    /// Installs whatever is missing. True when everything was already there.
    pub fn ensure(&self, ui: &Ui) -> Result<bool, String> {
        let mut ready = msvc(ui, self, false)?;
        ready &= self.rust(ui)?;
        ready &= self.node(ui)?;
        ready &= self.pnpm_tool(ui)?;
        ready &= self.python_tool(ui)?;
        Ok(ready)
    }

    fn rust(&self, ui: &Ui) -> Result<bool, String> {
        let rustc = self.cargo_bin().join("rustc.exe");
        if rustc.is_file() {
            if let Ok(version) =
                proc::capture(ui, self.command(&rustc).arg("--version"))
            {
                ui.ok(&version);
                return Ok(true);
            }
            ui.warn("Rust установлен не до конца, ставлю заново");
        }
        let init = self.downloads().join("rustup-init.exe");
        let sums = net::fetch_text(&format!("{RUSTUP_INIT}.sha256"))?;
        let sha = sums
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
        if sha.len() != 64 {
            return Err(
                "rustup: не удалось получить контрольную сумму установщика"
                    .into(),
            );
        }
        net::download(
            ui,
            RUSTUP_INIT,
            &init,
            Some(&sha),
            "Rust (rustup-init)",
        )?;
        let mut install = self.command(&init);
        install.args([
            "-y",
            "--no-modify-path",
            "--profile",
            "minimal",
            "--default-toolchain",
            "stable",
            "--default-host",
            "x86_64-pc-windows-msvc",
        ]);
        proc::run(ui, &mut install, "Установка Rust")?;
        let version = proc::capture(ui, self.command(&rustc).arg("--version"))
            .map_err(|e| format!("Rust не запускается после установки: {e}"))?;
        let _ = fs::remove_file(&init);
        ui.ok(&version);
        Ok(false)
    }

    fn node(&self, ui: &Ui) -> Result<bool, String> {
        if self.node.join("node.exe").is_file() {
            ui.ok(&format!("Node.js {NODE}"));
            return Ok(true);
        }
        let name = format!("node-v{NODE}-win-x64");
        let zip = self.downloads().join(format!("{name}.zip"));
        let url = format!("https://nodejs.org/dist/v{NODE}/{name}.zip");
        net::download(ui, &url, &zip, Some(NODE_SHA256), "Node.js")?;
        unpack(ui, &zip, &self.root, Some(&name), &self.node, "Node.js")?;
        ui.ok(&format!("Node.js {NODE}"));
        Ok(false)
    }

    fn pnpm_tool(&self, ui: &Ui) -> Result<bool, String> {
        let exe = self.pnpm();
        let version = &self.pnpm_version;
        if exe.is_file() {
            ui.ok(&format!("pnpm {version}"));
            return Ok(true);
        }
        let url = format!("https://github.com/pnpm/pnpm/releases/download/v{version}/pnpm-win-x64.exe");
        let sha = (version == PNPM).then_some(PNPM_SHA256);
        if sha.is_none() {
            ui.warn(&format!("pnpm {version}: проверенной контрольной суммы нет, беру с GitHub по HTTPS"));
        }
        net::download(ui, &url, &exe, sha, "pnpm")?;
        ui.ok(&format!("pnpm {version}"));
        Ok(false)
    }

    fn python_tool(&self, ui: &Ui) -> Result<bool, String> {
        if self.python().is_file() {
            ui.ok(&format!("Python {PYTHON} (для сборки движка)"));
            return Ok(true);
        }
        let name = format!("python-{PYTHON}-embed-amd64.zip");
        let zip = self.downloads().join(&name);
        let url = format!("https://www.python.org/ftp/python/{PYTHON}/{name}");
        net::download(ui, &url, &zip, Some(PYTHON_SHA256), "Python")?;
        unpack(ui, &zip, &self.root, None, &self.python_dir, "Python")?;
        ui.ok(&format!("Python {PYTHON} (для сборки движка)"));
        Ok(false)
    }
}

/// Unpacks a downloaded archive into `dest` (its `inner` folder, when it has one), atomically.
fn unpack(
    ui: &Ui,
    zip: &Path,
    root: &Path,
    inner: Option<&str>,
    dest: &Path,
    label: &str,
) -> Result<(), String> {
    let staging = root.join(format!(
        ".unpack-{}",
        dest.file_name().unwrap_or_default().to_string_lossy()
    ));
    util::remove_tree(&staging)
        .map_err(|e| format!("{}: {e}", staging.display()))?;
    ui.status(&format!("      {label}: распаковка…"));
    source::extract_zip(zip, &staging)?;
    let unpacked = match inner {
        Some(name) => staging.join(name),
        None => staging.clone(),
    };
    if !unpacked.is_dir() {
        return Err(format!(
            "{label}: в архиве нет папки {}",
            inner.unwrap_or(".")
        ));
    }
    util::remove_tree(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    util::rename(&unpacked, dest)
        .map_err(|e| format!("{}: {e}", dest.display()))?;
    let _ = util::remove_tree(&staging);
    let _ = fs::remove_file(zip);
    ui.clear_status();
    Ok(())
}

fn program_files_x86() -> PathBuf {
    std::env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"))
}

/// The Visual Studio instance with the C++ compiler, linker included, if there is one.
fn find_msvc(ui: &Ui, products: &str) -> Option<String> {
    let vswhere = program_files_x86()
        .join(r"Microsoft Visual Studio\Installer\vswhere.exe");
    if !vswhere.is_file() {
        return None;
    }
    let mut cmd = Command::new(&vswhere);
    cmd.args([
        "-latest",
        "-products",
        products,
        "-requires",
        VC_TOOLS,
        "-property",
        "installationPath",
        "-utf8",
        "-nologo",
    ]);
    let found = proc::capture(ui, &mut cmd).ok()?;
    let path = found.lines().next()?.trim().to_string();
    let tools = Path::new(&path).join(r"VC\Tools\MSVC");
    let linker = fs::read_dir(&tools)
        .ok()?
        .flatten()
        .any(|v| v.path().join(r"bin\Hostx64\x64\link.exe").is_file());
    linker.then_some(path)
}

fn windows_sdk() -> bool {
    let libs = program_files_x86().join(r"Windows Kits\10\Lib");
    fs::read_dir(libs)
        .map(|versions| {
            versions
                .flatten()
                .any(|v| v.path().join(r"um\x64\kernel32.lib").is_file())
        })
        .unwrap_or(false)
}

/// Visual Studio Build Tools: the C++ compiler, linker and Windows SDK Rust needs on Windows.
/// True when they were already installed.
pub fn msvc(ui: &Ui, tools: &Tools, force: bool) -> Result<bool, String> {
    if !force {
        if let Some(found) = find_msvc(ui, "*") {
            if windows_sdk() {
                ui.ok(&format!("Visual Studio C++: {found}"));
                return Ok(true);
            }
            ui.warn("Visual Studio найден, но без Windows SDK");
        }
    }
    ui.info("Нужны Visual Studio Build Tools от Microsoft: компилятор C++ и Windows SDK");
    ui.info("(загрузка около 2 ГБ, на диске около 6 ГБ; ставятся один раз).");
    ui.info("Откроется окно установщика Microsoft. Если Windows спросит разрешение, нажми «Да».");
    if !ui.confirm("      Установить сейчас?", true) {
        return Err("без Visual Studio Build Tools лаунчер не собрать".into());
    }
    let exe = tools.downloads().join("vs_BuildTools.exe");
    net::download(ui, VS_BUILD_TOOLS, &exe, None, "Visual Studio Build Tools")?;
    microsoft_signed(ui, &exe)?;
    let args = [
        "--passive",
        "--wait",
        "--norestart",
        "--nocache",
        "--addProductLang",
        "En-us",
        "--add",
        "Microsoft.VisualStudio.Workload.VCTools",
        "--includeRecommended",
    ];
    let label = "Установка Visual Studio Build Tools (идёт в окне Microsoft)";
    let code = match proc::wait(ui, Command::new(&exe).args(args), label) {
        Err(e) if e.ends_with("нужны права администратора") => {
            elevated(ui, &exe, &args, label)?
        }
        other => other?,
    };
    match code {
        0 => {}
        3010 | 1641 => ui.warn("Visual Studio просит перезагрузку; сборка обычно работает и без неё."),
        1602 | 5004 | -1073741510 => {
            return Err("установку Visual Studio отменили. Запусти сборщик снова и разреши установку.".into())
        }
        740 => return Err("установщику Visual Studio нужны права администратора".into()),
        1001 | 1618 | 8006 => {
            return Err("сейчас работает другой установщик или Visual Studio. Закрой их и запусти сборщик снова.".into())
        }
        5007 | 8010 => return Err("этот компьютер не подходит для Visual Studio Build Tools (нужна Windows 10 или 11)".into()),
        other => return Err(format!("установщик Visual Studio завершился с кодом {other}")),
    }
    let products = if force {
        "Microsoft.VisualStudio.Product.BuildTools"
    } else {
        "*"
    };
    match (find_msvc(ui, products), windows_sdk()) {
        (Some(found), true) => {
            ui.ok(&format!("Visual Studio C++: {found}"));
            let _ = fs::remove_file(&exe);
            Ok(false)
        }
        _ => Err("Visual Studio Build Tools установлены, но компилятор C++ или Windows SDK не найден. Перезагрузи компьютер и запусти сборщик снова.".into()),
    }
}

/// Starts the installer through UAC when Windows refuses to start it unelevated.
fn elevated(
    ui: &Ui,
    exe: &Path,
    args: &[&str],
    label: &str,
) -> Result<i32, String> {
    let quote = |s: &str| s.replace('\'', "''");
    let list: Vec<String> =
        args.iter().map(|a| format!("'{}'", quote(a))).collect();
    let script = format!(
        "$p = Start-Process -FilePath '{}' -ArgumentList {} -Verb RunAs -Wait -PassThru; exit $p.ExitCode",
        quote(&exe.display().to_string()),
        list.join(",")
    );
    proc::wait(ui, &mut powershell(&script), label)
}

/// The bootstrapper is downloaded from aka.ms: accept it only with a valid Microsoft signature.
fn microsoft_signed(ui: &Ui, exe: &Path) -> Result<(), String> {
    let script = format!(
        "$s = Get-AuthenticodeSignature -LiteralPath '{}'; if ($s.Status -eq 'Valid' -and $s.SignerCertificate.Subject -match 'O=Microsoft Corporation') {{ exit 0 }}; Write-Output $s.Status; exit 1",
        exe.display().to_string().replace('\'', "''")
    );
    proc::capture(ui, &mut powershell(&script))
        .map(|_| ())
        .map_err(|e| format!("у загруженного установщика Visual Studio нет действительной подписи Microsoft ({e})"))
}

/// Windows PowerShell from System32 running `script`. Without the inherited PSModulePath: started
/// from PowerShell 7 it lists PowerShell 7's own modules first, which Windows PowerShell cannot load,
/// and then even Get-AuthenticodeSignature and Start-Process are missing.
fn powershell(script: &str) -> Command {
    let root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let mut cmd = Command::new(
        root.join(r"System32\WindowsPowerShell\v1.0\powershell.exe"),
    );
    cmd.env_remove("PSModulePath").args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_powershell_starts_without_the_inherited_module_path() {
        let cmd = powershell("exit 0");
        assert!(cmd
            .get_program()
            .to_string_lossy()
            .ends_with(r"System32\WindowsPowerShell\v1.0\powershell.exe"));
        assert!(cmd
            .get_envs()
            .any(|(k, v)| k == "PSModulePath" && v.is_none()));
        assert_eq!(cmd.get_args().last().unwrap(), "exit 0");
    }

    #[test]
    fn tools_live_in_versioned_folders_and_come_first_on_path() {
        let tools = Tools::new(Path::new("H"), None);
        assert!(
            tools.pnpm().ends_with("pnpm-9.4.0/pnpm.exe")
                || tools.pnpm().ends_with(r"pnpm-9.4.0\pnpm.exe")
        );
        assert!(tools.python().to_string_lossy().contains("python-3.13.17"));
        let cmd = tools.command("x");
        let path = cmd
            .get_envs()
            .find(|(k, _)| k.eq_ignore_ascii_case("PATH"))
            .and_then(|(_, v)| v)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(path.starts_with(&tools.node.display().to_string()));
        assert!(cmd.get_envs().any(|(k, v)| k == "npm_config_store_dir"
            && v == Some(tools.pnpm_store().as_os_str())));
        let removed: Vec<_> = cmd
            .get_envs()
            .filter(|(_, v)| v.is_none())
            .map(|(k, _)| k.to_owned())
            .collect();
        assert!(
            removed.iter().any(|k| k == "RUSTFLAGS")
                && removed.iter().any(|k| k == "CARGO_TARGET_DIR")
        );
        assert_eq!(
            Tools::new(Path::new("H"), Some("9.5.0")).pnpm_version,
            "9.5.0"
        );
    }
}
