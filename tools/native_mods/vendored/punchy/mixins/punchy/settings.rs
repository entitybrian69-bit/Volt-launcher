fn settings_punchy(game:&crate::game::Game<'_>,mod_id:&str,buttons:&mut Vec<String>) {
    if mod_id=="punchy" {crate::game::punchy::settings(game,buttons);}
}
