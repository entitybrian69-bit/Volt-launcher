//! Volt Settings feature engines:
//! 1. **Volt Waypoints** (`WaypointStore`, 3D world-to-screen projection, 8x
//!    Overworld <-> Nether coordinate sync, automatic death waypoints).
//! 2. **Volt Minimap** (`MinimapFrame`, `MinimapTerrainMode`, `MinimapEntities`,
//!    radar entity classification, world-to-minimap projection, HUD readouts).
//! 3. **Volt Accurate Block Placement** (sub-face 5-zone quadrant targeting,
//!    orientation & slab-half lock, plane/line lock, reverse facing, vanilla
//!    rotation/cursor steering, and Carpet `accurateBlockPlacement` encoding).
//!
//! Every feature is **off by default** in `extras::Extras`.

use crate::{camera::view_matrix, minimap::View};
use glam::Vec3 as GlamVec3;
use horizon_core::geometry::Direction;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const WAYPOINTS_FILE: &str = "volt-waypoints.json";

/// Vivid preset colours for Volt Waypoints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaypointColor {
    #[default]
    Crimson,
    Gold,
    Emerald,
    Aqua,
    Sky,
    Violet,
    Coral,
    Snow,
}

impl WaypointColor {
    pub const ALL: [Self; 8] = [
        Self::Crimson,
        Self::Gold,
        Self::Emerald,
        Self::Aqua,
        Self::Sky,
        Self::Violet,
        Self::Coral,
        Self::Snow,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Crimson => "Crimson",
            Self::Gold => "Gold",
            Self::Emerald => "Emerald",
            Self::Aqua => "Aqua",
            Self::Sky => "Sky",
            Self::Violet => "Violet",
            Self::Coral => "Coral",
            Self::Snow => "White",
        }
    }

    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::Crimson => [239, 68, 68],
            Self::Gold => [245, 158, 11],
            Self::Emerald => [16, 185, 129],
            Self::Aqua => [6, 182, 212],
            Self::Sky => [59, 130, 246],
            Self::Violet => [168, 85, 247],
            Self::Coral => [244, 63, 94],
            Self::Snow => [248, 250, 252],
        }
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&c| c == self).unwrap_or(0);
        let next = if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        };
        Self::ALL[next]
    }
}

/// One saved Volt Waypoint, scoped to a singleplayer world or multiplayer server.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: u64,
    pub scope: String,
    pub name: String,
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    #[serde(default)]
    pub color: WaypointColor,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub is_death: bool,
}

fn default_true() -> bool {
    true
}

/// Persistent collection of Volt Waypoints (`volt-waypoints.json`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaypointStore {
    #[serde(default)]
    pub next_id: u64,
    #[serde(default)]
    pub entries: Vec<Waypoint>,
}

impl WaypointStore {
    pub fn load(dir: &Path) -> Self {
        fs::read_to_string(dir.join(WAYPOINTS_FILE))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, dir: &Path) {
        let path = dir.join(WAYPOINTS_FILE);
        if self.entries.is_empty() && !path.exists() {
            return;
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    fn alloc_id(&mut self) -> u64 {
        self.next_id = self
            .next_id
            .max(self.entries.iter().map(|w| w.id).max().unwrap_or(0))
            + 1;
        self.next_id
    }

    pub fn add(
        &mut self,
        scope: &str,
        name: &str,
        dimension: &str,
        pos: [i32; 3],
        color: WaypointColor,
    ) -> u64 {
        let id = self.alloc_id();
        let trimmed = name.trim();
        let final_name = if trimmed.is_empty() {
            let count = self
                .entries
                .iter()
                .filter(|w| w.scope == scope && !w.is_death)
                .count()
                + 1;
            format!("Waypoint {count}")
        } else {
            trimmed.chars().take(28).collect()
        };
        self.entries.push(Waypoint {
            id,
            scope: scope.to_owned(),
            name: final_name,
            dimension: dimension.to_owned(),
            x: pos[0],
            y: pos[1],
            z: pos[2],
            color,
            enabled: true,
            is_death: false,
        });
        id
    }

    /// Records or updates the player's latest death waypoint in `scope`,
    /// keeping at most 3 death markers per world/server.
    pub fn record_death(&mut self, scope: &str, dimension: &str, pos: [i32; 3]) -> u64 {
        for wp in &mut self.entries {
            if wp.scope == scope && wp.is_death && wp.name == "Last Death" {
                wp.name = format!("Death ({}, {}, {})", wp.x, wp.y, wp.z);
            }
        }
        let death_indices: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, w)| (w.scope == scope && w.is_death).then_some(i))
            .collect();
        if death_indices.len() >= 3 {
            let remove_count = death_indices.len() - 2;
            for &idx in death_indices.iter().take(remove_count).rev() {
                self.entries.remove(idx);
            }
        }
        let id = self.alloc_id();
        self.entries.push(Waypoint {
            id,
            scope: scope.to_owned(),
            name: "Last Death".to_owned(),
            dimension: dimension.to_owned(),
            x: pos[0],
            y: pos[1],
            z: pos[2],
            color: WaypointColor::Crimson,
            enabled: true,
            is_death: true,
        });
        id
    }

