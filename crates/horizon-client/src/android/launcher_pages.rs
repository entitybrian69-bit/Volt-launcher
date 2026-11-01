use super::*;
use horizon_client::osk::Output;
use winit::{
    event::{KeyEvent, Touch},
    keyboard::{KeyCode, PhysicalKey},
};

/// All-dark obsidian & crimson-red palette for the Android launcher.
pub(crate) mod theme {
    use horizon_client::ui_theme::color::hex;

    pub const BG: u32 = hex(0x07070A, 255);
    pub const BACKDROP: u32 = hex(0x030305, 215);
    pub const RAIL: u32 = hex(0x0C0C12, 255);
    pub const CARD: u32 = hex(0x111118, 255);
    pub const CARD_INNER: u32 = hex(0x181822, 255);
    pub const HOVER: u32 = hex(0x232331, 255);
    pub const HERO_BG: u32 = hex(0x19070D, 255);
    pub const HERO_GLOW: u32 = hex(0x4A0C1B, 90);
    pub const HERO_GLOW_SOFT: u32 = hex(0x300812, 70);
    pub const BORDER: u32 = hex(0x262635, 255);
    pub const BORDER_CRIMSON: u32 = hex(0x6D1729, 255);
    pub const CRIMSON: u32 = hex(0xDC143C, 255);
    pub const CRIMSON_BRIGHT: u32 = hex(0xFF2E54, 255);
    pub const CRIMSON_SOFT: u32 = hex(0xDC143C, 42);
    pub const CRIMSON_BADGE: u32 = hex(0x3B0D18, 255);
    pub const ON_CRIMSON: u32 = hex(0xFFFFFF, 255);
    pub const TEXT: u32 = hex(0xF4F4F8, 255);
    pub const MUTED: u32 = hex(0x9898AA, 255);
    pub const DIM: u32 = hex(0x68687A, 255);
    pub const ONLINE: u32 = hex(0x22C55E, 255);
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Action {
    Page(Page),
    Play,
    Search,
    ShaderSource(crate::shader_catalog::Origin),
    ModSource(crate::shader_catalog::Origin),
    Field(&'static str),
    FinishNameEdit,
    Install(usize),
    OpenProject(usize),
    Import(horizon_client::content_files::Kind),
    OpenFolder(horizon_client::content_files::Kind),
    ToggleSection(usize),
    ToggleLocal(usize),
    DeleteLocal(usize),
    ConfirmDeleteLocal,
    Next,
    Previous,
    Profile,
    Logs,
    RecordPerformance,
    ExportPerformance,
    ExportDiagnostics,
    Account,
    SignIn,
    DeviceSignIn,
    CopyLoginCode,
    CancelLogin,
    ReviewApp,
    Repository,
    SignOut,
    LoginLink,
    CloseLogin,
    Renderer,
    RenderScale,
    ChooseRenderScale(crate::display::RenderScale),
    Orientation,
    InputHost,
    ChooseInputHost(crate::catalog::InputHost),
    ChooseOrientation(crate::catalog::Orientation),
    ChooseRenderer(crate::catalog::RendererMode),
    ChooseProfile(u32),
    Dismiss,
    Immersive,
    Cutout,
    Aspect,
    ChooseAspect(crate::display::Aspect),
    DisplayHz,
    ChooseDisplayHz(u16),
    PerformanceHints,
    GpuProfile,
}

#[derive(Clone, Copy)]
enum BtnStyle {
    Primary,
    Secondary,
    CrimsonOutline,
    Subtle,
}

fn text_vcenter(y: f32, h: f32, size: f32) -> f32 {
    y + (h - (size * 1.45).round()) * 0.5
}

fn draw_centered_text(
    ui: &mut Ui,
    face: Face,
    size: f32,
    rect: [f32; 4],
    col: u32,
    label: &str,
) {
    let [x, y, w, h] = rect;
    let fitted = ui.fit_text(face, size, label, (w - 12.0).max(8.0));
    let tw = ui.measure(face, size, &fitted);
    ui.text(
        face,
        size,
        x + ((w - tw) * 0.5).max(4.0),
        text_vcenter(y, h, size),
        col,
        &fitted,
    );
}

fn styled_button(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    label: &str,
    action: Action,
    style: BtnStyle,
    centered: bool,
) {
    let [x, y, w, h] = rect;
    let radius = (h * 0.24).clamp(6.0, 11.0);
    let (bg, border_col, fg) = match style {
        BtnStyle::Primary => (theme::CRIMSON, theme::CRIMSON_BRIGHT, theme::ON_CRIMSON),
        BtnStyle::CrimsonOutline => (theme::CARD_INNER, theme::BORDER_CRIMSON, theme::TEXT),
        BtnStyle::Secondary => (theme::CARD_INNER, theme::BORDER, theme::TEXT),
        BtnStyle::Subtle => (theme::CARD, theme::BORDER, theme::MUTED),
    };
    ui.rounded(x, y, w, h, radius, bg);
    ui.border(x, y, w, h, radius, border_col);
    let base = if h >= 52.0 {
        18.0
    } else if w < 84.0 {
        12.5
    } else {
        14.5
    };
    let pad = (w * 0.1).clamp(8.0, 14.0);
    let font = if matches!(action, Action::Page(_)) {
        (base * ((w - 2.0 * pad) / ui.measure(Face::Medium, base, label).max(1.0)).min(1.0))
            .max(10.0)
    } else {
        base
    };
    if centered {
        draw_centered_text(ui, Face::Medium, font, rect, fg, label);
    } else {
        let fitted = ui.fit_text(Face::Medium, font, label, (w - 2.0 * pad).max(8.0));
        ui.text(Face::Medium, font, x + pad, text_vcenter(y, h, font), fg, &fitted);
    }
    targets.push((rect, action));
}

pub(super) fn button(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    label: &str,
    action: Action,
    accent: bool,
) {
    styled_button(
        ui,
        targets,
        rect,
        label,
        action,
        if accent {
            BtnStyle::Primary
        } else {
            BtnStyle::Secondary
        },
        accent,
    );
}

fn toggle(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    on: bool,
    action: Action,
) {
    let [x, y, w, h] = rect;
    ui.text(
        Face::Medium,
        13.5,
        x,
        text_vcenter(y, h, 13.5),
        if on { theme::TEXT } else { theme::MUTED },
        if on { "On" } else { "Off" },
    );
    let sw = 46.0;
    let sh = 24.0;
    let sx = x + w - sw;
    let sy = y + (h - sh) * 0.5;
    ui.rounded(
        sx,
        sy,
        sw,
        sh,
        12.0,
        if on { theme::CRIMSON } else { theme::HOVER },
    );
    ui.border(
        sx,
        sy,
        sw,
        sh,
        12.0,
        if on {
            theme::CRIMSON_BRIGHT
        } else {
            theme::BORDER
        },
    );
    let knob = 18.0;
    let kx = if on { sx + sw - knob - 3.0 } else { sx + 3.0 };
    ui.rounded(kx, sy + 3.0, knob, knob, 9.0, theme::ON_CRIMSON);
    targets.push((rect, action));
}

fn draw_page_icon(ui: &mut Ui, page: Page, cx: f32, cy: f32, col: u32) {
    match page {
        Page::Play => {
            ui.border(cx - 8.0, cy - 8.0, 16.0, 16.0, 8.0, col);
            ui.rounded(cx - 3.0, cy - 4.5, 2.5, 9.0, 1.0, col);
            ui.rounded(cx - 1.0, cy - 3.0, 2.5, 6.0, 1.0, col);
            ui.rounded(cx + 1.0, cy - 1.5, 2.5, 3.0, 1.0, col);
        }
        Page::Mods => {
            ui.border(cx - 7.0, cy - 7.0, 6.5, 6.5, 1.5, col);
            ui.rounded(cx + 0.5, cy - 7.0, 6.5, 6.5, 1.5, col);
            ui.border(cx - 7.0, cy + 0.5, 6.5, 6.5, 1.5, col);
            ui.border(cx + 0.5, cy + 0.5, 6.5, 6.5, 1.5, col);
        }
        Page::Packs => {
            ui.rounded(cx - 7.5, cy - 6.5, 15.0, 3.0, 1.5, col);
            ui.rounded(cx - 7.5, cy - 1.5, 15.0, 3.0, 1.5, col);
            ui.rounded(cx - 7.5, cy + 3.5, 15.0, 3.0, 1.5, col);
        }
        Page::Shaders => {
            ui.border(cx - 5.5, cy - 5.5, 11.0, 11.0, 5.5, col);
            ui.rounded(cx - 2.0, cy - 2.0, 4.0, 4.0, 2.0, col);
            ui.rounded(cx - 1.0, cy - 8.5, 2.0, 2.0, 1.0, col);
            ui.rounded(cx - 1.0, cy + 6.5, 2.0, 2.0, 1.0, col);
            ui.rounded(cx - 8.5, cy - 1.0, 2.0, 2.0, 1.0, col);
            ui.rounded(cx + 6.5, cy - 1.0, 2.0, 2.0, 1.0, col);
        }
        Page::Installed => {
            ui.rounded(cx - 1.0, cy - 7.5, 2.0, 9.0, 1.0, col);
            ui.rounded(cx - 4.0, cy - 1.0, 8.0, 2.0, 1.0, col);
            ui.border(cx - 7.5, cy + 2.5, 15.0, 5.0, 2.0, col);
        }
        Page::Settings => {
            ui.rounded(cx - 7.5, cy - 5.5, 15.0, 2.0, 1.0, col);
            ui.rounded(cx - 3.5, cy - 7.0, 4.0, 5.0, 2.0, col);
            ui.rounded(cx - 7.5, cy + 3.5, 15.0, 2.0, 1.0, col);
            ui.rounded(cx + 0.5, cy + 2.0, 4.0, 5.0, 2.0, col);
        }
    }
}

fn format_downloads(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn wrap_lines(ui: &Ui, face: Face, size: f32, text: &str, max_w: f32, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_owned()
        } else {
            format!("{current} {word}")
        };
        if ui.measure(face, size, &candidate) <= max_w || current.is_empty() {
            current = candidate;
        } else {
            lines.push(ui.fit_text(face, size, &current, max_w));
            if lines.len() + 1 >= max_lines {
                let rest_start = text.find(word).unwrap_or(0);
                lines.push(ui.fit_text(face, size, &text[rest_start..], max_w));
                return lines;
            }
            current = word.to_owned();
        }
    }
    if !current.is_empty() && lines.len() < max_lines {
        lines.push(ui.fit_text(face, size, &current, max_w));
    }
    lines
}

fn draw_account_chip(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    status: &str,
    signed_in: bool,
) {
    let [x, y, w, h] = rect;
    ui.rounded(x, y, w, h, h * 0.5, theme::CARD);
    ui.border(
        x,
        y,
        w,
        h,
        h * 0.5,
        if signed_in {
            theme::CRIMSON
        } else {
            theme::BORDER_CRIMSON
        },
    );
    let dot_col = if signed_in {
        theme::ONLINE
    } else {
        theme::CRIMSON_BRIGHT
    };
    ui.rounded(x + w - 16.0, y + (h - 8.0) * 0.5, 8.0, 8.0, 4.0, dot_col);
    let label = ui.fit_text(Face::Medium, 13.5, status, (w - 34.0).max(24.0));
    ui.text(
        Face::Medium,
        13.5,
        x + 14.0,
        text_vcenter(y, h, 13.5),
        theme::TEXT,
        &label,
    );
    targets.push((rect, Action::Account));
}

impl Launcher {
    /// The same spatial focus algorithm as the game's controller menus.
    pub(super) fn navigate(&mut self, key: &KeyEvent) -> bool {
        self.navigate_key(&key.logical_key)
    }
    pub(super) fn navigate_key(&mut self, key: &winit::keyboard::Key) -> bool {
        use horizon_client::gamepad::{self, Button};
        use winit::keyboard::{Key, NamedKey};
        if self.pending_focus.is_some() {
            return true;
        }
        if self.catalog.editing.is_some() || self.targets.is_empty() {
            return false;
        }
        let button = match key {
            Key::Named(NamedKey::ArrowUp) => Button::DPadUp,
            Key::Named(NamedKey::ArrowDown) => Button::DPadDown,
            Key::Named(NamedKey::ArrowLeft) => Button::DPadLeft,
            Key::Named(NamedKey::ArrowRight) => Button::DPadRight,
            Key::Named(NamedKey::Tab) => {
                self.focus = Some(self.focus.map_or(0, |i| (i + 1) % self.targets.len()));
                return true;
            }
            _ => return false,
        };
        if matches!(button, Button::DPadUp | Button::DPadDown)
            && let Some((_, action)) = self.focus.and_then(|i| self.targets.get(i))
            && let Some(index) = self.list_targets.iter().position(|(_, a)| a == action)
        {
            let rects: Vec<_> = self.list_targets.iter().map(|(r, _)| *r).collect();
            if let Some(next) = gamepad::focus_step_strict(rects[index], button, &rects)
                && let Some(view) = self.list_view
            {
                let (rect, action) = self.list_targets[next];
                let scroll = if let Some(chooser) = &mut self.chooser {
                    &mut chooser.scroll
                } else {
                    &mut self.catalog.scroll
                };
                *scroll = super::gesture::reveal(*scroll, rect, view);
                self.pending_focus = Some(action);
                self.focus = None;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                return true;
            }
        }
        self.focus = Some(
            if let Some(index) = self.focus.filter(|i| *i < self.targets.len()) {
                let rects: Vec<_> = self.targets.iter().map(|(rect, _)| *rect).collect();
                gamepad::focus_step_rect(rects[index], button, &rects).unwrap_or(index)
            } else {
                0
            },
        );
        true
    }

