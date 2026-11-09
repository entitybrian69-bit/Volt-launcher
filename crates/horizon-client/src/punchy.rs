//! Punchy, built into Volt Launcher: procedural first-person hand motion —
//! an extra movement sway while walking and sprinting, a spring-damped
//! landing dip scaled by fall speed, a jump drift, an idle breathing bob,
//! varied consecutive attack swings, and the option to show the empty off
//! hand. Everything is an extra transform on the vanilla first-person hand
//! matrices: item rendering, timing and gameplay are untouched.
//!
//! A clean-room recreation inspired by the feature set of PunchyDevGuy's
//! Punchy! for Minecraft (All Rights Reserved — no code or assets used).
//! The feature-by-feature parity table is in docs/BUILTIN-PARITY.md;
//! credits in THIRD_PARTY_NOTICES.md.
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const ID: &str = "punchy";
pub const NAME: &str = "Punchy";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Livelier first-person hands: movement sway, a landing dip that \
scales with the fall, a jump drift, idle breathing, varied consecutive attack swings and an \
optional visible empty off hand. Purely visual.";

static ENABLED: AtomicBool = AtomicBool::new(true);
static SWAY: AtomicBool = AtomicBool::new(true);
static LANDING: AtomicBool = AtomicBool::new(true);
static VARIETY: AtomicBool = AtomicBool::new(true);
static OFF_HAND: AtomicBool = AtomicBool::new(false);
/// 0 = Low, 1 = Medium, 2 = High.
static INTENSITY: AtomicUsize = AtomicUsize::new(1);
static STATE: Mutex<State> = Mutex::new(State::new());
/// Set while the mod's row in Options > Mods is switched off entirely.
static SUSPENDED: AtomicBool = AtomicBool::new(false);
pub fn set_suspended(off: bool) {
    SUSPENDED.store(off, Ordering::Relaxed);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flags {
    pub enabled: bool,
    pub sway: bool,
    pub landing: bool,
    pub variety: bool,
    pub off_hand: bool,
    pub intensity: usize,
}
pub fn flags() -> Flags {
    Flags {
        enabled: ENABLED.load(Ordering::Relaxed),
        sway: SWAY.load(Ordering::Relaxed),
        landing: LANDING.load(Ordering::Relaxed),
        variety: VARIETY.load(Ordering::Relaxed),
        off_hand: OFF_HAND.load(Ordering::Relaxed),
        intensity: INTENSITY.load(Ordering::Relaxed).min(2),
    }
}
pub fn set_flags(f: Flags) {
    ENABLED.store(f.enabled, Ordering::Relaxed);
    SWAY.store(f.sway, Ordering::Relaxed);
    LANDING.store(f.landing, Ordering::Relaxed);
    VARIETY.store(f.variety, Ordering::Relaxed);
    OFF_HAND.store(f.off_hand, Ordering::Relaxed);
    INTENSITY.store(f.intensity.min(2), Ordering::Relaxed);
}
pub const INTENSITY_NAMES: [&str; 3] = ["Low", "Medium", "High"];
pub fn intensity_scale(i: usize) -> f32 {
    [0.5, 1.0, 1.5][i.min(2)]
}
/// Whether the empty off hand should render in first person.
pub fn show_empty_off_hand() -> bool {
    let f = flags();
    !SUSPENDED.load(Ordering::Relaxed) && f.enabled && f.off_hand
}

/// What the state machine reads of the player each frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Inputs {
    pub dt: f32,
    pub on_ground: bool,
    pub sprinting: bool,
    /// Horizontal speed, blocks per second.
    pub speed: f32,
    /// Vertical velocity, blocks per second (negative is falling).
    pub vel_y: f32,
    /// Whether a swing animation is in progress this frame.
    pub swinging: bool,
}

/// The hand offset a frame produces: metres and radians, before the
/// intensity scale.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Offset {
    pub dx: f32,
    pub dy: f32,
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

