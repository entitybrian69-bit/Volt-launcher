//! Keyframed one-shot clips for Overhauled Entities: stretch / look-around /
//! arm-cross idle emotes, the mid-air flip, the sprint-fall roll, the sneak
//! sit, the three water entries (dive, cannonball, belly flop) and the wolf
//! howl. Every clip is an additive layer over the finished vanilla pose
//! (zero at both ends, so entering and leaving a clip never pops), sampled
//! with cosine easing between keys.
//!
//! The module also keeps the tiny per-entity state the clip triggers need:
//! a swing counter (rising-edge detection on the vanilla attack animation,
//! for attack variation), the water-entry edge with the fall distance that
//! preceded it (for the dives), and the active triggered clip. Idle emotes
//! are stateless: a deterministic per-entity schedule derived from the
//! `etf` randomizer, so the same entity always emotes at the same moments
//! and two entities never sync up.
//!
//! Behavioral parity with Actions & Stuff (Oreville Studios); no upstream
//! assets or code are used. Credits in THIRD_PARTY_NOTICES.md.
use super::anim::Poses;
use super::{emf, etf};
use horizon_core::geometry::trig::Mth;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::Mutex;

/// A one-shot clip. Every clip's channels start and end at zero (the sit
/// holds its last key until `clear`), so clips blend over any base pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clip {
    Stretch,
    LookAround,
    ArmCross,
    Flip,
    Roll,
    Sit,
    Dive,
    CannonBall,
    BellyFlop,
    Howl,
    Loaf,
    Lay,
}

/// One additive channel: a part, a pose axis (3 = x rot, 4 = y rot,
/// 5 = z rot) and keys as `(seconds, radians)` pairs.
pub struct Channel {
    pub part: &'static str,
    pub axis: usize,
    pub keys: &'static [(f32, f32)],
}

const fn ch(part: &'static str, axis: usize, keys: &'static [(f32, f32)]) -> Channel {
    Channel { part, axis, keys }
}

