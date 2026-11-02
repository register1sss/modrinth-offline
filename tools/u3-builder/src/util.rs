//! Small helpers: durations, sizes, timestamps, hashes and folder removal.

use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// «1 ч 05 мин», «12 мин 30 с», «45 с».
pub fn duration(d: Duration) -> String {
    let s = d.as_secs();
    match (s / 3600, s % 3600 / 60, s % 60) {
        (0, 0, sec) => format!("{sec} с"),
        (0, min, sec) => format!("{min} мин {sec:02} с"),
        (h, min, _) => format!("{h} ч {min:02} мин"),
    }
}

/// «04:31» or «1:04:31».
pub fn clock(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

pub fn mb(bytes: u64) -> String {
    format!("{:.1} МБ", bytes as f64 / 1_048_576.0)
}

pub fn gb(bytes: u64) -> String {
    format!("{:.1} ГБ", bytes as f64 / 1_073_741_824.0)
}

/// UTC «2026-11-02_09-41-07», for file names.
pub fn stamp(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (days, rest) = ((secs / 86_400) as i64, secs % 86_400);
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}_{:02}-{:02}-{:02}",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

pub fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// fs::rename, retried for a few seconds: right after unpacking, an antivirus scan can hold
/// the files.
pub fn rename(from: &Path, to: &Path) -> io::Result<()> {
    let mut tries = 0;
    loop {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(_) if tries < 20 => {
                tries += 1;
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(e) => return Err(e),
        }
    }
}

/// remove_dir_all that also takes read-only files; a missing folder is fine.
pub fn remove_tree(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if fs::remove_dir_all(path).is_ok() {
        return Ok(());
    }
    clear_readonly(path);
    fs::remove_dir_all(path)
}

fn clear_readonly(path: &Path) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        match entry.file_type() {
            Ok(t) if t.is_dir() => clear_readonly(&p),
            _ => writable(&p),
        }
    }
}

pub fn writable(path: &Path) {
    if let Ok(meta) = fs::metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            let _ = fs::set_permissions(path, perms);
        }
    }
}

/// Free space for the folder (or its nearest existing parent), when Windows can tell.
#[cfg(windows)]
pub fn free_space(path: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let mut dir = path;
    while !dir.exists() {
        dir = dir.parent()?;
    }
    let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut free = 0u64;
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    (ok != 0).then_some(free)
}

#[cfg(not(windows))]
pub fn free_space(_path: &Path) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_read_naturally() {
        assert_eq!(duration(Duration::from_secs(45)), "45 с");
        assert_eq!(duration(Duration::from_secs(750)), "12 мин 30 с");
        assert_eq!(duration(Duration::from_secs(3900)), "1 ч 05 мин");
        assert_eq!(clock(Duration::from_secs(271)), "04:31");
        assert_eq!(clock(Duration::from_secs(3871)), "1:04:31");
        assert_eq!(mb(1_572_864), "1.5 МБ");
    }

    #[test]
    fn stamps_are_utc_calendar_dates() {
        assert_eq!(stamp(UNIX_EPOCH), "1970-01-01_00-00-00");
        let t = UNIX_EPOCH + Duration::from_secs(1_793_612_467);
        assert_eq!(stamp(t), "2026-11-02_09-41-07");
        let leap = UNIX_EPOCH + Duration::from_secs(1_709_164_800);
        assert_eq!(stamp(leap), "2024-02-29_00-00-00");
    }
}
