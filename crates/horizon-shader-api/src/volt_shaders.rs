//! Built-in mobile-first native WGSL "Volt Shaders" engine (BSL + Complementary class).
//!
//! Designed specifically for tile-based mobile Vulkan GPUs (Arm Mali-G57,
//! Adreno, Xclipse):
//! - Zero shadow-map geometry re-draws and zero `PackAtlas` texture copies.
//! - Every toggle and mode compiles into SPIR-V specialization `override`
//!   constants so disabled effects cost zero ALU/texture instructions.
//! - Integrated Auto-PBR (`classify_sprite_layer`) gives Complementary-style
//!   selective Glowing Ores, metallic/gem/ice specular glints, crystal glass
//!   reflections, BSL/Complementary volumetric & voxel clouds, cloud ground
//!   shadows, water shoreline foam & caustics, Milky Way & shooting stars,
//!   post-rain rainbows, and eye-comfort highlight rolloff with zero external
//!   resource packs required.
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

/// Master aesthetic look inspired by BSL Shaders and Complementary Reimagined.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShaderVisualStyle {
    #[default]
    BslSoftCinema,
    ComplementaryCrisp,
    VoltBalanced,
    PastelDream,
}

impl ShaderVisualStyle {
    pub const ALL: [Self; 4] = [
        Self::BslSoftCinema,
        Self::ComplementaryCrisp,
        Self::VoltBalanced,
        Self::PastelDream,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BslSoftCinema => "bsl_soft",
            Self::ComplementaryCrisp => "complementary_crisp",
            Self::VoltBalanced => "volt_balanced",
            Self::PastelDream => "pastel_dream",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BslSoftCinema => "BSL Soft",
            Self::ComplementaryCrisp => "Complementary",
            Self::VoltBalanced => "Volt Balanced",
            Self::PastelDream => "Pastel Dream",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::BslSoftCinema => 0,
            Self::ComplementaryCrisp => 1,
            Self::VoltBalanced => 2,
            Self::PastelDream => 3,
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

/// Procedural sky cloud rendering mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CloudMode {
    Off,
    #[default]
    BslVolumetric,
    ComplementaryVoxel,
    CirrusWisps,
}

impl CloudMode {
    pub const ALL: [Self; 4] = [
        Self::Off,
        Self::BslVolumetric,
        Self::ComplementaryVoxel,
        Self::CirrusWisps,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::BslVolumetric => "bsl_volumetric",
            Self::ComplementaryVoxel => "complementary_voxel",
            Self::CirrusWisps => "cirrus_wisps",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::BslVolumetric => "BSL Puffy 3D",
            Self::ComplementaryVoxel => "Voxel Reimagined",
            Self::CirrusWisps => "Soft Cirrus",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::BslVolumetric => 1,
            Self::ComplementaryVoxel => 2,
            Self::CirrusWisps => 3,
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
    /// Overall BSL / Complementary aesthetic palette & shading curve.
    pub style: ShaderVisualStyle,
    // --- Wind & Moving Vegetation ---
    pub waving_plants: bool,
    pub waving_leaves: bool,
    pub waving_water: bool,
    /// Percentage `25..=200` (100 = 1.0x).
    pub wind_speed: u16,
    /// Percentage `25..=200` (100 = 1.0x).
    pub wind_strength: u16,
    pub storm_wind: bool,
    // --- Lighting, Integrated Auto-PBR & Shadows ---
    pub sun_shading: bool,
    /// Percentage `0..=100`.
    pub shadow_richness: u16,
    pub cloud_shadows: bool,
    pub foliage_subsurface: bool,
    pub glowing_ores: bool,
    /// Percentage `25..=200` (100 = 1.0x).
    pub ore_glow: u16,
    pub auto_pbr: bool,
    pub torch_style: TorchLightStyle,
    /// Percentage `50..=200` (100 = 1.0x).
    pub torch_intensity: u16,
    /// Percentage `0..=200` (100 = 1.0x).
    pub emissive_glow: u16,
    pub rain_wetness: bool,
    // --- Water ---
    pub water_ripples: bool,
    pub water_foam: bool,
    /// Percentage `0..=150` (100 = 1.0x).
    pub water_reflection: u16,
    pub water_glint: bool,
    pub clear_water: bool,
    pub underwater_refraction: bool,
    // --- Sky, Volumetric/Voxel Clouds & Atmosphere ---
    pub custom_sky: bool,
    pub clouds_mode: CloudMode,
    /// Percentage `30..=160` (100 = balanced cloud density).
    pub cloud_coverage: u16,
    /// Percentage `0..=200` (100 = 1.0x).
    pub sunset_glow: u16,
    pub sun_halo: bool,
    pub aurora: AuroraMode,
    pub night_sky_plus: bool,
    pub post_rain_rainbow: bool,
    pub depth_haze: DepthHazeMode,
    // --- Eye-Comfort, Post-Processing & Color Grading ---
    pub eye_comfort: bool,
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
                style: ShaderVisualStyle::BslSoftCinema,
                waving_plants: true,
                waving_leaves: false,
                waving_water: false,
                wind_speed: 100,
                wind_strength: 85,
                storm_wind: false,
                sun_shading: true,
                shadow_richness: 30,
                cloud_shadows: false,
                foliage_subsurface: false,
                glowing_ores: true,
                ore_glow: 85,
                auto_pbr: false,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 108,
                emissive_glow: 40,
                rain_wetness: false,
                water_ripples: false,
                water_foam: false,
                water_reflection: 30,
                water_glint: false,
                clear_water: true,
                underwater_refraction: false,
                custom_sky: true,
                clouds_mode: CloudMode::CirrusWisps,
                cloud_coverage: 85,
                sunset_glow: 75,
                sun_halo: false,
                aurora: AuroraMode::Off,
                night_sky_plus: false,
                post_rain_rainbow: false,
                depth_haze: DepthHazeMode::Off,
                eye_comfort: true,
                tonemap: TonemapMode::CrispNeutral,
                bloom: BloomMode::Off,
                sun_rays: false,
                vibrance: 112,
                saturation: 101,
                contrast: 102,
                exposure: 101,
                warmth: 2,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::Low => Self {
                enabled: true,
                profile: VoltShaderProfile::Low,
                style: ShaderVisualStyle::BslSoftCinema,
                waving_plants: true,
                waving_leaves: true,
                waving_water: false,
                wind_speed: 100,
                wind_strength: 95,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 45,
                cloud_shadows: false,
                foliage_subsurface: true,
                glowing_ores: true,
                ore_glow: 95,
                auto_pbr: false,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 118,
                emissive_glow: 65,
                rain_wetness: false,
                water_ripples: true,
                water_foam: true,
                water_reflection: 60,
                water_glint: true,
                clear_water: true,
                underwater_refraction: false,
                custom_sky: true,
                clouds_mode: CloudMode::BslVolumetric,
                cloud_coverage: 95,
                sunset_glow: 95,
                sun_halo: true,
                aurora: AuroraMode::Off,
                night_sky_plus: true,
                post_rain_rainbow: false,
                depth_haze: DepthHazeMode::Off,
                eye_comfort: true,
                tonemap: TonemapMode::FilmicAces,
                bloom: BloomMode::Off,
                sun_rays: false,
                vibrance: 116,
                saturation: 103,
                contrast: 103,
                exposure: 101,
                warmth: 4,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::Medium | VoltShaderProfile::Custom => Self {
                enabled: true,
                profile,
                style: ShaderVisualStyle::BslSoftCinema,
                waving_plants: true,
                waving_leaves: true,
                waving_water: true,
                wind_speed: 100,
                wind_strength: 100,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 58,
                cloud_shadows: true,
                foliage_subsurface: true,
                glowing_ores: true,
                ore_glow: 110,
                auto_pbr: true,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 126,
                emissive_glow: 90,
                rain_wetness: true,
                water_ripples: true,
                water_foam: true,
                water_reflection: 85,
                water_glint: true,
                clear_water: true,
                underwater_refraction: false,
                custom_sky: true,
                clouds_mode: CloudMode::BslVolumetric,
                cloud_coverage: 100,
                sunset_glow: 115,
                sun_halo: true,
                aurora: AuroraMode::Subtle,
                night_sky_plus: true,
                post_rain_rainbow: true,
                depth_haze: DepthHazeMode::Off,
                eye_comfort: true,
                tonemap: TonemapMode::FilmicAces,
                bloom: BloomMode::Off,
                sun_rays: false,
                vibrance: 120,
                saturation: 104,
                contrast: 104,
                exposure: 102,
                warmth: 5,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::High => Self {
                enabled: true,
                profile: VoltShaderProfile::High,
                style: ShaderVisualStyle::ComplementaryCrisp,
                waving_plants: true,
                waving_leaves: true,
                waving_water: true,
                wind_speed: 100,
                wind_strength: 105,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 68,
                cloud_shadows: true,
                foliage_subsurface: true,
                glowing_ores: true,
                ore_glow: 125,
                auto_pbr: true,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 132,
                emissive_glow: 105,
                rain_wetness: true,
                water_ripples: true,
                water_foam: true,
                water_reflection: 100,
                water_glint: true,
                clear_water: true,
                underwater_refraction: true,
                custom_sky: true,
                clouds_mode: CloudMode::ComplementaryVoxel,
                cloud_coverage: 105,
                sunset_glow: 130,
                sun_halo: true,
                aurora: AuroraMode::Medium,
                night_sky_plus: true,
                post_rain_rainbow: true,
                depth_haze: DepthHazeMode::Subtle,
                eye_comfort: true,
                tonemap: TonemapMode::FilmicAces,
                bloom: BloomMode::Low,
                sun_rays: false,
                vibrance: 124,
                saturation: 105,
                contrast: 105,
                exposure: 103,
                warmth: 6,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::Ultra => Self {
                enabled: true,
                profile: VoltShaderProfile::Ultra,
                style: ShaderVisualStyle::BslSoftCinema,
                waving_plants: true,
                waving_leaves: true,
                waving_water: true,
                wind_speed: 100,
                wind_strength: 110,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 76,
                cloud_shadows: true,
                foliage_subsurface: true,
                glowing_ores: true,
                ore_glow: 135,
                auto_pbr: true,
                torch_style: TorchLightStyle::CozyGold,
                torch_intensity: 140,
                emissive_glow: 120,
                rain_wetness: true,
                water_ripples: true,
                water_foam: true,
                water_reflection: 118,
                water_glint: true,
                clear_water: true,
                underwater_refraction: true,
                custom_sky: true,
                clouds_mode: CloudMode::BslVolumetric,
                cloud_coverage: 110,
                sunset_glow: 145,
                sun_halo: true,
                aurora: AuroraMode::Vibrant,
                night_sky_plus: true,
                post_rain_rainbow: true,
                depth_haze: DepthHazeMode::Lush,
                eye_comfort: true,
                tonemap: TonemapMode::LushCinema,
                bloom: BloomMode::High,
                sun_rays: true,
                vibrance: 126,
                saturation: 106,
                contrast: 106,
                exposure: 104,
                warmth: 7,
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
            ("shader_style".into(), f64::from(self.style.index())),
            ("sun_shading".into(), b(self.sun_shading)),
            ("shadow_richness".into(), pct(self.shadow_richness, 0, 100)),
            ("cloud_shadows".into(), b(self.cloud_shadows)),
            ("foliage_subsurface".into(), b(self.foliage_subsurface)),
            ("glowing_ores".into(), b(self.glowing_ores)),
            ("ore_glow".into(), pct(self.ore_glow, 25, 200)),
            ("auto_pbr".into(), b(self.auto_pbr)),
            ("torch_style".into(), f64::from(self.torch_style.index())),
            ("torch_intensity".into(), pct(self.torch_intensity, 50, 200)),
            ("emissive_glow".into(), pct(self.emissive_glow, 0, 200)),
            ("rain_wetness".into(), b(self.rain_wetness)),
            ("water_ripples".into(), b(self.water_ripples)),
            ("water_foam".into(), b(self.water_foam)),
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
            ("clouds_mode".into(), f64::from(self.clouds_mode.index())),
            ("cloud_coverage".into(), pct(self.cloud_coverage, 30, 160)),
            ("sunset_glow".into(), pct(self.sunset_glow, 0, 200)),
            ("sun_halo".into(), b(self.sun_halo)),
            ("aurora_mode".into(), f64::from(self.aurora.index())),
            ("night_sky_plus".into(), b(self.night_sky_plus)),
            ("post_rain_rainbow".into(), b(self.post_rain_rainbow)),
            ("depth_haze".into(), f64::from(self.depth_haze.index())),
            ("eye_comfort".into(), b(self.eye_comfort)),
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
        let report = prepare_trusted(&pack).map_err(|problems| {
            problems
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        Ok((report, self.pipeline_constants()))
    }
}

/// Number of `vec4<u32>` entries in the uniform table holding 4-bit sprite
/// material/wave classes (`256 * 32 = 8,192` block sprite layers supported).
pub const LAYER_KINDS_VECS: usize = 256;

/// Classifies a block sprite name (e.g. `"minecraft:block/short_grass"`) into
/// its 4-bit Volt Shaders wind & Integrated Auto-PBR material class:
/// - `0`: static matte block
/// - `1`: canopy / leaves / vines / multi-block stems (3D sway, foliage SSS)
/// - `2`: grounded plant single/lower (root-anchored sway, foliage SSS)
/// - `3`: 2-block tall plant upper half (tip sway continuing lower half, foliage SSS)
/// - `4`: ceiling-hanging plant (ceiling-anchored sway, foliage SSS)
/// - `5`: still water surface (3D water wave + crystal water + foam/caustics)
/// - `6`: flowing water / water overlay (crystal water + foam/caustics)
/// - `7`: ore / crystal vein block (Complementary selective gem/vein emission + glint)
/// - `8`: metallic / gem / smooth ice / polished block (Auto-PBR specular & Fresnel)
/// - `9`: translucent crystal / stained glass (BSL sky reflection + crisp specular)
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

    // Class 7: Complementary-style Glowing Ores & Crystal Clusters
    if (name.ends_with("_ore") && !name.contains("coal"))
        || matches!(
            name,
            "gilded_blackstone"
                | "ancient_debris_side"
                | "ancient_debris_top"
                | "amethyst_cluster"
                | "large_amethyst_bud"
                | "medium_amethyst_bud"
                | "small_amethyst_bud"
                | "budding_amethyst"
        )
    {
        return 7;
    }

    // Class 9: Translucent Crystal / Stained Glass
    if name == "glass"
        || name == "tinted_glass"
        || name.ends_with("_stained_glass")
        || name.ends_with("glass_pane_top")
    {
        return 9;
    }

    // Class 8: Integrated Auto-PBR Metals, Gems, Ice & Polished Stone
    if name.starts_with("polished_")
        || name.contains("copper")
        || matches!(
            name,
            "iron_block"
                | "gold_block"
                | "diamond_block"
                | "emerald_block"
                | "lapis_block"
                | "redstone_block"
                | "netherite_block"
                | "amethyst_block"
                | "quartz_block_top"
                | "quartz_block_side"
                | "quartz_block_bottom"
                | "quartz_bricks"
                | "quartz_pillar"
                | "quartz_pillar_top"
                | "chiseled_quartz_block"
                | "chiseled_quartz_block_top"
                | "ice"
                | "packed_ice"
                | "blue_ice"
                | "frosted_ice_0"
                | "frosted_ice_1"
                | "frosted_ice_2"
                | "frosted_ice_3"
                | "obsidian"
                | "crying_obsidian"
                | "anvil"
                | "anvil_top"
                | "cauldron_side"
                | "chain"
                | "iron_bars"
                | "iron_door_top"
                | "iron_door_bottom"
                | "iron_trapdoor"
                | "heavy_core"
        )
    {
        return 8;
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
version = "2.0.0"
api = "0.1"
description = "BSL & Complementary-class mobile-first native WGSL shader engine with 3D clouds, glowing ores, auto-PBR, water foam & zero-lag TBDR performance."
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
id = "shader_style"
name = "Visual Style"
type = "choice"
values = ["bsl_soft", "complementary_crisp", "volt_balanced", "pastel_dream"]
default = "bsl_soft"

[[options]]
id = "sun_shading"
name = "Sun Angle Shading"
type = "bool"
default = true

[[options]]
id = "shadow_richness"
name = "Shadow Richness"
type = "float"
default = 0.58
min = 0.0
max = 1.0
step = 0.05

[[options]]
id = "cloud_shadows"
name = "Cloud Ground Shadows"
type = "bool"
default = true

[[options]]
id = "foliage_subsurface"
name = "Foliage Subsurface Glow"
type = "bool"
default = true

[[options]]
id = "glowing_ores"
name = "Glowing Ores"
type = "bool"
default = true

[[options]]
id = "ore_glow"
name = "Ore Glow Strength"
type = "float"
default = 1.10
min = 0.25
max = 2.0
step = 0.05

[[options]]
id = "auto_pbr"
name = "Integrated Auto-PBR"
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
default = 1.26
min = 0.5
max = 2.0
step = 0.05

[[options]]
id = "emissive_glow"
name = "Emissive Block Glow"
type = "float"
default = 0.90
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
id = "water_foam"
name = "Water Foam & Caustics"
type = "bool"
default = true

[[options]]
id = "water_reflection"
name = "Water Sky Reflection"
type = "float"
default = 0.85
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
id = "clouds_mode"
name = "3D Sky Clouds"
type = "choice"
values = ["off", "bsl_volumetric", "complementary_voxel", "cirrus_wisps"]
default = "bsl_volumetric"

[[options]]
id = "cloud_coverage"
name = "Cloud Coverage"
type = "float"
default = 1.00
min = 0.30
max = 1.60
step = 0.05

[[options]]
id = "sunset_glow"
name = "Sunrise & Sunset Glow"
type = "float"
default = 1.15
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
id = "night_sky_plus"
name = "Milky Way & Shooting Stars"
type = "bool"
default = true

[[options]]
id = "post_rain_rainbow"
name = "Post-Rain Rainbow"
type = "bool"
default = true

[[options]]
id = "depth_haze"
name = "Atmospheric Depth Haze"
type = "choice"
values = ["off", "subtle", "lush"]
default = "off"

[[options]]
id = "eye_comfort"
name = "Eye-Comfort Highlight Rolloff"
type = "bool"
default = true

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
default = 1.04
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
override shader_style: u32;
override sun_shading: bool;
override shadow_richness: f32;
override cloud_shadows: bool;
override foliage_subsurface: bool;
override glowing_ores: bool;
override ore_glow: f32;
override auto_pbr: bool;
override torch_style: u32;
override torch_intensity: f32;
override emissive_glow: f32;
override rain_wetness: bool;
override water_ripples: bool;
override water_foam: bool;
override water_reflection: f32;
override water_glint: bool;
override clear_water: bool;
override underwater_refraction: bool;
override custom_sky: bool;
override clouds_mode: u32;
override cloud_coverage: f32;
override sunset_glow: f32;
override sun_halo: bool;
override aurora_mode: u32;
override night_sky_plus: bool;
override post_rain_rainbow: bool;
override depth_haze: u32;
override eye_comfort: bool;
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

fn volt_hash21(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(127.1, 311.7));
    let d = dot(q, q + vec2<f32>(34.23));
    return fract((q.x + q.y) * d);
}

fn volt_noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (vec2<f32>(3.0) - 2.0 * f);
    let a = volt_hash21(i);
    let b = volt_hash21(i + vec2<f32>(1.0, 0.0));
    let c = volt_hash21(i + vec2<f32>(0.0, 1.0));
    let d = volt_hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn volt_fbm3(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    var v = 0.52 * volt_noise2(p);
    p = vec2<f32>(p.x * 1.62 - p.y * 1.18, p.x * 1.18 + p.y * 1.62);
    v += 0.31 * volt_noise2(p);
    p = vec2<f32>(p.x * 1.68 + p.y * 1.12, -p.x * 1.12 + p.y * 1.68);
    v += 0.17 * volt_noise2(p);
    return v;
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
    let c = max(x * 1.04, vec3<f32>(0.0));
    let num = c * (2.58 * c + vec3<f32>(0.045));
    let den = c * (2.46 * c + vec3<f32>(0.56)) + vec3<f32>(0.135);
    return clamp(num / den, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn volt_grade(hdr: vec3<f32>) -> vec3<f32> {
    var style_wb = vec3<f32>(1.0, 1.0, 1.0);
    if shader_style == 0u {
        style_wb = vec3<f32>(1.03, 1.005, 0.985);
    } else if shader_style == 1u {
        style_wb = vec3<f32>(1.01, 1.01, 1.02);
    } else if shader_style == 3u {
        style_wb = vec3<f32>(1.04, 0.995, 1.03);
    }
    let wb = vec3<f32>(1.0 + warmth * 0.16, 1.0 + warmth * 0.02, 1.0 - warmth * 0.18) * style_wb;
    var c = max(hdr * exposure * wb, vec3<f32>(0.0));

    // BSL Eye-Comfort soft highlight shoulder & velvet shadow lift (zero harsh glare):
    if eye_comfort {
        c = c / (vec3<f32>(1.0) + c * 0.18) * 1.14;
        c += vec3<f32>(0.0045, 0.0050, 0.0068) * (1.0 - clamp(volt_luma(c) * 3.0, 0.0, 1.0));
    }

    if tonemap_mode == 1u {
        c = volt_aces(c);
    } else if tonemap_mode == 2u {
        c = volt_neutral(c);
    } else if tonemap_mode == 3u {
        c = volt_cinema(c);
    } else {
        c = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    let eff_contrast = select(contrast, contrast * 1.03, shader_style == 1u);
    c = (c - vec3<f32>(0.42)) * eff_contrast + vec3<f32>(0.42);
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
        return vec3<f32>(1.15, 0.80, 0.43);
    }
    if torch_style == 3u {
        return vec3<f32>(0.54, 0.89, 1.12);
    }
    return vec3<f32>(1.11, 0.69, 0.34);
}
"#;

const SKY_WGSL: &str = r#"
fn sky(input: SkyInput) -> vec4<f32> {
    let dir = input.direction / max(length(input.direction), 0.0001);
    var col = input.color.rgb;

    if frame.dimension == 0 {
        let up = clamp(dir.y, 0.0, 1.0);
        let horizon = 1.0 - smoothstep(0.0, 0.46, abs(dir.y));
        let sun_y = frame.sun_direction.y;
        let day = clamp(sun_y * 2.2 + 0.25, 0.0, 1.0);
        let night = clamp(-sun_y * 2.4, 0.0, 1.0);
        let storm = clamp(frame.rain * 0.75 + frame.thunder * 0.25, 0.0, 1.0);
        let twilight = clamp(1.0 - abs(sun_y - 0.03) * 3.0, 0.0, 1.0) * (1.0 - storm * 0.78);

        if custom_sky {
            var day_zenith = vec3<f32>(0.18, 0.44, 0.90);
            var day_horizon = vec3<f32>(0.64, 0.82, 0.99);
            var night_zenith = vec3<f32>(0.014, 0.026, 0.072);
            var night_horizon = vec3<f32>(0.052, 0.084, 0.168);
            if shader_style == 1u {
                // Complementary Reimagined deep sapphire zenith & crisp azure horizon
                day_zenith = vec3<f32>(0.13, 0.37, 0.86);
                day_horizon = vec3<f32>(0.56, 0.78, 0.98);
                night_zenith = vec3<f32>(0.010, 0.020, 0.058);
            } else if shader_style == 3u {
                // Pastel Dream soft periwinkle-rose sky
                day_zenith = vec3<f32>(0.30, 0.52, 0.94);
                day_horizon = vec3<f32>(0.76, 0.84, 0.99);
                night_horizon = vec3<f32>(0.085, 0.075, 0.175);
            }
            let target_zenith = mix(night_zenith, day_zenith, day);
            let target_horizon = mix(night_horizon, day_horizon, day);
            let curve = pow(max(up, 0.0001), 0.58);
            let grad = mix(target_horizon, target_zenith, curve);
            let sky_mask = smoothstep(-0.08, 0.14, dir.y);
            col = mix(col, mix(col, grad, 0.68), sky_mask * (1.0 - storm * 0.65));
            if storm > 0.01 {
                let storm_luma = max(volt_luma(col), 0.04);
                let slate = vec3<f32>(0.42, 0.47, 0.55) * storm_luma * 1.45;
                col = mix(col, slate, storm * 0.55);
            }
        }

        // Night Milky Way band & graceful shooting stars
        if night_sky_plus && night > 0.08 && dir.y > 0.04 && storm < 0.75 {
            let sky_vis = night * (1.0 - storm * 0.85) * smoothstep(0.04, 0.26, dir.y);
            let gal_axis = dir.x * 0.64 + dir.z * 0.52 - dir.y * 0.28;
            let gal_band = exp(-gal_axis * gal_axis * 11.0);
            let dust = volt_fbm3(dir.xz * 4.2 + vec2<f32>(dir.y * 2.1, 1.7));
            let nebula = gal_band * smoothstep(0.32, 0.78, dust);
            let mw_col = mix(vec3<f32>(0.07, 0.11, 0.24), vec3<f32>(0.18, 0.13, 0.28), dust);
            col += mw_col * (nebula * sky_vis * 0.55);

            // Occasional subtle shooting star streak every ~6 seconds
            let cycle = floor(frame.time * 0.16);
            let phase = fract(frame.time * 0.16);
            if phase < 0.18 {
                let seed = volt_hash21(vec2<f32>(cycle, cycle * 0.37 + 4.1));
                let ang = seed * 6.2831853;
                let center = vec2<f32>(cos(ang), sin(ang)) * 0.35 + vec2<f32>(phase * 1.9 - 0.17, phase * 1.15 - 0.10);
                let proj = dir.xz / (dir.y + 0.35) * 0.42;
                let delta = proj - center;
                let along = delta.x * 0.85 + delta.y * 0.52;
                let perp = -delta.x * 0.52 + delta.y * 0.85;
                let streak = exp(-perp * perp * 2600.0) * smoothstep(-0.14, 0.0, along) * (1.0 - smoothstep(0.0, 0.025, along));
                let fade = sin(phase / 0.18 * 3.1415927);
                col += vec3<f32>(0.85, 0.93, 1.10) * (streak * fade * sky_vis * 0.85);
            }
        }

        if sunset_glow > 0.001 && twilight > 0.001 {
            let sun_align = clamp(dot(dir, frame.sun_direction) * 0.5 + 0.5, 0.0, 1.0);
            let band = pow(max(horizon, 0.0001), 1.55) * pow(max(sun_align, 0.0001), 2.0);
            let warm_lo = select(vec3<f32>(0.94, 0.36, 0.26), vec3<f32>(0.96, 0.46, 0.56), shader_style == 3u);
            let warm_hi = vec3<f32>(1.06, 0.72, 0.30);
            let warm_band = mix(warm_lo, warm_hi, sun_align);
            col += warm_band * (band * twilight * sunset_glow * 0.52);
        }

        // BSL Soft Volumetric 3D Clouds / Complementary Reimagined Voxel Clouds / Cirrus
        if clouds_mode > 0u && dir.y > 0.015 {
            let dome_y = dir.y + 0.13;
            let wind = vec2<f32>(frame.time * 0.016, frame.time * 0.007);
            let cam_drift = frame.camera_position.xz * 0.00055;
            let base_uv = (dir.xz / dome_y) * 0.62 + wind + cam_drift;
            let sun_dot = clamp(dot(dir, frame.sun_direction), -1.0, 1.0);
            let mie = pow(max(sun_dot * 0.5 + 0.5, 0.0001), 5.0) * (1.0 - storm * 0.7);
            let sky_rim_fade = smoothstep(0.015, 0.17, dir.y);
            let cov_shift = (cloud_coverage - 1.0) * 0.26 + storm * 0.18;

            var density = 0.0;
            var lit_top = 0.5;
            if clouds_mode == 1u {
                // BSL Soft Volumetric multi-layer puffy clouds
                let n_base = volt_fbm3(base_uv * 1.35);
                let n_detail = volt_noise2(base_uv * 4.4 - wind * 1.6);
                let raw = n_base * 0.78 + n_detail * 0.22 + cov_shift;
                density = smoothstep(0.50, 0.78, raw);
                let sun_step = volt_fbm3((base_uv + frame.sun_direction.xz * 0.09) * 1.35);
                lit_top = clamp(0.55 + (raw - sun_step) * 2.6, 0.15, 1.0);
            } else if clouds_mode == 2u {
                // Complementary Reimagined crisp 3D voxel-stepped clouds
                let vox = floor(base_uv * 9.0) / 9.0;
                let cell = volt_fbm3(vox * 1.25) + cov_shift;
                let neighbor = volt_fbm3((vox + sign(frame.sun_direction.xz + vec2<f32>(0.001)) / 9.0) * 1.25) + cov_shift;
                density = smoothstep(0.53, 0.59, cell);
                let sub_uv = fract(base_uv * 9.0);
                let bevel = smoothstep(0.0, 0.16, min(min(sub_uv.x, 1.0 - sub_uv.x), min(sub_uv.y, 1.0 - sub_uv.y)));
                density *= 0.88 + 0.12 * bevel;
                lit_top = clamp(0.62 + (cell - neighbor) * 3.2, 0.18, 1.0);
            } else {
                // High-altitude feathery Cirrus Wisps
                let warp = volt_noise2(base_uv * 1.6);
                let cirrus_streak = volt_fbm3(vec2<f32>(base_uv.x * 2.8 + warp * 0.9, base_uv.y * 0.85));
                density = smoothstep(0.54, 0.86, cirrus_streak + cov_shift * 0.6) * 0.55;
                lit_top = 0.78;
            }

            let alpha_c = clamp(density * sky_rim_fade * 0.90, 0.0, 0.92);
            if alpha_c > 0.002 {
                let day_bright = vec3<f32>(1.03, 1.02, 1.01);
                let day_shade = vec3<f32>(0.62, 0.71, 0.86);
                let night_bright = vec3<f32>(0.10, 0.14, 0.24);
                let night_shade = vec3<f32>(0.04, 0.06, 0.12);
                var c_top = mix(night_bright, day_bright, day);
                var c_bot = mix(night_shade, day_shade, day);
                let sunset_tint = vec3<f32>(1.08, 0.56, 0.32) * (twilight * 0.75);
                c_top += sunset_tint;
                c_bot = mix(c_bot, vec3<f32>(0.48, 0.28, 0.42), twilight * 0.55);
                var cloud_rgb = mix(c_bot, c_top, lit_top);
                cloud_rgb += vec3<f32>(1.06, 0.92, 0.72) * (mie * (1.0 - density * 0.55) * (day * 0.42 + twilight * 0.55));
                if storm > 0.01 {
                    cloud_rgb = mix(cloud_rgb, vec3<f32>(0.34, 0.38, 0.45) * (0.35 + 0.65 * day), storm * 0.72);
                }
                col = mix(col, cloud_rgb, alpha_c);
            }
        }

        // Optical 42-degree Post-Rain / Sun-Shower Rainbow Arc
        if post_rain_rainbow && dir.y > 0.01 && day > 0.22 && frame.rain > 0.02 && frame.rain < 0.72 {
            let anti_sun = normalize(vec3<f32>(-frame.sun_direction.x, -frame.sun_direction.y, -frame.sun_direction.z));
            let ca = dot(dir, anti_sun);
            let ring = (ca - 0.743) * 26.0;
            if abs(ring) < 1.35 {
                let mask = exp(-ring * ring * 2.4) * smoothstep(0.01, 0.18, dir.y) * sin(clamp(frame.rain / 0.72, 0.0, 1.0) * 3.1415927);
                let t_rb = clamp(ring * 0.5 + 0.5, 0.0, 1.0);
                let rb = vec3<f32>(
                    smoothstep(0.45, 0.95, t_rb),
                    sin(t_rb * 3.1415927),
                    smoothstep(0.55, 0.05, t_rb)
                );
                col += rb * (mask * day * 0.18);
            }
        }

        if sun_halo && storm < 0.92 {
            let sun_cos = clamp(dot(dir, frame.sun_direction), 0.0, 1.0);
            let moon_cos = clamp(dot(dir, frame.moon_direction), 0.0, 1.0);
            let sun_vis = clamp(sun_y * 4.0 + 0.35, 0.0, 1.0) * (1.0 - storm * 0.85);
            let moon_vis = night * (1.0 - storm * 0.85);
            let sun_corona = pow(max(sun_cos, 0.0001), 16.0) * 0.22 + pow(max(sun_cos, 0.0001), 96.0) * 0.30;
            let moon_corona = pow(max(moon_cos, 0.0001), 24.0) * 0.13;
            let sun_tint = mix(vec3<f32>(1.05, 0.58, 0.25), vec3<f32>(1.05, 0.95, 0.80), day);
            col += sun_tint * (sun_corona * sun_vis);
            col += vec3<f32>(0.56, 0.74, 1.02) * (moon_corona * moon_vis);
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
                let aurora_col = mix(vec3<f32>(0.10, 0.94, 0.58), vec3<f32>(0.48, 0.30, 0.98), shift);
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
    let is_water = input.translucent && tag < 0.20;
    let is_glass = input.translucent && tag >= 0.20 && tag < 0.36;
    let is_foliage = !is_water && !is_glass && tag >= 0.36 && tag < 0.52;
    let is_ore = !is_water && !is_glass && tag >= 0.52 && tag < 0.68;
    let is_emissive = !is_water && !is_glass && tag >= 0.68 && tag < 0.84;
    let is_pbr_metal = !is_water && !is_glass && tag >= 0.84 && tag < 0.95;

    let dist = max(length(input.position), 0.001);
    let view_dir = input.position / dist;
    let cam_mod = frame.camera_position - floor(frame.camera_position * (1.0 / 64.0)) * 64.0;
    let wp = input.position + cam_mod;
    let world_xz = input.position.xz + frame.camera_position.xz;

    let block_l = input.light.x;
    let sky_l = input.light.y;
    let sun_y = frame.sun_direction.y;
    let day = select(0.0, clamp(sun_y * 2.2 + 0.15, 0.0, 1.0), frame.dimension == 0);
    let storm = clamp(frame.rain * 0.75 + frame.thunder * 0.25, 0.0, 1.0);
    let sunny = day * (1.0 - storm * 0.75);

    var lit = input.lightmap;

    // BSL / Complementary Rich Skylight & Cool Bounce Shadows
    if shadow_richness > 0.001 && frame.dimension == 0 {
        let shade_luma = volt_luma(input.tint.rgb);
        let in_shadow = clamp((1.0 - sky_l * 0.85) * 0.55 + (1.0 - shade_luma) * 0.45, 0.0, 1.0) * (1.0 - block_l * 0.75);
        let cool_shadow = select(vec3<f32>(0.83, 0.91, 1.07), vec3<f32>(0.80, 0.89, 1.10), shader_style == 1u);
        let warm_sun = select(vec3<f32>(1.07, 1.015, 0.94), vec3<f32>(1.06, 1.00, 0.98), shader_style == 3u);
        lit *= mix(warm_sun, cool_shadow, in_shadow * shadow_richness);
    }

    // Drifting Cloud Ground Shadows across sunlit landscapes
    var cloud_sun = 1.0;
    if cloud_shadows && frame.dimension == 0 && sunny > 0.04 && sky_l > 0.55 {
        let c_uv = (world_xz - frame.sun_direction.xz * 22.0) * 0.011 + vec2<f32>(frame.time * 0.016, frame.time * 0.007);
        let c_n = volt_fbm3(c_uv);
        let cloud_patch = smoothstep(0.47, 0.72, c_n + (cloud_coverage - 1.0) * 0.20);
        let outdoor_mask = smoothstep(0.55, 0.94, sky_l) * sunny * (1.0 - block_l * 0.65);
        cloud_sun = 1.0 - cloud_patch * outdoor_mask * 0.22;
        lit *= mix(vec3<f32>(1.0), vec3<f32>(0.78, 0.84, 0.95), cloud_patch * outdoor_mask * 0.75);
    }

    if sun_shading && frame.dimension == 0 && !is_water {
        let phase = dot(view_dir, frame.sun_direction) * 0.5 + 0.5;
        let sun_reach = smoothstep(0.45, 0.95, sky_l) * sunny * cloud_sun;
        let dir_boost = 1.0 + (phase - 0.45) * 0.15 * sun_reach;
        lit *= dir_boost;
    }

    if block_l > 0.02 {
        let curve = block_l * block_l * (0.58 + 0.42 * block_l);
        let daylight_damp = 1.0 - sunny * sky_l * 0.55;
        let strength = max(torch_intensity - 0.45, 0.0) * 0.58 * daylight_damp;
        lit += volt_torch_rgb() * (curve * strength);
    }

    var rgb = input.albedo.rgb * input.tint.rgb * lit;
    var alpha = input.albedo.a;

    // BSL / Complementary Warm Foliage Subsurface Scattering
    if foliage_subsurface && is_foliage && sunny > 0.02 && sky_l > 0.38 {
        let forward_sun = clamp(dot(view_dir, frame.sun_direction), 0.0, 1.0);
        let sss = (0.18 + 0.82 * pow(max(forward_sun, 0.0001), 2.3)) * smoothstep(0.38, 0.95, sky_l) * sunny * cloud_sun;
        rgb += input.albedo.rgb * input.tint.rgb * vec3<f32>(0.54, 0.60, 0.18) * (sss * 0.62);
    }

    // Complementary Reimagined Selective Glowing Ores (only colored gem/metal veins emit!)
    if glowing_ores && is_ore {
        let hi = max(input.albedo.r, max(input.albedo.g, input.albedo.b));
        let lo = min(input.albedo.r, min(input.albedo.g, input.albedo.b));
        let chroma = hi - lo;
        let tex_luma = volt_luma(input.albedo.rgb);
        let vein_mask = clamp(smoothstep(0.11, 0.30, chroma) + smoothstep(0.56, 0.80, tex_luma) * 0.75, 0.0, 1.0);
        if vein_mask > 0.01 {
            let cave_boost = 1.15 - sky_l * 0.35;
            let boost_tint = normalize(input.albedo.rgb + vec3<f32>(0.08)) * max(hi, 0.45);
            rgb = mix(rgb, boost_tint * (0.95 + ore_glow * 0.65 * cave_boost), vein_mask * clamp(ore_glow * 0.68, 0.0, 0.92));
        }
    }

    if is_emissive && emissive_glow > 0.001 {
        let emit_luma = volt_luma(input.albedo.rgb);
        let hot = smoothstep(0.24, 0.84, emit_luma);
        rgb *= 1.0 + hot * emissive_glow * 0.78;
    }

    // Integrated Auto-PBR Specular & Fresnel on Metals, Gems, Ice, Polished Blocks & Crystal Glass
    if auto_pbr && (is_pbr_metal || is_glass || is_ore) && dist < 56.0 {
        let fade_pbr = clamp(1.0 - dist * (1.0 / 56.0), 0.0, 1.0);
        let pbr_grazing = pow(max(1.0 - abs(view_dir.y), 0.0001), 3.2);
        let env_col = max(frame.sky_color * 0.90 + frame.fog_color * 0.30, vec3<f32>(0.06, 0.07, 0.10));
        if is_glass {
            let glass_refl = clamp((0.12 + pbr_grazing * 0.55) * fade_pbr * max(sky_l, 0.25), 0.0, 0.52);
            rgb = mix(rgb, env_col * 1.15, glass_refl);
            alpha = clamp(alpha + glass_refl * 0.28, 0.0, 0.92);
        } else if is_pbr_metal {
            let tex_gloss = smoothstep(0.25, 0.82, volt_luma(input.albedo.rgb));
            let sheen = pbr_grazing * tex_gloss * fade_pbr * 0.26 * max(sky_l, block_l * 0.7);
            rgb = mix(rgb, mix(env_col, input.albedo.rgb * 1.35, 0.55), sheen);
        }
        if sunny > 0.05 && sky_l > 0.60 {
            let pbr_h_raw = -view_dir + frame.sun_direction;
            let pbr_h_len = length(pbr_h_raw);
            if pbr_h_len > 0.001 {
                let pbr_h = pbr_h_raw / pbr_h_len;
                let spec_up = pow(max(pbr_h.y, 0.0001), 40.0) * sunny * cloud_sun * fade_pbr * smoothstep(0.60, 0.95, sky_l);
                rgb += vec3<f32>(1.10, 1.00, 0.86) * (spec_up * 0.42);
                if is_glass {
                    alpha = clamp(alpha + spec_up * 0.35, 0.0, 0.95);
                }
            }
        }
    }

    if rain_wetness && frame.dimension == 0 && frame.rain > 0.01 && !is_water && !is_glass && !is_foliage && sky_l > 0.68 {
        let outdoor = smoothstep(0.68, 0.96, sky_l) * frame.rain;
        let puddle = clamp(sin(wp.x * 0.7853982 + sin(wp.z * 0.3926991)) * cos(wp.z * 0.7853982) * 0.5 + 0.55, 0.0, 1.0);
        let wet = outdoor * (0.45 + 0.55 * puddle);
        rgb *= 1.0 - wet * 0.18;
        let grazing = pow(max(1.0 - abs(view_dir.y), 0.0001), 3.0);
        let wet_sky = max(frame.sky_color * 0.85 + frame.fog_color * 0.35, vec3<f32>(0.08));
        rgb = mix(rgb, wet_sky, clamp(grazing * wet * 0.38, 0.0, 0.48));
    }

    if is_water {
        var n = vec3<f32>(0.0, 1.0, 0.0);
        let t = frame.time;
        var wave_crest = 0.0;
        if water_ripples {
            let nx = cos(wp.x * 1.5707963 + t * 2.1 + wp.z * 0.7853982) * 0.085
                + sin(wp.x * 3.1415927 - t * 1.55 - wp.z * 1.5707963) * 0.045;
            let nz = sin(wp.z * 1.5707963 + t * 1.85 - wp.x * 0.7853982) * 0.085
                + cos(wp.z * 3.1415927 + t * 1.35 + wp.x * 1.5707963) * 0.045;
            let fade = clamp(1.0 - dist * (1.0 / 72.0), 0.15, 1.0);
            wave_crest = clamp((nx + nz) * 4.2 + 0.35, 0.0, 1.0) * fade;
            let raw_n = vec3<f32>(nx * fade, 1.0, nz * fade);
            n = raw_n / max(length(raw_n), 0.0001);
        }

        if clear_water {
            let water_palette = select(
                vec3<f32>(0.10, 0.52, 0.70),
                vec3<f32>(0.07, 0.56, 0.64),
                shader_style == 1u
            );
            let deep_tint = mix(input.tint.rgb, water_palette, 0.44);
            rgb = mix(rgb, deep_tint * lit * 1.10, 0.48);
            alpha = clamp(alpha * 0.72, 0.24, 0.78);
        }

        // BSL / Complementary Water Caustics & Crest / Shoreline Foam
        if water_foam && dist < 56.0 {
            let fade_f = clamp(1.0 - dist * (1.0 / 56.0), 0.0, 1.0);
            let c1 = sin(world_xz.x * 2.6 + t * 1.9 + cos(world_xz.y * 2.2 - t * 1.4));
            let c2 = cos(world_xz.y * 2.9 - t * 1.7 + sin(world_xz.x * 2.4 + t * 1.5));
            let caustic = pow(max(c1 * c2 * 0.5 + 0.5, 0.0001), 3.2) * fade_f;
            rgb += vec3<f32>(0.16, 0.36, 0.42) * lit * (caustic * 0.36);

            // Edge UV & wave-crest cellular foam
            let cell_u = fract(input.uv.x * 16.0);
            let cell_v = fract(input.uv.y * 16.0);
            let edge_band = max(
                1.0 - smoothstep(0.0, 0.12, min(cell_u, 1.0 - cell_u)),
                1.0 - smoothstep(0.0, 0.12, min(cell_v, 1.0 - cell_v))
            );
            let foam_n = volt_noise2(world_xz * 3.4 + vec2<f32>(t * 0.45, -t * 0.35));
            let crest_foam = smoothstep(0.68, 0.92, foam_n * 0.72 + wave_crest * 0.42 + edge_band * 0.14) * fade_f;
            if crest_foam > 0.01 {
                let foam_rgb = vec3<f32>(0.92, 0.96, 1.00) * max(lit, vec3<f32>(0.15));
                rgb = mix(rgb, foam_rgb, crest_foam * 0.42);
                alpha = clamp(alpha + crest_foam * 0.28, 0.0, 0.94);
            }
        }

        let cos_theta = abs(dot(view_dir, n));
        let fresnel = pow(max(1.0 - cos_theta, 0.0001), 3.4);

        if water_reflection > 0.001 {
            let sky_access = clamp(sky_l * 1.05, 0.08, 1.0);
            let twilight = select(0.0, clamp(1.0 - abs(sun_y - 0.04) * 3.0, 0.0, 1.0) * (1.0 - storm * 0.8), frame.dimension == 0);
            var refl_col = mix(frame.fog_color, frame.sky_color * 1.16 + vec3<f32>(0.04, 0.07, 0.12), 0.68);
            refl_col += vec3<f32>(0.96, 0.54, 0.26) * (twilight * 0.38);
            let refl_w = clamp(fresnel * water_reflection * sky_access * 0.80, 0.0, 0.84);
            rgb = mix(rgb, refl_col, refl_w);
            alpha = clamp(alpha + refl_w * 0.32, 0.0, 0.94);
        }

        if water_glint && frame.dimension == 0 && storm < 0.85 && sky_l > 0.55 {
            let light_dir = select(frame.moon_direction, frame.sun_direction, sun_y > -0.02);
            let glint_tint = select(vec3<f32>(0.64, 0.80, 1.06) * 0.48, vec3<f32>(1.18, 1.04, 0.84), sun_y > -0.02);
            let to_eye = -view_dir;
            let h_raw = to_eye + light_dir;
            let h_len = length(h_raw);
            if h_len > 0.001 && light_dir.y > 0.01 {
                let h = h_raw / h_len;
                let spec = pow(max(dot(n, h), 0.0001), 58.0) * smoothstep(0.55, 0.92, sky_l) * (1.0 - storm * 0.85) * cloud_sun;
                rgb += glint_tint * (spec * 0.88);
                alpha = clamp(alpha + spec * 0.36, 0.0, 0.96);
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
    let knee = smoothstep(0.66, 1.14, l);
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
        assert_eq!(cfg.style, ShaderVisualStyle::BslSoftCinema);
        assert_eq!(cfg.clouds_mode, CloudMode::BslVolumetric);
        assert!(cfg.glowing_ores);
        assert!(cfg.eye_comfort);
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
    fn sprite_layer_classifier_packs_vegetation_water_ores_and_auto_pbr_nibbles() {
        assert_eq!(classify_sprite_layer("minecraft:block/oak_leaves"), 1);
        assert_eq!(classify_sprite_layer("minecraft:block/short_grass"), 2);
        assert_eq!(classify_sprite_layer("minecraft:block/wheat_stage7"), 2);
        assert_eq!(classify_sprite_layer("minecraft:block/tall_grass_top"), 3);
        assert_eq!(classify_sprite_layer("minecraft:block/hanging_roots"), 4);
        assert_eq!(classify_sprite_layer("minecraft:block/water_still"), 5);
        assert_eq!(classify_sprite_layer("minecraft:block/water_flow"), 6);
        assert_eq!(classify_sprite_layer("minecraft:block/diamond_ore"), 7);
        assert_eq!(
            classify_sprite_layer("minecraft:block/deepslate_emerald_ore"),
            7
        );
        assert_eq!(classify_sprite_layer("minecraft:block/iron_block"), 8);
        assert_eq!(classify_sprite_layer("minecraft:block/blue_ice"), 8);
        assert_eq!(classify_sprite_layer("minecraft:block/glass"), 9);
        assert_eq!(
            classify_sprite_layer("minecraft:block/cyan_stained_glass"),
            9
        );
        assert_eq!(classify_sprite_layer("minecraft:block/stone"), 0);
        assert_eq!(classify_sprite_layer("minecraft:block/grass_block_top"), 0);

        let table = pack_sprite_layer_kinds([
            ("minecraft:block/oak_leaves", 0),
            ("minecraft:block/short_grass", 9),
            ("minecraft:block/water_still", 35),
            ("minecraft:block/diamond_ore", 36),
        ]);
        assert_eq!(table[0][0] & 0xF, 1);
        assert_eq!((table[0][1] >> 4) & 0xF, 2);
        assert_eq!((table[1][0] >> 12) & 0xF, 5);
        assert_eq!((table[1][0] >> 16) & 0xF, 7);
    }
}
