//! The U3 Fixes engine, compiled once by CI and shipped beside the launcher's executable.
//!
//! Horizon's `native-build` names a build after what it hashes (`native_mods::version_for`: the
//! horizon-mixins and Horizon revisions, the profile, the Rust version and the bundle's sources)
//! and reuses an installed version of that name instead of compiling. Before a build of exactly
//! the embedded U3 Fixes, the shipped engine is installed under that name the way Horizon installs
//! its own builds (a copy of the base version's folder with the engine in place of the base's), so
//! the first start skips downloading Rust and the 20-30 minute compile. Anything unexpected leaves
//! the local build in charge.
use super::*;

/// What Horizon launcher 0.2.28 (`RELEASE`) hashes besides the bundles (`native_mods::PROFILE`,
/// `FEATURES`, `rust_toolchain::VERSION`). Update them together with the runtime.
const PROFILE: &str = "release";
const FEATURES: &str = "horizon-client/mods";
pub(super) const RUST: &str = "1.99.1";
/// The engine's folder beside the launcher's executable (the installer's resources) and its facts.
const SHIPPED: &str = "u3-engine";
const INFO: &str = "engine.json";
const CLIENT: &str = "horizon-client.exe";
/// What Horizon's own install leaves out of the copied base version, plus this launcher's stamps,
/// which describe the base's engine rather than this one.
const NOT_COPIED: [&str; 6] = [
    CLIENT,
    ".installed",
    "Horizon.app",
    "native-mods.json",
    ".u3-engine-sha256",
    "u3-native-receipt.json",
];

