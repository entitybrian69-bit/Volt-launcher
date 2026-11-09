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
use horizon_client::mobs::{nea, overhauled, player_gestures as gestures};
use horizon_client::subtle_effects as subtle;
use std::sync::Mutex;
use std::time::Duration;

/// Accumulates frame time so mob-attached particles run at tick rate.
static MOB_FX_CLOCK: Mutex<Duration> = Mutex::new(Duration::ZERO);
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
                3 => { f.intensity = (f.intensity + 1) % 3; }
                _ => return,
            }
            overhauled::set_flags(f);
            if button < 3 {
                let keys = ["enabled", "players", "mobs"];
                let v = [f.enabled, f.players, f.mobs][button];
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
}
