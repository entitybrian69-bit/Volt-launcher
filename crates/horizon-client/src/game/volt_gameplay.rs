//! Live in-world & HUD runtime hooks for **Volt Settings 0.9.9**:
//! - **`Volt QoL` (`#2`, `#3`, `#8`, `#9`, `#15`, `#17`, `#18`, `#25`)**
//! - **`Volt Chat` (`#21` Chat Suite)**
//! - **`Volt Visuals` (`#19` Connected Glass, Better Grass, 3D Dropped Items)**
//! - **`Volt 50-Mod Catalog` (`volt_mods` pure-Rust HUDs & gameplay mechanics)**

use super::{Dim, Game, Screen};
use horizon_client::{
    extras::Corner,
    extras_hud::Corners,
    gizmos::Line,
    keybinds,
    hud::{HudBatch, rgba},
    text::{self, Text},
    volt_features::project_waypoint,
    volt_mods::VoltModId,
    volt_qol::{
        AtlasChunkTile, ClientTimeWeather, CrosshairStyle, eval_chat_math, is_slime_chunk,
        preset_blueprint_blocks,
    },
};
use std::collections::{HashMap, VecDeque};
use winit::keyboard::KeyCode;

/// Transient per-session runtime state for Volt QoL, JEI & Mods.
pub(super) struct VoltRuntime {
    pub freelook_active: bool,
    freelook_key_was_down: bool,
    zoom_toggle_active: bool,
    zoom_was_active: bool,
    zoom_scroll_steps: i32,
    zoom_scroll_accum: f32,
    zoom_display_scale: f32,
    zoom_display_smooth: bool,
    pub freelook_yaw: f32,
    pub freelook_pitch: f32,
    pub blueprint_anchor: Option<[i32; 3]>,
    pub blueprint_dim: String,
    pub atlas_scope_dim: Option<(String, String)>,
    pub atlas_scan_cursor: i32,
    pub replay_ring: VecDeque<([f64; 3], f32, f32)>,
    pub last_mob_hp: HashMap<u32, f32>,
    pub damage_popups: Vec<DamagePopup>,
    pub last_hit_dist: Option<(f32, f64)>,
    pub toggle_sprint_latched: bool,
    pub last_input_at: f64,
    pub last_chat_line: Option<(String, u32)>,
    /// The on-screen replay button's current touch gesture.
    pub replay_btn_touch: Option<ReplayBtnTouch>,
    /// The button's Start / Pause / End rows are open.
    pub replay_btn_menu: bool,
    /// A short caption under the button (saved, paused, adjust hints).
    pub replay_btn_note: Option<(String, std::time::Instant)>,
}

/// One finger on the on-screen replay button: a quick tap opens the
/// Start / Pause / End rows, an early move drags the button, and holding
/// still first turns the drag into the size (up-down) and opacity
/// (left-right) adjustment.
pub(super) struct ReplayBtnTouch {
    pub id: u64,
    pub start: (f32, f32),
    pub at: std::time::Instant,
    /// 0 = undecided, 1 = dragging the position, 2 = hold: size/opacity.
    pub mode: u8,
    /// (x pct, y pct, size, opacity pct) when the finger went down.
    pub base: (u16, u16, u16, u16),
}

pub(super) struct DamagePopup {
    pub pos: [f64; 3],
    pub amount: f32,
    pub born_at: f64,
}

impl Default for VoltRuntime {
    fn default() -> Self {
        Self {
            freelook_active: false,
            freelook_key_was_down: false,
            zoom_toggle_active: false,
            zoom_was_active: false,
            zoom_scroll_steps: 0,
            zoom_scroll_accum: 0.0,
            zoom_display_scale: 1.0,
            zoom_display_smooth: true,
            freelook_yaw: 0.0,
            freelook_pitch: 0.0,
            blueprint_anchor: None,
            blueprint_dim: String::new(),
            atlas_scope_dim: None,
            atlas_scan_cursor: 0,
            replay_ring: VecDeque::new(),
            last_mob_hp: HashMap::new(),
            damage_popups: Vec::new(),
            last_hit_dist: None,
            toggle_sprint_latched: false,
            last_input_at: 0.0,
            last_chat_line: None,
            replay_btn_touch: None,
            replay_btn_menu: false,
            replay_btn_note: None,
        }
    }
}

fn box_wireframe(lines: &mut Vec<Line>, min: [f64; 3], max: [f64; 3], color: u32, width: f32) {
    let c = [
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], min[1], max[2]],
        [min[0], min[1], max[2]],
        [min[0], max[1], min[2]],
        [max[0], max[1], min[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
    ];
    for &(a, b) in &[
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ] {
        lines.push(Line {
            start: c[a],
            end: c[b],
            color,
            width,
        });
    }
}

fn freelook_next_active(available: bool, toggle: bool, down: bool, was_down: bool, active: bool) -> bool {
    if !available {
        false
    } else if toggle {
        if down && !was_down { !active } else { active }
    } else {
        down
    }
}

