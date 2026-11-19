pub(super) struct Chooser {
    title: &'static str,
    help: &'static str,
    device: Option<String>,
    choices: Vec<(String, String, Action, bool)>,
    pub scroll: f32,
}
impl Chooser {
    pub(super) fn profiles(home: &std::path::Path, profile: u32) -> Self {
        let mut ids = vec![0, profile];
        ids.extend(
            std::fs::read_dir(home.join("profiles"))
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
                    id == profile,
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
        Self {
            title: "Choose profile",
            help: "Each profile keeps its own game settings and saves.",
            choices,
            device: None,
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
        let top = my + 108.0;
        self.choices
            .iter()
            .enumerate()
            .map(|(i, (_, _, action, _))| {
                (
                    [
                        mx + 20.0,
                        top + i as f32 * 76.0 - self.scroll,
                        mw - 40.0,
                        68.0,
                    ],
                    *action,
                )
            })
            .collect()
    }
    pub(super) fn viewport(&self, height: f32) -> [f32; 3] {
        let [_, my, _, mh] = Self::modal_box(1040.0, height);
        let top = my + 108.0;
        let bottom = my + mh - 68.0;
        [
            top,
            bottom,
            (self.choices.len() as f32 * 76.0 - (bottom - top)).max(0.0),
        ]
    }
    pub fn draw(&mut self, ui: &mut Ui, targets: &mut Vec<([f32; 4], Action)>) {
        use super::pages::theme;
        let (w, h) = (ui.w, ui.h);
        targets.clear();
        ui.rounded(0.0, 0.0, w, h, 0.0, theme::BACKDROP());
        let [mx, my, mw, mh] = Self::modal_box(w, h);
        ui.rounded(mx, my, mw, mh, 16.0, theme::CARD());
        ui.border(mx, my, mw, mh, 16.0, theme::BORDER_CRIMSON());

        ui.text(
            Face::LauncherMedium,
            23.0,
            mx + 24.0,
            my + 18.0,
            theme::TEXT(),
            self.title,
        );
        let help_fit = ui.fit_text(Face::LauncherRegular, 13.5, self.help, mw - 48.0);
        ui.text(
            Face::LauncherRegular,
            13.5,
            mx + 24.0,
            my + 54.0,
            theme::MUTED(),
            &help_fit,
        );
        if let Some(device) = &self.device {
            let dev_fit = ui.fit_text(Face::LauncherRegular, 12.5, device, mw - 48.0);
            ui.text(
                Face::LauncherRegular,
                12.5,
                mx + 24.0,
                my + 76.0,
                theme::CRIMSON_BRIGHT(),
                &dev_fit,
            );
        }
        ui.rounded(mx + 20.0, my + 100.0, mw - 40.0, 1.0, 0.0, theme::BORDER());

        let top = my + 108.0;
        let bottom = my + mh - 68.0;
        self.scroll = self.scroll.clamp(
            0.0,
            (self.choices.len() as f32 * 76.0 - (bottom - top)).max(0.0),
        );
        for (i, (label, detail, action, selected)) in self.choices.iter().enumerate() {
            let y = top + i as f32 * 76.0 - self.scroll;
            if y < top - 2.0 || y + 68.0 > bottom + 2.0 {
                continue;
            }
            let row_rect = [mx + 20.0, y, mw - 40.0, 68.0];
            ui.rounded(
                row_rect[0],
                row_rect[1],
                row_rect[2],
                row_rect[3],
                10.0,
                if *selected {
                    theme::HERO_BG()
                } else {
                    theme::CARD_INNER()
                },
            );
            ui.border(
                row_rect[0],
                row_rect[1],
                row_rect[2],
                row_rect[3],
                10.0,
                if *selected {
                    theme::CRIMSON()
                } else {
                    theme::BORDER()
                },
            );

            let rx = row_rect[0] + 16.0;
            let ry = y + 23.0;
            ui.border(
                rx,
                ry,
                22.0,
                22.0,
                11.0,
                if *selected {
                    theme::CRIMSON_BRIGHT()
                } else {
                    theme::MUTED()
                },
            );
            if *selected {
                ui.rounded(rx + 5.0, ry + 5.0, 12.0, 12.0, 6.0, theme::CRIMSON());
            }
            let lbl_y = if detail.is_empty() {
                y + 21.0
            } else {
                y + 10.0
            };
            let lbl_fit = ui.fit_text(Face::LauncherMedium, 16.5, label, row_rect[2] - 64.0);
            ui.text(
                Face::LauncherMedium,
                16.5,
                rx + 36.0,
                lbl_y,
                theme::TEXT(),
                &lbl_fit,
            );
            if !detail.is_empty() {
                let det_fit = ui.fit_text(Face::LauncherRegular, 13.0, detail, row_rect[2] - 64.0);
                ui.text(
                    Face::LauncherRegular,
                    13.0,
                    rx + 36.0,
                    y + 38.0,
                    theme::MUTED(),
                    &det_fit,
                );
            }
            targets.push((row_rect, *action));
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
