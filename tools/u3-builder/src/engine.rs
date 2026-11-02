//! The prebuilt U3 Fixes engine the installer ships: built by the source's own recipe
//! (.github/u3-engine/build.py: Horizon's native-build, as in GitHub Actions), kept by its cache
//! key, so it is built again only when U3 Fixes or the pinned Horizon change. An engine already
//! installed with U3drinth is reused when it is the same build.

use crate::proc;
use crate::source::ENGINE_DIR;
use crate::tools::Tools;
use crate::ui::Ui;
use crate::util;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = ".github/u3-engine/build.py";
const ENGINE_CONFIG: &str = "apps/app/tauri.u3-engine.conf.json";
const FILES: [&str; 2] = ["horizon-client.exe", "engine.json"];

pub struct Engine {
    pub dir: PathBuf,
    pub version: String,
}

/// The cache key build.py --key prints, from an engine.json.
pub fn key_of(facts: &Value) -> Option<String> {
    let version = facts["version"].as_str()?;
    let runtime = facts["runtime"].as_str()?;
    let recipe = facts["recipe"].as_u64()?;
    Some(format!("{version}-{}-r{recipe}", runtime.replace('@', "-")))
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 120
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}

/// The engine in `dir`, if it is the build `key` names and its file is intact.
pub fn check(dir: &Path, key: &str) -> Option<Engine> {
    let facts: Value = serde_json::from_str(
        &fs::read_to_string(dir.join("engine.json")).ok()?,
    )
    .ok()?;
    if key_of(&facts)? != key {
        return None;
    }
    let exe = dir.join("horizon-client.exe");
    if fs::metadata(&exe).ok()?.len() != facts["size"].as_u64()? {
        return None;
    }
    let sha = util::sha256_file(&exe).ok()?;
    (sha.eq_ignore_ascii_case(facts["sha256"].as_str()?)).then(|| Engine {
        dir: dir.to_path_buf(),
        version: facts["version"].as_str().unwrap_or_default().to_string(),
    })
}

pub fn supported(work: &Path) -> bool {
    work.join(RECIPE).is_file() && work.join(ENGINE_CONFIG).is_file()
}

pub fn config(work: &Path) -> PathBuf {
    work.join(ENGINE_CONFIG)
}

/// Where an installed U3drinth keeps its engine.
fn installed_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for var in ["LOCALAPPDATA", "ProgramFiles", "ProgramW6432"] {
        if let Some(base) = std::env::var_os(var) {
            let base = PathBuf::from(base);
            dirs.push(base.join("U3drinth"));
            dirs.push(base.join("Programs").join("U3drinth"));
        }
    }
    for hive in ["HKCU", "HKLM"] {
        let key = format!(
            r"{hive}\Software\Microsoft\Windows\CurrentVersion\Uninstall\U3drinth"
        );
        let Ok(out) = std::process::Command::new("reg.exe")
            .args(["query", &key, "/v", "UninstallString"])
            .output()
        else {
            continue;
        };
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(path) = text.lines().find_map(|l| {
            l.split_once("REG_SZ")
                .map(|(_, v)| v.trim().trim_matches('"').to_string())
        }) {
            if let Some(dir) = Path::new(&path).parent() {
                dirs.push(dir.to_path_buf());
            }
        }
    }
    dirs.into_iter().map(|d| d.join("u3-engine")).collect()
}

fn copy_engine(from: &Path, to: &Path) -> Result<(), String> {
    util::remove_tree(to).map_err(|e| format!("{}: {e}", to.display()))?;
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for name in FILES {
        fs::copy(from.join(name), to.join(name))
            .map_err(|e| format!("{}: {e}", from.join(name).display()))?;
    }
    Ok(())
}

