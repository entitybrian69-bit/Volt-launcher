//! Built-in 3D Animated **Volt Cosmetics** studio & world renderer (`#7`).
//!
//! Renders 86 pure-Rust procedural 3D animated cosmetics across 7 simultaneous
//! equipment slots (`Capes`, `Wings`, `Crowns & Hats`, `Halos & Horns`,
//! `Ears & Tails`, `Shoulder Pets`, `Auras & Trails`) using `HudBatch::quad_3d`
//! + `HudBatch::depth_since` hardware Z-buffering — zero extra GPU pipelines,
//! zero texture allocations, and zero cost when unequipped.

use crate::hud::{self, HudAtlas, HudBatch, Rect};

const DEG: f32 = std::f32::consts::PI / 180.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CosmeticCategory {
    #[default]
    Capes,
    Wings,
    CrownsHats,
    HalosHorns,
    EarsTails,
    ShoulderPets,
    AurasTrails,
}

impl CosmeticCategory {
    pub const ALL: [Self; 7] = [
        Self::Capes,
        Self::Wings,
        Self::CrownsHats,
        Self::HalosHorns,
        Self::EarsTails,
        Self::ShoulderPets,
        Self::AurasTrails,
    ];

    pub fn index(self) -> usize {
        match self {
            Self::Capes => 0,
            Self::Wings => 1,
            Self::CrownsHats => 2,
            Self::HalosHorns => 3,
            Self::EarsTails => 4,
            Self::ShoulderPets => 5,
            Self::AurasTrails => 6,
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Capes => "Capes",
            Self::Wings => "Wings",
            Self::CrownsHats => "Hats",
            Self::HalosHorns => "Halos",
            Self::EarsTails => "Ears/Tail",
            Self::ShoulderPets => "Pets",
            Self::AurasTrails => "Auras",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Capes => "Animated 3D Capes",
            Self::Wings => "Animated 3D Wings",
            Self::CrownsHats => "Crowns & Headwear",
            Self::HalosHorns => "Halos & Sculpted Horns",
            Self::EarsTails => "Animated Ears & Tails",
            Self::ShoulderPets => "3D Shoulder Companions",
            Self::AurasTrails => "3D Particle Auras & Trails",
        }
    }

