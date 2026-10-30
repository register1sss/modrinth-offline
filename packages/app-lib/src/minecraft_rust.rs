//! Horizon's released CLI, not its GUI. No arbitrary executables, commands or game arguments.
//! Pinned official runtime and isolated homes. Offline starts the prepared engine directly;
//! Microsoft authentication and ownership checks remain on licensed play and Mojang downloads.
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

#[cfg(test)]
#[path = "minecraft_rust/u3_edit/runtime.rs"]
mod editor_tests;
pub mod mods;
pub mod resourcepacks;

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
    pub offline_ready: bool,
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
    pub result_profile: Option<String>,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            supported: cfg!(all(windows, target_arch = "x86_64")),
            runtime: false,
            prepared: false,
            offline_ready: false,
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
            result_profile: None,
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
    #[serde(default)]
    pub project: String,
    #[serde(default)]
    pub checksum: String,
    #[serde(default)]
    pub trust_token: String,
    #[serde(default)]
    pub trust_native_code: bool,
    #[serde(default)]
    pub pack_file: String,
    #[serde(default = "resourcepacks::protect_default")]
    pub protect_gui: bool,
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
            | "mod_install"
            | "pack_install"
            | "pack_import"
            | "pack_toggle"
            | "pack_repair"
    ) {
        return Err("Unsupported Minecraft Rust action".into());
    }
    if r.action.starts_with("pack_") {
        resourcepacks::validate_request(r)?;
    }
    if r.action == "mod_install" {
        mods::validate_request(r)?;
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
    s.result_profile = None;
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
        "HORIZON_MOD_INDEX",
        "HORIZON_MOD_RATINGS",
    ] {
        cmd.env_remove(key);
    }
    if args.first() == Some(&"native-build") {
        cmd.env("CARGO_BUILD_JOBS", "2");
    }
    let auth = args == ["account", "sign-in"];
    let play = args.first() == Some(&"play");
    run_command(cmd, capture, auth, play).await
}
async fn run_command(
    mut cmd: tokio::process::Command,
    capture: bool,
    auth: bool,
    play: bool,
) -> Result<Vec<u8>> {
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Could not start Horizon: {e}"))?;
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
            result = tokio::time::timeout(Duration::from_secs(if auth { 1200 } else { 7200 }), result) => {
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
            s.offline_ready = false;
            s.instances = json!([]);
            s.versions = json!({});
            s.accounts = json!({});
        });
        return Ok(());
    }
    let accounts = query(root, &["--json", "account", "list"]).await?;
    let instances = query(root, &["--json", "instance", "list"]).await?;
    let versions = query(root, &["--json", "versions", "--installed"]).await?;
    let prepared = prepared_at(root, &versions);
    let offline_ready = versions["selected"]
        .as_str()
        .and_then(|tag| checked_engine(root, tag).ok())
        .is_some();
    change(|s| {
        s.runtime = true;
        s.prepared = prepared;
        s.offline_ready = offline_ready;
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
    tokio::fs::create_dir_all(root.join("game"))
        .await
        .map_err(|e| e.to_string())?;
    if r.action.starts_with("pack_") {
        resourcepacks::run(&root, &r).await?;
    } else if r.action == "mod_install" {
        mods::install(&root, &r).await?;
    } else if r.action == "install" {
        install_runtime_and_game(&root, "latest").await?;
    } else if matches!(r.action.as_str(), "play_online" | "play_offline") {
        // No path comes from the UI. Horizon searches this launcher's configured cache,
        // then the standard folders of supported launchers, checking the pinned game hashes.
        let source = State::get()
            .await
            .map_err(|e| e.to_string())?
            .directories
            .config_dir
            .clone();
        play_automatically(&root, &source, &r).await?;
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
            _ => return Err("Unsupported Minecraft Rust action".into()),
        }
    }
    refresh(&root).await
}
async fn install_runtime_and_game(root: &Path, tag: &str) -> Result<()> {
    change(|s| {
        s.stage = "download_runtime".into();
        s.total = 0;
        s.completed = 0;
    });
    if verified(root).await.is_err() {
        install_runtime(root).await?;
    }
    change(|s| {
        s.runtime = true;
        s.stage = "download_game".into();
        s.total = 0;
        s.completed = 0;
    });
    cli(root, &["install", tag], false).await?;
    let versions = query(root, &["--json", "versions", "--installed"]).await?;
    let installed = if tag == "latest" {
        versions["selected"]
            .as_str()
            .ok_or("Horizon did not select an installed version")?
    } else {
        tag
    };
    stamp_installed_engine(root, installed)
}
fn launch_version(
    instances: &Value,
    versions: &Value,
    id: &str,
) -> Result<Option<String>> {
    selected_instance(instances, id)?;
    let instance = instances
        .as_array()
        .and_then(|all| all.iter().find(|i| i["id"].as_str() == Some(id)))
        .ok_or("Choose a Horizon profile")?;
    let tag = instance["version"]
        .as_str()
        .or_else(|| versions["selected"].as_str());
    if tag.is_some_and(|tag| !safe_component(tag)) {
        return Err("Invalid Horizon version".into());
    }
    Ok(tag.map(str::to_owned))
}
fn needs_game_install(
    root: &Path,
    versions: &Value,
    tag: Option<&str>,
) -> bool {
    tag.map_or(true, |tag| {
        !versions["installed"]
            .as_array()
            .is_some_and(|all| all.iter().any(|v| v.as_str() == Some(tag)))
            || checked_engine(root, tag).is_err()
    })
}
async fn prepare_automatically(
    root: &Path,
    source: &Path,
    tag: &str,
) -> Result<()> {
    if prepared_at(root, &json!({"selected": tag})) {
        return Ok(());
    }
    change(|s| {
        s.stage = "prepare_auto".into();
        s.total = 0;
        s.completed = 0;
    });
    let source = source.to_str().ok_or("Invalid launcher data folder")?;
    // The verified manager's resolver also searches official/Prism/Modrinth/etc. folders.
    // Its normal ownership gate remains on downloads when no local files are found.
    if let Err(e) = cli(
        root,
        &["prepare", "--version", tag, "--minecraft", source],
        false,
    )
    .await
    {
        log_line(&e, false);
        return Err(
            "Automatic Minecraft preparation failed. See the operation log"
                .into(),
        );
    }
    if !prepared_at(root, &json!({"selected": tag})) {
        return Err(
            "Automatic Minecraft preparation failed. Resources are incomplete"
                .into(),
        );
    }
    Ok(())
}
async fn play_automatically(
    root: &Path,
    source: &Path,
    r: &Request,
) -> Result<()> {
    if verified(root).await.is_err() {
        change(|s| s.stage = "download_runtime".into());
        install_runtime(root).await?;
    }
    let instances = query(root, &["--json", "instance", "list"]).await?;
    let id = selected_instance(
        &instances,
        if r.profile.is_empty() {
            "horizon"
        } else {
            &r.profile
        },
    )?;
    if r.action == "play_online" {
        let accounts = query(root, &["--json", "account", "list"]).await?;
        if !accounts["owner"].as_bool().unwrap_or(false)
            || !accounts["accounts"].as_array().is_some_and(|all| {
                all.iter().any(|a| a["uuid"].as_str() == Some(&r.account))
            })
        {
            return Err("Choose a licensed Microsoft account".into());
        }
        cli(root, &["account", "select", "--", &r.account], false).await?;
    }
    let mut versions =
        query(root, &["--json", "versions", "--installed"]).await?;
    let tag = launch_version(&instances, &versions, &id)?;
    if tag.as_ref().is_some_and(|t| {
        t.starts_with("native-") && checked_engine(root, t).is_err()
    }) {
        return Err(
            "Native build is missing or changed. Reinstall a mod to rebuild it"
                .into(),
        );
    }
    if needs_game_install(root, &versions, tag.as_deref()) {
        install_runtime_and_game(root, tag.as_deref().unwrap_or("latest"))
            .await?;
        versions = query(root, &["--json", "versions", "--installed"]).await?;
    }
    let tag = launch_version(&instances, &versions, &id)?
        .ok_or("Install Horizon first")?;
    // Publish successful installation even if preparation later needs missing Minecraft files.
    refresh(root).await?;
    prepare_automatically(root, source, &tag).await?;
    refresh(root).await?;
    mods::ensure_fixes(root, &id).await?;
    // Native build can change the selected engine, not the selected game's home.
    // Reload BOTH lists before constructing the offline command or online launch.
    let instances = query(root, &["--json", "instance", "list"]).await?;
    let versions = query(root, &["--json", "versions", "--installed"]).await?;
    mods::check_build(root, &instances, &id)?;
    if r.action == "play_online" {
        change(|s| {
            s.stage = "playing".into();
            s.total = 0;
            s.completed = 0;
        });
        cli(root, &["play", "--instance", &id], false).await?;
    } else {
        let paths = offline_paths(root, &instances, &versions, &id)?;
        change(|s| {
            s.stage = "playing".into();
            s.total = 0;
            s.completed = 0;
        });
        run_command(offline_command(&paths, &r.name), false, false, true)
            .await?;
    }
    Ok(())
}
fn safe_component(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-@".contains(&b))
}
fn version_dir(root: &Path, tag: &str) -> Result<PathBuf> {
    if !safe_component(tag) {
        return Err("Invalid Horizon version".into());
    }
    let base = std::fs::canonicalize(root.join("launcher/versions"))
        .map_err(|_| "Install Horizon first".to_string())?;
    let dir = std::fs::canonicalize(base.join(tag))
        .map_err(|_| "The Horizon version is missing".to_string())?;
    if !dir.starts_with(&base)
        || dir == base
        || !dir.join(".installed").is_file()
    {
        return Err("Invalid Horizon version folder".into());
    }
    Ok(dir)
}
fn engine_hash(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 256 * 1024 * 1024 {
        return Err("Horizon engine exceeds the size limit".into());
    }
    let mut hash = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn engine_in(dir: &Path) -> Result<PathBuf> {
    let exe = std::fs::canonicalize(dir.join("horizon-client.exe")).map_err(
        |_| "Horizon engine is missing. Update the game".to_string(),
    )?;
    if exe.parent() != Some(dir) {
        return Err("Horizon engine is outside its version folder".into());
    }
    Ok(exe)
}
fn stamp_installed_engine(root: &Path, tag: &str) -> Result<()> {
    let dir = version_dir(root, tag)?;
    let hash = engine_hash(&engine_in(&dir)?)?;
    let temp = tempfile::Builder::new()
        .prefix(".u3-engine-")
        .tempfile_in(&dir)
        .map_err(|e| e.to_string())?;
    std::fs::write(temp.path(), hash).map_err(|e| e.to_string())?;
    temp.persist(dir.join(".u3-engine-sha256"))
        .map_err(|e| e.to_string())?;
    Ok(())
}
fn checked_engine(root: &Path, tag: &str) -> Result<PathBuf> {
    let dir = version_dir(root, tag)?;
    let exe = engine_in(&dir)?;
    let stamp = std::fs::read_to_string(dir.join(".u3-engine-sha256"))
        .map_err(|_| {
            "Update the game once to enable verified offline launch".to_string()
        })?;
    if stamp.len() != 64 || engine_hash(&exe)? != stamp {
        return Err(
            "Horizon engine changed. Update the game before offline launch"
                .into(),
        );
    }
    Ok(exe)
}
#[derive(Debug)]
struct OfflinePaths {
    program: PathBuf,
    resources: PathBuf,
    home: PathBuf,
}
fn offline_paths(
    root: &Path,
    instances: &Value,
    versions: &Value,
    id: &str,
) -> Result<OfflinePaths> {
    selected_instance(instances, id)?;
    let instance = instances
        .as_array()
        .and_then(|v| v.iter().find(|i| i["id"].as_str() == Some(id)))
        .ok_or("Choose a Horizon profile")?;
    if !matches!(instance["kind"].as_str(), Some("default" | "own")) {
        return Err(
            "Offline launch supports local Horizon profiles only".into()
        );
    }
    let tag = instance["version"]
        .as_str()
        .or_else(|| versions["selected"].as_str())
        .ok_or("Choose an installed Horizon version")?;
    if !versions["installed"]
        .as_array()
        .map_or(false, |v| v.iter().any(|v| v.as_str() == Some(tag)))
    {
        return Err("The profile's Horizon version is not installed".into());
    }
    let program = checked_engine(root, tag)?;
    let dir = version_dir(root, tag)?;
    if !prepared_at(root, &json!({"selected":tag})) {
        return Err("Prepare Minecraft files first".into());
    }
    let lock: Value = serde_json::from_slice(
        &std::fs::read(dir.join("research-lock.json"))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let game = lock["minecraft"]["version"]
        .as_str()
        .filter(|s| safe_component(s))
        .ok_or("Invalid Minecraft resource version")?;
    let resources =
        std::fs::canonicalize(dir.join(".local/horizon-client").join(game))
            .map_err(|e| e.to_string())?;
    if !resources.starts_with(&dir) {
        return Err("Prepared resources are outside the version folder".into());
    }
    let base =
        std::fs::canonicalize(root.join("game")).map_err(|e| e.to_string())?;
    let home = std::fs::canonicalize(
        instance["home"]
            .as_str()
            .ok_or("Profile folder is missing")?,
    )
    .map_err(|e| e.to_string())?;
    if !home.starts_with(&base) {
        return Err("Profile folder is outside Minecraft Rust".into());
    }
    Ok(OfflinePaths {
        program,
        resources,
        home,
    })
}
fn offline_command(
    paths: &OfflinePaths,
    nickname: &str,
) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(&paths.program);
    cmd.arg("--resources")
        .arg(&paths.resources)
        .arg("--name")
        .arg(nickname)
        .current_dir(&paths.home)
        .env("HORIZON_HOME", &paths.home)
        .env_remove("HORIZON_SESSION_FILE")
        .env_remove("HORIZON_PLAYER_NAME");
    cmd
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
            project: String::new(),
            checksum: String::new(),
            trust_token: String::new(),
            trust_native_code: false,
            pack_file: String::new(),
            protect_gui: true,
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

    fn offline_fixture() -> (tempfile::TempDir, Value, Value) {
        let dir = tempfile::tempdir().unwrap();
        let version = dir.path().join("launcher/versions/test");
        std::fs::create_dir_all(version.join(".local/horizon-client/26.3"))
            .unwrap();
        std::fs::create_dir_all(dir.path().join("game/instances/my-rust"))
            .unwrap();
        std::fs::write(version.join(".installed"), b"").unwrap();
        std::fs::write(
            version.join("horizon-client.exe"),
            b"MZ fixture engine",
        )
        .unwrap();
        std::fs::write(
            version.join("research-lock.json"),
            br#"{"minecraft":{"version":"26.3"}}"#,
        )
        .unwrap();
        std::fs::write(
            version.join(".local/horizon-client/26.3/horizon-resources.json"),
            b"{}",
        )
        .unwrap();
        stamp_installed_engine(dir.path(), "test").unwrap();
        let instances = json!([{"id":"horizon", "kind":"default", "java":null,"home":dir.path().join("game")}, {"id":"my-rust","kind":"own","java":null,"home":dir.path().join("game/instances/my-rust")}]);
        (
            dir,
            instances,
            json!({"selected":"test","installed":["test"]}),
        )
    }
    #[test]
    fn offline_command_does_not_need_an_account_or_session() {
        let (dir, instances, versions) = offline_fixture();
        let paths = offline_paths(dir.path(), &instances, &versions, "my-rust")
            .unwrap();
        let cmd = offline_command(&paths, "U3_Player");
        let command = cmd.as_std();
        let args: Vec<_> = command
            .get_args()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
        assert_eq!(
            args,
            vec![
                "--resources".to_string(),
                paths.resources.to_string_lossy().to_string(),
                "--name".to_string(),
                "U3_Player".to_string()
            ]
        );
        assert_eq!(command.get_current_dir(), Some(paths.home.as_path()));
        assert!(command
            .get_envs()
            .any(|(k, v)| k == "HORIZON_SESSION_FILE" && v.is_none()));
        assert!(paths.home.ends_with("my-rust"));
        assert!(!dir.path().join("launcher/accounts.json").exists());
    }
    #[test]
    fn refreshed_native_version_keeps_original_home_for_offline_play() {
        let (dir, mut instances, _) = offline_fixture();
        std::fs::rename(
            dir.path().join("launcher/versions/test"),
            dir.path().join("launcher/versions/native-fixed"),
        )
        .unwrap();
        instances[0]["version"] = json!("native-fixed");
        let versions =
            json!({"selected":"test","installed":["test","native-fixed"]});
        let paths = offline_paths(dir.path(), &instances, &versions, "horizon")
            .unwrap();
        assert!(paths.program.ends_with("native-fixed/horizon-client.exe"));
        assert_eq!(
            paths.home,
            std::fs::canonicalize(dir.path().join("game")).unwrap()
        );
        assert!(!paths.home.ends_with("native-mods"));
    }
    #[test]
    fn offline_engine_checks_integrity_and_preparation() {
        let (dir, instances, versions) = offline_fixture();
        let version = dir.path().join("launcher/versions/test");
        std::fs::write(
            version.join("horizon-client.exe"),
            b"MZ modified engine",
        )
        .unwrap();
        assert!(offline_paths(dir.path(), &instances, &versions, "horizon")
            .is_err());
        stamp_installed_engine(dir.path(), "test").unwrap();
        std::fs::remove_file(
            version.join(".local/horizon-client/26.3/horizon-resources.json"),
        )
        .unwrap();
        assert!(offline_paths(dir.path(), &instances, &versions, "horizon")
            .is_err());
    }
    #[test]
    fn offline_profiles_cannot_escape_game_root_or_pick_arbitrary_engines() {
        let (dir, mut instances, versions) = offline_fixture();
        instances[0]["home"] = json!(dir.path());
        assert!(offline_paths(dir.path(), &instances, &versions, "horizon")
            .is_err());
        assert!(offline_paths(
            dir.path(),
            &instances,
            &json!({"selected":"../test","installed":["../test"]}),
            "my-rust"
        )
        .is_err());
        instances[1]["java"] = json!({});
        assert!(offline_paths(dir.path(), &instances, &versions, "my-rust")
            .is_err());
    }
    #[cfg(unix)]
    #[test]
    fn offline_rejects_symlinks_outside_version_root() {
        let (dir, instances, versions) = offline_fixture();
        let version = dir.path().join("launcher/versions/test");
        let other = dir.path().join("outside.exe");
        std::fs::write(&other, b"MZ fixture engine").unwrap();
        std::fs::remove_file(version.join("horizon-client.exe")).unwrap();
        std::os::unix::fs::symlink(other, version.join("horizon-client.exe"))
            .unwrap();
        assert!(offline_paths(dir.path(), &instances, &versions, "horizon")
            .is_err());
    }
    #[test]
    fn automatic_launch_reuses_verified_install_and_prepared_files() {
        let (dir, instances, versions) = offline_fixture();
        let tag = launch_version(&instances, &versions, "horizon").unwrap();
        assert_eq!(tag.as_deref(), Some("test"));
        assert!(!needs_game_install(dir.path(), &versions, tag.as_deref()));
        assert!(prepared_at(dir.path(), &json!({"selected": tag})));
        std::fs::remove_file(
            dir.path().join("launcher/versions/test/.u3-engine-sha256"),
        )
        .unwrap();
        assert!(needs_game_install(dir.path(), &versions, tag.as_deref()));
    }
    #[test]
    fn automatic_launch_installs_missing_game_and_honors_profile_pin() {
        let (dir, mut instances, versions) = offline_fixture();
        assert!(needs_game_install(dir.path(), &json!({}), None));
        assert_eq!(
            launch_version(&instances, &json!({}), "horizon").unwrap(),
            None
        );
        instances[0]["version"] = json!("pinned");
        let tag = launch_version(&instances, &versions, "horizon").unwrap();
        assert_eq!(tag.as_deref(), Some("pinned"));
        assert!(needs_game_install(dir.path(), &versions, tag.as_deref()));
        instances[0]["version"] = json!("../escape");
        assert!(launch_version(&instances, &versions, "horizon").is_err());
        assert!(launch_version(&instances, &versions, "--help").is_err());
    }
    #[tokio::test]
    async fn automatic_preparation_is_skipped_when_already_prepared() {
        let (dir, _, _) = offline_fixture();
        // No manager exe or source folder exists: a second play must not prepare or download.
        prepare_automatically(
            dir.path(),
            &dir.path().join("absent-source"),
            "test",
        )
        .await
        .unwrap();
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

        // Install a real signed Horizon build, then exercise its native offline CLI without
        // Microsoft accounts, game resources or a graphics context. --help exits before rendering.
        cli(dir.path(), &["install", "latest"], false)
            .await
            .unwrap();
        let versions =
            query(dir.path(), &["--json", "versions", "--installed"])
                .await
                .unwrap();
        let tag = versions["selected"].as_str().unwrap();
        stamp_installed_engine(dir.path(), tag).unwrap();
        let program = checked_engine(dir.path(), tag).unwrap();
        let home = dir.path().join("game");
        std::fs::create_dir_all(&home).unwrap();
        let paths = OfflinePaths {
            program,
            resources: dir.path().join("unprepared-resources"),
            home,
        };
        let mut cmd = offline_command(&paths, "U3_NoAccount");
        cmd.arg("--help");
        let help = String::from_utf8_lossy(
            &run_command(cmd, true, false, false).await.unwrap(),
        )
        .to_string();
        assert!(help.contains("--resources"));
        assert!(help.contains("--name"));
        let accounts = query(dir.path(), &["--json", "account", "list"])
            .await
            .unwrap();
        assert_eq!(accounts["owner"], false);
    }
}