fn revision(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Horizon's `version_for` for one bundle `id`, from its archive entries: the files under
/// `mixins/<id>/`, by their path below it in byte order, each as its name, its length (u64 LE) and
/// its bytes.
pub(super) fn native_version(
    revision_: &str,
    mixins: &str,
    id: &str,
    files: &[(&str, &str)],
) -> Result<String> {
    if !revision(revision_) || !revision(mixins) {
        return Err("Native build revisions must be exact commits".into());
    }
    let mut hash = Sha256::new();
    for part in [mixins, PROFILE, FEATURES, RUST, revision_] {
        hash.update(part.as_bytes());
        hash.update([0]);
    }
    hash.update(id.as_bytes());
    hash.update([0]);
    let prefix = format!("mixins/{id}/");
    let mut sources: Vec<(&str, &str)> = files
        .iter()
        .filter_map(|(name, text)| {
            name.strip_prefix(prefix.as_str()).map(|n| (n, *text))
        })
        .collect();
    if sources.is_empty() {
        return Err("The native bundle has no sources".into());
    }
    sources.sort();
    for (name, text) in sources {
        hash.update(name.as_bytes());
        hash.update((text.len() as u64).to_le_bytes());
        hash.update(text.as_bytes());
    }
    let key: String = hash.finalize()[..6]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(format!("native-{}-{key}", &revision_[..7]))
}

fn manifest() -> Result<Value> {
    let text = mods::FIXES
        .iter()
        .find(|(name, _)| *name == "horizon-native.json")
        .map(|(_, text)| *text)
        .ok_or("U3 Fixes has no manifest")?;
    serde_json::from_str(text).map_err(|e| e.to_string())
}

/// The version Horizon builds the embedded U3 Fixes as, with the facts its build records.
pub(super) fn fixes_version() -> Result<String> {
    let m = manifest()?;
    native_version(
        m["horizon"].as_str().unwrap_or_default(),
        m["mixins"].as_str().unwrap_or_default(),
        m["id"].as_str().ok_or("U3 Fixes has no id")?,
        mods::FIXES,
    )
}

#[derive(Debug)]
struct Shipped {
    exe: PathBuf,
    version: String,
    sha256: String,
}

/// The engine in `dir` when it is the build of the embedded U3 Fixes (`expected`), else why not.
fn shipped_in(dir: &Path, expected: &str, m: &Value) -> Result<Shipped> {
    let path = dir.join(INFO);
    let meta = std::fs::metadata(&path)
        .map_err(|_| "No prebuilt engine is shipped".to_string())?;
    if meta.len() > 64 * 1024 {
        return Err("Prebuilt engine facts exceed the size limit".into());
    }
    let info: Value = serde_json::from_slice(
        &std::fs::read(&path).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let version = info["version"]
        .as_str()
        .filter(|v| safe_component(v) && v.starts_with("native-"))
        .ok_or("Prebuilt engine has no valid version")?;
    if version != expected {
        return Err(format!(
            "Prebuilt engine {version} was made from other sources than {expected}"
        ));
    }
    let mods = json!([{"id": m["id"], "version": m["version"]}]);
    if info["horizon"] != m["horizon"]
        || info["horizon_mixins"] != m["mixins"]
        || info["rust"] != RUST
        || info["mods"] != mods
    {
        return Err("Prebuilt engine was built from other inputs".into());
    }
    let sha256 = info["sha256"]
        .as_str()
        .filter(|s| {
            s.len() == 64
                && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        })
        .ok_or("Prebuilt engine has no checksum")?;
    let exe = dir.join(CLIENT);
    if !std::fs::symlink_metadata(&exe).is_ok_and(|m| m.is_file()) {
        return Err("Prebuilt engine file is missing".into());
    }
    Ok(Shipped {
        exe,
        version: version.to_owned(),
        sha256: sha256.to_owned(),
    })
}

/// Copies `from` to `to`: folders whole, files as they are. Links are left out (Horizon's
/// version folders hold none, and a link must never pull outside files into a version).
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(from).map_err(|e| e.to_string())?;
    if meta.is_dir() {
        std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
        for entry in std::fs::read_dir(from).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else if meta.is_file() {
        std::fs::copy(from, to)
            .map_err(|e| format!("Could not copy {}: {e}", from.display()))?;
    }
    Ok(())
}

/// Installs `shipped` in `versions` on top of installed base version `base` (folder `base_dir`),
/// as Horizon's `native_mods::install` does. An installed version of that name is kept as it is:
/// Horizon reuses it just the same.
fn install_on(
    versions: &Path,
    base_dir: &Path,
    base: &str,
    shipped: &Shipped,
    m: &Value,
) -> Result<bool> {
    let target = versions.join(&shipped.version);
    if target.join(".installed").is_file() {
        return Ok(false);
    }
    let partial = versions.join(format!(".{}.partial", shipped.version));
    let _ = std::fs::remove_dir_all(&partial);
    std::fs::create_dir_all(&partial).map_err(|e| e.to_string())?;
    let result = (|| {
        for entry in std::fs::read_dir(base_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            if NOT_COPIED.iter().any(|n| name == *n) {
                continue;
            }
            copy_tree(&entry.path(), &partial.join(&name))?;
        }
        let exe = partial.join(CLIENT);
        std::fs::copy(&shipped.exe, &exe).map_err(|e| e.to_string())?;
        if engine_hash(&exe)? != shipped.sha256 {
            return Err("The prebuilt engine is damaged. Reinstall U3drinth"
                .to_string());
        }
        let about = json!({
            "base": base,
            "horizon": m["horizon"],
            "horizon_mixins": m["mixins"],
            "hooks": null,
            "rust": RUST,
            "mods": [{"id": m["id"], "version": m["version"]}],
        });
        std::fs::write(partial.join("native-mods.json"), about.to_string())
            .map_err(|e| e.to_string())?;
        let _ = std::fs::remove_dir_all(&target);
        std::fs::rename(&partial, &target).map_err(|e| e.to_string())?;
        std::fs::write(target.join(".installed"), "").map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&partial);
    }
    result.map(|()| true)
}

fn shipped_dir() -> Option<PathBuf> {
    Some(std::env::current_exe().ok()?.parent()?.join(SHIPPED))
}

/// Whether this installation ships the engine of the embedded U3 Fixes.
pub(super) fn available() -> bool {
    let (Some(dir), Ok(m), Ok(expected)) =
        (shipped_dir(), manifest(), fixes_version())
    else {
        return false;
    };
    shipped_in(&dir, &expected, &m).is_ok()
}

fn prepare_in(dir: &Path, root: &Path, base: &str) -> Result<Option<String>> {
    if !dir.join(INFO).is_file() {
        return Ok(None);
    }
    if base.starts_with("native-") {
        return Err("The base version is itself a native build".into());
    }
    let m = manifest()?;
    let shipped = shipped_in(dir, &fixes_version()?, &m)?;
    let base_dir = version_dir(root, base)?;
    let versions = base_dir.parent().ok_or("Invalid Horizon version folder")?;
    install_on(versions, &base_dir, base, &shipped, &m)?;
    Ok(Some(shipped.version))
}

/// Before `native-build` of exactly the embedded U3 Fixes on `base`: puts the shipped engine in
/// place as the version that build would make. Some(version) when it is there now.
pub(super) async fn prepare(root: &Path, base: &str) -> Result<Option<String>> {
    let Some(dir) = shipped_dir() else {
        return Ok(None);
    };
    let (root, base) = (root.to_owned(), base.to_owned());
    tokio::task::spawn_blocking(move || prepare_in(&dir, &root, &base))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped(dir: &Path, version: &str, exe: &[u8]) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(CLIENT), exe).unwrap();
        let m = manifest().unwrap();
        std::fs::write(
            dir.join(INFO),
            json!({
                "version": version,
                "sha256": format!("{:x}", Sha256::digest(exe)),
                "horizon": m["horizon"],
                "horizon_mixins": m["mixins"],
                "rust": RUST,
                "mods": [{"id": "u3_fixes", "version": mods::FIXES_VERSION}],
            })
            .to_string(),
        )
        .unwrap();
    }

    fn base(root: &Path) -> PathBuf {
        let dir = root.join("launcher/versions/v0.2.97");
        let resources = dir.join(".local/horizon-client/26.3");
        std::fs::create_dir_all(resources.join("assets")).unwrap();
        std::fs::write(resources.join("horizon-resources.json"), "{}").unwrap();
        std::fs::write(resources.join("assets/sounds.json"), "{}").unwrap();
        std::fs::write(
            dir.join("research-lock.json"),
            r#"{"minecraft":{"version":"26.3"}}"#,
        )
        .unwrap();
        std::fs::write(dir.join(CLIENT), b"MZ base engine").unwrap();
        std::fs::write(dir.join(".u3-engine-sha256"), "a".repeat(64)).unwrap();
        std::fs::write(dir.join(".installed"), "").unwrap();
        dir
    }

    #[test]
    fn version_matches_horizons_own_hash() {
        // Horizon 0.2.28's version_for over a two-file bundle, computed independently.
        let files = [
            ("horizon-native.json", "{}"),
            ("mixins/demo/horizon-mixin.toml", "[mod]\nid = \"demo\"\n"),
            ("mixins/demo/a.rs", "fn a() {}\n"),
            ("other/ignored.rs", "fn b() {}\n"),
        ];
        assert_eq!(
            native_version(&"1".repeat(40), &"2".repeat(40), "demo", &files)
                .unwrap(),
            "native-1111111-750aade901bd"
        );
        assert!(
            native_version("main", &"2".repeat(40), "demo", &files).is_err()
        );
        assert!(native_version(
            &"1".repeat(40),
            &"2".repeat(40),
            "none",
            &files
        )
        .is_err());
    }

    #[test]
    fn embedded_fixes_build_as_the_pinned_version() {
        // Changing a U3 Fixes source changes this name: CI then builds and ships a new engine.
        assert_eq!(fixes_version().unwrap(), "native-3132240-82fef81db81e");
        let m = manifest().unwrap();
        assert_eq!(m["version"], mods::FIXES_VERSION);
    }

    #[test]
    fn shipped_engine_installs_as_horizons_own_build() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        let base_dir = base(&root);
        let engine = tmp.path().join("u3-engine");
        let version = fixes_version().unwrap();
        shipped(&engine, &version, b"MZ prebuilt engine");
        assert_eq!(
            prepare_in(&engine, &root, "v0.2.97").unwrap().as_deref(),
            Some(version.as_str())
        );
        let dir = root.join("launcher/versions").join(&version);
        assert!(dir.join(".installed").is_file());
        assert_eq!(
            std::fs::read(dir.join(CLIENT)).unwrap(),
            b"MZ prebuilt engine"
        );
        assert!(dir
            .join(".local/horizon-client/26.3/horizon-resources.json")
            .is_file());
        assert!(dir
            .join(".local/horizon-client/26.3/assets/sounds.json")
            .is_file());
        assert!(!dir.join(".u3-engine-sha256").exists());
        let about: Value = serde_json::from_slice(
            &std::fs::read(dir.join("native-mods.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(about["base"], "v0.2.97");
        assert_eq!(about["mods"], json!([{"id":"u3_fixes","version":"0.3.0"}]));
        assert_eq!(about["horizon"], manifest().unwrap()["horizon"]);
        // The base stays untouched and a second run keeps the installed build.
        assert_eq!(
            std::fs::read(base_dir.join(CLIENT)).unwrap(),
            b"MZ base engine"
        );
        std::fs::write(dir.join("marker"), "kept").unwrap();
        assert!(prepare_in(&engine, &root, "v0.2.97").unwrap().is_some());
        assert!(dir.join("marker").is_file());
        assert!(!root
            .join("launcher/versions")
            .join(format!(".{version}.partial"))
            .exists());
    }

    #[test]
    fn mismatched_or_damaged_engines_are_never_installed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        base(&root);
        let versions = root.join("launcher/versions");
        let version = fixes_version().unwrap();
        // No engine shipped: nothing to do.
        assert_eq!(
            prepare_in(&tmp.path().join("none"), &root, "v0.2.97").unwrap(),
            None
        );
        // Built from other sources.
        let other = tmp.path().join("other");
        shipped(&other, "native-3132240-000000000000", b"MZ other");
        assert!(prepare_in(&other, &root, "v0.2.97").is_err());
        // Damaged after it was built.
        let damaged = tmp.path().join("damaged");
        shipped(&damaged, &version, b"MZ prebuilt engine");
        std::fs::write(damaged.join(CLIENT), b"MZ changed").unwrap();
        assert!(prepare_in(&damaged, &root, "v0.2.97").is_err());
        assert!(!versions.join(&version).exists());
        assert!(!versions.join(format!(".{version}.partial")).exists());
        // Never on a native build or a version that is not installed.
        let good = tmp.path().join("good");
        shipped(&good, &version, b"MZ prebuilt engine");
        assert!(
            prepare_in(&good, &root, "native-3132240-000000000000").is_err()
        );
        assert!(prepare_in(&good, &root, "v9.9.9").is_err());
        assert!(!versions.join(&version).exists());
    }

    #[test]
    fn shipped_engine_from_ci_matches_the_embedded_fixes() {
        // CI points this at the engine it built with Horizon's own native-build.
        let Ok(path) = std::env::var("U3_ENGINE_DIR") else {
            return;
        };
        let m = manifest().unwrap();
        let shipped =
            shipped_in(Path::new(&path), &fixes_version().unwrap(), &m).expect(
                "the CI engine must be the build of the embedded U3 Fixes",
            );
        assert_eq!(engine_hash(&shipped.exe).unwrap(), shipped.sha256);
    }
}
