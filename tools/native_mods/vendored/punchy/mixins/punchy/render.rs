fn render_punchy(
    game: &mut crate::game::Game<'_>,
    alpha: f64,
) -> std::ops::ControlFlow<Vec<Vertex>> {
    crate::game::punchy::begin(game, alpha);
    if crate::game::punchy::ready(game) {
        let mut out = crate::game::punchy_geometry::render(game, alpha);
        crate::game::punchy::arms(game, alpha, &mut out);
        for vertex in &mut out {
            vertex.layer &= !horizon_client::gfx::OIT_ENTITY;
        }
        std::ops::ControlFlow::Break(out)
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