    fn draw_sidebar(&mut self, ui: &mut Ui, rx: f32, ry: f32, rw: f32, rh: f32) {
        ui.rounded(rx, ry, rw, rh, 14.0, theme::RAIL);
        ui.border(rx, ry, rw, rh, 14.0, theme::BORDER_CRIMSON);

        // Brand emblem + VOLT / HORIZON version header
        let bx = rx + 14.0;
        let by = ry + 14.0;
        ui.rounded(bx, by, 32.0, 32.0, 8.0, theme::CRIMSON_BADGE);
        ui.border(bx, by, 32.0, 32.0, 8.0, theme::CRIMSON);
        draw_centered_text(
            ui,
            Face::Medium,
            17.0,
            [bx, by, 32.0, 32.0],
            theme::CRIMSON_BRIGHT,
            "V",
        );
        ui.text(Face::Medium, 18.0, bx + 40.0, by - 2.0, theme::TEXT, "VOLT");
        ui.text(
            Face::Regular,
            10.5,
            bx + 40.0,
            by + 18.0,
            theme::MUTED,
            &format!("HORIZON {}", crate::backend::VERSION),
        );
        ui.rounded(rx + 12.0, ry + 58.0, rw - 24.0, 1.0, 0.0, theme::BORDER);

        // Vertical navigation items
        let nav_top = ry + 70.0;
        let nav_bottom = ry + rh - 58.0;
        let avail = (nav_bottom - nav_top).max(216.0);
        let step = (avail / Page::ALL.len() as f32).min(50.0);
        let item_h = (step - 6.0).clamp(34.0, 44.0);

        for (i, &page) in Page::ALL.iter().enumerate() {
            let iy = nav_top + i as f32 * step;
            let rect = [rx + 10.0, iy, rw - 20.0, item_h];
            let active = page == self.catalog.page;
            if active {
                ui.rounded(
                    rect[0],
                    rect[1],
                    rect[2],
                    rect[3],
                    9.0,
                    if page == Page::Play {
                        theme::CRIMSON
                    } else {
                        theme::CRIMSON_SOFT
                    },
                );
                ui.border(
                    rect[0],
                    rect[1],
                    rect[2],
                    rect[3],
                    9.0,
                    if page == Page::Play {
                        theme::CRIMSON_BRIGHT
                    } else {
                        theme::CRIMSON
                    },
                );
                ui.rounded(
                    rx + 2.0,
                    iy + 7.0,
                    3.5,
                    item_h - 14.0,
                    2.0,
                    theme::CRIMSON_BRIGHT,
                );
            }
            let fg = if active {
                theme::ON_CRIMSON
            } else {
                theme::MUTED
            };
            draw_page_icon(ui, page, rect[0] + 18.0, iy + item_h * 0.5, fg);
            ui.text(
                Face::Medium,
                14.5,
                rect[0] + 36.0,
                text_vcenter(iy, item_h, 14.5),
                fg,
                page.name(),
            );
            self.targets.push((rect, Action::Page(page)));
        }

        // Bottom GitHub button
        styled_button(
            ui,
            &mut self.targets,
            [rx + 12.0, ry + rh - 46.0, rw - 24.0, 34.0],
            "GitHub",
            Action::Repository,
            BtnStyle::Secondary,
            true,
        );
    }

    fn draw_compact_header(&mut self, ui: &mut Ui, w: f32, account_status: &str) {
        ui.rounded(12.0, 10.0, 132.0, 34.0, 9.0, theme::CARD);
        ui.border(12.0, 10.0, 132.0, 34.0, 9.0, theme::BORDER_CRIMSON);
        draw_centered_text(
            ui,
            Face::Medium,
            13.5,
            [12.0, 10.0, 132.0, 34.0],
            theme::TEXT,
            &format!("VOLT {}", crate::backend::VERSION),
        );
        draw_account_chip(
            ui,
            &mut self.targets,
            [152.0, 10.0, (w - 164.0).max(120.0), 34.0],
            account_status,
            self.login.signed_in,
        );

        // Segmented pill dock
        ui.rounded(12.0, 52.0, w - 24.0, 40.0, 10.0, theme::RAIL);
        ui.border(12.0, 52.0, w - 24.0, 40.0, 10.0, theme::BORDER);
        let tab_w = (w - 32.0) / Page::ALL.len() as f32;
        for (i, &page) in Page::ALL.iter().enumerate() {
            let tx = 16.0 + i as f32 * tab_w;
            let active = page == self.catalog.page;
            let label = match page {
                Page::Installed => "Files",
                Page::Settings => "Setup",
                _ => page.name(),
            };
            let rect = [tx + 1.0, 56.0, tab_w - 2.0, 32.0];
            if active {
                ui.rounded(rect[0], rect[1], rect[2], rect[3], 7.0, theme::CRIMSON);
            }
            draw_centered_text(
                ui,
                Face::Medium,
                12.5,
                rect,
                if active { theme::ON_CRIMSON } else { theme::MUTED },
                label,
            );
            self.targets.push((rect, Action::Page(page)));
        }
    }

