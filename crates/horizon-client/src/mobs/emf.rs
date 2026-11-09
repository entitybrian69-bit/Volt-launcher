//! Entity Model Features layer, built into Volt Launcher (no settings row
//! anywhere: an engine layer, not a user-facing mod). The procedural
//! motion toolkit the Overhauled Entities mod drives every model with:
//! additive pose channels over the vanilla `setupAnim` result, eased gait
//! harmonics, layered drift "noise" for organic idle motion, and stable
//! per-entity personalities (from the `etf` randomizer) so two cows never
//! move in lockstep. Inspired by Traben's Entity Model Features
//! (reimplemented from its documented behaviour; no upstream code is
//! used). Purely visual: positions, hitboxes and timing never change.
use super::anim::Poses;
use super::etf;
use horizon_core::geometry::trig::Mth;

/// A per-entity motion personality: a fixed phase so herds desynchronise,
/// an amplitude scale and a tempo scale, all stable for the entity's life.
#[derive(Clone, Copy, Debug)]
pub struct Personality {
    pub phase: f32,
    pub amp: f32,
    pub tempo: f32,
}

pub fn personality(entity_id: u32) -> Personality {
    Personality {
        phase: etf::entity_range(entity_id, 1, 0.0, std::f32::consts::TAU),
        amp: etf::entity_range(entity_id, 2, 0.85, 1.15),
        tempo: etf::entity_range(entity_id, 3, 0.92, 1.08),
    }
}

/// A plain sine oscillator: `t` in seconds, `hz` cycles per second.
pub fn osc(t: f32, hz: f32, phase: f32) -> f32 {
    (t * hz * std::f32::consts::TAU + phase).msin()
}

/// Layered slow "noise" in -1..1: three incommensurate sines, so idle
/// glances and head drift look organic instead of metronomic.
pub fn drift(t: f32, hz: f32, phase: f32) -> f32 {
    let a = t * hz * std::f32::consts::TAU + phase;
    (a.msin() + 0.55 * (a * 2.33 + 1.7).msin() + 0.27 * (a * 4.71 + 4.1).msin()) / 1.82
}

/// An eased gait harmonic: the double-frequency bounce layered on a limb's
/// vanilla `cos(pos * 0.6662)` swing, which reads as weight and follow-
/// through instead of a rigid metronome. `pos`/`speed` are the walk
/// animation position and speed the vanilla pose already used.
pub fn gait_bounce(pos: f32, speed: f32, phase_offset: f32) -> f32 {
    (pos * 1.3324 + phase_offset).msin() * speed
}

/// Additive write: adds `v` to the pose channel (3 = x rot, 4 = y rot,
/// 5 = z rot) of `part`, if the model has that part (unknown parts are a
/// no-op, so one overhaul routine serves many model families safely).
pub fn add(p: &mut Poses, part: &str, axis: usize, v: f32) {
    if p.has(part) {
        p.get(part)[axis] += v;
    }
}

/// Clamp helper: every overhaul channel stays within a sane additive
/// budget so no combination of terms can fold a model into itself.
pub fn capped(v: f32, cap: f32) -> f32 {
    v.clamp(-cap, cap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobs::Part;

    fn poses() -> Poses {
        let part = |name: &str| Part {
            name: name.into(),
            pose: [0.0; 9],
            polygons: Vec::new(),
            children: Vec::new(),
        };
        let mut root = part("body");
        root.children = ["head", "right_arm"].map(part).into();
        Poses::new(&root)
    }

    #[test]
    fn personality_is_stable_and_within_design_ranges() {
        let a = personality(123);
        let b = personality(123);
        assert_eq!(a.phase, b.phase);
        assert_eq!(a.amp, b.amp);
        assert_eq!(a.tempo, b.tempo);
        for id in 0..200 {
            let p = personality(id);
            assert!((0.0..=std::f32::consts::TAU).contains(&p.phase));
            assert!((0.85..=1.15).contains(&p.amp));
            assert!((0.92..=1.08).contains(&p.tempo));
        }
        // Two entities almost never share a phase.
        assert_ne!(personality(1).phase, personality(2).phase);
    }

    #[test]
    fn drift_stays_in_range_and_moves() {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for i in 0..2000 {
            let v = drift(i as f32 * 0.05, 0.11, 0.3);
            assert!((-1.01..=1.01).contains(&v), "bounded: {v}");
            min = min.min(v);
            max = max.max(v);
        }
        assert!(max - min > 0.8, "drift actually wanders: {min}..{max}");
    }

    #[test]
    fn add_writes_known_parts_and_ignores_unknown_ones() {
        let mut p = poses();
        add(&mut p, "head", 3, 0.25);
        assert_eq!(p.get("head")[3], 0.25);
        add(&mut p, "head", 3, 0.25);
        assert_eq!(p.get("head")[3], 0.5, "additive, not overwrite");
        // Unknown part: nothing panics, nothing is created.
        add(&mut p, "tail", 3, 9.0);
        assert!(!p.has("tail"));
    }

    #[test]
    fn gait_bounce_scales_with_speed_and_capped_caps() {
        assert_eq!(gait_bounce(1.7, 0.0, 0.0), 0.0, "standing still: no bounce");
        assert!(gait_bounce(1.2, 1.0, 0.0).abs() <= 1.0);
        assert_eq!(capped(2.0, 0.4), 0.4);
        assert_eq!(capped(-2.0, 0.4), -0.4);
        assert_eq!(capped(0.1, 0.4), 0.1);
    }
}
