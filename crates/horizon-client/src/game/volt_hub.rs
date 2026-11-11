//! Volt Hub: the in-game overlay (crimson on dark, left sidebar) that
//! fronts the Volt Recorder, Replay, quick toggles, performance controls
//! and screenshots. A `VOLT` pill on the HUD (or F10) opens it; while it
//! is closed nothing here costs a frame anything beyond that pill.
use super::*;
use horizon_client::ui_theme::{Face, Ui};

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
    /// A tap on the open window's body: consumed, never closes the hub.
    Panel,
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

/// A finger dragging the floating logo.
pub(super) struct LogoDrag {
    id: u64,
    offset: [f32; 2],
    start: (f32, f32),
    moved: bool,
}

/// The overlay's state on the `Game`.
#[derive(Default)]
pub(super) struct Hub {
    pub open: bool,
    /// Where the floating logo was dragged to (GUI px, top-left);
    /// None keeps the default top-right spot.
    pos: Option<[f32; 2]>,
    /// The touch currently dragging the logo.
    drag: Option<LogoDrag>,
    tab: HubTab,
    /// Click targets rebuilt every drawn frame, in GUI coordinates.
    targets: Vec<([f32; 4], Act)>,
    /// The last outcome line (recorder results and such).
    status: String,
    /// Open/close animation progress, 0 closed .. 1 open.
    anim: f32,
    /// The animation clock's last frame.
    anim_at: Option<std::time::Instant>,
    /// The target just tapped flashes briefly (GUI rect + when).
    flash: Option<([f32; 4], std::time::Instant)>,
}