/// The procedural state machine: a walk phase, a damped landing spring,
/// an idle clock and the attack-variety counter.
#[derive(Clone, Copy, Debug)]
pub struct State {
    walk_phase: f32,
    idle_clock: f32,
    spring_pos: f32,
    spring_vel: f32,
    fall_speed: f32,
    attack_count: u32,
    attack_timer: f32,
    prev_on_ground: bool,
    prev_swinging: bool,
}
impl State {
    pub const fn new() -> Self {
        Self {
            walk_phase: 0.0,
            idle_clock: 0.0,
            spring_pos: 0.0,
            spring_vel: 0.0,
            fall_speed: 0.0,
            attack_count: 0,
            attack_timer: 0.0,
            prev_on_ground: true,
            prev_swinging: false,
        }
    }
    /// Advances by one frame and returns this frame's raw offset.
    pub fn tick(&mut self, i: &Inputs, f: Flags) -> Offset {
        let dt = i.dt.clamp(0.0, 0.1);
        let mut out = Offset::default();
        // Movement sway: the walk cycle rolls and drifts the hands a touch
        // beyond vanilla's bob, stronger while sprinting.
        let walking = i.speed > 0.4;
        if walking {
            let rate = if i.sprinting { 11.0 } else { 7.5 };
            self.walk_phase += dt * rate * (i.speed / 4.3).clamp(0.3, 1.6);
        }
        if f.sway && walking {
            let amp = if i.sprinting { 1.8 } else { 1.0 };
            out.roll += self.walk_phase.sin() * 0.014 * amp;
            out.dx += self.walk_phase.sin() * 0.006 * amp;
            out.dy += (self.walk_phase * 2.0).cos() * 0.004 * amp;
        }
        // Idle breathing, when nearly still.
        self.idle_clock += dt;
        if f.sway && !walking {
            out.dy += (self.idle_clock * 1.6).sin() * 0.003;
            out.pitch += (self.idle_clock * 1.1).cos() * 0.002;
        }
        // Jumping and falling: the hands trail the vertical motion.
        if !i.on_ground {
            self.fall_speed = (-i.vel_y).max(self.fall_speed * 0.98);
            out.dy += (i.vel_y * 0.012).clamp(-0.05, 0.05);
            out.pitch += (i.vel_y * 0.01).clamp(-0.04, 0.04);
        }
        // Landing: a one-off downward impulse into a damped spring, scaled
        // by how fast the fall was.
        if i.on_ground && !self.prev_on_ground && f.landing {
            self.spring_vel -= (self.fall_speed.clamp(0.0, 20.0) / 20.0) * 1.6;
            self.fall_speed = 0.0;
        }
        if i.on_ground {
            self.fall_speed = 0.0;
        }
        let (k, c) = (160.0, 14.0);
        self.spring_vel += (-k * self.spring_pos - c * self.spring_vel) * dt;
        self.spring_pos += self.spring_vel * dt;
        out.dy += self.spring_pos;
        out.pitch += self.spring_pos * 0.35;
        // Attack variety: consecutive swings cycle three variants so they
        // stop looking identical; each fades over its window.
        if i.swinging && !self.prev_swinging {
            self.attack_count = self.attack_count.wrapping_add(1);
            self.attack_timer = 0.3;
        }
        if self.attack_timer > 0.0 {
            self.attack_timer = (self.attack_timer - dt).max(0.0);
            if f.variety {
                let fade = self.attack_timer / 0.3;
                match self.attack_count % 3 {
                    1 => out.yaw += 0.07 * fade,
                    2 => {
                        out.roll -= 0.09 * fade;
                        out.yaw -= 0.04 * fade;
                    }
                    _ => {} // every third swing stays vanilla
                }
            }
        }
        self.prev_on_ground = i.on_ground;
        self.prev_swinging = i.swinging;
        out
    }
    pub fn attack_variant(&self) -> u32 {
        self.attack_count % 3
    }
}

/// The per-frame tick the game bridge drives; stores the scaled offset.
static OFFSET: Mutex<Offset> = Mutex::new(Offset {
    dx: 0.0,
    dy: 0.0,
    roll: 0.0,
    pitch: 0.0,
    yaw: 0.0,
});
pub fn frame(i: &Inputs) {
    let f = flags();
    let mut state = STATE.lock().unwrap();
    if !f.enabled || SUSPENDED.load(Ordering::Relaxed) {
        *state = State::new();
        *OFFSET.lock().unwrap() = Offset::default();
        return;
    }
    let raw = state.tick(i, f);
    let s = intensity_scale(f.intensity);
    *OFFSET.lock().unwrap() = Offset {
        dx: raw.dx * s,
        dy: raw.dy * s,
        roll: raw.roll * s,
        pitch: raw.pitch * s,
        yaw: raw.yaw * s,
    };
}

