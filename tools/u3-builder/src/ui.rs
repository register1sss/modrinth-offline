//! Console output: steps, one live status line and prompts, all mirrored to the log file.

use std::fs::File;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type Log = Arc<Mutex<Option<File>>>;

pub fn write_log(log: &Log, text: &str) {
    if let Ok(mut file) = log.lock() {
        if let Some(file) = file.as_mut() {
            let _ = writeln!(file, "{text}");
        }
    }
}

pub struct Ui {
    log: Log,
    interactive: AtomicBool,
    terminal: bool,
    status_chars: AtomicUsize,
    heartbeat: Mutex<Instant>,
}

impl Ui {
    pub fn new() -> Ui {
        Ui {
            log: Arc::new(Mutex::new(None)),
            interactive: AtomicBool::new(io::stdin().is_terminal()),
            terminal: io::stdout().is_terminal(),
            status_chars: AtomicUsize::new(0),
            heartbeat: Mutex::new(Instant::now()),
        }
    }

    pub fn set_interactive(&self, on: bool) {
        self.interactive
            .store(on && io::stdin().is_terminal(), Ordering::Relaxed);
    }

    pub fn interactive(&self) -> bool {
        self.interactive.load(Ordering::Relaxed)
    }

    pub fn open_log(&self, path: &Path) -> io::Result<()> {
        let file = File::create(path)?;
        *self.log.lock().unwrap() = Some(file);
        Ok(())
    }

    pub fn log(&self) -> Log {
        self.log.clone()
    }

    pub fn log_line(&self, text: &str) {
        write_log(&self.log, text);
    }

    pub fn say(&self, text: &str) {
        self.clear_status();
        println!("{text}");
        self.log_line(text);
    }

    pub fn step(&self, n: usize, total: usize, text: &str) {
        self.say("");
        self.say(&format!("[{n}/{total}] {text}"));
    }

    pub fn ok(&self, text: &str) {
        self.say(&format!("    √ {text}"));
    }

    pub fn info(&self, text: &str) {
        self.say(&format!("      {text}"));
    }

    pub fn warn(&self, text: &str) {
        self.say(&format!("    ! {text}"));
    }

    pub fn error(&self, text: &str) {
        self.say("");
        self.say(&format!("ОШИБКА: {text}"));
    }

    /// The live status line: redrawn in place in a console, a line a minute otherwise.
    pub fn status(&self, text: &str) {
        if !self.terminal {
            let mut beat = self.heartbeat.lock().unwrap();
            if beat.elapsed() >= Duration::from_secs(60) {
                *beat = Instant::now();
                println!("      … {}", text.trim());
            }
            return;
        }
        let width = console_width().saturating_sub(1).max(20);
        let line: String = text.chars().take(width).collect();
        let chars = line.chars().count();
        let old = self.status_chars.swap(chars, Ordering::Relaxed);
        print!("\r{line}{}", " ".repeat(old.saturating_sub(chars)));
        let _ = io::stdout().flush();
    }

    pub fn clear_status(&self) {
        let old = self.status_chars.swap(0, Ordering::Relaxed);
        if old > 0 && self.terminal {
            print!("\r{}\r", " ".repeat(old));
            let _ = io::stdout().flush();
        }
    }

    /// A line typed by the user, or None when nobody can answer.
    pub fn ask(&self, prompt: &str) -> Option<String> {
        if !self.interactive() {
            return None;
        }
        self.clear_status();
        print!("{prompt}");
        let _ = io::stdout().flush();
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => {
                self.log_line(&format!("{prompt}{}", line.trim_end()));
                Some(line.trim().to_string())
            }
        }
    }

    pub fn confirm(&self, prompt: &str, default: bool) -> bool {
        let hint = if default { "Д/н" } else { "д/Н" };
        loop {
            let Some(answer) = self.ask(&format!("{prompt} [{hint}]: ")) else {
                return default;
            };
            if answer.is_empty() {
                return default;
            }
            if let Some(choice) = yes_no(&answer) {
                return choice;
            }
            self.say("      Ответь «д» (да) или «н» (нет).");
        }
    }
}

/// «д»/«да»/y/yes, «н»/«нет»/n/no, also typed with the other keyboard layout.
pub fn yes_no(answer: &str) -> Option<bool> {
    match answer.trim().to_lowercase().as_str() {
        "д" | "да" | "y" | "yes" | "l" | "lf" => Some(true),
        "н" | "нет" | "n" | "no" | "т" | "ytn" => Some(false),
        _ => None,
    }
}

#[cfg(windows)]
fn console_width() -> usize {
    use windows_sys::Win32::System::Console::{
        GetConsoleScreenBufferInfo, GetStdHandle, CONSOLE_SCREEN_BUFFER_INFO,
        STD_OUTPUT_HANDLE,
    };
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
        if GetConsoleScreenBufferInfo(handle, &mut info) != 0 {
            let width =
                info.srWindow.Right as i32 - info.srWindow.Left as i32 + 1;
            if width > 0 {
                return width as usize;
            }
        }
    }
    100
}

#[cfg(not(windows))]
fn console_width() -> usize {
    100
}

/// Whether this console window is ours alone (started from Explorer, not from a terminal),
/// so it should stay open at the end until the user has read it.
#[cfg(windows)]
pub fn owns_console() -> bool {
    use windows_sys::Win32::System::Console::GetConsoleProcessList;
    let mut ids = [0u32; 4];
    unsafe { GetConsoleProcessList(ids.as_mut_ptr(), ids.len() as u32) == 1 }
}

#[cfg(not(windows))]
pub fn owns_console() -> bool {
    false
}

#[cfg(windows)]
pub fn set_title(title: &str) {
    use windows_sys::Win32::System::Console::SetConsoleTitleW;
    let wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
    unsafe {
        SetConsoleTitleW(wide.as_ptr());
    }
}

#[cfg(not(windows))]
pub fn set_title(_title: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_in_either_layout() {
        for yes in ["д", "Да", "y", "YES", "l"] {
            assert_eq!(yes_no(yes), Some(true), "{yes}");
        }
        for no in ["н", "нет", "n", "No", "т"] {
            assert_eq!(yes_no(no), Some(false), "{no}");
        }
        assert_eq!(yes_no("может"), None);
    }
}
