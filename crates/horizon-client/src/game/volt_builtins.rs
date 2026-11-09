//! The bridge for Volt's in-client built-in mods — Not Enough Animations
//! (`mobs::nea`), Player Animations (`mobs::player_gestures`), Subtle
//! Effects (`subtle_effects`) and Overhauled Entities
//! (`mobs::overhauled`) — into the native-mod surface: rows and
//! settings pages in Options > Mods, keybinds, per-key persistence through
//! `native_setting`, and the per-frame feed of local player state. The
//! logic lives in the library modules; this file only translates. Both
//! platforms: nothing here is Android-specific. (Punchy is no longer a
//! built-in: the real Punchy native-mod port is woven into Android builds
//! from `tools/native_mods/vendored/punchy`.)
use super::{Game, native_hooks::NativeMod};
use horizon_client::mobs::anim::Anim;
use horizon_client::mobs::{etf, nea, overhauled, overhauled_clips as clips, player_gestures as gestures};
use horizon_client::particles::{ParticleOptions, ParticleType as T};
use horizon_client::subtle_effects as subtle;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use winit::keyboard::KeyCode;

/// Accumulates frame time so mob-attached particles run at tick rate.
static MOB_FX_CLOCK: Mutex<Duration> = Mutex::new(Duration::ZERO);
/// The Overhauled Entities environment pass runs on its own tick clock.
static OVH_FX_CLOCK: Mutex<Duration> = Mutex::new(Duration::ZERO);
/// A rolling counter that seeds the stateless particle randomness.
static FX_SALT: AtomicU32 = AtomicU32::new(0);
/// The own player's trigger edges (flip, roll, sit) between frames.
#[derive(Default)]
struct OwnState {
    prev_ground: bool,
    prev_crouch: bool,
    sneak_for: f32,
    sitting: bool,
    peak_fall: f32,
    fell_sprinting: bool,
}
static OWN_STATE: Mutex<OwnState> = Mutex::new(OwnState {
    prev_ground: false,
    prev_crouch: false,
    sneak_for: 0.0,
    sitting: false,
    peak_fall: 0.0,
    fell_sprinting: false,
});
/// Whether the gesture layer currently borrows the third-person front
/// camera (set while a clip plays from first person, cleared on restore).
static GESTURE_CAMERA: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
const ON: &str = "ON";
const OFF: &str = "OFF";
fn label(name: &str, on: bool) -> String {
    format!("{name}: {}", if on { ON } else { OFF })
}

