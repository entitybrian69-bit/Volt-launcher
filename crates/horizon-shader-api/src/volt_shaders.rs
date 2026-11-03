//! Built-in mobile-first native WGSL "Volt Shaders" engine.
//!
//! Designed specifically for tile-based mobile Vulkan GPUs (Arm Mali-G57,
//! Adreno, Xclipse):
//! - Zero shadow-map geometry re-draws and zero `PackAtlas` texture copies.
//! - Every toggle and mode compiles into SPIR-V specialization `override`
//!   constants so disabled effects cost zero ALU/texture instructions.
//! - When post-space effects (`Bloom`, `Sun Rays`, `Depth Haze`, `Underwater
//!   Refraction`, `Vignette`) are all off — including the `Potato`, `Low`, and
//!   `Medium` profiles — Volt Shaders omits the `[[post]]` stage entirely so
//!   the renderer stays in single-pass TBDR mode (`StoreOp::Discard` depth,
//!   zero off-screen render target bandwidth) and runs color grading inline.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use crate::check::{Report, prepare_trusted};
use crate::manifest::MANIFEST;
use crate::package::Pack;

/// Built-in Volt Shaders performance/quality profiles, plus `Custom`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VoltShaderProfile {
    Potato,
    Low,
    #[default]
    Medium,
    High,
    Ultra,
    Custom,
}

impl VoltShaderProfile {
    pub const ALL: [Self; 6] = [
        Self::Potato,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::Ultra,
        Self::Custom,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Potato => "potato",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Ultra => "ultra",
            Self::Custom => "custom",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Potato => "Potato",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
            Self::Custom => "Custom",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&p| p == self).unwrap_or(2);
        let next = if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        };
        Self::ALL[next]
    }
}

/// Block-light color temperature style for torches, lanterns, lava & glowstone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TorchLightStyle {
    Vanilla,
    #[default]
    WarmAmber,
    CozyGold,
    SoulCool,
}

impl TorchLightStyle {
    pub const ALL: [Self; 4] = [
        Self::Vanilla,
        Self::WarmAmber,
        Self::CozyGold,
        Self::SoulCool,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::WarmAmber => "warm_amber",
            Self::CozyGold => "cozy_gold",
            Self::SoulCool => "soul_cool",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::WarmAmber => "Warm Amber",
            Self::CozyGold => "Cozy Gold",
            Self::SoulCool => "Soul Cool",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Vanilla => 0,
            Self::WarmAmber => 1,
            Self::CozyGold => 2,
            Self::SoulCool => 3,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(1);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

/// Procedural night-sky Aurora Borealis intensity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AuroraMode {
    Off,
    #[default]
    Subtle,
    Medium,
    Vibrant,
}

impl AuroraMode {
    pub const ALL: [Self; 4] = [Self::Off, Self::Subtle, Self::Medium, Self::Vibrant];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Subtle => "subtle",
            Self::Medium => "medium",
            Self::Vibrant => "vibrant",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Subtle => "Subtle",
            Self::Medium => "Medium",
            Self::Vibrant => "Vibrant",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Subtle => 1,
            Self::Medium => 2,
            Self::Vibrant => 3,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

/// Distance depth haze mode (uses `scene_depth` in the post stage).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DepthHazeMode {
    #[default]
    Off,
    Subtle,
    Lush,
}

impl DepthHazeMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Subtle, Self::Lush];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Subtle => "subtle",
            Self::Lush => "lush",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Subtle => "Subtle",
            Self::Lush => "Lush",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Subtle => 1,
            Self::Lush => 2,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

/// Filmic tonemapping curve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TonemapMode {
    Off,
    #[default]
    FilmicAces,
    CrispNeutral,
    LushCinema,
}

impl TonemapMode {
    pub const ALL: [Self; 4] = [
        Self::Off,
        Self::FilmicAces,
        Self::CrispNeutral,
        Self::LushCinema,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::FilmicAces => "filmic_aces",
            Self::CrispNeutral => "crisp_neutral",
            Self::LushCinema => "lush_cinema",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::FilmicAces => "Filmic ACES",
            Self::CrispNeutral => "Crisp Neutral",
            Self::LushCinema => "Lush Cinema",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::FilmicAces => 1,
            Self::CrispNeutral => 2,
            Self::LushCinema => 3,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(1);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

/// Single-pass hardware-bilinear highlight bloom quality.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BloomMode {
    #[default]
    Off,
    Low,
    High,
}

impl BloomMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Low, Self::High];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::High => "high",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Low => "Low (Fast)",
            Self::High => "High (Smooth)",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Low => 1,
            Self::High => 2,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

/// Screen-edge cinematic vignette intensity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VignetteMode {
    #[default]
    Off,
    Subtle,
    Strong,
}

impl VignetteMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Subtle, Self::Strong];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Subtle => "subtle",
            Self::Strong => "strong",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Subtle => "Subtle",
            Self::Strong => "Strong",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Subtle => 1,
            Self::Strong => 2,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

/// Full configuration for Built-In Volt Shaders (`enabled` is `false` by default).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoltShadersConfig {
    /// Master toggle (OFF by default). When false, all customization options are hidden.
    pub enabled: bool,
    pub profile: VoltShaderProfile,
    // --- Wind & Moving Vegetation ---
    pub waving_plants: bool,
    pub waving_leaves: bool,
    pub waving_water: bool,
    /// Percentage `25..=200` (100 = 1.0x).
    pub wind_speed: u16,
    /// Percentage `25..=200` (100 = 1.0x).
    pub wind_strength: u16,
    pub storm_wind: bool,
    // --- Lighting & Shadows ---
    pub sun_shading: bool,
    /// Percentage `0..=100`.
    pub shadow_richness: u16,
    pub foliage_subsurface: bool,
    pub torch_style: TorchLightStyle,
    /// Percentage `50..=200` (100 = 1.0x).
    pub torch_intensity: u16,
    /// Percentage `0..=200` (100 = 1.0x).
    pub emissive_glow: u16,
    pub rain_wetness: bool,
    // --- Water ---
    pub water_ripples: bool,
    /// Percentage `0..=150` (100 = 1.0x).
    pub water_reflection: u16,
    pub water_glint: bool,
    pub clear_water: bool,
    pub underwater_refraction: bool,
    // --- Sky & Atmosphere ---
    pub custom_sky: bool,
    /// Percentage `0..=200` (100 = 1.0x).
    pub sunset_glow: u16,
    pub sun_halo: bool,
    pub aurora: AuroraMode,
    pub depth_haze: DepthHazeMode,
    // --- Post-Processing & Color Grading ---
    pub tonemap: TonemapMode,
    pub bloom: BloomMode,
    pub sun_rays: bool,
    /// Percentage `50..=180` (100 = neutral).
    pub vibrance: u16,
    /// Percentage `50..=180` (100 = neutral).
    pub saturation: u16,
    /// Percentage `70..=150` (100 = neutral).
    pub contrast: u16,
    /// Percentage `70..=150` (100 = neutral).
    pub exposure: u16,
    /// Signed temperature `-50..=50` (0 = neutral, positive = warm, negative = cool).
    pub warmth: i16,
    pub vignette: VignetteMode,
}

