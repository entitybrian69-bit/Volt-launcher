fn used(
    game: &crate::game::Game<'_>,
    hand: volt_client::first_person::Hand,
    value: bool,
) -> bool {
    crate::game::punchy::used(game, hand, value);
    value
}