const STRETCH: &[Channel] = &[
    ch("right_arm", 3, &[(0.0, 0.0), (0.7, -2.5), (2.1, -2.5), (2.8, 0.0)]),
    ch("left_arm", 3, &[(0.0, 0.0), (0.7, -2.5), (2.1, -2.5), (2.8, 0.0)]),
    ch("right_arm", 5, &[(0.0, 0.0), (0.7, 0.25), (2.1, 0.25), (2.8, 0.0)]),
    ch("left_arm", 5, &[(0.0, 0.0), (0.7, -0.25), (2.1, -0.25), (2.8, 0.0)]),
    ch("body", 3, &[(0.0, 0.0), (0.8, -0.12), (2.0, -0.12), (2.8, 0.0)]),
    ch("head", 3, &[(0.0, 0.0), (0.8, -0.35), (2.0, -0.35), (2.8, 0.0)]),
];
const LOOK_AROUND: &[Channel] = &[
    ch("head", 4, &[(0.0, 0.0), (0.7, 0.95), (1.5, 0.95), (2.3, -0.95), (3.0, -0.95), (3.6, 0.0)]),
    ch("body", 4, &[(0.0, 0.0), (0.9, 0.2), (1.5, 0.2), (2.4, -0.2), (3.0, -0.2), (3.6, 0.0)]),
    ch("head", 3, &[(0.0, 0.0), (0.5, -0.1), (3.2, -0.1), (3.6, 0.0)]),
];
const ARM_CROSS: &[Channel] = &[
    ch("right_arm", 3, &[(0.0, 0.0), (0.5, -0.55), (3.5, -0.55), (4.0, 0.0)]),
    ch("left_arm", 3, &[(0.0, 0.0), (0.5, -0.55), (3.5, -0.55), (4.0, 0.0)]),
    ch("right_arm", 5, &[(0.0, 0.0), (0.5, 0.45), (3.5, 0.45), (4.0, 0.0)]),
    ch("left_arm", 5, &[(0.0, 0.0), (0.5, -0.45), (3.5, -0.45), (4.0, 0.0)]),
    ch("head", 3, &[(0.0, 0.0), (0.6, 0.15), (3.4, 0.15), (4.0, 0.0)]),
];
const FLIP: &[Channel] = &[
    ch("body", 3, &[(0.0, 0.0), (0.35, -1.15), (0.9, 0.0)]),
    ch("head", 3, &[(0.0, 0.0), (0.35, -0.5), (0.9, 0.0)]),
    ch("right_leg", 3, &[(0.0, 0.0), (0.4, 1.25), (0.9, 0.0)]),
    ch("left_leg", 3, &[(0.0, 0.0), (0.4, 1.25), (0.9, 0.0)]),
    ch("right_arm", 3, &[(0.0, 0.0), (0.3, -2.3), (0.9, 0.0)]),
    ch("left_arm", 3, &[(0.0, 0.0), (0.3, -2.3), (0.9, 0.0)]),
];
const ROLL: &[Channel] = &[
    ch("body", 3, &[(0.0, 0.0), (0.3, 1.2), (0.8, 0.0)]),
    ch("head", 3, &[(0.0, 0.0), (0.3, 0.45), (0.8, 0.0)]),
    ch("right_leg", 3, &[(0.0, 0.0), (0.35, 1.2), (0.8, 0.0)]),
    ch("left_leg", 3, &[(0.0, 0.0), (0.35, 1.2), (0.8, 0.0)]),
    ch("right_arm", 3, &[(0.0, 0.0), (0.3, -1.3), (0.8, 0.0)]),
    ch("left_arm", 3, &[(0.0, 0.0), (0.3, -1.3), (0.8, 0.0)]),
];
const SIT: &[Channel] = &[
    ch("right_leg", 3, &[(0.0, 0.0), (0.5, -1.4)]),
    ch("left_leg", 3, &[(0.0, 0.0), (0.5, -1.4)]),
    ch("body", 3, &[(0.0, 0.0), (0.5, 0.1)]),
    ch("right_arm", 3, &[(0.0, 0.0), (0.5, -0.3)]),
    ch("left_arm", 3, &[(0.0, 0.0), (0.5, -0.3)]),
];
const DIVE: &[Channel] = &[
    ch("body", 3, &[(0.0, 0.0), (0.4, 1.35), (1.0, 0.0)]),
    ch("right_arm", 3, &[(0.0, 0.0), (0.35, -2.8), (1.0, 0.0)]),
    ch("left_arm", 3, &[(0.0, 0.0), (0.35, -2.8), (1.0, 0.0)]),
    ch("head", 3, &[(0.0, 0.0), (0.4, -0.4), (1.0, 0.0)]),
    ch("right_leg", 3, &[(0.0, 0.0), (0.45, 0.25), (1.0, 0.0)]),
    ch("left_leg", 3, &[(0.0, 0.0), (0.45, 0.25), (1.0, 0.0)]),
];
const CANNONBALL: &[Channel] = &[
    ch("body", 3, &[(0.0, 0.0), (0.35, 0.95), (0.9, 0.0)]),
    ch("right_leg", 3, &[(0.0, 0.0), (0.3, 1.35), (0.9, 0.0)]),
    ch("left_leg", 3, &[(0.0, 0.0), (0.3, 1.35), (0.9, 0.0)]),
    ch("right_arm", 3, &[(0.0, 0.0), (0.3, -1.0), (0.9, 0.0)]),
    ch("left_arm", 3, &[(0.0, 0.0), (0.3, -1.0), (0.9, 0.0)]),
    ch("right_arm", 5, &[(0.0, 0.0), (0.3, 0.6), (0.9, 0.0)]),
    ch("left_arm", 5, &[(0.0, 0.0), (0.3, -0.6), (0.9, 0.0)]),
    ch("head", 3, &[(0.0, 0.0), (0.3, -0.45), (0.9, 0.0)]),
];
const BELLY_FLOP: &[Channel] = &[
    ch("body", 3, &[(0.0, 0.0), (0.4, 1.4), (1.0, 0.0)]),
    ch("right_arm", 5, &[(0.0, 0.0), (0.35, 0.9), (1.0, 0.0)]),
    ch("left_arm", 5, &[(0.0, 0.0), (0.35, -0.9), (1.0, 0.0)]),
    ch("right_leg", 3, &[(0.0, 0.0), (0.4, 0.3), (1.0, 0.0)]),
    ch("left_leg", 3, &[(0.0, 0.0), (0.4, 0.3), (1.0, 0.0)]),
    ch("head", 3, &[(0.0, 0.0), (0.4, -0.55), (1.0, 0.0)]),
];
const LOAF: &[Channel] = &[
    // The cat loaf: all four legs folded under, tail curled around the
    // body, chin settled. Sustained until the next interact wakes it.
    ch("right_front_leg", 3, &[(0.0, 0.0), (0.6, 1.25)]),
    ch("left_front_leg", 3, &[(0.0, 0.0), (0.6, 1.25)]),
    ch("right_hind_leg", 3, &[(0.0, 0.0), (0.6, 1.25)]),
    ch("left_hind_leg", 3, &[(0.0, 0.0), (0.6, 1.25)]),
    ch("body", 3, &[(0.0, 0.0), (0.6, 0.12)]),
    ch("head", 3, &[(0.0, 0.0), (0.6, 0.18)]),
    ch("tail1", 4, &[(0.0, 0.0), (0.6, 0.85)]),
    ch("tail2", 4, &[(0.0, 0.0), (0.6, 0.95)]),
];
const LAY: &[Channel] = &[
    // The wolf lay-down: a sphinx rest — chest low, front legs folded,
    // hind legs tucked, head settled. Sustained until the next interact.
    ch("upper_body", 3, &[(0.0, 0.0), (0.7, 0.35)]),
    ch("right_front_leg", 3, &[(0.0, 0.0), (0.7, 1.2)]),
    ch("left_front_leg", 3, &[(0.0, 0.0), (0.7, 1.2)]),
    ch("right_hind_leg", 3, &[(0.0, 0.0), (0.7, 1.1)]),
    ch("left_hind_leg", 3, &[(0.0, 0.0), (0.7, 1.1)]),
    ch("real_head", 3, &[(0.0, 0.0), (0.7, 0.2)]),
    ch("head", 3, &[(0.0, 0.0), (0.7, 0.2)]),
    ch("real_tail", 3, &[(0.0, 0.0), (0.7, 0.3)]),
    ch("tail", 3, &[(0.0, 0.0), (0.7, 0.3)]),
];
const HOWL: &[Channel] = &[
    // The wolf model names its articulated head/tail `real_head`/
    // `real_tail`; `head`/`tail` cover other canine-shaped models
    // (unknown parts are a no-op).
    ch("real_head", 3, &[(0.0, 0.0), (0.6, -0.8), (2.6, -0.8), (3.2, 0.0)]),
    ch("head", 3, &[(0.0, 0.0), (0.6, -0.8), (2.6, -0.8), (3.2, 0.0)]),
    ch("upper_body", 3, &[(0.0, 0.0), (0.6, -0.18), (2.6, -0.18), (3.2, 0.0)]),
    ch("real_tail", 3, &[(0.0, 0.0), (0.6, -0.4), (2.6, -0.4), (3.2, 0.0)]),
    ch("tail", 3, &[(0.0, 0.0), (0.6, -0.4), (2.6, -0.4), (3.2, 0.0)]),
];

