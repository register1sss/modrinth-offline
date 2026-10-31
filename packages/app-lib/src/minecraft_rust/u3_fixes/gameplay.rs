// U3 Fixes: exact-revision repairs. No world format or network protocol changes.
use super::{Game, Screen};
use horizon_client::containers::BlockEntity;
use horizon_core::geometry::Vec3;
use horizon_protocol::packets::play::{self, Serverbound as S};

// remote_handle_input used to return for menus before release_using ran.
// Send one release before clearing local animation, including the integrated server.
pub(super) fn stop_menu_use(game: &mut Game<'_>) {
    if game.remote.is_none() || game.screen == Screen::Playing {
        return;
    }
    let hand = game.used_hand();
    if let Some((slot, ticks)) = game.using.take() {
        game.send(S::PlayerAction(play::PlayerAction {
            action: 6,
            pos: [0; 3],
            direction: 0,
            sequence: 0,
        }));
        // Same client-side Riptide release as RemoteInputWorld::release_using.
        if let Some(hand) = hand
            && game
                .stack_in(slot)
                .is_some_and(|s| &*s.item == "minecraft:trident")
        {
            game.with_hand(hand, |g| g.release_trident(ticks));
        }
    }
}

pub(super) fn has_ridden_chest(game: &Game<'_>) -> bool {
    game.riding
        .is_some_and(|id| game.boats.iter().any(|b| b.id == id && b.chest))
}

// HasCustomInventoryScreen includes chest boats, not just tamed living mounts.
// The client still requests the server's menu; it never invents remote slots.
pub(super) fn open_ridden_chest(game: &mut Game<'_>) -> bool {
    if game.remote.is_some() || game.spectator() {
        return false;
    }
    let Some(boat) = game
        .riding
        .and_then(|id| game.boats.iter().find(|b| b.id == id && b.chest))
    else {
        return false;
    };
    let key = (i32::MAX - 15, super::minecarts::CART_KEY_Y, boat.id as i32);
    let items = boat.items.clone();
    // An already open menu owns the latest slots. Do not overwrite them with
    // the vehicle's older snapshot if another player opens the same chest.
    if !game.world.block_entities.contains_key(&key) {
        game.world
            .block_entities
            .insert(key, BlockEntity::Chest(items));
    }
    game.station = None;
    game.craft_size = 2;
    game.container = Some(key);
    game.screen = Screen::Inventory;
    game.capture(false);
    game.right = false;
    true
}

// Shift while entering a straw bed must not immediately destroy it.
pub(super) fn tick_sleep(game: &mut Game<'_>) {
    game.sleep_timer =
        super::tick_sleep_timer(game.sleep_timer, game.sleeping.is_some());
    if let Some(head) = game.sleeping {
        // LivingEntity.tick's checkBedExists: a sleeper whose bed is
        // gone gets up.
        let bed = game.world.block(head[0], head[1], head[2]);
        let bed_gone = !game.res.facts().get(bed).name.ends_with("_bed");
        // Vanilla has no Shift wake-up: the chat screen's capitals must
        // not get the sleeper up.
        if bed_gone
            || game
                .player
                .as_ref()
                .is_none_or(|p| p.health.dead_or_dying())
        {
            game.wake_up();
            return;
        }
        if let Some(p) = game.player.as_mut() {
            p.position = Vec3::new(
                f64::from(head[0]) + 0.5,
                // setPosToBed: the bed's top (getSleepHeight) and 0.125 above it.
                f64::from(head[1]) + 0.5625 + 0.125,
                f64::from(head[2]) + 0.5,
            );
            p.velocity = Vec3::ZERO;
        }
    }
    // ServerLevel.tick: once enough of the players have slept 100 ticks,
    // skip to morning if `advance_time` is on, wake everyone, and
    // resetWeatherCycle if `advance_weather` is on and it is raining.
    if game.enough_slept() {
        if game.ui.rules.bool("advance_time") {
            let t = game.world.day_time;
            game.world.day_time = t + 24000 - t.rem_euclid(24000);
        }
        // Level.isRaining: the rain level is above 0.2.
        if game.ui.rules.bool("advance_weather")
            && game.world.weather.levels(1.0).0 > 0.2
        {
            game.world.weather.raining = false;
            game.world.weather.thundering = false;
            game.world.weather.rain_time = 0;
            game.world.weather.thunder_time = 0;
        }
        if game.sleeping.is_some() {
            game.wake_up();
        }
        game.wake_guests();
        // SleepStatus.removeAllSleepers: waking everyone is not announced.
        game.sleep_seen.1 = 0;
    }
}
