fn u3_boat_detect(
    game: &crate::game::Game<'_>,
) -> core::ops::ControlFlow<bool> {
    if crate::game::u3_gameplay::has_ridden_chest(game) {
        return core::ops::ControlFlow::Break(true);
    }
    core::ops::ControlFlow::Continue(())
}
