//! What `transform` would decide about a Modrinth pack, read with HTTP ranges: the zip's end
//! record, its central directory and pack.mcmeta, never the whole archive. The catalog hides
//! packs that `pack_install` would refuse, so every listed pack can be installed.
use super::{compatible, safe_name, MAX_ENTRIES, MAX_EXPANDED, MAX_PACK};
use serde_json::Value;
use std::{collections::BTreeSet, future::Future, io::Read};

/// Read first: the end record behind the longest comment, and with it the whole central
/// directory of most packs, or the whole pack when it is small.
const TAIL: u64 = 128 << 10;
const MAX_DIRECTORY: u64 = 8 << 20;
const MAX_META: u64 = 1 << 20;
/// Room for a local header's own extra field, so one range usually holds header and data.
const LOCAL_SLACK: u64 = 4096;
const END: [u8; 4] = *b"PK\x05\x06";
const LOCATOR: [u8; 4] = *b"PK\x06\x07";
const END64: [u8; 4] = *b"PK\x06\x06";
const CENTRAL: [u8; 4] = *b"PK\x01\x02";
const LOCAL: [u8; 4] = *b"PK\x03\x04";

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    Installable,
    /// `pack_install` would refuse the pack: hidden from the catalog.
    Blocked(&'static str),
    /// The check could not decide (network, an unusual archive): the pack stays listed.
    Unknown(&'static str),
}
use Verdict::*;

fn u16_at(b: &[u8], i: usize) -> u64 {
    b.get(i..i + 2)
        .map_or(0, |s| u16::from_le_bytes([s[0], s[1]]) as u64)
}
fn u32_at(b: &[u8], i: usize) -> u64 {
    b.get(i..i + 4)
        .map_or(0, |s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]) as u64)
}
fn u64_at(b: &[u8], i: usize) -> Option<u64> {
    let mut a = [0u8; 8];
    a.copy_from_slice(b.get(i..i + 8)?);
    Some(u64::from_le_bytes(a))
}

/// The central directory as `ZipArchive::new` finds it.
#[derive(Debug, PartialEq, Eq)]
struct Directory {
    start: u64,
    len: u64,
    entries: u64,
    /// Bytes before the archive proper, added to every header offset.
    shift: u64,
}

/// The central directory's place, from the last bytes of a `size`-byte zip.
fn directory(tail: &[u8], size: u64) -> Result<Directory, Verdict> {
    let base = size
        .checked_sub(tail.len() as u64)
        .ok_or(Unknown("short read"))?;
    if tail.len() < 22 {
        return Err(Blocked("not a zip archive"));
    }
    // zip searches back from the end over the longest comment.
    let low = size.saturating_sub(22 + 0xFFFF).saturating_sub(base) as usize;
    let at = (low..=tail.len() - 22)
        .rev()
        .find(|&i| tail[i..i + 4] == END)
        .ok_or(Blocked("not a zip archive"))?;
    let end = &tail[at..];
    let (disk, cd_disk) = (u16_at(end, 4), u16_at(end, 6));
    let (on_disk, len, offset) =
        (u16_at(end, 8), u32_at(end, 12), u32_at(end, 16));
    let comment = u16_at(end, 20);
    if 22 + comment > end.len() as u64 {
        return Err(Blocked("truncated zip comment"));
    }
    let too_small = disk == 0xFFFF
        || cd_disk == 0xFFFF
        || on_disk == 0xFFFF
        || u16_at(end, 10) == 0xFFFF
        || len == 0xFFFF_FFFF
        || offset == 0xFFFF_FFFF;
    if !too_small && disk != cd_disk {
        return Err(Blocked("multi-disk zip"));
    }
    let at_abs = base + at as u64;
    // The ZIP64 locator sits 20 bytes before an end record whose comment ends the file.
    let locator = size
        .checked_sub(42 + comment)
        .filter(|&l| l >= base)
        .map(|l| &tail[(l - base) as usize..])
        .filter(|l| l[..4] == LOCATOR);
    let Some(locator) = locator else {
        let shift = at_abs
            .checked_sub(len)
            .and_then(|x| x.checked_sub(offset))
            .ok_or(Blocked("invalid central directory"))?;
        return Ok(Directory {
            start: offset + shift,
            len,
            entries: on_disk,
            shift,
        });
    };
    if !too_small && disk != u32_at(locator, 4) {
        return Err(Blocked("multi-disk zip"));
    }
    let nominal = u64_at(locator, 8).ok_or(Unknown("zip64"))?;
    let upper = at_abs.checked_sub(60).ok_or(Blocked("invalid zip64 end"))?;
    if nominal < base {
        return Err(Unknown("zip64"));
    }
    let pos = (nominal..=upper)
        .find(|&p| tail[(p - base) as usize..][..4] == END64)
        .ok_or(Blocked("invalid zip64 end"))?;
    let end64 = &tail[(pos - base) as usize..];
    if u32_at(end64, 16) != u32_at(end64, 20) {
        return Err(Blocked("multi-disk zip"));
    }
    let shift = pos - nominal;
    let field = |i| u64_at(end64, i).ok_or(Blocked("invalid zip64 end"));
    let (entries, len, offset) = (field(32)?, field(40)?, field(48)?);
    Ok(Directory {
        start: offset
            .checked_add(shift)
            .ok_or(Blocked("invalid zip64 end"))?,
        len,
        entries,
        shift,
    })
}

