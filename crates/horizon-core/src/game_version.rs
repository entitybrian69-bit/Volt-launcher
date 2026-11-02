//! Pure-Rust multi-version descriptors for every Minecraft Java Edition release `>= 1.20.1`
//! (`1.20.1`..`1.20.6`, `1.21`..`1.21.11`, and `26.1`..`26.3`).
//!
//! Base engine assets remain anchored at [`crate::version::VANILLA_VERSION`] (`"26.3"`),
//! while [`active_version`] selects the active runtime target for protocol negotiation,
//! server status pings, resource-pack `pack_format` compatibility, isolated launcher
//! profile storage, catalog queries, UI labels, and era-accurate item/recipe/mob filtering.

use std::sync::atomic::{AtomicU8, Ordering};

/// All supported pure-Rust Minecraft Java Edition releases (`>= 1.20.1`), ordered
/// chronologically from `1.20.1` (`0`) to `26.3` (`20`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum GameVersion {
    V1_20_1 = 0,
    V1_20_2 = 1,
    V1_20_3 = 2,
    V1_20_4 = 3,
    V1_20_5 = 4,
    V1_20_6 = 5,
    V1_21 = 6,
    V1_21_1 = 7,
    V1_21_2 = 8,
    V1_21_3 = 9,
    V1_21_4 = 10,
    V1_21_5 = 11,
    V1_21_6 = 12,
    V1_21_7 = 13,
    V1_21_8 = 14,
    V1_21_9 = 15,
    V1_21_10 = 16,
    V1_21_11 = 17,
    V26_1 = 18,
    V26_2 = 19,
    #[default]
    V26_3 = 20,
}

static ACTIVE_VERSION: AtomicU8 = AtomicU8::new(GameVersion::V26_3 as u8);

/// Returns the currently selected runtime [`GameVersion`] (defaults to [`GameVersion::V26_3`]).
#[inline]
pub fn active_version() -> GameVersion {
    GameVersion::from_index(ACTIVE_VERSION.load(Ordering::Relaxed)).unwrap_or(GameVersion::V26_3)
}

/// Sets the active runtime [`GameVersion`] across all subsystems.
#[inline]
pub fn set_active_version(version: GameVersion) {
    ACTIVE_VERSION.store(version as u8, Ordering::Relaxed);
}

impl GameVersion {
    /// Every supported release from oldest (`1.20.1`) to newest (`26.3`).
    pub const ALL_OLDEST_FIRST: [Self; 21] = [
        Self::V1_20_1,
        Self::V1_20_2,
        Self::V1_20_3,
        Self::V1_20_4,
        Self::V1_20_5,
        Self::V1_20_6,
        Self::V1_21,
        Self::V1_21_1,
        Self::V1_21_2,
        Self::V1_21_3,
        Self::V1_21_4,
        Self::V1_21_5,
        Self::V1_21_6,
        Self::V1_21_7,
        Self::V1_21_8,
        Self::V1_21_9,
        Self::V1_21_10,
        Self::V1_21_11,
        Self::V26_1,
        Self::V26_2,
        Self::V26_3,
    ];

    /// Every supported release from newest (`26.3`) down to `1.20.1` for UI choosers.
    pub const ALL_NEWEST_FIRST: [Self; 21] = [
        Self::V26_3,
        Self::V26_2,
        Self::V26_1,
        Self::V1_21_11,
        Self::V1_21_10,
        Self::V1_21_9,
        Self::V1_21_8,
        Self::V1_21_7,
        Self::V1_21_6,
        Self::V1_21_5,
        Self::V1_21_4,
        Self::V1_21_3,
        Self::V1_21_2,
        Self::V1_21_1,
        Self::V1_21,
        Self::V1_20_6,
        Self::V1_20_5,
        Self::V1_20_4,
        Self::V1_20_3,
        Self::V1_20_2,
        Self::V1_20_1,
    ];

    #[inline]
    pub const fn from_index(idx: u8) -> Option<Self> {
        match idx {
            0 => Some(Self::V1_20_1),
            1 => Some(Self::V1_20_2),
            2 => Some(Self::V1_20_3),
            3 => Some(Self::V1_20_4),
            4 => Some(Self::V1_20_5),
            5 => Some(Self::V1_20_6),
            6 => Some(Self::V1_21),
            7 => Some(Self::V1_21_1),
            8 => Some(Self::V1_21_2),
            9 => Some(Self::V1_21_3),
            10 => Some(Self::V1_21_4),
            11 => Some(Self::V1_21_5),
            12 => Some(Self::V1_21_6),
            13 => Some(Self::V1_21_7),
            14 => Some(Self::V1_21_8),
            15 => Some(Self::V1_21_9),
            16 => Some(Self::V1_21_10),
            17 => Some(Self::V1_21_11),
            18 => Some(Self::V26_1),
            19 => Some(Self::V26_2),
            20 => Some(Self::V26_3),
            _ => None,
        }
    }

