//! U3drinth's source: a folder or a .zip, found even inside one or two wrapping folders, read
//! for its version, and mirrored into the builder's work folder, where dependencies and build
//! results from earlier builds are kept so the next build only redoes what changed.

use crate::ui::Ui;
use crate::util;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Never copied from the source nor removed from the work folder: build products.
const SKIP_NAMES: [&str; 4] = ["node_modules", "target", ".git", ".turbo"];
/// Written by the builder into the work folder.
pub const ENGINE_DIR: &str = "apps/app/u3-engine";
pub const BUILDER_CONFIG: &str = "apps/app/tauri.u3-builder.conf.json";

pub struct Source {
    pub root: PathBuf,
    pub product: String,
    pub version: String,
    pub pnpm: Option<String>,
}

pub fn is_root(dir: &Path) -> bool {
    dir.join("apps/app/tauri.conf.json").is_file()
        && dir.join("packages/app-lib/Cargo.toml").is_file()
        && dir.join("Cargo.toml").is_file()
        && dir.join("pnpm-lock.yaml").is_file()
}

/// The source root: the folder itself or a folder up to two levels inside it.
pub fn find_root(dir: &Path) -> Option<PathBuf> {
    let mut level = vec![dir.to_path_buf()];
    for _ in 0..3 {
        let mut next = Vec::new();
        for d in level {
            if is_root(&d) {
                return Some(d);
            }
            if let Ok(entries) = fs::read_dir(&d) {
                let mut dirs: Vec<PathBuf> = entries
                    .flatten()
                    .filter(|e| {
                        e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                    })
                    .map(|e| e.path())
                    .filter(|p| !skipped_name(p))
                    .collect();
                dirs.sort();
                next.extend(dirs);
            }
        }
        level = next;
    }
    None
}

fn skipped_name(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|n| SKIP_NAMES.iter().any(|s| n == *s))
}

/// Opens the dropped folder or .zip (unpacked into `incoming`).
pub fn open(ui: &Ui, input: &Path, incoming: &Path) -> Result<Source, String> {
    let dir = if input.is_file() {
        let zip = input
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("zip"));
        if !zip {
            return Err(format!(
                "{}: нужна папка или .zip с исходниками U3drinth",
                input.display()
            ));
        }
        util::remove_tree(incoming)
            .map_err(|e| format!("{}: {e}", incoming.display()))?;
        ui.status("      Распаковка архива…");
        extract_zip(input, incoming)?;
        ui.clear_status();
        incoming.to_path_buf()
    } else if input.is_dir() {
        input.to_path_buf()
    } else {
        return Err(format!("{}: такого файла или папки нет", input.display()));
    };
    let root = find_root(&dir).ok_or_else(|| {
        format!("в {} нет исходников U3drinth (не найдены apps/app/tauri.conf.json, packages/app-lib, Cargo.toml и pnpm-lock.yaml)", input.display())
    })?;
    let (product, version) = app_version(&root)?;
    Ok(Source {
        pnpm: pnpm_version(&root),
        root,
        product,
        version,
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// productName and version from apps/app/tauri.conf.json (version may point to a package.json).
pub fn app_version(root: &Path) -> Result<(String, String), String> {
    let conf_path = root.join("apps/app/tauri.conf.json");
    let conf = read_json(&conf_path)?;
    let package = &conf["package"];
    let product = package["productName"]
        .as_str()
        .unwrap_or("U3drinth")
        .to_string();
    let mut version =
        package["version"].as_str().unwrap_or("1.0.0").to_string();
    if version.ends_with(".json") {
        let referenced = root.join("apps/app").join(&version);
        version = read_json(&referenced)?["version"]
            .as_str()
            .unwrap_or("1.0.0")
            .to_string();
    }
    Ok((product, version))
}

/// The pnpm version in package.json's packageManager («pnpm@9.4.0+sha512…»).
pub fn pnpm_version(root: &Path) -> Option<String> {
    let manifest = read_json(&root.join("package.json")).ok()?;
    let spec = manifest["packageManager"].as_str()?;
    let version = spec.strip_prefix("pnpm@")?.split('+').next()?.trim();
    valid_version(version).then(|| version.to_string())
}

/// Three numbers, as Windows installers need: 1.1.1.
pub fn valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.len() <= 5
                && p.bytes().all(|b| b.is_ascii_digit())
                && p.parse::<u32>().is_ok_and(|n| n <= 65535)
        })
}

