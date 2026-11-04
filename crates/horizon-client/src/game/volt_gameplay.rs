//! Live in-world & HUD runtime hooks for **Volt Settings 0.9.9**:
//! - **`Volt QoL` (`#2`, `#3`, `#8`, `#9`, `#15`, `#17`, `#18`, `#25`)**
//! - **`Volt Chat` (`#21` 3D UDP Proximity Voice + Chat Suite)**
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
        AtlasChunkTile, ClientTimeWeather, CrosshairStyle, JEI_RECIPES, VoltChatConfig,
        VoltVoiceRuntime, eval_chat_math, is_slime_chunk, preset_blueprint_blocks,
    },
};
use std::collections::{HashMap, VecDeque};
use winit::keyboard::KeyCode;

/// Transient per-session runtime state for Volt QoL, Voice, JEI & Mods.
pub(super) struct VoltRuntime {
    pub freelook_active: bool,
    freelook_key_was_down: bool,
    pub freelook_yaw: f32,
    pub freelook_pitch: f32,
    pub voice: VoltVoiceRuntime,
    pub voice_ptt_touch_held: bool,
    pub voice_mic_scratch: Vec<i16>,
    pub blueprint_anchor: Option<[i32; 3]>,
    pub blueprint_dim: String,
    pub atlas_scope_dim: Option<(String, String)>,
    pub atlas_scan_cursor: i32,
    pub jei_selected: usize,
    pub jei_scroll: usize,
    pub replay_ring: VecDeque<([f64; 3], f32, f32)>,
    pub last_mob_hp: HashMap<u32, f32>,
    pub damage_popups: Vec<DamagePopup>,
    pub last_hit_dist: Option<(f32, f64)>,
    pub toggle_sprint_latched: bool,
    pub last_input_at: f64,
    pub last_chat_line: Option<(String, u32)>,
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
            freelook_yaw: 0.0,
            freelook_pitch: 0.0,
            voice: VoltVoiceRuntime::default(),
            voice_ptt_touch_held: false,
            voice_mic_scratch: Vec::with_capacity(320),
            blueprint_anchor: None,
            blueprint_dim: String::new(),
            atlas_scope_dim: None,
            atlas_scan_cursor: 0,
            jei_selected: 0,
            jei_scroll: 0,
            replay_ring: VecDeque::new(),
            last_mob_hp: HashMap::new(),
            damage_popups: Vec::new(),
            last_hit_dist: None,
            toggle_sprint_latched: false,
            last_input_at: 0.0,
            last_chat_line: None,
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