    pub fn default_yaw_for_preview(self) -> Option<f32> {
        match self {
            Self::Capes | Self::Wings => Some(158.0),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CosmeticDef {
    pub id: u8,
    pub slug: &'static str,
    pub name: &'static str,
    pub subtitle: &'static str,
    pub primary: [u8; 3],
    pub secondary: [u8; 3],
    pub accent: [u8; 3],
}

pub const CAPES: [CosmeticDef; 14] = [
    CosmeticDef { id: 1, slug: "volt_sovereign", name: "Crimson Volt Sovereign", subtitle: "Royal crimson silk & pulsing gold crest", primary: [186, 22, 38], secondary: [28, 18, 24], accent: [255, 206, 74] },
    CosmeticDef { id: 2, slug: "starlight_nebula", name: "Starlight Nebula", subtitle: "Deep cosmos cloak with drifting star shimmer", primary: [34, 28, 92], secondary: [88, 44, 156], accent: [145, 228, 255] },
    CosmeticDef { id: 3, slug: "sakura_breeze", name: "Sakura Blossom Breeze", subtitle: "Soft cherry-petal gradient & ivory trim", primary: [238, 134, 168], secondary: [252, 218, 230], accent: [255, 246, 250] },
    CosmeticDef { id: 4, slug: "solar_phoenix", name: "Solar Phoenix Mantle", subtitle: "Molten ember weave with rising solar wave", primary: [224, 72, 22], secondary: [128, 22, 18], accent: [255, 220, 78] },
    CosmeticDef { id: 5, slug: "glacial_frost", name: "Frostbite Glacial Cloak", subtitle: "Crystalline sapphire weave & frost pulse", primary: [42, 132, 198], secondary: [22, 58, 112], accent: [196, 246, 255] },
    CosmeticDef { id: 6, slug: "emerald_alchemist", name: "Emerald Alchemist", subtitle: "Verdant rune mantle with gilt filigree", primary: [24, 138, 78], secondary: [14, 58, 38], accent: [244, 212, 92] },
    CosmeticDef { id: 7, slug: "abyssal_void", name: "Abyssal Voidwalker", subtitle: "Obsidian rift cloth with violet core", primary: [20, 16, 34], secondary: [68, 24, 112], accent: [198, 98, 255] },
    CosmeticDef { id: 8, slug: "raijin_storm", name: "Thunderstorm Raijin", subtitle: "Cobalt stormweave with electric arcs", primary: [28, 72, 168], secondary: [18, 28, 64], accent: [120, 238, 255] },
    CosmeticDef { id: 9, slug: "golden_imperator", name: "Golden Imperator", subtitle: "Burnished imperial gold & marble hem", primary: [212, 162, 38], secondary: [240, 232, 212], accent: [255, 238, 148] },
    CosmeticDef { id: 10, slug: "soulfire_phantom", name: "Soulfire Phantom", subtitle: "Spectral turquoise soul-flame mantle", primary: [22, 148, 156], secondary: [18, 44, 56], accent: [132, 255, 248] },
    CosmeticDef { id: 11, slug: "amethyst_geode", name: "Amethyst Geode", subtitle: "Faceted crystal purple & magenta gleam", primary: [132, 56, 188], secondary: [58, 24, 92], accent: [242, 164, 255] },
    CosmeticDef { id: 12, slug: "cyber_synth", name: "Cyber Neon Grid", subtitle: "Carbon synthwave cloak with neon trim", primary: [24, 26, 38], secondary: [218, 36, 138], accent: [64, 244, 255] },
    CosmeticDef { id: 13, slug: "bloodmoon_eclipse", name: "Bloodmoon Eclipse", subtitle: "Midnight charcoal & scarlet eclipse ring", primary: [32, 14, 20], secondary: [152, 18, 34], accent: [255, 92, 92] },
    CosmeticDef { id: 14, slug: "prismatic_aurora", name: "Prismatic Aurora", subtitle: "Iridescent color-cycling northern lights silk", primary: [68, 214, 168], secondary: [118, 84, 236], accent: [255, 168, 228] },
];

pub const WINGS: [CosmeticDef; 12] = [
    CosmeticDef { id: 1, slug: "archangel", name: "Archangel Seraphim", subtitle: "6-feathered ivory & golden celestial wings", primary: [246, 244, 238], secondary: [218, 224, 238], accent: [255, 214, 92] },
    CosmeticDef { id: 2, slug: "crimson_dragon", name: "Crimson Dragonlord", subtitle: "Articulated scarlet & obsidian wyrm wings", primary: [192, 28, 42], secondary: [36, 22, 28], accent: [255, 138, 56] },
    CosmeticDef { id: 3, slug: "ender_wyrm", name: "Ender Void Wyrm", subtitle: "Obsidian dragon spars with glowing ender membrane", primary: [28, 22, 38], secondary: [98, 36, 158], accent: [216, 118, 255] },
    CosmeticDef { id: 4, slug: "valkyrie", name: "Celestial Valkyrie", subtitle: "Silver-azure battle plumes & gilt tips", primary: [198, 226, 252], secondary: [88, 148, 218], accent: [255, 222, 112] },
    CosmeticDef { id: 5, slug: "infernal_demon", name: "Infernal Archdemon", subtitle: "Charred basalt spars with magma veins", primary: [44, 24, 24], secondary: [176, 42, 18], accent: [255, 186, 48] },
    CosmeticDef { id: 6, slug: "monarch_butterfly", name: "Monarch Butterfly", subtitle: "Fluttering amber-gold & velvet wingbeats", primary: [248, 138, 28], secondary: [28, 24, 28], accent: [255, 232, 128] },
    CosmeticDef { id: 7, slug: "lunar_moth", name: "Lunar Actias Moth", subtitle: "Ethereal pale-jade & moonlit silver wings", primary: [158, 238, 198], secondary: [112, 186, 212], accent: [238, 255, 244] },
    CosmeticDef { id: 8, slug: "cyber_thrusters", name: "Cyber Mecha Blades", subtitle: "Hard-light neon energy flight vanes", primary: [36, 42, 58], secondary: [38, 218, 255], accent: [255, 52, 108] },
    CosmeticDef { id: 9, slug: "crystal_fairy", name: "Crystal Sylph", subtitle: "Iridescent rose-quartz & cyan gossamer wings", primary: [246, 158, 224], secondary: [132, 228, 255], accent: [255, 244, 255] },
    CosmeticDef { id: 10, slug: "phoenix_fire", name: "Solar Phoenix Pinions", subtitle: "Blazing layered gold-to-crimson fire plumes", primary: [255, 168, 34], secondary: [222, 48, 24], accent: [255, 242, 138] },
    CosmeticDef { id: 11, slug: "frost_wyvern", name: "Frost Wyvern", subtitle: "Glacial ice-crystal spars & rime mist", primary: [122, 208, 252], secondary: [44, 108, 184], accent: [232, 252, 255] },
    CosmeticDef { id: 12, slug: "shadow_raven", name: "Nightfall Raven", subtitle: "Midnight-indigo & twilight raven feathers", primary: [26, 30, 52], secondary: [58, 46, 96], accent: [142, 128, 228] },
];

pub const CROWNS_HATS: [CosmeticDef; 12] = [
    CosmeticDef { id: 1, slug: "volt_crown", name: "Volt Sovereign Crown", subtitle: "Royal gold spires & levitating crimson ruby", primary: [246, 196, 54], secondary: [198, 26, 46], accent: [255, 242, 156] },
    CosmeticDef { id: 2, slug: "grand_wizard", name: "Grand Astral Wizard Hat", subtitle: "Pointed arcane brim with orbiting starlight", primary: [52, 38, 122], secondary: [238, 192, 68], accent: [152, 234, 255] },
    CosmeticDef { id: 3, slug: "samurai_kabuto", name: "Shogun Kabuto Crest", subtitle: "Lacquered crimson plates & crescent moon", primary: [176, 26, 38], secondary: [28, 24, 30], accent: [248, 206, 74] },
    CosmeticDef { id: 4, slug: "golden_laurel", name: "Olympian Laurel Wreath", subtitle: "Sculpted golden olive leaves around brow", primary: [242, 198, 62], secondary: [188, 142, 32], accent: [255, 244, 168] },
    CosmeticDef { id: 5, slug: "aviator_goggles", name: "Steampunk Sky Goggles", subtitle: "Polished brass frames & glowing cyan lenses", primary: [184, 128, 52], secondary: [68, 44, 28], accent: [96, 242, 255] },
    CosmeticDef { id: 6, slug: "phantom_tophat", name: "Phantom Velvet Top Hat", subtitle: "Tall obsidian silk hat with crimson ribbon", primary: [28, 26, 36], secondary: [196, 32, 54], accent: [244, 240, 248] },
    CosmeticDef { id: 7, slug: "pirate_tricorne", name: "Corsair Tricorne", subtitle: "Captain's tricorn with swaying white plume", primary: [58, 36, 28], secondary: [224, 178, 58], accent: [248, 248, 252] },
    CosmeticDef { id: 8, slug: "astro_visor", name: "Starfarer Visor Helm", subtitle: "Sleek orbital frame & solar-gold visor", primary: [236, 240, 248], secondary: [242, 176, 48], accent: [88, 224, 255] },
    CosmeticDef { id: 9, slug: "toadstool_cap", name: "bouncy Toadstool Cap", subtitle: "Spotted scarlet mushroom cap with soft bounce", primary: [218, 42, 48], secondary: [248, 244, 236], accent: [255, 218, 128] },
    CosmeticDef { id: 10, slug: "ice_tiara", name: "Glacial Monarch Tiara", subtitle: "Frosted diamond spires & floating ice shard", primary: [148, 224, 255], secondary: [64, 142, 218], accent: [240, 252, 255] },
    CosmeticDef { id: 11, slug: "blossom_garland", name: "Sakura Flower Crown", subtitle: "Woven spring vines & pulsing cherry blossoms", primary: [248, 148, 184], secondary: [74, 168, 92], accent: [255, 236, 152] },
    CosmeticDef { id: 12, slug: "cyber_visor", name: "Cyberpunk HUD Visor", subtitle: "Wrap-around holographic crimson-cyan visor", primary: [28, 32, 46], secondary: [255, 42, 92], accent: [64, 246, 255] },
];

pub const HALOS_HORNS: [CosmeticDef; 12] = [
    CosmeticDef { id: 1, slug: "seraph_halo", name: "Seraphic Gold Halo", subtitle: "Levitating 8-segment radiant golden ring", primary: [255, 216, 84], secondary: [242, 168, 38], accent: [255, 248, 196] },
    CosmeticDef { id: 2, slug: "archfiend_horns", name: "Crimson Archfiend Horns", subtitle: "Swept obsidian horns with glowing ember tips", primary: [36, 24, 30], secondary: [198, 32, 44], accent: [255, 146, 58] },
    CosmeticDef { id: 3, slug: "elder_dragon_horns", name: "Elder Dragon Horns", subtitle: "Regal ivory horns banded with gold", primary: [236, 228, 210], secondary: [188, 168, 138], accent: [252, 204, 72] },
    CosmeticDef { id: 4, slug: "eclipse_halo", name: "Solar Eclipse Corona", subtitle: "Dark moon ring rimmed in blazing solar fire", primary: [24, 18, 32], secondary: [244, 118, 32], accent: [255, 218, 98] },
    CosmeticDef { id: 5, slug: "arcane_rune", name: "Arcane Rune Orbit", subtitle: "Counter-rotating dual mystic cyan sigil rings", primary: [64, 218, 255], secondary: [98, 118, 255], accent: [220, 252, 255] },
    CosmeticDef { id: 6, slug: "void_shards", name: "Shattered Void Crown", subtitle: "Orbiting levitating amethyst & ender crystals", primary: [156, 68, 242], secondary: [48, 22, 84], accent: [238, 168, 255] },
    CosmeticDef { id: 7, slug: "soulfire_halo", name: "Soulfire Spirit Ring", subtitle: "Floating turquoise flame halo", primary: [52, 232, 228], secondary: [22, 128, 148], accent: [198, 255, 252] },
    CosmeticDef { id: 8, slug: "ram_warlord", name: "Warlord Curved Horns", subtitle: "Curled dark-bronze mountain ram horns", primary: [92, 68, 52], secondary: [52, 38, 30], accent: [218, 168, 78] },
    CosmeticDef { id: 9, slug: "constellation", name: "Starlight Constellation", subtitle: "5 orbiting celestial stars above head", primary: [148, 214, 255], secondary: [255, 226, 124], accent: [255, 255, 255] },
    CosmeticDef { id: 10, slug: "druid_antlers", name: "Emerald Stag Antlers", subtitle: "Branching heartwood antlers & glowing leaves", primary: [118, 78, 46], secondary: [56, 198, 102], accent: [176, 255, 164] },
    CosmeticDef { id: 11, slug: "chroma_halo", name: "Prismatic Chroma Halo", subtitle: "Smoothly hue-shifting levitating light ring", primary: [255, 96, 188], secondary: [78, 232, 255], accent: [255, 238, 112] },
    CosmeticDef { id: 12, slug: "tesla_arc", name: "Tesla Storm Halo", subtitle: "Crackling electric cobalt & white energy ring", primary: [68, 148, 255], secondary: [188, 238, 255], accent: [255, 255, 255] },
];

pub const EARS_TAILS: [CosmeticDef; 12] = [
    CosmeticDef { id: 1, slug: "kitsune_nine", name: "Nine-Tailed Kitsune", subtitle: "Twitching fox ears & fan of flame-tipped tails", primary: [236, 108, 38], secondary: [250, 244, 236], accent: [255, 208, 92] },
    CosmeticDef { id: 2, slug: "midnight_neko", name: "Midnight Neko", subtitle: "Sleek velvet cat ears & swaying S-curve tail", primary: [32, 28, 42], secondary: [218, 118, 156], accent: [168, 132, 248] },
    CosmeticDef { id: 3, slug: "arctic_wolf", name: "Arctic Timber Wolf", subtitle: "Fluffy silver-white wolf ears & wagging tail", primary: [226, 234, 246], secondary: [142, 158, 182], accent: [188, 226, 255] },
    CosmeticDef { id: 4, slug: "snow_bunny", name: "Snowdrop Bunny", subtitle: "Tall bobbing bunny ears & cotton puff tail", primary: [248, 246, 250], secondary: [246, 168, 192], accent: [255, 224, 236] },
    CosmeticDef { id: 5, slug: "red_panda", name: "Bamboo Red Panda", subtitle: "Rounded russet-cream ears & striped ring tail", primary: [198, 72, 36], secondary: [246, 234, 218], accent: [62, 34, 26] },
    CosmeticDef { id: 6, slug: "pink_axolotl", name: "Lush Pink Axolotl", subtitle: "Fluttering external gill frills & aquatic tail", primary: [248, 156, 198], secondary: [232, 78, 138], accent: [255, 222, 240] },
    CosmeticDef { id: 7, slug: "crimson_wyvern", name: "Crimson Wyvern", subtitle: "Spined dragon fin-ears & spade-tipped tail", primary: [188, 28, 42], secondary: [38, 26, 32], accent: [252, 186, 68] },
    CosmeticDef { id: 8, slug: "cyber_neko", name: "Cyber Synth Neko", subtitle: "Mecha neon cat ears & segmented energy tail", primary: [34, 38, 52], secondary: [52, 238, 255], accent: [255, 58, 132] },
    CosmeticDef { id: 9, slug: "golden_shiba", name: "Golden Shiba Inu", subtitle: "Perky tan ears & happy curled wagging tail", primary: [228, 164, 78], secondary: [248, 242, 228], accent: [186, 118, 46] },
    CosmeticDef { id: 10, slug: "shadow_panther", name: "Obsidian Panther", subtitle: "Stealth panther ears & lithe midnight tail", primary: [24, 24, 32], secondary: [68, 52, 108], accent: [238, 196, 72] },
    CosmeticDef { id: 11, slug: "spirit_fox", name: "Celestial Spirit Fox", subtitle: "Luminous cyan-violet fox ears & astral tail", primary: [88, 216, 252], secondary: [138, 92, 248], accent: [236, 254, 255] },
    CosmeticDef { id: 12, slug: "imp_devil", name: "Crimson Imp Tail & Ears", subtitle: "Pointed imp ears & darting arrow-spade tail", primary: [178, 26, 44], secondary: [42, 20, 28], accent: [255, 112, 78] },
];

pub const SHOULDER_PETS: [CosmeticDef; 12] = [
    CosmeticDef { id: 1, slug: "mini_crimson_dragon", name: "Mini Crimson Dragon", subtitle: "Flapping ruby wyrmling hovering by shoulder", primary: [196, 30, 44], secondary: [38, 24, 30], accent: [255, 204, 68] },
    CosmeticDef { id: 2, slug: "blue_axolotl", name: "Rare Blue Axolotl", subtitle: "Wiggling gills & happy tail on your shoulder", primary: [92, 148, 246], secondary: [242, 152, 68], accent: [218, 238, 255] },
    CosmeticDef { id: 3, slug: "glowing_allay", name: "Luminous Allay", subtitle: "Dancing cyan spirit with fluttering wings", primary: [88, 228, 255], secondary: [48, 158, 218], accent: [228, 254, 255] },
    CosmeticDef { id: 4, slug: "mini_warden", name: "Pocket Warden", subtitle: "Tiny sculk guardian with pulsing sonic heart", primary: [18, 58, 68], secondary: [218, 212, 184], accent: [64, 244, 236] },
    CosmeticDef { id: 5, slug: "bumblebee", name: "Fuzzy Bumblebee", subtitle: "Hovering golden bee with fast-buzzing wings", primary: [248, 204, 48], secondary: [38, 30, 26], accent: [232, 248, 255] },
    CosmeticDef { id: 6, slug: "ender_dragonling", name: "Baby Ender Dragon", subtitle: "Obsidian dragonet with glowing violet eyes", primary: [28, 24, 36], secondary: [112, 48, 178], accent: [228, 128, 255] },
    CosmeticDef { id: 7, slug: "arctic_fox_pet", name: "Snowy Shoulder Fox", subtitle: "Cozy white fox kit nodding along with you", primary: [244, 248, 252], secondary: [182, 204, 228], accent: [52, 48, 58] },
    CosmeticDef { id: 8, slug: "copper_golem", name: "Mini Copper Golem", subtitle: "Spinning lightning rod & waving copper arm", primary: [212, 118, 68], secondary: [72, 178, 146], accent: [255, 218, 118] },
    CosmeticDef { id: 9, slug: "pocket_blaze", name: "Pocket Blaze", subtitle: "Golden blaze spirit with 4 orbiting rods", primary: [252, 178, 38], secondary: [214, 78, 22], accent: [255, 244, 152] },
    CosmeticDef { id: 10, slug: "glow_squid_pet", name: "Baby Glow Squid", subtitle: "Bioluminescent aqua squid with swaying arms", primary: [48, 196, 198], secondary: [24, 98, 122], accent: [176, 255, 246] },
    CosmeticDef { id: 11, slug: "slime_king", name: "Royal Slime King", subtitle: "Bouncing emerald slime wearing a tiny crown", primary: [92, 218, 88], secondary: [52, 154, 54], accent: [252, 208, 64] },
    CosmeticDef { id: 12, slug: "phoenix_chick", name: "Solar Phoenix Chick", subtitle: "Radiant golden firebird fluttering aloft", primary: [255, 154, 36], secondary: [224, 48, 32], accent: [255, 242, 148] },
];

pub const AURAS_TRAILS: [CosmeticDef; 12] = [
    CosmeticDef { id: 1, slug: "crimson_volt_storm", name: "Crimson Volt Storm", subtitle: "Orbiting scarlet & white lightning shards", primary: [228, 32, 52], secondary: [255, 118, 128], accent: [255, 244, 248] },
    CosmeticDef { id: 2, slug: "spiral_galaxy", name: "Spiral Galaxy Orbit", subtitle: "Tilted planetary rings of stars & nebula dust", primary: [108, 138, 255], secondary: [186, 96, 255], accent: [248, 252, 255] },
    CosmeticDef { id: 3, slug: "sakura_vortex", name: "Sakura Petal Vortex", subtitle: "Swirling 3D cherry-blossom petals in the breeze", primary: [252, 156, 192], secondary: [255, 214, 230], accent: [255, 246, 250] },
    CosmeticDef { id: 4, slug: "soulflame_whispers", name: "Soulflame Whispers", subtitle: "Rising helix of turquoise soul-fire motes", primary: [56, 236, 232], secondary: [26, 138, 162], accent: [204, 255, 252] },
    CosmeticDef { id: 5, slug: "golden_starfall", name: "Golden Starfall", subtitle: "Twinkling orbiting 3D golden star prisms", primary: [255, 214, 78], secondary: [246, 158, 42], accent: [255, 252, 208] },
    CosmeticDef { id: 6, slug: "frost_blizzard", name: "Frostbite Snowswirl", subtitle: "Crystalline ice-flakes spiraling around you", primary: [152, 228, 255], secondary: [78, 162, 238], accent: [244, 254, 255] },
    CosmeticDef { id: 7, slug: "emerald_alchemy", name: "Emerald Alchemy Sigils", subtitle: "Orbiting verdant cubes & rising rune sparks", primary: [58, 224, 116], secondary: [26, 142, 74], accent: [212, 255, 188] },
    CosmeticDef { id: 8, slug: "ender_sparks", name: "Ender Rift Sparks", subtitle: "Blinking violet dimensional cubes", primary: [188, 84, 255], secondary: [88, 32, 148], accent: [244, 192, 255] },
    CosmeticDef { id: 9, slug: "symphony_gems", name: "Symphony Harmonic Gems", subtitle: "Floating colorful rhythmic crystal motes", primary: [84, 218, 255], secondary: [255, 116, 198], accent: [255, 226, 96] },
    CosmeticDef { id: 10, slug: "ruby_hearts", name: "Ruby Heartbeat", subtitle: "Floating & pulsing 3D ruby charm motes", primary: [242, 54, 92], secondary: [255, 138, 168], accent: [255, 228, 236] },
    CosmeticDef { id: 11, slug: "abyssal_wisps", name: "Abyssal Shadow Wisps", subtitle: "Obsidian & amethyst void embers", primary: [44, 28, 68], secondary: [142, 64, 224], accent: [218, 168, 255] },
    CosmeticDef { id: 12, slug: "prismatic_helix", name: "Prismatic Rainbow Helix", subtitle: "Double-helix of hue-cycling 3D light prisms", primary: [255, 98, 152], secondary: [78, 238, 196], accent: [128, 178, 255] },
];

pub fn items_for(cat: CosmeticCategory) -> &'static [CosmeticDef] {
    match cat {
        CosmeticCategory::Capes => &CAPES,
        CosmeticCategory::Wings => &WINGS,
        CosmeticCategory::CrownsHats => &CROWNS_HATS,
        CosmeticCategory::HalosHorns => &HALOS_HORNS,
        CosmeticCategory::EarsTails => &EARS_TAILS,
        CosmeticCategory::ShoulderPets => &SHOULDER_PETS,
        CosmeticCategory::AurasTrails => &AURAS_TRAILS,
    }
}

/// Persistent equipped cosmetic loadout saved in `Extras` (`options.txt`).
/// `0` means `None` (unequipped) in each category; `1..=N` selects that item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VoltCosmeticsLoadout {
    pub cape: u8,
    pub wings: u8,
    pub hat: u8,
    pub halo: u8,
    pub ears_tail: u8,
    pub pet: u8,
    pub aura: u8,
    pub show_in_world: bool,
    pub anim_speed_pct: u16,
}

impl Default for VoltCosmeticsLoadout {
    fn default() -> Self {
        Self {
            cape: 0,
            wings: 0,
            hat: 0,
            halo: 0,
            ears_tail: 0,
            pet: 0,
            aura: 0,
            show_in_world: true,
            anim_speed_pct: 100,
        }
    }
}

impl VoltCosmeticsLoadout {
    pub fn any_equipped(&self) -> bool {
        self.cape != 0
            || self.wings != 0
            || self.hat != 0
            || self.halo != 0
            || self.ears_tail != 0
            || self.pet != 0
            || self.aura != 0
    }

