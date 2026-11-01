//! Local, bounded diagnostics. Never executes a mod or uploads its contents.
use crate::{process, profile, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{Read, Seek},
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime},
};
pub type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
const MAX_FILE: u64 = 256 * 1024 * 1024;
// Jar-in-Jar contents are read in memory only, within these bounds.
const MAX_NESTED_FILE: u64 = 16 * 1024 * 1024;
const MAX_NESTED_TOTAL: u64 = 96 * 1024 * 1024;
const MAX_NESTED_JARS: usize = 256;
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
/// A mod shipped inside another mod's JAR (Jar-in-Jar).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Bundled {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Version declared by the mod's own manifest.
    #[serde(default)]
    pub version: Option<String>,
    /// Manifest families present in the JAR: fabric, quilt, neoforge, forge.
    #[serde(default)]
    pub loaders: Vec<String>,
    /// Declared incompatibilities; `required` marks a hard break.
    #[serde(default)]
    pub conflicts: Vec<Dependency>,
    /// Mods shipped inside nested JARs; their ids count as installed.
    #[serde(default)]
    pub bundled: Vec<Bundled>,
}
fn dep(id: &str, required: bool, version: Value) -> Dependency {
    Dependency {
        id: id.into(),
        required,
        version,
    }
}
fn metadata<R: Read + Seek>(
    z: &mut zip::ZipArchive<R>,
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
/// Parses fabric.mod.json or quilt.mod.json and returns the nested JAR paths it declares.
fn parse_json(v: &Value, quilt: bool, out: &mut ModInfo) -> Vec<String> {
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
    if out.version.is_none() {
        out.version = m["version"].as_str().map(str::to_owned);
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
        if let Some(deps) = m["breaks"].as_array() {
            for d in deps {
                if let Some(id) = d.as_str().or_else(|| d["id"].as_str()) {
                    out.conflicts.push(dep(id, true, d["versions"].clone()));
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
        for (key, hard) in [("breaks", true), ("conflicts", false)] {
            if let Some(deps) = m[key].as_object() {
                for (id, version) in deps {
                    out.conflicts.push(dep(id, hard, version.clone()));
                }
            }
        }
    }
    out.metadata_complete = !out.ids.is_empty() && out.warnings.is_empty();
    let mut jars = vec![];
    for list in [&v["jars"], &m["jars"]] {
        for j in list.as_array().into_iter().flatten() {
            if let Some(file) = j.as_str().or_else(|| j["file"].as_str()) {
                jars.push(file.to_owned());
            }
        }
    }
    jars.sort();
    jars.dedup();
    jars
}
fn parse_toml(text: &str, out: &mut ModInfo) -> Result<()> {
    let v: toml::Value = text.parse().map_err(err)?;
    if let Some(mods) = v.get("mods").and_then(|x| x.as_array()) {
        let mut named = false;
        for m in mods {
            if let Some(id) = m.get("modId").and_then(|x| x.as_str()) {
                out.ids.push(id.into());
            }
            if let Some(name) = m.get("displayName").and_then(|x| x.as_str()) {
                if !named {
                    out.name = name.into();
                    named = true;
                }
            }
            if out.version.is_none() {
                out.version = m
                    .get("version")
                    .and_then(|x| x.as_str())
                    .map(str::to_owned);
            }
        }
    }
    if let Some(groups) = v.get("dependencies").and_then(|x| x.as_table()) {
        for (_, deps) in groups {
            if let Some(deps) = deps.as_array() {
                for d in deps {
                    let Some(id) = d.get("modId").and_then(|x| x.as_str())
                    else {
                        continue;
                    };
                    // Server-only entries do not affect whether a client starts.
                    if d.get("side").and_then(|x| x.as_str()) == Some("SERVER") {
                        continue;
                    }
                    let kind = d
                        .get("type")
                        .and_then(|x| x.as_str())
                        .map(|x| x.to_ascii_lowercase());
                    let range =
                        json!(d.get("versionRange").and_then(|x| x.as_str()));
                    match kind.as_deref() {
                        Some("incompatible") => {
                            out.conflicts.push(dep(id, true, range))
                        }
                        Some("discouraged") => {
                            out.conflicts.push(dep(id, false, range))
                        }
                        _ => {
                            let required = d
                                .get("mandatory")
                                .and_then(|x| x.as_bool())
                                .unwrap_or_else(|| {
                                    kind.as_deref().unwrap_or("required")
                                        == "required"
                                });
                            out.dependencies.push(dep(id, required, range));
                        }
                    }
                }
            }
        }
    }
    out.metadata_complete = !out.ids.is_empty();
    Ok(())
}
/// Forge's `${file.jarVersion}` placeholder resolves to the JAR manifest version.
fn manifest_version<R: Read + Seek>(z: &mut zip::ZipArchive<R>) -> Option<String> {
    metadata(z, "META-INF/MANIFEST.MF")
        .ok()?
        .lines()
        .find_map(|l| l.strip_prefix("Implementation-Version:"))
        .map(|x| x.trim().to_owned())
        .filter(|x| !x.is_empty() && !x.contains("${"))
}
/// Reads the manifest of one archive into `out` and returns declared nested JAR paths.
fn read_manifest<R: Read + Seek>(
    z: &mut zip::ZipArchive<R>,
    names: &BTreeSet<String>,
    out: &mut ModInfo,
) -> Result<Vec<String>> {
    for (file, loader) in [
        ("fabric.mod.json", "fabric"),
        ("quilt.mod.json", "quilt"),
        ("META-INF/neoforge.mods.toml", "neoforge"),
        ("META-INF/mods.toml", "forge"),
    ] {
        if names.contains(file) {
            out.loaders.push(loader.into());
        }
    }
    let listed = if names.contains("fabric.mod.json") {
        let v: Value = serde_json::from_str(&metadata(z, "fabric.mod.json")?)
            .map_err(err)?;
        parse_json(&v, false, out)
    } else if names.contains("quilt.mod.json") {
        let v: Value = serde_json::from_str(&metadata(z, "quilt.mod.json")?)
            .map_err(err)?;
        parse_json(&v, true, out)
    } else if let Some(n) = ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"]
        .iter()
        .find(|n| names.contains(**n))
    {
        parse_toml(&metadata(z, n)?, out)?;
        vec![]
    } else {
        return Err("No supported Fabric/Quilt/Forge/NeoForge manifest".into());
    };
    if out.version.as_deref().map_or(false, |v| v.contains("${")) {
        out.version = manifest_version(z);
    }
    Ok(listed)
}
/// Collects mods from nested JARs (read in memory, never extracted). Returns entries
/// that could not be inspected.
fn nested<R: Read + Seek>(
    z: &mut zip::ZipArchive<R>,
    depth: u8,
    budget: &mut u64,
    out: &mut ModInfo,
) -> Vec<String> {
    let jars: Vec<String> = z
        .file_names()
        .filter(|n| n.to_ascii_lowercase().ends_with(".jar"))
        .map(str::to_owned)
        .collect();
    let mut skipped = vec![];
    for name in jars {
        if out.bundled.len() >= MAX_NESTED_JARS {
            skipped.push(name);
            continue;
        }
        let data = match z.by_name(&name) {
            Ok(mut f) if f.size() <= MAX_NESTED_FILE && f.size() <= *budget => {
                let mut data = vec![];
                match f
                    .by_ref()
                    .take(MAX_NESTED_FILE + 1)
                    .read_to_end(&mut data)
                {
                    Ok(_) if data.len() as u64 <= MAX_NESTED_FILE => Some(data),
                    _ => None,
                }
            }
            _ => None,
        };
        let Some(data) = data else {
            skipped.push(name);
            continue;
        };
        *budget = budget.saturating_sub(data.len() as u64);
        let Ok(mut inner) = zip::ZipArchive::new(std::io::Cursor::new(data))
        else {
            skipped.push(name);
            continue;
        };
        let names: BTreeSet<String> =
            inner.file_names().map(str::to_owned).collect();
        let mut info = ModInfo::default();
        match read_manifest(&mut inner, &names, &mut info) {
            Ok(_) => {}
            // Plain libraries inside a mod have no mod manifest.
            Err(_) if info.loaders.is_empty() => {}
            Err(_) => skipped.push(name.clone()),
        }
        for id in &info.ids {
            out.bundled.push(Bundled {
                id: id.clone(),
                name: Some(info.name.clone()).filter(|x| !x.is_empty()),
                version: info.version.clone(),
            });
        }
        if depth < 2 {
            skipped.extend(nested(&mut inner, depth + 1, budget, out));
        }
    }
    skipped
}
fn inspect(path: &Path, rel: String) -> Result<ModInfo> {
    let bytes = path.metadata().map_err(err)?.len();
    let mut out = ModInfo {
        name: rel.clone(),
        disabled: rel.ends_with(".disabled"),
        path: rel,
        bytes,
        ..Default::default()
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
    match read_manifest(&mut z, &names, &mut out) {
        Ok(listed) => {
            let mut budget = MAX_NESTED_TOTAL;
            let skipped = nested(&mut z, 0, &mut budget, &mut out);
            if !skipped.is_empty() || listed.iter().any(|j| !names.contains(j)) {
                out.metadata_complete = false;
                out.warnings.push(
                    "Nested JAR dependencies require loader validation".into(),
                );
            }
        }
        Err(e) => out.warnings.push(e),
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
                disabled: rel.ends_with(".disabled"),
                path: rel,
                name,
                warnings: vec![e],
                ..Default::default()
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
    let mut evidence = vec![];
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
        if let Some(line) = s.lines().find(|l| l.contains(pattern)) {
            findings.push(kind);
            evidence.push(json!({
                "kind": kind,
                "line": line.trim().chars().take(300).collect::<String>(),
            }));
        }
    }
    findings.sort();
    findings.dedup();
    Ok(json!({
        "crashed": !findings.is_empty(),
        "findings": findings,
        "evidence": evidence,
        "details": loader_details(&s),
        "modified_ms": modified,
        "heuristic": true,
    }))
}
/// The loader's own explanation of unresolved or incompatible mods, line by line.
fn loader_details(log: &str) -> Vec<String> {
    let lines: Vec<&str> = log.lines().collect();
    let mut details: Vec<String> = vec![];
    for (i, line) in lines.iter().enumerate() {
        if !(line.contains("Incompatible mods found")
            || line.contains("Missing or unsupported mandatory dependencies"))
        {
            continue;
        }
        for next in lines.iter().skip(i + 1).take(40) {
            let t = next.trim();
            if t.starts_with("- ") || t.starts_with("Mod ID:") {
                let t: String = t.chars().take(300).collect();
                if !details.contains(&t) {
                    details.push(t);
                }
            }
        }
    }
    details.truncate(12);
    details
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
            ..Default::default()
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
    fn loader_explanations_are_extracted() {
        let log = "[main/ERROR]: Incompatible mods found!\nnet.fabricmc.loader.impl.FormattedException: Some of your mods are incompatible\nA potential solution has been determined:\n\t - Install fabric-api, any version.\nMore details:\n\t - Mod 'Sodium' (sodium) 0.5.8 requires any version of fabric-api, which is missing!\n\tat net.fabricmc.Main.main(Main.java:1)\n";
        let d = loader_details(log);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0], "- Install fabric-api, any version.");
        assert!(d[1].starts_with("- Mod 'Sodium'"));
        assert!(loader_details("[main/INFO]: - nothing").is_empty());
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
        let mut m = ModInfo::default();
        assert!(parse_toml("[not valid", &mut m).is_err());
    }
    #[test]
    fn fabric_version_and_breaks_are_reported() {
        let mut m = ModInfo::default();
        parse_json(
            &json!({"id":"a","version":"1.2.3","breaks":{"optifabric":"*"},"conflicts":{"b":"<2"}}),
            false,
            &mut m,
        );
        assert_eq!(m.version.as_deref(), Some("1.2.3"));
        assert_eq!(m.conflicts.len(), 2);
        assert!(m.conflicts.iter().any(|c| c.id == "optifabric" && c.required));
        assert!(m.conflicts.iter().any(|c| c.id == "b" && !c.required));
    }
    #[test]
    fn forge_incompatibility_and_jar_version() {
        let toml = br#"modLoader = "javafml"
loaderVersion = "[47,)"
[[mods]]
modId = "x"
version = "${file.jarVersion}"
displayName = "X Mod"
[[dependencies.x]]
modId = "forge"
mandatory = true
versionRange = "[47,)"
side = "BOTH"
[[dependencies.x]]
modId = "badmod"
type = "incompatible"
versionRange = "[1,)"
side = "CLIENT"
[[dependencies.x]]
modId = "serveronly"
mandatory = true
side = "SERVER"
"#;
        let (_d, p) = fixture(&[
            ("META-INF/mods.toml", toml),
            (
                "META-INF/MANIFEST.MF",
                b"Manifest-Version: 1.0\r\nImplementation-Version: 2.1.0\r\n",
            ),
        ]);
        let m = inspect(&p, "mods/x.jar".into()).unwrap();
        assert_eq!(m.name, "X Mod");
        assert_eq!(m.version.as_deref(), Some("2.1.0"));
        assert_eq!(m.loaders, ["forge"]);
        assert!(m.metadata_complete);
        assert_eq!(m.dependencies.len(), 1);
        assert!(m.dependencies[0].required);
        assert_eq!(m.conflicts.len(), 1);
        assert_eq!(m.conflicts[0].id, "badmod");
        assert!(m.conflicts[0].required);
    }
    fn jar_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            zip.start_file(*name, zip::write::FileOptions::default())
                .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }
    #[test]
    fn nested_mods_count_as_installed() {
        let inner = jar_bytes(&[(
            "fabric.mod.json",
            br#"{"id":"fabric-api-base","name":"Fabric API Base","version":"0.4.31"}"#,
        )]);
        let library = jar_bytes(&[("com/example/Lib.class", b"library")]);
        let (_d, p) = fixture(&[
            (
                "fabric.mod.json",
                br#"{"id":"fabric-api","version":"0.92.0","jars":[{"file":"META-INF/jars/base.jar"},{"file":"META-INF/jars/lib.jar"}]}"#,
            ),
            ("META-INF/jars/base.jar", &inner),
            ("META-INF/jars/lib.jar", &library),
        ]);
        let m = inspect(&p, "mods/fabric-api.jar".into()).unwrap();
        assert_eq!(m.version.as_deref(), Some("0.92.0"));
        assert_eq!(m.loaders, ["fabric"]);
        assert!(m.metadata_complete, "{:?}", m.warnings);
        assert_eq!(m.bundled.len(), 1);
        assert_eq!(m.bundled[0].id, "fabric-api-base");
        assert_eq!(m.bundled[0].name.as_deref(), Some("Fabric API Base"));
        assert_eq!(m.bundled[0].version.as_deref(), Some("0.4.31"));
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
