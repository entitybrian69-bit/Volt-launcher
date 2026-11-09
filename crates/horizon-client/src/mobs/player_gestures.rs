//! Player Animations, built into Volt Launcher: a keyframe animation layer
//! for the player model, in the spirit of KosmX's playerAnimator library
//! (MIT) for Minecraft — keyframed part channels, smooth interpolation and
//! fade-in/fade-out layering over the vanilla pose — plus a set of built-in
//! gestures (wave, point, salute, nod, shake, clap) played on your own
//! third-person body with a key. The keyframe data here is original.
//! Other built-in mods can layer their own clips through [`play_keyframes`].
use super::{Mob, anim};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const ID: &str = "player_animations";
pub const NAME: &str = "Player Animations";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "A keyframe animation layer for the player model, with built-in \
gestures: wave, point, salute, nod, shake the head and clap. Bind the gesture key, pick a \
gesture on the settings page, and it plays on your third-person body. Purely visual.";

static ENABLED: AtomicBool = AtomicBool::new(true);
static SELECTED: AtomicUsize = AtomicUsize::new(0);
/// Set while the mod's row in Options > Mods is switched off entirely.
static SUSPENDED: AtomicBool = AtomicBool::new(false);
pub fn set_suspended(off: bool) {
    SUSPENDED.store(off, Ordering::Relaxed);
    if off {
        *PLAYING.lock().unwrap() = None;
    }
}
/// The playing clip and the tick-time second it started, if any.
static PLAYING: Mutex<Option<(usize, f32)>> = Mutex::new(None);
/// A clip queued from the settings page: it starts on the first frame the
/// player is back in the world, so the whole gesture is visible instead of
/// running out behind the paused menu.
static PENDING: AtomicBool = AtomicBool::new(false);

/// Queues the selected gesture to start when gameplay resumes.
pub fn queue_selected() {
    if enabled() {
        PENDING.store(true, Ordering::Relaxed);
    }
}
pub fn pending() -> bool {
    PENDING.load(Ordering::Relaxed) && enabled() && !SUSPENDED.load(Ordering::Relaxed)
}
/// Starts the queued gesture at age-second `now` and clears the queue.
pub fn start_pending(now: f32) {
    if PENDING.swap(false, Ordering::Relaxed) {
        play_selected(now);
    }
}

/// One animated channel: a part's rotation axis through `(time s, radians)`
/// keys, sampled with smoothstep easing between neighbouring keys.
pub struct Channel {
    pub part: &'static str,
    /// 3 = xRot, 4 = yRot, 5 = zRot in a `Poses` entry.
    pub axis: usize,
    pub keys: &'static [(f32, f32)],
}
pub struct Gesture {
    pub name: &'static str,
    pub seconds: f32,
    pub channels: &'static [Channel],
}