    /// Parses a version string such as `"1.20.1"`, `"1.21"`, `"1.21.4"`, `"1.21.11"`, or `"26.3"`.
    pub fn from_id(s: &str) -> Option<Self> {
        match s.trim() {
            "1.20.1" => Some(Self::V1_20_1),
            "1.20.2" => Some(Self::V1_20_2),
            "1.20.3" => Some(Self::V1_20_3),
            "1.20.4" => Some(Self::V1_20_4),
            "1.20.5" => Some(Self::V1_20_5),
            "1.20.6" => Some(Self::V1_20_6),
            "1.21" | "1.21.0" => Some(Self::V1_21),
            "1.21.1" => Some(Self::V1_21_1),
            "1.21.2" => Some(Self::V1_21_2),
            "1.21.3" => Some(Self::V1_21_3),
            "1.21.4" => Some(Self::V1_21_4),
            "1.21.5" => Some(Self::V1_21_5),
            "1.21.6" => Some(Self::V1_21_6),
            "1.21.7" => Some(Self::V1_21_7),
            "1.21.8" => Some(Self::V1_21_8),
            "1.21.9" => Some(Self::V1_21_9),
            "1.21.10" => Some(Self::V1_21_10),
            "1.21.11" => Some(Self::V1_21_11),
            "26.1" => Some(Self::V26_1),
            "26.2" => Some(Self::V26_2),
            "26.3" => Some(Self::V26_3),
            _ => None,
        }
    }

