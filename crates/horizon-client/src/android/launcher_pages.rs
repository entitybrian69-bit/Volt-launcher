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
    Discord,
    AccountProviderTab(crate::volt_accounts::AccountProvider),
    SelectVoltAccount(usize),
    RemoveVoltAccount(usize),
    CreateOfflineAccount,
    UpdateOfflineName,
    ElySignIn,
    ElyRegister,
    ToggleSlimModel,
    ToggleServerSkinSync,
    ImportSkinPng,
    ImportCapePng,
    FetchSkinByUsername,
    LoadSkinUrl,
    LoadCapeUrl,
    EquipPresetCape(crate::volt_accounts::PresetCape),
    ClearCustomSkin,
    ClearCustomCape,
    ResetPreviewAngle,
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
    ImportControl,
    ControlLayouts,
    ChooseControlLayout(usize),
    ApplyControlsAllProfiles,
    ResetControls,
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
            ui.border(cx - 8.5, cy - 8.5, 17.0, 17.0, 8.5, col);
            ui.rounded(cx - 3.0, cy - 4.5, 2.5, 9.0, 1.0, col);
            ui.rounded(cx - 1.0, cy - 3.0, 2.5, 6.0, 1.0, col);
            ui.rounded(cx + 1.0, cy - 1.5, 2.5, 3.0, 1.0, col);
        }
        Page::Account => {
            ui.border(cx - 4.0, cy - 7.5, 8.0, 8.0, 4.0, col);
            ui.border(cx - 7.5, cy + 1.5, 15.0, 6.5, 3.2, col);
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

fn draw_discord_icon(ui: &mut Ui, cx: f32, cy: f32, col: u32, bg: u32) {
    // Crisp recognizable Discord Clyde controller silhouette
    ui.rounded(cx - 8.0, cy - 5.5, 16.0, 11.0, 4.5, col);
    ui.rounded(cx - 5.5, cy - 7.0, 4.0, 2.5, 1.2, col);
    ui.rounded(cx + 1.5, cy - 7.0, 4.0, 2.5, 1.2, col);
    ui.rounded(cx - 4.5, cy - 1.8, 3.0, 3.2, 1.5, bg);
    ui.rounded(cx + 1.5, cy - 1.8, 3.0, 3.2, 1.5, bg);
    ui.rounded(cx - 2.5, cy + 4.0, 5.0, 2.0, 1.0, bg);
}

fn draw_github_icon(ui: &mut Ui, cx: f32, cy: f32, col: u32) {
    ui.border(cx - 7.5, cy - 7.5, 15.0, 15.0, 7.5, col);
    ui.rounded(cx - 4.5, cy - 3.0, 9.0, 5.5, 2.5, col);
    ui.rounded(cx - 5.0, cy - 5.0, 2.5, 3.0, 1.0, col);
    ui.rounded(cx + 2.5, cy - 5.0, 2.5, 3.0, 1.0, col);
    ui.rounded(cx - 1.5, cy + 2.0, 3.0, 4.5, 1.0, col);
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
    targets.push((rect, Action::Page(Page::Account)));
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

}

struct DrawCtx<'a> {
    home: &'a PathBuf,
    install: &'a Arc<Mutex<horizon_client::official_install::InstallProgress>>,
    worker: &'a Option<mpsc::Receiver<std::result::Result<(), String>>>,
    error: &'a Option<String>,
    catalog: &'a mut Catalog,
    targets: &'a mut Vec<([f32; 4], Action)>,
    list_targets: &'a mut Vec<([f32; 4], Action)>,
    list_view: &'a mut Option<[f32; 3]>,
    login: &'a login::Login,
    volt_accounts: &'a crate::volt_accounts::VoltAccountManager,
    preview_yaw: f32,
    preview_pitch: f32,
    idle_phase: f32,
    preview_rect: &'a mut Option<[f32; 4]>,
}

