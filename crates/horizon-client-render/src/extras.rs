//! Volt's optional extras (waypoints, minimap, accurate block placement,
//! lighting, inventory tools): their settings, kept in `horizon-extras.txt`
//! next to `options.txt` so that file stays exactly vanilla's, and the logic
//! behind them. Every gameplay extra is off by default.
use crate::volt_features::{
    MinimapEntities, MinimapFrame, MinimapTerrainMode, PlacementLock, PlacementProtocol, PlaneLock,
    WaypointColor,
};
use std::{collections::VecDeque, fs, path::Path};

pub const FILE: &str = "horizon-extras.txt";
/// The names the options screen uses while an extra keybind waits for a key.
pub const SORT_KEY: &str = "horizon.sort";
pub const WAYPOINT_KEY: &str = "volt.waypoint_add";
pub const MINIMAP_ZOOM_KEY: &str = "volt.minimap_zoom";
pub const PLACEMENT_LOCK_KEY: &str = "volt.placement_lock";
pub const PLACEMENT_REVERSE_KEY: &str = "volt.placement_reverse";

/// The minimap's sizes in GUI pixels, and its zooms in blocks per pixel.
pub const MINIMAP_SIZES: [(u8, &str); 4] = [
    (64, "Small"),
    (96, "Medium"),
    (128, "Large"),
    (160, "XL"),
];
pub const MINIMAP_ZOOMS: [f32; 4] = [0.5, 1.0, 2.0, 4.0];
pub const MINIMAP_OPACITIES: [u8; 4] = [50, 75, 90, 100];
pub const WAYPOINT_SCALES: [u8; 3] = [75, 100, 125];
pub const WAYPOINT_DISTANCES: [u16; 5] = [0, 256, 512, 1024, 2048];
pub const PLACEMENT_DELAYS: [u8; 4] = [4, 2, 1, 0];

/// A screen corner for an overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
impl Corner {
    pub const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Corner::TopLeft => "top_left",
            Corner::TopRight => "top_right",
            Corner::BottomLeft => "bottom_left",
            Corner::BottomRight => "bottom_right",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Corner::TopLeft => "Top left",
            Corner::TopRight => "Top right",
            Corner::BottomLeft => "Bottom left",
            Corner::BottomRight => "Bottom right",
        }
    }
    fn parse(s: &str, default: Corner) -> Corner {
        Self::ALL
            .into_iter()
            .find(|c| c.name() == s)
            .unwrap_or(default)
    }
    pub fn cycle(self, back: bool) -> Corner {
        let i = Self::ALL.iter().position(|&c| c == self).unwrap_or(0);
        Self::ALL[if back {
            (i + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (i + 1) % Self::ALL.len()
        }]
    }
    pub fn top(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::TopRight)
    }
    pub fn left(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::BottomLeft)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Extras {
    pub fullbright: bool,
    /// Held, dropped and burning light sources light their surroundings (`dynamic_light`).
    pub dynamic_lights: bool,
    pub zoom: bool,
    /// A vanilla key name (`key.keyboard.z`).
    pub zoom_key: String,
    /// The zoom the key starts at.
    pub zoom_level: u8,
    /// Ease through vanilla's FOV easing (as the spyglass does).
    pub zoom_smooth: bool,
    /// Divide mouse turning by the zoom.
    pub zoom_sensitivity: bool,
    /// Legacy FPS counter / keystrokes extras (`moved_extras`).
    pub fps: bool,
    pub fps_corner: Corner,
    pub fps_lows: bool,
    pub keystrokes: bool,
    pub keystrokes_corner: Corner,
    pub keystrokes_cps: bool,
    pub moved: Vec<String>,
    // --- Volt Minimap (off by default) ---
    pub minimap: bool,
    pub minimap_corner: Corner,
    /// Side in GUI pixels (`MINIMAP_SIZES`).
    pub minimap_size: u8,
    pub minimap_round: bool,
    /// Blocks per pixel (`MINIMAP_ZOOMS`).
    pub minimap_zoom: f32,
    /// Turn with the player (forward up) instead of north up.
    pub minimap_rotate: bool,
    pub minimap_frame: MinimapFrame,
    pub minimap_opacity: u8,
    pub minimap_terrain_mode: MinimapTerrainMode,
    pub minimap_chunk_grid: bool,
    pub minimap_entities: MinimapEntities,
    pub minimap_coords: bool,
    pub minimap_biome: bool,
    pub minimap_time: bool,
    pub minimap_zoom_key: String,
    // --- Volt Waypoints (off by default) ---
    pub waypoints: bool,
    pub waypoint_beams: bool,
    pub waypoint_labels: bool,
    pub waypoint_distance: bool,
    pub waypoint_coords: bool,
    pub waypoint_scale: u8,
    pub waypoint_max_distance: u16,
    pub waypoint_edge_arrows: bool,
    pub waypoint_auto_death: bool,
    pub waypoint_dimension_sync: bool,
    pub waypoint_on_minimap: bool,
    pub waypoint_hud_button: bool,
    pub waypoint_new_color: WaypointColor,
    pub waypoint_key: String,
    // --- Volt Accurate Block Placement (off by default) ---
    pub accurate_placement: bool,
    pub placement_quadrant: bool,
    pub placement_reverse: bool,
    pub placement_lock: PlacementLock,
    pub placement_plane_lock: PlaneLock,
    pub placement_reach_around: bool,
    pub placement_fast_delay: u8,
    pub placement_hud_preview: bool,
    pub placement_protocol: PlacementProtocol,
    pub placement_rotation_sync: bool,
    pub placement_lock_key: String,
    pub placement_reverse_key: String,
    // --- Other client extras ---
    pub auto_reconnect: bool,
    pub own_name_tag: bool,
    pub mouse_tweaks: bool,
    pub mouse_tweaks_wheel: bool,
    pub sort: bool,
    pub sort_key: String,
    pub no_ai: bool,
    pub agent_control: bool,
    pub discord_presence: bool,
}

