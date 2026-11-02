//! U3drinth Builder: drop U3drinth's source (a folder or a .zip) on this program and it makes
//! the ready Windows installer, as the project's GitHub Actions workflow does: the prebuilt
//! U3 Fixes engine inside (Horizon's own native-build, kept until U3 Fixes changes), the launcher
//! built for speed. Everything the build needs installs itself on the first run.

mod app;
mod engine;
mod net;
mod proc;
mod source;
mod tools;
mod ui;
mod util;

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Instant, SystemTime};
use tools::Tools;
use ui::Ui;

const HELP: &str = "U3drinth Builder: собирает установщик U3drinth для Windows из исходников.

Перетащи папку или .zip с исходниками на U3drinthBuilder.exe или запусти его и перетащи их в окно.

U3drinthBuilder.exe [исходники] [параметры]
  --version 1.1.1   версия установщика (иначе спросит, по умолчанию версия из исходников)
  --out ПАПКА       куда положить установщик (по умолчанию рядом с исходниками)
  --tests           сначала прогнать тесты, как GitHub Actions
  --no-engine       без готового движка U3 Fixes (первый запуск игры 20-30 минут)
  --yes             ничего не спрашивать
  --home ПАПКА      где держать инструменты и кэш (по умолчанию %LOCALAPPDATA%\\U3drinthBuilder)";

#[derive(Default, Debug, PartialEq)]
struct Args {
    source: Option<PathBuf>,
    version: Option<String>,
    out: Option<PathBuf>,
    home: Option<PathBuf>,
    yes: bool,
    tests: bool,
    no_engine: bool,
    vs_only: bool,
    force_vs: bool,
}

fn parse(argv: Vec<OsString>) -> Result<Args, String> {
    let mut args = Args::default();
    let mut it = argv.into_iter();
    while let Some(arg) = it.next() {
        let text = arg.to_string_lossy().into_owned();
        let mut value = |name: &str| {
            it.next()
                .ok_or_else(|| format!("после {name} нужно значение"))
        };
        match text.as_str() {
            "--version" => {
                args.version =
                    Some(value(&text)?.to_string_lossy().into_owned())
            }
            "--out" => args.out = Some(PathBuf::from(value(&text)?)),
            "--home" => args.home = Some(PathBuf::from(value(&text)?)),
            "--yes" | "-y" => args.yes = true,
            "--tests" => args.tests = true,
            "--no-engine" => args.no_engine = true,
            "--vs-only" => args.vs_only = true,
            "--force-vs-install" => args.force_vs = true,
            s if s.starts_with("--") => {
                return Err(format!("неизвестный параметр {s}"))
            }
            _ if args.source.is_some() => {
                return Err(
                    "нужна одна папка или один .zip с исходниками".into()
                )
            }
            _ => args.source = Some(clean_path(&text)),
        }
    }
    Ok(args)
}

/// A path as typed or dropped into the console: «"C:\a b\src.zip"», «& 'C:\x'».
fn clean_path(text: &str) -> PathBuf {
    let mut text = text.trim();
    if let Some(rest) = text.strip_prefix("& ") {
        text = rest.trim();
    }
    PathBuf::from(text.trim_matches(|c| c == '"' || c == '\''))
}

fn default_home() -> PathBuf {
    if let Some(home) = std::env::var_os("U3_BUILDER_HOME") {
        return PathBuf::from(home);
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let home = PathBuf::from(local).join("U3drinthBuilder");
        // Compilers and Horizon's build are safest on a plain ASCII path.
        if home.to_string_lossy().is_ascii() {
            return home;
        }
    }
    let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    PathBuf::from(format!(r"{drive}\U3drinthBuilder"))
}

fn lock(home: &Path) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    options
        .open(home.join("builder.lock"))
        .map_err(|_| "сборщик уже запущен в другом окне".to_string())
}

fn open_log(ui: &Ui, home: &Path) -> Option<PathBuf> {
    let dir = home.join("logs");
    fs::create_dir_all(&dir).ok()?;
    let mut old: Vec<PathBuf> = fs::read_dir(&dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "log"))
        .collect();
    old.sort();
    while old.len() >= 10 {
        let _ = fs::remove_file(old.remove(0));
    }
    let path =
        dir.join(format!("build-{}.log", util::stamp(SystemTime::now())));
    ui.open_log(&path).ok()?;
    Some(path)
}