pub fn channels(c: Clip) -> &'static [Channel] {
    match c {
        Clip::Stretch => STRETCH,
        Clip::LookAround => LOOK_AROUND,
        Clip::ArmCross => ARM_CROSS,
        Clip::Flip => FLIP,
        Clip::Roll => ROLL,
        Clip::Sit => SIT,
        Clip::Dive => DIVE,
        Clip::CannonBall => CANNONBALL,
        Clip::BellyFlop => BELLY_FLOP,
        Clip::Howl => HOWL,
        Clip::Loaf => LOAF,
        Clip::Lay => LAY,
    }
}
pub fn duration(c: Clip) -> f32 {
    match c {
        Clip::Stretch => 2.8,
        Clip::LookAround => 3.6,
        Clip::ArmCross => 4.0,
        Clip::Flip => 0.9,
        Clip::Roll => 0.8,
        Clip::Sit => 0.5,
        Clip::Dive => 1.0,
        Clip::CannonBall => 0.9,
        Clip::BellyFlop => 1.0,
        Clip::Howl => 3.2,
        Clip::Loaf => 0.6,
        Clip::Lay => 0.7,
    }
}
/// Sustained clips hold their last key until `clear` instead of expiring.
pub fn sustained(c: Clip) -> bool {
    matches!(c, Clip::Sit | Clip::Loaf | Clip::Lay)
}