impl Game<'_> {
    /// Returns true while `#2` 360° Freelook is actively orbiting the player.
    pub(super) fn freelook_active(&self) -> bool {
        self.extras.volt.freelook_active
            && (self.settings.extras.volt_qol.freelook_enabled
                || self
                    .settings
                    .extras
                    .mod_catalog
                    .is_on(VoltModId::Freelook))
            && self.screen == Screen::Playing
            && self.title.is_none()
    }

    /// Read mod holds from physical inputs, not vanilla's translated key set.
    fn volt_binding_held(&self, mapping: &str) -> bool {
        let binding = self.settings.keys.get(mapping);
        keybinds::key_code(binding).is_some_and(|key| self.physical_keys.contains(&key))
            || keybinds::mouse_button(binding).is_some_and(|button| self.physical_buttons.contains(&button))
    }

    /// Returns the native optic zoom `(factor, smooth, turn_scale)` when active.
    pub(super) fn volt_qol_zoom(&self) -> Option<(f32, bool, f64)> {
        let qol = &self.settings.extras.volt_qol;
        let cat = &self.settings.extras.mod_catalog;
        if !(qol.zoom_enabled || self.settings.extras.zoom || cat.is_on(VoltModId::Zoomify))
            || self.screen != Screen::Playing || self.title.is_some()
            || self.screen_over_playing() || !self.focused
        {
            return None;
        }
        let active = if cat.zoomify_toggle {
            self.extras.volt.zoom_toggle_active
        } else {
            self.volt_binding_held("key.volt.zoom")
        };
        if !active { return None; }
        let factor = horizon_client::volt_mods::zoomify_factor(qol.zoom_fov_pct, self.extras.volt.zoom_scroll_steps);
        let turn = if cat.zoomify_relative_sensitivity { 1.0 / factor } else { 1.0 };
        Some((factor, qol.zoom_smooth, f64::from(turn)))
    }

    /// FOV scale for the active optic zoom or its smooth exit transition.
    pub(super) fn volt_zoom_fov_scale(&self) -> Option<f32> {
        let state = &self.extras.volt;
        if let Some((factor, smooth, _)) = self.volt_qol_zoom() {
            Some(if smooth { state.zoom_display_scale } else { 1.0 / factor })
        } else if state.zoom_display_scale < 1.0 {
            Some(state.zoom_display_scale)
        } else {
            None
        }
    }

    /// While zoomed, take vertical wheel input before it changes the hotbar.
    /// Fractional touchpad events are accumulated until one full step arrives.
    pub(super) fn volt_zoom_wheel(&mut self, amount: f32) -> bool {
        if !self.settings.extras.mod_catalog.zoomify_scroll || self.volt_qol_zoom().is_none()
            || !amount.is_finite() || amount == 0.0
        {
            return false;
        }
        let runtime = &mut self.extras.volt;
        runtime.zoom_scroll_accum = (runtime.zoom_scroll_accum + amount).clamp(-20.0, 20.0);
        let whole = runtime.zoom_scroll_accum.trunc() as i32;
        runtime.zoom_scroll_accum -= whole as f32;
        let (min, max) = horizon_client::volt_mods::zoomify_step_bounds(self.settings.extras.volt_qol.zoom_fov_pct);
        runtime.zoom_scroll_steps = (runtime.zoom_scroll_steps + whole).clamp(min, max);
        true
    }

    /// Intercepts mouse/touch look deltas when `#2` 360° Freelook is held.
    pub(super) fn try_freelook_turn(&mut self, xo: f64, yo: f64) -> bool {
        if !self.freelook_active() {
            return false;
        }
        self.extras.volt.freelook_yaw =
            (self.extras.volt.freelook_yaw + xo as f32 * 0.15).rem_euclid(360.0);
        self.extras.volt.freelook_pitch =
            (self.extras.volt.freelook_pitch + yo as f32 * 0.15).clamp(-89.5, 89.5);
        true
    }

    /// Open native mod screens from a physical key or mouse binding.
    pub(super) fn volt_mod_input(&mut self, name: &str) -> bool {
        if name == "key.keyboard.unknown" || self.screen != Screen::Playing || self.title.is_some() {
            return false;
        }
        let cat = &self.settings.extras.mod_catalog;
        if self.focused && !self.screen_over_playing()
            && cat.zoomify_toggle
            && (self.settings.extras.volt_qol.zoom_enabled || self.settings.extras.zoom || cat.is_on(VoltModId::Zoomify))
            && self.settings.keys.get("key.volt.zoom") == name
        {
            self.extras.volt.zoom_toggle_active = !self.extras.volt.zoom_toggle_active;
            return true;
        }
        if self.settings.keys.get("key.volt.world_map") == name
            && (self.settings.extras.volt_qol.atlas_enabled
                || self.settings.extras.mod_catalog.is_on(VoltModId::XaerosWorldMap))
        {
            self.sync_waypoint_ui_context();
            self.open_options_at(false, horizon_client::options_ui::Page::VoltAtlas);
            return true;
        }
        false
    }

    pub(super) fn volt_gameplay_key(&mut self, key: KeyCode, pressed: bool) -> bool {
        if pressed {
            self.extras.volt.last_input_at = self.started.elapsed().as_secs_f64();
        }
        if !pressed || self.screen != Screen::Playing || self.title.is_some() {
            return false;
        }
        if key == KeyCode::ControlLeft && self.settings.extras.volt_mods.toggle_sprint {
            self.extras.volt.toggle_sprint_latched = !self.extras.volt.toggle_sprint_latched;
        }
        false
    }

    /// Client-only day_time & weather override (`Volt Mods` -> `Client Time & Weather`).
    pub(super) fn volt_effective_time_weather(
        &self,
        day_time: i64,
        rain: f32,
        thunder: f32,
    ) -> (i64, f32, f32) {
        match self.settings.extras.volt_mods.time_weather {
            ClientTimeWeather::ServerLive => (day_time, rain, thunder),
            ClientTimeWeather::SunnyNoon => (6000, 0.0, 0.0),
            ClientTimeWeather::GoldenSunset => (12650, 0.0, 0.0),
            ClientTimeWeather::StarryMidnight => (18000, 0.0, 0.0),
            ClientTimeWeather::CozyRain => (day_time, 1.0, 0.0),
        }
    }

    /// Protects valuable tools with <= 3 durability left (`Volt Mods` -> `Tool-Break Protection`).
    pub(super) fn blocks_tool_break(&mut self) -> bool {
        if !self.settings.extras.volt_mods.tool_break_shield {
            return false;
        }
        let Some(stack) = self.held() else {
            return false;
        };
        let id = stack.item.as_ref();
        let is_tool = id.ends_with("_pickaxe")
            || id.ends_with("_axe")
            || id.ends_with("_shovel")
            || id.ends_with("_hoe")
            || id.ends_with("_sword")
            || id == "minecraft:trident"
            || id == "minecraft:bow"
            || id == "minecraft:crossbow"
            || id == "minecraft:elytra";
        if !is_tool || stack.damage == 0 {
            return false;
        }
        let max_dur: u32 = if id.contains("netherite") {
            2031
        } else if id.contains("diamond") {
            1561
        } else if id.contains("iron") {
            250
        } else if id.contains("stone") || id.contains("copper") {
            131
        } else if id.contains("golden") {
            32
        } else if id.contains("wooden") {
            59
        } else {
            384
        };
        let remaining = max_dur.saturating_sub(stack.damage);
        if (1..=3).contains(&remaining) {
            self.titles.set_overlay(
                Text::literal(format!(
                    "[Tool Shield] Saved tool at {remaining} durability!"
                ))
                .with_color(text::RED),
                false,
            );
            return true;
        }
        false
    }

    /// Per-frame background tick for World Atlas, Sound Physics Lite,
    /// Damage Popups, Replay Buffer, and Cosmetics skin sync.
    pub(super) fn tick_volt_frame(&mut self, underwater: bool) {
        let now = self.started.elapsed().as_secs_f64();
        if self.extras.mouse[0] || self.extras.mouse[1] {
            self.extras.volt.last_input_at = now;
        }
        self.ui.options.skin_slim = self.player_model.as_ref().is_some_and(|m| m.slim);
        let cur_skin = self.player_model.as_ref().and_then(|m| m.skin.as_deref());
        if self.ui.options.skin_key.as_deref() != cur_skin {
            self.ui.options.skin_key = cur_skin.map(str::to_owned);
        }

        let zoom_available = (self.settings.extras.volt_qol.zoom_enabled
            || self.settings.extras.zoom
            || self.settings.extras.mod_catalog.is_on(VoltModId::Zoomify))
            && self.screen == Screen::Playing && self.title.is_none()
            && !self.screen_over_playing() && self.focused;
        if !zoom_available || !self.settings.extras.mod_catalog.zoomify_toggle {
            self.extras.volt.zoom_toggle_active = false;
        }
        let zoom = self.volt_qol_zoom();
        let zoom_active = zoom.is_some();
        if let Some((_, smooth, _)) = zoom {
            self.extras.volt.zoom_display_smooth = smooth;
        }
        let target_scale = zoom.map_or(1.0, |(factor, _, _)| 1.0 / factor);
        self.extras.volt.zoom_display_scale = horizon_client::volt_mods::zoomify_ease_scale(
            self.extras.volt.zoom_display_scale,
            target_scale,
            self.frame_ticks / 20.0,
            self.extras.volt.zoom_display_smooth,
        );
        if !zoom_active && self.extras.volt.zoom_was_active {
            self.extras.volt.zoom_scroll_accum = 0.0;
            if !self.settings.extras.mod_catalog.zoomify_retain_steps {
                self.extras.volt.zoom_scroll_steps = 0;
            }
        }
        self.extras.volt.zoom_was_active = zoom_active;

        let freelook_available = (self.settings.extras.volt_qol.freelook_enabled
            || self.settings.extras.mod_catalog.is_on(VoltModId::Freelook))
            && self.screen == Screen::Playing
            && self.title.is_none();
        let key_down = freelook_available && self.volt_binding_held("key.volt.freelook");
        let was_active = self.extras.volt.freelook_active;
        let active = freelook_next_active(
            freelook_available,
            self.settings.extras.mod_catalog.freelook_toggle,
            key_down,
            self.extras.volt.freelook_key_was_down,
            was_active,
        );
        self.extras.volt.freelook_key_was_down = key_down;
        self.extras.volt.freelook_active = active;
        if active && !was_active {
            self.extras.volt.freelook_yaw = self.yaw;
            self.extras.volt.freelook_pitch = self.pitch;
        }

        let qol = &self.settings.extras.volt_qol;
        let mods = &self.settings.extras.volt_mods;

        // 1. #15 Sound Physics Lite (Cave Reverb + Underwater LPF + Wall Muffle + Muffler)
        let mut cave_reverb = 0.0f32;
        let mut wall_occ = 0.0f32;
        if let Some(p) = self.player.as_ref() {
            let bx = p.position.x.floor() as i32;
            let by = (p.position.y + 1.6).floor() as i32;
            let bz = p.position.z.floor() as i32;
            let sky_l = self.world.light(bx, by, bz) >> 4;
            if qol.sound_cave_reverb && sky_l <= 2 && self.dim != Dim::End {
                cave_reverb = (0.30 * (1.0 - f32::from(sky_l) / 3.0)).clamp(0.0, 0.30);
            }
            if qol.sound_wall_occlusion && sky_l == 0 {
                wall_occ = 0.25;
            }
        }
        self.audio.set_volt_sound_physics(
            qol.sound_underwater_lpf && underwater,
            cave_reverb,
            wall_occ,
            [
                qol.muffle_portal_pct,
                qol.muffle_piston_pct,
                qol.muffle_minecart_pct,
                qol.muffle_explosion_pct,
                qol.muffle_rain_pct,
            ],
        );

        if self.title.is_some() {
            self.extras.volt.blueprint_anchor = None;
            self.extras.volt.last_mob_hp.clear();
            self.extras.volt.damage_popups.clear();
            return;
        }

        // 3. #17 Interactive World Atlas / Xaero's World Map incremental chunk sampling (1 chunk/frame -> 0 stutter!)
        if (qol.atlas_enabled
            || self
                .settings
                .extras
                .mod_catalog
                .is_on(VoltModId::XaerosWorldMap))
            && let Some((scope, dim, pos)) = self.current_waypoint_context()
        {
            self.ui.options.world_atlas.player_pos = [
                pos[0] as f32 + 0.5,
                pos[1] as f32,
                pos[2] as f32 + 0.5,
            ];
            self.ui.options.world_atlas.player_yaw = self.yaw;
            if self.extras.volt.atlas_scope_dim.as_ref() != Some(&(scope.clone(), dim.clone())) {
                self.ui.options.world_atlas.tiles.clear();
                self.ui.options.world_atlas.center_on_player();
                self.extras.volt.atlas_scope_dim = Some((scope, dim));
            }
            let pcx = pos[0] >> 4;
            let pcz = pos[2] >> 4;
            let idx = self.extras.volt.atlas_scan_cursor.rem_euclid(81);
            self.extras.volt.atlas_scan_cursor = self.extras.volt.atlas_scan_cursor.wrapping_add(1);
            let cx = pcx + (idx % 9) - 4;
            let cz = pcz + (idx / 9) - 4;
            if !self.ui.options.world_atlas.tiles.contains_key(&(cx, cz))
                && let Some(chunk) = self.world.chunks.get(&(cx, cz))
            {
                let mut rgb_cells = [[58u8, 110, 58]; 16];
                let mut heights = [64i16; 16];
                for sx in 0i32..4 {
                    for sz in 0i32..4 {
                        let wx = cx * 16 + sx * 4 + 2;
                        let wz = cz * 16 + sz * 4 + 2;
                        let mut top_y = (pos[1] + 48).min(319);
                        let min_y = (pos[1] - 64).max(-64);
                        let mut cell_rgb = [58u8, 110, 58];
                        while top_y >= min_y {
                            let st = chunk.block(wx & 15, top_y, wz & 15);
                            let facts = self.res.facts().get(st);
                            let nm = facts.name.as_ref();
                            if nm != "minecraft:air"
                                && nm != "minecraft:cave_air"
                                && nm != "minecraft:void_air"
                            {
                                cell_rgb = classify_atlas_block_rgb(nm);
                                break;
                            }
                            top_y -= 2;
                        }
                        let cell = (sz * 4 + sx) as usize;
                        rgb_cells[cell] = cell_rgb;
                        heights[cell] = top_y as i16;
                    }
                }
                self.ui.options.world_atlas.record_tile(
                    cx,
                    cz,
                    AtlasChunkTile {
                        rgb: rgb_cells,
                        height: heights,
                    },
                );
            }
        }

        // 4. #8 Blueprint anchor management
        if qol.blueprint_enabled {
            if self.extras.volt.blueprint_anchor.is_none()
                || self.extras.volt.blueprint_dim != self.dim.id()
            {
                if let Some(p) = self.player.as_ref() {
                    let yaw_rad = self.yaw.to_radians();
                    let ax = (p.position.x - f64::from(yaw_rad.sin()) * 6.0).floor() as i32;
                    let ay = p.position.y.floor() as i32;
                    let az = (p.position.z + f64::from(yaw_rad.cos()) * 6.0).floor() as i32;
                    self.extras.volt.blueprint_anchor = Some([ax, ay, az]);
                    self.extras.volt.blueprint_dim = self.dim.id().to_owned();
                }
            }
        } else {
            self.extras.volt.blueprint_anchor = None;
        }

        // 5. #9 Keyframe Replay Ring (movement-sampled)
        if qol.replay_buffer
            && let Some(p) = self.player.as_ref()
        {
            let pos = [p.position.x, p.position.y, p.position.z];
            let push_it = self
                .extras
                .volt
                .replay_ring
                .back()
                .is_none_or(|(prev, _, _)| {
                    let dx = pos[0] - prev[0];
                    let dz = pos[2] - prev[2];
                    dx * dx + dz * dz > 0.04
                });
            if push_it {
                if self.extras.volt.replay_ring.len() >= 600 {
                    self.extras.volt.replay_ring.pop_front();
                }
                self.extras
                    .volt
                    .replay_ring
                    .push_back((pos, self.yaw, self.pitch));
            }
        }

        // 6. Volt Mods #16 Floating Damage Popups + #4 Attack Reach tracking
        if mods.damage_popups || mods.reach_hud {
            for mob in &self.mobs {
                let prev = self
                    .extras
                    .volt
                    .last_mob_hp
                    .insert(mob.id, mob.health)
                    .unwrap_or(mob.health);
                let lost = prev - mob.health;
                if lost >= 0.25 && mob.hurt_time >= 8 {
                    if mods.damage_popups && self.extras.volt.damage_popups.len() < 24 {
                        self.extras.volt.damage_popups.push(DamagePopup {
                            pos: [mob.position.x, mob.position.y + 1.5, mob.position.z],
                            amount: lost,
                            born_at: now,
                        });
                    }
                    if mods.reach_hud
                        && let Some(eye) = self.eye_position()
                    {
                        let dx = mob.position.x - eye[0];
                        let dy = (mob.position.y + 0.9) - eye[1];
                        let dz = mob.position.z - eye[2];
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt() as f32;
                        if dist <= 6.5 {
                            self.extras.volt.last_hit_dist = Some((dist, now));
                        }
                    }
                }
            }
            self.extras
                .volt
                .damage_popups
                .retain(|d| now - d.born_at < 1.15);
            if self.extras.volt.last_mob_hp.len() > 256 {
                self.extras.volt.last_mob_hp.clear();
            }
        }
    }

    /// Emits hardware depth-tested 3D gizmo lines for
    /// `#8` Blueprint 3D Hologram, `TrajectoryArc`, and `HitboxPlus`.
    pub(super) fn append_volt_world_gizmos(&self, lines: &mut Vec<Line>) {
        if self.title.is_some() || self.ui.hide_gui {
            return;
        }
        // Replay Studio: the keyframe camera path hangs in the world as a
        // red spline with mint keyframe cubes, visible from the free
        // camera (hidden in POV and while the camera flies the path).
        if let Some(rp) = self.remote.as_ref().and_then(|r| r.replay.as_ref()) {
            use std::sync::atomic::Ordering::Relaxed;
            if !rp.control.pov.load(Relaxed) && !rp.cam_path && rp.cam_keys.len() >= 2 {
                let (first, last) = (rp.cam_keys[0].ms, rp.cam_keys[rp.cam_keys.len() - 1].ms);
                let steps = (rp.cam_keys.len() * 16).clamp(24, 160) as u32;
                let mut prev: Option<[f64; 3]> = None;
                for i in 0..=steps {
                    let t = first + ((u64::from(last - first) * u64::from(i)
                        / u64::from(steps)) as u32);
                    if let Some((pos, _, _)) =
                        horizon_client::replay::sample_cam_path(&rp.cam_keys, t)
                    {
                        if let Some(a) = prev {
                            lines.push(Line {
                                start: a,
                                end: pos,
                                color: 0xFFE8_445A,
                                width: 2.4,
                            });
                        }
                        prev = Some(pos);
                    }
                }
                for k in &rp.cam_keys {
                    box_wireframe(
                        lines,
                        [k.pos[0] - 0.18, k.pos[1] - 0.18, k.pos[2] - 0.18],
                        [k.pos[0] + 0.18, k.pos[1] + 0.18, k.pos[2] + 0.18],
                        0xFF79_E59C,
                        2.0,
                    );
                }
            }
            // The rest of the gizmos are gameplay aids; a replay shows none.
            return;
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let qol = &self.settings.extras.volt_qol;
        let mods = &self.settings.extras.volt_mods;

        // B. #8 Volt Blueprint 3D Hologram
        if qol.blueprint_enabled
            && let Some([ax, ay, az]) = self.extras.volt.blueprint_anchor
        {
            for blk in preset_blueprint_blocks(qol.blueprint_preset) {
                if qol.blueprint_layer >= 0 && blk.rel[1] != i32::from(qol.blueprint_layer) {
                    continue;
                }
                let wx = ax + blk.rel[0];
                let wy = ay + blk.rel[1];
                let wz = az + blk.rel[2];
                let cur_name = self.res.facts().get(self.world.block(wx, wy, wz)).name.as_ref();
                let cur_short = cur_name.trim_start_matches("minecraft:");
                let color = if cur_short == blk.block_key {
                    0x9032_E678 // Placed & matching (Emerald)
                } else if cur_short == "air" || cur_short == "cave_air" || cur_short == "void_air" {
                    0xB848_B4FF // Ghost block to place (Sky Cyan)
                } else {
                    0xE0FF_3848 // Wrong block warning (Crimson)
                };
                box_wireframe(
                    lines,
                    [
                        f64::from(wx) + 0.03,
                        f64::from(wy) + 0.03,
                        f64::from(wz) + 0.03,
                    ],
                    [
                        f64::from(wx) + 0.97,
                        f64::from(wy) + 0.97,
                        f64::from(wz) + 0.97,
                    ],
                    color,
                    1.8,
                );
            }
        }

        // C. Volt Mods #15 Projectile Trajectory Preview Arc
        if mods.trajectory_arc
            && let Some(held) = self.held()
        {
            let item = held.item.as_ref();
            let params: Option<(f64, f64, f64)> = match item {
                "minecraft:bow" => Some((3.0, 0.05, 0.99)),
                "minecraft:crossbow" => Some((3.15, 0.05, 0.99)),
                "minecraft:trident" => Some((2.5, 0.05, 0.99)),
                "minecraft:ender_pearl"
                | "minecraft:snowball"
                | "minecraft:egg"
                | "minecraft:wind_charge" => Some((1.5, 0.03, 0.99)),
                "minecraft:splash_potion" | "minecraft:lingering_potion" => Some((0.5, 0.05, 0.99)),
                _ => None,
            };
            if let Some((speed, gravity, drag)) = params
                && let Some(eye) = self.eye_position()
            {
                let yr = f64::from(self.yaw.to_radians());
                let pr = f64::from(self.pitch.to_radians());
                let mut vx = -yr.sin() * pr.cos() * speed;
                let mut vy = -pr.sin() * speed;
                let mut vz = yr.cos() * pr.cos() * speed;
                let mut cur = [eye[0], eye[1] - 0.1, eye[2]];
                for step in 0i32..42 {
                    let next = [cur[0] + vx, cur[1] + vy, cur[2] + vz];
                    let col = if step % 2 == 0 {
                        0xE046_F0B4
                    } else {
                        0xB028_B8FF
                    };
                    lines.push(Line {
                        start: cur,
                        end: next,
                        color: col,
                        width: 2.2,
                    });
                    let st = self.world.block(
                        next[0].floor() as i32,
                        next[1].floor() as i32,
                        next[2].floor() as i32,
                    );
                    if self.res.facts().get(st).solid_render {
                        box_wireframe(
                            lines,
                            [next[0] - 0.22, next[1] - 0.05, next[2] - 0.22],
                            [next[0] + 0.22, next[1] + 0.05, next[2] + 0.22],
                            0xFFFF_4458,
                            2.6,
                        );
                        break;
                    }
                    cur = next;
                    vx *= drag;
                    vy = vy * drag - gravity;
                    vz *= drag;
                }
            }
        }

        // D. Volt Mods #17 Tactical Hitbox Vectors
        if mods.hitbox_plus {
            for mob in self.mobs.iter().take(32) {
                if mob.death_time > 0 || mob.is_invisible() {
                    continue;
                }
                let dx = mob.position.x - p.position.x;
                let dz = mob.position.z - p.position.z;
                if dx * dx + dz * dz > 24.0 * 24.0 {
                    continue;
                }
                let hw = mob.kind.width * 0.5;
                let hh = mob.kind.height;
                let col = if mob.hurt_time > 0 {
                    0xFFFF_384C
                } else {
                    0xC858_DCFF
                };
                box_wireframe(
                    lines,
                    [mob.position.x - hw, mob.position.y, mob.position.z - hw],
                    [
                        mob.position.x + hw,
                        mob.position.y + hh,
                        mob.position.z + hw,
                    ],
                    col,
                    1.6,
                );
                let yr = f64::from(mob.head_yaw.to_radians());
                let pr = f64::from(mob.head_pitch.to_radians());
                let eye = [mob.position.x, mob.position.y + mob.kind.eye_height, mob.position.z];
                lines.push(Line {
                    start: eye,
                    end: [
                        eye[0] - yr.sin() * pr.cos() * 1.4,
                        eye[1] - pr.sin() * 1.4,
                        eye[2] + yr.cos() * pr.cos() * 1.4,
                    ],
                    color: 0xFFE8_445A,
                    width: 2.0,
                });
            }
        }
    }

    /// Draws all active Volt QoL and Mod Catalog HUDs!
    pub(super) fn draw_volt_all_huds(&mut self, batch: &mut HudBatch, corners: &mut Corners) {
        if self.title.is_some() || self.ui.hide_gui {
            return;
        }
        // Replay Studio: while a replay is watched, the editor panel is
        // the whole interface; gameplay HUDs stay out of the footage.
        if self.replay_control().is_some() {
            self.draw_replay_hud(batch);
            return;
        }
        // The floating replay button (Start / Pause / End) rides over the
        // live HUD; costs nothing while its setting is off.
        if self.settings.extras.volt_qol.replay_button {
            self.draw_replay_button(batch);
        }
        // The Volt Hub V logo draws late in `Game::hud` instead, so it
        // rides over every in-game screen, not only the live HUD.
        let now = self.started.elapsed().as_secs_f64();
        let (w, h) = (batch.width, batch.height);
        let my_name = self.options.player_name();
        let local_ping_ms: i32 = self
            .remote_tab_players()
            .and_then(|(players, _, _)| {
                players
                    .into_iter()
                    .find(|p| p.name == my_name)
                    .map(|p| p.latency.max(0))
            })
            .unwrap_or(0);
        let atlas = &self.gui.atlas;

        let qol = &self.settings.extras.volt_qol;
        let mods = &self.settings.extras.volt_mods;
        let cat = &self.settings.extras.mod_catalog;

        // 1. #3 World Inspector (Jade / WTHIT Top-Center Card)
        if qol.world_inspector && self.screen == Screen::Playing {
            let mut title_str: Option<String> = None;
            let mut sub_str = String::new();
            if let Some((idx, dist)) = self.mob_hit()
                && dist <= 6.0
                && let Some(mob) = self.mobs.get(idx)
            {
                let clean = mob
                    .kind
                    .id
                    .trim_start_matches("minecraft:")
                    .replace('_', " ");
                title_str = Some(capitalize_words(&clean));
                sub_str = format!("HP: {:.1} \u{2764}  \u{2022}  {:.1}m", mob.health * 0.5, dist);
            } else if let Some(hit) = self.pick() {
                let st = self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
                let facts = self.res.facts().get(st);
                let raw = facts.name.trim_start_matches("minecraft:");
                if raw != "air" && raw != "cave_air" && raw != "void_air" {
                    title_str = Some(capitalize_words(&raw.replace('_', " ")));
                    let light = self.world.light(hit.pos[0], hit.pos[1] + 1, hit.pos[2]);
                    let tool = recommended_tool_tag(raw);
                    sub_str = format!(
                        "{tool}  \u{2022}  Light {} (Sky {})",
                        light & 15,
                        light >> 4
                    );
                }
            }
            if let Some(title) = title_str {
                let cw = (atlas
                    .font
                    .width(&title)
                    .max(atlas.font.width(&sub_str))
                    + 18.0)
                    .max(124.0);
                let cx = ((w - cw) * 0.5).floor();
                let cy = 6.0;
                batch.fill(atlas, cx, cy, cw, 24.0, rgba(12, 14, 22, 215));
                batch.fill(atlas, cx, cy, cw, 1.5, rgba(224, 48, 68, 245));
                batch.text(
                    atlas,
                    &title,
                    cx + 7.0,
                    cy + 4.0,
                    rgba(250, 250, 255, 255),
                    true,
                );
                batch.text(
                    atlas,
                    &sub_str,
                    cx + 7.0,
                    cy + 14.0,
                    rgba(175, 215, 245, 255),
                    true,
                );
            }
        }

        // 2. #8 Blueprint Material Checklist HUD
        if qol.blueprint_enabled
            && qol.blueprint_checklist
            && let Some([ax, ay, az]) = self.extras.volt.blueprint_anchor
        {
            let mut missing: HashMap<&'static str, u32> = HashMap::new();
            let mut done = 0u32;
            let blocks = preset_blueprint_blocks(qol.blueprint_preset);
            for blk in &blocks {
                let wx = ax + blk.rel[0];
                let wy = ay + blk.rel[1];
                let wz = az + blk.rel[2];
                let cur = self.res.facts().get(self.world.block(wx, wy, wz)).name.as_ref();
                if cur.trim_start_matches("minecraft:") == blk.block_key {
                    done += 1;
                } else {
                    *missing.entry(blk.block_key).or_insert(0) += 1;
                }
            }
            let rows_n = missing.len().min(4) + 1;
            let bh = 14.0 + rows_n as f32 * 10.0;
            let (bx, by) = corners.place(Corner::TopLeft, 138.0, bh);
            batch.fill(atlas, bx, by, 138.0, bh, rgba(12, 16, 24, 200));
            batch.fill(atlas, bx, by, 2.0, bh, rgba(72, 180, 255, 245));
            batch.text(
                atlas,
                &format!(
                    "Blueprint: {done}/{} ({}%)",
                    blocks.len(),
                    (done * 100) / (blocks.len().max(1) as u32)
                ),
                bx + 6.0,
                by + 3.0,
                rgba(110, 230, 255, 255),
                true,
            );
            let mut list: Vec<(&str, u32)> = missing.into_iter().collect();
            list.sort_by(|a, b| b.1.cmp(&a.1));
            for (i, (id, cnt)) in list.into_iter().take(4).enumerate() {
                let clean = id.replace('_', " ");
                batch.text(
                    atlas,
                    &format!("{}x {}", cnt, capitalize_words(&clean)),
                    bx + 6.0,
                    by + 14.0 + i as f32 * 10.0,
                    rgba(235, 238, 248, 255),
                    true,
                );
            }
        }

        // 4. Volt Mods HUDs (CPS, Ping, Coordinates, Reach, FPS Pill, Armor, Durability, Potions, Inventory HUD+)
        let mut pill_parts: Vec<String> = Vec::new();
        if mods.fps_ping_pill {
            let r = self.extras.fps.reading();
            pill_parts.push(format!("{} FPS ({:.1}ms)", r.fps, self.extras.ema_frame_ms));
        }
        if mods.cps_hud || cat.is_on(VoltModId::CpsDisplay) {
            let lcps = self.extras.cps[0].per_second(now);
            let rcps = self.extras.cps[1].per_second(now);
            pill_parts.push(format!("CPS {lcps} | {rcps}"));
        }
        if cat.is_on(VoltModId::PingDisplay) {
            pill_parts.push(format!("{local_ping_ms} ms"));
        }
        if mods.reach_hud {
            let txt = match self.extras.volt.last_hit_dist {
                Some((d, at)) if now - at < 4.0 => format!("Reach {d:.2}m"),
                _ => "Reach --".into(),
            };
            pill_parts.push(txt);
        }
        if mods.toggle_sprint && self.extras.volt.toggle_sprint_latched {
            pill_parts.push("[Sprint Toggled]".into());
        }
        if !pill_parts.is_empty() {
            let joined = pill_parts.join("  \u{2022}  ");
            let pw = atlas.font.width(&joined) + 12.0;
            let (px, py) = corners.place(Corner::TopLeft, pw, 14.0);
            batch.fill(atlas, px, py, pw, 14.0, rgba(12, 14, 22, 195));
            batch.fill(atlas, px, py, 2.0, 14.0, rgba(224, 48, 68, 245));
            batch.text(
                atlas,
                &joined,
                px + 5.0,
                py + 3.0,
                rgba(246, 248, 255, 255),
                true,
            );
        }

        // Coordinates HUD (`VoltModId::CoordinatesHud`)
        if cat.is_on(VoltModId::CoordinatesHud)
            && let Some(p) = self.player.as_ref()
        {
            let bx = p.position.x.floor() as i32;
            let by = p.position.y.floor() as i32;
            let bz = p.position.z.floor() as i32;
            let dir = match (((self.yaw.rem_euclid(360.0) + 45.0) / 90.0).floor() as i32) & 3 {
                0 => "S (+Z)",
                1 => "W (-X)",
                2 => "N (-Z)",
                _ => "E (+X)",
            };
            let mut coord_txt = format!("XYZ: {bx}, {by}, {bz}  [{dir}]");
            if cat.coords_show_biome
                && let Some(chunk) = self.world.chunks.get(&(bx >> 4, bz >> 4))
            {
                let bname = self
                    .res
                    .biomes()
                    .get(chunk.biome(bx, by, bz))
                    .name
                    .trim_start_matches("minecraft:")
                    .replace('_', " ");
                coord_txt.push_str(&format!("  \u{2022}  {}", capitalize_words(&bname)));
            }
            let cw = atlas.font.width(&coord_txt) + 12.0;
            let (cx, cy) = corners.place(Corner::TopLeft, cw, 14.0);
            batch.fill(atlas, cx, cy, cw, 14.0, rgba(12, 16, 24, 198));
            batch.fill(atlas, cx, cy, 2.0, 14.0, rgba(88, 212, 245, 245));
            batch.text(
                atlas,
                &coord_txt,
                cx + 5.0,
                cy + 3.0,
                rgba(235, 246, 255, 255),
                true,
            );
        }

        // Armor & Durability HUD
        if mods.armor_hud || cat.is_on(VoltModId::ArmorHud) {
            let names = ["Helm", "Chest", "Legs", "Boots"];
            let mut eq_lines = Vec::new();
            for (slot, label) in self.armor.iter().zip(names) {
                if let Some(st) = slot {
                    let max_d = self
                        .res
                        .items()
                        .get(&st.item)
                        .map(|i| i.max_damage)
                        .filter(|&m| m > 0)
                        .unwrap_or_else(|| estimate_armor_max_dur(&st.item));
                    let rem = max_d.saturating_sub(st.damage);
                    eq_lines.push((
                        format!("{label}: {rem}/{max_d}"),
                        if rem * 5 < max_d {
                            rgba(255, 92, 92, 255)
                        } else {
                            rgba(165, 245, 190, 255)
                        },
                    ));
                }
            }
            if !eq_lines.is_empty() {
                let bh = 4.0 + eq_lines.len() as f32 * 10.0;
                let (ax, ay) = corners.place(Corner::BottomRight, 102.0, bh);
                batch.fill(atlas, ax, ay, 102.0, bh, rgba(12, 14, 22, 185));
                for (i, (txt, col)) in eq_lines.iter().enumerate() {
                    batch.text(atlas, txt, ax + 5.0, ay + 3.0 + i as f32 * 10.0, *col, true);
                }
            }
        }

        // Durability Display (`VoltModId::DurabilityDisplay`) for Mainhand & Offhand
        if cat.is_on(VoltModId::DurabilityDisplay) {
            let mut dur_lines = Vec::new();
            for (tag, stack_opt) in [("Main", self.held()), ("Off", self.offhand.as_ref())] {
                if let Some(st) = stack_opt
                    && let Some(max_d) = self
                        .res
                        .items()
                        .get(&st.item)
                        .map(|i| i.max_damage)
                        .filter(|&m| m > 0)
                {
                    let rem = max_d.saturating_sub(st.damage);
                    let pct = (rem * 100) / max_d.max(1);
                    let col = if pct <= 15 {
                        rgba(255, 84, 84, 255)
                    } else if pct <= 40 {
                        rgba(255, 206, 84, 255)
                    } else {
                        rgba(120, 245, 175, 255)
                    };
                    dur_lines.push((format!("{tag}: {rem}/{max_d} ({pct}%)"), col));
                }
            }
            if !dur_lines.is_empty() {
                let bh = 4.0 + dur_lines.len() as f32 * 10.0;
                let (dx, dy) = corners.place(Corner::BottomRight, 118.0, bh);
                batch.fill(atlas, dx, dy, 118.0, bh, rgba(12, 14, 22, 190));
                batch.fill(atlas, dx, dy, 2.0, bh, rgba(120, 245, 175, 235));
                for (i, (txt, col)) in dur_lines.iter().enumerate() {
                    batch.text(atlas, txt, dx + 5.0, dy + 3.0 + i as f32 * 10.0, *col, true);
                }
            }
        }

        // Compact Potion Timers HUD (`Status Effect Timer`)
        if (mods.potion_hud || cat.is_on(VoltModId::StatusEffectTimer))
            && !self.potions.effects.list.is_empty()
        {
            let n = self.potions.effects.list.len().min(5);
            let bh = 4.0 + n as f32 * 10.0;
            let (ex, ey) = corners.place(Corner::TopRight, 112.0, bh);
            batch.fill(atlas, ex, ey, 112.0, bh, rgba(12, 14, 22, 185));
            for (i, eff) in self.potions.effects.list.iter().take(5).enumerate() {
                let secs = (eff.duration.max(0) / 20) as u32;
                let clean = eff.id.trim_start_matches("minecraft:").replace('_', " ");
                let line = format!(
                    "{} {} {:02}:{:02}",
                    capitalize_words(&clean),
                    eff.amplifier + 1,
                    secs / 60,
                    secs % 60
                );
                batch.text(
                    atlas,
                    &line,
                    ex + 4.0,
                    ey + 3.0 + i as f32 * 10.0,
                    rgba(185, 230, 255, 255),
                    true,
                );
            }
        }

        // Keystrokes Overlay
        if mods.keystrokes_hud && self.screen == Screen::Playing {
            let (kx, ky) = corners.place(Corner::BottomLeft, 58.0, 40.0);
            let keys = [
                ("W", 20.0, 0.0, 18.0, self.keys.contains(&KeyCode::KeyW)),
                ("A", 0.0, 20.0, 18.0, self.keys.contains(&KeyCode::KeyA)),
                ("S", 20.0, 20.0, 18.0, self.keys.contains(&KeyCode::KeyS)),
                ("D", 40.0, 20.0, 18.0, self.keys.contains(&KeyCode::KeyD)),
            ];
            for (lbl, ox, oy, kw, down) in keys {
                let bg = if down {
                    rgba(224, 48, 68, 220)
                } else {
                    rgba(16, 18, 26, 185)
                };
                batch.fill(atlas, kx + ox, ky + oy, kw, 18.0, bg);
                batch.text(
                    atlas,
                    lbl,
                    kx + ox + 6.0,
                    ky + oy + 5.0,
                    rgba(250, 250, 255, 255),
                    true,
                );
            }
        }

        // AppleSkin Food & Saturation Preview above hunger bar
        if (mods.appleskin_hud || cat.is_on(VoltModId::AppleSkin))
            && let Some(p) = self.player.as_ref()
            && !self.creative
            && !self.spectator()
        {
            let sat = p.health.food.saturation.clamp(0.0, 20.0);
            let bar_w = (sat / 20.0 * 81.0).round();
            let hx = (w * 0.5).floor() + 10.0;
            let hy = h - 41.0;
            if cat.appleskin_saturation && bar_w > 0.5 {
                batch.fill(atlas, hx, hy, bar_w, 2.0, rgba(255, 212, 64, 225));
            }
            if cat.appleskin_exhaustion && self.remote.is_none() {
                let exhausted = (p.health.food.exhaustion / 4.0).clamp(0.0, 1.0);
                if exhausted > 0.0 {
                    batch.fill(atlas, hx, hy + 3.0, (exhausted * 81.0).round(), 2.0, rgba(235, 90, 70, 210));
                }
            }
            let food = self.stack_in(self.selected).and_then(|held| self.items().get(&held.item).and_then(|item| item.food))
                .or_else(|| if cat.appleskin_offhand {
                    self.offhand.as_ref().and_then(|held| self.items().get(&held.item).and_then(|item| item.food))
                } else { None });
            if cat.appleskin_food_preview && let Some((food_pts, sat_pts)) = food {
                let txt = format!("+{food_pts} Food  +{sat_pts:.1} Sat");
                batch.text(
                    atlas,
                    &txt,
                    hx,
                    hy - 10.0,
                    rgba(255, 232, 125, 255),
                    true,
                );
            }
        }

        // Slime Chunk & Light/Tech Readout
        if mods.slime_tech_hud
            && let Some(p) = self.player.as_ref()
        {
            let bx = p.position.x.floor() as i32;
            let by = p.position.y.floor() as i32;
            let bz = p.position.z.floor() as i32;
            let is_slime = is_slime_chunk(bx >> 4, bz >> 4, self.level.seed);
            let light = self.world.light(bx, by, bz);
            let txt = format!(
                "Chunk ({}, {}) \u{2022} {} \u{2022} L:{}/{}",
                bx >> 4,
                bz >> 4,
                if is_slime { "SLIME CHUNK!" } else { "Normal" },
                light & 15,
                light >> 4
            );
            let tw = atlas.font.width(&txt) + 10.0;
            let (tx, ty) = corners.place(Corner::BottomRight, tw, 13.0);
            batch.fill(atlas, tx, ty, tw, 13.0, rgba(12, 16, 22, 195));
            batch.text(
                atlas,
                &txt,
                tx + 5.0,
                ty + 2.5,
                if is_slime {
                    rgba(110, 255, 130, 255)
                } else {
                    rgba(215, 225, 240, 255)
                },
                true,
            );
        }

        // Custom Crosshair Pro (#7 in Volt Mods)
        if mods.custom_crosshair != CrosshairStyle::Vanilla && self.screen == Screen::Playing {
            let cx = (w * 0.5).floor();
            let cy = (h * 0.5).floor();
            let over_mob = self.mob_hit().is_some_and(|(_, d)| d <= 4.5);
            let col = if over_mob {
                rgba(255, 62, 82, 245)
            } else {
                rgba(90, 250, 210, 235)
            };
            match mods.custom_crosshair {
                CrosshairStyle::Vanilla => {}
                CrosshairStyle::PrecisionDot => {
                    batch.fill(atlas, cx - 1.5, cy - 1.5, 3.0, 3.0, col);
                }
                CrosshairStyle::CrispPlus => {
                    batch.fill(atlas, cx - 5.0, cy - 0.5, 3.5, 1.0, col);
                    batch.fill(atlas, cx + 1.5, cy - 0.5, 3.5, 1.0, col);
                    batch.fill(atlas, cx - 0.5, cy - 5.0, 1.0, 3.5, col);
                    batch.fill(atlas, cx - 0.5, cy + 1.5, 1.0, 3.5, col);
                }
                CrosshairStyle::CircleRing => {
                    batch.fill(atlas, cx - 1.0, cy - 1.0, 2.0, 2.0, col);
                    batch.fill(atlas, cx - 4.0, cy - 4.0, 8.0, 1.0, col);
                    batch.fill(atlas, cx - 4.0, cy + 3.0, 8.0, 1.0, col);
                    batch.fill(atlas, cx - 4.0, cy - 3.0, 1.0, 6.0, col);
                    batch.fill(atlas, cx + 3.0, cy - 3.0, 1.0, 6.0, col);
                }
                CrosshairStyle::PvpChevron => {
                    batch.fill(atlas, cx - 3.0, cy + 1.0, 3.0, 1.2, col);
                    batch.fill(atlas, cx, cy + 1.0, 3.0, 1.2, col);
                    batch.fill(atlas, cx - 1.0, cy - 1.0, 2.0, 2.0, col);
                }
            }
        }

        // Damage Direction Indicator Arc (#18 in Volt Mods)
        if mods.damage_direction_hud
            && let Some(p) = self.player.as_ref()
            && p.health.hurt_animation_time > 0
        {
            let ang = (p.health.hurt_dir - self.yaw).to_radians();
            let cx = w * 0.5 + ang.sin() * 34.0;
            let cy = h * 0.5 - ang.cos() * 34.0;
            let alpha = ((p.health.hurt_animation_time as u32) * 24).min(235) as u8;
            batch.fill(atlas, cx - 5.0, cy - 5.0, 10.0, 10.0, rgba(255, 42, 64, alpha));
        }

        // Floating Damage Popups (#16 in Volt Mods)
        if mods.damage_popups
            && let Some((cam_pos, cam_yaw, cam_pitch, cam_fov)) = self.locator_camera
        {
            for pop in &self.extras.volt.damage_popups {
                let age = (now - pop.born_at) as f32;
                let world_pt = [pop.pos[0], pop.pos[1] + f64::from(age * 0.65), pop.pos[2]];
                let proj = project_waypoint(cam_pos, cam_yaw, cam_pitch, cam_fov, w, h, world_pt);
                if proj.in_front && proj.on_screen && proj.distance < 36.0 {
                    let txt = format!("-{:.1}", pop.amount);
                    batch.text(
                        atlas,
                        &txt,
                        proj.screen_x - 8.0,
                        proj.screen_y - 4.0,
                        rgba(255, 86, 96, 255),
                        true,
                    );
                }
            }
        }

        // #9 Volt Cam Rule-of-Thirds Grid & 21:9 Cinema Letterbox
        if qol.photo_mode_overlays {
            let c = rgba(255, 255, 255, 58);
            batch.fill(atlas, w / 3.0, 0.0, 0.8, h, c);
            batch.fill(atlas, w * 2.0 / 3.0, 0.0, 0.8, h, c);
            batch.fill(atlas, 0.0, h / 3.0, w, 0.8, c);
            batch.fill(atlas, 0.0, h * 2.0 / 3.0, w, 0.8, c);
        }
        if qol.cam_letterbox {
            let bar = (h * 0.10).round();
            batch.fill(atlas, 0.0, 0.0, w, bar, rgba(0, 0, 0, 235));
            batch.fill(atlas, 0.0, h - bar, w, bar, rgba(0, 0, 0, 235));
        }

        // Dynamic Crosshair (`VoltModId::DynamicCrosshair`) — reactive target brackets & swing bloom
        if cat.is_on(VoltModId::DynamicCrosshair) && self.screen == Screen::Playing {
            let cx = (w * 0.5).floor();
            let cy = (h * 0.5).floor();
            let hit_mob = self.mob_hit().filter(|(_, d)| *d <= 4.5);
            let hit_blk = self.pick().is_some();
            let swing_gap = if self.swing.current.is_some() {
                3.5
            } else {
                0.0
            };
            let (col, show_brackets) = if hit_mob.is_some() {
                (rgba(255, 64, 88, 240), true)
            } else if hit_blk {
                (rgba(250, 210, 96, 195), true)
            } else {
                (rgba(200, 235, 255, 130), swing_gap > 0.0)
            };
            if show_brackets {
                let r = 6.0 + swing_gap;
                // 4 crisp corner brackets around reticle
                batch.fill(atlas, cx - r, cy - r, 3.0, 1.0, col);
                batch.fill(atlas, cx - r, cy - r, 1.0, 3.0, col);
                batch.fill(atlas, cx + r - 2.0, cy - r, 3.0, 1.0, col);
                batch.fill(atlas, cx + r, cy - r, 1.0, 3.0, col);
                batch.fill(atlas, cx - r, cy + r, 3.0, 1.0, col);
                batch.fill(atlas, cx - r, cy + r - 2.0, 1.0, 3.0, col);
                batch.fill(atlas, cx + r - 2.0, cy + r, 3.0, 1.0, col);
                batch.fill(atlas, cx + r, cy + r - 2.0, 1.0, 3.0, col);
            }
        }

        // #24 OLED AFK Power Saver (dims screen cleanly after 90s idle)
        if mods.oled_afk_saver && now - self.extras.volt.last_input_at > 90.0 {
            batch.fill(atlas, 0.0, 0.0, w, h, rgba(0, 0, 0, 215));
            let msg = "Volt OLED Saver Active \u{2014} Touch or Move to Wake";
            let mw = atlas.font.width(msg);
            batch.text(
                atlas,
                msg,
                (w - mw) * 0.5,
                h * 0.5 - 4.0,
                rgba(160, 170, 190, 200),
                true,
            );
        }

        // Inventory HUD+ (`VoltModId::InventoryHudPlus`) — live 9x3 main-inventory mini-grid
        if self
            .settings
            .extras
            .mod_catalog
            .is_on(VoltModId::InventoryHudPlus)
            && self.screen == Screen::Playing
        {
            let op_pct = self
                .settings
                .extras
                .mod_catalog
                .inventory_hud_opacity
                .clamp(20, 100);
            let bg_a = ((u32::from(op_pct) * 210) / 100) as u8;
            let cell = 18.0;
            let gw = cell * 9.0 + 4.0;
            let gh = cell * 3.0 + 4.0;
            let (gx, gy) = corners.place(Corner::BottomLeft, gw, gh);
            batch.fill(
                &self.gui.atlas,
                gx,
                gy,
                gw,
                gh,
                rgba(10, 13, 20, bg_a),
            );
            batch.fill(
                &self.gui.atlas,
                gx,
                gy,
                gw,
                1.0,
                rgba(45, 212, 191, bg_a),
            );
            for slot_idx in 9usize..36 {
                let r = (slot_idx - 9) / 9;
                let c = (slot_idx - 9) % 9;
                let sx = gx + 2.0 + c as f32 * cell;
                let sy = gy + 2.0 + r as f32 * cell;
                batch.fill(
                    &self.gui.atlas,
                    sx + 0.5,
                    sy + 0.5,
                    17.0,
                    17.0,
                    rgba(22, 28, 40, bg_a.saturating_sub(30)),
                );
                if let Some(st) = self.inventory[slot_idx].clone() {
                    self.draw_item(batch, &st, sx + 1.0, sy + 1.0);
                    if st.count > 1 {
                        let cnt = st.count.to_string();
                        let tw = HudBatch::text_width(&self.gui.atlas, &cnt);
                        batch.text(
                            &self.gui.atlas,
                            &cnt,
                            sx + 17.0 - tw,
                            sy + 9.0,
                            u32::MAX,
                            true,
                        );
                    }
                }
            }
        }
    }

    /// The floating replay button's rectangle, from its saved centre
    /// (percent of the screen) and edge length.
    pub(super) fn replay_btn_rect(&self, w: f32, h: f32) -> [f32; 4] {
        let q = &self.settings.extras.volt_qol;
        let s = f32::from(q.replay_btn_size.clamp(22, 72));
        let cx = w * f32::from(q.replay_btn_x_pct.min(100)) / 100.0;
        let cy = h * f32::from(q.replay_btn_y_pct.min(100)) / 100.0;
        [
            (cx - s * 0.5).clamp(2.0, (w - s - 2.0).max(2.0)),
            (cy - s * 0.5).clamp(2.0, (h - s - 2.0).max(2.0)),
            s,
            s,
        ]
    }

    /// The Start / Pause / End rows beside the button, kept on screen.
    pub(super) fn replay_btn_menu_rects(&self, w: f32, h: f32) -> [[f32; 4]; 3] {
        let b = self.replay_btn_rect(w, h);
        let (mw, mh, gap) = (62.0, 15.0, 3.0);
        let x = if b[0] + b[2] + 6.0 + mw <= w - 2.0 {
            b[0] + b[2] + 6.0
        } else {
            (b[0] - 6.0 - mw).max(2.0)
        };
        let total = mh * 3.0 + gap * 2.0;
        let y0 = (b[1] + b[3] * 0.5 - total * 0.5).clamp(2.0, (h - total - 2.0).max(2.0));
        [
            [x, y0, mw, mh],
            [x, y0 + mh + gap, mw, mh],
            [x, y0 + 2.0 * (mh + gap), mw, mh],
        ]
    }

    /// Draws the floating replay button, its state, its Start / Pause /
    /// End rows and the fading caption under it.
    fn draw_replay_button(&mut self, batch: &mut HudBatch) {
        let Some(rec) = self.remote.as_ref().and_then(|r| r.recorder.clone()) else {
            return;
        };
        // Holding still on the button turns the touch into the adjust
        // gesture even when the finger never moves a pixel.
        if let Some(t) = self.extras.volt.replay_btn_touch.as_mut()
            && t.mode == 0
            && t.at.elapsed().as_millis() >= 350
        {
            t.mode = 2;
            self.extras.volt.replay_btn_note = Some((
                "drag up/down: size \u{2022} left/right: fade".to_owned(),
                std::time::Instant::now(),
            ));
        }
        let (w, h) = (batch.width, batch.height);
        let b = self.replay_btn_rect(w, h);
        let q = &self.settings.extras.volt_qol;
        let a = (f32::from(q.replay_btn_opacity_pct.clamp(25, 100)) * 2.55) as u8;
        let state = rec.state();
        let adjusting = self
            .extras
            .volt
            .replay_btn_touch
            .as_ref()
            .is_some_and(|t| t.mode != 0);
        let frame = if adjusting {
            rgba(121, 229, 156, a)
        } else {
            rgba(235, 240, 250, a / 2)
        };
        batch.fill(&self.gui.atlas, b[0] - 1.0, b[1] - 1.0, b[2] + 2.0, b[3] + 2.0, frame);
        batch.fill(&self.gui.atlas, b[0], b[1], b[2], b[3], rgba(10, 14, 22, a));
        // The state glyph in the middle: a red dot while recording (it
        // breathes), pause bars while frozen, a hollow dot while idle.
        let (cx, cy) = (b[0] + b[2] * 0.5, b[1] + b[3] * 0.5);
        let g = (b[2] * 0.22).clamp(4.0, 12.0);
        match state {
            horizon_client::replay::REC_ON => {
                let pulse = (self.started.elapsed().as_secs_f32() * 2.0).sin() * 0.5 + 0.5;
                let ga = (f32::from(a) * (0.6 + 0.4 * pulse)) as u8;
                batch.fill(
                    &self.gui.atlas,
                    cx - g * 0.5,
                    cy - g * 0.5,
                    g,
                    g,
                    rgba(235, 80, 90, ga),
                );
            }
            horizon_client::replay::REC_PAUSED => {
                let bw = (g * 0.38).max(2.0);
                batch.fill(&self.gui.atlas, cx - g * 0.5, cy - g * 0.5, bw, g, rgba(235, 240, 250, a));
                batch.fill(&self.gui.atlas, cx + g * 0.5 - bw, cy - g * 0.5, bw, g, rgba(235, 240, 250, a));
            }
            _ => {
                batch.fill(&self.gui.atlas, cx - g * 0.5, cy - g * 0.5, g, g, rgba(235, 80, 90, a / 2));
                batch.fill(
                    &self.gui.atlas,
                    cx - g * 0.5 + 1.0,
                    cy - g * 0.5 + 1.0,
                    g - 2.0,
                    g - 2.0,
                    rgba(10, 14, 22, a),
                );
            }
        }
        // The running clock under the button while anything is captured.
        if state != horizon_client::replay::REC_OFF && b[2] >= 30.0 {
            let clock = horizon_client::replay::format_time(rec.duration_ms());
            let tw = HudBatch::text_width(&self.gui.atlas, &clock);
            batch.text(
                &self.gui.atlas,
                &clock,
                cx - tw * 0.5,
                b[1] + b[3] + 2.0,
                rgba(235, 240, 250, a),
                true,
            );
        }
        // The caption: adjust hints, saved/paused notes.
        let expired = self
            .extras
            .volt
            .replay_btn_note
            .as_ref()
            .is_some_and(|(_, at)| at.elapsed().as_secs_f32() >= 3.0);
        if expired && !adjusting {
            self.extras.volt.replay_btn_note = None;
        }
        if let Some((text, at)) = &self.extras.volt.replay_btn_note
            && (at.elapsed().as_secs_f32() < 3.0 || adjusting)
        {
            let label = if adjusting {
                format!(
                    "{} \u{2022} {}px {}%",
                    text, q.replay_btn_size, q.replay_btn_opacity_pct
                )
            } else {
                text.clone()
            };
            let tw = HudBatch::text_width(&self.gui.atlas, &label);
            batch.text(
                &self.gui.atlas,
                &label,
                (cx - tw * 0.5).clamp(2.0, (w - tw - 2.0).max(2.0)),
                (b[1] - 12.0).max(2.0),
                rgba(255, 228, 160, 245),
                true,
            );
        }
        // The Start / Pause / End rows.
        if self.extras.volt.replay_btn_menu {
            let rows = self.replay_btn_menu_rects(w, h);
            let labels = [
                "START",
                if state == horizon_client::replay::REC_PAUSED {
                    "PAUSED"
                } else {
                    "PAUSE"
                },
                "END",
            ];
            for (i, (rect, label)) in rows.iter().zip(labels).enumerate() {
                let active = (i == 0 && state == horizon_client::replay::REC_ON)
                    || (i == 1 && state == horizon_client::replay::REC_PAUSED);
                let fill = if active {
                    rgba(42, 92, 62, 240)
                } else if i == 2 {
                    rgba(84, 30, 36, 240)
                } else {
                    rgba(28, 36, 52, 235)
                };
                batch.fill(&self.gui.atlas, rect[0], rect[1], rect[2], rect[3], fill);
                let color = if active {
                    rgba(121, 229, 156, 255)
                } else if i == 2 {
                    rgba(255, 150, 158, 255)
                } else {
                    rgba(235, 240, 250, 255)
                };
                let tw = HudBatch::text_width(&self.gui.atlas, label);
                batch.text(
                    &self.gui.atlas,
                    label,
                    rect[0] + ((rect[2] - tw) * 0.5).max(1.0),
                    rect[1] + 4.0,
                    color,
                    true,
                );
            }
        }
    }

    fn replay_btn_caption(&mut self, text: &str) {
        self.extras.volt.replay_btn_note =
            Some((text.to_owned(), std::time::Instant::now()));
    }

    /// A tapped Start / Pause / End row.
    fn replay_btn_menu_act(&mut self, row: usize) {
        let Some(rec) = self.remote.as_ref().and_then(|r| r.recorder.clone()) else {
            return;
        };
        let state = rec.state();
        match row {
            0 => match state {
                horizon_client::replay::REC_PAUSED => {
                    rec.resume();
                    self.replay_btn_caption("Recording");
                }
                horizon_client::replay::REC_ON => {
                    self.replay_btn_caption("Already recording");
                }
                _ => self.replay_btn_caption("Saved \u{2022} rejoin to record again"),
            },
            1 => match state {
                horizon_client::replay::REC_ON => {
                    rec.pause();
                    self.replay_btn_caption("Paused \u{2022} START resumes");
                }
                horizon_client::replay::REC_PAUSED => {
                    self.replay_btn_caption("Already paused");
                }
                _ => self.replay_btn_caption("Nothing recording"),
            },
            _ => match state {
                horizon_client::replay::REC_OFF => {
                    self.replay_btn_caption("Nothing recording");
                }
                _ => {
                    rec.finish();
                    self.replay_btn_caption("Saved \u{2022} Home > Replays");
                }
            },
        }
        self.extras.volt.replay_btn_menu = false;
    }

    /// One finger on or owning the floating replay button. Consumes the
    /// touch while the button, its rows, or a drag own it.
    pub(super) fn replay_button_touch(
        &mut self,
        id: u64,
        p: (f32, f32),
        phase: winit::event::TouchPhase,
    ) -> bool {
        use winit::event::TouchPhase;
        // A gesture in progress owns its touch id to the end; everything
        // else rejects in a couple of comparisons, so an idle frame pays
        // nothing for this hook.
        let owns = self
            .extras
            .volt
            .replay_btn_touch
            .as_ref()
            .is_some_and(|t| t.id == id);
        if !owns
            && (phase != TouchPhase::Started
                || !self.settings.extras.volt_qol.replay_button
                || self.title.is_some()
                || self.ui.hide_gui
                || self.screen != Screen::Playing
                || self.replay_control().is_some()
                || self.remote.as_ref().is_none_or(|r| r.recorder.is_none()))
        {
            return false;
        }
        let (w, h) = self.gui_size();
        if owns {
            match phase {
                TouchPhase::Started => return false,
                TouchPhase::Moved => {
                    let t = self.extras.volt.replay_btn_touch.as_mut().unwrap();
                    let dx = p.0 - t.start.0;
                    let dy = p.1 - t.start.1;
                    if t.mode == 0 && (dx.abs() >= 6.0 || dy.abs() >= 6.0) {
                        t.mode = if t.at.elapsed().as_millis() >= 350 { 2 } else { 1 };
                    }
                    let (mode, base) = (t.mode, t.base);
                    let q = &mut self.settings.extras.volt_qol;
                    if mode == 1 {
                        let nx = f32::from(base.0) + dx / w.max(1.0) * 100.0;
                        let ny = f32::from(base.1) + dy / h.max(1.0) * 100.0;
                        q.replay_btn_x_pct = nx.round().clamp(0.0, 100.0) as u16;
                        q.replay_btn_y_pct = ny.round().clamp(0.0, 100.0) as u16;
                    } else if mode == 2 {
                        let size = f32::from(base.2) - dy * 0.5;
                        let op = f32::from(base.3) + dx * 0.35;
                        q.replay_btn_size = size.round().clamp(22.0, 72.0) as u16;
                        q.replay_btn_opacity_pct = op.round().clamp(25.0, 100.0) as u16;
                    }
                    return true;
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    let t = self.extras.volt.replay_btn_touch.take().unwrap();
                    if phase == TouchPhase::Ended
                        && t.mode == 0
                        && t.at.elapsed().as_millis() < 350
                    {
                        // A tap: the Start / Pause / End rows.
                        self.extras.volt.replay_btn_menu = !self.extras.volt.replay_btn_menu;
                        self.audio.play_button_click();
                    } else if t.mode != 0 {
                        // A finished drag or adjust: keep it.
                        self.extras.volt.replay_btn_note = None;
                        self.settings.save(&Self::options_path());
                    }
                    return true;
                }
            }
        }
        let hit = |r: &[f32; 4]| {
            p.0 >= r[0] && p.0 <= r[0] + r[2] && p.1 >= r[1] && p.1 <= r[1] + r[3]
        };
        // The open rows first; a tap elsewhere just closes them.
        if self.extras.volt.replay_btn_menu {
            let rows = self.replay_btn_menu_rects(w, h);
            for (i, rect) in rows.iter().enumerate() {
                if hit(rect) {
                    self.replay_btn_menu_act(i);
                    self.audio.play_button_click();
                    return true;
                }
            }
            if !hit(&self.replay_btn_rect(w, h)) {
                self.extras.volt.replay_btn_menu = false;
                return false;
            }
        }
        if hit(&self.replay_btn_rect(w, h)) {
            let q = &self.settings.extras.volt_qol;
            self.extras.volt.replay_btn_touch = Some(ReplayBtnTouch {
                id,
                start: p,
                at: std::time::Instant::now(),
                mode: 0,
                base: (
                    q.replay_btn_x_pct,
                    q.replay_btn_y_pct,
                    q.replay_btn_size,
                    q.replay_btn_opacity_pct,
                ),
            });
            return true;
        }
        false
    }

    /// Replay Studio's editor panel: a full-width dock at the bottom.
    /// Returns (panel, feature row + its labels, transport row, timeline
    /// track). The feature labels shrink to codes on narrow screens.
    fn replay_studio_rects(
        &self,
        w: f32,
        h: f32,
    ) -> (
        [f32; 4],
        [[f32; 4]; 11],
        [&'static str; 11],
        [[f32; 4]; 8],
        [f32; 4],
    ) {
        let atlas = &self.gui.atlas;
        let panel = [0.0, h - 64.0, w, 64.0];
        let full: [&'static str; 11] = [
            "MARK", "<M", "M>", "+KF", "CLR", "PATH", "LAPSE", "MUSIC", "LOOP", "SHADER", "RENDER",
        ];
        let compact: [&'static str; 11] =
            ["MRK", "<M", "M>", "KF", "CLR", "PTH", "TLP", "MUS", "LP", "SHD", "REC"];
        let measure = |set: &[&'static str; 11]| -> ([f32; 11], f32) {
            let mut ws = [0.0f32; 11];
            let mut total = 0.0;
            for (i, s) in set.iter().enumerate() {
                ws[i] = HudBatch::text_width(atlas, s) + 10.0;
                total += ws[i];
            }
            (ws, total + 10.0 * 3.0)
        };
        let (mut ws, mut total) = measure(&full);
        let mut labels = full;
        if total > w - 8.0 {
            (ws, total) = measure(&compact);
            labels = compact;
        }
        let mut row_a = [[0.0f32; 4]; 11];
        let mut x = ((w - total) * 0.5).max(4.0);
        for (i, rect) in row_a.iter_mut().enumerate() {
            *rect = [x, panel[1] + 4.0, ws[i], 15.0];
            x += ws[i] + 3.0;
        }
        let track = [10.0, panel[1] + 24.0, w - 20.0, 10.0];
        let tws: [f32; 8] = [24.0, 30.0, 46.0, 30.0, 24.0, 36.0, 42.0, 36.0];
        let ttotal: f32 = tws.iter().sum::<f32>() + 7.0 * 4.0;
        let mut row_t = [[0.0f32; 4]; 8];
        let mut x = ((w - ttotal) * 0.5).max(4.0);
        for (i, rect) in row_t.iter_mut().enumerate() {
            *rect = [x, panel[1] + 40.0, tws[i], 18.0];
            x += tws[i] + 4.0;
        }
        (panel, row_a, labels, row_t, track)
    }

    /// Replay Studio's editor panel while a replay is being watched.
    pub(super) fn draw_replay_hud(&mut self, batch: &mut HudBatch) {
        use std::sync::atomic::Ordering::Relaxed;
        let Some(ctl) = self.replay_control() else {
            return;
        };
        let Some((markers, key_times, cam_path_on, music_on, loop_on, exporting, hud_age, note, stem, shader_menu)) =
            self.remote
                .as_ref()
                .and_then(|r| r.replay.as_ref())
                .map(|rp| {
                    (
                        rp.markers.clone(),
                        rp.cam_keys.iter().map(|k| k.ms).collect::<Vec<_>>(),
                        rp.cam_path,
                        rp.music,
                        rp.loop_play,
                        rp.export.is_some(),
                        rp.hud_touch.elapsed().as_secs_f32(),
                        rp.note.clone(),
                        rp.path
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_default(),
                        rp.shader_menu,
                    )
                })
        else {
            return;
        };
        // SHD lights up whenever a shader runs over the playback (the
        // studio's own pick, or the player's already-on shader).
        let shader_on = self.settings.extras.volt_shaders.enabled
            || !self.settings.horizon.shader_pack.is_empty();
        let (w, h) = (batch.width, batch.height);
        // While exporting, the whole studio hides itself so it is not
        // burned into the video; a tap near the panel brings it back.
        if exporting && hud_age > 3.0 {
            let tag = "REC";
            let tag_w = HudBatch::text_width(&self.gui.atlas, tag);
            batch.fill(
                &self.gui.atlas,
                w - tag_w - 16.0,
                8.0,
                6.0,
                6.0,
                rgba(235, 80, 90, 235),
            );
            batch.text(
                &self.gui.atlas,
                tag,
                w - tag_w - 6.0,
                6.0,
                rgba(235, 80, 90, 235),
                true,
            );
            return;
        }
        let (panel, row_a, labels_a, row_t, track) = self.replay_studio_rects(w, h);
        let t = ctl.time_ms.load(Relaxed);
        let d = ctl.duration_ms.load(Relaxed).max(1);
        let paused = ctl.paused.load(Relaxed);
        let pov = ctl.pov.load(Relaxed);
        let speed = ctl.speed_pct.load(Relaxed);
        let timelapse_on = speed >= 1600;
        // The dock: dark glass with a mint hairline on top.
        batch.fill(
            &self.gui.atlas,
            panel[0],
            panel[1],
            panel[2],
            panel[3],
            rgba(10, 14, 22, 215),
        );
        batch.fill(
            &self.gui.atlas,
            panel[0],
            panel[1],
            panel[2],
            1.0,
            rgba(121, 229, 156, 235),
        );
        // The header line above the dock: the studio tag and the clock.
        let title = format!("\u{2022} REPLAY STUDIO \u{2014} {stem}");
        batch.text(
            &self.gui.atlas,
            &title,
            6.0,
            panel[1] - 12.0,
            rgba(121, 229, 156, 235),
            true,
        );
        let clock = format!(
            "{} / {}{}",
            horizon_client::replay::format_time(t),
            horizon_client::replay::format_time(d),
            if ctl.finished.load(Relaxed) {
                " \u{2022} END"
            } else {
                ""
            },
        );
        let cw = HudBatch::text_width(&self.gui.atlas, &clock);
        batch.text(
            &self.gui.atlas,
            &clock,
            w - cw - 6.0,
            panel[1] - 12.0,
            rgba(235, 240, 250, 235),
            true,
        );
        // The feature row: markers, keyframes, path, timelapse, music,
        // loop and render.
        for (i, (rect, label)) in row_a.iter().zip(labels_a).enumerate() {
            let on = match i {
                5 => cam_path_on,
                6 => timelapse_on,
                7 => music_on,
                8 => loop_on,
                9 => shader_on || shader_menu,
                _ => false,
            };
            let fill = match i {
                10 if exporting => rgba(118, 34, 42, 240),
                _ if on => rgba(42, 92, 62, 240),
                _ => rgba(28, 36, 52, 235),
            };
            batch.fill(&self.gui.atlas, rect[0], rect[1], rect[2], rect[3], fill);
            let color = match i {
                10 if exporting => rgba(255, 120, 130, 255),
                _ if on => rgba(121, 229, 156, 255),
                _ => rgba(235, 240, 250, 255),
            };
            let tw = HudBatch::text_width(&self.gui.atlas, label);
            batch.text(
                &self.gui.atlas,
                label,
                rect[0] + ((rect[2] - tw) * 0.5).max(1.0),
                rect[1] + 4.0,
                color,
                true,
            );
        }
        // The timeline: progress, playhead, markers (amber) and camera
        // keyframes (mint).
        batch.fill(
            &self.gui.atlas,
            track[0],
            track[1],
            track[2],
            track[3],
            rgba(30, 38, 54, 225),
        );
        let frac = (t as f32 / d as f32).clamp(0.0, 1.0);
        if frac > 0.0 {
            batch.fill(
                &self.gui.atlas,
                track[0],
                track[1],
                track[2] * frac,
                track[3],
                rgba(60, 118, 86, 235),
            );
        }
        for m in &markers {
            let x = track[0] + track[2] * (*m as f32 / d as f32).clamp(0.0, 1.0);
            batch.fill(
                &self.gui.atlas,
                x - 1.0,
                track[1] - 2.0,
                2.0,
                track[3] + 4.0,
                rgba(255, 196, 64, 240),
            );
        }
        // Camera keyframes ride the path's own clock, not the replay's:
        // a mint bead per keyframe, queued left of the PATH button's row.
        for (i, _) in key_times.iter().take(24).enumerate() {
            batch.fill(
                &self.gui.atlas,
                track[0] + i as f32 * 8.0,
                track[1] - 5.0,
                5.0,
                4.0,
                rgba(121, 229, 156, 250),
            );
        }
        // The playhead on top of everything.
        let px = track[0] + track[2] * frac;
        batch.fill(
            &self.gui.atlas,
            px - 1.0,
            track[1] - 3.0,
            2.0,
            track[3] + 6.0,
            rgba(245, 248, 255, 250),
        );
        // The transport row: jump to start/end, skip, play, speed, the
        // camera and the exit.
        let speed_label = format!(
            "x{}",
            match speed {
                10 => "0.1",
                25 => "0.25",
                50 => "0.5",
                200 => "2",
                400 => "4",
                800 => "8",
                1600 => "16",
                3200 => "32",
                6400 => "64",
                _ => "1",
            }
        );
        let labels_t: [&str; 8] = [
            "|<",
            "<<",
            if paused { "|>" } else { "||" },
            ">>",
            ">|",
            &speed_label,
            if pov { "POV" } else { "FREE" },
            "EXIT",
        ];
        for (i, (rect, label)) in row_t.iter().zip(labels_t).enumerate() {
            let fill = match i {
                2 => rgba(42, 92, 62, 240),
                7 => rgba(84, 30, 36, 240),
                _ => rgba(28, 36, 52, 235),
            };
            batch.fill(&self.gui.atlas, rect[0], rect[1], rect[2], rect[3], fill);
            let color = match i {
                2 => rgba(121, 229, 156, 255),
                7 => rgba(255, 150, 158, 255),
                _ => rgba(235, 240, 250, 255),
            };
            let tw = HudBatch::text_width(&self.gui.atlas, label);
            batch.text(
                &self.gui.atlas,
                label,
                rect[0] + ((rect[2] - tw) * 0.5).max(1.0),
                rect[1] + 5.0,
                color,
                true,
            );
        }
        // Status notes (marker added, keyframe saved, export outcome).
        if let Some((text, at)) = &note
            && at.elapsed().as_secs_f32() < 5.0
        {
            let tw = HudBatch::text_width(&self.gui.atlas, text);
            batch.text(
                &self.gui.atlas,
                text,
                ((w - tw) * 0.5).max(0.0),
                panel[1] - 26.0,
                rgba(255, 228, 160, 245),
                true,
            );
        }
        // SHADER: the picker above the dock — the built-in cinema look
        // and every shader pack in this instance's shaderpacks folder.
        // The pick colors the playback (and any RENDER export); the
        // player's own shader settings come back untouched on exit.
        if shader_menu {
            let entries = self.replay_shader_entries();
            let rows = Self::replay_shader_rows(w, h, entries.len());
            let current = self.replay_effective_shader();
            if let Some(first) = rows.first() {
                let head = "SHADER \u{2014} colors playback & render";
                batch.text(
                    &self.gui.atlas,
                    head,
                    first[0],
                    first[1] - 12.0,
                    rgba(121, 229, 156, 235),
                    true,
                );
            }
            for (rect, (label, pick)) in rows.iter().zip(entries.iter()) {
                let active = *pick == current;
                let fill = if active {
                    rgba(42, 92, 62, 240)
                } else {
                    rgba(10, 14, 22, 235)
                };
                batch.fill(&self.gui.atlas, rect[0], rect[1], rect[2], rect[3], fill);
                let color = if active {
                    rgba(121, 229, 156, 255)
                } else {
                    rgba(235, 240, 250, 255)
                };
                let label = if HudBatch::text_width(&self.gui.atlas, label) > rect[2] - 8.0 {
                    let mut s = label.clone();
                    while s.len() > 4
                        && HudBatch::text_width(&self.gui.atlas, &format!("{s}..")) > rect[2] - 8.0
                    {
                        s.pop();
                    }
                    format!("{s}..")
                } else {
                    label.clone()
                };
                batch.text(
                    &self.gui.atlas,
                    &label,
                    rect[0] + 4.0,
                    rect[1] + 4.0,
                    color,
                    true,
                );
            }
        }
    }

    /// What currently shades the frame: `__volt_shaders__`, a pack's file
    /// name, or empty for the plain renderer.
    fn replay_effective_shader(&self) -> String {
        if self.settings.extras.volt_shaders.enabled {
            super::native_shaders::VOLT_SHADERS_PICKED.to_owned()
        } else {
            self.settings.horizon.shader_pack.clone()
        }
    }

    /// The picker's entries: (label, what to pick).
    fn replay_shader_entries(&self) -> Vec<(String, String)> {
        let mut entries = vec![
            ("NO SHADER".to_owned(), String::new()),
            (
                "VOLT CINEMA \u{2022} BUILT IN".to_owned(),
                super::native_shaders::VOLT_SHADERS_PICKED.to_owned(),
            ),
        ];
        for pack in self.settings.shader_packs.iter().take(9) {
            let title = if pack.title.is_empty() {
                pack.file.clone()
            } else {
                pack.title.clone()
            };
            entries.push((title, pack.file.clone()));
        }
        entries
    }

    /// The picker's row rectangles, stacked upward from the dock.
    fn replay_shader_rows(w: f32, h: f32, n: usize) -> Vec<[f32; 4]> {
        let (rw, rh, gap) = (210.0_f32.min(w - 8.0), 14.0, 2.0);
        let x = ((w - rw) * 0.5).max(4.0);
        let total = n as f32 * (rh + gap);
        let y0 = (h - 64.0 - 6.0 - total).max(16.0);
        (0..n)
            .map(|i| [x, y0 + i as f32 * (rh + gap), rw, rh])
            .collect()
    }

    /// SHD tapped: open or close the picker; scan the shaderpacks folder
    /// the first time so external packs show up.
    fn replay_toggle_shader_menu(&mut self) {
        if self.settings.shader_packs.is_empty() {
            self.settings.shader_packs =
                horizon_client::shader_pack_list::scan(&Self::shader_packs_dir());
        }
        if let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) {
            rp.shader_menu = !rp.shader_menu;
        }
    }

    /// A picker row tapped: shade the playback with the pick. The pick
    /// lives only inside this viewing — the player's own shader settings
    /// are remembered on the first pick and restored when the viewer
    /// leaves, so normal play never pays for it.
    fn replay_apply_shader(&mut self, pick: &str, label: &str) {
        if self.remote.as_ref().and_then(|r| r.replay.as_ref()).is_none() {
            return;
        }
        let saved = (
            self.settings.horizon.shader_pack.clone(),
            self.settings.extras.volt_shaders.enabled,
        );
        if pick == super::native_shaders::VOLT_SHADERS_PICKED {
            self.settings.extras.volt_shaders.enabled = true;
        } else {
            self.settings.extras.volt_shaders.enabled = false;
            self.settings.horizon.shader_pack = pick.to_owned();
        }
        let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) else {
            return;
        };
        if rp.shader_saved.is_none() {
            rp.shader_saved = Some(saved);
        }
        rp.shader_pick = Some(pick.to_owned());
        rp.shader_menu = false;
        let note = if pick.is_empty() {
            "Shader off: the plain frame.".to_owned()
        } else {
            format!("{label} shades the playback \u{2014} RENDER bakes it in.")
        };
        self.replay_note(note);
    }

    /// Taps on the replay playback bar.
    pub(super) fn try_click_replay_hud(&mut self, cursor: (f32, f32), pressed: bool) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        if !pressed {
            return false;
        }
        let Some(ctl) = self.replay_control() else {
            return false;
        };
        if self.title.is_some() || self.ui.hide_gui || self.screen != Screen::Playing {
            return false;
        }
        let (w, h) = self.gui_size();
        let (panel, row_a, _labels_a, row_t, track) = self.replay_studio_rects(w, h);
        let hit = |r: &[f32; 4]| {
            cursor.0 >= r[0] && cursor.0 <= r[0] + r[2] && cursor.1 >= r[1] && cursor.1 <= r[1] + r[3]
        };
        // While exporting, the hidden studio comes back with one tap near
        // the dock (and that tap does nothing else).
        if let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) {
            if rp.export.is_some() && rp.hud_touch.elapsed().as_secs_f32() > 3.0 {
                if cursor.1 >= panel[1] - 16.0 {
                    rp.hud_touch = std::time::Instant::now();
                    return true;
                }
                return false;
            }
            rp.hud_touch = std::time::Instant::now();
        }
        // The shader picker above the dock gets the tap first while it is
        // open; a tap anywhere else closes it and falls through.
        if self
            .remote
            .as_ref()
            .and_then(|r| r.replay.as_ref())
            .is_some_and(|rp| rp.shader_menu)
        {
            let entries = self.replay_shader_entries();
            let rows = Self::replay_shader_rows(w, h, entries.len());
            let picked = rows
                .iter()
                .position(hit)
                .and_then(|i| entries.get(i).cloned());
            if let Some((label, pick)) = picked {
                self.replay_apply_shader(&pick, &label);
                self.audio.play_button_click();
                return true;
            }
            if let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) {
                rp.shader_menu = false;
            }
            // SHD itself closes the picker; without this the fall-through
            // would just reopen it.
            if hit(&row_a[9]) {
                self.audio.play_button_click();
                return true;
            }
        }
        let t = ctl.time_ms.load(Relaxed);
        let d = ctl.duration_ms.load(Relaxed);
        let acted = if hit(&row_a[0]) {
            self.replay_toggle_marker();
            true
        } else if hit(&row_a[1]) {
            self.replay_jump_marker(false);
            true
        } else if hit(&row_a[2]) {
            self.replay_jump_marker(true);
            true
        } else if hit(&row_a[3]) {
            self.replay_add_keyframe();
            true
        } else if hit(&row_a[4]) {
            self.replay_clear_keyframes();
            true
        } else if hit(&row_a[5]) {
            self.replay_toggle_path();
            true
        } else if hit(&row_a[6]) {
            self.replay_toggle_timelapse();
            true
        } else if hit(&row_a[7]) {
            self.replay_toggle_music();
            true
        } else if hit(&row_a[8]) {
            self.replay_toggle_loop();
            true
        } else if hit(&row_a[9]) {
            self.replay_toggle_shader_menu();
            true
        } else if hit(&row_a[10]) {
            self.replay_toggle_export();
            true
        } else if hit(&row_t[0]) {
            ctl.request_seek(0);
            true
        } else if hit(&row_t[1]) {
            ctl.request_seek(t.saturating_sub(10_000));
            true
        } else if hit(&row_t[2]) {
            let paused = ctl.paused.load(Relaxed);
            ctl.paused.store(!paused, Relaxed);
            true
        } else if hit(&row_t[3]) {
            ctl.request_seek(t.saturating_add(10_000).min(d));
            true
        } else if hit(&row_t[4]) {
            ctl.request_seek(d);
            true
        } else if hit(&row_t[5]) {
            let next = match ctl.speed_pct.load(Relaxed) {
                10 => 25,
                25 => 50,
                50 => 100,
                100 => 200,
                200 => 400,
                400 => 800,
                800 => 10,
                // Timelapse speeds (LAPSE) drop back to real time.
                _ => 100,
            };
            ctl.speed_pct.store(next, Relaxed);
            true
        } else if hit(&row_t[6]) {
            let pov = ctl.pov.load(Relaxed);
            ctl.pov.store(!pov, Relaxed);
            // Going back to the recorded view lands a running flight.
            if !pov
                && let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut())
                && rp.cam_path
            {
                rp.cam_path = false;
                rp.path_started = None;
                rp.control.flying.store(false, Relaxed);
            }
            true
        } else if hit(&row_t[7]) {
            ctl.exit.store(true, Relaxed);
            true
        } else if hit(&track)
            || (cursor.0 >= track[0]
                && cursor.0 <= track[0] + track[2]
                && cursor.1 >= track[1] - 4.0
                && cursor.1 <= track[1] + track[3] + 4.0)
        {
            let frac = ((cursor.0 - track[0]) / track[2]).clamp(0.0, 1.0);
            ctl.request_seek((frac * d as f32) as u32);
            true
        } else {
            hit(&panel)
        };
        if acted {
            self.audio.play_button_click();
        }
        acted
    }

    /// Puts a status line above the playback bar for a few seconds.
    fn replay_note(&mut self, text: String) {
        if let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) {
            rp.note = Some((text, std::time::Instant::now()));
        }
    }

    /// MRK: drops a marker at the playhead, or removes the one within a
    /// second of it. Markers persist in the replay's sidecar.
    fn replay_toggle_marker(&mut self) {
        use std::sync::atomic::Ordering::Relaxed;
        let Some(ctl) = self.replay_control() else {
            return;
        };
        let t = ctl.time_ms.load(Relaxed);
        let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) else {
            return;
        };
        let note = if let Some(i) = rp.markers.iter().position(|m| m.abs_diff(t) < 1000) {
            rp.markers.remove(i);
            "Marker removed.".to_string()
        } else {
            rp.markers.push(t);
            rp.markers.sort_unstable();
            format!(
                "Marker at {}.",
                horizon_client::replay::format_time(t)
            )
        };
        horizon_client::replay::save_sidecar(&rp.path, &rp.markers, &rp.cam_keys);
        rp.note = Some((note, std::time::Instant::now()));
    }

    /// `<M` / `M>`: seek to the previous or next marker.
    fn replay_jump_marker(&mut self, forward: bool) {
        use std::sync::atomic::Ordering::Relaxed;
        let Some(ctl) = self.replay_control() else {
            return;
        };
        let t = ctl.time_ms.load(Relaxed);
        let target = self
            .remote
            .as_ref()
            .and_then(|r| r.replay.as_ref())
            .and_then(|rp| {
                if forward {
                    rp.markers.iter().find(|&&m| m > t.saturating_add(500)).copied()
                } else {
                    rp.markers
                        .iter()
                        .rev()
                        .find(|&&m| m.saturating_add(500) < t)
                        .copied()
                }
            });
        match target {
            Some(m) => ctl.request_seek(m),
            None => self.replay_note(if forward {
                "No marker ahead.".to_string()
            } else {
                "No marker behind.".to_string()
            }),
        }
    }

    /// Seconds of flight between two camera keyframes.
    const REPLAY_KEY_GAP_MS: u32 = 5_000;

    /// +KF: saves the free camera's position and angles as the next stop
    /// on the camera path. The path has its own timeline (a keyframe
    /// every 5 seconds of flight), so keyframes can be framed calmly
    /// while the replay is paused — they never collide or get replaced.
    fn replay_add_keyframe(&mut self) {
        use std::sync::atomic::Ordering::Relaxed;
        let Some(ctl) = self.replay_control() else {
            return;
        };
        if ctl.pov.load(Relaxed) {
            self.replay_note("Keyframes are placed in FREE view.".to_string());
            return;
        }
        let Some(player) = self.player.as_ref() else {
            return;
        };
        let (pos, yaw, pitch) = (
            [player.position.x, player.position.y, player.position.z],
            self.yaw,
            self.pitch,
        );
        let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) else {
            return;
        };
        let ms = rp
            .cam_keys
            .last()
            .map(|k| k.ms + Self::REPLAY_KEY_GAP_MS)
            .unwrap_or(0);
        rp.cam_keys.push(horizon_client::replay::CamKey {
            ms,
            pos,
            yaw,
            pitch,
        });
        let n = rp.cam_keys.len();
        let note = if n < 2 {
            format!("Keyframe {n} saved (2+ make a path).")
        } else {
            format!(
                "Keyframe {n} saved \u{2014} {} flight.",
                horizon_client::replay::format_time(ms)
            )
        };
        horizon_client::replay::save_sidecar(&rp.path, &rp.markers, &rp.cam_keys);
        rp.note = Some((note, std::time::Instant::now()));
    }

    /// CLR: forgets the camera path.
    fn replay_clear_keyframes(&mut self) {
        let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) else {
            return;
        };
        if rp.cam_keys.is_empty() {
            rp.note = Some(("No keyframes to clear.".to_string(), std::time::Instant::now()));
            return;
        }
        rp.cam_keys.clear();
        rp.cam_path = false;
        rp.path_started = None;
        rp.control
            .flying
            .store(false, std::sync::atomic::Ordering::Relaxed);
        horizon_client::replay::save_sidecar(&rp.path, &rp.markers, &rp.cam_keys);
        rp.note = Some(("Camera path cleared.".to_string(), std::time::Instant::now()));
    }

    /// PATH: flies the camera through the keyframes, start to finish, on
    /// the path's own clock - the world underneath can be paused, slowed
    /// or timelapsed and the flight stays smooth. A second tap stops the
    /// flight.
    fn replay_toggle_path(&mut self) {
        use std::sync::atomic::Ordering::Relaxed;
        let Some(ctl) = self.replay_control() else {
            return;
        };
        let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) else {
            return;
        };
        if rp.cam_path {
            rp.cam_path = false;
            rp.path_started = None;
            rp.control.flying.store(false, Relaxed);
            rp.note = Some((
                "Camera path stopped: free flight.".to_string(),
                std::time::Instant::now(),
            ));
            return;
        }
        if rp.cam_keys.len() < 2 {
            rp.note = Some((
                "Place at least 2 keyframes (+KF) first.".to_string(),
                std::time::Instant::now(),
            ));
            return;
        }
        ctl.pov.store(false, Relaxed);
        rp.cam_path = true;
        rp.path_started = Some(std::time::Instant::now());
        rp.control.flying.store(true, Relaxed);
        let end = rp.cam_keys.last().map(|k| k.ms).unwrap_or(0);
        rp.note = Some((
            format!(
                "Flying the camera path: {} keyframes, {}.",
                rp.cam_keys.len(),
                horizon_client::replay::format_time(end)
            ),
            std::time::Instant::now(),
        ));
    }

    /// LAPSE: cycles the timelapse speeds (x16 -> x32 -> x64 -> off).
    /// With PATH on at the same time, the camera flies its keyframe
    /// spline in real time while the world races underneath: a cinematic
    /// timelapse. Shaders stay on throughout and RENDER captures it all.
    fn replay_toggle_timelapse(&mut self) {
        use std::sync::atomic::Ordering::Relaxed;
        let Some(ctl) = self.replay_control() else {
            return;
        };
        let next = match ctl.speed_pct.load(Relaxed) {
            1600 => 3200,
            3200 => 6400,
            6400 => 100,
            _ => 1600,
        };
        ctl.speed_pct.store(next, Relaxed);
        self.replay_note(match next {
            1600 => "Timelapse x16.".to_string(),
            3200 => "Timelapse x32.".to_string(),
            6400 => "Timelapse x64.".to_string(),
            _ => "Timelapse off: real time.".to_string(),
        });
    }

    /// MUS: continuous soundtrack while the replay plays, like a video's
    /// background music.
    fn replay_toggle_music(&mut self) {
        let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) else {
            return;
        };
        rp.music = !rp.music;
        rp.note = Some((
            if rp.music {
                "Music on: the soundtrack plays continuously.".to_string()
            } else {
                "Music off.".to_string()
            },
            std::time::Instant::now(),
        ));
    }

    /// LP: playback starts over when it reaches the end.
    fn replay_toggle_loop(&mut self) {
        let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) else {
            return;
        };
        rp.loop_play = !rp.loop_play;
        rp.note = Some((
            if rp.loop_play {
                "Loop on: the replay repeats.".to_string()
            } else {
                "Loop off.".to_string()
            },
            std::time::Instant::now(),
        ));
    }

    /// REC: starts or stops recording playback into an MP4 in the
    /// profile's `recordings/` folder — the same place Volt Recorder
    /// saves, so every video shows in the launcher's Captures panel and
    /// the system Files app (Volt Launcher > Recordings). The playback
    /// clock stamps the frames, so the video runs at the chosen speed
    /// regardless of render rate.
    fn replay_toggle_export(&mut self) {
        use std::sync::atomic::Ordering::Relaxed;
        // A tap while recording stops and finalizes.
        let stopping = self
            .remote
            .as_ref()
            .and_then(|r| r.replay.as_ref())
            .is_some_and(|rp| rp.export.is_some());
        if stopping {
            self.replay_finish_export();
            return;
        }
        let Some(ctl) = self.replay_control() else {
            return;
        };
        let Some(out) = self
            .remote
            .as_ref()
            .and_then(|r| r.replay.as_ref())
            .map(|rp| {
                let stem = rp
                    .path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "replay".to_string());
                // The game runs inside the profile's game folder, so this
                // is `{game}/recordings/`, exactly where Volt Recorder
                // saves (the Captures panel and Files app list it).
                let t = ctl.time_ms.load(Relaxed);
                std::path::PathBuf::from("recordings").join(format!(
                    "{stem}-{}.mp4",
                    horizon_client::replay::format_time(t).replace(':', ".")
                ))
            })
        else {
            return;
        };
        let exporter = super::video_export::Exporter::start(out);
        let Some(sender) = exporter.sender() else {
            self.replay_note("Export could not start.".to_string());
            return;
        };
        // The capture closure runs inside the renderer, right after each
        // frame is read back: it stamps frames with the playback clock.
        let control = ctl.clone();
        let mut last_t: Option<u32> = None;
        let mut pts_us: i64 = 0;
        let mut last_wall = std::time::Instant::now();
        let sink = move |size: [u32; 2], bgra: bool, pixels: Vec<[u8; 4]>| {
            let flying = control.flying.load(Relaxed);
            let t = control.time_ms.load(Relaxed);
            // A camera-path flight records on its own wall clock, so a
            // pan over a paused (or seeking) world still lands on video.
            if !flying && (control.paused.load(Relaxed) || last_t.is_some_and(|prev| t <= prev)) {
                last_wall = std::time::Instant::now();
                return;
            }
            if let Some(prev) = last_t
                && t > prev
                && !flying
            {
                let speed = control.speed_pct.load(Relaxed).clamp(1, 10_000) as i64;
                pts_us += i64::from(t - prev) * 1000 * 100 / speed;
            } else if last_t.is_some() {
                pts_us += last_wall.elapsed().as_micros().min(200_000) as i64;
            }
            last_wall = std::time::Instant::now();
            last_t = Some(t);
            let _ = sender.try_send(super::video_export::Frame {
                size,
                bgra,
                pts_us,
                pixels,
            });
        };
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.video_sink = Some(Box::new(sink));
        } else {
            self.replay_note("Export needs the renderer.".to_string());
            return;
        }
        if let Some(rp) = self.remote.as_mut().and_then(|r| r.replay.as_mut()) {
            rp.export = Some(exporter);
            rp.hud_touch = std::time::Instant::now();
        }
        self.replay_note("Recording MP4... tap REC to stop.".to_string());
    }

    /// Uninstalls the capture and finalizes the MP4; the outcome goes to
    /// the note line.
    fn replay_finish_export(&mut self) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.video_sink = None;
        }
        let note = self
            .remote
            .as_mut()
            .and_then(|r| r.replay.as_mut())
            .and_then(|rp| rp.export.take())
            .map(|mut e| e.finish());
        if let Some(note) = note {
            self.replay_note(note);
        }
    }

    /// Ends an export when playback finishes or the viewer leaves, and
    /// restarts playback at the end while LP (loop) is on.
    pub(super) fn replay_export_tick(&mut self) {
        use std::sync::atomic::Ordering::Relaxed;
        // The launcher's one-tap "MP4" button: the watch opened with the
        // export requested, so start recording as soon as playback exists.
        if self.options.replay_export
            && self.replay_control().is_some()
            && self
                .remote
                .as_ref()
                .and_then(|r| r.replay.as_ref())
                .is_some_and(|rp| rp.export.is_none())
        {
            self.options.replay_export = false;
            self.replay_toggle_export();
        }
        let ended = self
            .remote
            .as_ref()
            .and_then(|r| r.replay.as_ref())
            .is_some_and(|rp| {
                rp.export.is_some()
                    && (rp.control.finished.load(Relaxed) || rp.control.exit.load(Relaxed))
            });
        if ended {
            self.replay_finish_export();
        }
        if let Some(rp) = self.remote.as_ref().and_then(|r| r.replay.as_ref())
            && rp.loop_play
            && rp.control.finished.load(Relaxed)
            && !rp.control.exit.load(Relaxed)
        {
            // `request_seek` clears `finished`; the feeder restarts the
            // stream from the top.
            rp.control.request_seek(0);
        }
    }

    /// Handles touch/mouse press on the Volt HUD (the hub pill or window).
    pub(super) fn try_click_volt_hud(&mut self, cursor: (f32, f32), pressed: bool) -> bool {
        self.extras.volt.last_input_at = self.started.elapsed().as_secs_f64();
        // No title gate: the hub rides over every in-game screen.
        if !pressed || self.ui.hide_gui {
            return false;
        }
        self.try_click_volt_hub_overlay(cursor)
    }

    /// Evaluates `= <math>` inline calculator expressions in chat when `ChatCalculator` is ON!
    pub(super) fn try_volt_chat_calculator(&mut self, message: &str) -> bool {
        if !self.settings.extras.volt_mods.chat_calculator {
            return false;
        }
        let Some(expr) = message.trim().strip_prefix('=') else {
            return false;
        };
        if let Some(ans) = eval_chat_math(message) {
            let stacks = (ans.abs() / 64.0).floor() as i64;
            let rem = (ans.abs() as i64) % 64;
            let stack_hint = if ans.fract().abs() < 1e-6 && ans >= 64.0 {
                format!("  ({stacks} stacks + {rem})")
            } else {
                String::new()
            };
            self.system_message(
                Text::literal(format!("[Volt Calc] {} = {:.4}{stack_hint}", expr.trim(), ans))
                    .trim_end_zeros_helper()
                    .with_color(text::AQUA),
            );
            return true;
        }
        false
    }
}

