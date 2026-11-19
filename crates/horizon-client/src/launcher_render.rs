//! The renderer used by both native launchers and game startup.
use crate::{
    Result,
    gfx::{Frame, Renderer},
    hud::{HudAtlas, HudBatch},
};
use std::sync::Arc;
use winit::window::Window;

/// Everything the title and menus draw: the GUI atlas with its font and the
/// title panorama. Kept apart from the world bundle so the title can appear
/// before the block sprites, models and gameplay tables are ready.
pub struct UiResources {
    /// The GUI, font, particle, map and skin atlas.
    pub atlas: HudAtlas,
    /// The title screen's panorama faces.
    pub panorama: Arc<crate::gfx::panorama::Faces>,
}
impl UiResources {
    /// From a resource root layering packs over `defaults`, the game's own
    /// resources (`root` itself when no pack is on).
    pub fn load(root: &std::path::Path, defaults: &std::path::Path) -> Result<Self> {
        Ok(Self {
            atlas: HudAtlas::build_over(root, defaults)?,
            panorama: Arc::new(crate::gfx::panorama::Faces::load(root)),
        })
    }
    /// What a server's game holds, since it draws nothing: the font, which books
    /// and chat measure with, and a one-pixel panorama.
    pub fn server(root: &std::path::Path) -> Result<Self> {
        Ok(Self {
            atlas: HudAtlas::fonts(root)?,
            panorama: Arc::new(crate::gfx::panorama::Faces {
                size: 1,
                pixels: [0u8; 4].repeat(6),
            }),
        })
    }
}
/// The loading screen's renderer: the real HUD atlas and font over one opaque
/// white block-sprite layer, an empty border texture and the title panorama
/// once the menu bundle is ready.
pub fn loading_renderer(
    window: Arc<Window>,
    offscreen: bool,
    gui: Option<&UiResources>,
    previous: Option<&Renderer>,
    api: crate::graphics::GraphicsApi,
) -> Result<Renderer> {
    static FORCEFIELD: [u8; 4] = [0; 4];
    static EMPTY_HUD: [u8; 4] = [0, 0, 0, 255];
    static EMPTY_GLINT: [u8; 4] = [0; 4];
    let empty_panorama = crate::gfx::panorama::Faces {
        size: 1,
        pixels: [0u8; 4].repeat(6),
    };
    let panorama = gui.map_or(&empty_panorama, |gui| gui.panorama.as_ref());
    let atlas = gui.map(|gui| &gui.atlas);
    let (pixels, width, height) = match atlas {
        Some(atlas) => (atlas.pixels.as_slice(), atlas.width, atlas.height),
        None => (EMPTY_HUD.as_slice(), 1, 1),
    };
    let (glint_pixels, glint_width, glint_height) = match atlas {
        Some(atlas) => (atlas.glint.0.as_slice(), atlas.glint.1, atlas.glint.2),
        None => (EMPTY_GLINT.as_slice(), 1, 1),
    };
    let (glint_armor_pixels, glint_armor_width, glint_armor_height) = match atlas {
        Some(atlas) => (
            atlas.glint_armor.0.as_slice(),
            atlas.glint_armor.1,
            atlas.glint_armor.2,
        ),
        None => (EMPTY_GLINT.as_slice(), 1, 1),
    };
    // A glyph rasterized into an earlier texture is not in these pixels; the
    // new texture gets it again when the character is drawn (see
    // `build_renderer`).
    if let Some(atlas) = atlas {
        atlas.font.reset_cache();
    }
    let mut renderer = pollster::block_on(Renderer::new(
        window,
        offscreen,
        &[255, 255, 255, 255],
        1,
        1,
        1,
        &[],
        &[],
        (pixels, width, height),
        (glint_pixels, glint_width, glint_height),
        (glint_armor_pixels, glint_armor_width, glint_armor_height),
        (&FORCEFIELD[..], 1),
        panorama,
        None,
        previous,
        api,
    ))?;
    if let Some(atlas) = atlas {
        renderer.hud.sprite_animations = atlas
            .sprite_animations
            .iter()
            .map(|(rect, animation)| (*rect, animation.clone(), (usize::MAX, 0, 0)))
            .collect();
    }
    Ok(renderer)
}

/// A title-shaped frame: black clear and panorama, only the HUD is drawn.
pub fn loading_frame(hud: &HudBatch) -> Frame<'_> {
    Frame {
        weather: &[],
        weather_time: crate::weather::Time::new(0, 0.0),
        camera: glam::DVec3::new(0.5, 100.0, 0.5),
        yaw: 0.0,
        pitch: 10.0,
        fov_degrees: 85.0,
        bob: glam::Mat4::IDENTITY,
        spin: glam::Mat4::IDENTITY,
        sky: crate::environment::Sky::default(),
        distance: 12.0,
        detail_distance: 12.0,
        depth_far: 1000.0,
        underwater: false,
        cave_culling: false,
        title: true,
        sky_blocked: false,
        skybox: false,
        end_sky: false,
        end_sky_sprite: None,
        environment_fog: None,
        water_fog: [0.0; 3],
        fog: None,
        dark_disc: false,
        outline: Vec::new(),
        outline_after_terrain: false,
        outline_high_contrast: false,
        debug_lines: &[],
        debug_lines_on_top: &[],
        debug_crosshair: &[],
        hud,
        glint_alpha: 0.75,
        glint_ticks: 0.0,
        nether_lighting: false,
        entities: &[],
        entity_spans: &[],
        glowing: &[],
        dynamic_lights: &[],
        clouds: None,
        lightning: Default::default(),
        additive: &[],
        see_through: &[],
        celestial: &[],
        stars: None,
        hand: &[],
        hand_fov_degrees: 70.0,
        screen: &[],
        pack: Default::default(),
        border: None,
        game_time: 0.0,
        shader_game_time: 0.0,
        end_portal_sprites: [[0.0; 4]; 2],
        crumbling_sprite: [0.0; 4],
        ticks: 0,
        darkness: (0.0, 0.0),
        night_vision: 0.0,
        block_flicker: 0.0,
        boss_darken: 0.0,
        chunk_fade: 0.0,
        post_effects: crate::post_chain::Request::none(),
        menu_blur: None,
    }
}
