//! Resource packs are data, not native mods. Fixed Modrinth endpoints, bounded downloads,
//! checksum verification and private per-profile files. Never unpack into the game tree.
use super::*;
use sha2::Sha512;
use std::{
    collections::BTreeSet,
    io::{Cursor, Read, Write},
};
const MAX_PACK: usize = 128 << 20;
const MAX_EXPANDED: u64 = 512 << 20;
const MAX_ENTRIES: usize = 30000;
const PAGE: u32 = 20;
pub(super) fn protect_default() -> bool {
    true
}
fn library() -> Vec<Value> {
    serde_json::from_str(include_str!("resourcepacks.json"))
        .expect("embedded resource catalog")
}
pub(super) fn validate_request(r: &Request) -> Result<()> {
    if r.project.len() > 512
        || r.profile.len() > 160
        || r.pack_file.len() > 4096
        || r.project.chars().any(char::is_control)
        || r.pack_file.chars().any(char::is_control)
    {
        return Err("Invalid resource pack request".into());
    }
    if r.action == "pack_import" && !Path::new(&r.pack_file).is_absolute() {
        return Err("Choose a ZIP archive".into());
    }
    Ok(())
}
fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("U3drinth/1.0 (Horizon resource packs)")
        .timeout(Duration::from_secs(180))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())
}
async fn response_bytes(
    response: reqwest::Response,
    max: usize,
) -> Result<Vec<u8>> {
    let mut response =
        response.error_for_status().map_err(|e| e.to_string())?;
    if response.content_length().is_some_and(|s| s > max as u64) {
        return Err("Resource pack response is too large".into());
    }
    let mut data = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if data.len().saturating_add(chunk.len()) > max {
            return Err("Resource pack response is too large".into());
        }
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}
async fn api(path: &str, params: &[(&str, String)]) -> Result<Value> {
    let response = client()?
        .get(format!("https://api.modrinth.com/v2/{path}"))
        .query(params)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    serde_json::from_slice(&response_bytes(response, 4 << 20).await?)
        .map_err(|e| e.to_string())
}
fn identifier(s: &str) -> Result<&str> {
    if s.is_empty()
        || s.len() > 100
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        Err("Invalid Modrinth project".into())
    } else {
        Ok(s)
    }
}
fn format(v: &Value) -> Option<(u64, u64)> {
    if let Some(n) = v.as_u64() {
        return Some((n, 0));
    }
    let a = v.as_array()?;
    Some((
        a.first()?.as_u64()?,
        a.get(1).and_then(Value::as_u64).unwrap_or(0),
    ))
}
fn compatible(meta: &Value) -> bool {
    let p = &meta["pack"];
    let legacy = format(&p["pack_format"]);
    let supported = &p["supported_formats"];
    let range = if supported.is_array() {
        supported
            .as_array()
            .and_then(|a| Some((format(a.first()?)?, format(a.get(1)?)?)))
    } else {
        format(&supported["min_inclusive"])
            .zip(format(&supported["max_inclusive"]))
    };
    let lo = format(&p["min_format"]).or(range.map(|r| r.0)).or(legacy);
    let hi = format(&p["max_format"]).or(range.map(|r| r.1)).or(legacy);
    let hi = hi.map(|(major, minor)| {
        (major, if minor == 0 { u64::MAX } else { minor })
    });
    matches!((lo,hi),(Some(lo),Some(hi)) if lo <= (97,1) && (97,1) <= hi)
}
fn cache_path(root: &Path, hash: &str) -> Result<PathBuf> {
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid pack checksum".into());
    }
    Ok(root.join("resource-cache").join(format!("{hash}.zip")))
}
fn sha(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn cache_read(root: &Path, entry: &Value) -> Result<Vec<u8>> {
    let path =
        cache_path(root, entry["sha256"].as_str().ok_or("Missing checksum")?)?;
    let data = read_bounded(&path, MAX_PACK)?;
    if sha(&data) != entry["sha256"] {
        return Err("Resource pack checksum mismatch".into());
    }
    Ok(data)
}
fn read_bounded(path: &Path, max: usize) -> Result<Vec<u8>> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > max as u64 {
        return Err("Invalid or oversized resource pack file".into());
    }
    let mut data = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(max as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > max {
        return Err("Resource pack is too large".into());
    }
    Ok(data)
}
fn write_new(path: &Path, data: &[u8]) -> Result<()> {
    use std::fs::OpenOptions;
    if path.exists() {
        if read_bounded(path, MAX_PACK)? == data {
            return Ok(());
        }
        return Err(
            "An existing resource pack differs. It was not overwritten".into(),
        );
    }
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    if let Err(e) = f.write_all(data).and_then(|_| f.sync_all()) {
        drop(f);
        let _ = std::fs::remove_file(path);
        return Err(e.to_string());
    }
    Ok(())
}
async fn download(url: &str) -> Result<Vec<u8>> {
    let u = url::Url::parse(url).map_err(|e| e.to_string())?;
    if u.scheme() != "https"
        || u.host_str() != Some("cdn.modrinth.com")
        || u.port().is_some()
        || !u.username().is_empty()
        || u.password().is_some()
    {
        return Err("Untrusted resource pack download".into());
    }
    let response = client()?.get(u).send().await.map_err(|e| e.to_string())?;
    response_bytes(response, MAX_PACK).await
}
async fn source(root: &Path, project: &str) -> Result<Vec<u8>> {
    if let Some(e) = library().into_iter().find(|e| e["id"] == project) {
        if let Ok(data) = cache_read(root, &e) {
            return Ok(data);
        }
        let url = e["url"].as_str().ok_or(
            "This pack is local. Import PvP.zip once with Add archive",
        )?;
        let data = download(url).await?;
        if sha(&data) != e["sha256"]
            || data.len() as u64 != e["size"].as_u64().unwrap_or(0)
        {
            return Err("Resource pack checksum mismatch".into());
        }
        let path = cache_path(root, e["sha256"].as_str().unwrap())?;
        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|e| e.to_string())?;
        write_new(&path, &data)?;
        return Ok(data);
    }
    let id = identifier(
        project.strip_prefix("mr:").ok_or("Unknown resource pack")?,
    )?;
    let p = api(&format!("project/{id}"), &[]).await?;
    if p["project_type"] != "resourcepack" {
        return Err("The selected project is not a resource pack".into());
    }
    let versions = api(
        &format!("project/{id}/version"),
        &[("game_versions", "[\"26.3\"]".into())],
    )
    .await?;
    let v = versions
        .as_array()
        .into_iter()
        .flatten()
        .find(|v| {
            v["game_versions"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == "26.3"))
        })
        .ok_or("No resource pack version for Minecraft 26.3")?;
    let f = v["files"]
        .as_array()
        .and_then(|a| a.iter().find(|f| f["primary"] == true).or(a.first()))
        .ok_or("No pack download")?;
    let hash = f["hashes"]["sha512"]
        .as_str()
        .ok_or("Missing SHA-512 checksum")?;
    if hash.len() != 128 {
        return Err("Invalid resource pack checksum".into());
    }
    let data = download(f["url"].as_str().ok_or("No download URL")?).await?;
    if format!("{:x}", Sha512::digest(&data)) != hash
        || f["size"].as_u64() != Some(data.len() as u64)
    {
        return Err("Resource pack checksum mismatch".into());
    }
    Ok(data)
}
// A zip is inspected in full even when entries are omitted. No paths are extracted to disk.
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 1024
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name.contains(':')
        && !name.chars().any(char::is_control)
        && !name.split('/').any(|p| p == ".." || p == ".")
}
fn protected(name: &str) -> bool {
    let parts: Vec<_> = name.split('/').collect();
    let Some(i) = parts.iter().position(|s| *s == "assets") else {
        return false;
    };
    let p = parts
        .get(i + 2..)
        .unwrap_or(&[])
        .join("/")
        .to_ascii_lowercase();
    p.starts_with("textures/gui/")
        || p.starts_with("font/")
        || p.starts_with("textures/font/")
        || p.starts_with("shaders/")
        || p == "atlases/gui.json"
        || p.starts_with("lang/")
        || p.starts_with("optifine/gui/")
        || p.starts_with("optifine/font/")
}
fn transform(
    data: &[u8],
    protect: bool,
    allow_incompatible: bool,
) -> Result<Vec<u8>> {
    let mut zip =
        zip::ZipArchive::new(Cursor::new(data)).map_err(|e| e.to_string())?;
    if zip.len() > MAX_ENTRIES {
        return Err("Too many resource pack entries".into());
    }
    let mut names = BTreeSet::new();
    let mut total = 0u64;
    let mut metas = Vec::new();
    for i in 0..zip.len() {
        let f = zip.by_index(i).map_err(|e| e.to_string())?;
        if !safe_name(f.name())
            || f.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000)
            || !names.insert(f.name().to_lowercase())
        {
            return Err("Unsafe or duplicate resource pack path".into());
        }
        total = total
            .checked_add(f.size())
            .ok_or("Resource pack size overflow")?;
        if total > MAX_EXPANDED || f.size() > MAX_PACK as u64 {
            return Err("Expanded resource pack is too large".into());
        }
        if f.name() == "pack.mcmeta" || f.name().ends_with("/pack.mcmeta") {
            metas.push(f.name().to_string());
        }
    }
    let meta_path = if metas.iter().any(|s| s == "pack.mcmeta") {
        "pack.mcmeta".to_owned()
    } else if metas.len() == 1 {
        metas[0].clone()
    } else {
        return Err(
            "ZIP must contain one resource pack, not a collection".into()
        );
    };
    let prefix = meta_path.trim_end_matches("pack.mcmeta");
    let mut meta_data = Vec::new();
    zip.by_name(&meta_path)
        .map_err(|e| e.to_string())?
        .take((1 << 20) + 1)
        .read_to_end(&mut meta_data)
        .map_err(|e| e.to_string())?;
    if meta_data.len() > 1 << 20 {
        return Err("Pack metadata is too large".into());
    }
    let mut meta: Value =
        serde_json::from_slice(&meta_data).map_err(|e| e.to_string())?;
    if !meta["pack"].is_object() {
        return Err("Invalid pack metadata".into());
    }
    if !allow_incompatible && !compatible(&meta) {
        return Err("This pack format does not support Minecraft 26.3 (97.1). Choose a compatible pack".into());
    }
    if protect {
        meta.as_object_mut()
            .ok_or("Invalid metadata")?
            .remove("filter");
    }
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|e| e.to_string())?;
        // Reading fully verifies ZIP CRC, including ignored GUI files.
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        let Some(name) = f.name().strip_prefix(prefix) else {
            continue;
        };
        if name.is_empty() || f.is_dir() || (protect && protected(name)) {
            continue;
        }
        if name == "pack.mcmeta" {
            bytes = serde_json::to_vec(&meta).map_err(|e| e.to_string())?;
        }
        output
            .start_file(name, options)
            .map_err(|e| e.to_string())?;
        output.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    let bytes = output.finish().map_err(|e| e.to_string())?.into_inner();
    if bytes.len() > MAX_PACK {
        return Err("Processed resource pack is too large".into());
    }
    Ok(bytes)
}
fn import(root: &Path, path: &Path) -> Result<usize> {
    let data = read_bounded(path, MAX_PACK)?;
    let entries = library();
    let mut packs = Vec::new();
    let direct = sha(&data);
    if entries.iter().any(|e| e["sha256"] == direct) {
        packs.push(data);
    } else {
        let mut z = zip::ZipArchive::new(Cursor::new(data))
            .map_err(|e| e.to_string())?;
        if z.len() > 256 {
            return Err("Too many files in pack collection".into());
        }
        let mut total = 0u64;
        for i in 0..z.len() {
            let mut f = z.by_index(i).map_err(|e| e.to_string())?;
            total = total.saturating_add(f.size());
            if total > MAX_EXPANDED || f.size() > MAX_PACK as u64 {
                return Err("Pack collection is too large".into());
            }
            if f.is_dir() || !f.name().to_lowercase().ends_with(".zip") {
                continue;
            }
            let mut b = Vec::new();
            f.read_to_end(&mut b).map_err(|e| e.to_string())?;
            if entries.iter().any(|e| e["sha256"] == sha(&b)) {
                packs.push(b);
            }
        }
    }
    if packs.is_empty() {
        return Err("No matching packs. Choose the provided PvP.zip, Default.zip or an original pack from them".into());
    }
    std::fs::create_dir_all(root.join("resource-cache"))
        .map_err(|e| e.to_string())?;
    let count = packs.len();
    for data in packs {
        let hash = sha(&data);
        write_new(&cache_path(root, &hash)?, &data)?;
    }
    Ok(count)
}
async fn profile(root: &Path, requested: &str) -> Result<(String, PathBuf)> {
    let instances = query(root, &["--json", "instance", "list"]).await?;
    let id = selected_instance(&instances, requested)?;
    let path = mods::profile_home(root, &instances, &id)?;
    Ok((id, path))
}
fn record(home: &Path, file: &str) -> Value {
    if !file.starts_with("u3-") || !safe_name(file) || file.contains('/') {
        return json!({});
    }
    read_bounded(
        &home.join("u3-resourcepacks").join(format!("{file}.json")),
        65536,
    )
    .ok()
    .and_then(|b| serde_json::from_slice(&b).ok())
    .unwrap_or(json!({}))
}
fn record_write(home: &Path, file: &str, value: &Value) -> Result<()> {
    let dir = home.join("u3-resourcepacks");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // Content-addressed pack names make this record immutable too.
    let path = dir.join(format!("{file}.json"));
    if !path.exists() {
        write_new(
            &path,
            &serde_json::to_vec(value).map_err(|e| e.to_string())?,
        )?;
    }
    Ok(())
}
async fn pack_list(root: &Path, id: &str) -> Result<Value> {
    query(
        root,
        &[
            "--json",
            "packs",
            "list",
            "--kind",
            "resource",
            "--instance",
            id,
        ],
    )
    .await
}
fn pack_folder(home: &Path) -> Result<PathBuf> {
    let dir = home.join("resourcepacks");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let canonical = std::fs::canonicalize(&dir).map_err(|e| e.to_string())?;
    if !canonical.starts_with(home) {
        return Err("Resource pack folder is outside the profile".into());
    }
    Ok(canonical)
}
fn backup(home: &Path) -> Result<()> {
    let original = home.join("options.txt");
    if original.exists() {
        let data = read_bounded(&original, 1 << 20)?;
        let dir = home.join("u3-pack-backups");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        write_new(&dir.join(format!("options-{}.txt", sha(&data))), &data)?;
    }
    Ok(())
}
fn replace_options(
    text: &str,
    replacements: &[(String, String)],
) -> Result<String> {
    let mut output = String::new();
    let mut found = false;
    for line in text.split_inclusive('\n') {
        if let Some(value) = line
            .trim_end_matches(['\r', '\n'])
            .strip_prefix("resourcePacks:")
        {
            if found {
                return Err(
                    "Duplicate resourcePacks option. Settings were not changed"
                        .into(),
                );
            }
            found = true;
            let mut ids: Vec<String> = serde_json::from_str(value)
                .map_err(|_| "Invalid resourcePacks option")?;
            for id in &mut ids {
                if let Some((_, new)) = replacements
                    .iter()
                    .find(|(old, _)| *id == format!("file/{old}"))
                {
                    *id = format!("file/{new}");
                }
            }
            output.push_str("resourcePacks:");
            output.push_str(
                &serde_json::to_string(&ids).map_err(|e| e.to_string())?,
            );
            if line.ends_with("\r\n") {
                output.push_str("\r\n");
            } else if line.ends_with('\n') {
                output.push('\n');
            }
        } else {
            output.push_str(line);
        }
    }
    if !found && !replacements.is_empty() {
        return Err(
            "Missing resourcePacks option. Settings were not changed".into()
        );
    }
    Ok(output)
}
fn replace_enabled(
    home: &Path,
    replacements: &[(String, String)],
) -> Result<()> {
    if replacements.is_empty() {
        return Ok(());
    }
    let path = home.join("options.txt");
    let bytes = read_bounded(&path, 1 << 20)?;
    let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    let output = replace_options(text, replacements)?;
    let temp = home.join(format!(".u3-options-{}.tmp", uuid::Uuid::new_v4()));
    write_new(&temp, output.as_bytes())?;
    let result = std::fs::rename(&temp, &path).map_err(|e| e.to_string());
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    result
}
async fn toggle(root: &Path, id: &str, name: &str, enable: bool) -> Result<()> {
    cli(
        root,
        &[
            "packs",
            if enable { "on" } else { "off" },
            name,
            "--kind",
            "resource",
            "--instance",
            id,
        ],
        false,
    )
    .await?;
    Ok(())
}
pub(super) async fn run(root: &Path, r: &Request) -> Result<()> {
    if r.action == "pack_import" {
        let base = root.to_owned();
        let file = PathBuf::from(&r.pack_file);
        let n = tokio::task::spawn_blocking(move || import(&base, &file))
            .await
            .map_err(|e| e.to_string())??;
        log_line(&format!("Cached {n} original resource packs"), false);
        return Ok(());
    }
    if verified(root).await.is_err() {
        install_runtime(root).await?;
    }
    let (id, home) = profile(root, &r.profile).await?;
    let dir = pack_folder(&home)?;
    if r.action == "pack_install" {
        let data = source(root, &r.project).await?;
        let protect = r.protect_gui;
        let bytes = tokio::task::spawn_blocking(move || {
            transform(&data, protect, false)
        })
        .await
        .map_err(|e| e.to_string())??;
        let hash = sha(&bytes);
        let name = format!(
            "u3-{}-{}.zip",
            if protect { "safe" } else { "original" },
            hash
        );
        write_new(&dir.join(&name), &bytes)?;
        let title = library()
            .iter()
            .find(|p| p["id"] == r.project)
            .and_then(|p| p["name"].as_str())
            .unwrap_or(&r.project)
            .to_owned();
        record_write(
            &home,
            &name,
            &json!({"name":title,"project":r.project,"protect_gui":protect}),
        )?;
        backup(&home)?;
        let options = home.join("options.txt");
        let before = std::fs::read(&options).ok();
        let list = pack_list(root, &id).await?;
        let result: Result<()> = async {
            for old in list
                .as_array()
                .into_iter()
                .flatten()
                .filter(|p| p["enabled"] == true)
            {
                let file =
                    old["file"].as_str().ok_or("Invalid pack filename")?;
                if file != name && record(&home, file)["project"] == r.project {
                    toggle(root, &id, file, false).await?;
                }
            }
            toggle(root, &id, &name, true).await
        }
        .await;
        if let Err(e) = result {
            if let Some(bytes) = before {
                std::fs::write(options, bytes).map_err(|e| e.to_string())?;
            }
            return Err(e);
        }
        log_line(
            if protect {
                "Resource pack enabled with standard GUI protection"
            } else {
                "Resource pack enabled with original GUI"
            },
            false,
        );
    } else if r.action == "pack_toggle" {
        let list = pack_list(root, &id).await?;
        let p = list
            .as_array()
            .into_iter()
            .flatten()
            .find(|p| p["file"] == r.project)
            .ok_or("Installed resource pack not found")?;
        let name = p["file"].as_str().ok_or("Invalid installed pack")?;
        if name.starts_with('-') || !safe_name(name) || name.contains('/') {
            return Err("Invalid installed pack filename".into());
        }
        backup(&home)?;
        toggle(root, &id, name, p["enabled"] != true).await?;
    } else if r.action == "pack_repair" {
        let list = pack_list(root, &id).await?;
        let mut replacements = Vec::new();
        for p in list
            .as_array()
            .into_iter()
            .flatten()
            .filter(|p| p["enabled"] == true)
        {
            let name = p["file"].as_str().ok_or("Invalid installed pack")?;
            if name.starts_with("u3-safe-") {
                continue;
            }
            if !safe_name(name) || name.contains('/') || name.starts_with('-') {
                return Err("Invalid installed pack filename".into());
            }
            let data=read_bounded(&dir.join(name),MAX_PACK).map_err(|_|"Repair supports ZIP packs only. Disable folder packs in the game first".to_string())?;
            let bytes = tokio::task::spawn_blocking(move || {
                transform(&data, true, false)
            })
            .await
            .map_err(|e| e.to_string())??;
            let new = format!("u3-safe-{}.zip", sha(&bytes));
            write_new(&dir.join(&new), &bytes)?;
            let old_record = record(&home, name);
            record_write(
                &home,
                &new,
                &json!({"name":old_record["name"].as_str().unwrap_or(name),"project":old_record["project"],"protect_gui":true}),
            )?;
            replacements.push((name.to_string(), new));
        }
        backup(&home)?;
        replace_enabled(&home, &replacements)?;
        log_line("Protected copies enabled. Original ZIP files preserved; options backed up in u3-pack-backups",false);
    }
    change(|s| s.result_profile = Some(id));
    Ok(())
}
pub async fn catalog(
    requested: &str,
    tab: &str,
    search: &str,
    offset: u32,
) -> Result<Value> {
    if search.chars().count() > 100
        || offset > 100000
        || !matches!(tab, "pvp" | "default" | "modrinth")
    {
        return Err("Invalid resource pack search".into());
    }
    let root = home().await?;
    let installed = if verified(&root).await.is_ok() {
        let (id, _) = profile(&root, requested).await?;
        pack_list(&root, &id).await?
    } else {
        json!([])
    };
    let mut entries = Vec::new();
    let mut total = 0;
    if tab == "modrinth" {
        let value = api(
            "search",
            &[
                (
                    "facets",
                    "[[\"project_type:resourcepack\"],[\"versions:26.3\"]]"
                        .into(),
                ),
                ("query", search.into()),
                ("limit", PAGE.to_string()),
                ("offset", offset.to_string()),
                ("index", "downloads".into()),
            ],
        )
        .await?;
        total = value["total_hits"].as_u64().unwrap_or(0);
        for p in value["hits"].as_array().into_iter().flatten() {
            if p["project_type"] != "resourcepack" {
                continue;
            }
            entries.push(json!({"id":format!("mr:{}",p["project_id"].as_str().unwrap_or("")),"name":p["title"],"summary":p["description"],"author":p["author"],"available":true,"local":false,"project":p["project_id"]}));
        }
    } else {
        for mut p in library().into_iter().filter(|p| p["category"] == tab) {
            if !p["name"]
                .as_str()
                .unwrap_or("")
                .to_lowercase()
                .contains(&search.to_lowercase())
            {
                continue;
            }
            p["available"] = json!(compatible(&p["meta"]));
            p["local"] =
                json!(p["url"].is_null() && cache_read(&root, &p).is_err());
            entries.push(p);
        }
    }
    let profile_path = if verified(&root).await.is_ok() {
        Some(profile(&root, requested).await?.1)
    } else {
        None
    };
    // Do not expose local filesystem paths in the frontend catalog.
    let installed:Vec<Value>=installed.as_array().into_iter().flatten().map(|p| {let r=profile_path.as_ref().map(|h|record(h,p["file"].as_str().unwrap_or(""))).unwrap_or(json!({}));json!({"file":p["file"],"enabled":p["enabled"],"description":p["description"],"name":r["name"],"project":r["project"],"protect_gui":r["protect_gui"]})}).collect();
    Ok(
        json!({"entries":entries,"installed":installed,"total":total,"offset":offset,"limit":PAGE}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn archive(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in files {
            z.start_file(*name, zip::write::FileOptions::default())
                .unwrap();
            z.write_all(data).unwrap();
        }
        z.finish().unwrap().into_inner()
    }
    const META: &[u8] = br#"{"pack":{"min_format":[97,0],"max_format":[97,1],"description":"test"},"filter":{"block":[{"namespace":"minecraft","path":".*"}]}}"#;
    #[test]
    fn formats_are_checked_without_rewriting_version() {
        assert!(compatible(&serde_json::from_slice(META).unwrap()));
        assert!(!compatible(&json!({"pack":{"pack_format":75}})));
        assert!(compatible(&json!({"pack":{"pack_format":97}})));
        assert!(compatible(
            &json!({"pack":{"min_format":75,"max_format":97}})
        ));
        assert!(compatible(
            &json!({"pack":{"pack_format":15,"supported_formats":[15,1000]}})
        ));
        assert!(compatible(
            &json!({"pack":{"supported_formats":{"min_inclusive":75,"max_inclusive":100}}})
        ));
        assert!(!compatible(
            &json!({"pack":{"min_format":[97,2],"max_format":[100,0]}})
        ));
        assert!(!compatible(&json!({"pack":{}})));
    }
    #[test]
    fn protected_copy_removes_gui_fonts_shaders_and_filters_including_overlays()
    {
        let data = archive(&[
            ("pack.mcmeta", META),
            ("assets/minecraft/textures/block/stone.png", b"stone"),
            (
                "assets/minecraft/textures/gui/sprites/hud/heart/full.png",
                b"broken",
            ),
            ("assets/minecraft/font/default.json", b"{}"),
            ("assets/minecraft/textures/font/ascii.png", b"bad"),
            ("assets/minecraft/atlases/gui.json", b"{}"),
            ("assets/minecraft/shaders/core/gui.json", b"bad"),
            ("assets/minecraft/lang/en_us.json", b"{}"),
            ("overlay/assets/example/textures/gui/a.png", b"bad"),
        ]);
        let original = data.clone();
        let safe = transform(&data, true, false).unwrap();
        assert_eq!(data, original);
        let mut z = zip::ZipArchive::new(Cursor::new(safe)).unwrap();
        assert_eq!(z.len(), 2);
        assert!(z
            .by_name("assets/minecraft/textures/block/stone.png")
            .is_ok());
        let mut b = Vec::new();
        z.by_name("pack.mcmeta")
            .unwrap()
            .read_to_end(&mut b)
            .unwrap();
        let m: Value = serde_json::from_slice(&b).unwrap();
        assert!(m.get("filter").is_none());
        assert_eq!(m["pack"]["max_format"], json!([97, 1]));
        let full = transform(&data, false, false).unwrap();
        let z = zip::ZipArchive::new(Cursor::new(full)).unwrap();
        assert_eq!(z.len(), 9);
    }
    #[test]
    fn wrapped_pack_is_normalized_not_nested_collection() {
        let d = archive(&[
            ("wrapper/pack.mcmeta", META),
            ("wrapper/assets/minecraft/textures/block/a.png", b"x"),
            ("readme.txt", b"ignored"),
        ]);
        let out = transform(&d, true, false).unwrap();
        let mut z = zip::ZipArchive::new(Cursor::new(out)).unwrap();
        assert_eq!(z.len(), 2);
        assert!(z.by_name("pack.mcmeta").is_ok());
        assert!(transform(
            &archive(&[("a/pack.mcmeta", META), ("b/pack.mcmeta", META)]),
            true,
            false
        )
        .is_err());
    }
    #[test]
    fn traversal_case_duplicates_and_incompatible_formats_rejected() {
        for name in ["../bad", "/bad", "C:/bad", "assets/../bad", "assets\\bad"]
        {
            assert!(
                transform(
                    &archive(&[("pack.mcmeta", META), (name, b"x")]),
                    true,
                    false
                )
                .is_err(),
                "{}",
                name
            );
        }
        assert!(transform(
            &archive(&[("pack.mcmeta", META), ("PACK.MCMETA", META)]),
            true,
            false
        )
        .is_err());
        assert!(transform(
            &archive(&[("pack.mcmeta", br#"{"pack":{"pack_format":75}}"#)]),
            true,
            false
        )
        .is_err());
    }
    #[test]
    fn catalog_has_52_exact_packs_with_only_three_local_archives() {
        let packs = library();
        assert_eq!(packs.len(), 52);
        assert_eq!(packs.iter().filter(|p| p["url"].is_null()).count(), 3);
        assert_eq!(packs.iter().filter(|p| p["category"] == "pvp").count(), 14);
        let mut ids = BTreeSet::new();
        for p in packs {
            assert!(ids.insert(p["id"].as_str().unwrap().to_owned()));
            assert!(cache_path(
                Path::new("/tmp"),
                p["sha256"].as_str().unwrap()
            )
            .is_ok());
            if let Some(url) = p["url"].as_str() {
                let u = url::Url::parse(url).unwrap();
                assert_eq!(u.host_str(), Some("cdn.modrinth.com"));
                assert_eq!(u.scheme(), "https");
            }
        }
    }
    #[test]
    fn managed_files_are_not_overwritten_and_backups_are_content_addressed() {
        let d = std::env::temp_dir()
            .join(format!("u3-resource-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("pack.zip");
        write_new(&p, b"old").unwrap();
        assert!(write_new(&p, b"new").is_err());
        assert_eq!(std::fs::read(p).unwrap(), b"old");
        std::fs::write(d.join("options.txt"), b"resourcePacks:[]\nvolume:1\n")
            .unwrap();
        backup(&d).unwrap();
        backup(&d).unwrap();
        assert_eq!(
            std::fs::read_dir(d.join("u3-pack-backups"))
                .unwrap()
                .count(),
            1
        );
        std::fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn repairing_keeps_stack_order_and_all_other_settings() {
        let text="volume:0.7\r\nresourcePacks:[\"vanilla\",\"file/z.zip\",\"file/a.zip\"]\r\nkey:thing\r\n";
        let out = replace_options(
            text,
            &[
                ("a.zip".into(), "safe-a.zip".into()),
                ("z.zip".into(), "safe-z.zip".into()),
            ],
        )
        .unwrap();
        assert_eq!(out,"volume:0.7\r\nresourcePacks:[\"vanilla\",\"file/safe-z.zip\",\"file/safe-a.zip\"]\r\nkey:thing\r\n");
        assert!(replace_options("resourcePacks:bad", &[]).is_err());
        assert!(
            replace_options("resourcePacks:[]\nresourcePacks:[]", &[]).is_err()
        );
    }
    #[test]
    fn protection_defaults_to_true_and_invalid_import_paths_fail() {
        let r: Request =
            serde_json::from_value(json!({"action":"pack_install"})).unwrap();
        assert!(r.protect_gui);
        let r: Request = serde_json::from_value(
            json!({"action":"pack_import","pack_file":"../pvp.zip"}),
        )
        .unwrap();
        assert!(validate_request(&r).is_err());
    }
    #[test]
    #[ignore = "Local QA: run with U3_PACK_LIBRARY pointing at the extracted user collection"]
    fn all_user_archives_are_validated_and_originals_unchanged() {
        let root = std::env::var("U3_PACK_LIBRARY").unwrap();
        let mut good = 0;
        for p in library() {
            let original = std::fs::read(
                Path::new(&root)
                    .join(format!("{}.zip", p["id"].as_str().unwrap())),
            )
            .unwrap();
            assert_eq!(sha(&original), p["sha256"]);
            let result = transform(&original, true, false);
            if compatible(&p["meta"]) {
                result.unwrap_or_else(|e| panic!("{}: {}", p["name"], e));
                good += 1;
            } else {
                assert!(result.is_err());
            }
            assert_eq!(
                sha(&std::fs::read(
                    Path::new(&root)
                        .join(format!("{}.zip", p["id"].as_str().unwrap()))
                )
                .unwrap()),
                p["sha256"]
            );
        }
        eprintln!("Compatible safe copies: {good}/52");
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;
    #[tokio::test]
    #[ignore = "Uses the real Modrinth API and CDN"]
    async fn modrinth_search_and_verified_download() {
        let value = api(
            "search",
            &[
                (
                    "facets",
                    "[[\"project_type:resourcepack\"],[\"versions:26.3\"]]"
                        .into(),
                ),
                ("limit", "3".into()),
            ],
        )
        .await
        .unwrap();
        let hits = value["hits"].as_array().unwrap();
        assert!(!hits.is_empty());
        for p in hits {
            assert_eq!(p["project_type"], "resourcepack");
            assert!(p["versions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "26.3"));
        }
        let pack = library()
            .into_iter()
            .find(|p| {
                !p["url"].is_null()
                    && compatible(&p["meta"])
                    && p["size"].as_u64().unwrap() < 100000
            })
            .unwrap();
        let dir = std::env::temp_dir()
            .join(format!("u3-packs-live-{}", uuid::Uuid::new_v4()));
        let data = source(&dir, pack["id"].as_str().unwrap()).await.unwrap();
        assert_eq!(sha(&data), pack["sha256"]);
        transform(&data, true, false).unwrap();
        assert_eq!(cache_read(&dir, &pack).unwrap(), data);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    #[ignore = "Uses the supplied PvP.zip collection"]
    fn supplied_collection_import_is_hash_matched_and_private() {
        let path = std::env::var("U3_PVP_ARCHIVE").unwrap();
        let dir = std::env::temp_dir()
            .join(format!("u3-packs-import-{}", uuid::Uuid::new_v4()));
        assert_eq!(import(&dir, Path::new(&path)).unwrap(), 14);
        for p in library().into_iter().filter(|p| p["category"] == "pvp") {
            assert!(cache_read(&dir, &p).is_ok());
        }
        assert!(!dir.join("game").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
