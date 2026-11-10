//! Volt Hub: the in-game overlay (crimson on dark, left sidebar) that
//! fronts the Volt Recorder, Replay, quick toggles, performance controls
//! and screenshots. A `VOLT` pill on the HUD (or F10) opens it; while it
//! is closed nothing here costs a frame anything beyond that pill.
use super::*;

/// The sidebar tabs, top to bottom.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum HubTab {
    #[default]
    Recorder,
    Replay,
    Toggles,
    Performance,
    Screenshot,
    Settings,
}

impl HubTab {
    const ALL: [HubTab; 6] = [
        HubTab::Recorder,
        HubTab::Replay,
        HubTab::Toggles,
        HubTab::Performance,
        HubTab::Screenshot,
        HubTab::Settings,
    ];
    fn label(self) -> &'static str {
        match self {
            HubTab::Recorder => "Recorder",
            HubTab::Replay => "Replay",
            HubTab::Toggles => "Quick Toggles",
            HubTab::Performance => "Performance",
            HubTab::Screenshot => "Screenshot",
            HubTab::Settings => "Settings",
        }
    }
}

/// What a click target does.
#[derive(Clone, Copy, PartialEq)]
enum Act {
    OpenHub,
    Close,
    Tab(HubTab),
    RecToggle,
    RecClip,
    RecHeight,
    RecFps,
    RecControls,
    RecHud,
    RecClipMode,
    Preset,
    PerfHud,
    PurgeRam,
    FpsCounter,
    Keystrokes,
    ReplayButton,
    TakeScreenshot,
}

/// The overlay's state on the `Game`.
#[derive(Default)]
pub(super) struct Hub {
    pub open: bool,
    tab: HubTab,
    /// Click targets rebuilt every drawn frame, in GUI coordinates.
    targets: Vec<([f32; 4], Act)>,
    /// The last outcome line (recorder results and such).
    status: String,
}

// Palette: the owner's crimson on near-black glass.
const CRIMSON: u32 = 0xFF3C14DC; // ABGR of #DC143C
const PANEL: u32 = 0xF014100E;
const SIDEBAR: u32 = 0xF61A1012;
const ROW: u32 = 0x66241A1E;
const TEXT: u32 = 0xFFF2EFF4;
const DIM: u32 = 0xFF9A8F96;
const GREY: u32 = 0xFF5C5258;
const SCRIM: u32 = 0x8C000000;

