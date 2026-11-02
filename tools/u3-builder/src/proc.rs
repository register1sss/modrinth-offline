//! Child processes: their output goes to the log, its latest line to the status line, and the
//! last lines are shown when one fails.

use crate::ui::{write_log, Log, Ui};
use crate::util;
use std::collections::VecDeque;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const TAIL: usize = 40;

#[derive(Default)]
struct Output {
    last: String,
    tail: VecDeque<String>,
}

pub fn describe(cmd: &Command) -> String {
    let mut text = cmd.get_program().to_string_lossy().into_owned();
    for arg in cmd.get_args() {
        text.push(' ');
        text.push_str(&arg.to_string_lossy());
    }
    if let Some(dir) = cmd.get_current_dir() {
        text.push_str(&format!("   (в {})", dir.display()));
    }
    text
}

fn program(cmd: &Command) -> String {
    std::path::Path::new(cmd.get_program())
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Runs a console program to its end, showing its progress.
pub fn run(ui: &Ui, cmd: &mut Command, label: &str) -> Result<(), String> {
    ui.log_line(&format!("> {}", describe(cmd)));
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| {
        format!("{label}: не запускается {}: {e}", program(cmd))
    })?;
    let output = Arc::new(Mutex::new(Output::default()));
    let finished = Arc::new(AtomicUsize::new(0));
    let mut pumps = 0;
    if let Some(out) = child.stdout.take() {
        pump(out, ui.log(), output.clone(), finished.clone());
        pumps += 1;
    }
    if let Some(err) = child.stderr.take() {
        pump(err, ui.log(), output.clone(), finished.clone());
        pumps += 1;
    }
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => return Err(format!("{label}: {e}")),
        }
        let last = output.lock().unwrap().last.clone();
        ui.status(&format!(
            "      {label} · {} · {last}",
            util::clock(started.elapsed())
        ));
        std::thread::sleep(Duration::from_millis(250));
    };
    // The rest of its output, unless something it started still holds the pipes open.
    let drained = Instant::now();
    while finished.load(Ordering::SeqCst) < pumps
        && drained.elapsed() < Duration::from_secs(5)
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    ui.clear_status();
    ui.log_line(&format!(
        "< {label}: {status}, {}",
        util::duration(started.elapsed())
    ));
    if status.success() {
        return Ok(());
    }
    let tail: Vec<String> =
        output.lock().unwrap().tail.iter().cloned().collect();
    if !tail.is_empty() {
        ui.say("      Последние строки вывода:");
        for line in tail {
            ui.say(&format!("      | {line}"));
        }
    }
    let code = status
        .code()
        .map_or_else(|| "?".to_string(), |c| c.to_string());
    Err(format!("{label}: завершилось с кодом {code}"))
}

/// Runs a program with its own window (an installer) and returns its exit code.
pub fn wait(ui: &Ui, cmd: &mut Command, label: &str) -> Result<i32, String> {
    ui.log_line(&format!("> {}", describe(cmd)));
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = cmd.spawn().map_err(|e| {
        if e.raw_os_error() == Some(740) {
            format!("{label}: нужны права администратора")
        } else {
            format!("{label}: не запускается {}: {e}", program(cmd))
        }
    })?;
    wait_child(ui, &mut child, label)
}

pub fn wait_child(
    ui: &Ui,
    child: &mut std::process::Child,
    label: &str,
) -> Result<i32, String> {
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => return Err(format!("{label}: {e}")),
        }
        ui.status(&format!(
            "      {label} · {}",
            util::clock(started.elapsed())
        ));
        std::thread::sleep(Duration::from_millis(500));
    };
    ui.clear_status();
    ui.log_line(&format!(
        "< {label}: {status}, {}",
        util::duration(started.elapsed())
    ));
    Ok(status.code().unwrap_or(-1))
}

/// Runs a short command and returns its trimmed standard output.
pub fn capture(ui: &Ui, cmd: &mut Command) -> Result<String, String> {
    ui.log_line(&format!("> {}", describe(cmd)));
    let out = cmd
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("не запускается {}: {e}", program(cmd)))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    for text in [&stdout, &stderr] {
        if !text.trim().is_empty() {
            ui.log_line(text.trim_end());
        }
    }
    if out.status.success() {
        return Ok(stdout.trim().to_string());
    }
    let said = if stderr.trim().is_empty() {
        &stdout
    } else {
        &stderr
    };
    let last: Vec<&str> =
        said.lines().filter(|l| !l.trim().is_empty()).collect();
    let tail = last[last.len().saturating_sub(5)..].join(" / ");
    Err(format!("{} завершилась с ошибкой: {tail}", program(cmd)))
}