    fn draw_play_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        account_status: &str,
    ) {
        use horizon_client::official_install::InstallStage;
        let [cx, cy, cw, ch] = area;
        let installed = horizon_client::official_install::ready(&self.home);
        let installing = self.worker.is_some();
        let progress = *self.install.lock().unwrap();

        let (hero_rect, right_rect) = if compact {
            let hero_h = (ch * 0.54).clamp(240.0, 340.0);
            (
                [cx, cy, cw, hero_h],
                Some([cx, cy + hero_h + 12.0, cw, (ch - hero_h - 12.0).max(160.0)]),
            )
        } else {
            let hero_w = (cw * 0.58).round();
            let right_w = cw - hero_w - 16.0;
            (
                [cx, cy, hero_w, ch - 34.0],
                Some([cx + hero_w + 16.0, cy, right_w, ch - 34.0]),
            )
        };

        // Hero Launch Card (01-play.png style)
        let [hx, hy, hw, hh] = hero_rect;
        ui.rounded(hx, hy, hw, hh, 16.0, theme::HERO_BG);
        ui.rounded(hx + 8.0, hy + 8.0, hw * 0.65, hh * 0.45, 14.0, theme::HERO_GLOW_SOFT);
        ui.rounded(hx + 16.0, hy + 14.0, hw * 0.42, hh * 0.28, 12.0, theme::HERO_GLOW);
        ui.border(hx, hy, hw, hh, 16.0, theme::BORDER_CRIMSON);

        let heading = if installing {
            "Getting Game Ready"
        } else if installed {
            "Ready to Play"
        } else if compact {
            "Minecraft Java 26.3"
        } else {
            "Java Worlds on Android"
        };
        let heading_sz = if compact { 26.0 } else { 36.0 };
        let fitted_heading = ui.fit_text(Face::Medium, heading_sz, heading, hw - 52.0);
        ui.text(
            Face::Medium,
            heading_sz,
            hx + 26.0,
            hy + if compact { 18.0 } else { 32.0 },
            theme::TEXT,
            &fitted_heading,
        );

        // Version badge pill
        let badge_y = hy + if compact { 60.0 } else { 92.0 };
        let ver_w = 192.0f32.min(hw - 52.0);
        ui.rounded(hx + 26.0, badge_y, ver_w, 34.0, 9.0, theme::CRIMSON_BADGE);
        ui.border(hx + 26.0, badge_y, ver_w, 34.0, 9.0, theme::BORDER_CRIMSON);
        ui.rounded(hx + 38.0, badge_y + 11.0, 12.0, 12.0, 3.0, theme::CRIMSON_BRIGHT);
        ui.text(
            Face::Medium,
            13.5,
            hx + 58.0,
            text_vcenter(badge_y, 34.0, 13.5),
            theme::CRIMSON_BRIGHT,
            "Minecraft Java 26.3",
        );

        // Profile selector pill (interactive -> Action::Profile!)
        let prof_y = badge_y + if compact { 42.0 } else { 48.0 };
        styled_button(
            ui,
            &mut self.targets,
            [hx + 26.0, prof_y, 152.0, 36.0],
            &format!("Profile {}   v", self.catalog.config.profile),
            Action::Profile,
            BtnStyle::CrimsonOutline,
            false,
        );

        if installing {
            let current = InstallStage::ALL
                .iter()
                .position(|s| *s == progress.stage)
                .unwrap_or(0);
            let list_top = prof_y + 46.0;
            let bar_y = hy + hh - 62.0;
            let avail_rows = bar_y - list_top - 8.0;
            if avail_rows >= 132.0 {
                let step_h = (avail_rows / 6.0).clamp(20.0, 28.0);
                for (idx, stage) in InstallStage::ALL.iter().enumerate() {
                    let sy = list_top + idx as f32 * step_h;
                    let label = if idx < current {
                        format!("{:02}  {}  [done]", idx + 1, stage.label())
                    } else {
                        format!("{:02}  {}", idx + 1, stage.label())
                    };
                    let col = if idx == current {
                        theme::CRIMSON_BRIGHT
                    } else if idx < current {
                        theme::TEXT
                    } else {
                        theme::MUTED
                    };
                    let fitted = ui.fit_text(Face::Regular, 13.5, &label, hw - 52.0);
                    ui.text(Face::Regular, 13.5, hx + 26.0, sy, col, &fitted);
                }
            } else {
                ui.text(
                    Face::Medium,
                    15.5,
                    hx + 26.0,
                    list_top + 4.0,
                    theme::TEXT,
                    progress.stage.label(),
                );
                ui.text(
                    Face::Regular,
                    13.0,
                    hx + 26.0,
                    list_top + 28.0,
                    theme::MUTED,
                    &format!("Step {} of 6", current + 1),
                );
            }
            let fraction = if progress.total > 0 {
                progress.done as f32 / progress.total as f32
            } else {
                0.0
            };
            let overall = ((current as f32 + fraction.clamp(0.0, 1.0)) / 6.0).clamp(0.0, 1.0);
            ui.rounded(hx + 26.0, bar_y, hw - 52.0, 8.0, 4.0, theme::CARD_INNER);
            if overall > 0.0 {
                ui.rounded(
                    hx + 26.0,
                    bar_y,
                    ((hw - 52.0) * overall).max(8.0),
                    8.0,
                    4.0,
                    theme::CRIMSON,
                );
            }
            let detail = match progress.stage {
                InstallStage::Download => format!(
                    "{:.1} / {:.1} MiB verified",
                    progress.done as f64 / 1048576.0,
                    progress.total as f64 / 1048576.0
                ),
                InstallStage::Controls => {
                    format!("{} / {} control files verified", progress.done, progress.total)
                }
                _ => "Keep launcher open; verified files are cached for retry.".into(),
            };
            let fitted = ui.fit_text(Face::Regular, 12.5, &detail, hw - 52.0);
            ui.text(Face::Regular, 12.5, hx + 26.0, bar_y + 14.0, theme::MUTED, &fitted);
        } else {
            let cta_h = if compact { 48.0 } else { 58.0 };
            let cta_y = hy + hh - cta_h - if compact { 18.0 } else { 28.0 };
            let cta_label = if installed {
                "PLAY"
            } else if self.error.is_some() {
                "RETRY INSTALLATION"
            } else {
                "DOWNLOAD MINECRAFT 26.3"
            };
            styled_button(
                ui,
                &mut self.targets,
                [hx + 26.0, cta_y, hw - 52.0, cta_h],
                cta_label,
                Action::Play,
                BtnStyle::Primary,
                true,
            );
        }

        // Right Column: Account Card + Quick Start Guide Card
        if let Some([rx, ry, rw, rh]) = right_rect {
            let guide_rect = if compact {
                [rx, ry, rw, rh]
            } else {
                // Account Card at top-right (matching 01-play.png)
                let acc_h = 126.0;
                ui.rounded(rx, ry, rw, acc_h, 14.0, theme::CARD);
                ui.border(rx, ry, rw, acc_h, 14.0, theme::BORDER_CRIMSON);
                ui.text(Face::Medium, 13.5, rx + 20.0, ry + 14.0, theme::MUTED, "Account");

                let avatar_y = ry + 48.0;
                ui.rounded(rx + 20.0, avatar_y, 44.0, 44.0, 22.0, theme::CRIMSON);
                let initial = self
                    .login
                    .profile
                    .as_deref()
                    .or_else(|| {
                        let trimmed = self.catalog.config.name.trim();
                        (!trimmed.is_empty()).then_some(trimmed)
                    })
                    .and_then(|s| s.chars().next())
                    .map(|c| c.to_ascii_uppercase().to_string())
                    .unwrap_or_else(|| "P".into());
                draw_centered_text(
                    ui,
                    Face::Medium,
                    18.0,
                    [rx + 20.0, avatar_y, 44.0, 44.0],
                    theme::ON_CRIMSON,
                    &initial,
                );

                let status_line = ui.fit_text(Face::Medium, 15.5, account_status, (rw - 96.0).max(40.0));
                ui.text(
                    Face::Medium,
                    15.5,
                    rx + 76.0,
                    avatar_y + 2.0,
                    theme::TEXT,
                    &status_line,
                );
                let sub = if self.login.signed_in {
                    "Microsoft session active"
                } else {
                    "Tap to sign in or edit name"
                };
                let sub_fit = ui.fit_text(Face::Regular, 12.5, sub, (rw - 96.0).max(40.0));
                ui.text(Face::Regular, 12.5, rx + 76.0, avatar_y + 24.0, theme::MUTED, &sub_fit);
                self.targets.push(([rx, ry, rw, acc_h], Action::Account));

                [rx, ry + acc_h + 14.0, rw, (rh - acc_h - 14.0).max(160.0)]
            };

            let [gx, gy, gw, gh] = guide_rect;
            ui.rounded(gx, gy, gw, gh, 14.0, theme::CARD);
            ui.border(gx, gy, gw, gh, 14.0, theme::BORDER_CRIMSON);
            ui.text(
                Face::Medium,
                16.5,
                gx + 20.0,
                gy + 16.0,
                theme::TEXT,
                "Quick Start Guide",
            );
            let rows: [(&str, &str, Option<Action>); 3] = if installed {
                [
                    ("01", "Open a world or join a Java server", Some(Action::Play)),
                    (
                        "02",
                        "Touch setup: Options > Controls",
                        Some(Action::Page(Page::Settings)),
                    ),
                    (
                        "03",
                        "Find packs and .hmod in the tabs",
                        Some(Action::Page(Page::Mods)),
                    ),
                ]
            } else {
                [
                    ("01", "Download resources from Mojang", Some(Action::Play)),
                    ("02", "Install and verify on this phone", None),
                    (
                        "03",
                        "Play offline or sign in for servers",
                        Some(Action::Account),
                    ),
                ]
            };
            let tiles_top = gy + 52.0;
            let tiles_avail = (gh - 66.0).max(108.0);
            let tile_gap = 8.0;
            let tile_h = ((tiles_avail - 2.0 * tile_gap) / 3.0).clamp(34.0, 56.0);
            for (idx, (num, text, act)) in rows.into_iter().enumerate() {
                let ty = tiles_top + idx as f32 * (tile_h + tile_gap);
                let trect = [gx + 16.0, ty, gw - 32.0, tile_h];
                ui.rounded(trect[0], trect[1], trect[2], trect[3], 10.0, theme::CARD_INNER);
                ui.border(trect[0], trect[1], trect[2], trect[3], 10.0, theme::BORDER);

                let bsize = (tile_h - 12.0).clamp(22.0, 32.0);
                let by = ty + (tile_h - bsize) * 0.5;
                ui.rounded(
                    trect[0] + 10.0,
                    by,
                    bsize + 4.0,
                    bsize,
                    7.0,
                    theme::CRIMSON_BADGE,
                );
                draw_centered_text(
                    ui,
                    Face::Medium,
                    12.5,
                    [trect[0] + 10.0, by, bsize + 4.0, bsize],
                    theme::CRIMSON_BRIGHT,
                    num,
                );
                let line = ui.fit_text(
                    Face::Medium,
                    13.5,
                    text,
                    (trect[2] - bsize - 44.0).max(40.0),
                );
                ui.text(
                    Face::Medium,
                    13.5,
                    trect[0] + bsize + 24.0,
                    text_vcenter(ty, tile_h, 13.5),
                    theme::TEXT,
                    &line,
                );
                ui.text(
                    Face::Medium,
                    14.0,
                    trect[0] + trect[2] - 18.0,
                    text_vcenter(ty, tile_h, 14.0),
                    theme::DIM,
                    ">",
                );
                if let Some(action) = act {
                    self.targets.push((trect, action));
                }
            }
        }

        if !compact {
            let status = self.error.as_deref().unwrap_or(&self.catalog.status);
            if !status.is_empty() {
                let fitted = ui.fit_text(Face::Regular, 13.0, status, cw);
                ui.text(
                    Face::Regular,
                    13.0,
                    cx + 4.0,
                    cy + ch - 24.0,
                    if self.error.is_some() {
                        theme::CRIMSON_BRIGHT
                    } else {
                        theme::MUTED
                    },
                    &fitted,
                );
            }
        }
    }

