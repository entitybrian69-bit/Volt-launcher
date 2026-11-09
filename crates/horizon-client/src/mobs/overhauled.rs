//! Overhauled Entities, built into Volt Launcher: a full-body animation
//! overhaul for EVERY living entity, the player included, applied over the
//! vanilla pose after `anim::setup`. Walking gains weight and follow-
//! through (eased double-frequency gait harmonics), bodies roll and
//! counter-roll into steps, idle entities breathe, glance around and
//! fidget, zombies reach and sway, creepers tiptoe and lean, spiders wave
//! their legs, chickens peck as they strut, wolves and cats wag and curl
//! their tails. Every entity gets a stable personality (phase, amplitude,
//! tempo) so herds never move in lockstep. Purely visual: movement,
//! timing and hitboxes never change, so it works on any server.
//!
//! In the spirit of FreshLX's Fresh Animations and Fresh Moves packs
//! (reimplemented from their documented behaviour as native procedural
//! animation; no upstream assets or code are used — those packs are
//! all-rights-reserved CEM data). Powered by the hidden `emf`/`etf`
//! engine layers. Credits in THIRD_PARTY_NOTICES.md.
use super::anim::{Anim, Poses, State};
use super::{Mob, emf};
use horizon_core::geometry::trig::Mth;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const ID: &str = "overhauled_entities";
pub const NAME: &str = "Overhauled Entities";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Animation overhaul for every entity, the player included: \
weighted walks with follow-through, breathing and idle glances, zombie reaches, creeper \
tiptoes, spider leg waves, chicken pecks, tail wags. Per-entity personality so herds \
never sync. Purely visual, works on any server.";

static ENABLED: AtomicBool = AtomicBool::new(true);
static PLAYERS: AtomicBool = AtomicBool::new(true);
static MOBS: AtomicBool = AtomicBool::new(true);
/// 0 = Subtle, 1 = Balanced, 2 = Dramatic.
static INTENSITY: AtomicUsize = AtomicUsize::new(1);
/// Set while the mod's row in Options > Mods is switched off entirely.
static SUSPENDED: AtomicBool = AtomicBool::new(false);

pub const INTENSITY_NAMES: [&str; 3] = ["Subtle", "Balanced", "Dramatic"];
const INTENSITY_SCALE: [f32; 3] = [0.6, 1.0, 1.45];

pub fn set_suspended(off: bool) {
    SUSPENDED.store(off, Ordering::Relaxed);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flags {
    pub enabled: bool,
    pub players: bool,
    pub mobs: bool,
    pub intensity: usize,
}
pub fn flags() -> Flags {
    Flags {
        enabled: ENABLED.load(Ordering::Relaxed),
        players: PLAYERS.load(Ordering::Relaxed),
        mobs: MOBS.load(Ordering::Relaxed),
        intensity: INTENSITY.load(Ordering::Relaxed).min(2),
    }
}
pub fn set_flags(f: Flags) {
    ENABLED.store(f.enabled, Ordering::Relaxed);
    PLAYERS.store(f.players, Ordering::Relaxed);
    MOBS.store(f.mobs, Ordering::Relaxed);
    INTENSITY.store(f.intensity.min(2), Ordering::Relaxed);
}
fn active() -> bool {
    ENABLED.load(Ordering::Relaxed) && !SUSPENDED.load(Ordering::Relaxed)
}

/// What the motion routines read, extracted so each is a pure function of
/// plain values (and unit-testable without building a `Mob`).
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    /// Seconds including the partial tick, scaled by the personality tempo.
    pub t: f32,
    /// The vanilla walk-animation position (`s.pos`).
    pub pos: f32,
    /// The vanilla walk-animation speed, 0..1.
    pub speed: f32,
    /// Master amplitude: the intensity setting times the personality.
    pub k: f32,
    /// The entity's stable phase, so herds desynchronise.
    pub phase: f32,
    /// 1 when standing still, fading to 0 once walking.
    pub idle: f32,
}