/// Sample a channel's keys at `t` seconds: cosine-eased between keys,
/// clamped to the first and last values outside the key range.
pub fn sample(keys: &[(f32, f32)], t: f32) -> f32 {
    let Some(first) = keys.first() else { return 0.0 };
    if t <= first.0 {
        return first.1;
    }
    for w in keys.windows(2) {
        let (a, b) = (w[0], w[1]);
        if t <= b.0 {
            let u = ((t - a.0) / (b.0 - a.0).max(1e-6)).clamp(0.0, 1.0);
            let e = 0.5 - 0.5 * (u * PI).mcos();
            return a.1 + (b.1 - a.1) * e;
        }
    }
    keys[keys.len() - 1].1
}

/// Additively apply clip `c` at `t` seconds into the clip, scaled by `k`.
pub fn apply_clip(p: &mut Poses, c: Clip, t: f32, k: f32) {
    for channel in channels(c) {
        emf::add(p, channel.part, channel.axis, sample(channel.keys, t) * k);
    }
}

/// Which clip families an entity is eligible for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Humanoid,
    Wolf,
    Cat,
    Other,
}

/// What `tick` reads each frame, per entity. `now` is the entity's own
/// clock in seconds (its age including the partial tick).
pub struct Obs {
    pub id: u32,
    pub now: f32,
    pub swing: f32,
    pub in_water: bool,
    pub fall: f32,
    pub crouching: bool,
    pub idle: bool,
    pub family: Family,
    pub howl_ok: bool,
}

/// What `tick` produces: the swing counter for attack variation and the
/// clip to play, as `(clip, seconds into it, is an idle emote)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sig {
    pub swing_index: u32,
    pub clip: Option<(Clip, f32, bool)>,
}

#[derive(Clone, Copy, Default)]
struct Ent {
    prev_swing: f32,
    swing_index: u32,
    prev_in_water: bool,
    peak_fall: f32,
    crouch_early: bool,
    active: Option<(Clip, f32)>,
    splash: bool,
    interacts: u32,
}

static ENTS: Mutex<Option<HashMap<u32, Ent>>> = Mutex::new(None);

fn with_ent<R>(id: u32, f: impl FnOnce(&mut Ent) -> R) -> R {
    let mut guard = ENTS.lock().unwrap();
    let map = guard.get_or_insert_with(HashMap::new);
    // The map only ever holds entities that animated recently; if a very
    // long session grows it past any sane bound, start over (the cost is
    // one missed emote window, not a leak).
    if map.len() > 4096 {
        map.clear();
    }
    f(map.entry(id).or_default())
}

/// Per-frame update for one entity: edge detection and the active clip.
pub fn tick(o: &Obs) -> Sig {
    let (swing_index, mut active) = with_ent(o.id, |e| {
        // Attack swings: a rising edge on the vanilla swing counter.
        if o.swing > 0.02 && e.prev_swing <= 0.02 {
            e.swing_index = e.swing_index.wrapping_add(1);
        }
        e.prev_swing = o.swing;
        // Arm the dive while falling: remember the deepest fall and
        // whether the crouch came early in it (the cannonball wind-up).
        if !o.in_water {
            if o.fall > e.peak_fall {
                e.peak_fall = o.fall;
            }
            if o.crouching && o.fall > 0.2 && o.fall < 2.5 {
                e.crouch_early = true;
            }
        }
        if o.in_water && !e.prev_in_water {
            // The water-entry edge: a real leap becomes a dive. A mid-air
            // flip yields to the entry (its crouch was the cannonball
            // wind-up, not a separate move).
            let yields = matches!(e.active, None | Some((Clip::Flip, _)));
            if o.family == Family::Humanoid && e.peak_fall >= 1.5 && yields {
                let c = if e.crouch_early {
                    Clip::CannonBall
                } else if o.crouching {
                    Clip::BellyFlop
                } else {
                    Clip::Dive
                };
                e.active = Some((c, o.now));
                e.splash = true;
            }
            e.peak_fall = 0.0;
            e.crouch_early = false;
        } else if !o.in_water && o.fall < 0.05 {
            e.peak_fall = 0.0;
            e.crouch_early = false;
        }
        e.prev_in_water = o.in_water;
        // Expire the active clip (sustained clips wait for `clear`); a
        // backwards clock (world reload) also clears it.
        if let Some((c, start)) = e.active
            && (o.now < start - 0.5 || (!sustained(c) && o.now - start > duration(c)))
        {
            e.active = None;
        }
        (e.swing_index, e.active)
    });
    let mut emote = false;
    if active.is_none() && o.idle {
        if let Some(s) = scheduled(o) {
            active = Some(s);
            emote = true;
        }
    }
    Sig {
        swing_index,
        clip: active.map(|(c, start)| (c, o.now - start, emote)),
    }
}

