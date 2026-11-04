//! Volt & Horizon Split-Pane Frosted-Glass Video / Graphics Settings (`graphics_ui`)
//! matching Image 1: left sidebar (`Volt / Video Settings`, `Search options`,
//! `General / Quality / Performance / Advanced / HDR` tabs, `Undo` + `Done`),
//! right frosted glass groups, bottom description + `Cost` dock, and smooth animations.

use std::time::Instant;

use crate::{
    graphics::{FarDetail, GraphicsApi, Overlay, Pacing, PresentMode, Preset, Upscaling},
    graphics_style::{self as style, Cost},
    hud::{HudAtlas, HudBatch},
    lang,
    settings::{Opt, Settings},
    text::Text,
    ui,
};

/// Compatibility entry level for `options_ui`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Simple,
    Advanced,
    Hdr,
}

/// The five sidebar tabs of the split-pane Graphics / Video Settings screen (Image 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    General,
    Quality,
    Performance,
    Advanced,
    Hdr,
}

impl Page {
    pub const ALL: [Page; 5] = [
        Page::General,
        Page::Quality,
        Page::Performance,
        Page::Advanced,
        Page::Hdr,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Page::General => "General",
            Page::Quality => "Quality",
            Page::Performance => "Performance",
            Page::Advanced => "Advanced",
            Page::Hdr => "HDR",
        }
    }

    pub const fn about(self) -> &'static str {
        match self {
            Page::General => "Display, resolution, field of view, frame rate and camera comfort.",
            Page::Quality => "Terrain detail, lighting, clouds, weather, particles and filtering.",
            Page::Performance => "Culling, chunk streaming, upload budgets and mobile power saving.",
            Page::Advanced => "How frames reach the screen, anti-aliasing, and the performance overlay.",
            Page::Hdr => "High dynamic range peak brightness, paper white and black level.",
        }
    }

    pub const fn from_level(level: Level) -> Self {
        match level {
            Level::Simple => Page::General,
            Level::Advanced => Page::Advanced,
            Level::Hdr => Page::Hdr,
        }
    }
}

/// Horizon's own settings (`graphics::Graphics`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum H {
    Preset,
    LodDistance,
    FarDetail,
    CaveCulling,
    EntityCulling,
    ChunkUpdates,
    UploadBudget,
    Sky,
    Fog,
    Weather,
    AnimatedTextures,
    PresentMode,
    Pacing,
    Workers,
    Overlay,
    ResolutionScale,
    Antialiasing,
    Upscaling,
    Hdr,
    HdrSettings,
    HdrBlackLevel,
    HdrKey,
    HdrPeak,
    HdrPaperWhite,
    HdrUiBrightness,
    BatteryMode,
    GraphicsApi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Vanilla(Opt),
    Horizon(H),
}

#[derive(Clone, Copy, Debug)]
pub struct Setting {
    pub item: Item,
    pub cost: Cost,
    pub text: &'static str,
}