pub const GESTURES: &[Gesture] = &[
    Gesture {
        name: "Wave",
        seconds: 2.0,
        channels: &[
            Channel { part: "right_arm", axis: 3, keys: &[(0.0, 0.0), (0.25, -2.9), (1.75, -2.9), (2.0, 0.0)] },
            Channel { part: "right_arm", axis: 5, keys: &[(0.0, 0.0), (0.25, 0.3), (0.55, -0.45), (0.85, 0.45), (1.15, -0.45), (1.45, 0.45), (1.75, 0.3), (2.0, 0.0)] },
        ],
    },
    Gesture {
        name: "Point",
        seconds: 1.6,
        channels: &[
            Channel { part: "right_arm", axis: 3, keys: &[(0.0, 0.0), (0.3, -1.57), (1.3, -1.57), (1.6, 0.0)] },
            Channel { part: "right_arm", axis: 4, keys: &[(0.0, 0.0), (0.3, -0.12), (1.3, -0.12), (1.6, 0.0)] },
        ],
    },
    Gesture {
        name: "Salute",
        seconds: 1.6,
        channels: &[
            Channel { part: "right_arm", axis: 3, keys: &[(0.0, 0.0), (0.3, -2.2), (1.3, -2.2), (1.6, 0.0)] },
            Channel { part: "right_arm", axis: 4, keys: &[(0.0, 0.0), (0.3, -0.55), (1.3, -0.55), (1.6, 0.0)] },
            Channel { part: "right_arm", axis: 5, keys: &[(0.0, 0.0), (0.3, 0.35), (1.3, 0.35), (1.6, 0.0)] },
        ],
    },
    Gesture {
        name: "Nod",
        seconds: 1.2,
        channels: &[Channel { part: "head", axis: 3, keys: &[(0.0, 0.0), (0.3, 0.42), (0.6, 0.05), (0.9, 0.42), (1.2, 0.0)] }],
    },
    Gesture {
        name: "Shake head",
        seconds: 1.2,
        channels: &[Channel { part: "head", axis: 4, keys: &[(0.0, 0.0), (0.2, -0.5), (0.5, 0.5), (0.8, -0.5), (1.1, 0.35), (1.2, 0.0)] }],
    },
    Gesture {
        name: "Clap",
        seconds: 1.8,
        channels: &[
            Channel { part: "right_arm", axis: 3, keys: &[(0.0, 0.0), (0.25, -1.45), (1.55, -1.45), (1.8, 0.0)] },
            Channel { part: "left_arm", axis: 3, keys: &[(0.0, 0.0), (0.25, -1.45), (1.55, -1.45), (1.8, 0.0)] },
            Channel { part: "right_arm", axis: 4, keys: &[(0.0, 0.0), (0.25, -0.5), (0.45, -0.2), (0.65, -0.5), (0.85, -0.2), (1.05, -0.5), (1.25, -0.2), (1.55, -0.5), (1.8, 0.0)] },
            Channel { part: "left_arm", axis: 4, keys: &[(0.0, 0.0), (0.25, 0.5), (0.45, 0.2), (0.65, 0.5), (0.85, 0.2), (1.05, 0.5), (1.25, 0.2), (1.55, 0.5), (1.8, 0.0)] },
        ],
    },
];

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
    if !on {
        *PLAYING.lock().unwrap() = None;
        PENDING.store(false, Ordering::Relaxed);
    }
}
pub fn selected() -> usize {
    SELECTED.load(Ordering::Relaxed) % GESTURES.len()
}
pub fn select_next() -> &'static str {
    let next = (selected() + 1) % GESTURES.len();
    SELECTED.store(next, Ordering::Relaxed);
    GESTURES[next].name
}
/// Starts the selected gesture at age-second `now` (`State.age` is ticks;
/// the caller passes `age / 20.0`).
pub fn play_selected(now: f32) {
    play_keyframes(selected(), now);
}
/// Starts gesture `index` (the public API other built-ins may drive).
pub fn play_keyframes(index: usize, now: f32) {
    if enabled() && index < GESTURES.len() {
        *PLAYING.lock().unwrap() = Some((index, now));
    }
}
pub fn playing() -> bool {
    PLAYING.lock().unwrap().is_some()
}

/// Smoothstep-eased sample of `keys` at `t`, 0 outside the span.
fn sample(keys: &[(f32, f32)], t: f32) -> f32 {
    let Some(&(first_t, first_v)) = keys.first() else {
        return 0.0;
    };
    let &(last_t, last_v) = keys.last().unwrap();
    if t <= first_t {
        return first_v;
    }
    if t >= last_t {
        return last_v;
    }
    for pair in keys.windows(2) {
        let [(t0, v0), (t1, v1)] = [pair[0], pair[1]];
        if t >= t0 && t <= t1 {
            let span = (t1 - t0).max(1e-6);
            let x = ((t - t0) / span).clamp(0.0, 1.0);
            let ease = x * x * (3.0 - 2.0 * x);
            return v0 + (v1 - v0) * ease;
        }
    }
    last_v
}