/// pack.mcmeta's entry, as `transform` picks it.
#[derive(Debug)]
struct Meta {
    header: u64,
    name_len: u64,
    method: u64,
    crc: u64,
    compressed: u64,
}

/// zip's names: UTF-8 when flagged, else code page 437, whose upper half has no control
/// characters (stand-ins keep the checks below the same).
fn name(raw: &[u8], utf8: bool) -> String {
    if utf8 {
        String::from_utf8_lossy(raw).into_owned()
    } else {
        raw.iter()
            .map(|&b| {
                if b < 0x80 {
                    b as char
                } else {
                    char::from_u32(0xE000 + b as u32).unwrap_or('?')
                }
            })
            .collect()
    }
}

/// Every entry checked like `transform` (paths, links, duplicates, sizes, readable methods),
/// then the pack.mcmeta it would read.
fn meta_entry(cd: &[u8], d: &Directory) -> Result<Meta, Verdict> {
    if d.entries > MAX_ENTRIES as u64 {
        return Err(Blocked("too many entries"));
    }
    let (mut at, mut total) = (0usize, 0u64);
    let mut names = BTreeSet::new();
    let mut metas = Vec::new();
    for _ in 0..d.entries {
        let h = cd.get(at..at + 46).ok_or(Unknown("short directory"))?;
        if h[..4] != CENTRAL {
            return Err(Blocked("invalid central directory"));
        }
        let (n, e, c) = (
            u16_at(h, 28) as usize,
            u16_at(h, 30) as usize,
            u16_at(h, 32) as usize,
        );
        let raw = cd
            .get(at + 46..at + 46 + n)
            .ok_or(Unknown("short directory"))?;
        let extra = cd
            .get(at + 46 + n..at + 46 + n + e)
            .ok_or(Unknown("short directory"))?;
        at += 46 + n + e + c;
        if at > cd.len() {
            return Err(Unknown("short directory"));
        }
        let (flags, method) = (u16_at(h, 8), u16_at(h, 10));
        let (mut compressed, mut size) = (u32_at(h, 20), u32_at(h, 24));
        let mut header = u32_at(h, 42);
        // ZIP64 sizes and offset, in zip's order.
        let mut i = 0;
        while i + 4 <= extra.len() {
            let (kind, len) = (u16_at(extra, i), u16_at(extra, i + 2) as usize);
            if kind == 1 {
                let mut j = i + 4;
                for field in [&mut size, &mut compressed, &mut header] {
                    if *field == 0xFFFF_FFFF {
                        if let Some(v) = u64_at(extra, j) {
                            *field = v;
                        }
                        j += 8;
                    }
                }
            }
            i += 4 + len;
        }
        if flags & 1 != 0 || method == 99 {
            return Err(Blocked("encrypted entry"));
        }
        if !matches!(method, 0 | 8 | 12 | 93) {
            return Err(Blocked("unsupported compression"));
        }
        let name = name(raw, flags & (1 << 11) != 0);
        let external = u32_at(h, 38);
        let link = external != 0
            && u16_at(h, 4) >> 8 == 3
            && (external >> 16) & 0o170000 == 0o120000;
        if !safe_name(&name) || link || !names.insert(name.to_lowercase()) {
            return Err(Blocked("unsafe or duplicate path"));
        }
        total = total.checked_add(size).ok_or(Blocked("too large"))?;
        if total > MAX_EXPANDED || size > MAX_PACK as u64 {
            return Err(Blocked("too large"));
        }
        if name == "pack.mcmeta" || name.ends_with("/pack.mcmeta") {
            let header = header
                .checked_add(d.shift)
                .ok_or(Blocked("invalid header offset"))?;
            let crc = u32_at(h, 16);
            let name_len = n as u64;
            metas.push((
                name,
                Meta {
                    header,
                    name_len,
                    method,
                    crc,
                    compressed,
                },
            ));
        }
    }
    let pick = metas
        .iter()
        .position(|(n, _)| n == "pack.mcmeta")
        .or((metas.len() == 1).then_some(0))
        .ok_or(Blocked("not one resource pack"))?;
    Ok(metas.swap_remove(pick).1)
}