impl Game<'_> {
    /// F10 toggles the hub while playing.
    pub(super) fn volt_hub_key(&mut self, key: KeyCode) -> bool {
        if key == KeyCode::F10 && self.screen == Screen::Playing && self.title.is_none() {
            self.volt_hub.open = !self.volt_hub.open;
            return true;
        }
        false
    }

    /// Stops the recorder when its session cannot continue (the player
    /// left the world or started watching a replay).
    pub(super) fn volt_recorder_tick(&mut self) {
        if self.recorder.is_some() && (self.title.is_some() || self.replay_control().is_some()) {
            self.volt_recorder_stop();
        }
    }

    fn volt_recorder_stop(&mut self) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.video_sink = None;
        }
        if let Some(mut rec) = self.recorder.take() {
            self.volt_hub.status = rec.finish();
        }
    }

    /// Starts or stops the Volt Recorder with the current settings.
    fn volt_recorder_toggle(&mut self) {
        if self.recorder.is_some() {
            self.volt_recorder_stop();
            return;
        }
        if self.replay_control().is_some() {
            self.volt_hub.status = "Finish the replay first.".to_string();
            return;
        }
        if self.renderer.as_ref().is_some_and(|r| r.video_sink.is_some()) {
            self.volt_hub.status = "The frame capture is already in use.".to_string();
            return;
        }
        let s = self.recorder_settings;
        let rec = super::recorder::Recording::start(std::path::PathBuf::from("recordings"), s);
        let Some(sender) = rec.sender() else {
            self.volt_hub.status = "The recorder could not start.".to_string();
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            self.volt_hub.status = "The recorder needs the renderer.".to_string();
            return;
        };
        let start = std::time::Instant::now();
        let sink = move |size: [u32; 2], bgra: bool, pixels: Vec<[u8; 4]>| {
            let _ = sender.try_send(super::video_export::Frame {
                size,
                bgra,
                pts_us: start.elapsed().as_micros() as i64,
                pixels,
            });
        };
        renderer.video_layers = (s.hud, s.controls);
        renderer.video_interval =
            std::time::Duration::from_micros(1_000_000 / u64::from(s.fps.max(1)));
        renderer.video_sink = Some(Box::new(sink));
        self.recorder = Some(rec);
        self.volt_hub.status = if s.clip {
            "Clip ring armed: the last 30 s are kept.".to_string()
        } else {
            "Recording started.".to_string()
        };
    }

    /// The screen size available to the capture, in pixels.
    fn volt_screen_px(&self) -> [u32; 2] {
        self.window
            .as_ref()
            .map(|w| {
                let s = w.inner_size();
                [s.width, s.height]
            })
            .unwrap_or([1920, 1080])
    }

    /// One recorder settings change: persists and, while recording, points
    /// out that the running file keeps its settings.
    fn volt_recorder_edit(&mut self, change: impl FnOnce(&mut super::recorder::Settings)) {
        change(&mut self.recorder_settings);
        self.recorder_settings.save(std::path::Path::new(""));
        if self.recorder.is_some() {
            self.volt_hub.status = "Applies to the next recording.".to_string();
        }
    }

    /// Handles a press at `cursor` (GUI coordinates). Returns whether the
    /// hub consumed it.
    pub(super) fn try_click_volt_hub_overlay(&mut self, cursor: (f32, f32)) -> bool {
        if self.title.is_some() || self.ui.hide_gui || self.replay_control().is_some() {
            self.volt_hub.open = false;
            self.volt_hub.targets.clear();
            return false;
        }
        let hit = self
            .volt_hub
            .targets
            .iter()
            .find(|([x, y, w, h], _)| {
                cursor.0 >= *x && cursor.0 <= x + w && cursor.1 >= *y && cursor.1 <= y + h
            })
            .map(|&(_, act)| act);
        let Some(act) = hit else {
            if self.volt_hub.open {
                // A tap outside the panel closes the hub; either way the
                // touch must not reach the world below.
                self.volt_hub.open = false;
                return true;
            }
            return false;
        };
        self.audio.play_button_click();
        match act {
            Act::OpenHub => self.volt_hub.open = true,
            Act::Close => self.volt_hub.open = false,
            Act::Tab(tab) => self.volt_hub.tab = tab,
            Act::RecToggle => self.volt_recorder_toggle(),
            Act::RecClip => {
                if let Some(rec) = &self.recorder {
                    rec.save_clip();
                    self.volt_hub.status = "Saving the last 30 seconds...".to_string();
                }
            }
            Act::RecHeight => {
                let screen = self.volt_screen_px();
                self.volt_recorder_edit(|s| {
                    let heights = super::recorder::HEIGHTS;
                    let at = heights.iter().position(|&v| v == s.height).unwrap_or(0);
                    for step in 1..=heights.len() {
                        let next = heights[(at + step) % heights.len()];
                        if super::recorder::Settings::height_supported(next, screen) {
                            s.height = next;
                            break;
                        }
                    }
                });
            }
            Act::RecFps => self.volt_recorder_edit(|s| {
                let rates = super::recorder::RATES;
                let at = rates.iter().position(|&v| v == s.fps).unwrap_or(0);
                s.fps = rates[(at + 1) % rates.len()];
            }),
            Act::RecControls => self.volt_recorder_edit(|s| s.controls = !s.controls),
            Act::RecHud => self.volt_recorder_edit(|s| s.hud = !s.hud),
            Act::RecClipMode => self.volt_recorder_edit(|s| s.clip = !s.clip),
            Act::Preset => {
                let next = self.settings.extras.volt_preset.next();
                self.settings.extras.apply_volt_preset(next);
                self.settings.save(&Self::options_path());
            }
            Act::PerfHud => {
                self.settings.extras.perf_hud = self.settings.extras.perf_hud.next();
                self.settings.save(&Self::options_path());
            }
            Act::PurgeRam => {
                self.settings.extras.purge_ram_requested = true;
                self.volt_hub.status = "RAM purge requested.".to_string();
            }
            Act::FpsCounter => {
                self.settings.extras.fps = !self.settings.extras.fps;
                self.settings.save(&Self::options_path());
            }
            Act::Keystrokes => {
                self.settings.extras.keystrokes = !self.settings.extras.keystrokes;
                self.settings.save(&Self::options_path());
            }
            Act::ReplayButton => {
                self.settings.extras.volt_qol.replay_button =
                    !self.settings.extras.volt_qol.replay_button;
                self.settings.save(&Self::options_path());
            }
            Act::TakeScreenshot => {
                // Close first so the grabbed frame is clean gameplay.
                self.volt_hub.open = false;
                self.take_screenshot();
            }
        }
        true
    }

    /// Draws the pill (closed) or the hub window (open) and rebuilds the
    /// click targets.
    pub(super) fn draw_volt_hub(&mut self, batch: &mut HudBatch) {
        let mut targets: Vec<([f32; 4], Act)> = Vec::new();
        let (w, h) = (batch.width, batch.height);
        let atlas = &self.gui.atlas;
        if !self.volt_hub.open {
            // The HUD pill, top right; red while recording.
            let (pw, ph) = (34.0, 13.0);
            let (px, py) = (w - pw - 4.0, 4.0);
            let recording = self.recorder.is_some();
            let bg = if recording { CRIMSON } else { 0xDC14100E };
            batch.fill(atlas, px, py, pw, ph, bg);
            batch.fill(atlas, px, py, pw, 1.0, CRIMSON);
            batch.text(atlas, "VOLT", px + 6.0, py + 3.0, TEXT, true);
            targets.push(([px, py, pw, ph], Act::OpenHub));
            if recording && let Some(rec) = &self.recorder {
                let t = rec.recorded_seconds() as u64;
                let label = if rec.settings.clip {
                    format!("RING {t:02}s")
                } else {
                    format!("REC {}:{:02}", t / 60, t % 60)
                };
                batch.text(atlas, &label, px - 42.0, py + 3.0, CRIMSON, true);
            }
            self.volt_hub.targets = targets;
            return;
        }
        // Scrim and the ~85% window.
        batch.fill(atlas, 0.0, 0.0, w, h, SCRIM);
        let (hx, hy) = ((w * 0.075).round(), (h * 0.075).round());
        let (hw, hh) = (w - hx * 2.0, h - hy * 2.0);
        batch.fill(atlas, hx, hy, hw, hh, PANEL);
        batch.fill(atlas, hx, hy, hw, 1.2, CRIMSON);
        // Sidebar.
        let sw = 64.0_f32.min(hw * 0.3);
        batch.fill(atlas, hx, hy + 1.2, sw, hh - 1.2, SIDEBAR);
        batch.text(atlas, "VOLT HUB", hx + 6.0, hy + 6.0, CRIMSON, true);
        let mut ty = hy + 20.0;
        for tab in HubTab::ALL {
            let active = tab == self.volt_hub.tab;
            if active {
                batch.fill(atlas, hx, ty - 2.0, sw, 13.0, ROW);
                batch.fill(atlas, hx, ty - 2.0, 1.5, 13.0, CRIMSON);
            }
            batch.text(
                atlas,
                tab.label(),
                hx + 7.0,
                ty,
                if active { TEXT } else { DIM },
                false,
            );
            targets.push(([hx, ty - 2.0, sw, 13.0], Act::Tab(tab)));
            ty += 14.0;
        }
        // Close button.
        let (cx, cy) = (hx + hw - 13.0, hy + 4.0);
        batch.text(atlas, "X", cx + 3.0, cy + 1.0, DIM, false);
        targets.push(([cx, cy, 11.0, 11.0], Act::Close));
        // Content area.
        let x = hx + sw + 8.0;
        let cw = hw - sw - 16.0;
        let mut y = hy + 8.0;
        let mut row = |batch: &mut HudBatch,
                       targets: &mut Vec<([f32; 4], Act)>,
                       y: &mut f32,
                       label: &str,
                       value: &str,
                       act: Option<Act>,
                       greyed: bool| {
            batch.fill(atlas, x, *y, cw, 12.0, ROW);
            batch.text(atlas, label, x + 4.0, *y + 2.5, if greyed { GREY } else { TEXT }, false);
            let vw = HudBatch::text_width(atlas, value);
            batch.text(
                atlas,
                value,
                x + cw - vw - 4.0,
                *y + 2.5,
                if greyed { GREY } else { CRIMSON },
                false,
            );
            if let Some(act) = act
                && !greyed
            {
                targets.push(([x, *y, cw, 12.0], act));
            }
            *y += 14.0;
        };
        match self.volt_hub.tab {
            HubTab::Recorder => {
                let s = self.recorder_settings;
                let screen = self.volt_screen_px();
                let state = match &self.recorder {
                    Some(rec) if rec.settings.clip => {
                        format!("Ring: last {:.0}s held", rec.recorded_seconds())
                    }
                    Some(rec) => {
                        let t = rec.recorded_seconds() as u64;
                        format!("Recording {}:{:02}", t / 60, t % 60)
                    }
                    None => "Idle".to_string(),
                };
                row(batch, &mut targets, &mut y, "Recorder", &state, None, false);
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    if self.recorder.is_some() { "Stop" } else { "Start" },
                    if s.clip { "clip ring" } else { "record" },
                    Some(Act::RecToggle),
                    false,
                );
                if self.recorder.as_ref().is_some_and(|r| r.settings.clip) {
                    row(
                        batch,
                        &mut targets,
                        &mut y,
                        "Save clip",
                        "last 30 s",
                        Some(Act::RecClip),
                        false,
                    );
                }
                let hs = super::recorder::Settings::height_supported(s.height, screen);
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Resolution",
                    &format!("{}p{}", s.height, if hs { "" } else { " (too big)" }),
                    Some(Act::RecHeight),
                    false,
                );
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Frame rate",
                    &format!("{} fps", s.fps),
                    Some(Act::RecFps),
                    false,
                );
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Record with controls",
                    if s.controls { "ON" } else { "OFF" },
                    Some(Act::RecControls),
                    false,
                );
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Record with HUD",
                    if s.hud { "ON" } else { "OFF" },
                    Some(Act::RecHud),
                    false,
                );
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Clip mode (last 30 s)",
                    if s.clip { "ON" } else { "OFF" },
                    Some(Act::RecClipMode),
                    false,
                );
                batch.text(
                    atlas,
                    &format!("~{:.0} MB/min - H.264 + AAC game audio", s.mb_per_min(screen)),
                    x + 2.0,
                    y + 2.0,
                    DIM,
                    false,
                );
                y += 12.0;
            }
            HubTab::Replay => {
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Floating replay button",
                    if self.settings.extras.volt_qol.replay_button {
                        "ON"
                    } else {
                        "OFF"
                    },
                    Some(Act::ReplayButton),
                    false,
                );
                let replays = Self::scan_replay_list();
                if replays.is_empty() {
                    batch.text(atlas, "No replays yet.", x + 2.0, y + 2.0, DIM, false);
                    y += 12.0;
                } else {
                    for r in replays.iter().take(7) {
                        row(batch, &mut targets, &mut y, &r.title, &r.line, None, false);
                    }
                }
                batch.text(
                    atlas,
                    "Watch & export to MP4 from the launcher's Replays tab.",
                    x + 2.0,
                    y + 2.0,
                    DIM,
                    false,
                );
                y += 12.0;
            }
            HubTab::Toggles => {
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "FPS counter",
                    if self.settings.extras.fps { "ON" } else { "OFF" },
                    Some(Act::FpsCounter),
                    false,
                );
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Keystrokes",
                    if self.settings.extras.keystrokes { "ON" } else { "OFF" },
                    Some(Act::Keystrokes),
                    false,
                );
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Performance HUD",
                    self.settings.extras.perf_hud.as_str(),
                    Some(Act::PerfHud),
                    false,
                );
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Floating replay button",
                    if self.settings.extras.volt_qol.replay_button {
                        "ON"
                    } else {
                        "OFF"
                    },
                    Some(Act::ReplayButton),
                    false,
                );
            }
            HubTab::Performance => {
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Volt Boost preset",
                    self.settings.extras.volt_preset.label(),
                    Some(Act::Preset),
                    false,
                );
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Performance HUD",
                    self.settings.extras.perf_hud.as_str(),
                    Some(Act::PerfHud),
                    false,
                );
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Purge RAM",
                    "now",
                    Some(Act::PurgeRam),
                    false,
                );
                batch.text(
                    atlas,
                    "Performance presets also pause cosmetic mods (Mod Lag Shield).",
                    x + 2.0,
                    y + 2.0,
                    DIM,
                    false,
                );
                y += 12.0;
            }
            HubTab::Screenshot => {
                row(
                    batch,
                    &mut targets,
                    &mut y,
                    "Take screenshot",
                    "F2",
                    Some(Act::TakeScreenshot),
                    false,
                );
                let mut names: Vec<String> = std::fs::read_dir("screenshots")
                    .map(|dir| {
                        dir.filter_map(|e| e.ok())
                            .filter_map(|e| e.file_name().into_string().ok())
                            .filter(|n| n.ends_with(".png"))
                            .collect()
                    })
                    .unwrap_or_default();
                names.sort_unstable_by(|a, b| b.cmp(a));
                if names.is_empty() {
                    batch.text(atlas, "No screenshots yet.", x + 2.0, y + 2.0, DIM, false);
                    y += 12.0;
                } else {
                    for name in names.iter().take(7) {
                        row(batch, &mut targets, &mut y, name, "", None, false);
                    }
                }
            }
            HubTab::Settings => {
                let s = self.recorder_settings;
                batch.text(
                    atlas,
                    "Recordings save to the game's recordings/ folder.",
                    x + 2.0,
                    y + 2.0,
                    DIM,
                    false,
                );
                y += 12.0;
                batch.text(
                    atlas,
                    "Screenshots save to screenshots/, replays to replays/.",
                    x + 2.0,
                    y + 2.0,
                    DIM,
                    false,
                );
                y += 12.0;
                batch.text(
                    atlas,
                    &format!(
                        "Recorder: {}p / {} fps / controls {} / HUD {}",
                        s.height,
                        s.fps,
                        if s.controls { "on" } else { "off" },
                        if s.hud { "on" } else { "off" }
                    ),
                    x + 2.0,
                    y + 2.0,
                    DIM,
                    false,
                );
                y += 12.0;
                batch.text(
                    atlas,
                    "Everything else lives in Volt Settings (pause menu).",
                    x + 2.0,
                    y + 2.0,
                    DIM,
                    false,
                );
                y += 12.0;
            }
        }
        if !self.volt_hub.status.is_empty() {
            let max = (cw / 5.0) as usize;
            let line: String = self.volt_hub.status.chars().take(max.max(20)).collect();
            batch.text(atlas, &line, x + 2.0, hy + hh - 12.0, CRIMSON, false);
        }
        let _ = y;
        self.volt_hub.targets = targets;
    }
}
