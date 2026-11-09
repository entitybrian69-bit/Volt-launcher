//! Volt 0.9.9 Built-in Pure-Rust 50-Mod Suite, Custom 8x8 Pixel-Art Logos,
//! Persistent Two-Way Sync Store (`horizon-extras.txt`), and the In-Game
//! Split-Pane `Mod Menu` UI (Image 2) with Smooth Frame-Rate-Independent Animations.

use std::time::Instant;

use crate::{
    extras::Extras,
    graphics::Graphics,
    graphics_style as style,
    hud::{HudAtlas, HudBatch},
    lang,
    options_volt_pages::VoltAct,
    settings::Settings,
    text::Text,
    ui,
};

pub const TOTAL_VOLT_MODS: usize = 49;

/// Native zoom steps use Zoomify's default 1.5x ratio and ten tiers. This
/// is independent of the animated camera FOV, which remains owned by the game.
pub fn zoomify_step_bounds(fov_pct: u16) -> (i32, i32) {
    let initial = 100.0_f32 / f32::from(fov_pct.clamp(10, 80));
    let out_steps = (initial.ln() / 1.5_f32.ln()).ceil() as i32;
    (-out_steps.clamp(0, 10), 10)
}

pub fn zoomify_factor(fov_pct: u16, steps: i32) -> f32 {
    let initial = 100.0_f32 / f32::from(fov_pct.clamp(10, 80));
    let (min, max) = zoomify_step_bounds(fov_pct);
    (initial * 1.5_f32.powi(steps.clamp(min, max))).clamp(1.0, 500.0)
}

/// Ease the native zoom's FOV scale independently from vanilla's camera
/// modifier, which clamps to 0.1 and cannot represent zooms stronger than 10x.
pub fn zoomify_ease_scale(current: f32, target: f32, seconds: f32, smooth: bool) -> f32 {
    let target = if target.is_finite() { target.clamp(0.002, 1.0) } else { 1.0 };
    let current = if current.is_finite() { current.clamp(0.002, 1.0) } else { 1.0 };
    if !smooth || current == target { return target; }
    let dt = if seconds.is_finite() { seconds.clamp(0.0, 0.25) } else { 0.0 };
    let next = current + (target - current) * (1.0 - (-8.0 * dt).exp());
    if (next - target).abs() < 0.00001 { target } else { next }
}

/// Original upstream artwork for entries whose source and redistribution terms
/// have been verified. Other entries retain the built-in fallback artwork.
/// These are native equivalents, not the original Java mods.
pub fn upstream_icon(id: VoltModId) -> Option<(&'static str, &'static [u8])> {
    match id {
        VoltModId::AppleSkin => Some(("horizon/mod-icon/appleskin", include_bytes!("../assets/mod_icons/appleskin.png"))),
        VoltModId::ArmorHud => Some(("horizon/mod-icon/armor_hud", include_bytes!("../assets/mod_icons/armor_hud.png"))),
        VoltModId::LowFire => Some(("horizon/mod-icon/low_fire", include_bytes!("../assets/mod_icons/low_fire.png"))),
        VoltModId::ModMenu => Some(("horizon/mod-icon/mod_menu", include_bytes!("../assets/mod_icons/mod_menu.png"))),
        VoltModId::ShulkerBoxTooltip => Some(("horizon/mod-icon/shulkerboxtooltip", include_bytes!("../assets/mod_icons/shulkerboxtooltip.png"))),
        VoltModId::LanguageReload => Some(("horizon/mod-icon/language_reload", include_bytes!("../assets/mod_icons/language_reload.png"))),
        VoltModId::FastQuit => Some(("horizon/mod-icon/fast_quit", include_bytes!("../assets/mod_icons/fast_quit.png"))),
        VoltModId::BetterF3 => Some(("horizon/mod-icon/betterf3", include_bytes!("../assets/mod_icons/betterf3.png"))),
        VoltModId::BetterPingDisplay => Some(("horizon/mod-icon/better_ping_display", include_bytes!("../assets/mod_icons/better_ping_display.png"))),
        VoltModId::Controlling => Some(("horizon/mod-icon/controlling", include_bytes!("../assets/mod_icons/controlling.png"))),
        VoltModId::MouseTweaks => Some(("horizon/mod-icon/mouse_tweaks", include_bytes!("../assets/mod_icons/mouse_tweaks.png"))),
        VoltModId::NoFog => Some(("horizon/mod-icon/no_fog", include_bytes!("../assets/mod_icons/no_fog.png"))),
        VoltModId::FerriteCore => Some(("horizon/mod-icon/ferritecore", include_bytes!("../assets/mod_icons/ferritecore.png"))),
        VoltModId::DynamicFps => Some(("horizon/mod-icon/dynamic_fps", include_bytes!("../assets/mod_icons/dynamic_fps.png"))),
        VoltModId::BadOptimizations => Some(("horizon/mod-icon/bad_optimizations", include_bytes!("../assets/mod_icons/bad_optimizations.png"))),
        VoltModId::ScreenshotToClipboard => Some(("horizon/mod-icon/screenshot_clipboard", include_bytes!("../assets/mod_icons/screenshot_clipboard.png"))),
        VoltModId::ChatHeads => Some(("horizon/mod-icon/chat_heads", include_bytes!("../assets/mod_icons/chat_heads.png"))),
        VoltModId::BetterClouds => Some(("horizon/mod-icon/better_clouds", include_bytes!("../assets/mod_icons/better_clouds.png"))),
        _ => None,
    }
}

/// Categories organizing all 49 native Rust mods in both the Launcher `Mods` tab
/// and the in-game `Mod Menu`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VoltModCategory {
    PvpUtility,
    PvpHud,
    Visual,
    Performance,
    Additional,
}

impl VoltModCategory {
    pub const ALL: [Self; 5] = [
        Self::PvpUtility,
        Self::PvpHud,
        Self::Visual,
        Self::Performance,
        Self::Additional,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::PvpUtility => "PvP & Utility",
            Self::PvpHud => "PvP HUD & Display",
            Self::Visual => "Visual Mods",
            Self::Performance => "Performance",
            Self::Additional => "Additional",
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::PvpUtility => "PvP & Utility",
            Self::PvpHud => "PvP HUD",
            Self::Visual => "Visual",
            Self::Performance => "Performance",
            Self::Additional => "Utility+",
        }
    }
}

/// Every single one of the 49 built-in pure-Rust mods in Volt Launcher `0.9.9`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum VoltModId {
    // --- 1. PvP & Utility (12) ---
    AppleSkin = 0,
    BetterF3 = 1,
    BetterPingDisplay = 2,
    Controlling = 3,
    MouseTweaks = 4,
    ItemScroller = 5,
    ShulkerBoxTooltip = 6,
    ChatHeads = 7,
    Freelook = 8,
    Zoomify = 9,
    DynamicCrosshair = 10,
    BetterStats = 11,
    // --- 2. PvP HUD & Display (11) ---
    ModMenu = 12,
    InventoryHudPlus = 13,
    ArmorHud = 14,
    StatusEffectTimer = 15,
    KeystrokesHud = 16,
    CpsDisplay = 17,
    PingDisplay = 18,
    FpsDisplay = 19,
    CoordinatesHud = 20,
    DurabilityDisplay = 21,
    CustomCrosshair = 22,
    // --- 3. Visual Mods (9) ---
    Fullbright = 23,
    LowFire = 24,
    NoFog = 25,
    BetterClouds = 26,
    EnhancedBlockEntities = 27,
    EntityModelFeatures = 28,
    EntityTextureFeatures = 29,
    LambDynamicLights = 30,
    BetterAnimations = 31,
    // --- 4. Performance Mods (10) ---
    Sodium = 32,
    Lithium = 33,
    FerriteCore = 34,
    ImmediatelyFast = 35,
    EntityCulling = 36,
    MoreCulling = 37,
    DynamicFps = 38,
    ModernFix = 39,
    BadOptimizations = 40,
    Krypton = 41,
    // --- 5. Additional Mods (7) ---
    ScreenshotToClipboard = 42,
    BetterMountHud = 43,
    NotEnoughAnimations = 44,
    XaerosMinimap = 45,
    XaerosWorldMap = 46,
    LanguageReload = 47,
    FastQuit = 48,
}