    pub fn equipped_count(&self) -> usize {
        CosmeticCategory::ALL
            .into_iter()
            .filter(|&c| self.get(c) != 0)
            .count()
    }

    pub fn get(&self, cat: CosmeticCategory) -> u8 {
        match cat {
            CosmeticCategory::Capes => self.cape,
            CosmeticCategory::Wings => self.wings,
            CosmeticCategory::CrownsHats => self.hat,
            CosmeticCategory::HalosHorns => self.halo,
            CosmeticCategory::EarsTails => self.ears_tail,
            CosmeticCategory::ShoulderPets => self.pet,
            CosmeticCategory::AurasTrails => self.aura,
        }
    }

    pub fn set(&mut self, cat: CosmeticCategory, id: u8) {
        let max_id = items_for(cat).len() as u8;
        let clamped = id.min(max_id);
        match cat {
            CosmeticCategory::Capes => self.cape = clamped,
            CosmeticCategory::Wings => self.wings = clamped,
            CosmeticCategory::CrownsHats => self.hat = clamped,
            CosmeticCategory::HalosHorns => self.halo = clamped,
            CosmeticCategory::EarsTails => self.ears_tail = clamped,
            CosmeticCategory::ShoulderPets => self.pet = clamped,
            CosmeticCategory::AurasTrails => self.aura = clamped,
        }
    }

    pub fn clear_all(&mut self) {
        self.cape = 0;
        self.wings = 0;
        self.hat = 0;
        self.halo = 0;
        self.ears_tail = 0;
        self.pet = 0;
        self.aura = 0;
    }
}

/// Interactive UI state for the split-screen `Volt Cosmetics` studio page.
#[derive(Clone, Debug)]
pub struct CosmeticsStudioState {
    pub category: CosmeticCategory,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub yaw_vel: f32,
    pub auto_spin: bool,
    pub dragging_model: bool,
    pub last_drag: [f32; 2],
    pub scroll: [f32; 7],
    pub dragging_list: bool,
    pub list_drag_start_y: f32,
    pub list_drag_moved: bool,
    pub last_time: f32,
    pub epoch: std::time::Instant,
}

impl Default for CosmeticsStudioState {
    fn default() -> Self {
        Self {
            category: CosmeticCategory::Capes,
            yaw_deg: 152.0,
            pitch_deg: 10.0,
            yaw_vel: 0.0,
            auto_spin: false,
            dragging_model: false,
            last_drag: [0.0, 0.0],
            scroll: [0.0; 7],
            dragging_list: false,
            list_drag_start_y: 0.0,
            list_drag_moved: false,
            last_time: 0.0,
            epoch: std::time::Instant::now(),
        }
    }
}

impl CosmeticsStudioState {
    pub fn elapsed_s(&self) -> f32 {
        self.epoch.elapsed().as_secs_f32()
    }

    pub fn step_time_live(&mut self) {
        let now = self.elapsed_s();
        self.step_time(now);
    }

    pub fn step_time(&mut self, time_s: f32) {
        let dt = if self.last_time > 0.0 {
            (time_s - self.last_time).clamp(0.0, 0.1)
        } else {
            1.0 / 60.0
        };
        self.last_time = time_s;
        if !self.dragging_model {
            if self.auto_spin {
                self.yaw_deg = (self.yaw_deg + dt * 36.0).rem_euclid(360.0);
            } else if self.yaw_vel.abs() > 0.05 {
                self.yaw_deg = (self.yaw_deg + self.yaw_vel * dt).rem_euclid(360.0);
                self.yaw_vel *= (1.0 - dt * 5.5).clamp(0.0, 1.0);
            }
        }
    }
}

/// Layout geometry for the split-screen Cosmetics Studio (`width x height` GUI px).
#[derive(Clone, Copy, Debug)]
pub struct StudioLayout {
    pub left_box: [f32; 4],
    pub model_box: [f32; 4],
    pub btn_front: [f32; 4],
    pub btn_back: [f32; 4],
    pub btn_spin: [f32; 4],
    pub right_box: [f32; 4],
    pub tabs_y: f32,
    pub list_box: [f32; 4],
    pub btn_world_vis: [f32; 4],
    pub btn_clear: [f32; 4],
}

impl StudioLayout {
    pub fn compute(width: f32, height: f32) -> Self {
        let top = 30.0;
        let bottom = (height - 32.0).max(top + 120.0);
        let pad = 10.0;
        let total_w = (width - pad * 2.0).max(260.0);
        let left_w = (total_w * 0.37).clamp(112.0, 196.0);
        let left_x = pad;
        let right_x = left_x + left_w + 8.0;
        let right_w = (width - pad - right_x).max(140.0);
        let pane_h = bottom - top;

        let ctrl_h = 18.0;
        let model_h = (pane_h - ctrl_h - 10.0).max(80.0);
        let bw = ((left_w - 12.0) / 3.0).max(28.0);
        let by = top + pane_h - ctrl_h - 4.0;

        let tabs_y = top + 4.0;
        let header_h = 38.0;
        let footer_h = 22.0;
        let list_y = top + header_h;
        let list_h = (pane_h - header_h - footer_h - 4.0).max(60.0);
        let fy = top + pane_h - footer_h;
        let fw = ((right_w - 12.0) * 0.5).max(50.0);

        Self {
            left_box: [left_x, top, left_w, pane_h],
            model_box: [left_x + 3.0, top + 3.0, left_w - 6.0, model_h],
            btn_front: [left_x + 4.0, by, bw, ctrl_h],
            btn_back: [left_x + 6.0 + bw, by, bw, ctrl_h],
            btn_spin: [left_x + 8.0 + bw * 2.0, by, bw, ctrl_h],
            right_box: [right_x, top, right_w, pane_h],
            tabs_y,
            list_box: [right_x + 4.0, list_y, right_w - 8.0, list_h],
            btn_world_vis: [right_x + 4.0, fy, fw, 18.0],
            btn_clear: [right_x + 8.0 + fw, fy, fw, 18.0],
        }
    }