    /// Official Mojang version identifier (e.g. `"1.20.1"`, `"1.21.4"`, `"26.3"`).
    pub const fn id(self) -> &'static str {
        match self {
            Self::V1_20_1 => "1.20.1",
            Self::V1_20_2 => "1.20.2",
            Self::V1_20_3 => "1.20.3",
            Self::V1_20_4 => "1.20.4",
            Self::V1_20_5 => "1.20.5",
            Self::V1_20_6 => "1.20.6",
            Self::V1_21 => "1.21",
            Self::V1_21_1 => "1.21.1",
            Self::V1_21_2 => "1.21.2",
            Self::V1_21_3 => "1.21.3",
            Self::V1_21_4 => "1.21.4",
            Self::V1_21_5 => "1.21.5",
            Self::V1_21_6 => "1.21.6",
            Self::V1_21_7 => "1.21.7",
            Self::V1_21_8 => "1.21.8",
            Self::V1_21_9 => "1.21.9",
            Self::V1_21_10 => "1.21.10",
            Self::V1_21_11 => "1.21.11",
            Self::V26_1 => "26.1",
            Self::V26_2 => "26.2",
            Self::V26_3 => "26.3",
        }
    }

    /// Human-readable release drop / era name.
    pub const fn era_name(self) -> &'static str {
        match self {
            Self::V1_20_1 => "Trails & Tales (Pure Rust)",
            Self::V1_20_2 => "Trails & Tales • Config Net",
            Self::V1_20_3 | Self::V1_20_4 => "Bats & Pots",
            Self::V1_20_5 | Self::V1_20_6 => "Armored Paws",
            Self::V1_21 | Self::V1_21_1 => "Tricky Trials",
            Self::V1_21_2 | Self::V1_21_3 => "Bundles of Bravery",
            Self::V1_21_4 => "The Garden Awakens",
            Self::V1_21_5 => "Spring to Life",
            Self::V1_21_6 | Self::V1_21_7 | Self::V1_21_8 => "Chase the Skies",
            Self::V1_21_9 | Self::V1_21_10 => "The Copper Age",
            Self::V1_21_11 => "Mounts of Mayhem",
            Self::V26_1 => "2026 Drop 1",
            Self::V26_2 => "2026 Drop 2",
            Self::V26_3 => "Latest Pure-Rust Engine",
        }
    }

    /// Official Java Edition network protocol version (`763`..`777`).
    pub const fn protocol(self) -> i32 {
        match self {
            Self::V1_20_1 => 763,
            Self::V1_20_2 => 764,
            Self::V1_20_3 | Self::V1_20_4 => 765,
            Self::V1_20_5 | Self::V1_20_6 => 766,
            Self::V1_21 | Self::V1_21_1 => 767,
            Self::V1_21_2 | Self::V1_21_3 => 768,
            Self::V1_21_4 => 769,
            Self::V1_21_5 => 770,
            Self::V1_21_6 => 771,
            Self::V1_21_7 | Self::V1_21_8 => 772,
            Self::V1_21_9 | Self::V1_21_10 => 773,
            Self::V1_21_11 => 774,
            Self::V26_1 => 775,
            Self::V26_2 => 776,
            Self::V26_3 => crate::version::PROTOCOL_VERSION,
        }
    }

    /// Official Mojang `DataVersion` (`3465`..`5023`).
    pub const fn data_version(self) -> i32 {
        match self {
            Self::V1_20_1 => 3465,
            Self::V1_20_2 => 3578,
            Self::V1_20_3 => 3698,
            Self::V1_20_4 => 3700,
            Self::V1_20_5 => 3837,
            Self::V1_20_6 => 3839,
            Self::V1_21 => 3953,
            Self::V1_21_1 => 3955,
            Self::V1_21_2 => 4080,
            Self::V1_21_3 => 4082,
            Self::V1_21_4 => 4189,
            Self::V1_21_5 => 4325,
            Self::V1_21_6 => 4435,
            Self::V1_21_7 => 4438,
            Self::V1_21_8 => 4440,
            Self::V1_21_9 => 4554,
            Self::V1_21_10 => 4556,
            Self::V1_21_11 => 4671,
            Self::V26_1 => 4786,
            Self::V26_2 => 4902,
            Self::V26_3 => crate::version::DATA_VERSION,
        }
    }

    /// Official client resource-pack `pack_format` `(major, minor)`.
    pub const fn resource_pack_format(self) -> (i64, i64) {
        match self {
            Self::V1_20_1 => (15, 0),
            Self::V1_20_2 => (18, 0),
            Self::V1_20_3 | Self::V1_20_4 => (22, 0),
            Self::V1_20_5 | Self::V1_20_6 => (32, 0),
            Self::V1_21 | Self::V1_21_1 => (34, 0),
            Self::V1_21_2 | Self::V1_21_3 => (42, 0),
            Self::V1_21_4 => (46, 0),
            Self::V1_21_5 => (55, 0),
            Self::V1_21_6 => (63, 0),
            Self::V1_21_7 | Self::V1_21_8 => (64, 0),
            Self::V1_21_9 | Self::V1_21_10 => (69, 0),
            Self::V1_21_11 => (75, 0),
            Self::V26_1 => (84, 0),
            Self::V26_2 => (91, 0),
            Self::V26_3 => crate::version::RESOURCE_PACK_FORMAT,
        }
    }

    /// Official server data-pack `pack_format` `(major, minor)`.
    pub const fn data_pack_format(self) -> (i64, i64) {
        match self {
            Self::V1_20_1 => (15, 0),
            Self::V1_20_2 => (18, 0),
            Self::V1_20_3 | Self::V1_20_4 => (26, 0),
            Self::V1_20_5 | Self::V1_20_6 => (41, 0),
            Self::V1_21 | Self::V1_21_1 => (48, 0),
            Self::V1_21_2 | Self::V1_21_3 => (57, 0),
            Self::V1_21_4 => (61, 0),
            Self::V1_21_5 => (71, 0),
            Self::V1_21_6 => (80, 0),
            Self::V1_21_7 | Self::V1_21_8 => (81, 0),
            Self::V1_21_9 | Self::V1_21_10 => (88, 0),
            Self::V1_21_11 => (94, 0),
            Self::V26_1 => (101, 0),
            Self::V26_2 => (111, 0),
            Self::V26_3 => crate::version::DATA_PACK_FORMAT,
        }
    }

    /// `true` if the protocol includes the post-1.20.2 `Configuration` connection state (`>= 764`).
    #[inline]
    pub const fn has_configuration_state(self) -> bool {
        self.protocol() >= 764
    }

    /// Returns `true` if `raw_id` (an item or block identifier such as `"minecraft:mace"` or `"crafter"`)
    /// existed in this [`GameVersion`]. Used by Creative tabs, Search, and Crafting recipes.
    pub fn is_item_available(self, raw_id: &str) -> bool {
        if self == Self::V26_3 {
            return true;
        }
        let id = raw_id.strip_prefix("minecraft:").unwrap_or(raw_id);
        self >= item_introduced_in(id)
    }

    /// Returns `true` if `raw_id` (an entity/mob species id) existed in this [`GameVersion`].
    pub fn is_mob_available(self, raw_id: &str) -> bool {
        if self == Self::V26_3 {
            return true;
        }
        let id = raw_id.strip_prefix("minecraft:").unwrap_or(raw_id);
        match id {
            "breeze" => self >= Self::V1_21,
            "armadillo" => self >= Self::V1_20_5,
            "bogged" => self >= Self::V1_21,
            "creaking" => self >= Self::V1_21_4,
            "happy_ghast" => self >= Self::V1_21_6,
            "copper_golem" => self >= Self::V1_21_9,
            "nautilus" | "zombie_nautilus" | "camel_husk" | "parched" => self >= Self::V1_21_11,
            _ => true,
        }
    }
}