pub enum Row {
    Header(&'static str),
    Setting(Setting),
}

fn s_v(opt: Opt, cost: Cost, text: &'static str) -> Setting {
    Setting {
        item: Item::Vanilla(opt),
        cost,
        text,
    }
}

fn s_h(opt: H, cost: Cost, text: &'static str) -> Setting {
    Setting {
        item: Item::Horizon(opt),
        cost,
        text,
    }
}

/// Legacy flat row helper retained for `graphics.rs` unit tests.
pub fn rows(level: Level) -> Vec<Row> {
    let target = Page::from_level(level);
    let mut out = Vec::new();
    for (page, groups) in page_groups() {
        if level == Level::Simple && page != Page::General && page != Page::Performance {
            continue;
        }
        if level == Level::Hdr && page != Page::Hdr {
            continue;
        }
        let _ = target;
        for (gname, items) in groups {
            if level != Level::Simple {
                out.push(Row::Header(gname));
            }
            for st in items {
                out.push(Row::Setting(st));
            }
        }
    }
    out
}

/// Full 5-page grouped definition matching Image 1 (`General`, `Quality`, `Performance`, `Advanced`, `HDR`).
pub fn page_groups() -> Vec<(Page, Vec<(&'static str, Vec<Setting>)>)> {
    use Cost::*;
    let far_desc = if crate::graphics::reduced_far_detail_suggested() {
        "Reduced draws far terrain coarser: faster on this integrated GPU."
    } else {
        "Reduced draws far terrain coarser, for GPUs it slows down."
    };

    let mut adv_system = vec![
        s_h(
            H::PresentMode,
            None,
            "How frames reach the screen while VSync is off (Auto, FIFO, Mailbox, Immediate).",
        ),
        s_h(
            H::Pacing,
            None,
            "Efficient sleeps until the next frame; Precise spins, like vanilla.",
        ),
    ];
    if !cfg!(any(target_os = "macos", target_os = "android")) {
        adv_system.push(s_h(
            H::GraphicsApi,
            None,
            "Vulkan, DirectX 12 or OpenGL backend. Applies on next launch.",
        ));
    }

    vec![
        (
            Page::General,
            vec![
                (
                    "Preset & Scale",
                    vec![
                        s_h(
                            H::Preset,
                            None,
                            "Sets quality and distance options together. Changing one makes it Custom.",
                        ),
                        s_v(
                            Opt::RenderDistance,
                            High,
                            "Full-detail terrain range. LOD Distance can extend local worlds farther.",
                        ),
                        s_v(
                            Opt::SimulationDistance,
                            Medium,
                            "How far mobs, crops and redstone keep ticking around you.",
                        ),
                        s_h(
                            H::ResolutionScale,
                            High,
                            "World pixels per screen pixel (25%..=200%). Lowering boosts mobile FPS dramatically.",
                        ),
                        s_v(Opt::GuiScale, None, "Size of menus, inventory and the HUD."),
                    ],
                ),
                (
                    "Display & Frame Rate",
                    vec![
                        s_v(Opt::Fov, Low, "Field of view. Wider shows more terrain at once."),
                        s_v(
                            Opt::MaxFps,
                            None,
                            "Caps frames per second; capping saves battery and keeps thermals steady.",
                        ),
                        s_v(
                            Opt::Vsync,
                            None,
                            "Synchronizes presentation with display refresh to prevent tearing.",
                        ),
                        s_v(
                            Opt::InactivityFps,
                            Saves,
                            "Lowers frame rate automatically when minimized or AFK.",
                        ),
                        s_v(Opt::Fullscreen, None, "Fills the display borderlessly."),
                        s_v(Opt::Gamma, None, "Adjusts scene brightness in dark areas."),
                    ],
                ),
                (
                    "View & Indicators",
                    vec![
                        s_v(Opt::ViewBobbing, None, "Gentle camera sway while walking and sprinting."),
                        s_v(Opt::ChunkFade, None, "Smoothly fades newly meshed chunks in."),
                        s_v(Opt::Vignette, None, "Subtle darkening around screen corners."),
                        s_v(Opt::AttackIndicator, None, "Shows melee recharge indicator at crosshair or hotbar."),
                        s_v(Opt::Autosave, None, "Shows subtle indicator when local worlds autosave."),
                    ],
                ),
            ],
        ),
        (
            Page::Quality,
            vec![
                (
                    "World & Atmosphere",
                    vec![
                        s_v(
                            Opt::SmoothLighting,
                            Low,
                            "Ambient occlusion soft corner shading. Costs chunk meshing time only.",
                        ),
                        s_v(
                            Opt::BiomeBlend,
                            Low,
                            "Blends grass, foliage and water tints smoothly across biome borders.",
                        ),
                        s_v(Opt::Clouds, Low, "Cloud rendering style: Off, Fast (2D) or Fancy (3D)."),
                        s_v(Opt::CloudRange, Low, "How far horizon clouds extend, in chunks."),
                        s_h(H::Sky, Low, "Draws sky dome, sun, moon and starfield."),
                        s_h(
                            H::Fog,
                            None,
                            "Atmospheric distance fog. Turn off (No Fog) for crystal-clear horizons.",
                        ),
                        s_h(H::Weather, High, "Draws falling rain and snow particles."),
                        s_v(Opt::WeatherRadius, Medium, "Radius of falling rain and snow around you."),
                    ],
                ),
                (
                    "Entities, Leaves & Textures",
                    vec![
                        s_v(Opt::Particles, Medium, "Controls density of gameplay and ambient particles."),
                        s_v(
                            Opt::EntityDistance,
                            Medium,
                            "Maximum distance at which mobs, players and block-entities render.",
                        ),
                        s_v(Opt::EntityShadows, Low, "Soft circular shadows beneath entities."),
                        s_v(
                            Opt::CutoutLeaves,
                            Medium,
                            "See-through gaps in tree foliage; turning off draws solid fast leaves.",
                        ),
                        s_v(
                            Opt::ImprovedTransparency,
                            Medium,
                            "Exact depth sorting for stained glass and translucent water.",
                        ),
                        s_h(
                            H::AnimatedTextures,
                            Low,
                            "Ticks animated water, lava, fire and portal sprites.",
                        ),
                        s_v(
                            Opt::Mipmaps,
                            None,
                            "Filtered mip levels reducing distant block shimmer.",
                        ),
                        s_v(
                            Opt::TextureFiltering,
                            Low,
                            "Distant block sampling filter (None, RGSS or Anisotropic).",
                        ),
                        s_v(
                            Opt::MaxAnisotropy,
                            Medium,
                            "Anisotropic sharpness multiplier at oblique angles.",
                        ),
                    ],
                ),
            ],
        ),
        (
            Page::Performance,
            vec![
                (
                    "Occlusion & Distant Terrain",
                    vec![
                        s_h(
                            H::CaveCulling,
                            Saves,
                            "Skips meshing & drawing sections occluded underground (Sodium fast-path).",
                        ),
                        s_h(
                            H::EntityCulling,
                            Saves,
                            "Skips entities and block-entities hidden behind opaque walls.",
                        ),
                        s_h(
                            H::LodDistance,
                            Medium,
                            "Coarse distant terrain in local worlds. Integrated GPUs stop at 128 chunks.",
                        ),
                        s_h(H::FarDetail, Medium, far_desc),
                    ],
                ),
                (
                    "Streaming, Threads & Power",
                    vec![
                        s_h(
                            H::BatteryMode,
                            Saves,
                            "60 FPS cap (30 in menus or unfocused) and balanced workers for cool mobile play.",
                        ),
                        s_h(
                            H::Workers,
                            None,
                            "Background threads generating, lighting and meshing chunks.",
                        ),
                        s_h(
                            H::ChunkUpdates,
                            Low,
                            "Finished chunk sections uploaded per frame; lower keeps frame times ultra-flat.",
                        ),
                        s_h(
                            H::UploadBudget,
                            Low,
                            "Maximum GPU mesh upload bandwidth per frame.",
                        ),
                        s_v(
                            Opt::ChunkBuilder,
                            Low,
                            "Which block updates rebuild synchronously before presenting the frame.",
                        ),
                    ],
                ),
            ],
        ),
        (
            Page::Advanced,
            vec![
                ("Frame Delivery", adv_system),
                (
                    "Anti-Aliasing & Reconstruction",
                    vec![
                        s_h(
                            H::Antialiasing,
                            Medium,
                            "Smooths jagged edges: Off (sharpest), FXAA (lightweight), or TAA.",
                        ),
                        s_h(
                            H::Upscaling,
                            Saves,
                            "Temporal reconstruction ratio when TAA is active.",
                        ),
                    ],
                ),
                (
                    "Diagnostics",
                    vec![s_h(
                        H::Overlay,
                        None,
                        "Live frame rate and frame-time telemetry graph over gameplay.",
                    )],
                ),
            ],
        ),
        (
            Page::Hdr,
            vec![(
                "High Dynamic Range",
                vec![
                    s_h(
                        H::Hdr,
                        None,
                        "Lava, fire and sunlight shine brighter than SDR white on HDR displays.",
                    ),
                    s_h(
                        H::HdrBlackLevel,
                        None,
                        "Display minimum black level (0.00 nits for OLED).",
                    ),
                    s_h(
                        H::HdrPeak,
                        None,
                        "Peak highlight luminance in nits (Auto uses system EDID).",
                    ),
                    s_h(
                        H::HdrPaperWhite,
                        None,
                        "Reference diffuse white luminance in nits.",
                    ),
                    s_h(
                        H::HdrUiBrightness,
                        None,
                        "HUD and menu luminance relative to paper white.",
                    ),
                    s_h(
                        H::HdrKey,
                        None,
                        "Keyboard shortcut opening HDR calibration directly in-game.",
                    ),
                ],
            )],
        ),
    ]
}

const UPLOAD_STEPS: [u32; 8] = [0, 256, 512, 1024, 2048, 4096, 8192, 16384];

fn on_off(on: bool) -> String {
    lang::get(if on { "On" } else { "Off" })
}

fn labelled(caption: &str, value: impl std::fmt::Display) -> Text {
    let value = value.to_string();
    Text::literal(format!("{}: {}", lang::get(caption), lang::get(&value)))
}

fn max_workers() -> u32 {
    std::thread::available_parallelism().map_or(8, |n| n.get() as u32)
}

impl H {
    fn slider(self, s: &Settings) -> Option<f32> {
        let g = &s.horizon;
        match self {
            H::ChunkUpdates => Some(g.chunk_updates as f32 / 64.0),
            H::UploadBudget => {
                let i = UPLOAD_STEPS
                    .iter()
                    .position(|&b| b >= g.upload_budget_kib)
                    .unwrap_or(UPLOAD_STEPS.len() - 1);
                Some(i as f32 / (UPLOAD_STEPS.len() - 1) as f32)
            }
            H::Workers => Some(g.workers as f32 / max_workers() as f32),
            H::ResolutionScale => {
                Some((g.resolution_scale.clamp(25, 200) as f32 - 25.0) / 175.0)
            }
            H::HdrPaperWhite => Some(match g.hdr_paper_white {
                0 => 0.0,
                n => (n as f32 - 70.0) / 410.0,
            }),
            H::HdrPeak => Some(match g.hdr_peak {
                0 => 0.0,
                n => (n as f32 - 150.0) / 1850.0,
            }),
            H::HdrBlackLevel => Some(g.hdr_black_level as f32 / 100.0),
            H::HdrUiBrightness => Some((g.hdr_ui_brightness as f32 - 25.0) / 75.0),
            _ => None,
        }
    }

    fn label(self, s: &Settings) -> Text {
        let g = &s.horizon;
        match self {
            H::Preset => labelled("Preset", g.preset.label()),
            H::LodDistance => match g.lod_distance {
                0 => labelled("LOD Distance", "Off"),
                n => labelled("LOD Distance", lang::tr("%s chunks", &[&n])),
            },
            H::FarDetail => labelled("Far Detail", g.far_detail.label()),
            H::CaveCulling => labelled("Cave Culling", on_off(g.cave_culling)),
            H::EntityCulling => labelled("Entity Culling", on_off(g.entity_culling)),
            H::ChunkUpdates => match g.chunk_updates {
                0 => labelled("Chunk Updates", "Unlimited"),
                n => labelled("Chunk Updates", lang::tr("%s per frame", &[&n])),
            },
            H::UploadBudget => match g.upload_budget_kib {
                0 => labelled("Upload Budget", "Unlimited"),
                k if k >= 1024 => {
                    labelled("Upload Budget", lang::tr("%s MiB/frame", &[&(k / 1024)]))
                }
                k => labelled("Upload Budget", lang::tr("%s KiB/frame", &[&k])),
            },
            H::Sky => labelled("Sky", on_off(g.sky)),
            H::Fog => labelled("Fog", on_off(g.fog)),
            H::Weather => labelled("Weather", on_off(g.weather)),
            H::AnimatedTextures => labelled("Animated Textures", on_off(g.animated_textures)),
            H::PresentMode => labelled("Present Mode", g.present_mode.label()),
            H::Pacing => labelled(
                "Frame Pacing",
                match g.pacing {
                    Pacing::Efficient => "Efficient",
                    Pacing::Precise => "Precise",
                },
            ),
            H::Workers => match g.workers {
                0 => labelled(
                    "Worker Threads",
                    lang::tr("Auto (%s)", &[&g.worker_threads()]),
                ),
                n => labelled("Worker Threads", n),
            },
            H::ResolutionScale => labelled("Resolution Scale", format!("{}%", g.resolution_scale)),
            H::Antialiasing => labelled(
                "Anti-aliasing",
                match (g.fxaa, g.taa) {
                    (_, true) => "TAA",
                    (true, false) => "FXAA",
                    (false, false) => "Off",
                },
            ),
            H::Upscaling => labelled("Upscaling", g.upscaling.label()),
            H::Hdr => labelled("HDR", on_off(g.hdr)),
            H::HdrSettings => Text::literal(lang::get("HDR Settings...")),
            H::HdrPaperWhite => match g.hdr_paper_white {
                0 => labelled(
                    "Paper White",
                    lang::tr("Auto (%s nits)", &[&g.paper_white_nits()]),
                ),
                n => labelled("Paper White", lang::tr("%s nits", &[&n])),
            },
            H::HdrPeak => match g.hdr_peak {
                0 => labelled(
                    "Peak Brightness",
                    lang::tr("Auto (%s nits)", &[&g.peak_nits()]),
                ),
                n => labelled("Peak Brightness", lang::tr("%s nits", &[&n])),
            },
            H::HdrBlackLevel => labelled(
                "Black Level",
                lang::tr(
                    "%s nits",
                    &[&format!("{:.2}", g.hdr_black_level as f32 / 100.0)],
                ),
            ),
            H::HdrUiBrightness => labelled("UI Brightness", format!("{}%", g.hdr_ui_brightness)),
            H::HdrKey => {
                Text::literal("Shortcut: ").append(crate::keybinds::display_name(&g.hdr_key))
            }
            H::BatteryMode => labelled("Battery Mode", on_off(g.battery)),
            H::GraphicsApi => labelled("Graphics API", g.api.label()),
            H::Overlay => labelled(
                "Performance Overlay",
                match g.overlay {
                    Overlay::Off => "Off",
                    Overlay::Fps => "FPS",
                    Overlay::Graph => "FPS and Graph",
                },
            ),
        }
    }

    fn set_slider(self, s: &mut Settings, v: f32) {
        let g = &mut s.horizon;
        match self {
            H::ChunkUpdates => g.chunk_updates = (v * 64.0).round() as u32,
            H::UploadBudget => {
                g.upload_budget_kib =
                    UPLOAD_STEPS[(v * (UPLOAD_STEPS.len() - 1) as f32).round() as usize]
            }
            H::Workers => g.workers = (v * max_workers() as f32).round() as u32,
            H::ResolutionScale => g.resolution_scale = (25.0 + (v * 35.0).round() * 5.0) as u32,
            H::HdrPaperWhite => {
                g.hdr_paper_white = match (v * 41.0).round() as u32 {
                    0 => 0,
                    step => 70 + step * 10,
                }
            }
            H::HdrPeak => {
                g.hdr_peak = match (v * 37.0).round() as u32 {
                    0 => 0,
                    step => 150 + step * 50,
                }
            }
            H::HdrBlackLevel => g.hdr_black_level = (v * 100.0).round() as u32,
            H::HdrUiBrightness => g.hdr_ui_brightness = (25.0 + (v * 15.0).round() * 5.0) as u32,
            _ => {}
        }
    }

    fn cycle(self, s: &mut Settings, back: bool) {
        fn step<T: Copy + PartialEq>(all: &[T], now: T, back: bool) -> T {
            let i = all.iter().position(|x| *x == now).unwrap_or(0);
            let n = all.len();
            all[if back { (i + n - 1) % n } else { (i + 1) % n }]
        }
        if self == H::Preset {
            let preset = step(&Preset::ALL, s.horizon.preset, back);
            crate::graphics::apply_preset(preset, s);
            s.horizon.preset = preset;
            return;
        }
        let g = &mut s.horizon;
        match self {
            H::LodDistance => {
                g.lod_distance = step(&[0, 64, 128, 256, 512, 1024], g.lod_distance, back)
            }
            H::FarDetail => {
                g.far_detail = step(&[FarDetail::Full, FarDetail::Reduced], g.far_detail, back)
            }
            H::CaveCulling => g.cave_culling = !g.cave_culling,
            H::EntityCulling => g.entity_culling = !g.entity_culling,
            H::Sky => g.sky = !g.sky,
            H::Fog => {
                g.fog = !g.fog;
                s.extras
                    .mod_catalog
                    .set(crate::volt_mods::VoltModId::NoFog, !g.fog);
            }
            H::Weather => g.weather = !g.weather,
            H::AnimatedTextures => g.animated_textures = !g.animated_textures,
            H::PresentMode => g.present_mode = step(&PresentMode::ALL, g.present_mode, back),
            H::Antialiasing => {
                let i = if g.taa { 2 } else { usize::from(g.fxaa) };
                let i = (i + if back { 2 } else { 1 }) % 3;
                (g.fxaa, g.taa) = (i == 1, i == 2);
                if !g.taa {
                    g.upscaling = Upscaling::Native;
                }
            }
            H::Upscaling => {
                g.upscaling = step(&Upscaling::ALL, g.upscaling, back);
                if g.upscaling != Upscaling::Native {
                    (g.fxaa, g.taa) = (false, true);
                }
            }
            H::Hdr => g.hdr = !g.hdr,
            H::BatteryMode => g.battery = !g.battery,
            H::GraphicsApi => {
                if !cfg!(target_os = "android") {
                    g.api = step(GraphicsApi::choices(), g.api, back);
                }
            }
            H::Pacing => {
                g.pacing = match g.pacing {
                    Pacing::Efficient => Pacing::Precise,
                    Pacing::Precise => Pacing::Efficient,
                }
            }
            H::Overlay => {
                g.overlay = step(
                    &[Overlay::Off, Overlay::Fps, Overlay::Graph],
                    g.overlay,
                    back,
                )
            }
            _ => {}
        }
    }
}

impl Item {
    fn available(self) -> bool {
        !matches!(self, Item::Vanilla(o) if !o.available())
    }

    pub fn blocked(self, s: &Settings) -> Option<&'static str> {
        if !self.available() {
            return Some("This feature is not yet available in Volt.");
        }
        match self {
            Item::Horizon(H::HdrSettings | H::Hdr) if !crate::graphics::hdr_offered() => {
                Some("Your display or system doesn't offer HDR output.")
            }
            Item::Horizon(
                H::HdrBlackLevel | H::HdrPeak | H::HdrPaperWhite | H::HdrUiBrightness,
            ) if !s.horizon.hdr => Some("Turn HDR on to use it."),
            _ => None,
        }
    }

    pub fn switch(self, s: &Settings) -> Option<bool> {
        let g = &s.horizon;
        Some(match self {
            Item::Horizon(H::CaveCulling) => g.cave_culling,
            Item::Horizon(H::EntityCulling) => g.entity_culling,
            Item::Horizon(H::Sky) => g.sky,
            Item::Horizon(H::Fog) => g.fog,
            Item::Horizon(H::Weather) => g.weather,
            Item::Horizon(H::AnimatedTextures) => g.animated_textures,
            Item::Horizon(H::Hdr) => g.hdr,
            Item::Horizon(H::BatteryMode) => g.battery,
            Item::Vanilla(Opt::ViewBobbing) => s.view_bobbing,
            Item::Vanilla(Opt::Vsync) => s.vsync,
            Item::Vanilla(Opt::Fullscreen) => s.fullscreen,
            Item::Vanilla(Opt::SmoothLighting) => s.smooth_lighting,
            Item::Vanilla(Opt::EntityShadows) => s.entity_shadows,
            Item::Vanilla(Opt::Vignette) => s.vignette,
            Item::Vanilla(Opt::Autosave) => s.autosave_indicator,
            Item::Vanilla(Opt::CutoutLeaves) => s.cutout_leaves,
            Item::Vanilla(Opt::ImprovedTransparency) => s.improved_transparency,
            _ => return None,
        })
    }

    pub fn slider(self, s: &Settings) -> Option<f32> {
        match self {
            Item::Vanilla(o) => o.slider_position(s),
            Item::Horizon(h) => h.slider(s),
        }
    }

    pub fn label(self, s: &Settings) -> Text {
        match self {
            Item::Vanilla(Opt::RenderDistance) if s.horizon.lod_distance > 0 => labelled(
                "Full Detail Distance",
                lang::tr("%s chunks", &[&s.render_distance]),
            ),
            Item::Vanilla(o) => o.label(s),
            Item::Horizon(h) => h.label(s),
        }
    }

    pub fn set_slider(self, s: &mut Settings, v: f32) {
        if !self.available() {
            return;
        }
        match self {
            Item::Vanilla(o) => o.set_slider(s, v),
            Item::Horizon(h) => h.set_slider(s, v),
        }
        self.touched(s);
    }

    pub fn cycle(self, s: &mut Settings, back: bool, framebuffer: [i32; 2]) {
        if !self.available() {
            return;
        }
        match self {
            Item::Vanilla(o) => o.cycle(s, back, framebuffer),
            Item::Horizon(h) => h.cycle(s, back),
        }
        self.touched(s);
    }

    fn touched(self, s: &mut Settings) {
        let presetless = matches!(
            self,
            Item::Horizon(
                H::Preset
                    | H::Overlay
                    | H::Pacing
                    | H::PresentMode
                    | H::Workers
                    | H::BatteryMode
                    | H::GraphicsApi
            ) | Item::Vanilla(
                Opt::GuiScale
                    | Opt::Fullscreen
                    | Opt::Vsync
                    | Opt::MaxFps
                    | Opt::Fov
                    | Opt::Gamma
                    | Opt::InactivityFps
                    | Opt::ViewBobbing
                    | Opt::ChunkFade
                    | Opt::Vignette
                    | Opt::Autosave
                    | Opt::AttackIndicator
            )
        );
        if !presetless {
            s.horizon.preset = Preset::Custom;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    ButtonClick,
    Changed,
    SliderDrag,
    OpenAdvanced,
    OpenSimple,
    OpenHdr,
    BindKey(&'static str),
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusTarget {
    Search,
    Page(Page),
    Item(Item),
    Undo,
    Done,
}

#[derive(Clone, Copy)]
struct Line {
    y: f32,
    page: Page,
    group: Option<(&'static str, f32)>,
    setting: Option<Setting>,
}

impl Line {
    fn h(&self) -> f32 {
        if self.setting.is_some() {
            style::ROW
        } else if self.group.is_some() {
            style::GROUP_ROW
        } else {
            style::PAGE_ROW
        }
    }
}

/// Exact Image 1 Split-Pane column geometry.
pub struct Columns {
    pub side_x: f32,
    pub side_y: f32,
    pub plate: [f32; 4],
    pub dock: [f32; 4],
    pub list_x: f32,
    pub list_w: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Columns {
    pub fn of(w: f32, h: f32) -> Self {
        let gap = 8.0;
        let max_total = 560.0;
        let margin = ((w - max_total) * 0.5).max(8.0).floor();
        let side_x = margin;
        let px = side_x + style::CONTROL_W + gap;
        let pw = (w - margin - px).max(160.0);
        let side_y = if h >= style::TITLED { 20.0 } else { 8.0 };
        let dock_h = if h < 210.0 { 36.0 } else { style::DOCK_H };
        let dock = [px, h - 8.0 - dock_h, pw, dock_h];
        let plate = [px, side_y, pw, (dock[1] - 4.0 - side_y).max(60.0)];
        let list_x = px + 8.0;
        Self {
            side_x,
            side_y,
            plate,
            dock,
            list_x,
            list_w: (pw - 16.0 - style::SCROLLER_W - 2.0).max(120.0),
            top: side_y + 28.0,
            bottom: plate[1] + plate[3] - 6.0,
        }
    }

    pub fn search(&self) -> [f32; 4] {
        [
            self.side_x,
            self.side_y + 24.0,
            style::CONTROL_W,
            style::SEARCH_H,
        ]
    }

    pub fn clear(&self) -> [f32; 4] {
        let [x, y, w, h] = self.search();
        [x + w - h, y, h, h]
    }

    pub fn scrolls(&self) -> bool {
        self.bottom - self.top >= 36.0
    }

    pub fn bar_x(&self) -> f32 {
        self.plate[0] + self.plate[2] - 8.0 - style::SCROLLER_W
    }

    pub fn page(&self, i: usize) -> [f32; 4] {
        [
            self.side_x,
            self.side_y + 46.0 + i as f32 * 18.0,
            style::CONTROL_W,
            16.0,
        ]
    }

    pub fn button(&self, i: usize) -> [f32; 4] {
        let foot = self.dock[1] + self.dock[3] - style::BUTTON_H;
        let y = if i == 0 {
            foot
        } else {
            foot - 4.0 - style::BUTTON_H
        };
        [self.side_x, y, style::CONTROL_W, style::BUTTON_H]
    }

    pub fn track(&self) -> (f32, f32) {
        let tw = style::CONTROL_W.min((self.list_w * 0.46).max(76.0));
        let x = self.list_x + self.list_w - style::INSET - tw;
        (x, tw)
    }

    pub fn slider_at(&self, x: f32) -> f32 {
        let (tx, tw) = self.track();
        ((x - tx - 4.0) / (tw - 8.0)).clamp(0.0, 1.0)
    }
}

fn shown(item: Item, s: &Settings) -> bool {
    item != Item::Horizon(H::Upscaling) || s.horizon.taa
}

fn parts(item: Item, s: &Settings, binding: bool) -> (String, Option<String>) {
    if item == Item::Horizon(H::HdrKey) {
        let key = crate::keybinds::display_name(&s.horizon.hdr_key).plain();
        let key = if binding { format!("> {key} <") } else { key };
        return (lang::get("Shortcut"), Some(key));
    }
    let label = item.label(s).plain();
    let format = lang::get("options.generic_value");
    let sep = format
        .split("%s")
        .nth(1)
        .filter(|sep| !sep.is_empty() && label.contains(*sep))
        .unwrap_or(": ");
    match label.split_once(sep) {
        Some((name, value)) => (name.to_owned(), Some(value.to_owned())),
        None => (label, None),
    }
}

pub struct GraphicsUi {
    scroll: ui::Scroll,
    dragging: Option<Item>,
    pub focus: Option<FocusTarget>,
    pub binding: Option<&'static str>,
    pub search: ui::EditBox,
    pub page: Page,
    picked: Option<Page>,
    before: Option<Box<Settings>>,
    pub changed: bool,
    // Smooth animation state
    last_frame: Option<Instant>,
    tab_anim_y: f32,
    scroll_visual: f32,
    page_slide: f32,
}

impl Default for GraphicsUi {
    fn default() -> Self {
        let mut search = ui::EditBox::new(0.0, 0.0, 0.0, 8.0);
        search.bordered = false;
        search.focused = false;
        search.message = lang::get("Search options");
        Self {
            scroll: ui::Scroll::default(),
            dragging: None,
            focus: None,
            binding: None,
            search,
            page: Page::General,
            picked: None,
            before: None,
            changed: false,
            last_frame: None,
            tab_anim_y: 0.0,
            scroll_visual: 0.0,
            page_slide: 1.0,
        }
    }
}

impl GraphicsUi {
    pub fn reset(&mut self) {
        self.open(Page::General);
    }

    pub fn open_level(&mut self, level: Level) {
        self.open(Page::from_level(level));
    }

    pub fn open(&mut self, page: Page) {
        *self = Self {
            page,
            ..Self::default()
        };
    }

    pub fn typing(&self) -> bool {
        self.focus == Some(FocusTarget::Search)
    }

    fn query(&self) -> String {
        self.search.value.trim().to_lowercase()
    }

    fn keep(&mut self, s: &Settings) {
        if self.before.is_none() {
            self.before = Some(Box::new(s.clone()));
        }
    }

    fn undo(&mut self, s: &mut Settings) {
        let Some(b) = self.before.as_deref() else {
            return;
        };
        s.horizon = b.horizon.clone();
        s.render_distance = b.render_distance;
        s.simulation_distance = b.simulation_distance;
        s.gamma = b.gamma;
        s.gui_scale = b.gui_scale;
        s.vsync = b.vsync;
        s.max_fps = b.max_fps;
        s.inactivity_fps_limit = b.inactivity_fps_limit.clone();
        s.fov = b.fov;
        s.view_bobbing = b.view_bobbing;
        s.attack_indicator = b.attack_indicator;
        s.autosave_indicator = b.autosave_indicator;
        s.clouds = b.clouds.clone();
        s.cloud_range = b.cloud_range;
        s.weather_radius = b.weather_radius;
        s.particles = b.particles;
        s.smooth_lighting = b.smooth_lighting;
        s.biome_blend = b.biome_blend;
        s.entity_distance = b.entity_distance;
        s.entity_shadows = b.entity_shadows;
        s.vignette = b.vignette;
        s.chunk_fade = b.chunk_fade;
        s.mipmap_levels = b.mipmap_levels;
        s.cutout_leaves = b.cutout_leaves;
        s.improved_transparency = b.improved_transparency;
        s.graphics_preset = b.graphics_preset.clone();
        self.changed = false;
    }

    fn lines(&self, s: &Settings) -> (Vec<Line>, f32) {
        let query = self.query();
        let mut lines = Vec::new();
        let mut y: f32 = 0.0;
        for (page, groups) in page_groups() {
            if query.is_empty() && page != self.page {
                continue;
            }
            let groups: Vec<(&'static str, Vec<Setting>)> = groups
                .into_iter()
                .map(|(name, items)| {
                    let items = items
                        .into_iter()
                        .filter(|r| shown(r.item, s))
                        .filter(|r| {
                            query.is_empty()
                                || self.focus == Some(FocusTarget::Item(r.item))
                                || self.dragging == Some(r.item)
                                || parts(r.item, s, false).0.to_lowercase().contains(&query)
                                || lang::get(r.text).to_lowercase().contains(&query)
                        })
                        .collect::<Vec<_>>();
                    (name, items)
                })
                .filter(|(_, items)| !items.is_empty())
                .collect();
            if groups.is_empty() {
                continue;
            }
            if !query.is_empty() {
                if !lines.is_empty() {
                    y += style::PAGE_GAP;
                }
                lines.push(Line {
                    y,
                    page,
                    group: None,
                    setting: None,
                });
                y += style::PAGE_ROW;
            }
            for (name, items) in groups {
                lines.push(Line {
                    y,
                    page,
                    group: Some((name, items.len() as f32 * style::ROW)),
                    setting: None,
                });
                y += style::GROUP_ROW;
                for setting in items {
                    lines.push(Line {
                        y,
                        page,
                        group: None,
                        setting: Some(setting),
                    });
                    y += style::ROW;
                }
                y += style::GROUP_GAP;
            }
        }
        (lines, (y - style::GROUP_GAP).max(0.0))
    }

    fn layout(&mut self, s: &Settings, w: f32, h: f32) -> (Columns, Vec<Line>, f32) {
        let cols = Columns::of(w, h);
        let (lines, content) = self.lines(s);
        self.scroll.clamp(cols.top, cols.bottom, content);
        (cols, lines, content)
    }

    fn has(&self, page: Page, lines: &[Line]) -> bool {
        self.query().is_empty() || lines.iter().any(|l| l.page == page)
    }

    fn current(&self, lines: &[Line]) -> Option<Page> {
        if self.query().is_empty() {
            return Some(self.page);
        }
        self.picked.or_else(|| {
            lines
                .iter()
                .find(|l| l.y + l.h() > self.scroll.amount)
                .map(|l| l.page)
        })
    }

    fn pick(&mut self, page: Page, lines: &[Line], cols: &Columns, content: f32) {
        if self.page != page {
            self.page_slide = 0.0;
        }
        self.page = page;
        self.scroll.amount = 0.0;
        self.scroll_visual = 0.0;
        if !self.query().is_empty()
            && let Some(line) = lines.iter().find(|l| l.page == page)
        {
            self.scroll.amount = line.y;
            self.scroll.clamp(cols.top, cols.bottom, content);
            self.scroll_visual = self.scroll.amount;
            self.picked = Some(page);
        }
    }

    fn step_animations(&mut self, cols: &Columns, current: Option<Page>) {
        let now = Instant::now();
        let dt = self
            .last_frame
            .map(|p| now.duration_since(p).as_secs_f32().clamp(0.0, 0.08))
            .unwrap_or(0.016);
        self.last_frame = Some(now);
        let fast = 1.0 - (-20.0 * dt).exp();
        self.page_slide = (self.page_slide + (1.0 - self.page_slide) * fast).clamp(0.0, 1.0);
        if (self.scroll_visual - self.scroll.amount).abs() > 140.0 || self.scroll.held() {
            self.scroll_visual = self.scroll.amount;
        } else {
            self.scroll_visual += (self.scroll.amount - self.scroll_visual) * fast;
        }
        if let Some(p) = current
            && let Some(idx) = Page::ALL.iter().position(|&x| x == p)
        {
            let target_y = cols.page(idx)[1];
            if self.tab_anim_y == 0.0 || (self.tab_anim_y - target_y).abs() > 120.0 {
                self.tab_anim_y = target_y;
            } else {
                self.tab_anim_y += (target_y - self.tab_anim_y) * fast;
            }
        }
    }

    fn row_y(&self, cols: &Columns, line: &Line) -> f32 {
        let slide_offset = (1.0 - self.page_slide) * 5.0;
        cols.top + line.y - self.scroll_visual + slide_offset
    }

    pub fn draw(
        &mut self,
        _level: Level,
        s: &Settings,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        cursor: (f32, f32),
        in_world: bool,
    ) {
        self.keep(s);
        let (w, h) = (batch.width, batch.height);
        let (cols, lines, content) = self.layout(s, w, h);
        let current = self.current(&lines);
        self.step_animations(&cols, current);

        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, in_world);
        style::fill(batch, atlas, [0.0, 0.0, w, h], style::VEIL);

        self.draw_side(batch, atlas, &cols, &lines, current, cursor);
        self.draw_search(batch, atlas, &cols, cursor);

        // Right frosted glass plate
        let [px, py, pw, _] = cols.plate;
        style::panel(batch, atlas, cols.plate);
        let query = self.search.value.trim();
        let heading = if query.is_empty() {
            lang::get(current.unwrap_or(self.page).name())
        } else {
            format!("Results for \"{query}\"")
        };
        let heading = Text::literal(style::fit(atlas, &heading, pw - 24.0));
        batch.text_styled(atlas, &heading, px + 12.0, py + 10.0, style::WHITE, true);
        style::fill(
            batch,
            atlas,
            [px + 8.0, py + 23.0, pw - 16.0, 1.0],
            style::HEAD_RULE,
        );

        let over_list = cursor.1 >= cols.top && cursor.1 < cols.bottom;
        let start = batch.vertices.len();
        let mark = crate::widgets::targets_mark();
        let mut described = None;
        let shows = |y: f32, rh: f32| y + rh > cols.top && y < cols.bottom;

        for line in &lines {
            let y = self.row_y(&cols, line);
            if let Some((_, panel)) = line.group
                && shows(y + style::GROUP_ROW, panel)
            {
                let rect = [cols.list_x, y + style::GROUP_ROW, cols.list_w, panel];
                style::group(batch, atlas, rect);
            }
        }

        let mut first = false;
        for line in &lines {
            let y = self.row_y(&cols, line);
            let after_name = std::mem::replace(&mut first, line.group.is_some());
            if !shows(y, line.h()) {
                continue;
            }
            let Some(setting) = line.setting else {
                let (name, color, dy) = match line.group {
                    Some((name, _)) => (name, style::MUTED, 3.0),
                    None => (line.page.name(), style::ACCENT, 4.0),
                };
                batch.text_styled(
                    atlas,
                    &Text::literal(lang::get(name)),
                    cols.list_x + 2.0,
                    y + dy,
                    color,
                    true,
                );
                continue;
            };
            let item = setting.item;
            if !after_name {
                style::fill(
                    batch,
                    atlas,
                    [cols.list_x + 8.0, y, cols.list_w - 16.0, 1.0],
                    style::ROW_RULE,
                );
            }
            let rect = [cols.list_x, y, cols.list_w, style::ROW];
            let hover = over_list && ui::hovered(cursor, rect[0], rect[1], rect[2], rect[3]);
            let focused = self.focus == Some(FocusTarget::Item(item));
            if hover || self.dragging == Some(item) || (focused && described.is_none()) {
                described = Some(setting);
            }
            self.draw_row(batch, atlas, s, &cols, item, y, hover, focused);
        }
        if lines.is_empty() {
            batch.text_styled(
                atlas,
                &Text::literal("No settings match."),
                cols.list_x + 4.0,
                cols.top + 6.0,
                style::MUTED,
                true,
            );
        }

        batch.clip_since(
            start,
            cols.list_x,
            cols.top,
            cols.list_x + cols.list_w,
            cols.bottom,
        );
        crate::widgets::clip_targets(mark, cols.top, cols.bottom);
        self.draw_scroller(batch, atlas, &cols, content);
        self.draw_dock(batch, atlas, s, &cols, described, current);

        // Bottom-left Undo (amber) + Done (violet) buttons (Image 1)
        let done_rect = cols.button(0);
        let done_lbl = Text::literal(lang::get("gui.done"));
        let done_hov = ui::hovered(cursor, done_rect[0], done_rect[1], done_rect[2], done_rect[3])
            || self.focus == Some(FocusTarget::Done);
        crate::widgets::record(&done_lbl.plain(), done_rect, true);
        crate::widgets::target(done_rect);
        style::button(batch, atlas, done_rect, &done_lbl, (done_hov, true, true));

        if self.changed {
            let undo_rect = cols.button(1);
            let undo_lbl = Text::literal(lang::get("Undo"));
            let undo_hov =
                ui::hovered(cursor, undo_rect[0], undo_rect[1], undo_rect[2], undo_rect[3])
                    || self.focus == Some(FocusTarget::Undo);
            crate::widgets::record(&undo_lbl.plain(), undo_rect, true);
            crate::widgets::target(undo_rect);
            style::undo_button(batch, atlas, undo_rect, &undo_lbl, undo_hov);
        }
    }

    fn draw_side(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        cols: &Columns,
        lines: &[Line],
        current: Option<Page>,
        cursor: (f32, f32),
    ) {
        let (x, y) = (cols.side_x, cols.side_y);
        style::fill(batch, atlas, [x, y + 1.0, 6.0, 6.0], style::ACCENT);
        batch.text_styled(
            atlas,
            &Text::literal("Volt"),
            x + 9.0,
            y,
            style::WHITE,
            true,
        );
        batch.text_styled(
            atlas,
            &Text::literal(lang::get("Graphics")),
            x,
            y + 10.0,
            style::MUTED,
            true,
        );

        if current.is_some() {
            let anim_rect = [cols.side_x, self.tab_anim_y, style::CONTROL_W, 16.0];
            style::fill(batch, atlas, anim_rect, style::OPEN);
            style::fill(
                batch,
                atlas,
                [cols.side_x, self.tab_anim_y, 2.0, 16.0],
                style::ACCENT,
            );
        }

        for (i, page) in Page::ALL.into_iter().enumerate() {
            let rect = cols.page(i);
            let [px, py, pw, ph] = rect;
            let has = self.has(page, lines);
            let open = current == Some(page);
            let name = Text::literal(lang::get(page.name()));
            crate::widgets::record_as(&name.plain(), rect, has, crate::widgets::Kind::Tab(open));
            if has {
                crate::widgets::target_as(rect, crate::widgets::Kind::Tab(open));
            }
            let hover = has && ui::hovered(cursor, px, py, pw, ph);
            let focused = self.focus == Some(FocusTarget::Page(page));
            if open || focused {
                style::ring(batch, atlas, rect, style::WHITE);
            } else if hover {
                style::fill(batch, atlas, rect, style::HOVER);
            }
            let color = match (open || hover, has) {
                (true, _) => style::WHITE,
                (false, true) => style::SOFT,
                (false, false) => style::MUTED,
            };
            let ty = py + ((ph - 8.0) / 2.0).floor();
            let shown = Text::literal(style::fit(atlas, &name.plain(), pw - 12.0));
            batch.text_styled(atlas, &shown, px + 8.0, ty, color, true);
        }
    }

    fn draw_search(
        &mut self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        cols: &Columns,
        cursor: (f32, f32),
    ) {
        let rect = cols.search();
        style::well(batch, atlas, rect);
        let focused = self.focus == Some(FocusTarget::Search);
        self.search.focused = focused;
        let [x, y, w, h] = rect;
        let clear = cols.clear();
        (self.search.x, self.search.y) = (x + 5.0, y + ((h - 8.0) / 2.0).floor());
        self.search.width = clear[0] - self.search.x;
        self.search.text_color = style::WHITE;
        self.search.draw(batch, atlas, None);
        if self.search.value.is_empty() && !focused {
            batch.text_styled(
                atlas,
                &Text::literal(&self.search.message),
                x + 5.0,
                self.search.y,
                style::MUTED,
                true,
            );
        }
        if !self.search.value.is_empty() {
            let hover = ui::hovered(cursor, clear[0], clear[1], clear[2], clear[3]);
            let color = if hover { style::WHITE } else { style::MUTED };
            let cy = y + ((h - 8.0) / 2.0).floor();
            ui::centered_text(batch, atlas, &Text::literal("x"), clear[0], cy, clear[2], color);
        }
        style::ring(
            batch,
            atlas,
            [x, y, w, h],
            if focused { style::WHITE } else { style::EDGE },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_row(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        s: &Settings,
        cols: &Columns,
        item: Item,
        y: f32,
        hover: bool,
        focused: bool,
    ) {
        let rect = [cols.list_x, y, cols.list_w, style::ROW];
        let live = item.blocked(s).is_none();
        let label = item.label(s).plain();
        let (name, value) = parts(item, s, self.binding.is_some());
        let (tx, tw) = cols.track();
        let right = tx + tw;
        let control = [
            tx,
            y + (style::ROW - style::CONTROL_H) / 2.0,
            tw,
            style::CONTROL_H,
        ];
        let active = live && (hover || focused || self.dragging == Some(item));
        if active {
            style::fill(
                batch,
                atlas,
                [rect[0] + 2.0, y + 1.0, rect[2] - 4.0, style::ROW - 2.0],
                style::HOVER,
            );
        }
        crate::widgets::widget_cursor(hover, live);
        let slider = item.slider(s);
        let switch = item.switch(s);
        let kind = match slider {
            Some(v) => crate::widgets::Kind::Slider {
                value: v,
                track: [tx + 4.0, tw - 8.0],
            },
            None => crate::widgets::Kind::Button,
        };
        crate::widgets::record_as(&label, rect, live, kind);
        if live {
            crate::widgets::target_as(rect, kind);
        }

        let color = if live { style::WHITE } else { style::MUTED };
        let ty = y + ((style::ROW - 8.0) / 2.0).floor();
        let lit = live && (hover || self.dragging == Some(item));
        let left = rect[0] + style::INSET;

        if switch.is_none() && slider.is_none() && value.is_none() {
            let wide = [left, control[1], right - left, style::CONTROL_H];
            style::button(batch, atlas, wide, &Text::literal(name), (lit, live, false));
        } else {
            let value = value.unwrap_or_default();
            let vw = atlas.font.width(&value);
            let start = match (switch, slider) {
                (Some(_), _) => right - style::CONTROL_H,
                (None, Some(_)) => tx,
                (None, None) => (right - (vw + 20.0).max(58.0)).max(tx),
            };
            match (switch, slider) {
                (Some(on), _) => style::checkbox(batch, atlas, right, y, on, live),
                (None, Some(pos)) => {
                    style::slider(
                        batch,
                        atlas,
                        control,
                        &Text::literal(value),
                        pos,
                        (lit, live),
                    );
                }
                (None, None) => {
                    style::cycler(
                        batch,
                        atlas,
                        [start, y + 3.0, right - start, 14.0],
                        &Text::literal(value),
                        (lit, live),
                    );
                }
            }
            let shown = Text::literal(style::fit(atlas, &name, start - 6.0 - left));
            batch.text_styled(atlas, &shown, left, ty, color, true);
        }
        if focused {
            style::ring(
                batch,
                atlas,
                [rect[0] + 1.0, y, rect[2] - 2.0, style::ROW],
                style::WHITE,
            );
        }
    }

    fn draw_scroller(&self, batch: &mut HudBatch, atlas: &HudAtlas, cols: &Columns, content: f32) {
        if !cols.scrolls() {
            return;
        }
        let Some((y, bar)) =
            style::scrollbar_thumb(cols.top, cols.bottom, content, self.scroll_visual)
        else {
            return;
        };
        let x = cols.bar_x();
        let track = [x, cols.top, style::SCROLLER_W, cols.bottom - cols.top];
        style::scroller(batch, atlas, track, [x, y, style::SCROLLER_W, bar]);
    }

    fn draw_dock(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        s: &Settings,
        cols: &Columns,
        setting: Option<Setting>,
        current: Option<Page>,
    ) {
        let [x, y, w, h] = cols.dock;
        style::panel(batch, atlas, cols.dock);
        let left = x + 12.0;
        match setting {
            Some(setting) => {
                let (name, _) = parts(setting.item, s, false);
                let cost_lbl = lang::get(setting.cost.label());
                let head = style::fit(atlas, &name, w - 120.0);
                batch.text_styled(atlas, &Text::literal(&head), left, y + 7.0, style::WHITE, true);
                let cx = left + atlas.font.width(&head) + 8.0;
                style::fill(
                    batch,
                    atlas,
                    [cx, y + 8.0, 6.0, 6.0],
                    style::cost_color(setting.cost),
                );
                batch.text_styled(
                    atlas,
                    &Text::literal(cost_lbl),
                    cx + 9.0,
                    7.0 + y,
                    style::cost_color(setting.cost),
                    true,
                );
                let desc = setting
                    .item
                    .blocked(s)
                    .unwrap_or(setting.text);
                let wrapped = style::wrap_lines(atlas, &lang::get(desc), w - 24.0);
                let max_lines = if h < 42.0 { 1 } else { 2 };
                for (i, line) in wrapped.iter().take(max_lines).enumerate() {
                    batch.text_styled(
                        atlas,
                        &Text::literal(line.clone()),
                        left,
                        y + 20.0 + i as f32 * 10.0,
                        style::GREY,
                        true,
                    );
                }
            }
            None => {
                let page = current.unwrap_or(self.page);
                let line = format!("{} — {}", lang::get(page.name()), lang::get(page.about()));
                let wrapped = style::wrap_lines(atlas, &line, w - 24.0);
                for (i, ln) in wrapped.iter().take(2).enumerate() {
                    batch.text_styled(
                        atlas,
                        &Text::literal(ln.clone()),
                        left,
                        y + 10.0 + i as f32 * 11.0,
                        if i == 0 { style::SOFT } else { style::MUTED },
                        true,
                    );
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn click(
        &mut self,
        _level: Level,
        s: &mut Settings,
        (x, y): (f32, f32),
        (w, h): (f32, f32),
        framebuffer: [i32; 2],
        drag: bool,
        right: bool,
    ) -> Vec<Action> {
        self.keep(s);
        let (cols, lines, content) = self.layout(s, w, h);
        if drag {
            if self.scroll.held() {
                self.scroll.drag(y, cols.top, cols.bottom, content);
                self.scroll_visual = self.scroll.amount;
                self.picked = None;
                return Vec::new();
            }
            if let Some(item) = self.dragging {
                item.set_slider(s, cols.slider_at(x));
                self.changed = true;
                return vec![Action::SliderDrag];
            }
            return Vec::new();
        }

        let at = |[rx, ry, rw, rh]: [f32; 4]| ui::hovered((x, y), rx, ry, rw, rh);

        if cols.scrolls()
            && self
                .scroll
                .press((x, y), cols.bar_x(), cols.top, cols.bottom, content)
        {
            self.picked = None;
            return Vec::new();
        }

        if !self.search.value.is_empty() && at(cols.clear()) {
            self.search.set_value("");
            self.scroll.amount = 0.0;
            self.scroll_visual = 0.0;
            return vec![Action::ButtonClick];
        }
        if at(cols.search()) {
            self.focus = Some(FocusTarget::Search);
            self.search.focused = true;
            return Vec::new();
        }
        if self.focus == Some(FocusTarget::Search) {
            self.focus = None;
            self.search.focused = false;
        }

        if at(cols.button(0)) {
            return vec![Action::ButtonClick, Action::Done];
        }
        if self.changed && at(cols.button(1)) {
            self.undo(s);
            return vec![Action::ButtonClick, Action::Changed];
        }

        for (i, page) in Page::ALL.into_iter().enumerate() {
            if at(cols.page(i)) && self.has(page, &lines) {
                self.focus = Some(FocusTarget::Page(page));
                self.pick(page, &lines, &cols, content);
                return vec![Action::ButtonClick];
            }
        }

        if y < cols.top || y >= cols.bottom {
            return Vec::new();
        }
        let Some(item) = lines.iter().find_map(|line| {
            let setting = line.setting?;
            let ry = cols.top + line.y - self.scroll.amount;
            at([cols.list_x, ry, cols.list_w, style::ROW]).then_some(setting.item)
        }) else {
            return Vec::new();
        };

        if item.blocked(s).is_some() {
            return Vec::new();
        }
        self.focus = Some(FocusTarget::Item(item));
        match item {
            Item::Horizon(H::HdrSettings) => vec![Action::ButtonClick, Action::OpenHdr],
            Item::Horizon(H::HdrKey) => {
                self.changed = true;
                vec![
                    Action::ButtonClick,
                    Action::BindKey(crate::graphics::HDR_KEY),
                ]
            }
            _ if item.slider(s).is_some() => {
                if x >= cols.track().0 - 6.0 {
                    self.dragging = Some(item);
                    item.set_slider(s, cols.slider_at(x));
                    self.changed = true;
                    vec![Action::SliderDrag]
                } else {
                    Vec::new()
                }
            }
            _ => {
                item.cycle(s, right, framebuffer);
                self.changed = true;
                vec![Action::ButtonClick, Action::Changed]
            }
        }
    }

    pub fn release(&mut self) -> bool {
        self.scroll.release();
        self.dragging.take().is_some()
    }

    pub fn scroll(&mut self, _level: Level, s: &Settings, h: f32, amount: f32) {
        let (cols, _, content) = self.layout(s, 480.0, h);
        self.picked = None;
        self.scroll
            .wheel(amount, 18.0, cols.top, cols.bottom, content);
    }

    pub fn scroll_sized(&mut self, s: &Settings, (w, h): (f32, f32), amount: f32) {
        let (cols, _, content) = self.layout(s, w, h);
        self.picked = None;
        self.scroll
            .wheel(amount, 18.0, cols.top, cols.bottom, content);
    }

    pub fn type_text(&mut self, text: &str) {
        if self.focus == Some(FocusTarget::Search) {
            self.search.insert_text(text);
            self.scroll.amount = 0.0;
            self.scroll_visual = 0.0;
            self.picked = None;
        }
    }

    pub fn key(
        &mut self,
        s: &mut Settings,
        key: winit::keyboard::KeyCode,
        text: Option<&str>,
        shift: bool,
    ) -> (Vec<Action>, bool) {
        use winit::keyboard::KeyCode as K;
        self.keep(s);
        if key == K::Escape {
            if self.focus == Some(FocusTarget::Search) {
                self.focus = None;
                self.search.focused = false;
                return (Vec::new(), true);
            }
            return (Vec::new(), false);
        }
        if self.focus == Some(FocusTarget::Search) {
            let before = self.search.value.clone();
            if !self.search.key(key, false, shift)
                && let Some(t) = text
            {
                let clean: String = t.chars().filter(|c| !c.is_control()).collect();
                self.search.insert_text(&clean);
            }
            if self.search.value != before {
                self.scroll.amount = 0.0;
                self.scroll_visual = 0.0;
                self.picked = None;
            }
            return (Vec::new(), true);
        }
        (Vec::new(), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_five_pages_populate_and_undo_restores_settings() {
        let mut s = Settings::default();
        let mut ui = GraphicsUi::default();
        for p in Page::ALL {
            ui.open(p);
            let (_, lines, _) = ui.layout(&s, 640.0, 360.0);
            assert!(!lines.is_empty(), "page {p:?} must have rows");
        }
        ui.open(Page::General);
        let cols = Columns::of(640.0, 360.0);
        ui.keep(&s);
        let orig_vsync = s.vsync;
        s.vsync = !orig_vsync;
        ui.changed = true;
        let undo_btn = cols.button(1);
        let actions = ui.click(
            Level::Simple,
            &mut s,
            (undo_btn[0] + 4.0, undo_btn[1] + 4.0),
            (640.0, 360.0),
            [1280, 720],
            false,
            false,
        );
        assert!(actions.contains(&Action::Changed));
        assert_eq!(s.vsync, orig_vsync);
        assert!(!ui.changed);
    }

    #[test]
    fn hdr_controls_report_blocked_when_unoffered_or_disabled() {
        let s = Settings::default();
        if !crate::graphics::hdr_offered() {
            assert!(Item::Horizon(H::Hdr).blocked(&s).is_some());
        }
        assert!(Item::Horizon(H::HdrPeak).blocked(&s).is_some());
    }
}