    pub fn tab_rect(&self, idx: usize) -> [f32; 4] {
        let [rx, _, rw, _] = self.right_box;
        let n = CosmeticCategory::ALL.len() as f32;
        let gap = 2.0;
        let tw = ((rw - 8.0 - gap * (n - 1.0)) / n).max(18.0);
        [rx + 4.0 + idx as f32 * (tw + gap), self.tabs_y, tw, 18.0]
    }
}

fn hit([x, y, w, h]: [f32; 4], px: f32, py: f32) -> bool {
    px >= x && px < x + w && py >= y && py < y + h
}

/// Handles mouse/touch press inside `Page::VoltCosmetics`. Returns `true` if
/// `loadout` changed (so `OptionsUi` marks `dirty = true`).
pub fn pointer_down(
    studio: &mut CosmeticsStudioState,
    loadout: &mut VoltCosmeticsLoadout,
    (width, height): (f32, f32),
    (px, py): (f32, f32),
) -> (bool, bool) {
    let lay = StudioLayout::compute(width, height);
    if hit(lay.model_box, px, py) {
        studio.dragging_model = true;
        studio.auto_spin = false;
        studio.yaw_vel = 0.0;
        studio.last_drag = [px, py];
        return (false, false);
    }
    if hit(lay.btn_front, px, py) {
        studio.auto_spin = false;
        studio.yaw_vel = 0.0;
        studio.yaw_deg = 24.0;
        studio.pitch_deg = 8.0;
        return (false, true);
    }
    if hit(lay.btn_back, px, py) {
        studio.auto_spin = false;
        studio.yaw_vel = 0.0;
        studio.yaw_deg = 160.0;
        studio.pitch_deg = 10.0;
        return (false, true);
    }
    if hit(lay.btn_spin, px, py) {
        studio.auto_spin = !studio.auto_spin;
        return (false, true);
    }
    for (i, &cat) in CosmeticCategory::ALL.iter().enumerate() {
        if hit(lay.tab_rect(i), px, py) {
            studio.category = cat;
            if let Some(pref_yaw) = cat.default_yaw_for_preview() {
                if !studio.auto_spin && (studio.yaw_deg < 85.0 || studio.yaw_deg > 275.0) {
                    studio.yaw_deg = pref_yaw;
                }
            } else if !studio.auto_spin && studio.yaw_deg > 95.0 && studio.yaw_deg < 265.0 {
                studio.yaw_deg = 26.0;
            }
            return (false, true);
        }
    }
    if hit(lay.btn_world_vis, px, py) {
        loadout.show_in_world = !loadout.show_in_world;
        return (true, true);
    }
    if hit(lay.btn_clear, px, py) {
        loadout.clear_all();
        return (true, true);
    }
    if hit(lay.list_box, px, py) {
        studio.dragging_list = true;
        studio.list_drag_start_y = py;
        studio.list_drag_moved = false;
        studio.last_drag = [px, py];
        let cat = studio.category;
        let scroll = studio.scroll[cat.index()];
        let rel_y = py - lay.list_box[1] + scroll - 2.0;
        if rel_y >= 0.0 {
            let row = (rel_y / 25.0).floor() as usize;
            let items = items_for(cat);
            if row == 0 {
                loadout.set(cat, 0);
                return (true, true);
            } else if row - 1 < items.len() {
                let chosen = items[row - 1].id;
                let next = if loadout.get(cat) == chosen { 0 } else { chosen };
                loadout.set(cat, next);
                return (true, true);
            }
        }
    }
    (false, false)
}

pub fn pointer_move(
    studio: &mut CosmeticsStudioState,
    (width, height): (f32, f32),
    (px, py): (f32, f32),
) {
    let lay = StudioLayout::compute(width, height);
    if studio.dragging_model {
        let dx = px - studio.last_drag[0];
        let dy = py - studio.last_drag[1];
        studio.last_drag = [px, py];
        studio.yaw_deg = (studio.yaw_deg + dx * 1.15).rem_euclid(360.0);
        studio.pitch_deg = (studio.pitch_deg + dy * 0.65).clamp(-32.0, 34.0);
        studio.yaw_vel = (dx * 28.0).clamp(-320.0, 320.0);
    } else if studio.dragging_list {
        let dy = py - studio.last_drag[1];
        studio.last_drag = [px, py];
        if (py - studio.list_drag_start_y).abs() > 4.0 {
            studio.list_drag_moved = true;
        }
        let idx = studio.category.index();
        let count = items_for(studio.category).len() + 1; // +1 for None card
        let max_scroll = ((count as f32 * 25.0) - lay.list_box[3] + 6.0).max(0.0);
        studio.scroll[idx] = (studio.scroll[idx] - dy).clamp(0.0, max_scroll);
    }
}

pub fn pointer_up(
    studio: &mut CosmeticsStudioState,
    loadout: &mut VoltCosmeticsLoadout,
    (width, height): (f32, f32),
    (px, py): (f32, f32),
) -> (bool, bool) {
    let was_list = studio.dragging_list;
    let moved = studio.list_drag_moved;
    studio.dragging_model = false;
    studio.dragging_list = false;
    studio.list_drag_moved = false;

    if was_list && !moved {
        let lay = StudioLayout::compute(width, height);
        if hit(lay.list_box, px, py) {
            let cat = studio.category;
            let scroll = studio.scroll[cat.index()];
            let rel_y = py - lay.list_box[1] + scroll - 2.0;
            if rel_y >= 0.0 {
                let row = (rel_y / 25.0).floor() as usize;
                let items = items_for(cat);
                if row == 0 {
                    loadout.set(cat, 0);
                    return (true, true);
                } else if row - 1 < items.len() {
                    let chosen = items[row - 1].id;
                    let next = if loadout.get(cat) == chosen { 0 } else { chosen };
                    loadout.set(cat, next);
                    return (true, true);
                }
            }
        }
    }
    (false, false)
}

pub fn scroll_wheel(studio: &mut CosmeticsStudioState, (width, height): (f32, f32), lines: f32) {
    let lay = StudioLayout::compute(width, height);
    let idx = studio.category.index();
    let count = items_for(studio.category).len() + 1;
    let max_scroll = ((count as f32 * 25.0) - lay.list_box[3] + 6.0).max(0.0);
    studio.scroll[idx] = (studio.scroll[idx] - lines * 22.0).clamp(0.0, max_scroll);
}

// ============================================================================
// 3D Geometry & Cosmetic Rig (`HudBatch::quad_3d` + `depth_since`)
// ============================================================================

#[derive(Clone, Copy)]
struct PoseTransform {
    yaw_sin: f32,
    yaw_cos: f32,
    pitch_sin: f32,
    pitch_cos: f32,
    cx: f32,
    cy: f32,
    scale: f32,
}

impl PoseTransform {
    fn studio(cx: f32, cy: f32, scale: f32, yaw_deg: f32, pitch_deg: f32) -> Self {
        let (yaw_sin, yaw_cos) = (yaw_deg * DEG).sin_cos();
        let (pitch_sin, pitch_cos) = (pitch_deg * DEG).sin_cos();
        Self {
            yaw_sin,
            yaw_cos,
            pitch_sin,
            pitch_cos,
            cx,
            cy,
            scale,
        }
    }

    /// Maps model-space `(x, y_down, z_front)` into screen `[sx, sy, depth]`.
    fn project(self, p: [f32; 3]) -> [f32; 3] {
        let rx = p[0] * self.yaw_cos + p[2] * self.yaw_sin;
        let rz0 = -p[0] * self.yaw_sin + p[2] * self.yaw_cos;
        let ry = p[1] * self.pitch_cos - rz0 * self.pitch_sin;
        let rz = p[1] * self.pitch_sin + rz0 * self.pitch_cos;
        [self.cx + rx * self.scale, self.cy + ry * self.scale, rz]
    }

    fn rotate_normal(self, n: [f32; 3]) -> [f32; 3] {
        let rx = n[0] * self.yaw_cos + n[2] * self.yaw_sin;
        let rz0 = -n[0] * self.yaw_sin + n[2] * self.yaw_cos;
        let ry = n[1] * self.pitch_cos - rz0 * self.pitch_sin;
        let rz = n[1] * self.pitch_sin + rz0 * self.pitch_cos;
        [rx, ry, rz]
    }
}

/// Sink that emits lit 3D quads either into the 2D HUD Studio preview OR
/// through the 3D camera `view_proj` matrix for in-world `F5` player rendering.
enum QuadSink<'a> {
    Studio(PoseTransform),
    World {
        view_proj: &'a [[f32; 4]; 4],
        player_rel: [f32; 3],
        body_yaw_sin: f32,
        body_yaw_cos: f32,
        gui_w: f32,
        gui_h: f32,
    },
}