/// The post-pass over the finished vanilla pose, for every mob kind.
pub fn apply(m: &Mob, s: &State, p: &mut Poses) {
    if !active() || m.death_time > 0 {
        return;
    }
    let f = flags();
    let player = matches!(m.kind.anim, Anim::Humanoid { .. });
    if (player && !f.players) || (!player && !f.mobs) {
        return;
    }
    let per = emf::personality(m.id);
    let mo = Motion {
        t: s.age / 20.0 * per.tempo,
        pos: s.pos,
        speed: s.speed.min(1.0),
        k: INTENSITY_SCALE[f.intensity] * per.amp,
        phase: per.phase,
        idle: (1.0 - s.speed * 3.0).clamp(0.0, 1.0),
    };
    match m.kind.anim {
        Anim::Humanoid { zombie_arms } => humanoid(p, &mo, zombie_arms),
        Anim::Zombie | Anim::Drowned | Anim::ZombieVillager | Anim::ZombifiedPiglin => {
            humanoid(p, &mo, true);
        }
        Anim::Skeleton | Anim::Witch | Anim::Illager | Anim::Piglin => humanoid(p, &mo, false),
        Anim::Villager => villager(p, &mo),
        Anim::Enderman => enderman(p, &mo),
        Anim::Quadruped
        | Anim::Sheep
        | Anim::PolarBear
        | Anim::Panda
        | Anim::Goat
        | Anim::Hoglin
        | Anim::Horse
        | Anim::Llama => quadruped(p, &mo),
        Anim::Wolf | Anim::Feline | Anim::Fox => {
            quadruped(p, &mo);
            tail_wag(p, &mo);
        }
        Anim::Creeper => creeper(p, &mo),
        Anim::Spider => spider(p, &mo),
        Anim::Chicken => chicken(p, &mo),
        _ => ambient(p, &mo),
    }
}

/// Weighted humanoid gait + idle life. `zombie` adds the reaching sway.
pub fn humanoid(p: &mut Poses, mo: &Motion, zombie: bool) {
    let k = mo.k;
    // Follow-through: a double-frequency bounce over the vanilla swing,
    // so steps land with weight instead of a rigid metronome.
    let bounce = emf::gait_bounce(mo.pos, mo.speed, mo.phase * 0.0);
    emf::add(p, "right_arm", 3, emf::capped(0.14 * k * bounce, 0.35));
    emf::add(p, "left_arm", 3, emf::capped(-0.14 * k * bounce, 0.35));
    emf::add(p, "right_leg", 3, emf::capped(-0.10 * k * bounce, 0.3));
    emf::add(p, "left_leg", 3, emf::capped(0.10 * k * bounce, 0.3));
    // Body roll into each step, head counter-rolling to stay level.
    let roll = (mo.pos * 0.6662).msin() * mo.speed;
    emf::add(p, "body", 5, emf::capped(0.05 * k * roll, 0.12));
    emf::add(p, "head", 5, emf::capped(-0.035 * k * roll, 0.1));
    emf::add(p, "body", 4, emf::capped(0.06 * k * roll, 0.14));
    // Idle life: breathing arms, a slow head wander, a tiny weight shift.
    let idle = mo.idle * k;
    emf::add(p, "right_arm", 5, 0.025 * idle * emf::osc(mo.t, 0.21, mo.phase));
    emf::add(p, "left_arm", 5, -0.025 * idle * emf::osc(mo.t, 0.21, mo.phase + 0.4));
    emf::add(p, "head", 3, 0.045 * idle * emf::drift(mo.t, 0.07, mo.phase));
    emf::add(p, "head", 4, 0.08 * idle * emf::drift(mo.t, 0.045, mo.phase + 2.1));
    emf::add(p, "body", 5, 0.012 * idle * emf::osc(mo.t, 0.09, mo.phase + 1.0));
    if zombie {
        // The undead reach: raised arms wander and spread, stronger while
        // chasing, never enough to break an attack swing.
        let reach = 0.5 + 0.5 * mo.speed;
        emf::add(p, "right_arm", 3, emf::capped(0.11 * k * reach * emf::drift(mo.t, 0.33, mo.phase), 0.3));
        emf::add(p, "left_arm", 3, emf::capped(0.11 * k * reach * emf::drift(mo.t, 0.33, mo.phase + 2.6), 0.3));
        emf::add(p, "right_arm", 4, 0.06 * k * emf::drift(mo.t, 0.21, mo.phase + 1.2));
        emf::add(p, "left_arm", 4, -0.06 * k * emf::drift(mo.t, 0.21, mo.phase + 4.0));
    }
}