/// The installer's file name: «U3drinth1.1.1.exe».
pub fn installer_name(product: &str, version: &str) -> String {
    let product: String = product
        .chars()
        .filter(|c| {
            !matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
                && !c.is_control()
        })
        .collect();
    let product = product.trim().trim_end_matches('.');
    let product = if product.is_empty() {
        "U3drinth"
    } else {
        product
    };
    format!("{product}{version}.exe")
}

pub fn extract_zip(zip: &Path, dest: &Path) -> Result<(), String> {
    let file =
        File::open(zip).map_err(|e| format!("{}: {e}", zip.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("{}: не zip-архив: {e}", zip.display()))?;
    fs::create_dir_all(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("{}: {e}", zip.display()))?;
        let Some(rel) = entry.enclosed_name() else {
            continue;
        };
        let out = dest.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&out)
                .map_err(|e| format!("{}: {e}", out.display()))?;
            continue;
        }
        if let Some(dir) = out.parent() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let mut target = File::create(&out)
            .map_err(|e| format!("{}: {e}", out.display()))?;
        io::copy(&mut entry, &mut target)
            .map_err(|e| format!("{}: {e}", out.display()))?;
    }
    Ok(())
}

fn rel_key(rel: &Path) -> String {
    rel.to_string_lossy().replace('\\', "/")
}

fn kept(rel: &Path) -> bool {
    let key = rel_key(rel);
    rel.components()
        .any(|c| SKIP_NAMES.iter().any(|s| c.as_os_str() == *s))
        || key == ENGINE_DIR
        || key.starts_with(&format!("{ENGINE_DIR}/"))
        || key == BUILDER_CONFIG
}

/// Source files below `root`, relative, without build products.
fn files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        for entry in fs::read_dir(root.join(&rel))? {
            let entry = entry?;
            let child = rel.join(entry.file_name());
            if kept(&child) {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_dir() {
                stack.push(child);
            } else if kind.is_file() {
                out.push(child);
            }
        }
    }
    Ok(out)
}