impl Default for VoltShadersConfig {
    fn default() -> Self {
        let mut cfg = Self::preset(VoltShaderProfile::Medium);
        cfg.enabled = false;
        cfg
    }
}

impl VoltShadersConfig {
    /// Returns a configuration populated with `profile`'s performance-tuned values.
    pub fn preset(profile: VoltShaderProfile) -> Self {
        match profile {
            VoltShaderProfile::Potato => Self {
                enabled: true,
                profile: VoltShaderProfile::Potato,
                waving_plants: true,
                waving_leaves: false,
                waving_water: false,
                wind_speed: 100,
                wind_strength: 85,
                storm_wind: false,
                sun_shading: true,
                shadow_richness: 25,
                foliage_subsurface: false,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 105,
                emissive_glow: 35,
                rain_wetness: false,
                water_ripples: false,
                water_reflection: 25,
                water_glint: false,
                clear_water: true,
                underwater_refraction: false,
                custom_sky: true,
                sunset_glow: 65,
                sun_halo: false,
                aurora: AuroraMode::Off,
                depth_haze: DepthHazeMode::Off,
                tonemap: TonemapMode::CrispNeutral,
                bloom: BloomMode::Off,
                sun_rays: false,
                vibrance: 110,
                saturation: 100,
                contrast: 102,
                exposure: 100,
                warmth: 0,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::Low => Self {
                enabled: true,
                profile: VoltShaderProfile::Low,
                waving_plants: true,
                waving_leaves: true,
                waving_water: false,
                wind_speed: 100,
                wind_strength: 95,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 40,
                foliage_subsurface: false,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 115,
                emissive_glow: 60,
                rain_wetness: false,
                water_ripples: true,
                water_reflection: 55,
                water_glint: false,
                clear_water: true,
                underwater_refraction: false,
                custom_sky: true,
                sunset_glow: 90,
                sun_halo: true,
                aurora: AuroraMode::Off,
                depth_haze: DepthHazeMode::Off,
                tonemap: TonemapMode::FilmicAces,
                bloom: BloomMode::Off,
                sun_rays: false,
                vibrance: 115,
                saturation: 102,
                contrast: 104,
                exposure: 100,
                warmth: 3,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::Medium | VoltShaderProfile::Custom => Self {
                enabled: true,
                profile,
                waving_plants: true,
                waving_leaves: true,
                waving_water: true,
                wind_speed: 100,
                wind_strength: 100,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 55,
                foliage_subsurface: true,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 125,
                emissive_glow: 85,
                rain_wetness: true,
                water_ripples: true,
                water_reflection: 80,
                water_glint: true,
                clear_water: true,
                underwater_refraction: false,
                custom_sky: true,
                sunset_glow: 110,
                sun_halo: true,
                aurora: AuroraMode::Subtle,
                depth_haze: DepthHazeMode::Off,
                tonemap: TonemapMode::FilmicAces,
                bloom: BloomMode::Off,
                sun_rays: false,
                vibrance: 120,
                saturation: 104,
                contrast: 105,
                exposure: 102,
                warmth: 5,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::High => Self {
                enabled: true,
                profile: VoltShaderProfile::High,
                waving_plants: true,
                waving_leaves: true,
                waving_water: true,
                wind_speed: 100,
                wind_strength: 105,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 65,
                foliage_subsurface: true,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 130,
                emissive_glow: 100,
                rain_wetness: true,
                water_ripples: true,
                water_reflection: 95,
                water_glint: true,
                clear_water: true,
                underwater_refraction: true,
                custom_sky: true,
                sunset_glow: 125,
                sun_halo: true,
                aurora: AuroraMode::Medium,
                depth_haze: DepthHazeMode::Subtle,
                tonemap: TonemapMode::FilmicAces,
                bloom: BloomMode::Low,
                sun_rays: false,
                vibrance: 124,
                saturation: 105,
                contrast: 106,
                exposure: 103,
                warmth: 6,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::Ultra => Self {
                enabled: true,
                profile: VoltShaderProfile::Ultra,
                waving_plants: true,
                waving_leaves: true,
                waving_water: true,
                wind_speed: 100,
                wind_strength: 110,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 75,
                foliage_subsurface: true,
                torch_style: TorchLightStyle::CozyGold,
                torch_intensity: 140,
                emissive_glow: 120,
                rain_wetness: true,
                water_ripples: true,
                water_reflection: 115,
                water_glint: true,
                clear_water: true,
                underwater_refraction: true,
                custom_sky: true,
                sunset_glow: 140,
                sun_halo: true,
                aurora: AuroraMode::Vibrant,
                depth_haze: DepthHazeMode::Lush,
                tonemap: TonemapMode::LushCinema,
                bloom: BloomMode::High,
                sun_rays: true,
                vibrance: 128,
                saturation: 106,
                contrast: 108,
                exposure: 104,
                warmth: 8,
                vignette: VignetteMode::Subtle,
            },
        }
    }

    /// Applies `profile` while preserving `self.enabled`. Selecting `Custom`
    /// keeps the current option values untouched so they can be tweaked freely.
    pub fn apply_profile(&mut self, profile: VoltShaderProfile) {
        if profile == VoltShaderProfile::Custom {
            self.profile = VoltShaderProfile::Custom;
            return;
        }
        let enabled = self.enabled;
        *self = Self::preset(profile);
        self.enabled = enabled;
    }

    /// Resets options to the active preset (or `Medium` when on `Custom`).
    pub fn reset_current_profile(&mut self) {
        let target = if self.profile == VoltShaderProfile::Custom {
            VoltShaderProfile::Medium
        } else {
            self.profile
        };
        let enabled = self.enabled;
        *self = Self::preset(target);
        self.enabled = enabled;
    }

    /// Marks the active profile as `Custom` whenever an individual option is edited.
    pub fn mark_custom(&mut self) {
        self.profile = VoltShaderProfile::Custom;
    }

    /// Whether any screen-space / depth-buffer effect requires the `[[post]]`
    /// pass. When false, Volt Shaders omits `[[post]]` so mobile TBDR GPUs keep
    /// `StoreOp::Discard` on the depth buffer and avoid off-screen render targets.
    pub fn needs_post_pass(&self) -> bool {
        self.bloom != BloomMode::Off
            || self.sun_rays
            || self.depth_haze != DepthHazeMode::Off
            || self.underwater_refraction
            || self.vignette != VignetteMode::Off
    }

    /// Stable 64-bit fingerprint of all shader parameters for zero-overhead change detection.
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// Pipeline specialization constants for `NativeSky`, `NativeMaterial`, and `NativePost`.
    pub fn pipeline_constants(&self) -> Vec<(String, f64)> {
        let b = |v: bool| f64::from(u8::from(v));
        let pct = |v: u16, lo: u16, hi: u16| f64::from(v.clamp(lo, hi)) / 100.0;
        let post_on = self.needs_post_pass();
        vec![
            ("post_active".into(), b(post_on)),
            ("sun_shading".into(), b(self.sun_shading)),
            ("shadow_richness".into(), pct(self.shadow_richness, 0, 100)),
            ("foliage_subsurface".into(), b(self.foliage_subsurface)),
            ("torch_style".into(), f64::from(self.torch_style.index())),
            ("torch_intensity".into(), pct(self.torch_intensity, 50, 200)),
            ("emissive_glow".into(), pct(self.emissive_glow, 0, 200)),
            ("rain_wetness".into(), b(self.rain_wetness)),
            ("water_ripples".into(), b(self.water_ripples)),
            (
                "water_reflection".into(),
                pct(self.water_reflection, 0, 150),
            ),
            ("water_glint".into(), b(self.water_glint)),
            ("clear_water".into(), b(self.clear_water)),
            (
                "underwater_refraction".into(),
                b(self.underwater_refraction),
            ),
            ("custom_sky".into(), b(self.custom_sky)),
            ("sunset_glow".into(), pct(self.sunset_glow, 0, 200)),
            ("sun_halo".into(), b(self.sun_halo)),
            ("aurora_mode".into(), f64::from(self.aurora.index())),
            ("depth_haze".into(), f64::from(self.depth_haze.index())),
            ("tonemap_mode".into(), f64::from(self.tonemap.index())),
            ("bloom_mode".into(), f64::from(self.bloom.index())),
            ("sun_rays".into(), b(self.sun_rays)),
            ("vibrance".into(), pct(self.vibrance, 50, 180)),
            ("saturation".into(), pct(self.saturation, 50, 180)),
            ("contrast".into(), pct(self.contrast, 70, 150)),
            ("exposure".into(), pct(self.exposure, 70, 150)),
            (
                "warmth".into(),
                f64::from(self.warmth.clamp(-50, 50)) / 100.0,
            ),
            ("vignette_mode".into(), f64::from(self.vignette.index())),
            // Vertex-stage wind & material tag overrides consumed by `NativeMaterial`:
            ("horizon_terrain_volt_pack".into(), 1.0),
            ("horizon_terrain_wave_plants".into(), b(self.waving_plants)),
            ("horizon_terrain_wave_leaves".into(), b(self.waving_leaves)),
            ("horizon_terrain_wave_water".into(), b(self.waving_water)),
            ("horizon_terrain_storm_wind".into(), b(self.storm_wind)),
            (
                "horizon_terrain_wind_speed".into(),
                pct(self.wind_speed, 25, 200),
            ),
            (
                "horizon_terrain_wind_strength".into(),
                pct(self.wind_strength, 25, 200),
            ),
        ]
    }

    /// Builds the validated Naga stages and specialization constants ready for
    /// `Renderer::set_native_shader_pack`.
    pub fn build_runtime(&self) -> Result<(Report, Vec<(String, f64)>), String> {
        let pack = build_pack(self.needs_post_pass())?;
        let report = prepare_trusted(&pack)
            .map_err(|problems| problems.iter().map(ToString::to_string).collect::<Vec<_>>().join("; "))?;
        Ok((report, self.pipeline_constants()))
    }
}

/// Number of `vec4<u32>` entries in the uniform table holding 4-bit sprite
/// material/wave classes (`256 * 32 = 8,192` block sprite layers supported).
pub const LAYER_KINDS_VECS: usize = 256;

/// Classifies a block sprite name (e.g. `"minecraft:block/short_grass"`) into
/// its 4-bit Volt Shaders wind/material class:
/// - `0`: static block
/// - `1`: canopy / leaves / vines / multi-block stems (3D sway, foliage SSS)
/// - `2`: grounded plant single/lower (root-anchored sway, foliage SSS)
/// - `3`: 2-block tall plant upper half (tip sway continuing lower half, foliage SSS)
/// - `4`: ceiling-hanging plant (ceiling-anchored sway, foliage SSS)
/// - `5`: still water surface (3D water wave + water shader)
/// - `6`: flowing water / water overlay (water shader, anchored geometry)
pub fn classify_sprite_layer(sprite: &str) -> u8 {
    let name = sprite.strip_prefix("minecraft:").unwrap_or(sprite);
    let name = name.strip_prefix("block/").unwrap_or(name);

    if name == "water_still" {
        return 5;
    }
    if matches!(name, "water_flow" | "water_overlay") {
        return 6;
    }
    if name.starts_with("potted_")
        || matches!(
            name,
            "grass_block_top"
                | "grass_block_side"
                | "grass_block_side_overlay"
                | "grass_block_snow"
                | "dirt_path_top"
                | "dirt_path_side"
                | "podzol_top"
                | "podzol_side"
                | "mycelium_top"
                | "mycelium_side"
                | "moss_block"
                | "moss_carpet"
                | "pale_moss_block"
                | "pale_moss_carpet"
                | "pink_petals"
                | "pink_petals_stem"
                | "wildflowers"
                | "wildflowers_stem"
                | "leaf_litter"
                | "lily_pad"
                | "spore_blossom_base"
        )
    {
        return 0;
    }
    if name.ends_with("_leaves")
        || matches!(
            name,
            "vine"
                | "sugar_cane"
                | "bamboo_stalk"
                | "bamboo_singleleaf"
                | "bamboo_large_leaves"
                | "bamboo_small_leaves"
                | "kelp_plant"
                | "cave_vines_plant"
                | "cave_vines_plant_lit"
                | "weeping_vines_plant"
                | "twisting_vines_plant"
                | "azalea_plant"
                | "azalea_side"
                | "azalea_top"
                | "flowering_azalea_side"
                | "flowering_azalea_top"
        )
    {
        return 1;
    }
    if matches!(
        name,
        "tall_grass_top"
            | "large_fern_top"
            | "sunflower_top"
            | "sunflower_front"
            | "sunflower_back"
            | "lilac_top"
            | "rose_bush_top"
            | "peony_top"
            | "pitcher_plant_top"
            | "tall_seagrass_top"
    ) {
        return 3;
    }
    if matches!(
        name,
        "hanging_roots"
            | "spore_blossom"
            | "cave_vines"
            | "cave_vines_lit"
            | "weeping_vines"
            | "pale_hanging_moss"
            | "pale_hanging_moss_tip"
    ) {
        return 4;
    }
    if name.ends_with("_sapling")
        || name.ends_with("_tulip")
        || name.starts_with("wheat_stage")
        || name.starts_with("carrots_stage")
        || name.starts_with("potatoes_stage")
        || name.starts_with("beetroots_stage")
        || name.starts_with("sweet_berry_bush_stage")
        || name.starts_with("torchflower_crop_stage")
        || name.starts_with("pitcher_crop_")
        || matches!(
            name,
            "short_grass"
                | "grass"
                | "fern"
                | "short_dry_grass"
                | "tall_dry_grass"
                | "dead_bush"
                | "bush"
                | "firefly_bush"
                | "firefly_bush_emissive"
                | "dandelion"
                | "poppy"
                | "blue_orchid"
                | "allium"
                | "azure_bluet"
                | "oxeye_daisy"
                | "cornflower"
                | "lily_of_the_valley"
                | "wither_rose"
                | "torchflower"
                | "open_eyeblossom"
                | "open_eyeblossom_emissive"
                | "closed_eyeblossom"
                | "cactus_flower"
                | "mangrove_propagule"
                | "mangrove_propagule_hanging"
                | "crimson_roots"
                | "crimson_roots_pot"
                | "warped_roots"
                | "warped_roots_pot"
                | "nether_sprouts"
                | "seagrass"
                | "kelp"
                | "twisting_vines"
                | "melon_stem"
                | "attached_melon_stem"
                | "pumpkin_stem"
                | "attached_pumpkin_stem"
                | "bamboo_stage0"
                | "tall_grass_bottom"
                | "large_fern_bottom"
                | "sunflower_bottom"
                | "lilac_bottom"
                | "rose_bush_bottom"
                | "peony_bottom"
                | "pitcher_plant_bottom"
                | "tall_seagrass_bottom"
                | "small_dripleaf_top"
                | "small_dripleaf_stem_top"
                | "small_dripleaf_stem_bottom"
                | "big_dripleaf_top"
                | "big_dripleaf_tip"
                | "big_dripleaf_side"
                | "big_dripleaf_stem"
        )
    {
        return 2;
    }
    0
}

/// Packs `(sprite_name, layer_index)` pairs into the `256 x vec4<u32>` uniform
/// lookup table consumed by `NativeMaterial`'s vertex shader.
pub fn pack_sprite_layer_kinds<'a>(
    sprites: impl IntoIterator<Item = (&'a str, u32)>,
) -> [[u32; 4]; LAYER_KINDS_VECS] {
    let mut table = [[0u32; 4]; LAYER_KINDS_VECS];
    let max_layers = (LAYER_KINDS_VECS * 32) as u32;
    for (name, layer) in sprites {
        if layer >= max_layers {
            continue;
        }
        let kind = u32::from(classify_sprite_layer(name) & 0x0F);
        if kind == 0 {
            continue;
        }
        let vec_idx = (layer >> 5) as usize;
        let comp_idx = ((layer >> 3) & 3) as usize;
        let shift = (layer & 7) * 4;
        table[vec_idx][comp_idx] |= kind << shift;
    }
    table
}

/// Builds the in-memory `Pack` for Volt Shaders (`with_post == false` omits `[[post]]`).
pub fn build_pack(with_post: bool) -> Result<Pack, String> {
    let mut manifest = String::from(MANIFEST_HEADER);
    if with_post {
        manifest.push_str(MANIFEST_POST_STAGE);
    }
    manifest.push_str(MANIFEST_OPTIONS);

    let mut files = BTreeMap::new();
    files.insert(MANIFEST.to_owned(), manifest.into_bytes());
    files.insert("common.wgsl".to_owned(), COMMON_WGSL.as_bytes().to_vec());
    files.insert("sky.wgsl".to_owned(), SKY_WGSL.as_bytes().to_vec());
    files.insert("material.wgsl".to_owned(), MATERIAL_WGSL.as_bytes().to_vec());
    if with_post {
        files.insert("post.wgsl".to_owned(), POST_WGSL.as_bytes().to_vec());
    }
    Pack::from_files(files)
}

const MANIFEST_HEADER: &str = r#"schema = 1
id = "volt_shaders"
name = "Volt Shaders"
version = "1.0.0"
api = "0.1"
description = "Mobile-first native WGSL shader engine with moving vegetation, clear water, dynamic lighting and zero-shadow-pass performance."
authors = ["Volt"]
shared = ["common.wgsl"]

[budget]
gpu_ms = 1.5

[sky]
file = "sky.wgsl"
name = "Volt Sky"

[material]
file = "material.wgsl"
name = "Volt Terrain"
"#;

const MANIFEST_POST_STAGE: &str = r#"
[[post]]
file = "post.wgsl"
name = "Volt Post"
"#;

const MANIFEST_OPTIONS: &str = r#"
[[options]]
id = "post_active"
name = "Post Stage Active"
type = "bool"
default = false

[[options]]
id = "sun_shading"
name = "Sun Angle Shading"
type = "bool"
default = true

[[options]]
id = "shadow_richness"
name = "Shadow Richness"
type = "float"
default = 0.55
min = 0.0
max = 1.0
step = 0.05

[[options]]
id = "foliage_subsurface"
name = "Foliage Subsurface Glow"
type = "bool"
default = true

[[options]]
id = "torch_style"
name = "Torchlight Color"
type = "choice"
values = ["vanilla", "warm_amber", "cozy_gold", "soul_cool"]
default = "warm_amber"

[[options]]
id = "torch_intensity"
name = "Torchlight Intensity"
type = "float"
default = 1.25
min = 0.5
max = 2.0
step = 0.05

[[options]]
id = "emissive_glow"
name = "Emissive Block Glow"
type = "float"
default = 0.85
min = 0.0
max = 2.0
step = 0.05

[[options]]
id = "rain_wetness"
name = "Rain Wet Reflections"
type = "bool"
default = true

[[options]]
id = "water_ripples"
name = "Water Surface Ripples"
type = "bool"
default = true

[[options]]
id = "water_reflection"
name = "Water Sky Reflection"
type = "float"
default = 0.80
min = 0.0
max = 1.5
step = 0.05

[[options]]
id = "water_glint"
name = "Water Sun Glint"
type = "bool"
default = true

[[options]]
id = "clear_water"
name = "Clear Water"
type = "bool"
default = true

[[options]]
id = "underwater_refraction"
name = "Underwater Refraction"
type = "bool"
default = false

[[options]]
id = "custom_sky"
name = "Custom Sky Gradient"
type = "bool"
default = true

[[options]]
id = "sunset_glow"
name = "Sunrise & Sunset Glow"
type = "float"
default = 1.10
min = 0.0
max = 2.0
step = 0.05

[[options]]
id = "sun_halo"
name = "Sun & Moon Halo"
type = "bool"
default = true

[[options]]
id = "aurora_mode"
name = "Night Aurora"
type = "choice"
values = ["off", "subtle", "medium", "vibrant"]
default = "subtle"

[[options]]
id = "depth_haze"
name = "Atmospheric Depth Haze"
type = "choice"
values = ["off", "subtle", "lush"]
default = "off"

[[options]]
id = "tonemap_mode"
name = "Tonemapper"
type = "choice"
values = ["off", "filmic_aces", "crisp_neutral", "lush_cinema"]
default = "filmic_aces"

[[options]]
id = "bloom_mode"
name = "Bloom"
type = "choice"
values = ["off", "low", "high"]
default = "off"

[[options]]
id = "sun_rays"
name = "Sun Rays"
type = "bool"
default = false

[[options]]
id = "vibrance"
name = "Vibrance"
type = "float"
default = 1.20
min = 0.5
max = 1.8
step = 0.05

[[options]]
id = "saturation"
name = "Saturation"
type = "float"
default = 1.04
min = 0.5
max = 1.8
step = 0.02

[[options]]
id = "contrast"
name = "Contrast"
type = "float"
default = 1.05
min = 0.7
max = 1.5
step = 0.02

[[options]]
id = "exposure"
name = "Exposure"
type = "float"
default = 1.02
min = 0.7
max = 1.5
step = 0.02

[[options]]
id = "warmth"
name = "Color Warmth"
type = "float"
default = 0.05
min = -0.5
max = 0.5
step = 0.05

[[options]]
id = "vignette_mode"
name = "Vignette"
type = "choice"
values = ["off", "subtle", "strong"]
default = "off"
"#;

const COMMON_WGSL: &str = r#"
override post_active: bool;
override sun_shading: bool;
override shadow_richness: f32;
override foliage_subsurface: bool;
override torch_style: u32;
override torch_intensity: f32;
override emissive_glow: f32;
override rain_wetness: bool;
override water_ripples: bool;
override water_reflection: f32;
override water_glint: bool;
override clear_water: bool;
override underwater_refraction: bool;
override custom_sky: bool;
override sunset_glow: f32;
override sun_halo: bool;
override aurora_mode: u32;
override depth_haze: u32;
override tonemap_mode: u32;
override bloom_mode: u32;
override sun_rays: bool;
override vibrance: f32;
override saturation: f32;
override contrast: f32;
override exposure: f32;
override warmth: f32;
override vignette_mode: u32;

fn volt_luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn volt_aces(x: vec3<f32>) -> vec3<f32> {
    let c = max(x, vec3<f32>(0.0));
    let num = c * (2.51 * c + vec3<f32>(0.03));
    let den = c * (2.43 * c + vec3<f32>(0.59)) + vec3<f32>(0.14);
    return clamp(num / den, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn volt_neutral(x: vec3<f32>) -> vec3<f32> {
    let c = max(x, vec3<f32>(0.0));
    let num = c * (0.35 * c + vec3<f32>(0.92));
    let den = c * (0.35 * c + vec3<f32>(0.90)) + vec3<f32>(0.09);
    return clamp(num / den, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn volt_cinema(x: vec3<f32>) -> vec3<f32> {
    let c = max(x * 1.05, vec3<f32>(0.0));
    let num = c * (2.62 * c + vec3<f32>(0.04));
    let den = c * (2.48 * c + vec3<f32>(0.53)) + vec3<f32>(0.13);
    return clamp(num / den, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn volt_grade(hdr: vec3<f32>) -> vec3<f32> {
    let wb = vec3<f32>(1.0 + warmth * 0.16, 1.0 + warmth * 0.02, 1.0 - warmth * 0.18);
    var c = max(hdr * exposure * wb, vec3<f32>(0.0));
    if tonemap_mode == 1u {
        c = volt_aces(c);
    } else if tonemap_mode == 2u {
        c = volt_neutral(c);
    } else if tonemap_mode == 3u {
        c = volt_cinema(c);
    } else {
        c = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    c = (c - vec3<f32>(0.42)) * contrast + vec3<f32>(0.42);
    c = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let luma = volt_luma(c);
    let hi = max(c.r, max(c.g, c.b));
    let lo = min(c.r, min(c.g, c.b));
    let vib = (vibrance - 1.0) * (1.0 - (hi - lo));
    let sat = max(saturation + vib, 0.0);
    c = mix(vec3<f32>(luma), c, sat);
    return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn volt_torch_rgb() -> vec3<f32> {
    if torch_style == 0u {
        return vec3<f32>(1.00, 0.91, 0.78);
    }
    if torch_style == 2u {
        return vec3<f32>(1.14, 0.79, 0.42);
    }
    if torch_style == 3u {
        return vec3<f32>(0.54, 0.89, 1.12);
    }
    return vec3<f32>(1.10, 0.68, 0.33);
}
"#;

const SKY_WGSL: &str = r#"
fn sky(input: SkyInput) -> vec4<f32> {
    let dir = input.direction / max(length(input.direction), 0.0001);
    var col = input.color.rgb;

    if frame.dimension == 0 {
        let up = clamp(dir.y, 0.0, 1.0);
        let horizon = 1.0 - smoothstep(0.0, 0.45, abs(dir.y));
        let sun_y = frame.sun_direction.y;
        let day = clamp(sun_y * 2.2 + 0.25, 0.0, 1.0);
        let night = clamp(-sun_y * 2.4, 0.0, 1.0);
        let storm = clamp(frame.rain * 0.75 + frame.thunder * 0.25, 0.0, 1.0);

        if custom_sky {
            let day_zenith = vec3<f32>(0.20, 0.46, 0.92);
            let day_horizon = vec3<f32>(0.62, 0.80, 0.99);
            let night_zenith = vec3<f32>(0.015, 0.028, 0.075);
            let night_horizon = vec3<f32>(0.055, 0.085, 0.165);
            let target_zenith = mix(night_zenith, day_zenith, day);
            let target_horizon = mix(night_horizon, day_horizon, day);
            let curve = pow(max(up, 0.0001), 0.62);
            let grad = mix(target_horizon, target_zenith, curve);
            let sky_mask = smoothstep(-0.08, 0.14, dir.y);
            col = mix(col, mix(col, grad, 0.55), sky_mask * (1.0 - storm * 0.65));
            if storm > 0.01 {
                let storm_luma = max(volt_luma(col), 0.04);
                let slate = vec3<f32>(0.42, 0.47, 0.55) * storm_luma * 1.45;
                col = mix(col, slate, storm * 0.55);
            }
        }

        if sunset_glow > 0.001 {
            let twilight = clamp(1.0 - abs(sun_y - 0.04) * 3.1, 0.0, 1.0) * (1.0 - storm * 0.80);
            if twilight > 0.001 {
                let sun_align = clamp(dot(dir, frame.sun_direction) * 0.5 + 0.5, 0.0, 1.0);
                let band = pow(max(horizon, 0.0001), 1.6) * pow(max(sun_align, 0.0001), 2.2);
                let warm_band = mix(vec3<f32>(0.92, 0.34, 0.24), vec3<f32>(1.05, 0.68, 0.26), sun_align);
                col += warm_band * (band * twilight * sunset_glow * 0.48);
            }
        }

        if sun_halo && storm < 0.92 {
            let sun_cos = clamp(dot(dir, frame.sun_direction), 0.0, 1.0);
            let moon_cos = clamp(dot(dir, frame.moon_direction), 0.0, 1.0);
            let sun_vis = clamp(sun_y * 4.0 + 0.35, 0.0, 1.0) * (1.0 - storm * 0.85);
            let moon_vis = night * (1.0 - storm * 0.85);
            let sun_corona = pow(max(sun_cos, 0.0001), 18.0) * 0.24 + pow(max(sun_cos, 0.0001), 96.0) * 0.32;
            let moon_corona = pow(max(moon_cos, 0.0001), 24.0) * 0.12;
            let sun_tint = mix(vec3<f32>(1.05, 0.58, 0.25), vec3<f32>(1.05, 0.95, 0.78), day);
            col += sun_tint * (sun_corona * sun_vis);
            col += vec3<f32>(0.55, 0.72, 1.00) * (moon_corona * moon_vis);
        }

        if aurora_mode > 0u && dir.y > 0.03 {
            let aurora_vis = night * (1.0 - storm * 0.85) * smoothstep(0.03, 0.28, dir.y) * (1.0 - smoothstep(0.62, 0.98, dir.y));
            if aurora_vis > 0.002 {
                let p = dir.xz / (dir.y + 0.22);
                let t = frame.time * 0.18;
                let w1 = sin(p.x * 2.4 + t + sin(p.y * 1.7 - t * 0.8) * 1.4);
                let w2 = sin(p.y * 3.1 - t * 1.15 + cos(p.x * 1.9 + t * 0.6) * 1.2);
                let ribbon = clamp((w1 * 0.55 + w2 * 0.45) * 0.5 + 0.5, 0.0, 1.0);
                let curtain = pow(max(ribbon, 0.0001), 4.5);
                let shift = clamp(sin(p.x * 0.9 - p.y * 0.7 + t * 0.5) * 0.5 + 0.5, 0.0, 1.0);
                let aurora_col = mix(vec3<f32>(0.10, 0.92, 0.56), vec3<f32>(0.46, 0.28, 0.96), shift);
                var strength = 0.22;
                if aurora_mode == 2u {
                    strength = 0.38;
                } else if aurora_mode == 3u {
                    strength = 0.58;
                }
                col += aurora_col * (curtain * aurora_vis * strength);
            }
        }
    }

    if !post_active {
        col = volt_grade(col);
    }
    return vec4<f32>(col, 1.0);
}
"#;

const MATERIAL_WGSL: &str = r#"
fn material(input: MaterialInput) -> Surface {
    let tag = input.tint.a;
    let is_water = input.translucent && tag < 0.375;
    let is_foliage = !is_water && tag < 0.625;
    let is_emissive = !is_water && !is_foliage && tag < 0.875;

    let dist = max(length(input.position), 0.001);
    let view_dir = input.position / dist;
    let cam_mod = frame.camera_position - floor(frame.camera_position * (1.0 / 64.0)) * 64.0;
    let wp = input.position + cam_mod;

    let block_l = input.light.x;
    let sky_l = input.light.y;
    let sun_y = frame.sun_direction.y;
    let day = select(0.0, clamp(sun_y * 2.2 + 0.15, 0.0, 1.0), frame.dimension == 0);
    let storm = clamp(frame.rain * 0.75 + frame.thunder * 0.25, 0.0, 1.0);
    let sunny = day * (1.0 - storm * 0.75);

    var lit = input.lightmap;

    if shadow_richness > 0.001 && frame.dimension == 0 {
        let shade_luma = volt_luma(input.tint.rgb);
        let in_shadow = clamp((1.0 - sky_l * 0.85) * 0.55 + (1.0 - shade_luma) * 0.45, 0.0, 1.0) * (1.0 - block_l * 0.75);
        let cool_shadow = vec3<f32>(0.84, 0.91, 1.06);
        let warm_sun = vec3<f32>(1.06, 1.01, 0.94);
        lit *= mix(warm_sun, cool_shadow, in_shadow * shadow_richness);
    }

    if sun_shading && frame.dimension == 0 && !is_water {
        let phase = dot(view_dir, frame.sun_direction) * 0.5 + 0.5;
        let sun_reach = smoothstep(0.45, 0.95, sky_l) * sunny;
        let dir_boost = 1.0 + (phase - 0.45) * 0.14 * sun_reach;
        lit *= dir_boost;
    }

    if block_l > 0.02 {
        let curve = block_l * block_l * (0.60 + 0.40 * block_l);
        let daylight_damp = 1.0 - sunny * sky_l * 0.55;
        let strength = max(torch_intensity - 0.45, 0.0) * 0.55 * daylight_damp;
        lit += volt_torch_rgb() * (curve * strength);
    }

    var rgb = input.albedo.rgb * input.tint.rgb * lit;
    var alpha = input.albedo.a;

    if foliage_subsurface && is_foliage && sunny > 0.02 && sky_l > 0.40 {
        let forward_sun = clamp(dot(view_dir, frame.sun_direction), 0.0, 1.0);
        let sss = pow(max(forward_sun, 0.0001), 2.5) * smoothstep(0.40, 0.95, sky_l) * sunny;
        rgb += input.albedo.rgb * input.tint.rgb * vec3<f32>(0.52, 0.58, 0.18) * (sss * 0.65);
    }

    if is_emissive && emissive_glow > 0.001 {
        let tex_luma = volt_luma(input.albedo.rgb);
        let hot = smoothstep(0.25, 0.85, tex_luma);
        rgb *= 1.0 + hot * emissive_glow * 0.75;
    }

    if rain_wetness && frame.dimension == 0 && frame.rain > 0.01 && !is_water && !is_foliage && sky_l > 0.68 {
        let outdoor = smoothstep(0.68, 0.96, sky_l) * frame.rain;
        let puddle = clamp(sin(wp.x * 0.7853982 + sin(wp.z * 0.3926991)) * cos(wp.z * 0.7853982) * 0.5 + 0.55, 0.0, 1.0);
        let wet = outdoor * (0.45 + 0.55 * puddle);
        rgb *= 1.0 - wet * 0.18;
        let grazing = pow(max(1.0 - abs(view_dir.y), 0.0001), 3.0);
        let wet_sky = max(frame.sky_color * 0.85 + frame.fog_color * 0.35, vec3<f32>(0.08));
        rgb = mix(rgb, wet_sky, clamp(grazing * wet * 0.36, 0.0, 0.45));
    }

    if is_water {
        var n = vec3<f32>(0.0, 1.0, 0.0);
        if water_ripples {
            let t = frame.time;
            let nx = cos(wp.x * 1.5707963 + t * 2.1 + wp.z * 0.7853982) * 0.085
                + sin(wp.x * 3.1415927 - t * 1.55 - wp.z * 1.5707963) * 0.045;
            let nz = sin(wp.z * 1.5707963 + t * 1.85 - wp.x * 0.7853982) * 0.085
                + cos(wp.z * 3.1415927 + t * 1.35 + wp.x * 1.5707963) * 0.045;
            let fade = clamp(1.0 - dist * (1.0 / 72.0), 0.15, 1.0);
            let raw_n = vec3<f32>(nx * fade, 1.0, nz * fade);
            n = raw_n / max(length(raw_n), 0.0001);
        }

        if clear_water {
            let deep_tint = mix(input.tint.rgb, vec3<f32>(0.12, 0.52, 0.68), 0.38);
            rgb = mix(rgb, deep_tint * lit * 1.08, 0.45);
            alpha = clamp(alpha * 0.74, 0.26, 0.80);
        }

        let cos_theta = abs(dot(view_dir, n));
        let fresnel = pow(max(1.0 - cos_theta, 0.0001), 3.4);

        if water_reflection > 0.001 {
            let sky_access = clamp(sky_l * 1.05, 0.08, 1.0);
            let twilight = select(0.0, clamp(1.0 - abs(sun_y - 0.04) * 3.0, 0.0, 1.0) * (1.0 - storm * 0.8), frame.dimension == 0);
            var refl_col = mix(frame.fog_color, frame.sky_color * 1.15 + vec3<f32>(0.04, 0.07, 0.12), 0.65);
            refl_col += vec3<f32>(0.95, 0.52, 0.24) * (twilight * 0.35);
            let refl_w = clamp(fresnel * water_reflection * sky_access * 0.78, 0.0, 0.82);
            rgb = mix(rgb, refl_col, refl_w);
            alpha = clamp(alpha + refl_w * 0.32, 0.0, 0.94);
        }

        if water_glint && frame.dimension == 0 && storm < 0.85 && sky_l > 0.55 {
            let light_dir = select(frame.moon_direction, frame.sun_direction, sun_y > -0.02);
            let glint_tint = select(vec3<f32>(0.62, 0.78, 1.05) * 0.45, vec3<f32>(1.15, 1.02, 0.82), sun_y > -0.02);
            let to_eye = -view_dir;
            let h_raw = to_eye + light_dir;
            let h_len = length(h_raw);
            if h_len > 0.001 && light_dir.y > 0.01 {
                let h = h_raw / h_len;
                let spec = pow(max(dot(n, h), 0.0001), 56.0) * smoothstep(0.55, 0.92, sky_l) * (1.0 - storm * 0.85);
                rgb += glint_tint * (spec * 0.85);
                alpha = clamp(alpha + spec * 0.35, 0.0, 0.96);
            }
        }
    }

    if !post_active {
        rgb = volt_grade(rgb);
    }
    return Surface(vec4<f32>(rgb, alpha));
}
"#;

const POST_WGSL: &str = r#"
fn volt_bright(c: vec3<f32>) -> vec3<f32> {
    let l = volt_luma(c);
    let knee = smoothstep(0.68, 1.15, l);
    return c * knee;
}

fn post(input: PostInput) -> vec4<f32> {
    var uv = input.uv;
    var rgb = input.color.rgb;
    let texel = vec2<f32>(1.0) / max(frame.resolution, vec2<f32>(1.0));

    if underwater_refraction && frame.eye_in == 1u {
        let t = frame.time;
        let wave_x = sin(uv.y * 22.0 + t * 2.4) * 0.0022 + cos(uv.x * 16.0 - t * 1.8) * 0.0012;
        let wave_y = cos(uv.x * 20.0 + t * 2.1) * 0.0020;
        uv = clamp(uv + vec2<f32>(wave_x, wave_y), vec2<f32>(0.002), vec2<f32>(0.998));
        rgb = textureSampleLevel(scene_color, linear_sampler, uv, 0.0).rgb;
        rgb = mix(rgb * vec3<f32>(0.76, 0.96, 1.04), vec3<f32>(0.04, 0.24, 0.34), 0.16);
    }

    if bloom_mode > 0u {
        let r1 = texel * 2.0;
        var glow = volt_bright(textureSampleLevel(scene_color, linear_sampler, clamp(uv + vec2<f32>(r1.x, r1.y), vec2<f32>(0.001), vec2<f32>(0.999)), 0.0).rgb);
        glow += volt_bright(textureSampleLevel(scene_color, linear_sampler, clamp(uv + vec2<f32>(-r1.x, r1.y), vec2<f32>(0.001), vec2<f32>(0.999)), 0.0).rgb);
        glow += volt_bright(textureSampleLevel(scene_color, linear_sampler, clamp(uv + vec2<f32>(r1.x, -r1.y), vec2<f32>(0.001), vec2<f32>(0.999)), 0.0).rgb);
        glow += volt_bright(textureSampleLevel(scene_color, linear_sampler, clamp(uv + vec2<f32>(-r1.x, -r1.y), vec2<f32>(0.001), vec2<f32>(0.999)), 0.0).rgb);

        if bloom_mode >= 2u {
            let r2 = texel * 6.0;
            var outer = volt_bright(textureSampleLevel(scene_color, linear_sampler, clamp(uv + vec2<f32>(r2.x, r2.y), vec2<f32>(0.001), vec2<f32>(0.999)), 0.0).rgb);
            outer += volt_bright(textureSampleLevel(scene_color, linear_sampler, clamp(uv + vec2<f32>(-r2.x, r2.y), vec2<f32>(0.001), vec2<f32>(0.999)), 0.0).rgb);
            outer += volt_bright(textureSampleLevel(scene_color, linear_sampler, clamp(uv + vec2<f32>(r2.x, -r2.y), vec2<f32>(0.001), vec2<f32>(0.999)), 0.0).rgb);
            outer += volt_bright(textureSampleLevel(scene_color, linear_sampler, clamp(uv + vec2<f32>(-r2.x, -r2.y), vec2<f32>(0.001), vec2<f32>(0.999)), 0.0).rgb);
            rgb += (glow * 0.14 + outer * 0.16) * 0.36;
        } else {
            rgb += glow * (0.25 * 0.28);
        }
    }

    if depth_haze > 0u && input.depth > 0.0 && frame.eye_in == 0u && frame.dimension == 0 {
        let world_pos = view_position(uv, input.depth);
        let dist = length(world_pos);
        let span = max(frame.render_distance, 32.0);
        let start = span * select(0.32, 0.20, depth_haze >= 2u);
        let amount = select(0.22, 0.36, depth_haze >= 2u);
        let haze_t = smoothstep(start, span * 0.96, dist) * amount;
        let view_dir = world_pos / max(dist, 0.001);
        let sun_scatter = clamp(dot(view_dir, frame.sun_direction) * 0.5 + 0.5, 0.0, 1.0)
            * clamp(frame.sun_direction.y * 2.5 + 0.3, 0.0, 1.0)
            * (1.0 - frame.rain * 0.8);
        let haze_col = mix(frame.fog_color, vec3<f32>(1.02, 0.86, 0.66), sun_scatter * 0.28);
        rgb = mix(rgb, haze_col, haze_t);
    }

    if sun_rays && frame.dimension == 0 && frame.eye_in == 0u && frame.sun_direction.y > -0.06 && frame.rain < 0.85 {
        let sun_view = frame.view * vec4<f32>(frame.sun_direction * 64.0, 1.0);
        let sun_clip = frame.projection * sun_view;
        if sun_clip.w > 0.001 {
            let sun_ndc = sun_clip.xy / sun_clip.w;
            let sun_uv = vec2<f32>(sun_ndc.x * 0.5 + 0.5, 0.5 - sun_ndc.y * 0.5);
            let aspect = frame.resolution.x / max(frame.resolution.y, 1.0);
            let delta = sun_uv - uv;
            let radial = length(vec2<f32>(delta.x * aspect, delta.y));
            if radial < 0.95 {
                let falloff = pow(max(1.0 - radial / 0.95, 0.0001), 2.2);
                let step_uv = delta * 0.22;
                var occ = 0.0;
                let uv1 = clamp(uv + step_uv * 1.0, vec2<f32>(0.001), vec2<f32>(0.999));
                let uv2 = clamp(uv + step_uv * 2.0, vec2<f32>(0.001), vec2<f32>(0.999));
                let uv3 = clamp(uv + step_uv * 3.0, vec2<f32>(0.001), vec2<f32>(0.999));
                let uv4 = clamp(uv + step_uv * 4.0, vec2<f32>(0.001), vec2<f32>(0.999));
                occ += select(0.0, 0.25, depth_at(uv1) == 0.0);
                occ += select(0.0, 0.25, depth_at(uv2) == 0.0);
                occ += select(0.0, 0.25, depth_at(uv3) == 0.0);
                occ += select(0.0, 0.25, depth_at(uv4) == 0.0);
                let vis = clamp(frame.sun_direction.y * 3.0 + 0.25, 0.0, 1.0) * (1.0 - frame.rain * 0.85);
                let shaft_col = mix(vec3<f32>(1.05, 0.62, 0.28), vec3<f32>(1.05, 0.95, 0.78), clamp(frame.sun_direction.y * 2.0, 0.0, 1.0));
                rgb += shaft_col * (occ * falloff * vis * 0.22);
            }
        }
    }

    rgb = volt_grade(rgb);

    if vignette_mode > 0u {
        let centered = uv * 2.0 - vec2<f32>(1.0);
        let d2 = dot(centered, centered);
        let strength = select(0.14, 0.28, vignette_mode >= 2u);
        rgb *= clamp(1.0 - d2 * strength, 0.35, 1.0);
    }

    return vec4<f32>(rgb, 1.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::check;

    #[test]
    fn default_volt_shaders_config_is_disabled() {
        let cfg = VoltShadersConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.profile, VoltShaderProfile::Medium);
    }

    #[test]
    fn zero_post_tbdr_pack_passes_full_multi_backend_shader_check() {
        let pack = build_pack(false).expect("zero-post Volt Shaders pack builds");
        let report = check(&pack).unwrap_or_else(|problems| {
            panic!(
                "zero-post Volt Shaders pack failed check: {}",
                problems
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        });
        assert_eq!(report.stages.len(), 2);
    }

    #[test]
    fn single_pass_post_pack_passes_full_multi_backend_shader_check() {
        let pack = build_pack(true).expect("post Volt Shaders pack builds");
        let report = check(&pack).unwrap_or_else(|problems| {
            panic!(
                "post Volt Shaders pack failed check: {}",
                problems
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        });
        assert_eq!(report.stages.len(), 3);
    }

    #[test]
    fn profiles_separate_zero_post_tbdr_from_high_ultra_post_and_switch_to_custom() {
        for p in [
            VoltShaderProfile::Potato,
            VoltShaderProfile::Low,
            VoltShaderProfile::Medium,
        ] {
            let cfg = VoltShadersConfig::preset(p);
            assert!(
                !cfg.needs_post_pass(),
                "{p:?} must stay in zero-post TBDR mode"
            );
            assert!(cfg.build_runtime().is_ok());
        }
        for p in [VoltShaderProfile::High, VoltShaderProfile::Ultra] {
            let cfg = VoltShadersConfig::preset(p);
            assert!(cfg.needs_post_pass(), "{p:?} enables single-pass post");
            assert!(cfg.build_runtime().is_ok());
        }
        let mut cfg = VoltShadersConfig::preset(VoltShaderProfile::Low);
        cfg.mark_custom();
        assert_eq!(cfg.profile, VoltShaderProfile::Custom);
    }

    #[test]
    fn sprite_layer_classifier_packs_vegetation_and_water_nibbles() {
        assert_eq!(classify_sprite_layer("minecraft:block/oak_leaves"), 1);
        assert_eq!(classify_sprite_layer("minecraft:block/short_grass"), 2);
        assert_eq!(classify_sprite_layer("minecraft:block/wheat_stage7"), 2);
        assert_eq!(classify_sprite_layer("minecraft:block/tall_grass_top"), 3);
        assert_eq!(classify_sprite_layer("minecraft:block/hanging_roots"), 4);
        assert_eq!(classify_sprite_layer("minecraft:block/water_still"), 5);
        assert_eq!(classify_sprite_layer("minecraft:block/water_flow"), 6);
        assert_eq!(classify_sprite_layer("minecraft:block/stone"), 0);
        assert_eq!(classify_sprite_layer("minecraft:block/grass_block_top"), 0);

        let table = pack_sprite_layer_kinds([
            ("minecraft:block/oak_leaves", 0),
            ("minecraft:block/short_grass", 9),
            ("minecraft:block/water_still", 35),
        ]);
        assert_eq!(table[0][0] & 0xF, 1);
        assert_eq!((table[0][1] >> 4) & 0xF, 2);
        assert_eq!((table[1][0] >> 12) & 0xF, 5);
    }
}
