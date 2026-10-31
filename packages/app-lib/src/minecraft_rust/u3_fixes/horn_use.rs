fn u3_horn_use(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
) -> core::ops::ControlFlow<Result<bool>> {
    if &*stack.item == "minecraft:goat_horn" {
        return core::ops::ControlFlow::Break(Ok(crate::game::u3_horn::start(
            game, stack, true,
        )));
    }
    core::ops::ControlFlow::Continue(())
}
