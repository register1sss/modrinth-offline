fn u3_sleep(game: &mut crate::game::Game<'_>) -> core::ops::ControlFlow<()> {
    crate::game::u3_gameplay::tick_sleep(game);
    core::ops::ControlFlow::Break(())
}
