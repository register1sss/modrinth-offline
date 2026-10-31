fn u3_boat_menu(
    game: &mut crate::game::Game<'_>,
) -> core::ops::ControlFlow<bool> {
    if crate::game::u3_gameplay::open_ridden_chest(game) {
        return core::ops::ControlFlow::Break(true);
    }
    core::ops::ControlFlow::Continue(())
}
