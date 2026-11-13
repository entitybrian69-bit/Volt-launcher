//! Not Enough Animations, built into Volt Launcher: third-person
//! animations vanilla leaves out, applied over the humanoid pose after
//! `anim::setup` — eating and drinking raised to the mouth with a chew,
//! maps held in both hands, hand-over-hand ladder climbing, and boat
//! rowing. Purely visual: nothing here changes movement, timing or hitboxes.
//!
//! A port of the MIT `not_enough_animations` Horizon native mod by CayMac
//! (github.com/CayMac97/not-enough-animations), itself inspired by tr7zw's
//! Not Enough Animations for Minecraft (reimplemented; tr7zw's code is not
//! used). Credits in THIRD_PARTY_NOTICES.md.
use super::{Mob, anim};
use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "not_enough_animations";
pub const NAME: &str = "Not Enough Animations";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Third-person animations vanilla leaves out: eating and drinking \
raised to the mouth, two-handed map holding, hand-over-hand ladder climbing and boat rowing. \
Purely visual, works on any server.";

static ENABLED: AtomicBool = AtomicBool::new(true);
static EATING: AtomicBool = AtomicBool::new(true);
static MAPS: AtomicBool = AtomicBool::new(true);
static CLIMBING: AtomicBool = AtomicBool::new(true);
static BOATS: AtomicBool = AtomicBool::new(true);
/// The local player's climbing and boat state, fed once a frame by the
/// game (other players' mobs carry theirs on the Mob itself).
static LOCAL_CLIMBING: AtomicBool = AtomicBool::new(false);
static LOCAL_IN_BOAT: AtomicBool = AtomicBool::new(false);
static LOCAL_BOAT_MOVING: AtomicBool = AtomicBool::new(false);
/// Set while the mod's row in Options > Mods is switched off entirely.
static SUSPENDED: AtomicBool = AtomicBool::new(false);
pub fn set_suspended(off: bool) {
    SUSPENDED.store(off, Ordering::Relaxed);
}

/// The five switches, as the settings page lists them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flags {
    pub enabled: bool,
    pub eating: bool,
    pub maps: bool,
    pub climbing: bool,
    pub boats: bool,
}
pub fn flags() -> Flags {
    Flags {
        enabled: ENABLED.load(Ordering::Relaxed),
        eating: EATING.load(Ordering::Relaxed),
        maps: MAPS.load(Ordering::Relaxed),
        climbing: CLIMBING.load(Ordering::Relaxed),
        boats: BOATS.load(Ordering::Relaxed),
    }
}
pub fn set_flags(f: Flags) {
    ENABLED.store(f.enabled, Ordering::Relaxed);
    EATING.store(f.eating, Ordering::Relaxed);
    MAPS.store(f.maps, Ordering::Relaxed);
    CLIMBING.store(f.climbing, Ordering::Relaxed);
    BOATS.store(f.boats, Ordering::Relaxed);
}
pub fn set_local(climbing: bool, in_boat: bool, boat_moving: bool) {
    LOCAL_CLIMBING.store(climbing, Ordering::Relaxed);
    LOCAL_IN_BOAT.store(in_boat, Ordering::Relaxed);
    LOCAL_BOAT_MOVING.store(boat_moving, Ordering::Relaxed);
}

/// What `apply_to` reads of a humanoid, extracted so the pose math is a
/// pure function of plain values (and unit-testable without a Mob).
#[derive(Clone, Copy, Debug, Default)]
pub struct Subject {
    pub using_item_ticks: i32,
    pub using_off_hand: bool,
    pub blocking: bool,
    pub main_arm_left: bool,
    /// The held item ids, `None` for an empty hand.
    pub main_hand_is_map: bool,
    pub off_hand_is_map: bool,
    pub main_hand_empty: bool,
    pub off_hand_empty: bool,
    /// The used hand consumes (not a bow, shield, spyglass, horn...).
    pub consuming: bool,
    pub climbing: bool,
    pub moving: bool,
    pub passenger: bool,
    pub in_boat: bool,
    pub boat_moving: bool,
    /// `position.y`, for the climb cycle's phase.
    pub y: f32,
}

fn consuming(id: &str) -> bool {
    !(id.ends_with(":bow")
        || id.ends_with(":crossbow")
        || id.ends_with(":trident")
        || id.ends_with(":shield")
        || id.ends_with(":brush")
        || id.ends_with(":spyglass")
        || id.ends_with(":goat_horn")
        || id.ends_with(":spear"))
}
fn is_map(id: &str) -> bool {
    id == "minecraft:filled_map" || id == "minecraft:map"
}