fn ask_source(ui: &Ui) -> Result<PathBuf, String> {
    ui.say("Перетащи в это окно папку или .zip с исходниками U3drinth и нажми Enter");
    ui.say("(можно и сразу перетащить их на значок U3drinthBuilder.exe).");
    loop {
        let Some(answer) = ui.ask("> ") else {
            return Err("не указаны исходники: перетащи папку или .zip на U3drinthBuilder.exe".into());
        };
        let path = clean_path(&answer);
        if path.as_os_str().is_empty() {
            continue;
        }
        if path.exists() {
            return Ok(path);
        }
        ui.say(&format!("      Не найдено: {}", path.display()));
    }
}

fn choose_version(
    ui: &Ui,
    args: &Args,
    current: &str,
) -> Result<String, String> {
    if let Some(version) = &args.version {
        let version = version.trim().trim_start_matches(['v', 'V']);
        return if source::valid_version(version) {
            Ok(version.to_string())
        } else {
            Err(format!("версия «{version}» не подходит: нужно три числа, например 1.1.1"))
        };
    }
    let fallback = source::valid_version(current).then(|| current.to_string());
    loop {
        let prompt = match &fallback {
            Some(v) => format!("      Версия установщика (Enter — {v}): "),
            None => "      Версия установщика, например 1.1.1: ".to_string(),
        };
        let Some(answer) = ui.ask(&prompt) else {
            return fallback.ok_or_else(|| format!("версия в исходниках «{current}» не подходит для установщика: укажи --version 1.1.1"));
        };
        let answer = answer.trim().trim_start_matches(['v', 'V']);
        if answer.is_empty() {
            if let Some(v) = &fallback {
                return Ok(v.clone());
            }
        } else if source::valid_version(answer) {
            return Ok(answer.to_string());
        }
        ui.say("      Нужно три числа через точку, например 1.1.1");
    }
}

fn output_dir(args: &Args, input: &Path) -> PathBuf {
    if let Some(out) = &args.out {
        return std::path::absolute(out).unwrap_or_else(|_| out.clone());
    }
    input
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| input.to_path_buf())
}

fn writable(dir: &Path) -> bool {
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".u3drinth-builder-probe");
    let ok = File::create(&probe).is_ok();
    let _ = fs::remove_file(&probe);
    ok
}

/// Copies the installer to its place; a busy file of the same name gets «(2)», «(3)»…
fn deliver(
    setup: &Path,
    dirs: &[PathBuf],
    name: &str,
) -> Result<PathBuf, String> {
    let stem = name.trim_end_matches(".exe");
    let mut last = String::new();
    for dir in dirs.iter().filter(|d| writable(d)) {
        for n in 1..=9 {
            let target = if n == 1 {
                dir.join(name)
            } else {
                dir.join(format!("{stem} ({n}).exe"))
            };
            match fs::copy(setup, &target) {
                Ok(_) => return Ok(target),
                Err(e) => last = format!("{}: {e}", target.display()),
            }
        }
    }
    Err(format!(
        "не удалось сохранить установщик ({last}); он лежит здесь: {}",
        setup.display()
    ))
}

fn check_space(ui: &Ui, home: &Path) {
    let first = !home.join("target").exists();
    let need: u64 = if first { 25 << 30 } else { 8 << 30 };
    if let Some(free) = util::free_space(home) {
        if free < need {
            ui.warn(&format!(
                "На диске с {} свободно {}, а для сборки нужно около {}.",
                home.display(),
                util::gb(free),
                util::gb(need)
            ));
        }
    }
}

struct Done {
    path: PathBuf,
    sha256: String,
    size: u64,
    engine: Option<String>,
}

