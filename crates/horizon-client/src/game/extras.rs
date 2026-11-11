//! Volt's gameplay extras in the game loop:
//! - **Volt Minimap** ("Minimal with many features": terrain/cave slicing,
//!   chunk grid, entity radar, waypoint pins, and live XYZ/Biome/Clock pills)
//! - **Volt Waypoints** (3D world markers, beacon beams, auto-death pins, 8x
//!   Nether sync, screen-edge chevrons, quick-add hotkey & touch HUD button)
//! Plus mods' minimap/zoom/click/FPS hooks. Every Volt extra does nothing
//! while its master switch is off.
use super::{Dim, Game, Screen, native_hooks::NativeMinimap};
use horizon_client::{
    clipboard::Clipboard,
    extras::{Corner, Cps, FpsCounter, MINIMAP_ZOOMS, fov_target},
    extras_hud::{self, Corners, MinimapPin, RadarDot, WorldWaypointDraw},
    hud::{HudBatch, MINIMAP_SPRITE},
    keybinds,
    minimap::{self, Layer, Minimap, View},
    multiplayer,
    reconnect::{Retry, Tracker},
    text::{self, Text},
    title::Mode,
    volt_features::{
        MinimapEntities, MinimapTerrainMode, RadarKind, classify_mob_id,
        effective_waypoint_coords, format_coords, format_heading_and_biome, format_world_clock,
        project_waypoint, world_to_minimap_offset,
    },
};
use std::time::Duration;
use winit::{event::MouseButton, keyboard::KeyCode};

pub(super) struct ExtrasState {
    pub(super) fps: FpsCounter,
    /// Left and right mouse buttons: held, and their clicks.
    pub(super) mouse: [bool; 2],
    pub(super) cps: [Cps; 2],
    minimap: Minimap,
    /// The dimension and terrain mode the minimap's tiles belong to.
    minimap_dim: Option<(Dim, MinimapTerrainMode)>,
    /// Auto-reconnect's attempts at the current server.
    reconnect: Tracker,
    /// Prevents recording duplicate death waypoints during a single death screen.
    pub(super) recorded_current_death: bool,
    /// Volt Boost rolling 48-frame times (ms) for Perf HUD sparkline, Auto-DRS & Thermal Guard.
    pub(super) frame_ms_ring: [f32; 48],
    pub(super) frame_ms_head: usize,
    pub(super) ema_frame_ms: f32,
    pub(super) drs_scale: f32,
    pub(super) last_drs_apply_at: f64,
    pub(super) cached_rss_mib: (f64, u64),
    pub(super) last_mem_trim_at: f64,
    pub(super) drawn_entities: std::sync::atomic::AtomicU32,
    pub(super) volt: super::volt_gameplay::VoltRuntime,
}

impl Default for ExtrasState {
    fn default() -> Self {
        Self {
            fps: FpsCounter::default(),
            mouse: [false; 2],
            cps: Default::default(),
            minimap: Minimap::default(),
            minimap_dim: None,
            reconnect: Tracker::default(),
            recorded_current_death: false,
            frame_ms_ring: [0.0; 48],
            frame_ms_head: 0,
            ema_frame_ms: 16.0,
            drs_scale: 1.0,
            last_drs_apply_at: 0.0,
            cached_rss_mib: (-10.0, 0),
            last_mem_trim_at: 0.0,
            drawn_entities: std::sync::atomic::AtomicU32::new(0),
            volt: Default::default(),
        }
    }
}

impl ExtrasState {
    pub(super) fn effective_drs_scale(&self) -> f32 {
        self.drs_scale.clamp(0.50, 1.0)
    }
}

/// CPU time the minimap's tiles may take per frame.
const MINIMAP_BUDGET: Duration = Duration::from_micros(300);