// Palette: the owner's crimson on near-black glass.
const CRIMSON: u32 = 0xFF3C14DC; // ABGR of #DC143C
const PANEL: u32 = 0xF014100E;
const ROW: u32 = 0x66241A1E;
const TEXT: u32 = 0xFFF2EFF4;
const DIM: u32 = 0xFF9A8F96;
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
            renderer.video_scale = None;
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
        // The GPU scales each capture to the output size before readback:
        // the worker then converts and encodes exactly the pixels it needs,
        // whatever the panel's resolution (above it included).
        let screen = [renderer.config.width.max(2), renderer.config.height.max(2)];
        renderer.video_scale = Some([s.width_for(screen), s.height]);
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

    /// The floating logo's rectangle in GUI coordinates, clamped on screen.
    pub(super) fn volt_hub_logo_rect(&self, w: f32, h: f32) -> [f32; 4] {
        const S: f32 = 18.0;
        let [x, y] = self.volt_hub.pos.unwrap_or([w - S - 4.0, 4.0]);
        [
            x.clamp(0.0, (w - S).max(0.0)),
            y.clamp(0.0, (h - S).max(0.0)),
            S,
            S,
        ]
    }

    /// The logo's own gesture, owning its touch from press to release like
    /// the floating replay button: a tap opens the hub, a drag moves the
    /// logo anywhere on screen. Idle frames reject in a couple of compares.
    pub(super) fn volt_hub_logo_touch(
        &mut self,
        id: u64,
        p: (f32, f32),
        phase: winit::event::TouchPhase,
    ) -> bool {
        use winit::event::TouchPhase;
        let owns = self.volt_hub.drag.as_ref().is_some_and(|d| d.id == id);
        if !owns
            && (phase != TouchPhase::Started
                || self.volt_hub.open
                || self.title.is_some()
                || self.ui.hide_gui
                || self.screen != Screen::Playing)
        {
            return false;
        }
        let (w, h) = self.gui_size();
        if owns {
            match phase {
                TouchPhase::Started => false,
                TouchPhase::Moved => {
                    let r = self.volt_hub_logo_rect(w, h);
                    let (ox, oy, moved) = {
                        let d = self.volt_hub.drag.as_mut().expect("owned drag");
                        if (p.0 - d.start.0).abs() >= 4.0 || (p.1 - d.start.1).abs() >= 4.0 {
                            d.moved = true;
                        }
                        (d.offset[0], d.offset[1], d.moved)
                    };
                    if moved {
                        self.volt_hub.pos = Some([
                            (p.0 - ox).clamp(0.0, (w - r[2]).max(0.0)),
                            (p.1 - oy).clamp(0.0, (h - r[3]).max(0.0)),
                        ]);
                    }
                    true
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    let d = self.volt_hub.drag.take().expect("owned drag");
                    if phase == TouchPhase::Ended && !d.moved {
                        self.volt_hub.open = true;
                        self.audio.play_button_click();
                    }
                    true
                }
            }
        } else {
            let r = self.volt_hub_logo_rect(w, h);
            if p.0 < r[0] || p.0 > r[0] + r[2] || p.1 < r[1] || p.1 > r[1] + r[3] {
                return false;
            }
            self.volt_hub.drag = Some(LogoDrag {
                id,
                offset: [p.0 - r[0], p.1 - r[1]],
                start: p,
                moved: false,
            });
            true
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
            .map(|&(rect, act)| (rect, act));
        let Some((rect, act)) = hit else {
            if self.volt_hub.open {
                // A tap outside the panel closes the hub; either way the
                // touch must not reach the world below.
                self.volt_hub.open = false;
                return true;
            }
            return false;
        };
        if act != Act::Panel {
            self.audio.play_button_click();
            self.volt_hub.flash = Some((rect, std::time::Instant::now()));
        }
        match act {
            Act::OpenHub => self.volt_hub.open = true,
            // The window body swallows its taps: only the X (or a tap on
            // the scrim outside the window, or F10) closes the hub.
            Act::Panel => {}
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
                self.volt_recorder_edit(|s| {
                    let heights = super::recorder::HEIGHTS;
                    let at = heights.iter().position(|&v| v == s.height).unwrap_or(0);
                    s.height = heights[(at + 1) % heights.len()];
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
                // The FPS profiles exist to raise the frame rate: they also
                // release vanilla's VSync (Fifo) and 120-FPS defaults, which
                // otherwise cap every phone at its panel's refresh rate.
                if matches!(
                    next,
                    horizon_client::volt_features::VoltBoostPreset::Competitive
                        | horizon_client::volt_features::VoltBoostPreset::UltraFps
                ) {
                    self.settings.vsync = false;
                    self.settings.max_fps = self.settings.max_fps.max(260);
                }
                self.apply_ui_settings();
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

    /// Draws the floating logo (closed) or the hub window (open) and
    /// rebuilds the click targets. Rendering goes through the launcher's
    /// `Ui` layer: the real Volt logo, smooth vector text and rounded
    /// panels, with a short open/close animation and tap feedback.
    pub(super) fn draw_volt_hub(&mut self, batch: &mut HudBatch) {
        // The open/close animation clock: an exponential approach with a
        // smoothstep ease reads as "smooth" at any frame rate.
        let now = std::time::Instant::now();
        let dt = self
            .volt_hub
            .anim_at
            .map_or(0.0, |at| now.duration_since(at).as_secs_f32())
            .min(0.1);
        self.volt_hub.anim_at = Some(now);
        let target = if self.volt_hub.open { 1.0 } else { 0.0 };
        let a = self.volt_hub.anim + (target - self.volt_hub.anim) * (dt * 14.0).min(1.0);
        self.volt_hub.anim = if (a - target).abs() < 0.01 { target } else { a };
        let t = {
            let a = self.volt_hub.anim;
            a * a * (3.0 - 2.0 * a)
        };

        /// `color` with its alpha scaled by `f` (ABGR, alpha in the top byte).
        fn fade(color: u32, f: f32) -> u32 {
            let a = ((color >> 24) as f32 * f.clamp(0.0, 1.0)) as u32;
            (a << 24) | (color & 0x00FF_FFFF)
        }

        let mut targets: Vec<([f32; 4], Act)> = Vec::new();
        let scale = batch.scale;
        let (gw, gh) = (batch.width, batch.height);
        let logo_rect = self.volt_hub_logo_rect(gw, gh);
        let recording = self.recorder.is_some();
        let rec_line = self.recorder.as_ref().map(|rec| {
            let secs = rec.recorded_seconds();
            if rec.settings.clip {
                format!("RING {:02.0}s", secs)
            } else {
                let t = secs as u64;
                format!("REC {}:{:02}", t / 60, t % 60)
            }
        });
        let tab = self.volt_hub.tab;
        let status = self.volt_hub.status.clone();
        let flash = self.volt_hub.flash;
        let open = self.volt_hub.open;

        let mut ui = Ui::new(&self.gui.atlas, batch, (0.0, 0.0));
        // dp -> GUI pixels, for the click targets the input pipeline reads.
        let dk = ui.density / scale;
        // GUI pixels -> dp, for rects kept in GUI coordinates (the logo).
        let kd = scale / ui.density;

        if !open {
            // The floating Volt logo on a soft dark chip; crimson while
            // recording. Its rect stays in GUI coordinates for the drag
            // gesture, so convert for drawing only.
            let [bx, by, bs, _] = logo_rect;
            let (x, y, s) = (bx * kd, by * kd, bs * kd);
            let chip = if recording { CRIMSON } else { 0xE618100E };
            ui.rounded(x, y, s, s, s * 0.28, chip);
            ui.border(x, y, s, s, s * 0.28, fade(CRIMSON, if recording { 0.0 } else { 0.9 }));
            let pad = s * 0.12;
            ui.volt_logo(x + pad, y + pad, s - pad * 2.0, s - pad * 2.0);
            targets.push((logo_rect, Act::OpenHub));
            if let Some(line) = &rec_line {
                let size = (s * 0.42).clamp(9.0, 14.0);
                let tw = ui.measure(Face::LauncherMedium, size, line);
                let lx = if x > tw + 12.0 { x - tw - 8.0 } else { x + s + 8.0 };
                ui.text(
                    Face::LauncherMedium,
                    size,
                    lx,
                    y + (s - size * 1.2) / 2.0,
                    CRIMSON,
                    line,
                );
            }
        }

        if t > 0.0 {
            // Scrim fades in with the window.
            ui.rounded(0.0, 0.0, ui.w, ui.h, 0.0, fade(SCRIM, t));
            // The window scales in from 96%.
            // 4% smaller as it opens: extra margin melts away with t.
            let grow = (1.0 - t) * 0.02;
            let (mx, my) = (ui.w * (0.075 + grow), ui.h * (0.075 + grow));
            let (mw, mh) = (ui.w - mx * 2.0, ui.h - my * 2.0);
            ui.rounded(mx, my, mw, mh, 14.0, fade(PANEL, t));
            ui.border(mx, my, mw, mh, 14.0, fade(0x5A3C14DC, t));
            ui.rounded(mx, my, mw, 3.0, 1.5, fade(CRIMSON, t));

            // Header: the logo, the title, the close chip.
            let head_h = 40.0;
            ui.volt_logo(mx + 12.0, my + 9.0, 22.0, 22.0);
            ui.text(
                Face::LauncherMedium,
                15.0,
                mx + 42.0,
                my + 11.0,
                fade(TEXT, t),
                "Volt Hub",
            );
            let xr = [mx + mw - 34.0, my + 8.0, 24.0, 24.0];
            ui.rounded(xr[0], xr[1], xr[2], xr[3], 8.0, fade(ROW, t));
            let xw = ui.measure(Face::LauncherMedium, 12.5, "X");
            ui.text(
                Face::LauncherMedium,
                12.5,
                xr[0] + (xr[2] - xw) / 2.0,
                xr[1] + 4.5,
                fade(DIM, t),
                "X",
            );
            if open {
                targets.push((
                    [
                        (xr[0] - 6.0) * dk,
                        (xr[1] - 6.0) * dk,
                        (xr[2] + 12.0) * dk,
                        (xr[3] + 12.0) * dk,
                    ],
                    Act::Close,
                ));
            }

            // Sidebar.
            let sw = 128.0_f32.min(mw * 0.32);
            let mut ty = my + head_h + 4.0;
            for tb in HubTab::ALL {
                let active = tb == tab;
                let row_rect = [mx + 8.0, ty, sw - 12.0, 26.0];
                if active {
                    ui.rounded(row_rect[0], row_rect[1], row_rect[2], row_rect[3], 8.0, fade(ROW, t));
                    ui.rounded(row_rect[0], row_rect[1] + 5.0, 3.0, 16.0, 1.5, fade(CRIMSON, t));
                }
                let label = ui.fit_text(Face::LauncherMedium, 12.5, tb.label(), row_rect[2] - 18.0);
                ui.text(
                    Face::LauncherMedium,
                    12.5,
                    row_rect[0] + 12.0,
                    ty + 6.0,
                    fade(if active { TEXT } else { DIM }, t),
                    &label,
                );
                if open {
                    targets.push((
                        [
                            row_rect[0] * dk,
                            row_rect[1] * dk,
                            row_rect[2] * dk,
                            row_rect[3] * dk,
                        ],
                        Act::Tab(tb),
                    ));
                }
                ty += 28.0;
            }

            // Content rows.
            let x = mx + sw + 8.0;
            let cw = mw - sw - 24.0;
            let mut y = my + head_h + 4.0;
            let bottom = my + mh - 26.0;
            let mut row = |ui: &mut Ui,
                           targets: &mut Vec<([f32; 4], Act)>,
                           y: &mut f32,
                           label: &str,
                           value: &str,
                           act: Option<Act>|
             {
                if *y + 24.0 > bottom {
                    return;
                }
                ui.rounded(x, *y, cw, 24.0, 8.0, fade(ROW, t));
                let lab = ui.fit_text(Face::LauncherRegular, 13.0, label, cw * 0.62);
                ui.text(Face::LauncherRegular, 13.0, x + 10.0, *y + 5.0, fade(TEXT, t), &lab);
                let vw = ui.measure(Face::LauncherMedium, 13.0, value);
                ui.text(
                    Face::LauncherMedium,
                    13.0,
                    x + cw - vw - 10.0,
                    *y + 5.0,
                    fade(CRIMSON, t),
                    value,
                );
                if let Some(act) = act
                    && open
                {
                    targets.push(([x * dk, *y * dk, cw * dk, 24.0 * dk], act));
                }
                *y += 28.0;
            };
            let mut note = |ui: &mut Ui, y: &mut f32, text: &str| {
                if *y + 16.0 > bottom {
                    return;
                }
                let line = ui.fit_text(Face::LauncherRegular, 11.5, text, cw - 4.0);
                ui.text(Face::LauncherRegular, 11.5, x + 4.0, *y + 2.0, fade(DIM, t), &line);
                *y += 18.0;
            };
            match tab {
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
                    row(&mut ui, &mut targets, &mut y, "Recorder", &state, None);
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        if recording { "Stop" } else { "Start" },
                        if s.clip { "clip ring" } else { "record" },
                        Some(Act::RecToggle),
                    );
                    if self.recorder.as_ref().is_some_and(|r| r.settings.clip) {
                        row(
                            &mut ui,
                            &mut targets,
                            &mut y,
                            "Save clip",
                            "last 30 s",
                            Some(Act::RecClip),
                        );
                    }
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Resolution",
                        &format!("{}p", s.height),
                        Some(Act::RecHeight),
                    );
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Frame rate",
                        &format!("{} fps", s.fps),
                        Some(Act::RecFps),
                    );
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Record with controls",
                        if s.controls { "ON" } else { "OFF" },
                        Some(Act::RecControls),
                    );
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Record with HUD",
                        if s.hud { "ON" } else { "OFF" },
                        Some(Act::RecHud),
                    );
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Clip mode (last 30 s)",
                        if s.clip { "ON" } else { "OFF" },
                        Some(Act::RecClipMode),
                    );
                    note(
                        &mut ui,
                        &mut y,
                        &format!("~{:.0} MB/min - H.264 + AAC game audio", s.mb_per_min(screen)),
                    );
                }
                HubTab::Replay => {
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Floating replay button",
                        if self.settings.extras.volt_qol.replay_button {
                            "ON"
                        } else {
                            "OFF"
                        },
                        Some(Act::ReplayButton),
                    );
                    let replays = Self::scan_replay_list();
                    if replays.is_empty() {
                        note(&mut ui, &mut y, "No replays yet.");
                    } else {
                        for r in replays.iter().take(7) {
                            row(&mut ui, &mut targets, &mut y, &r.title, &r.line, None);
                        }
                    }
                    note(
                        &mut ui,
                        &mut y,
                        "Watch & export to MP4 from the launcher's Replays tab.",
                    );
                }
                HubTab::Toggles => {
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "FPS counter",
                        if self.settings.extras.fps { "ON" } else { "OFF" },
                        Some(Act::FpsCounter),
                    );
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Keystrokes",
                        if self.settings.extras.keystrokes { "ON" } else { "OFF" },
                        Some(Act::Keystrokes),
                    );
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Performance HUD",
                        self.settings.extras.perf_hud.as_str(),
                        Some(Act::PerfHud),
                    );
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Floating replay button",
                        if self.settings.extras.volt_qol.replay_button {
                            "ON"
                        } else {
                            "OFF"
                        },
                        Some(Act::ReplayButton),
                    );
                }
                HubTab::Performance => {
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Volt Boost preset",
                        self.settings.extras.volt_preset.label(),
                        Some(Act::Preset),
                    );
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Performance HUD",
                        self.settings.extras.perf_hud.as_str(),
                        Some(Act::PerfHud),
                    );
                    row(&mut ui, &mut targets, &mut y, "Purge RAM", "now", Some(Act::PurgeRam));
                    note(
                        &mut ui,
                        &mut y,
                        "Performance presets also pause cosmetic mods (Mod Lag Shield).",
                    );
                }
                HubTab::Screenshot => {
                    row(
                        &mut ui,
                        &mut targets,
                        &mut y,
                        "Take screenshot",
                        "F2",
                        Some(Act::TakeScreenshot),
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
                        note(&mut ui, &mut y, "No screenshots yet.");
                    } else {
                        for name in names.iter().take(7) {
                            row(&mut ui, &mut targets, &mut y, name, "", None);
                        }
                    }
                }
                HubTab::Settings => {
                    let s = self.recorder_settings;
                    note(&mut ui, &mut y, "Recordings save to the game's recordings/ folder.");
                    note(
                        &mut ui,
                        &mut y,
                        "Screenshots save to screenshots/, replays to replays/.",
                    );
                    note(
                        &mut ui,
                        &mut y,
                        &format!(
                            "Recorder: {}p / {} fps / controls {} / HUD {}",
                            s.height,
                            s.fps,
                            if s.controls { "on" } else { "off" },
                            if s.hud { "on" } else { "off" }
                        ),
                    );
                    note(
                        &mut ui,
                        &mut y,
                        "Everything else lives in Volt Settings (pause menu).",
                    );
                }
            }
            if !status.is_empty() {
                let line = ui.fit_text(Face::LauncherRegular, 11.5, &status, cw);
                ui.text(
                    Face::LauncherRegular,
                    11.5,
                    x,
                    my + mh - 20.0,
                    fade(CRIMSON, t),
                    &line,
                );
            }
            // A just-tapped control flashes for a moment.
            if let Some((fr, at)) = flash {
                let age = now.duration_since(at).as_secs_f32();
                if age < 0.18 {
                    let f = 1.0 - age / 0.18;
                    ui.rounded(
                        fr[0] * kd,
                        fr[1] * kd,
                        fr[2] * kd,
                        fr[3] * kd,
                        8.0,
                        fade(0x66FFFFFF, f),
                    );
                } else {
                    self.volt_hub.flash = None;
                }
            }
            if open {
                // Pushed last so every control above wins the hit test: a tap
                // anywhere else on the window body is consumed, not a close.
                targets.push(([mx * dk, my * dk, mw * dk, mh * dk], Act::Panel));
            }
        }
        ui.finish();
        self.volt_hub.targets = targets;
    }
}
