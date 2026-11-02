//! Supported Minecraft Java Edition versions in Volt Launcher.
//!
//! Currently targets `26.3` (`GameVersion::V26_3`), structured so `26.4` can be
//! added directly alongside `V26_3` when released.

use std::sync::atomic::{AtomicU8, Ordering};

/// Supported Minecraft Java Edition versions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum GameVersion {
    #[default]
    V26_3 = 0,
}

static ACTIVE_VERSION: AtomicU8 = AtomicU8::new(GameVersion::V26_3 as u8);

/// Sets the active Minecraft version for the current session.
pub fn set_active_version(version: GameVersion) {
    ACTIVE_VERSION.store(version as u8, Ordering::Relaxed);
}

/// Returns the active Minecraft version for the current session.
pub fn active_version() -> GameVersion {
    match ACTIVE_VERSION.load(Ordering::Relaxed) {
        _ => GameVersion::V26_3,
    }
}

impl GameVersion {
    /// All supported Minecraft Java Edition versions (oldest to newest).
    /// Future `26.4` drops in here after `Self::V26_3`.
    pub const ALL: &'static [Self] = &[Self::V26_3];

    /// All supported Minecraft Java Edition versions (newest to oldest).
    pub const ALL_NEWEST_FIRST: &'static [Self] = &[Self::V26_3];

    /// Mojang version string (e.g. `"26.3"`).
    pub const fn id(self) -> &'static str {
        match self {
            Self::V26_3 => "26.3",
        }
    }

    /// Network protocol number.
    pub const fn protocol(self) -> i32 {
        match self {
            Self::V26_3 => 777,
        }
    }

    /// World `DataVersion`.
    pub const fn data_version(self) -> i32 {
        match self {
            Self::V26_3 => 5023,
        }
    }

    /// Resource pack `(major, minor)` format version.
    pub const fn resource_pack_format(self) -> (u32, u32) {
        match self {
            Self::V26_3 => (84, 0),
        }
    }

    /// Parses a supported version string. Legacy pre-`26.3` strings return `None`
    /// so saved configs cleanly fall back to `Self::V26_3`.
    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim() {
            "26.3" => Some(Self::V26_3),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_26_3_is_present_and_legacy_ids_normalize_to_default() {
        assert_eq!(GameVersion::ALL, &[GameVersion::V26_3]);
        assert_eq!(GameVersion::ALL_NEWEST_FIRST, &[GameVersion::V26_3]);
        assert_eq!(GameVersion::V26_3.id(), crate::VANILLA_VERSION);
        assert_eq!(GameVersion::from_id("26.3"), Some(GameVersion::V26_3));
        for removed in [
            "1.20.1", "1.20.4", "1.20.6", "1.21", "1.21.1", "1.21.4", "1.21.8", "1.21.11", "26.1",
            "26.2",
        ] {
            assert_eq!(GameVersion::from_id(removed), None);
        }
    }
}
