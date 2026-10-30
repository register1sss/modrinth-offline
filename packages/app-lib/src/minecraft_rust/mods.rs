//! Fixed official registry, checksum-bound native-code consent and verified build receipts.
use super::*;
use std::collections::BTreeMap;
use std::io::{Cursor, Write};
const INDEX: &str = "https://horizon-rust.github.io/horizon-mods/index.json";
const MIXINS: &str = "0d6b103c70633d8fd9443d8d0acd3aadaacedb06";
const LIMIT: u64 = 16 * 1024 * 1024;
fn digest(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
fn hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub(super) fn validate_request(r: &Request) -> Result<()> {
    if !id(&r.project)
        || !hash(&r.checksum)
        || !hash(&r.trust_token)
        || !safe_component(&r.profile)
        || r.profile.len() > 100
    {
        return Err("Invalid mod request. Reload the catalog".into());
    }
    if !r.trust_native_code {
        return Err("Native code requires explicit permission".into());
    }
    Ok(())
}
fn read_json(path: &Path) -> Result<Value> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > LIMIT {
        return Err("Mod metadata exceeds size limit".into());
    }
    serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("Missing mod folder")?;
    let mut f =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    f.write_all(bytes).map_err(|e| e.to_string())?;
    f.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
fn embedded() -> Result<Vec<u8>> {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opt = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    for (name, content) in [
        (
            "horizon-native.json",
            include_str!("u3_edit/horizon-native.json"),
        ),
        (
            "mixins/u3_edit/horizon-mixin.toml",
            include_str!("u3_edit/horizon-mixin.toml"),
        ),
        (
            "mixins/u3_edit/runtime.rs",
            include_str!("u3_edit/runtime.rs"),
        ),
        (
            "mixins/u3_edit/command.rs",
            include_str!("u3_edit/command.rs"),
        ),
        ("mixins/u3_edit/frame.rs", include_str!("u3_edit/frame.rs")),
        ("mixins/u3_edit/list.rs", include_str!("u3_edit/list.rs")),
    ] {
        z.start_file(name, opt).map_err(|e| e.to_string())?;
        z.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(z.finish().map_err(|e| e.to_string())?.into_inner())
}
async fn index(root: &Path) -> Result<Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())?;
    let mut response = client
        .get(INDEX)
        .send()
        .await
        .map_err(|e| format!("Catalog unavailable: {e}"))?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    while let Some(b) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + b.len() > LIMIT as usize {
            return Err("Catalog exceeds size limit".into());
        }
        bytes.extend_from_slice(&b);
    }
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if v["schema"] != 1
        || v["projects"].as_array().map_or(true, |a| a.len() > 200)
        || !v["revoked"].is_array()
    {
        return Err("Unsupported Horizon catalog".into());
    }
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    atomic(&root.join("u3-mod-index.json"), &bytes)?;
    Ok(v)
}
fn revoked(index: &Value, project: &str, checksum: &str) -> bool {
    index["revoked"].as_array().is_some_and(|all| {
        all.iter().any(|v| {
            v["id"].as_str() == Some(project)
                && (v["sha256"].as_str() == Some(checksum)
                    || v["sha256"].is_null())
        })
    })
}
fn entries(index: &Value) -> Vec<Value> {
    index["projects"].as_array().into_iter().flatten().filter_map(|p| {
        let project = p["id"].as_str()?;
        if !id(project) || project == "u3_edit" { return None; }
        let v = p["versions"].as_array()?.first()?;
        let checksum = v["file"]["sha256"].as_str()?;
        if !hash(checksum) { return None; }
        let api = v["api"].as_str().unwrap_or("");
        let native = p["kind"] == "native-mod" && api.starts_with("native-") && api.ends_with(MIXINS);
        let no_deps = v["dependencies"].as_array().map_or(v["dependencies"].is_null(), |a| a.is_empty());
        let available = native && no_deps && v["game"] == "26.3" && !revoked(index, project, checksum);
        Some(json!({"id":project,"name":p["name"],"authors":p["authors"],"summary":p["summary"],"source":p["source"],"version":v["version"],"checksum":checksum,"available":available,"native":native,"reason":if available { "" } else { "Эта версия пока не поддерживается" }}))
    }).collect()
}
fn profile_home(
    root: &Path,
    instances: &Value,
    profile: &str,
) -> Result<PathBuf> {
    let p = instances
        .as_array()
        .and_then(|a| a.iter().find(|p| p["id"] == profile))
        .ok_or("Choose a Horizon profile")?;
    if !p["java"].is_null()
        || !matches!(p["kind"].as_str(), Some("own" | "default"))
    {
        return Err("Only local Horizon profiles support native mods".into());
    }
    let base =
        std::fs::canonicalize(root.join("game")).map_err(|e| e.to_string())?;
    let path = std::fs::canonicalize(
        p["home"].as_str().ok_or("Missing profile home")?,
    )
    .map_err(|e| e.to_string())?;
    if !path.starts_with(&base) {
        return Err("Profile is outside Minecraft Rust".into());
    }
    Ok(path)
}
fn target(instances: &Value, profile: &str) -> String {
    if profile.is_empty() || profile == "horizon" {
        instances
            .as_array()
            .into_iter()
            .flatten()
            .find(|p| {
                p["name"] == "U3 Mods"
                    && p["kind"] == "own"
                    && p["java"].is_null()
            })
            .and_then(|p| p["id"].as_str())
            .unwrap_or("")
            .to_owned()
    } else {
        profile.to_owned()
    }
}
fn bundles(home: &Path) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    let dir = home.join("horizon-native-mods");
    if !dir.exists() {
        return Ok(files);
    }
    let real = std::fs::canonicalize(&dir).map_err(|e| e.to_string())?;
    if !real.starts_with(home) {
        return Err("Native mod folder is outside the profile".into());
    }
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|v| v.to_str()) != Some("hnmod") {
            continue;
        }
        if files.len() >= 100 {
            return Err("Too many native mods".into());
        }
        let path =
            std::fs::canonicalize(entry.path()).map_err(|e| e.to_string())?;
        if path.parent() != Some(real.as_path())
            || std::fs::metadata(&path).map_err(|e| e.to_string())?.len()
                > 64 * 1024 * 1024
        {
            return Err("Invalid native mod file".into());
        }
        let name = entry
            .file_name()
            .to_str()
            .ok_or("Invalid mod filename")?
            .to_owned();
        files.insert(
            name,
            digest(&std::fs::read(path).map_err(|e| e.to_string())?),
        );
    }
    Ok(files)
}
fn token(profile: &str, files: &BTreeMap<String, String>) -> String {
    digest(&serde_json::to_vec(&(profile, files)).unwrap_or_default())
}
fn settings_hash(home: &Path) -> Result<String> {
    match std::fs::read(home.join("horizon-mods.txt")) {
        Ok(bytes) if bytes.len() <= LIMIT as usize => Ok(digest(&bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(digest(b"")),
        _ => Err("Cannot read native mod settings".into()),
    }
}
fn enabled_inputs(
    home: &Path,
    files: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    use std::io::Read;
    let settings = std::fs::read_to_string(home.join("horizon-mods.txt"))
        .map_err(|e| e.to_string())?;
    let mut result = BTreeMap::new();
    for name in files.keys() {
        let file =
            std::fs::File::open(home.join("horizon-native-mods").join(name))
                .map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        zip.by_name("horizon-native.json")
            .map_err(|e| e.to_string())?
            .take(65537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65536 {
            return Err("Native manifest exceeds size limit".into());
        }
        let manifest: Value =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let id = manifest["id"]
            .as_str()
            .filter(|s| id(s))
            .ok_or("Invalid native mod ID")?;
        let version = manifest["version"]
            .as_str()
            .ok_or("Missing native mod version")?;
        if settings
            .lines()
            .filter_map(|l| l.split_once(':'))
            .any(|(k, v)| k.trim() == format!("mod.{id}") && v.trim() == "true")
        {
            if result.insert(id.to_owned(), version.to_owned()).is_some() {
                return Err("Duplicate native mod ID".into());
            }
        }
    }
    Ok(result)
}
fn ready(
    root: &Path,
    instances: &Value,
    profile: &str,
    files: &BTreeMap<String, String>,
) -> bool {
    let Some(p) = instances
        .as_array()
        .into_iter()
        .flatten()
        .find(|p| p["id"] == profile)
    else {
        return false;
    };
    let Some(tag) = p["version"].as_str() else {
        return false;
    };
    version_dir(root, tag)
        .ok()
        .and_then(|d| read_json(&d.join("u3-native-receipt.json")).ok())
        .is_some_and(|r| {
            r["files"] == json!(files)
                && profile_home(root, instances, profile)
                    .ok()
                    .and_then(|h| settings_hash(&h).ok())
                    .is_some_and(|hash| r["settings"] == hash)
                && checked_engine(root, tag).is_ok()
        })
}
async fn make_catalog(
    root: &Path,
    profile: &str,
    idx: &Value,
) -> Result<Value> {
    let instances = if verified(root).await.is_ok() {
        query(root, &["--json", "instance", "list"]).await?
    } else {
        json!([])
    };
    let target = target(&instances, profile);
    let files = if target.is_empty() {
        BTreeMap::new()
    } else {
        bundles(&profile_home(root, &instances, &target)?)?
    };
    let bytes = embedded()?;
    let own = json!({"id":"u3_edit","name":"U3 Edit","authors":["U3"],"source":"https://github.com/register1sss/modrinth-offline/tree/main/packages/app-lib/src/minecraft_rust/u3_edit","summary":"Выделение, заливка, замена блоков и стены. До 32768 блоков. Без отмены изменений.","version":"0.1.0","checksum":digest(&bytes),"available":true,"native":true});
    Ok(
        json!({"own":[own],"community":entries(idx),"target":target,"target_name":if profile.is_empty() || profile == "horizon" { "U3 Mods".to_owned() } else { instances.as_array().into_iter().flatten().find(|p| p["id"] == profile).and_then(|p| p["name"].as_str()).unwrap_or(profile).to_owned() },"trust_token":token(profile,&files),"files":files,"ready":ready(root,&instances,&target,&files)}),
    )
}
pub async fn catalog(profile: &str) -> Result<Value> {
    if !profile.is_empty() && (!safe_component(profile) || profile.len() > 100)
    {
        return Err("Invalid profile".into());
    }
    let root = home().await?;
    let idx = index(&root).await?;
    make_catalog(&root, profile, &idx).await
}
pub(super) async fn install(root: &Path, r: &Request) -> Result<()> {
    validate_request(r)?;
    let idx = index(root).await?;
    let cat = make_catalog(root, &r.profile, &idx).await?;
    let entry = cat["own"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(cat["community"].as_array().into_iter().flatten())
        .find(|e| e["id"] == r.project)
        .ok_or("Mod not found")?;
    if entry["available"] != true
        || entry["checksum"] != r.checksum
        || cat["trust_token"] != r.trust_token
    {
        return Err(
            "The catalog or installed mods changed. Reload and confirm again"
                .into(),
        );
    }
    if verified(root).await.is_err() {
        install_runtime_and_game(root, "latest").await?;
    }
    let mut instances = query(root, &["--json", "instance", "list"]).await?;
    let mut target_id = target(&instances, &r.profile);
    if target_id.is_empty() {
        cli(root, &["instance", "create", "--", "U3 Mods"], false).await?;
        instances = query(root, &["--json", "instance", "list"]).await?;
        target_id = target(&instances, &r.profile);
    }
    let home = profile_home(root, &instances, &target_id)?;
    let before = bundles(&home)?;
    if token(&r.profile, &before) != r.trust_token {
        return Err("Installed mods changed. Confirm again".into());
    }
    for (file, checksum) in &before {
        if revoked(&idx, file.trim_end_matches(".hnmod"), checksum) {
            return Err(
                "A native mod was revoked. Remove it before rebuilding".into(),
            );
        }
    }
    let mut versions =
        query(root, &["--json", "versions", "--installed"]).await?;
    let mut base = launch_version(&instances, &versions, &target_id)?;
    if let Some(tag) = base.as_ref().filter(|t| t.starts_with("native-")) {
        let info =
            read_json(&version_dir(root, tag)?.join("native-mods.json"))?;
        base = Some(
            info["base"]
                .as_str()
                .filter(|b| safe_component(b) && !b.starts_with("native-"))
                .ok_or("Missing original Horizon version")?
                .into(),
        );
    }
    if needs_game_install(root, &versions, base.as_deref()) {
        install_runtime_and_game(root, base.as_deref().unwrap_or("latest"))
            .await?;
        versions = query(root, &["--json", "versions", "--installed"]).await?;
    }
    let base = base
        .or_else(|| versions["selected"].as_str().map(str::to_owned))
        .ok_or("Install Horizon first")?;
    let source = State::get()
        .await
        .map_err(|e| e.to_string())?
        .directories
        .config_dir
        .clone();
    prepare_automatically(root, &source, &base).await?;
    change(|s| s.stage = "mod_install".into());
    // Receipt invalidation prevents a stale engine from masquerading as this mod set after failure.
    atomic(&home.join("u3-mods-pending"), b"build required")?;
    if r.project == "u3_edit" {
        let dir = home.join("horizon-native-mods");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        if let Some(old) = before.get("u3_edit.hnmod") {
            let receipt = read_json(&home.join("u3-edit-installed.json"))?;
            if receipt["checksum"] != *old {
                return Err("U3 Edit file was changed outside the launcher. Back it up and remove it before reinstalling".into());
            }
        }
        atomic(&dir.join("u3_edit.hnmod"), &embedded()?)?;
        atomic(
            &home.join("u3-edit-installed.json"),
            json!({"checksum":r.checksum}).to_string().as_bytes(),
        )?;
    } else {
        cli(
            root,
            &[
                "mods",
                "install",
                "--source",
                "horizon",
                &r.project,
                "--instance",
                &target_id,
            ],
            false,
        )
        .await?;
    }
    let after = bundles(&home)?;
    let expected_file = format!("{}.hnmod", r.project);
    if after.get(&expected_file) != Some(&r.checksum) {
        return Err("Installed mod checksum does not match your consent".into());
    }
    let mut expected = before;
    expected.insert(expected_file, r.checksum.clone());
    if after != expected {
        return Err("Additional native code appeared. Review the catalog before building".into());
    }
    if r.project == "u3_edit" {
        let path = home.join("horizon-mods.txt");
        let text = match std::fs::read_to_string(&path) {
            Ok(t) if t.len() <= LIMIT as usize => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            _ => return Err("Cannot read mod settings".into()),
        };
        let mut lines: Vec<&str> = text
            .lines()
            .filter(|l| !l.starts_with("mod.u3_edit:"))
            .collect();
        lines.push("mod.u3_edit:true");
        atomic(&path, format!("{}\n", lines.join("\n")).as_bytes())?;
    } else {
        cli(
            root,
            &["mods", "enable", &r.project, "--instance", &target_id],
            false,
        )
        .await?;
    }
    let inputs = enabled_inputs(&home, &after)?;
    let settings = settings_hash(&home)?;
    change(|s| {
        s.stage = "build_native".into();
        s.total = 0;
    });
    cli(
        root,
        &[
            "native-build",
            "--instance",
            &target_id,
            "--version",
            &base,
            "--trust-native-code",
        ],
        false,
    )
    .await?;
    if bundles(&home)? != after || settings_hash(&home)? != settings {
        return Err("Native mod files changed during the build".into());
    }
    instances = query(root, &["--json", "instance", "list"]).await?;
    versions = query(root, &["--json", "versions", "--installed"]).await?;
    let tag = launch_version(&instances, &versions, &target_id)?
        .filter(|t| t.starts_with("native-"))
        .ok_or("Horizon did not select the native build")?;
    let dir = version_dir(root, &tag)?;
    let info = read_json(&dir.join("native-mods.json"))?;
    if info["base"] != base
        || !info["mods"].as_array().is_some_and(|a| {
            a.iter().any(|m| {
                m["id"] == r.project && m["version"] == entry["version"]
            })
        })
    {
        return Err("Native build does not contain the requested mod".into());
    }
    let built: BTreeMap<String, String> = info["mods"]
        .as_array()
        .ok_or("Missing native build inputs")?
        .iter()
        .map(|m| {
            Ok((
                m["id"].as_str().ok_or("Missing built mod ID")?.to_owned(),
                m["version"]
                    .as_str()
                    .ok_or("Missing built mod version")?
                    .to_owned(),
            ))
        })
        .collect::<Result<_>>()?;
    if built != inputs {
        return Err("Native build inputs do not match the enabled mods".into());
    }
    stamp_installed_engine(root, &tag)?;
    atomic(
        &dir.join("u3-native-receipt.json"),
        json!({"files":after,"settings":settings,"profile":target_id,"base":base})
            .to_string()
            .as_bytes(),
    )?;
    std::fs::remove_file(home.join("u3-mods-pending"))
        .map_err(|e| e.to_string())?;
    change(|s| s.result_profile = Some(target_id));
    Ok(())
}
pub(super) fn check_build(
    root: &Path,
    instances: &Value,
    profile: &str,
) -> Result<()> {
    let home = profile_home(root, instances, profile)?;
    if home.join("u3-mods-pending").exists() {
        return Err("Native mod installation is unfinished. Reinstall the mod to finish building".into());
    }
    let files = bundles(&home)?;
    if files.is_empty() {
        return Ok(());
    }
    if !ready(root, instances, profile, &files) {
        return Err("Native mod files changed. Reinstall a mod and confirm the build again".into());
    }
    if let Ok(idx) = read_json(&root.join("u3-mod-index.json")) {
        for (name, hash) in files {
            if revoked(&idx, name.trim_end_matches(".hnmod"), &hash) {
                return Err("A native mod has been revoked. Remove it using Horizon and rebuild".into());
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_bundle_is_deterministic_source_only() {
        let bytes = embedded().unwrap();
        assert_eq!(bytes, embedded().unwrap());
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(zip.len(), 6);
        let mut manifest = String::new();
        std::io::Read::read_to_string(
            &mut zip.by_name("horizon-native.json").unwrap(),
            &mut manifest,
        )
        .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&manifest).unwrap()["id"],
            "u3_edit"
        );
    }
    #[test]
    fn consent_binds_all_files_and_profile() {
        let mut files = BTreeMap::new();
        let initial = token("horizon", &files);
        files.insert("zoom.hnmod".into(), "a".repeat(64));
        assert_ne!(initial, token("horizon", &files));
        assert_ne!(token("own", &files), token("horizon", &files));
    }
    #[test]
    fn native_consent_is_required_before_any_install() {
        let mut r: Request = serde_json::from_value(json!({"action":"mod_install","project":"u3_edit","profile":"horizon","checksum":"a".repeat(64),"trust_token":"b".repeat(64)})).unwrap();
        assert!(validate_request(&r).is_err());
        r.trust_native_code = true;
        assert!(validate_request(&r).is_ok());
    }
    #[test]
    fn identifiers_are_not_paths() {
        for s in ["../zoom", "--help", "Zoom", "", "a/b"] {
            assert!(!id(s));
        }
        assert!(id("u3_edit"));
    }
    #[test]
    fn wasm_and_revoked_entries_cannot_install() {
        let mut idx = json!({"projects":[{"id":"zoom","kind":"mod","name":"Zoom","versions":[{"api":"0.1","game":"26.3","file":{"sha256":"a".repeat(64)}}]}],"revoked":[]});
        assert_eq!(entries(&idx)[0]["available"], false);
        idx["projects"][0]["kind"] = json!("native-mod");
        idx["projects"][0]["versions"][0]["api"] =
            json!(format!("native-1/revision/{MIXINS}"));
        assert_eq!(entries(&idx)[0]["available"], true);
        idx["revoked"] = json!([{"id":"zoom","sha256":"a".repeat(64)}]);
        assert_eq!(entries(&idx)[0]["available"], false);
    }
}