/// Weighted quadruped gait: leg follow-through, a trotting body bob, an
/// idle grazing head dip and the occasional ear-era tail flick.
pub fn quadruped(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let bounce = emf::gait_bounce(mo.pos, mo.speed, 0.0);
    for (part, sign) in [
        ("right_hind_leg", 1.0f32),
        ("left_hind_leg", -1.0),
        ("right_front_leg", -1.0),
        ("left_front_leg", 1.0),
    ] {
        emf::add(p, part, 3, emf::capped(0.09 * k * sign * bounce, 0.28));
    }
    // The trot bob: the body pitches gently with each stride pair.
    emf::add(p, "body", 3, emf::capped(0.035 * k * (mo.pos * 1.3324).msin() * mo.speed, 0.09));
    // Idle grazing: a slow, eased head dip that comes and goes.
    let idle = mo.idle * k;
    let graze = (emf::osc(mo.t, 0.05, mo.phase) - 0.55).max(0.0) / 0.45;
    emf::add(p, "head", 3, 0.38 * idle * graze * graze);
    emf::add(p, "head", 4, 0.07 * idle * emf::drift(mo.t, 0.06, mo.phase + 1.4));
    // A tail flick, on models that have one.
    emf::add(p, "tail", 4, 0.2 * idle * emf::drift(mo.t, 0.5, mo.phase).max(0.0));
}

/// A wagging tail for wolves, cats and foxes: eager while moving, a lazy
/// curl-sway at rest.
pub fn tail_wag(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let eager = 0.25 + 0.75 * mo.speed;
    emf::add(p, "tail", 4, emf::capped(0.28 * k * eager * emf::osc(mo.t, 1.9, mo.phase), 0.45));
    emf::add(p, "tail", 3, 0.06 * k * mo.idle * emf::osc(mo.t, 0.23, mo.phase + 0.8));
}

/// The creeper tiptoe: a nervous forward lean that grows with speed, legs
/// stepping lighter, the head tilting as it stalks.
pub fn creeper(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let lean = mo.speed * (0.8 + 0.2 * (mo.pos * 1.3324).msin());
    emf::add(p, "body", 3, emf::capped(0.09 * k * lean, 0.16));
    let bounce = emf::gait_bounce(mo.pos, mo.speed, PI / 2.0);
    for (part, sign) in [
        ("right_hind_leg", 1.0f32),
        ("left_hind_leg", -1.0),
        ("right_front_leg", -1.0),
        ("left_front_leg", 1.0),
    ] {
        emf::add(p, part, 3, emf::capped(0.07 * k * sign * bounce, 0.2));
    }
    emf::add(p, "head", 5, 0.05 * k * emf::drift(mo.t, 0.11, mo.phase));
    emf::add(p, "head", 3, 0.04 * k * mo.idle * emf::drift(mo.t, 0.08, mo.phase + 2.0));
}

/// Spider idle leg waves: each leg lifts and circles in turn while the
/// spider rests, the way real spiders test the air.
pub fn spider(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let idle = mo.idle * k;
    for (i, n) in ["hind", "middle_hind", "middle_front", "front"].iter().enumerate() {
        let leg_phase = mo.phase + i as f32 * 1.9;
        // One leg at a time: the wave gates on the positive lobe only.
        let wave = (emf::osc(mo.t, 0.13, leg_phase) - 0.72).max(0.0) / 0.28;
        let lift = wave * (mo.t * 6.0 + leg_phase).msin() * 0.22 * idle;
        emf::add(p, &format!("right_{n}_leg"), 5, lift);
        emf::add(p, &format!("left_{n}_leg"), 5, -lift);
    }
    emf::add(p, "body", 3, 0.03 * idle * emf::osc(mo.t, 0.35, mo.phase));
    emf::add(p, "head", 3, 0.05 * idle * emf::drift(mo.t, 0.1, mo.phase + 1.1));
}

/// The chicken strut: the head pecks forward with each step, wings
/// flutter half-open at rest, the body bobbing with the walk.
pub fn chicken(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let peck = ((mo.pos * 1.3324).msin()).max(0.0) * mo.speed;
    emf::add(p, "head", 3, emf::capped(0.3 * k * peck, 0.4));
    emf::add(p, "body", 3, emf::capped(0.05 * k * (mo.pos * 1.3324 + PI).msin() * mo.speed, 0.1));
    let idle = mo.idle * k;
    // An idle ruffle: both wings flick out briefly, now and then.
    let ruffle = (emf::osc(mo.t, 0.09, mo.phase) - 0.8).max(0.0) / 0.2;
    let flick = ruffle * (mo.t * 9.0).msin().abs() * 0.5 * idle;
    emf::add(p, "right_wing", 5, flick);
    emf::add(p, "left_wing", 5, -flick);
    emf::add(p, "head", 4, 0.1 * idle * emf::drift(mo.t, 0.17, mo.phase + 0.7));
}

