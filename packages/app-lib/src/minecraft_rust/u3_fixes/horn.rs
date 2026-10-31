use super::{Game, Screen};
use horizon_client::items::ItemStack;
use horizon_world::nbt::Tag;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub(super) struct Instrument {
    pub sound: String,
    pub ticks: u32,
    pub volume: f32,
}
fn number(tag: &Tag) -> Option<f64> {
    match tag {
        Tag::Int(v) => Some(f64::from(*v)),
        Tag::Float(v) => Some(f64::from(f32::from_bits(*v))),
        Tag::Double(v) => Some(f64::from_bits(*v)),
        _ => None,
    }
}
// Both registry instruments and inline ItemStack components use this codec.
pub(super) fn decode(tag: &Tag) -> Option<Instrument> {
    let sound = tag.get("sound_event")?;
    let sound = match sound {
        Tag::String(s) => s.to_string_lossy(),
        tag => match tag.get("sound_id")? {
            Tag::String(s) => s.to_string_lossy(),
            _ => return None,
        },
    };
    let seconds = number(tag.get("use_duration")?)?;
    let range = number(tag.get("range")?)?;
    // Reject malformed/network-provided values instead of overflowing timers
    // or passing non-finite gains into the audio backend.
    if sound.is_empty()
        || sound.len() > 512
        || !seconds.is_finite()
        || !(0.05..=3600.0).contains(&seconds)
        || !range.is_finite()
        || !(0.0..=16384.0).contains(&range)
    {
        return None;
    }
    Some(Instrument {
        sound,
        ticks: (seconds * 20.0) as u32,
        volume: (range / 16.0) as f32,
    })
}
fn resolve(game: &Game<'_>, stack: &ItemStack) -> Option<Instrument> {
    let component = stack.data().and_then(|d| d.other("minecraft:instrument"));
    let id = match component {
        Some(Tag::String(id)) => id.to_string_lossy(),
        Some(inline) => return decode(inline),
        None => "minecraft:ponder_goat_horn".to_owned(),
    };
    let id = if id.contains(':') {
        id
    } else {
        format!("minecraft:{id}")
    };
    // A remote server may provide its own instrument; prefer its data.
    if let Some(data) = game
        .remote
        .as_ref()
        .and_then(|r| r.registries.registries.get("minecraft:instrument"))
        .and_then(|entries| entries.iter().find(|(name, _)| name == &id))
        .and_then(|(_, data)| data.as_ref())
    {
        return decode(data);
    }
    // Known-pack registry entries omit their data on the wire. Resolve those
    // from the pinned resource registry, not a hard-coded sound-number table.
    let sync =
        super::lan::sync_data(&game.options.resources, &game.pack_files())
            .ok()?;
    let data = horizon_client::lan::data::u3_instruments::lookup(&sync, &id)?;
    decode(&data)
}
pub(super) fn start(
    game: &mut Game<'_>,
    stack: &ItemStack,
    sound: bool,
) -> bool {
    if game.using.is_some() {
        return true;
    }
    let Some(instrument) = resolve(game, stack) else {
        return false;
    };
    let default = game
        .items()
        .get(&stack.item)
        .and_then(|i| i.use_cooldown.as_ref());
    let group = horizon_client::cooldowns::group(stack, default);
    if game.cooldowns.is_on_cooldown(&group) {
        return false;
    }
    game.using = Some((game.held_slot(), 0));
    game.cooldowns
        .add(Arc::clone(&group), instrument.ticks as i32);
    if game.lan_acting.is_some() {
        game.guest_cooldowns.push((group, instrument.ticks as i32));
    }
    if sound {
        let at = game
            .player
            .as_ref()
            .map(|p| [p.position.x, p.position.y, p.position.z]);
        game.audio
            .play(&instrument.sound, at, instrument.volume, 1.0);
        game.stats.add("used", &stack.item, 1);
        if let Some(at) = at {
            let ctx = game.who_context(game.sculk_actor(), None);
            game.post_game_event("minecraft:instrument_play", at, ctx);
        }
    }
    true
}
pub(super) fn tick(game: &mut Game<'_>) -> bool {
    let Some((slot, ticks)) = game.using else {
        return false;
    };
    let Some(stack) = game.stack_in(slot).cloned() else {
        return false;
    };
    if &*stack.item != "minecraft:goat_horn" {
        return false;
    }
    let held = slot == game.selected || slot == super::shield::OFFHAND_SLOT;
    game.using = resolve(game, &stack).and_then(|i| {
        (held
            && game.right
            && game.screen == Screen::Playing
            && ticks + 1 < i.ticks)
            .then_some((slot, ticks + 1))
    });
    true
}