impl<'a> QuadSink<'a> {
    fn emit_quad(
        &self,
        batch: &mut HudBatch,
        pts_model: [[f32; 3]; 4],
        normal_model: [f32; 3],
        uv: [[f32; 2]; 4],
        rgb: [u8; 3],
        alpha: u8,
        emissive: bool,
    ) {
        let shade = if emissive {
            1.0
        } else {
            let n = match self {
                Self::Studio(pose) => pose.rotate_normal(normal_model),
                Self::World {
                    body_yaw_sin,
                    body_yaw_cos,
                    ..
                } => [
                    normal_model[0] * body_yaw_cos + normal_model[2] * body_yaw_sin,
                    normal_model[1],
                    -normal_model[0] * body_yaw_sin + normal_model[2] * body_yaw_cos,
                ],
            };
            let dot_l = (n[0] * -0.36 + n[1] * -0.58 + n[2] * 0.73).max(0.0);
            (0.56 + 0.44 * dot_l).clamp(0.45, 1.0)
        };
        let color = hud::rgba(
            (f32::from(rgb[0]) * shade).round().clamp(0.0, 255.0) as u8,
            (f32::from(rgb[1]) * shade).round().clamp(0.0, 255.0) as u8,
            (f32::from(rgb[2]) * shade).round().clamp(0.0, 255.0) as u8,
            alpha,
        );

        match self {
            Self::Studio(pose) => {
                let p = pts_model.map(|v| pose.project(v));
                batch.quad_3d(p, uv, color);
            }
            Self::World {
                view_proj,
                player_rel,
                body_yaw_sin,
                body_yaw_cos,
                gui_w,
                gui_h,
            } => {
                // Model space uses 16 units = ~0.9375 blocks, `y_down` with feet at `+16.0`.
                const UNIT_TO_BLOCKS: f32 = 0.9375 / 16.0;
                let mut out = [[0.0f32; 3]; 4];
                for (i, m) in pts_model.iter().enumerate() {
                    let bx = (-m[0] * body_yaw_cos - m[2] * body_yaw_sin) * UNIT_TO_BLOCKS;
                    let by = (16.0 - m[1]) * UNIT_TO_BLOCKS;
                    let bz = (m[0] * body_yaw_sin - m[2] * body_yaw_cos) * UNIT_TO_BLOCKS;
                    let wx = player_rel[0] + bx;
                    let wy = player_rel[1] + by;
                    let wz = player_rel[2] + bz;
                    let cx = view_proj[0][0] * wx
                        + view_proj[1][0] * wy
                        + view_proj[2][0] * wz
                        + view_proj[3][0];
                    let cy = view_proj[0][1] * wx
                        + view_proj[1][1] * wy
                        + view_proj[2][1] * wz
                        + view_proj[3][1];
                    let cz = view_proj[0][2] * wx
                        + view_proj[1][2] * wy
                        + view_proj[2][2] * wz
                        + view_proj[3][2];
                    let cw = view_proj[0][3] * wx
                        + view_proj[1][3] * wy
                        + view_proj[2][3] * wz
                        + view_proj[3][3];
                    if cw <= 0.12 {
                        return;
                    }
                    let inv_w = 1.0 / cw;
                    let sx = (cx * inv_w * 0.5 + 0.5) * gui_w;
                    let sy = (0.5 - cy * inv_w * 0.5) * gui_h;
                    // Higher `out[2]` = closer to camera for `HudBatch::depth_since`
                    out[i] = [sx, sy, 100.0 - (cz * inv_w).clamp(0.0, 1.0) * 100.0];
                }
                batch.quad_3d(out, uv, color);
            }
        }
    }
}

fn white_uv(atlas: &HudAtlas) -> [[f32; 2]; 4] {
    let w = atlas.white;
    let cx = (w.u0 + w.u1) * 0.5;
    let cy = (w.v0 + w.v1) * 0.5;
    [[cx, cy]; 4]
}

fn rot_point(p: [f32; 3], pivot: [f32; 3], rx: f32, ry: f32, rz: f32) -> [f32; 3] {
    let mut x = p[0] - pivot[0];
    let mut y = p[1] - pivot[1];
    let mut z = p[2] - pivot[2];
    if rz.abs() > 0.0001 {
        let (s, c) = rz.sin_cos();
        let nx = x * c - y * s;
        let ny = x * s + y * c;
        x = nx;
        y = ny;
    }
    if rx.abs() > 0.0001 {
        let (s, c) = rx.sin_cos();
        let ny = y * c - z * s;
        let nz = y * s + z * c;
        y = ny;
        z = nz;
    }
    if ry.abs() > 0.0001 {
        let (s, c) = ry.sin_cos();
        let nx = x * c + z * s;
        let nz = -x * s + z * c;
        x = nx;
        z = nz;
    }
    [x + pivot[0], y + pivot[1], z + pivot[2]]
}

#[allow(clippy::too_many_arguments)]
fn emit_solid_box(
    sink: &QuadSink<'_>,
    batch: &mut HudBatch,
    w_uv: [[f32; 2]; 4],
    min: [f32; 3],
    max: [f32; 3],
    pivot: [f32; 3],
    angles: [f32; 3],
    rgb: [u8; 3],
    alpha: u8,
    emissive: bool,
) {
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;
    let faces: [([[f32; 3]; 4], [f32; 3]); 6] = [
        ([[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]], [0.0, 0.0, 1.0]),
        ([[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]], [0.0, 0.0, -1.0]),
        ([[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]], [1.0, 0.0, 0.0]),
        ([[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]], [-1.0, 0.0, 0.0]),
        ([[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]], [0.0, -1.0, 0.0]),
        ([[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]], [0.0, 1.0, 0.0]),
    ];
    for (verts, norm) in faces {
        let rv = verts.map(|v| rot_point(v, pivot, angles[0], angles[1], angles[2]));
        let rn = rot_point(norm, [0.0; 3], angles[0], angles[1], angles[2]);
        sink.emit_quad(batch, rv, rn, w_uv, rgb, alpha, emissive);
    }
}

fn hue_cycle_rgb(t: f32) -> [u8; 3] {
    let r = (t.sin() * 0.5 + 0.5) * 215.0 + 40.0;
    let g = ((t + 2.094_395).sin() * 0.5 + 0.5) * 215.0 + 40.0;
    let b = ((t + 4.188_790).sin() * 0.5 + 0.5) * 215.0 + 40.0;
    [r as u8, g as u8, b as u8]
}

fn lerp_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let k = t.clamp(0.0, 1.0);
    [
        (f32::from(a[0]) + (f32::from(b[0]) - f32::from(a[0])) * k).round() as u8,
        (f32::from(a[1]) + (f32::from(b[1]) - f32::from(a[1])) * k).round() as u8,
        (f32::from(a[2]) + (f32::from(b[2]) - f32::from(a[2])) * k).round() as u8,
    ]
}