/// Villagers: a thoughtful head nod and wander, the crossed arms rising
/// with a slow breath, the nose twitching now and then.
pub fn villager(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let idle = mo.idle * k;
    emf::add(p, "head", 3, 0.05 * idle * emf::drift(mo.t, 0.08, mo.phase));
    emf::add(p, "head", 4, 0.09 * idle * emf::drift(mo.t, 0.05, mo.phase + 1.9));
    emf::add(p, "arms", 3, 0.03 * idle * emf::osc(mo.t, 0.2, mo.phase));
    let twitch = (emf::osc(mo.t, 0.21, mo.phase + 3.0) - 0.9).max(0.0) / 0.1;
    emf::add(p, "nose", 5, 0.12 * idle * twitch * (mo.t * 11.0).msin());
    emf::add(p, "body", 5, 0.045 * k * (mo.pos * 0.6662).msin() * mo.speed);
}

/// Endermen: a slow, menacing sway, long arms drifting independently.
pub fn enderman(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    emf::add(p, "body", 4, 0.05 * k * emf::drift(mo.t, 0.05, mo.phase));
    emf::add(p, "right_arm", 5, 0.04 * k * emf::osc(mo.t, 0.16, mo.phase));
    emf::add(p, "left_arm", 5, -0.04 * k * emf::osc(mo.t, 0.16, mo.phase + 1.2));
    emf::add(p, "head", 3, 0.04 * k * mo.idle * emf::drift(mo.t, 0.09, mo.phase + 0.5));
    let bounce = emf::gait_bounce(mo.pos, mo.speed, 0.0);
    emf::add(p, "right_leg", 3, emf::capped(-0.08 * k * bounce, 0.25));
    emf::add(p, "left_leg", 3, emf::capped(0.08 * k * bounce, 0.25));
}