/// The engine for this source: cached, taken from an installed U3drinth, or built.
pub fn prepare(
    ui: &Ui,
    tools: &Tools,
    home: &Path,
    work: &Path,
) -> Result<Engine, String> {
    let recipe = work.join(RECIPE);
    let key = proc::capture(
        ui,
        tools
            .command(tools.python())
            .arg(&recipe)
            .arg("--key")
            .current_dir(work),
    )
    .map_err(|e| format!("ключ движка: {e}"))?;
    let key = key.lines().last().unwrap_or_default().trim().to_string();
    if !valid_key(&key) {
        return Err(format!("build.py --key вернул неожиданное: {key}"));
    }
    let engines = home.join("engines");
    let cached = engines.join(&key);
    ui.status("      Проверка готового движка…");
    if let Some(engine) = check(&cached, &key) {
        ui.ok(&format!("Движок уже собран: {}", engine.version));
        return Ok(engine);
    }
    for dir in installed_dirs() {
        if check(&dir, &key).is_some() {
            copy_engine(&dir, &cached)?;
            if let Some(engine) = check(&cached, &key) {
                ui.ok(&format!(
                    "Движок {} взят из установленного U3drinth ({})",
                    engine.version,
                    dir.display()
                ));
                return Ok(engine);
            }
        }
    }
    ui.clear_status();
    ui.info("Сборка движка Horizon с U3 Fixes: первый раз 15–40 минут, дальше берётся готовый.");
    let partial = engines.join(format!("{key}.partial"));
    util::remove_tree(&partial)
        .map_err(|e| format!("{}: {e}", partial.display()))?;
    let temp = home.join("tmp");
    fs::create_dir_all(&temp)
        .map_err(|e| format!("{}: {e}", temp.display()))?;
    let mut build = tools.command(tools.python());
    build
        .arg(&recipe)
        .arg(&partial)
        .current_dir(work)
        .env("RUNNER_TEMP", &temp);
    if let Err(e) = proc::run(ui, &mut build, "Сборка движка") {
        let log = temp.join("u3-engine/launcher/native/build.log");
        if log.is_file() {
            ui.info(&format!("Журнал сборки Horizon: {}", log.display()));
        }
        return Err(e);
    }
    let Some(_) = check(&partial, &key) else {
        return Err(format!("сборка движка не дала {key}"));
    };
    util::remove_tree(&cached)
        .map_err(|e| format!("{}: {e}", cached.display()))?;
    util::rename(&partial, &cached)
        .map_err(|e| format!("{}: {e}", cached.display()))?;
    let _ = util::remove_tree(&temp.join("u3-engine"));
    let engine = check(&cached, &key)
        .ok_or_else(|| format!("{}: движок повреждён", cached.display()))?;
    ui.ok(&format!("Движок собран: {}", engine.version));
    Ok(engine)
}

/// Puts the engine where the installer config takes it from.
pub fn stage(engine: &Engine, work: &Path) -> Result<(), String> {
    copy_engine(&engine.dir, &work.join(ENGINE_DIR))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keys_match_build_py() {
        let facts = json!({"version": "native-3132240-82fef81db81e", "runtime": "launcher@0.2.28", "recipe": 1});
        assert_eq!(
            key_of(&facts).as_deref(),
            Some("native-3132240-82fef81db81e-launcher-0.2.28-r1")
        );
        assert_eq!(key_of(&json!({"version": "x"})), None);
        assert!(valid_key("native-3132240-82fef81db81e-launcher-0.2.28-r1"));
        assert!(!valid_key("../x") && !valid_key("a b") && !valid_key(""));
    }

    #[test]
    fn only_an_intact_engine_of_the_same_build_is_used() {
        let dir = std::env::temp_dir()
            .join(format!("u3-builder-test-engine-{}", std::process::id()));
        let _ = util::remove_tree(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("horizon-client.exe"), b"MZengine").unwrap();
        let sha = util::sha256_file(&dir.join("horizon-client.exe")).unwrap();
        let facts = json!({"version": "native-1", "runtime": "launcher@0.2.28", "recipe": 1, "size": 8, "sha256": sha});
        fs::write(dir.join("engine.json"), facts.to_string()).unwrap();
        assert_eq!(
            check(&dir, "native-1-launcher-0.2.28-r1").unwrap().version,
            "native-1"
        );
        assert!(check(&dir, "native-2-launcher-0.2.28-r1").is_none());
        fs::write(dir.join("horizon-client.exe"), b"MZengin!").unwrap();
        assert!(check(&dir, "native-1-launcher-0.2.28-r1").is_none());
        let _ = util::remove_tree(&dir);
    }
}
