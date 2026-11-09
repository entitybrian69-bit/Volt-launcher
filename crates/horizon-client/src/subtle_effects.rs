//! Subtle Effects, built into Volt Launcher: small ambient particle touches
//! layered over vanilla's — rising sparks from torches, candles and
//! campfires, glowing dust drifting off glowstone and redstone blocks,
//! ember sparks on burning mobs, and slime trails behind slimes. Every
//! effect uses vanilla particle sprites, is purely visual, and can be
//! switched off per effect with a density setting for weak devices.
//!
//! A clean-room recreation inspired by the feature set of MincraftEinstein's
//! Subtle Effects for Minecraft (All Rights Reserved — no code or assets
//! used). The full feature-by-feature parity table is in
//! docs/BUILTIN-PARITY.md; credits in THIRD_PARTY_NOTICES.md.
use crate::particles::{ParticleEngine, ParticleOptions, ParticleType as T};
use horizon_core::random::{LegacyRandom, RandomSource};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const ID: &str = "subtle_effects";
pub const NAME: &str = "Subtle Effects";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Small ambient particle touches: sparks rising from torches, \
candles and campfires, glowing dust from glowstone and redstone blocks, embers on burning \
mobs and slime trails. Vanilla sprites only, purely visual, with a density setting.";

static ENABLED: AtomicBool = AtomicBool::new(true);
static SPARKS: AtomicBool = AtomicBool::new(true);
static DUST: AtomicBool = AtomicBool::new(true);
static BURNING: AtomicBool = AtomicBool::new(true);
static SLIMES: AtomicBool = AtomicBool::new(true);
/// 0 = Low, 1 = Medium, 2 = High.
static DENSITY: AtomicUsize = AtomicUsize::new(1);
/// Set while the mod's row in Options > Mods is switched off entirely.
static SUSPENDED: AtomicBool = AtomicBool::new(false);
pub fn set_suspended(off: bool) {
    SUSPENDED.store(off, Ordering::Relaxed);
}
fn active() -> bool {
    !SUSPENDED.load(Ordering::Relaxed) && ENABLED.load(Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flags {
    pub enabled: bool,
    pub sparks: bool,
    pub dust: bool,
    pub burning: bool,
    pub slimes: bool,
    pub density: usize,
}
pub fn flags() -> Flags {
    Flags {
        enabled: ENABLED.load(Ordering::Relaxed),
        sparks: SPARKS.load(Ordering::Relaxed),
        dust: DUST.load(Ordering::Relaxed),
        burning: BURNING.load(Ordering::Relaxed),
        slimes: SLIMES.load(Ordering::Relaxed),
        density: DENSITY.load(Ordering::Relaxed).min(2),
    }
}
pub fn set_flags(f: Flags) {
    ENABLED.store(f.enabled, Ordering::Relaxed);
    SPARKS.store(f.sparks, Ordering::Relaxed);
    DUST.store(f.dust, Ordering::Relaxed);
    BURNING.store(f.burning, Ordering::Relaxed);
    SLIMES.store(f.slimes, Ordering::Relaxed);
    DENSITY.store(f.density.min(2), Ordering::Relaxed);
}
pub const DENSITY_NAMES: [&str; 3] = ["Low", "Medium", "High"];
/// How spawn chances scale with the density setting.
pub fn density_scale(density: usize) -> f32 {
    [0.5, 1.0, 2.0][density.min(2)]
}

/// Whether a `next_float` roll passes `base` odds at the current density.
fn roll(r: &mut LegacyRandom, base: f32) -> bool {
    r.next_float() < (base * density_scale(flags().density)).min(1.0)
}

/// A rising spark from a small flame (torches, candles): called by
/// `animate_tick` right where vanilla spawns the flame particle.
pub fn flame_spark(engine: &mut ParticleEngine, p: [f64; 3]) {
    let f = flags();
    if !active() || !f.sparks || !roll(engine.random(), 0.55) {
        return;
    }
    let r = engine.random();
    let v = [
        f64::from(r.next_float() - 0.5) * 0.025,
        0.035 + f64::from(r.next_float()) * 0.045,
        f64::from(r.next_float() - 0.5) * 0.025,
    ];
    let at = [p[0], p[1] + 0.05, p[2]];
    engine.add(&ParticleOptions::simple(T::SmallFlame), at, v);
}

/// Extra embers over a lit campfire, beyond vanilla's occasional lava pop.
pub fn campfire_sparks(engine: &mut ParticleEngine, p: [f64; 3]) {
    let f = flags();
    if !active() || !f.sparks || !roll(engine.random(), 0.7) {
        return;
    }
    // A short column of full-size embers, clearly taller than vanilla's
    // occasional pop, plus a rare arcing lava spark.
    for _ in 0..2 {
        let r = engine.random();
        let at = [
            p[0] + f64::from(r.next_float() - 0.5) * 0.5,
            p[1] + 0.3 + f64::from(r.next_float()) * 0.2,
            p[2] + f64::from(r.next_float() - 0.5) * 0.5,
        ];
        let v = [
            f64::from(r.next_float() - 0.5) * 0.03,
            0.06 + f64::from(r.next_float()) * 0.07,
            f64::from(r.next_float() - 0.5) * 0.03,
        ];
        engine.add(&ParticleOptions::simple(T::Flame), at, v);
    }
    if engine.random().next_float() < 0.15 {
        let at = [p[0], p[1] + 0.4, p[2]];
        engine.add(&ParticleOptions::simple(T::Lava), at, [0.0; 3]);
    }
}

/// The glowing dust a block sheds, if any: colour, size and base odds.
pub fn block_dust(name: &str) -> Option<([f32; 3], f32, f32)> {
    match name {
        "minecraft:glowstone" => Some(([0.98, 0.83, 0.42], 0.9, 0.25)),
        "minecraft:redstone_block" => Some(([0.86, 0.09, 0.04], 0.9, 0.25)),
        _ => None,
    }
}
pub fn dust_on() -> bool {
    active() && flags().dust
}
/// Ambient dust drifting off glowstone and redstone blocks; `animate_tick`
/// calls it for each sampled block with its registry name.
pub fn ambient_dust(engine: &mut ParticleEngine, name: &str, pos: [i32; 3]) {
    let Some((rgb, scale, base)) = block_dust(name) else {
        return;
    };
    if !roll(engine.random(), base) {
        return;
    }
    let r = engine.random();
    // A random point on a random face, drifting slowly down.
    let (u, v) = (f64::from(r.next_float()), f64::from(r.next_float()));
    let face = next(r, 6);
    let (fx, fy, fz) = (f64::from(pos[0]), f64::from(pos[1]), f64::from(pos[2]));
    let o = 1.05;
    let p = match face {
        0 => [fx + u, fy - 0.05, fz + v],
        1 => [fx + u, fy + o, fz + v],
        2 => [fx + u, fy + v, fz - 0.05],
        3 => [fx + u, fy + v, fz + o],
        4 => [fx - 0.05, fy + u, fz + v],
        _ => [fx + o, fy + u, fz + v],
    };
    engine.add(&ParticleOptions::dust(rgb, scale), p, [0.0, -0.01, 0.0]);
}
fn next(r: &mut LegacyRandom, bound: i32) -> i32 {
    r.next_int_bound(bound).unwrap_or(0)
}

/// Ember sparks and smoke on a burning mob: the per-tick call gets the
/// mob's feet position and its size, and spawns within the body.
pub fn burning_mob(engine: &mut ParticleEngine, feet: [f64; 3], width: f64, height: f64) {
    let f = flags();
    if !active() || !f.burning || !roll(engine.random(), 0.8) {
        return;
    }
    // One ember and one smoke puff per pass: unmistakable at a glance.
    for ty in [T::Flame, T::Smoke] {
        let r = engine.random();
        let p = [
            feet[0] + f64::from(r.next_float() - 0.5) * width,
            feet[1] + f64::from(r.next_float()) * height,
            feet[2] + f64::from(r.next_float() - 0.5) * width,
        ];
        let v = [
            f64::from(r.next_float() - 0.5) * 0.03,
            0.045 + f64::from(r.next_float()) * 0.055,
            f64::from(r.next_float() - 0.5) * 0.03,
        ];
        engine.add(&ParticleOptions::simple(ty), p, v);
    }
}

/// A slime trail blob under a slime that is moving on the ground.
pub fn slime_trail(engine: &mut ParticleEngine, feet: [f64; 3], width: f64) {
    let f = flags();
    if !active() || !f.slimes || !roll(engine.random(), 0.55) {
        return;
    }
    for _ in 0..2 {
        let r = engine.random();
        let p = [
            feet[0] + f64::from(r.next_float() - 0.5) * width * 0.9,
            feet[1] + 0.02,
            feet[2] + f64::from(r.next_float() - 0.5) * width * 0.9,
        ];
        engine.add(&ParticleOptions::simple(T::ItemSlime), p, [0.0; 3]);
    }
}
/// Whether a mob kind leaves slime trails.
pub fn is_slime(kind: &str) -> bool {
    kind == "minecraft:slime"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::path::Path;
    use std::sync::Mutex;

    /// Tests that touch the global flags take this lock so they cannot
    /// observe each other's intermediate states.
    static FLAGS_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn every_effect_spawns_clearly_visible_particles() {
        let _guard = FLAGS_LOCK.lock().unwrap();
        let before = flags();
        set_flags(Flags {
            enabled: true,
            sparks: true,
            dust: true,
            burning: true,
            slimes: true,
            density: 1,
        });
        set_suspended(false);
        let mut engine = ParticleEngine::new(Path::new(""), &HashMap::new(), &HashSet::new());
        engine.seed_randoms(7, 7, 7);
        // Capture every `add` the way the parity probe's stand-in level does.
        engine.requested = Some(Vec::new());
        for i in 0..100 {
            let at = [f64::from(i), 64.0, 0.0];
            flame_spark(&mut engine, at);
            campfire_sparks(&mut engine, at);
            ambient_dust(&mut engine, "minecraft:glowstone", [i, 64, 0]);
            burning_mob(&mut engine, at, 0.6, 1.8);
            slime_trail(&mut engine, at, 1.0);
        }
        let got = engine.requested.take().unwrap();
        let count = |ty: T| got.iter().filter(|(o, _, _)| o.ty == ty).count();
        // Torch/candle sparks: at ~55% odds, 100 calls make dozens.
        let small = count(T::SmallFlame);
        assert!(small > 25, "torch sparks spawn often: {small}");
        // Campfire embers are full-size flames, two per pass.
        let embers = count(T::Flame);
        assert!(embers > 60, "campfire + burning embers: {embers}");
        assert!(count(T::Lava) > 0, "the rare arcing lava spark happens");
        assert!(count(T::Dust) > 10, "glowing dust is frequent");
        assert!(count(T::Smoke) > 25, "burning mobs smoke");
        assert!(count(T::ItemSlime) > 25, "slime trails leave blobs");
        // Sparks genuinely rise: every flame's velocity points up, hard.
        for (o, _, v) in &got {
            if matches!(o.ty, T::SmallFlame | T::Flame) {
                assert!(v[1] >= 0.035, "spark rises visibly, vy = {}", v[1]);
            }
        }
        // Dust is big enough to see.
        let (_, scale, _) = block_dust("minecraft:glowstone").unwrap();
        assert!(scale >= 0.9, "dust scaled up to be visible");
        set_flags(before);
    }

    #[test]
    fn density_scales_and_clamps() {
        assert_eq!(density_scale(0), 0.5);
        assert_eq!(density_scale(1), 1.0);
        assert_eq!(density_scale(2), 2.0);
        assert_eq!(density_scale(99), 2.0, "out of range clamps to High");
        assert_eq!(DENSITY_NAMES.len(), 3);
    }

    #[test]
    fn dust_covers_exactly_the_glowing_blocks() {
        assert!(block_dust("minecraft:glowstone").is_some());
        assert!(block_dust("minecraft:redstone_block").is_some());
        assert!(block_dust("minecraft:stone").is_none());
        assert!(block_dust("minecraft:redstone_ore").is_none());
        let (rgb, scale, base) = block_dust("minecraft:glowstone").unwrap();
        assert!(rgb.iter().all(|c| (0.0..=1.0).contains(c)));
        assert!(scale > 0.0 && base > 0.0 && base < 1.0);
    }

    #[test]
    fn flags_round_trip_and_gate() {
        let _guard = FLAGS_LOCK.lock().unwrap();
        let before = flags();
        set_flags(Flags {
            enabled: false,
            sparks: true,
            dust: true,
            burning: true,
            slimes: true,
            density: 7,
        });
        let f = flags();
        assert!(!f.enabled);
        assert_eq!(f.density, 2, "density stored clamped");
        assert!(!dust_on(), "master switch gates dust");
        set_flags(before);
    }

    #[test]
    fn slime_detection_is_exact() {
        assert!(is_slime("minecraft:slime"));
        assert!(!is_slime("minecraft:magma_cube"));
        assert!(!is_slime("minecraft:slime_ball"));
    }
}