fn subject(m: &Mob) -> Subject {
    let used = if m.extra.using_off_hand {
        m.extra.off_hand.as_deref()
    } else {
        m.extra.main_hand.as_deref()
    };
    Subject {
        using_item_ticks: m.extra.using_item,
        using_off_hand: m.extra.using_off_hand,
        blocking: m.extra.blocking,
        main_arm_left: m.main_arm_left(),
        main_hand_is_map: m.extra.main_hand.as_deref().is_some_and(is_map),
        off_hand_is_map: m.extra.off_hand.as_deref().is_some_and(is_map),
        main_hand_empty: m.extra.main_hand.is_none(),
        off_hand_empty: m.extra.off_hand.is_none(),
        consuming: used.is_some_and(consuming),
        climbing: m.climbing || (m.own && LOCAL_CLIMBING.load(Ordering::Relaxed)),
        moving: m.limb_speed > 0.01 || m.velocity.y.abs() > 0.01,
        passenger: m.extra.passenger,
        in_boat: m.own && LOCAL_IN_BOAT.load(Ordering::Relaxed),
        boat_moving: m.own && LOCAL_BOAT_MOVING.load(Ordering::Relaxed),
        y: m.position.y as f32,
    }
}

/// The entry `anim::setup` calls for humanoids, after vanilla's pose.
pub fn apply(m: &Mob, s: &anim::State, p: &mut anim::Poses) {
    if SUSPENDED.load(Ordering::Relaxed) {
        return;
    }
    let f = flags();
    if !f.enabled {
        return;
    }
    apply_to(p, s.age, s.partial, s.swing, &subject(m), f);
}

