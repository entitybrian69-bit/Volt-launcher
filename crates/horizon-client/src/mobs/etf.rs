//! Entity Texture Features layer, built into Volt Launcher (no settings
//! row anywhere: an engine layer, not a user-facing mod). The core of
//! ETF-style texture features is a STABLE per-entity randomizer: every
//! entity keeps the same "personality" for its whole life, so animation
//! variance, glance timing and flourish phases never pop between frames
//! or re-rolls. Inspired by Traben's Entity Texture Features (reimplemented
//! from its documented behaviour; no upstream code is used). Features that
//! need custom texture assets (blinking, random skins from OptiFine packs)
//! are intentionally absent: Volt ships no third-party textures.

/// A stable pseudo-random in `0..1` for an entity id and a salt: the same
/// entity and salt always produce the same value (SplitMix64-style mixing,
/// which passes avalanche tests and is cheap enough for render code).
pub fn entity_random(id: u32, salt: u32) -> f32 {
    let mut z = (u64::from(id) << 32 | u64::from(salt)).wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    // The top 24 bits, mapped to [0, 1).
    (z >> 40) as f32 / 16_777_216.0
}

/// A stable value in `lo..hi` for an entity id and salt.
pub fn entity_range(id: u32, salt: u32, lo: f32, hi: f32) -> f32 {
    lo + (hi - lo) * entity_random(id, salt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_random_is_stable_and_spread() {
        // Stability: the same id and salt always agree.
        for id in [0u32, 1, 77, u32::MAX] {
            for salt in [0u32, 9, 1234] {
                assert_eq!(entity_random(id, salt), entity_random(id, salt));
            }
        }
        // Different salts decorrelate the same entity.
        assert_ne!(entity_random(42, 0), entity_random(42, 1));
        // Spread: over many ids the values cover the unit interval evenly
        // enough that personalities differ visibly between entities.
        let mut lo = 0usize;
        let mut hi = 0usize;
        for id in 0..1000u32 {
            let v = entity_random(id, 7);
            assert!((0.0..1.0).contains(&v));
            if v < 0.25 {
                lo += 1;
            }
            if v > 0.75 {
                hi += 1;
            }
        }
        assert!(lo > 150 && hi > 150, "quartiles populated: {lo} {hi}");
    }

    #[test]
    fn entity_range_maps_into_the_interval() {
        for id in 0..100u32 {
            let v = entity_range(id, 3, 0.85, 1.15);
            assert!((0.85..1.15).contains(&v));
        }
    }
}
