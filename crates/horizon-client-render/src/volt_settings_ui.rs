//! Volt 0.9.9 Split-Pane Frosted-Glass `Volt Settings` UI (`volt_settings_ui`)
//! matching Image 1: left sidebar (`Volt / Volt Settings`, `Search options`,
//! 8 smooth-gliding category tabs, warm amber `Undo` + violet `Done`), right
//! frosted glass groups with checkboxes, sliders, `< value >` cyclers & action
//! buttons, and bottom description + `Cost` dock.

use std::time::Instant;

use crate::{
    extras::{self, Item as ExtraItem},
    graphics_style::{self as style, Cost},
    graphics_ui::Columns,
    hud::{HudAtlas, HudBatch},
    lang,
    options_volt_pages::{BOOST_PLUS_ITEMS, CHAT_SECTIONS, MODS_SECTIONS, QOL_SECTIONS, VoltAct},
    settings::{Opt, Settings},
    text::Text,
    ui,
    volt_mods::{VoltModId, apply_mod_to_engine, sync_catalog_from_engine},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum VoltTab {
    #[default]
    Boost,
    Visuals,
    Shaders,
    HudQol,
    ChatVoice,
    Maps,
    Controls,
    Extras,
}

impl VoltTab {
    pub const ALL: [Self; 8] = [
        Self::Boost,
        Self::Visuals,
        Self::Shaders,
        Self::HudQol,
        Self::ChatVoice,
        Self::Maps,
        Self::Controls,
        Self::Extras,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Boost => "Boost",
            Self::Visuals => "Visuals",
            Self::Shaders => "Shaders",
            Self::HudQol => "HUD & QoL",
            Self::ChatVoice => "Chat & Voice",
            Self::Maps => "Maps",
            Self::Controls => "Controls",
            Self::Extras => "Extras",
        }
    }

    pub const fn about(self) -> &'static str {
        match self {
            Self::Boost => "Resolution Scaler (25%..=200%), Volt Boost FPS profiles (LOD kept OFF) & Vulkan culling.",
            Self::Visuals => "Volt Shaders gateway, Connected Glass, Better Grass, 3D Items, Fullbright & Dynamic Lights.",
            Self::Shaders => "Native WGSL Volt Shaders (Potato..Custom, Moving Vegetation, BSL & Complementary lighting).",
            Self::HudQol => "PvP HUDs, Zoomify, Freelook, Viewmodel customization, Sound Physics & Mod Menu.",
            Self::ChatVoice => "3D Positional Proximity Voice Chat, Chat Heads, timestamps & spam compaction.",
            Self::Maps => "Xaero's Minimap, Xaero's World Map & 3D world waypoints.",
            Self::Controls => "Controls Studio presets (Pojav, Zalith, FCL, PvP Claw), touch layout & gamepads.",
            Self::Extras => "Mod Menu (50 native Rust mods), Shader Packs, Video Settings, inventory & system tools.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoltRowKind {
    Opt(Opt),
    Extra(ExtraItem),
    Volt(VoltAct),
    Mod(VoltModId),
    OpenModMenu,
    OpenVideoSettings,
    OpenShaderPacks,
    OpenWorldMap,
    OpenWaypointsManager,
    OpenPadBinds,
    OpenTouchEditor,
    OpenAgent,
}

#[derive(Clone, Copy)]
struct VoltEntry {
    kind: VoltRowKind,
    cost: Cost,
    desc: &'static str,
}

#[derive(Clone, Copy)]
struct Line {
    y: f32,
    tab: VoltTab,
    group: Option<(&'static str, f32)>,
    entry: Option<VoltEntry>,
}

impl Line {
    fn h(&self) -> f32 {
        if self.entry.is_some() {
            style::ROW
        } else if self.group.is_some() {
            style::GROUP_ROW
        } else {
            style::PAGE_ROW
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoltUiAction {
    ButtonClick,
    Changed,
    SliderDrag,
    BindKey(&'static str),
    OpenModMenu,
    OpenVideoSettings,
    OpenShaderPacks,
    OpenWorldMap,
    OpenWaypointsManager,
    OpenPadBinds,
    OpenTouchEditor,
    OpenAgent,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Search,
    Tab(VoltTab),
    Row(VoltRowKind),
    Undo,
    Done,
}

fn e_opt(opt: Opt, cost: Cost, desc: &'static str) -> VoltEntry {
    VoltEntry {
        kind: VoltRowKind::Opt(opt),
        cost,
        desc,
    }
}

fn e_ext(item: ExtraItem, cost: Cost) -> VoltEntry {
    VoltEntry {
        kind: VoltRowKind::Extra(item),
        cost: item.cost().unwrap_or(cost),
        desc: item.description(),
    }
}

fn e_volt(act: VoltAct, cost: Cost) -> VoltEntry {
    VoltEntry {
        kind: VoltRowKind::Volt(act),
        cost,
        desc: act.description(),
    }
}

fn e_mod(id: VoltModId, cost: Cost) -> VoltEntry {
    let m = id.meta();
    VoltEntry {
        kind: VoltRowKind::Mod(id),
        cost,
        desc: m.description,
    }
}

fn tab_groups(s: &Settings) -> Vec<(VoltTab, Vec<(&'static str, Vec<VoltEntry>)>)> {
    use Cost::*;

    // 1. BOOST
    let mut boost_groups = vec![
        (
            "Resolution & Boost Profiles",
            vec![
                e_opt(
                    Opt::VoltResolutionScale,
                    Saves,
                    "Internal 3D world resolution scale (25%..=200%) while keeping HUD crisp.",
                ),
                e_ext(ExtraItem::VoltPreset, Saves),
                e_ext(ExtraItem::VoltPerfHud, None),
                e_ext(ExtraItem::VoltPerfHudCorner, None),
            ],
        ),
        (
            "Vulkan FPS Engine+",
            BOOST_PLUS_ITEMS
                .iter()
                .map(|&a| e_volt(a, Saves))
                .collect(),
        ),
        (
            "Performance Mods (Live Sync)",
            vec![
                e_mod(VoltModId::Sodium, Saves),
                e_mod(VoltModId::Lithium, Saves),
                e_mod(VoltModId::FerriteCore, Saves),
                e_mod(VoltModId::ImmediatelyFast, Saves),
                e_mod(VoltModId::EntityCulling, Saves),
                e_mod(VoltModId::MoreCulling, Saves),
                e_mod(VoltModId::DynamicFps, Saves),
                e_mod(VoltModId::ModernFix, Saves),
                e_mod(VoltModId::BadOptimizations, Saves),
                e_mod(VoltModId::Krypton, Saves),
            ],
        ),
    ];
    let mut tier_rows = Vec::new();
    for r in extras::volt_boost_rows() {
        if let Ok(item) = r
            && !matches!(
                item,
                ExtraItem::VoltPreset | ExtraItem::VoltPerfHud | ExtraItem::VoltPerfHudCorner
            )
        {
            tier_rows.push(e_ext(item, Saves));
        }
    }
    if !tier_rows.is_empty() {
        boost_groups.push(("Granular Engine Caps & Culling", tier_rows));
    }

    // 2. VISUALS (includes Volt Shaders master & quick jump)
    let visuals_groups = vec![
        (
            "Shaders & Graphics Pipeline",
            vec![
                e_ext(ExtraItem::VoltShaderEnable, Medium),
                VoltEntry {
                    kind: VoltRowKind::OpenShaderPacks,
                    cost: None,
                    desc: "Browse and select external .hshader / shader packs.",
                },
                VoltEntry {
                    kind: VoltRowKind::OpenVideoSettings,
                    cost: None,
                    desc: "Open Split-Pane Video & Graphics Settings (General, Quality, Performance, Advanced, HDR).",
                },
            ],
        ),
        (
            "Connected Textures, Foliage & 3D Items",
            vec![
                e_volt(VoltAct::VisualsCtmGlass, Low),
                e_volt(VoltAct::VisualsBetterGrass, Low),
                e_volt(VoltAct::VisualsDroppedItems3d, Low),
            ],
        ),
        (
            "Visual Mods & Lighting",
            vec![
                e_mod(VoltModId::Fullbright, None),
                e_mod(VoltModId::LambDynamicLights, Low),
                e_mod(VoltModId::LowFire, None),
                e_mod(VoltModId::NoFog, Saves),
                e_mod(VoltModId::BetterClouds, Low),
                e_mod(VoltModId::EnhancedBlockEntities, Saves),
                e_mod(VoltModId::EntityModelFeatures, Low),
                e_mod(VoltModId::EntityTextureFeatures, Low),
                e_mod(VoltModId::BetterAnimations, Low),
            ],
        ),
    ];

    // 3. SHADERS (Standing rule: when Enable Shaders is OFF, hide ALL child customization controls!)
    let shaders_groups = if !s.extras.volt_shaders.enabled {
        vec![(
            "Volt Native WGSL Shaders",
            vec![e_ext(ExtraItem::VoltShaderEnable, None)],
        )]
    } else {
        vec![
            (
                "Master & Profiles",
                vec![
                    e_ext(ExtraItem::VoltShaderEnable, Medium),
                    e_ext(ExtraItem::VoltShaderProfile, Medium),
                    e_volt(VoltAct::ShaderStyle, Low),
                    e_volt(VoltAct::ShaderEyeComfort, None),
                    e_ext(ExtraItem::VoltShaderReset, None),
                ],
            ),
            (
                "Moving Vegetation & Wind",
                vec![
                    e_ext(ExtraItem::VoltShaderWavePlants, Low),
                    e_ext(ExtraItem::VoltShaderWaveLeaves, Low),
                    e_ext(ExtraItem::VoltShaderWaveWater, Low),
                    e_ext(ExtraItem::VoltShaderStormWind, None),
                    e_opt(
                        Opt::VoltShaderWindSpeed,
                        None,
                        "Controls waving speed of grass, crops, vines and tree leaves.",
                    ),
                    e_opt(
                        Opt::VoltShaderWindStrength,
                        None,
                        "Controls sway amplitude of foliage in calm and stormy weather.",
                    ),
                ],
            ),
            (
                "BSL & Complementary Lighting & PBR",
                vec![
                    e_ext(ExtraItem::VoltShaderSunShading, Low),
                    e_ext(ExtraItem::VoltShaderFoliageGlow, Low),
                    e_opt(
                        Opt::VoltShaderShadowRichness,
                        Low,
                        "Controls ambient shadow depth on unlit block faces.",
                    ),
                    e_volt(VoltAct::ShaderAutoPbr, Low),
                    e_volt(VoltAct::ShaderGlowingOres, Low),
                    e_volt(VoltAct::ShaderOreGlow, None),
                    e_volt(VoltAct::ShaderCloudShadows, Low),
                    e_ext(ExtraItem::VoltShaderRainWetness, Low),
                    e_ext(ExtraItem::VoltShaderTorchStyle, None),
                    e_opt(
                        Opt::VoltShaderTorchIntensity,
                        None,
                        "Controls block-light warmth and falloff strength.",
                    ),
                    e_opt(
                        Opt::VoltShaderEmissiveGlow,
                        Low,
                        "Controls self-illumination boost on lava, lanterns and glowing blocks.",
                    ),
                ],
            ),
            (
                "Atmosphere, Clouds & Aurora",
                vec![
                    e_volt(VoltAct::ShaderCloudsMode, Low),
                    e_volt(VoltAct::ShaderCloudCoverage, None),
                    e_ext(ExtraItem::VoltShaderCustomSky, Low),
                    e_ext(ExtraItem::VoltShaderSunHalo, Low),
                    e_ext(ExtraItem::VoltShaderAurora, Low),
                    e_volt(VoltAct::ShaderNightSkyPlus, Low),
                    e_volt(VoltAct::ShaderRainbow, Low),
                    e_ext(ExtraItem::VoltShaderDepthHaze, None),
                    e_opt(
                        Opt::VoltShaderSunsetGlow,
                        None,
                        "Golden-hour horizon warmth at sunrise and sunset.",
                    ),
                ],
            ),
            (
                "Water & Post-Processing",
                vec![
                    e_ext(ExtraItem::VoltShaderWaterRipples, Low),
                    e_volt(VoltAct::ShaderWaterFoam, Low),
                    e_ext(ExtraItem::VoltShaderWaterGlint, Low),
                    e_ext(ExtraItem::VoltShaderClearWater, None),
                    e_ext(ExtraItem::VoltShaderUnderwaterRefract, Low),
                    e_opt(
                        Opt::VoltShaderWaterReflection,
                        Medium,
                        "Sky and sun reflection strength across water surfaces.",
                    ),
                    e_ext(ExtraItem::VoltShaderTonemap, None),
                    e_ext(ExtraItem::VoltShaderBloom, Medium),
                    e_ext(ExtraItem::VoltShaderSunRays, Medium),
                    e_ext(ExtraItem::VoltShaderVignette, None),
                    e_opt(Opt::VoltShaderVibrance, None, "Smart color boost preserving skin and sky tones."),
                    e_opt(Opt::VoltShaderSaturation, None, "Global color saturation."),
                    e_opt(Opt::VoltShaderContrast, None, "Midtone contrast curve."),
                    e_opt(Opt::VoltShaderExposure, None, "Overall shader brightness exposure."),
                    e_opt(Opt::VoltShaderWarmth, None, "Color temperature balance."),
                ],
            ),
        ]
    };

    // 4. HUD & QoL
    let mut hud_groups = vec![
        (
            "Built-in Mod Menu & PvP HUDs",
            vec![
                VoltEntry {
                    kind: VoltRowKind::OpenModMenu,
                    cost: None,
                    desc: "Open the full Split-Pane Mod Menu to browse, search and toggle all 50 native Rust mods.",
                },
                e_mod(VoltModId::InventoryHudPlus, Low),
                e_mod(VoltModId::ArmorHud, None),
                e_mod(VoltModId::StatusEffectTimer, None),
                e_mod(VoltModId::KeystrokesHud, None),
                e_mod(VoltModId::CpsDisplay, None),
                e_mod(VoltModId::PingDisplay, None),
                e_mod(VoltModId::FpsDisplay, None),
                e_mod(VoltModId::CoordinatesHud, None),
                e_mod(VoltModId::DurabilityDisplay, None),
                e_mod(VoltModId::CustomCrosshair, None),
                e_mod(VoltModId::DynamicCrosshair, None),
                e_mod(VoltModId::BetterF3, None),
                e_mod(VoltModId::Punchy, None),
            ],
        ),
    ];
    for &(sec_name, acts) in QOL_SECTIONS {
        hud_groups.push((sec_name, acts.iter().map(|&a| e_volt(a, Low)).collect()));
    }
    for &(sec_name, acts) in MODS_SECTIONS {
        hud_groups.push((sec_name, acts.iter().map(|&a| e_volt(a, None)).collect()));
    }

    // 5. CHAT & VOICE
    let mut chat_groups = vec![(
        "Chat Mods",
        vec![e_mod(VoltModId::ChatHeads, None), e_mod(VoltModId::BetterPingDisplay, None)],
    )];
    for &(sec_name, acts) in CHAT_SECTIONS {
        chat_groups.push((sec_name, acts.iter().map(|&a| e_volt(a, Low)).collect()));
    }

    // 6. MAPS & WAYPOINTS
    let mut map_rows = vec![
        e_mod(VoltModId::XaerosMinimap, Low),
        e_mod(VoltModId::XaerosWorldMap, Low),
        VoltEntry {
            kind: VoltRowKind::OpenWorldMap,
            cost: None,
            desc: "Open Xaero's World Map fullscreen explored-chunk viewer.",
        },
        VoltEntry {
            kind: VoltRowKind::OpenWaypointsManager,
            cost: None,
            desc: "Manage saved per-world waypoints, colors, death markers and quick teleport.",
        },
    ];
    for r in extras::minimap_rows() {
        if let Ok(item) = r {
            map_rows.push(e_ext(item, Low));
        }
    }
    let mut wp_rows = Vec::new();
    for r in extras::waypoint_rows() {
        if let Ok(item) = r {
            wp_rows.push(e_ext(item, Low));
        }
    }
    let maps_groups = vec![
        ("Xaero's Minimap & World Map", map_rows),
        ("3D World Waypoints", wp_rows),
    ];

    // 7. CONTROLS (Touch Presets + Gamepad)
    let mut touch_vec = vec![
        e_opt(
            Opt::TouchPreset,
            None,
            "One-tap mobile HUD layout presets: Pojav, Zalith, FCL, PvP Claw & Volt Default.",
        ),
        e_opt(Opt::TouchMode, None, "Touch aiming & interaction scheme."),
        e_opt(Opt::TouchSensitivity, None, "Touch camera look sensitivity."),
        e_opt(Opt::TouchButtonScale, None, "Scale of on-screen touch action buttons."),
        e_opt(Opt::TouchCircleSize, None, "Size of the touch interaction circle."),
        e_opt(Opt::TouchAnalog, None, "Smooth 360-degree analog joystick movement."),
        e_opt(Opt::TouchOffhand, None, "Show quick offhand swap button on touch HUD."),
        e_opt(Opt::TouchShield, None, "Show dedicated Shield guard button on touch HUD."),
        e_opt(Opt::TouchSprintButton, None, "Show dedicated Sprint button on touch HUD."),
        e_opt(Opt::TouchSprintToggle, None, "Latch Sprint on tap instead of holding."),
        e_opt(Opt::AndroidGyroscope, None, "Precision gyroscope motion aiming on Android."),
        e_opt(Opt::AndroidGyroSensitivity, None, "Gyroscope aiming speed."),
    ];
    if cfg!(target_os = "android") {
        touch_vec.push(VoltEntry {
            kind: VoltRowKind::OpenTouchEditor,
            cost: None,
            desc: "Drag, resize and customize every on-screen touch button directly.",
        });
    }
    let controls_groups = vec![
        ("Touch Controls & Presets", touch_vec),
        (
            "Controller & Gamepad",
            vec![
                VoltEntry {
                    kind: VoltRowKind::OpenPadBinds,
                    cost: None,
                    desc: "Configure controller button bindings.",
                },
                e_opt(Opt::PadEnabled, None, "Enable gamepad / controller input."),
                e_opt(Opt::PadLookSensitivity, None, "Controller right-stick look speed."),
                e_opt(Opt::PadVibration, None, "Controller haptic rumble feedback."),
                e_opt(Opt::PadMoveDeadzone, None, "Left stick inner deadzone."),
                e_opt(Opt::PadLookDeadzone, None, "Right stick inner deadzone."),
            ],
        ),
    ];

    // 8. EXTRAS
    let mut ext_rows = vec![
        VoltEntry {
            kind: VoltRowKind::OpenModMenu,
            cost: None,
            desc: "Open the in-game Mod Menu (50 native Rust mods with live toggles).",
        },
        e_mod(VoltModId::MouseTweaks, None),
        e_mod(VoltModId::ItemScroller, None),
        e_mod(VoltModId::Controlling, None),
        e_mod(VoltModId::BetterStats, None),
        e_mod(VoltModId::ScreenshotToClipboard, None),
        e_mod(VoltModId::BetterMountHud, None),
        e_mod(VoltModId::NotEnoughAnimations, None),
        e_mod(VoltModId::LanguageReload, Saves),
        e_mod(VoltModId::FastQuit, Saves),
    ];
    for r in extras::rows() {
        if let Ok(item) = r {
            ext_rows.push(e_ext(item, None));
        }
    }
    ext_rows.push(e_ext(ExtraItem::DiscordPresence, None));
    ext_rows.push(e_ext(ExtraItem::NoAi, None));
    if !s.extras.no_ai {
        ext_rows.push(VoltEntry {
            kind: VoltRowKind::OpenAgent,
            cost: None,
            desc: "Configure AI Agent socket & connection status.",
        });
    }
    let extras_groups = vec![("Mods, Inventory & System Extras", ext_rows)];

    vec![
        (VoltTab::Boost, boost_groups),
        (VoltTab::Visuals, visuals_groups),
        (VoltTab::Shaders, shaders_groups),
        (VoltTab::HudQol, hud_groups),
        (VoltTab::ChatVoice, chat_groups),
        (VoltTab::Maps, maps_groups),
        (VoltTab::Controls, controls_groups),
        (VoltTab::Extras, extras_groups),
    ]
}

fn entry_parts(kind: VoltRowKind, s: &Settings, binding: Option<&'static str>) -> (String, Option<String>) {
    match kind {
        VoltRowKind::Opt(opt) => {
            let lbl = opt.label(s).plain();
            match lbl.split_once(": ") {
                Some((n, v)) => (n.to_owned(), Some(v.to_owned())),
                None => (lbl, None),
            }
        }
        VoltRowKind::Extra(item) => {
            if let Some(kname) = item.key_name()
                && binding == Some(kname)
            {
                return (
                    item.caption().to_owned(),
                    Some(format!("> {} <", item.value(&s.extras))),
                );
            }
            let val = item.value(&s.extras);
            if val.is_empty() {
                (item.caption().to_owned(), None)
            } else {
                (item.caption().to_owned(), Some(val))
            }
        }
        VoltRowKind::Volt(act) => (act.caption().to_owned(), Some(act.value(&s.extras))),
        VoltRowKind::Mod(id) => {
            let meta = id.meta();
            let on = s.extras.mod_catalog.is_on(id);
            (
                meta.title.to_owned(),
                Some(if on { "ON" } else { "OFF" }.to_owned()),
            )
        }
        VoltRowKind::OpenModMenu => (
            format!(
                "Open Mod Menu ({}/50 Enabled)...",
                s.extras.mod_catalog.enabled_count()
            ),
            None,
        ),
        VoltRowKind::OpenVideoSettings => ("Open Video & Graphics Settings...".to_owned(), None),
        VoltRowKind::OpenShaderPacks => ("Browse External Shader Packs...".to_owned(), None),
        VoltRowKind::OpenWorldMap => ("Open Xaero's World Map...".to_owned(), None),
        VoltRowKind::OpenWaypointsManager => ("Manage Saved World Waypoints...".to_owned(), None),
        VoltRowKind::OpenPadBinds => ("Controller Button Binds...".to_owned(), None),
        VoltRowKind::OpenTouchEditor => ("Customize Touch Layout...".to_owned(), None),
        VoltRowKind::OpenAgent => ("AI Agent Settings...".to_owned(), None),
    }
}

fn entry_switch(kind: VoltRowKind, s: &Settings) -> Option<bool> {
    match kind {
        VoltRowKind::Extra(item) => item.switch(&s.extras),
        VoltRowKind::Mod(id) => Some(s.extras.mod_catalog.is_on(id)),
        VoltRowKind::Volt(act) => match act.value(&s.extras).as_str() {
            "ON" => Some(true),
            "OFF" => Some(false),
            _ => None,
        },
        VoltRowKind::Opt(opt) => match opt {
            Opt::TouchAnalog => Some(s.touch.analog_movement),
            Opt::TouchOffhand => Some(s.touch.visible(crate::touch::Control::Offhand)),
            Opt::TouchShield => Some(s.touch.visible(crate::touch::Control::Shield)),
            Opt::TouchSprintButton => Some(s.touch.visible(crate::touch::Control::Sprint)),
            Opt::TouchSprintToggle => Some(s.touch.sprint_toggle),
            Opt::PadEnabled => Some(s.controller.enabled),
            Opt::PadVibration => Some(s.controller.vibration),
            _ => None,
        },
        _ => None,
    }
}

fn entry_slider(kind: VoltRowKind, s: &Settings) -> Option<f32> {
    match kind {
        VoltRowKind::Opt(opt) => opt.slider_position(s),
        _ => None,
    }
}

pub struct VoltSettingsUi {
    scroll: ui::Scroll,
    dragging: Option<Opt>,
    pub focus: Option<Focus>,
    pub binding: Option<&'static str>,
    pub search: ui::EditBox,
    pub tab: VoltTab,
    picked: Option<VoltTab>,
    before: Option<Box<Settings>>,
    pub changed: bool,
    // Smooth animation state
    last_frame: Option<Instant>,
    tab_anim_y: f32,
    scroll_visual: f32,
    page_slide: f32,
}

impl Default for VoltSettingsUi {
    fn default() -> Self {
        let mut search = ui::EditBox::new(0.0, 0.0, 0.0, 8.0);
        search.bordered = false;
        search.focused = false;
        Self {
            scroll: ui::Scroll::default(),
            dragging: None,
            focus: None,
            binding: None,
            search,
            tab: VoltTab::Boost,
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

impl VoltSettingsUi {
    pub fn open(&mut self, tab: VoltTab) {
        *self = Self {
            tab,
            ..Self::default()
        };
    }

    pub fn typing(&self) -> bool {
        self.focus == Some(Focus::Search)
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
        s.extras = b.extras.clone();
        s.horizon = b.horizon.clone();
        s.touch = b.touch.clone();
        s.controller = b.controller.clone();
        self.changed = false;
    }

    fn lines(&self, s: &Settings) -> (Vec<Line>, f32) {
        let query = self.query();
        let mut lines = Vec::new();
        let mut y: f32 = 0.0;
        for (tab, groups) in tab_groups(s) {
            if query.is_empty() && tab != self.tab {
                continue;
            }
            let groups: Vec<(&'static str, Vec<VoltEntry>)> = groups
                .into_iter()
                .map(|(gname, entries)| {
                    let entries = entries
                        .into_iter()
                        .filter(|e| {
                            if let VoltRowKind::Extra(it) = e.kind
                                && let Some(parent) = it.parent()
                                && parent.switch(&s.extras) == Some(false)
                            {
                                return false;
                            }
                            if query.is_empty()
                                || self.focus == Some(Focus::Row(e.kind))
                                || matches!((self.dragging, e.kind), (Some(o1), VoltRowKind::Opt(o2)) if o1 == o2)
                            {
                                return true;
                            }
                            let (nm, _) = entry_parts(e.kind, s, None);
                            nm.to_lowercase().contains(&query)
                                || e.desc.to_lowercase().contains(&query)
                        })
                        .collect::<Vec<_>>();
                    (gname, entries)
                })
                .filter(|(_, entries)| !entries.is_empty())
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
                    tab,
                    group: None,
                    entry: None,
                });
                y += style::PAGE_ROW;
            }
            for (gname, entries) in groups {
                lines.push(Line {
                    y,
                    tab,
                    group: Some((gname, entries.len() as f32 * style::ROW)),
                    entry: None,
                });
                y += style::GROUP_ROW;
                for entry in entries {
                    lines.push(Line {
                        y,
                        tab,
                        group: None,
                        entry: Some(entry),
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

    fn tab_rect(&self, cols: &Columns, i: usize) -> [f32; 4] {
        // Fit all 8 tabs cleanly between the search bar and Undo/Done even at 240p GUI height
        let start_y = cols.side_y + 44.0;
        let avail = (cols.button(1)[1] - 4.0 - start_y).max(96.0);
        let step = (avail / VoltTab::ALL.len() as f32).clamp(12.0, 17.0).floor();
        let th = (step - 1.0).clamp(11.0, 15.0);
        [cols.side_x, start_y + i as f32 * step, style::CONTROL_W, th]
    }

    fn has(&self, tab: VoltTab, lines: &[Line]) -> bool {
        self.query().is_empty() || lines.iter().any(|l| l.tab == tab)
    }

    fn current(&self, lines: &[Line]) -> Option<VoltTab> {
        if self.query().is_empty() {
            return Some(self.tab);
        }
        self.picked.or_else(|| {
            lines
                .iter()
                .find(|l| l.y + l.h() > self.scroll.amount)
                .map(|l| l.tab)
        })
    }

    fn pick(&mut self, tab: VoltTab, lines: &[Line], cols: &Columns, content: f32) {
        if self.tab != tab {
            self.page_slide = 0.0;
        }
        self.tab = tab;
        self.scroll.amount = 0.0;
        self.scroll_visual = 0.0;
        if !self.query().is_empty()
            && let Some(line) = lines.iter().find(|l| l.tab == tab)
        {
            self.scroll.amount = line.y;
            self.scroll.clamp(cols.top, cols.bottom, content);
            self.scroll_visual = self.scroll.amount;
            self.picked = Some(tab);
        }
    }

    fn step_animations(&mut self, cols: &Columns, current: Option<VoltTab>) {
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
        if let Some(t) = current
            && let Some(idx) = VoltTab::ALL.iter().position(|&x| x == t)
        {
            let target_y = self.tab_rect(cols, idx)[1];
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

        // Left sidebar header + tabs + search
        let (sx, sy) = (cols.side_x, cols.side_y);
        style::fill(batch, atlas, [sx, sy + 1.0, 6.0, 6.0], style::ACCENT);
        batch.text_styled(
            atlas,
            &Text::literal("Volt"),
            sx + 9.0,
            sy,
            style::WHITE,
            true,
        );
        batch.text_styled(
            atlas,
            &Text::literal("Volt Settings"),
            sx,
            sy + 10.0,
            style::MUTED,
            true,
        );

        let th = self.tab_rect(&cols, 0)[3];
        if current.is_some() {
            style::fill(
                batch,
                atlas,
                [cols.side_x, self.tab_anim_y, style::CONTROL_W, th],
                style::OPEN,
            );
            style::fill(
                batch,
                atlas,
                [cols.side_x, self.tab_anim_y, 2.0, th],
                style::ACCENT,
            );
        }

        for (i, tab) in VoltTab::ALL.into_iter().enumerate() {
            let rect = self.tab_rect(&cols, i);
            let [px, py, pw, ph] = rect;
            let has = self.has(tab, &lines);
            let open = current == Some(tab);
            crate::widgets::record_as(tab.name(), rect, has, crate::widgets::Kind::Tab(open));
            if has {
                crate::widgets::target_as(rect, crate::widgets::Kind::Tab(open));
            }
            let hov = has && ui::hovered(cursor, px, py, pw, ph);
            let focused = self.focus == Some(Focus::Tab(tab));
            if open || focused {
                style::ring(batch, atlas, rect, style::WHITE);
            } else if hov {
                style::fill(batch, atlas, rect, style::HOVER);
            }
            let col = match (open || hov, has) {
                (true, _) => style::WHITE,
                (false, true) => style::SOFT,
                (false, false) => style::MUTED,
            };
            let ty = py + ((ph - 8.0) / 2.0).floor();
            batch.text_styled(
                atlas,
                &Text::literal(style::fit(atlas, tab.name(), pw - 12.0)),
                px + 8.0,
                ty,
                col,
                true,
            );
        }

        // Search field
        let srect = cols.search();
        style::well(batch, atlas, srect);
        let sfocus = self.focus == Some(Focus::Search);
        self.search.focused = sfocus;
        let clear = cols.clear();
        (self.search.x, self.search.y) = (srect[0] + 5.0, srect[1] + 4.0);
        self.search.width = clear[0] - self.search.x;
        self.search.text_color = style::WHITE;
        self.search.draw(batch, atlas, None);
        if self.search.value.is_empty() && !sfocus {
            batch.text_styled(
                atlas,
                &Text::literal(&lang::get("Search options")),
                srect[0] + 5.0,
                self.search.y,
                style::MUTED,
                true,
            );
        } else if !self.search.value.is_empty() {
            let ch = ui::hovered(cursor, clear[0], clear[1], clear[2], clear[3]);
            ui::centered_text(
                batch,
                atlas,
                &Text::literal("x"),
                clear[0],
                srect[1] + 4.0,
                clear[2],
                if ch { style::WHITE } else { style::MUTED },
            );
        }
        style::ring(
            batch,
            atlas,
            srect,
            if sfocus { style::WHITE } else { style::EDGE },
        );

        // Right frosted glass plate
        let [px, py, pw, _] = cols.plate;
        style::panel(batch, atlas, cols.plate);
        let query = self.search.value.trim();
        let heading = if query.is_empty() {
            current.unwrap_or(self.tab).name().to_owned()
        } else {
            format!("Results for \"{query}\"")
        };
        batch.text_styled(
            atlas,
            &Text::literal(style::fit(atlas, &heading, pw - 24.0)),
            px + 12.0,
            py + 10.0,
            style::WHITE,
            true,
        );
        style::fill(
            batch,
            atlas,
            [px + 8.0, py + 23.0, pw - 16.0, 1.0],
            style::HEAD_RULE,
        );

        let over_list = cursor.1 >= cols.top && cursor.1 < cols.bottom;
        let start = batch.vertices.len();
        let mark = crate::widgets::targets_mark();
        let mut described: Option<VoltEntry> = None;
        let shows = |y: f32, rh: f32| y + rh > cols.top && y < cols.bottom;

        for line in &lines {
            let y = self.row_y(&cols, line);
            if let Some((_, panel)) = line.group
                && shows(y + style::GROUP_ROW, panel)
            {
                style::group(
                    batch,
                    atlas,
                    [cols.list_x, y + style::GROUP_ROW, cols.list_w, panel],
                );
            }
        }

        let mut first = false;
        for line in &lines {
            let y = self.row_y(&cols, line);
            let after_name = std::mem::replace(&mut first, line.group.is_some());
            if !shows(y, line.h()) {
                continue;
            }
            let Some(entry) = line.entry else {
                let (name, color, dy) = match line.group {
                    Some((name, _)) => (name, style::MUTED, 3.0),
                    None => (line.tab.name(), style::ACCENT, 4.0),
                };
                batch.text_styled(
                    atlas,
                    &Text::literal(name),
                    cols.list_x + 2.0,
                    y + dy,
                    color,
                    true,
                );
                continue;
            };
            if !after_name {
                style::fill(
                    batch,
                    atlas,
                    [cols.list_x + 8.0, y, cols.list_w - 16.0, 1.0],
                    style::ROW_RULE,
                );
            }
            let rect = [cols.list_x, y, cols.list_w, style::ROW];
            let hov = over_list && ui::hovered(cursor, rect[0], rect[1], rect[2], rect[3]);
            let focused = self.focus == Some(Focus::Row(entry.kind));
            let dragging_this =
                matches!((self.dragging, entry.kind), (Some(o1), VoltRowKind::Opt(o2)) if o1 == o2);
            if hov || dragging_this || (focused && described.is_none()) {
                described = Some(entry);
            }
            self.draw_row(batch, atlas, s, &cols, entry, y, hov, focused, dragging_this);
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

        if cols.scrolls()
            && let Some((sy, bar)) =
                style::scrollbar_thumb(cols.top, cols.bottom, content, self.scroll_visual)
        {
            let bx = cols.bar_x();
            style::scroller(
                batch,
                atlas,
                [bx, cols.top, style::SCROLLER_W, cols.bottom - cols.top],
                [bx, sy, style::SCROLLER_W, bar],
            );
        }

        self.draw_dock(batch, atlas, s, &cols, described, current);

        // Bottom-left Undo + Done
        let done_rect = cols.button(0);
        let done_lbl = Text::literal(lang::get("gui.done"));
        let done_hov = ui::hovered(cursor, done_rect[0], done_rect[1], done_rect[2], done_rect[3]);
        crate::widgets::record(&done_lbl.plain(), done_rect, true);
        crate::widgets::target(done_rect);
        style::button(batch, atlas, done_rect, &done_lbl, (done_hov, true, true));

        if self.changed {
            let undo_rect = cols.button(1);
            let undo_lbl = Text::literal(lang::get("Undo"));
            let undo_hov =
                ui::hovered(cursor, undo_rect[0], undo_rect[1], undo_rect[2], undo_rect[3]);
            crate::widgets::record(&undo_lbl.plain(), undo_rect, true);
            crate::widgets::target(undo_rect);
            style::undo_button(batch, atlas, undo_rect, &undo_lbl, undo_hov);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_row(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        s: &Settings,
        cols: &Columns,
        entry: VoltEntry,
        y: f32,
        hover: bool,
        focused: bool,
        dragging: bool,
    ) {
        let rect = [cols.list_x, y, cols.list_w, style::ROW];
        let (name, value) = entry_parts(entry.kind, s, self.binding);
        let (tx, tw) = cols.track();
        let right = tx + tw;
        let control = [
            tx,
            y + (style::ROW - style::CONTROL_H) / 2.0,
            tw,
            style::CONTROL_H,
        ];
        if hover || focused || dragging {
            style::fill(
                batch,
                atlas,
                [rect[0] + 2.0, y + 1.0, rect[2] - 4.0, style::ROW - 2.0],
                style::HOVER,
            );
        }
        let slider = entry_slider(entry.kind, s);
        let switch = entry_switch(entry.kind, s);
        let wkind = match slider {
            Some(v) => crate::widgets::Kind::Slider {
                value: v,
                track: [tx + 4.0, tw - 8.0],
            },
            None => crate::widgets::Kind::Button,
        };
        crate::widgets::record_as(&name, rect, true, wkind);
        crate::widgets::target_as(rect, wkind);

        let ty = y + ((style::ROW - 8.0) / 2.0).floor();
        let lit = hover || dragging;
        let left = rect[0] + style::INSET;

        if switch.is_none() && slider.is_none() && value.is_none() {
            let wide = [left, control[1], right - left, style::CONTROL_H];
            style::button(batch, atlas, wide, &Text::literal(name), (lit, true, false));
        } else {
            let value = value.unwrap_or_default();
            let vw = atlas.font.width(&value);
            let start = match (switch, slider) {
                (Some(_), _) => right - style::CONTROL_H,
                (None, Some(_)) => tx,
                (None, None) => (right - (vw + 20.0).max(58.0)).max(tx),
            };
            match (switch, slider) {
                (Some(on), _) => style::checkbox(batch, atlas, right, y, on, true),
                (None, Some(pos)) => {
                    style::slider(
                        batch,
                        atlas,
                        control,
                        &Text::literal(value),
                        pos,
                        (lit, true),
                    );
                }
                (None, None) => {
                    style::cycler(
                        batch,
                        atlas,
                        [start, y + 3.0, right - start, 14.0],
                        &Text::literal(value),
                        (lit, true),
                    );
                }
            }
            let shown = style::fit(atlas, &name, start - 6.0 - left);
            batch.text_styled(atlas, &Text::literal(shown), left, ty, style::WHITE, true);
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

    fn draw_dock(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        s: &Settings,
        cols: &Columns,
        entry: Option<VoltEntry>,
        current: Option<VoltTab>,
    ) {
        let [x, y, w, h] = cols.dock;
        style::panel(batch, atlas, cols.dock);
        let left = x + 12.0;
        match entry {
            Some(entry) => {
                let (name, _) = entry_parts(entry.kind, s, None);
                let head = style::fit(atlas, &name, w - 120.0);
                batch.text_styled(atlas, &Text::literal(&head), left, y + 7.0, style::WHITE, true);
                let cx = left + atlas.font.width(&head) + 8.0;
                style::fill(
                    batch,
                    atlas,
                    [cx, y + 8.0, 6.0, 6.0],
                    style::cost_color(entry.cost),
                );
                batch.text_styled(
                    atlas,
                    &Text::literal(lang::get(entry.cost.label())),
                    cx + 9.0,
                    y + 7.0,
                    style::cost_color(entry.cost),
                    true,
                );
                let wrapped = style::wrap_lines(atlas, entry.desc, w - 24.0);
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
                let tab = current.unwrap_or(self.tab);
                let line = format!("{} — {}", tab.name(), tab.about());
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
        s: &mut Settings,
        (x, y): (f32, f32),
        (w, h): (f32, f32),
        framebuffer: [i32; 2],
        drag: bool,
        right: bool,
    ) -> Vec<VoltUiAction> {
        self.keep(s);
        let (cols, lines, content) = self.layout(s, w, h);
        if drag {
            if self.scroll.held() {
                self.scroll.drag(y, cols.top, cols.bottom, content);
                self.scroll_visual = self.scroll.amount;
                self.picked = None;
                return Vec::new();
            }
            if let Some(opt) = self.dragging {
                opt.set_slider(s, cols.slider_at(x));
                self.changed = true;
                return vec![VoltUiAction::SliderDrag];
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
            return vec![VoltUiAction::ButtonClick];
        }
        if at(cols.search()) {
            self.focus = Some(Focus::Search);
            self.search.focused = true;
            return Vec::new();
        }
        if self.focus == Some(Focus::Search) {
            self.focus = None;
            self.search.focused = false;
        }

        if at(cols.button(0)) {
            return vec![VoltUiAction::ButtonClick, VoltUiAction::Done];
        }
        if self.changed && at(cols.button(1)) {
            self.undo(s);
            return vec![VoltUiAction::ButtonClick, VoltUiAction::Changed];
        }

        for (i, tab) in VoltTab::ALL.into_iter().enumerate() {
            if at(self.tab_rect(&cols, i)) && self.has(tab, &lines) {
                self.focus = Some(Focus::Tab(tab));
                self.pick(tab, &lines, &cols, content);
                return vec![VoltUiAction::ButtonClick];
            }
        }

        if y < cols.top || y >= cols.bottom {
            return Vec::new();
        }
        let Some(entry) = lines.iter().find_map(|line| {
            let entry = line.entry?;
            let ry = cols.top + line.y - self.scroll.amount;
            at([cols.list_x, ry, cols.list_w, style::ROW]).then_some(entry)
        }) else {
            return Vec::new();
        };

        self.focus = Some(Focus::Row(entry.kind));
        match entry.kind {
            VoltRowKind::OpenModMenu => {
                vec![VoltUiAction::ButtonClick, VoltUiAction::OpenModMenu]
            }
            VoltRowKind::OpenVideoSettings => {
                vec![VoltUiAction::ButtonClick, VoltUiAction::OpenVideoSettings]
            }
            VoltRowKind::OpenShaderPacks => {
                vec![VoltUiAction::ButtonClick, VoltUiAction::OpenShaderPacks]
            }
            VoltRowKind::OpenWorldMap => {
                vec![VoltUiAction::ButtonClick, VoltUiAction::OpenWorldMap]
            }
            VoltRowKind::OpenWaypointsManager => {
                vec![VoltUiAction::ButtonClick, VoltUiAction::OpenWaypointsManager]
            }
            VoltRowKind::OpenPadBinds => {
                vec![VoltUiAction::ButtonClick, VoltUiAction::OpenPadBinds]
            }
            VoltRowKind::OpenTouchEditor => {
                vec![VoltUiAction::ButtonClick, VoltUiAction::OpenTouchEditor]
            }
            VoltRowKind::OpenAgent => vec![VoltUiAction::ButtonClick, VoltUiAction::OpenAgent],
            VoltRowKind::Opt(opt) => {
                if opt.slider_position(s).is_some() {
                    if x >= cols.track().0 - 6.0 {
                        self.dragging = Some(opt);
                        opt.set_slider(s, cols.slider_at(x));
                        self.changed = true;
                        vec![VoltUiAction::SliderDrag]
                    } else {
                        Vec::new()
                    }
                } else {
                    opt.cycle(s, right, framebuffer);
                    self.changed = true;
                    vec![VoltUiAction::ButtonClick, VoltUiAction::Changed]
                }
            }
            VoltRowKind::Extra(item) => {
                if let Some(kname) = item.key_name() {
                    self.changed = true;
                    vec![VoltUiAction::ButtonClick, VoltUiAction::BindKey(kname)]
                } else {
                    item.cycle(&mut s.extras, right);
                    sync_catalog_from_engine(&mut s.extras, &s.horizon);
                    self.changed = true;
                    vec![VoltUiAction::ButtonClick, VoltUiAction::Changed]
                }
            }
            VoltRowKind::Volt(act) => {
                act.trigger(&mut s.extras, right);
                sync_catalog_from_engine(&mut s.extras, &s.horizon);
                self.changed = true;
                vec![VoltUiAction::ButtonClick, VoltUiAction::Changed]
            }
            VoltRowKind::Mod(id) => {
                let next = !s.extras.mod_catalog.is_on(id);
                apply_mod_to_engine(id, next, &mut s.extras, &mut s.horizon);
                self.changed = true;
                vec![VoltUiAction::ButtonClick, VoltUiAction::Changed]
            }
        }
    }

    pub fn release(&mut self) -> bool {
        self.scroll.release();
        self.dragging.take().is_some()
    }

    pub fn scroll(&mut self, s: &Settings, (w, h): (f32, f32), amount: f32) {
        let (cols, _, content) = self.layout(s, w, h);
        self.picked = None;
        self.scroll
            .wheel(amount, 18.0, cols.top, cols.bottom, content);
    }

    pub fn type_text(&mut self, text: &str) {
        if self.focus == Some(Focus::Search) {
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
    ) -> (Vec<VoltUiAction>, bool) {
        use winit::keyboard::KeyCode as K;
        self.keep(s);
        if key == K::Escape {
            if self.focus == Some(Focus::Search) {
                self.focus = None;
                self.search.focused = false;
                return (Vec::new(), true);
            }
            return (Vec::new(), false);
        }
        if self.focus == Some(Focus::Search) {
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
    fn all_eight_tabs_build_rows_and_shaders_gate_children_when_off() {
        let mut s = Settings::default();
        let mut ui = VoltSettingsUi::default();
        assert_eq!(VoltTab::ALL.len(), 8);
        for tab in VoltTab::ALL {
            ui.tab = tab;
            let (_, rows, content) = ui.layout(&s, 854.0, 480.0);
            assert!(!rows.is_empty(), "{tab:?} must expose rows");
            assert!(content > 0.0);
        }
        // Enable Shaders defaults to OFF and hides child shader controls until turned ON.
        assert!(!s.extras.volt_shaders.enabled);
        ui.tab = VoltTab::Shaders;
        let (_, off_rows, _) = ui.layout(&s, 854.0, 480.0);
        s.extras.volt_shaders.enabled = true;
        let (_, on_rows, _) = ui.layout(&s, 854.0, 480.0);
        assert!(on_rows.len() > off_rows.len() + 10);
    }

    #[test]
    fn global_search_filters_across_all_volt_tabs() {
        let s = Settings::default();
        let mut ui = VoltSettingsUi::default();
        ui.search.set_value("Resolution Scaler");
        let (_, rows, _) = ui.layout(&s, 854.0, 480.0);
        assert!(rows.iter().any(|r| {
            r.entry
                .is_some_and(|e| e.kind == VoltRowKind::Opt(Opt::VoltResolutionScale))
        }));
    }
}