fn same(a: &Path, b: &Path) -> bool {
    let (Ok(ma), Ok(mb)) = (fs::metadata(a), fs::metadata(b)) else {
        return false;
    };
    if ma.len() != mb.len() || !mb.is_file() {
        return false;
    }
    let (Ok(mut fa), Ok(mut fb)) = (File::open(a), File::open(b)) else {
        return false;
    };
    let mut ba = vec![0u8; 64 * 1024];
    let mut bb = vec![0u8; 64 * 1024];
    loop {
        let n = match fa.read(&mut ba) {
            Ok(n) => n,
            Err(_) => return false,
        };
        if n == 0 {
            return true;
        }
        if fb.read_exact(&mut bb[..n]).is_err() || ba[..n] != bb[..n] {
            return false;
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Synced {
    pub files: usize,
    pub copied: usize,
    pub removed: usize,
}

/// Makes `work` hold exactly the source files of `from`, keeping build products. Changed files
/// get the current time, so Cargo rebuilds them even when the new copy is older.
pub fn sync(from: &Path, work: &Path) -> Result<Synced, String> {
    let err = |p: &Path, e: io::Error| format!("{}: {e}", p.display());
    fs::create_dir_all(work).map_err(|e| err(work, e))?;
    let source = files(from).map_err(|e| err(from, e))?;
    let mut stats = Synced {
        files: source.len(),
        ..Synced::default()
    };
    let now = SystemTime::now();
    for rel in &source {
        let (src, dst) = (from.join(rel), work.join(rel));
        if same(&src, &dst) {
            continue;
        }
        if let Some(dir) = dst.parent() {
            if dir.is_file() {
                fs::remove_file(dir).map_err(|e| err(dir, e))?;
            }
            fs::create_dir_all(dir).map_err(|e| err(dir, e))?;
        }
        if dst.is_dir() {
            util::remove_tree(&dst).map_err(|e| err(&dst, e))?;
        } else if dst.exists() {
            util::writable(&dst);
        }
        fs::copy(&src, &dst).map_err(|e| err(&src, e))?;
        util::writable(&dst);
        if let Ok(file) = fs::OpenOptions::new().write(true).open(&dst) {
            let _ = file.set_modified(now);
        }
        stats.copied += 1;
    }
    let wanted: HashSet<String> = source.iter().map(|r| rel_key(r)).collect();
    for rel in files(work).map_err(|e| err(work, e))? {
        if !wanted.contains(&rel_key(&rel)) {
            let path = work.join(&rel);
            util::writable(&path);
            fs::remove_file(&path).map_err(|e| err(&path, e))?;
            stats.removed += 1;
        }
    }
    remove_empty_dirs(work, Path::new(""));
    Ok(stats)
}

fn remove_empty_dirs(root: &Path, rel: &Path) -> bool {
    let dir = root.join(rel);
    let Ok(entries) = fs::read_dir(&dir) else {
        return false;
    };
    let mut empty = true;
    for entry in entries.flatten() {
        let child = rel.join(entry.file_name());
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if kept(&child) || !is_dir || !remove_empty_dirs(root, &child) {
            empty = false;
        }
    }
    if empty && !rel.as_os_str().is_empty() {
        return fs::remove_dir(&dir).is_ok();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("u3-builder-test-{name}-{}", std::process::id()));
        let _ = util::remove_tree(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn fake_source(root: &Path, version: &str) {
        write(
            &root.join("apps/app/tauri.conf.json"),
            &format!(
                r#"{{"package":{{"productName":"U3drinth","version":"{version}"}}}}"#
            ),
        );
        write(&root.join("packages/app-lib/Cargo.toml"), "[package]");
        write(&root.join("Cargo.toml"), "[workspace]");
        write(&root.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'");
        write(
            &root.join("package.json"),
            r#"{"packageManager":"pnpm@9.4.0+sha512.abc"}"#,
        );
    }

    #[test]
    fn versions_are_three_numbers() {
        for good in ["1.0.0", "1.1.1", "10.20.65535"] {
            assert!(valid_version(good), "{good}");
        }
        for bad in [
            "",
            "1.1",
            "1.1.1.1",
            "1.1.x",
            "v1.1.1",
            "1.1.1-beta",
            "1.1.65536",
            "1..1",
        ] {
            assert!(!valid_version(bad), "{bad}");
        }
        assert_eq!(installer_name("U3drinth", "1.1.1"), "U3drinth1.1.1.exe");
        assert_eq!(installer_name("A/B:C", "2.0.0"), "ABC2.0.0.exe");
        assert_eq!(installer_name("", "2.0.0"), "U3drinth2.0.0.exe");
    }

    #[test]
    fn the_root_is_found_inside_wrapping_folders_and_read() {
        let dir = scratch("root");
        let root = dir.join("modrinth-offline-u3drinth/inner");
        fake_source(&root, "1.0.0");
        fs::create_dir_all(dir.join("node_modules/x")).unwrap();
        assert_eq!(find_root(&dir), Some(root.clone()));
        assert_eq!(
            find_root(&dir.join("modrinth-offline-u3drinth")),
            Some(root.clone())
        );
        assert_eq!(find_root(&root.join("apps")), None);
        assert_eq!(
            app_version(&root).unwrap(),
            ("U3drinth".to_string(), "1.0.0".to_string())
        );
        assert_eq!(pnpm_version(&root).as_deref(), Some("9.4.0"));
        write(
            &root.join("apps/app/tauri.conf.json"),
            r#"{"package":{"productName":"U3drinth","version":"../package.json"}}"#,
        );
        write(&root.join("apps/package.json"), r#"{"version":"2.3.4"}"#);
        assert_eq!(app_version(&root).unwrap().1, "2.3.4");
        let _ = util::remove_tree(&dir);
    }

    #[test]
    fn a_zip_is_unpacked_and_opened() {
        let dir = scratch("zip");
        let zip_path = dir.join("U3drinth-source.zip");
        {
            let mut zip = zip::ZipWriter::new(File::create(&zip_path).unwrap());
            let options = zip::write::SimpleFileOptions::default();
            for (name, text) in [
                (
                    "U3drinth/apps/app/tauri.conf.json",
                    r#"{"package":{"productName":"U3drinth","version":"1.2.3"}}"#,
                ),
                ("U3drinth/packages/app-lib/Cargo.toml", "[package]"),
                ("U3drinth/Cargo.toml", "[workspace]"),
                ("U3drinth/pnpm-lock.yaml", "x"),
                ("U3drinth/package.json", "{}"),
            ] {
                zip.start_file(name, options).unwrap();
                std::io::Write::write_all(&mut zip, text.as_bytes()).unwrap();
            }
            zip.finish().unwrap();
        }
        let ui = Ui::new();
        let source = open(&ui, &zip_path, &dir.join("incoming")).unwrap();
        assert_eq!(source.root, dir.join("incoming/U3drinth"));
        assert_eq!((source.version.as_str(), source.pnpm), ("1.2.3", None));
        assert!(
            open(&ui, &dir.join("missing.zip"), &dir.join("incoming")).is_err()
        );
        let _ = util::remove_tree(&dir);
    }

    #[test]
    fn sync_mirrors_sources_and_keeps_build_products() {
        let dir = scratch("sync");
        let (from, work) = (dir.join("from"), dir.join("work"));
        fake_source(&from, "1.0.0");
        write(&from.join("src/a.rs"), "a");
        write(&from.join("src/b.rs"), "b");
        write(&from.join("node_modules/dep/index.js"), "dep");
        write(&from.join("apps/app/u3-engine/engine.json"), "stale");
        let first = sync(&from, &work).unwrap();
        assert_eq!(
            first,
            Synced {
                files: 7,
                copied: 7,
                removed: 0
            }
        );
        assert!(
            !work.join("node_modules").exists()
                && !work.join("apps/app/u3-engine").exists()
        );
        write(&work.join("node_modules/kept/index.js"), "kept");
        write(&work.join("target/release/app.exe"), "exe");
        write(&work.join(ENGINE_DIR).join("engine.json"), "engine");
        write(&work.join(BUILDER_CONFIG), "{}");
        let old =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000);
        fs::remove_file(from.join("src/b.rs")).unwrap();
        write(&from.join("src/a.rs"), "a2");
        fs::OpenOptions::new()
            .write(true)
            .open(from.join("src/a.rs"))
            .unwrap()
            .set_modified(old)
            .unwrap();
        write(&from.join("src/new/c.rs"), "c");
        let second = sync(&from, &work).unwrap();
        assert_eq!(
            second,
            Synced {
                files: 7,
                copied: 2,
                removed: 1
            }
        );
        assert_eq!(fs::read_to_string(work.join("src/a.rs")).unwrap(), "a2");
        assert!(
            fs::metadata(work.join("src/a.rs"))
                .unwrap()
                .modified()
                .unwrap()
                > old
        );
        assert!(!work.join("src/b.rs").exists());
        for kept in [
            "node_modules/kept/index.js",
            "target/release/app.exe",
            "apps/app/u3-engine/engine.json",
            BUILDER_CONFIG,
        ] {
            assert!(work.join(kept).exists(), "{kept}");
        }
        fs::remove_dir_all(from.join("src/new")).unwrap();
        let third = sync(&from, &work).unwrap();
        assert_eq!(
            third,
            Synced {
                files: 6,
                copied: 0,
                removed: 1
            }
        );
        assert!(!work.join("src/new").exists());
        let _ = util::remove_tree(&dir);
    }
}