/// The deterministic idle-emote schedule: stable per entity and window,
/// so replays and remote views agree and herds never sync.
fn scheduled(o: &Obs) -> Option<(Clip, f32)> {
    let (window, chance, pool): (f32, f32, &[Clip]) = match o.family {
        Family::Humanoid => (16.0, 0.3, &[Clip::Stretch, Clip::LookAround, Clip::ArmCross]),
        Family::Wolf if o.howl_ok => (40.0, 0.4, &[Clip::Howl]),
        _ => return None,
    };
    if o.now <= 0.0 {
        return None;
    }
    let win = (o.now / window) as u32;
    let r = etf::entity_random(o.id, 97u32.wrapping_add(win));
    if r >= chance {
        return None;
    }
    let c = pool[(r * 1024.0) as usize % pool.len()];
    let start = win as f32 * window + 1.0 + r * 5.0;
    let t = o.now - start;
    (t >= 0.0 && t <= duration(c)).then_some((c, start))
}

/// Start a clip on an entity (the own-player triggers: flip, roll, sit).
pub fn trigger(id: u32, c: Clip, now: f32) {
    with_ent(id, |e| e.active = Some((c, now)));
}
/// Stop whatever clip the entity is playing (ends a sustained sit).
pub fn clear(id: u32) {
    with_ent(id, |e| e.active = None);
}
/// Take the one-shot "just splashed down" flag (the particle pass burns
/// a bubble burst on it).
pub fn take_splash(id: u32) -> bool {
    with_ent(id, |e| std::mem::take(&mut e.splash))
}

