//! Volt's gameplay extras in the game loop:
//! - **Volt Minimap** ("Minimal with many features": terrain/cave slicing,
//!   chunk grid, entity radar, waypoint pins, and live XYZ/Biome/Clock pills)
//! - **Volt Waypoints** (3D world markers, beacon beams, auto-death pins, 8x
//!   Nether sync, screen-edge chevrons, quick-add hotkey & touch HUD button)
//! - **Volt Accurate Block Placement** (sub-face 5-zone aiming, orientation &
//!   slab-half lock, plane/line lock, reach-around edges, fast place delay,
//!   Vanilla-Safe look sync & Carpet protocol)
//! Plus mods' minimap/zoom/click/FPS hooks. Every Volt extra does nothing
//! while its master switch is off.
use super::{Dim, Game, Hit, Screen, native_hooks::NativeMinimap};
use horizon_client::{
    blocks::Fluid,
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
        MinimapEntities, MinimapTerrainMode, PlacementLock, PlaneLock, RadarKind,
        ResolvedPlacement, classify_mob_id, effective_waypoint_coords, format_coords,
        format_heading_and_biome, format_world_clock, is_orientable_block_item, plane_lock_allows,
        project_waypoint, resolve_accurate_placement, world_to_minimap_offset,
    },
};
use horizon_world::registry::BlockStateId;
use std::time::Duration;
use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Default)]
pub(super) struct ExtrasState {
    fps: FpsCounter,
    /// Left and right mouse buttons: held, and their clicks.
    mouse: [bool; 2],
    cps: [Cps; 2],
    minimap: Minimap,
    /// The dimension and terrain mode the minimap's tiles belong to.
    minimap_dim: Option<(Dim, MinimapTerrainMode)>,
    /// Auto-reconnect's attempts at the current server.
    reconnect: Tracker,
    /// Active continuous right-click drag anchor `(first_pos, row_axis_is_x)` for Plane Lock.
    pub(super) drag_plane_anchor: Option<([i32; 3], Option<bool>)>,
    /// Prevents recording duplicate death waypoints during a single death screen.
    pub(super) recorded_current_death: bool,
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
        if button == MouseButton::Right && !pressed {
            self.extras.drag_plane_anchor = None;
        }
        if pressed {
            let now = self.extras_now();
            self.extras.cps[i].click(now);
        }
    }
    /// Once per frame: the frame's time, and keys let go behind screens.
    pub(super) fn extras_frame(&mut self, elapsed: Duration) {
        let focused = self.focused || self.control.is_some();
        let playing = self.screen == Screen::Playing && self.title.is_none() && focused;
        if !playing {
            self.extras.mouse = [false; 2];
            self.extras.drag_plane_anchor = None;
        }
        if !self.right {
            self.extras.drag_plane_anchor = None;
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
        self.extras.fps.record(now, elapsed.as_secs_f32() * 1000.0);
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
    // Volt Hotkeys (Waypoints, Minimap Zoom, Placement Lock & Reverse)
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
        if self.settings.extras.accurate_placement {
            if self.settings.extras.placement_reverse_key == name {
                self.settings.extras.placement_reverse = !self.settings.extras.placement_reverse;
                let state = if self.settings.extras.placement_reverse {
                    "ON"
                } else {
                    "OFF"
                };
                self.settings.save(&Self::options_path());
                self.titles.set_overlay(
                    Text::literal(format!("Reverse Block Facing: {state}"))
                        .with_color(text::YELLOW),
                    false,
                );
                return true;
            }
            if self.settings.extras.placement_lock_key == name {
                if self.settings.extras.placement_lock != PlacementLock::Off {
                    self.settings.extras.placement_lock = PlacementLock::Off;
                    self.settings.save(&Self::options_path());
                    self.titles.set_overlay(
                        Text::literal("Orientation Lock: OFF").with_color(text::GRAY),
                        false,
                    );
                } else {
                    let next_lock = self
                        .preview_accurate_placement()
                        .map(|r| PlacementLock::from_direction(r.facing))
                        .unwrap_or_else(|| {
                            let dir = self.aim().map_or([0.0, 0.0, 1.0], |(_, d)| d);
                            PlacementLock::from_direction(
                                horizon_core::geometry::Direction::approximate_nearest(
                                    horizon_core::geometry::Vec3::new(dir[0], dir[1], dir[2]),
                                ),
                            )
                        });
                    self.settings.extras.placement_lock = next_lock;
                    self.settings.save(&Self::options_path());
                    self.titles.set_overlay(
                        Text::literal(format!("Orientation Locked: {}", next_lock.label()))
                            .with_color(text::GOLD),
                        false,
                    );
                }
                return true;
            }
        }
        false
    }

    // ========================================================================
    // Volt Accurate Block Placement
    // ========================================================================

    pub(super) fn configured_use_delay(&self) -> u8 {
        if !self.settings.extras.accurate_placement {
            return 4;
        }
        let holding_block = self
            .held()
            .and_then(|h| self.items().get(&h.item))
            .is_some_and(|i| i.block.is_some())
            || self
                .offhand
                .as_ref()
                .and_then(|h| self.items().get(&h.item))
                .is_some_and(|i| i.block.is_some());
        if holding_block {
            self.settings.extras.placement_fast_delay.min(4)
        } else {
            4
        }
    }

    /// Standard `self.pick()` with optional Reach-Around edge fallback when the
    /// player aims just past the edge or underside of a nearby solid block while
    /// holding a block item.
    pub(super) fn pick_for_placement(&self, holding_block: bool) -> Option<Hit> {
        if let Some(hit) = self.pick() {
            return Some(hit);
        }
        if !holding_block
            || !self.settings.extras.accurate_placement
            || !self.settings.extras.placement_reach_around
            || self.guest_use.is_some()
        {
            return None;
        }
        self.reach_around_hit()
    }

    fn reach_around_hit(&self) -> Option<Hit> {
        let (eye, dir) = self.aim()?;
        let reach = self.reach();
        let facts = self.res.facts();
        let mut t = 0.55;
        while t <= reach {
            let p = [eye[0] + dir[0] * t, eye[1] + dir[1] * t, eye[2] + dir[2] * t];
            let cell = [
                p[0].floor() as i32,
                p[1].floor() as i32,
                p[2].floor() as i32,
            ];
            let cell_state = self.world.block(cell[0], cell[1], cell[2]);
            let cell_facts = facts.get(cell_state);
            if cell_facts.outline.is_empty() && cell_facts.fluid == Fluid::None {
                let mut best: Option<(f64, [i32; 3], [i32; 3])> = None;
                for normal in [
                    [1, 0, 0],
                    [-1, 0, 0],
                    [0, 1, 0],
                    [0, -1, 0],
                    [0, 0, 1],
                    [0, 0, -1],
                ] {
                    // `support` is the solid neighbour; `normal` points from `support` into `cell`.
                    let support = [
                        cell[0] - normal[0],
                        cell[1] - normal[1],
                        cell[2] - normal[2],
                    ];
                    let sup_state = self.world.block(support[0], support[1], support[2]);
                    let sup_facts = facts.get(sup_state);
                    if self.world.is_air(sup_state)
                        || sup_facts.replaceable
                        || sup_facts.outline.is_empty()
                    {
                        continue;
                    }
                    // Distance from ray sample `p` inside `cell` to the shared face plane:
                    let plane_dist = match normal {
                        [1, 0, 0] => p[0] - f64::from(cell[0]),
                        [-1, 0, 0] => f64::from(cell[0] + 1) - p[0],
                        [0, 1, 0] => p[1] - f64::from(cell[1]),
                        [0, -1, 0] => f64::from(cell[1] + 1) - p[1],
                        [0, 0, 1] => p[2] - f64::from(cell[2]),
                        _ => f64::from(cell[2] + 1) - p[2],
                    };
                    if (0.0..=0.42).contains(&plane_dist)
                        && best.is_none_or(|(d, _, _)| plane_dist < d)
                    {
                        best = Some((plane_dist, support, normal));
                    }
                }
                if let Some((_, support, face)) = best {
                    let point = [
                        match face[0] {
                            1 => f64::from(support[0] + 1),
                            -1 => f64::from(support[0]),
                            _ => p[0].clamp(f64::from(support[0]) + 0.15, f64::from(support[0]) + 0.85),
                        },
                        match face[1] {
                            1 => f64::from(support[1] + 1),
                            -1 => f64::from(support[1]),
                            _ => p[1].clamp(f64::from(support[1]) + 0.15, f64::from(support[1]) + 0.85),
                        },
                        match face[2] {
                            1 => f64::from(support[2] + 1),
                            -1 => f64::from(support[2]),
                            _ => p[2].clamp(f64::from(support[2]) + 0.15, f64::from(support[2]) + 0.85),
                        },
                    ];
                    let dx = point[0] - eye[0];
                    let dy = point[1] - eye[1];
                    let dz = point[2] - eye[2];
                    if dx * dx + dy * dy + dz * dz <= reach * reach {
                        return Some(Hit {
                            pos: support,
                            face,
                            point,
                        });
                    }
                }
            }
            t += 0.18;
        }
        None
    }

    pub(super) fn resolve_local_accurate_placement(
        &self,
        item: &str,
        hit: &Hit,
    ) -> Option<ResolvedPlacement> {
        if !self.settings.extras.accurate_placement || self.guest_use.is_some() {
            return None;
        }
        let e = &self.settings.extras;
        Some(resolve_accurate_placement(
            item,
            hit.pos,
            hit.face,
            hit.point,
            self.yaw,
            self.pitch,
            e.placement_quadrant,
            e.placement_reverse,
            e.placement_lock,
            e.placement_protocol,
        ))
    }

    fn preview_accurate_placement(&self) -> Option<ResolvedPlacement> {
        if !self.settings.extras.accurate_placement {
            return None;
        }
        let item = self
            .held()
            .filter(|s| self.items().get(&s.item).is_some_and(|i| i.block.is_some()))
            .or_else(|| {
                self.offhand
                    .as_ref()
                    .filter(|s| self.items().get(&s.item).is_some_and(|i| i.block.is_some()))
            })?;
        let hit = self.pick_for_placement(true)?;
        if !is_orientable_block_item(&item.item)
            && self.settings.extras.placement_lock == PlacementLock::Off
            && !self.settings.extras.placement_reverse
        {
            return None;
        }
        self.resolve_local_accurate_placement(&item.item, &hit)
    }

    /// Checks whether `candidate_target` (`clicked_pos + face`) satisfies the
    /// active Plane / Line Lock during a continuous right-click hold.
    pub(super) fn plane_lock_permits_and_record(&mut self, hit: &Hit) -> bool {
        if !self.settings.extras.accurate_placement || self.guest_use.is_some() {
            return true;
        }
        let mode = self.settings.extras.placement_plane_lock;
        if mode == PlaneLock::Off {
            return true;
        }
        let state = self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
        let replaceable = self.res.facts().get(state).replaceable && !self.world.is_air(state);
        let target = if replaceable {
            hit.pos
        } else {
            [
                hit.pos[0] + hit.face[0],
                hit.pos[1] + hit.face[1],
                hit.pos[2] + hit.face[2],
            ]
        };
        match self.extras.drag_plane_anchor {
            None => {
                self.extras.drag_plane_anchor = Some((target, None));
                true
            }
            Some((anchor, axis_x)) => {
                if !plane_lock_allows(mode, anchor, axis_x, target) {
                    return false;
                }
                if mode == PlaneLock::RowAxis && axis_x.is_none() && target != anchor {
                    let chosen_x = target[0] != anchor[0];
                    self.extras.drag_plane_anchor = Some((anchor, Some(chosen_x)));
                }
                true
            }
        }
    }

    /// Applies singleplayer post-placement axis / half overrides for blocks whose
    /// vanilla `getStateForPlacement` uses `ctx.face` directly (logs, pillars,
    /// hoppers, slabs on Up/Down faces) without shifting `clicked_pos + face`.
    pub(super) fn refine_singleplayer_placed_state(
        &mut self,
        target: [i32; 3],
        state: BlockStateId,
        resolved: &ResolvedPlacement,
    ) -> BlockStateId {
        let Some(mut reg_state) = self.pack.assets.registry.state(state).cloned() else {
            return state;
        };
        let dir_str = match resolved.facing {
            horizon_core::geometry::Direction::Down => "down",
            horizon_core::geometry::Direction::Up => "up",
            horizon_core::geometry::Direction::North => "north",
            horizon_core::geometry::Direction::South => "south",
            horizon_core::geometry::Direction::West => "west",
            horizon_core::geometry::Direction::East => "east",
        };
        let axis_str = match resolved.facing {
            horizon_core::geometry::Direction::West | horizon_core::geometry::Direction::East => {
                "x"
            }
            horizon_core::geometry::Direction::Down | horizon_core::geometry::Direction::Up => "y",
            horizon_core::geometry::Direction::North | horizon_core::geometry::Direction::South => {
                "z"
            }
        };
        let mut changed = false;
        if reg_state.properties.contains_key("axis")
            && reg_state.properties.get("axis").map(String::as_str) != Some(axis_str)
        {
            reg_state.properties.insert("axis".into(), axis_str.into());
            changed = true;
        }
        if &*reg_state.name == "minecraft:hopper" {
            let hopper_dir = if dir_str == "up" { "down" } else { dir_str };
            if reg_state.properties.get("facing").map(String::as_str) != Some(hopper_dir) {
                reg_state
                    .properties
                    .insert("facing".into(), hopper_dir.into());
                changed = true;
            }
        }
        if let Some(top) = resolved.top_half {
            let want = if top { "top" } else { "bottom" };
            if let Some(cur) = reg_state.properties.get("type").cloned()
                && (cur == "top" || cur == "bottom")
                && cur != want
            {
                reg_state.properties.insert("type".into(), want.into());
                changed = true;
            }
            if let Some(cur) = reg_state.properties.get("half").cloned()
                && (cur == "top" || cur == "bottom")
                && cur != want
            {
                reg_state.properties.insert("half".into(), want.into());
                changed = true;
            }
        }
        if !changed {
            return state;
        }
        if let Ok(new_id) = self.pack.assets.registry.resolve(&reg_state) {
            let _ = self.level_set(target, new_id);
            new_id
        } else {
            state
        }
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
        if let Some(z) = self.native_zoom_now() {
            return (z.factor, z.smooth, f64::from(z.turn));
        }
        self.mod_zoom().map_or((1.0, true, 1.0), |z| {
            (z.factor, z.smooth, f64::from(z.turn))
        })
    }
    pub(super) fn zoomed_fov_target(&self, vanilla: f32) -> f32 {
        match self.zoom_now() {
            (factor, true, _) => fov_target(vanilla, factor),
            _ => vanilla,
        }
    }
    pub(super) fn zoomed_fov_degrees(&self, fov: f32) -> f32 {
        match self.zoom_now() {
            (factor, false, _) => fov / factor,
            _ => fov,
        }
    }
    pub(super) fn zoom_turn_scale(&self) -> f64 {
        self.zoom_now().2
    }

    /// Renders Volt 3D waypoints, touch `[+WP]` pin button, Accurate Block
    /// Placement crosshair indicator, Volt Minimap, and mod corner panels.
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

        // 3. Accurate Block Placement crosshair reticle & orientation pill:
        if self.screen == Screen::Playing
            && self.title.is_none()
            && self.settings.extras.accurate_placement
            && self.settings.extras.placement_hud_preview
            && let Some(preview) = self.preview_accurate_placement()
        {
            extras_hud::draw_placement_hud_badge(
                batch,
                &self.gui.atlas,
                batch.width,
                batch.height,
                &preview.badge,
                self.settings.extras.placement_quadrant,
                self.settings.extras.placement_lock != PlacementLock::Off,
            );
        }

        // 4. Minimap & corner panels:
        let mod_panels = self.mods.has_panels();
        let map = self.minimap_shown();
        let visible = |corner| !(self.debug && corner == Corner::TopLeft);
        let mut corners = Corners::new(batch.width, batch.height, self.effect_rows_height());
        #[cfg(target_os = "android")]
        corners.reserve(Corner::TopLeft, 26.0);
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