impl Game<'_> {
    fn stored_bool(&self, id: &str, key: &str, default: bool) -> bool {
        match self.native_setting(id, key).as_deref() {
            Some("on") => true,
            Some("off") => false,
            _ => default,
        }
    }
    fn store_bool(&self, id: &str, key: &str, v: bool) {
        self.native_set_setting(id, key, if v { "on" } else { "off" });
    }
    fn stored_usize(&self, id: &str, key: &str, default: usize) -> usize {
        self.native_setting(id, key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    /// Loads every built-in's stored settings into its statics; ran when
    /// the mod list is built so toggles survive restarts.
    pub(super) fn volt_builtins_load(&self) {
        nea::set_flags(nea::Flags {
            enabled: self.stored_bool(nea::ID, "enabled", true),
            eating: self.stored_bool(nea::ID, "eating", true),
            maps: self.stored_bool(nea::ID, "maps", true),
            climbing: self.stored_bool(nea::ID, "climbing", true),
            boats: self.stored_bool(nea::ID, "boats", true),
        });
        subtle::set_flags(subtle::Flags {
            enabled: self.stored_bool(subtle::ID, "enabled", true),
            sparks: self.stored_bool(subtle::ID, "sparks", true),
            dust: self.stored_bool(subtle::ID, "dust", true),
            burning: self.stored_bool(subtle::ID, "burning", true),
            slimes: self.stored_bool(subtle::ID, "slimes", true),
            density: self.stored_usize(subtle::ID, "density", 1),
        });
        gestures::set_enabled(self.stored_bool(gestures::ID, "enabled", true));
        overhauled::set_flags(overhauled::Flags {
            enabled: self.stored_bool(overhauled::ID, "enabled", true),
            players: self.stored_bool(overhauled::ID, "players", true),
            mobs: self.stored_bool(overhauled::ID, "mobs", true),
            rain: self.stored_bool(overhauled::ID, "rain", true),
            intensity: self.stored_usize(overhauled::ID, "intensity", 1),
        });
    }

    /// Options > Mods rows, after the woven and bundled mods' rows.
    pub(super) fn volt_builtins_list(&self, mods: &mut Vec<NativeMod>) {
        self.volt_builtins_load();
        let row = |id: &str, name: &str, version: &str, description: &str,
                   keys: Vec<[String; 3]>| NativeMod {
            id: id.into(),
            title: format!("{name} {version}"),
            description: description.into(),
            keys,
        };
        mods.push(row(nea::ID, nea::NAME, nea::VERSION, nea::DESCRIPTION, vec![[
            "toggle".into(),
            "Toggle animations".into(),
            "key.keyboard.unknown".into(),
        ]]));
        mods.push(row(subtle::ID, subtle::NAME, subtle::VERSION, subtle::DESCRIPTION, vec![]));
        mods.push(row(
            overhauled::ID,
            overhauled::NAME,
            overhauled::VERSION,
            overhauled::DESCRIPTION,
            vec![],
        ));
        mods.push(row(
            gestures::ID,
            gestures::NAME,
            gestures::VERSION,
            gestures::DESCRIPTION,
            vec![[
                "gesture".into(),
                "Play gesture".into(),
                "key.keyboard.b".into(),
            ]],
        ));
    }

    /// The settings page labels, asked each time a page opens.
    pub(super) fn volt_builtins_settings(&self, id: &str, buttons: &mut Vec<String>) {
        match id {
            _ if id == nea::ID => {
                let f = nea::flags();
                buttons.push(label("Animations", f.enabled));
                buttons.push(label("Eating & drinking", f.eating));
                buttons.push(label("Map holding", f.maps));
                buttons.push(label("Ladder climbing", f.climbing));
                buttons.push(label("Boat rowing", f.boats));
            }
            _ if id == subtle::ID => {
                let f = subtle::flags();
                buttons.push(label("Subtle Effects", f.enabled));
                buttons.push(label("Flame sparks", f.sparks));
                buttons.push(label("Glowing block dust", f.dust));
                buttons.push(label("Burning mob embers", f.burning));
                buttons.push(label("Slime trails", f.slimes));
                buttons.push(format!("Density: {}", subtle::DENSITY_NAMES[f.density]));
            }
            _ if id == overhauled::ID => {
                let f = overhauled::flags();
                buttons.push(label("Overhauled Entities", f.enabled));
                buttons.push(label("Player animations", f.players));
                buttons.push(label("Mob animations", f.mobs));
                buttons.push(label("Better rain particles", f.rain));
                buttons.push(format!(
                    "Intensity: {}",
                    overhauled::INTENSITY_NAMES[f.intensity]
                ));
            }
            _ if id == gestures::ID => {
                buttons.push(label("Gestures", gestures::enabled()));
                buttons.push(format!(
                    "Gesture: {}",
                    gestures::GESTURES[gestures::selected()].name
                ));
                buttons.push("Play now".into());
            }
            _ => {}
        }
    }

    /// A click on a settings page button.
    pub(super) fn volt_builtins_click(&mut self, id: &str, button: usize) {
        if id == nea::ID {
            let mut f = nea::flags();
            let (key, v) = match button {
                0 => { f.enabled = !f.enabled; ("enabled", f.enabled) }
                1 => { f.eating = !f.eating; ("eating", f.eating) }
                2 => { f.maps = !f.maps; ("maps", f.maps) }
                3 => { f.climbing = !f.climbing; ("climbing", f.climbing) }
                4 => { f.boats = !f.boats; ("boats", f.boats) }
                _ => return,
            };
            nea::set_flags(f);
            self.store_bool(nea::ID, key, v);
        } else if id == subtle::ID {
            let mut f = subtle::flags();
            match button {
                0 => { f.enabled = !f.enabled; }
                1 => { f.sparks = !f.sparks; }
                2 => { f.dust = !f.dust; }
                3 => { f.burning = !f.burning; }
                4 => { f.slimes = !f.slimes; }
                5 => { f.density = (f.density + 1) % 3; }
                _ => return,
            }
            subtle::set_flags(f);
            let keys = ["enabled", "sparks", "dust", "burning", "slimes"];
            if button < 5 {
                let v = [f.enabled, f.sparks, f.dust, f.burning, f.slimes][button];
                self.store_bool(subtle::ID, keys[button], v);
            } else {
                self.native_set_setting(subtle::ID, "density", &f.density.to_string());
            }
        } else if id == overhauled::ID {
            let mut f = overhauled::flags();
            match button {
                0 => { f.enabled = !f.enabled; }
                1 => { f.players = !f.players; }
                2 => { f.mobs = !f.mobs; }
                3 => { f.rain = !f.rain; }
                4 => { f.intensity = (f.intensity + 1) % 3; }
                _ => return,
            }
            overhauled::set_flags(f);
            if button < 4 {
                let keys = ["enabled", "players", "mobs", "rain"];
                let v = [f.enabled, f.players, f.mobs, f.rain][button];
                self.store_bool(overhauled::ID, keys[button], v);
            } else {
                self.native_set_setting(overhauled::ID, "intensity", &f.intensity.to_string());
            }
        } else if id == gestures::ID {
            match button {
                0 => {
                    let on = !gestures::enabled();
                    gestures::set_enabled(on);
                    self.store_bool(gestures::ID, "enabled", on);
                }
                1 => {
                    let name = gestures::select_next();
                    self.native_set_setting(gestures::ID, "selected", name);
                }
                // "Play now" queues the clip: it starts on the first frame
                // back in the world (`volt_builtins_frame`), so it is not
                // already over while the settings page still covers the
                // screen.
                2 => gestures::queue_selected(),
                _ => {}
            }
        }
    }

    /// A key went down: the NEA toggle and the gesture key.
    pub(super) fn volt_builtins_key(&mut self, key: &str, pressed: bool) {
        if !pressed || !self.native_playing() {
            return;
        }
        if self.native_on(nea::ID)
            && key == self.native_mod_key(nea::ID, "toggle", "key.keyboard.unknown")
        {
            let mut f = nea::flags();
            f.enabled = !f.enabled;
            nea::set_flags(f);
            self.store_bool(nea::ID, "enabled", f.enabled);
        }
        if self.native_on(gestures::ID)
            && key == self.native_mod_key(gestures::ID, "gesture", "key.keyboard.b")
        {
            self.volt_builtins_gesture();
        }
    }
    fn volt_builtins_gesture(&self) {
        if let Some(m) = self.player_model.as_ref() {
            gestures::play_selected(m.age as f32 / 20.0);
        }
    }

    /// Once a frame, beside `native_frame`: feeds the local player's state
    /// to NEA, starts queued gestures, and runs mob-attached Subtle Effects
    /// particles at tick rate. Each branch is gated on its row's toggle.
    pub(super) fn volt_builtins_frame(&mut self, elapsed: Duration) {
        // The row toggles in Options > Mods suspend each mod wholesale,
        // including the code paths that run outside this bridge.
        nea::set_suspended(!self.native_on(nea::ID));
        gestures::set_suspended(!self.native_on(gestures::ID));
        subtle::set_suspended(!self.native_on(subtle::ID));
        overhauled::set_suspended(!self.native_on(overhauled::ID));
        // (Light Overlay's stale persisted `active` flag is handled at the
        // source: `native_hooks::native_setting` forces reads of it to
        // "false" until the first in-session write, so the overlay is
        // session-only with no frame-order race.)
        // A gesture queued from the settings page ("Play now") starts on
        // the first frame the player is back in the world, so the clip is
        // not already over behind the paused menu.
        if gestures::pending()
            && self.native_playing()
            && let Some(m) = self.player_model.as_ref()
        {
            gestures::start_pending(m.age as f32 / 20.0);
        }
        // A gesture is invisible from inside the player's own head: while
        // one plays from first person, borrow the front third-person
        // camera (as emote mods do), and give it back when the clip ends.
        // A manual perspective change while borrowed is respected: the
        // restore only touches the camera if it is still the front view.
        {
            use super::perspective::CameraType;
            let now = self.player_model.as_ref().map(|m| m.age as f32 / 20.0);
            let active = now.is_some_and(gestures::still_playing);
            let borrowed = GESTURE_CAMERA.load(std::sync::atomic::Ordering::Relaxed);
            if active && !borrowed && self.camera_type.first_person() {
                GESTURE_CAMERA.store(true, std::sync::atomic::Ordering::Relaxed);
                self.camera_type = CameraType::ThirdPersonFront;
                self.check_entity_post_effect(false);
            } else if !active && borrowed {
                GESTURE_CAMERA.store(false, std::sync::atomic::Ordering::Relaxed);
                if self.camera_type == CameraType::ThirdPersonFront {
                    self.camera_type = CameraType::FirstPerson;
                    self.check_entity_post_effect(true);
                }
            }
        }
        // Not Enough Animations: the local climbing and boat state (other
        // players' mobs carry theirs on the Mob).
        if self.native_on(nea::ID) {
            match self.player.as_ref() {
                Some(p) => {
                    let feet = [
                        p.position.x.floor() as i32,
                        p.position.y.floor() as i32,
                        p.position.z.floor() as i32,
                    ];
                    let on_climbable = p.last_climbable.is_some()
                        || self.block_in_tag("minecraft:climbable", &self.block_name(feet));
                    let climbing = !p.flying && !p.ground.on_ground && on_climbable;
                    let in_boat = self
                        .riding
                        .is_some_and(|id| self.boats.iter().any(|b| b.id == id));
                    let moving =
                        in_boat && (p.velocity.x.abs() > 0.01 || p.velocity.z.abs() > 0.01);
                    nea::set_local(climbing, in_boat, moving);
                }
                None => nea::set_local(false, false, false),
            }
        } else {
            nea::set_local(false, false, false);
        }
        // Overhauled Entities: the own player's trigger edges and the
        // environment particle pass (before Subtle Effects' early return).
        self.volt_overhauled_frame(elapsed);
        // Subtle Effects on mobs, at tick rate: embers on burning mobs and
        // slime trails, for mobs near the camera.
        if !self.native_on(subtle::ID) {
            return;
        }
        {
            let f = subtle::flags();
            if !f.enabled || (!f.burning && !f.slimes) {
                return;
            }
        }
        let due = {
            let mut clock = MOB_FX_CLOCK.lock().unwrap();
            *clock += elapsed;
            let due = *clock >= Duration::from_millis(50);
            if due {
                *clock = Duration::ZERO;
            }
            due
        };
        if !due || self.player.is_none() {
            return;
        }
        let eye = self.eye_position().unwrap_or_default();
        self.ensure_item_particle_sprites();
        let (mobs, particles) = (&self.mobs, &mut self.particles);
        for m in mobs.iter().filter(|m| !m.removed && m.alive()) {
            let d = [
                m.position.x - eye[0],
                m.position.y - eye[1],
                m.position.z - eye[2],
            ];
            if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > 32.0 * 32.0 {
                continue;
            }
            let feet = [m.position.x, m.position.y, m.position.z];
            if m.is_on_fire() {
                subtle::burning_mob(particles, feet, m.kind.width, m.kind.height);
            }
            if subtle::is_slime(&m.kind.id)
                && m.on_ground
                && m.limb_speed > 0.01
            {
                subtle::slime_trail(particles, feet, m.kind.width);
            }
        }
    }

    /// Overhauled Entities, once a frame: feeds the environment (full-moon
    /// nights for the wolf howl), the own player's crouch and ledge state,
    /// fires the own-player clip triggers (flip, roll, sit), and runs the
    /// environment particles at tick rate — rain impact splashes, ambient
    /// air motes, boat wakes and oar splashes, XP-orb trails, mining
    /// sparks, glow-squid twinkles and dive-entry bubble bursts.
    fn volt_overhauled_frame(&mut self, elapsed: Duration) {
        if !self.native_on(overhauled::ID) {
            overhauled::set_own_edge(false);
            return;
        }
        let f = overhauled::flags();
        if !f.enabled {
            overhauled::set_own_edge(false);
            return;
        }
        // Wolves howl on full-moon nights: day 0 of the 8-day moon cycle.
        let dt = self.world.day_time.rem_euclid(192_000);
        overhauled::set_env((13_000..=23_000).contains(&(dt % 24_000)) && dt / 24_000 == 0);
        let crouch = self.native_playing() && self.keys.contains(&KeyCode::ShiftLeft);
        overhauled::set_own_crouch(crouch);
        // The own player's trigger edges: crouch mid-air flips, landing a
        // sprint fall of three-plus blocks rolls, and five seconds of
        // sneaking while looking down sits (released with the sneak key).
        if let (Some(p), Some(mm)) = (self.player.as_ref(), self.player_model.as_ref()) {
            let now = mm.age as f32 / 20.0;
            let own = mm.id;
            let airborne = !p.flying && !p.ground.on_ground && !p.swimming;
            let mut st = OWN_STATE.lock().unwrap();
            if airborne {
                st.peak_fall = st.peak_fall.max(p.fall_distance as f32);
                st.fell_sprinting |= p.sprinting;
            }
            if crouch && !st.prev_crouch && airborne {
                clips::trigger(own, clips::Clip::Flip, now);
            }
            if p.ground.on_ground && !st.prev_ground && st.fell_sprinting && st.peak_fall >= 3.0 {
                clips::trigger(own, clips::Clip::Roll, now);
            }
            if p.ground.on_ground {
                st.peak_fall = 0.0;
                st.fell_sprinting = false;
            }
            let still = p.velocity.x.abs() + p.velocity.z.abs() < 0.01;
            if crouch && p.ground.on_ground && still {
                st.sneak_for += elapsed.as_secs_f32();
            } else {
                st.sneak_for = 0.0;
            }
            if !st.sitting && st.sneak_for >= 5.0 && self.pitch > 35.0 {
                st.sitting = true;
                clips::trigger(own, clips::Clip::Sit, now);
            }
            if st.sitting && !crouch {
                st.sitting = false;
                clips::clear(own);
            }
            st.prev_crouch = crouch;
            st.prev_ground = p.ground.on_ground;
        }
        // Everything below runs at tick rate, not frame rate.
        let due = {
            let mut clock = OVH_FX_CLOCK.lock().unwrap();
            *clock += elapsed;
            let due = *clock >= Duration::from_millis(50);
            if due {
                *clock = Duration::ZERO;
            }
            due
        };
        if !due || self.player.is_none() {
            return;
        }
        let Some(eye) = self.eye_position() else { return };
        // The ledge probe for the balance pose: standing on a block with
        // open air under two or more of its sides.
        if let Some(p) = self.player.as_ref() {
            let bx = p.position.x.floor() as i32;
            let by = (p.position.y - 0.5).floor() as i32;
            let bz = p.position.z.floor() as i32;
            let air = |x: i32, z: i32| self.block_name([x, by, z]).ends_with("air");
            let open = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .filter(|&&(dx, dz)| air(bx + dx, bz + dz))
                .count();
            overhauled::set_own_edge(p.ground.on_ground && !crouch && !air(bx, bz) && open >= 2);
        }
        // Stateless randomness for the particle pass.
        let rnd = |stream: u32| etf::entity_random(FX_SALT.fetch_add(1, Ordering::Relaxed), stream);
        let mut spawn: Vec<(T, [f64; 3], [f64; 3])> = Vec::new();
        // Rain impact splashes: random nearby columns, splashing on the
        // first surface under open sky (a covered probe is skipped, so
        // caves and interiors stay dry).
        if self.raining && f.rain {
            let top = (eye[1] + 6.0).floor() as i32;
            for _ in 0..6 {
                let x = eye[0] + f64::from(rnd(1) - 0.5) * 16.0;
                let z = eye[2] + f64::from(rnd(2) - 0.5) * 16.0;
                let (bx, bz) = (x.floor() as i32, z.floor() as i32);
                if !self.block_name([bx, top, bz]).ends_with("air") {
                    continue;
                }
                for yy in (top - 16..top).rev() {
                    if !self.block_name([bx, yy, bz]).ends_with("air") {
                        let at = [x, f64::from(yy) + 1.05, z];
                        spawn.push((T::Splash, at, [0.0; 3]));
                        if rnd(3) < 0.5 {
                            spawn.push((T::Rain, at, [0.0; 3]));
                        }
                        break;
                    }
                }
            }
        }
        // Ambient air motes: sparse dust drifting in the air nearby.
        if rnd(4) < 0.35 {
            let at = [
                eye[0] + f64::from(rnd(5) - 0.5) * 18.0,
                eye[1] + f64::from(rnd(6) - 0.5) * 8.0,
                eye[2] + f64::from(rnd(7) - 0.5) * 18.0,
            ];
            if self
                .block_name([at[0].floor() as i32, at[1].floor() as i32, at[2].floor() as i32])
                .ends_with("air")
            {
                spawn.push((T::Mycelium, at, [0.0, -0.002, 0.0]));
            }
        }
        // Boat wakes: trailing splashes while under way, oar splashes
        // while paddling, idle foam while afloat.
        for b in &self.boats {
            let d = [b.position.x - eye[0], b.position.y - eye[1], b.position.z - eye[2]];
            if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > 32.0 * 32.0 || b.on_ground {
                continue;
            }
            let yaw = f64::from(b.yaw).to_radians();
            let (fx, fz) = (-yaw.sin(), yaw.cos());
            let speed2 = b.velocity.x * b.velocity.x + b.velocity.z * b.velocity.z;
            if speed2 > 0.0009 {
                let stern = [
                    b.position.x - fx * 1.1 + f64::from(rnd(8) - 0.5) * 0.8,
                    b.position.y + 0.05,
                    b.position.z - fz * 1.1 + f64::from(rnd(9) - 0.5) * 0.8,
                ];
                spawn.push((T::Splash, stern, [-b.velocity.x * 0.3, 0.04, -b.velocity.z * 0.3]));
                if rnd(10) < 0.6 {
                    spawn.push((
                        T::BubblePop,
                        [b.position.x + fz * 0.7, b.position.y + 0.02, b.position.z - fx * 0.7],
                        [0.0; 3],
                    ));
                }
            } else if rnd(11) < 0.35 {
                // Stationary foam lapping at the hull.
                let a = f64::from(rnd(12)) * std::f64::consts::TAU;
                spawn.push((
                    T::BubblePop,
                    [b.position.x + a.cos() * 0.8, b.position.y + 0.02, b.position.z + a.sin() * 0.8],
                    [0.0; 3],
                ));
            }
            for (i, side) in [(0usize, 0.9f64), (1, -0.9)] {
                if b.paddling[i] && rnd(13 + i as u32) < 0.7 {
                    spawn.push((
                        T::Splash,
                        [
                            b.position.x + fz * side - fx * 0.2,
                            b.position.y + 0.05,
                            b.position.z - fx * side - fz * 0.2,
                        ],
                        [0.0, 0.05, 0.0],
                    ));
                }
            }
        }
        // XP orbs leave a twinkling trail.
        for o in &self.orbs {
            let d = [o.position.x - eye[0], o.position.y - eye[1], o.position.z - eye[2]];
            if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > 32.0 * 32.0 || rnd(15) >= 0.45 {
                continue;
            }
            spawn.push((
                T::Glow,
                [
                    o.position.x + f64::from(rnd(16) - 0.5) * 0.3,
                    o.position.y + 0.1 + f64::from(rnd(17) - 0.5) * 0.3,
                    o.position.z + f64::from(rnd(18) - 0.5) * 0.3,
                ],
                [0.0, 0.01, 0.0],
            ));
        }
        // Mining sparks: chips fly from the face being broken.
        if let Some(mi) = self.mining.as_ref()
            && mi.progress > 0.02
        {
            let center = [
                f64::from(mi.pos[0]) + 0.5,
                f64::from(mi.pos[1]) + 0.5,
                f64::from(mi.pos[2]) + 0.5,
            ];
            let d = [eye[0] - center[0], eye[1] - center[1], eye[2] - center[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(0.001);
            for _ in 0..2 {
                let at = [
                    center[0] + d[0] / len * 0.55 + f64::from(rnd(19) - 0.5) * 0.5,
                    center[1] + d[1] / len * 0.55 + f64::from(rnd(20) - 0.5) * 0.5,
                    center[2] + d[2] / len * 0.55 + f64::from(rnd(21) - 0.5) * 0.5,
                ];
                spawn.push((T::Crit, at, [0.0, 0.02, 0.0]));
            }
        }
        // Mobs: glow-squid twinkles and dive-entry bubble bursts.
        for m in self.mobs.iter().filter(|m| !m.removed && m.alive()) {
            let d = [m.position.x - eye[0], m.position.y - eye[1], m.position.z - eye[2]];
            if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > 32.0 * 32.0 {
                continue;
            }
            if m.kind.id.contains("glow_squid") && rnd(22) < 0.3 {
                spawn.push((
                    T::Glow,
                    [
                        m.position.x + f64::from(rnd(23) - 0.5) * m.kind.width,
                        m.position.y + f64::from(rnd(24)) * m.kind.height,
                        m.position.z + f64::from(rnd(25) - 0.5) * m.kind.width,
                    ],
                    [0.0; 3],
                ));
            }
            if matches!(m.kind.anim, Anim::Humanoid { .. }) && clips::take_splash(m.id) {
                splash_burst(&mut spawn, [m.position.x, m.position.y, m.position.z], &rnd);
            }
        }
        // The own player's dive splash (the own model is not in `mobs`).
        if let Some(mm) = self.player_model.as_ref()
            && clips::take_splash(mm.id)
        {
            splash_burst(&mut spawn, [mm.position.x, mm.position.y, mm.position.z], &rnd);
        }
        let particles = &mut self.particles;
        for (ty, at, vel) in spawn {
            particles.add(&ParticleOptions::simple(ty), at, vel);
        }
    }
}

/// The bubble-and-spray burst a dive, cannonball or belly flop kicks up.
fn splash_burst(
    spawn: &mut Vec<(T, [f64; 3], [f64; 3])>,
    feet: [f64; 3],
    rnd: &impl Fn(u32) -> f32,
) {
    for _ in 0..8 {
        spawn.push((
            T::Bubble,
            [
                feet[0] + f64::from(rnd(26) - 0.5) * 0.9,
                feet[1] + f64::from(rnd(27)) * 0.6,
                feet[2] + f64::from(rnd(28) - 0.5) * 0.9,
            ],
            [
                f64::from(rnd(29) - 0.5) * 0.1,
                0.08,
                f64::from(rnd(30) - 0.5) * 0.1,
            ],
        ));
    }
    for _ in 0..4 {
        spawn.push((
            T::Splash,
            [
                feet[0] + f64::from(rnd(31) - 0.5) * 1.2,
                feet[1] + 0.9,
                feet[2] + f64::from(rnd(32) - 0.5) * 1.2,
            ],
            [0.0, 0.06, 0.0],
        ));
    }
}