impl VoltModId {
    pub const ALL: [Self; TOTAL_VOLT_MODS] = [
        Self::AppleSkin,
        Self::BetterF3,
        Self::BetterPingDisplay,
        Self::Controlling,
        Self::MouseTweaks,
        Self::ItemScroller,
        Self::ShulkerBoxTooltip,
        Self::ChatHeads,
        Self::Freelook,
        Self::Zoomify,
        Self::DynamicCrosshair,
        Self::BetterStats,
        Self::ModMenu,
        Self::InventoryHudPlus,
        Self::ArmorHud,
        Self::StatusEffectTimer,
        Self::KeystrokesHud,
        Self::CpsDisplay,
        Self::PingDisplay,
        Self::FpsDisplay,
        Self::CoordinatesHud,
        Self::DurabilityDisplay,
        Self::CustomCrosshair,
        Self::Fullbright,
        Self::LowFire,
        Self::NoFog,
        Self::BetterClouds,
        Self::EnhancedBlockEntities,
        Self::EntityModelFeatures,
        Self::EntityTextureFeatures,
        Self::LambDynamicLights,
        Self::BetterAnimations,
        Self::Sodium,
        Self::Lithium,
        Self::FerriteCore,
        Self::ImmediatelyFast,
        Self::EntityCulling,
        Self::MoreCulling,
        Self::DynamicFps,
        Self::ModernFix,
        Self::BadOptimizations,
        Self::Krypton,
        Self::ScreenshotToClipboard,
        Self::BetterMountHud,
        Self::NotEnoughAnimations,
        Self::XaerosMinimap,
        Self::XaerosWorldMap,
        Self::LanguageReload,
        Self::FastQuit,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.meta().slug == slug)
    }

    pub const fn meta(self) -> VoltModMeta {
        CATALOG[self as usize]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VoltModMeta {
    pub id: VoltModId,
    pub slug: &'static str,
    pub title: &'static str,
    pub version: &'static str,
    pub category: VoltModCategory,
    pub default_on: bool,
    pub summary: &'static str,
    pub description: &'static str,
}

pub const CATALOG: [VoltModMeta; TOTAL_VOLT_MODS] = [
    // --- PvP & Utility (13) ---
    VoltModMeta {
        id: VoltModId::AppleSkin,
        slug: "appleskin",
        title: "AppleSkin",
        version: "3.0.6",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Food hunger & golden saturation preview on HUD and tooltips.",
        description: "Displays golden saturation outlines on your hunger bar, previews exact hunger and saturation restored by held food, and appends nutrition stats to food item tooltips.",
    },
    VoltModMeta {
        id: VoltModId::BetterF3,
        slug: "betterf3",
        title: "BetterF3",
        version: "11.0.3",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Color-coded, readable modular F3 debug overlay.",
        description: "Replaces the dense vanilla F3 wall of text with clean color-coded cards for FPS & Frame Pacing, XYZ & Biome, Chunk & Light, Vulkan GPU, and Active Native Mods.",
    },
    VoltModMeta {
        id: VoltModId::BetterPingDisplay,
        slug: "better_ping_display",
        title: "Better Ping Display",
        version: "1.1.2",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Exact numeric ms latency next to every player in Tab.",
        description: "Renders each player's exact millisecond latency beside their name in the Tab player list, color-coded from emerald (<60ms) to amber and coral.",
    },
    VoltModMeta {
        id: VoltModId::Controlling,
        slug: "controlling",
        title: "Controlling",
        version: "19.0.4",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Keybind search, conflict detection & unbound filtering.",
        description: "Upgrades the Key Binds screen with instant search by action or key name, live red conflict badges, and one-tap filters for Conflicting or Unbound controls.",
    },
    VoltModMeta {
        id: VoltModId::MouseTweaks,
        slug: "mouse_tweaks",
        title: "Mouse Tweaks",
        version: "2.26.1",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Smooth hold-drag item sweeping across inventory slots.",
        description: "Drag across container slots with Left/Right mouse or touch to smoothly collect matching stacks, quick-move swept rows, or place single items cleanly.",
    },
    VoltModMeta {
        id: VoltModId::ItemScroller,
        slug: "item_scroller",
        title: "Item Scroller",
        version: "0.24.0",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Scroll-wheel & swipe item transfers between containers.",
        description: "Roll the mouse wheel over any hovered inventory or chest slot to move items one-by-one (or whole stacks with Shift) between player inventory and open containers.",
    },
    VoltModMeta {
        id: VoltModId::ShulkerBoxTooltip,
        slug: "shulkerboxtooltip",
        title: "ShulkerBoxTooltip",
        version: "5.1.2",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Visual 3x9 slot grid preview inside Shulker & Bundle tooltips.",
        description: "Renders a crisp inline 3x9 container preview grid with item icons, stack counts, and capacity readout whenever hovering a Shulker Box or Bundle.",
    },
    VoltModMeta {
        id: VoltModId::ChatHeads,
        slug: "chat_heads",
        title: "Chat Heads",
        version: "0.13.7",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "8x8 player skin face icons next to every chat message.",
        description: "Draws each sender's live 8x8 skin face and hat layer directly beside their messages in the chat HUD for instant visual identification on busy servers.",
    },
    VoltModMeta {
        id: VoltModId::Freelook,
        slug: "freelook",
        title: "Freelook",
        version: "1.3.0",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Client-safe 360° orbit camera without altering movement heading.",
        description: "Hold the Freelook key (Left Alt / touch button) to orbit the camera 360° around your character while sprinting, bridging, or flying without changing server rotation packets.",
    },
    VoltModMeta {
        id: VoltModId::Zoomify,
        slug: "zoomify",
        title: "Zoomify",
        version: "2.14.2",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Silky optic zoom with scroll-wheel magnification & damping.",
        description: "Hold C (or touch Zoom) for smooth exponentially damped magnification with scroll-wheel zoom stepping (2x to 12x) and proportional camera sensitivity scaling.",
    },
    VoltModMeta {
        id: VoltModId::DynamicCrosshair,
        slug: "dynamic_crosshair",
        title: "Dynamic Crosshair",
        version: "9.3.0",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Context-reactive crosshair for melee reach, bow draw & blocks.",
        description: "Dynamically expands and tints your crosshair when aiming at an entity within melee reach, drawing a bow/crossbow, targeting an interactable block, or recharging attack cooldown.",
    },
    VoltModMeta {
        id: VoltModId::BetterStats,
        slug: "better_stats",
        title: "Better Statistics Screen",
        version: "3.13.4",
        category: VoltModCategory::PvpUtility,
        default_on: false,
        summary: "Searchable Statistics screen with non-zero filter & summary.",
        description: "Enhances the pause-menu Statistics screen with an instant live search filter, a Hide Zero Entries toggle, and clean summary totals at the top.",
    },
    // --- PvP HUD & Display (11) ---
    VoltModMeta {
        id: VoltModId::ModMenu,
        slug: "mod_menu",
        title: "Mod Menu",
        version: "11.0.3",
        category: VoltModCategory::PvpHud,
        default_on: true,
        summary: "Interactive in-game Mod Menu screen & live toggle manager.",
        description: "Provides this split-pane Mod Menu screen (accessible from Options, Volt Settings, or pressing M in-game) to search, favorite, configure, and toggle all 49 native mods live.",
    },
    VoltModMeta {
        id: VoltModId::InventoryHudPlus,
        slug: "inventory_hud_plus",
        title: "Inventory HUD+",
        version: "3.4.26",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Translucent 3x9 live main-inventory mini-grid on your HUD.",
        description: "Renders a compact frosted 3x9 mini-grid of your main inventory slots above or beside the hotbar so you can track arrows, totems, potions, and blocks mid-fight.",
    },
    VoltModMeta {
        id: VoltModId::ArmorHud,
        slug: "armor_hud",
        title: "Armor HUD",
        version: "2.5.1",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Equipped armor pieces & remaining durability bars on HUD.",
        description: "Shows your equipped Helmet, Chestplate, Leggings, and Boots on-screen with color-coded durability bars and low-armor break alerts.",
    },
    VoltModMeta {
        id: VoltModId::StatusEffectTimer,
        slug: "status_effect_timer",
        title: "Status Effect Timer",
        version: "2.0.1",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Exact MM:SS countdown timers on active potion HUD icons.",
        description: "Overlays crisp MM:SS duration timers and potency numerals on every active status effect badge, flashing amber/red during the final 10 seconds.",
    },
    VoltModMeta {
        id: VoltModId::KeystrokesHud,
        slug: "keystrokes_hud",
        title: "Keystrokes HUD",
        version: "2.3.0",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Live animated WASD, Space, LMB & RMB PvP keystrokes panel.",
        description: "Displays a crisp translucent WASD + LMB/RMB + Spacebar overlay that lights up in real time with keyboard, controller, or touch movement inputs.",
    },
    VoltModMeta {
        id: VoltModId::CpsDisplay,
        slug: "cps_display",
        title: "CPS Display",
        version: "1.4.0",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Real-time Left & Right Clicks-Per-Second HUD badge.",
        description: "Tracks rolling 1-second Left and Right click/tap timestamps and displays a clean zero-allocation LMB | RMB CPS pill on the HUD.",
    },
    VoltModMeta {
        id: VoltModId::PingDisplay,
        slug: "ping_display",
        title: "Ping Display",
        version: "1.3.1",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Dedicated live network latency (ms) HUD pill.",
        description: "Displays your current server round-trip ping in milliseconds with a color-coded signal quality dot directly on your gameplay HUD.",
    },
    VoltModMeta {
        id: VoltModId::FpsDisplay,
        slug: "fps_display",
        title: "FPS Display",
        version: "1.5.0",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Clean live FPS & frame-time stability HUD pill.",
        description: "Renders a compact frosted FPS counter pill with instant frame-rate color grading without opening the full F3 debug screen.",
    },
    VoltModMeta {
        id: VoltModId::CoordinatesHud,
        slug: "coordinates_hud",
        title: "Coordinates HUD",
        version: "1.6.2",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Compact XYZ position, cardinal facing & biome HUD bar.",
        description: "Shows your exact X / Y / Z block coordinates, N/S/E/W cardinal direction (+X/-Z axis hint), and optional current biome in a clean top-left pill.",
    },
    VoltModMeta {
        id: VoltModId::DurabilityDisplay,
        slug: "durability_display",
        title: "Durability Display",
        version: "1.5.0",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Exact numeric durability readout for held tools & offhand.",
        description: "Displays exact remaining uses (cur / max) and percentage warning colors for your held mainhand tool/weapon and offhand shield or totem.",
    },
    VoltModMeta {
        id: VoltModId::CustomCrosshair,
        slug: "custom_crosshair",
        title: "Custom Crosshair",
        version: "1.6.0",
        category: VoltModCategory::PvpHud,
        default_on: false,
        summary: "Crisp PvP crosshair presets (Dot, Plus, Ring, Chevron).",
        description: "Replaces the blurry inverted vanilla crosshair with razor-sharp pixel-aligned PvP reticles: Precision Dot, Crisp Plus, Circle Ring, or PvP Chevron.",
    },
    // --- Visual Mods (9) ---
    VoltModMeta {
        id: VoltModId::Fullbright,
        slug: "fullbright",
        title: "Fullbright",
        version: "2.1.0",
        category: VoltModCategory::Visual,
        default_on: false,
        summary: "Clean cave, underwater & night visibility boost.",
        description: "Lifts minimum ambient lightmap floors smoothly so caves, mineshafts, and night terrain remain clearly visible without harsh gamma washout.",
    },
    VoltModMeta {
        id: VoltModId::LowFire,
        slug: "low_fire",
        title: "Low Fire",
        version: "1.4.0",
        category: VoltModCategory::Visual,
        default_on: false,
        summary: "Lowers first-person burning fire & shield overlay height.",
        description: "Drops first-person flame and held shield overlays lower on the screen so burning in PvP or Nether fights never blocks your aim.",
    },
    VoltModMeta {
        id: VoltModId::NoFog,
        slug: "no_fog",
        title: "No Fog",
        version: "1.5.2",
        category: VoltModCategory::Visual,
        default_on: false,
        summary: "Removes terrain distance haze, Nether mist & water fog.",
        description: "Pushes terrain, Nether, lava, and underwater fog planes back for crystal-clear distant horizon visibility across all dimensions.",
    },
    VoltModMeta {
        id: VoltModId::BetterClouds,
        slug: "better_clouds",
        title: "Better Clouds",
        version: "1.7.6",
        category: VoltModCategory::Visual,
        default_on: false,
        summary: "Soft layered horizon clouds with dawn/dusk atmospheric tint.",
        description: "Enhances sky clouds with smoother multi-band altitude shading, extended horizon fade, and warm golden-hour sunset coloring.",
    },
    VoltModMeta {
        id: VoltModId::EnhancedBlockEntities,
        slug: "enhanced_block_entities",
        title: "Enhanced Block Entities",
        version: "0.10.2",
        category: VoltModCategory::Visual,
        default_on: false,
        summary: "Fast static Chest, Shulker, Sign & Bed rendering with active anims.",
        description: "Skips per-frame dynamic rebuilds for stationary Chests, Ender Chests, Shulker Boxes, Signs, and Beds beyond interaction range while preserving opening animations.",
    },
    VoltModMeta {
        id: VoltModId::EntityModelFeatures,
        slug: "entity_model_features",
        title: "Entity Model Features",
        version: "2.4.1",
        category: VoltModCategory::Visual,
        default_on: false,
        summary: "Native CEM (.jem/.jpm) custom entity model support.",
        description: "Enables OptiFine/CEM custom entity geometry overrides from resource packs alongside enhanced 3D mob part articulation in pure native Rust.",
    },
    VoltModMeta {
        id: VoltModId::EntityTextureFeatures,
        slug: "entity_texture_features",
        title: "Entity Texture Features",
        version: "6.2.9",
        category: VoltModCategory::Visual,
        default_on: false,
        summary: "Random mob variants, emissive _e glow textures & skin blink.",
        description: "Supports deterministic per-entity random texture variants, emissive (_e) glowing mob eyes/layers, and natural periodic player/mob eye blinking.",
    },
    VoltModMeta {
        id: VoltModId::LambDynamicLights,
        slug: "lambdynamiclights",
        title: "LambDynamicLights",
        version: "3.2.4",
        category: VoltModCategory::Visual,
        default_on: false,
        summary: "Smooth dynamic point lights for held & dropped light sources.",
        description: "Illuminates surrounding cave walls and floors in real time when holding or dropping torches, soul lanterns, glowstone, lava buckets, or fighting blazes.",
    },
    VoltModMeta {
        id: VoltModId::BetterAnimations,
        slug: "better_animations",
        title: "Better Animations Collection",
        version: "8.0.0",
        category: VoltModCategory::Visual,
        default_on: false,
        summary: "Fluid procedural mob limb, tail, tentacle & ear motion.",
        description: "Adds natural secondary motion to mobs: swaying ghast/squid tentacles, wagging wolf/cat tails, breathing idle cycles, and smoother stride transitions.",
    },
    // --- Performance Mods (10) ---
    VoltModMeta {
        id: VoltModId::Sodium,
        slug: "sodium",
        title: "Sodium",
        version: "0.6.5",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "Flagship Vulkan chunk mesher & hidden-face terrain pipeline.",
        description: "Accelerates world rendering on mobile & desktop GPUs via sealed-section interior occlusion, greedy-aligned quad emission, and zero-stall Vulkan chunk uploads.",
    },
    VoltModMeta {
        id: VoltModId::Lithium,
        slug: "lithium",
        title: "Lithium",
        version: "0.14.7",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "General-purpose physics, AI tick & block-state fast paths.",
        description: "Optimizes AABB voxel collision sweeps, throttles distant passive AI pathing, and short-circuits redundant neighbor state lookups without altering vanilla mechanics.",
    },
    VoltModMeta {
        id: VoltModId::FerriteCore,
        slug: "ferritecore",
        title: "FerriteCore",
        version: "7.1.1",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "Compact block-state tables & pooled mesh scratch memory.",
        description: "Slashes RAM usage and allocator churn on Android devices by pooling chunk mesher vertex buffers and compacting palette/model metadata in memory.",
    },
    VoltModMeta {
        id: VoltModId::ImmediatelyFast,
        slug: "immediately_fast",
        title: "ImmediatelyFast",
        version: "1.3.4",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "Coalesced HUD, text, entity & item-icon GPU draw batching.",
        description: "Batches immediate-mode UI quads, font glyphs, item icons, and entity layers into single contiguous GPU uploads to minimize Vulkan pipeline switches.",
    },
    VoltModMeta {
        id: VoltModId::EntityCulling,
        slug: "entity_culling",
        title: "EntityCulling",
        version: "1.7.2",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "Skips rendering mobs & block-entities hidden behind walls.",
        description: "Performs fast 3D visibility & depth occlusion checks to skip building vertices for entities and block-entities occluded by solid cave walls or terrain.",
    },
    VoltModMeta {
        id: VoltModId::MoreCulling,
        slug: "more_culling",
        title: "MoreCulling",
        version: "1.2.6",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "Dense leaf-shell culling, item-frame LOD & unseen face stripping.",
        description: "Strips unseen interior foliage faces, culls back-facing signs and distant item frames, and tightens block-entity draw radii in dense forests and bases.",
    },
    VoltModMeta {
        id: VoltModId::DynamicFps,
        slug: "dynamic_fps",
        title: "Dynamic FPS",
        version: "3.7.7",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "Auto-throttles FPS when idle, unfocused, or backgrounded.",
        description: "Automatically lowers frame pacing when the window/app is unfocused or untouched on static menus, saving mobile battery and keeping Helio/Mali thermals cool.",
    },
    VoltModMeta {
        id: VoltModId::ModernFix,
        slug: "modernfix",
        title: "ModernFix",
        version: "5.20.2",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "Streaming mip-major startup & non-blocking pack reload.",
        description: "Eliminates startup and world-entry stalls by streaming atlas mipmaps row-by-row, preparing animations off-thread, and freeing staging pixels immediately.",
    },
    VoltModMeta {
        id: VoltModId::BadOptimizations,
        slug: "bad_optimizations",
        title: "BadOptimizations",
        version: "2.2.1",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "Caches sky colors, lightmap uniforms & unchanged HUD strings.",
        description: "Avoids recomputing sky/fog gradients, lightmap tables, and formatted HUD strings on frames where world time, biome, and camera state have not changed.",
    },
    VoltModMeta {
        id: VoltModId::Krypton,
        slug: "krypton",
        title: "Krypton",
        version: "0.2.8",
        category: VoltModCategory::Performance,
        default_on: true,
        summary: "Optimized network packet framing, compression & socket buffers.",
        description: "Reduces multiplayer CPU overhead and packet jitter via pre-sized network decode scratch buffers and streamlined chunk/entity packet batching.",
    },
    // --- Additional Mods (7) ---
    VoltModMeta {
        id: VoltModId::ScreenshotToClipboard,
        slug: "screenshot_clipboard",
        title: "Screenshot to Clipboard",
        version: "1.1.0",
        category: VoltModCategory::Additional,
        default_on: false,
        summary: "Copies every captured F2 screenshot directly to clipboard.",
        description: "Whenever a screenshot is saved (F2 or touch capture), automatically copies the captured image path/data straight to the system clipboard with an instant toast.",
    },
    VoltModMeta {
        id: VoltModId::BetterMountHud,
        slug: "better_mount_hud",
        title: "Better Mount HUD",
        version: "1.2.4",
        category: VoltModCategory::Additional,
        default_on: false,
        summary: "Shows player Hunger + Mount Health & Jump bar together.",
        description: "Keeps your own food/hunger bar visible alongside your horse, camel, donkey, or strider's health hearts and jump-charge bar while riding.",
    },
    VoltModMeta {
        id: VoltModId::NotEnoughAnimations,
        slug: "not_enough_animations",
        title: "Not Enough Animations",
        version: "1.9.1",
        category: VoltModCategory::Additional,
        default_on: false,
        summary: "Brings eating, map-holding, rowing & climbing to 3rd-person.",
        description: "Animates third-person character arms naturally when eating/drinking food, holding maps or compasses, rowing boats, climbing ladders, and aiming.",
    },
    VoltModMeta {
        id: VoltModId::XaerosMinimap,
        slug: "xaeros_minimap",
        title: "Xaero's Minimap",
        version: "24.7.1",
        category: VoltModCategory::Additional,
        default_on: false,
        summary: "Live rotating/north-locked chunk minimap with entity radar.",
        description: "Renders a crisp corner minimap with surface terrain colors, height shading, cardinal compass markers, entity radar dots, and pinned world waypoints.",
    },
    VoltModMeta {
        id: VoltModId::XaerosWorldMap,
        slug: "xaeros_world_map",
        title: "Xaero's World Map",
        version: "1.39.2",
        category: VoltModCategory::Additional,
        default_on: false,
        summary: "Fullscreen zoomable & pannable explored-world map (M / J).",
        description: "Records explored chunks automatically and opens an interactive fullscreen World Map (press J or Open World Map) with zoom, pan, chunk grid, and waypoints.",
    },
    VoltModMeta {
        id: VoltModId::LanguageReload,
        slug: "language_reload",
        title: "Language Reload",
        version: "1.7.2",
        category: VoltModCategory::Additional,
        default_on: true,
        summary: "Instant zero-freeze language switching without atlas reload.",
        description: "Switches client languages instantaneously without rebuilding block/entity GPU texture atlases and supports quick language cycling via F3+J.",
    },
    VoltModMeta {
        id: VoltModId::FastQuit,
        slug: "fast_quit",
        title: "FastQuit",
        version: "3.0.0",
        category: VoltModCategory::Additional,
        default_on: true,
        summary: "Returns to Title Screen instantly while saving in background.",
        description: "Lets singleplayer worlds return to the Title Screen immediately upon clicking Save and Quit while world region serialization finishes safely in the background.",
    },
];

/// 3-color 8x8 pixel-art logo specification (`'.'` = bg, `'#'` = primary, `'+'` = accent).
#[derive(Clone, Copy, Debug)]
pub struct ModLogoSpec {
    pub bg: u32,
    pub primary: u32,
    pub accent: u32,
    pub rows: [&'static str; 8],
}

/// Returns a bespoke 8x8 pixel-art logo for every one of the 50 mods.
pub const fn mod_logo_spec(id: VoltModId) -> ModLogoSpec {
    match id {
        VoltModId::AppleSkin => ModLogoSpec {
            bg: 0x261015,
            primary: 0xe63946,
            accent: 0xffd166,
            rows: [
                "....++..",
                "...+##..",
                ".######.",
                "##++####",
                "#+######",
                "########",
                ".######.",
                "..####..",
            ],
        },
        VoltModId::BetterF3 => ModLogoSpec {
            bg: 0x0f1a24,
            primary: 0x38bdf8,
            accent: 0x4ade80,
            rows: [
                "####.+++",
                "#......+",
                "###...++",
                "#......+",
                "#....+++",
                "........",
                "####.+++",
                "##...++.",
            ],
        },
        VoltModId::BetterPingDisplay => ModLogoSpec {
            bg: 0x0e2019,
            primary: 0x22c55e,
            accent: 0x86efac,
            rows: [
                "......++",
                "......++",
                "....##++",
                "....##++",
                "..####++",
                "..####++",
                "######++",
                "######++",
            ],
        },
        VoltModId::Controlling => ModLogoSpec {
            bg: 0x1b1630,
            primary: 0xa78bfa,
            accent: 0xf8fafc,
            rows: [
                ".######.",
                "#+....+#",
                "#.+..+.#",
                "#.++...#",
                "#.+..+.#",
                "#+....+#",
                ".######.",
                "..####..",
            ],
        },
        VoltModId::MouseTweaks => ModLogoSpec {
            bg: 0x171d2b,
            primary: 0x94a3b8,
            accent: 0xfbbf24,
            rows: [
                "..####..",
                ".##+#+##",
                ".######.",
                ".######.",
                ".######.",
                "..####..",
                "...++...",
                "..++++..",
            ],
        },
        VoltModId::ItemScroller => ModLogoSpec {
            bg: 0x10212b,
            primary: 0x2dd4bf,
            accent: 0xfef08a,
            rows: [
                "...++...",
                "..++++..",
                "...++...",
                ".######.",
                ".######.",
                "...++...",
                "..++++..",
                "...++...",
            ],
        },
        VoltModId::ShulkerBoxTooltip => ModLogoSpec {
            bg: 0x231330,
            primary: 0xc084fc,
            accent: 0xfacc15,
            rows: [
                ".######.",
                "########",
                "###++###",
                "........",
                "###++###",
                "########",
                "########",
                ".######.",
            ],
        },
        VoltModId::ChatHeads => ModLogoSpec {
            bg: 0x1e1814,
            primary: 0xf4a261,
            accent: 0x60a5fa,
            rows: [
                ".######.",
                ".######.",
                ".+#..#+.",
                ".######.",
                ".##++##.",
                "..####..",
                ".....+++",
                "....++++",
            ],
        },
        VoltModId::Freelook => ModLogoSpec {
            bg: 0x111e2e,
            primary: 0x60a5fa,
            accent: 0xf8fafc,
            rows: [
                "..####..",
                ".#....#.",
                "#..++..#",
                "#.++++.#",
                "#..++..#",
                ".#....#.",
                "..####..",
                "........",
            ],
        },
        VoltModId::Zoomify => ModLogoSpec {
            bg: 0x1a162b,
            primary: 0x818cf8,
            accent: 0x67e8f9,
            rows: [
                ".####...",
                "#++++#..",
                "#+..+#..",
                "#+..+#..",
                "#++++#..",
                ".####+#.",
                ".....###",
                "......##",
            ],
        },
        VoltModId::DynamicCrosshair => ModLogoSpec {
            bg: 0x121822,
            primary: 0xf43f5e,
            accent: 0x2dd4bf,
            rows: [
                "...##...",
                "...##...",
                "........",
                "##.++.&&",
                "##.++.&&",
                "........",
                "...##...",
                "...##...",
            ],
        },
        VoltModId::BetterStats => ModLogoSpec {
            bg: 0x17201a,
            primary: 0x34d399,
            accent: 0xfbbf24,
            rows: [
                "......++",
                "....##++",
                "....##++",
                "..####++",
                "..####++",
                "######++",
                "########",
                "++++++++",
            ],
        },
        VoltModId::ModMenu => ModLogoSpec {
            bg: 0x0f252a,
            primary: 0x2dd4bf,
            accent: 0x99f6e4,
            rows: [
                ".##..++.",
                "####++++",
                ".##..++.",
                "........",
                ".++..##.",
                "++++####",
                ".++..##.",
                "........",
            ],
        },
        VoltModId::InventoryHudPlus => ModLogoSpec {
            bg: 0x191d28,
            primary: 0x93c5fd,
            accent: 0xfbbf24,
            rows: [
                "##.##.##",
                "##.##.##",
                "........",
                "##.##.++",
                "##.##.++",
                "........",
                "##.##.##",
                "##.##.##",
            ],
        },
        VoltModId::ArmorHud => ModLogoSpec {
            bg: 0x0e222e,
            primary: 0x38bdf8,
            accent: 0xbae6fd,
            rows: [
                "##....##",
                "###..###",
                "########",
                "##++++##",
                ".######.",
                ".######.",
                "..####..",
                "..####..",
            ],
        },
        VoltModId::StatusEffectTimer => ModLogoSpec {
            bg: 0x24132c,
            primary: 0xe879f9,
            accent: 0x67e8f9,
            rows: [
                "...++...",
                "..++++..",
                "...##...",
                "..####..",
                ".##++##.",
                ".######.",
                ".######.",
                "..####..",
            ],
        },
        VoltModId::KeystrokesHud => ModLogoSpec {
            bg: 0x161925,
            primary: 0xa5b4fc,
            accent: 0x34d399,
            rows: [
                "...++...",
                "...++...",
                "........",
                "##.##.##",
                "##.##.##",
                "........",
                ".######.",
                "........",
            ],
        },
        VoltModId::CpsDisplay => ModLogoSpec {
            bg: 0x241612,
            primary: 0xf97316,
            accent: 0xfef08a,
            rows: [
                "....++..",
                "...++...",
                "..++++..",
                ".######.",
                "...++...",
                "..++....",
                ".++.....",
                "........",
            ],
        },
        VoltModId::PingDisplay => ModLogoSpec {
            bg: 0x0d221d,
            primary: 0x10b981,
            accent: 0x6ee7b7,
            rows: [
                "..####..",
                ".#....#.",
                "#..++..#",
                "..+..+..",
                "...##...",
                "........",
                "...++...",
                "...++...",
            ],
        },
        VoltModId::FpsDisplay => ModLogoSpec {
            bg: 0x142016,
            primary: 0x4ade80,
            accent: 0xfacc15,
            rows: [
                "..####..",
                ".#....#.",
                "#....+.#",
                "#...++.#",
                "#..++..#",
                ".#....#.",
                "########",
                "........",
            ],
        },
        VoltModId::CoordinatesHud => ModLogoSpec {
            bg: 0x1b1828,
            primary: 0xf87171,
            accent: 0xe2e8f0,
            rows: [
                "...##...",
                "..####..",
                "..#++#..",
                "...++...",
                "...++...",
                "..++++..",
                ".######.",
                "........",
            ],
        },
        VoltModId::DurabilityDisplay => ModLogoSpec {
            bg: 0x1c1914,
            primary: 0x38bdf8,
            accent: 0x4ade80,
            rows: [
                ".#####..",
                "##...#..",
                "#..#....",
                "...##...",
                "....##..",
                ".....#..",
                "++++++++",
                "######..",
            ],
        },
        VoltModId::CustomCrosshair => ModLogoSpec {
            bg: 0x111827,
            primary: 0x22d3ee,
            accent: 0xffffff,
            rows: [
                "...##...",
                "...##...",
                "..####..",
                "###++###",
                "###++###",
                "..####..",
                "...##...",
                "...##...",
            ],
        },
        VoltModId::Fullbright => ModLogoSpec {
            bg: 0x261f0d,
            primary: 0xfacc15,
            accent: 0xfef9c3,
            rows: [
                "#..##..#",
                ".######.",
                "##++++##",
                "#+####+#",
                "#+####+#",
                "##++++##",
                ".######.",
                "#..##..#",
            ],
        },
        VoltModId::LowFire => ModLogoSpec {
            bg: 0x26120e,
            primary: 0xf97316,
            accent: 0xfde047,
            rows: [
                "........",
                "........",
                "........",
                "..#..#..",
                ".######.",
                "##++++##",
                "#+####+#",
                ".######.",
            ],
        },
        VoltModId::NoFog => ModLogoSpec {
            bg: 0x102030,
            primary: 0x60a5fa,
            accent: 0xf8fafc,
            rows: [
                ".....++.",
                "....++++",
                "..#.....",
                ".###..#.",
                "#####.##",
                "########",
                "########",
                "........",
            ],
        },
        VoltModId::BetterClouds => ModLogoSpec {
            bg: 0x122338,
            primary: 0xe2e8f0,
            accent: 0x93c5fd,
            rows: [
                "........",
                "...###..",
                "..#####.",
                ".#######",
                "########",
                "++++++++",
                ".++++++.",
                "........",
            ],
        },
        VoltModId::EnhancedBlockEntities => ModLogoSpec {
            bg: 0x24190e,
            primary: 0xd97706,
            accent: 0xfacc15,
            rows: [
                ".######.",
                "########",
                "###++###",
                "+++##+++",
                "###++###",
                "########",
                "########",
                ".######.",
            ],
        },
        VoltModId::EntityModelFeatures => ModLogoSpec {
            bg: 0x112416,
            primary: 0x22c55e,
            accent: 0x052e16,
            rows: [
                "########",
                "#.+..+.#",
                "#.+..+.#",
                "#..++..#",
                "#.++++.#",
                "#.++++.#",
                "#.+..+.#",
                "########",
            ],
        },
        VoltModId::EntityTextureFeatures => ModLogoSpec {
            bg: 0x1b1026,
            primary: 0xa855f7,
            accent: 0xf0abfc,
            rows: [
                "........",
                ".######.",
                "########",
                "+++##+++",
                "+++##+++",
                "########",
                ".######.",
                "........",
            ],
        },
        VoltModId::LambDynamicLights => ModLogoSpec {
            bg: 0x261c0e,
            primary: 0xf59e0b,
            accent: 0xfef3c7,
            rows: [
                "...##...",
                "..####..",
                ".##++##.",
                ".#++++#.",
                ".#++++#.",
                ".##++##.",
                "..####..",
                "..####..",
            ],
        },
        VoltModId::BetterAnimations => ModLogoSpec {
            bg: 0x122028,
            primary: 0x38bdf8,
            accent: 0xf472b6,
            rows: [
                "...++...",
                "...++...",
                ".######.",
                "..####..",
                "..#..#..",
                ".#....#.",
                "#......#",
                "........",
            ],
        },
        VoltModId::Sodium => ModLogoSpec {
            bg: 0x0b261c,
            primary: 0x10b981,
            accent: 0xa7f3d0,
            rows: [
                ".######.",
                "########",
                "#+..+#.#",
                "#++.##.#",
                "#.#++#.#",
                "#.#..#+#",
                "########",
                ".######.",
            ],
        },
        VoltModId::Lithium => ModLogoSpec {
            bg: 0x221429,
            primary: 0xc084fc,
            accent: 0xfbbf24,
            rows: [
                "..####..",
                ".#....#.",
                "#..++..#",
                "#..++..#",
                ".#....#.",
                "..####..",
                "#......#",
                "........",
            ],
        },
        VoltModId::FerriteCore => ModLogoSpec {
            bg: 0x241512,
            primary: 0xef4444,
            accent: 0xfca5a5,
            rows: [
                "...##...",
                "..####..",
                ".##++##.",
                "##++++##",
                "##++++##",
                ".##++##.",
                "..####..",
                "...##...",
            ],
        },
        VoltModId::ImmediatelyFast => ModLogoSpec {
            bg: 0x0f1d2e,
            primary: 0x38bdf8,
            accent: 0x7dd3fc,
            rows: [
                "#...#...",
                "##..##..",
                "###.###+",
                "########",
                "###.###+",
                "##..##..",
                "#...#...",
                "........",
            ],
        },
        VoltModId::EntityCulling => ModLogoSpec {
            bg: 0x141d26,
            primary: 0x64748b,
            accent: 0x2dd4bf,
            rows: [
                "####....",
                "####..++",
                "####.+++",
                "####..++",
                "####....",
                "####.+++",
                "####.+++",
                "####....",
            ],
        },
        VoltModId::MoreCulling => ModLogoSpec {
            bg: 0x112318,
            primary: 0x22c55e,
            accent: 0x86efac,
            rows: [
                "..++++..",
                ".++..++.",
                "+..##..+",
                "+.####.+",
                "+.####.+",
                "+..##..+",
                ".++..++.",
                "..++++..",
            ],
        },
        VoltModId::DynamicFps => ModLogoSpec {
            bg: 0x13202a,
            primary: 0x34d399,
            accent: 0x93c5fd,
            rows: [
                "..####..",
                ".######.",
                ".#++++#.",
                ".#+..+#.",
                ".#++++#.",
                ".#++++#.",
                ".######.",
                "........",
            ],
        },
        VoltModId::ModernFix => ModLogoSpec {
            bg: 0x231a12,
            primary: 0xf59e0b,
            accent: 0xe2e8f0,
            rows: [
                "..+++...",
                ".++..+..",
                ".++.....",
                "..###...",
                "...###..",
                "....###.",
                ".....###",
                "......##",
            ],
        },
        VoltModId::BadOptimizations => ModLogoSpec {
            bg: 0x162419,
            primary: 0x4ade80,
            accent: 0xffffff,
            rows: [
                "........",
                ".......+",
                "......++",
                "#....##.",
                "##..##..",
                ".####...",
                "..##....",
                "........",
            ],
        },
        VoltModId::Krypton => ModLogoSpec {
            bg: 0x0d2428,
            primary: 0x06b6d4,
            accent: 0xa5f3fc,
            rows: [
                ".######.",
                "#+....+#",
                "#..++..#",
                "#.++++.#",
                "#.++++.#",
                "#..++..#",
                "#+....+#",
                ".######.",
            ],
        },
        VoltModId::ScreenshotToClipboard => ModLogoSpec {
            bg: 0x191d26,
            primary: 0x94a3b8,
            accent: 0x38bdf8,
            rows: [
                "..####..",
                "########",
                "##++++##",
                "#+####+#",
                "#+####+#",
                "##++++##",
                "########",
                "........",
            ],
        },
        VoltModId::BetterMountHud => ModLogoSpec {
            bg: 0x241811,
            primary: 0xd97706,
            accent: 0xf43f5e,
            rows: [
                ".##.....",
                "####..++",
                "#####+++",
                ".####.++",
                ".######.",
                ".##..##.",
                ".#....#.",
                "........",
            ],
        },
        VoltModId::NotEnoughAnimations => ModLogoSpec {
            bg: 0x1e162b,
            primary: 0xc084fc,
            accent: 0xfbbf24,
            rows: [
                "...++...",
                "...++...",
                "..####++",
                ".#.###..",
                "...##...",
                "..#..#..",
                ".#....#.",
                "........",
            ],
        },
        VoltModId::XaerosMinimap => ModLogoSpec {
            bg: 0x0e2229,
            primary: 0x2dd4bf,
            accent: 0xf87171,
            rows: [
                "..++++..",
                ".######.",
                "###++###",
                "###++###",
                "########",
                "####.###",
                ".######.",
                "..####..",
            ],
        },
        VoltModId::XaerosWorldMap => ModLogoSpec {
            bg: 0x14261c,
            primary: 0x4ade80,
            accent: 0x60a5fa,
            rows: [
                "##.++.&&",
                "##.++.&&",
                "###++###",
                "++++###+",
                "+++####+",
                "##++++##",
                "##.++.##",
                "........",
            ],
        },
        VoltModId::LanguageReload => ModLogoSpec {
            bg: 0x121d30,
            primary: 0x60a5fa,
            accent: 0x34d399,
            rows: [
                "..####..",
                ".#.##.#.",
                "########",
                "#.####.#",
                "########",
                ".#.##+++",
                "..##++++",
                ".....++.",
            ],
        },
        VoltModId::FastQuit => ModLogoSpec {
            bg: 0x261518,
            primary: 0xfb7185,
            accent: 0xfef08a,
            rows: [
                ".#####..",
                ".#...#..",
                ".#...+++",
                ".#..++++",
                ".#...+++",
                ".#...#..",
                ".#####..",
                "........",
            ],
        },
    }
}

const fn rgb_to_hud(rgb: u32, a: u8) -> u32 {
    (a as u32) << 24 | (rgb & 0xff) << 16 | ((rgb >> 8) & 0xff) << 8 | ((rgb >> 16) & 0xff)
}

/// Renders a mod's 8x8 pixel-art logo into an in-game `HudBatch` at `(x, y)` of square `size`.
pub fn draw_mod_logo_hud(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    id: VoltModId,
    x: f32,
    y: f32,
    size: f32,
) {
    if let Some((name, _)) = upstream_icon(id)
        && atlas.sprites.contains_key(name)
    {
        batch.sprite_scaled(atlas, name, x, y, size, size, u32::MAX);
        return;
    }
    let spec = mod_logo_spec(id);
    let bg = rgb_to_hud(spec.bg, 255);
    let pri = rgb_to_hud(spec.primary, 255);
    let acc = rgb_to_hud(spec.accent, 255);
    batch.fill(atlas, x, y, size, size, bg);
    let cell = (size / 8.0).max(1.0);
    for (ry, row) in spec.rows.iter().enumerate() {
        for (rx, ch) in row.bytes().enumerate().take(8) {
            let col = match ch {
                b'#' => pri,
                b'+' | b'&' => acc,
                _ => continue,
            };
            let px = (x + rx as f32 * cell).floor();
            let py = (y + ry as f32 * cell).floor();
            let pw = ((x + (rx + 1) as f32 * cell).floor() - px).max(1.0);
            let ph = ((y + (ry + 1) as f32 * cell).floor() - py).max(1.0);
            batch.fill(atlas, px, py, pw, ph, col);
        }
    }
    style::ring(batch, atlas, [x, y, size, size], rgb_to_hud(spec.primary, 96));
}

/// Persistent state for all 50 built-in native Rust mods, stored in `horizon-extras.txt`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoltModCatalogState {
    pub enabled: [bool; TOTAL_VOLT_MODS],
    pub favorite: [bool; TOTAL_VOLT_MODS],
    pub betterf3_compact: bool,
    pub inventory_hud_opacity: u8,
    pub coords_show_biome: bool,
    pub dynamic_fps_idle_cap: u8,
    pub better_clouds_density: u8,
    pub mouse_tweaks_rmb: bool,
    pub item_scroller_wheel: bool,
    /// Controls the native saturation bar and held-food preview independently.
    pub appleskin_saturation: bool,
    pub appleskin_food_preview: bool,
    pub appleskin_tooltip: bool,
    pub appleskin_tooltip_always: bool,
    pub appleskin_offhand: bool,
    pub appleskin_exhaustion: bool,
    pub appleskin_debug: bool,
    pub better_ping_auto_color: bool,
    pub better_ping_show_bars: bool,
    pub freelook_toggle: bool,
    pub zoomify_toggle: bool,
    pub zoomify_scroll: bool,
    pub zoomify_retain_steps: bool,
    pub zoomify_relative_sensitivity: bool,
}

impl Default for VoltModCatalogState {
    fn default() -> Self {
        let mut enabled = [false; TOTAL_VOLT_MODS];
        let mut i = 0;
        while i < TOTAL_VOLT_MODS {
            enabled[i] = CATALOG[i].default_on;
            i += 1;
        }
        Self {
            enabled,
            favorite: [false; TOTAL_VOLT_MODS],
            betterf3_compact: true,
            inventory_hud_opacity: 80,
            coords_show_biome: true,
            dynamic_fps_idle_cap: 15,
            better_clouds_density: 100,
            mouse_tweaks_rmb: true,
            item_scroller_wheel: true,
            appleskin_saturation: true,
            appleskin_food_preview: true,
            appleskin_tooltip: true,
            appleskin_tooltip_always: true,
            appleskin_offhand: true,
            appleskin_exhaustion: true,
            appleskin_debug: true,
            better_ping_auto_color: true,
            better_ping_show_bars: false,
            freelook_toggle: false,
            zoomify_toggle: false,
            zoomify_scroll: true,
            zoomify_retain_steps: false,
            zoomify_relative_sensitivity: true,
        }
    }
}

impl VoltModCatalogState {
    pub fn is_on(&self, id: VoltModId) -> bool {
        self.enabled[id.index()]
    }

    pub fn set(&mut self, id: VoltModId, on: bool) {
        self.enabled[id.index()] = on;
    }

    pub fn toggle(&mut self, id: VoltModId) -> bool {
        let idx = id.index();
        self.enabled[idx] = !self.enabled[idx];
        self.enabled[idx]
    }

    pub fn is_fav(&self, id: VoltModId) -> bool {
        self.favorite[id.index()]
    }

    pub fn toggle_fav(&mut self, id: VoltModId) -> bool {
        let idx = id.index();
        self.favorite[idx] = !self.favorite[idx];
        self.favorite[idx]
    }

    pub fn enabled_count(&self) -> usize {
        self.enabled.iter().filter(|&&b| b).count()
    }

    pub fn fav_count(&self) -> usize {
        self.favorite.iter().filter(|&&b| b).count()
    }

    pub fn parse_entry(&mut self, key: &str, val: &str) -> bool {
        if let Some(slug) = key.strip_prefix("voltMod.")
            && let Some(id) = VoltModId::from_slug(slug)
        {
            self.enabled[id.index()] = val == "true";
            return true;
        }
        if let Some(slug) = key.strip_prefix("voltModFav.")
            && let Some(id) = VoltModId::from_slug(slug)
        {
            self.favorite[id.index()] = val == "true";
            return true;
        }
        match key {
            "voltModCfg.zoomifyToggle" => { self.zoomify_toggle = val == "true"; true }
            "voltModCfg.zoomifyScroll" => { self.zoomify_scroll = val == "true"; true }
            "voltModCfg.zoomifyRetainSteps" => { self.zoomify_retain_steps = val == "true"; true }
            "voltModCfg.zoomifyRelativeSensitivity" => { self.zoomify_relative_sensitivity = val == "true"; true }
            "voltModCfg.appleSkinTooltip" => { self.appleskin_tooltip = val == "true"; true }
            "voltModCfg.appleSkinTooltipAlways" => { self.appleskin_tooltip_always = val == "true"; true }
            "voltModCfg.appleSkinOffhand" => { self.appleskin_offhand = val == "true"; true }
            "voltModCfg.appleSkinExhaustion" => { self.appleskin_exhaustion = val == "true"; true }
            "voltModCfg.appleSkinDebug" => { self.appleskin_debug = val == "true"; true }
            "voltModCfg.freelookToggle" => {
                self.freelook_toggle = val == "true";
                true
            }
            "voltModCfg.betterPingAutoColor" => {
                self.better_ping_auto_color = val == "true";
                true
            }
            "voltModCfg.betterPingShowBars" => {
                self.better_ping_show_bars = val == "true";
                true
            }
            "voltModCfg.appleSkinSaturation" => {
                self.appleskin_saturation = val == "true";
                true
            }
            "voltModCfg.appleSkinFoodPreview" => {
                self.appleskin_food_preview = val == "true";
                true
            }
            "voltModCfg.betterf3Compact" => {
                self.betterf3_compact = val == "true";
                true
            }
            "voltModCfg.invHudOpacity" => {
                if let Ok(v) = val.parse::<u8>() {
                    self.inventory_hud_opacity = v.clamp(25, 100);
                }
                true
            }
            "voltModCfg.coordsBiome" => {
                self.coords_show_biome = val == "true";
                true
            }
            "voltModCfg.dynFpsIdle" => {
                if let Ok(v) = val.parse::<u8>() {
                    self.dynamic_fps_idle_cap = v.clamp(10, 60);
                }
                true
            }
            "voltModCfg.cloudsDensity" => {
                if let Ok(v) = val.parse::<u8>() {
                    self.better_clouds_density = v.clamp(50, 150);
                }
                true
            }
            "voltModCfg.mouseTweaksRmb" => {
                self.mouse_tweaks_rmb = val == "true";
                true
            }
            "voltModCfg.itemScrollerWheel" => {
                self.item_scroller_wheel = val == "true";
                true
            }
            _ => false,
        }
    }

    pub fn serialize_into(&self, mut push: impl FnMut(&str, &str)) {
        let bool_s = |b: bool| if b { "true" } else { "false" };
        for id in VoltModId::ALL {
            let meta = id.meta();
            if self.enabled[id.index()] != meta.default_on {
                let k = format!("voltMod.{}", meta.slug);
                push(&k, bool_s(self.enabled[id.index()]));
            }
            if self.favorite[id.index()] {
                let k = format!("voltModFav.{}", meta.slug);
                push(&k, "true");
            }
        }
        push("voltModCfg.zoomifyToggle", bool_s(self.zoomify_toggle));
        push("voltModCfg.zoomifyScroll", bool_s(self.zoomify_scroll));
        push("voltModCfg.zoomifyRetainSteps", bool_s(self.zoomify_retain_steps));
        push("voltModCfg.zoomifyRelativeSensitivity", bool_s(self.zoomify_relative_sensitivity));
        push("voltModCfg.appleSkinTooltip", bool_s(self.appleskin_tooltip));
        push("voltModCfg.appleSkinTooltipAlways", bool_s(self.appleskin_tooltip_always));
        push("voltModCfg.appleSkinOffhand", bool_s(self.appleskin_offhand));
        push("voltModCfg.appleSkinExhaustion", bool_s(self.appleskin_exhaustion));
        push("voltModCfg.appleSkinDebug", bool_s(self.appleskin_debug));
        push("voltModCfg.freelookToggle", bool_s(self.freelook_toggle));
        push("voltModCfg.betterPingAutoColor", bool_s(self.better_ping_auto_color));
        push("voltModCfg.betterPingShowBars", bool_s(self.better_ping_show_bars));
        push("voltModCfg.appleSkinSaturation", bool_s(self.appleskin_saturation));
        push("voltModCfg.appleSkinFoodPreview", bool_s(self.appleskin_food_preview));
        push("voltModCfg.betterf3Compact", bool_s(self.betterf3_compact));
        push(
            "voltModCfg.invHudOpacity",
            &self.inventory_hud_opacity.to_string(),
        );
        push("voltModCfg.coordsBiome", bool_s(self.coords_show_biome));
        push(
            "voltModCfg.dynFpsIdle",
            &self.dynamic_fps_idle_cap.to_string(),
        );
        push(
            "voltModCfg.cloudsDensity",
            &self.better_clouds_density.to_string(),
        );
        push("voltModCfg.mouseTweaksRmb", bool_s(self.mouse_tweaks_rmb));
        push(
            "voltModCfg.itemScrollerWheel",
            bool_s(self.item_scroller_wheel),
        );
    }
}

/// Keeps a toggled mod's state synchronized with corresponding `Extras` & `Graphics` engine flags
/// so `Volt Settings`, `Video Settings`, `Mod Menu`, and the Launcher `Mods` tab stay 100% unified.
pub fn apply_mod_to_engine(id: VoltModId, on: bool, extras: &mut Extras, horizon: &mut Graphics) {
    extras.mod_catalog.set(id, on);
    match id {
        VoltModId::AppleSkin => extras.volt_mods.appleskin_hud = on,
        VoltModId::MouseTweaks => extras.mouse_tweaks = on,
        // Item Scroller has its own catalog switch and wheel option. Do not
        // overwrite Mouse Tweaks' independently configured wheel behavior.
        VoltModId::ItemScroller => {},
        VoltModId::ShulkerBoxTooltip => extras.volt_mods.shulker_preview = on,
        VoltModId::Freelook => extras.volt_qol.freelook_enabled = on,
        VoltModId::Zoomify => {
            extras.volt_qol.zoom_enabled = on;
            extras.zoom = on;
        }
        VoltModId::ArmorHud => extras.volt_mods.armor_hud = on,
        VoltModId::StatusEffectTimer => extras.volt_mods.potion_hud = on,
        VoltModId::KeystrokesHud => {
            extras.volt_mods.keystrokes_hud = on;
            extras.keystrokes = on;
        }
        VoltModId::CpsDisplay => extras.volt_mods.cps_hud = on,
        VoltModId::FpsDisplay => {
            extras.volt_mods.fps_ping_pill = on;
            extras.fps = on;
        }
        VoltModId::CustomCrosshair => {
            extras.volt_mods.custom_crosshair = if on {
                crate::volt_qol::CrosshairStyle::CrispPlus
            } else {
                crate::volt_qol::CrosshairStyle::Vanilla
            };
        }
        VoltModId::Fullbright => extras.fullbright = on,
        VoltModId::LowFire => extras.volt_mods.low_shield_fire = on,
        VoltModId::NoFog => horizon.fog = !on,
        VoltModId::LambDynamicLights => extras.dynamic_lights = on,
        VoltModId::EntityCulling => horizon.entity_culling = on,
        VoltModId::Sodium => {
            horizon.cave_culling = on;
            extras.volt_boost_plus.cave_chunk_occlusion = on;
        }
        VoltModId::FerriteCore => extras.volt_boost_plus.zero_stutter_mesh_pool = on,
        VoltModId::MoreCulling => extras.cull_leaves = on,
        VoltModId::XaerosMinimap => extras.minimap = on,
        VoltModId::XaerosWorldMap => extras.volt_qol.atlas_enabled = on,
        _ => {}
    }
}

/// Pulls any direct edits made in `Volt Settings` or `Video Settings` back into `mod_catalog`.
pub fn sync_catalog_from_engine(extras: &mut Extras, horizon: &Graphics) {
    extras
        .mod_catalog
        .set(VoltModId::AppleSkin, extras.volt_mods.appleskin_hud);
    extras.mod_catalog.set(VoltModId::MouseTweaks, extras.mouse_tweaks);
    extras
        .mod_catalog
        .set(VoltModId::ShulkerBoxTooltip, extras.volt_mods.shulker_preview);
    extras
        .mod_catalog
        .set(VoltModId::Freelook, extras.volt_qol.freelook_enabled);
    extras.mod_catalog.set(
        VoltModId::Zoomify,
        extras.volt_qol.zoom_enabled || extras.zoom,
    );
    extras
        .mod_catalog
        .set(VoltModId::ArmorHud, extras.volt_mods.armor_hud);
    extras
        .mod_catalog
        .set(VoltModId::StatusEffectTimer, extras.volt_mods.potion_hud);
    extras.mod_catalog.set(
        VoltModId::KeystrokesHud,
        extras.volt_mods.keystrokes_hud || extras.keystrokes,
    );
    extras
        .mod_catalog
        .set(VoltModId::CpsDisplay, extras.volt_mods.cps_hud);
    extras.mod_catalog.set(
        VoltModId::FpsDisplay,
        extras.volt_mods.fps_ping_pill || extras.fps,
    );
    extras.mod_catalog.set(
        VoltModId::CustomCrosshair,
        extras.volt_mods.custom_crosshair != crate::volt_qol::CrosshairStyle::Vanilla,
    );
    extras
        .mod_catalog
        .set(VoltModId::Fullbright, extras.fullbright);
    extras
        .mod_catalog
        .set(VoltModId::LowFire, extras.volt_mods.low_shield_fire);
    extras.mod_catalog.set(VoltModId::NoFog, !horizon.fog);
    extras
        .mod_catalog
        .set(VoltModId::LambDynamicLights, extras.dynamic_lights);
    extras
        .mod_catalog
        .set(VoltModId::XaerosMinimap, extras.minimap);
    extras.mod_catalog.set(VoltModId::XaerosWorldMap, extras.volt_qol.atlas_enabled);
    extras.mod_catalog.set(VoltModId::EntityCulling, horizon.entity_culling);
    extras.mod_catalog.set(VoltModId::MoreCulling, extras.cull_leaves);
    extras.mod_catalog.set(VoltModId::FerriteCore, extras.volt_boost_plus.zero_stutter_mesh_pool);
}

// ============================================================================
// IMAGE 2 IN-GAME MOD MENU UI (`ModMenuUi`) + SMOOTH ANIMATIONS
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ModFilterTab {
    #[default]
    All,
    On,
    Off,
    Fav,
}

impl ModFilterTab {
    pub const ALL: [Self; 4] = [Self::All, Self::On, Self::Off, Self::Fav];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModMenuAction {
    ButtonClick,
    Changed,
    OpenFolder,
    OpenWorldMap,
    OpenKeyBinds,
    Done,
}

pub struct ModMenuUi {
    pub search: ui::EditBox,
    pub search_focused: bool,
    pub filter: ModFilterTab,
    pub selected: VoltModId,
    pub scroll: ui::Scroll,
    cfg_scroll: f32,
    cfg_drag_y: Option<f32>,
    pub preset_index: u8,
    // Smooth animation state
    last_frame: Option<Instant>,
    sel_anim_y: f32,
    toggle_anim: f32,
    open_anim: f32,
}

impl Default for ModMenuUi {
    fn default() -> Self {
        let mut search = ui::EditBox::new(0.0, 0.0, 120.0, 8.0);
        search.bordered = false;
        search.focused = false;
        Self {
            search,
            search_focused: false,
            filter: ModFilterTab::All,
            selected: VoltModId::ModMenu,
            scroll: ui::Scroll::default(),
            cfg_scroll: 0.0,
            cfg_drag_y: None,
            preset_index: 0,
            last_frame: None,
            sel_anim_y: 0.0,
            toggle_anim: 1.0,
            open_anim: 0.0,
        }
    }
}

struct MenuLayout {
    modal: [f32; 4],
    gear_btn: [f32; 4],
    search_box: [f32; 4],
    search_clear: [f32; 4],
    tabs: [[f32; 4]; 4],
    list_x: f32,
    list_w: f32,
    list_top: f32,
    list_bottom: f32,
    folder_btn: [f32; 4],
    detail_card: [f32; 4],
    toggle_bar: [f32; 4],
    fav_btn: [f32; 4],
    cfg_top: f32,
    cfg_bottom: f32,
    cfg_content_h: f32,
    cfg_rows: Vec<([f32; 4], ModCfgKind)>,
    done_btn: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModCfgKind {
    VoltAct(VoltAct),
    BetterF3Compact,
    AppleSkinSaturation,
    AppleSkinFoodPreview,
    AppleSkinTooltip,
    AppleSkinTooltipAlways,
    AppleSkinOffhand,
    AppleSkinExhaustion,
    AppleSkinDebug,
    BetterPingAutoColor,
    BetterPingShowBars,
    FreelookMode,
    ZoomifyMode,
    ZoomifyScroll,
    ZoomifyRetainSteps,
    ZoomifySensitivity,
    InvHudOpacity,
    CoordsBiome,
    DynFpsIdle,
    CloudsDensity,
    MouseTweaksRmb,
    ItemScrollerWheel,
    OpenWorldMap,
    OpenKeyBinds,
}

const ROW_H: f32 = 28.0;
const ROW_GAP: f32 = 3.0;

// Image 2 exact dark slate + teal/mint/coral palette
const MM_VEIL: u32 = rgb_to_hud(0x06080d, 150);
const MM_PLATE: u32 = rgb_to_hud(0x0d121a, 236);
const MM_EDGE: u32 = rgb_to_hud(0x253244, 220);
const MM_INNER: u32 = rgb_to_hud(0x121924, 235);
const MM_INNER_HOVER: u32 = rgb_to_hud(0x192332, 245);
const MM_SEL_FILL: u32 = rgb_to_hud(0x112d35, 245);
const MM_TEAL: u32 = rgb_to_hud(0x2dd4bf, 255);
const MM_TEAL_BTN: u32 = rgb_to_hud(0x179b88, 255);
const MM_TEAL_HOVER: u32 = rgb_to_hud(0x20b8a2, 255);
const MM_MINT: u32 = rgb_to_hud(0x4ade80, 255);
const MM_MINT_BG: u32 = rgb_to_hud(0x113628, 235);
const MM_CORAL: u32 = rgb_to_hud(0xf07a68, 255);
const MM_CORAL_BG: u32 = rgb_to_hud(0x3a181b, 235);
const MM_BADGE_MOD: u32 = rgb_to_hud(0x113740, 240);
const MM_BADGE_SLUG: u32 = rgb_to_hud(0x1b2433, 240);
const MM_BADGE_CAT: u32 = rgb_to_hud(0x241c42, 240);
const MM_MUTED: u32 = rgb_to_hud(0x8a99b0, 255);
const MM_SOFT: u32 = rgb_to_hud(0xc5d0e0, 255);

impl ModMenuUi {
    pub fn open(&mut self, s: &mut Settings) {
        sync_catalog_from_engine(&mut s.extras, &s.horizon);
        self.search.set_value("");
        self.search_focused = false;
        self.cfg_scroll = 0.0;
        self.cfg_drag_y = None;
        self.last_frame = None;
        self.open_anim = 0.0;
        self.toggle_anim = if s.extras.mod_catalog.is_on(self.selected) {
            1.0
        } else {
            0.0
        };
    }

    pub fn typing(&self) -> bool {
        self.search_focused
    }

    fn filtered_mods(&self, s: &Settings) -> Vec<VoltModId> {
        let q = self.search.value.trim().to_lowercase();
        VoltModId::ALL
            .into_iter()
            .filter(|&id| {
                let on = s.extras.mod_catalog.is_on(id);
                let fav = s.extras.mod_catalog.is_fav(id);
                let tab_ok = match self.filter {
                    ModFilterTab::All => true,
                    ModFilterTab::On => on,
                    ModFilterTab::Off => !on,
                    ModFilterTab::Fav => fav,
                };
                if !tab_ok {
                    return false;
                }
                if q.is_empty() {
                    return true;
                }
                let m = id.meta();
                m.title.to_lowercase().contains(&q)
                    || m.slug.contains(&q)
                    || m.category.label().to_lowercase().contains(&q)
                    || m.summary.to_lowercase().contains(&q)
            })
            .collect()
    }

    fn mod_config_items(id: VoltModId) -> Vec<ModCfgKind> {
        match id {
            VoltModId::AppleSkin => vec![ModCfgKind::AppleSkinSaturation, ModCfgKind::AppleSkinFoodPreview, ModCfgKind::AppleSkinOffhand, ModCfgKind::AppleSkinTooltip, ModCfgKind::AppleSkinTooltipAlways, ModCfgKind::AppleSkinExhaustion, ModCfgKind::AppleSkinDebug],
            VoltModId::BetterPingDisplay => vec![ModCfgKind::BetterPingAutoColor, ModCfgKind::BetterPingShowBars],
            VoltModId::Zoomify => vec![
                ModCfgKind::OpenKeyBinds,
                ModCfgKind::VoltAct(VoltAct::QolZoomFov),
                ModCfgKind::VoltAct(VoltAct::QolZoomSmooth),
                ModCfgKind::ZoomifyMode,
                ModCfgKind::ZoomifyScroll,
                ModCfgKind::ZoomifyRetainSteps,
                ModCfgKind::ZoomifySensitivity,
            ],
            VoltModId::Freelook => vec![ModCfgKind::OpenKeyBinds, ModCfgKind::FreelookMode],
            VoltModId::ModMenu => vec![ModCfgKind::OpenKeyBinds],
            VoltModId::CustomCrosshair => vec![ModCfgKind::VoltAct(VoltAct::ModCrosshair)],
            VoltModId::BetterF3 => vec![ModCfgKind::BetterF3Compact],
            VoltModId::InventoryHudPlus => vec![ModCfgKind::InvHudOpacity],
            VoltModId::CoordinatesHud => vec![ModCfgKind::CoordsBiome],
            VoltModId::DynamicFps => vec![ModCfgKind::DynFpsIdle],
            VoltModId::BetterClouds => vec![ModCfgKind::CloudsDensity],
            VoltModId::MouseTweaks => vec![ModCfgKind::MouseTweaksRmb],
            VoltModId::ItemScroller => vec![ModCfgKind::ItemScrollerWheel],
            VoltModId::XaerosWorldMap => vec![
                ModCfgKind::OpenWorldMap,
                ModCfgKind::OpenKeyBinds,
                ModCfgKind::VoltAct(VoltAct::QolAtlasMode),
                ModCfgKind::VoltAct(VoltAct::QolAtlasGrid),
            ],
            VoltModId::Controlling => vec![ModCfgKind::OpenKeyBinds],
            _ => Vec::new(),
        }
    }

    fn cfg_label(kind: ModCfgKind, s: &Settings) -> String {
        let cat = &s.extras.mod_catalog;
        match kind {
            ModCfgKind::VoltAct(act) => format!("{}: {}", act.caption(), act.value(&s.extras)),
            ModCfgKind::AppleSkinSaturation => format!("Show saturation overlay: {}", if cat.appleskin_saturation { "ON" } else { "OFF" }),
            ModCfgKind::AppleSkinFoodPreview => format!("Show held food preview: {}", if cat.appleskin_food_preview { "ON" } else { "OFF" }),
            ModCfgKind::AppleSkinTooltip => format!("Food values in tooltip: {}", if cat.appleskin_tooltip { "ON" } else { "OFF" }),
            ModCfgKind::AppleSkinTooltipAlways => format!("Always show food values: {}", if cat.appleskin_tooltip_always { "ON" } else { "OFF" }),
            ModCfgKind::AppleSkinOffhand => format!("Offhand food preview: {}", if cat.appleskin_offhand { "ON" } else { "OFF" }),
            ModCfgKind::AppleSkinExhaustion => format!("Show exhaustion (local): {}", if cat.appleskin_exhaustion { "ON" } else { "OFF" }),
            ModCfgKind::AppleSkinDebug => format!("Food info in debug: {}", if cat.appleskin_debug { "ON" } else { "OFF" }),
            ModCfgKind::BetterPingAutoColor => format!("Auto-color Ping Text: {}", if cat.better_ping_auto_color { "ON" } else { "OFF" }),
            ModCfgKind::BetterPingShowBars => format!("Render Ping Bars: {}", if cat.better_ping_show_bars { "ON" } else { "OFF" }),
            ModCfgKind::FreelookMode => format!("Freelook mode: {}", if cat.freelook_toggle { "Toggle" } else { "Hold" }),
            ModCfgKind::ZoomifyMode => format!("Zoom key behavior: {}", if cat.zoomify_toggle { "Toggle" } else { "Hold" }),
            ModCfgKind::ZoomifyScroll => format!("Enable Scroll Zoom: {}", if cat.zoomify_scroll { "ON" } else { "OFF" }),
            ModCfgKind::ZoomifyRetainSteps => format!("Remember Zoom Steps: {}", if cat.zoomify_retain_steps { "ON" } else { "OFF" }),
            ModCfgKind::ZoomifySensitivity => format!("Relative Sensitivity: {}", if cat.zoomify_relative_sensitivity { "ON" } else { "OFF" }),
            ModCfgKind::BetterF3Compact => format!(
                "BetterF3 Layout: {}",
                if cat.betterf3_compact {
                    "Compact Cards"
                } else {
                    "Full Columns"
                }
            ),
            ModCfgKind::InvHudOpacity => {
                format!("Inventory HUD+ Opacity: {}%", cat.inventory_hud_opacity)
            }
            ModCfgKind::CoordsBiome => format!(
                "Show Current Biome: {}",
                if cat.coords_show_biome { "ON" } else { "OFF" }
            ),
            ModCfgKind::DynFpsIdle => {
                format!("Idle / Unfocused FPS Cap: {} FPS", cat.dynamic_fps_idle_cap)
            }
            ModCfgKind::CloudsDensity => {
                format!("Cloud Layer Density: {}%", cat.better_clouds_density)
            }
            ModCfgKind::MouseTweaksRmb => format!(
                "RMB Slot Drag Sweep: {}",
                if cat.mouse_tweaks_rmb { "ON" } else { "OFF" }
            ),
            ModCfgKind::ItemScrollerWheel => format!(
                "Mouse Wheel Slot Transfer: {}",
                if cat.item_scroller_wheel { "ON" } else { "OFF" }
            ),
            ModCfgKind::OpenWorldMap => "Open Xaero's World Map...".to_owned(),
            ModCfgKind::OpenKeyBinds => "Open Key Binds...".to_owned(),
        }
    }

    fn compute_layout(&mut self, s: &Settings, w: f32, h: f32, count: usize) -> MenuLayout {
        let pad_x = if w > 520.0 { 24.0 } else { 8.0 };
        let pad_y = if h > 300.0 { 16.0 } else { 6.0 };
        let mw = (w - pad_x * 2.0).clamp(300.0, 620.0);
        let mh = (h - pad_y * 2.0).clamp(200.0, 380.0);
        let mx = ((w - mw) * 0.5).floor();
        let my = ((h - mh) * 0.5).floor();
        let modal = [mx, my, mw, mh];

        let search_w = (mw * 0.30).clamp(104.0, 168.0);
        let search_box = [mx + mw - 10.0 - search_w, my + 8.0, search_w, 16.0];
        let search_clear = [search_box[0] + search_w - 16.0, search_box[1], 16.0, 16.0];
        let gear_btn = [search_box[0] - 20.0, my + 8.0, 16.0, 16.0];

        let list_x = mx + 10.0;
        let list_w = (mw * 0.36).clamp(124.0, 196.0);
        let tab_y = my + 30.0;
        let tab_w = ((list_w - 6.0) / 4.0).floor();
        let tabs = std::array::from_fn(|i| {
            [
                list_x + i as f32 * (tab_w + 2.0),
                tab_y,
                tab_w,
                14.0,
            ]
        });

        let list_top = tab_y + 18.0;
        let footer_y = my + mh - 24.0;
        let folder_btn = [list_x, footer_y, list_w, 18.0];
        let list_bottom = folder_btn[1] - 6.0;

        let content_h = (count as f32 * (ROW_H + ROW_GAP) - ROW_GAP).max(0.0);
        self.scroll.clamp(list_top, list_bottom, content_h);

        let dx = list_x + list_w + 10.0;
        let dw = (mx + mw - 10.0 - dx).max(120.0);
        let detail_card = [dx, tab_y, dw, footer_y - 6.0 - tab_y];
        let fav_btn = [dx + dw - 24.0, tab_y + 8.0, 16.0, 16.0];
        let toggle_bar = [dx + 10.0, tab_y + 86.0, dw - 20.0, 22.0];

        let cfg_top = toggle_bar[1] + toggle_bar[3] + 6.0;
        let cfg_bottom = (detail_card[1] + detail_card[3] - 6.0).max(cfg_top);
        let config_items = Self::mod_config_items(self.selected);
        let cfg_content_h = (config_items.len() as f32 * 21.0 - 3.0).max(0.0);
        self.cfg_scroll = self.cfg_scroll.clamp(0.0, (cfg_content_h - (cfg_bottom - cfg_top)).max(0.0));
        let cfg_rows = config_items.into_iter().enumerate().filter_map(|(i, kind)| {
            let cy = cfg_top + i as f32 * 21.0 - self.cfg_scroll;
            (cy + 18.0 > cfg_top && cy < cfg_bottom)
                .then_some(([dx + 10.0, cy, dw - 26.0, 18.0], kind))
        }).collect();
        let _ = s;
        let done_btn = [mx + mw - 86.0, footer_y, 76.0, 18.0];

        MenuLayout {
            modal,
            gear_btn,
            search_box,
            search_clear,
            tabs,
            list_x,
            list_w,
            list_top,
            list_bottom,
            folder_btn,
            detail_card,
            toggle_bar,
            fav_btn,
            cfg_top,
            cfg_bottom,
            cfg_content_h,
            cfg_rows,
            done_btn,
        }
    }

    fn step_animations(&mut self, target_sel_y: f32, selected_on: bool) {
        let now = Instant::now();
        let dt = self
            .last_frame
            .map(|p| now.duration_since(p).as_secs_f32().clamp(0.0, 0.08))
            .unwrap_or(0.016);
        self.last_frame = Some(now);
        let fast = 1.0 - (-18.0 * dt).exp();
        self.open_anim += (1.0 - self.open_anim) * fast;
        let target_toggle = if selected_on { 1.0 } else { 0.0 };
        self.toggle_anim += (target_toggle - self.toggle_anim) * fast;
        if (self.sel_anim_y - target_sel_y).abs() > 160.0 || self.sel_anim_y == 0.0 {
            self.sel_anim_y = target_sel_y;
        } else {
            self.sel_anim_y += (target_sel_y - self.sel_anim_y) * fast;
        }
    }

    pub fn draw(
        &mut self,
        s: &Settings,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        cursor: (f32, f32),
        in_world: bool,
    ) {
        let (w, h) = (batch.width, batch.height);
        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, in_world);
        batch.fill(atlas, 0.0, 0.0, w, h, MM_VEIL);

        let mods = self.filtered_mods(s);
        if !mods.is_empty() && !mods.contains(&self.selected) {
            self.selected = mods[0];
            self.cfg_scroll = 0.0;
        }
        let lay = self.compute_layout(s, w, h, mods.len());
        let sel_idx = mods.iter().position(|&m| m == self.selected).unwrap_or(0);
        let raw_sel_y = lay.list_top + sel_idx as f32 * (ROW_H + ROW_GAP) - self.scroll.amount;
        let sel_on = s.extras.mod_catalog.is_on(self.selected);
        self.step_animations(raw_sel_y, sel_on);

        // Outer glass modal
        let [mx, my, mw, mh] = lay.modal;
        batch.fill(atlas, mx, my, mw, mh, MM_PLATE);
        style::ring(batch, atlas, lay.modal, MM_EDGE);
        batch.fill(atlas, mx + 8.0, my + 26.0, mw - 16.0, 1.0, MM_EDGE);

        // Header: "Mod Menu" + "X/49 on"
        let on_total = s.extras.mod_catalog.enabled_count();
        let title = Text::literal("Mod Menu");
        batch.text_styled(atlas, &title, mx + 10.0, my + 11.0, style::WHITE, true);
        let count_str = format!("{on_total}/{TOTAL_VOLT_MODS} on");
        let title_w = atlas.font.width("Mod Menu");
        batch.text_styled(
            atlas,
            &Text::literal(count_str),
            mx + 16.0 + title_w,
            my + 11.0,
            MM_MUTED,
            true,
        );

        // Gear quick-preset button + Search box
        let gear_hover = ui::hovered(
            cursor,
            lay.gear_btn[0],
            lay.gear_btn[1],
            lay.gear_btn[2],
            lay.gear_btn[3],
        );
        batch.fill(
            atlas,
            lay.gear_btn[0],
            lay.gear_btn[1],
            lay.gear_btn[2],
            lay.gear_btn[3],
            if gear_hover { MM_INNER_HOVER } else { MM_INNER },
        );
        style::ring(batch, atlas, lay.gear_btn, MM_EDGE);
        ui::centered_text(
            batch,
            atlas,
            &Text::literal("*"),
            lay.gear_btn[0],
            lay.gear_btn[1] + 4.0,
            lay.gear_btn[2],
            if gear_hover { MM_TEAL } else { MM_SOFT },
        );

        let [sx, sy, sw, sh] = lay.search_box;
        batch.fill(atlas, sx, sy, sw, sh, MM_INNER);
        style::ring(
            batch,
            atlas,
            lay.search_box,
            if self.search_focused {
                MM_TEAL
            } else {
                MM_EDGE
            },
        );
        self.search.focused = self.search_focused;
        self.search.x = sx + 5.0;
        self.search.y = sy + 4.0;
        self.search.width = lay.search_clear[0] - self.search.x;
        self.search.text_color = style::WHITE;
        self.search.draw(batch, atlas, None);
        if self.search.value.is_empty() && !self.search_focused {
            batch.text_styled(
                atlas,
                &Text::literal("Search mods..."),
                sx + 5.0,
                sy + 4.0,
                MM_MUTED,
                true,
            );
        } else if !self.search.value.is_empty() {
            let ch = ui::hovered(
                cursor,
                lay.search_clear[0],
                lay.search_clear[1],
                lay.search_clear[2],
                lay.search_clear[3],
            );
            ui::centered_text(
                batch,
                atlas,
                &Text::literal("x"),
                lay.search_clear[0],
                sy + 4.0,
                lay.search_clear[2],
                if ch { style::WHITE } else { MM_MUTED },
            );
        }

        // Filter tabs: All / On / Off / Fav
        let fav_total = s.extras.mod_catalog.fav_count();
        let tab_labels = [
            (ModFilterTab::All, format!("All {TOTAL_VOLT_MODS}")),
            (ModFilterTab::On, format!("On {on_total}")),
            (
                ModFilterTab::Off,
                format!("Off {}", TOTAL_VOLT_MODS - on_total),
            ),
            (ModFilterTab::Fav, format!("Fav {fav_total}")),
        ];
        for (i, (tab, label)) in tab_labels.iter().enumerate() {
            let r = lay.tabs[i];
            let active = self.filter == *tab;
            let hov = ui::hovered(cursor, r[0], r[1], r[2], r[3]);
            batch.fill(
                atlas,
                r[0],
                r[1],
                r[2],
                r[3],
                if active {
                    MM_SEL_FILL
                } else if hov {
                    MM_INNER_HOVER
                } else {
                    MM_INNER
                },
            );
            if active {
                batch.fill(atlas, r[0], r[1] + r[3] - 2.0, r[2], 2.0, MM_TEAL);
            }
            let fit_lbl = style::fit(atlas, label, r[2] - 2.0);
            ui::centered_text(
                batch,
                atlas,
                &Text::literal(fit_lbl),
                r[0],
                r[1] + 3.0,
                r[2],
                if active { style::WHITE } else { MM_MUTED },
            );
        }

        // Left scrollable mod list
        let clip_start = batch.vertices.len();
        let over_list = cursor.1 >= lay.list_top && cursor.1 < lay.list_bottom;
        if !mods.is_empty()
            && self.sel_anim_y + ROW_H > lay.list_top
            && self.sel_anim_y < lay.list_bottom
        {
            let hl = [lay.list_x, self.sel_anim_y, lay.list_w - 6.0, ROW_H];
            batch.fill(atlas, hl[0], hl[1], hl[2], hl[3], MM_SEL_FILL);
            style::ring(batch, atlas, hl, MM_TEAL);
        }
        for (i, &id) in mods.iter().enumerate() {
            let ry = lay.list_top + i as f32 * (ROW_H + ROW_GAP) - self.scroll.amount;
            if ry + ROW_H <= lay.list_top || ry >= lay.list_bottom {
                continue;
            }
            let row_w = lay.list_w - 6.0;
            let hov = over_list && ui::hovered(cursor, lay.list_x, ry, row_w, ROW_H);
            let is_sel = id == self.selected;
            if !is_sel {
                batch.fill(
                    atlas,
                    lay.list_x,
                    ry,
                    row_w,
                    ROW_H,
                    if hov { MM_INNER_HOVER } else { MM_INNER },
                );
                style::ring(batch, atlas, [lay.list_x, ry, row_w, ROW_H], MM_EDGE);
            }
            draw_mod_logo_hud(batch, atlas, id, lay.list_x + 5.0, ry + 5.0, 18.0);
            let meta = id.meta();
            let on = s.extras.mod_catalog.is_on(id);
            let max_title_w = row_w - 42.0;
            let fitted = style::fit(atlas, meta.title, max_title_w);
            batch.text_styled(
                atlas,
                &Text::literal(fitted),
                lay.list_x + 28.0,
                ry + 4.0,
                style::WHITE,
                true,
            );
            let (st_txt, st_col) = if on {
                ("Enabled", MM_TEAL)
            } else {
                ("Disabled", MM_CORAL)
            };
            batch.text_styled(
                atlas,
                &Text::literal(st_txt),
                lay.list_x + 28.0,
                ry + 15.0,
                st_col,
                true,
            );
            // Status dot on right edge (Image 2)
            let dot_x = lay.list_x + row_w - 9.0;
            batch.fill(atlas, dot_x, ry + 12.0, 4.0, 4.0, st_col);
        }
        if mods.is_empty() {
            batch.text_styled(
                atlas,
                &Text::literal("No mods match filter."),
                lay.list_x + 4.0,
                lay.list_top + 8.0,
                MM_MUTED,
                true,
            );
        }
        batch.clip_since(
            clip_start,
            lay.list_x,
            lay.list_top,
            lay.list_x + lay.list_w,
            lay.list_bottom,
        );

        let content_h = (mods.len() as f32 * (ROW_H + ROW_GAP) - ROW_GAP).max(0.0);
        if let Some((ty, th)) =
            style::scrollbar_thumb(lay.list_top, lay.list_bottom, content_h, self.scroll.amount)
        {
            let bx = lay.list_x + lay.list_w - 4.0;
            style::scroller(
                batch,
                atlas,
                [bx, lay.list_top, 4.0, lay.list_bottom - lay.list_top],
                [bx, ty, 4.0, th],
            );
        }

        // Bottom-left "Open mods folder" button
        let fhov = ui::hovered(
            cursor,
            lay.folder_btn[0],
            lay.folder_btn[1],
            lay.folder_btn[2],
            lay.folder_btn[3],
        );
        batch.fill(
            atlas,
            lay.folder_btn[0],
            lay.folder_btn[1],
            lay.folder_btn[2],
            lay.folder_btn[3],
            if fhov { MM_INNER_HOVER } else { MM_INNER },
        );
        style::ring(batch, atlas, lay.folder_btn, MM_EDGE);
        ui::centered_text(
            batch,
            atlas,
            &Text::literal(style::fit(atlas, "Open mods folder", lay.folder_btn[2] - 8.0)),
            lay.folder_btn[0],
            lay.folder_btn[1] + 5.0,
            lay.folder_btn[2],
            if fhov { style::WHITE } else { MM_SOFT },
        );

        // Right detail card (Image 2)
        let [dx, dy, dw, dh] = lay.detail_card;
        batch.fill(atlas, dx, dy, dw, dh, MM_INNER);
        style::ring(batch, atlas, lay.detail_card, MM_EDGE);

        let meta = self.selected.meta();
        let is_on = s.extras.mod_catalog.is_on(self.selected);
        let is_fav = s.extras.mod_catalog.is_fav(self.selected);

        draw_mod_logo_hud(batch, atlas, self.selected, dx + 10.0, dy + 10.0, 28.0);
        let head_title = style::fit(atlas, meta.title, dw - 108.0);
        batch.text_styled(
            atlas,
            &Text::literal(&head_title),
            dx + 44.0,
            dy + 11.0,
            style::WHITE,
            true,
        );
        let ver_str = format!("v{}", meta.version);
        batch.text_styled(
            atlas,
            &Text::literal(ver_str),
            dx + 48.0 + atlas.font.width(&head_title),
            dy + 11.0,
            MM_MUTED,
            true,
        );

        // Favorite star button at top-right of detail card
        let fav_hov = ui::hovered(
            cursor,
            lay.fav_btn[0],
            lay.fav_btn[1],
            lay.fav_btn[2],
            lay.fav_btn[3],
        );
        batch.fill(
            atlas,
            lay.fav_btn[0],
            lay.fav_btn[1],
            lay.fav_btn[2],
            lay.fav_btn[3],
            if fav_hov { MM_INNER_HOVER } else { MM_PLATE },
        );
        style::ring(
            batch,
            atlas,
            lay.fav_btn,
            if is_fav { MM_TEAL } else { MM_EDGE },
        );
        ui::centered_text(
            batch,
            atlas,
            &Text::literal(if is_fav { "*" } else { "+" }),
            lay.fav_btn[0],
            lay.fav_btn[1] + 4.0,
            lay.fav_btn[2],
            if is_fav { MM_TEAL } else { MM_MUTED },
        );

        // Pill badges: MOD | ENABLED/DISABLED | <slug> | <Category>
        let mut bx = dx + 44.0;
        let by = dy + 24.0;
        let badges: [(&str, u32, u32); 4] = [
            ("MOD", MM_BADGE_MOD, MM_TEAL),
            (
                if is_on { "ENABLED" } else { "DISABLED" },
                if is_on { MM_MINT_BG } else { MM_CORAL_BG },
                if is_on { MM_MINT } else { MM_CORAL },
            ),
            (meta.slug, MM_BADGE_SLUG, MM_SOFT),
            (meta.category.short_label(), MM_BADGE_CAT, style::ACCENT),
        ];
        for (txt, bg, fg) in badges {
            let bw = atlas.font.width(txt) + 8.0;
            if bx + bw > dx + dw - 8.0 {
                break;
            }
            batch.fill(atlas, bx, by, bw, 12.0, bg);
            batch.text_styled(atlas, &Text::literal(txt), bx + 4.0, by + 2.0, fg, true);
            bx += bw + 4.0;
        }

        // Wrapped description
        let desc_lines = style::wrap_lines(atlas, meta.description, dw - 20.0);
        for (li, line) in desc_lines.iter().take(3).enumerate() {
            batch.text_styled(
                atlas,
                &Text::literal(line.clone()),
                dx + 10.0,
                dy + 44.0 + li as f32 * 11.0,
                MM_SOFT,
                true,
            );
        }

        // Full-width interactive Toggle Switch Bar
        let [tx, ty, tw, th] = lay.toggle_bar;
        let thov = ui::hovered(cursor, tx, ty, tw, th);
        batch.fill(
            atlas,
            tx,
            ty,
            tw,
            th,
            if is_on {
                MM_MINT_BG
            } else if thov {
                MM_INNER_HOVER
            } else {
                MM_PLATE
            },
        );
        style::ring(
            batch,
            atlas,
            lay.toggle_bar,
            if is_on {
                MM_TEAL
            } else if thov {
                style::WHITE
            } else {
                MM_EDGE
            },
        );
        let toggle_caption = if is_on {
            "Enabled — Active in native Rust"
        } else {
            "Disabled — Tap to enable mod"
        };
        batch.text_styled(
            atlas,
            &Text::literal(style::fit(atlas, toggle_caption, tw - 44.0)),
            tx + 8.0,
            ty + 7.0,
            if is_on { style::WHITE } else { MM_SOFT },
            true,
        );
        // Animated pill switch on the right of toggle_bar
        style::switch_pill(
            batch,
            atlas,
            [tx + tw - 32.0, ty + 5.0, 24.0, 12.0],
            self.toggle_anim,
            is_on,
        );

        // The settings viewport scrolls independently of the mod list.
        let cfg_start = batch.vertices.len();
        for (rect, kind) in &lay.cfg_rows {
            let chov = ui::hovered(cursor, rect[0], rect[1], rect[2], rect[3]);
            batch.fill(
                atlas,
                rect[0],
                rect[1],
                rect[2],
                rect[3],
                if chov { MM_INNER_HOVER } else { MM_PLATE },
            );
            style::ring(
                batch,
                atlas,
                *rect,
                if chov { style::WHITE } else { MM_EDGE },
            );
            let lbl = style::fit(atlas, &Self::cfg_label(*kind, s), rect[2] - 12.0);
            batch.text_styled(
                atlas,
                &Text::literal(lbl),
                rect[0] + 6.0,
                rect[1] + 5.0,
                style::WHITE,
                true,
            );
        }

        batch.clip_since(cfg_start, dx + 10.0, lay.cfg_top, dx + dw - 10.0, lay.cfg_bottom);
        if let Some((ty, th)) = style::scrollbar_thumb(lay.cfg_top, lay.cfg_bottom, lay.cfg_content_h, self.cfg_scroll) {
            let bx = dx + dw - 15.0;
            style::scroller(batch, atlas, [bx, lay.cfg_top, 4.0, lay.cfg_bottom - lay.cfg_top], [bx, ty, 4.0, th]);
        }

        // Bottom bar: "Esc to go back" + teal "Done" button
        batch.text_styled(
            atlas,
            &Text::literal("Esc to go back"),
            dx,
            lay.done_btn[1] + 5.0,
            MM_MUTED,
            true,
        );
        let dhov = ui::hovered(
            cursor,
            lay.done_btn[0],
            lay.done_btn[1],
            lay.done_btn[2],
            lay.done_btn[3],
        );
        let done_lbl = lang::get("gui.done");
        crate::widgets::record(&done_lbl, lay.done_btn, true);
        crate::widgets::target(lay.done_btn);
        batch.fill(
            atlas,
            lay.done_btn[0],
            lay.done_btn[1],
            lay.done_btn[2],
            lay.done_btn[3],
            if dhov { MM_TEAL_HOVER } else { MM_TEAL_BTN },
        );
        if dhov {
            style::ring(batch, atlas, lay.done_btn, style::WHITE);
        }
        ui::centered_text(
            batch,
            atlas,
            &Text::literal(done_lbl),
            lay.done_btn[0],
            lay.done_btn[1] + 5.0,
            lay.done_btn[2],
            style::WHITE,
        );
    }

    pub fn click(
        &mut self,
        s: &mut Settings,
        atlas: &HudAtlas,
        (x, y): (f32, f32),
        (w, h): (f32, f32),
        drag: bool,
        right: bool,
    ) -> Vec<ModMenuAction> {
        let mods = self.filtered_mods(s);
        let lay = self.compute_layout(s, w, h, mods.len());
        let content_h = (mods.len() as f32 * (ROW_H + ROW_GAP) - ROW_GAP).max(0.0);

        if drag {
            if self.scroll.held() {
                self.scroll.drag(y, lay.list_top, lay.list_bottom, content_h);
            }
            if let Some(last) = self.cfg_drag_y {
                let view = lay.cfg_bottom - lay.cfg_top;
                let max_scroll = (lay.cfg_content_h - view).max(0.0);
                if let Some((_, thumb)) = style::scrollbar_thumb(lay.cfg_top, lay.cfg_bottom, lay.cfg_content_h, self.cfg_scroll) {
                    let track = (view - thumb).max(1.0);
                    self.cfg_scroll = (self.cfg_scroll + (y - last) * max_scroll / track).clamp(0.0, max_scroll);
                }
                self.cfg_drag_y = Some(y);
            }
            return Vec::new();
        }

        let hit = |r: [f32; 4]| ui::hovered((x, y), r[0], r[1], r[2], r[3]);

        if self.scroll.press(
            (x, y),
            lay.list_x + lay.list_w - 6.0,
            lay.list_top,
            lay.list_bottom,
            content_h,
        ) {
            return Vec::new();
        }

        let [dx, _, dw, _] = lay.detail_card;
        if x >= dx + dw - 17.0 && x <= dx + dw - 8.0
            && y >= lay.cfg_top && y < lay.cfg_bottom
            && lay.cfg_content_h > lay.cfg_bottom - lay.cfg_top
        {
            self.cfg_drag_y = Some(y);
            return Vec::new();
        }

        // Search clear / focus
        if !self.search.value.is_empty() && hit(lay.search_clear) {
            self.search.set_value("");
            self.scroll.amount = 0.0;
            return vec![ModMenuAction::ButtonClick];
        }
        if hit(lay.search_box) {
            self.search_focused = true;
            self.search.focused = true;
            self.search.click(atlas, x, false, false);
            return Vec::new();
        }
        self.search_focused = false;
        self.search.focused = false;

        // Gear preset cycle
        if hit(lay.gear_btn) {
            self.preset_index = (self.preset_index + 1) % 3;
            for id in VoltModId::ALL {
                let meta = id.meta();
                let on = match self.preset_index {
                    0 => meta.default_on,
                    1 => matches!(
                        meta.category,
                        VoltModCategory::Performance
                            | VoltModCategory::PvpUtility
                            | VoltModCategory::PvpHud
                    ),
                    _ => true,
                };
                apply_mod_to_engine(id, on, &mut s.extras, &mut s.horizon);
            }
            return vec![ModMenuAction::ButtonClick, ModMenuAction::Changed];
        }

        // Filter tabs
        for (i, tab) in ModFilterTab::ALL.into_iter().enumerate() {
            if hit(lay.tabs[i]) {
                self.filter = tab;
                self.scroll.amount = 0.0;
                return vec![ModMenuAction::ButtonClick];
            }
        }

        // Mod list rows (left click selects, clicking already-selected or right-clicking toggles!)
        if y >= lay.list_top && y < lay.list_bottom {
            let row_w = lay.list_w - 6.0;
            for (i, &id) in mods.iter().enumerate() {
                let ry = lay.list_top + i as f32 * (ROW_H + ROW_GAP) - self.scroll.amount;
                if ui::hovered((x, y), lay.list_x, ry, row_w, ROW_H) {
                    if right || (self.selected == id && x >= lay.list_x + row_w - 28.0) {
                        let next = !s.extras.mod_catalog.is_on(id);
                        apply_mod_to_engine(id, next, &mut s.extras, &mut s.horizon);
                        if self.selected != id { self.cfg_scroll = 0.0; }
                        self.selected = id;
                        return vec![ModMenuAction::ButtonClick, ModMenuAction::Changed];
                    }
                    if self.selected != id {
                        self.cfg_scroll = 0.0;
                    }
                    self.selected = id;
                    self.toggle_anim = if s.extras.mod_catalog.is_on(id) {
                        1.0
                    } else {
                        0.0
                    };
                    return vec![ModMenuAction::ButtonClick];
                }
            }
        }

        if hit(lay.folder_btn) {
            return vec![ModMenuAction::ButtonClick, ModMenuAction::OpenFolder];
        }
        if hit(lay.done_btn) {
            return vec![ModMenuAction::ButtonClick, ModMenuAction::Done];
        }
        if hit(lay.fav_btn) {
            s.extras.mod_catalog.toggle_fav(self.selected);
            return vec![ModMenuAction::ButtonClick, ModMenuAction::Changed];
        }
        if hit(lay.toggle_bar) {
            let next = !s.extras.mod_catalog.is_on(self.selected);
            apply_mod_to_engine(self.selected, next, &mut s.extras, &mut s.horizon);
            return vec![ModMenuAction::ButtonClick, ModMenuAction::Changed];
        }

        for (rect, kind) in lay.cfg_rows {
            if y >= lay.cfg_top && y < lay.cfg_bottom && hit(rect) {
                return match kind {
                    ModCfgKind::OpenWorldMap => {
                        vec![ModMenuAction::ButtonClick, ModMenuAction::OpenWorldMap]
                    }
                    ModCfgKind::OpenKeyBinds => {
                        vec![ModMenuAction::ButtonClick, ModMenuAction::OpenKeyBinds]
                    }
                    ModCfgKind::VoltAct(act) => {
                        act.trigger(&mut s.extras, right);
                        sync_catalog_from_engine(&mut s.extras, &s.horizon);
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::ZoomifyMode => {
                        s.extras.mod_catalog.zoomify_toggle = !s.extras.mod_catalog.zoomify_toggle;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::ZoomifyScroll => {
                        s.extras.mod_catalog.zoomify_scroll = !s.extras.mod_catalog.zoomify_scroll;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::ZoomifyRetainSteps => {
                        s.extras.mod_catalog.zoomify_retain_steps = !s.extras.mod_catalog.zoomify_retain_steps;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::ZoomifySensitivity => {
                        s.extras.mod_catalog.zoomify_relative_sensitivity = !s.extras.mod_catalog.zoomify_relative_sensitivity;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::FreelookMode => {
                        s.extras.mod_catalog.freelook_toggle = !s.extras.mod_catalog.freelook_toggle;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::BetterPingAutoColor => {
                        s.extras.mod_catalog.better_ping_auto_color = !s.extras.mod_catalog.better_ping_auto_color;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::BetterPingShowBars => {
                        s.extras.mod_catalog.better_ping_show_bars = !s.extras.mod_catalog.better_ping_show_bars;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::AppleSkinTooltip => {
                        s.extras.mod_catalog.appleskin_tooltip = !s.extras.mod_catalog.appleskin_tooltip;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::AppleSkinTooltipAlways => {
                        s.extras.mod_catalog.appleskin_tooltip_always = !s.extras.mod_catalog.appleskin_tooltip_always;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::AppleSkinOffhand => {
                        s.extras.mod_catalog.appleskin_offhand = !s.extras.mod_catalog.appleskin_offhand;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::AppleSkinExhaustion => {
                        s.extras.mod_catalog.appleskin_exhaustion = !s.extras.mod_catalog.appleskin_exhaustion;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::AppleSkinDebug => {
                        s.extras.mod_catalog.appleskin_debug = !s.extras.mod_catalog.appleskin_debug;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::AppleSkinSaturation => {
                        s.extras.mod_catalog.appleskin_saturation = !s.extras.mod_catalog.appleskin_saturation;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::AppleSkinFoodPreview => {
                        s.extras.mod_catalog.appleskin_food_preview = !s.extras.mod_catalog.appleskin_food_preview;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::BetterF3Compact => {
                        s.extras.mod_catalog.betterf3_compact =
                            !s.extras.mod_catalog.betterf3_compact;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::InvHudOpacity => {
                        let cur = s.extras.mod_catalog.inventory_hud_opacity;
                        s.extras.mod_catalog.inventory_hud_opacity = if cur >= 100 {
                            40
                        } else {
                            (cur + 20).min(100)
                        };
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::CoordsBiome => {
                        s.extras.mod_catalog.coords_show_biome =
                            !s.extras.mod_catalog.coords_show_biome;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::DynFpsIdle => {
                        let cur = s.extras.mod_catalog.dynamic_fps_idle_cap;
                        s.extras.mod_catalog.dynamic_fps_idle_cap = match cur {
                            10 => 15,
                            15 => 30,
                            30 => 60,
                            _ => 10,
                        };
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::CloudsDensity => {
                        let cur = s.extras.mod_catalog.better_clouds_density;
                        s.extras.mod_catalog.better_clouds_density = match cur {
                            75 => 100,
                            100 => 125,
                            125 => 150,
                            _ => 75,
                        };
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::MouseTweaksRmb => {
                        s.extras.mod_catalog.mouse_tweaks_rmb =
                            !s.extras.mod_catalog.mouse_tweaks_rmb;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                    ModCfgKind::ItemScrollerWheel => {
                        s.extras.mod_catalog.item_scroller_wheel =
                            !s.extras.mod_catalog.item_scroller_wheel;
                        vec![ModMenuAction::ButtonClick, ModMenuAction::Changed]
                    }
                };
            }
        }
        Vec::new()
    }

    pub fn release(&mut self) {
        self.scroll.release();
        self.cfg_drag_y = None;
    }

    pub fn scroll(&mut self, s: &Settings, (w, h): (f32, f32), cursor: (f32, f32), amount: f32) {
        let count = self.filtered_mods(s).len();
        let lay = self.compute_layout(s, w, h, count);
        let [dx, _, dw, _] = lay.detail_card;
        if cursor.0 >= dx && cursor.0 < dx + dw && cursor.1 >= lay.cfg_top && cursor.1 < lay.cfg_bottom {
            let max_scroll = (lay.cfg_content_h - (lay.cfg_bottom - lay.cfg_top)).max(0.0);
            self.cfg_scroll = (self.cfg_scroll - amount * 22.0).clamp(0.0, max_scroll);
        } else {
            let content_h = (count as f32 * (ROW_H + ROW_GAP) - ROW_GAP).max(0.0);
            self.scroll.wheel(amount, 22.0, lay.list_top, lay.list_bottom, content_h);
        }
    }

    pub fn type_text(&mut self, text: &str) {
        if self.search_focused {
            self.search.insert_text(text);
            self.scroll.amount = 0.0;
        }
    }

    pub fn key(
        &mut self,
        s: &mut Settings,
        key: winit::keyboard::KeyCode,
        text: Option<&str>,
        shift: bool,
    ) -> (Vec<ModMenuAction>, bool) {
        use winit::keyboard::KeyCode as K;
        if key == K::Escape {
            if self.search_focused {
                self.search_focused = false;
                self.search.focused = false;
                return (Vec::new(), true);
            }
            return (Vec::new(), false);
        }
        if self.search_focused {
            let before = self.search.value.clone();
            if !self.search.key(key, false, shift)
                && let Some(t) = text
            {
                let clean: String = t.chars().filter(|c| !c.is_control()).collect();
                self.search.insert_text(&clean);
            }
            if self.search.value != before {
                self.scroll.amount = 0.0;
            }
            return (Vec::new(), true);
        }
        match key {
            K::Space | K::Enter | K::NumpadEnter => {
                let next = !s.extras.mod_catalog.is_on(self.selected);
                apply_mod_to_engine(self.selected, next, &mut s.extras, &mut s.horizon);
                (
                    vec![ModMenuAction::ButtonClick, ModMenuAction::Changed],
                    true,
                )
            }
            K::ArrowDown | K::ArrowUp => {
                let mods = self.filtered_mods(s);
                if let Some(pos) = mods.iter().position(|&m| m == self.selected) {
                    let next = if key == K::ArrowDown {
                        (pos + 1).min(mods.len() - 1)
                    } else {
                        pos.saturating_sub(1)
                    };
                    if self.selected != mods[next] { self.cfg_scroll = 0.0; }
                    self.selected = mods[next];
                }
                (Vec::new(), true)
            }
            _ => (Vec::new(), false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verified_upstream_icons_decode_and_are_available_in_full_atlas() {
        let atlas = HudAtlas::build(std::path::Path::new("/nonexistent/volt-mod-icons-test")).unwrap();
        let mut count = 0;
        for id in VoltModId::ALL {
            if let Some((name, bytes)) = upstream_icon(id) {
                let (w, h, pixels) = crate::hud::decode_bytes(bytes).unwrap();
                assert_eq!((w, h, pixels.len()), (64, 64, 64 * 64 * 4));
                assert!(atlas.sprites.contains_key(name));
                count += 1;
            }
        }
        assert_eq!(count, 18);
        assert!(upstream_icon(VoltModId::Zoomify).is_none());
    }

    #[test]
    fn short_mod_menu_scrolls_settings_independently() {
        let settings = Settings::default();
        let mut menu = ModMenuUi::default();
        menu.selected = VoltModId::XaerosWorldMap;
        let layout = menu.compute_layout(&settings, 320.0, 200.0, 50);
        assert!(layout.cfg_content_h > layout.cfg_bottom - layout.cfg_top);
        assert!(layout.cfg_rows.iter().any(|(_, kind)| *kind == ModCfgKind::OpenWorldMap));
        let cursor = (layout.detail_card[0] + 12.0, (layout.cfg_top + layout.cfg_bottom) / 2.0);
        menu.scroll(&settings, (320.0, 200.0), cursor, -10.0);
        let scrolled = menu.compute_layout(&settings, 320.0, 200.0, 50);
        assert!(menu.cfg_scroll > 0.0);
        assert_eq!(menu.scroll.amount, 0.0);
        assert!(scrolled.cfg_rows.iter().any(|(_, kind)| *kind == ModCfgKind::VoltAct(VoltAct::QolAtlasGrid)));
        assert!(style::scrollbar_thumb(scrolled.cfg_top, scrolled.cfg_bottom, scrolled.cfg_content_h, menu.cfg_scroll).is_some());
        menu.selected = VoltModId::AppleSkin;
        menu.cfg_scroll = 0.0;
        menu.scroll(&settings, (320.0, 200.0), cursor, -20.0);
        let appleskin = menu.compute_layout(&settings, 320.0, 200.0, 50);
        assert!(appleskin.cfg_rows.iter().any(|(_, kind)| *kind == ModCfgKind::AppleSkinDebug));
    }

    #[test]
    fn zoomify_steps_and_options_are_bounded_and_persistent() {
        assert_eq!(zoomify_step_bounds(25), (-4, 10));
        assert!((zoomify_factor(25, 0) - 4.0).abs() < 0.0001);
        assert!((zoomify_factor(25, 1) - 6.0).abs() < 0.0001);
        assert_eq!(zoomify_factor(25, -999), 1.0);
        assert_eq!(zoomify_factor(25, 999), zoomify_factor(25, 10));
        assert!(zoomify_factor(0, 1000).is_finite());
        assert!(zoomify_factor(25, 10) > 10.0);
        let target = 1.0 / zoomify_factor(25, 10);
        assert_eq!(zoomify_ease_scale(1.0, target, 0.0, false), target);
        let halfway = zoomify_ease_scale(1.0, target, 0.05, true);
        assert!(halfway > target && halfway < 1.0);
        assert_eq!(zoomify_ease_scale(f32::NAN, f32::NAN, 0.1, true), 1.0);
        let mut extras = Extras::default();
        assert!(!extras.volt_qol.zoom_enabled);
        assert!(!extras.volt_qol.freelook_enabled);
        assert!(!extras.mod_catalog.zoomify_toggle);
        assert!(extras.mod_catalog.zoomify_scroll);
        extras.mod_catalog.zoomify_toggle = true;
        extras.mod_catalog.zoomify_scroll = false;
        extras.mod_catalog.zoomify_retain_steps = true;
        extras.mod_catalog.zoomify_relative_sensitivity = false;
        let parsed = Extras::parse(&extras.serialize());
        assert!(parsed.mod_catalog.zoomify_toggle);
        assert!(!parsed.mod_catalog.zoomify_scroll);
        assert!(parsed.mod_catalog.zoomify_retain_steps);
        assert!(!parsed.mod_catalog.zoomify_relative_sensitivity);
        assert_eq!(ModMenuUi::mod_config_items(VoltModId::Zoomify).len(), 7);
    }

    #[test]
    fn appleskin_options_persist() {
        let mut extras = Extras::default();
        assert!(extras.mod_catalog.appleskin_tooltip && extras.mod_catalog.appleskin_tooltip_always);
        assert!(extras.mod_catalog.appleskin_offhand && extras.mod_catalog.appleskin_exhaustion && extras.mod_catalog.appleskin_debug);
        extras.mod_catalog.appleskin_tooltip = false;
        extras.mod_catalog.appleskin_tooltip_always = false;
        extras.mod_catalog.appleskin_offhand = false;
        extras.mod_catalog.appleskin_exhaustion = false;
        extras.mod_catalog.appleskin_debug = false;
        let parsed = Extras::parse(&extras.serialize());
        assert!(!parsed.mod_catalog.appleskin_tooltip);
        assert!(!parsed.mod_catalog.appleskin_tooltip_always);
        assert!(!parsed.mod_catalog.appleskin_offhand);
        assert!(!parsed.mod_catalog.appleskin_exhaustion);
        assert!(!parsed.mod_catalog.appleskin_debug);
    }

    #[test]
    fn better_ping_settings_persist_with_upstream_defaults() {
        let mut extras = Extras::default();
        assert!(extras.mod_catalog.better_ping_auto_color);
        assert!(!extras.mod_catalog.better_ping_show_bars);
        extras.mod_catalog.better_ping_auto_color = false;
        extras.mod_catalog.better_ping_show_bars = true;
        let parsed = Extras::parse(&extras.serialize());
        assert!(!parsed.mod_catalog.better_ping_auto_color);
        assert!(parsed.mod_catalog.better_ping_show_bars);
    }

    #[test]
    fn mod_keybindings_survive_options_round_trip() {
        let mut settings = Settings::default();
        settings.keys.set("key.volt.zoom", "key.keyboard.v");
        settings.keys.set("key.volt.freelook", "key.mouse.4");
        let restored = Settings::parse(&settings.serialize());
        assert_eq!(restored.keys.get("key.volt.zoom"), "key.keyboard.v");
        assert_eq!(restored.keys.get("key.volt.freelook"), "key.mouse.4");
    }

    #[test]
    fn appleskin_preview_options_persist() {
        let mut extras = Extras::default();
        assert!(extras.mod_catalog.appleskin_saturation);
        assert!(extras.mod_catalog.appleskin_food_preview);
        extras.mod_catalog.appleskin_saturation = false;
        extras.mod_catalog.appleskin_food_preview = false;
        let parsed = Extras::parse(&extras.serialize());
        assert!(!parsed.mod_catalog.appleskin_saturation);
        assert!(!parsed.mod_catalog.appleskin_food_preview);
        assert_eq!(ModMenuUi::mod_config_items(VoltModId::AppleSkin).len(), 7);
    }

    #[test]
    fn all_forty_nine_mods_have_exact_category_counts_unique_slugs_and_logos() {
        assert_eq!(VoltModId::ALL.len(), 49);
        let count_cat = |c: VoltModCategory| {
            VoltModId::ALL
                .iter()
                .filter(|id| id.meta().category == c)
                .count()
        };
        assert_eq!(count_cat(VoltModCategory::PvpUtility), 12);
        assert_eq!(count_cat(VoltModCategory::PvpHud), 11);
        assert_eq!(count_cat(VoltModCategory::Visual), 9);
        assert_eq!(count_cat(VoltModCategory::Performance), 10);
        assert_eq!(count_cat(VoltModCategory::Additional), 7);

        let mut slugs = std::collections::HashSet::new();
        for (idx, id) in VoltModId::ALL.into_iter().enumerate() {
            assert_eq!(id.index(), idx);
            let m = id.meta();
            assert_eq!(m.id, id);
            assert!(slugs.insert(m.slug), "duplicate slug {}", m.slug);
            assert!(!m.title.is_empty());
            assert!(!m.version.is_empty());
            assert!(!m.description.is_empty());
            let logo = mod_logo_spec(id);
            assert_eq!(logo.rows.len(), 8);
            for r in logo.rows {
                assert_eq!(r.len(), 8, "logo row width for {}", m.slug);
            }
        }
    }

    #[test]
    fn independent_wheel_options_and_engine_edits_stay_in_sync() {
        let mut s = Settings::default();
        apply_mod_to_engine(VoltModId::MouseTweaks, true, &mut s.extras, &mut s.horizon);
        assert!(s.extras.mouse_tweaks_wheel);
        apply_mod_to_engine(VoltModId::ItemScroller, true, &mut s.extras, &mut s.horizon);
        apply_mod_to_engine(VoltModId::ItemScroller, false, &mut s.extras, &mut s.horizon);
        assert!(s.extras.mouse_tweaks_wheel, "disabling Item Scroller must not change Mouse Tweaks' wheel setting");
        s.extras.mouse_tweaks = false;
        s.extras.volt_qol.atlas_enabled = true;
        s.extras.cull_leaves = true;
        s.extras.volt_boost_plus.zero_stutter_mesh_pool = true;
        s.horizon.entity_culling = true;
        sync_catalog_from_engine(&mut s.extras, &s.horizon);
        assert!(!s.extras.mod_catalog.is_on(VoltModId::MouseTweaks));
        for id in [VoltModId::XaerosWorldMap, VoltModId::MoreCulling, VoltModId::FerriteCore, VoltModId::EntityCulling] {
            assert!(s.extras.mod_catalog.is_on(id));
        }
        s.extras.volt_qol.atlas_enabled = false;
        s.extras.cull_leaves = false;
        s.extras.volt_boost_plus.zero_stutter_mesh_pool = false;
        s.horizon.entity_culling = false;
        sync_catalog_from_engine(&mut s.extras, &s.horizon);
        for id in [VoltModId::XaerosWorldMap, VoltModId::MoreCulling, VoltModId::FerriteCore, VoltModId::EntityCulling] {
            assert!(!s.extras.mod_catalog.is_on(id));
        }
    }

    #[test]
    fn mod_catalog_round_trips_and_syncs_with_engine() {
        let mut s = Settings::default();
        assert!(s.extras.mod_catalog.is_on(VoltModId::Sodium));
        assert!(s.extras.mod_catalog.is_on(VoltModId::ModMenu));
        assert!(!s.extras.mod_catalog.is_on(VoltModId::Fullbright));

        apply_mod_to_engine(VoltModId::Fullbright, true, &mut s.extras, &mut s.horizon);
        apply_mod_to_engine(
            VoltModId::XaerosMinimap,
            true,
            &mut s.extras,
            &mut s.horizon,
        );
        s.extras.mod_catalog.toggle_fav(VoltModId::Zoomify);
        assert!(s.extras.fullbright);
        assert!(s.extras.minimap);

        let parsed = Extras::parse(&s.extras.serialize());
        assert_eq!(parsed.mod_catalog, s.extras.mod_catalog);
        assert!(parsed.mod_catalog.is_on(VoltModId::Fullbright));
        assert!(parsed.mod_catalog.is_fav(VoltModId::Zoomify));
    }
}
