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
    Profile,
    Orientation,
    InputHost,
    ControlLayout,
}
include!("../launcher_ui/chooser.rs");
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
            Kind::Profile => {
                self.chooser = Some(Chooser::profiles(&self.home, c.profile));
                return;
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
        self.chooser = Some(Chooser {
            title,
            help,
            choices,
            device,
            scroll: 0.0,
        });
    }
}
impl Chooser {
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
}