/// pack.mcmeta read and judged as `transform` does.
fn judge(meta: &Meta, compressed: &[u8]) -> Verdict {
    let mut data = Vec::new();
    let read = match meta.method {
        0 => compressed.take(MAX_META + 1).read_to_end(&mut data),
        8 => flate2::read::DeflateDecoder::new(compressed)
            .take(MAX_META + 1)
            .read_to_end(&mut data),
        _ => return Unknown("compression"),
    };
    if read.is_err() {
        return Blocked("corrupt metadata");
    }
    if data.len() as u64 > MAX_META {
        return Blocked("metadata too large");
    }
    let mut crc = flate2::Crc::new();
    crc.update(&data);
    if crc.sum() as u64 != meta.crc {
        return Blocked("metadata checksum");
    }
    match serde_json::from_slice::<Value>(&data) {
        Ok(v) if !v["pack"].is_object() => Blocked("invalid metadata"),
        Ok(v) if !compatible(&v) => Blocked("format"),
        Ok(_) => Installable,
        Err(_) => Blocked("invalid metadata"),
    }
}

/// Judge a `size`-byte zip read through `fetch(start, end)`, which returns exactly those bytes.
pub(super) async fn check<F, Fut>(size: u64, fetch: F) -> Verdict
where
    F: Fn(u64, u64) -> Fut,
    Fut: Future<Output = Option<Vec<u8>>>,
{
    let base = size.saturating_sub(TAIL);
    let Some(tail) = fetch(base, size).await else {
        return Unknown("download");
    };
    // Ranges inside the tail are not fetched again.
    let get = |start: u64, end: u64| {
        let held = (start >= base && end <= size).then(|| {
            tail[(start - base) as usize..(end - base) as usize].to_vec()
        });
        let fetched = held.is_none().then(|| fetch(start, end));
        async move {
            match fetched {
                Some(f) => f.await,
                None => held,
            }
        }
    };
    let d = match directory(&tail, size) {
        Ok(d) => d,
        Err(v) => return v,
    };
    if d.len > MAX_DIRECTORY {
        return Unknown("large directory");
    }
    if d.start.checked_add(d.len).map_or(true, |end| end > size) {
        return Blocked("invalid central directory");
    }
    let Some(cd) = get(d.start, d.start + d.len).await else {
        return Unknown("download");
    };
    let meta = match meta_entry(&cd, &d) {
        Ok(m) => m,
        Err(v) => return v,
    };
    if meta.header >= size {
        return Blocked("invalid local header");
    }
    // Deflate hardly grows data, so a larger entry holds more than the 1 MiB transform accepts.
    if meta.compressed > 2 * MAX_META {
        return Blocked("metadata too large");
    }
    let guess =
        meta.header + 30 + meta.name_len + LOCAL_SLACK + meta.compressed;
    let Some(local) = get(meta.header, guess.min(size)).await else {
        return Unknown("download");
    };
    if local.len() < 30 || local[..4] != LOCAL {
        return Blocked("invalid local header");
    }
    let start = 30 + u16_at(&local, 26) + u16_at(&local, 28);
    let end = start + meta.compressed;
    if meta.header + end > size {
        return Blocked("truncated entry");
    }
    let data = if end <= local.len() as u64 {
        local[start as usize..end as usize].to_vec()
    } else {
        match get(meta.header + start, meta.header + end).await {
            Some(d) => d,
            None => return Unknown("download"),
        }
    };
    judge(&meta, &data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::Cell,
        io::{Cursor, Write},
    };
    const GOOD: &str = r#"{"pack":{"min_format":[97,0],"max_format":[97,1],"description":"t"}}"#;

    fn zip_of(files: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data, stored) in files {
            let method = if *stored {
                zip::CompressionMethod::Stored
            } else {
                zip::CompressionMethod::Deflated
            };
            z.start_file(
                *name,
                zip::write::FileOptions::default().compression_method(method),
            )
            .unwrap();
            z.write_all(data).unwrap();
        }
        z.finish().unwrap().into_inner()
    }
    /// The verdict over in-memory bytes, and how many ranges it fetched.
    fn verdict(data: &[u8]) -> (Verdict, usize) {
        let calls = Cell::new(0);
        let v = tokio_test_block_on(check(data.len() as u64, |s, e| {
            calls.set(calls.get() + 1);
            let part = data.get(s as usize..e as usize).map(<[u8]>::to_vec);
            async move { part }
        }));
        (v, calls.get())
    }
    fn tokio_test_block_on<F: Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }
    /// Noise that deflate cannot shrink, so archives outgrow the tail.
    fn noise(len: usize) -> Vec<u8> {
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        (0..len)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x as u8
            })
            .collect()
    }
    /// The range check agrees with the full `transform` that installing runs.
    fn agrees(data: &[u8]) -> Verdict {
        let (v, _) = verdict(data);
        if !matches!(v, Unknown(_)) {
            assert_eq!(
                v == Installable,
                super::super::transform(data, true, false).is_ok(),
                "{v:?}"
            );
        }
        v
    }

    #[test]
    fn small_compatible_packs_need_one_range() {
        let data = zip_of(&[
            ("pack.mcmeta", GOOD.as_bytes(), false),
            ("assets/a.txt", b"a", false),
        ]);
        assert_eq!(verdict(&data), (Installable, 1));
        assert_eq!(
            agrees(&zip_of(&[("pack.mcmeta", GOOD.as_bytes(), true)])),
            Installable
        );
        assert_eq!(
            agrees(&zip_of(&[
                ("Pack/pack.mcmeta", GOOD.as_bytes(), false),
                ("Pack/a", b"a", false)
            ])),
            Installable
        );
    }

    #[test]
    fn refused_packs_are_blocked() {
        for (files, why) in [
            (
                vec![(
                    "pack.mcmeta",
                    &br#"{"pack":{"pack_format":75}}"#[..],
                    false,
                )],
                "format",
            ),
            (
                vec![(
                    "pack.mcmeta",
                    &br#"{"pack":{"min_format":97.1,"max_format":97.1}}"#[..],
                    false,
                )],
                "format",
            ),
            (
                vec![(
                    "pack.mcmeta",
                    &b"\xEF\xBB\xBF{\"pack\":{\"pack_format\":97}}"[..],
                    false,
                )],
                "invalid metadata",
            ),
            (
                vec![("pack.mcmeta", &br#"{"pack":3}"#[..], false)],
                "invalid metadata",
            ),
            (
                vec![("assets/a", &b"a"[..], false)],
                "not one resource pack",
            ),
            (
                vec![
                    ("a/pack.mcmeta", GOOD.as_bytes(), false),
                    ("b/pack.mcmeta", GOOD.as_bytes(), false),
                ],
                "not one resource pack",
            ),
            (
                vec![
                    ("pack.mcmeta", GOOD.as_bytes(), false),
                    ("a\\b", &b"a"[..], false),
                ],
                "unsafe or duplicate path",
            ),
            (
                vec![
                    ("pack.mcmeta", GOOD.as_bytes(), false),
                    ("A", &b"a"[..], false),
                    ("a", &b"a"[..], false),
                ],
                "unsafe or duplicate path",
            ),
        ] {
            assert_eq!(agrees(&zip_of(&files)), Blocked(why));
        }
        assert_eq!(verdict(b"not a zip").0, Blocked("not a zip archive"));
        assert_eq!(verdict(&[]).0, Blocked("not a zip archive"));
    }

    #[test]
    fn large_packs_are_read_by_ranges_wherever_the_metadata_is() {
        let big = noise(600 << 10);
        let first = zip_of(&[
            ("pack.mcmeta", GOOD.as_bytes(), false),
            ("assets/big.bin", &big, true),
        ]);
        let (v, calls) = verdict(&first);
        assert_eq!((v, calls), (Installable, 2));
        assert_eq!(agrees(&first), Installable);
        let last = zip_of(&[
            ("assets/big.bin", &big, true),
            ("pack.mcmeta", GOOD.as_bytes(), false),
        ]);
        assert_eq!(verdict(&last), (Installable, 1));
        let bad = zip_of(&[
            ("pack.mcmeta", &br#"{"pack":{"pack_format":34}}"#[..], false),
            ("assets/big.bin", &big, true),
        ]);
        assert_eq!(agrees(&bad), Blocked("format"));
    }

    #[test]
    fn prepended_bytes_are_followed_like_zip_does() {
        let mut data = b"junk before the archive".to_vec();
        data.extend(zip_of(&[
            ("pack.mcmeta", GOOD.as_bytes(), false),
            ("assets/a", b"a", false),
        ]));
        assert_eq!(agrees(&data), Installable);
    }

    #[test]
    fn failed_ranges_keep_the_pack_listed() {
        let data = zip_of(&[("pack.mcmeta", GOOD.as_bytes(), false)]);
        let v = tokio_test_block_on(check(data.len() as u64, |_, _| async {
            None
        }));
        assert_eq!(v, Unknown("download"));
    }
}
