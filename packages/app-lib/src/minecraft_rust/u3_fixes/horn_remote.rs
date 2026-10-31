fn u3_horn_remote(
    game: &mut crate::game::Game<'_>,
    held: &horizon_client::items::ItemStack,
) -> core::ops::ControlFlow<()> {
    if &*held.item == "minecraft:goat_horn" {
        crate::game::u3_horn::start(game, held, false);
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(())
}