impl Game<'_> {
    fn extras_now(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }
    /// Mouse buttons while playing, for mods' clicks and touch `[+WP]` pin button.
    pub(super) fn extras_mouse(&mut self, button: MouseButton, pressed: bool) {
        self.native_mouse_event(button, pressed);
        let i = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            _ => return,
        };
        self.extras.mouse[i] = pressed;
        if pressed {
            let now = self.extras_now();
            self.extras.cps[i].click(now);
        }
    }
    /// Once per frame: the frame's time, and keys let go behind screens.
    pub(super) fn extras_frame(&mut self, elapsed: Duration) {
        // A held virtual joystick or camera drag is input even on frames
        // without a new touch event: never let the AFK limiters engage
        // under a resting finger.
        #[cfg(target_os = "android")]
        if self.pad.touch.any_finger_down() {
            self.last_input = std::time::Instant::now();
        }
        let focused = self.focused || self.control.is_some();
        let playing = self.screen == Screen::Playing && self.title.is_none() && focused;
        if !playing {
            self.extras.mouse = [false; 2];
        }
        if self.screen != Screen::Dead
            && self
                .player
                .as_ref()
                .is_some_and(|p| !p.health.dead_or_dying())
        {
            self.extras.recorded_current_death = false;
        }
        self.update_minimap();
        let now = self.extras_now();
        let ms = (elapsed.as_secs_f32() * 1000.0).clamp(0.1, 250.0);
        self.extras.fps.record(now, ms);
        let head = self.extras.frame_ms_head;
        self.extras.frame_ms_ring[head] = ms;
        self.extras.frame_ms_head = (head + 1) % 48;
        self.extras.ema_frame_ms = self.extras.ema_frame_ms * 0.90 + ms * 0.10;

        if now - self.extras.cached_rss_mib.0 >= 1.0
            && (self.settings.extras.perf_hud != horizon_client::volt_features::PerfHudMode::Off
                || self.settings.extras.memory_saver)
        {
            self.extras.cached_rss_mib = (now, horizon_threads::resident_ram_mib().unwrap_or(0));
        }

        if self.settings.extras.memory_saver && now - self.extras.last_mem_trim_at >= 30.0 {
            self.extras.last_mem_trim_at = now;
            self.particles.trim_memory();
            self.pending_uploads.shrink_to_fit();
            horizon_threads::release_free_memory();
        }
        if self.settings.extras.purge_ram_requested {
            self.settings.extras.purge_ram_requested = false;
            self.particles.trim_memory();
            self.pending_uploads.shrink_to_fit();
            horizon_threads::release_free_memory();
            let rss = horizon_threads::resident_ram_mib().unwrap_or(0);
            self.extras.cached_rss_mib = (now, rss);
            self.titles.set_overlay(
                Text::literal(format!("Volt Boost: Purged unused RAM ({rss} MiB resident)"))
                    .with_color(text::GREEN),
                false,
            );
        }

        // Auto-DRS & Mobile Thermal Guard dynamic 3D resolution controller
        let drs_cfg = self.settings.extras.auto_drs.target_and_floor();
        let thermal_on = self.settings.extras.thermal_guard;
        let desired_drs = if playing && (drs_cfg.is_some() || thermal_on) {
            let (target_fps, floor) = drs_cfg.unwrap_or((60.0, 0.72));
            let target_ms = 1000.0 / target_fps;
            let mut s = self.extras.drs_scale;
            if self.extras.ema_frame_ms > target_ms * 1.12 {
                s = (s - 0.04).max(floor);
            } else if thermal_on && self.extras.ema_frame_ms > 24.0 {
                s = (s - 0.05).max(0.70);
            } else if self.extras.ema_frame_ms < target_ms * 0.80 {
                s = (s + 0.02).min(1.0);
            }
            s
        } else {
            1.0
        };
        if (desired_drs - self.extras.drs_scale).abs() >= 0.035
            && now - self.extras.last_drs_apply_at >= 0.45
        {
            self.extras.drs_scale = desired_drs;
            self.extras.last_drs_apply_at = now;
            if let Some(r) = self.renderer.as_mut() {
                let base = self.settings.horizon.resolution_scale as f32 / 100.0;
                r.set_world_post(
                    (base * desired_drs).clamp(0.25, 2.0),
                    self.settings.horizon.fxaa,
                    self.settings.horizon.taa,
                    self.settings.horizon.upscaling.ratio(),
                );
            }
        }
        if thermal_on && playing && self.extras.ema_frame_ms > 28.0 {
            self.particles.max_particles = self.particles.max_particles.min(1024);
        } else {
            self.particles.max_particles = self.settings.extras.particle_budget.max_particles();
        }
        self.volt_builtins_frame(elapsed);
        self.native_frame(elapsed);
    }
    /// The frame counter for mods: fps, min, 1% low.
    pub(super) fn frame_reading(&self) -> (u32, u32, u32) {
        let r = self.extras.fps.reading();
        (r.fps, r.min, r.low)
    }
    /// Attack and use clicks in the last second, and whether each is held.
    pub(super) fn mouse_reading(&mut self) -> ((u32, u32), [bool; 2]) {
        let now = self.extras_now();
        let [l, r] = &mut self.extras.cps;
        let cps = (l.per_second(now) as u32, r.per_second(now) as u32);
        (cps, self.extras.mouse)
    }
    /// Auto-reconnect: a connection to `address` as `name` starts.
    pub(super) fn extras_connecting(&mut self, address: &str, name: &str) {
        self.extras.reconnect.connecting(address, name);
    }
    /// The server this game connected to last.
    pub(super) fn server_address(&self) -> Option<&str> {
        self.extras.reconnect.address()
    }
    /// Auto-reconnect: the server let the player in.
    pub(super) fn extras_joined(&mut self) {
        self.extras.reconnect.joined(std::time::Instant::now());
    }
    /// Auto-reconnect: the player left the server on purpose.
    pub(super) fn extras_left(&mut self) {
        self.extras.reconnect.reset();
    }
    /// Auto-reconnect's next attempt after a disconnect, when it is on and
    /// the reason allows one.
    pub(super) fn reconnect_after(&mut self, reason: &str, from_server: bool) -> Option<Retry> {
        if !self.settings.extras.auto_reconnect {
            self.extras.reconnect.reset();
            return None;
        }
        let now = std::time::Instant::now();
        self.extras.reconnect.ended(reason, from_server, now)
    }
    /// Show the attempt's countdown on the Disconnected screen.
    pub(super) fn offer_reconnect(&mut self, retry: Option<Retry>) {
        let Some(retry) = retry else {
            return;
        };
        if let Some(title) = self.title.as_mut()
            && let Mode::Multiplayer(multiplayer::Screen::Disconnected { retry: slot, .. }) =
                &mut title.mode
        {
            log::info!(
                "Volt: reconnecting to {} in {} s",
                horizon_client::net::shown_address(&retry.address),
                retry.seconds_left(std::time::Instant::now())
            );
            *slot = Some(retry);
        }
    }
    pub(super) fn extras_retry(&mut self) -> horizon_client::Result<()> {
        if let Some(retry) = self.title.as_mut().and_then(|t| t.due_retry()) {
            self.connect_server(&retry.address, &retry.name)?;
        }
        Ok(())
    }

    // ========================================================================
    // Volt Waypoints
    // ========================================================================

    pub(super) fn current_world_scope(&self) -> Option<String> {
        if self.title.is_some() {
            return None;
        }
        if self.remote.is_some() {
            let addr = self
                .server_address()
                .map(horizon_client::net::shown_address)
                .unwrap_or_else(|| "server".to_owned());
            Some(format!("mp:{addr}"))
        } else {
            let folder = self
                .save
                .root
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "world".to_owned());
            Some(format!("sp:{folder}"))
        }
    }

    pub(super) fn current_waypoint_context(&self) -> Option<(String, String, [i32; 3])> {
        let scope = self.current_world_scope()?;
        let p = self.player.as_ref()?;
        let pos = [
            p.position.x.floor() as i32,
            p.position.y.floor() as i32,
            p.position.z.floor() as i32,
        ];
        Some((scope, self.dim.id().to_owned(), pos))
    }

    pub(super) fn sync_waypoint_ui_context(&mut self) {
        self.ui.options.waypoint_context = self.current_waypoint_context();
    }

    /// Drops a Crimson `"Last Death"` waypoint on death when Waypoints + Death
    /// Waypoints are enabled.
    pub(super) fn record_death_waypoint(&mut self) {
        if self.extras.recorded_current_death
            || !self.settings.extras.waypoints
            || !self.settings.extras.waypoint_auto_death
        {
            return;
        }
        let Some((scope, dim, pos)) = self.current_waypoint_context() else {
            return;
        };
        self.extras.recorded_current_death = true;
        self.settings.volt_waypoints.record_death(&scope, &dim, pos);
        self.settings.save(&Self::options_path());
        self.system_message(
            Text::literal(format!(
                "[Volt] Saved death waypoint at {}, {}, {}",
                pos[0], pos[1], pos[2]
            ))
            .with_color(text::RED),
        );
    }

    pub(super) fn quick_add_waypoint_here(&mut self) {
        let Some((scope, dim, pos)) = self.current_waypoint_context() else {
            return;
        };
        let color = self.settings.extras.waypoint_new_color;
        let id = self.settings.volt_waypoints.add(&scope, "", &dim, pos, color);
        let name = self
            .settings
            .volt_waypoints
            .entries
            .iter()
            .find(|w| w.id == id)
            .map_or_else(|| "Waypoint".to_owned(), |w| w.name.clone());
        self.settings.save(&Self::options_path());
        self.titles.set_overlay(
            Text::literal(format!(
                "Pinned {name} ({}, {}, {})",
                pos[0], pos[1], pos[2]
            ))
            .with_color(text::GREEN),
            false,
        );
    }

    /// Handles a tap on the HUD `[+WP]` button when shown; returns true if consumed.
    pub(super) fn try_click_waypoint_hud_button(&mut self, cursor: (f32, f32)) -> bool {
        if !self.settings.extras.waypoints
            || !self.settings.extras.waypoint_hud_button
            || self.title.is_some()
            || self.ui.hide_gui
        {
            return false;
        }
        let (w, h) = self.gui_size();
        let [rx, ry, rw, rh] = extras_hud::waypoint_hud_button_rect(w, h);
        if cursor.0 >= rx && cursor.0 < rx + rw && cursor.1 >= ry && cursor.1 < ry + rh {
            self.audio.play_button_click();
            self.quick_add_waypoint_here();
            true
        } else {
            false
        }
    }

    pub(super) fn execute_waypoint_action(&mut self, idx: usize) {
        let Some(wp) = self.settings.volt_waypoints.entries.get(idx).cloned() else {
            return;
        };
        let tp_cmd = format!("/tp @s {} {} {}", wp.x, wp.y, wp.z);
        self.clipboard.set(&tp_cmd);
        let same_dim = self.title.is_none() && wp.dimension == self.dim.id();
        let can_tp = same_dim
            && (if self.remote.is_some() {
                self.can_use_game_master_blocks()
            } else {
                self.allow_commands() || self.creative
            });
        if can_tp {
            self.screen = Screen::Playing;
            self.capture(true);
            self.handle_chat_input(&tp_cmd);
            self.titles.set_overlay(
                Text::literal(format!(
                    "Teleported to {} ({}, {}, {})",
                    wp.name, wp.x, wp.y, wp.z
                ))
                .with_color(text::AQUA),
                false,
            );
        } else {
            self.system_message(
                Text::literal(format!(
                    "[Volt] Copied '{}' ({}, {}, {}) to clipboard",
                    wp.name, wp.x, wp.y, wp.z
                ))
                .with_color(text::YELLOW),
            );
        }
    }

    // ========================================================================
    // Volt Hotkeys (Waypoints & Minimap Zoom)
    // ========================================================================

    pub(super) fn volt_key_press(&mut self, key: KeyCode) -> bool {
        let Some(name) = keybinds::key_name(key) else {
            return false;
        };
        self.volt_input(name)
    }

    pub(super) fn volt_mouse_press(&mut self, button: MouseButton) -> bool {
        self.volt_input(&keybinds::mouse_name(button))
    }

    fn volt_input(&mut self, name: &str) -> bool {
        if self.screen != Screen::Playing || self.title.is_some() || name == "key.keyboard.unknown"
        {
            return false;
        }
        if self.settings.extras.waypoints && self.settings.extras.waypoint_key == name {
            self.quick_add_waypoint_here();
            return true;
        }
        if self.settings.extras.minimap && self.settings.extras.minimap_zoom_key == name {
            let cur = self.settings.extras.minimap_zoom;
            let i = MINIMAP_ZOOMS.iter().position(|&z| z == cur).unwrap_or(0);
            let next = MINIMAP_ZOOMS[(i + 1) % MINIMAP_ZOOMS.len()];
            self.settings.extras.minimap_zoom = next;
            self.settings.save(&Self::options_path());
            self.titles.set_overlay(
                Text::literal(format!("Minimap Zoom: {next}x")).with_color(text::AQUA),
                false,
            );
            return true;
        }
        false
    }

    // ========================================================================
    // Volt Minimap & Overlays
    // ========================================================================

    fn minimap_shown(&self) -> Option<(Corner, f32, View)> {
        let p = self.player.as_ref()?;
        if self.title.is_some() || self.ui.hide_gui {
            return None;
        }
        let m = self.minimap_wanted()?;
        let view = View {
            x: p.position.x,
            z: p.position.z,
            yaw: self.yaw,
            rotate: m.rotate,
            blocks_per_pixel: m.blocks_per_pixel,
            round: m.round,
        };
        Some((m.corner, m.size, view))
    }

    /// Priority: native mod -> WASM mod -> built-in Volt Minimap (`extras.minimap`).
    fn minimap_wanted(&self) -> Option<NativeMinimap> {
        if let Some(m) = self.native_minimap_now() {
            return Some(m);
        }
        if let Some(m) = self.mod_minimap() {
            return Some(NativeMinimap {
                corner: m.corner,
                size: m.size,
                blocks_per_pixel: m.blocks_per_pixel,
                rotate: m.rotate,
                round: m.round,
            });
        }
        let e = &self.settings.extras;
        if e.minimap {
            return Some(NativeMinimap {
                corner: e.minimap_corner,
                size: f32::from(e.minimap_size),
                blocks_per_pixel: e.minimap_zoom,
                rotate: e.minimap_rotate,
                round: e.minimap_round,
            });
        }
        None
    }

    fn update_minimap(&mut self) {
        let Some((_, _, view)) = self.minimap_shown() else {
            if self.minimap_wanted().is_none() || self.player.is_none() {
                self.extras.minimap.clear();
                self.extras.minimap_dim = None;
            }
            return;
        };
        let terrain_mode = self.settings.extras.minimap_terrain_mode;
        if self.extras.minimap_dim != Some((self.dim, terrain_mode)) {
            self.extras.minimap.clear();
            self.extras.minimap_dim = Some((self.dim, terrain_mode));
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let head = (p.position.y + f64::from(p.dimensions.eye_height)).floor() as i32;
        let (bx, bz) = (view.x.floor() as i32, view.z.floor() as i32);
        let covered = self.world.light(bx, head, bz) >> 4 == 0;
        let layer = match terrain_mode {
            MinimapTerrainMode::Surface if self.dim != Dim::Nether => Layer::Surface,
            MinimapTerrainMode::Cave => Layer::Below(head),
            MinimapTerrainMode::Auto | MinimapTerrainMode::Surface => {
                if self.dim == Dim::Nether || covered {
                    Layer::Below(head)
                } else {
                    Layer::Surface
                }
            }
        };
        let reach = minimap::SIZE as f32 / 2.0 * view.blocks_per_pixel * std::f32::consts::SQRT_2;
        let radius = ((reach / 16.0).ceil() as i32 + 1).min(self.settings.render_distance.max(2));
        self.extras.minimap.update(
            &self.world,
            self.res.facts(),
            (bx >> 4, bz >> 4),
            radius,
            layer,
            MINIMAP_BUDGET,
        );
        if !self.extras.minimap.compose(view) {
            return;
        }
        let Some(rect) = self.gui.atlas.sprites.get(MINIMAP_SPRITE).copied() else {
            return;
        };
        let (w, h) = (self.gui.atlas.width as f32, self.gui.atlas.height as f32);
        self.map_patches.push(horizon_client::font::Patch {
            x: (rect.u0 * w).round() as u32,
            y: (rect.v0 * h).round() as u32,
            width: minimap::SIZE as u32,
            height: minimap::SIZE as u32,
            rgba: self.extras.minimap.image.clone(),
        });
    }

    fn collect_radar_dots(&self, view: &View, map_size: f32) -> Vec<RadarDot> {
        let filter = self.settings.extras.minimap_entities;
        if filter == MinimapEntities::Off || !self.settings.extras.minimap {
            return Vec::new();
        }
        let mut dots = Vec::new();
        if filter == MinimapEntities::All {
            for item in self.dropped.iter().take(64) {
                if let Some((dx, dy, _)) = world_to_minimap_offset(
                    item.position.x,
                    item.position.z,
                    view,
                    map_size,
                    false,
                ) {
                    dots.push(RadarDot {
                        dx,
                        dy,
                        rgb: RadarKind::Item.rgb(),
                        radius: 1.1,
                    });
                }
            }
        }
        for mob in &self.mobs {
            if mob.death_time > 0 || mob.is_invisible() {
                continue;
            }
            let kind = if self
                .remote
                .as_ref()
                .is_some_and(|r| r.level_players.contains_key(&(mob.id as i32)))
            {
                RadarKind::Player
            } else if mob.kind.category == "monster" {
                RadarKind::Hostile
            } else {
                classify_mob_id(&mob.kind.id)
            };
            if !kind.shown_by(filter) {
                continue;
            }
            if let Some((dx, dy, _)) =
                world_to_minimap_offset(mob.position.x, mob.position.z, view, map_size, false)
            {
                dots.push(RadarDot {
                    dx,
                    dy,
                    rgb: kind.rgb(),
                    radius: if kind == RadarKind::Player { 1.9 } else { 1.4 },
                });
            }
        }
        if let Some(host) = &self.lan {
            for guest in &host.hub.guests {
                if guest.dead || !host.hub.here(guest) {
                    continue;
                }
                if let Some((dx, dy, _)) = world_to_minimap_offset(
                    guest.position[0],
                    guest.position[2],
                    view,
                    map_size,
                    false,
                ) {
                    dots.push(RadarDot {
                        dx,
                        dy,
                        rgb: RadarKind::Player.rgb(),
                        radius: 1.9,
                    });
                }
            }
        }
        dots
    }

    fn collect_minimap_pins(&self, view: &View, map_size: f32) -> Vec<MinimapPin> {
        let e = &self.settings.extras;
        if !e.waypoint_on_minimap || (!e.waypoints && !e.minimap) {
            return Vec::new();
        }
        let Some(scope) = self.current_world_scope() else {
            return Vec::new();
        };
        let cur_dim = self.dim.id();
        let mut pins = Vec::new();
        for idx in self.settings.volt_waypoints.scoped_indices(&scope) {
            let wp = &self.settings.volt_waypoints.entries[idx];
            if !wp.enabled {
                continue;
            }
            let Some((coords, _)) =
                effective_waypoint_coords(wp, cur_dim, e.waypoint_dimension_sync)
            else {
                continue;
            };
            if let Some((dx, dy, clamped)) =
                world_to_minimap_offset(coords[0], coords[2], view, map_size, true)
            {
                pins.push(MinimapPin {
                    dx,
                    dy,
                    rgb: wp.color.rgb(),
                    clamped,
                });
            }
        }
        pins
    }

    fn collect_minimap_info_lines(&self) -> Vec<String> {
        let e = &self.settings.extras;
        if !e.minimap {
            return Vec::new();
        }
        let Some(p) = self.player.as_ref() else {
            return Vec::new();
        };
        let pos = [p.position.x, p.position.y, p.position.z];
        let mut lines = Vec::with_capacity(3);
        if e.minimap_coords {
            lines.push(format_coords(pos));
        }
        if e.minimap_biome {
            let (bx, by, bz) = (
                pos[0].floor() as i32,
                pos[1].floor() as i32,
                pos[2].floor() as i32,
            );
            let biome = self
                .world
                .chunks
                .get(&(bx >> 4, bz >> 4))
                .map(|chunk| self.res.biomes().get(chunk.biome(bx, by, bz)).name.as_str());
            lines.push(format_heading_and_biome(self.yaw, biome));
        }
        if e.minimap_time {
            lines.push(format_world_clock(self.world.day_time));
        }
        lines
    }

    fn collect_world_waypoints(&self, screen_w: f32, screen_h: f32) -> Vec<WorldWaypointDraw> {
        let e = &self.settings.extras;
        if !e.waypoints || self.title.is_some() {
            return Vec::new();
        }
        let Some(scope) = self.current_world_scope() else {
            return Vec::new();
        };
        let Some((cam_pos, cam_yaw, cam_pitch, cam_fov)) = self.locator_camera else {
            return Vec::new();
        };
        let cur_dim = self.dim.id();
        let scale = f32::from(e.waypoint_scale) / 100.0;
        let max_dist = f64::from(e.waypoint_max_distance);
        let mut out = Vec::new();
        for idx in self.settings.volt_waypoints.scoped_indices(&scope) {
            let wp = &self.settings.volt_waypoints.entries[idx];
            if !wp.enabled {
                continue;
            }
            let Some((coords, dim_tag)) =
                effective_waypoint_coords(wp, cur_dim, e.waypoint_dimension_sync)
            else {
                continue;
            };
            let projected = project_waypoint(
                cam_pos,
                cam_yaw,
                cam_pitch,
                cam_fov,
                screen_w,
                screen_h,
                coords,
            );
            if max_dist > 0.0 && projected.distance > max_dist {
                continue;
            }
            let mut title = wp.name.clone();
            if let Some(tag) = dim_tag {
                title.push_str(&format!(" [{tag}]"));
            }
            if e.waypoint_distance {
                title.push_str(&format!(" ({:.0}m)", projected.distance));
            }
            let focused = projected.center_focus >= 0.992;
            let subtitle = if e.waypoint_coords || focused {
                Some(format!(
                    "{}, {}, {}",
                    coords[0].floor() as i32,
                    coords[1].floor() as i32,
                    coords[2].floor() as i32
                ))
            } else {
                None
            };
            out.push(WorldWaypointDraw {
                projected,
                rgb: wp.color.rgb(),
                title,
                subtitle,
                show_beam: e.waypoint_beams,
                show_label: e.waypoint_labels,
                show_edge_arrow: e.waypoint_edge_arrows,
                scale: if focused { scale * 1.12 } else { scale },
            });
        }
        // Farther waypoints first so closer markers draw cleanly on top.
        out.sort_by(|a, b| b.projected.distance.total_cmp(&a.projected.distance));
        out
    }

    fn zoom_now(&self) -> (f32, bool, f64) {
        if let Some(z) = self.volt_qol_zoom() {
            return z;
        }
        if let Some(z) = self.native_zoom_now() {
            return (z.factor, z.smooth, f64::from(z.turn));
        }
        self.mod_zoom().map_or((1.0, true, 1.0), |z| {
            (z.factor, z.smooth, f64::from(z.turn))
        })
    }
    pub(super) fn zoomed_fov_target(&self, vanilla: f32) -> f32 {
        // The native optic zoom is applied after the vanilla camera's 0.1
        // modifier clamp, so high zoom factors remain visible.
        if self.volt_zoom_fov_scale().is_some() {
            return vanilla;
        }
        match self.zoom_now() {
            (factor, true, _) => fov_target(vanilla, factor),
            _ => vanilla,
        }
    }
    pub(super) fn zoomed_fov_degrees(&self, fov: f32) -> f32 {
        if let Some(scale) = self.volt_zoom_fov_scale() {
            return fov * scale;
        }
        match self.zoom_now() {
            (factor, false, _) => fov / factor,
            _ => fov,
        }
    }
    pub(super) fn zoom_turn_scale(&self) -> f64 {
        self.zoom_now().2
    }

    /// Renders Volt 3D waypoints, touch `[+WP]` pin button, Volt Minimap,
    /// and mod corner panels.
    pub(super) fn draw_extras(&mut self, batch: &mut HudBatch) {
        // 1. In-world 3D Waypoints (beams, diamonds, distance/XYZ labels, rim chevrons):
        let world_wps = self.collect_world_waypoints(batch.width, batch.height);
        if !world_wps.is_empty() {
            extras_hud::draw_world_waypoints(batch, &self.gui.atlas, &world_wps);
        }

        // 2. Optional touch-friendly [+WP] HUD quick-pin button:
        if self.settings.extras.waypoints
            && self.settings.extras.waypoint_hud_button
            && self.title.is_none()
        {
            let rect = extras_hud::waypoint_hud_button_rect(batch.width, batch.height);
            extras_hud::draw_waypoint_hud_button(batch, &self.gui.atlas, rect);
        }

        // 3. Minimap & corner panels:
        let mod_panels = self.mods.has_panels();
        let map = self.minimap_shown();
        let visible = |corner| !(self.debug && corner == Corner::TopLeft);
        let mut corners = Corners::new(batch.width, batch.height, self.effect_rows_height());
        reserve_android_inset(&mut corners);
        if self.settings.extras.perf_hud != horizon_client::volt_features::PerfHudMode::Off
            && visible(self.settings.extras.perf_hud_corner)
        {
            let reading = self.extras.fps.reading();
            let base_pct = self.settings.horizon.resolution_scale as f32;
            let scale_pct = (base_pct * self.extras.effective_drs_scale()).round() as u32;
            let total_ent = (self.mobs.len()
                + self.dropped.len()
                + self.armor_stands.len()
                + self.orbs.len()) as u32;
            let (cap, mut cap_reason) = self.framerate_limit_reasoned();
            // VSync caps at the panel's refresh outside this chain entirely:
            // name it whenever it is the strictest limiter in sight.
            if self.settings.vsync && cap_reason == "-" {
                cap_reason = "VSYNC";
            }
            let perf_data = extras_hud::VoltPerfHudData {
                full: self.settings.extras.perf_hud
                    == horizon_client::volt_features::PerfHudMode::Full,
                corner: self.settings.extras.perf_hud_corner,
                fps: reading.fps,
                low_1pct: reading.low,
                frame_ms: self.extras.ema_frame_ms,
                ram_mib: self.extras.cached_rss_mib.1,
                scale_pct,
                entities_drawn: self
                    .extras
                    .drawn_entities
                    .load(std::sync::atomic::Ordering::Relaxed),
                entities_total: total_ent,
                particles: self.particles.active_count() as u32,
                chunks: self.world.chunks.len() as u32,
                active_boosts: self.settings.extras.active_volt_optimizations(),
                samples_ms: &self.extras.frame_ms_ring,
                sample_head: self.extras.frame_ms_head,
                cap,
                cap_reason,
                idle_s: self.last_input.elapsed().as_secs() as u32,
            };
            extras_hud::draw_volt_perf_hud(batch, &self.gui.atlas, &mut corners, &perf_data);
        }
        if let Some((map_corner, map_size, view)) = map
            && visible(map_corner)
            && !self.extras.minimap.image.is_empty()
            && let Some(image) = self.gui.atlas.sprites.get(MINIMAP_SPRITE).copied()
        {
            let (arrow, north) = if view.rotate {
                (0.0, Some(view.north()))
            } else {
                (180.0 + view.yaw, None)
            };
            let volt_map_on = self.settings.extras.minimap;
            let radar_dots = self.collect_radar_dots(&view, map_size);
            let waypoint_pins = self.collect_minimap_pins(&view, map_size);
            let info_lines = self.collect_minimap_info_lines();
            let e = &self.settings.extras;
            let look = extras_hud::MinimapLook {
                corner: map_corner,
                size: map_size,
                image,
                arrow,
                north,
                round: view.round,
                frame: if volt_map_on {
                    e.minimap_frame
                } else {
                    horizon_client::volt_features::MinimapFrame::CleanMinimal
                },
                opacity: if volt_map_on { e.minimap_opacity } else { 100 },
                chunk_grid: volt_map_on && e.minimap_chunk_grid,
                blocks_per_pixel: view.blocks_per_pixel,
                player_world_xz: (view.x, view.z),
                rotate_yaw: view.rotate.then_some(view.yaw),
                radar_dots,
                waypoint_pins,
                info_lines,
            };
            extras_hud::draw_minimap(batch, &self.gui.atlas, &mut corners, &look);
        }
        if mod_panels {
            self.draw_mod_panels(batch, &mut corners);
        }
        self.native_hud(batch, &mut corners);
        self.bundled_hud_draw(batch, &mut corners);
        self.draw_volt_all_huds(batch, &mut corners);
    }

    fn effect_rows_height(&self) -> f32 {
        let effects = &self.potions.effects.list;
        if effects.is_empty() || self.screen != Screen::Playing {
            return 0.0;
        }
        let harmful = effects.iter().any(|e| {
            horizon_client::brewing::effect(e.id)
                .is_some_and(|d| d.category != horizon_client::brewing::Category::Beneficial)
        });
        if harmful { 50.0 } else { 25.0 }
    }
}

/// Android leaves room for the system inset in the top-left corner. A plain
/// call keeps `draw_extras` free of statement-level cfg, so compile-time
/// mixins can instrument it.
#[cfg(target_os = "android")]
fn reserve_android_inset(corners: &mut Corners) {
    corners.reserve(Corner::TopLeft, 26.0);
}
#[cfg(not(target_os = "android"))]
fn reserve_android_inset(_corners: &mut Corners) {}
