fn mining(game: &crate::game::Game<'_>, start: bool) {
    let ready =
        start || (game.creative && game.remote.as_ref().is_some_and(|r| r.destroy_delay == 0));
    if ready && game.pick().is_some() {
        crate::game::punchy::mining_attempt(game);
    }
}
