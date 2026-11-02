//! The launcher itself: dependencies, optional tests and the Tauri NSIS installer, built as the
//! GitHub Actions workflow builds it, with the release profile tuned for speed.

use crate::engine::{self, Engine};
use crate::proc;
use crate::source::{BUILDER_CONFIG, ENGINE_DIR};
use crate::tools::Tools;
use crate::ui::Ui;
use crate::util;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn dependencies(ui: &Ui, tools: &Tools, work: &Path) -> Result<(), String> {
    let mut install = tools.command(tools.pnpm());
    install
        .args([
            "install",
            "--frozen-lockfile",
            "--ignore-scripts",
            "--store-dir",
        ])
        .arg(tools.pnpm_store())
        .current_dir(work);
    proc::run(ui, &mut install, "pnpm install")?;
    let mut esbuild = tools.command(tools.pnpm());
    esbuild.args(["rebuild", "esbuild"]).current_dir(work);
    proc::run(ui, &mut esbuild, "pnpm rebuild esbuild")?;
    ui.ok("Зависимости на месте");
    Ok(())
}

/// Cargo settings shared by the tests and the build, so they reuse each other's work.
fn cargo_env(cmd: &mut Command, home: &Path) {
    cmd.env("CARGO_TARGET_DIR", home.join("target"))
        .env("CARGO_PROFILE_RELEASE_OPT_LEVEL", "3")
        .env("CARGO_INCREMENTAL", "0");
}

pub fn tests(
    ui: &Ui,
    tools: &Tools,
    home: &Path,
    work: &Path,
    engine: Option<&Engine>,
) -> Result<(), String> {
    let mut frontend = tools.command(tools.pnpm());
    frontend
        .args(["--filter", "@modrinth/app-frontend", "test"])
        .current_dir(work);
    proc::run(ui, &mut frontend, "Тесты интерфейса")?;
    let mut native = tools.command(tools.cargo());
    native
        .args(["test", "-p", "theseus", "--lib", "--locked", "--release"])
        .current_dir(work);
    cargo_env(&mut native, home);
    if engine.is_some() {
        native.env("U3_ENGINE_DIR", work.join(ENGINE_DIR));
    }
    proc::run(ui, &mut native, "Тесты лаунчера")?;
    ui.ok("Тесты прошли");
    Ok(())
}

/// The config the build uses: the engine's resources (when shipped) and the chosen version.
pub fn builder_config(engine_config: Option<Value>, version: &str) -> Value {
    let mut config = engine_config.unwrap_or_else(|| json!({}));
    if !config.is_object() {
        config = json!({});
    }
    let package = config
        .as_object_mut()
        .unwrap()
        .entry("package")
        .or_insert_with(|| json!({}));
    if !package.is_object() {
        *package = json!({});
    }
    package
        .as_object_mut()
        .unwrap()
        .insert("version".into(), json!(version));
    config
}

pub fn build(
    ui: &Ui,
    tools: &Tools,
    home: &Path,
    work: &Path,
    version: &str,
    engine: Option<&Engine>,
) -> Result<PathBuf, String> {
    let engine_config = match engine {
        Some(_) => {
            let path = engine::config(work);
            let text = fs::read_to_string(&path)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            Some(
                serde_json::from_str(&text)
                    .map_err(|e| format!("{}: {e}", path.display()))?,
            )
        }
        None => {
            let _ = util::remove_tree(&work.join(ENGINE_DIR));
            None
        }
    };
    let config = builder_config(engine_config, version);
    let config_path = work.join(BUILDER_CONFIG);
    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| format!("{}: {e}", config_path.display()))?;
    // Tauri honours CARGO_TARGET_DIR; the work folder's own target is only a fallback.
    let bundles: Vec<PathBuf> = [home.join("target"), work.join("target")]
        .iter()
        .map(|t| t.join("release").join("bundle").join("nsis"))
        .collect();
    for bundle in &bundles {
        util::remove_tree(bundle)
            .map_err(|e| format!("{}: {e}", bundle.display()))?;
    }
    let mut tauri = tools.command(tools.pnpm());
    tauri
        .args(["exec", "tauri", "build", "--bundles", "nsis", "--config"])
        .arg(config_path.file_name().unwrap())
        .current_dir(work.join("apps/app"));
    cargo_env(&mut tauri, home);
    proc::run(ui, &mut tauri, "Сборка лаунчера")?;
    let mut setups: Vec<PathBuf> = bundles
        .iter()
        .filter_map(|b| fs::read_dir(b).ok())
        .flat_map(|entries| entries.flatten().map(|e| e.path()))
        .filter(|p| p.to_string_lossy().to_lowercase().ends_with("-setup.exe"))
        .collect();
    setups.sort();
    setups.pop().ok_or_else(|| {
        format!("в {} не появился установщик", bundles[0].display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_build_config_adds_the_version_to_the_engine_resources() {
        let engine = json!({"tauri": {"bundle": {"resources": ["u3-engine/horizon-client.exe", "u3-engine/engine.json"]}}});
        let config = builder_config(Some(engine), "1.1.1");
        assert_eq!(config["package"]["version"], "1.1.1");
        assert_eq!(
            config["tauri"]["bundle"]["resources"][1],
            "u3-engine/engine.json"
        );
        assert_eq!(
            builder_config(None, "2.0.0"),
            json!({"package": {"version": "2.0.0"}})
        );
        assert_eq!(
            builder_config(Some(json!({"package": 5})), "2.0.0"),
            json!({"package": {"version": "2.0.0"}})
        );
    }
}
