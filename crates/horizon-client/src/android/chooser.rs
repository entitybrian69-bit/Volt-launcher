//! Explicit, scrollable selections. Values never cycle on repeated button taps.
use super::{
    Launcher,
    pages::{Action, button},
};
use crate::catalog::RendererMode;
use horizon_client::ui_theme::{Face, Ui};

pub(super) enum Kind {
    Aspect,
    DisplayHz,
    Renderer,
    RenderScale,
    GameVersion,
    Profile,
    Orientation,
    InputHost,
    ControlLayout,
}
pub(super) struct Chooser {
    title: &'static str,
    help: &'static str,
    device: Option<String>,
    choices: Vec<(String, String, Action, bool)>,
    pub scroll: f32,
}
impl Launcher {
    pub(super) fn open_chooser(&mut self, kind: Kind) {
        crate::platform::hide();
        self.catalog.editing = None;
        let device = if matches!(kind, Kind::Renderer) {
            self.renderer.as_ref().map(|r| {
                format!(
                    "GPU: {} / {} {}",
                    r.info.name, r.info.driver, r.info.driver_info
                )
            })
        } else {
            None
        };
        let c = &self.catalog.config;
        let (title, help, choices) = match kind {
            Kind::Aspect => (
                "Aspect ratio",
                "Fits the entire picture without stretching. Touch coordinates follow the picture. Portrait swaps width and height.",
                [
                    crate::display::Aspect::Native,
                    crate::display::Aspect::Wide,
                    crate::display::Aspect::Classic,
                    crate::display::Aspect::Square,
                ]
                .into_iter()
                .map(|a| {
                    (
                        a.label().into(),
                        String::new(),
                        Action::ChooseAspect(a),
                        c.screen.aspect == a,
                    )
                })
                .collect(),
            ),
            Kind::DisplayHz => (
                "Display refresh rate",
                "An Android display preference, not a game FPS cap. Unsupported rates may be ignored.",
                crate::device_preferences::DISPLAY_RATES
                    .into_iter()
                    .map(|hz| {
                        (
                            if hz == 0 {
                                "System default".into()
                            } else {
                                format!("{hz} Hz")
                            },
                            String::new(),
                            Action::ChooseDisplayHz(hz),
                            c.device.display_hz == hz,
                        )
                    })
                    .collect(),
            ),
            Kind::RenderScale => {
                let mut choices: Vec<_> = crate::display::RenderScale::PRESETS
                    .into_iter()
                    .map(|percent| {
                        (
                            format!("{percent}%"),
                            if percent == 100 {
                                "Native resolution".into()
                            } else {
                                "Lower resolution for the whole picture".into()
                            },
                            Action::ChooseRenderScale(crate::display::RenderScale::new(percent)),
                            c.render_scale.percent() == percent,
                        )
                    })
                    .collect();
                choices.push((
                    "Custom percentage...".into(),
                    "Enter a value from 25 to 100".into(),
                    Action::Field("render_scale"),
                    false,
                ));
                (
                    "Client resolution",
                    "Small screens retain a usable menu size.",
                    choices,
                )
            }
            Kind::InputHost => (
                "Input mode",
                "Mouse mode uses a physical keyboard. Touch mode keeps the Android keyboard.",
                crate::catalog::InputHost::ALL
                    .into_iter()
                    .map(|mode| {
                        (
                            mode.label().into(),
                            String::new(),
                            Action::ChooseInputHost(mode),
                            c.input_host == mode,
                        )
                    })
                    .collect(),
            ),
            Kind::Orientation => (
                "Screen orientation",
                "Applies to the launcher and game. Does not change touch bindings.",
                crate::catalog::Orientation::ALL
                    .into_iter()
                    .map(|mode| {
                        (
                            mode.label().into(),
                            String::new(),
                            Action::ChooseOrientation(mode),
                            c.orientation == mode,
                        )
                    })
                    .collect(),
            ),
            Kind::Renderer => (
                "Rendering mode",
                "Selects Vulkan, Volt's built-in Vulkan Supporter for unsupported/Vulkan-1.0 GPUs, or OpenGL ES.",
                vec![
                    (
                        "Vulkan / compatibility (auto)".into(),
                        "Tries fast Vulkan first and auto-falls back to Vulkan Supporter if needed."
                            .into(),
                        Action::ChooseRenderer(RendererMode::VulkanCompatibility),
                        c.renderer == RendererMode::VulkanCompatibility,
                    ),
                    (
                        "Vulkan Supporter / legacy GPUs".into(),
                        "Built-in Vulkan 1.0 & driver-quirk supporter for Mali, PowerVR, Adreno & 32-bit GPUs."
                            .into(),
                        Action::ChooseRenderer(RendererMode::VulkanSupporter),
                        c.renderer == RendererMode::VulkanSupporter,
                    ),
                    (
                        "Vulkan / GPU batching".into(),
                        "Groups terrain draws and enables supported GPU culling on modern GPUs."
                            .into(),
                        Action::ChooseRenderer(RendererMode::Vulkan),
                        c.renderer == RendererMode::Vulkan,
                    ),
                    (
                        "OpenGL ES / fallback".into(),
                        "Fallback only if Vulkan drivers fail completely on your device.".into(),
                        Action::ChooseRenderer(RendererMode::OpenGl),
                        c.renderer == RendererMode::OpenGl,
                    ),
                ],
            ),
            Kind::GameVersion => {
                let active = c.selected_version();
                let choices = horizon_core::game_version::GameVersion::ALL_NEWEST_FIRST
                    .into_iter()
                    .map(|ver| {
                        let pack_fmt = ver.resource_pack_format();
                        let pack_str = if pack_fmt.1 == 0 {
                            format!("{}", pack_fmt.0)
                        } else {
                            format!("{}.{}", pack_fmt.0, pack_fmt.1)
                        };
                        (
                            format!("Minecraft Java {}", ver.id()),
                            format!(
                                "{} • Protocol {} • Pack format {}",
                                ver.era_name(),
                                ver.protocol(),
                                pack_str
                            ),
                            Action::ChooseGameVersion(ver),
                            ver == active,
                        )
                    })
                    .collect();
                (
                    "Choose Minecraft version",
                    "Pure-Rust multi-version engine (1.20.1 to 26.3). Each version isolates saves & packs and matches server protocols.",
                    choices,
                )
            }
            Kind::Profile => {
                let mut ids = vec![0, c.profile];
                ids.extend(
                    std::fs::read_dir(self.home.join("profiles"))
                        .into_iter()
                        .flatten()
                        .flatten()
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| {
                            e.file_name()
                                .to_str()?
                                .strip_prefix("profile-")?
                                .parse::<u32>()
                                .ok()
                        }),
                );
                ids.sort_unstable();
                ids.dedup();
                let next = ids.last().copied().unwrap_or(0).checked_add(1);
                let mut choices: Vec<_> = ids
                    .into_iter()
                    .map(|id| {
                        (
                            format!("Profile {id}"),
                            if id == 0 {
                                "Default profile".into()
                            } else {
                                "Separate saves, mods and packs".into()
                            },
                            Action::ChooseProfile(id),
                            id == c.profile,
                        )
                    })
                    .collect();
                if let Some(id) = next {
                    choices.push((
                        "Create profile".into(),
                        format!("Create Profile {id}"),
                        Action::ChooseProfile(id),
                        false,
                    ));
                }
                (
                    "Choose profile",
                    "Each profile keeps its own game settings and saves.",
                    choices,
                )
            }
            Kind::ControlLayout => {
                let mut choices = vec![(
                    "Default Volt Touch Controls".into(),
                    "Built-in joystick, attack, use & quick HUD buttons".into(),
                    Action::ResetControls,
                    self.catalog.active_controls.is_none(),
                )];
                for (idx, preset) in self.catalog.control_presets.iter().enumerate() {
                    let is_active = self
                        .catalog
                        .active_controls
                        .as_ref()
                        .is_some_and(|m| m.name == preset.meta.name && m.format == preset.meta.format);
                    let detail = format!(
                        "{} • {} buttons{}{}",
                        preset.meta.format,
                        preset.meta.buttons,
                        if preset.meta.drawers > 0 {
                            format!(" • {} drawers", preset.meta.drawers)
                        } else {
                            String::new()
                        },
                        if preset.meta.has_joystick {
                            " • joystick"
                        } else {
                            ""
                        }
                    );
                    choices.push((
                        preset.meta.name.clone(),
                        detail,
                        Action::ChooseControlLayout(idx),
                        is_active,
                    ));
                }
                choices.push((
                    "Import control file (.json / .zip)...".into(),
                    "PojavLauncher, Amethyst, Mojo, Zalith, Fold Craft (FCL) or Volt JSON".into(),
                    Action::ImportControl,
                    false,
                ));
                (
                    "Import Control Layouts",
                    "Select an imported Java launcher layout for this profile or import a new file.",
                    choices,
                )
            }
        };
        let step = if choices.len() > 6 { 60.0 } else { 76.0 };
        let initial_scroll = choices
            .iter()
            .position(|(_, _, _, sel)| *sel)
            .map_or(0.0, |idx| (idx as f32 * step - step).max(0.0));
        self.chooser = Some(Chooser {
            title,
            help,
            choices,
            device,
            scroll: initial_scroll,
        });
    }
}
impl Chooser {
    fn row_metrics(&self) -> (f32, f32) {
        if self.choices.len() > 6 {
            (54.0, 60.0)
        } else {
            (68.0, 76.0)
        }
    }
    pub(super) fn confirm_delete(name: String) -> Self {
        Self {
            title: "Remove add-on?",
            help: "Deletes only the selected file from this profile. This cannot be undone.",
            device: Some(name),
            choices: vec![
                (
                    "Remove".into(),
                    String::new(),
                    Action::ConfirmDeleteLocal,
                    false,
                ),
                ("Cancel".into(), String::new(), Action::Dismiss, true),
            ],
            scroll: 0.0,
        }
    }
    fn modal_box(width: f32, height: f32) -> [f32; 4] {
        let mw = (width - 48.0).clamp(280.0, 680.0);
        let mh = (height - 40.0).clamp(240.0, 620.0);
        [(width - mw) * 0.5, (height - mh) * 0.5, mw, mh]
    }
    pub(super) fn navigation(&self, width: f32, height: f32) -> Vec<([f32; 4], Action)> {
        let [mx, my, mw, _] = Self::modal_box(width, height);
        let (row_h, step) = self.row_metrics();
        let top = my + 108.0;
        self.choices
            .iter()
            .enumerate()
            .map(|(i, (_, _, action, _))| {
                (
                    [
                        mx + 20.0,
                        top + i as f32 * step - self.scroll,
                        mw - 40.0,
                        row_h,
                    ],
                    *action,
                )
            })
            .collect()
    }
    pub(super) fn viewport(&self, height: f32) -> [f32; 3] {
        let [_, my, _, mh] = Self::modal_box(1040.0, height);
        let (_, step) = self.row_metrics();
        let top = my + 108.0;
        let bottom = my + mh - 68.0;
        [
            top,
            bottom,
            (self.choices.len() as f32 * step - (bottom - top)).max(0.0),
        ]
    }
    pub fn draw(&mut self, ui: &mut Ui, targets: &mut Vec<([f32; 4], Action)>) {
        use super::pages::theme;
        let (w, h) = (ui.w, ui.h);
        let (row_h, step) = self.row_metrics();
        targets.clear();
        ui.rounded(0.0, 0.0, w, h, 0.0, theme::BACKDROP);
        let [mx, my, mw, mh] = Self::modal_box(w, h);
        ui.rounded(mx, my, mw, mh, 16.0, theme::CARD);
        ui.border(mx, my, mw, mh, 16.0, theme::BORDER_CRIMSON);

        ui.text(Face::Medium, 23.0, mx + 24.0, my + 18.0, theme::TEXT, self.title);
        let help_fit = ui.fit_text(Face::Regular, 13.5, self.help, mw - 48.0);
        ui.text(Face::Regular, 13.5, mx + 24.0, my + 54.0, theme::MUTED, &help_fit);
        if let Some(device) = &self.device {
            let dev_fit = ui.fit_text(Face::Regular, 12.5, device, mw - 48.0);
            ui.text(Face::Regular, 12.5, mx + 24.0, my + 76.0, theme::CRIMSON_BRIGHT, &dev_fit);
        }
        ui.rounded(mx + 20.0, my + 100.0, mw - 40.0, 1.0, 0.0, theme::BORDER);

        let top = my + 108.0;
        let bottom = my + mh - 68.0;
        self.scroll = self.scroll.clamp(
            0.0,
            (self.choices.len() as f32 * step - (bottom - top)).max(0.0),
        );
        for (i, (label, detail, action, selected)) in self.choices.iter().enumerate() {
            let y = top + i as f32 * step - self.scroll;
            if y + row_h * 0.5 < top || y + row_h * 0.5 > bottom {
                continue;
            }
            let vis_top = y.max(top);
            let vis_bottom = (y + row_h).min(bottom);
            let row_rect = [mx + 20.0, vis_top, mw - 40.0, (vis_bottom - vis_top).max(24.0)];
            if y >= top - 2.0 && y + row_h <= bottom + 2.0 {
                ui.rounded(
                    mx + 20.0,
                    y,
                    mw - 40.0,
                    row_h,
                    10.0,
                    if *selected {
                        theme::HERO_BG
                    } else {
                        theme::CARD_INNER
                    },
                );
                ui.border(
                    mx + 20.0,
                    y,
                    mw - 40.0,
                    row_h,
                    10.0,
                    if *selected {
                        theme::CRIMSON
                    } else {
                        theme::BORDER
                    },
                );

                let rx = mx + 36.0;
                let ry = y + (row_h - 22.0) * 0.5;
                ui.border(
                    rx,
                    ry,
                    22.0,
                    22.0,
                    11.0,
                    if *selected {
                        theme::CRIMSON_BRIGHT
                    } else {
                        theme::MUTED
                    },
                );
                if *selected {
                    ui.rounded(rx + 5.0, ry + 5.0, 12.0, 12.0, 6.0, theme::CRIMSON);
                }
                let lbl_y = if detail.is_empty() {
                    y + (row_h - 22.0) * 0.5
                } else {
                    y + if row_h < 60.0 { 6.0 } else { 10.0 }
                };
                let lbl_fit = ui.fit_text(Face::Medium, 16.0, label, mw - 104.0);
                ui.text(Face::Medium, 16.0, rx + 36.0, lbl_y, theme::TEXT, &lbl_fit);
                if !detail.is_empty() {
                    let det_y = y + if row_h < 60.0 { 30.0 } else { 38.0 };
                    let det_fit = ui.fit_text(Face::Regular, 12.5, detail, mw - 104.0);
                    ui.text(Face::Regular, 12.5, rx + 36.0, det_y, theme::MUTED, &det_fit);
                }
                targets.push(([mx + 20.0, y, mw - 40.0, row_h], *action));
            } else {
                let _ = row_rect;
            }
        }
        button(
            ui,
            targets,
            [mx + mw - 156.0, my + mh - 52.0, 136.0, 38.0],
            "Cancel",
            Action::Dismiss,
            false,
        );
    }
}
