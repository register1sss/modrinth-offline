fn u3_edit_command(
    game: &mut crate::game::Game<'_>,
    command: &str,
    ran: &mut bool,
) {
    use crate::game::u3_edit as m;
    let mut parts = command.splitn(2, char::is_whitespace);
    if parts.next() != Some("u3edit") || !game.native_on(m::ID) {
        return;
    }
    *ran = true;
    let player = game.native_player().map(|p| m::Player {
        pos: m::Pos(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32),
        dimension: p.dimension.into(),
    });
    let result = m::EDITOR.with(|e| {
        e.borrow_mut()
            .handle(parts.next().unwrap_or("help"), player.as_ref())
    });
    // Same route as clicking a vanilla chat command: the integrated/remote server is authoritative.
    // Never run as console, change permissions or write chunk data behind the server.
    for line in result.commands {
        game.click_command(&line);
    }
    game.native_chat(&result.message);
}