/// Renders all equipped 3D cosmetics into `sink` (shared by Studio & in-world F5).
fn emit_equipped_cosmetics(
    sink: &QuadSink<'_>,
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    loadout: &VoltCosmeticsLoadout,
    time_s: f32,
    move_speed: f32,
) {
    let w_uv = white_uv(atlas);
    let rate = (f32::from(loadout.anim_speed_pct.clamp(25, 200)) / 100.0).clamp(0.25, 2.0);
    let t = time_s * rate;

    // 1. CAPES: 6-segment articulated 3D waving silk cape + center emblem!
    if loadout.cape > 0
        && let Some(def) = CAPES.get(usize::from(loadout.cape - 1))
    {
        let base_tilt = 0.16 + (move_speed * 0.55).clamp(0.0, 0.65);
        let mut prev_y = -8.0f32;
        let mut prev_z = -2.15f32;
        let segs = 6;
        let seg_len = 2.65f32;
        for s in 0..segs {
            let frac = s as f32 / segs as f32;
            let wave = (t * 3.1 - frac * 2.6).sin() * (0.08 + frac * 0.14)
                + (t * 1.9 + frac * 1.4).cos() * 0.04;
            let ang = base_tilt + frac * 0.18 + wave;
            let ny = prev_y + ang.cos() * seg_len;
            let nz = prev_z - ang.sin() * seg_len;
            let half_w = 4.6 + frac * 0.65;
            let col = if def.id == 14 {
                hue_cycle_rgb(t * 1.6 + frac * 2.2)
            } else {
                lerp_rgb(def.primary, def.secondary, frac)
            };
            let pts_back = [
                [-half_w, prev_y, prev_z - 0.35],
                [half_w, prev_y, prev_z - 0.35],
                [half_w + 0.1, ny, nz - 0.35],
                [-half_w - 0.1, ny, nz - 0.35],
            ];
            let pts_front = [
                [half_w, prev_y, prev_z],
                [-half_w, prev_y, prev_z],
                [-half_w - 0.1, ny, nz],
                [half_w + 0.1, ny, nz],
            ];
            sink.emit_quad(batch, pts_back, [0.0, 0.15, -0.98], w_uv, col, 255, false);
            sink.emit_quad(batch, pts_front, [0.0, 0.15, 0.98], w_uv, def.secondary, 255, false);
            if s == 1 || s == 2 {
                let ew = if s == 1 { 2.0 } else { 1.2 };
                let emb = [
                    [-ew, prev_y + 0.25, prev_z - 0.48],
                    [ew, prev_y + 0.25, prev_z - 0.48],
                    [ew * 0.75, ny - 0.25, nz - 0.48],
                    [-ew * 0.75, ny - 0.25, nz - 0.48],
                ];
                sink.emit_quad(batch, emb, [0.0, 0.0, -1.0], w_uv, def.accent, 255, true);
            }
            if s == segs - 1 {
                let hem = [
                    [-half_w - 0.1, ny - 0.45, nz - 0.40],
                    [half_w + 0.1, ny - 0.45, nz - 0.40],
                    [half_w + 0.1, ny, nz - 0.40],
                    [-half_w - 0.1, ny, nz - 0.40],
                ];
                sink.emit_quad(batch, hem, [0.0, 0.0, -1.0], w_uv, def.accent, 255, true);
            }
            prev_y = ny;
            prev_z = nz;
        }
    }

    // 2. WINGS: Articulated symmetric flapping 5-blade 3D wings!
    if loadout.wings > 0
        && let Some(def) = WINGS.get(usize::from(loadout.wings - 1))
    {
        let fast = matches!(def.id, 6 | 7 | 9);
        let freq = if fast { 6.2 } else { 2.4 };
        let flap = (t * freq).sin() * if fast { 0.48 } else { 0.34 } + 0.38;
        for side in [-1.0f32, 1.0f32] {
            let pivot = [side * 1.4, -6.0, -2.1];
            let ry = -side * flap;
            // Upper main spar
            emit_solid_box(
                sink,
                batch,
                w_uv,
                [side * 1.4, -7.2, -2.5],
                [side * 12.8, -5.6, -1.9],
                pivot,
                [0.0, ry, -side * 0.24],
                def.primary,
                255,
                false,
            );
            // 4 fanned flight feathers / energy blades
            for b in 0..4 {
                let bf = b as f32;
                let span = 11.8 - bf * 1.55;
                let y_top = -5.4 + bf * 2.15;
                let feather_col = if b == 0 {
                    def.accent
                } else if b % 2 == 1 {
                    def.primary
                } else {
                    def.secondary
                };
                let sub_flap = ry + -side * ((t * freq - bf * 0.35).sin() * 0.09);
                let x_min = (side * 2.0).min(side * span);
                let x_max = (side * 2.0).max(side * span);
                emit_solid_box(
                    sink,
                    batch,
                    w_uv,
                    [x_min, y_top, -2.35],
                    [x_max, y_top + 1.85, -2.0],
                    pivot,
                    [0.0, sub_flap, -side * (0.16 - bf * 0.09)],
                    feather_col,
                    if matches!(def.id, 8 | 9 | 10) { 232 } else { 255 },
                    matches!(def.id, 8 | 10),
                );
            }
        }
    }

    // 3. CROWNS & HATS: Sculpted 3D headwear anchored to head (`y = -16..-8`)!
    if loadout.hat > 0
        && let Some(def) = CROWNS_HATS.get(usize::from(loadout.hat - 1))
    {
        let head_bob = (t * 2.2).sin() * 0.22;
        match def.id {
            1 | 10 => {
                // Royal / Glacial Crown + floating center gem
                emit_solid_box(sink, batch, w_uv, [-4.4, -17.2, -4.4], [4.4, -15.8, 4.4], [0.0; 3], [0.0; 3], def.primary, 255, false);
                for sx in [-3.4f32, 0.0, 3.4] {
                    for sz in [-3.8f32, 3.8] {
                        emit_solid_box(sink, batch, w_uv, [sx - 0.7, -19.2, sz - 0.6], [sx + 0.7, -17.2, sz + 0.6], [0.0; 3], [0.0; 3], def.primary, 255, false);
                    }
                }
                let gy = -20.6 + head_bob * 1.8;
                emit_solid_box(sink, batch, w_uv, [-1.0, gy - 1.0, -1.0], [1.0, gy + 1.0, 1.0], [0.0, gy, 0.0], [0.4, t * 2.0, 0.4], def.secondary, 255, true);
            }
            2 => {
                // Grand Wizard Hat (wide brim + angled cone + orbiting star)
                emit_solid_box(sink, batch, w_uv, [-6.8, -16.6, -6.8], [6.8, -15.7, 6.8], [0.0; 3], [0.0; 3], def.primary, 255, false);
                emit_solid_box(sink, batch, w_uv, [-4.3, -17.6, -4.3], [4.3, -16.5, 4.3], [0.0; 3], [0.0; 3], def.secondary, 255, false);
                emit_solid_box(sink, batch, w_uv, [-3.3, -21.0, -3.3], [3.3, -17.5, 3.3], [0.0, -17.5, 0.0], [-0.14, 0.0, 0.0], def.primary, 255, false);
                emit_solid_box(sink, batch, w_uv, [-1.8, -24.2, -2.2], [1.8, -20.8, 1.4], [0.0, -20.8, 0.0], [-0.28, 0.0, 0.0], def.primary, 255, false);
                let (os, oc) = (t * 2.4).sin_cos();
                emit_solid_box(sink, batch, w_uv, [oc * 3.8 - 0.6, -22.5 + head_bob, os * 3.8 - 0.6], [oc * 3.8 + 0.6, -21.3 + head_bob, os * 3.8 + 0.6], [0.0; 3], [0.0; 3], def.accent, 255, true);
            }
            6 => {
                // Phantom Velvet Top Hat
                emit_solid_box(sink, batch, w_uv, [-6.0, -16.6, -6.0], [6.0, -15.8, 6.0], [0.0; 3], [0.0; 3], def.primary, 255, false);
                emit_solid_box(sink, batch, w_uv, [-4.1, -17.8, -4.1], [4.1, -16.6, 4.1], [0.0; 3], [0.0; 3], def.secondary, 255, true);
                emit_solid_box(sink, batch, w_uv, [-4.0, -23.0, -4.0], [4.0, -17.8, 4.0], [0.0; 3], [0.0; 3], def.primary, 255, false);
            }
            9 => {
                // Bouncy Toadstool Mushroom Cap
                let sq = (t * 3.2).sin() * 0.25;
                emit_solid_box(sink, batch, w_uv, [-6.6 - sq, -19.8 + sq, -6.6 - sq], [6.6 + sq, -16.0, 6.6 + sq], [0.0; 3], [0.0; 3], def.primary, 255, false);
                for [sx, sz] in [[-3.6f32, 3.6], [3.6, -3.2], [0.0, -4.2], [-3.2, -2.8]] {
                    emit_solid_box(sink, batch, w_uv, [sx - 1.2, -20.1 + sq, sz - 1.2], [sx + 1.2, -19.6 + sq, sz + 1.2], [0.0; 3], [0.0; 3], def.secondary, 255, false);
                }
            }
            5 | 8 | 12 => {
                // Goggles / Starfarer / Cyberpunk HUD Visor
                let vy = if def.id == 5 { -14.8 } else { -13.6 };
                emit_solid_box(sink, batch, w_uv, [-4.5, vy - 1.2, 3.6], [4.5, vy + 1.2, 4.85], [0.0; 3], [0.0; 3], def.accent, 236, true);
                emit_solid_box(sink, batch, w_uv, [-4.6, vy - 1.4, -4.4], [4.6, vy - 0.9, 4.6], [0.0; 3], [0.0; 3], def.primary, 255, false);
            }
            _ => {
                // Kabuto / Laurel / Tricorne / Blossom Garland
                emit_solid_box(sink, batch, w_uv, [-4.6, -17.3, -4.6], [4.6, -15.6, 4.6], [0.0; 3], [0.0; 3], def.primary, 255, false);
                emit_solid_box(sink, batch, w_uv, [-2.8, -20.2 + head_bob, 3.9], [2.8, -17.0 + head_bob, 4.7], [0.0; 3], [0.0; 3], def.accent, 255, true);
                emit_solid_box(sink, batch, w_uv, [-1.0, -21.6 + head_bob, -2.0], [1.0, -17.3 + head_bob, 2.0], [0.0; 3], [0.0; 3], def.secondary, 255, false);
            }
        }
    }

    // 4. HALOS & HORNS: Floating rotating 8-segment 3D halos or sculpted horns!
    if loadout.halo > 0
        && let Some(def) = HALOS_HORNS.get(usize::from(loadout.halo - 1))
    {
        let is_horns = matches!(def.id, 2 | 3 | 8 | 10);
        if is_horns {
            for side in [-1.0f32, 1.0f32] {
                for seg in 0..3 {
                    let sf = seg as f32;
                    let x = side * (3.6 + sf * 1.45);
                    let y = -15.8 - sf * 1.85;
                    let z = 1.2 - sf * if def.id == 8 { -1.2 } else { 1.35 };
                    let r = 1.05 - sf * 0.22;
                    let col = if seg == 2 { def.accent } else if seg == 1 { def.secondary } else { def.primary };
                    emit_solid_box(
                        sink,
                        batch,
                        w_uv,
                        [x - r, y - r, z - r],
                        [x + r, y + r, z + r],
                        [x, y, z],
                        [0.2, 0.0, side * 0.35],
                        col,
                        255,
                        seg == 2,
                    );
                }
            }
        } else {
            let hy = -19.8 + (t * 2.4).sin() * 0.65;
            let n_segs = if def.id == 9 { 5 } else { 8 };
            let radius = 4.2f32;
            for i in 0..n_segs {
                let ang = t * 1.45 + (i as f32 / n_segs as f32) * std::f32::consts::TAU;
                let (s, c) = ang.sin_cos();
                let cx = c * radius;
                let cz = s * radius;
                let col = if def.id == 11 {
                    hue_cycle_rgb(t * 2.0 + i as f32 * 0.7)
                } else if i % 2 == 0 {
                    def.primary
                } else {
                    def.accent
                };
                emit_solid_box(
                    sink,
                    batch,
                    w_uv,
                    [cx - 1.15, hy - 0.42, cz - 0.55],
                    [cx + 1.15, hy + 0.42, cz + 0.55],
                    [cx, hy, cz],
                    [0.0, -ang + std::f32::consts::FRAC_PI_2, 0.0],
                    col,
                    255,
                    true,
                );
            }
        }
    }

    // 5. EARS & TAILS: Twitching 3D ears + swaying articulated 3D tail(s)!
    if loadout.ears_tail > 0
        && let Some(def) = EARS_TAILS.get(usize::from(loadout.ears_tail - 1))
    {
        let twitch = ((t * 3.8).sin().max(0.65) - 0.65) * 0.65;
        let tall = def.id == 4; // Bunny ears
        for side in [-1.0f32, 1.0f32] {
            let ex = side * 2.6;
            let eh = if tall { 5.2 } else { 2.8 };
            let piv = [ex, -16.0, 0.4];
            emit_solid_box(
                sink,
                batch,
                w_uv,
                [ex - 1.05, -16.0 - eh, -0.2],
                [ex + 1.05, -15.8, 1.1],
                piv,
                [0.0, 0.0, side * (0.18 + twitch)],
                def.primary,
                255,
                false,
            );
            emit_solid_box(
                sink,
                batch,
                w_uv,
                [ex - 0.55, -15.6 - eh * 0.8, 0.85],
                [ex + 0.55, -16.0, 1.25],
                piv,
                [0.0, 0.0, side * (0.18 + twitch)],
                def.secondary,
                255,
                def.id == 8 || def.id == 11,
            );
        }

        // Articulated 3D Tail(s) anchored at lower back `(0, 2.8, -2.0)`
        let tail_count = if def.id == 1 { 5 } else { 1 };
        for tc in 0..tail_count {
            let fan = if tail_count > 1 {
                (tc as f32 - 2.0) * 0.34
            } else {
                0.0
            };
            let sway = (t * 2.8 + tc as f32 * 0.7).sin() * 0.38 + fan;
            let mut tx = 0.0f32;
            let mut ty = 2.8f32;
            let mut tz = -2.1f32;
            for seg in 0..4 {
                let sf = seg as f32;
                let yaw_seg = sway * (0.55 + sf * 0.25);
                let lift = -0.35 + sf * 0.22;
                let step = 2.15f32;
                let nx = tx + yaw_seg.sin() * step;
                let ny = ty - lift * step;
                let nz = tz - yaw_seg.cos() * step;
                let r = if def.id == 4 { 1.35 } else { 1.05 - sf * 0.12 };
                let col = if seg == 3 { def.secondary } else { def.primary };
                emit_solid_box(
                    sink,
                    batch,
                    w_uv,
                    [nx - r, ny - r, nz - r],
                    [nx + r, ny + r, nz + r],
                    [nx, ny, nz],
                    [0.0, yaw_seg, 0.0],
                    col,
                    255,
                    def.id == 8 || def.id == 11,
                );
                tx = nx;
                ty = ny;
                tz = nz;
                if def.id == 4 {
                    break; // Fluffy bunny pom tail
                }
            }
        }
    }

    // 6. SHOULDER PETS: Animated hovering/perched 3D mini companion on left shoulder!
    if loadout.pet > 0
        && let Some(def) = SHOULDER_PETS.get(usize::from(loadout.pet - 1))
    {
        let bob = (t * 3.1).sin() * 0.85;
        let px = 6.8f32;
        let py = -11.2 + bob;
        let pz = 0.8f32;
        // Pet body + head + eyes + flapping mini wings
        emit_solid_box(sink, batch, w_uv, [px - 1.6, py - 1.4, pz - 1.8], [px + 1.6, py + 1.4, pz + 1.6], [px, py, pz], [0.0, (t * 1.4).sin() * 0.18, 0.0], def.primary, 255, false);
        emit_solid_box(sink, batch, w_uv, [px - 1.3, py - 2.6, pz + 0.4], [px + 1.3, py - 0.6, pz + 2.6], [px, py, pz], [0.0; 3], def.primary, 255, false);
        // Glowing eyes / sonic heart / slime crown
        emit_solid_box(sink, batch, w_uv, [px - 0.95, py - 2.0, pz + 2.45], [px - 0.35, py - 1.4, pz + 2.8], [0.0; 3], [0.0; 3], def.accent, 255, true);
        emit_solid_box(sink, batch, w_uv, [px + 0.35, py - 2.0, pz + 2.45], [px + 0.95, py - 1.4, pz + 2.8], [0.0; 3], [0.0; 3], def.accent, 255, true);
        let p_flap = (t * 8.2).sin() * 0.55;
        for ws in [-1.0f32, 1.0f32] {
            let wx0 = (px + ws * 1.4).min(px + ws * 3.6);
            let wx1 = (px + ws * 1.4).max(px + ws * 3.6);
            emit_solid_box(
                sink,
                batch,
                w_uv,
                [wx0, py - 1.2, pz - 0.8],
                [wx1, py - 0.7, pz + 0.9],
                [px + ws * 1.4, py - 0.9, pz],
                [0.0, 0.0, ws * p_flap],
                def.secondary,
                235,
                def.id == 3 || def.id == 9 || def.id == 12,
            );
        }
    }

    // 7. AURAS & TRAILS: 10 orbiting & rising 3D particle prisms around the player!
    if loadout.aura > 0
        && let Some(def) = AURAS_TRAILS.get(usize::from(loadout.aura - 1))
    {
        for i in 0..10 {
            let fi = i as f32;
            let phase = t * 1.65 + fi * (std::f32::consts::TAU / 10.0);
            let (s, c) = phase.sin_cos();
            let rad = 7.6 + (t * 2.1 + fi).sin() * 1.4;
            let rise = ((t * 0.55 + fi * 0.13).fract() * 28.0) - 16.0;
            let ax = c * rad;
            let ay = rise;
            let az = s * rad;
            let r = 0.62 + (fi * 1.7).sin().abs() * 0.32;
            let col = if def.id == 12 {
                hue_cycle_rgb(t * 2.4 + fi * 0.6)
            } else if i % 3 == 0 {
                def.accent
            } else if i % 2 == 0 {
                def.primary
            } else {
                def.secondary
            };
            emit_solid_box(
                sink,
                batch,
                w_uv,
                [ax - r, ay - r, az - r],
                [ax + r, ay + r, az + r],
                [ax, ay, az],
                [t * 2.2 + fi, t * 1.7, 0.4],
                col,
                235,
                true,
            );
        }
    }
}

