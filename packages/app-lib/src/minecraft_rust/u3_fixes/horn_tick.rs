fn u3_horn_tick(
    game: &mut crate::game::Game<'_>,
) -> core::ops::ControlFlow<()> {
    if crate::game::u3_horn::tick(game) {
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(())
}
