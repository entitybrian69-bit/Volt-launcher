fn click_punchy(game:&crate::game::Game<'_>,mod_id:&str,button:usize) {
    if mod_id=="punchy" {crate::game::punchy::click(game,button);}
}