    /// Returns `#2` Optic Zoom `(factor, smooth, turn_scale)` while its binding is held.
    pub(super) fn volt_qol_zoom(&self) -> Option<(f32, bool, f64)> {
        let qol = &self.settings.extras.volt_qol;
        let zoom_on =
            qol.zoom_enabled || self.settings.extras.mod_catalog.is_on(VoltModId::Zoomify);
        if !zoom_on || self.screen != Screen::Playing || self.title.is_some() {
            return None;
        }
        if !self.volt_binding_held("key.volt.zoom") {
            return None;
        }
        let pct = f32::from(qol.zoom_fov_pct.clamp(10, 80));
        let factor = (100.0 / pct).clamp(1.25, 10.0);
        Some((factor, qol.zoom_smooth, f64::from(1.0 / factor)))
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
        if self.settings.keys.get("key.volt.mod_menu") == name
            && self.settings.extras.mod_catalog.is_on(VoltModId::ModMenu)
        {
            self.open_options_at(false, horizon_client::options_ui::Page::ModMenu);
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

    /// Per-frame background tick for World Atlas, Sound Physics Lite, 3D Voice,
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
        let chat_cfg = &self.settings.extras.volt_chat;

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

        // 2. #21 3D Positional Proximity Voice Chat
        let ptt_held = self.extras.volt.voice_ptt_touch_held
            || (self.screen == Screen::Playing && self.keys.contains(&KeyCode::KeyV));
        let mic_active = chat_cfg.voice_enabled
            && self.title.is_none()
            && (!chat_cfg.voice_ptt_mode || ptt_held);
        self.audio
            .poll_voice_mic(mic_active, &mut self.extras.volt.voice_mic_scratch);
        if chat_cfg.voice_enabled && self.title.is_none() {
            let listener_pos = self.player.as_ref().map_or([0.0; 3], |p| {
                [
                    p.position.x as f32,
                    (p.position.y + 1.62) as f32,
                    p.position.z as f32,
                ]
            });
            let in_cave = self.player.as_ref().is_some_and(|p| {
                (self.world.light(
                    p.position.x.floor() as i32,
                    (p.position.y + 1.6).floor() as i32,
                    p.position.z.floor() as i32,
                ) >> 4)
                    == 0
            });
            let relay_host = self
                .server_address()
                .and_then(|s| s.split(':').next().map(str::to_owned));
            let uname = self.player_display_name().plain();
            let uid = horizon_core::random::java_string_hash(uname.encode_utf16()) as u64;
            let mic_slice = std::mem::take(&mut self.extras.volt.voice_mic_scratch);
            self.extras.volt.voice.ptt_held = ptt_held;
            self.extras.volt.voice.tick(
                chat_cfg,
                uid ^ 0x564F_4C54,
                &uname,
                listener_pos,
                self.yaw,
                in_cave,
                relay_host.as_deref(),
                &mic_slice,
                now as f32,
            );
            self.audio
                .push_voice_frames(&mut self.extras.volt.voice.mixed_pcm_out);
            self.extras.volt.voice_mic_scratch = mic_slice;
        } else {
            let off_cfg = VoltChatConfig::default();
            self.extras
                .volt
                .voice
                .tick(&off_cfg, 0, "", [0.0; 3], 0.0, false, None, &[], now as f32);
        }

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

    /// Emits hardware depth-tested 3D gizmo lines for `#3` Spawn Light-0,
    /// `#8` Blueprint 3D Hologram, `TrajectoryArc`, and `HitboxPlus`.
    pub(super) fn append_volt_world_gizmos(&self, lines: &mut Vec<Line>) {
        if self.title.is_some() || self.ui.hide_gui {
            return;
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let qol = &self.settings.extras.volt_qol;
        let mods = &self.settings.extras.volt_mods;

        // A. #3 Mob-Spawn Light-0 3D Floor Crosses
        if qol.spawn_light_overlay {
            let px = p.position.x.floor() as i32;
            let py = p.position.y.floor() as i32;
            let pz = p.position.z.floor() as i32;
            for dx in -9i32..=9 {
                for dz in -9i32..=9 {
                    if dx * dx + dz * dz > 85 {
                        continue;
                    }
                    let wx = px + dx;
                    let wz = pz + dz;
                    for dy in -3i32..=3 {
                        let wy = py + dy;
                        let ground = self.world.block(wx, wy - 1, wz);
                        let air = self.world.block(wx, wy, wz);
                        if !self.res.facts().get(ground).solid_render
                            || self.res.facts().get(air).solid_render
                        {
                            continue;
                        }
                        let light = self.world.light(wx, wy, wz);
                        let block_l = light & 15;
                        let sky_l = light >> 4;
                        if block_l == 0 {
                            let col = if sky_l == 0 {
                                0xE0FF_2A3C // Always hostile-spawnable (Red)
                            } else {
                                0xD0FF_CC22 // Night-only spawnable (Amber)
                            };
                            let y = f64::from(wy) + 0.03;
                            lines.push(Line {
                                start: [f64::from(wx) + 0.2, y, f64::from(wz) + 0.2],
                                end: [f64::from(wx) + 0.8, y, f64::from(wz) + 0.8],
                                color: col,
                                width: 2.0,
                            });
                            lines.push(Line {
                                start: [f64::from(wx) + 0.8, y, f64::from(wz) + 0.2],
                                end: [f64::from(wx) + 0.2, y, f64::from(wz) + 0.8],
                                color: col,
                                width: 2.0,
                            });
                        }
                    }
                }
            }
        }

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

    /// Draws all active Volt QoL, Volt Voice, and 50-Mod Catalog HUDs!
    pub(super) fn draw_volt_all_huds(&mut self, batch: &mut HudBatch, corners: &mut Corners) {
        if self.title.is_some() || self.ui.hide_gui {
            return;
        }
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
        let chat_cfg = &self.settings.extras.volt_chat;

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

        // 3. #21 Proximity Voice PTT Button & Active Speaker Pills
        if chat_cfg.voice_enabled {
            if chat_cfg.voice_hud_button {
                let [vx, vy, vw, vh] = Self::voice_ptt_rect(w, h);
                let active = self.extras.volt.voice.local_speaking;
                let bg = if active {
                    rgba(34, 156, 86, 225)
                } else {
                    rgba(22, 26, 36, 205)
                };
                batch.fill(atlas, vx, vy, vw, vh, bg);
                batch.fill(atlas, vx, vy, vw, 1.2, rgba(224, 52, 72, 240));
                let lbl = if active { "MIC ON" } else { "MIC [V]" };
                batch.text(
                    atlas,
                    lbl,
                    vx + 5.0,
                    vy + 5.0,
                    rgba(250, 252, 255, 255),
                    true,
                );
            }
            if !self.extras.volt.voice.peers.is_empty() {
                let n = self.extras.volt.voice.peers.len().min(4);
                let bh = 4.0 + n as f32 * 11.0;
                let (px, py) = corners.place(Corner::BottomLeft, 120.0, bh);
                batch.fill(atlas, px, py, 120.0, bh, rgba(12, 18, 24, 195));
                for (i, peer) in self.extras.volt.voice.peers.values().take(4).enumerate() {
                    batch.text(
                        atlas,
                        &format!("\u{25cf} {} ({:.0}m)", peer.name, peer.distance_m),
                        px + 5.0,
                        py + 3.0 + i as f32 * 11.0,
                        rgba(96, 245, 152, 255),
                        true,
                    );
                }
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

    pub(super) fn voice_ptt_rect(w: f32, h: f32) -> [f32; 4] {
        [(w - 58.0).max(4.0), (h * 0.52).floor(), 52.0, 18.0]
    }

    /// Handles touch/mouse press on the HUD Push-to-Talk `[MIC]` button.
    pub(super) fn try_click_volt_hud(&mut self, cursor: (f32, f32), pressed: bool) -> bool {
        self.extras.volt.last_input_at = self.started.elapsed().as_secs_f64();
        if !pressed {
            self.extras.volt.voice_ptt_touch_held = false;
            return false;
        }
        if self.title.is_some() || self.ui.hide_gui {
            return false;
        }
        let (w, h) = self.gui_size();
        if self.settings.extras.volt_chat.voice_enabled
            && self.settings.extras.volt_chat.voice_hud_button
        {
            let [vx, vy, vw, vh] = Self::voice_ptt_rect(w, h);
            if cursor.0 >= vx && cursor.0 <= vx + vw && cursor.1 >= vy && cursor.1 <= vy + vh {
                self.extras.volt.voice_ptt_touch_held = true;
                self.audio.play_button_click();
                return true;
            }
        }
        false
    }

    /// Draws `#18` **Volt JEI** Item & 3x3 Crafting Recipe Browser on the right side of `Screen::Inventory`!
    pub(super) fn draw_volt_jei_sidebar(&self, batch: &mut HudBatch) {
        if !self.settings.extras.volt_qol.jei_enabled || self.screen != Screen::Inventory {
            return;
        }
        let (w, h) = (batch.width, batch.height);
        let panel_w = 118.0;
        if w < 312.0 {
            return;
        }
        let px = w - panel_w - 4.0;
        let py = 8.0;
        let ph = (h - 16.0).max(140.0);
        let atlas = &self.gui.atlas;

        batch.fill(atlas, px, py, panel_w, ph, rgba(14, 16, 24, 228));
        batch.fill(atlas, px, py, panel_w, 1.5, rgba(224, 48, 68, 245));
        batch.text(
            atlas,
            "Volt JEI Recipes",
            px + 6.0,
            py + 4.0,
            rgba(255, 220, 130, 255),
            true,
        );

        let sel = self.extras.volt.jei_selected.min(JEI_RECIPES.len() - 1);
        let rec = &JEI_RECIPES[sel];
        // Selected 3x3 crafting recipe preview card at top
        let card_y = py + 15.0;
        batch.fill(atlas, px + 4.0, card_y, panel_w - 8.0, 56.0, rgba(22, 26, 38, 235));
        batch.text(
            atlas,
            &format!("{} (x{})", rec.title, rec.count),
            px + 7.0,
            card_y + 3.0,
            rgba(120, 240, 185, 255),
            true,
        );
        for row in 0usize..3 {
            for col in 0usize..3 {
                let cell = rec.grid[row * 3 + col];
                let gx = px + 8.0 + col as f32 * 33.0;
                let gy = card_y + 14.0 + row as f32 * 13.0;
                batch.fill(atlas, gx, gy, 31.0, 11.5, rgba(34, 38, 54, 245));
                if !cell.is_empty() {
                    let short: String = cell.chars().take(5).collect();
                    batch.text(
                        atlas,
                        &short,
                        gx + 2.0,
                        gy + 2.0,
                        rgba(240, 242, 252, 255),
                        true,
                    );
                }
            }
        }

        // Scrollable recipe catalog list below
        let list_y = card_y + 60.0;
        let max_rows = ((ph - 80.0) / 13.0).max(3.0) as usize;
        let start = self
            .extras
            .volt
            .jei_scroll
            .min(JEI_RECIPES.len().saturating_sub(max_rows));
        for (vis_i, idx) in (start..(start + max_rows).min(JEI_RECIPES.len())).enumerate() {
            let ry = list_y + vis_i as f32 * 13.0;
            let active = idx == sel;
            batch.fill(
                atlas,
                px + 4.0,
                ry,
                panel_w - 8.0,
                12.0,
                if active {
                    rgba(186, 40, 62, 225)
                } else {
                    rgba(24, 28, 40, 200)
                },
            );
            batch.text(
                atlas,
                JEI_RECIPES[idx].title,
                px + 7.0,
                ry + 2.0,
                rgba(245, 247, 255, 255),
                true,
            );
        }
    }

    /// Handles clicks on the `#18` **Volt JEI** sidebar on `Screen::Inventory`.
    pub(super) fn try_click_volt_jei(&mut self, cursor: (f32, f32)) -> bool {
        if !self.settings.extras.volt_qol.jei_enabled || self.screen != Screen::Inventory {
            return false;
        }
        let (w, h) = self.gui_size();
        let panel_w = 118.0;
        if w < 312.0 {
            return false;
        }
        let px = w - panel_w - 4.0;
        let py = 8.0;
        let ph = (h - 16.0).max(140.0);
        if cursor.0 < px || cursor.0 > px + panel_w || cursor.1 < py || cursor.1 > py + ph {
            return false;
        }
        let list_y = py + 75.0;
        let max_rows = ((ph - 80.0) / 13.0).max(3.0) as usize;
        let start = self
            .extras
            .volt
            .jei_scroll
            .min(JEI_RECIPES.len().saturating_sub(max_rows));
        if cursor.1 >= list_y {
            let row = ((cursor.1 - list_y) / 13.0).floor() as usize;
            if start + row < JEI_RECIPES.len() {
                self.extras.volt.jei_selected = start + row;
                self.audio.play_button_click();
            }
        } else {
            self.extras.volt.jei_scroll = (self.extras.volt.jei_scroll + 4) % JEI_RECIPES.len();
            self.audio.play_button_click();
        }
        true
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