/// The pose math itself: pure in its inputs, `p` is vanilla's finished
/// humanoid pose. Angles are radians in `[3]`/`[4]`/`[5]` of each part.
pub fn apply_to(
    p: &mut anim::Poses,
    age: f32,
    partial: f32,
    swing: f32,
    m: &Subject,
    f: Flags,
) {
    let head_pitch = p.get("head")[3];
    let head_yaw = p.get("head")[4];

    // 1. Eating and drinking: the using arm lifts to the mouth with an
    //    ease-in over four ticks, chews at ~4 Hz, and the head nods along.
    if f.eating && m.using_item_ticks > 0 && !m.blocking && m.consuming {
        let is_left = m.main_arm_left != m.using_off_hand;
        let arm = if is_left { "left_arm" } else { "right_arm" };
        let ticks = m.using_item_ticks as f32 + partial;
        let raise = (ticks / 4.0).min(1.0);
        let ease = raise * raise * (3.0 - 2.0 * raise);
        let chew = (ticks * 1.2).sin();
        let chew_x = chew * 0.12 * ease;
        let chew_y = chew * 0.06 * ease;
        let target_x = -1.65 + head_pitch * 0.6 + chew_x;
        let inward = if is_left { 0.52 } else { -0.52 };
        let target_y = inward + head_yaw * 0.35 + if is_left { -chew_y } else { chew_y };
        let target_z = if is_left { -0.28 } else { 0.28 };
        let [cur_x, cur_y, cur_z] = [p.get(arm)[3], p.get(arm)[4], p.get(arm)[5]];
        p.xrot(arm, cur_x + (target_x - cur_x) * ease);
        p.yrot(arm, cur_y + (target_y - cur_y) * ease);
        p.zrot(arm, cur_z + (target_z - cur_z) * ease);
        p.get("head")[3] += chew.max(0.0) * 0.08 * ease;
        return;
    }

    // 2. Maps: held up in front of the chest, breathing gently. Two arms
    //    when the map is alone or mirrored, one otherwise.
    if f.maps && swing <= 0.0 && m.using_item_ticks <= 0 && (m.main_hand_is_map || m.off_hand_is_map)
    {
        let breathe = (age * 0.08).sin() * 0.025;
        let two_handed = (m.main_hand_is_map && (m.off_hand_empty || m.off_hand_is_map))
            || (m.off_hand_is_map && m.main_hand_empty);
        if two_handed || m.main_hand_is_map {
            p.xrot("right_arm", -1.25 + head_pitch * 0.4 + breathe);
            p.yrot("right_arm", -0.45 + head_yaw * 0.35);
            p.zrot("right_arm", 0.25);
        }
        if two_handed || m.off_hand_is_map {
            p.xrot("left_arm", -1.25 + head_pitch * 0.4 + breathe);
            p.yrot("left_arm", 0.45 + head_yaw * 0.35);
            p.zrot("left_arm", -0.25);
        }
        return;
    }

    // 3. Ladders and vines: arms high on the rungs, hand over hand while
    //    moving, with the legs stepping alternately.
    if f.climbing && m.climbing {
        let cycle = if m.moving {
            (m.y * 5.0 + age * 0.15).sin()
        } else {
            0.5
        };
        p.xrot("right_arm", -2.45 + cycle * 0.65);
        p.yrot("right_arm", 0.18);
        p.zrot("right_arm", 0.12);
        p.xrot("left_arm", -2.45 - cycle * 0.65);
        p.yrot("left_arm", -0.18);
        p.zrot("left_arm", -0.12);
        let legs = if m.moving { cycle } else { 0.0 };
        p.xrot("right_leg", legs * 0.55);
        p.xrot("left_leg", -legs * 0.55);
        return;
    }

    // 4. Boats: both arms row with the oars instead of resting.
    if f.boats && (m.in_boat || m.passenger) {
        let phase = if m.boat_moving || m.moving {
            age * 0.35 + partial * 0.35
        } else {
            0.0
        };
        let (row_sin, row_cos) = (phase.sin(), phase.cos());
        p.xrot("right_arm", -0.78 + row_cos * 0.32);
        p.yrot("right_arm", 0.38 + row_sin * 0.18);
        p.zrot("right_arm", 0.12);
        p.xrot("left_arm", -0.78 + row_cos * 0.32);
        p.yrot("left_arm", -0.38 - row_sin * 0.18);
        p.zrot("left_arm", -0.12);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobs::Part;

    fn poses() -> anim::Poses {
        let part = |name: &str| Part {
            name: name.into(),
            pose: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            polygons: Vec::new(),
            children: Vec::new(),
        };
        let mut root = part("body");
        root.children = ["head", "right_arm", "left_arm", "right_leg", "left_leg"]
            .map(part)
            .into();
        anim::Poses::new(&root)
    }
    const ON: Flags = Flags {
        enabled: true,
        eating: true,
        maps: true,
        climbing: true,
        boats: true,
    };

    #[test]
    fn eating_lifts_the_using_arm_and_nods() {
        let mut p = poses();
        let m = Subject {
            using_item_ticks: 20,
            consuming: true,
            ..Subject::default()
        };
        apply_to(&mut p, 100.0, 0.5, 0.0, &m, ON);
        let arm = p.get("right_arm");
        assert!(arm[3] < -1.2, "arm lifted towards the mouth: {}", arm[3]);
        assert!(arm[4] < -0.2, "arm turned inward: {}", arm[4]);
        assert_eq!(p.get("left_arm")[3], 0.0, "the other arm stays");
        // The off hand eats on the left arm.
        let mut p = poses();
        let off = Subject {
            using_off_hand: true,
            ..m
        };
        apply_to(&mut p, 100.0, 0.5, 0.0, &off, ON);
        assert!(p.get("left_arm")[3] < -1.2);
        // Blocking, or a bow, is not eating.
        for not_eating in [
            Subject {
                blocking: true,
                ..m
            },
            Subject {
                consuming: false,
                ..m
            },
        ] {
            let mut p = poses();
            apply_to(&mut p, 100.0, 0.5, 0.0, &not_eating, ON);
            assert_eq!(p.get("right_arm")[3], 0.0);
        }
    }

    #[test]
    fn maps_hold_two_arms_up_but_not_mid_swing() {
        let m = Subject {
            main_hand_is_map: true,
            off_hand_empty: true,
            ..Subject::default()
        };
        let mut p = poses();
        apply_to(&mut p, 50.0, 0.0, 0.0, &m, ON);
        assert!((p.get("right_arm")[3] + 1.25).abs() < 0.1);
        assert!((p.get("left_arm")[3] + 1.25).abs() < 0.1, "two-handed");
        let mut p = poses();
        apply_to(&mut p, 50.0, 0.0, 0.6, &m, ON); // mid-swing: vanilla wins
        assert_eq!(p.get("right_arm")[3], 0.0);
    }

    #[test]
    fn climbing_reaches_up_and_boats_row() {
        let m = Subject {
            climbing: true,
            moving: true,
            y: 12.0,
            ..Subject::default()
        };
        let mut p = poses();
        apply_to(&mut p, 10.0, 0.0, 0.0, &m, ON);
        assert!(p.get("right_arm")[3] < -1.7, "arms reach the rungs");
        assert!(
            (p.get("right_leg")[3] + p.get("left_leg")[3]).abs() < 1e-5,
            "legs alternate"
        );
        let m = Subject {
            passenger: true,
            ..Subject::default()
        };
        let mut p = poses();
        apply_to(&mut p, 10.0, 0.0, 0.0, &m, ON);
        assert!(p.get("right_arm")[3] < -0.4, "rowing pose");
        assert!((p.get("left_arm")[4] + 0.38).abs() < 0.5);
    }

    #[test]
    fn flags_gate_each_feature_and_the_whole_mod() {
        let m = Subject {
            using_item_ticks: 20,
            consuming: true,
            climbing: true,
            ..Subject::default()
        };
        let mut p = poses();
        apply_to(
            &mut p,
            10.0,
            0.0,
            0.0,
            &m,
            Flags {
                eating: false,
                ..ON
            },
        );
        assert!(p.get("right_arm")[3] < -1.7, "falls through to climbing");
        let mut p = poses();
        apply_to(
            &mut p,
            10.0,
            0.0,
            0.0,
            &m,
            Flags {
                enabled: false,
                eating: false,
                maps: false,
                climbing: false,
                boats: false,
            },
        );
        assert_eq!(p.get("right_arm")[3], 0.0, "all off: vanilla pose");
        // The statics round-trip.
        let before = flags();
        set_flags(Flags {
            enabled: false,
            ..before
        });
        assert!(!flags().enabled);
        set_flags(before);
    }
}