/// Maps post-1.20.1 items and blocks to the exact release that introduced them.
fn item_introduced_in(id: &str) -> GameVersion {
    // 26.x additions
    if id.starts_with("sulfur") || id.contains("sulfur_") {
        return GameVersion::V26_1;
    }

    // 1.21.11 (Mounts of Mayhem)
    if id.contains("nautilus")
        || id.ends_with("_spear")
        || id == "spear"
        || id.starts_with("camel_husk")
        || id.starts_with("parched")
    {
        return GameVersion::V1_21_11;
    }

    // 1.21.9 (The Copper Age)
    if id.starts_with("copper_golem")
        || id.ends_with("copper_chest")
        || id.ends_with("copper_bars")
        || id.ends_with("copper_chain")
        || id.ends_with("copper_lantern")
        || matches!(
            id,
            "copper_torch"
                | "copper_nugget"
                | "copper_sword"
                | "copper_pickaxe"
                | "copper_axe"
                | "copper_shovel"
                | "copper_hoe"
                | "copper_helmet"
                | "copper_chestplate"
                | "copper_leggings"
                | "copper_boots"
                | "copper_horse_armor"
        )
        || id.ends_with("_shelf")
    {
        return GameVersion::V1_21_9;
    }

    // 1.21.6 (Chase the Skies)
    if id == "dried_ghast"
        || id == "happy_ghast_spawn_egg"
        || id == "music_disc_tears"
        || id.ends_with("_harness")
    {
        return GameVersion::V1_21_6;
    }
    if id == "music_disc_lava_chicken" {
        return GameVersion::V1_21_7;
    }

    // 1.21.5 (Spring to Life)
    if matches!(
        id,
        "bush"
            | "firefly_bush"
            | "wildflowers"
            | "leaf_litter"
            | "cactus_flower"
            | "short_dry_grass"
            | "tall_dry_grass"
            | "blue_egg"
            | "brown_egg"
            | "test_block"
            | "test_instance_block"
    ) {
        return GameVersion::V1_21_5;
    }

    // 1.21.4 (The Garden Awakens)
    if id.starts_with("pale_oak_")
        || id.starts_with("stripped_pale_oak_")
        || id.starts_with("pale_moss")
        || id == "pale_hanging_moss"
        || id == "open_eyeblossom"
        || id == "closed_eyeblossom"
        || id.starts_with("resin_")
        || id == "chiseled_resin_bricks"
        || id == "creaking_heart"
        || id == "creaking_spawn_egg"
    {
        return GameVersion::V1_21_4;
    }

    // 1.21.2 (Bundles of Bravery)
    if id == "bundle"
        || id.ends_with("_bundle")
        || id == "bordure_indented_banner_pattern"
        || id == "field_masoned_banner_pattern"
    {
        return GameVersion::V1_21_2;
    }

    // 1.20.5 / 1.20.6 (Armored Paws)
    if matches!(
        id,
        "wolf_armor" | "armadillo_scute" | "armadillo_spawn_egg"
    ) {
        return GameVersion::V1_20_5;
    }

    // 1.21 (Tricky Trials — experimental in 1.20.3/1.20.5, standard survival/creative in 1.21)
    if matches!(
        id,
        "crafter"
            | "trial_spawner"
            | "vault"
            | "trial_key"
            | "ominous_trial_key"
            | "ominous_bottle"
            | "breeze_rod"
            | "breeze_spawn_egg"
            | "bogged_spawn_egg"
            | "wind_charge"
            | "heavy_core"
            | "mace"
            | "flow_banner_pattern"
            | "guster_banner_pattern"
            | "flow_armor_trim_smithing_template"
            | "bolt_armor_trim_smithing_template"
            | "flow_pottery_sherd"
            | "guster_pottery_sherd"
            | "scrape_pottery_sherd"
            | "music_disc_creator"
            | "music_disc_creator_music_box"
            | "music_disc_precipice"
            | "tuff_stairs"
            | "tuff_slab"
            | "tuff_wall"
            | "chiseled_tuff"
            | "polished_tuff"
            | "polished_tuff_stairs"
            | "polished_tuff_slab"
            | "polished_tuff_wall"
            | "tuff_bricks"
            | "tuff_brick_stairs"
            | "tuff_brick_slab"
            | "tuff_brick_wall"
            | "chiseled_tuff_bricks"
    ) || id.ends_with("copper_bulb")
        || id.ends_with("copper_grate")
        || id.ends_with("copper_door")
        || id.ends_with("copper_trapdoor")
        || id.ends_with("chiseled_copper")
    {
        return GameVersion::V1_21;
    }

    GameVersion::V1_20_1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_versions_round_trip_and_ascend() {
        let mut prev_proto = 0;
        let mut prev_data = 0;
        for (idx, v) in GameVersion::ALL_OLDEST_FIRST.iter().copied().enumerate() {
            assert_eq!(GameVersion::from_index(idx as u8), Some(v));
            assert_eq!(GameVersion::from_id(v.id()), Some(v));
            assert!(v.protocol() >= prev_proto);
            assert!(v.data_version() > prev_data);
            prev_proto = v.protocol();
            prev_data = v.data_version();
        }
        assert_eq!(GameVersion::V1_20_1.protocol(), 763);
        assert_eq!(GameVersion::V1_20_1.resource_pack_format(), (15, 0));
        assert!(!GameVersion::V1_20_1.has_configuration_state());
        assert!(GameVersion::V1_20_2.has_configuration_state());
        assert_eq!(GameVersion::V1_21.protocol(), 767);
        assert_eq!(GameVersion::V1_21_4.protocol(), 769);
        assert_eq!(GameVersion::V1_21_11.protocol(), 774);
        assert_eq!(GameVersion::V26_3.protocol(), 777);
    }

    #[test]
    fn era_item_gating_matches_mojang_releases() {
        // 1.20.1 has Cherry Grove / Sniffer / Trim templates, NOT Mace / Pale Oak / Copper Sword / Spear
        assert!(GameVersion::V1_20_1.is_item_available("minecraft:cherry_log"));
        assert!(GameVersion::V1_20_1.is_item_available("minecraft:sniffer_egg"));
        assert!(!GameVersion::V1_20_1.is_item_available("minecraft:wolf_armor"));
        assert!(!GameVersion::V1_20_1.is_item_available("minecraft:mace"));
        assert!(!GameVersion::V1_20_1.is_item_available("minecraft:pale_oak_log"));
        assert!(!GameVersion::V1_20_1.is_item_available("minecraft:copper_sword"));
        assert!(!GameVersion::V1_20_1.is_item_available("minecraft:iron_spear"));

        // 1.20.5 adds wolf_armor, still before Tricky Trials Mace
        assert!(GameVersion::V1_20_5.is_item_available("minecraft:wolf_armor"));
        assert!(!GameVersion::V1_20_5.is_item_available("minecraft:mace"));

        // 1.21.1 adds Mace & Crafter, still before Pale Oak
        assert!(GameVersion::V1_21_1.is_item_available("minecraft:mace"));
        assert!(GameVersion::V1_21_1.is_item_available("minecraft:crafter"));
        assert!(!GameVersion::V1_21_1.is_item_available("minecraft:pale_oak_log"));

        // 1.21.4 adds Pale Oak & Creaking Heart
        assert!(GameVersion::V1_21_4.is_item_available("minecraft:pale_oak_log"));
        assert!(GameVersion::V1_21_4.is_item_available("minecraft:creaking_heart"));
        assert!(!GameVersion::V1_21_4.is_item_available("minecraft:copper_sword"));

        // 1.21.9 adds Copper Age tools/blocks, 1.21.11 adds Mounts of Mayhem spears
        assert!(GameVersion::V1_21_9.is_item_available("minecraft:copper_sword"));
        assert!(!GameVersion::V1_21_9.is_item_available("minecraft:iron_spear"));
        assert!(GameVersion::V1_21_11.is_item_available("minecraft:iron_spear"));
        assert!(GameVersion::V26_3.is_item_available("minecraft:iron_spear"));
    }
}