impl Default for Extras {
    fn default() -> Self {
        Self {
            fullbright: false,
            dynamic_lights: false,
            zoom: false,
            zoom_key: "key.keyboard.z".into(),
            zoom_level: 4,
            zoom_smooth: true,
            zoom_sensitivity: true,
            fps: false,
            fps_corner: Corner::TopLeft,
            fps_lows: false,
            keystrokes: false,
            keystrokes_corner: Corner::TopLeft,
            keystrokes_cps: true,
            moved: Vec::new(),
            // Master toggles MUST be off by default:
            minimap: false,
            minimap_corner: Corner::TopRight,
            minimap_size: 96,
            minimap_round: false,
            minimap_zoom: 1.0,
            minimap_rotate: false,
            minimap_frame: MinimapFrame::VoltCrimson,
            minimap_opacity: 100,
            minimap_terrain_mode: MinimapTerrainMode::Auto,
            minimap_chunk_grid: false,
            minimap_entities: MinimapEntities::All,
            minimap_coords: true,
            minimap_biome: true,
            minimap_time: false,
            minimap_zoom_key: "key.keyboard.unknown".into(),
            waypoints: false,
            waypoint_beams: true,
            waypoint_labels: true,
            waypoint_distance: true,
            waypoint_coords: false,
            waypoint_scale: 100,
            waypoint_max_distance: 0,
            waypoint_edge_arrows: true,
            waypoint_auto_death: true,
            waypoint_dimension_sync: true,
            waypoint_on_minimap: true,
            waypoint_hud_button: false,
            waypoint_new_color: WaypointColor::Crimson,
            waypoint_key: "key.keyboard.b".into(),
            accurate_placement: false,
            placement_quadrant: true,
            placement_reverse: false,
            placement_lock: PlacementLock::Off,
            placement_plane_lock: PlaneLock::Off,
            placement_reach_around: true,
            placement_fast_delay: 4,
            placement_hud_preview: true,
            placement_protocol: PlacementProtocol::VanillaSafe,
            placement_rotation_sync: true,
            placement_lock_key: "key.keyboard.k".into(),
            placement_reverse_key: "key.keyboard.v".into(),
            auto_reconnect: false,
            own_name_tag: false,
            mouse_tweaks: false,
            mouse_tweaks_wheel: true,
            sort: false,
            sort_key: "key.keyboard.unknown".into(),
            no_ai: false,
            agent_control: false,
            discord_presence: true,
        }
    }
}

fn bool_text(v: bool) -> &'static str {
    if v { "true" } else { "false" }
}