    fn draw_catalog_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        page: Page,
        account_status: &str,
    ) {
        let [cx, cy, cw, ch] = area;
        let kind = match page {
            Page::Mods => horizon_client::content_files::Kind::Mods,
            Page::Shaders => horizon_client::content_files::Kind::Shaders,
            _ => horizon_client::content_files::Kind::Packs,
        };
        let title = match page {
            Page::Mods => "Horizon Mods",
            Page::Shaders => "Shader Packs",
            _ => "Resource Packs",
        };
        let hint = match page {
            Page::Mods
                if self.catalog.config.mod_source == crate::shader_catalog::Origin::Modrinth =>
            {
                "Modrinth Java mods are browse-only; Fabric/Forge/NeoForge cannot run here"
            }
            Page::Mods => "Search Horizon mods or paste an HTTPS .hmod link",
            Page::Shaders
                if self.catalog.config.shader_source == crate::shader_catalog::Origin::Modrinth =>
            {
                "Modrinth shader ZIPs / experimental compatibility; some packs may not work"
            }
            Page::Shaders => "Horizon catalog / native .hshader & experimental ZIP support",
            _ => "Verified downloads / enable in Installed",
        };

        // Top header row (matching 02-mods.png / 03-packs.png / 04-shaders.png)
        if !compact {
            ui.text(Face::Medium, 24.0, cx, cy, theme::TEXT, title);
            let hint_max = (cw - 450.0).max(140.0);
            let fitted_hint = ui.fit_text(Face::Regular, 13.0, hint, hint_max);
            ui.text(Face::Regular, 13.0, cx, cy + 30.0, theme::MUTED, &fitted_hint);

            let acc_w = 196.0;
            draw_account_chip(
                ui,
                &mut self.targets,
                [cx + cw - acc_w, cy + 4.0, acc_w, 38.0],
                account_status,
                self.login.signed_in,
            );

            if matches!(page, Page::Mods | Page::Shaders) {
                use crate::shader_catalog::Origin;
                let seg_w = 204.0;
                let seg_x = cx + cw - acc_w - seg_w - 12.0;
                ui.rounded(seg_x, cy + 4.0, seg_w, 38.0, 10.0, theme::CARD);
                ui.border(seg_x, cy + 4.0, seg_w, 38.0, 10.0, theme::BORDER);
                let active_origin = if page == Page::Mods {
                    self.catalog.config.mod_source
                } else {
                    self.catalog.config.shader_source
                };
                for (idx, origin) in [Origin::Horizon, Origin::Modrinth].into_iter().enumerate() {
                    let ox = seg_x + 4.0 + idx as f32 * 98.0;
                    let orect = [ox, cy + 8.0, 94.0, 30.0];
                    let is_on = active_origin == origin;
                    if is_on {
                        ui.rounded(orect[0], orect[1], orect[2], orect[3], 7.0, theme::CRIMSON_BADGE);
                        ui.border(orect[0], orect[1], orect[2], orect[3], 7.0, theme::CRIMSON);
                    }
                    draw_centered_text(
                        ui,
                        Face::Medium,
                        13.0,
                        orect,
                        if is_on { theme::ON_CRIMSON } else { theme::MUTED },
                        origin.source().label(),
                    );
                    self.targets.push((
                        orect,
                        if page == Page::Mods {
                            Action::ModSource(origin)
                        } else {
                            Action::ShaderSource(origin)
                        },
                    ));
                }
            }
        } else {
            ui.text(Face::Medium, 18.0, cx, cy, theme::TEXT, title);
            if matches!(page, Page::Mods | Page::Shaders) {
                use crate::shader_catalog::Origin;
                let active_origin = if page == Page::Mods {
                    self.catalog.config.mod_source
                } else {
                    self.catalog.config.shader_source
                };
                for (idx, origin) in [Origin::Horizon, Origin::Modrinth].into_iter().enumerate() {
                    let orect = [cx + cw - 172.0 + idx as f32 * 88.0, cy - 2.0, 84.0, 28.0];
                    styled_button(
                        ui,
                        &mut self.targets,
                        orect,
                        origin.source().label(),
                        if page == Page::Mods {
                            Action::ModSource(origin)
                        } else {
                            Action::ShaderSource(origin)
                        },
                        if active_origin == origin {
                            BtnStyle::Primary
                        } else {
                            BtnStyle::Secondary
                        },
                        true,
                    );
                }
            }
        }

        // Unified Command Toolbar Card (Search + Search CTA + Open folder + Import file)
        let bar_y = cy + if compact { 32.0 } else { 54.0 };
        let bar_h = if compact { 88.0 } else { 52.0 };
        ui.rounded(cx, bar_y, cw, bar_h, 12.0, theme::CARD);
        ui.border(cx, bar_y, cw, bar_h, 12.0, theme::BORDER);

        let search_btn_label = if page == Page::Mods && self.catalog.query.trim().starts_with("https://") {
            "Import .hmod"
        } else {
            "Search"
        };
        let placeholder = match page {
            Page::Mods => "Search Horizon mods or paste .hmod URL...",
            Page::Shaders => "Search shader packs...",
            _ => "Search resource packs...",
        };
        let display_query = if self.catalog.query.is_empty() && self.catalog.editing != Some("search") {
            placeholder.to_owned()
        } else {
            self.catalog.query.clone()
        };
        let query_is_placeholder =
            self.catalog.query.is_empty() && self.catalog.editing != Some("search");

        if compact {
            let s_w = cw - 122.0;
            let srect = [cx + 8.0, bar_y + 8.0, s_w, 34.0];
            ui.rounded(srect[0], srect[1], srect[2], srect[3], 8.0, theme::CARD_INNER);
            ui.border(
                srect[0],
                srect[1],
                srect[2],
                srect[3],
                8.0,
                if self.catalog.editing == Some("search") {
                    theme::CRIMSON
                } else {
                    theme::BORDER
                },
            );
            let qfit = ui.fit_text(Face::Regular, 13.5, &display_query, s_w - 24.0);
            ui.text(
                Face::Regular,
                13.5,
                srect[0] + 12.0,
                text_vcenter(srect[1], srect[3], 13.5),
                if query_is_placeholder {
                    theme::DIM
                } else {
                    theme::TEXT
                },
                &qfit,
            );
            self.targets.push((srect, Action::Field("search")));

            styled_button(
                ui,
                &mut self.targets,
                [cx + cw - 106.0, bar_y + 8.0, 98.0, 34.0],
                search_btn_label,
                Action::Search,
                BtnStyle::Primary,
                true,
            );
            let half = (cw - 24.0) * 0.5;
            styled_button(
                ui,
                &mut self.targets,
                [cx + 8.0, bar_y + 48.0, half, 32.0],
                "Open folder",
                Action::OpenFolder(kind),
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                &mut self.targets,
                [cx + 16.0 + half, bar_y + 48.0, half, 32.0],
                "Import file",
                Action::Import(kind),
                BtnStyle::CrimsonOutline,
                true,
            );
        } else {
            let btn_folder_w = 124.0;
            let btn_import_w = 124.0;
            let btn_search_w = 118.0;
            let input_w = (cw - btn_folder_w - btn_import_w - btn_search_w - 44.0).max(160.0);
            let srect = [cx + 8.0, bar_y + 7.0, input_w, 38.0];
            ui.rounded(srect[0], srect[1], srect[2], srect[3], 9.0, theme::CARD_INNER);
            ui.border(
                srect[0],
                srect[1],
                srect[2],
                srect[3],
                9.0,
                if self.catalog.editing == Some("search") {
                    theme::CRIMSON
                } else {
                    theme::BORDER
                },
            );
            let qfit = ui.fit_text(Face::Regular, 14.0, &display_query, input_w - 28.0);
            ui.text(
                Face::Regular,
                14.0,
                srect[0] + 14.0,
                text_vcenter(srect[1], srect[3], 14.0),
                if query_is_placeholder {
                    theme::DIM
                } else {
                    theme::TEXT
                },
                &qfit,
            );
            self.targets.push((srect, Action::Field("search")));

            let mut bx = srect[0] + input_w + 8.0;
            styled_button(
                ui,
                &mut self.targets,
                [bx, bar_y + 7.0, btn_search_w, 38.0],
                search_btn_label,
                Action::Search,
                BtnStyle::Primary,
                true,
            );
            bx += btn_search_w + 8.0;
            styled_button(
                ui,
                &mut self.targets,
                [bx, bar_y + 7.0, btn_folder_w, 38.0],
                "Open folder",
                Action::OpenFolder(kind),
                BtnStyle::Secondary,
                true,
            );
            bx += btn_folder_w + 8.0;
            styled_button(
                ui,
                &mut self.targets,
                [bx, bar_y + 7.0, btn_import_w, 38.0],
                "Import file",
                Action::Import(kind),
                BtnStyle::CrimsonOutline,
                true,
            );
        }

        // Catalog viewport + bottom pagination bar
        let footer_h = 44.0;
        let top = bar_y + bar_h + 12.0;
        let bottom = cy + ch - footer_h - 10.0;
        let view_h = (bottom - top).max(80.0);

        if self.catalog.results.is_empty() {
            // Atmospheric Empty State Card (matching 04-shaders.png)
            ui.rounded(cx, top, cw, view_h, 14.0, theme::CARD);
            ui.border(cx, top, cw, view_h, 14.0, theme::BORDER);

            let mid_x = cx + cw * 0.5;
            let mid_y = top + view_h * 0.5;

            // Layered crimson diamond/stack emblem
            ui.rounded(mid_x - 36.0, mid_y - 68.0, 72.0, 48.0, 12.0, theme::HERO_GLOW_SOFT);
            for layer in 0..3 {
                let ly = mid_y - 64.0 + layer as f32 * 11.0;
                ui.border(mid_x - 24.0, ly, 48.0, 16.0, 4.0, theme::CRIMSON_BRIGHT);
            }

            let empty_title = if page == Page::Shaders
                && self.catalog.config.shader_source == crate::shader_catalog::Origin::Horizon
            {
                "No native shader packs published yet"
            } else if !self.catalog.status.is_empty() {
                &self.catalog.status
            } else {
                "No packages listed yet"
            };
            draw_centered_text(
                ui,
                Face::Medium,
                18.0,
                [cx + 20.0, mid_y - 8.0, cw - 40.0, 26.0],
                theme::TEXT,
                empty_title,
            );
            let empty_sub = match page {
                Page::Shaders => "Import a .hshader or experimental shaders ZIP to get started.",
                Page::Mods => "Paste an HTTPS .hmod URL above or import a local .hmod package.",
                _ => "Search above or import a resource pack ZIP from your device.",
            };
            draw_centered_text(
                ui,
                Face::Regular,
                13.5,
                [cx + 20.0, mid_y + 20.0, cw - 40.0, 22.0],
                theme::MUTED,
                empty_sub,
            );

            let import_cta = match page {
                Page::Shaders => "Import .hshader / ZIP",
                Page::Mods => "Import .hmod",
                _ => "Import pack ZIP",
            };
            if mid_y + 52.0 + 40.0 <= bottom - 6.0 {
                styled_button(
                    ui,
                    &mut self.targets,
                    [mid_x - 104.0, mid_y + 52.0, 208.0, 40.0],
                    import_cta,
                    Action::Import(kind),
                    BtnStyle::Primary,
                    true,
                );
            }
        } else {
            let cols = if compact {
                1usize
            } else if page == Page::Packs && cw >= 860.0 {
                3usize
            } else {
                2usize
            };
            let gap = 12.0;
            let card_w = (cw - (cols.saturating_sub(1) as f32) * gap) / cols as f32;
            let card_h = if compact {
                122.0
            } else if cols == 3 {
                168.0
            } else {
                156.0
            };
            let row_step = card_h + gap;
            let total_rows = self.catalog.results.len().div_ceil(cols);
            let content_h = (total_rows as f32 * row_step - gap).max(0.0);
            let max_scroll = (content_h - view_h).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            self.list_view = Some([top, bottom, max_scroll]);

            for (i, p) in self.catalog.results.iter().enumerate() {
                let row = i / cols;
                let col_idx = i % cols;
                let x = cx + col_idx as f32 * (card_w + gap);
                let y = top + row as f32 * row_step - self.catalog.scroll;

                let download = self.catalog.download_label(p);
                let can_install = download.is_none()
                    && !crate::catalog::source_only_mod(p)
                    && !(p.kind == crate::browser::Kind::Mod
                        && p.source == crate::browser::SourceId::Modrinth)
                    && !(p.kind == crate::browser::Kind::Mod && !cfg!(target_pointer_width = "64"));

                let btn_w = if cols == 3 { 98.0 } else { 116.0 };
                let btn_h = 34.0;
                let btn_rect = if cols == 3 {
                    [x + card_w - btn_w - 14.0, y + 16.0, btn_w, btn_h]
                } else {
                    [x + card_w - btn_w - 14.0, y + card_h - btn_h - 12.0, btn_w, btn_h]
                };

                if can_install {
                    self.list_targets.push((btn_rect, Action::Install(i)));
                }
                if y + 8.0 < top || y + card_h - 8.0 > bottom {
                    continue;
                }

                ui.rounded(x, y, card_w, card_h, 12.0, theme::CARD);
                ui.border(x, y, card_w, card_h, 12.0, theme::BORDER_CRIMSON);

                if cols == 3 {
                    // 3-Column Resource Pack Card layout (matching 03-packs.png)
                    ui.rounded(x + 14.0, y + 14.0, 44.0, 44.0, 10.0, theme::CRIMSON_BADGE);
                    ui.border(x + 14.0, y + 14.0, 44.0, 44.0, 10.0, theme::BORDER_CRIMSON);
                    draw_page_icon(ui, page, x + 36.0, y + 36.0, theme::CRIMSON_BRIGHT);

                    let title_fit = ui.fit_text(Face::Medium, 16.0, &p.title, card_w - 28.0);
                    ui.text(
                        Face::Medium,
                        16.0,
                        x + 14.0,
                        y + 66.0,
                        theme::CRIMSON_BRIGHT,
                        &title_fit,
                    );
                    let desc_lines =
                        wrap_lines(ui, Face::Regular, 12.5, &p.summary, card_w - 28.0, 3);
                    for (li, line) in desc_lines.iter().enumerate() {
                        ui.text(
                            Face::Regular,
                            12.5,
                            x + 14.0,
                            y + 92.0 + li as f32 * 18.0,
                            theme::MUTED,
                            line,
                        );
                    }
                } else {
                    // 2-Column Rich Mod / Shader Card layout (matching 02-mods.png)
                    let icon_sz = if compact { 42.0 } else { 56.0 };
                    ui.rounded(x + 14.0, y + 14.0, icon_sz, icon_sz, 11.0, theme::HERO_BG);
                    ui.border(x + 14.0, y + 14.0, icon_sz, icon_sz, 11.0, theme::BORDER_CRIMSON);
                    draw_page_icon(
                        ui,
                        page,
                        x + 14.0 + icon_sz * 0.5,
                        y + 14.0 + icon_sz * 0.5,
                        theme::CRIMSON_BRIGHT,
                    );

                    let text_x = x + 26.0 + icon_sz;
                    let dl_text = if p.downloads > 0 {
                        format!("{} dl", format_downloads(p.downloads))
                    } else {
                        String::new()
                    };
                    let dl_w = if dl_text.is_empty() {
                        0.0
                    } else {
                        ui.measure(Face::Regular, 12.0, &dl_text) + 12.0
                    };
                    let title_max = (card_w - (text_x - x) - dl_w - 14.0).max(60.0);
                    let title_fit = ui.fit_text(Face::Medium, 16.5, &p.title, title_max);
                    ui.text(Face::Medium, 16.5, text_x, y + 12.0, theme::TEXT, &title_fit);
                    if !dl_text.is_empty() {
                        ui.text(
                            Face::Regular,
                            12.0,
                            x + card_w - dl_w - 2.0,
                            y + 15.0,
                            theme::MUTED,
                            &dl_text,
                        );
                    }

                    // Tag pills (source + loader + version)
                    if !compact {
                        let mut tx = text_x;
                        let src_tag = match p.source {
                            crate::browser::SourceId::Horizon => "Horizon",
                            crate::browser::SourceId::Modrinth => "Modrinth",
                        };
                        let sw = ui.measure(Face::Medium, 11.0, src_tag) + 14.0;
                        ui.rounded(tx, y + 38.0, sw, 20.0, 6.0, theme::CRIMSON_BADGE);
                        draw_centered_text(
                            ui,
                            Face::Medium,
                            11.0,
                            [tx, y + 38.0, sw, 20.0],
                            theme::CRIMSON_BRIGHT,
                            src_tag,
                        );
                        tx += sw + 6.0;

                        for tag in p
                            .loaders
                            .iter()
                            .take(1)
                            .chain(p.game_versions.iter().take(2))
                        {
                            let tw = ui.measure(Face::Regular, 11.0, tag) + 14.0;
                            if tx + tw > x + card_w - 14.0 {
                                break;
                            }
                            ui.rounded(tx, y + 38.0, tw, 20.0, 6.0, theme::CARD_INNER);
                            draw_centered_text(
                                ui,
                                Face::Regular,
                                11.0,
                                [tx, y + 38.0, tw, 20.0],
                                theme::MUTED,
                                tag,
                            );
                            tx += tw + 6.0;
                        }
                    }

                    let desc_y = if compact { y + 38.0 } else { y + 64.0 };
                    let desc_w = (card_w - (text_x - x) - 14.0).max(80.0);
                    let lines = wrap_lines(ui, Face::Regular, 12.5, &p.summary, desc_w, 2);
                    for (li, line) in lines.iter().enumerate() {
                        ui.text(
                            Face::Regular,
                            12.5,
                            text_x,
                            desc_y + li as f32 * 17.0,
                            theme::MUTED,
                            line,
                        );
                    }

                    // Optional "View details" button (matching 02-mods.png)
                    if !compact
                        && (p.page.starts_with("https://") || p.page.starts_with("http://"))
                        && y + card_h - btn_h - 12.0 >= top
                        && y + card_h - 12.0 <= bottom
                    {
                        styled_button(
                            ui,
                            &mut self.targets,
                            [x + 14.0, y + card_h - btn_h - 12.0, 102.0, btn_h],
                            "View details",
                            Action::OpenProject(i),
                            BtnStyle::Secondary,
                            true,
                        );
                    }
                }

                // Install button or status pill
                if btn_rect[1] >= top && btn_rect[1] + btn_rect[3] <= bottom {
                    if let Some(state) = download {
                        ui.rounded(
                            btn_rect[0],
                            btn_rect[1],
                            btn_rect[2],
                            btn_rect[3],
                            8.0,
                            theme::CARD_INNER,
                        );
                        ui.border(
                            btn_rect[0],
                            btn_rect[1],
                            btn_rect[2],
                            btn_rect[3],
                            8.0,
                            theme::CRIMSON,
                        );
                        draw_centered_text(
                            ui,
                            Face::Medium,
                            13.0,
                            btn_rect,
                            theme::TEXT,
                            &state,
                        );
                    } else if p.kind == crate::browser::Kind::Mod
                        && p.source == crate::browser::SourceId::Modrinth
                    {
                        draw_centered_text(
                            ui,
                            Face::Regular,
                            12.5,
                            btn_rect,
                            theme::MUTED,
                            "Java mod",
                        );
                    } else if crate::catalog::source_only_mod(p) {
                        draw_centered_text(
                            ui,
                            Face::Regular,
                            12.5,
                            btn_rect,
                            theme::MUTED,
                            ".hnmod: later",
                        );
                    } else if p.kind == crate::browser::Kind::Mod
                        && !cfg!(target_pointer_width = "64")
                    {
                        draw_centered_text(
                            ui,
                            Face::Regular,
                            12.0,
                            btn_rect,
                            theme::MUTED,
                            "64-bit req.",
                        );
                    } else {
                        styled_button(
                            ui,
                            &mut self.targets,
                            btn_rect,
                            self.catalog.install_button(p),
                            Action::Install(i),
                            BtnStyle::Primary,
                            true,
                        );
                    }
                }
            }
        }

        // Bottom Pagination & Status Bar (matching 02-mods.png)
        let fy = cy + ch - footer_h;
        ui.rounded(cx, fy, cw, footer_h, 11.0, theme::CARD);
        ui.border(cx, fy, cw, footer_h, 11.0, theme::BORDER);
        styled_button(
            ui,
            &mut self.targets,
            [cx + 6.0, fy + 5.0, 108.0, 34.0],
            "<- Previous",
            Action::Previous,
            BtnStyle::Secondary,
            true,
        );
        styled_button(
            ui,
            &mut self.targets,
            [cx + cw - 114.0, fy + 5.0, 108.0, 34.0],
            "Next ->",
            Action::Next,
            BtnStyle::Primary,
            true,
        );
        if !compact {
            let status_text = self
                .error
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    if !self.catalog.status.is_empty() {
                        &self.catalog.status
                    } else {
                        hint
                    }
                });
            draw_centered_text(
                ui,
                Face::Regular,
                13.0,
                [cx + 124.0, fy, (cw - 248.0).max(40.0), footer_h],
                if self.error.is_some() {
                    theme::CRIMSON_BRIGHT
                } else {
                    theme::MUTED
                },
                status_text,
            );
        }
    }

    fn draw_installed_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        account_status: &str,
    ) {
        use horizon_client::content_files::Kind;
        let [cx, cy, cw, ch] = area;

        if !compact {
            // Top bar: Installed Content + Profile pill + Account chip
            ui.text(Face::Medium, 24.0, cx, cy, theme::TEXT, "Installed Content");
            styled_button(
                ui,
                &mut self.targets,
                [cx + cw - 360.0, cy + 2.0, 148.0, 38.0],
                &format!("Profile {}   v", self.catalog.config.profile),
                Action::Profile,
                BtnStyle::CrimsonOutline,
                false,
            );
            draw_account_chip(
                ui,
                &mut self.targets,
                [cx + cw - 200.0, cy + 2.0, 200.0, 38.0],
                account_status,
                self.login.signed_in,
            );

            // 3-Column Modular Cards (matching 05-installed.png)
            let top = cy + 52.0;
            let footer_h = 40.0;
            let bottom = cy + ch - footer_h - 10.0;
            let card_h = (bottom - top).max(180.0);
            let gap = 12.0;
            let col_w = (cw - 2.0 * gap) / 3.0;

            let max_items = Kind::ALL
                .iter()
                .enumerate()
                .map(|(sec, k)| {
                    if self.catalog.sections[sec] {
                        self.catalog.local.iter().filter(|e| e.kind == *k).count()
                    } else {
                        0
                    }
                })
                .max()
                .unwrap_or(0);
            let inner_top = top + 98.0;
            let inner_bottom = bottom - 12.0;
            let inner_h = (inner_bottom - inner_top).max(60.0);
            let item_h = 68.0;
            let max_scroll = (max_items as f32 * item_h - inner_h).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            self.list_view = Some([inner_top, inner_bottom, max_scroll]);

            for (section, kind) in Kind::ALL.into_iter().enumerate() {
                let col_x = cx + section as f32 * (col_w + gap);
                ui.rounded(col_x, top, col_w, card_h, 13.0, theme::CARD);
                ui.border(col_x, top, col_w, card_h, 13.0, theme::BORDER_CRIMSON);

                let count = self.catalog.local.iter().filter(|e| e.kind == kind).count();
                let header_label = format!("{} ({count})", kind.label());
                let hfit = ui.fit_text(Face::Medium, 16.5, &header_label, col_w - 60.0);
                ui.text(Face::Medium, 16.5, col_x + 16.0, top + 12.0, theme::TEXT, &hfit);

                let chev = if self.catalog.sections[section] { "v" } else { ">" };
                styled_button(
                    ui,
                    &mut self.targets,
                    [col_x + col_w - 42.0, top + 10.0, 28.0, 28.0],
                    chev,
                    Action::ToggleSection(section),
                    BtnStyle::Subtle,
                    true,
                );
                ui.rounded(col_x + 12.0, top + 46.0, col_w - 24.0, 1.0, 0.0, theme::BORDER);

                // Import + Folder action strip
                styled_button(
                    ui,
                    &mut self.targets,
                    [col_x + 14.0, top + 56.0, 96.0, 32.0],
                    "Import",
                    Action::Import(kind),
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    &mut self.targets,
                    [col_x + col_w - 104.0, top + 56.0, 90.0, 32.0],
                    "Folder",
                    Action::OpenFolder(kind),
                    BtnStyle::Secondary,
                    true,
                );

                if !self.catalog.sections[section] {
                    draw_centered_text(
                        ui,
                        Face::Regular,
                        13.0,
                        [col_x + 14.0, inner_top + 20.0, col_w - 28.0, 28.0],
                        theme::MUTED,
                        "Section collapsed — tap > to expand",
                    );
                    continue;
                }

                if count == 0 {
                    // Empty dropzone box inside column (matching 05-installed.png)
                    let dz = [col_x + 14.0, inner_top, col_w - 28.0, inner_h];
                    ui.rounded(dz[0], dz[1], dz[2], dz[3], 10.0, theme::BG);
                    ui.border(dz[0], dz[1], dz[2], dz[3], 10.0, theme::BORDER);
                    let mid_y = dz[1] + dz[3] * 0.5;
                    draw_page_icon(ui, Page::Installed, dz[0] + dz[2] * 0.5, mid_y - 26.0, theme::TEXT);
                    draw_centered_text(
                        ui,
                        Face::Regular,
                        13.5,
                        [dz[0] + 8.0, mid_y - 6.0, dz[2] - 16.0, 22.0],
                        theme::TEXT,
                        "Tap Import to add files",
                    );
                    draw_centered_text(
                        ui,
                        Face::Regular,
                        12.5,
                        [dz[0] + 8.0, mid_y + 18.0, dz[2] - 16.0, 20.0],
                        theme::MUTED,
                        "No items installed",
                    );
                    self.targets.push((dz, Action::Import(kind)));
                } else {
                    let mut idx_in_col = 0usize;
                    for (i, item) in self
                        .catalog
                        .local
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.kind == kind)
                    {
                        let iy = inner_top + idx_in_col as f32 * item_h - self.catalog.scroll;
                        idx_in_col += 1;
                        let toggle_rect = [col_x + 18.0, iy + 32.0, 88.0, 28.0];
                        let del_rect = [col_x + col_w - 92.0, iy + 32.0, 74.0, 28.0];
                        self.list_targets.push((toggle_rect, Action::ToggleLocal(i)));
                        self.list_targets.push((del_rect, Action::DeleteLocal(i)));
                        if iy < inner_top || iy + item_h - 4.0 > inner_bottom {
                            continue;
                        }
                        ui.rounded(
                            col_x + 12.0,
                            iy,
                            col_w - 24.0,
                            item_h - 6.0,
                            9.0,
                            theme::CARD_INNER,
                        );
                        ui.border(
                            col_x + 12.0,
                            iy,
                            col_w - 24.0,
                            item_h - 6.0,
                            9.0,
                            theme::BORDER,
                        );
                        let tfit = ui.fit_text(Face::Medium, 13.5, &item.title, col_w - 40.0);
                        ui.text(Face::Medium, 13.5, col_x + 20.0, iy + 6.0, theme::TEXT, &tfit);
                        if kind == Kind::Mods && !cfg!(target_pointer_width = "64") {
                            ui.text(
                                Face::Regular,
                                12.0,
                                col_x + 20.0,
                                iy + 34.0,
                                theme::MUTED,
                                "64-bit required",
                            );
                        } else {
                            toggle(ui, &mut self.targets, toggle_rect, item.enabled, Action::ToggleLocal(i));
                        }
                        styled_button(
                            ui,
                            &mut self.targets,
                            del_rect,
                            "Remove",
                            Action::DeleteLocal(i),
                            BtnStyle::Secondary,
                            true,
                        );
                    }
                }
            }

            // Bottom status bar (matching 05-installed.png)
            let fy = cy + ch - footer_h;
            ui.rounded(cx, fy, cw, footer_h, 10.0, theme::CARD);
            ui.border(cx, fy, cw, footer_h, 10.0, theme::BORDER);
            let default_msg = format!(
                "Manage enabled mods, packs and shaders for Profile {}",
                self.catalog.config.profile
            );
            let msg = self
                .error
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    if !self.catalog.status.is_empty() {
                        &self.catalog.status
                    } else {
                        &default_msg
                    }
                });
            let fitted = ui.fit_text(Face::Regular, 13.5, msg, cw - 32.0);
            ui.text(
                Face::Regular,
                13.5,
                cx + 16.0,
                text_vcenter(fy, footer_h, 13.5),
                theme::TEXT,
                &fitted,
            );
        } else {
            // Compact portrait stacked sections
            let top = cy;
            let bottom = cy + ch;
            let row_height = 96.0;
            let total = Kind::ALL
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    48.0 + if self.catalog.sections[i] {
                        self.catalog.local.iter().filter(|e| e.kind == *k).count() as f32
                            * row_height
                    } else {
                        0.0
                    }
                })
                .sum::<f32>();
            let maximum = (total - (bottom - top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, maximum);
            self.list_view = Some([top, bottom, maximum]);
            let mut offset = 0.0;
            for (section, kind) in Kind::ALL.into_iter().enumerate() {
                let count = self.catalog.local.iter().filter(|e| e.kind == kind).count();
                let y = top + offset - self.catalog.scroll;
                let label = format!(
                    "{} {} ({count})",
                    if self.catalog.sections[section] { "-" } else { "+" },
                    kind.label()
                );
                let sec_rect = [cx, y, cw - 110.0, 40.0];
                let imp_rect = [cx + cw - 102.0, y, 102.0, 40.0];
                self.list_targets.push((sec_rect, Action::ToggleSection(section)));
                self.list_targets.push((imp_rect, Action::Import(kind)));
                if y >= top && y + 40.0 <= bottom {
                    styled_button(
                        ui,
                        &mut self.targets,
                        sec_rect,
                        &label,
                        Action::ToggleSection(section),
                        BtnStyle::CrimsonOutline,
                        false,
                    );
                    styled_button(
                        ui,
                        &mut self.targets,
                        imp_rect,
                        "Import",
                        Action::Import(kind),
                        BtnStyle::Primary,
                        true,
                    );
                }
                offset += 48.0;
                if !self.catalog.sections[section] {
                    continue;
                }
                for (i, item) in self
                    .catalog
                    .local
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.kind == kind)
                {
                    let y = top + offset - self.catalog.scroll;
                    offset += row_height;
                    let tog_rect = [cx + 12.0, y + 48.0, 104.0, 34.0];
                    let del_rect = [cx + cw - 112.0, y + 48.0, 100.0, 34.0];
                    self.list_targets.push((tog_rect, Action::ToggleLocal(i)));
                    self.list_targets.push((del_rect, Action::DeleteLocal(i)));
                    if y < top || y + row_height - 6.0 > bottom {
                        continue;
                    }
                    ui.rounded(cx, y, cw, row_height - 8.0, 10.0, theme::CARD);
                    ui.border(cx, y, cw, row_height - 8.0, 10.0, theme::BORDER);
                    let tfit = ui.fit_text(Face::Medium, 15.0, &item.title, cw - 24.0);
                    ui.text(Face::Medium, 15.0, cx + 12.0, y + 10.0, theme::TEXT, &tfit);
                    if kind == Kind::Mods && !cfg!(target_pointer_width = "64") {
                        ui.text(
                            Face::Regular,
                            13.0,
                            cx + 12.0,
                            y + 54.0,
                            theme::MUTED,
                            "64-bit required",
                        );
                    } else {
                        toggle(ui, &mut self.targets, tog_rect, item.enabled, Action::ToggleLocal(i));
                    }
                    styled_button(
                        ui,
                        &mut self.targets,
                        del_rect,
                        "Remove",
                        Action::DeleteLocal(i),
                        BtnStyle::Secondary,
                        true,
                    );
                }
            }
        }
    }

    fn draw_settings_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        account_status: &str,
    ) {
        let [cx, cy, cw, ch] = area;
        let c = &self.catalog.config;
        let render_scale_val = if self.catalog.editing == Some("render_scale") {
            self.catalog.render_scale_text.clone()
        } else {
            format!("{}%", c.render_scale.percent())
        };
        let server_val = if c.server.is_empty() {
            "Singleplayer".to_owned()
        } else {
            c.server.clone()
        };

        let left_items: [(&str, String, Action, Option<bool>, bool); 8] = [
            (
                "Rendering",
                c.renderer.label().to_owned(),
                Action::Renderer,
                None,
                true,
            ),
            (
                "Client resolution",
                render_scale_val,
                Action::RenderScale,
                None,
                true,
            ),
            (
                "Orientation",
                c.orientation.label().to_owned(),
                Action::Orientation,
                None,
                true,
            ),
            (
                "Immersive fullscreen",
                String::new(),
                Action::Immersive,
                Some(c.screen.immersive),
                false,
            ),
            (
                "Avoid camera cutout",
                String::new(),
                Action::Cutout,
                Some(c.screen.avoid_cutout),
                false,
            ),
            (
                "Aspect ratio",
                c.screen.aspect.label().to_owned(),
                Action::Aspect,
                None,
                true,
            ),
            (
                "Display refresh rate",
                c.device.display_label(),
                Action::DisplayHz,
                None,
                true,
            ),
            (
                "Performance hints",
                String::new(),
                Action::PerformanceHints,
                Some(c.device.performance_hints),
                false,
            ),
        ];

        let right_items: [(&str, String, Action, Option<bool>, bool); 7] = [
            (
                "Profile",
                format!("Profile {}", c.profile),
                Action::Profile,
                None,
                true,
            ),
            (
                "Input",
                c.input_host.label().to_owned(),
                Action::InputHost,
                None,
                true,
            ),
            ("Server", server_val, Action::Field("server"), None, false),
            (
                "GPU profiling",
                String::new(),
                Action::GpuProfile,
                Some(c.device.gpu_profile),
                false,
            ),
            (
                "Performance recording",
                String::new(),
                Action::RecordPerformance,
                Some(c.record_performance),
                false,
            ),
            (
                "Diagnostic report",
                "Export ZIP".to_owned(),
                Action::ExportDiagnostics,
                None,
                false,
            ),
            (
                "Raw performance data",
                "Open Reports".to_owned(),
                Action::ExportPerformance,
                None,
                false,
            ),
        ];

        if !compact {
            // Two-Column Categorized Settings Deck (matching 06-settings.png)
            let gap = 16.0;
            let col_w = (cw - gap) * 0.5;
            let footer_h = 84.0;
            let cards_h = (ch - footer_h - 12.0).max(200.0);

            ui.rounded(cx, cy, col_w, cards_h, 14.0, theme::CARD);
            ui.border(cx, cy, col_w, cards_h, 14.0, theme::BORDER);
            ui.text(
                Face::Medium,
                18.0,
                cx + 20.0,
                cy + 14.0,
                theme::TEXT,
                "Display & Graphics",
            );

            let rx = cx + col_w + gap;
            ui.rounded(rx, cy, col_w, cards_h, 14.0, theme::CARD);
            ui.border(rx, cy, col_w, cards_h, 14.0, theme::BORDER);
            ui.text(
                Face::Medium,
                18.0,
                rx + 20.0,
                cy + 14.0,
                theme::TEXT,
                "Profile, Input & Diagnostics",
            );

            let list_top = cy + 52.0;
            let list_bottom = cy + cards_h - 10.0;
            let row_h = 52.0;
            let max_rows = left_items.len().max(right_items.len());
            let max_scroll = (max_rows as f32 * row_h - (list_bottom - list_top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            self.list_view = Some([list_top, list_bottom, max_scroll]);

            for (slice, base_x) in [(&left_items[..], cx), (&right_items[..], rx)] {
                for (i, (label, val, act, sw, dropdown)) in slice.iter().enumerate() {
                    let y = list_top + i as f32 * row_h - self.catalog.scroll;
                    let ctrl_w = (col_w * 0.50).clamp(132.0, 236.0);
                    let ctrl_rect = [base_x + col_w - ctrl_w - 16.0, y + 6.0, ctrl_w, 36.0];
                    self.list_targets.push((ctrl_rect, *act));
                    if y < list_top - 4.0 || y + row_h - 4.0 > list_bottom {
                        continue;
                    }
                    let lbl_fit =
                        ui.fit_text(Face::Regular, 14.0, label, col_w - ctrl_w - 40.0);
                    ui.text(
                        Face::Regular,
                        14.0,
                        base_x + 20.0,
                        text_vcenter(y, row_h - 4.0, 14.0),
                        theme::MUTED,
                        &lbl_fit,
                    );
                    if let Some(on) = sw {
                        toggle(ui, &mut self.targets, ctrl_rect, *on, *act);
                    } else {
                        let text = if *dropdown {
                            format!("{val}   v")
                        } else {
                            val.clone()
                        };
                        styled_button(
                            ui,
                            &mut self.targets,
                            ctrl_rect,
                            &text,
                            *act,
                            BtnStyle::Secondary,
                            false,
                        );
                    }
                    if i + 1 < slice.len() {
                        ui.rounded(
                            base_x + 20.0,
                            y + row_h - 2.0,
                            col_w - 36.0,
                            1.0,
                            0.0,
                            theme::BORDER,
                        );
                    }
                }
            }

            // Bottom row matching 06-settings.png: status on left, Startup diagnostics + crimson Microsoft/Xbox account + account pill on right
            let btn_y = cy + cards_h + 12.0;
            let half_right = (col_w - 12.0) * 0.5;
            styled_button(
                ui,
                &mut self.targets,
                [rx, btn_y, half_right, 44.0],
                "Startup diagnostics",
                Action::Logs,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                &mut self.targets,
                [rx + half_right + 12.0, btn_y, half_right, 44.0],
                "Microsoft / Xbox account",
                Action::Account,
                BtnStyle::Primary,
                true,
            );
            draw_account_chip(
                ui,
                &mut self.targets,
                [rx + col_w - 190.0, btn_y + 52.0, 190.0, 30.0],
                account_status,
                self.login.signed_in,
            );

            let status = self.error.as_deref().unwrap_or(&self.catalog.status);
            if !status.is_empty() {
                let lines = wrap_lines(ui, Face::Regular, 13.0, status, col_w, 2);
                for (li, line) in lines.iter().enumerate() {
                    ui.text(
                        Face::Regular,
                        13.0,
                        cx + 4.0,
                        btn_y + 6.0 + li as f32 * 19.0,
                        if self.error.is_some() {
                            theme::CRIMSON_BRIGHT
                        } else {
                            theme::MUTED
                        },
                        line,
                    );
                }
            }
        } else {
            // Compact single-column Settings
            let footer_h = 52.0;
            let top = cy;
            let bottom = cy + ch - footer_h;
            let all_items: Vec<_> = left_items.into_iter().chain(right_items).collect();
            let row_h = 68.0;
            let max_scroll = (all_items.len() as f32 * row_h - (bottom - top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            self.list_view = Some([top, bottom, max_scroll]);

            for (i, (label, val, act, sw, dropdown)) in all_items.iter().enumerate() {
                let y = top + i as f32 * row_h - self.catalog.scroll;
                let ctrl_rect = [cx, y + 24.0, cw, 38.0];
                self.list_targets.push((ctrl_rect, *act));
                if y < top || y + row_h - 4.0 > bottom {
                    continue;
                }
                ui.text(Face::Regular, 13.5, cx + 2.0, y, theme::MUTED, label);
                if let Some(on) = sw {
                    toggle(ui, &mut self.targets, ctrl_rect, *on, *act);
                } else {
                    let text = if *dropdown {
                        format!("{val}   v")
                    } else {
                        val.clone()
                    };
                    styled_button(
                        ui,
                        &mut self.targets,
                        ctrl_rect,
                        &text,
                        *act,
                        BtnStyle::Secondary,
                        false,
                    );
                }
            }
            let half = (cw - 10.0) * 0.5;
            styled_button(
                ui,
                &mut self.targets,
                [cx, cy + ch - 44.0, half, 40.0],
                "Diagnostics",
                Action::Logs,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                &mut self.targets,
                [cx + half + 10.0, cy + ch - 44.0, half, 40.0],
                "Microsoft account",
                Action::Account,
                BtnStyle::Primary,
                true,
            );
        }
    }

    pub(super) fn draw_pages(&mut self) -> Result<()> {
        let (Some(r), Some(gui)) = (&mut self.renderer, &self.gui) else {
            return Ok(());
        };
        if r.suspended {
            return Ok(());
        }
        self.targets.clear();
        self.list_targets.clear();
        self.list_view = None;
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        let mut ui = Ui::mobile(&gui.atlas, &mut b, (0.0, 0.0));
        // OEM navigation bars can remain visible even in fullscreen (keypad phones).
        // Inset the launcher layout only; keep surface and input coordinates intact.
        if let Some(window) = &self.window {
            use winit::platform::android::WindowExtAndroid;
            let rect = window.content_rect();
            let input = winit::platform::android::native_input_scale();
            if rect.bottom > 0 {
                ui.h = ui.h.min(rect.bottom as f32 * input[1] as f32 / ui.density);
            }
            if rect.right > 0 {
                ui.w = ui.w.min(rect.right as f32 * input[0] as f32 / ui.density);
            }
        }
        if let Some(bottom) = crate::native_ime::viewport_bottom() {
            ui.h = ui.h.min(bottom as f32 / ui.density);
        }
        let (w, h) = (ui.w, ui.h);
        let scale = ui.density / ui.batch.scale;
        ui.batch
            .fill(&gui.atlas, 0.0, 0.0, w * scale, h * scale, theme::BG);

        let compact = w < 760.0;
        let account_status = self.login.status(&self.catalog.config.name);
        let content_area = if compact {
            self.draw_compact_header(&mut ui, w, &account_status);
            [12.0, 102.0, (w - 24.0).max(120.0), (h - 148.0).max(120.0)]
        } else {
            let rail_w = 176.0;
            self.draw_sidebar(&mut ui, 12.0, 12.0, rail_w, (h - 24.0).max(260.0));
            [
                12.0 + rail_w + 16.0,
                14.0,
                (w - rail_w - 40.0).max(240.0),
                (h - 28.0).max(200.0),
            ]
        };

        let page = self.catalog.page;
        match page {
            Page::Play => self.draw_play_page(&mut ui, content_area, compact, &account_status),
            Page::Mods | Page::Packs | Page::Shaders => {
                self.draw_catalog_page(&mut ui, content_area, compact, page, &account_status)
            }
            Page::Installed => {
                self.draw_installed_page(&mut ui, content_area, compact, &account_status)
            }
            Page::Settings => {
                self.draw_settings_page(&mut ui, content_area, compact, &account_status)
            }
        }

        if compact {
            let status = self.error.as_deref().unwrap_or(&self.catalog.status);
            if !status.is_empty() {
                let sfit = ui.fit_text(Face::Regular, 12.0, status, (w - 118.0).max(40.0));
                ui.text(Face::Regular, 12.0, 14.0, h - 32.0, theme::MUTED, &sfit);
            }
            styled_button(
                &mut ui,
                &mut self.targets,
                [w - 94.0, h - 38.0, 82.0, 28.0],
                "GitHub",
                Action::Repository,
                BtnStyle::Secondary,
                true,
            );
        }

        if self.login.visible {
            self.list_targets.clear();
            self.list_view = None;
            self.login.draw(
                &mut ui,
                &mut self.targets,
                &self.catalog.config.name,
                self.catalog.editing == Some("name"),
            );
        }
        if let Some(chooser) = &mut self.chooser {
            chooser.draw(&mut ui, &mut self.targets);
            self.list_targets = chooser.navigation(ui.w, ui.h);
            self.list_view = Some(chooser.viewport(ui.h));
        }
        if let Some([top, bottom, maximum]) = self.list_view
            && maximum > 0.0
            && bottom > top
        {
            let scroll = self
                .chooser
                .as_ref()
                .map_or(self.catalog.scroll, |c| c.scroll);
            let height = bottom - top;
            let thumb = (height * height / (height + maximum)).max(20.0).min(height);
            let y = top + (height - thumb) * (scroll / maximum).clamp(0.0, 1.0);
            ui.rounded(w - 8.0, top, 4.0, height, 2.0, theme::BORDER);
            ui.rounded(w - 8.0, y, 4.0, thumb, 2.0, theme::CRIMSON);
        }
        let caret_visible = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .is_ok_and(|t| t.as_millis() % 1000 < 500);
        if caret_visible && let Some(name) = self.catalog.editing {
            let (value, font_sz, pad_x) = match name {
                "name" => (self.catalog.config.name.clone(), 14.5, 14.0),
                "server" => (self.catalog.config.server.clone(), 14.5, 12.0),
                "render_scale" => (self.catalog.render_scale_text.clone(), 14.5, 12.0),
                _ => (self.catalog.query.clone(), 14.0, 14.0),
            };
            let face = if name == "search" {
                Face::Regular
            } else {
                Face::Medium
            };
            for ([x, y, bw, bh], action) in &self.targets {
                if *action == Action::Field(name)
                    || (name == "render_scale" && *action == Action::RenderScale)
                {
                    let text = ui.fit_text(face, font_sz, &value, (bw - 28.0).max(8.0));
                    let caret_x = x + pad_x + ui.measure(face, font_sz, &text) + 1.0;
                    ui.rounded(
                        caret_x.min(x + bw - 8.0),
                        y + (bh - 18.0) * 0.5,
                        1.5,
                        18.0,
                        0.5,
                        theme::CRIMSON_BRIGHT,
                    );
                }
            }
        }
        if let Some(action) = self.pending_focus.take() {
            self.focus = self.targets.iter().position(|(_, a)| *a == action);
        }
        if let Some(index) = self.focus
            && let Some(([x, y, bw, bh], _)) = self.targets.get(index)
        {
            ui.border(
                *x - 2.0,
                *y - 2.0,
                *bw + 4.0,
                *bh + 4.0,
                10.0,
                theme::CRIMSON_BRIGHT,
            );
            self.cursor = (
                f64::from((x + bw / 2.0) * ui.density),
                f64::from((y + bh / 2.0) * ui.density),
            );
        }
        if self.mouse_down || self.drag.as_ref().is_some_and(|drag| !drag.moved) {
            let point = [
                self.cursor.0 as f32 / ui.density,
                self.cursor.1 as f32 / ui.density,
            ];
            if let Some(([x, y, bw, bh], _)) = self.targets.iter().find(|(rect, _)| {
                point[0] >= rect[0]
                    && point[0] <= rect[0] + rect[2]
                    && point[1] >= rect[1]
                    && point[1] <= rect[1] + rect[3]
            }) {
                ui.rounded(
                    *x,
                    *y,
                    *bw,
                    *bh,
                    8.0,
                    horizon_client::hud::rgba(220, 20, 60, 44),
                );
            }
        }
        if !self.mouse_down
            && self.drag.is_none()
            && let Some((action, time)) = self.feedback
            && time.elapsed() < std::time::Duration::from_millis(160)
            && let Some(([x, y, bw, bh], _)) = self.targets.iter().find(|(_, a)| *a == action)
        {
            ui.rounded(
                *x,
                *y,
                *bw,
                *bh,
                8.0,
                horizon_client::hud::rgba(220, 20, 60, 44),
            );
        }
        crate::documents::draw_ui(&mut ui);
        ui.finish();

        if !r.render(&game::loading_frame(&b))? && !b.patches.is_empty() {
            // A suspended/lost surface skips the frame, including its glyph
            // uploads. Rebuild them next time instead of caching invisible text.
            horizon_client::ui_theme::reset_atlas();
        }
        Ok(())
    }

    pub(super) fn activate(&mut self, events: &ActiveEventLoop) -> Result<()> {
        if self.session_worker.is_some() {
            return Ok(());
        }
        let Some(r) = &self.renderer else {
            return Ok(());
        };
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        let ui = Ui::mobile(&self.gui.as_ref().unwrap().atlas, &mut b, (0.0, 0.0));
        let p = self.focus.and_then(|index| self.targets.get(index)).map_or(
            [
                self.cursor.0 as f32 / ui.density,
                self.cursor.1 as f32 / ui.density,
            ],
            |([x, y, w, h], _)| [x + w / 2.0, y + h / 2.0],
        );
        let action = self
            .targets
            .iter()
            .find(|([x, y, w, h], _)| {
                (*x..*x + *w).contains(&p[0]) && (*y..*y + *h).contains(&p[1])
            })
            .map(|(_, a)| *a);
        if self.catalog.busy() || crate::documents::busy() {
            return Ok(());
        }
        self.feedback = action.map(|action| (action, std::time::Instant::now()));
        self.focus = None;
        if action.is_some() && horizon_client::official_install::ready(&self.home) {
            let audio = self.audio.get_or_insert_with(|| {
                let mut audio = horizon_client::audio::Audio::new(
                    &self.home.join(".local/horizon-client/26.3"),
                );
                let settings = horizon_client::settings::Settings::load(
                    &self
                        .catalog
                        .config
                        .game_home(&self.home)
                        .join("options.txt"),
                );
                audio.master = settings.volumes[0];
                let mut volumes = settings.volumes;
                volumes[0] = 1.0;
                horizon_client::settings::set_source_volumes(volumes);
                audio
            });
            audio.play_button_click();
        }
        match action {
            Some(Action::Immersive) => {
                self.catalog.config.screen.immersive ^= true;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
            }
            Some(Action::Cutout) => {
                self.catalog.config.screen.avoid_cutout ^= true;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
            }
            Some(Action::Aspect) => self.open_chooser(chooser::Kind::Aspect),
            Some(Action::ChooseAspect(aspect)) => {
                self.catalog.config.screen.aspect = aspect;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
                self.chooser = None;
            }
            Some(Action::DisplayHz) => self.open_chooser(chooser::Kind::DisplayHz),
            Some(Action::ChooseDisplayHz(hz)) => {
                self.catalog.config.device.display_hz = hz;
                self.catalog.config.device.sanitize();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
                self.chooser = None;
            }
            Some(Action::PerformanceHints) => {
                self.catalog.config.device.performance_hints ^= true;
                self.catalog.status = "ADPF gives Android CPU frame-work hints on supported devices. It does not cap FPS or guarantee a speedup.".into();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
            }
            Some(Action::GpuProfile) => {
                self.catalog.config.device.gpu_profile ^= true;
                self.catalog.status = "Detailed GPU profiling records diagnostics and has overhead. Reports remain on this device.".into();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
            }
            Some(Action::ModSource(origin)) => {
                self.catalog.config.mod_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.catalog.browse(&self.home);
            }
            Some(Action::ShaderSource(origin)) => {
                self.catalog.config.shader_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.catalog.browse(&self.home);
            }
            Some(Action::OpenProject(i)) => {
                if let Some(url) = self.catalog.results.get(i).map(|p| p.page.as_str()) {
                    self.catalog.status = if crate::platform::open_url(url) {
                        "Opened project page in browser.".into()
                    } else {
                        "Could not open project URL.".into()
                    };
                }
            }
            Some(Action::OpenFolder(kind)) => {
                crate::platform::hide();
                let folder = match kind {
                    horizon_client::content_files::Kind::Mods => "mods",
                    horizon_client::content_files::Kind::Packs => "packs",
                    horizon_client::content_files::Kind::Shaders => "shaders",
                };
                let id = crate::document_store::folder_id(self.catalog.config.profile, folder);
                self.catalog.status = if crate::platform::open_documents(&id) {
                    "Opened in Files. Return here after copying files.".into()
                } else {
                    "Open the system Files app and choose Horizon Android.".into()
                };
            }
            Some(Action::Import(kind)) => {
                crate::platform::hide();
                let home = self.catalog.config.game_home(&self.home);
                if let Err(e) = crate::documents::begin(kind, home) {
                    self.catalog.status = e;
                }
            }
            Some(Action::ToggleSection(i)) => {
                self.catalog.sections[i] = !self.catalog.sections[i];
                self.catalog.scroll = 0.0;
            }
            Some(Action::ToggleLocal(i)) => self.catalog.toggle_local(&self.home, i),
            Some(Action::DeleteLocal(i)) => {
                self.catalog.pending_delete = self.catalog.local.get(i).cloned();
                if let Some(item) = &self.catalog.pending_delete {
                    self.chooser = Some(super::chooser::Chooser::confirm_delete(item.file.clone()));
                }
            }
            Some(Action::ConfirmDeleteLocal) => {
                self.catalog.remove_local(&self.home);
                self.chooser = None;
            }
            Some(Action::Repository) => {
                self.catalog.status = if crate::platform::open_repository() {
                    "Repository opened in browser."
                } else {
                    "No browser available to open the repository."
                }
                .into();
            }
            Some(Action::Account) => {
                self.login.visible = true;
            }
            Some(Action::CloseLogin) => {
                self.login.hide();
                crate::platform::hide();
                self.catalog.editing = None;
            }
            Some(Action::DeviceSignIn) => self
                .login
                .start_device(&self.catalog.config.microsoft_client_id),
            Some(Action::CopyLoginCode) => self.login.copy_code(),
            Some(Action::CancelLogin) => {
                self.login.close();
                self.login.visible = true;
            }
            Some(Action::ReviewApp) => self.login.review_app(),
            Some(Action::SignIn) => self.login.start(&self.catalog.config.microsoft_client_id),
            Some(Action::SignOut) => self.login.sign_out(),
            Some(Action::LoginLink) => self.login.open_link(),
            Some(Action::Page(page)) => {
                self.catalog.editing = None;
                if crate::platform::active() {
                    crate::platform::hide();
                }
                self.catalog.page = page;
                self.catalog.scroll = 0.0;
                self.error = None;
                self.catalog.offset = 0;
                self.catalog.browse(&self.home);
            }
            Some(Action::Play) => self.action(events)?,
            Some(Action::Search) => {
                crate::platform::hide();
                self.catalog.offset = 0;
                self.catalog.search(&self.home);
            }
            Some(Action::FinishNameEdit) => {
                self.catalog.editing = None;
                crate::platform::hide();
                self.catalog.config.save(&self.home)?;
            }
            Some(Action::Field(name)) => {
                if name == "render_scale" {
                    self.catalog.render_scale_text =
                        self.catalog.config.render_scale.percent().to_string();
                    self.chooser = None;
                }
                self.catalog.editing = Some(name);
                let initial = match name {
                    "name" => &self.catalog.config.name,
                    "server" => &self.catalog.config.server,
                    "render_scale" => &self.catalog.render_scale_text,
                    "client_id" => &self.catalog.config.microsoft_client_id,
                    _ => &self.catalog.query,
                };
                crate::platform::show(initial);
            }
            Some(Action::Install(i)) => self.catalog.install(&self.home, i),
            Some(Action::Next) => {
                self.catalog.offset += 12;
                self.catalog.search(&self.home);
            }
            Some(Action::Previous) => {
                self.catalog.offset = self.catalog.offset.saturating_sub(12);
                self.catalog.search(&self.home);
            }
            Some(Action::Profile) => self.open_chooser(chooser::Kind::Profile),
            Some(Action::Renderer) => self.open_chooser(chooser::Kind::Renderer),
            Some(Action::RenderScale) => self.open_chooser(chooser::Kind::RenderScale),
            Some(Action::ChooseRenderScale(scale)) => {
                self.catalog.config.render_scale = scale;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::display::set_scale(scale);
            }
            Some(Action::Orientation) => self.open_chooser(chooser::Kind::Orientation),
            Some(Action::InputHost) => self.open_chooser(chooser::Kind::InputHost),
            Some(Action::ChooseInputHost(mode)) => {
                self.catalog.config.input_host = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::configure_input_mode(mode);
            }
            Some(Action::ChooseOrientation(mode)) => {
                self.catalog.config.orientation = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::platform::set_orientation(mode);
            }
            Some(Action::ChooseRenderer(mode)) => {
                self.catalog.config.renderer = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
            }
            Some(Action::ChooseProfile(profile)) => {
                std::fs::create_dir_all(if profile == 0 {
                    self.home.clone()
                } else {
                    self.home
                        .join("profiles")
                        .join(format!("profile-{profile}"))
                })?;
                self.catalog.config.profile = profile;
                self.catalog.config.save(&self.home)?;
                self.catalog.refresh(&self.home);
                self.chooser = None;
            }
            Some(Action::Dismiss) => self.chooser = None,
            Some(Action::Logs) => {
                self.catalog.status = crate::diagnostics::summary(&self.home);
                if let Ok(error) = std::fs::read_to_string(self.home.join("android-error.txt")) {
                    self.catalog.status = format!(
                        "Last handled error: {}. {}",
                        crate::diagnostics::redact(&error),
                        self.catalog.status
                    );
                }
            }
            Some(Action::ExportDiagnostics) if self.diagnostic_export.is_none() => {
                let home = self.home.clone();
                let (sender, receiver) = mpsc::channel();
                self.diagnostic_export = Some(receiver);
                self.catalog.status =
                    "Preparing local diagnostic ZIP. No upload; review before sharing.".into();
                std::thread::spawn(move || {
                    let result = crate::diagnostics::export(&home).map(|p| {
                        p.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned()
                    });
                    let _ = sender.send(result);
                });
            }
            Some(Action::ExportDiagnostics) => {}
            Some(Action::RecordPerformance) => {
                self.catalog.config.record_performance = !self.catalog.config.record_performance;
                if !self.catalog.config.record_performance {
                    crate::performance::stop();
                }
                self.catalog.config.save(&self.home)?;
                self.catalog.status = if self.catalog.config.record_performance {
                    "Recording starts on Play. Detailed timings add overhead. Return here to save the report."
                } else {
                    "Performance recording off."
                }
                .into();
            }
            Some(Action::ExportPerformance) => {
                self.catalog.status = match crate::performance::export(&self.home) {
                    Ok(()) => "Reports opened in Files. Copy or share a report from there.".into(),
                    Err(e) => e,
                };
            }
            None => {}
        }
        Ok(())
    }

    pub(super) fn ui_density(&self) -> f32 {
        let (Some(r), Some(gui)) = (&self.renderer, &self.gui) else {
            return 1.0;
        };
        let mut batch = HudBatch::new([r.config.width, r.config.height]);
        Ui::mobile(&gui.atlas, &mut batch, (0.0, 0.0)).density
    }

    pub(super) fn touch_event(&mut self, events: &ActiveEventLoop, t: Touch) -> Result<()> {
        self.focus = None;
        self.pending_focus = None;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        self.cursor = (t.location.x, t.location.y);
        match t.phase {
            TouchPhase::Started => {
                self.drag = Some(super::gesture::Drag::new(t.location.x, t.location.y));
            }
            TouchPhase::Moved => {
                let density = self.ui_density() as f64;
                if let Some(drag) = &mut self.drag {
                    let delta = drag.update(t.location.x, t.location.y, density);
                    if drag.moved {
                        if let Some(chooser) = &mut self.chooser {
                            chooser.scroll = (chooser.scroll + delta).max(0.0);
                        } else if self
                            .list_view
                            .is_some_and(|view| drag.origin_y() / density >= view[0] as f64)
                        {
                            self.catalog.scroll = (self.catalog.scroll + delta).max(0.0);
                        }
                    }
                }
            }
            TouchPhase::Ended => {
                if self.drag.take().is_some_and(|drag| !drag.moved) {
                    self.activate(events)?;
                }
            }
            TouchPhase::Cancelled => {
                self.drag = None;
            }
        }
        Ok(())
    }

    pub(super) fn type_key(&mut self, key: KeyEvent) {
        if key.physical_key == PhysicalKey::Code(KeyCode::Escape)
            || key.logical_key
                == winit::keyboard::Key::Named(winit::keyboard::NamedKey::BrowserBack)
        {
            if self.login.visible {
                self.login.hide();
                crate::platform::hide();
                self.catalog.editing = None;
                return;
            }
            if self.chooser.take().is_some() {
                return;
            }
            crate::platform::hide();
            self.catalog.editing = None;
            let _ = self.catalog.config.save(&self.home);
            return;
        }
        if self.catalog.editing.is_none() {
            return;
        }
        let out = match key.physical_key {
            PhysicalKey::Code(KeyCode::Backspace) => Some(Output::Backspace),
            PhysicalKey::Code(KeyCode::Enter) => {
                crate::platform::hide();
                Some(Output::Enter)
            }
            _ => key.text.map(|s| Output::Text(s.into())),
        };
        if let Some(out) = out {
            self.catalog.edit(out, &self.home);
        }
    }
}
