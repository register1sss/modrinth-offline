fn u3_edit_frame(game: &mut crate::game::Game<'_>) {
    use crate::game::u3_edit as m;
    if !game.native_window() {
        return;
    }
    let player = game.native_player().map(|p| m::Player {
        pos: m::Pos(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32),
        dimension: p.dimension.into(),
    });
    m::EDITOR.with(|e| e.borrow_mut().observe(player.as_ref()));
}