impl Extras {
    pub fn load(dir: &Path) -> Self {
        fs::read_to_string(dir.join(FILE))
            .map(|t| Self::parse(&t))
            .unwrap_or_default()
    }
    pub fn parse(text: &str) -> Self {
        let mut e = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.trim();
            let flag = |d: bool| match value {
                "true" => true,
                "false" => false,
                _ => d,
            };
            match key.trim() {
                "fullbright" => e.fullbright = flag(e.fullbright),
                "dynamicLights" => e.dynamic_lights = flag(e.dynamic_lights),
                "zoom" => e.zoom = flag(e.zoom),
                "zoomKey" if value.starts_with("key.") => e.zoom_key = value.to_owned(),
                "zoomLevel" => {
                    if let Ok(v) = value.parse::<u8>() {
                        e.zoom_level = v.clamp(2, 10);
                    }
                }
                "zoomSmooth" => e.zoom_smooth = flag(e.zoom_smooth),
                "zoomSensitivity" => e.zoom_sensitivity = flag(e.zoom_sensitivity),
                "fps" => e.fps = flag(e.fps),
                "fpsCorner" => e.fps_corner = Corner::parse(value, e.fps_corner),
                "fpsLows" => e.fps_lows = flag(e.fps_lows),
                "keystrokes" => e.keystrokes = flag(e.keystrokes),
                "keystrokesCorner" => {
                    e.keystrokes_corner = Corner::parse(value, e.keystrokes_corner)
                }
                "keystrokesCps" => e.keystrokes_cps = flag(e.keystrokes_cps),
                "movedToMods" => {
                    e.moved = value
                        .split(',')
                        .map(str::trim)
                        .filter(|id| !id.is_empty())
                        .map(str::to_owned)
                        .collect()
                }
                // Minimap:
                "minimap" => e.minimap = flag(e.minimap),
                "minimapCorner" => e.minimap_corner = Corner::parse(value, e.minimap_corner),
                "minimapSize" => {
                    if let Some(&(v, _)) = MINIMAP_SIZES.iter().find(|s| s.0.to_string() == value) {
                        e.minimap_size = v;
                    }
                }
                "minimapShape" => e.minimap_round = value == "round",
                "minimapZoom" => {
                    if let Some(&z) = MINIMAP_ZOOMS.iter().find(|z| z.to_string() == value) {
                        e.minimap_zoom = z;
                    }
                }
                "minimapRotate" => e.minimap_rotate = flag(e.minimap_rotate),
                "minimapFrame" => e.minimap_frame = MinimapFrame::parse(value),
                "minimapOpacity" => {
                    if let Some(&op) = MINIMAP_OPACITIES.iter().find(|o| o.to_string() == value) {
                        e.minimap_opacity = op;
                    }
                }
                "minimapTerrain" => e.minimap_terrain_mode = MinimapTerrainMode::parse(value),
                "minimapChunkGrid" => e.minimap_chunk_grid = flag(e.minimap_chunk_grid),
                "minimapEntities" => e.minimap_entities = MinimapEntities::parse(value),
                "minimapCoords" => e.minimap_coords = flag(e.minimap_coords),
                "minimapBiome" => e.minimap_biome = flag(e.minimap_biome),
                "minimapTime" => e.minimap_time = flag(e.minimap_time),
                "minimapZoomKey" if value.starts_with("key.") => {
                    e.minimap_zoom_key = value.to_owned()
                }
                // Waypoints:
                "waypoints" => e.waypoints = flag(e.waypoints),
                "waypointBeams" => e.waypoint_beams = flag(e.waypoint_beams),
                "waypointLabels" => e.waypoint_labels = flag(e.waypoint_labels),
                "waypointDistance" => e.waypoint_distance = flag(e.waypoint_distance),
                "waypointCoords" => e.waypoint_coords = flag(e.waypoint_coords),
                "waypointScale" => {
                    if let Some(&sc) = WAYPOINT_SCALES.iter().find(|s| s.to_string() == value) {
                        e.waypoint_scale = sc;
                    }
                }
                "waypointMaxDistance" => {
                    if let Some(&d) = WAYPOINT_DISTANCES.iter().find(|d| d.to_string() == value) {
                        e.waypoint_max_distance = d;
                    }
                }
                "waypointEdgeArrows" => e.waypoint_edge_arrows = flag(e.waypoint_edge_arrows),
                "waypointAutoDeath" => e.waypoint_auto_death = flag(e.waypoint_auto_death),
                "waypointDimensionSync" => {
                    e.waypoint_dimension_sync = flag(e.waypoint_dimension_sync)
                }
                "waypointOnMinimap" => e.waypoint_on_minimap = flag(e.waypoint_on_minimap),
                "waypointHudButton" => e.waypoint_hud_button = flag(e.waypoint_hud_button),
                "waypointNewColor" => {
                    if let Some(&c) = WaypointColor::ALL
                        .iter()
                        .find(|c| c.label().eq_ignore_ascii_case(value))
                    {
                        e.waypoint_new_color = c;
                    }
                }
                "waypointKey" if value.starts_with("key.") => e.waypoint_key = value.to_owned(),
                // Accurate Block Placement:
                "accuratePlacement" => e.accurate_placement = flag(e.accurate_placement),
                "placementQuadrant" => e.placement_quadrant = flag(e.placement_quadrant),
                "placementReverse" => e.placement_reverse = flag(e.placement_reverse),
                "placementLock" => e.placement_lock = PlacementLock::parse(value),
                "placementPlaneLock" => e.placement_plane_lock = PlaneLock::parse(value),
                "placementReachAround" => e.placement_reach_around = flag(e.placement_reach_around),
                "placementFastDelay" => {
                    if let Some(&d) = PLACEMENT_DELAYS.iter().find(|d| d.to_string() == value) {
                        e.placement_fast_delay = d;
                    }
                }
                "placementHudPreview" => e.placement_hud_preview = flag(e.placement_hud_preview),
                "placementProtocol" => e.placement_protocol = PlacementProtocol::parse(value),
                "placementRotationSync" => {
                    e.placement_rotation_sync = flag(e.placement_rotation_sync)
                }
                "placementLockKey" if value.starts_with("key.") => {
                    e.placement_lock_key = value.to_owned()
                }
                "placementReverseKey" if value.starts_with("key.") => {
                    e.placement_reverse_key = value.to_owned()
                }
                // General:
                "autoReconnect" => e.auto_reconnect = flag(e.auto_reconnect),
                "ownNameTag" => e.own_name_tag = flag(e.own_name_tag),
                "mouseTweaks" => e.mouse_tweaks = flag(e.mouse_tweaks),
                "mouseTweaksWheel" => e.mouse_tweaks_wheel = flag(e.mouse_tweaks_wheel),
                "sort" => e.sort = flag(e.sort),
                "sortKey" if value.starts_with("key.") => e.sort_key = value.to_owned(),
                horizon_core::no_ai::KEY => e.no_ai = flag(e.no_ai),
                "agentControl" => e.agent_control = flag(e.agent_control),
                "discordPresence" => e.discord_presence = flag(e.discord_presence),
                _ => {}
            }
        }
        e
    }
    pub fn serialize(&self) -> String {
        let mut s = String::with_capacity(1024);
        let mut push = |k: &str, v: &str| {
            s.push_str(k);
            s.push(':');
            s.push_str(v);
            s.push('\n');
        };
        push("fullbright", bool_text(self.fullbright));
        push("dynamicLights", bool_text(self.dynamic_lights));
        push("zoom", bool_text(self.zoom));
        push("zoomKey", &self.zoom_key);
        push("zoomLevel", &self.zoom_level.to_string());
        push("zoomSmooth", bool_text(self.zoom_smooth));
        push("zoomSensitivity", bool_text(self.zoom_sensitivity));
        push("fps", bool_text(self.fps));
        push("fpsCorner", self.fps_corner.name());
        push("fpsLows", bool_text(self.fps_lows));
        push("keystrokes", bool_text(self.keystrokes));
        push("keystrokesCorner", self.keystrokes_corner.name());
        push("keystrokesCps", bool_text(self.keystrokes_cps));
        push("movedToMods", &self.moved.join(","));
        push("minimap", bool_text(self.minimap));
        push("minimapCorner", self.minimap_corner.name());
        push("minimapSize", &self.minimap_size.to_string());
        push(
            "minimapShape",
            if self.minimap_round {
                "round"
            } else {
                "square"
            },
        );
        push("minimapZoom", &self.minimap_zoom.to_string());
        push("minimapRotate", bool_text(self.minimap_rotate));
        push("minimapFrame", self.minimap_frame.name());
        push("minimapOpacity", &self.minimap_opacity.to_string());
        push("minimapTerrain", self.minimap_terrain_mode.name());
        push("minimapChunkGrid", bool_text(self.minimap_chunk_grid));
        push("minimapEntities", self.minimap_entities.name());
        push("minimapCoords", bool_text(self.minimap_coords));
        push("minimapBiome", bool_text(self.minimap_biome));
        push("minimapTime", bool_text(self.minimap_time));
        push("minimapZoomKey", &self.minimap_zoom_key);
        push("waypoints", bool_text(self.waypoints));
        push("waypointBeams", bool_text(self.waypoint_beams));
        push("waypointLabels", bool_text(self.waypoint_labels));
        push("waypointDistance", bool_text(self.waypoint_distance));
        push("waypointCoords", bool_text(self.waypoint_coords));
        push("waypointScale", &self.waypoint_scale.to_string());
        push(
            "waypointMaxDistance",
            &self.waypoint_max_distance.to_string(),
        );
        push("waypointEdgeArrows", bool_text(self.waypoint_edge_arrows));
        push("waypointAutoDeath", bool_text(self.waypoint_auto_death));
        push(
            "waypointDimensionSync",
            bool_text(self.waypoint_dimension_sync),
        );
        push("waypointOnMinimap", bool_text(self.waypoint_on_minimap));
        push("waypointHudButton", bool_text(self.waypoint_hud_button));
        push("waypointNewColor", self.waypoint_new_color.label());
        push("waypointKey", &self.waypoint_key);
        push("accuratePlacement", bool_text(self.accurate_placement));
        push("placementQuadrant", bool_text(self.placement_quadrant));
        push("placementReverse", bool_text(self.placement_reverse));
        push("placementLock", self.placement_lock.name());
        push("placementPlaneLock", self.placement_plane_lock.name());
        push(
            "placementReachAround",
            bool_text(self.placement_reach_around),
        );
        push("placementFastDelay", &self.placement_fast_delay.to_string());
        push("placementHudPreview", bool_text(self.placement_hud_preview));
        push("placementProtocol", self.placement_protocol.name());
        push(
            "placementRotationSync",
            bool_text(self.placement_rotation_sync),
        );
        push("placementLockKey", &self.placement_lock_key);
        push("placementReverseKey", &self.placement_reverse_key);
        push("autoReconnect", bool_text(self.auto_reconnect));
        push("ownNameTag", bool_text(self.own_name_tag));
        push("mouseTweaks", bool_text(self.mouse_tweaks));
        push("mouseTweaksWheel", bool_text(self.mouse_tweaks_wheel));
        push("sort", bool_text(self.sort));
        push("sortKey", &self.sort_key);
        push(horizon_core::no_ai::KEY, bool_text(self.no_ai));
        push("agentControl", bool_text(self.agent_control));
        push("discordPresence", bool_text(self.discord_presence));
        s
    }
    /// Binds an extra key (`SORT_KEY`, `WAYPOINT_KEY`, `MINIMAP_ZOOM_KEY`,
    /// `PLACEMENT_LOCK_KEY`, `PLACEMENT_REVERSE_KEY`); false for vanilla keys.
    pub fn set_key(&mut self, name: &str, value: String) -> bool {
        match name {
            SORT_KEY => self.sort_key = value,
            WAYPOINT_KEY => self.waypoint_key = value,
            MINIMAP_ZOOM_KEY => self.minimap_zoom_key = value,
            PLACEMENT_LOCK_KEY => self.placement_lock_key = value,
            PLACEMENT_REVERSE_KEY => self.placement_reverse_key = value,
            _ => return false,
        }
        true
    }
    /// Written only once something differs from the defaults, so a vanilla
    /// setup gains no file.
    pub fn save(&self, dir: &Path) {
        let path = dir.join(FILE);
        if *self == Self::default() && !path.exists() {
            return;
        }
        let _ = fs::write(path, self.serialize());
    }
}