    pub fn scoped_indices(&self, scope: &str) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, w)| (w.scope == scope).then_some(i))
            .collect()
    }

    pub fn toggle(&mut self, index: usize) -> bool {
        if let Some(wp) = self.entries.get_mut(index) {
            wp.enabled = !wp.enabled;
            true
        } else {
            false
        }
    }

    pub fn cycle_color(&mut self, index: usize, back: bool) -> bool {
        if let Some(wp) = self.entries.get_mut(index) {
            wp.color = wp.color.cycle(back);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if index < self.entries.len() {
            self.entries.remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear_deaths(&mut self, scope: &str) -> usize {
        let before = self.entries.len();
        self.entries.retain(|w| !(w.scope == scope && w.is_death));
        before - self.entries.len()
    }
}

/// Effective target coordinates of a waypoint in `current_dim`, applying 8x
/// Overworld <-> Nether scaling when `nether_sync` is enabled.
/// Returns `(world_x, world_y, world_z, cross_dimension_tag)`.
pub fn effective_waypoint_coords(
    wp: &Waypoint,
    current_dim: &str,
    nether_sync: bool,
) -> Option<([f64; 3], Option<&'static str>)> {
    if wp.dimension == current_dim {
        return Some((
            [
                f64::from(wp.x) + 0.5,
                f64::from(wp.y) + 0.5,
                f64::from(wp.z) + 0.5,
            ],
            None,
        ));
    }
    if !nether_sync {
        return None;
    }
    let overworld = "minecraft:overworld";
    let nether = "minecraft:the_nether";
    if wp.dimension == overworld && current_dim == nether {
        Some((
            [
                (f64::from(wp.x) + 0.5) / 8.0,
                f64::from(wp.y).clamp(16.0, 120.0) + 0.5,
                (f64::from(wp.z) + 0.5) / 8.0,
            ],
            Some("OW"),
        ))
    } else if wp.dimension == nether && current_dim == overworld {
        Some((
            [
                (f64::from(wp.x) + 0.5) * 8.0,
                f64::from(wp.y) + 0.5,
                (f64::from(wp.z) + 0.5) * 8.0,
            ],
            Some("Nether"),
        ))
    } else {
        None
    }
}

pub fn short_dimension_label(dim: &str) -> &'static str {
    match dim {
        "minecraft:overworld" => "OW",
        "minecraft:the_nether" => "Nether",
        "minecraft:the_end" => "End",
        _ => "Dim",
    }
}

/// Screen-space projection of a 3D world waypoint for HUD rendering.
#[derive(Clone, Copy, Debug)]
pub struct ProjectedWaypoint {
    pub screen_x: f32,
    pub screen_y: f32,
    pub beam_top_y: f32,
    pub beam_bottom_y: f32,
    pub distance: f64,
    pub in_front: bool,
    pub on_screen: bool,
    /// Cosine similarity (`0..=1`) between camera centre and marker ray.
    pub center_focus: f32,
    /// Clamped rim position `(x, y)` for off-screen edge arrows.
    pub edge_pos: (f32, f32),
    /// Angle in radians pointing from screen centre toward the waypoint.
    pub edge_angle: f32,
}

pub fn project_waypoint(
    cam_pos: [f64; 3],
    yaw: f32,
    pitch: f32,
    fov_degrees: f32,
    screen_w: f32,
    screen_h: f32,
    target: [f64; 3],
) -> ProjectedWaypoint {
    let dx = target[0] - cam_pos[0];
    let dy = target[1] - cam_pos[1];
    let dz = target[2] - cam_pos[2];
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();
    let view = view_matrix(yaw, pitch);
    let aspect = (screen_w / screen_h.max(1.0)).max(0.1);
    let tan_half = (fov_degrees.clamp(10.0, 150.0).to_radians() * 0.5)
        .tan()
        .max(0.05);

    let project_pt = |wy: f64| -> (f32, f32, f32) {
        let rel = GlamVec3::new(dx as f32, (wy - cam_pos[1]) as f32, dz as f32);
        let eye = view.transform_point3(rel);
        let depth = -eye.z;
        let z_abs = depth.abs().max(0.05);
        let ndc_x = eye.x / (z_abs * tan_half * aspect);
        let ndc_y = eye.y / (z_abs * tan_half);
        (
            (ndc_x * 0.5 + 0.5) * screen_w,
            (0.5 - ndc_y * 0.5) * screen_h,
            depth,
        )
    };

    let (sx, sy, depth) = project_pt(target[1] + 1.35);
    let (_, top_y, _) = project_pt(target[1] + 180.0);
    let (_, bot_y, _) = project_pt(target[1] - 48.0);
    let in_front = depth > 0.1;
    let margin = 18.0;
    let on_screen = in_front
        && sx >= margin
        && sx <= screen_w - margin
        && sy >= margin
        && sy <= screen_h - margin;

    let rel_main = GlamVec3::new(dx as f32, (target[1] + 1.35 - cam_pos[1]) as f32, dz as f32);
    let eye_main = view.transform_point3(rel_main);
    let len = eye_main.length().max(0.001);
    let center_focus = (-eye_main.z / len).clamp(-1.0, 1.0);

    // Direction on the 2D screen plane from screen centre toward the waypoint.
    let mut dir_x = eye_main.x;
    let mut dir_y = -eye_main.y;
    if !in_front {
        // Behind camera: push cleanly toward the horizontal side so turning toward
        // the arrow always brings the waypoint into view.
        if dir_x.abs() < 0.001 {
            dir_x = 1.0;
        }
        dir_y = dir_y.clamp(-0.35 * dir_x.abs(), 0.35 * dir_x.abs());
    }
    let edge_angle = dir_y.atan2(dir_x);
    let (cx, cy) = (screen_w * 0.5, screen_h * 0.5);
    let half_w = (screen_w * 0.5 - 22.0).max(16.0);
    let half_h = (screen_h * 0.5 - 26.0).max(16.0);
    let scale_x = if dir_x.abs() > 1e-4 {
        half_w / dir_x.abs()
    } else {
        f32::INFINITY
    };
    let scale_y = if dir_y.abs() > 1e-4 {
        half_h / dir_y.abs()
    } else {
        f32::INFINITY
    };
    let t = scale_x.min(scale_y);
    let edge_pos = if t.is_finite() {
        (cx + dir_x * t, cy + dir_y * t)
    } else {
        (cx + half_w, cy)
    };

    ProjectedWaypoint {
        screen_x: sx,
        screen_y: sy,
        beam_top_y: top_y.min(bot_y).clamp(-32.0, screen_h + 32.0),
        beam_bottom_y: top_y.max(bot_y).clamp(-32.0, screen_h + 32.0),
        distance,
        in_front,
        on_screen,
        center_focus,
        edge_pos,
        edge_angle,
    }
}

// ============================================================================
// Volt Minimap Features
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapFrame {
    #[default]
    VoltCrimson,
    ClassicDark,
    CleanMinimal,
}

impl MinimapFrame {
    pub const ALL: [Self; 3] = [Self::VoltCrimson, Self::ClassicDark, Self::CleanMinimal];
    pub fn name(self) -> &'static str {
        match self {
            Self::VoltCrimson => "crimson",
            Self::ClassicDark => "dark",
            Self::CleanMinimal => "minimal",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::VoltCrimson => "Volt Crimson",
            Self::ClassicDark => "Classic Dark",
            Self::CleanMinimal => "Minimal Clean",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.name() == s)
            .unwrap_or_default()
    }
    pub fn cycle(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[if back {
            (i + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (i + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapTerrainMode {
    #[default]
    Auto,
    Surface,
    Cave,
}

impl MinimapTerrainMode {
    pub const ALL: [Self; 3] = [Self::Auto, Self::Surface, Self::Cave];
    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Surface => "surface",
            Self::Cave => "cave",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto (Cave/Surface)",
            Self::Surface => "Surface Only",
            Self::Cave => "Cave Slice",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.name() == s)
            .unwrap_or_default()
    }
    pub fn cycle(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[if back {
            (i + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (i + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapEntities {
    Off,
    Players,
    Hostiles,
    #[default]
    All,
}

impl MinimapEntities {
    pub const ALL: [Self; 4] = [Self::Off, Self::Players, Self::Hostiles, Self::All];
    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Players => "players",
            Self::Hostiles => "hostiles",
            Self::All => "all",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Players => "Players Only",
            Self::Hostiles => "Players & Hostiles",
            Self::All => "All Entities",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.name() == s)
            .unwrap_or_default()
    }
    pub fn cycle(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[if back {
            (i + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (i + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadarKind {
    Player,
    Hostile,
    Passive,
    Item,
}

impl RadarKind {
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::Player => [56, 189, 248],
            Self::Hostile => [239, 68, 68],
            Self::Passive => [34, 197, 94],
            Self::Item => [250, 204, 21],
        }
    }

    pub fn shown_by(self, filter: MinimapEntities) -> bool {
        match filter {
            MinimapEntities::Off => false,
            MinimapEntities::Players => self == Self::Player,
            MinimapEntities::Hostiles => matches!(self, Self::Player | Self::Hostile),
            MinimapEntities::All => true,
        }
    }
}

pub fn classify_mob_id(id: &str) -> RadarKind {
    let bare = id.strip_prefix("minecraft:").unwrap_or(id);
    if bare == "player" {
        return RadarKind::Player;
    }
    const HOSTILES: &[&str] = &[
        "zombie",
        "zombie_villager",
        "husk",
        "drowned",
        "zombified_piglin",
        "zoglin",
        "skeleton",
        "stray",
        "bogged",
        "wither_skeleton",
        "creeper",
        "spider",
        "cave_spider",
        "enderman",
        "endermite",
        "witch",
        "slime",
        "magma_cube",
        "blaze",
        "ghast",
        "phantom",
        "pillager",
        "vindicator",
        "evoker",
        "illusioner",
        "ravager",
        "vex",
        "guardian",
        "elder_guardian",
        "silverfish",
        "shulker",
        "piglin_brute",
        "hoglin",
        "warden",
        "breeze",
        "creaking",
        "ender_dragon",
        "wither",
    ];
    if HOSTILES.contains(&bare) {
        RadarKind::Hostile
    } else {
        RadarKind::Passive
    }
}

/// Projects world `(wx, wz)` into local minimap coordinates `(-half..=half, -half..=half)`
/// relative to the minimap centre `(cx, cy)` for a given `map_size` in GUI pixels.
/// When `clamp_rim` is true, out-of-range points are clamped onto the map rim and
/// returned with `clamped = true`; otherwise returns `None` when outside the map.
pub fn world_to_minimap_offset(
    wx: f64,
    wz: f64,
    view: &View,
    map_size: f32,
    clamp_rim: bool,
) -> Option<(f32, f32, bool)> {
    let scale = map_size / crate::minimap::SIZE as f32;
    let bpp = view.blocks_per_pixel.max(0.05);
    let dx = (wx - view.x) as f32 / bpp * scale;
    let dz = (wz - view.z) as f32 / bpp * scale;
    let (rx, ry) = if view.rotate {
        // Match `minimap::compose`: forward is up (-Y on screen).
        let (sin, cos) = (view.yaw + 180.0).to_radians().sin_cos();
        // Inverse of compose's `(u * c - v * s, u * s + v * c)`:
        (dx * cos + dz * sin, -dx * sin + dz * cos)
    } else {
        (dx, dz)
    };
    let half = (map_size * 0.5 - 3.0).max(4.0);
    if view.round {
        let dist = (rx * rx + ry * ry).sqrt();
        if dist <= half {
            Some((rx, ry, false))
        } else if clamp_rim && dist > 1e-4 {
            Some((rx / dist * half, ry / dist * half, true))
        } else {
            None
        }
    } else {
        let max_axis = rx.abs().max(ry.abs());
        if max_axis <= half {
            Some((rx, ry, false))
        } else if clamp_rim && max_axis > 1e-4 {
            Some((rx / max_axis * half, ry / max_axis * half, true))
        } else {
            None
        }
    }
}

pub fn format_coords(pos: [f64; 3]) -> String {
    format!(
        "{}, {}, {}",
        pos[0].floor() as i32,
        pos[1].floor() as i32,
        pos[2].floor() as i32
    )
}

pub fn format_heading_and_biome(yaw: f32, biome: Option<&str>) -> String {
    let mut wrapped = yaw % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    // Vanilla yaw: 0 = South (+Z), -90 = East (+X), +/-180 = North (-Z), 90 = West (-X).
    let norm = (wrapped + 360.0) % 360.0;
    let dirs = ["S", "SW", "W", "NW", "N", "NE", "E", "SE"];
    let idx = (((norm + 22.5) / 45.0).floor() as usize) % 8;
    let cardinal = dirs[idx];
    if let Some(raw) = biome {
        let pretty = prettify_identifier(raw);
        format!("{cardinal} · {pretty}")
    } else {
        format!("{cardinal} ({wrapped:.0}°)")
    }
}

fn prettify_identifier(id: &str) -> String {
    let bare = id.rsplit(':').next().unwrap_or(id);
    bare.split('_')
        .filter(|s| !s.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_world_clock(day_time: i64) -> String {
    let day = day_time.div_euclid(24_000) + 1;
    let ticks = day_time.rem_euclid(24_000);
    // Tick 0 in Minecraft is 06:00 AM.
    let total_minutes = ((ticks + 6_000) % 24_000) * 60 / 1_000;
    let hour = total_minutes / 60;
    let minute = total_minutes % 60;
    let phase = if (0..12_500).contains(&ticks) {
        "Day"
    } else {
        "Night"
    };
    format!("D{day} {hour:02}:{minute:02} ({phase})")
}

// ============================================================================
// Volt Accurate Block Placement
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlacementProtocol {
    #[default]
    VanillaSafe,
    Carpet,
}

impl PlacementProtocol {
    pub const ALL: [Self; 2] = [Self::VanillaSafe, Self::Carpet];
    pub fn name(self) -> &'static str {
        match self {
            Self::VanillaSafe => "vanilla",
            Self::Carpet => "carpet",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::VanillaSafe => "Vanilla Safe",
            Self::Carpet => "Carpet Protocol",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|p| p.name() == s)
            .unwrap_or_default()
    }
    pub fn cycle(self, _back: bool) -> Self {
        match self {
            Self::VanillaSafe => Self::Carpet,
            Self::Carpet => Self::VanillaSafe,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlacementLock {
    #[default]
    Off,
    North,
    South,
    West,
    East,
    Up,
    Down,
    TopHalf,
    BottomHalf,
}

impl PlacementLock {
    pub const ALL: [Self; 9] = [
        Self::Off,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
        Self::Up,
        Self::Down,
        Self::TopHalf,
        Self::BottomHalf,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
            Self::Up => "up",
            Self::Down => "down",
            Self::TopHalf => "top",
            Self::BottomHalf => "bottom",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::North => "North",
            Self::South => "South",
            Self::West => "West",
            Self::East => "East",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::TopHalf => "Slab/Stair Top",
            Self::BottomHalf => "Slab/Stair Bottom",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.name() == s)
            .unwrap_or_default()
    }
    pub fn cycle(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[if back {
            (i + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (i + 1) % Self::ALL.len()
        }]
    }
    pub fn from_direction(dir: Direction) -> Self {
        match dir {
            Direction::North => Self::North,
            Direction::South => Self::South,
            Direction::West => Self::West,
            Direction::East => Self::East,
            Direction::Up => Self::Up,
            Direction::Down => Self::Down,
        }
    }
    pub fn direction(self) -> Option<Direction> {
        Some(match self {
            Self::North => Direction::North,
            Self::South => Direction::South,
            Self::West => Direction::West,
            Self::East => Direction::East,
            Self::Up => Direction::Up,
            Self::Down => Direction::Down,
            Self::Off | Self::TopHalf | Self::BottomHalf => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlaneLock {
    #[default]
    Off,
    HorizontalY,
    VerticalColumn,
    RowAxis,
}

impl PlaneLock {
    pub const ALL: [Self; 4] = [
        Self::Off,
        Self::HorizontalY,
        Self::VerticalColumn,
        Self::RowAxis,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::HorizontalY => "horizontal",
            Self::VerticalColumn => "column",
            Self::RowAxis => "row",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::HorizontalY => "Horizontal Layer",
            Self::VerticalColumn => "Vertical Column",
            Self::RowAxis => "Straight Row",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.name() == s)
            .unwrap_or_default()
    }
    pub fn cycle(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[if back {
            (i + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (i + 1) % Self::ALL.len()
        }]
    }
}

/// Checks whether a continuous right-click drag placement at `target` satisfies `mode`
/// relative to the first block `anchor` placed in the drag (`second_axis` locks X vs Z
/// once the second block in `RowAxis` is placed).
pub fn plane_lock_allows(
    mode: PlaneLock,
    anchor: [i32; 3],
    row_axis_x: Option<bool>,
    target: [i32; 3],
) -> bool {
    match mode {
        PlaneLock::Off => true,
        PlaneLock::HorizontalY => target[1] == anchor[1],
        PlaneLock::VerticalColumn => target[0] == anchor[0] && target[2] == anchor[2],
        PlaneLock::RowAxis => {
            if target[1] != anchor[1] {
                return false;
            }
            match row_axis_x {
                None => target[0] == anchor[0] || target[2] == anchor[2],
                Some(true) => target[2] == anchor[2],
                Some(false) => target[0] == anchor[0],
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedPlacement {
    pub facing: Direction,
    /// `Some(true)` for top half, `Some(false)` for bottom half.
    pub top_half: Option<bool>,
    /// Player `(yaw, pitch)` that makes vanilla `getStateForPlacement` pick `facing`.
    pub effective_yaw: f32,
    pub effective_pitch: f32,
    /// Clicked face normal to feed into `PlaceContext` / `UseItemOn`.
    pub effective_face: [i32; 3],
    /// Clean sub-block hit coordinates `[0.05..=0.95]^3` for local `PlaceContext` & prediction.
    pub local_cursor: [f32; 3],
    /// Wire cursor for `UseItemOn` (`local_cursor` in `VanillaSafe`, or Carpet encoded `cursor[0]`).
    pub cursor: [f32; 3],
    /// Whether multiplayer `VanillaSafe` benefits from sending a temporary `MovePlayerRot`.
    pub needs_rotation_packet: bool,
    /// Compact HUD badge text (e.g. `"NORTH · Top"`).
    pub badge: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OrientKind {
    /// `nearest_looking_direction().opposite()` (piston, dispenser, dropper, barrel, crafter, command).
    LookOppositeAll,
    /// `nearest_looking_direction()` (observer).
    LookSameAll,
    /// 6-way face-oriented blocks (shulker_box, end_rod, lightning_rod, amethyst_cluster).
    FaceDir,
    /// `horizontal.opposite()` (repeater, comparator, furnace, chest, lectern, terracotta, trapdoor...).
    HorizontalOpposite,
    /// `horizontal` (stairs, fence_gate, door, campfire, decorated_pot).
    HorizontalSame,
    /// `clockwise(horizontal)` (anvil).
    Anvil,
    /// `axis_of(ctx.face)` (logs, wood, pillars, hay, chain, basalt, bone_block, creaking_heart).
    FaceAxis,
    /// Hopper (`ctx.face.opposite()`, Up -> Down).
    Hopper,
    /// Slab / general block.
    Other,
}

fn counter_clockwise_horiz(dir: Direction) -> Direction {
    match dir {
        Direction::North => Direction::West,
        Direction::West => Direction::South,
        Direction::South => Direction::East,
        Direction::East => Direction::North,
        other => other,
    }
}

fn clockwise_horiz(dir: Direction) -> Direction {
    match dir {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        other => other,
    }
}

fn classify_block_item(item: &str) -> (OrientKind, bool) {
    let b = item.strip_prefix("minecraft:").unwrap_or(item);
    let has_half = b.ends_with("_slab") || b.ends_with("_stairs") || b.ends_with("_trapdoor");
    if b == "observer" {
        return (OrientKind::LookSameAll, false);
    }
    if matches!(
        b,
        "piston"
            | "sticky_piston"
            | "dispenser"
            | "dropper"
            | "barrel"
            | "crafter"
            | "command_block"
            | "chain_command_block"
            | "repeating_command_block"
    ) {
        return (OrientKind::LookOppositeAll, false);
    }
    if b == "shulker_box"
        || b.ends_with("_shulker_box")
        || b == "end_rod"
        || b == "lightning_rod"
        || b.ends_with("_lightning_rod")
        || b == "amethyst_cluster"
        || b.ends_with("_amethyst_bud")
    {
        return (OrientKind::FaceDir, false);
    }
    if matches!(b, "anvil" | "chipped_anvil" | "damaged_anvil") {
        return (OrientKind::Anvil, false);
    }
    if b.ends_with("_stairs") {
        return (OrientKind::HorizontalSame, true);
    }
    if b.ends_with("_fence_gate")
        || b.ends_with("_door")
        || b.ends_with("_bed")
        || matches!(b, "campfire" | "soul_campfire" | "decorated_pot")
    {
        return (OrientKind::HorizontalSame, false);
    }
    if b.ends_with("_trapdoor") {
        return (OrientKind::HorizontalOpposite, true);
    }
    if b == "hopper" {
        return (OrientKind::Hopper, false);
    }
    if b.ends_with("_log")
        || b.ends_with("_wood")
        || b.ends_with("_stem")
        || b.ends_with("_hyphae")
        || b.ends_with("_pillar")
        || matches!(
            b,
            "bamboo_block"
                | "stripped_bamboo_block"
                | "basalt"
                | "polished_basalt"
                | "bone_block"
                | "hay_block"
                | "ochre_froglight"
                | "verdant_froglight"
                | "pearlescent_froglight"
                | "chain"
                | "creaking_heart"
        )
    {
        return (OrientKind::FaceAxis, false);
    }
    if b.ends_with("_glazed_terracotta")
        || b.ends_with("_chest")
        || b.ends_with("_shelf")
        || (b.ends_with("_sign") && !b.ends_with("_hanging_sign"))
        || b.ends_with("_banner")
        || matches!(
            b,
            "repeater"
                | "comparator"
                | "furnace"
                | "blast_furnace"
                | "smoker"
                | "lectern"
                | "loom"
                | "stonecutter"
                | "chest"
                | "beehive"
                | "bee_nest"
                | "carved_pumpkin"
                | "jack_o_lantern"
                | "chiseled_bookshelf"
                | "end_portal_frame"
                | "vault"
                | "dried_ghast"
                | "small_dripleaf"
        )
    {
        return (OrientKind::HorizontalOpposite, false);
    }
    (OrientKind::Other, has_half)
}

/// Returns whether holding `item` benefits from Accurate Block Placement's
/// directional / half HUD preview and orientation controls.
pub fn is_orientable_block_item(item: &str) -> bool {
    let (kind, has_half) = classify_block_item(item);
    kind != OrientKind::Other || has_half
}

fn face_to_direction(face: [i32; 3]) -> Direction {
    match face {
        [0, -1, 0] => Direction::Down,
        [0, 1, 0] => Direction::Up,
        [0, 0, -1] => Direction::North,
        [0, 0, 1] => Direction::South,
        [-1, 0, 0] => Direction::West,
        _ => Direction::East,
    }
}

/// Half-width of the centre zone (`0.20..=0.80`) on a clicked block face.
pub const SUBFACE_CENTER_HALF: f32 = 0.30;

/// 5-zone sub-face quadrant resolver:
/// - Centre (`|du| <= 0.30 && |dv| <= 0.30`) picks the clicked `face` normal itself.
/// - Outer rim wedges (`< 0.20` or `> 0.80`) pick the tangent `Direction` toward the aimed edge!
pub fn subface_quadrant_direction(face: [i32; 3], local: [f32; 3]) -> Direction {
    let base = face_to_direction(face);
    let (u, v, pos_u, neg_u, pos_v, neg_v) = match base {
        Direction::Up | Direction::Down => (
            local[0] - 0.5,
            local[2] - 0.5,
            Direction::East,
            Direction::West,
            Direction::South,
            Direction::North,
        ),
        Direction::North | Direction::South => (
            local[0] - 0.5,
            local[1] - 0.5,
            Direction::East,
            Direction::West,
            Direction::Up,
            Direction::Down,
        ),
        Direction::West | Direction::East => (
            local[2] - 0.5,
            local[1] - 0.5,
            Direction::South,
            Direction::North,
            Direction::Up,
            Direction::Down,
        ),
    };
    if u.abs() <= SUBFACE_CENTER_HALF && v.abs() <= SUBFACE_CENTER_HALF {
        return base;
    }
    if u.abs() >= v.abs() {
        if u >= 0.0 { pos_u } else { neg_u }
    } else if v >= 0.0 {
        pos_v
    } else {
        neg_v
    }
}

/// Yaw and pitch looking straight along `dir`.
pub fn angles_looking_toward(dir: Direction) -> (f32, f32) {
    match dir {
        Direction::South => (0.0, 0.0),
        Direction::West => (90.0, 0.0),
        Direction::North => (180.0, 0.0),
        Direction::East => (-90.0, 0.0),
        Direction::Down => (0.0, 89.0),
        Direction::Up => (0.0, -89.0),
    }
}

fn horizontal_from_yaw(player_yaw: f32) -> Direction {
    let mut w = (player_yaw % 360.0 + 360.0) % 360.0;
    if w >= 360.0 {
        w -= 360.0;
    }
    match (((w + 45.0) / 90.0).floor() as usize) % 4 {
        0 => Direction::South,
        1 => Direction::West,
        2 => Direction::North,
        _ => Direction::East,
    }
}

fn horizontal_fallback_for_kind(dir: Direction, kind: OrientKind, player_yaw: f32) -> Direction {
    match dir {
        Direction::North | Direction::South | Direction::West | Direction::East => dir,
        Direction::Up | Direction::Down => {
            let look = horizontal_from_yaw(player_yaw);
            match kind {
                OrientKind::HorizontalOpposite => look.opposite(),
                OrientKind::Anvil => clockwise_horiz(look),
                _ => look,
            }
        }
    }
}

pub fn direction_label(dir: Direction) -> &'static str {
    match dir {
        Direction::Down => "DOWN",
        Direction::Up => "UP",
        Direction::North => "NORTH",
        Direction::South => "SOUTH",
        Direction::West => "WEST",
        Direction::East => "EAST",
    }
}

/// Resolves Accurate Block Placement for a hit on `pos` (`face`, `point`) while
/// holding `item`.
#[allow(clippy::too_many_arguments)]
pub fn resolve_accurate_placement(
    item: &str,
    pos: [i32; 3],
    face: [i32; 3],
    point: [f64; 3],
    player_yaw: f32,
    player_pitch: f32,
    quadrant: bool,
    reverse: bool,
    lock: PlacementLock,
    protocol: PlacementProtocol,
) -> ResolvedPlacement {
    let local = [
        (point[0] - f64::from(pos[0])).clamp(0.0, 1.0) as f32,
        (point[1] - f64::from(pos[1])).clamp(0.0, 1.0) as f32,
        (point[2] - f64::from(pos[2])).clamp(0.0, 1.0) as f32,
    ];
    let (orient_kind, has_half) = classify_block_item(item);

    // 1. Determine base desired facing direction before reverse flip.
    let mut facing = if let Some(locked_dir) = lock.direction() {
        locked_dir
    } else if quadrant {
        subface_quadrant_direction(face, local)
    } else {
        match orient_kind {
            OrientKind::LookOppositeAll => {
                if player_pitch > 50.0 {
                    Direction::Up
                } else if player_pitch < -50.0 {
                    Direction::Down
                } else {
                    horizontal_from_yaw(player_yaw).opposite()
                }
            }
            OrientKind::LookSameAll => {
                if player_pitch > 50.0 {
                    Direction::Down
                } else if player_pitch < -50.0 {
                    Direction::Up
                } else {
                    horizontal_from_yaw(player_yaw)
                }
            }
            OrientKind::Hopper => face_to_direction(face).opposite(),
            _ => face_to_direction(face),
        }
    };
    if matches!(
        orient_kind,
        OrientKind::HorizontalOpposite | OrientKind::HorizontalSame | OrientKind::Anvil
    ) {
        facing = horizontal_fallback_for_kind(facing, orient_kind, player_yaw);
    }
    if reverse {
        facing = match orient_kind {
            OrientKind::FaceAxis => match facing {
                Direction::Up | Direction::Down => horizontal_from_yaw(player_yaw),
                Direction::North | Direction::South => Direction::East,
                Direction::East | Direction::West => Direction::South,
            },
            _ => facing.opposite(),
        };
    }

    // 2. Determine slab / stair / trapdoor top vs bottom half.
    let top_half = if has_half {
        Some(match lock {
            PlacementLock::TopHalf => true,
            PlacementLock::BottomHalf => false,
            _ => match face_to_direction(face) {
                Direction::Down => !reverse,
                Direction::Up => reverse,
                _ => {
                    let raw_top = local[1] > 0.5;
                    if reverse { !raw_top } else { raw_top }
                }
            },
        })
    } else {
        None
    };

    // 3. Compute effective player `(yaw, pitch)`, `effective_face`, `local_cursor`, and `cursor`.
    let mut effective_yaw = player_yaw;
    let mut effective_pitch = player_pitch;
    let mut effective_face = face;
    let mut local_cursor = [
        local[0].clamp(0.05, 0.95),
        local[1].clamp(0.05, 0.95),
        local[2].clamp(0.05, 0.95),
    ];
    let mut needs_rotation_packet = false;

    if let Some(top) = top_half {
        local_cursor[1] = if top { 0.85 } else { 0.15 };
    }
    let mut cursor = local_cursor;

    match orient_kind {
        OrientKind::LookOppositeAll => {
            let (y, p) = angles_looking_toward(facing.opposite());
            effective_yaw = y;
            effective_pitch = p;
            needs_rotation_packet = true;
        }
        OrientKind::LookSameAll => {
            let (y, p) = angles_looking_toward(facing);
            effective_yaw = y;
            effective_pitch = p;
            needs_rotation_packet = true;
        }
        OrientKind::HorizontalOpposite => {
            let (y, _) = angles_looking_toward(facing.opposite());
            effective_yaw = y;
            effective_pitch = 0.0;
            needs_rotation_packet = true;
        }
        OrientKind::HorizontalSame => {
            let (y, _) = angles_looking_toward(facing);
            effective_yaw = y;
            effective_pitch = 0.0;
            needs_rotation_packet = true;
        }
        OrientKind::Anvil => {
            let (y, _) = angles_looking_toward(counter_clockwise_horiz(facing));
            effective_yaw = y;
            effective_pitch = 0.0;
            needs_rotation_packet = true;
        }
        OrientKind::FaceDir | OrientKind::FaceAxis | OrientKind::Hopper => {
            let _ = &mut effective_face;
        }
        OrientKind::Other => {}
    }

    if protocol == PlacementProtocol::Carpet {
        // Standard Carpet / Tweakeroo AccurateBlockPlacement encoding:
        // facing ordinal (0..=5) + bit 0x10 when half == top, packed into
        // `cursor[0] = local_x + 2.0 + (code * 2) as f32`.
        let mut code = facing as u32;
        if top_half == Some(true) {
            code |= 0x10;
        }
        cursor[0] = local[0].clamp(0.0, 0.999) + 2.0 + (code * 2) as f32;
        needs_rotation_packet = false;
    }

    let mut badge_parts = Vec::with_capacity(3);
    if lock != PlacementLock::Off {
        badge_parts.push(format!("LOCK {}", lock.label()));
    } else if orient_kind != OrientKind::Other {
        badge_parts.push(direction_label(facing).to_owned());
    }
    if let Some(top) = top_half
        && !matches!(lock, PlacementLock::TopHalf | PlacementLock::BottomHalf)
    {
        badge_parts.push(if top { "Top" } else { "Bottom" }.to_owned());
    }
    if reverse {
        badge_parts.push("REV".to_owned());
    }
    if badge_parts.is_empty() {
        badge_parts.push(direction_label(facing).to_owned());
    }

    ResolvedPlacement {
        facing,
        top_half,
        effective_yaw,
        effective_pitch,
        effective_face,
        local_cursor,
        cursor,
        needs_rotation_packet,
        badge: badge_parts.join(" · "),
    }
}

/// Decodes Carpet / Tweakeroo / Volt Accurate Block Placement `UseItemOn` cursor
/// encoding (`cursor[0] >= 2.0`), returning `(restored_cursor, desired_facing, top_half)`.
pub fn decode_carpet_cursor(cursor: [f32; 3]) -> Option<([f32; 3], Direction, Option<bool>)> {
    if !cursor.iter().all(|c| c.is_finite())
        || cursor[0] < 2.0
        || !(0.0..=1.0).contains(&cursor[1])
        || !(0.0..=1.0).contains(&cursor[2])
    {
        return None;
    }
    let raw = cursor[0] - 2.0;
    if raw > 128.0 {
        return None;
    }
    let code = (raw.floor() as u32) / 2;
    let dir = *Direction::ALL.get((code & 0x7) as usize)?;
    let top_half = Some((code & 0x10) != 0);
    let clean_x = (raw - (code * 2) as f32).clamp(0.0, 1.0);
    Some(([clean_x, cursor[1], cursor[2]], dir, top_half))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waypoint_store_handles_death_limit_and_nether_sync() {
        let mut store = WaypointStore::default();
        let id = store.add("sp:Test", "  Base  ", "minecraft:overworld", [80, 64, -160], WaypointColor::Emerald);
        assert_eq!(id, 1);
        assert_eq!(store.entries[0].name, "Base");

        let wp = &store.entries[0];
        let (nether_pos, tag) =
            effective_waypoint_coords(wp, "minecraft:the_nether", true).unwrap();
        assert_eq!(tag, Some("OW"));
        assert!((nether_pos[0] - 10.0625).abs() < 0.01);
        assert!((nether_pos[2] - -19.9375).abs() < 0.01);
        assert!(effective_waypoint_coords(wp, "minecraft:the_nether", false).is_none());

        for x in [10, 20, 30, 40] {
            store.record_death("sp:Test", "minecraft:overworld", [x, 70, 0]);
        }
        let deaths: Vec<&Waypoint> = store.entries.iter().filter(|w| w.is_death).collect();
        assert_eq!(deaths.len(), 3);
        assert_eq!(deaths.last().unwrap().name, "Last Death");
        assert_eq!(deaths.last().unwrap().x, 40);
        assert_eq!(store.clear_deaths("sp:Test"), 3);
    }

    #[test]
    fn projection_places_forward_waypoint_at_screen_center() {
        // Yaw 0 in Minecraft faces South (+Z).
        let proj = project_waypoint(
            [0.0, 65.62, 0.0],
            0.0,
            0.0,
            70.0,
            480.0,
            270.0,
            [0.0, 64.27, 25.0],
        );
        assert!(proj.in_front && proj.on_screen);
        assert!((proj.screen_x - 240.0).abs() < 1.0, "{}", proj.screen_x);
        assert!((proj.screen_y - 135.0).abs() < 2.0, "{}", proj.screen_y);
        assert!(proj.center_focus > 0.99);
    }

    #[test]
    fn accurate_placement_quadrants_and_protocols_work() {
        // Clicking the East wedge of the Up face of a block with a piston:
        let res = resolve_accurate_placement(
            "minecraft:piston",
            [10, 64, 10],
            [0, 1, 0],
            [10.9, 65.0, 10.5],
            0.0,
            45.0,
            true,
            false,
            PlacementLock::Off,
            PlacementProtocol::VanillaSafe,
        );
        assert_eq!(res.facing, Direction::East);
        assert!(res.needs_rotation_packet);
        assert!(res.cursor.iter().all(|&c| (0.05..=0.95).contains(&c)));

        // Carpet protocol encodes facing + top half into cursor[0] >= 2.0:
        let carpet = resolve_accurate_placement(
            "minecraft:oak_stairs",
            [10, 64, 10],
            [0, 0, 1],
            [10.5, 64.8, 11.0],
            0.0,
            0.0,
            false,
            false,
            PlacementLock::TopHalf,
            PlacementProtocol::Carpet,
        );
        assert_eq!(carpet.top_half, Some(true));
        assert!(carpet.cursor[0] >= 2.0);
        assert!(carpet.local_cursor.iter().all(|&c| (0.05..=0.95).contains(&c)));
        assert!(!carpet.needs_rotation_packet);
        let (clean, dir, half) = decode_carpet_cursor(carpet.cursor).unwrap();
        assert_eq!(dir, carpet.facing);
        assert_eq!(half, Some(true));
        assert!((0.0..=1.0).contains(&clean[0]));

        // Facing North (180°), clicking floor centre:
        // - Chest faces South (toward player), Reverse flips to North.
        // - Stairs ascend North (away from player), Reverse flips to South.
        // - Log stands vertically (Up), Reverse lays it horizontally along look (North).
        let chest = resolve_accurate_placement(
            "minecraft:chest",
            [0, 63, 0],
            [0, 1, 0],
            [0.5, 64.0, 0.72],
            180.0,
            45.0,
            true,
            false,
            PlacementLock::Off,
            PlacementProtocol::Carpet,
        );
        assert_eq!(chest.facing, Direction::South);
        let chest_rev = resolve_accurate_placement(
            "minecraft:chest",
            [0, 63, 0],
            [0, 1, 0],
            [0.5, 64.0, 0.72],
            180.0,
            45.0,
            true,
            true,
            PlacementLock::Off,
            PlacementProtocol::Carpet,
        );
        assert_eq!(chest_rev.facing, Direction::North);
        let stairs = resolve_accurate_placement(
            "minecraft:oak_stairs",
            [0, 63, 0],
            [0, 1, 0],
            [0.5, 64.0, 0.72],
            180.0,
            45.0,
            true,
            false,
            PlacementLock::Off,
            PlacementProtocol::Carpet,
        );
        assert_eq!(stairs.facing, Direction::North);
        let log_rev = resolve_accurate_placement(
            "minecraft:oak_log",
            [0, 63, 0],
            [0, 1, 0],
            [0.5, 64.0, 0.5],
            180.0,
            45.0,
            true,
            true,
            PlacementLock::Off,
            PlacementProtocol::Carpet,
        );
        assert_eq!(log_rev.facing, Direction::North);
    }

    #[test]
    fn plane_lock_constraints_hold() {
        assert!(plane_lock_allows(PlaneLock::HorizontalY, [0, 64, 0], None, [5, 64, -3]));
        assert!(!plane_lock_allows(PlaneLock::HorizontalY, [0, 64, 0], None, [0, 65, 0]));
        assert!(plane_lock_allows(PlaneLock::VerticalColumn, [2, 64, 3], None, [2, 80, 3]));
        assert!(!plane_lock_allows(PlaneLock::VerticalColumn, [2, 64, 3], None, [3, 64, 3]));
        assert!(plane_lock_allows(PlaneLock::RowAxis, [0, 64, 0], Some(true), [4, 64, 0]));
        assert!(!plane_lock_allows(PlaneLock::RowAxis, [0, 64, 0], Some(true), [4, 64, 1]));
    }

    #[test]
    fn volt_boost_presets_round_trip_and_default_off() {
        assert_eq!(VoltBoostPreset::default(), VoltBoostPreset::Off);
        for p in VoltBoostPreset::ALL {
            assert_eq!(VoltBoostPreset::parse(p.as_str()), Some(p));
        }
        for m in PerfHudMode::ALL {
            assert_eq!(PerfHudMode::parse(m.as_str()), Some(m));
        }
        for c in EntityCap::ALL {
            assert_eq!(EntityCap::parse(c.as_str()), Some(c));
        }
        for b in ParticleBudget::ALL {
            assert_eq!(ParticleBudget::parse(b.as_str()), Some(b));
        }
    }
}

// ============================================================================
// 4. Volt Boost — Tier 1, Tier 2 & Tier 3 Performance & Optimization Types
// ============================================================================

/// Master one-tap profile for **Volt Boost**. `Off` preserves 100% vanilla behaviour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoltBoostPreset {
    #[default]
    Off,
    Balanced,
    Competitive,
    UltraFps,
    BatterySaver,
    LowRam,
}

impl VoltBoostPreset {
    pub const ALL: [Self; 6] = [
        Self::Off,
        Self::Balanced,
        Self::Competitive,
        Self::UltraFps,
        Self::BatterySaver,
        Self::LowRam,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Balanced => "balanced",
            Self::Competitive => "competitive",
            Self::UltraFps => "ultra_fps",
            Self::BatterySaver => "battery_saver",
            Self::LowRam => "low_ram",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Self::Off),
            "balanced" => Some(Self::Balanced),
            "competitive" => Some(Self::Competitive),
            "ultra_fps" => Some(Self::UltraFps),
            "battery_saver" => Some(Self::BatterySaver),
            "low_ram" => Some(Self::LowRam),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Off => "options.off",
            Self::Balanced => "volt.boost.preset.balanced",
            Self::Competitive => "volt.boost.preset.competitive",
            Self::UltraFps => "volt.boost.preset.ultra_fps",
            Self::BatterySaver => "volt.boost.preset.battery_saver",
            Self::LowRam => "volt.boost.preset.low_ram",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Balanced => "Balanced",
            Self::Competitive => "Competitive PvP",
            Self::UltraFps => "Ultra FPS",
            Self::BatterySaver => "Battery Saver",
            Self::LowRam => "Low-RAM Rescue",
        }
    }
}

/// Live performance pill overlay (`Off`, `Compact` FPS/1%-low/RAM, `Full` telemetry + graph).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfHudMode {
    #[default]
    Off,
    Compact,
    Full,
}

impl PerfHudMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Compact, Self::Full];

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Compact,
            Self::Compact => Self::Full,
            Self::Full => Self::Off,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Compact => "compact",
            Self::Full => "full",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" | "false" => Some(Self::Off),
            "compact" | "true" => Some(Self::Compact),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Off => "options.off",
            Self::Compact => "volt.boost.perf_hud.compact",
            Self::Full => "volt.boost.perf_hud.full",
        }
    }
}

/// Per-frame chunk GPU upload limiter to prevent camera-turn frame spikes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkUploadCap {
    #[default]
    Auto,
    Smooth4,
    Smooth8,
    Balanced16,
    Burst32,
}

impl ChunkUploadCap {
    pub const ALL: [Self; 5] = [
        Self::Auto,
        Self::Smooth4,
        Self::Smooth8,
        Self::Balanced16,
        Self::Burst32,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn max_uploads(self) -> Option<usize> {
        match self {
            Self::Auto => None,
            Self::Smooth4 => Some(4),
            Self::Smooth8 => Some(8),
            Self::Balanced16 => Some(16),
            Self::Burst32 => Some(32),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Smooth4 => "4",
            Self::Smooth8 => "8",
            Self::Balanced16 => "16",
            Self::Burst32 => "32",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(Self::Auto),
            "4" | "smooth4" => Some(Self::Smooth4),
            "8" | "smooth8" => Some(Self::Smooth8),
            "16" | "balanced16" => Some(Self::Balanced16),
            "32" | "burst32" => Some(Self::Burst32),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Smooth4 => "4 / frame",
            Self::Smooth8 => "8 / frame",
            Self::Balanced16 => "16 / frame",
            Self::Burst32 => "32 / frame",
        }
    }
}

/// Maximum rendered entities (nearest-first priority so nearby combat targets never vanish).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityCap {
    #[default]
    Unlimited,
    Cap200,
    Cap128,
    Cap96,
    Cap64,
    Cap32,
}

impl EntityCap {
    pub const ALL: [Self; 6] = [
        Self::Unlimited,
        Self::Cap200,
        Self::Cap128,
        Self::Cap96,
        Self::Cap64,
        Self::Cap32,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn limit(self) -> Option<usize> {
        match self {
            Self::Unlimited => None,
            Self::Cap200 => Some(200),
            Self::Cap128 => Some(128),
            Self::Cap96 => Some(96),
            Self::Cap64 => Some(64),
            Self::Cap32 => Some(32),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unlimited => "unlimited",
            Self::Cap200 => "200",
            Self::Cap128 => "128",
            Self::Cap96 => "96",
            Self::Cap64 => "64",
            Self::Cap32 => "32",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "unlimited" | "off" => Some(Self::Unlimited),
            "200" => Some(Self::Cap200),
            "128" => Some(Self::Cap128),
            "96" => Some(Self::Cap96),
            "64" => Some(Self::Cap64),
            "32" => Some(Self::Cap32),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Unlimited => "Unlimited",
            Self::Cap200 => "200 Max",
            Self::Cap128 => "128 Max",
            Self::Cap96 => "96 Max",
            Self::Cap64 => "64 Max",
            Self::Cap32 => "32 Max",
        }
    }
}

/// Dropped ground-item rendering cap & distance cutoffs for farm/explosion drops.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemRenderCap {
    #[default]
    Unlimited,
    Cap128,
    Cap64,
    Cap32,
}

impl ItemRenderCap {
    pub const ALL: [Self; 4] = [Self::Unlimited, Self::Cap128, Self::Cap64, Self::Cap32];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn limit_and_dist_sq(self) -> Option<(usize, f64)> {
        match self {
            Self::Unlimited => None,
            Self::Cap128 => Some((128, 40.0 * 40.0)),
            Self::Cap64 => Some((64, 28.0 * 28.0)),
            Self::Cap32 => Some((32, 20.0 * 20.0)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unlimited => "unlimited",
            Self::Cap128 => "128",
            Self::Cap64 => "64",
            Self::Cap32 => "32",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "unlimited" | "off" => Some(Self::Unlimited),
            "128" => Some(Self::Cap128),
            "64" => Some(Self::Cap64),
            "32" => Some(Self::Cap32),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Unlimited => "Unlimited",
            Self::Cap128 => "128 (40m)",
            Self::Cap64 => "64 (28m)",
            Self::Cap32 => "32 (20m)",
        }
    }
}

/// Chest / sign / banner / skull / shulker block-entity render cap + wall occlusion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockEntityCap {
    #[default]
    Unlimited,
    Cap64,
    Cap32,
    Cap16,
}

impl BlockEntityCap {
    pub const ALL: [Self; 4] = [Self::Unlimited, Self::Cap64, Self::Cap32, Self::Cap16];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn limit_and_dist_sq(self) -> Option<(usize, f64)> {
        match self {
            Self::Unlimited => None,
            Self::Cap64 => Some((64, 48.0 * 48.0)),
            Self::Cap32 => Some((32, 32.0 * 32.0)),
            Self::Cap16 => Some((16, 24.0 * 24.0)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unlimited => "unlimited",
            Self::Cap64 => "64",
            Self::Cap32 => "32",
            Self::Cap16 => "16",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "unlimited" | "off" => Some(Self::Unlimited),
            "64" => Some(Self::Cap64),
            "32" => Some(Self::Cap32),
            "16" => Some(Self::Cap16),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Unlimited => "Unlimited",
            Self::Cap64 => "64 + Occlude",
            Self::Cap32 => "32 + Occlude",
            Self::Cap16 => "16 + Occlude",
        }
    }
}

/// Entity contact-shadow simplification mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FastShadowMode {
    #[default]
    Vanilla,
    Nearby16,
    Off,
}

impl FastShadowMode {
    pub const ALL: [Self; 3] = [Self::Vanilla, Self::Nearby16, Self::Off];

    pub fn next(self) -> Self {
        match self {
            Self::Vanilla => Self::Nearby16,
            Self::Nearby16 => Self::Off,
            Self::Off => Self::Vanilla,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Nearby16 => "nearby_16",
            Self::Off => "off",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "vanilla" => Some(Self::Vanilla),
            "nearby_16" | "nearby" => Some(Self::Nearby16),
            "off" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Vanilla => "volt.boost.shadows.vanilla",
            Self::Nearby16 => "volt.boost.shadows.nearby",
            Self::Off => "options.off",
        }
    }
}

/// Active particle engine cap & distance cutoff.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticleBudget {
    #[default]
    Vanilla16k,
    Cap4096,
    Cap2048,
    Cap512,
    Off,
}

impl ParticleBudget {
    pub const ALL: [Self; 5] = [
        Self::Vanilla16k,
        Self::Cap4096,
        Self::Cap2048,
        Self::Cap512,
        Self::Off,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn max_particles(self) -> usize {
        match self {
            Self::Vanilla16k => 16_384,
            Self::Cap4096 => 4_096,
            Self::Cap2048 => 2_048,
            Self::Cap512 => 512,
            Self::Off => 0,
        }
    }

    pub fn max_distance_sq(self) -> Option<f64> {
        match self {
            Self::Vanilla16k => None,
            Self::Cap4096 => Some(36.0 * 36.0),
            Self::Cap2048 => Some(24.0 * 24.0),
            Self::Cap512 => Some(16.0 * 16.0),
            Self::Off => Some(0.0),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla16k => "16384",
            Self::Cap4096 => "4096",
            Self::Cap2048 => "2048",
            Self::Cap512 => "512",
            Self::Off => "0",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "16384" | "vanilla" => Some(Self::Vanilla16k),
            "4096" => Some(Self::Cap4096),
            "2048" => Some(Self::Cap2048),
            "512" => Some(Self::Cap512),
            "0" | "off" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla16k => "16K (Vanilla)",
            Self::Cap4096 => "4,096 (36m)",
            Self::Cap2048 => "2,048 (24m)",
            Self::Cap512 => "512 (16m)",
            Self::Off => "OFF",
        }
    }
}

/// Enchantment glint & armor-trim shader simplification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlintTrimMode {
    #[default]
    Full,
    NearbyOnly,
    Off,
}

impl GlintTrimMode {
    pub const ALL: [Self; 3] = [Self::Full, Self::NearbyOnly, Self::Off];

    pub fn next(self) -> Self {
        match self {
            Self::Full => Self::NearbyOnly,
            Self::NearbyOnly => Self::Off,
            Self::Off => Self::Full,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::NearbyOnly => "nearby",
            Self::Off => "off",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "full" => Some(Self::Full),
            "nearby" => Some(Self::NearbyOnly),
            "off" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Full => "volt.boost.glint.full",
            Self::NearbyOnly => "volt.boost.glint.nearby",
            Self::Off => "options.off",
        }
    }
}

/// Automatic Dynamic Resolution Scaling (scales 3D scene while keeping HUD 100% native crisp).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoDrsMode {
    #[default]
    Off,
    Target60,
    Target120,
    Aggressive50,
}

impl AutoDrsMode {
    pub const ALL: [Self; 4] = [
        Self::Off,
        Self::Target60,
        Self::Target120,
        Self::Aggressive50,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn target_and_floor(self) -> Option<(f32, f32)> {
        match self {
            Self::Off => None,
            Self::Target60 => Some((60.0, 0.65)),
            Self::Target120 => Some((120.0, 0.60)),
            Self::Aggressive50 => Some((60.0, 0.50)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Target60 => "target_60",
            Self::Target120 => "target_120",
            Self::Aggressive50 => "aggressive_50",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Self::Off),
            "target_60" => Some(Self::Target60),
            "target_120" => Some(Self::Target120),
            "aggressive_50" => Some(Self::Aggressive50),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Target60 => "60 FPS (65-100%)",
            Self::Target120 => "120 FPS (60-100%)",
            Self::Aggressive50 => "Ultra (50-100%)",
        }
    }
}

/// Dynamic FPS cap when paused, in menus, or AFK-idle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicIdleFps {
    #[default]
    Off,
    Fps30,
    Fps15,
}

impl DynamicIdleFps {
    pub const ALL: [Self; 3] = [Self::Off, Self::Fps30, Self::Fps15];

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Fps30,
            Self::Fps30 => Self::Fps15,
            Self::Fps15 => Self::Off,
        }
    }

    pub fn cap_fps(self) -> Option<u32> {
        match self {
            Self::Off => None,
            Self::Fps30 => Some(30),
            Self::Fps15 => Some(15),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Fps30 => "30",
            Self::Fps15 => "15",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Self::Off),
            "30" => Some(Self::Fps30),
            "15" => Some(Self::Fps15),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Fps30 => "30 FPS Idle",
            Self::Fps15 => "15 FPS Eco",
        }
    }
}

/// Pause / menu background blur saver (skips multi-pass Kawase blur on mobile tile GPUs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MenuBgMode {
    #[default]
    PanoramaBlur,
    FastDark,
}

impl MenuBgMode {
    pub const ALL: [Self; 2] = [Self::PanoramaBlur, Self::FastDark];

    pub fn next(self) -> Self {
        match self {
            Self::PanoramaBlur => Self::FastDark,
            Self::FastDark => Self::PanoramaBlur,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PanoramaBlur => "blur",
            Self::FastDark => "fast_dark",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "blur" | "panorama" => Some(Self::PanoramaBlur),
            "fast_dark" | "fast" => Some(Self::FastDark),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::PanoramaBlur => "volt.boost.menu_bg.blur",
            Self::FastDark => "volt.boost.menu_bg.fast_dark",
        }
    }
}

/// Active simultaneous audio mixer voice cap to avoid CPU mixer spikes in mob farms/TNT.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioVoiceCap {
    #[default]
    Vanilla,
    Cap32,
    Cap16,
}

impl AudioVoiceCap {
    pub const ALL: [Self; 3] = [Self::Vanilla, Self::Cap32, Self::Cap16];

    pub fn next(self) -> Self {
        match self {
            Self::Vanilla => Self::Cap32,
            Self::Cap32 => Self::Cap16,
            Self::Cap16 => Self::Vanilla,
        }
    }

    pub fn max_per_tick(self) -> Option<usize> {
        match self {
            Self::Vanilla => None,
            Self::Cap32 => Some(8),
            Self::Cap16 => Some(4),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Cap32 => "32",
            Self::Cap16 => "16",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "vanilla" | "off" => Some(Self::Vanilla),
            "32" => Some(Self::Cap32),
            "16" => Some(Self::Cap16),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Cap32 => "32 Voices",
            Self::Cap16 => "16 Voices",
        }
    }
}