/// Renders a 3D player character with textured skin + all equipped cosmetics in
/// the left Studio viewport.
fn draw_studio_character(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    skin_sprite: Rect,
    box_rect: [f32; 4],
    studio: &CosmeticsStudioState,
    loadout: &VoltCosmeticsLoadout,
    time_s: f32,
) {
    let [bx, by, bw, bh] = box_rect;
    let cx = bx + bw * 0.5;
    let cy = by + bh * 0.54;
    let scale = ((bh / 42.0).min(bw / 28.0)).clamp(1.6, 5.2);
    let pose = PoseTransform::studio(cx, cy, scale, studio.yaw_deg, studio.pitch_deg);
    let sink = QuadSink::Studio(pose);

    let start = batch.vertices.len();
    let idle = (time_s * 1.8).sin() * 0.06;

    // Helper for textured Minecraft 64x64 skin cuboids
    let skin_uv = |px: [f32; 4], flip_x: bool| -> [[f32; 2]; 4] {
        let du = skin_sprite.u1 - skin_sprite.u0;
        let dv = skin_sprite.v1 - skin_sprite.v0;
        let mut u0 = skin_sprite.u0 + (px[0] / 64.0) * du;
        let mut u1 = skin_sprite.u0 + (px[2] / 64.0) * du;
        if flip_x {
            std::mem::swap(&mut u0, &mut u1);
        }
        let v0 = skin_sprite.v0 + (px[1] / 64.0) * dv;
        let v1 = skin_sprite.v0 + (px[3] / 64.0) * dv;
        [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
    };

    let emit_skin_part = |batch: &mut HudBatch,
                          min: [f32; 3],
                          max: [f32; 3],
                          pivot: [f32; 3],
                          ang: [f32; 3],
                          uvs: [[f32; 4]; 6]| {
        let [x0, y0, z0] = min;
        let [x1, y1, z1] = max;
        let faces: [([[f32; 3]; 4], [f32; 3], usize, bool); 6] = [
            ([[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]], [0.0, 0.0, 1.0], 0, true),
            ([[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]], [0.0, 0.0, -1.0], 1, true),
            ([[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]], [1.0, 0.0, 0.0], 2, true),
            ([[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]], [-1.0, 0.0, 0.0], 3, true),
            ([[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]], [0.0, -1.0, 0.0], 4, false),
            ([[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]], [0.0, 1.0, 0.0], 5, false),
        ];
        for (verts, norm, ui, flip) in faces {
            let rv = verts.map(|v| rot_point(v, pivot, ang[0], ang[1], ang[2]));
            let rn = rot_point(norm, [0.0; 3], ang[0], ang[1], ang[2]);
            sink.emit_quad(batch, rv, rn, skin_uv(uvs[ui], flip), [255, 255, 255], 255, false);
        }
    };

    // Head + Hat overlay
    emit_skin_part(
        batch,
        [-4.0, -16.0, -4.0],
        [4.0, -8.0, 4.0],
        [0.0, -8.0, 0.0],
        [0.0, 0.0, 0.0],
        [[8.0, 8.0, 16.0, 16.0], [24.0, 8.0, 32.0, 16.0], [16.0, 8.0, 24.0, 16.0], [0.0, 8.0, 8.0, 16.0], [8.0, 0.0, 16.0, 8.0], [16.0, 0.0, 24.0, 8.0]],
    );
    emit_skin_part(
        batch,
        [-4.35, -16.35, -4.35],
        [4.35, -7.65, 4.35],
        [0.0, -8.0, 0.0],
        [0.0; 3],
        [[40.0, 8.0, 48.0, 16.0], [56.0, 8.0, 64.0, 16.0], [48.0, 8.0, 56.0, 16.0], [32.0, 8.0, 40.0, 16.0], [40.0, 0.0, 48.0, 8.0], [48.0, 0.0, 56.0, 8.0]],
    );
    // Torso
    emit_skin_part(
        batch,
        [-4.0, -8.0, -2.0],
        [4.0, 4.0, 2.0],
        [0.0; 3],
        [0.0; 3],
        [[20.0, 20.0, 28.0, 32.0], [32.0, 20.0, 40.0, 32.0], [28.0, 20.0, 32.0, 32.0], [16.0, 20.0, 20.0, 32.0], [20.0, 16.0, 28.0, 20.0], [28.0, 16.0, 36.0, 20.0]],
    );
    // Right & Left Arms (gentle breathing sway)
    emit_skin_part(
        batch,
        [-8.0, -8.0, -2.0],
        [-4.0, 4.0, 2.0],
        [-6.0, -6.0, 0.0],
        [idle, 0.0, -0.05],
        [[44.0, 20.0, 48.0, 32.0], [52.0, 20.0, 56.0, 32.0], [48.0, 20.0, 52.0, 32.0], [40.0, 20.0, 44.0, 32.0], [44.0, 16.0, 48.0, 20.0], [48.0, 16.0, 52.0, 20.0]],
    );
    emit_skin_part(
        batch,
        [4.0, -8.0, -2.0],
        [8.0, 4.0, 2.0],
        [6.0, -6.0, 0.0],
        [-idle, 0.0, 0.05],
        [[36.0, 52.0, 40.0, 64.0], [44.0, 52.0, 48.0, 64.0], [40.0, 52.0, 44.0, 64.0], [32.0, 52.0, 36.0, 64.0], [36.0, 48.0, 40.0, 52.0], [40.0, 48.0, 44.0, 52.0]],
    );
    // Right & Left Legs
    emit_skin_part(
        batch,
        [-4.0, 4.0, -2.0],
        [0.0, 16.0, 2.0],
        [-2.0, 4.0, 0.0],
        [0.0; 3],
        [[4.0, 20.0, 8.0, 32.0], [12.0, 20.0, 16.0, 32.0], [8.0, 20.0, 12.0, 32.0], [0.0, 20.0, 4.0, 32.0], [4.0, 16.0, 8.0, 20.0], [8.0, 16.0, 12.0, 20.0]],
    );
    emit_skin_part(
        batch,
        [0.0, 4.0, -2.0],
        [4.0, 16.0, 2.0],
        [2.0, 4.0, 0.0],
        [0.0; 3],
        [[20.0, 52.0, 24.0, 64.0], [28.0, 52.0, 32.0, 64.0], [24.0, 52.0, 28.0, 64.0], [16.0, 52.0, 20.0, 64.0], [20.0, 48.0, 24.0, 52.0], [24.0, 48.0, 28.0, 52.0]],
    );

    emit_equipped_cosmetics(&sink, batch, atlas, loadout, time_s, 0.18);
    batch.depth_since(start, [bx, by, bx + bw, by + bh]);
}

/// Draws the full split-screen **Volt Cosmetics** studio on `Page::VoltCosmetics`.
pub fn draw_cosmetics_studio(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    studio: &CosmeticsStudioState,
    loadout: &VoltCosmeticsLoadout,
    skin_key: Option<&str>,
    time_s: f32,
) {
    let lay = StudioLayout::compute(batch.width, batch.height);

    let panel_bg = hud::rgba(14, 15, 22, 222);
    let panel_border = hud::rgba(212, 42, 62, 196);
    let inner_bg = hud::rgba(9, 10, 16, 232);

    let draw_framed = |batch: &mut HudBatch, [x, y, w, h]: [f32; 4], bg: u32, border: u32| {
        batch.fill(atlas, x, y, w, h, bg);
        batch.fill(atlas, x, y, w, 1.0, border);
        batch.fill(atlas, x, y + h - 1.0, w, 1.0, border);
        batch.fill(atlas, x, y, 1.0, h, border);
        batch.fill(atlas, x + w - 1.0, y, 1.0, h, border);
    };

    // Left Pane: 3D Rotatable Character Showcase
    draw_framed(batch, lay.left_box, panel_bg, panel_border);
    draw_framed(batch, lay.model_box, inner_bg, hud::rgba(64, 70, 92, 180));

    // Pedestal glow bar inside model viewport
    let [mx, my, mw, mh] = lay.model_box;
    batch.fill(
        atlas,
        mx + mw * 0.18,
        my + mh * 0.86,
        mw * 0.64,
        2.0,
        hud::rgba(218, 44, 64, 160),
    );

    let skin_rect = skin_key
        .and_then(|k| atlas.sprites.get(k).copied())
        .or_else(|| atlas.sprites.get("entity/player/wide/steve").copied())
        .or_else(|| atlas.sprites.get("entity/player/wide/alex").copied())
        .unwrap_or(atlas.white);

    draw_studio_character(batch, atlas, skin_rect, lay.model_box, studio, loadout, time_s);

    batch.text(
        atlas,
        "Drag 360\u{00b0}",
        mx + 4.0,
        my + 4.0,
        hud::rgba(196, 204, 224, 210),
        true,
    );
    let eq_badge = format!("{}/7 Active", loadout.equipped_count());
    let eq_w = atlas.font.width(&eq_badge);
    batch.text(
        atlas,
        &eq_badge,
        (mx + mw - eq_w - 4.0).max(mx + 4.0),
        my + 4.0,
        hud::rgba(255, 208, 92, 240),
        true,
    );

    let draw_mini_btn = |batch: &mut HudBatch, r: [f32; 4], label: &str, active: bool| {
        let bg = if active {
            hud::rgba(192, 34, 54, 235)
        } else {
            hud::rgba(32, 35, 48, 235)
        };
        let brd = if active {
            hud::rgba(255, 138, 152, 245)
        } else {
            hud::rgba(88, 96, 122, 210)
        };
        draw_framed(batch, r, bg, brd);
        let tw = atlas.font.width(label);
        batch.text(
            atlas,
            label,
            r[0] + (r[2] - tw) * 0.5,
            r[1] + (r[3] - 8.0) * 0.5,
            hud::rgba(248, 248, 252, 255),
            true,
        );
    };
    draw_mini_btn(batch, lay.btn_front, "Front", false);
    draw_mini_btn(batch, lay.btn_back, "Back", false);
    draw_mini_btn(batch, lay.btn_spin, "Spin", studio.auto_spin);

    // Right Pane: 7 Category Tabs + Scrollable Animated Cosmetic Catalog
    draw_framed(batch, lay.right_box, panel_bg, panel_border);

    for (i, &cat) in CosmeticCategory::ALL.iter().enumerate() {
        let tr = lay.tab_rect(i);
        let selected = studio.category == cat;
        let has_item = loadout.get(cat) != 0;
        let bg = if selected {
            hud::rgba(196, 32, 52, 242)
        } else if has_item {
            hud::rgba(44, 52, 42, 228)
        } else {
            hud::rgba(28, 30, 42, 225)
        };
        let brd = if selected {
            hud::rgba(255, 168, 178, 255)
        } else if has_item {
            hud::rgba(104, 224, 132, 220)
        } else {
            hud::rgba(74, 80, 102, 195)
        };
        draw_framed(batch, tr, bg, brd);
        let lbl = cat.short_label();
        let tw = atlas.font.width(lbl);
        batch.text(
            atlas,
            lbl,
            tr[0] + (tr[2] - tw) * 0.5,
            tr[1] + 5.0,
            hud::rgba(250, 250, 254, 255),
            true,
        );
    }

    let [lx, ly, lw, lh] = lay.list_box;
    batch.text(
        atlas,
        studio.category.title(),
        lx + 2.0,
        ly - 12.0,
        hud::rgba(255, 214, 108, 255),
        true,
    );
    draw_framed(batch, lay.list_box, inner_bg, hud::rgba(64, 70, 92, 180));

    let items = items_for(studio.category);
    let active_id = loadout.get(studio.category);
    let scroll = studio.scroll[studio.category.index()];
    let row_h = 25.0f32;

    for row in 0..=items.len() {
        let ry = ly + 2.0 + row as f32 * row_h - scroll;
        if ry + 23.0 <= ly + 1.0 || ry >= ly + lh - 1.0 {
            continue;
        }
        let card_top = ry.max(ly + 1.0);
        let card_bot = (ry + 23.0).min(ly + lh - 1.0);
        let card_h = card_bot - card_top;
        if card_h < 6.0 {
            continue;
        }
        let is_eq = if row == 0 {
            active_id == 0
        } else {
            active_id == items[row - 1].id
        };
        let row_bg = if is_eq {
            hud::rgba(62, 24, 34, 232)
        } else if row % 2 == 0 {
            hud::rgba(20, 22, 32, 215)
        } else {
            hud::rgba(26, 28, 40, 215)
        };
        let row_brd = if is_eq {
            hud::rgba(248, 72, 92, 245)
        } else {
            hud::rgba(52, 58, 78, 175)
        };
        draw_framed(batch, [lx + 3.0, card_top, lw - 10.0, card_h], row_bg, row_brd);

        if ry >= ly && ry + 21.0 <= ly + lh {
            if row == 0 {
                batch.fill(atlas, lx + 7.0, ry + 4.0, 14.0, 14.0, hud::rgba(68, 72, 88, 255));
                batch.text(
                    atlas,
                    "None (Unequipped)",
                    lx + 26.0,
                    ry + 7.0,
                    hud::rgba(210, 214, 228, 255),
                    true,
                );
            } else {
                let def = &items[row - 1];
                let pulse = ((time_s * 3.0 + row as f32 * 0.5).sin() * 0.5 + 0.5) * 0.35;
                let sw = lerp_rgb(def.primary, def.accent, pulse);
                batch.fill(
                    atlas,
                    lx + 7.0,
                    ry + 3.0,
                    15.0,
                    15.0,
                    hud::rgba(sw[0], sw[1], sw[2], 255),
                );
                batch.fill(
                    atlas,
                    lx + 15.0,
                    ry + 11.0,
                    7.0,
                    7.0,
                    hud::rgba(def.secondary[0], def.secondary[1], def.secondary[2], 255),
                );
                batch.text(
                    atlas,
                    def.name,
                    lx + 27.0,
                    ry + 3.0,
                    hud::rgba(250, 250, 255, 255),
                    true,
                );
                batch.text(
                    atlas,
                    def.subtitle,
                    lx + 27.0,
                    ry + 12.5,
                    hud::rgba(162, 172, 196, 235),
                    true,
                );
            }
            if is_eq {
                let tag = "ON";
                let tw = atlas.font.width(tag);
                batch.text(
                    atlas,
                    tag,
                    lx + lw - tw - 13.0,
                    ry + 7.0,
                    hud::rgba(112, 248, 148, 255),
                    true,
                );
            }
        }
    }

    // Scrollbar thumb
    let total_h = (items.len() + 1) as f32 * row_h;
    if total_h > lh {
        let thumb_h = ((lh / total_h) * (lh - 4.0)).clamp(14.0, lh - 4.0);
        let max_s = (total_h - lh + 6.0).max(1.0);
        let ty = ly + 2.0 + (scroll / max_s).clamp(0.0, 1.0) * (lh - 4.0 - thumb_h);
        batch.fill(atlas, lx + lw - 5.0, ty, 3.0, thumb_h, hud::rgba(218, 52, 74, 220));
    }

    let vis_lbl = if loadout.show_in_world {
        "3rd-Person View: ON"
    } else {
        "3rd-Person View: OFF"
    };
    draw_mini_btn(batch, lay.btn_world_vis, vis_lbl, loadout.show_in_world);
    draw_mini_btn(batch, lay.btn_clear, "Unequip All", false);
}

/// Projects equipped 3D Cosmetics onto the local player during 3rd-person (`F5`) gameplay!
#[allow(clippy::too_many_arguments)]
pub fn draw_world_player_cosmetics(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    loadout: &VoltCosmeticsLoadout,
    view_proj: &[[f32; 4]; 4],
    camera_pos: [f64; 3],
    player_feet: [f64; 3],
    body_yaw_deg: f32,
    move_speed: f32,
    sneaking: bool,
    time_s: f32,
) {
    if !loadout.show_in_world || !loadout.any_equipped() {
        return;
    }
    let rel = [
        (player_feet[0] - camera_pos[0]) as f32,
        (player_feet[1] - camera_pos[1]) as f32 - if sneaking { 0.22 } else { 0.0 },
        (player_feet[2] - camera_pos[2]) as f32,
    ];
    let dist_sq = rel[0] * rel[0] + rel[1] * rel[1] + rel[2] * rel[2];
    if !(0.20..=4096.0).contains(&dist_sq) {
        return;
    }
    let (body_yaw_sin, body_yaw_cos) = (body_yaw_deg * DEG).sin_cos();
    let sink = QuadSink::World {
        view_proj,
        player_rel: rel,
        body_yaw_sin,
        body_yaw_cos,
        gui_w: batch.width,
        gui_h: batch.height,
    };
    let start = batch.vertices.len();
    emit_equipped_cosmetics(&sink, batch, atlas, loadout, time_s, move_speed);
    batch.depth_since(start, [0.0, 0.0, batch.width, batch.height]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cosmetic_category_has_at_least_twelve_animated_items() {
        let mut total = 0usize;
        for cat in CosmeticCategory::ALL {
            let list = items_for(cat);
            assert!(
                list.len() >= 12,
                "{cat:?} only had {} items",
                list.len()
            );
            for (idx, item) in list.iter().enumerate() {
                assert_eq!(usize::from(item.id), idx + 1);
                assert!(!item.name.is_empty());
            }
            total += list.len();
        }
        assert_eq!(total, 86);
    }

    #[test]
    fn loadout_roundtrips_and_clamps_invalid_ids() {
        let mut l = VoltCosmeticsLoadout::default();
        assert!(!l.any_equipped());
        l.set(CosmeticCategory::Capes, 14);
        l.set(CosmeticCategory::Wings, 99);
        assert_eq!(l.cape, 14);
        assert_eq!(l.wings, 12);
        assert_eq!(l.equipped_count(), 2);
        l.clear_all();
        assert!(!l.any_equipped());
    }
}