/// Layers the playing gesture over the finished pose of the OWN player
/// model; `anim::setup` calls it for humanoids after `nea::apply`.
pub fn apply(m: &Mob, s: &anim::State, p: &mut anim::Poses) {
    if !m.own || SUSPENDED.load(Ordering::Relaxed) {
        return;
    }
    let now = (s.age + s.partial) / 20.0;
    let mut playing = PLAYING.lock().unwrap();
    let Some((index, started)) = *playing else {
        return;
    };
    let Some(g) = GESTURES.get(index) else {
        *playing = None;
        return;
    };
    let t = now - started;
    // age can reset (a new world): treat going backwards as finished.
    if !(0.0..=g.seconds).contains(&t) {
        *playing = None;
        return;
    }
    drop(playing);
    apply_clip(p, g, t);
}

/// The pure layer: gesture `g` at clip-second `t` over `p`.
pub fn apply_clip(p: &mut anim::Poses, g: &Gesture, t: f32) {
    for c in g.channels {
        // The clip owns its parts for its whole run: every channel starts
        // and ends at the vanilla-relative zero, so there is no snap.
        let v = sample(c.keys, t);
        let slot = &mut p.get(c.part)[c.axis];
        *slot += v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobs::Part;

    fn poses() -> anim::Poses {
        let part = |name: &str| Part {
            name: name.into(),
            pose: [0.0; 9],
            polygons: Vec::new(),
            children: Vec::new(),
        };
        let mut root = part("body");
        root.children = ["head", "right_arm", "left_arm"].map(part).into();
        anim::Poses::new(&root)
    }

    #[test]
    fn sampling_eases_between_keys_and_clamps_outside() {
        let keys = [(0.0, 0.0), (1.0, 2.0)];
        assert_eq!(sample(&keys, -1.0), 0.0);
        assert_eq!(sample(&keys, 2.0), 2.0);
        assert_eq!(sample(&keys, 0.5), 1.0, "midpoint of smoothstep");
        assert!(sample(&keys, 0.25) < 0.5, "ease-in starts slow");
        assert!(sample(&keys, 0.75) > 1.5, "ease-out ends slow");
    }

    #[test]
    fn every_gesture_starts_and_ends_at_zero() {
        for g in GESTURES {
            for c in g.channels {
                assert_eq!(c.keys.first().unwrap().1, 0.0, "{}: starts neutral", g.name);
                assert_eq!(c.keys.last().unwrap().1, 0.0, "{}: ends neutral", g.name);
                assert!(
                    c.keys.last().unwrap().0 <= g.seconds + 1e-5,
                    "{}: keys inside the clip",
                    g.name
                );
                assert!((3..=5).contains(&c.axis));
            }
        }
    }

    #[test]
    fn wave_lifts_the_arm_mid_clip_and_releases_after() {
        let mut p = poses();
        apply_clip(&mut p, &GESTURES[0], 1.0);
        assert!(p.get("right_arm")[3] < -2.5, "arm up mid-wave");
        let mut p = poses();
        apply_clip(&mut p, &GESTURES[0], 2.0);
        assert!(p.get("right_arm")[3].abs() < 1e-5, "neutral at the end");
    }

    #[test]
    fn play_select_and_disable_round_trip() {
        set_enabled(true);
        let first = GESTURES[selected()].name;
        let next = select_next();
        assert_ne!(first, next);
        play_selected(10.0);
        assert!(playing());
        set_enabled(false);
        assert!(!playing(), "disabling stops the clip");
        set_enabled(true);
        // Selection survives; wrap-around reaches every gesture.
        for _ in 0..GESTURES.len() {
            select_next();
        }
        assert_eq!(GESTURES[selected()].name, next);
        // "Play now" queues; the clip starts when gameplay resumes.
        *PLAYING.lock().unwrap() = None;
        queue_selected();
        assert!(pending(), "queued, not yet playing");
        assert!(!playing());
        start_pending(3.0);
        assert!(playing(), "starts when gameplay resumes");
        assert!(!pending(), "the queue is consumed");
        set_enabled(false);
        queue_selected();
        assert!(!pending(), "disabled: nothing queues");
        set_enabled(true);
        queue_selected();
        set_enabled(false);
        assert!(!pending(), "disabling clears the queue");
        set_enabled(true);
    }
}
