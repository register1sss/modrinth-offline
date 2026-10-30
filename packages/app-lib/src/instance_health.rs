//! Local, bounded diagnostics. Never executes a mod or uploads its contents.
use crate::{process, profile, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::Read,
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime},
};
pub type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
const MAX_FILE: u64 = 256 * 1024 * 1024;
fn link(m: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        m.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        m.file_type().is_symlink()
    }
}
async fn root(id: &str) -> Result<PathBuf> {
    if id.is_empty()
        || Path::new(id)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("Unsafe profile identifier".into());
    }
    if profile::get(id).await.map_err(err)?.is_none() {
        return Err("Unknown profile".into());
    }
    let state = State::get().await.map_err(err)?;
    let base =
        std::fs::canonicalize(state.directories.profiles_dir()).map_err(err)?;
    let mut p = base.clone();
    for c in Path::new(id).components() {
        p.push(c.as_os_str());
        if link(&std::fs::symlink_metadata(&p).map_err(err)?) {
            return Err("Profile links are not allowed".into());
        }
    }
    let p = std::fs::canonicalize(p).map_err(err)?;
    if p == base || !p.starts_with(base) {
        return Err("Profile escapes launcher directory".into());
    }
    Ok(p)
}
fn child(base: &Path, rel: &str) -> Result<PathBuf> {
    if rel.is_empty()
        || rel.contains('\\')
        || Path::new(rel)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("Unsafe relative path".into());
    }
    let mut p = base.to_path_buf();
    for c in Path::new(rel).components() {
        p.push(c.as_os_str());
        if link(&std::fs::symlink_metadata(&p).map_err(err)?) {
            return Err("Links are not permitted".into());
        }
    }
    let p = std::fs::canonicalize(p).map_err(err)?;
    if !p.starts_with(base) {
        return Err("Path escapes profile".into());
    }
    Ok(p)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    pub required: bool,
    pub version: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub path: String,
    pub name: String,
    pub ids: Vec<String>,
    pub dependencies: Vec<Dependency>,
    pub disabled: bool,
    pub sha256: String,
    pub bytes: u64,
    pub warnings: Vec<String>,
    pub metadata_complete: bool,
}
fn dep(id: &str, required: bool, version: Value) -> Dependency {
    Dependency {
        id: id.into(),
        required,
        version,
    }
}
fn metadata(
    z: &mut zip::ZipArchive<std::fs::File>,
    name: &str,
) -> Result<String> {
    let mut f = z.by_name(name).map_err(err)?;
    if f.size() > 1024 * 1024 {
        return Err("Mod metadata exceeds 1 MiB limit".into());
    }
    let mut text = String::new();
    f.by_ref()
        .take(1024 * 1024 + 1)
        .read_to_string(&mut text)
        .map_err(err)?;
    if text.len() > 1024 * 1024 {
        return Err("Mod metadata exceeds limit".into());
    }
    Ok(text)
}
fn parse_json(v: &Value, quilt: bool, out: &mut ModInfo) {
    let m = if quilt { &v["quilt_loader"] } else { v };
    if let Some(id) = m["id"].as_str() {
        out.ids.push(id.into());
    }
    if let Some(name) = m["name"]
        .as_str()
        .or_else(|| m["metadata"]["name"].as_str())
    {
        out.name = name.into();
    }
    if let Some(provides) = m["provides"].as_array() {
        for p in provides {
            if let Some(id) = p.as_str().or_else(|| p["id"].as_str()) {
                out.ids.push(id.into());
            }
        }
    }
    if quilt {
        if let Some(deps) = m["depends"].as_array() {
            for d in deps {
                if let Some(id) = d.as_str().or_else(|| d["id"].as_str()) {
                    out.dependencies.push(dep(
                        id,
                        !d["optional"].as_bool().unwrap_or(false),
                        d["versions"].clone(),
                    ));
                } else {
                    out.warnings.push("Quilt dependency alternatives cannot be resolved reliably".into());
                }
            }
        }
    } else {
        for (key, required) in [
            ("depends", true),
            ("recommends", false),
            ("suggests", false),
        ] {
            if let Some(deps) = m[key].as_object() {
                for (id, version) in deps {
                    out.dependencies.push(dep(id, required, version.clone()));
                }
            }
        }
    }
    out.metadata_complete = !out.ids.is_empty() && out.warnings.is_empty();
    if v["jars"].as_array().map_or(false, |x| !x.is_empty())
        || m["jars"].as_array().map_or(false, |x| !x.is_empty())
    {
        out.metadata_complete = false;
        out.warnings
            .push("Nested JAR dependencies require loader validation".into());
    }
}
fn parse_toml(text: &str, out: &mut ModInfo) -> Result<()> {
    let v: toml::Value = text.parse().map_err(err)?;
    if let Some(mods) = v.get("mods").and_then(|x| x.as_array()) {
        for m in mods {
            if let Some(id) = m.get("modId").and_then(|x| x.as_str()) {
                out.ids.push(id.into());
            }
            if let Some(name) = m.get("displayName").and_then(|x| x.as_str()) {
                out.name = name.into();
            }
        }
    }
    if let Some(groups) = v.get("dependencies").and_then(|x| x.as_table()) {
        for (_, deps) in groups {
            if let Some(deps) = deps.as_array() {
                for d in deps {
                    if let Some(id) = d.get("modId").and_then(|x| x.as_str()) {
                        if matches!(
                            d.get("type").and_then(|x| x.as_str()),
                            Some("incompatible" | "discouraged")
                        ) {
                            continue;
                        }
                        let required = d
                            .get("mandatory")
                            .and_then(|x| x.as_bool())
                            .unwrap_or_else(|| {
                                d.get("type")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("required")
                                    == "required"
                            });
                        // Server-only dependencies do not prevent a client from starting.
                        if d.get("side").and_then(|x| x.as_str())
                            != Some("SERVER")
                        {
                            out.dependencies.push(dep(
                                id,
                                required,
                                json!(d
                                    .get("versionRange")
                                    .and_then(|x| x.as_str())),
                            ));
                        }
                    }
                }
            }
        }
    }
    out.metadata_complete = !out.ids.is_empty();
    Ok(())
}
fn inspect(path: &Path, rel: String) -> Result<ModInfo> {
    let bytes = path.metadata().map_err(err)?.len();
    let mut out = ModInfo {
        name: rel.clone(),
        disabled: rel.ends_with(".disabled"),
        path: rel,
        ids: vec![],
        dependencies: vec![],
        sha256: String::new(),
        bytes,
        warnings: vec![],
        metadata_complete: false,
    };
    if bytes > MAX_FILE {
        out.warnings
            .push("File exceeds 256 MiB inspection limit; not checked".into());
        return Ok(out);
    }
    let mut f = std::fs::File::open(path).map_err(err)?.take(MAX_FILE + 1);
    let mut consumed = 0u64;
    let mut hash = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf).map_err(err)?;
        if n == 0 {
            break;
        }
        consumed += n as u64;
        if consumed > MAX_FILE {
            out.warnings.push(
                "File changed or grew past inspection limit; not fully checked"
                    .into(),
            );
            return Ok(out);
        }
        hash.update(&buf[..n]);
    }
    out.sha256 = format!("{:x}", hash.finalize());
    let mut z =
        match zip::ZipArchive::new(std::fs::File::open(path).map_err(err)?) {
            Ok(x) => x,
            Err(_) => {
                out.warnings.push(
                    "Not a readable JAR/ZIP; format was not inspected".into(),
                );
                return Ok(out);
            }
        };
    if z.len() > 20000 {
        out.warnings
            .push("Archive has too many entries; not inspected".into());
        return Ok(out);
    }
    let names: BTreeSet<String> = z.file_names().map(str::to_owned).collect();
    let result = if names.contains("fabric.mod.json") {
        metadata(&mut z, "fabric.mod.json").and_then(|s| {
            let v = serde_json::from_str(&s).map_err(err)?;
            parse_json(&v, false, &mut out);
            Ok(())
        })
    } else if names.contains("quilt.mod.json") {
        metadata(&mut z, "quilt.mod.json").and_then(|s| {
            let v = serde_json::from_str(&s).map_err(err)?;
            parse_json(&v, true, &mut out);
            Ok(())
        })
    } else if let Some(n) =
        ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"]
            .iter()
            .find(|n| names.contains(**n))
    {
        metadata(&mut z, n).and_then(|s| parse_toml(&s, &mut out))
    } else {
        Err("No supported Fabric/Quilt/Forge/NeoForge manifest".into())
    };
    if let Err(e) = result {
        out.warnings.push(e);
    }
    let mut inspected = 0u64;
    for i in 0..z.len() {
        let mut f = match z.by_index(i) {
            Ok(f) => f,
            Err(_) => {
                out.warnings
                    .push("Unreadable/encrypted archive entry".into());
                break;
            }
        };
        let name = f.name().to_owned();
        let lower = name.to_ascii_lowercase();
        if name.starts_with('/')
            || name.contains('\\')
            || Path::new(&name).components().any(|c| {
                matches!(c, Component::ParentDir | Component::Prefix(_))
            })
        {
            out.warnings.push(format!(
                "Unsafe archive path: {}",
                name.chars().take(160).collect::<String>()
            ));
        }
        if [".exe", ".ps1", ".bat", ".cmd", ".vbs"]
            .iter()
            .any(|ext| lower.ends_with(ext))
        {
            out.warnings.push(format!(
                "Contains executable/script: {} (not proof of malware)",
                name.chars().take(160).collect::<String>()
            ));
        }
        if !lower.ends_with(".class") {
            continue;
        }
        if f.size() > 2 * 1024 * 1024 || inspected + f.size() > 32 * 1024 * 1024
        {
            out.warnings.push(
                "Bytecode inspection limit reached; scan is partial".into(),
            );
            break;
        }
        let mut data = vec![];
        f.by_ref()
            .take(2 * 1024 * 1024 + 1)
            .read_to_end(&mut data)
            .map_err(err)?;
        inspected += data.len() as u64;
        let s = String::from_utf8_lossy(&data).to_ascii_lowercase();
        if (s.contains("java/lang/runtime")
            || s.contains("java/lang/processbuilder"))
            && (s.contains("powershell")
                || s.contains("cmd.exe")
                || s.contains("/bin/sh"))
        {
            out.warnings.push(format!(
                "Starts a command shell: {} (review recommended)",
                name.chars().take(160).collect::<String>()
            ));
        }
        if s.contains("discord") && s.contains("local storage/leveldb") {
            out.warnings.push(format!(
                "References token storage: {} (review recommended)",
                name.chars().take(160).collect::<String>()
            ));
        }
        if out.warnings.len() > 100 {
            out.warnings.push("Additional findings omitted".into());
            break;
        }
    }
    out.ids.sort();
    out.ids.dedup();
    Ok(out)
}
pub async fn inventory(id: &str) -> Result<Vec<ModInfo>> {
    let base = root(id).await?;
    tokio::task::spawn_blocking(move || {
        if !base.join("mods").exists() {
            return Ok(vec![]);
        }
        let folder = child(&base, "mods")?;
        let mut result = vec![];
        for e in std::fs::read_dir(&folder).map_err(err)? {
            let e = e.map_err(err)?;
            let name = e.file_name().to_string_lossy().into_owned();
            if e.file_type().map_err(err)?.is_dir() {
                continue;
            }
            if result.len() >= 1024 {
                return Err(
                    "More than 1024 mod files; inspection stopped".into()
                );
            }
            let rel = format!("mods/{}", name);
            let item =
                child(&base, &rel).and_then(|p| inspect(&p, rel.clone()));
            result.push(item.unwrap_or_else(|e| ModInfo {
                path: rel,
                name,
                ids: vec![],
                dependencies: vec![],
                disabled: false,
                sha256: String::new(),
                bytes: 0,
                warnings: vec![e],
                metadata_complete: false,
            }));
        }
        result.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(result)
    })
    .await
    .map_err(err)?
}
/// Only fresh, strong crash signatures, never a generic ERROR line.
pub async fn crash_report(id: &str, since_ms: u64) -> Result<Value> {
    let base = root(id).await?;
    let p = match child(&base, "logs/latest.log") {
        Ok(p) => p,
        Err(_) => {
            return Ok(json!({"crashed": false, "reason": "No log available"}))
        }
    };
    let mut f = std::fs::File::open(p).map_err(err)?;
    let m = f.metadata().map_err(err)?;
    let modified = m
        .modified()
        .map_err(err)?
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(err)?
        .as_millis() as u64;
    if modified < since_ms {
        return Ok(json!({"crashed": false, "reason": "Stale log ignored"}));
    }
    use std::io::{Seek, SeekFrom};
    f.seek(SeekFrom::Start(m.len().saturating_sub(512 * 1024)))
        .map_err(err)?;
    let mut data = vec![];
    f.take(512 * 1024).read_to_end(&mut data).map_err(err)?;
    let s = String::from_utf8_lossy(&data);
    let mut findings = vec![];
    for (pattern, kind) in [
        ("OutOfMemoryError", "memory"),
        ("UnsupportedClassVersionError", "java"),
        ("ModResolutionException", "dependencies"),
        ("Incompatible mods found", "dependencies"),
        ("MixinApplyError", "mixin"),
        ("Crash report saved to", "crash"),
        ("Reported exception thrown!", "crash"),
        ("Failed to download file", "files"),
        ("ZipException", "files"),
    ] {
        if s.contains(pattern) {
            findings.push(kind);
        }
    }
    findings.sort();
    findings.dedup();
    Ok(
        json!({"crashed": !findings.is_empty(), "findings": findings, "modified_ms": modified, "heuristic": true}),
    )
}
async fn stopped(id: &str) -> Result<()> {
    if !process::get_by_profile_path(id)
        .await
        .map_err(err)?
        .is_empty()
    {
        return Err("Stop the instance before changing files".into());
    }
    Ok(())
}
/// Reinstalls only launcher-managed game/loader files. Does not change mods, configs or worlds.
pub async fn repair(id: &str) -> Result<Value> {
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    let _guard = LOCK
        .try_lock()
        .map_err(|_| "Another repair is already running".to_string())?;
    stopped(id).await?;
    let base = root(id).await?;
    let backup_parent = base.join("u3-repair-backups");
    if backup_parent.exists() {
        child(&base, "u3-repair-backups")?;
    } else {
        std::fs::create_dir(&backup_parent).map_err(err)?;
    }
    let name = uuid::Uuid::new_v4().to_string();
    let backup = backup_parent.join(&name);
    std::fs::create_dir(&backup).map_err(err)?;
    let saved = profile::get(id)
        .await
        .map_err(err)?
        .ok_or("Profile not found")?;
    std::fs::write(
        backup.join("profile.json"),
        serde_json::to_vec_pretty(&saved).map_err(err)?,
    )
    .map_err(err)?;
    // Base-file repair never edits config or saves; keep a copy of options for troubleshooting.
    for rel in ["options.txt", "optionsof.txt"] {
        if base.join(rel).exists() {
            let src = child(&base, rel)?;
            if src.metadata().map_err(err)?.len() > 1024 * 1024 {
                return Err("Options file exceeds backup limit".into());
            }
            std::fs::copy(src, backup.join(rel)).map_err(err)?;
        }
    }
    std::fs::write(backup.join("README.txt"), "Profile metadata and options before base-game repair. Worlds, mods and config are not modified by this repair. Shared downloaded game files are redownloaded, not backed up.\n").map_err(err)?;
    if let Err(e) = profile::install(id, true).await {
        let stage = saved.install_stage;
        let rollback = profile::edit(id, |p| {
            p.install_stage = stage;
            async { Ok(()) }
        })
        .await;
        return Err(format!("Repair failed; backup retained in u3-repair-backups/{}: {}. Restore install state: {:?}", name, e, rollback.err()));
    }
    Ok(
        json!({"backup": format!("u3-repair-backups/{}", name), "message": "Base game files restored. Dependency, memory and driver issues may still need manual changes."}),
    )
}
pub async fn resources(id: &str) -> Result<Value> {
    root(id).await?;
    let cached = process::get_by_profile_path(id).await.map_err(err)?;
    tokio::task::spawn_blocking(move || {
        let mut s = sysinfo::System::new(); s.refresh_processes(); std::thread::sleep(Duration::from_millis(800)); s.refresh_processes();
        let mut rows = vec![];
        for p in cached { if let Some(actual) = s.process(sysinfo::Pid::from_u32(p.pid as u32)) {
            if actual.start_time() as i64 != p.start_time || actual.name() != p.name || actual.exe().map(|x| x.to_string_lossy().to_string()).unwrap_or_default() != p.executable { continue; }
            rows.push(json!({"pid": p.pid, "memory_bytes": actual.memory(), "cpu_percent": actual.cpu_usage()}));
        } }
        Ok(json!({"processes": rows, "cpu_basis": "100% = one logical CPU; JVM process only, not per mod"}))
    }).await.map_err(err)?
}
/// Windows Defender file-only scan, with remediation explicitly disabled.
pub async fn antivirus(id: &str, rel: &str) -> Result<Value> {
    let base = root(id).await?;
    if !rel.starts_with("mods/") {
        return Err("Only mod files can be checked".into());
    }
    let path = child(&base, rel)?;
    if !path.is_file() || path.metadata().map_err(err)?.len() > MAX_FILE {
        return Err("Unsupported file or size".into());
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Ok(
            json!({"status": "unavailable", "message": "System antivirus scan is available only on Windows with Microsoft Defender. Static inspection is not an antivirus."}),
        )
    }
    #[cfg(windows)]
    {
        let program_files = std::env::var_os("ProgramFiles")
            .ok_or("ProgramFiles is unavailable")?;
        let exe =
            PathBuf::from(program_files).join("Windows Defender/MpCmdRun.exe");
        if !exe.is_file() {
            return Ok(
                json!({"status": "unavailable", "message": "Microsoft Defender command-line scanner was not found"}),
            );
        }
        let mut cmd = tokio::process::Command::new(exe);
        cmd.args(["-Scan", "-ScanType", "3", "-File"])
            .arg(&path)
            .arg("-DisableRemediation")
            .kill_on_drop(true);
        let status =
            tokio::time::timeout(Duration::from_secs(300), cmd.status())
                .await
                .map_err(|_| "Antivirus scan timed out".to_string())?
                .map_err(err)?;
        // Exit code 2 includes both detections and errors. Never label it confirmed malware.
        Ok(
            json!({"status": if status.code() == Some(0) { "no_detection" } else { "review" }, "exit_code": status.code(), "message": if status.code() == Some(0) { "No threats reported by Defender. This is not a safety guarantee." } else { "Defender reported a detection or scan error. Review Windows Security protection history; consider disabling the mod pending review." }}),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fabric_dependencies_and_aliases() {
        let mut m = ModInfo {
            path: "mods/a.jar".into(),
            name: "a".into(),
            ids: vec![],
            dependencies: vec![],
            disabled: false,
            sha256: String::new(),
            bytes: 0,
            warnings: vec![],
            metadata_complete: false,
        };
        parse_json(
            &json!({"id":"a","provides":["alias"],"depends":{"fabric-api":">=1"},"recommends":{"b":"*"}}),
            false,
            &mut m,
        );
        assert!(m.ids.contains(&"alias".into()));
        assert!(m.dependencies[0].required);
        assert!(!m.dependencies[1].required);
    }
    #[test]
    fn no_parent_paths() {
        let d = tempfile::tempdir().unwrap();
        assert!(child(d.path(), "../outside").is_err());
        assert!(child(d.path(), "/etc/passwd").is_err());
        assert!(child(d.path(), "mods\\a.jar").is_err());
    }
    #[test]
    fn malformed_toml_is_error() {
        let mut m = ModInfo {
            path: String::new(),
            name: String::new(),
            ids: vec![],
            dependencies: vec![],
            disabled: false,
            sha256: String::new(),
            bytes: 0,
            warnings: vec![],
            metadata_complete: false,
        };
        assert!(parse_toml("[not valid", &mut m).is_err());
    }
    fn fixture(entries: &[(&str, &[u8])]) -> (tempfile::TempDir, PathBuf) {
        use std::io::Write;
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("test.jar");
        let mut zip =
            zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        for (name, bytes) in entries {
            zip.start_file(*name, zip::write::FileOptions::default())
                .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
        (d, path)
    }
    #[test]
    fn jar_inspection_reports_risk_without_executing() {
        let (_d, p) = fixture(&[
            (
                "fabric.mod.json",
                br#"{"id":"test","depends":{"fabric-api":"*"}}"#,
            ),
            ("payload.ps1", b"Write-Output test"),
            ("A.class", b"java/lang/Runtime powershell"),
        ]);
        let m = inspect(&p, "mods/test.jar".into()).unwrap();
        assert_eq!(m.sha256.len(), 64);
        assert_eq!(m.ids, ["test"]);
        assert!(m.warnings.iter().any(|s| s.contains("script")));
        assert!(m.warnings.iter().any(|s| s.contains("shell")));
    }
    #[test]
    fn zip_slip_is_reported_not_extracted() {
        let (d, p) = fixture(&[("../escaped.txt", b"test")]);
        assert!(inspect(&p, "mods/test.jar".into())
            .unwrap()
            .warnings
            .iter()
            .any(|s| s.contains("Unsafe archive path")));
        assert!(!d.path().join("escaped.txt").exists());
    }
    #[test]
    fn missing_toml_mods_never_panics() {
        let (_d, p) =
            fixture(&[("META-INF/mods.toml", b"modLoader = \"javafml\"\n")]);
        let m = inspect(&p, "mods/test.jar".into()).unwrap();
        assert!(!m.metadata_complete);
    }
    #[test]
    fn nested_jars_are_explicitly_incomplete() {
        let (_d, p) = fixture(&[(
            "fabric.mod.json",
            br#"{"id":"test","jars":[{"file":"inner.jar"}]}"#,
        )]);
        assert!(
            !inspect(&p, "mods/test.jar".into())
                .unwrap()
                .metadata_complete
        );
    }
    #[cfg(unix)]
    #[test]
    fn symlinks_cannot_escape_scan_scope() {
        let d = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("/etc/passwd", d.path().join("link"))
            .unwrap();
        assert!(child(d.path(), "link").is_err());
    }
}