/// The field-of-view modifier target `Camera.tickFov` eases toward: vanilla's
/// (`getFieldOfViewModifier`) divided by a mod's zoom (`render.set-zoom`). A
/// zoom of 1 leaves it as is.
pub fn fov_target(vanilla: f32, zoom: f32) -> f32 {
    if zoom == 1.0 { vanilla } else { vanilla / zoom }
}
/// Frame times over the last few seconds: frames per second, the minimum
/// and the 1% low, recomputed twice a second.
pub struct FpsCounter {
    /// (end time in seconds, frame milliseconds), oldest first.
    frames: VecDeque<(f64, f32)>,
    shown: FpsReading,
    computed_at: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FpsReading {
    pub fps: u32,
    pub min: u32,
    pub low: u32,
}
/// Seconds the minimum and 1% low look back.
const LOW_WINDOW: f64 = 5.0;
impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            frames: VecDeque::new(),
            shown: FpsReading::default(),
            computed_at: f64::NEG_INFINITY,
        }
    }
}
impl FpsCounter {
    /// A frame that ended at `now` (seconds) and took `ms`.
    pub fn record(&mut self, now: f64, ms: f32) {
        self.frames.push_back((now, ms));
        while self
            .frames
            .front()
            .is_some_and(|&(t, _)| now - t > LOW_WINDOW)
        {
            self.frames.pop_front();
        }
        if now - self.computed_at >= 0.5 {
            self.computed_at = now;
            self.shown = self.compute(now);
        }
    }
    fn compute(&self, now: f64) -> FpsReading {
        let fps = self.frames.iter().filter(|&&(t, _)| now - t < 1.0).count() as u32;
        let mut times: Vec<f32> = self.frames.iter().map(|f| f.1).collect();
        if times.is_empty() {
            return FpsReading::default();
        }
        times.sort_by(|a, b| b.total_cmp(a));
        let to_fps = |ms: f32| (1000.0 / ms.max(0.001)).round() as u32;
        // The 1% low: the frame rate of the 99th percentile frame time.
        let low = times[(times.len() - 1) / 100];
        FpsReading {
            fps,
            min: to_fps(times[0]),
            low: to_fps(low),
        }
    }
    pub fn reading(&self) -> FpsReading {
        self.shown
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Clicks of one button in the last second.
#[derive(Default)]
pub struct Cps {
    clicks: VecDeque<f64>,
}
impl Cps {
    pub fn click(&mut self, now: f64) {
        self.clicks.push_back(now);
        self.forget(now);
    }
    fn forget(&mut self, now: f64) {
        while self.clicks.front().is_some_and(|&t| now - t >= 1.0) {
            self.clicks.pop_front();
        }
    }
    pub fn per_second(&mut self, now: f64) -> usize {
        self.forget(now);
        self.clicks.len()
    }
}

/// Every configurable Volt Settings / Extras option item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Fullbright,
    DynamicLights,
    OwnNameTag,
    AutoReconnect,
    MouseTweaks,
    MouseTweaksWheel,
    Sort,
    SortKey,
    /// Volt Settings switches:
    NoAi,
    AgentControl,
    DiscordPresence,
    /// Volt Waypoints page:
    Waypoints,
    WaypointKey,
    WaypointAutoDeath,
    WaypointDimensionSync,
    WaypointBeams,
    WaypointLabels,
    WaypointDistance,
    WaypointCoords,
    WaypointEdgeArrows,
    WaypointOnMinimap,
    WaypointScale,
    WaypointMaxDistance,
    WaypointHudButton,
    WaypointNewColor,
    /// Volt Minimap page:
    Minimap,
    MinimapCorner,
    MinimapSize,
    MinimapShape,
    MinimapZoom,
    MinimapRotate,
    MinimapFrameStyle,
    MinimapOpacity,
    MinimapTerrain,
    MinimapChunkGrid,
    MinimapEntityRadar,
    MinimapCoords,
    MinimapBiome,
    MinimapTime,
    MinimapZoomKey,
    /// Volt Accurate Block Placement page:
    AccuratePlacement,
    PlacementQuadrant,
    PlacementReverse,
    PlacementLockMode,
    PlacementPlaneMode,
    PlacementReachAround,
    PlacementFastDelay,
    PlacementHudPreview,
    PlacementServerProtocol,
    PlacementRotationSync,
    PlacementLockKey,
    PlacementReverseKey,
}

/// The Volt Extras page: headers and items.
pub fn rows() -> Vec<Result<Item, &'static str>> {
    use Item::*;
    vec![
        Err("Lighting"),
        Ok(Fullbright),
        Ok(DynamicLights),
        Err("Third person"),
        Ok(OwnNameTag),
        Err("Inventory"),
        Ok(MouseTweaks),
        Ok(MouseTweaksWheel),
        Ok(Sort),
        Ok(SortKey),
        Err("Multiplayer"),
        Ok(AutoReconnect),
    ]
}

/// The Volt Waypoints settings rows (followed in the UI by the Waypoint Manager).
pub fn waypoint_rows() -> Vec<Result<Item, &'static str>> {
    use Item::*;
    vec![
        Err("Waypoints"),
        Ok(Waypoints),
        Ok(WaypointKey),
        Ok(WaypointAutoDeath),
        Ok(WaypointDimensionSync),
        Ok(WaypointHudButton),
        Ok(WaypointOnMinimap),
        Err("In-World Markers & Beams"),
        Ok(WaypointBeams),
        Ok(WaypointLabels),
        Ok(WaypointDistance),
        Ok(WaypointCoords),
        Ok(WaypointEdgeArrows),
        Ok(WaypointScale),
        Ok(WaypointMaxDistance),
    ]
}

/// The Volt Minimap ("Minimal with many features") settings rows.
pub fn minimap_rows() -> Vec<Result<Item, &'static str>> {
    use Item::*;
    vec![
        Err("Minimal Minimap"),
        Ok(Minimap),
        Ok(MinimapCorner),
        Ok(MinimapSize),
        Ok(MinimapShape),
        Ok(MinimapZoom),
        Ok(MinimapRotate),
        Ok(MinimapFrameStyle),
        Ok(MinimapOpacity),
        Ok(MinimapZoomKey),
        Ok(MinimapChunkGrid),
        Err("Terrain, Radar & Readouts"),
        Ok(MinimapTerrain),
        Ok(MinimapEntityRadar),
        Ok(WaypointOnMinimap),
        Ok(MinimapCoords),
        Ok(MinimapBiome),
        Ok(MinimapTime),
    ]
}

/// The Volt Accurate Block Placement settings rows.
pub fn placement_rows() -> Vec<Result<Item, &'static str>> {
    use Item::*;
    vec![
        Err("Accurate Block Placement"),
        Ok(AccuratePlacement),
        Ok(PlacementQuadrant),
        Ok(PlacementReverse),
        Ok(PlacementLockMode),
        Ok(PlacementPlaneMode),
        Ok(PlacementReachAround),
        Ok(PlacementFastDelay),
        Ok(PlacementHudPreview),
        Err("Multiplayer & Hotkeys"),
        Ok(PlacementServerProtocol),
        Ok(PlacementRotationSync),
        Ok(PlacementLockKey),
        Ok(PlacementReverseKey),
    ]
}

