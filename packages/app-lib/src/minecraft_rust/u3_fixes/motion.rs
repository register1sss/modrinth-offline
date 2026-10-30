fn u3_fixes_motion(
    game: &mut crate::game::Game<'_>,
    id: i32,
    velocity: [f64; 3],
) {
    if let Some(t) = game.tridents.iter_mut().find(|t| t.id == id as u32) {
        crate::game::u3_fixes::authoritative_motion(t, velocity);
    }
}