fn pump(
    mut from: impl Read + Send + 'static,
    log: Log,
    output: Arc<Mutex<Output>>,
    finished: Arc<AtomicUsize>,
) {
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut lines = Lines::default();
        loop {
            let n = match from.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            for (text, keep) in lines.feed(&buf[..n]) {
                commit(&text, keep, &log, &output);
            }
        }
        if let Some((text, keep)) = lines.finish() {
            commit(&text, keep, &log, &output);
        }
        finished.fetch_add(1, Ordering::SeqCst);
    });
}

fn commit(text: &str, keep: bool, log: &Log, output: &Mutex<Output>) {
    let text = clean(text);
    if text.is_empty() {
        return;
    }
    let mut out = output.lock().unwrap();
    out.last = text.clone();
    if keep {
        write_log(log, &text);
        out.tail.push_back(text);
        if out.tail.len() > TAIL {
            out.tail.pop_front();
        }
    }
}

/// Splits output into lines: «\n» and «\r\n» end a line, a lone «\r» ends a progress redraw
/// (shown on the status line, not kept).
#[derive(Default)]
struct Lines {
    line: Vec<u8>,
    cr: bool,
}

impl Lines {
    fn feed(&mut self, bytes: &[u8]) -> Vec<(String, bool)> {
        let mut done = Vec::new();
        for &b in bytes {
            if self.cr {
                self.cr = false;
                if b == b'\n' {
                    done.push((self.take(), true));
                    continue;
                }
                done.push((self.take(), false));
            }
            match b {
                b'\n' => done.push((self.take(), true)),
                b'\r' => self.cr = true,
                _ => self.line.push(b),
            }
        }
        done
    }

    fn finish(&mut self) -> Option<(String, bool)> {
        let keep = !self.cr;
        let text = self.take();
        (!text.is_empty()).then_some((text, keep))
    }

    fn take(&mut self) -> String {
        let text = String::from_utf8_lossy(&self.line).into_owned();
        self.line.clear();
        text
    }
}

/// Without terminal colour codes and control characters.
pub fn clean(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\u{1b}' => match chars.peek() {
                Some('[') => {
                    chars.next();
                    for c in chars.by_ref() {
                        if ('\u{40}'..='\u{7e}').contains(&c) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == '\u{7}'
                            || (c == '\u{1b}' && chars.peek() == Some(&'\\'))
                        {
                            if c == '\u{1b}' {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            },
            '\t' => out.push(' '),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_end_on_newlines_and_progress_redraws_are_not_kept() {
        let mut lines = Lines::default();
        let mut got = lines.feed(b"one\r\ntwo\n 10%\r 20%\r");
        got.extend(lines.feed(b"\nthree"));
        got.extend(lines.finish());
        let want = [
            ("one", true),
            ("two", true),
            (" 10%", false),
            (" 20%", true),
            ("three", true),
        ];
        let got: Vec<(&str, bool)> =
            got.iter().map(|(t, k)| (t.as_str(), *k)).collect();
        assert_eq!(got, want);
        let mut split = Lines::default();
        let mut got = split.feed(b"a\r");
        got.extend(split.feed(b"\nb\r"));
        got.extend(split.feed(b"c"));
        got.extend(split.finish());
        let got: Vec<(&str, bool)> =
            got.iter().map(|(t, k)| (t.as_str(), *k)).collect();
        assert_eq!(got, [("a", true), ("b", false), ("c", true)]);
    }

    #[test]
    fn colour_codes_and_controls_are_removed() {
        assert_eq!(
            clean("\u{1b}[1m\u{1b}[32m   Compiling\u{1b}[0m tauri v1.7.1"),
            "Compiling tauri v1.7.1"
        );
        assert_eq!(clean("\u{1b}]0;title\u{7}done\tok"), "done ok");
        assert_eq!(clean("\u{1b}]8;;http://x\u{1b}\\link"), "link");
        assert_eq!(clean("  plain  "), "plain");
    }
}