fn run(ui: &Ui, args: &Args, home: &Path) -> Result<Option<Done>, String> {
    if args.vs_only {
        ui.step(1, 1, "Visual Studio Build Tools");
        tools::msvc(ui, &Tools::new(home, None), args.force_vs)?;
        return Ok(None);
    }
    let input = match &args.source {
        Some(path) => path.clone(),
        None => ask_source(ui)?,
    };
    let input = std::path::absolute(&input).unwrap_or(input);
    let total = if args.tests { 6 } else { 5 };

    ui.step(1, total, "Исходники");
    let source = source::open(ui, &input, &home.join("incoming"))?;
    ui.ok(&format!(
        "{} {}: {}",
        source.product,
        source.version,
        source.root.display()
    ));
    let version = choose_version(ui, args, &source.version)?;
    let name = source::installer_name(&source.product, &version);
    let out_dir = output_dir(args, &input);
    ui.info(&format!("Получится {}", out_dir.join(&name).display()));
    let work = home.join("work").join("src");
    ui.status("      Копирование в рабочую папку…");
    let synced = source::sync(&source.root, &work)?;
    ui.ok(&format!(
        "Рабочая папка {}: файлов {}, обновлено {}, удалено {}",
        work.display(),
        synced.files,
        synced.copied,
        synced.removed
    ));

    ui.step(2, total, "Инструменты сборки");
    check_space(ui, home);
    let tools = Tools::new(home, source.pnpm.as_deref());
    tools.ensure(ui)?;

    ui.step(
        3,
        total,
        "Движок U3 Fixes (первый запуск игры без компиляции)",
    );
    let engine = if args.no_engine {
        ui.warn("Без готового движка (--no-engine): у игроков первый запуск займёт 20–30 минут.");
        None
    } else if !engine::supported(&work) {
        ui.warn("В этих исходниках нет рецепта готового движка (.github/u3-engine/build.py):");
        ui.warn("установщик соберётся без него, первый запуск игры у игроков займёт 20–30 минут.");
        None
    } else {
        match engine::prepare(ui, &tools, home, &work) {
            Ok(engine) => Some(engine),
            Err(e) => {
                ui.error(&e);
                let question = "      Собрать установщик без готового движка (первый запуск игры у игроков займёт 20–30 минут)?";
                if !ui.interactive() || !ui.confirm(question, false) {
                    return Err(e);
                }
                None
            }
        }
    };
    if let Some(engine) = &engine {
        engine::stage(engine, &work)?;
    }

    ui.step(4, total, "Зависимости");
    app::dependencies(ui, &tools, &work)?;
    let mut step = 5;
    if args.tests {
        ui.step(step, total, "Тесты");
        app::tests(ui, &tools, home, &work, engine.as_ref())?;
        step += 1;
    }

    ui.step(
        step,
        total,
        "Сборка лаунчера и установщика (оптимизация по скорости)",
    );
    let setup = app::build(ui, &tools, home, &work, &version, engine.as_ref())?;
    let mut dirs = vec![out_dir];
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        dirs.push(PathBuf::from(profile).join("Desktop"));
    }
    dirs.push(home.join("output"));
    let path = deliver(&setup, &dirs, &name)?;
    let sha256 = util::sha256_file(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    Ok(Some(Done {
        path,
        sha256,
        size,
        engine: engine.map(|e| e.version),
    }))
}

#[cfg(windows)]
fn show_in_explorer(path: &Path) {
    use std::os::windows::process::CommandExt;
    let _ = std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", path.display()))
        .spawn();
}

#[cfg(not(windows))]
fn show_in_explorer(_path: &Path) {}