/// An interact on a cat or wolf: a loafing/laying pet wakes; otherwise a
/// stable 50% roll settles it into the loaf (cats) or the lay (wolves).
/// Purely visual — the vanilla interaction outcome is untouched.
pub fn interact_toggle(id: u32, family: Family, now: f32) {
    let want = match family {
        Family::Cat => Clip::Loaf,
        Family::Wolf => Clip::Lay,
        _ => return,
    };
    with_ent(id, |e| {
        e.interacts = e.interacts.wrapping_add(1);
        if matches!(e.active, Some((c, _)) if c == want) {
            e.active = None;
        } else if e.active.is_none() && etf::entity_random(id, 700u32.wrapping_add(e.interacts)) < 0.5
        {
            e.active = Some((want, now));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobs::Part;

    const ALL: [Clip; 12] = [
        Clip::Stretch,
        Clip::LookAround,
        Clip::ArmCross,
        Clip::Flip,
        Clip::Roll,
        Clip::Sit,
        Clip::Dive,
        Clip::CannonBall,
        Clip::BellyFlop,
        Clip::Howl,
        Clip::Loaf,
        Clip::Lay,
    ];

    fn poses() -> Poses {
        let part = |name: &str| Part {
            name: name.into(),
            pose: [0.0; 9],
            polygons: Vec::new(),
            children: Vec::new(),
        };
        let mut root = part("body");
        root.children = [
            "head",
            "right_arm",
            "left_arm",
            "right_leg",
            "left_leg",
            "upper_body",
            "real_head",
            "real_tail",
            "tail",
            "right_front_leg",
            "left_front_leg",
            "right_hind_leg",
            "left_hind_leg",
            "tail1",
            "tail2",
        ]
        .map(part)
        .into();
        Poses::new(&root)
    }
    fn obs(id: u32, now: f32) -> Obs {
        Obs {
            id,
            now,
            swing: 0.0,
            in_water: false,
            fall: 0.0,
            crouching: false,
            idle: false,
            family: Family::Humanoid,
            howl_ok: false,
        }
    }

    #[test]
    fn every_clip_starts_and_ends_at_zero() {
        for c in ALL {
            for channel in channels(c) {
                let keys = channel.keys;
                assert_eq!(keys[0], (0.0, 0.0), "{c:?} {} starts at rest", channel.part);
                if !sustained(c) {
                    let last = keys[keys.len() - 1];
                    assert_eq!(last.1, 0.0, "{c:?} {} ends at rest", channel.part);
                    assert!(
                        (last.0 - duration(c)).abs() < 1e-4,
                        "{c:?} {} last key sits on the duration",
                        channel.part
                    );
                }
            }
        }
    }

    #[test]
    fn clips_move_mid_way_and_stay_within_budget() {
        for c in ALL {
            let mut p = poses();
            apply_clip(&mut p, c, duration(c) * 0.45, 1.0);
            let total: f32 = ["head", "body", "right_arm", "left_arm", "right_leg", "left_leg",
                "upper_body", "real_head", "real_tail", "tail", "right_front_leg", "tail1"]
                .iter()
                .map(|n| (3..6).map(|a| p.get(n)[a].abs()).sum::<f32>())
                .sum();
            assert!(total > 0.1, "{c:?} visibly moves mid-clip: {total}");
            // Budget sweep: nothing bends past 3.0 radians at any point.
            for i in 0..40 {
                let mut q = poses();
                apply_clip(&mut q, c, duration(c) * i as f32 / 39.0, 1.0);
                for n in ["head", "body", "right_arm", "left_arm", "right_leg", "left_leg"] {
                    for a in 3..6 {
                        assert!(q.get(n)[a].abs() <= 3.0, "{c:?} {n}[{a}] within budget");
                    }
                }
            }
        }
    }

    #[test]
    fn sampling_is_zero_at_both_ends_and_eases_between() {
        let keys = &[(0.0f32, 0.0f32), (1.0, 2.0), (2.0, 0.0)];
        assert_eq!(sample(keys, 0.0), 0.0);
        assert_eq!(sample(keys, 2.0), 0.0);
        assert_eq!(sample(keys, 5.0), 0.0, "clamped after the last key");
        assert!((sample(keys, 1.0) - 2.0).abs() < 1e-5);
        let q = sample(keys, 0.5);
        assert!(q > 0.0 && q < 2.0, "eased between keys: {q}");
    }

    #[test]
    fn swings_count_once_per_rising_edge() {
        let id = 910_001;
        let base = tick(&obs(id, 0.0)).swing_index;
        let mut o = obs(id, 0.1);
        o.swing = 0.4;
        assert_eq!(tick(&o).swing_index, base + 1, "the edge counts");
        o.now = 0.2;
        o.swing = 0.8;
        assert_eq!(tick(&o).swing_index, base + 1, "still the same swing");
        o.now = 0.3;
        o.swing = 0.0;
        tick(&o);
        o.now = 0.4;
        o.swing = 0.3;
        assert_eq!(tick(&o).swing_index, base + 2, "a new swing counts again");
    }

    #[test]
    fn a_leap_into_water_dives_exactly_once() {
        let id = 910_002;
        let mut o = obs(id, 0.0);
        o.fall = 3.0;
        assert!(tick(&o).clip.is_none(), "still falling");
        o.now = 0.1;
        o.fall = 0.0;
        o.in_water = true;
        let c = tick(&o).clip.expect("the entry starts a dive");
        assert_eq!(c.0, Clip::Dive);
        assert!(!c.2, "a dive is a trigger, not an emote");
        assert!(take_splash(id), "the splash flag is set once");
        assert!(!take_splash(id), "and consumed");
        o.now = 0.2;
        let again = tick(&o).clip.expect("the dive keeps playing");
        assert!(again.1 > c.1, "time advances into the clip");
        o.now = 5.0;
        assert!(tick(&o).clip.is_none(), "and it expires");
    }

    #[test]
    fn crouching_early_in_the_fall_cannonballs() {
        let id = 910_003;
        let mut o = obs(id, 0.0);
        o.fall = 1.0;
        o.crouching = true;
        tick(&o);
        o.now = 0.1;
        o.fall = 4.0;
        o.crouching = false;
        tick(&o);
        o.now = 0.2;
        o.fall = 0.0;
        o.in_water = true;
        let c = tick(&o).clip.expect("entry");
        assert_eq!(c.0, Clip::CannonBall);
        // And crouching at the entry itself belly-flops.
        let id2 = 910_004;
        let mut o2 = obs(id2, 0.0);
        o2.fall = 4.0;
        tick(&o2);
        o2.now = 0.1;
        o2.fall = 0.0;
        o2.in_water = true;
        o2.crouching = true;
        assert_eq!(tick(&o2).clip.expect("entry").0, Clip::BellyFlop);
    }

    #[test]
    fn triggered_clips_play_and_sustained_sits_wait_for_clear() {
        let id = 910_005;
        trigger(id, Clip::Flip, 10.0);
        let mut o = obs(id, 10.3);
        let c = tick(&o).clip.expect("the flip plays");
        assert_eq!(c.0, Clip::Flip);
        o.now = 20.0;
        assert!(tick(&o).clip.is_none(), "the flip expired");
        trigger(id, Clip::Sit, 20.0);
        o.now = 60.0;
        assert_eq!(tick(&o).clip.expect("still sitting").0, Clip::Sit);
        clear(id);
        o.now = 60.1;
        assert!(tick(&o).clip.is_none(), "cleared");
    }

    #[test]
    fn idle_emotes_are_deterministic_gated_on_idle_and_eventually_happen() {
        let id = 910_006;
        let mut hit = None;
        for i in 0..4000 {
            let mut o = obs(id, i as f32 * 0.25);
            o.idle = true;
            if let Some((c, t, emote)) = tick(&o).clip {
                assert!(emote, "scheduled clips are emotes");
                assert!(t >= 0.0 && t <= duration(c) + 1e-3);
                hit = Some((o.now, c, t));
                break;
            }
        }
        let (now, c, t) = hit.expect("an emote happens within ~17 minutes of idling");
        // Determinism: the same entity at the same moment replays the same clip.
        let mut o = obs(id, now);
        o.idle = true;
        let again = tick(&o).clip.expect("stable schedule");
        assert_eq!((again.0, again.1), (c, t));
        // And a moving entity never emotes.
        let mut o = obs(id, now);
        o.idle = false;
        assert!(tick(&o).clip.is_none(), "emotes are idle-only");
    }

    #[test]
    fn interacts_toggle_the_loaf_and_the_lay() {
        // The 50% roll is stable per (entity, interact count): poke until
        // one lands, then the next interact must wake it.
        let mut settled = None;
        for id in 920_000..920_040u32 {
            interact_toggle(id, Family::Cat, 5.0);
            let mut o = obs(id, 5.1);
            o.family = Family::Cat;
            if let Some((c, _, emote)) = tick(&o).clip {
                assert_eq!(c, Clip::Loaf, "cats loaf");
                assert!(!emote, "the loaf is a trigger");
                settled = Some(id);
                break;
            }
        }
        let id = settled.expect("about half of forty cats loaf");
        // Sustained: still loafing much later.
        let mut o = obs(id, 400.0);
        o.family = Family::Cat;
        assert_eq!(tick(&o).clip.expect("still loafing").0, Clip::Loaf);
        // The next interact wakes it, every time.
        interact_toggle(id, Family::Cat, 400.0);
        assert!(tick(&o).clip.is_none(), "woken");
        // Wolves lay instead, and other families never react.
        let mut laid = false;
        for id in 921_000..921_040u32 {
            interact_toggle(id, Family::Wolf, 5.0);
            let mut o = obs(id, 5.1);
            o.family = Family::Wolf;
            if let Some((c, _, _)) = tick(&o).clip {
                assert_eq!(c, Clip::Lay, "wolves lay");
                laid = true;
                break;
            }
        }
        assert!(laid, "about half of forty wolves lay");
        interact_toggle(922_000, Family::Humanoid, 5.0);
        let o = obs(922_000, 5.1);
        assert!(tick(&o).clip.is_none(), "players never loaf");
    }

    #[test]
    fn wolves_howl_only_under_the_full_moon_flag() {
        let id = 910_007;
        let mut howled = false;
        for i in 0..8000 {
            let mut o = obs(id, i as f32 * 0.5);
            o.idle = true;
            o.family = Family::Wolf;
            o.howl_ok = true;
            if let Some((c, _, _)) = tick(&o).clip {
                assert_eq!(c, Clip::Howl);
                // The same moment without the moon is silent.
                let mut day = obs(id, o.now);
                day.idle = true;
                day.family = Family::Wolf;
                assert!(tick(&day).clip.is_none(), "no moon, no howl");
                howled = true;
                break;
            }
        }
        assert!(howled, "a full-moon wolf howls within ~an hour of idling");
    }
}
