//! Horizon's released CLI, not its GUI. No arbitrary executables, commands or game arguments.
//! Pinned official runtime, isolated launcher/game homes, upstream ownership checks preserved.
use crate::{assistant::redact, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    },
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

static CANCEL_SIGN_IN: AtomicBool = AtomicBool::new(false);
pub fn cancel_sign_in() -> Result<()> {
    let s = status()?;
    if s.running && s.action == "sign_in" {
        CANCEL_SIGN_IN.store(true, Ordering::SeqCst);
    }
    Ok(())
}
type Result<T> = std::result::Result<T, String>;
const RELEASE: &str = "launcher@0.2.28";
const ARCHIVE_URL: &str = "https://github.com/horizon-rust/horizon/releases/download/launcher%400.2.28/horizon-launcher-windows-x64.zip";
const ARCHIVE_HASH: &str =
    "6b7b362ad454a29b6e63b0444468b9511a680a5872356fabd976af7e660726b5";
const EXE_HASH: &str =
    "ef901348e259c39c83b8d246ecc92ccdeb4134aa07390777550799f2cfe63499";
const MAX_DOWNLOAD: u64 = 40 * 1024 * 1024;
const MAX_OUTPUT: usize = 2 * 1024 * 1024;
#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub supported: bool,
    pub runtime: bool,
    pub prepared: bool,
    pub release: String,
    pub running: bool,
    pub action: String,
    pub stage: String,
    pub completed: u64,
    pub total: u64,
    pub error: Option<String>,
    pub logs: Vec<String>,
    pub auth_url: Option<String>,
    pub auth_code: Option<String>,
    pub accounts: Value,
    pub instances: Value,
    pub versions: Value,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            supported: cfg!(all(windows, target_arch = "x86_64")),
            runtime: false,
            prepared: false,
            release: RELEASE.into(),
            running: false,
            action: String::new(),
            stage: "idle".into(),
            completed: 0,
            total: 0,
            error: None,
            logs: vec![],
            auth_url: None,
            auth_code: None,
            accounts: json!({}),
            instances: json!([]),
            versions: json!({}),
        }
    }
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub action: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub profile: String,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub minecraft: String,
}
fn state() -> &'static Mutex<Snapshot> {
    static S: OnceLock<Mutex<Snapshot>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Snapshot::default()))
}
fn change(f: impl FnOnce(&mut Snapshot)) {
    if let Ok(mut s) = state().lock() {
        f(&mut s);
    }
}
pub fn status() -> Result<Snapshot> {
    state().lock().map(|s| s.clone()).map_err(|e| e.to_string())
}
async fn home() -> Result<PathBuf> {
    Ok(State::get()
        .await
        .map_err(|e| e.to_string())?
        .directories
        .settings_dir
        .join("minecraft-rust"))
}
fn binary(root: &Path) -> PathBuf {
    root.join("runtime").join("horizon-launcher.exe")
}
fn validate(r: &Request) -> Result<()> {
    if !matches!(
        r.action.as_str(),
        "refresh"
            | "install"
            | "prepare_local"
            | "prepare_download"
            | "sign_in"
            | "create"
            | "play_online"
            | "play_offline"
    ) {
        return Err("Unsupported Minecraft Rust action".into());
    }
    if r.action == "create"
        && (r.name.trim().is_empty()
            || r.name.chars().count() > 48
            || r.name.chars().any(char::is_control))
    {
        return Err("Profile name must contain 1 to 48 characters".into());
    }
    if r.action == "play_offline" && !(3..=16).contains(&r.name.len()) {
        return Err(
            "Offline name must contain 3 to 16 letters, digits or _".into()
        );
    }
    if r.action == "play_offline"
        && !r
            .name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(
            "Offline name must contain 3 to 16 letters, digits or _".into()
        );
    }
    if r.minecraft.len() > 4096 || r.minecraft.chars().any(char::is_control) {
        return Err("Invalid Minecraft folder".into());
    }
    Ok(())
}
pub fn start(request: Request) -> Result<Snapshot> {
    validate(&request)?;
    if !cfg!(all(windows, target_arch = "x86_64")) {
        return Err(
            "Minecraft Rust is currently available on Windows x64".into()
        );
    }
    let mut s = state().lock().map_err(|e| e.to_string())?;
    if s.running {
        return Err("A Minecraft Rust operation is already running".into());
    }
    CANCEL_SIGN_IN.store(false, Ordering::SeqCst);
    s.running = true;
    s.action = request.action.clone();
    s.stage = request.action.clone();
    s.error = None;
    s.logs.clear();
    s.auth_url = None;
    s.auth_code = None;
    s.completed = 0;
    s.total = 0;
    let response = s.clone();
    drop(s);
    tokio::spawn(async move {
        let result = run(request).await;
        change(|s| {
            s.running = false;
            s.auth_code = None;
            s.auth_url = None;
            match result {
                Ok(()) => s.stage = "done".into(),
                Err(e) => {
                    s.stage = "error".into();
                    s.error = Some(redact(&e));
                }
            }
        });
    });
    Ok(response)
}
fn log_line(line: &str, auth: bool) {
    if line.trim().is_empty() {
        return;
    }
    // The released CLI prints the device-code instruction to stderr. Never collect tokens.
    if auth {
        if let Some((url, code)) = device_prompt(line) {
            change(|s| {
                s.auth_url = Some(url);
                s.auth_code = Some(code);
            });
            return;
        }
    }
    let clean = redact(line).replace(['\u{1b}', '\0'], "");
    change(|s| {
        s.logs.push(clean.chars().take(600).collect());
        if s.logs.len() > 120 {
            s.logs.remove(0);
        }
    });
}
fn device_prompt(line: &str) -> Option<(String, String)> {
    let re = regex::Regex::new(r"open (https://[^\s]+) in a browser and enter the code ([A-Z0-9-]{6,20})").ok()?;
    let c = re.captures(line)?;
    let url = url::Url::parse(&c[1]).ok()?;
    if url.scheme() != "https"
        || !matches!(
            url.host_str(),
            Some(
                "microsoft.com"
                    | "www.microsoft.com"
                    | "login.microsoftonline.com"
            )
        )
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    Some((url.to_string(), c[2].to_string()))
}
async fn read_pipe(
    mut pipe: impl AsyncRead + Unpin,
    capture: bool,
    auth: bool,
) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut pending = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = pipe.read(&mut buf).await.map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if capture {
            if output.len() + n > MAX_OUTPUT {
                return Err("Horizon output exceeded the size limit".into());
            }
            output.extend_from_slice(&buf[..n]);
        }
        for b in &buf[..n] {
            if *b == b'\n' || *b == b'\r' || pending.len() >= 4096 {
                log_line(&String::from_utf8_lossy(&pending), auth);
                pending.clear();
            }
            if *b != b'\n' && *b != b'\r' {
                pending.push(*b);
            }
        }
    }
    log_line(&String::from_utf8_lossy(&pending), auth);
    Ok(output)
}
async fn verified(root: &Path) -> Result<()> {
    if tokio::fs::metadata(binary(root))
        .await
        .map(|m| m.len() > 80 * 1024 * 1024)
        .unwrap_or(false)
    {
        return Err("Horizon runtime exceeds the size limit".into());
    }
    let bytes = tokio::fs::read(binary(root))
        .await
        .map_err(|_| "Install Minecraft Rust first".to_string())?;
    if format!("{:x}", Sha256::digest(&bytes)) != EXE_HASH {
        return Err(
            "Horizon runtime verification failed. Install it again".into()
        );
    }
    Ok(())
}
async fn cli(root: &Path, args: &[&str], capture: bool) -> Result<Vec<u8>> {
    verified(root).await?;
    let mut cmd = tokio::process::Command::new(binary(root));
    cmd.args(args)
        .env("HORIZON_LAUNCHER_HOME", root.join("launcher"))
        .env("HORIZON_HOME", root.join("game"))
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    // Fixed release uses its own built-in OAuth app. Never inherit endpoint / client-ID overrides.
    for key in [
        "HORIZON_MSA_CLIENT_ID",
        "HORIZON_MSA_MICROSOFT_ENDPOINT",
        "HORIZON_LAUNCHER_PAGE",
    ] {
        cmd.env_remove(key);
    }
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Could not start Horizon: {e}"))?;
    let auth = args == ["account", "sign-in"];
    let play = args.first() == Some(&"play");
    let result = async {
        let stdout = read_pipe(
            child.stdout.take().ok_or("No Horizon output")?,
            capture,
            auth,
        );
        let stderr = read_pipe(
            child.stderr.take().ok_or("No Horizon errors")?,
            false,
            auth,
        );
        let (out, _, status) = tokio::try_join!(stdout, stderr, async {
            child.wait().await.map_err(|e| e.to_string())
        })?;
        if !status.success() {
            return Err(format!(
                "Horizon exited with {status}. See the operation log"
            ));
        }
        Ok(out)
    };
    // Game lifetime is intentionally unbounded; never kill a running game for a UI timeout.
    if play {
        result.await
    } else {
        tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(if auth { 1200 } else { 3600 }), result) => {
                result.map_err(|_| "Horizon operation timed out. Check its log before retrying".to_string())?
            }
            _ = async {
                loop {
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    if auth && CANCEL_SIGN_IN.load(Ordering::SeqCst) { break; }
                }
            } => Err("Microsoft sign-in cancelled".into()),
        }
    }
}
async fn query(root: &Path, args: &[&str]) -> Result<Value> {
    serde_json::from_slice(&cli(root, args, true).await?).map_err(|_| {
        "Unexpected Horizon response. This runtime may be incompatible".into()
    })
}
async fn refresh(root: &Path) -> Result<()> {
    if !binary(root).exists() {
        change(|s| {
            s.runtime = false;
            s.prepared = false;
        });
        return Ok(());
    }
    let accounts = query(root, &["--json", "account", "list"]).await?;
    let instances = query(root, &["--json", "instance", "list"]).await?;
    let versions = query(root, &["--json", "versions", "--installed"]).await?;
    let prepared = prepared_at(root, &versions);
    change(|s| {
        s.runtime = true;
        s.prepared = prepared;
        s.accounts = accounts;
        s.instances = instances;
        s.versions = versions;
    });
    Ok(())
}
fn prepared_at(root: &Path, versions: &Value) -> bool {
    let safe = |s: &str| {
        !s.is_empty()
            && s != "."
            && s != ".."
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-@".contains(&b))
    };
    let tag = match versions["selected"].as_str().filter(|s| safe(s)) {
        Some(s) => s,
        None => return false,
    };
    let dir = root.join("launcher").join("versions").join(tag);
    let lock: Value = match std::fs::read(dir.join("research-lock.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
    {
        Some(v) => v,
        None => return false,
    };
    let game = match lock["minecraft"]["version"].as_str().filter(|s| safe(s)) {
        Some(s) => s,
        None => return false,
    };
    dir.join(".local/horizon-client")
        .join(game)
        .join("horizon-resources.json")
        .is_file()
}
fn selected_instance(instances: &Value, id: &str) -> Result<String> {
    instances
        .as_array()
        .and_then(|all| {
            all.iter()
                .find(|i| i["id"].as_str() == Some(id) && i["java"].is_null())
        })
        .and_then(|i| i["id"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| "Choose a Horizon profile".into())
}
async fn run(r: Request) -> Result<()> {
    let root = home().await?;
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|e| e.to_string())?;
    if r.action == "install" {
        change(|s| s.stage = "download_runtime".into());
        install_runtime(&root).await?;
        change(|s| {
            s.runtime = true;
            s.stage = "download_game".into();
            s.total = 0;
            s.completed = 0;
        });
        // The released backend checks game archive integrity; no Minecraft files are bundled.
        cli(&root, &["install", "latest"], false).await?;
    } else if r.action != "refresh" {
        verified(&root).await?;
        match r.action.as_str() {
            "sign_in" => {
                cli(&root, &["account", "sign-in"], false).await?;
            }
            "create" => {
                cli(&root, &["instance", "create", "--", r.name.trim()], false)
                    .await?;
            }
            "prepare_local" => {
                if !Path::new(&r.minecraft).is_absolute()
                    || !Path::new(&r.minecraft).is_dir()
                {
                    return Err(
                        "Choose an existing Minecraft game folder".into()
                    );
                }
                cli(&root, &["prepare", "--minecraft", &r.minecraft], false)
                    .await?;
            }
            "prepare_download" => {
                cli(&root, &["prepare", "--download"], false).await?;
            }
            "play_online" | "play_offline" => {
                let instances =
                    query(&root, &["--json", "instance", "list"]).await?;
                let id = selected_instance(&instances, &r.profile)?;
                let accounts =
                    query(&root, &["--json", "account", "list"]).await?;
                if r.action == "play_online" {
                    if !accounts["accounts"].as_array().map_or(false, |a| {
                        a.iter().any(|i| i["uuid"].as_str() == Some(&r.account))
                    }) {
                        return Err(
                            "Choose a licensed Microsoft account".into()
                        );
                    }
                    cli(&root, &["account", "select", "--", &r.account], false)
                        .await?;
                } else {
                    cli(&root, &["account", "offline", "--", &r.name], false)
                        .await?;
                }
                change(|s| s.stage = "playing".into());
                cli(&root, &["play", "--instance", &id], false).await?;
            }
            _ => return Err("Unsupported Minecraft Rust action".into()),
        }
    }
    refresh(&root).await
}
fn allowed_download_url(u: &url::Url) -> bool {
    u.scheme() == "https"
        && u.username().is_empty()
        && u.password().is_none()
        && u.port().is_none()
        && matches!(
            u.host_str(),
            Some(
                "github.com"
                    | "release-assets.githubusercontent.com"
                    | "objects.githubusercontent.com"
            )
        )
}
async fn install_runtime(root: &Path) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("U3drinth/1.0 (Horizon integration)")
        .timeout(Duration::from_secs(900))
        .redirect(reqwest::redirect::Policy::custom(|a| {
            if a.previous().len() >= 5 || !allowed_download_url(a.url()) {
                a.error("Untrusted Horizon download redirect")
            } else {
                a.follow()
            }
        }))
        .build()
        .map_err(|e| e.to_string())?;
    let mut response = client
        .get(ARCHIVE_URL)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let total = response.content_length().unwrap_or(0);
    if total > MAX_DOWNLOAD {
        return Err("Horizon archive exceeds the download limit".into());
    }
    change(|s| s.total = total);
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() as u64 + chunk.len() as u64 > MAX_DOWNLOAD {
            return Err("Horizon archive exceeds the download limit".into());
        }
        bytes.extend_from_slice(&chunk);
        change(|s| s.completed = bytes.len() as u64);
    }
    let executable = unpack_runtime(&bytes)?;
    let runtime = root.join("runtime");
    tokio::fs::create_dir_all(&runtime)
        .await
        .map_err(|e| e.to_string())?;
    // Verify first, then publish. No executable from a partially downloaded archive can run.
    let temp = tempfile::Builder::new()
        .prefix("horizon-")
        .tempfile_in(&runtime)
        .map_err(|e| e.to_string())?;
    let mut file =
        tokio::fs::File::from_std(temp.reopen().map_err(|e| e.to_string())?);
    file.write_all(&executable)
        .await
        .map_err(|e| e.to_string())?;
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);
    temp.persist(binary(root)).map_err(|e| e.to_string())?;
    Ok(())
}
fn unpack_runtime(bytes: &[u8]) -> Result<Vec<u8>> {
    if format!("{:x}", Sha256::digest(bytes)) != ARCHIVE_HASH {
        return Err("Horizon archive checksum mismatch".into());
    }
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| e.to_string())?;
    let mut member = zip
        .by_name("horizon-launcher.exe")
        .map_err(|_| "Horizon executable is missing".to_string())?;
    if member.size() > 80 * 1024 * 1024 {
        return Err("Horizon executable exceeds the size limit".into());
    }
    let mut exe = Vec::new();
    std::io::Read::read_to_end(&mut member, &mut exe)
        .map_err(|e| e.to_string())?;
    if !exe.starts_with(b"MZ")
        || format!("{:x}", Sha256::digest(&exe)) != EXE_HASH
    {
        return Err("Horizon executable checksum mismatch".into());
    }
    Ok(exe)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn request(action: &str, name: &str) -> Request {
        Request {
            action: action.into(),
            name: name.into(),
            profile: String::new(),
            account: String::new(),
            minecraft: String::new(),
        }
    }
    #[test]
    fn only_fixed_actions() {
        assert!(validate(&request("shell", "")).is_err());
        assert!(validate(&request("install", "")).is_ok());
    }
    #[test]
    fn offline_names() {
        for n in ["ab", "--help", "../world", "bad name", "Игрок"] {
            assert!(validate(&request("play_offline", n)).is_err());
        }
        assert!(validate(&request("play_offline", "U3_Player")).is_ok());
    }
    #[test]
    fn no_empty_profile() {
        assert!(validate(&request("create", "  ")).is_err());
        assert!(validate(&request("create", "Minecraft Rust")).is_ok());
    }
    #[test]
    fn only_microsoft_device_links() {
        assert!(device_prompt("To sign in, open https://www.microsoft.com/link in a browser and enter the code ABCD12345").is_some());
        assert!(device_prompt("open https://microsoft.com.evil.test in a browser and enter the code ABCD12345").is_none());
    }
    #[test]
    fn redirects_locked_down() {
        for u in [
            "http://github.com/a",
            "https://evil.test/a",
            "https://user@github.com/a",
        ] {
            assert!(!allowed_download_url(&url::Url::parse(u).unwrap()));
        }
    }
    #[test]
    fn reject_modified_archive() {
        assert!(unpack_runtime(b"MZ not a verified archive").is_err());
    }
    #[test]
    fn only_native_instances() {
        let v =
            json!([{"id":"horizon", "java":null}, {"id":"java", "java":{}}]);
        assert!(selected_instance(&v, "horizon").is_ok());
        assert!(selected_instance(&v, "java").is_err());
        assert!(selected_instance(&v, "../x").is_err());
    }
    #[tokio::test]
    async fn stream_capture_is_bounded_and_redacted() {
        let output = read_pipe(
            std::io::Cursor::new(b"access_token=secret123\nnext\n"),
            true,
            false,
        )
        .await
        .unwrap();
        assert!(output.starts_with(b"access_token="));
        assert!(!status().unwrap().logs.join("\n").contains("secret123"));
        assert!(read_pipe(
            std::io::Cursor::new(vec![b'x'; MAX_OUTPUT + 1]),
            true,
            false
        )
        .await
        .is_err());
    }
    #[test]
    fn preparation_marker_is_version_scoped() {
        let dir = tempfile::tempdir().unwrap();
        let version = dir.path().join("launcher/versions/test");
        std::fs::create_dir_all(version.join(".local/horizon-client/26.3"))
            .unwrap();
        std::fs::write(
            version.join("research-lock.json"),
            br#"{"minecraft":{"version":"26.3"}}"#,
        )
        .unwrap();
        let v = json!({"selected":"test"});
        assert!(!prepared_at(dir.path(), &v));
        std::fs::write(
            version.join(".local/horizon-client/26.3/horizon-resources.json"),
            b"{}",
        )
        .unwrap();
        assert!(prepared_at(dir.path(), &v));
        assert!(!prepared_at(dir.path(), &json!({"selected":"../test"})));
    }
    // Windows CI checks the actual pinned release, not a native API mock.
    #[cfg(windows)]
    #[tokio::test]
    async fn released_runtime_cli_contract() {
        let dir = tempfile::tempdir().unwrap();
        install_runtime(dir.path()).await.unwrap();
        let accounts = query(dir.path(), &["--json", "account", "list"])
            .await
            .unwrap();
        assert_eq!(accounts["owner"], false);
        assert!(accounts["sign_in_configured"].is_boolean());
        let instances = query(dir.path(), &["--json", "instance", "list"])
            .await
            .unwrap();
        assert_eq!(instances[0]["id"], "horizon");
        cli(
            dir.path(),
            &["instance", "create", "--", "U3 Rust Test"],
            false,
        )
        .await
        .unwrap();
        let instances = query(dir.path(), &["--json", "instance", "list"])
            .await
            .unwrap();
        assert_eq!(instances.as_array().unwrap().len(), 2);
        let versions =
            query(dir.path(), &["--json", "versions", "--installed"])
                .await
                .unwrap();
        assert_eq!(versions["installed"], json!([]));
        assert!(cli(
            dir.path(),
            &["account", "offline", "--", "U3_Test"],
            false
        )
        .await
        .is_err());
    }
}