fn main() -> ExitCode {
    ui::set_title("U3drinth Builder");
    let ui = Ui::new();
    let owns_console = ui::owns_console();
    let argv: Vec<OsString> = std::env::args_os().skip(1).collect();
    if argv.iter().any(|a| a == "--help" || a == "-h" || a == "/?") {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let finish = |ui: &Ui| {
        if owns_console && ui.interactive() {
            ui.say("");
            ui.ask("Нажми Enter, чтобы закрыть окно.");
        }
    };
    let args = match parse(argv) {
        Ok(args) => args,
        Err(e) => {
            ui.error(&e);
            ui.say(HELP);
            finish(&ui);
            return ExitCode::from(2);
        }
    };
    ui.set_interactive(!args.yes);
    let home = args.home.clone().unwrap_or_else(default_home);
    let home = std::path::absolute(&home).unwrap_or(home);
    ui.say(&format!("U3drinth Builder {}", env!("CARGO_PKG_VERSION")));
    ui.say("Собирает готовый установщик U3drinth для Windows: движок U3 Fixes уже внутри");
    ui.say("(первый запуск игры без компиляции), лаунчер собран с оптимизацией по скорости.");
    if !home.join("tools").exists() && !args.vs_only {
        ui.say("Первая сборка ставит инструменты и собирает движок: 40–90 минут. Следующие обычно 5–15 минут.");
    }
    ui.say(&format!("Инструменты и кэш: {}", home.display()));
    ui.say("");
    if let Err(e) = fs::create_dir_all(&home) {
        ui.error(&format!("{}: {e}", home.display()));
        finish(&ui);
        return ExitCode::FAILURE;
    }
    let _lock = match lock(&home) {
        Ok(lock) => lock,
        Err(e) => {
            ui.error(&e);
            finish(&ui);
            return ExitCode::FAILURE;
        }
    };
    let log = open_log(&ui, &home);
    let started = Instant::now();
    let code = match run(&ui, &args, &home) {
        Ok(Some(done)) => {
            ui.say("");
            ui.say(&format!(
                "ГОТОВО за {}: {}",
                util::duration(started.elapsed()),
                done.path.display()
            ));
            ui.say(&format!(
                "Размер {}, SHA-256 {}",
                util::mb(done.size),
                done.sha256
            ));
            match &done.engine {
                Some(engine) => ui.say(&format!("Движок U3 Fixes внутри: {engine}")),
                None => ui.say("Без готового движка: первый запуск игры у игроков займёт 20–30 минут."),
            }
            if owns_console && ui.interactive() {
                show_in_explorer(&done.path);
            }
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(e) => {
            ui.error(&e);
            if let Some(log) = &log {
                ui.say(&format!("Подробный журнал: {}", log.display()));
            }
            ui.say("Можно просто запустить сборку ещё раз: уже готовое второй раз не делается.");
            ExitCode::FAILURE
        }
    };
    finish(&ui);
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn arguments() {
        let args = parse(os(&[
            "D:\\src.zip",
            "--version",
            "1.1.1",
            "--yes",
            "--tests",
        ]))
        .unwrap();
        assert_eq!(args.source, Some(PathBuf::from("D:\\src.zip")));
        assert_eq!(
            (args.version.as_deref(), args.yes, args.tests),
            (Some("1.1.1"), true, true)
        );
        assert!(parse(os(&["a", "b"])).is_err());
        assert!(parse(os(&["--version"])).is_err());
        assert!(parse(os(&["--what"])).is_err());
        assert_eq!(parse(os(&[])).unwrap(), Args::default());
    }

    #[test]
    fn dropped_paths_lose_their_quotes() {
        assert_eq!(
            clean_path("  \"C:\\My Files\\src.zip\" "),
            PathBuf::from("C:\\My Files\\src.zip")
        );
        assert_eq!(clean_path("& 'C:\\x y'"), PathBuf::from("C:\\x y"));
        assert_eq!(clean_path("C:\\plain"), PathBuf::from("C:\\plain"));
    }

    #[test]
    fn versions_from_arguments_are_checked() {
        let ui = Ui::new();
        ui.set_interactive(false);
        let with = |v: &str| Args {
            version: Some(v.into()),
            ..Args::default()
        };
        assert_eq!(
            choose_version(&ui, &with("v1.1.1"), "1.0.0").unwrap(),
            "1.1.1"
        );
        assert!(choose_version(&ui, &with("1.1"), "1.0.0").is_err());
        assert_eq!(
            choose_version(&ui, &Args::default(), "1.0.0").unwrap(),
            "1.0.0"
        );
        assert!(choose_version(&ui, &Args::default(), "1.0.0-beta").is_err());
    }

    #[test]
    fn the_installer_lands_next_to_the_source_or_falls_back() {
        let dir = std::env::temp_dir()
            .join(format!("u3-builder-test-deliver-{}", std::process::id()));
        let _ = util::remove_tree(&dir);
        fs::create_dir_all(&dir).unwrap();
        let setup = dir.join("U3drinth_1.0.0_x64-setup.exe");
        fs::write(&setup, b"MZ").unwrap();
        let out =
            deliver(&setup, &[dir.join("out")], "U3drinth1.1.1.exe").unwrap();
        assert_eq!(out, dir.join("out").join("U3drinth1.1.1.exe"));
        assert_eq!(fs::read(&out).unwrap(), b"MZ");
        let args = Args::default();
        assert_eq!(output_dir(&args, &dir.join("src.zip")), dir);
        let _ = util::remove_tree(&dir);
    }
}