/// The extra hand transform for this frame: identity while disabled.
pub fn hand_matrix() -> glam::Mat4 {
    if !flags().enabled || SUSPENDED.load(Ordering::Relaxed) {
        return glam::Mat4::IDENTITY;
    }
    let o = *OFFSET.lock().unwrap();
    glam::Mat4::from_translation(glam::Vec3::new(o.dx, o.dy, 0.0))
        * glam::Mat4::from_rotation_z(o.roll)
        * glam::Mat4::from_rotation_x(o.pitch)
        * glam::Mat4::from_rotation_y(o.yaw)
}

#[cfg(test)]
mod tests {
    use super::*;
    const ON: Flags = Flags {
        enabled: true,
        sway: true,
        landing: true,
        variety: true,
        off_hand: false,
        intensity: 1,
    };
    fn airborne(vel_y: f32) -> Inputs {
        Inputs {
            dt: 1.0 / 60.0,
            on_ground: false,
            vel_y,
            ..Inputs::default()
        }
    }
    fn grounded() -> Inputs {
        Inputs {
            dt: 1.0 / 60.0,
            on_ground: true,
            ..Inputs::default()
        }
    }

    #[test]
    fn landing_dips_then_settles() {
        let mut s = State::new();
        for _ in 0..30 {
            s.tick(&airborne(-12.0), ON);
        }
        let first = s.tick(&grounded(), ON);
        let mut lowest = first.dy;
        for _ in 0..10 {
            lowest = lowest.min(s.tick(&grounded(), ON).dy);
        }
        assert!(lowest < -0.01, "landing dips the hands: {lowest}");
        for _ in 0..300 {
            s.tick(&grounded(), ON);
        }
        let settled = s.tick(&grounded(), ON);
        assert!(settled.dy.abs() < 0.004, "spring settles: {}", settled.dy);
        // With the landing toggle off, no dip at all.
        let mut s = State::new();
        let off = Flags { landing: false, sway: false, ..ON };
        for _ in 0..30 {
            s.tick(&airborne(-12.0), off);
        }
        for _ in 0..10 {
            assert!(s.tick(&grounded(), off).dy.abs() < 1e-4);
        }
    }

    #[test]
    fn sway_is_bounded_and_only_while_moving() {
        let mut s = State::new();
        let run = Inputs {
            dt: 1.0 / 60.0,
            on_ground: true,
            sprinting: true,
            speed: 5.6,
            ..Inputs::default()
        };
        let mut peak: f32 = 0.0;
        for _ in 0..240 {
            let o = s.tick(&run, ON);
            peak = peak.max(o.roll.abs());
            assert!(o.roll.abs() < 0.05 && o.dx.abs() < 0.02);
        }
        assert!(peak > 0.005, "sprint sway is visible: {peak}");
        let o = State::new().tick(
            &grounded(),
            Flags { sway: false, ..ON },
        );
        assert_eq!(o.roll, 0.0);
    }

    #[test]
    fn attack_variants_cycle_and_fade() {
        let mut s = State::new();
        let swing = Inputs {
            dt: 1.0 / 60.0,
            on_ground: true,
            swinging: true,
            ..Inputs::default()
        };
        let mut seen = [false; 3];
        for _ in 0..3 {
            s.tick(&swing, ON);
            seen[s.attack_variant() as usize] = true;
            for _ in 0..30 {
                s.tick(&grounded(), ON); // swing released, timer runs out
            }
        }
        assert_eq!(seen, [true; 3], "three variants in rotation");
        // Holding the button is one swing: the counter moves once.
        let mut s = State::new();
        for _ in 0..20 {
            s.tick(&swing, ON);
        }
        assert_eq!(s.attack_variant(), 1);
    }

    #[test]
    fn disabled_means_identity_and_flags_round_trip() {
        let before = flags();
        set_flags(Flags { enabled: false, ..before });
        frame(&grounded());
        assert_eq!(hand_matrix(), glam::Mat4::IDENTITY);
        assert!(!show_empty_off_hand());
        set_flags(Flags {
            enabled: true,
            off_hand: true,
            intensity: 9,
            ..before
        });
        assert!(show_empty_off_hand());
        assert_eq!(flags().intensity, 2, "intensity stored clamped");
        set_flags(before);
    }
}