impl Item {
    pub fn caption(self) -> &'static str {
        match self {
            Item::Fullbright => "Fullbright",
            Item::DynamicLights => "Dynamic lights",
            Item::OwnNameTag => "Own name tag",
            Item::AutoReconnect => "Auto-reconnect",
            Item::MouseTweaks => "Mouse Tweaks controls",
            Item::MouseTweaksWheel => "Scroll wheel moves items",
            Item::Sort => "Sort buttons",
            Item::SortKey => "Sort key",
            Item::NoAi => "No AI",
            Item::AgentControl => "Agent Control",
            Item::DiscordPresence => "Discord Rich Presence",
            // Waypoints:
            Item::Waypoints => "Waypoints",
            Item::WaypointKey => "Quick-Add Key",
            Item::WaypointAutoDeath => "Death Waypoints",
            Item::WaypointDimensionSync => "Nether 8x Sync",
            Item::WaypointBeams => "Beacon Beams",
            Item::WaypointLabels => "Floating Labels",
            Item::WaypointDistance => "Show Distance",
            Item::WaypointCoords => "Show XYZ",
            Item::WaypointEdgeArrows => "Edge Arrows",
            Item::WaypointOnMinimap => "Minimap Pins",
            Item::WaypointScale => "Marker Scale",
            Item::WaypointMaxDistance => "Max Render Dist",
            Item::WaypointHudButton => "HUD [+WP] Button",
            Item::WaypointNewColor => "New Pin Color",
            // Minimap:
            Item::Minimap => "Minimap",
            Item::MinimapCorner => "Corner",
            Item::MinimapSize => "Map Size",
            Item::MinimapShape => "Map Shape",
            Item::MinimapZoom => "Zoom Scale",
            Item::MinimapRotate => "Orientation",
            Item::MinimapFrameStyle => "Frame Style",
            Item::MinimapOpacity => "Map Opacity",
            Item::MinimapTerrain => "Terrain Layer",
            Item::MinimapChunkGrid => "Chunk Grid",
            Item::MinimapEntityRadar => "Entity Radar",
            Item::MinimapCoords => "XYZ Readout",
            Item::MinimapBiome => "Heading & Biome",
            Item::MinimapTime => "World Clock",
            Item::MinimapZoomKey => "Zoom Key",
            // Accurate Block Placement:
            Item::AccuratePlacement => "Accurate Placement",
            Item::PlacementQuadrant => "Sub-Face Aiming",
            Item::PlacementReverse => "Reverse Facing",
            Item::PlacementLockMode => "Orientation Lock",
            Item::PlacementPlaneMode => "Plane / Line Lock",
            Item::PlacementReachAround => "Reach-Around",
            Item::PlacementFastDelay => "Place Delay",
            Item::PlacementHudPreview => "Crosshair Badge",
            Item::PlacementServerProtocol => "MP Protocol",
            Item::PlacementRotationSync => "MP Look Sync",
            Item::PlacementLockKey => "Lock Key",
            Item::PlacementReverseKey => "Reverse Key",
        }
    }
    /// The switch this setting belongs to: it does nothing while that is off.
    pub fn parent(self) -> Option<Item> {
        use Item::*;
        Some(match self {
            MouseTweaksWheel => MouseTweaks,
            SortKey => Sort,
            WaypointKey
            | WaypointAutoDeath
            | WaypointDimensionSync
            | WaypointBeams
            | WaypointLabels
            | WaypointDistance
            | WaypointCoords
            | WaypointEdgeArrows
            | WaypointScale
            | WaypointMaxDistance
            | WaypointHudButton => Waypoints,
            MinimapCorner
            | MinimapSize
            | MinimapShape
            | MinimapZoom
            | MinimapRotate
            | MinimapFrameStyle
            | MinimapOpacity
            | MinimapTerrain
            | MinimapChunkGrid
            | MinimapEntityRadar
            | MinimapCoords
            | MinimapBiome
            | MinimapTime
            | MinimapZoomKey => Minimap,
            PlacementQuadrant
            | PlacementReverse
            | PlacementLockMode
            | PlacementPlaneMode
            | PlacementReachAround
            | PlacementFastDelay
            | PlacementHudPreview
            | PlacementServerProtocol
            | PlacementRotationSync
            | PlacementLockKey
            | PlacementReverseKey => AccuratePlacement,
            Fullbright | DynamicLights | OwnNameTag | MouseTweaks | Sort | AutoReconnect | NoAi
            | AgentControl | DiscordPresence | Waypoints | WaypointOnMinimap | WaypointNewColor
            | Minimap | AccuratePlacement => {
                return None;
            }
        })
    }
    /// What the row says in its hover tooltip.
    pub fn description(self) -> &'static str {
        match self {
            Item::Fullbright => "Show blocks and entities at full light, even in dark areas.",
            Item::DynamicLights => {
                "Torches and other light sources held, dropped or on fire light the blocks around them."
            }
            Item::OwnNameTag => {
                "Shows your name above your head in third person, as other players see it. Hidden when the server hides your team's names or you're invisible."
            }
            Item::AutoReconnect => {
                "Rejoins a server after a lost connection or a restart, never after a kick or ban."
            }
            Item::MouseTweaks => {
                "Right-drag places one per slot, left-drag gathers the held item, Shift + left-drag moves every slot."
            }
            Item::MouseTweaksWheel => {
                "Down sends one item of the slot to the other side, up brings one back."
            }
            Item::Sort => {
                "Sorts the inventory, chests, barrels, ender chests and shulker boxes in creative tab order, merging stacks."
            }
            Item::SortKey => {
                "Sorts the part under the cursor. Click, then press a key; Escape unbinds it."
            }
            Item::NoAi => {
                "Hides and turns off every AI feature in Volt: agent control, CLI automation and MCP servers."
            }
            Item::DiscordPresence => {
                "Shows Playing Volt on your Discord profile while Discord runs: menus, single player or multiplayer, the dimension and the time played. Never a world's name or a server's address."
            }
            Item::AgentControl => {
                "Your own AI agent can play Volt on this device only when enabled."
            }
            // Waypoints:
            Item::Waypoints => {
                "Master switch for Volt Waypoints (off by default). Renders 3D world markers, beacon beams, death pins and per-world/server saved locations."
            }
            Item::WaypointKey => {
                "Press in-game to drop a new waypoint at your current coordinates immediately. Escape unbinds."
            }
            Item::WaypointAutoDeath => {
                "Automatically drops a Crimson 'Last Death' waypoint whenever you die in singleplayer or multiplayer (keeps up to 3 recent deaths)."
            }
            Item::WaypointDimensionSync => {
                "Projects Overworld waypoints into the Nether at 1/8 X/Z scale (and Nether waypoints into the Overworld at 8x scale)."
            }
            Item::WaypointBeams => {
                "Draws vertical translucent beacon light columns at nearby active waypoints."
            }
            Item::WaypointLabels => {
                "Shows floating diamond icons and nameplates in 3D space."
            }
            Item::WaypointDistance => {
                "Displays live distance in metres next to each waypoint label."
            }
            Item::WaypointCoords => {
                "Always shows [X, Y, Z] coordinates on waypoint badges (even when off, aiming directly at a marker reveals its coordinates)."
            }
            Item::WaypointEdgeArrows => {
                "Shows sleek coloured directional chevrons along the screen rim pointing toward off-screen waypoints."
            }
            Item::WaypointOnMinimap => {
                "Pins active waypoints onto the Volt Minimap and clamps distant markers onto the map border."
            }
            Item::WaypointScale => {
                "Adjusts HUD marker and nameplate size (75%, 100% or 125%)."
            }
            Item::WaypointMaxDistance => {
                "Hides in-world waypoint markers beyond the chosen distance (Unlimited, 256m, 512m, 1024m or 2048m)."
            }
            Item::WaypointHudButton => {
                "Adds a compact [+WP] quick-pin button on the touch/in-game HUD for one-tap waypoint creation on mobile."
            }
            Item::WaypointNewColor => {
                "Selects the colour preset assigned to newly created waypoints."
            }
            // Minimap:
            Item::Minimap => {
                "Master switch for Volt's minimal, feature-rich live world minimap (off by default)."
            }
            Item::MinimapCorner => {
                "Screen corner where the minimap and its info pills dock, automatically avoiding status effects."
            }
            Item::MinimapSize => "Minimap side length in GUI pixels (Small 64, Medium 96, Large 128, XL 160).",
            Item::MinimapShape => "Switches between a crisp square minimap and a circular compass minimap.",
            Item::MinimapZoom => {
                "Map scale in blocks per pixel (0.5x zoomed-in detail up to 4x wide-area overview)."
            }
            Item::MinimapRotate => {
                "North Up keeps north at the top; Rotate turns the map and N/E/S/W compass ring with your look direction."
            }
            Item::MinimapFrameStyle => {
                "Chooses between Volt Crimson, Classic Dark and ultra-clean Minimal border bezels."
            }
            Item::MinimapOpacity => "Controls minimap terrain translucency (50%, 75%, 90% or 100%).",
            Item::MinimapTerrain => {
                "Auto switches to a cave slice when underground; Surface always shows top sky-lit blocks; Cave always slices near head height."
            }
            Item::MinimapChunkGrid => {
                "Overlays subtle 16x16 chunk boundary lines across the minimap."
            }
            Item::MinimapEntityRadar => {
                "Plots live dots for nearby Players (cyan), Hostile mobs (red), Passive mobs (green) and Dropped items (gold)."
            }
            Item::MinimapCoords => "Shows a compact X, Y, Z coordinate readout pill beneath the minimap.",
            Item::MinimapBiome => "Shows your cardinal heading and current biome beneath the minimap.",
            Item::MinimapTime => "Shows the current in-game Day number and 24-hour clock beneath the minimap.",
            Item::MinimapZoomKey => "Press in-game to cycle minimap zoom levels on the fly. Escape unbinds.",
            // Accurate Block Placement:
            Item::AccuratePlacement => {
                "Master switch for Volt Accurate Block Placement (off by default): sub-face aiming, orientation/half lock, plane lock, reach-around and fast place."
            }
            Item::PlacementQuadrant => {
                "Divides every block face into 5 aiming zones (centre + 4 edges) so pistons, observers, stairs, repeaters and hoppers face whichever edge you aim at."
            }
            Item::PlacementReverse => {
                "Flips placed block orientation (and top/bottom slab or stair half) to the opposite direction."
            }
            Item::PlacementLockMode => {
                "Locks placed blocks to a fixed cardinal/vertical direction or forces Top / Bottom slabs and stairs."
            }
            Item::PlacementPlaneMode => {
                "Constrains continuous right-click drag building to a single Horizontal Y layer, Vertical column or Straight row."
            }
            Item::PlacementReachAround => {
                "Allows placing blocks against the far edge or underside of the block in front of you when aiming into empty air."
            }
            Item::PlacementFastDelay => {
                "Reduces vanilla's 4-tick right-click repeat delay while holding blocks (4t Vanilla, 2t Fast, 1t Faster, 0t Instant)."
            }
            Item::PlacementHudPreview => {
                "Displays a subtle directional / slab-half indicator pill below the crosshair while aiming with an orientable block."
            }
            Item::PlacementServerProtocol => {
                "Vanilla Safe keeps hit cursors strictly inside [0..1] for anti-cheat safety; Carpet Protocol sends Tweakeroo/Carpet accurateBlockPlacement hit vectors."
            }
            Item::PlacementRotationSync => {
                "In Vanilla Safe multiplayer mode, briefly aligns look packets during directional block placement so vanilla servers place the exact chosen facing."
            }
            Item::PlacementLockKey => {
                "Press in-game to lock or unlock your current aimed block orientation. Escape unbinds."
            }
            Item::PlacementReverseKey => {
                "Press in-game to toggle Reverse Facing on or off. Escape unbinds."
            }
        }
    }
    /// The frame-time cost the main switch of an extra shows.
    pub fn cost(self) -> Option<crate::graphics_style::Cost> {
        use crate::graphics_style::Cost;
        match self {
            Item::Fullbright => Some(Cost::None),
            Item::DynamicLights => Some(Cost::Low),
            Item::OwnNameTag
            | Item::AutoReconnect
            | Item::MouseTweaks
            | Item::Sort
            | Item::Waypoints
            | Item::AccuratePlacement => Some(Cost::None),
            Item::Minimap => Some(Cost::Low),
            _ => None,
        }
    }
    /// On/off items and their value.
    pub fn switch(self, e: &Extras) -> Option<bool> {
        Some(match self {
            Item::Fullbright => e.fullbright,
            Item::DynamicLights => e.dynamic_lights,
            Item::OwnNameTag => e.own_name_tag,
            Item::AutoReconnect => e.auto_reconnect,
            Item::MouseTweaks => e.mouse_tweaks,
            Item::MouseTweaksWheel => e.mouse_tweaks_wheel,
            Item::Sort => e.sort,
            Item::NoAi => e.no_ai,
            Item::AgentControl => e.agent_control,
            Item::DiscordPresence => e.discord_presence,
            // Waypoints:
            Item::Waypoints => e.waypoints,
            Item::WaypointAutoDeath => e.waypoint_auto_death,
            Item::WaypointDimensionSync => e.waypoint_dimension_sync,
            Item::WaypointBeams => e.waypoint_beams,
            Item::WaypointLabels => e.waypoint_labels,
            Item::WaypointDistance => e.waypoint_distance,
            Item::WaypointCoords => e.waypoint_coords,
            Item::WaypointEdgeArrows => e.waypoint_edge_arrows,
            Item::WaypointOnMinimap => e.waypoint_on_minimap,
            Item::WaypointHudButton => e.waypoint_hud_button,
            // Minimap:
            Item::Minimap => e.minimap,
            Item::MinimapChunkGrid => e.minimap_chunk_grid,
            Item::MinimapCoords => e.minimap_coords,
            Item::MinimapBiome => e.minimap_biome,
            Item::MinimapTime => e.minimap_time,
            // Accurate Placement:
            Item::AccuratePlacement => e.accurate_placement,
            Item::PlacementQuadrant => e.placement_quadrant,
            Item::PlacementReverse => e.placement_reverse,
            Item::PlacementReachAround => e.placement_reach_around,
            Item::PlacementHudPreview => e.placement_hud_preview,
            Item::PlacementRotationSync => e.placement_rotation_sync,
            _ => return None,
        })
    }
    /// Formatted value shown on the option button.
    pub fn value(self, e: &Extras) -> String {
        match self {
            Item::SortKey => crate::keybinds::display_name(&e.sort_key).plain(),
            Item::WaypointKey => crate::keybinds::display_name(&e.waypoint_key).plain(),
            Item::MinimapZoomKey => crate::keybinds::display_name(&e.minimap_zoom_key).plain(),
            Item::PlacementLockKey => crate::keybinds::display_name(&e.placement_lock_key).plain(),
            Item::PlacementReverseKey => {
                crate::keybinds::display_name(&e.placement_reverse_key).plain()
            }
            Item::WaypointScale => format!("{}%", e.waypoint_scale),
            Item::WaypointMaxDistance => {
                if e.waypoint_max_distance == 0 {
                    "Unlimited".into()
                } else {
                    format!("{}m", e.waypoint_max_distance)
                }
            }
            Item::WaypointNewColor => e.waypoint_new_color.label().into(),
            Item::MinimapCorner => e.minimap_corner.label().into(),
            Item::MinimapSize => MINIMAP_SIZES
                .iter()
                .find(|s| s.0 == e.minimap_size)
                .map_or("Medium", |s| s.1)
                .into(),
            Item::MinimapShape => if e.minimap_round { "Round" } else { "Square" }.into(),
            Item::MinimapZoom => format!("{}x", e.minimap_zoom),
            Item::MinimapRotate => if e.minimap_rotate {
                "Rotate"
            } else {
                "North Up"
            }
            .into(),
            Item::MinimapFrameStyle => e.minimap_frame.label().into(),
            Item::MinimapOpacity => format!("{}%", e.minimap_opacity),
            Item::MinimapTerrain => e.minimap_terrain_mode.label().into(),
            Item::MinimapEntityRadar => e.minimap_entities.label().into(),
            Item::PlacementLockMode => e.placement_lock.label().into(),
            Item::PlacementPlaneMode => e.placement_plane_lock.label().into(),
            Item::PlacementFastDelay => match e.placement_fast_delay {
                0 => "Instant (0t)".into(),
                1 => "Faster (1t)".into(),
                2 => "Fast (2t)".into(),
                _ => "Vanilla (4t)".into(),
            },
            Item::PlacementServerProtocol => e.placement_protocol.label().into(),
            other => match other.switch(e) {
                Some(true) => "On".into(),
                _ => "Off".into(),
            },
        }
    }
    /// Advances or reverses (`back`) this item's setting.
    pub fn cycle(self, e: &mut Extras, back: bool) {
        fn step<T: Copy + PartialEq>(slice: &[T], cur: T, back: bool) -> T {
            let idx = slice.iter().position(|&v| v == cur).unwrap_or(0);
            slice[if back {
                (idx + slice.len() - 1) % slice.len()
            } else {
                (idx + 1) % slice.len()
            }]
        }
        match self {
            Item::Fullbright => e.fullbright = !e.fullbright,
            Item::DynamicLights => e.dynamic_lights = !e.dynamic_lights,
            Item::OwnNameTag => e.own_name_tag = !e.own_name_tag,
            Item::AutoReconnect => e.auto_reconnect = !e.auto_reconnect,
            Item::MouseTweaks => e.mouse_tweaks = !e.mouse_tweaks,
            Item::MouseTweaksWheel => e.mouse_tweaks_wheel = !e.mouse_tweaks_wheel,
            Item::Sort => e.sort = !e.sort,
            Item::NoAi => e.no_ai = !e.no_ai,
            Item::AgentControl => e.agent_control = !e.agent_control,
            Item::DiscordPresence => e.discord_presence = !e.discord_presence,
            // Waypoints:
            Item::Waypoints => e.waypoints = !e.waypoints,
            Item::WaypointAutoDeath => e.waypoint_auto_death = !e.waypoint_auto_death,
            Item::WaypointDimensionSync => e.waypoint_dimension_sync = !e.waypoint_dimension_sync,
            Item::WaypointBeams => e.waypoint_beams = !e.waypoint_beams,
            Item::WaypointLabels => e.waypoint_labels = !e.waypoint_labels,
            Item::WaypointDistance => e.waypoint_distance = !e.waypoint_distance,
            Item::WaypointCoords => e.waypoint_coords = !e.waypoint_coords,
            Item::WaypointEdgeArrows => e.waypoint_edge_arrows = !e.waypoint_edge_arrows,
            Item::WaypointOnMinimap => e.waypoint_on_minimap = !e.waypoint_on_minimap,
            Item::WaypointHudButton => e.waypoint_hud_button = !e.waypoint_hud_button,
            Item::WaypointScale => e.waypoint_scale = step(&WAYPOINT_SCALES, e.waypoint_scale, back),
            Item::WaypointMaxDistance => {
                e.waypoint_max_distance = step(&WAYPOINT_DISTANCES, e.waypoint_max_distance, back)
            }
            Item::WaypointNewColor => e.waypoint_new_color = e.waypoint_new_color.cycle(back),
            // Minimap:
            Item::Minimap => e.minimap = !e.minimap,
            Item::MinimapCorner => e.minimap_corner = e.minimap_corner.cycle(back),
            Item::MinimapSize => {
                let sizes: [u8; 4] = MINIMAP_SIZES.map(|s| s.0);
                e.minimap_size = step(&sizes, e.minimap_size, back);
            }
            Item::MinimapShape => e.minimap_round = !e.minimap_round,
            Item::MinimapZoom => e.minimap_zoom = step(&MINIMAP_ZOOMS, e.minimap_zoom, back),
            Item::MinimapRotate => e.minimap_rotate = !e.minimap_rotate,
            Item::MinimapFrameStyle => e.minimap_frame = e.minimap_frame.cycle(back),
            Item::MinimapOpacity => {
                e.minimap_opacity = step(&MINIMAP_OPACITIES, e.minimap_opacity, back)
            }
            Item::MinimapTerrain => e.minimap_terrain_mode = e.minimap_terrain_mode.cycle(back),
            Item::MinimapChunkGrid => e.minimap_chunk_grid = !e.minimap_chunk_grid,
            Item::MinimapEntityRadar => e.minimap_entities = e.minimap_entities.cycle(back),
            Item::MinimapCoords => e.minimap_coords = !e.minimap_coords,
            Item::MinimapBiome => e.minimap_biome = !e.minimap_biome,
            Item::MinimapTime => e.minimap_time = !e.minimap_time,
            // Accurate Block Placement:
            Item::AccuratePlacement => e.accurate_placement = !e.accurate_placement,
            Item::PlacementQuadrant => e.placement_quadrant = !e.placement_quadrant,
            Item::PlacementReverse => e.placement_reverse = !e.placement_reverse,
            Item::PlacementLockMode => e.placement_lock = e.placement_lock.cycle(back),
            Item::PlacementPlaneMode => e.placement_plane_lock = e.placement_plane_lock.cycle(back),
            Item::PlacementReachAround => e.placement_reach_around = !e.placement_reach_around,
            Item::PlacementFastDelay => {
                e.placement_fast_delay = step(&PLACEMENT_DELAYS, e.placement_fast_delay, back)
            }
            Item::PlacementHudPreview => e.placement_hud_preview = !e.placement_hud_preview,
            Item::PlacementServerProtocol => {
                e.placement_protocol = e.placement_protocol.cycle(back)
            }
            Item::PlacementRotationSync => e.placement_rotation_sync = !e.placement_rotation_sync,
            Item::SortKey
            | Item::WaypointKey
            | Item::MinimapZoomKey
            | Item::PlacementLockKey
            | Item::PlacementReverseKey => {}
        }
    }
    /// The key name the options screen waits on for a key item.
    pub fn key_name(self) -> Option<&'static str> {
        match self {
            Item::SortKey => Some(SORT_KEY),
            Item::WaypointKey => Some(WAYPOINT_KEY),
            Item::MinimapZoomKey => Some(MINIMAP_ZOOM_KEY),
            Item::PlacementLockKey => Some(PLACEMENT_LOCK_KEY),
            Item::PlacementReverseKey => Some(PLACEMENT_REVERSE_KEY),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_is_off_by_default_and_round_trips() {
        let d = Extras::default();
        assert!(!d.fullbright && !d.dynamic_lights);
        assert!(!d.zoom && !d.fps && !d.keystrokes && !d.minimap && !d.auto_reconnect);
        assert!(!d.waypoints && !d.accurate_placement);
        assert!(!d.mouse_tweaks && !d.sort && !d.own_name_tag);
        assert!(d.discord_presence);
        assert_eq!(Extras::parse(""), d);
        assert_eq!(Extras::parse(&d.serialize()), d);
        let mut e = d.clone();
        Item::Fullbright.cycle(&mut e, false);
        assert_eq!(Item::Fullbright.switch(&e), Some(true));
        Item::DynamicLights.cycle(&mut e, false);
        assert_eq!(Item::DynamicLights.switch(&e), Some(true));
        Item::Waypoints.cycle(&mut e, false);
        Item::Minimap.cycle(&mut e, false);
        Item::AccuratePlacement.cycle(&mut e, false);
        Item::MinimapSize.cycle(&mut e, false);
        Item::MinimapFrameStyle.cycle(&mut e, false);
        Item::MinimapTerrain.cycle(&mut e, false);
        Item::MinimapEntityRadar.cycle(&mut e, false);
        Item::WaypointScale.cycle(&mut e, false);
        Item::WaypointMaxDistance.cycle(&mut e, false);
        Item::PlacementLockMode.cycle(&mut e, false);
        Item::PlacementPlaneMode.cycle(&mut e, false);
        Item::PlacementFastDelay.cycle(&mut e, false);
        Item::PlacementServerProtocol.cycle(&mut e, false);
        assert!(e.set_key(WAYPOINT_KEY, "key.keyboard.n".into()));
        assert!(e.set_key(MINIMAP_ZOOM_KEY, "key.keyboard.m".into()));
        assert!(e.set_key(PLACEMENT_LOCK_KEY, "key.keyboard.l".into()));
        assert!(e.set_key(PLACEMENT_REVERSE_KEY, "key.keyboard.x".into()));
        assert_eq!(Extras::parse(&e.serialize()), e);
    }

    #[test]
    fn bad_lines_keep_defaults() {
        let e = Extras::parse("zoom:maybe\nzoomLevel:99\nfpsCorner:middle\nzoomKey:z\nnoise");
        assert!(!e.zoom);
        assert_eq!(e.zoom_level, 10);
        assert_eq!(e.fps_corner, Corner::TopLeft);
        assert_eq!(e.zoom_key, "key.keyboard.z");
    }

    #[test]
    fn a_mods_zoom_eases_like_the_spyglass() {
        let mut camera = crate::camera::Camera::default();
        camera.tick(None, 1.0);
        for _ in 0..30 {
            camera.tick(None, 1.0);
        }
        let target = fov_target(1.0, 4.0);
        camera.tick(None, target);
        let first = camera.calculate_fov(1.0, 70, 1.0, None, Default::default(), false);
        assert!((first - 70.0 * 0.625).abs() < 1e-3, "{first}");
        for _ in 0..30 {
            camera.tick(None, target);
        }
        let settled = camera.calculate_fov(1.0, 70, 1.0, None, Default::default(), false);
        assert!((settled - 70.0 / 4.0).abs() < 1e-3, "{settled}");
        for _ in 0..30 {
            camera.tick(None, fov_target(1.0, 20.0));
        }
        let floor = camera.calculate_fov(1.0, 70, 1.0, None, Default::default(), false);
        assert!((floor - 7.0).abs() < 1e-3, "{floor}");
    }

    #[test]
    fn fps_counter_reports_rate_minimum_and_low() {
        let mut c = FpsCounter::default();
        let mut t = 0.0;
        for i in 0..200 {
            let ms = if i == 150 { 50.0 } else { 10.0 };
            t += f64::from(ms) / 1000.0;
            c.record(t, ms);
        }
        c.computed_at = f64::NEG_INFINITY;
        c.record(t + 0.01, 10.0);
        let r = c.reading();
        assert!((95..=101).contains(&r.fps), "{r:?}");
        assert_eq!(r.min, 20);
        assert_eq!(r.low, 100, "one frame in 200 is under the 1% mark");
        let mut c = FpsCounter::default();
        let mut t = 0.0;
        for i in 0..100 {
            let ms = if i % 25 == 0 { 40.0 } else { 10.0 };
            t += f64::from(ms) / 1000.0;
            c.record(t, ms);
        }
        c.computed_at = f64::NEG_INFINITY;
        c.record(t, 10.0);
        assert_eq!(c.reading().low, 25);
    }

    #[test]
    fn old_frames_leave_the_window() {
        let mut c = FpsCounter::default();
        c.record(0.0, 100.0);
        for i in 1..=700 {
            c.record(f64::from(i) / 100.0, 10.0);
        }
        assert_eq!(c.reading().min, 100, "the 100 ms frame is 7 s old");
    }

    #[test]
    fn cps_counts_the_last_second() {
        let mut cps = Cps::default();
        for i in 0..10 {
            cps.click(f64::from(i) * 0.1);
        }
        assert_eq!(cps.per_second(0.95), 10);
        assert_eq!(cps.per_second(1.25), 7);
        assert_eq!(cps.per_second(3.0), 0);
    }

    #[test]
    fn items_cycle_through_their_values() {
        let mut e = Extras::default();
        Item::Sort.cycle(&mut e, true);
        assert_eq!(Item::Sort.value(&e), "On");
        assert_eq!(
            Extras::parse("minimapSize:7\nminimapZoom:3").minimap_zoom,
            1.0
        );
        for rows_fn in [rows, waypoint_rows, minimap_rows, placement_rows] {
            for item in rows_fn().into_iter().flatten() {
                assert!(!item.caption().is_empty() && !item.description().is_empty());
            }
        }
    }
}