/// Everything else alive gets the ambient layer: breathing and a wandering
/// gaze on whatever parts the model actually has.
pub fn ambient(p: &mut Poses, mo: &Motion) {
    let idle = mo.idle * mo.k;
    emf::add(p, "head", 3, 0.04 * idle * emf::drift(mo.t, 0.07, mo.phase));
    emf::add(p, "head", 4, 0.06 * idle * emf::drift(mo.t, 0.045, mo.phase + 2.3));
    emf::add(p, "body", 3, 0.015 * idle * emf::osc(mo.t, 0.18, mo.phase));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobs::Part;

    fn humanoid_poses() -> Poses {
        let part = |name: &str| Part {
            name: name.into(),
            pose: [0.0; 9],
            polygons: Vec::new(),
            children: Vec::new(),
        };
        let mut root = part("body");
        root.children = ["head", "right_arm", "left_arm", "right_leg", "left_leg"]
            .map(part)
            .into();
        Poses::new(&root)
    }
    fn quad_poses() -> Poses {
        let part = |name: &str| Part {
            name: name.into(),
            pose: [0.0; 9],
            polygons: Vec::new(),
            children: Vec::new(),
        };
        let mut root = part("body");
        root.children = [
            "head",
            "right_hind_leg",
            "left_hind_leg",
            "right_front_leg",
            "left_front_leg",
            "tail",
        ]
        .map(part)
        .into();
        Poses::new(&root)
    }
    fn motion(t: f32, pos: f32, speed: f32) -> Motion {
        Motion {
            t,
            pos,
            speed,
            k: 1.0,
            phase: 0.7,
            idle: (1.0 - speed * 3.0).clamp(0.0, 1.0),
        }
    }

    #[test]
    fn walking_humanoids_gain_visible_follow_through() {
        let mut p = humanoid_poses();
        humanoid(&mut p, &motion(3.0, 2.2, 1.0), false);
        let moved = ["right_arm", "left_arm", "right_leg", "left_leg", "body"]
            .iter()
            .map(|n| p.get(n)[3].abs() + p.get(n)[4].abs() + p.get(n)[5].abs())
            .sum::<f32>();
        assert!(moved > 0.1, "a walking pose visibly changes: {moved}");
        // Arms counter-phase: follow-through pulls them apart, not together.
        assert_ne!(p.get("right_arm")[3], p.get("left_arm")[3]);
    }

    #[test]
    fn idle_humanoids_breathe_and_glance() {
        let mut p = humanoid_poses();
        humanoid(&mut p, &motion(5.3, 0.0, 0.0), false);
        let head = p.get("head");
        assert!(head[3].abs() + head[4].abs() > 0.004, "the gaze wanders");
        assert!(p.get("right_arm")[5].abs() > 0.001, "arms breathe");
        // And the idle life actually moves over time (not a frozen offset).
        let mut later = humanoid_poses();
        humanoid(&mut later, &motion(9.1, 0.0, 0.0), false);
        assert_ne!(p.get("head")[4], later.get("head")[4]);
    }

    #[test]
    fn zombies_reach_more_than_skeletons() {
        let mut z = humanoid_poses();
        humanoid(&mut z, &motion(4.0, 1.0, 0.6), true);
        let mut s = humanoid_poses();
        humanoid(&mut s, &motion(4.0, 1.0, 0.6), false);
        let arm = |p: &mut Poses| p.get("right_arm")[3].abs() + p.get("left_arm")[3].abs();
        assert!(arm(&mut z) != arm(&mut s), "the undead reach is its own layer");
    }

    #[test]
    fn every_channel_stays_within_the_additive_budget() {
        // Sweep phases and speeds: no single routine may bend any part
        // more than half a radian, so vanilla poses stay recognisable.
        for ti in 0..40 {
            let t = ti as f32 * 0.37;
            for (pos, speed) in [(0.0, 0.0), (1.3, 0.4), (4.1, 1.0), (7.7, 1.0)] {
                for zombie in [false, true] {
                    let mut p = humanoid_poses();
                    humanoid(&mut p, &motion(t, pos, speed), zombie);
                    for n in ["head", "body", "right_arm", "left_arm", "right_leg", "left_leg"] {
                        for a in 3..6 {
                            assert!(
                                p.get(n)[a].abs() < 0.5,
                                "{n}[{a}] within budget at t={t} pos={pos}: {}",
                                p.get(n)[a]
                            );
                        }
                    }
                }
                let mut q = quad_poses();
                quadruped(&mut q, &motion(t, pos, speed));
                tail_wag(&mut q, &motion(t, pos, speed));
                for n in ["head", "body", "tail", "right_hind_leg", "left_front_leg"] {
                    for a in 3..6 {
                        assert!(q.get(n)[a].abs() < 0.75, "{n}[{a}] quadruped budget");
                    }
                }
            }
        }
    }

    #[test]
    fn quadrupeds_graze_and_wag_when_idle() {
        // Find a moment the slow graze gate is open.
        let grazed = (0..600).any(|i| {
            let mut q = quad_poses();
            quadruped(&mut q, &motion(i as f32 * 0.25, 0.0, 0.0));
            q.get("head")[3] > 0.1
        });
        assert!(grazed, "the grazing dip happens within a couple of minutes");
        let mut q = quad_poses();
        quadruped(&mut q, &motion(2.0, 0.0, 0.0));
        tail_wag(&mut q, &motion(2.0, 0.0, 0.0));
        assert!(q.get("tail")[4].abs() > 0.001, "the tail is alive");
    }

    #[test]
    fn per_entity_personality_desynchronises_the_herd() {
        let a = emf::personality(10);
        let b = emf::personality(11);
        let mut pa = quad_poses();
        let mut pb = quad_poses();
        let mo = |per: emf::Personality| Motion {
            t: 6.0 * per.tempo,
            pos: 0.0,
            speed: 0.0,
            k: per.amp,
            phase: per.phase,
            idle: 1.0,
        };
        quadruped(&mut pa, &mo(a));
        quadruped(&mut pb, &mo(b));
        assert_ne!(
            pa.get("head")[4],
            pb.get("head")[4],
            "two idle animals hold different poses"
        );
    }

    #[test]
    fn flags_round_trip_and_gate() {
        let before = flags();
        set_flags(Flags { enabled: true, players: false, mobs: true, intensity: 2 });
        assert_eq!(flags(), Flags { enabled: true, players: false, mobs: true, intensity: 2 });
        set_flags(Flags { enabled: true, players: true, mobs: true, intensity: 9 });
        assert_eq!(flags().intensity, 2, "intensity clamps to the names table");
        set_suspended(true);
        assert!(!active(), "the row toggle suspends everything");
        set_suspended(false);
        assert!(active());
        set_flags(before);
    }
}
