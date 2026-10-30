// U3 Edit 0.1.0 - a small original region editor, not WorldEdit or a port of its code.
// Pure planning logic. The adapter sends vanilla commands with the player's permissions.
use std::time::{Duration, Instant};
pub const ID: &str = "u3_edit";
pub const LIMIT: i64 = 32_768;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos(pub i32, pub i32, pub i32);
#[derive(Clone, Debug)]
pub struct Player {
    pub pos: Pos,
    pub dimension: String,
}
#[derive(Default)]
pub struct Editor {
    first: Option<Pos>,
    second: Option<Pos>,
    dimension: Option<String>,
    pending: Option<Plan>,
}
struct Plan {
    commands: Vec<String>,
    expires: Instant,
}
#[derive(Default, Debug)]
pub struct Reply {
    pub message: String,
    pub commands: Vec<String>,
}
fn reply(message: impl Into<String>) -> Reply {
    Reply {
        message: message.into(),
        commands: Vec::new(),
    }
}
impl Editor {
    pub fn observe(&mut self, player: Option<&Player>) {
        let dimension = player.map(|p| p.dimension.as_str());
        if self.dimension.as_deref() != dimension {
            self.clear();
            self.dimension = dimension.map(str::to_owned);
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|p| Instant::now() > p.expires)
        {
            self.pending = None;
        }
    }
    pub fn clear(&mut self) {
        self.first = None;
        self.second = None;
        self.pending = None;
    }
    pub fn handle(&mut self, args: &str, player: Option<&Player>) -> Reply {
        self.observe(player);
        let words: Vec<_> = args.split_whitespace().collect();
        let verb = words.first().copied().unwrap_or("help");
        if verb == "help" {
            return reply("U3 Edit: /u3edit pos1|pos2 [x y z], info, set <block>, replace <from> <to>, walls <block>, apply, cancel, clear. Limit 32768 blocks. Cheats/operator rights required. No undo: back up your world first.");
        }
        if matches!(verb, "clear" | "cancel") && words.len() == 1 {
            if verb == "clear" {
                self.clear();
            } else {
                self.pending = None;
            }
            return reply("U3 Edit: cleared.");
        }
        let Some(player) = player else {
            return reply("U3 Edit: enter a world first.");
        };
        if matches!(verb, "pos1" | "pos2") {
            let pos = match words.len() {
                1 => Ok(player.pos),
                4 => parse_pos(&words[1..]),
                _ => Err("Use pos1/pos2 or pos1/pos2 x y z"),
            };
            let pos = match pos
                .and_then(|p| valid_pos(p, &player.dimension).map(|()| p))
            {
                Ok(p) => p,
                Err(e) => return reply(format!("U3 Edit: {e}")),
            };
            if verb == "pos1" {
                self.first = Some(pos);
            } else {
                self.second = Some(pos);
            }
            self.pending = None;
            return reply(format!(
                "U3 Edit: {verb} = {} {} {}",
                pos.0, pos.1, pos.2
            ));
        }
        if verb == "apply" && words.len() == 1 {
            return match self.pending.take() {
                Some(plan) if Instant::now() <= plan.expires => Reply {
                    message: "U3 Edit: commands sent. The server checks permissions and reports the result. No undo.".into(), commands: plan.commands,
                },
                _ => reply("U3 Edit: no pending edit. Preview set, replace or walls first."),
            };
        }
        let (a, b, count) =
            match region(self.first, self.second, &player.dimension) {
                Ok(r) => r,
                Err(e) => return reply(format!("U3 Edit: {e}")),
            };
        if verb == "info" && words.len() == 1 {
            return reply(format!(
                "U3 Edit: {} {} {} to {} {} {}, {count} blocks.",
                a.0, a.1, a.2, b.0, b.1, b.2
            ));
        }
        // Never keep a previous destructive plan after a malformed new request.
        self.pending = None;
        let commands = match plan(&words, a, b) {
            Ok(c) => c,
            Err(e) => return reply(format!("U3 Edit: {e}")),
        };
        self.pending = Some(Plan {
            commands,
            expires: Instant::now() + Duration::from_secs(60),
        });
        reply(format!("U3 Edit preview: {verb}, region {count} blocks. No undo. /u3edit apply within 60 seconds, or /u3edit cancel."))
    }
}
fn parse_pos(words: &[&str]) -> Result<Pos, &'static str> {
    let read = |s: &str| {
        s.parse::<i32>()
            .map_err(|_| "Coordinates must be whole numbers")
    };
    Ok(Pos(read(words[0])?, read(words[1])?, read(words[2])?))
}
fn valid_pos(p: Pos, dimension: &str) -> Result<(), &'static str> {
    let (bottom, top) = match dimension {
        "minecraft:overworld" => (-64, 319),
        "minecraft:the_nether" | "minecraft:the_end" => (0, 255),
        _ => return Err("This dimension is not supported"),
    };
    if p.0.abs_diff(0) > 29_999_984
        || p.2.abs_diff(0) > 29_999_984
        || !(bottom..=top).contains(&p.1)
    {
        return Err("Position is outside the world bounds");
    }
    Ok(())
}
fn region(
    first: Option<Pos>,
    second: Option<Pos>,
    dimension: &str,
) -> Result<(Pos, Pos, i64), &'static str> {
    let (a, b) = (
        first.ok_or("Set pos1 first")?,
        second.ok_or("Set pos2 first")?,
    );
    valid_pos(a, dimension)?;
    valid_pos(b, dimension)?;
    let min = Pos(a.0.min(b.0), a.1.min(b.1), a.2.min(b.2));
    let max = Pos(a.0.max(b.0), a.1.max(b.1), a.2.max(b.2));
    let count = (i64::from(max.0) - i64::from(min.0) + 1)
        .checked_mul(i64::from(max.1) - i64::from(min.1) + 1)
        .and_then(|v| v.checked_mul(i64::from(max.2) - i64::from(min.2) + 1))
        .ok_or("Region is too large")?;
    if count > LIMIT {
        return Err("Region exceeds 32768 blocks");
    }
    Ok((min, max, count))
}
fn block(s: &str) -> Result<String, &'static str> {
    let s = if s.contains(':') {
        s.to_owned()
    } else {
        format!("minecraft:{s}")
    };
    let Some((namespace, path)) = s.split_once(':') else {
        return Err("Invalid block id");
    };
    if namespace.is_empty()
        || path.is_empty()
        || s.len() > 128
        || !namespace
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
        || !path.bytes().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || b"_./-".contains(&c)
        })
    {
        return Err("Use a plain block id, e.g. minecraft:stone. NBT and block-state syntax are not supported yet");
    }
    Ok(s)
}
fn fill(a: Pos, b: Pos, target: &str, suffix: &str) -> String {
    format!(
        "fill {} {} {} {} {} {} {target}{suffix}",
        a.0, a.1, a.2, b.0, b.1, b.2
    )
}
fn plan(words: &[&str], a: Pos, b: Pos) -> Result<Vec<String>, &'static str> {
    match words {
        ["set", target] => Ok(vec![fill(a, b, &block(target)?, "")]),
        ["replace", from, to] => Ok(vec![fill(
            a,
            b,
            &block(to)?,
            &format!(" replace {}", block(from)?),
        )]),
        ["walls", target] => {
            let target = block(target)?;
            Ok(vec![
                fill(a, Pos(b.0, b.1, a.2), &target, ""),
                fill(Pos(a.0, a.1, b.2), b, &target, ""),
                fill(a, Pos(a.0, b.1, b.2), &target, ""),
                fill(Pos(b.0, a.1, a.2), b, &target, ""),
            ])
        }
        _ => {
            Err("Use set <block>, replace <from> <to>, walls <block>, or help")
        }
    }
}
thread_local! { pub static EDITOR: std::cell::RefCell<Editor> = std::cell::RefCell::new(Editor::default()); }
#[cfg(test)]
mod tests {
    use super::*;
    fn player() -> Player {
        Player {
            pos: Pos(0, 64, 0),
            dimension: "minecraft:overworld".into(),
        }
    }
    fn selected() -> Editor {
        let mut e = Editor::default();
        e.handle("pos1 0 64 0", Some(&player()));
        e.handle("pos2 2 66 2", Some(&player()));
        e
    }
    #[test]
    fn fill_requires_preview_and_explicit_apply() {
        let mut e = selected();
        let r = e.handle("set stone", Some(&player()));
        assert!(r.commands.is_empty());
        let r = e.handle("apply", Some(&player()));
        assert_eq!(r.commands, ["fill 0 64 0 2 66 2 minecraft:stone"]);
        assert!(e.handle("apply", Some(&player())).commands.is_empty());
    }
    #[test]
    fn replace_and_walls() {
        let mut e = selected();
        e.handle("replace dirt stone", Some(&player()));
        assert_eq!(
            e.handle("apply", Some(&player())).commands,
            ["fill 0 64 0 2 66 2 minecraft:stone replace minecraft:dirt"]
        );
        e.handle("walls glass", Some(&player()));
        assert_eq!(e.handle("apply", Some(&player())).commands.len(), 4);
    }
    #[test]
    fn dimension_change_and_leaving_world_clear_state() {
        let mut e = selected();
        e.handle("set stone", Some(&player()));
        let mut p = player();
        p.dimension = "minecraft:the_nether".into();
        assert!(e.handle("apply", Some(&p)).commands.is_empty());
        e = selected();
        e.observe(None);
        assert!(e.handle("apply", Some(&player())).commands.is_empty());
    }
    #[test]
    fn malformed_new_plan_cancels_old_one() {
        let mut e = selected();
        e.handle("set stone", Some(&player()));
        e.handle("set stone;op", Some(&player()));
        assert!(e.handle("apply", Some(&player())).commands.is_empty());
    }
    #[test]
    fn cancel_and_selection_change_cancel_plan() {
        let mut e = selected();
        e.handle("set stone", Some(&player()));
        e.handle("cancel", Some(&player()));
        assert!(e.handle("apply", Some(&player())).commands.is_empty());
        e.handle("set stone", Some(&player()));
        e.handle("pos1", Some(&player()));
        assert!(e.handle("apply", Some(&player())).commands.is_empty());
    }
    #[test]
    fn bounds_and_large_regions() {
        assert!(region(
            Some(Pos(0, 0, 0)),
            Some(Pos(100, 100, 100)),
            "minecraft:overworld"
        )
        .is_err());
        assert!(valid_pos(Pos(i32::MIN, 64, 0), "minecraft:overworld").is_err());
        assert!(valid_pos(Pos(0, 320, 0), "minecraft:overworld").is_err());
        assert!(valid_pos(Pos(0, -1, 0), "minecraft:the_nether").is_err());
    }
    #[test]
    fn command_injection_is_rejected() {
        for s in [
            "stone\nkill @a",
            "stone;op",
            "stone[foo=bar]",
            "stone{a:1}",
            "#minecraft:all",
            "minecraft:",
        ] {
            assert!(block(s).is_err(), "{}", s);
        }
    }
    #[test]
    fn pending_plan_expires() {
        let mut e = selected();
        e.handle("set stone", Some(&player()));
        e.pending.as_mut().unwrap().expires =
            Instant::now() - Duration::from_secs(1);
        assert!(e.handle("apply", Some(&player())).commands.is_empty());
    }
    #[test]
    fn reversed_positions_normalize() {
        let r = region(
            Some(Pos(3, 66, 3)),
            Some(Pos(1, 64, 1)),
            "minecraft:overworld",
        )
        .unwrap();
        assert_eq!(r, (Pos(1, 64, 1), Pos(3, 66, 3), 27));
    }
}