trait TextCleanExt {
    fn trim_end_zeros_helper(self) -> Text;
}
impl TextCleanExt for Text {
    fn trim_end_zeros_helper(self) -> Text {
        let s = self.plain();
        if let Some((lhs, rhs)) = s.split_once(" = ") {
            let (num_part, rest) = rhs.split_once("  (").unwrap_or((rhs, ""));
            let trimmed = num_part.trim_end_matches('0').trim_end_matches('.');
            if rest.is_empty() {
                return Text::literal(format!("{lhs} = {trimmed}"));
            }
            return Text::literal(format!("{lhs} = {trimmed}  ({rest}"));
        }
        self
    }
}

fn classify_atlas_block_rgb(name: &str) -> [u8; 3] {
    if name.contains("water") || name.contains("kelp") || name.contains("seagrass") {
        [46, 108, 214]
    } else if name.contains("lava") || name.contains("magma") {
        [238, 96, 32]
    } else if name.contains("sand") || name.contains("sandstone") {
        [222, 206, 148]
    } else if name.contains("snow") || name.contains("ice") {
        [232, 244, 255]
    } else if name.contains("leaves") || name.contains("vine") || name.contains("bamboo") {
        [52, 138, 58]
    } else if name.contains("grass") || name.contains("moss") {
        [84, 168, 72]
    } else if name.contains("planks") || name.contains("log") || name.contains("wood") {
        [148, 108, 66]
    } else if name.contains("netherrack") || name.contains("crimson") {
        [136, 46, 48]
    } else if name.contains("end_stone") {
        [218, 222, 164]
    } else {
        [128, 132, 140]
    }
}

fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn recommended_tool_tag(raw_block: &str) -> &'static str {
    if raw_block.contains("log")
        || raw_block.contains("planks")
        || raw_block.contains("chest")
        || raw_block.contains("wood")
    {
        "Axe"
    } else if raw_block.contains("dirt")
        || raw_block.contains("grass_block")
        || raw_block.contains("sand")
        || raw_block.contains("gravel")
    {
        "Shovel"
    } else if raw_block.contains("leaves") || raw_block.contains("wool") {
        "Shears/Hoe"
    } else {
        "Pickaxe"
    }
}

fn estimate_armor_max_dur(item: &str) -> u32 {
    if item.contains("netherite") {
        481
    } else if item.contains("diamond") {
        429
    } else if item.contains("iron") {
        195
    } else if item.contains("golden") {
        91
    } else {
        165
    }
}

/// Returns concise enchantment max-level & target gear note (`Volt Mods` -> `Enchantment Max Guide`).
pub(super) fn enchant_guide_note(line_plain: &str) -> Option<&'static str> {
    let low = line_plain.to_lowercase();
    if low.contains("sharpness") {
        Some("  [Max V \u{2022} +Melee Damage \u{2022} Sword/Axe]")
    } else if low.contains("protection")
        && !low.contains("blast")
        && !low.contains("fire")
        && !low.contains("projectile")
    {
        Some("  [Max IV \u{2022} All-Damage Reduction \u{2022} Armor]")
    } else if low.contains("efficiency") {
        Some("  [Max V \u{2022} Mining Speed \u{2022} Tools]")
    } else if low.contains("unbreaking") {
        Some("  [Max III \u{2022} Durability Saver \u{2022} Any Gear]")
    } else if low.contains("mending") {
        Some("  [Max I \u{2022} XP Repairs Gear \u{2022} Treasure]")
    } else if low.contains("fortune") {
        Some("  [Max III \u{2022} Multiplies Ore Drops \u{2022} Tools]")
    } else if low.contains("silk touch") {
        Some("  [Max I \u{2022} Mines Exact Block \u{2022} Tools]")
    } else if low.contains("looting") {
        Some("  [Max III \u{2022} Extra Mob Drops \u{2022} Sword]")
    } else if low.contains("feather falling") {
        Some("  [Max IV \u{2022} Fall Reduction \u{2022} Boots]")
    } else if low.contains("power") {
        Some("  [Max V \u{2022} +Arrow Damage \u{2022} Bow]")
    } else {
        None
    }
}

#[cfg(test)]
mod freelook_tests {
    use super::freelook_next_active;

    #[test]
    fn hold_and_toggle_follow_key_edges_and_close_on_screen_change() {
        assert!(freelook_next_active(true, false, true, false, false));
        assert!(!freelook_next_active(true, false, false, true, true));
        assert!(freelook_next_active(true, true, true, false, false));
        assert!(freelook_next_active(true, true, true, true, true));
        assert!(freelook_next_active(true, true, false, true, true));
        assert!(!freelook_next_active(true, true, true, false, true));
        assert!(!freelook_next_active(false, true, false, true, true));
    }
}