impl DrawCtx<'_> {
    fn draw_sidebar(&mut self, ui: &mut Ui, rx: f32, ry: f32, rw: f32, rh: f32) {
        ui.rounded(rx, ry, rw, rh, 14.0, theme::RAIL);
        ui.border(rx, ry, rw, rh, 14.0, theme::BORDER_CRIMSON);

        // Brand emblem + VOLT LAUNCHER header
        let bx = rx + 12.0;
        let by = ry + 13.0;
        ui.rounded(bx, by, 34.0, 34.0, 9.0, theme::BG);
        ui.border(bx, by, 34.0, 34.0, 9.0, theme::CRIMSON);
        ui.volt_logo(bx + 3.0, by + 3.0, 28.0, 28.0);
        ui.text(Face::Medium, 18.0, bx + 42.0, by - 1.0, theme::TEXT, "VOLT");
        ui.text(
            Face::Medium,
            10.5,
            bx + 42.0,
            by + 18.0,
            theme::MUTED,
            &format!("LAUNCHER {}", crate::backend::VERSION),
        );
        ui.rounded(rx + 12.0, ry + 56.0, rw - 24.0, 1.0, 0.0, theme::BORDER);

        // Full-height vertical navigation items filling the entire sidebar!
        let nav_top = ry + 64.0;
        let nav_bottom = ry + rh - 96.0;
        let avail = (nav_bottom - nav_top).max(252.0);
        let step = avail / Page::ALL.len() as f32;
        let item_h = (step - 6.0).max(34.0);

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
                    10.0,
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
                    10.0,
                    if page == Page::Play {
                        theme::CRIMSON_BRIGHT
                    } else {
                        theme::CRIMSON
                    },
                );
                ui.rounded(
                    rx + 2.5,
                    iy + (item_h * 0.18).clamp(6.0, 12.0),
                    4.0,
                    (item_h * 0.64).max(16.0),
                    2.0,
                    theme::CRIMSON_BRIGHT,
                );
            } else {
                ui.rounded(rect[0], rect[1], rect[2], rect[3], 10.0, theme::CARD);
                ui.border(rect[0], rect[1], rect[2], rect[3], 10.0, theme::BORDER);
            }
            let fg = if active {
                theme::ON_CRIMSON
            } else {
                theme::TEXT
            };
            draw_page_icon(ui, page, rect[0] + 20.0, iy + item_h * 0.5, fg);
            ui.text(
                Face::Medium,
                15.5,
                rect[0] + 40.0,
                text_vcenter(iy, item_h, 15.5),
                fg,
                page.name(),
            );
            self.targets.push((rect, Action::Page(page)));
        }

        // Discord button directly above GitHub button
        let disc_rect = [rx + 12.0, ry + rh - 86.0, rw - 24.0, 36.0];
        ui.rounded(disc_rect[0], disc_rect[1], disc_rect[2], disc_rect[3], 9.0, theme::CRIMSON_BADGE);
        ui.border(disc_rect[0], disc_rect[1], disc_rect[2], disc_rect[3], 9.0, theme::BORDER_CRIMSON);
        draw_discord_icon(
            ui,
            disc_rect[0] + 26.0,
            disc_rect[1] + disc_rect[3] * 0.5,
            theme::CRIMSON_BRIGHT,
            theme::CRIMSON_BADGE,
        );
        ui.text(
            Face::Medium,
            13.5,
            disc_rect[0] + 44.0,
            text_vcenter(disc_rect[1], disc_rect[3], 13.5),
            theme::TEXT,
            "Discord",
        );
        self.targets.push((disc_rect, Action::Discord));

        // GitHub button at bottom
        let gh_rect = [rx + 12.0, ry + rh - 44.0, rw - 24.0, 34.0];
        ui.rounded(gh_rect[0], gh_rect[1], gh_rect[2], gh_rect[3], 9.0, theme::CARD_INNER);
        ui.border(gh_rect[0], gh_rect[1], gh_rect[2], gh_rect[3], 9.0, theme::BORDER);
        draw_github_icon(
            ui,
            gh_rect[0] + 26.0,
            gh_rect[1] + gh_rect[3] * 0.5,
            theme::TEXT,
        );
        ui.text(
            Face::Medium,
            13.5,
            gh_rect[0] + 44.0,
            text_vcenter(gh_rect[1], gh_rect[3], 13.5),
            theme::TEXT,
            "GitHub",
        );
        self.targets.push((gh_rect, Action::Repository));
    }

    fn draw_compact_header(&mut self, ui: &mut Ui, w: f32, account_status: &str) {
        ui.rounded(12.0, 10.0, 168.0, 34.0, 9.0, theme::CARD);
        ui.border(12.0, 10.0, 168.0, 34.0, 9.0, theme::BORDER_CRIMSON);
        ui.volt_logo(18.0, 14.0, 26.0, 26.0);
        ui.text(
            Face::Medium,
            12.5,
            50.0,
            text_vcenter(10.0, 34.0, 12.5),
            theme::TEXT,
            "VOLT LAUNCHER",
        );
        draw_account_chip(
            ui,
            self.targets,
            [188.0, 10.0, (w - 200.0).max(110.0), 34.0],
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
            let hero_h = (ch * 0.42).clamp(176.0, 240.0);
            (
                [cx, cy, cw, hero_h],
                [cx, cy + hero_h + 12.0, cw, (ch - hero_h - 12.0).max(200.0)],
            )
        } else {
            let hero_w = (cw * 0.56).round();
            let right_w = cw - hero_w - 16.0;
            (
                [cx, cy, hero_w, ch - 30.0],
                [cx + hero_w + 16.0, cy, right_w, ch - 30.0],
            )
        };

        // Left Hero Card (Launch overview + active identity; PLAY moved to bottom-right!)
        let [hx, hy, hw, hh] = hero_rect;
        ui.rounded(hx, hy, hw, hh, 16.0, theme::HERO_BG);
        ui.rounded(hx + 8.0, hy + 8.0, hw * 0.65, hh * 0.45, 14.0, theme::HERO_GLOW_SOFT);
        ui.rounded(hx + 16.0, hy + 14.0, hw * 0.42, hh * 0.28, 12.0, theme::HERO_GLOW);
        ui.border(hx, hy, hw, hh, 16.0, theme::BORDER_CRIMSON);

        if !compact {
            let lx = hx + hw - 86.0;
            let ly = hy + 24.0;
            ui.rounded(lx, ly, 60.0, 60.0, 14.0, theme::BG);
            ui.border(lx, ly, 60.0, 60.0, 14.0, theme::BORDER_CRIMSON);
            ui.volt_logo(lx + 6.0, ly + 6.0, 48.0, 48.0);
        }

        let heading = if installing {
            "Getting Game Ready"
        } else if installed {
            "Ready to Play"
        } else if compact {
            "Minecraft Java 26.3"
        } else {
            "Java Worlds on Android"
        };
        let heading_sz = if compact { 24.0 } else { 36.0 };
        let heading_max = if compact { hw - 52.0 } else { hw - 124.0 };
        let fitted_heading = ui.fit_text(Face::Medium, heading_sz, heading, heading_max);
        ui.text(
            Face::Medium,
            heading_sz,
            hx + 26.0,
            hy + if compact { 16.0 } else { 30.0 },
            theme::TEXT,
            &fitted_heading,
        );

        // Version badge pill
        let badge_y = hy + if compact { 56.0 } else { 88.0 };
        let ver_w = 196.0f32.min(hw - 52.0);
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

        // Profile selector pill
        let prof_y = badge_y + if compact { 40.0 } else { 46.0 };
        styled_button(
            ui,
            self.targets,
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
            let bar_y = hy + hh - 42.0;
            let avail_rows = bar_y - list_top - 8.0;
            if avail_rows >= 120.0 {
                let step_h = (avail_rows / 6.0).clamp(18.0, 26.0);
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
                    15.0,
                    hx + 26.0,
                    list_top + 4.0,
                    theme::TEXT,
                    progress.stage.label(),
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
        } else if !compact {
            // Active loadout summary card anchored cleanly at the bottom of the left hero card
            let info_h = 84.0f32.min((hy + hh - (prof_y + 52.0) - 22.0).max(0.0));
            if info_h >= 64.0 {
                let iy = hy + hh - info_h - 22.0;
                ui.rounded(hx + 26.0, iy, hw - 52.0, info_h, 12.0, theme::CARD);
                ui.border(hx + 26.0, iy, hw - 52.0, info_h, 12.0, theme::BORDER_CRIMSON);
                let active_acc = self.volt_accounts.active();
                let (acc_name, badge_txt, skin_txt, cape_txt) = match active_acc {
                    Some(a) => (
                        a.name.as_str(),
                        a.provider.badge(),
                        a.skin_label.as_str(),
                        a.cape_label.as_str(),
                    ),
                    None => (account_status, "OFFLINE", "Default", "None"),
                };
                let title = ui.fit_text(
                    Face::Medium,
                    15.5,
                    &format!("{acc_name}  •  {badge_txt}"),
                    hw - 84.0,
                );
                ui.text(Face::Medium, 15.5, hx + 42.0, iy + 14.0, theme::TEXT, &title);
                let sub = ui.fit_text(
                    Face::Regular,
                    12.5,
                    &format!("Skin: {skin_txt}   |   Cape: {cape_txt}   |   Renderer: {}", self.catalog.config.renderer.label()),
                    hw - 84.0,
                );
                ui.text(Face::Regular, 12.5, hx + 42.0, iy + 42.0, theme::MUTED, &sub);
                self.targets
                    .push(([hx + 26.0, iy, hw - 52.0, info_h], Action::Page(Page::Account)));
            }
        }

        // Right Column: Interactive Rotatable 3D Minecraft Character + Bottom-Right PLAY Button!
        let [rx, ry, rw, rh] = right_rect;
        let cta_h = if compact { 50.0 } else { 60.0 };
        let char_h = (rh - cta_h - 14.0).max(140.0);
        let char_card = [rx, ry, rw, char_h];

        ui.rounded(char_card[0], char_card[1], char_card[2], char_card[3], 16.0, theme::CARD);
        ui.rounded(
            char_card[0] + char_card[2] * 0.16,
            char_card[1] + char_card[3] * 0.14,
            char_card[2] * 0.68,
            char_card[3] * 0.68,
            18.0,
            theme::HERO_GLOW_SOFT,
        );
        ui.border(
            char_card[0],
            char_card[1],
            char_card[2],
            char_card[3],
            16.0,
            theme::BORDER_CRIMSON,
        );

        let active_acc = self.volt_accounts.active();
        let (acc_name, provider_badge, default_uuid) = match active_acc {
            Some(a) => (a.name.as_str(), a.provider.badge(), a.uuid_u128()),
            None => ("Player", "OFFLINE", 0u128),
        };

        // Header badge inside 3D character showcase card
        let pill_w = (ui.measure(Face::Medium, 11.5, provider_badge) + 20.0).min(110.0);
        ui.rounded(rx + 16.0, ry + 14.0, pill_w, 24.0, 7.0, theme::CRIMSON_BADGE);
        ui.border(rx + 16.0, ry + 14.0, pill_w, 24.0, 7.0, theme::BORDER_CRIMSON);
        draw_centered_text(
            ui,
            Face::Medium,
            11.5,
            [rx + 16.0, ry + 14.0, pill_w, 24.0],
            theme::CRIMSON_BRIGHT,
            provider_badge,
        );

        let name_fit = ui.fit_text(
            Face::Medium,
            15.5,
            acc_name,
            (rw - pill_w - 114.0).max(48.0),
        );
        ui.text(
            Face::Medium,
            15.5,
            rx + 24.0 + pill_w,
            text_vcenter(ry + 14.0, 24.0, 15.5),
            theme::TEXT,
            &name_fit,
        );

        // Quick 180-deg flip button (registered BEFORE char_card so tapping Flip hits first!)
        styled_button(
            ui,
            self.targets,
            [rx + rw - 78.0, ry + 12.0, 62.0, 28.0],
            "Flip",
            Action::ResetPreviewAngle,
            BtnStyle::Secondary,
            true,
        );

        // Subtle crimson pedestal glow under feet
        let ped_w = (rw * 0.42).clamp(76.0, 150.0);
        let ped_y = ry + char_h - 36.0;
        ui.rounded(
            rx + (rw - ped_w) * 0.5,
            ped_y,
            ped_w,
            8.0,
            4.0,
            theme::CRIMSON_BADGE,
        );

        // Draw rotatable GPU 3D Minecraft character + 3D cape!
        let model_bounds = [
            rx + 12.0,
            ry + 42.0,
            (rw - 24.0).max(40.0),
            (char_h - 72.0).max(60.0),
        ];
        crate::skin_preview::draw_player_3d(
            ui,
            model_bounds,
            self.preview_yaw,
            self.preview_pitch,
            self.idle_phase,
            0,
            1,
            self.volt_accounts.active_skin_preview(),
            self.volt_accounts.active_cape_preview(),
            default_uuid,
        );

        let hint = ui.fit_text(
            Face::Regular,
            12.0,
            "Drag to rotate  •  Tap character for Account & Skins",
            rw - 28.0,
        );
        draw_centered_text(
            ui,
            Face::Regular,
            12.0,
            [rx + 14.0, ry + char_h - 26.0, rw - 28.0, 20.0],
            theme::MUTED,
            &hint,
        );

        *self.preview_rect = Some(char_card);
        self.targets.push((char_card, Action::Page(Page::Account)));

        // Bottom-Right PLAY CTA Button!
        let cta_y = ry + rh - cta_h;
        let cta_label = if installing {
            "INSTALLING..."
        } else if installed {
            "PLAY"
        } else if self.error.is_some() {
            "RETRY INSTALLATION"
        } else {
            "DOWNLOAD MINECRAFT 26.3"
        };
        styled_button(
            ui,
            self.targets,
            [rx, cta_y, rw, cta_h],
            cta_label,
            Action::Play,
            BtnStyle::Primary,
            true,
        );

        if !compact {
            let status = self.error.as_deref().unwrap_or(&self.catalog.status);
            if !status.is_empty() {
                let fitted = ui.fit_text(Face::Regular, 13.0, status, cw);
                ui.text(
                    Face::Regular,
                    13.0,
                    cx + 4.0,
                    cy + ch - 22.0,
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
                self.targets,
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
                        self.targets,
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
                self.targets,
                [cx + cw - 106.0, bar_y + 8.0, 98.0, 34.0],
                search_btn_label,
                Action::Search,
                BtnStyle::Primary,
                true,
            );
            let half = (cw - 24.0) * 0.5;
            styled_button(
                ui,
                self.targets,
                [cx + 8.0, bar_y + 48.0, half, 32.0],
                "Open folder",
                Action::OpenFolder(kind),
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
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
                self.targets,
                [bx, bar_y + 7.0, btn_search_w, 38.0],
                search_btn_label,
                Action::Search,
                BtnStyle::Primary,
                true,
            );
            bx += btn_search_w + 8.0;
            styled_button(
                ui,
                self.targets,
                [bx, bar_y + 7.0, btn_folder_w, 38.0],
                "Open folder",
                Action::OpenFolder(kind),
                BtnStyle::Secondary,
                true,
            );
            bx += btn_folder_w + 8.0;
            styled_button(
                ui,
                self.targets,
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
                    self.targets,
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
            *self.list_view = Some([top, bottom, max_scroll]);

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
                        let src_tag = p.source.label();
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
                            self.targets,
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
                            self.targets,
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
            self.targets,
            [cx + 6.0, fy + 5.0, 108.0, 34.0],
            "<- Previous",
            Action::Previous,
            BtnStyle::Secondary,
            true,
        );
        styled_button(
            ui,
            self.targets,
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
                self.targets,
                [cx + cw - 360.0, cy + 2.0, 148.0, 38.0],
                &format!("Profile {}   v", self.catalog.config.profile),
                Action::Profile,
                BtnStyle::CrimsonOutline,
                false,
            );
            draw_account_chip(
                ui,
                self.targets,
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
            *self.list_view = Some([inner_top, inner_bottom, max_scroll]);

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
                    self.targets,
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
                    self.targets,
                    [col_x + 14.0, top + 56.0, 96.0, 32.0],
                    "Import",
                    Action::Import(kind),
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
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
                            toggle(ui, self.targets, toggle_rect, item.enabled, Action::ToggleLocal(i));
                        }
                        styled_button(
                            ui,
                            self.targets,
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
            *self.list_view = Some([top, bottom, maximum]);
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
                        self.targets,
                        sec_rect,
                        &label,
                        Action::ToggleSection(section),
                        BtnStyle::CrimsonOutline,
                        false,
                    );
                    styled_button(
                        ui,
                        self.targets,
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
                        toggle(ui, self.targets, tog_rect, item.enabled, Action::ToggleLocal(i));
                    }
                    styled_button(
                        ui,
                        self.targets,
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

        let active_ctrl_summary = if let Some(m) = &self.catalog.active_controls {
            format!(
                "Active: {} • {} ({} buttons{})",
                m.name,
                m.format,
                m.buttons,
                if m.has_joystick { ", joystick" } else { "" }
            )
        } else {
            format!(
                "Active: Default Volt Touch Controls (Profile {})",
                c.profile
            )
        };
        let saved_label = format!("Saved Layouts ({})", self.catalog.control_presets.len());

        if !compact {
            // Dedicated "Import Control" Hero Section Card at top of Settings
            let import_h = 96.0;
            ui.rounded(cx, cy, cw, import_h, 14.0, theme::HERO_BG);
            ui.rounded(
                cx + 8.0,
                cy + 8.0,
                cw * 0.46,
                import_h - 16.0,
                12.0,
                theme::HERO_GLOW_SOFT,
            );
            ui.border(cx, cy, cw, import_h, 14.0, theme::BORDER_CRIMSON);

            let btn_grid_w = (cw * 0.45).clamp(256.0, 344.0);
            let btn_x = cx + cw - btn_grid_w - 16.0;
            let half_btn_w = (btn_grid_w - 8.0) * 0.5;
            let text_max = (btn_x - cx - 32.0).max(140.0);

            ui.text(
                Face::Medium,
                18.0,
                cx + 20.0,
                cy + 11.0,
                theme::TEXT,
                "Import Control",
            );
            if text_max >= 260.0 {
                let badge_x = cx + 162.0;
                ui.rounded(badge_x, cy + 12.0, 108.0, 22.0, 6.0, theme::CRIMSON_BADGE);
                ui.border(badge_x, cy + 12.0, 108.0, 22.0, 6.0, theme::BORDER_CRIMSON);
                draw_centered_text(
                    ui,
                    Face::Medium,
                    11.5,
                    [badge_x, cy + 12.0, 108.0, 22.0],
                    theme::CRIMSON_BRIGHT,
                    "Java Controls",
                );
            }

            let active_fit = ui.fit_text(Face::Medium, 13.5, &active_ctrl_summary, text_max);
            ui.text(
                Face::Medium,
                13.5,
                cx + 20.0,
                cy + 41.0,
                if self.catalog.active_controls.is_some() {
                    theme::CRIMSON_BRIGHT
                } else {
                    theme::TEXT
                },
                &active_fit,
            );
            let sub_hint =
                "Import .json / .zip from PojavLauncher, Amethyst, Mojo, Zalith or FCL";
            let sub_fit = ui.fit_text(Face::Regular, 12.5, sub_hint, text_max);
            ui.text(
                Face::Regular,
                12.5,
                cx + 20.0,
                cy + 65.0,
                theme::MUTED,
                &sub_fit,
            );

            styled_button(
                ui,
                self.targets,
                [btn_x, cy + 13.0, half_btn_w, 34.0],
                "Import Control",
                Action::ImportControl,
                BtnStyle::Primary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x + half_btn_w + 8.0, cy + 13.0, half_btn_w, 34.0],
                &saved_label,
                Action::ControlLayouts,
                BtnStyle::CrimsonOutline,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x, cy + 53.0, half_btn_w, 31.0],
                "Apply All Profiles",
                Action::ApplyControlsAllProfiles,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x + half_btn_w + 8.0, cy + 53.0, half_btn_w, 31.0],
                "Reset Default",
                Action::ResetControls,
                BtnStyle::Secondary,
                true,
            );

            // Two-Column Categorized Settings Deck below Import Control
            let gap = 16.0;
            let col_w = (cw - gap) * 0.5;
            let footer_h = 52.0;
            let cards_y = cy + import_h + 10.0;
            let cards_h = (ch - import_h - 10.0 - footer_h - 10.0).max(142.0);

            ui.rounded(cx, cards_y, col_w, cards_h, 14.0, theme::CARD);
            ui.border(cx, cards_y, col_w, cards_h, 14.0, theme::BORDER);
            ui.text(
                Face::Medium,
                17.0,
                cx + 20.0,
                cards_y + 12.0,
                theme::TEXT,
                "Display & Graphics",
            );

            let rx = cx + col_w + gap;
            ui.rounded(rx, cards_y, col_w, cards_h, 14.0, theme::CARD);
            ui.border(rx, cards_y, col_w, cards_h, 14.0, theme::BORDER);
            ui.text(
                Face::Medium,
                17.0,
                rx + 20.0,
                cards_y + 12.0,
                theme::TEXT,
                "Profile, Input & Diagnostics",
            );

            let list_top = cards_y + 46.0;
            let list_bottom = cards_y + cards_h - 8.0;
            let row_h = 48.0;
            let max_rows = left_items.len().max(right_items.len());
            let max_scroll = (max_rows as f32 * row_h - (list_bottom - list_top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([list_top, list_bottom, max_scroll]);

            for (slice, base_x) in [(&left_items[..], cx), (&right_items[..], rx)] {
                for (i, (label, val, act, sw, dropdown)) in slice.iter().enumerate() {
                    let y = list_top + i as f32 * row_h - self.catalog.scroll;
                    let ctrl_w = (col_w * 0.50).clamp(132.0, 236.0);
                    let ctrl_rect = [base_x + col_w - ctrl_w - 16.0, y + 5.0, ctrl_w, 34.0];
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
                        toggle(ui, self.targets, ctrl_rect, *on, *act);
                    } else {
                        let text = if *dropdown {
                            format!("{val}   v")
                        } else {
                            val.clone()
                        };
                        styled_button(
                            ui,
                            self.targets,
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

            // Bottom row: status on left, Diagnostics + Microsoft/Xbox + account chip inline on right
            let btn_y = cards_y + cards_h + 10.0;
            let acc_chip_w = 152.0f32.min(col_w * 0.36);
            let btns_avail = (col_w - acc_chip_w - 16.0).max(160.0);
            let half_right = (btns_avail - 8.0) * 0.5;
            styled_button(
                ui,
                self.targets,
                [rx, btn_y, half_right, 40.0],
                "Diagnostics",
                Action::Logs,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [rx + half_right + 8.0, btn_y, half_right, 40.0],
                "Xbox / MS Login",
                Action::Account,
                BtnStyle::Primary,
                true,
            );
            draw_account_chip(
                ui,
                self.targets,
                [rx + col_w - acc_chip_w, btn_y + 3.0, acc_chip_w, 34.0],
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
                        btn_y + 2.0 + li as f32 * 18.0,
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
            // Compact single-column Settings with "Import Control" section at top
            let footer_h = 52.0;
            let top = cy;
            let bottom = cy + ch - footer_h;
            let all_items: Vec<_> = left_items.into_iter().chain(right_items).collect();
            let import_card_h = 154.0;
            let import_block_h = import_card_h + 12.0;
            let row_h = 68.0;
            let total_h = import_block_h + all_items.len() as f32 * row_h;
            let max_scroll = (total_h - (bottom - top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([top, bottom, max_scroll]);

            let iy = top - self.catalog.scroll;
            let half_btn = (cw - 32.0 - 8.0) * 0.5;
            let imp_btn_r = [cx + 16.0, iy + 68.0, half_btn, 36.0];
            let sav_btn_r = [cx + 16.0 + half_btn + 8.0, iy + 68.0, half_btn, 36.0];
            let all_btn_r = [cx + 16.0, iy + 110.0, half_btn, 32.0];
            let rst_btn_r = [cx + 16.0 + half_btn + 8.0, iy + 110.0, half_btn, 32.0];
            self.list_targets.push((imp_btn_r, Action::ImportControl));
            self.list_targets.push((sav_btn_r, Action::ControlLayouts));
            self.list_targets
                .push((all_btn_r, Action::ApplyControlsAllProfiles));
            self.list_targets.push((rst_btn_r, Action::ResetControls));

            if iy >= top - 4.0 && iy + import_card_h <= bottom + 4.0 {
                ui.rounded(cx, iy, cw, import_card_h, 12.0, theme::HERO_BG);
                ui.border(cx, iy, cw, import_card_h, 12.0, theme::BORDER_CRIMSON);
                ui.text(
                    Face::Medium,
                    16.5,
                    cx + 16.0,
                    iy + 10.0,
                    theme::TEXT,
                    "Import Control",
                );
                let act_fit =
                    ui.fit_text(Face::Regular, 12.5, &active_ctrl_summary, cw - 32.0);
                ui.text(
                    Face::Regular,
                    12.5,
                    cx + 16.0,
                    iy + 38.0,
                    if self.catalog.active_controls.is_some() {
                        theme::CRIMSON_BRIGHT
                    } else {
                        theme::MUTED
                    },
                    &act_fit,
                );
                styled_button(
                    ui,
                    self.targets,
                    imp_btn_r,
                    "Import Control",
                    Action::ImportControl,
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    sav_btn_r,
                    &saved_label,
                    Action::ControlLayouts,
                    BtnStyle::CrimsonOutline,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    all_btn_r,
                    "Apply All",
                    Action::ApplyControlsAllProfiles,
                    BtnStyle::Secondary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    rst_btn_r,
                    "Reset Default",
                    Action::ResetControls,
                    BtnStyle::Secondary,
                    true,
                );
            }

            for (i, (label, val, act, sw, dropdown)) in all_items.iter().enumerate() {
                let y = top + import_block_h + i as f32 * row_h - self.catalog.scroll;
                let ctrl_rect = [cx, y + 24.0, cw, 38.0];
                self.list_targets.push((ctrl_rect, *act));
                if y < top || y + row_h - 4.0 > bottom {
                    continue;
                }
                ui.text(Face::Regular, 13.5, cx + 2.0, y, theme::MUTED, label);
                if let Some(on) = sw {
                    toggle(ui, self.targets, ctrl_rect, *on, *act);
                } else {
                    let text = if *dropdown {
                        format!("{val}   v")
                    } else {
                        val.clone()
                    };
                    styled_button(
                        ui,
                        self.targets,
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
                self.targets,
                [cx, cy + ch - 44.0, half, 40.0],
                "Diagnostics",
                Action::Logs,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [cx + half + 10.0, cy + ch - 44.0, half, 40.0],
                "Microsoft account",
                Action::Account,
                BtnStyle::Primary,
                true,
            );
        }
    }

    fn draw_input_field(
        &mut self,
        ui: &mut Ui,
        rect: [f32; 4],
        field_name: &'static str,
        value: &str,
        placeholder: &str,
        masked: bool,
    ) {
        let [x, y, w, h] = rect;
        let focused = self.catalog.editing == Some(field_name);
        ui.rounded(x, y, w, h, 8.0, theme::CARD_INNER);
        ui.border(
            x,
            y,
            w,
            h,
            8.0,
            if focused {
                theme::CRIMSON_BRIGHT
            } else {
                theme::BORDER
            },
        );
        let display = if value.is_empty() {
            placeholder.to_owned()
        } else if masked {
            "*".repeat(value.chars().count())
        } else {
            value.to_owned()
        };
        let col = if value.is_empty() {
            theme::DIM
        } else {
            theme::TEXT
        };
        let fitted = ui.fit_text(Face::Medium, 13.5, &display, (w - 24.0).max(16.0));
        ui.text(
            Face::Medium,
            13.5,
            x + 12.0,
            text_vcenter(y, h, 13.5),
            col,
            &fitted,
        );
        self.targets.push((rect, Action::Field(field_name)));
    }

    fn draw_account_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        account_status: &str,
    ) {
        use crate::volt_accounts::{AccountProvider, PresetCape};
        let [cx, cy, cw, ch] = area;

        // Top Page Title + Status Bar
        let header_h = if compact { 32.0 } else { 36.0 };
        ui.text(
            Face::Medium,
            if compact { 18.0 } else { 22.0 },
            cx + 4.0,
            cy + 2.0,
            theme::TEXT,
            "Account & Custom Skin Wardrobe",
        );
        if !compact {
            let chip_w = (cw * 0.34).clamp(180.0, 290.0);
            draw_account_chip(
                ui,
                self.targets,
                [cx + cw - chip_w, cy, chip_w, 34.0],
                account_status,
                self.volt_accounts
                    .active()
                    .is_some_and(|a| a.provider != AccountProvider::Offline),
            );
        }

        let body_y = cy + header_h + 8.0;
        let body_h = (ch - header_h - if compact { 8.0 } else { 28.0 }).max(180.0);

        let active_acc = self.volt_accounts.active().cloned();
        let (acc_name, acc_badge, acc_slim, acc_sync, acc_signed, skin_lbl, cape_lbl, default_uuid) =
            match &active_acc {
                Some(a) => (
                    a.name.as_str(),
                    a.provider.badge(),
                    a.slim,
                    a.sync_to_servers,
                    a.signed_skin_url.is_some() || a.skin_label.starts_with("Mojang:"),
                    a.skin_label.as_str(),
                    a.cape_label.as_str(),
                    a.uuid_u128(),
                ),
                None => (
                    "Player", "OFFLINE", false, true, false, "Default", "None", 0u128,
                ),
            };

        if compact {
            // Stacked scrollable mobile portrait layout
            let total_h = 260.0 + 320.0 + 310.0 + 24.0;
            let max_scroll = (total_h - body_h).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([body_y, body_y + body_h, max_scroll]);
            let s = self.catalog.scroll;

            let p_rect = [cx, body_y - s, cw, 250.0];
            if p_rect[1] + p_rect[3] > body_y && p_rect[1] < body_y + body_h {
                self.draw_wardrobe_card(
                    ui,
                    p_rect,
                    acc_name,
                    acc_badge,
                    acc_slim,
                    skin_lbl,
                    cape_lbl,
                    default_uuid,
                );
            }
            let a_rect = [cx, body_y + 262.0 - s, cw, 310.0];
            if a_rect[1] + a_rect[3] > body_y && a_rect[1] < body_y + body_h {
                self.draw_accounts_auth_card(ui, a_rect);
            }
            let c_rect = [cx, body_y + 584.0 - s, cw, 300.0];
            if c_rect[1] + c_rect[3] > body_y && c_rect[1] < body_y + body_h {
                self.draw_cosmetics_loader_card(ui, c_rect, acc_sync, acc_signed);
            }
            return;
        }

        // Wide 3-column bento layout (Left: 3D Character Wardrobe | Center: Accounts & Auth | Right: Skin & Cape Loader)
        let gap = 12.0;
        let left_w = (cw * 0.29).clamp(200.0, 290.0).round();
        let rem_w = (cw - left_w - gap * 2.0).max(320.0);
        let mid_w = (rem_w * 0.48).round();
        let right_w = rem_w - mid_w;

        self.draw_wardrobe_card(
            ui,
            [cx, body_y, left_w, body_h],
            acc_name,
            acc_badge,
            acc_slim,
            skin_lbl,
            cape_lbl,
            default_uuid,
        );
        self.draw_accounts_auth_card(ui, [cx + left_w + gap, body_y, mid_w, body_h]);
        self.draw_cosmetics_loader_card(
            ui,
            [cx + left_w + gap + mid_w + gap, body_y, right_w, body_h],
            acc_sync,
            acc_signed,
        );

        let status = self
            .volt_accounts
            .busy
            .as_deref()
            .or(self.error.as_deref())
            .or_else(|| {
                (!self.catalog.status.is_empty()).then_some(self.catalog.status.as_str())
            })
            .or(self.volt_accounts.status.as_deref());
        if let Some(status) = status {
            let fitted = ui.fit_text(Face::Medium, 12.5, status, cw);
            ui.text(
                Face::Medium,
                12.5,
                cx + 4.0,
                cy + ch - 18.0,
                if self.error.is_some() {
                    theme::CRIMSON_BRIGHT
                } else {
                    theme::MUTED
                },
                &fitted,
            );
        }
        let _ = PresetCape::VoltCrimson;
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_wardrobe_card(
        &mut self,
        ui: &mut Ui,
        rect: [f32; 4],
        acc_name: &str,
        acc_badge: &str,
        acc_slim: bool,
        skin_lbl: &str,
        cape_lbl: &str,
        default_uuid: u128,
    ) {
        let [x, y, w, h] = rect;
        ui.rounded(x, y, w, h, 14.0, theme::CARD);
        ui.rounded(
            x + w * 0.14,
            y + h * 0.14,
            w * 0.72,
            h * 0.58,
            16.0,
            theme::HERO_GLOW_SOFT,
        );
        ui.border(x, y, w, h, 14.0, theme::BORDER_CRIMSON);

        let badge_w = (ui.measure(Face::Medium, 11.0, acc_badge) + 18.0).min(96.0);
        ui.rounded(x + 14.0, y + 12.0, badge_w, 22.0, 6.0, theme::CRIMSON_BADGE);
        ui.border(x + 14.0, y + 12.0, badge_w, 22.0, 6.0, theme::BORDER_CRIMSON);
        draw_centered_text(
            ui,
            Face::Medium,
            11.0,
            [x + 14.0, y + 12.0, badge_w, 22.0],
            theme::CRIMSON_BRIGHT,
            acc_badge,
        );

        let nfit = ui.fit_text(
            Face::Medium,
            14.5,
            acc_name,
            (w - badge_w - 94.0).max(36.0),
        );
        ui.text(
            Face::Medium,
            14.5,
            x + 20.0 + badge_w,
            text_vcenter(y + 12.0, 22.0, 14.5),
            theme::TEXT,
            &nfit,
        );

        styled_button(
            ui,
            self.targets,
            [x + w - 68.0, y + 10.0, 56.0, 26.0],
            "Flip",
            Action::ResetPreviewAngle,
            BtnStyle::Secondary,
            true,
        );

        let footer_h = 68.0;
        let model_bounds = [
            x + 10.0,
            y + 40.0,
            (w - 20.0).max(40.0),
            (h - 46.0 - footer_h).max(60.0),
        ];
        crate::skin_preview::draw_player_3d(
            ui,
            model_bounds,
            self.preview_yaw,
            self.preview_pitch,
            self.idle_phase,
            2,
            3,
            self.volt_accounts.active_skin_preview(),
            self.volt_accounts.active_cape_preview(),
            default_uuid,
        );
        *self.preview_rect = Some(model_bounds);

        // Classic vs Slim arm toggle + equipped summary at bottom of 3D preview card
        let arm_label = if acc_slim {
            "Model: Slim (3px)"
        } else {
            "Model: Classic (4px)"
        };
        styled_button(
            ui,
            self.targets,
            [x + 14.0, y + h - 62.0, w - 28.0, 30.0],
            arm_label,
            Action::ToggleSlimModel,
            BtnStyle::CrimsonOutline,
            true,
        );
        let summary = ui.fit_text(
            Face::Regular,
            11.5,
            &format!("Skin: {skin_lbl}  •  Cape: {cape_lbl}"),
            w - 24.0,
        );
        draw_centered_text(
            ui,
            Face::Regular,
            11.5,
            [x + 12.0, y + h - 26.0, w - 24.0, 20.0],
            theme::MUTED,
            &summary,
        );
    }

    fn draw_accounts_auth_card(&mut self, ui: &mut Ui, rect: [f32; 4]) {
        use crate::volt_accounts::AccountProvider;
        let [x, y, w, h] = rect;
        ui.rounded(x, y, w, h, 14.0, theme::CARD);
        ui.border(x, y, w, h, 14.0, theme::BORDER_CRIMSON);

        ui.text(
            Face::Medium,
            15.5,
            x + 16.0,
            y + 12.0,
            theme::TEXT,
            "Accounts",
        );

        // Saved accounts list (top section)
        let max_list_rows = ((h - 216.0) / 36.0).floor().clamp(2.0, 4.0) as usize;
        let list_start_y = y + 38.0;
        let active_id = self.volt_accounts.active_id.as_str();
        for (idx, acc) in self
            .volt_accounts
            .accounts
            .iter()
            .take(max_list_rows)
            .enumerate()
        {
            let ry = list_start_y + idx as f32 * 36.0;
            let selected = acc.id == active_id;
            let row_w = (w - 68.0).max(80.0);
            let row_rect = [x + 14.0, ry, row_w, 31.0];
            ui.rounded(
                row_rect[0],
                row_rect[1],
                row_rect[2],
                row_rect[3],
                8.0,
                if selected {
                    theme::CRIMSON_SOFT
                } else {
                    theme::CARD_INNER
                },
            );
            ui.border(
                row_rect[0],
                row_rect[1],
                row_rect[2],
                row_rect[3],
                8.0,
                if selected {
                    theme::CRIMSON
                } else {
                    theme::BORDER
                },
            );
            let tag = format!("[{}] {}", acc.provider.badge(), acc.name);
            let tfit = ui.fit_text(Face::Medium, 12.5, &tag, row_w - 20.0);
            ui.text(
                Face::Medium,
                12.5,
                row_rect[0] + 10.0,
                text_vcenter(row_rect[1], row_rect[3], 12.5),
                if selected {
                    theme::ON_CRIMSON
                } else {
                    theme::TEXT
                },
                &tfit,
            );
            self.targets.push((row_rect, Action::SelectVoltAccount(idx)));

            styled_button(
                ui,
                self.targets,
                [x + w - 48.0, ry, 34.0, 31.0],
                "Del",
                Action::RemoveVoltAccount(idx),
                BtnStyle::Subtle,
                true,
            );
        }

        // Provider Tabs: Offline | Ely.by | Microsoft
        let form_top = list_start_y + (max_list_rows.min(self.volt_accounts.accounts.len().max(1)) as f32) * 36.0 + 8.0;
        ui.rounded(x + 14.0, form_top, w - 28.0, 30.0, 8.0, theme::RAIL);
        ui.border(x + 14.0, form_top, w - 28.0, 30.0, 8.0, theme::BORDER);
        let tabs = [
            (AccountProvider::Offline, "Offline"),
            (AccountProvider::ElyBy, "Ely.by"),
            (AccountProvider::Microsoft, "Microsoft"),
        ];
        let tab_w = (w - 32.0) / 3.0;
        for (ti, (prov, label)) in tabs.into_iter().enumerate() {
            let tx = x + 16.0 + ti as f32 * tab_w;
            let trect = [tx, form_top + 2.0, tab_w - 2.0, 26.0];
            let active = self.catalog.account_tab == prov;
            if active {
                ui.rounded(trect[0], trect[1], trect[2], trect[3], 6.0, theme::CRIMSON);
            }
            draw_centered_text(
                ui,
                Face::Medium,
                12.0,
                trect,
                if active { theme::ON_CRIMSON } else { theme::MUTED },
                label,
            );
            self.targets.push((trect, Action::AccountProviderTab(prov)));
        }

        let fy = form_top + 38.0;
        match self.catalog.account_tab {
            AccountProvider::Offline => {
                let val = self.catalog.offline_name_input.clone();
                self.draw_input_field(
                    ui,
                    [x + 14.0, fy, w - 28.0, 34.0],
                    "offline_name",
                    &val,
                    "Offline username (1-16 chars)",
                    false,
                );
                let bw = (w - 36.0) * 0.5;
                styled_button(
                    ui,
                    self.targets,
                    [x + 14.0, fy + 42.0, bw, 34.0],
                    "Create / Switch",
                    Action::CreateOfflineAccount,
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    [x + 22.0 + bw, fy + 42.0, bw, 34.0],
                    "Rename Active",
                    Action::UpdateOfflineName,
                    BtnStyle::Secondary,
                    true,
                );
                let note = ui.fit_text(
                    Face::Regular,
                    11.5,
                    "Supports full offline skin & cape equipping + server sync.",
                    w - 28.0,
                );
                ui.text(Face::Regular, 11.5, x + 14.0, fy + 84.0, theme::MUTED, &note);
            }
            AccountProvider::ElyBy => {
                let u_val = self.catalog.ely_user_input.clone();
                let p_val = self.catalog.ely_pass_input.clone();
                let t_val = self.catalog.ely_totp_input.clone();
                self.draw_input_field(
                    ui,
                    [x + 14.0, fy, w - 28.0, 32.0],
                    "ely_user",
                    &u_val,
                    "Ely.by email or username",
                    false,
                );
                let pw_w = (w - 36.0) * 0.68;
                let totp_w = (w - 36.0) - pw_w;
                self.draw_input_field(
                    ui,
                    [x + 14.0, fy + 38.0, pw_w, 32.0],
                    "ely_pass",
                    &p_val,
                    "Password",
                    true,
                );
                self.draw_input_field(
                    ui,
                    [x + 22.0 + pw_w, fy + 38.0, totp_w, 32.0],
                    "ely_totp",
                    &t_val,
                    "2FA",
                    false,
                );
                let sign_w = (w - 36.0) * 0.62;
                let reg_w = (w - 36.0) - sign_w;
                styled_button(
                    ui,
                    self.targets,
                    [x + 14.0, fy + 78.0, sign_w, 34.0],
                    if self.volt_accounts.busy.is_some() {
                        "Signing in..."
                    } else {
                        "Sign In (Ely.by)"
                    },
                    Action::ElySignIn,
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    [x + 22.0 + sign_w, fy + 78.0, reg_w, 34.0],
                    "Register",
                    Action::ElyRegister,
                    BtnStyle::CrimsonOutline,
                    true,
                );
            }
            AccountProvider::Microsoft => {
                let lines = wrap_lines(
                    ui,
                    Face::Regular,
                    12.0,
                    "Official Microsoft / Xbox Live login for Hypixel and online-mode Java servers using device code.",
                    w - 28.0,
                    3,
                );
                for (li, line) in lines.iter().enumerate() {
                    ui.text(
                        Face::Regular,
                        12.0,
                        x + 14.0,
                        fy + li as f32 * 17.0,
                        theme::MUTED,
                        line,
                    );
                }
                let btn_y = fy + 56.0;
                styled_button(
                    ui,
                    self.targets,
                    [x + 14.0, btn_y, w - 28.0, 36.0],
                    if self.login.signed_in {
                        "Manage / Re-authenticate Microsoft"
                    } else {
                        "Sign In with Microsoft"
                    },
                    Action::Account,
                    BtnStyle::Primary,
                    true,
                );
                if self.login.signed_in {
                    styled_button(
                        ui,
                        self.targets,
                        [x + 14.0, btn_y + 42.0, w - 28.0, 30.0],
                        "Sign Out Microsoft",
                        Action::SignOut,
                        BtnStyle::Subtle,
                        true,
                    );
                }
            }
        }
    }

    fn draw_cosmetics_loader_card(
        &mut self,
        ui: &mut Ui,
        rect: [f32; 4],
        acc_sync: bool,
        acc_signed: bool,
    ) {
        use crate::volt_accounts::PresetCape;
        let [x, y, w, h] = rect;
        ui.rounded(x, y, w, h, 14.0, theme::CARD);
        ui.border(x, y, w, h, 14.0, theme::BORDER_CRIMSON);

        ui.text(
            Face::Medium,
            15.5,
            x + 16.0,
            y + 12.0,
            theme::TEXT,
            "Custom Skin & Cape Loader",
        );

        // Server-wide visibility toggle row
        let sync_sub = if !acc_sync {
            "Local + LAN only"
        } else if acc_signed {
            "Server Sync Ready (Signed + Ely.by + LAN)"
        } else {
            "Auto-syncs on SkinsRestorer, Ely.by & LAN"
        };
        let sync_fit = ui.fit_text(Face::Regular, 12.0, sync_sub, (w - 86.0).max(60.0));
        ui.text(Face::Regular, 12.0, x + 16.0, y + 38.0, theme::MUTED, &sync_fit);
        toggle(
            ui,
            self.targets,
            [x + w - 62.0, y + 33.0, 48.0, 26.0],
            acc_sync,
            Action::ToggleServerSkinSync,
        );

        // Shared Username / URL input box
        let q_val = self.catalog.skin_query_input.clone();
        self.draw_input_field(
            ui,
            [x + 14.0, y + 66.0, w - 28.0, 34.0],
            "skin_query",
            &q_val,
            "Player name (Mojang/Ely.by) or PNG URL",
            false,
        );

        // Skin Loader Buttons
        ui.text(
            Face::Medium,
            12.5,
            x + 16.0,
            y + 106.0,
            theme::CRIMSON_BRIGHT,
            "SKIN",
        );
        let half = (w - 36.0) * 0.5;
        styled_button(
            ui,
            self.targets,
            [x + 14.0, y + 124.0, half, 32.0],
            "Import PNG",
            Action::ImportSkinPng,
            BtnStyle::Primary,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 22.0 + half, y + 124.0, half, 32.0],
            "Fetch Username",
            Action::FetchSkinByUsername,
            BtnStyle::CrimsonOutline,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 14.0, y + 162.0, half, 30.0],
            "Load Skin URL",
            Action::LoadSkinUrl,
            BtnStyle::Secondary,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 22.0 + half, y + 162.0, half, 30.0],
            "Reset Skin",
            Action::ClearCustomSkin,
            BtnStyle::Subtle,
            true,
        );

        // Cape Loader + One-Tap Volt Preset Capes
        ui.text(
            Face::Medium,
            12.5,
            x + 16.0,
            y + 198.0,
            theme::CRIMSON_BRIGHT,
            "CAPE",
        );
        let third = (w - 44.0) / 3.0;
        styled_button(
            ui,
            self.targets,
            [x + 14.0, y + 216.0, third, 30.0],
            "Import PNG",
            Action::ImportCapePng,
            BtnStyle::CrimsonOutline,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 22.0 + third, y + 216.0, third, 30.0],
            "Cape URL",
            Action::LoadCapeUrl,
            BtnStyle::Secondary,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 30.0 + third * 2.0, y + 216.0, third, 30.0],
            "Remove",
            Action::ClearCustomCape,
            BtnStyle::Subtle,
            true,
        );

        if y + 282.0 <= y + h {
            styled_button(
                ui,
                self.targets,
                [x + 14.0, y + 252.0, third, 30.0],
                "Volt V",
                Action::EquipPresetCape(PresetCape::VoltCrimson),
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [x + 22.0 + third, y + 252.0, third, 30.0],
                "Crown",
                Action::EquipPresetCape(PresetCape::RoyalGold),
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [x + 30.0 + third * 2.0, y + 252.0, third, 30.0],
                "Stealth",
                Action::EquipPresetCape(PresetCape::ObsidianStealth),
                BtnStyle::Secondary,
                true,
            );
        }
    }

}

impl Launcher {
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
        self.preview_rect = None;
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
        let account_status = self
            .volt_accounts
            .active()
            .map(|a| format!("{} • {}", a.name, a.provider.badge()))
            .unwrap_or_else(|| self.login.status(&self.catalog.config.name));
        let idle_phase = self.launch_start.elapsed().as_secs_f32();
        {
            let mut ctx = DrawCtx {
                home: &self.home,
                install: &self.install,
                worker: &self.worker,
                error: &self.error,
                catalog: &mut self.catalog,
                targets: &mut self.targets,
                list_targets: &mut self.list_targets,
                list_view: &mut self.list_view,
                login: &self.login,
                volt_accounts: &self.volt_accounts,
                preview_yaw: self.preview_yaw,
                preview_pitch: self.preview_pitch,
                idle_phase,
                preview_rect: &mut self.preview_rect,
            };
            let content_area = if compact {
                ctx.draw_compact_header(&mut ui, w, &account_status);
                [12.0, 102.0, (w - 24.0).max(120.0), (h - 148.0).max(120.0)]
            } else {
                let rail_w = 188.0;
                ctx.draw_sidebar(&mut ui, 12.0, 12.0, rail_w, (h - 24.0).max(260.0));
                [
                    12.0 + rail_w + 16.0,
                    14.0,
                    (w - rail_w - 40.0).max(240.0),
                    (h - 28.0).max(200.0),
                ]
            };

            let page = ctx.catalog.page;
            match page {
                Page::Play => ctx.draw_play_page(&mut ui, content_area, compact, &account_status),
                Page::Account => {
                    ctx.draw_account_page(&mut ui, content_area, compact, &account_status)
                }
                Page::Mods | Page::Packs | Page::Shaders => {
                    ctx.draw_catalog_page(&mut ui, content_area, compact, page, &account_status)
                }
                Page::Installed => {
                    ctx.draw_installed_page(&mut ui, content_area, compact, &account_status)
                }
                Page::Settings => {
                    ctx.draw_settings_page(&mut ui, content_area, compact, &account_status)
                }
            }
        }

        if compact {
            let status = self.error.as_deref().unwrap_or(&self.catalog.status);
            if !status.is_empty() {
                let sfit = ui.fit_text(Face::Regular, 12.0, status, (w - 196.0).max(40.0));
                ui.text(Face::Regular, 12.0, 14.0, h - 32.0, theme::MUTED, &sfit);
            }
            styled_button(
                &mut ui,
                &mut self.targets,
                [w - 178.0, h - 38.0, 78.0, 28.0],
                "Discord",
                Action::Discord,
                BtnStyle::CrimsonOutline,
                true,
            );
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
                "offline_name" => (self.catalog.offline_name_input.clone(), 13.5, 12.0),
                "ely_user" => (self.catalog.ely_user_input.clone(), 13.5, 12.0),
                "ely_pass" => (
                    "*".repeat(self.catalog.ely_pass_input.chars().count()),
                    13.5,
                    12.0,
                ),
                "ely_totp" => (self.catalog.ely_totp_input.clone(), 13.5, 12.0),
                "skin_query" => (self.catalog.skin_query_input.clone(), 13.5, 12.0),
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
                    "Open the system Files app and choose Volt Launcher.".into()
                };
            }
            Some(Action::Import(kind)) => {
                crate::platform::hide();
                let home = self.catalog.config.game_home(&self.home);
                if let Err(e) = crate::documents::begin(kind, home) {
                    self.catalog.status = e;
                }
            }
            Some(Action::ImportControl) => {
                crate::platform::hide();
                self.chooser = None;
                let profile_home = self.catalog.config.game_home(&self.home);
                if let Err(e) = crate::documents::begin_controls(self.home.clone(), profile_home) {
                    self.catalog.status = e;
                }
            }
            Some(Action::ControlLayouts) => {
                self.catalog.refresh_controls(&self.home);
                self.open_chooser(chooser::Kind::ControlLayout);
            }
            Some(Action::ChooseControlLayout(idx)) => {
                self.chooser = None;
                if let Some(preset) = self.catalog.control_presets.get(idx).cloned() {
                    let profile_home = self.catalog.config.game_home(&self.home);
                    match horizon_client::touch::import::load_preset(&self.home, &preset.slug)
                        .and_then(|s| {
                            horizon_client::touch::import::apply_to_profile(&profile_home, &s)
                        }) {
                        Ok(()) => {
                            if matches!(
                                self.catalog.config.input_host,
                                crate::catalog::InputHost::KeyboardMouse
                                    | crate::catalog::InputHost::Controller
                            ) {
                                self.catalog.config.input_host = crate::catalog::InputHost::Auto;
                                let _ = self.catalog.config.save(&self.home);
                                crate::configure_input_mode(self.catalog.config.input_host);
                            }
                            self.catalog.refresh_controls(&self.home);
                            self.catalog.status = format!(
                                "Activated \"{}\" ({}) for Profile {}.",
                                preset.meta.name,
                                preset.meta.format,
                                self.catalog.config.profile
                            );
                        }
                        Err(e) => self.catalog.status = format!("Could not apply preset: {e}"),
                    }
                }
            }
            Some(Action::ApplyControlsAllProfiles) => {
                let profile_home = self.catalog.config.game_home(&self.home);
                let current = horizon_client::touch::Settings::load_from(
                    &profile_home.join("horizon-touch.json"),
                );
                match horizon_client::touch::import::apply_to_all_profiles(&self.home, &current) {
                    Ok(count) => {
                        self.catalog.refresh_controls(&self.home);
                        self.catalog.status = format!(
                            "Applied active touch control layout across {count} profile(s)."
                        );
                    }
                    Err(e) => self.catalog.status = format!("Apply failed: {e}"),
                }
            }
            Some(Action::ResetControls) => {
                self.chooser = None;
                let profile_home = self.catalog.config.game_home(&self.home);
                match horizon_client::touch::import::reset_profile(&profile_home) {
                    Ok(()) => {
                        self.catalog.refresh_controls(&self.home);
                        self.catalog.status = format!(
                            "Restored default Volt touch controls for Profile {}.",
                            self.catalog.config.profile
                        );
                    }
                    Err(e) => self.catalog.status = format!("Reset failed: {e}"),
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
                    "Volt Launcher GitHub opened in browser."
                } else {
                    "No browser available to open GitHub."
                }
                .into();
            }
            Some(Action::Discord) => {
                self.catalog.status = if crate::platform::open_discord() {
                    "Volt Launcher Discord opened in browser."
                } else {
                    "No browser available to open Discord."
                }
                .into();
            }
            Some(Action::AccountProviderTab(prov)) => {
                crate::platform::hide();
                self.catalog.editing = None;
                self.catalog.account_tab = prov;
            }
            Some(Action::SelectVoltAccount(idx)) => {
                if let Some(acc) = self.volt_accounts.accounts.get(idx).cloned() {
                    self.volt_accounts.select(&acc.id);
                    self.catalog.config.name = acc.name.clone();
                    self.catalog.offline_name_input = acc.name;
                    self.catalog.account_tab = acc.provider;
                    self.catalog.config.save(&self.home)?;
                }
            }
            Some(Action::RemoveVoltAccount(idx)) => {
                if let Some(id) = self
                    .volt_accounts
                    .accounts
                    .get(idx)
                    .map(|a| a.id.clone())
                {
                    if self.volt_accounts.remove_account(&id).is_some() {
                        self.login.sign_out();
                    }
                    if let Some(active) = self.volt_accounts.active() {
                        self.catalog.config.name = active.name.clone();
                        self.catalog.offline_name_input = active.name.clone();
                        self.catalog.config.save(&self.home)?;
                    }
                }
            }
            Some(Action::CreateOfflineAccount) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let input = self.catalog.offline_name_input.clone();
                match self.volt_accounts.create_offline(&input) {
                    Ok(()) => {
                        if let Some(acc) = self.volt_accounts.active() {
                            self.catalog.config.name = acc.name.clone();
                            self.catalog.config.save(&self.home)?;
                        }
                        self.catalog.status = "Offline account active.".into();
                    }
                    Err(e) => self.catalog.status = e,
                }
            }
            Some(Action::UpdateOfflineName) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let input = self.catalog.offline_name_input.clone();
                match self.volt_accounts.rename_active_offline(&input) {
                    Ok(()) => {
                        if let Some(acc) = self.volt_accounts.active() {
                            self.catalog.config.name = acc.name.clone();
                            self.catalog.config.save(&self.home)?;
                        }
                        self.catalog.status = "Offline username updated.".into();
                    }
                    Err(e) => self.catalog.status = e,
                }
            }
            Some(Action::ElySignIn) => {
                crate::platform::hide();
                self.catalog.editing = None;
                self.volt_accounts.begin_ely_login(
                    &self.catalog.ely_user_input,
                    &self.catalog.ely_pass_input,
                    &self.catalog.ely_totp_input,
                );
                self.catalog.ely_pass_input.clear();
                self.catalog.ely_totp_input.clear();
            }
            Some(Action::ElyRegister) => {
                let _ = crate::platform::open_ely_register();
            }
            Some(Action::ToggleSlimModel) => {
                self.volt_accounts.toggle_slim();
            }
            Some(Action::ToggleServerSkinSync) => {
                self.volt_accounts.toggle_server_sync();
            }
            Some(Action::ImportSkinPng) => {
                if let Some(id) = self.volt_accounts.active().map(|a| a.id.clone()) {
                    let dest = self.volt_accounts.skin_png_path(&id);
                    if let Err(e) = crate::documents::begin_cosmetic(false, dest) {
                        self.catalog.status = e;
                    }
                }
            }
            Some(Action::ImportCapePng) => {
                if let Some(id) = self.volt_accounts.active().map(|a| a.id.clone()) {
                    let dest = self.volt_accounts.cape_png_path(&id);
                    if let Err(e) = crate::documents::begin_cosmetic(true, dest) {
                        self.catalog.status = e;
                    }
                }
            }
            Some(Action::FetchSkinByUsername) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let q = self.catalog.skin_query_input.clone();
                self.volt_accounts.begin_fetch_skin_by_username(&q);
            }
            Some(Action::LoadSkinUrl) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let q = self.catalog.skin_query_input.clone();
                self.volt_accounts.begin_fetch_cosmetic_url(&q, false);
            }
            Some(Action::LoadCapeUrl) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let q = self.catalog.skin_query_input.clone();
                self.volt_accounts.begin_fetch_cosmetic_url(&q, true);
            }
            Some(Action::EquipPresetCape(preset)) => {
                if let Err(e) = self.volt_accounts.apply_preset_cape(preset) {
                    self.catalog.status = e;
                } else {
                    // Rotate character toward back so user immediately sees their newly equipped 3D cape!
                    self.preview_yaw = 162.0;
                    self.catalog.status = format!("Equipped {}!", preset.label());
                }
            }
            Some(Action::ClearCustomSkin) => {
                self.volt_accounts.clear_skin();
            }
            Some(Action::ClearCustomCape) => {
                self.volt_accounts.clear_cape();
            }
            Some(Action::ResetPreviewAngle) => {
                let normalized = (self.preview_yaw + 180.0).rem_euclid(360.0) - 180.0;
                self.preview_yaw = if normalized.abs() < 90.0 { 162.0 } else { -18.0 };
                self.preview_pitch = 6.0;
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
            Some(Action::SignOut) => {
                self.login.sign_out();
                self.volt_accounts
                    .sync_msa_accounts(&crate::account::Accounts::load(), false);
            }
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
                    "offline_name" => &self.catalog.offline_name_input,
                    "ely_user" => &self.catalog.ely_user_input,
                    "ely_pass" => &self.catalog.ely_pass_input,
                    "ely_totp" => &self.catalog.ely_totp_input,
                    "skin_query" => &self.catalog.skin_query_input,
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
                    let (dx, dy) = drag.update_xy(t.location.x, t.location.y, density);
                    if drag.moved {
                        let [ox, oy] = drag.origin_xy();
                        let ux = (ox / density) as f32;
                        let uy = (oy / density) as f32;
                        let in_preview = !self.login.visible
                            && self.chooser.is_none()
                            && self.preview_rect.is_some_and(|[rx, ry, rw, rh]| {
                                ux >= rx && ux <= rx + rw && uy >= ry && uy <= ry + rh
                            });
                        if in_preview {
                            self.preview_yaw =
                                (self.preview_yaw + dx * 0.9).rem_euclid(360.0);
                            self.preview_pitch =
                                (self.preview_pitch - dy * 0.45).clamp(-28.0, 28.0);
                        } else if let Some(chooser) = &mut self.chooser {
                            chooser.scroll = (chooser.scroll + dy).max(0.0);
                        } else if self
                            .list_view
                            .is_some_and(|view| oy / density >= view[0] as f64)
                        {
                            self.catalog.scroll = (self.catalog.scroll + dy).max(0.0);
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
