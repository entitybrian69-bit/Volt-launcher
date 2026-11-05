use super::*;
use horizon_client::osk::Output;
use winit::{
    event::{KeyEvent, Touch},
    keyboard::{KeyCode, PhysicalKey},
};

/// Launcher colors based on Horizon Nightly's glass/night tokens (f319701e).
/// The scene is approximated in this mobile renderer; no backdrop blur is claimed.
pub(crate) mod theme {
    #![allow(non_snake_case)]
    use horizon_client::ui_theme::color::{hex, lerp};
    use std::cell::Cell;
    use horizon_theme::glass::{Glass, ACCENT, BLUR, TINT};
    #[derive(Clone, Copy)]
    struct Look { dark: bool, glass: Glass }
    thread_local! { static LOOK: Cell<Option<Look>> = const { Cell::new(None) }; }
    pub fn set(dark: bool, accent: u32) {
        let accent = accent & 0xffffff;
        LOOK.with(|cell| {
            if cell.get().is_none_or(|look| look.dark != dark || look.glass.accent != accent) {
                cell.set(Some(Look { dark, glass: Glass::new(accent, TINT, BLUR) }));
            }
        });
    }
    fn look() -> Look { LOOK.with(|cell| cell.get().unwrap_or_else(|| Look {
        dark: true, glass: Glass::new(ACCENT, TINT, BLUR)
    })) }
    pub fn dark() -> bool { look().dark }
    fn accent() -> u32 { hex(look().glass.accent, 255) }
    fn choose(dark_value: u32, light_value: u32) -> u32 { if dark() { dark_value } else { light_value } }
    pub fn BG() -> u32 { choose(hex(0x0a0919,255), hex(0xe9e6f6,255)) }
    pub fn BACKDROP() -> u32 { hex(0x07050f, 215) }
    pub fn RAIL() -> u32 { choose(hex(0x15112e,235), hex(0xfdfcff,225)) }
    pub fn CARD() -> u32 { choose(hex(0x15112e,225), hex(0xfdfcff,230)) }
    pub fn CARD_INNER() -> u32 { choose(hex(0x29213d,235), hex(0xebe8f5,245)) }
    pub fn HOVER() -> u32 { choose(hex(0x403652,230), hex(0xded9ee,245)) }
    pub fn HERO_BG() -> u32 { CARD() }
    pub fn HERO_GLOW() -> u32 { lerp(CARD(), accent(), 0.22) }
    pub fn HERO_GLOW_SOFT() -> u32 { lerp(CARD(), accent(), 0.12) }
    pub fn BORDER() -> u32 { choose(hex(0xb9acff,42), hex(0x3b2f78,38)) }
    pub fn BORDER_CRIMSON() -> u32 { lerp(BORDER(), accent(), 0.32) }
    pub fn CRIMSON() -> u32 { accent() }
    pub fn CRIMSON_BRIGHT() -> u32 {
        let glass = look().glass;
        hex(if dark() { glass.accent_soft } else { glass.accent }, 255)
    }
    pub fn CRIMSON_SOFT() -> u32 { (accent() & 0x00ffffff) | (42u32 << 24) }
    pub fn CRIMSON_BADGE() -> u32 { choose(hex(0xffffff,26), hex(0x100a1f,26)) }
    pub fn ON_CRIMSON() -> u32 { hex(look().glass.on_accent,255) }
    pub fn TEXT() -> u32 { choose(hex(0xf4f2ff,255), hex(0x1b1530,255)) }
    pub fn MUTED() -> u32 { choose(hex(0xcfc9ea,255), hex(0x585071,255)) }
    pub fn DIM() -> u32 { choose(hex(0xa59fc6,255), hex(0x706782,255)) }
    pub fn ONLINE() -> u32 { choose(hex(0x4ade80,255), hex(0x15803d,255)) }
    pub fn GLASS() -> u32 {
        if !dark() { return hex(0xffffff, 214); }
        let glass = look().glass;
        hex(glass.plate, (glass.plate_alpha * 255.0).round() as u8)
    }
    pub fn GLASS_INNER() -> u32 { choose(hex(0xffffff,16), hex(0x100a1f,12)) }
    pub fn GLASS_BORDER() -> u32 { choose(hex(0xffffff,34), hex(0x3b2f78,30)) }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum HomeTab { Overview, Worlds, Multiplayer, Screenshots, Console }

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Action {
    HomeTab(HomeTab),
    ToggleAppearance,
    CycleAccent,
    ConnectServer(usize),
    SaveServer,
    Page(Page),
    Play,
    Search,
    ShaderSource(crate::shader_catalog::Origin),
    ModSource(crate::shader_catalog::Origin),
    PackSource(crate::shader_catalog::Origin),
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
    ImportVulkanDriver,
    ResetVulkanDriver,
    InstallUpdate,
    CancelUpdate,
    OpenVoltBoost,
    CloseVoltBoost,
    VoltExtra(horizon_client::extras::Item),
    VoltPreset(horizon_client::volt_features::VoltBoostPreset),
    ToggleVoltMod(horizon_client::volt_mods::VoltModId),
    InstallBundledZoom,
    HorizonModBrowser(bool),
    RemoveBundled(usize),
    ModFilter(u8),
    EnableModPack,
    ResetModDefaults,
    ClearModSearch,
    Studio(super::controls_studio::StudioAction),
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

thread_local! {
    static ACTIVE_FEEDBACK: std::cell::RefCell<Option<(Action, std::time::Instant)>> =
        const { std::cell::RefCell::new(None) };
    static TOGGLE_ANIMS: std::cell::RefCell<std::collections::HashMap<u32, (f32, std::time::Instant)>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}

fn toggle_key(action: Action) -> u32 {
    match action {
        Action::Immersive => 1,
        Action::Cutout => 2,
        Action::PerformanceHints => 3,
        Action::GpuProfile => 4,
        Action::RecordPerformance => 5,
        Action::ToggleServerSkinSync => 6,
        Action::ToggleSlimModel => 7,
        Action::ToggleLocal(i) => 1000 + i as u32,
        Action::ToggleSection(i) => 2000 + i as u32,
        Action::VoltExtra(item) => 3000 + item as u32,
        Action::ToggleVoltMod(id) => 5000 + id.index() as u32,
        _ => 9999,
    }
}

fn has_active_toggle_anims() -> bool {
    TOGGLE_ANIMS.with(|map| {
        map.borrow()
            .values()
            .any(|&(pos, ts)| (pos > 0.005 && pos < 0.995) && ts.elapsed().as_secs_f32() < 0.4)
    })
}

#[inline]
fn point_in(p: (f32, f32), r: [f32; 4]) -> bool {
    p.0 >= r[0] && p.0 <= r[0] + r[2] && p.1 >= r[1] && p.1 <= r[1] + r[3]
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
    let press_t = ACTIVE_FEEDBACK.with(|fb| {
        fb.borrow().and_then(|(act, ts)| {
            let el = ts.elapsed().as_secs_f32();
            (act == action && el < 0.22).then_some(el / 0.22)
        })
    });
    let v_start = ui.batch_mut().vertices.len();
    let radius = (h * 0.24).clamp(6.0, 11.0);
    let (mut bg, mut border_col, fg) = match style {
        BtnStyle::Primary => (theme::CRIMSON(), theme::CRIMSON_BRIGHT(), theme::ON_CRIMSON()),
        BtnStyle::CrimsonOutline => (theme::CARD_INNER(), theme::BORDER_CRIMSON(), theme::TEXT()),
        BtnStyle::Secondary => (theme::CARD_INNER(), theme::BORDER(), theme::TEXT()),
        BtnStyle::Subtle => (theme::CARD(), theme::BORDER(), theme::MUTED()),
    };
    if let Some(t) = press_t {
        let flash = (1.0 - t).powi(2);
        bg = horizon_client::ui_theme::color::lerp(bg, theme::CRIMSON_BRIGHT(), flash * 0.30);
        border_col =
            horizon_client::ui_theme::color::lerp(border_col, theme::CRIMSON_BRIGHT(), flash * 0.85);
    }
    ui.rounded(x, y, w, h, radius, bg);
    ui.border(x, y, w, h, radius, border_col);
    let base = if h >= 50.0 {
        19.5
    } else if w < 84.0 {
        13.5
    } else {
        15.5
    };
    let pad = (w * 0.1).clamp(8.0, 14.0);
    let font = if matches!(action, Action::Page(_)) {
        (base * ((w - 2.0 * pad) / ui.measure(Face::LauncherMedium, base, label).max(1.0)).min(1.0))
            .max(11.0)
    } else {
        base
    };
    if centered {
        draw_centered_text(ui, Face::LauncherMedium, font, rect, fg, label);
    } else {
        let fitted = ui.fit_text(Face::LauncherMedium, font, label, (w - 2.0 * pad).max(8.0));
        ui.text(Face::LauncherMedium, font, x + pad, text_vcenter(y, h, font), fg, &fitted);
    }
    if let Some(t) = press_t {
        let dip = (t * std::f32::consts::PI).sin() * (1.0 - t * 0.45);
        let scale = 1.0 - 0.042 * dip;
        ui.animate_vertices(v_start, 0.0, 0.0, scale, [x + w * 0.5, y + h * 0.5], 1.0);
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
    let target_pos = if on { 1.0f32 } else { 0.0f32 };
    let anim_pos = TOGGLE_ANIMS.with(|map| {
        let mut map = map.borrow_mut();
        let now = std::time::Instant::now();
        let key = toggle_key(action);
        let entry = map.entry(key).or_insert((target_pos, now));
        let dt = now.duration_since(entry.1).as_secs_f32().clamp(0.0, 0.1);
        let alpha = 1.0 - (-dt * 18.0).exp();
        entry.0 += (target_pos - entry.0) * alpha;
        if (target_pos - entry.0).abs() < 0.004 {
            entry.0 = target_pos;
        }
        entry.1 = now;
        entry.0
    });
    if w >= 72.0 {
        ui.text(
            Face::LauncherMedium,
            14.5,
            x,
            text_vcenter(y, h, 14.5),
            horizon_client::ui_theme::color::lerp(theme::MUTED(), theme::TEXT(), anim_pos),
            if on { "On" } else { "Off" },
        );
    }
    let sw = 48.0;
    let sh = 26.0;
    let sx = x + w - sw;
    let sy = y + (h - sh) * 0.5;
    let track_col = horizon_client::ui_theme::color::lerp(theme::HOVER(), theme::CRIMSON(), anim_pos);
    let border_col =
        horizon_client::ui_theme::color::lerp(theme::BORDER(), theme::CRIMSON_BRIGHT(), anim_pos);
    ui.rounded(sx, sy, sw, sh, 13.0, track_col);
    ui.border(sx, sy, sw, sh, 13.0, border_col);
    let knob = 20.0;
    let kx = sx + 3.0 + (sw - knob - 6.0) * anim_pos;
    ui.rounded(kx, sy + 3.0, knob, knob, 10.0, theme::ON_CRIMSON());
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
        Page::VoltSettings => {
            // Crisp Volt lightning-bolt / boost gauge icon
            ui.border(cx - 8.0, cy - 8.0, 16.0, 16.0, 5.0, col);
            ui.rounded(cx - 1.5, cy - 6.0, 4.5, 5.0, 1.0, col);
            ui.rounded(cx - 4.0, cy - 1.5, 8.0, 2.5, 1.0, col);
            ui.rounded(cx - 3.0, cy + 1.0, 4.5, 5.0, 1.0, col);
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

/// A low-cost mobile approximation of Nightly's pixel-sunset backdrop.
/// Its palette is taken from the pinned upstream scene; it is not the desktop renderer.
fn draw_launcher_scene(ui: &mut Ui, w: f32, h: f32) {
    use horizon_client::ui_theme::color::{hex, lerp};
    const NIGHT: [u32; 15] = [0x0a0919, 0x0f0c28, 0x151136, 0x1d1444, 0x281850,
        0x361b5b, 0x481e66, 0x5d236e, 0x752873, 0x902f76, 0xab3a75, 0xc64a71,
        0xde5f6a, 0xef7c63, 0xf89d62];
    const DAWN: [u32; 6] = [0xc9d3f2, 0xdcd8f3, 0xead5ec, 0xf5d3d8, 0xf9d6bd, 0xfbe4c6];
    let stops: &[u32] = if theme::dark() { &NIGHT } else { &DAWN };
    for row in 0..48 {
        let t = (row as f32 / 47.0).powf(2.0) * (stops.len() - 1) as f32;
        let index = (t as usize).min(stops.len() - 2);
        let color = lerp(hex(stops[index],255), hex(stops[index+1],255), t - index as f32);
        ui.rounded(0.0, row as f32 * h / 48.0, w, h / 48.0 + 1.0, 0.0, color);
    }
    if theme::dark() {
        for i in 0..52 {
            let x = w * (((i * 73 + 19) % 101) as f32 / 101.0);
            let y = h * (((i * 37 + 11) % 64) as f32 / 100.0);
            let size = if i % 8 == 0 { 1.8 } else { 1.0 };
            ui.rounded(x, y, size, size, 0.0, hex(0xd2c8ff, if i % 3 == 0 { 140 } else { 80 }));
        }
    }
    // A pixel sun and three layered ridges, as in the glass reference.
    let sun_x = w * 0.72;
    let sun_y = h * 0.67;
    let radius = h.min(w) * 0.17;
    for band in 0..16 {
        let yy = (band as f32 + 0.5) / 8.0 - 1.0;
        let half = (1.0 - yy * yy).max(0.0).sqrt() * radius;
        let color = lerp(hex(0xfff5d2,255), hex(0xfb6c62,255), (yy + 1.0) * 0.5);
        ui.rounded(sun_x - half, sun_y + yy * radius, half * 2.0, radius / 8.0 + 1.0, 0.0, color);
    }
    for layer in 0..3 {
        let colors = if theme::dark() { [0x5b2a67, 0x2b194e, 0x100d2b] } else { [0xc99abc, 0x9c8bb3, 0x77749e] };
        let base = h * (0.79 + layer as f32 * 0.07);
        for col in 0..48 {
            let wave = ((col as f32 * 0.31 + layer as f32 * 2.1).sin() * 0.5 + 0.5)
                * h * (0.055 + layer as f32 * 0.01);
            let y = base - wave;
            ui.rounded(col as f32 * w / 48.0, y, w / 48.0 + 1.0, h - y, 0.0, hex(colors[layer], 255));
        }
    }
}

fn draw_account_chip(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    status: &str,
    signed_in: bool,
) {
    let [x, y, w, h] = rect;
    ui.rounded(x, y, w, h, h * 0.5, theme::CARD());
    ui.border(
        x,
        y,
        w,
        h,
        h * 0.5,
        if signed_in {
            theme::CRIMSON()
        } else {
            theme::BORDER_CRIMSON()
        },
    );
    let dot_col = if signed_in {
        theme::ONLINE()
    } else {
        theme::CRIMSON_BRIGHT()
    };
    ui.rounded(x + w - 16.0, y + (h - 8.0) * 0.5, 8.0, 8.0, 4.0, dot_col);
    let label = ui.fit_text(Face::LauncherMedium, 13.5, status, (w - 34.0).max(24.0));
    ui.text(
        Face::LauncherMedium,
        13.5,
        x + 14.0,
        text_vcenter(y, h, 13.5),
        theme::TEXT(),
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
    home_tab: HomeTab,
    targets: &'a mut Vec<([f32; 4], Action)>,
    list_targets: &'a mut Vec<([f32; 4], Action)>,
    list_view: &'a mut Option<[f32; 3]>,
    login: &'a login::Login,
    volt_accounts: &'a crate::volt_accounts::VoltAccountManager,
    preview_yaw: f32,
    preview_pitch: f32,
    preview_vel_yaw: f32,
    sidebar_anim: f32,
    idle_phase: f32,
    preview_rect: &'a mut Option<[f32; 4]>,
}

impl DrawCtx<'_> {
    fn draw_sidebar(&mut self, ui: &mut Ui, rx: f32, ry: f32, rw: f32, rh: f32) {
        ui.rounded(rx, ry, rw, rh, 16.0, theme::GLASS());
        ui.border(rx, ry, rw, rh, 16.0, theme::GLASS_BORDER());
        ui.text(Face::LauncherMedium, 12.0, rx + 14.0, ry + 16.0, theme::MUTED(), "Instances");
        let add = [rx + rw - 42.0, ry + 7.0, 32.0, 32.0];
        draw_centered_text(ui, Face::LauncherMedium, 22.0, add, theme::TEXT(), "+");
        self.targets.push((add, Action::Profile)); // chooser includes Create profile

        let mut ids = vec![0, self.catalog.config.profile];
        ids.extend(std::fs::read_dir(self.home.join("profiles")).into_iter().flatten().flatten()
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().to_str()?.strip_prefix("profile-")?.parse::<u32>().ok()));
        ids.sort_unstable();
        ids.dedup();
        let max_rows = (((rh - 260.0).max(55.0) / 54.0) as usize).clamp(1, 7);
        if ids.len() > max_rows && !ids[..max_rows].contains(&self.catalog.config.profile) {
            ids.truncate(max_rows.saturating_sub(1));
            ids.push(self.catalog.config.profile);
        } else {
            ids.truncate(max_rows);
        }
        for (index, id) in ids.iter().enumerate() {
            let row = [rx + 8.0, ry + 48.0 + index as f32 * 54.0, rw - 16.0, 50.0];
            if *id == self.catalog.config.profile {
                ui.rounded(row[0], row[1], row[2], row[3], 9.0, theme::CRIMSON_BADGE());
                ui.border(row[0], row[1], row[2], row[3], 9.0, theme::GLASS_BORDER());
            }
            ui.volt_logo(row[0] + 7.0, row[1] + 7.0, 35.0, 35.0);
            let name = if *id == 0 { "Volt".to_owned() } else { format!("Profile {id}") };
            let name = ui.fit_text(Face::LauncherMedium, 13.0, &name, rw - 76.0);
            ui.text(Face::LauncherMedium, 13.0, row[0] + 51.0, row[1] + 6.0, theme::TEXT(), &name);
            ui.rounded(row[0] + 51.0, row[1] + 33.0, 6.0, 6.0, 3.0, theme::ONLINE());
            ui.text(Face::LauncherRegular, 11.0, row[0] + 61.0, row[1] + 27.0, theme::MUTED(), "Native · 26.3");
            self.targets.push((row, Action::ChooseProfile(*id)));
        }

        // All existing destinations stay accessible; Account contains the 3D preview.
        let short = rh < 320.0;
        let foot_y = ry + rh - if short { 148.0 } else { 196.0 };
        let foot_step = if short { 29.0 } else { 37.0 };
        let foot_height = if short { 26.0 } else { 33.0 };
        for (i, (label, action)) in [("Account", Action::Page(Page::Account)),
            ("Volt Settings", Action::Page(Page::VoltSettings))].iter().enumerate() {
            styled_button(ui, self.targets, [rx + 10.0, foot_y + i as f32 * foot_step, rw - 20.0, foot_height],
                label, *action, BtnStyle::Subtle, false);
        }
        let discord_h = if short { 30.0 } else { 38.0 };
        let discord = [rx + 10.0, ry + rh - if short { 83.0 } else { 112.0 }, rw - 20.0, discord_h];
        let discord_bg = horizon_client::ui_theme::color::hex(0x5865f2, 255);
        ui.rounded(discord[0], discord[1], discord[2], discord[3], 10.0, discord_bg);
        draw_discord_icon(ui, discord[0] + 25.0, discord[1] + discord[3] * 0.5, theme::TEXT(), discord_bg);
        draw_centered_text(ui, Face::LauncherMedium, 13.0,
            [discord[0] + 26.0, discord[1], discord[2] - 26.0, discord[3]], theme::TEXT(), "Join the Discord");
        self.targets.push((discord, Action::Discord));
        let github = [rx + 10.0, ry + rh - if short { 46.0 } else { 67.0 }, rw - 20.0, 32.0];
        draw_github_icon(ui, github[0] + 20.0, github[1] + 16.0, theme::MUTED());
        ui.text(Face::LauncherMedium, 13.0, github[0] + 38.0, text_vcenter(github[1], github[3], 13.0), theme::MUTED(), "GitHub");
        self.targets.push((github, Action::Repository));
        if !short {
            ui.text(Face::LauncherRegular, 11.0, rx + 14.0, ry + rh - 27.0, theme::MUTED(),
                &format!("Volt Launcher {}", crate::backend::VERSION));
        }
    }

    /// Screenshot-style top navigation. Every entry points at an existing Volt page.
    fn draw_topbar(&mut self, ui: &mut Ui, area: [f32; 4], account_status: &str) {
        let [x, y, w, _] = area;
        let account_w = if w >= 730.0 { 150.0 } else { 106.0 };
        let tabs = [("Home", Action::Page(Page::Play)), ("Mods", Action::Page(Page::Mods)),
            ("Servers", Action::HomeTab(HomeTab::Multiplayer)), ("Packs", Action::Page(Page::Packs)),
            ("Shaders", Action::Page(Page::Shaders)), ("Files", Action::Page(Page::Installed)),
            ("Settings", Action::Page(Page::Settings))];
        let available = (w - account_w - 12.0).max(240.0);
        let tab_w = (available / tabs.len() as f32).min(88.0);
        for (i, (label, action)) in tabs.iter().enumerate() {
            let rect = [x + i as f32 * tab_w, y, tab_w - 4.0, 36.0];
            let selected = match action {
                Action::HomeTab(tab) => self.catalog.page == Page::Play && self.home_tab == *tab,
                Action::Page(Page::Play) => self.catalog.page == Page::Play && self.home_tab != HomeTab::Multiplayer,
                Action::Page(page) => self.catalog.page == *page,
                _ => false,
            };
            if selected {
                ui.rounded(rect[0], rect[1], rect[2], rect[3], 10.0, theme::CRIMSON_BADGE());
            }
            draw_centered_text(ui, Face::LauncherMedium, 12.5, rect,
                if selected { theme::TEXT() } else { theme::MUTED() }, label);
            self.targets.push((rect, *action));
        }
        draw_account_chip(ui, self.targets, [x + w - account_w, y, account_w, 38.0],
            account_status, self.login.signed_in);
    }

    fn draw_compact_header(&mut self, ui: &mut Ui, w: f32, account_status: &str) {
        ui.rounded(12.0, 10.0, 168.0, 34.0, 9.0, theme::CARD());
        ui.border(12.0, 10.0, 168.0, 34.0, 9.0, theme::BORDER_CRIMSON());
        ui.volt_logo(18.0, 14.0, 26.0, 26.0);
        ui.text(
            Face::LauncherMedium,
            12.5,
            50.0,
            text_vcenter(10.0, 34.0, 12.5),
            theme::TEXT(),
            "VOLT LAUNCHER",
        );
        draw_account_chip(
            ui,
            self.targets,
            [188.0, 10.0, (w - 200.0).max(110.0), 34.0],
            account_status,
            self.login.signed_in,
        );

        // Segmented pill dock with a smoothly gliding selection.
        ui.rounded(12.0, 52.0, w - 24.0, 40.0, 10.0, theme::RAIL());
        ui.border(12.0, 52.0, w - 24.0, 40.0, 10.0, theme::BORDER());
        let tab_w = (w - 32.0) / Page::ALL.len() as f32;
        ui.rounded(
            16.0 + self.sidebar_anim * tab_w + 1.0,
            56.0,
            tab_w - 2.0,
            32.0,
            7.0,
            theme::CRIMSON(),
        );
        for (i, &page) in Page::ALL.iter().enumerate() {
            let tx = 16.0 + i as f32 * tab_w;
            let prox = (1.0 - (self.sidebar_anim - i as f32).abs()).clamp(0.0, 1.0);
            let label = match page {
                Page::Installed => "Files",
                Page::VoltSettings => "Volt",
                Page::Settings => "Setup",
                _ => page.name(),
            };
            let rect = [tx + 1.0, 56.0, tab_w - 2.0, 32.0];
            draw_centered_text(
                ui,
                Face::LauncherMedium,
                12.5,
                rect,
                horizon_client::ui_theme::color::lerp(theme::MUTED(), theme::ON_CRIMSON(), prox),
                label,
            );
            self.targets.push((rect, Action::Page(page)));
        }
    }

    /// The instance header, actions and tabs shared by Home and its Mods tab.
    fn draw_instance_page(&mut self, ui: &mut Ui, area: [f32; 4], compact: bool, account_status: &str) {
        use horizon_client::official_install::InstallStage;
        let [x, y, w, h] = area;
        // When the keyboard or a very short viewport is open, keep the mod search
        // and its results reachable instead of consuming the screen with the header.
        if self.catalog.page == Page::Mods && h < 320.0 {
            self.draw_mods_landing(ui, area, account_status);
            return;
        }
        let installed = horizon_client::official_install::ready(self.home);
        let installing = self.worker.is_some();
        let progress = *self.install.lock().unwrap();
        // Instance title and controls mirror the proportions of the reference.
        let logo = if compact { 38.0 } else { 48.0 };
        ui.volt_logo(x + 2.0, y + 5.0, logo, logo);
        let name_size = if compact { 29.0 } else { 38.0 };
        let name = ui.fit_text(Face::Display, name_size, "Volt Launcher", (w - logo - 16.0).max(24.0));
        ui.text(Face::Display, name_size, x + logo + 12.0, y, theme::TEXT(), &name);
        let desc = if installing { "Preparing Minecraft 26.3" }
            else if installed { "Minecraft 26.3 · Native Rust edition" }
            else { "Minecraft 26.3 · Game files need to be downloaded" };
        let desc = ui.fit_text(Face::LauncherRegular, 12.5, desc, w - 8.0);
        ui.text(Face::LauncherRegular, 12.5, x + 4.0, y + 57.0, theme::TEXT(), &desc);
        let ay = y + 82.0;
        let play_w = if compact { 94.0 } else { 115.0 };
        let label = if installing { "Installing" } else if installed { "Play" }
            else if self.error.is_some() { "Retry" } else { "Download" };
        styled_button(ui, self.targets, [x, ay, play_w, 42.0], label,
            Action::Play, BtnStyle::Primary, true);
        let profile_w = if compact { 112.0 } else { 176.0 };
        styled_button(ui, self.targets, [x + play_w + 8.0, ay, profile_w, 42.0],
            &format!("Profile {} · 26.3  v", self.catalog.config.profile), Action::Profile,
            BtnStyle::Secondary, true);
        if w >= play_w + profile_w + 124.0 {
            styled_button(ui, self.targets, [x + play_w + profile_w + 16.0, ay, 106.0, 42.0],
                "Volt Boost", Action::OpenVoltBoost, BtnStyle::Secondary, true);
        }
        let tabs_y = ay + 64.0;
        let tabs = [("Overview", Action::HomeTab(HomeTab::Overview)),
            ("Mods", Action::Page(Page::Mods)), ("Worlds", Action::HomeTab(HomeTab::Worlds)),
            ("Multiplayer", Action::HomeTab(HomeTab::Multiplayer)), ("Packs", Action::Page(Page::Packs)),
            ("Screenshots", Action::HomeTab(HomeTab::Screenshots)), ("Console", Action::HomeTab(HomeTab::Console))];
        let tab_w = (w / tabs.len() as f32).min(105.0);
        for (i, (label, action)) in tabs.iter().enumerate() {
            let r = [x + i as f32 * tab_w, tabs_y, tab_w - 3.0, 34.0];
            let selected = match action {
                Action::HomeTab(tab) => self.catalog.page == Page::Play && self.home_tab == *tab,
                Action::Page(page) => self.catalog.page == *page,
                _ => false,
            };
            if selected {
                ui.rounded(r[0], r[1], r[2], r[3], 9.0, theme::CRIMSON_BADGE());
                ui.border(r[0], r[1], r[2], r[3], 9.0, theme::GLASS_BORDER());
            }
            draw_centered_text(ui, Face::LauncherMedium, if compact { 11.0 } else { 12.5 }, r,
                if selected { theme::TEXT() } else { theme::MUTED() }, label);
            self.targets.push((r, *action));
        }
        let body_y = tabs_y + 42.0;
        let body_h = (y + h - body_y).max(0.0);
        if self.catalog.page == Page::Mods {
            // Single-column list matches the reference and retains search, filters and toggles.
            self.draw_mods_landing(ui, [x, body_y, w, body_h], account_status);
        } else if body_h >= 60.0 {
            if installing {
                let index = InstallStage::ALL.iter().position(|s| *s == progress.stage).unwrap_or(0);
                let part = if progress.total > 0 { progress.done as f32 / progress.total as f32 } else { 0.0 };
                let fraction = ((index as f32 + part.clamp(0.0, 1.0)) / InstallStage::ALL.len() as f32).clamp(0.0, 1.0);
                let stage = ui.fit_text(Face::LauncherMedium, 14.0, progress.stage.label(), w - 32.0);
                ui.text(Face::LauncherMedium, 14.0, x + 14.0, body_y + 12.0, theme::TEXT(), &stage);
                ui.rounded(x + 14.0, body_y + 42.0, w - 28.0, 7.0, 3.0, theme::CARD_INNER());
                if fraction > 0.0 {
                    ui.rounded(x + 14.0, body_y + 42.0, ((w - 28.0) * fraction).max(7.0), 7.0, 3.0, theme::CRIMSON());
                }
            } else {
                self.draw_home_tab(ui, [x, body_y, w, body_h], account_status, installed);
            }
        }
    }

    fn draw_home_tab(&mut self, ui: &mut Ui, area: [f32; 4], account_status: &str, installed: bool) {
        let [x, y, w, h] = area;
        match self.home_tab {
            HomeTab::Overview => {
                // Separate compact cards rather than an empty full-height rectangle.
                let gap = 12.0;
                let half = (w - gap) * 0.5;
                let card_h = h.min(132.0);
                for r in [[x, y, half, card_h], [x + half + gap, y, half, card_h]] {
                    ui.rounded(r[0], r[1], r[2], r[3], 12.0, theme::GLASS());
                    ui.border(r[0], r[1], r[2], r[3], 12.0, theme::GLASS_BORDER());
                }
                ui.text(Face::LauncherMedium, 15.0, x + 14.0, y + 12.0, theme::TEXT(), "Instance");
                let state = self.error.as_deref().unwrap_or(if installed { "Game files ready · 26.3" } else { "Game files need to be downloaded" });
                let state = ui.fit_text(Face::LauncherRegular, 12.5, state, half - 28.0);
                ui.text(Face::LauncherRegular, 12.5, x + 14.0, y + 42.0, theme::MUTED(), &state);
                let account = ui.fit_text(Face::LauncherRegular, 12.5, account_status, half - 28.0);
                if card_h >= 95.0 {
                    ui.text(Face::LauncherRegular, 12.5, x + 14.0, y + 67.0, theme::MUTED(), &account);
                    self.targets.push(([x + 8.0, y + 62.0, half - 16.0, 30.0], Action::Page(Page::Account)));
                }
                let dx = x + half + gap;
                ui.text(Face::LauncherMedium, 15.0, dx + 14.0, y + 12.0, theme::TEXT(), "Native mods");
                let note = format!("{} of 50 switches on", self.catalog.extras.mod_catalog.enabled_count());
                ui.text(Face::LauncherRegular, 12.5, dx + 14.0, y + 42.0, theme::MUTED(), &note);
                if card_h >= 95.0 {
                    let note = ui.fit_text(Face::LauncherRegular, 11.5, "Some native features are still in development", half - 28.0);
                    ui.text(Face::LauncherRegular, 11.5, dx + 14.0, y + 67.0, theme::MUTED(), &note);
                }
                self.targets.push(([dx, y, half, card_h], Action::Page(Page::Mods)));
            }
            HomeTab::Worlds => {
                ui.text(Face::LauncherMedium, 15.0, x + 8.0, y + 8.0, theme::TEXT(), "Worlds");
                let mut worlds: Vec<String> = std::fs::read_dir(self.catalog.config.game_home(self.home).join("saves"))
                    .into_iter().flatten().flatten().filter(|entry| entry.path().is_dir())
                    .map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
                worlds.sort_unstable();
                let summary = if worlds.is_empty() { "No local world folders found".to_owned() }
                    else { format!("{} local world folders · Open them from the game", worlds.len()) };
                let summary = ui.fit_text(Face::LauncherRegular, 12.5, &summary, w - 16.0);
                ui.text(Face::LauncherRegular, 12.5, x + 8.0, y + 35.0, theme::MUTED(), &summary);
                let top = y + 60.0;
                let bottom = y + h;
                let view_h = (bottom - top).max(0.0);
                let maximum = (worlds.len() as f32 * 28.0 - view_h).max(0.0);
                self.catalog.scroll = self.catalog.scroll.clamp(0.0, maximum);
                *self.list_view = Some([top, bottom, maximum]);
                let start = ui.batch_mut().vertices.len();
                for (i, world) in worlds.iter().enumerate() {
                    let row_y = top + i as f32 * 28.0 - self.catalog.scroll;
                    if row_y + 28.0 < top || row_y > bottom { continue; }
                    let label = ui.fit_text(Face::LauncherRegular, 13.0, world, w - 24.0);
                    ui.text(Face::LauncherRegular, 13.0, x + 12.0, row_y + 3.0, theme::TEXT(), &label);
                }
                let bounds = [ui.to_batch(x), ui.to_batch(top), ui.to_batch(x + w), ui.to_batch(bottom)];
                ui.batch_mut().clip_since(start, bounds[0], bounds[1], bounds[2], bounds[3]);
            }
            HomeTab::Multiplayer => {
                ui.text(Face::LauncherMedium, 15.0, x + 8.0, y + 8.0, theme::TEXT(), "Multiplayer");
                ui.text(Face::LauncherRegular, 12.5, x + 8.0, y + 34.0, theme::MUTED(), "Select a saved server or enter an address");
                let r = [x + 8.0, y + 56.0, (w - 104.0).max(80.0).min(380.0), 38.0];
                ui.rounded(r[0], r[1], r[2], r[3], 10.0, theme::CARD_INNER());
                let value = if self.catalog.config.server.is_empty() { "Server address (optional)" } else { &self.catalog.config.server };
                let value = ui.fit_text(Face::LauncherRegular, 13.0, value, r[2] - 24.0);
                ui.text(Face::LauncherRegular, 13.0, r[0] + 12.0, text_vcenter(r[1], r[3], 13.0), theme::TEXT(), &value);
                self.targets.push((r, Action::Field("server")));
                if w >= 200.0 {
                    styled_button(ui, self.targets, [r[0] + r[2] + 8.0, r[1], 80.0, r[3]],
                        "Save", Action::SaveServer, BtnStyle::Secondary, true);
                }
                let servers = &self.catalog.saved_servers;
                let top = y + 104.0;
                let bottom = y + h;
                let max = (servers.len() as f32 * 54.0 - (bottom - top).max(0.0)).max(0.0);
                self.catalog.scroll = self.catalog.scroll.clamp(0.0, max);
                *self.list_view = Some([top, bottom, max]);
                if servers.is_empty() && bottom > top + 20.0 {
                    ui.text(Face::LauncherRegular, 12.5, x + 8.0, top + 5.0, theme::MUTED(), "No saved servers in this instance.");
                }
                let start = ui.batch_mut().vertices.len();
                for (i, server) in servers.iter().enumerate() {
                    let row_y = top + i as f32 * 54.0 - self.catalog.scroll;
                    let row = [x + 8.0, row_y, w - 16.0, 48.0];
                    self.list_targets.push((row, Action::ConnectServer(i)));
                    if row_y < top || row_y + 48.0 > bottom { continue; }
                    ui.rounded(row[0], row[1], row[2], row[3], 12.0, theme::GLASS_INNER());
                    let title = ui.fit_text(Face::LauncherMedium, 13.0, &server.name, row[2] - 24.0);
                    let address = ui.fit_text(Face::LauncherRegular, 11.5, &server.ip, row[2] - 24.0);
                    ui.text(Face::LauncherMedium, 13.0, row[0] + 12.0, row[1] + 4.0, theme::TEXT(), &title);
                    ui.text(Face::LauncherRegular, 11.5, row[0] + 12.0, row[1] + 25.0, theme::MUTED(), &address);
                    self.targets.push((row, Action::ConnectServer(i)));
                }
                let bounds = [ui.to_batch(x), ui.to_batch(top), ui.to_batch(x+w), ui.to_batch(bottom)];
                ui.batch_mut().clip_since(start, bounds[0], bounds[1], bounds[2], bounds[3]);
            }
            HomeTab::Screenshots => {
                ui.text(Face::LauncherMedium, 15.0, x + 8.0, y + 8.0, theme::TEXT(), "Screenshots");
                let shots = &self.catalog.screenshots;
                let summary = if shots.is_empty() { "No screenshots yet".to_owned() }
                    else { format!("{} screenshots in this instance", shots.len()) };
                ui.text(Face::LauncherRegular, 12.5, x + 8.0, y + 34.0, theme::MUTED(), &summary);
                let top = y + 58.0;
                let bottom = y + h;
                let maximum = (shots.len() as f32 * 28.0 - (bottom - top).max(0.0)).max(0.0);
                self.catalog.scroll = self.catalog.scroll.clamp(0.0, maximum);
                *self.list_view = Some([top, bottom, maximum]);
                let start = ui.batch_mut().vertices.len();
                for (i, shot) in shots.iter().enumerate() {
                    let row_y = top + i as f32 * 28.0 - self.catalog.scroll;
                    if row_y + 28.0 < top || row_y > bottom { continue; }
                    let label = ui.fit_text(Face::LauncherRegular, 13.0, shot, w - 24.0);
                    ui.text(Face::LauncherRegular, 13.0, x + 12.0, row_y + 3.0, theme::TEXT(), &label);
                }
                let bounds = [ui.to_batch(x), ui.to_batch(top), ui.to_batch(x+w), ui.to_batch(bottom)];
                ui.batch_mut().clip_since(start, bounds[0], bounds[1], bounds[2], bounds[3]);
            }
            HomeTab::Console => {
                ui.text(Face::LauncherMedium, 15.0, x + 8.0, y + 8.0, theme::TEXT(), "Launcher diagnostics");
                let status = if self.catalog.status.is_empty() { "Tap Refresh to read the latest local diagnostic summary." } else { &self.catalog.status };
                let status = ui.fit_text(Face::LauncherRegular, 12.5, status, w - 16.0);
                ui.text(Face::LauncherRegular, 12.5, x + 8.0, y + 38.0, theme::MUTED(), &status);
                if h >= 100.0 {
                    styled_button(ui, self.targets, [x + 8.0, y + 64.0, 100.0, 32.0], "Refresh", Action::Logs, BtnStyle::Secondary, true);
                }
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
        use crate::shader_catalog::Origin;
        let hint = match page {
            Page::Mods if self.catalog.config.mod_source != Origin::Horizon => {
                "Java mods (Modrinth / CurseForge) are browse-only; native .hmod mods run on Android"
            }
            Page::Mods => "Search Horizon mods or paste an HTTPS .hmod link",
            Page::Shaders if self.catalog.config.shader_source != Origin::Horizon => {
                "Modrinth & CurseForge shader ZIPs / experimental compatibility"
            }
            Page::Shaders => "Horizon catalog / native .hshader & experimental ZIP support",
            _ => "Modrinth & CurseForge verified resource packs / enable in Installed",
        };
        let origins: &[Origin] = if page == Page::Packs {
            &[Origin::Modrinth, Origin::CurseForge, Origin::Both]
        } else {
            &[
                Origin::Horizon,
                Origin::Modrinth,
                Origin::CurseForge,
                Origin::Both,
            ]
        };
        let active_origin = match page {
            Page::Mods => self.catalog.config.mod_source,
            Page::Shaders => self.catalog.config.shader_source,
            _ => self.catalog.config.pack_source,
        };
        let origin_action = |o: Origin| match page {
            Page::Mods => Action::ModSource(o),
            Page::Shaders => Action::ShaderSource(o),
            _ => Action::PackSource(o),
        };

        // Top header row with Modrinth | CurseForge | Both platform selector!
        if !compact {
            ui.text(Face::LauncherMedium, 24.0, cx, cy, theme::TEXT(), title);
            let acc_w = 186.0;
            let tab_item_w = if origins.len() == 4 { 84.0 } else { 88.0 };
            let seg_w = 8.0 + origins.len() as f32 * tab_item_w;
            let hint_max = (cw - acc_w - seg_w - 32.0).max(120.0);
            let fitted_hint = ui.fit_text(Face::LauncherRegular, 13.0, hint, hint_max);
            ui.text(Face::LauncherRegular, 13.0, cx, cy + 30.0, theme::MUTED(), &fitted_hint);

            draw_account_chip(
                ui,
                self.targets,
                [cx + cw - acc_w, cy + 4.0, acc_w, 38.0],
                account_status,
                self.login.signed_in,
            );

            let seg_x = cx + cw - acc_w - seg_w - 10.0;
            ui.rounded(seg_x, cy + 4.0, seg_w, 38.0, 10.0, theme::CARD());
            ui.border(seg_x, cy + 4.0, seg_w, 38.0, 10.0, theme::BORDER());
            for (idx, &origin) in origins.iter().enumerate() {
                let ox = seg_x + 4.0 + idx as f32 * tab_item_w;
                let orect = [ox + 1.0, cy + 8.0, tab_item_w - 2.0, 30.0];
                let is_on = active_origin == origin;
                if is_on {
                    ui.rounded(orect[0], orect[1], orect[2], orect[3], 7.0, theme::CRIMSON_BADGE());
                    ui.border(orect[0], orect[1], orect[2], orect[3], 7.0, theme::CRIMSON());
                }
                draw_centered_text(
                    ui,
                    Face::LauncherMedium,
                    12.5,
                    orect,
                    if is_on { theme::ON_CRIMSON() } else { theme::MUTED() },
                    origin.label(),
                );
                self.targets.push((orect, origin_action(origin)));
            }
        } else {
            ui.text(Face::LauncherMedium, 18.0, cx, cy, theme::TEXT(), title);
            let seg_y = cy + 26.0;
            ui.rounded(cx, seg_y, cw, 34.0, 9.0, theme::CARD());
            ui.border(cx, seg_y, cw, 34.0, 9.0, theme::BORDER());
            let tab_item_w = (cw - 8.0) / origins.len() as f32;
            for (idx, &origin) in origins.iter().enumerate() {
                let orect = [
                    cx + 4.0 + idx as f32 * tab_item_w + 1.0,
                    seg_y + 3.0,
                    tab_item_w - 2.0,
                    28.0,
                ];
                let is_on = active_origin == origin;
                if is_on {
                    ui.rounded(orect[0], orect[1], orect[2], orect[3], 6.5, theme::CRIMSON());
                }
                draw_centered_text(
                    ui,
                    Face::LauncherMedium,
                    12.0,
                    orect,
                    if is_on { theme::ON_CRIMSON() } else { theme::MUTED() },
                    origin.label(),
                );
                self.targets.push((orect, origin_action(origin)));
            }
        }

        // Unified Command Toolbar Card (Search + Search CTA + Open folder + Import file)
        let bar_y = cy + if compact { 66.0 } else { 54.0 };
        let bar_h = if compact { 88.0 } else { 52.0 };
        ui.rounded(cx, bar_y, cw, bar_h, 12.0, theme::CARD());
        ui.border(cx, bar_y, cw, bar_h, 12.0, theme::BORDER());

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
            ui.rounded(srect[0], srect[1], srect[2], srect[3], 8.0, theme::CARD_INNER());
            ui.border(
                srect[0],
                srect[1],
                srect[2],
                srect[3],
                8.0,
                if self.catalog.editing == Some("search") {
                    theme::CRIMSON()
                } else {
                    theme::BORDER()
                },
            );
            let qfit = ui.fit_text(Face::LauncherRegular, 13.5, &display_query, s_w - 24.0);
            ui.text(
                Face::LauncherRegular,
                13.5,
                srect[0] + 12.0,
                text_vcenter(srect[1], srect[3], 13.5),
                if query_is_placeholder {
                    theme::DIM()
                } else {
                    theme::TEXT()
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
            ui.rounded(srect[0], srect[1], srect[2], srect[3], 9.0, theme::CARD_INNER());
            ui.border(
                srect[0],
                srect[1],
                srect[2],
                srect[3],
                9.0,
                if self.catalog.editing == Some("search") {
                    theme::CRIMSON()
                } else {
                    theme::BORDER()
                },
            );
            let qfit = ui.fit_text(Face::LauncherRegular, 14.0, &display_query, input_w - 28.0);
            ui.text(
                Face::LauncherRegular,
                14.0,
                srect[0] + 14.0,
                text_vcenter(srect[1], srect[3], 14.0),
                if query_is_placeholder {
                    theme::DIM()
                } else {
                    theme::TEXT()
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
            ui.rounded(cx, top, cw, view_h, 14.0, theme::CARD());
            ui.border(cx, top, cw, view_h, 14.0, theme::BORDER());

            let mid_x = cx + cw * 0.5;
            let mid_y = top + view_h * 0.5;

            // Layered crimson diamond/stack emblem
            ui.rounded(mid_x - 36.0, mid_y - 68.0, 72.0, 48.0, 12.0, theme::HERO_GLOW_SOFT());
            for layer in 0..3 {
                let ly = mid_y - 64.0 + layer as f32 * 11.0;
                ui.border(mid_x - 24.0, ly, 48.0, 16.0, 4.0, theme::CRIMSON_BRIGHT());
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
                Face::LauncherMedium,
                18.0,
                [cx + 20.0, mid_y - 8.0, cw - 40.0, 26.0],
                theme::TEXT(),
                empty_title,
            );
            let empty_sub = match page {
                Page::Shaders => "Import a .hshader or experimental shaders ZIP to get started.",
                Page::Mods => "Paste an HTTPS .hmod URL above or import a local .hmod package.",
                _ => "Search above or import a resource pack ZIP from your device.",
            };
            draw_centered_text(
                ui,
                Face::LauncherRegular,
                13.5,
                [cx + 20.0, mid_y + 20.0, cw - 40.0, 22.0],
                theme::MUTED(),
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

            let results_age = self.catalog.results_born.elapsed().as_secs_f32();
            for (i, p) in self.catalog.results.iter().enumerate() {
                let row = i / cols;
                let col_idx = i % cols;
                let x = cx + col_idx as f32 * (card_w + gap);
                let y = top + row as f32 * row_step - self.catalog.scroll;

                let download = self.catalog.download_label(p);
                let is_java_mod = p.kind == crate::browser::Kind::Mod
                    && matches!(
                        p.source,
                        crate::browser::SourceId::Modrinth | crate::browser::SourceId::CurseForge
                    );
                let can_install = download.is_none()
                    && !crate::catalog::source_only_mod(p)
                    && !is_java_mod
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

                let card_v_start = ui.batch_mut().vertices.len();
                let icon_slot = 4 + (i % 24);
                let project_icon = self.catalog.project_icon(p);

                ui.rounded(x, y, card_w, card_h, 12.0, theme::CARD());
                ui.border(x, y, card_w, card_h, 12.0, theme::BORDER_CRIMSON());

                if cols == 3 {
                    // 3-Column Resource Pack Card layout with real project logo + source pill!
                    let icon_sz = 46.0;
                    ui.rounded(x + 14.0, y + 13.0, icon_sz, icon_sz, 10.0, theme::HERO_BG());
                    let mut drew_logo = false;
                    if let Some(ic) = project_icon {
                        let fade = (ic.loaded_at.elapsed().as_secs_f32() / 0.22).clamp(0.0, 1.0);
                        let alpha = (ease_out_cubic(fade) * 255.0).round() as u8;
                        drew_logo = ui.draw_dynamic_rgba(
                            icon_slot,
                            ic.hash,
                            64,
                            64,
                            &ic.rgba,
                            x + 15.0,
                            y + 14.0,
                            icon_sz - 2.0,
                            icon_sz - 2.0,
                            alpha,
                        );
                    }
                    if !drew_logo {
                        let pulse = (self.idle_phase * 4.2 + i as f32 * 0.7).sin() * 0.5 + 0.5;
                        ui.rounded(
                            x + 14.0,
                            y + 13.0,
                            icon_sz,
                            icon_sz,
                            10.0,
                            horizon_client::ui_theme::color::lerp(
                                theme::CRIMSON_BADGE(),
                                theme::HERO_BG(),
                                pulse,
                            ),
                        );
                        draw_page_icon(
                            ui,
                            page,
                            x + 14.0 + icon_sz * 0.5,
                            y + 13.0 + icon_sz * 0.5,
                            theme::CRIMSON_BRIGHT(),
                        );
                    }
                    ui.border(
                        x + 14.0,
                        y + 13.0,
                        icon_sz,
                        icon_sz,
                        10.0,
                        theme::BORDER_CRIMSON(),
                    );

                    // Platform badge next to icon so Both mode clearly distinguishes Modrinth vs CurseForge
                    let src_tag = p.source.label();
                    let sw = ui.measure(Face::LauncherMedium, 10.5, src_tag) + 12.0;
                    if x + 66.0 + sw < btn_rect[0] - 6.0 {
                        ui.rounded(x + 66.0, y + 25.0, sw, 20.0, 6.0, theme::CRIMSON_BADGE());
                        draw_centered_text(
                            ui,
                            Face::LauncherMedium,
                            10.5,
                            [x + 66.0, y + 25.0, sw, 20.0],
                            theme::CRIMSON_BRIGHT(),
                            src_tag,
                        );
                    }

                    let title_fit = ui.fit_text(Face::LauncherMedium, 16.0, &p.title, card_w - 28.0);
                    ui.text(
                        Face::LauncherMedium,
                        16.0,
                        x + 14.0,
                        y + 66.0,
                        theme::CRIMSON_BRIGHT(),
                        &title_fit,
                    );
                    let desc_lines =
                        wrap_lines(ui, Face::LauncherRegular, 12.5, &p.summary, card_w - 28.0, 3);
                    for (li, line) in desc_lines.iter().enumerate() {
                        ui.text(
                            Face::LauncherRegular,
                            12.5,
                            x + 14.0,
                            y + 92.0 + li as f32 * 18.0,
                            theme::MUTED(),
                            line,
                        );
                    }
                } else {
                    // 2-Column Rich Mod / Shader / Pack Card layout with real project logo!
                    let icon_sz = if compact { 44.0 } else { 56.0 };
                    ui.rounded(x + 14.0, y + 14.0, icon_sz, icon_sz, 11.0, theme::HERO_BG());
                    let mut drew_logo = false;
                    if let Some(ic) = project_icon {
                        let fade = (ic.loaded_at.elapsed().as_secs_f32() / 0.22).clamp(0.0, 1.0);
                        let alpha = (ease_out_cubic(fade) * 255.0).round() as u8;
                        drew_logo = ui.draw_dynamic_rgba(
                            icon_slot,
                            ic.hash,
                            64,
                            64,
                            &ic.rgba,
                            x + 15.0,
                            y + 15.0,
                            icon_sz - 2.0,
                            icon_sz - 2.0,
                            alpha,
                        );
                    }
                    if !drew_logo {
                        let pulse = (self.idle_phase * 4.2 + i as f32 * 0.7).sin() * 0.5 + 0.5;
                        ui.rounded(
                            x + 14.0,
                            y + 14.0,
                            icon_sz,
                            icon_sz,
                            11.0,
                            horizon_client::ui_theme::color::lerp(
                                theme::CRIMSON_BADGE(),
                                theme::HERO_BG(),
                                pulse,
                            ),
                        );
                        draw_page_icon(
                            ui,
                            page,
                            x + 14.0 + icon_sz * 0.5,
                            y + 14.0 + icon_sz * 0.5,
                            theme::CRIMSON_BRIGHT(),
                        );
                    }
                    ui.border(
                        x + 14.0,
                        y + 14.0,
                        icon_sz,
                        icon_sz,
                        11.0,
                        theme::BORDER_CRIMSON(),
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
                        ui.measure(Face::LauncherRegular, 12.0, &dl_text) + 12.0
                    };
                    let title_max = (card_w - (text_x - x) - dl_w - 14.0).max(60.0);
                    let title_fit = ui.fit_text(Face::LauncherMedium, 16.5, &p.title, title_max);
                    ui.text(Face::LauncherMedium, 16.5, text_x, y + 12.0, theme::TEXT(), &title_fit);
                    if !dl_text.is_empty() {
                        ui.text(
                            Face::LauncherRegular,
                            12.0,
                            x + card_w - dl_w - 2.0,
                            y + 15.0,
                            theme::MUTED(),
                            &dl_text,
                        );
                    }

                    // Tag pills (source + loader + version)
                    if !compact {
                        let mut tx = text_x;
                        let src_tag = p.source.label();
                        let sw = ui.measure(Face::LauncherMedium, 11.0, src_tag) + 14.0;
                        ui.rounded(tx, y + 38.0, sw, 20.0, 6.0, theme::CRIMSON_BADGE());
                        draw_centered_text(
                            ui,
                            Face::LauncherMedium,
                            11.0,
                            [tx, y + 38.0, sw, 20.0],
                            theme::CRIMSON_BRIGHT(),
                            src_tag,
                        );
                        tx += sw + 6.0;

                        for tag in p
                            .loaders
                            .iter()
                            .take(1)
                            .chain(p.game_versions.iter().take(2))
                        {
                            let tw = ui.measure(Face::LauncherRegular, 11.0, tag) + 14.0;
                            if tx + tw > x + card_w - 14.0 {
                                break;
                            }
                            ui.rounded(tx, y + 38.0, tw, 20.0, 6.0, theme::CARD_INNER());
                            draw_centered_text(
                                ui,
                                Face::LauncherRegular,
                                11.0,
                                [tx, y + 38.0, tw, 20.0],
                                theme::MUTED(),
                                tag,
                            );
                            tx += tw + 6.0;
                        }
                    }

                    let desc_y = if compact { y + 38.0 } else { y + 64.0 };
                    let desc_w = (card_w - (text_x - x) - 14.0).max(80.0);
                    let lines = wrap_lines(ui, Face::LauncherRegular, 12.5, &p.summary, desc_w, 2);
                    for (li, line) in lines.iter().enumerate() {
                        ui.text(
                            Face::LauncherRegular,
                            12.5,
                            text_x,
                            desc_y + li as f32 * 17.0,
                            theme::MUTED(),
                            line,
                        );
                    }

                    // Widened "View details" button so the full label never truncates!
                    if !compact
                        && (p.page.starts_with("https://") || p.page.starts_with("http://"))
                        && y + card_h - btn_h - 12.0 >= top
                        && y + card_h - 12.0 <= bottom
                    {
                        styled_button(
                            ui,
                            self.targets,
                            [x + 14.0, y + card_h - btn_h - 12.0, 128.0, btn_h],
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
                            theme::CARD_INNER(),
                        );
                        ui.border(
                            btn_rect[0],
                            btn_rect[1],
                            btn_rect[2],
                            btn_rect[3],
                            8.0,
                            theme::CRIMSON(),
                        );
                        draw_centered_text(
                            ui,
                            Face::LauncherMedium,
                            13.0,
                            btn_rect,
                            theme::TEXT(),
                            &state,
                        );
                    } else if is_java_mod {
                        draw_centered_text(
                            ui,
                            Face::LauncherRegular,
                            12.5,
                            btn_rect,
                            theme::MUTED(),
                            "Java mod",
                        );
                    } else if crate::catalog::source_only_mod(p) {
                        draw_centered_text(
                            ui,
                            Face::LauncherRegular,
                            12.5,
                            btn_rect,
                            theme::MUTED(),
                            ".hnmod: later",
                        );
                    } else if p.kind == crate::browser::Kind::Mod
                        && !cfg!(target_pointer_width = "64")
                    {
                        draw_centered_text(
                            ui,
                            Face::LauncherRegular,
                            12.0,
                            btn_rect,
                            theme::MUTED(),
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

                // Staggered card entrance slide + micro-scale + fade!
                let delay = (i.min(8) as f32) * 0.028;
                if results_age < delay + 0.25 {
                    let raw_t = ((results_age - delay) / 0.25).clamp(0.0, 1.0);
                    let ease = ease_out_cubic(raw_t);
                    ui.animate_vertices(
                        card_v_start,
                        0.0,
                        (1.0 - ease) * 14.0,
                        0.97 + 0.03 * ease,
                        [x + card_w * 0.5, y + card_h * 0.5],
                        ease,
                    );
                }
            }
        }

        // Bottom Pagination & Status Bar (matching 02-mods.png)
        let fy = cy + ch - footer_h;
        ui.rounded(cx, fy, cw, footer_h, 11.0, theme::CARD());
        ui.border(cx, fy, cw, footer_h, 11.0, theme::BORDER());
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
                Face::LauncherRegular,
                13.0,
                [cx + 124.0, fy, (cw - 248.0).max(40.0), footer_h],
                if self.error.is_some() {
                    theme::CRIMSON_BRIGHT()
                } else {
                    theme::MUTED()
                },
                status_text,
            );
        }
    }

    /// Choose between Volt's switches and the live Horizon registry.
    fn draw_mods_landing(&mut self, ui: &mut Ui, area: [f32; 4], account_status: &str) {
        let [x, y, w, h] = area;
        let tab_w = (w * 0.5).min(160.0);
        for (i, (label, browser)) in [("Horizon browser", true), ("Volt mods", false)].iter().enumerate() {
            let rect = [x + i as f32 * (tab_w + 6.0), y, tab_w, 30.0];
            styled_button(ui, self.targets, rect, label, Action::HorizonModBrowser(*browser),
                if self.catalog.horizon_mod_browser == *browser { BtnStyle::Primary } else { BtnStyle::Secondary }, true);
        }
        let body = [x, y + 36.0, w, (h - 36.0).max(0.0)];
        if self.catalog.horizon_mod_browser {
            self.draw_horizon_mod_browser(ui, body);
        } else {
            self.draw_mods_page(ui, body, true, account_status);
        }
    }

    /// A paged, in-app registry browser. Native downloads remain source-only
    /// unless their exact bundle is already compiled into this APK.
    fn draw_horizon_mod_browser(&mut self, ui: &mut Ui, area: [f32; 4]) {
        let [x, y, w, h] = area;
        if h < 52.0 || w < 180.0 { return; }
        let input = [x, y, (w - 96.0).max(40.0), 32.0];
        ui.rounded(input[0], input[1], input[2], input[3], 8.0, theme::CARD_INNER());
        ui.border(input[0], input[1], input[2], input[3], 8.0, theme::BORDER());
        let placeholder = self.catalog.query.is_empty() && self.catalog.editing != Some("search");
        let text = if placeholder { "Search Horizon mods..." } else { &self.catalog.query };
        let text = ui.fit_text(Face::LauncherRegular, 13.0, text, input[2] - 18.0);
        ui.text(Face::LauncherRegular, 13.0, x + 9.0, y + 6.0,
            if placeholder { theme::DIM() } else { theme::TEXT() }, &text);
        self.targets.push((input, Action::Field("search")));
        styled_button(ui, self.targets, [x + w - 90.0, y, 90.0, 32.0], "Search",
            Action::Search, BtnStyle::Primary, true);
        let status = ui.fit_text(Face::LauncherRegular, 11.0, &self.catalog.status, w - 8.0);
        ui.text(Face::LauncherRegular, 11.0, x + 4.0, y + 37.0, theme::MUTED(), &status);
        let top = y + 56.0;
        let bottom = y + h - 35.0;
        let view_h = (bottom - top).max(0.0);
        if view_h < 1.0 { return; }
        let row_h = 76.0;
        let step = 83.0;
        let total = self.catalog.results.len() as f32 * step;
        let max_scroll = (total - view_h).max(0.0);
        self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
        *self.list_view = Some([top, bottom, max_scroll]);
        if self.catalog.results.is_empty() {
            let hint = if self.catalog.status.is_empty() { "Search the Horizon registry" } else { &self.catalog.status };
            let hint = ui.fit_text(Face::LauncherRegular, 12.0, hint, w - 16.0);
            ui.text(Face::LauncherRegular, 12.0, x + 8.0, top + 8.0, theme::MUTED(), &hint);
        }
        for (i, p) in self.catalog.results.iter().enumerate() {
            let ry = top + i as f32 * step - self.catalog.scroll;
            let button = [x + w - 108.0, ry + 8.0, 98.0, 29.0];
            let native = crate::catalog::source_only_mod(p);
            let installed = self.catalog.download_label(p);
            let supported = !(p.kind == crate::browser::Kind::Mod && !native && !cfg!(target_pointer_width = "64"));
            let prebuilt = native && horizon_client::bundled_native::supports(&p.id);
            let remove_bundled = prebuilt && self.catalog.bundled_mods.contains(&p.id) && !self.catalog.busy();
            let can_press = installed.is_none() && supported && !self.catalog.busy();
            if can_press { self.list_targets.push((button, Action::Install(i))); }
            if remove_bundled { self.list_targets.push((button, Action::RemoveBundled(i))); }
            if ry + row_h < top || ry > bottom { continue; }
            let start = ui.batch_mut().vertices.len();
            ui.rounded(x, ry, w, row_h, 10.0, theme::CARD());
            ui.border(x, ry, w, row_h, 10.0, theme::BORDER());
            let icon = [x + 8.0, ry + 9.0, 42.0, 42.0];
            ui.rounded(icon[0], icon[1], icon[2], icon[3], 8.0, theme::HERO_BG());
            if let Some(img) = self.catalog.project_icon(p) {
                ui.draw_dynamic_rgba(4 + i % 24, img.hash, 64, 64, &img.rgba,
                    icon[0], icon[1], icon[2], icon[3], 255);
            }
            let tx = x + 58.0;
            let title_width = (w - 178.0).max(32.0);
            let title = ui.fit_text(Face::LauncherMedium, 14.0, &p.title, title_width);
            ui.text(Face::LauncherMedium, 14.0, tx, ry + 8.0, theme::TEXT(), &title);
            let kind = if native {
                if prebuilt && self.catalog.bundled_mods.contains(&p.id) { "Native · bundled build installed" }
                else if prebuilt { "Native · 2.0.0 prebuilt only" }
                else if self.catalog.native_sources.contains(&p.id) { "Native source saved · no Volt build" }
                else { "Native · source only, no Volt build" }
            } else if supported { "Horizon mod · 64-bit" } else { "Horizon mod · 64-bit required" };
            let subtitle = format!("{} · {}", p.author, kind);
            let subtitle = ui.fit_text(Face::LauncherRegular, 11.0, &subtitle, title_width);
            ui.text(Face::LauncherRegular, 11.0, tx, ry + 30.0, theme::DIM(), &subtitle);
            let summary = if native && !prebuilt {
                "Source only — needs a compatible Android build"
            } else { &p.summary };
            let summary = ui.fit_text(Face::LauncherRegular, 11.0, summary, (w - 70.0).max(30.0));
            ui.text(Face::LauncherRegular, 11.0, tx, ry + 49.0, theme::MUTED(), &summary);
            if remove_bundled {
                let first = self.targets.len();
                styled_button(ui, self.targets, button, "Remove", Action::RemoveBundled(i), BtnStyle::Secondary, true);
                if let Some(clipped) = super::gesture::clip_target_y(button, top, bottom) {
                    self.targets[first].0 = clipped;
                } else { self.targets.truncate(first); }
            } else if let Some(label) = installed {
                draw_centered_text(ui, Face::LauncherMedium, 11.0, button, theme::MUTED(), &label);
            } else if can_press {
                let first = self.targets.len();
                styled_button(ui, self.targets, button, if native && !prebuilt { "Download" } else { "Install" },
                    Action::Install(i), BtnStyle::Primary, true);
                if let Some(clipped) = super::gesture::clip_target_y(button, top, bottom) {
                    self.targets[first].0 = clipped;
                } else { self.targets.truncate(first); }
            } else if !supported {
                draw_centered_text(ui, Face::LauncherRegular, 11.0, button, theme::MUTED(), "64-bit only");
            }
            let bounds = [ui.to_batch(x), ui.to_batch(top), ui.to_batch(x + w), ui.to_batch(bottom)];
            ui.batch_mut().clip_since(start, bounds[0], bounds[1], bounds[2], bounds[3]);
        }
        let footer = y + h - 30.0;
        let prev_start = self.targets.len();
        let prev_active = self.catalog.offset > 0 && !self.catalog.busy();
        styled_button(ui, self.targets, [x, footer, 86.0, 28.0], "Previous", Action::Previous,
            if prev_active { BtnStyle::Secondary } else { BtnStyle::Subtle }, true);
        if !prev_active { self.targets.truncate(prev_start); }
        let page = format!("Page {}", self.catalog.offset / 12 + 1);
        draw_centered_text(ui, Face::LauncherRegular, 11.0, [x + 88.0, footer, (w - 176.0).max(0.0), 28.0], theme::MUTED(), &page);
        let next_start = self.targets.len();
        let next_active = self.catalog.results.len() >= 11 && !self.catalog.busy();
        styled_button(ui, self.targets, [x + w - 86.0, footer, 86.0, 28.0], "Next", Action::Next,
            if next_active { BtnStyle::Secondary } else { BtnStyle::Subtle }, true);
        if !next_active { self.targets.truncate(next_start); }
    }

    fn draw_mods_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        account_status: &str,
    ) {
        use horizon_client::ui_theme::color::hex;
        use horizon_client::volt_mods::{
            CATALOG, TOTAL_VOLT_MODS, VoltModCategory, mod_logo_spec,
        };

        let [cx, cy, cw, ch] = area;
        let enabled_total = self.catalog.extras.mod_catalog.enabled_count();

        let filters: [(u8, &str); 8] = [
            (0, "All"),
            (1, "On"),
            (2, "Off"),
            (3, "PvP"),
            (4, "HUD"),
            (5, "Visual"),
            (6, "Perf"),
            (7, "QoL"),
        ];

        if !compact {
            ui.text(Face::LauncherMedium, 24.0, cx, cy, theme::TEXT(), "Volt Native Mods");
            let pill_txt = format!("{enabled_total} / {TOTAL_VOLT_MODS} Active");
            let pill_w = ui.measure(Face::LauncherMedium, 12.0, &pill_txt) + 22.0;
            let pill_x = cx + ui.measure(Face::LauncherMedium, 24.0, "Volt Native Mods") + 14.0;
            ui.rounded(pill_x, cy + 4.0, pill_w, 24.0, 12.0, hex(0x0E2925, 235));
            ui.border(pill_x, cy + 4.0, pill_w, 24.0, 12.0, hex(0x2DD4BF, 220));
            draw_centered_text(
                ui,
                Face::LauncherMedium,
                12.0,
                [pill_x, cy + 4.0, pill_w, 24.0],
                hex(0x5EEAD4, 255),
                &pill_txt,
            );

            let acc_w = 186.0;
            let hint =
                "50 catalog entries • Native features are still being implemented";
            let fitted_hint = ui.fit_text(Face::LauncherRegular, 13.0, hint, (cw - acc_w - 24.0).max(120.0));
            ui.text(Face::LauncherRegular, 13.0, cx, cy + 30.0, theme::MUTED(), &fitted_hint);

            draw_account_chip(
                ui,
                self.targets,
                [cx + cw - acc_w, cy + 4.0, acc_w, 38.0],
                account_status,
                self.login.signed_in,
            );
        } else {
            let hdr = format!("Volt Mods  ({enabled_total}/{TOTAL_VOLT_MODS} On)");
            ui.text(Face::LauncherMedium, 18.0, cx, cy, theme::TEXT(), &hdr);
        }

        // Command Search + Category Filter Bar (no .hmod / Modrinth / CurseForge)
        let bar_y = cy + if compact { 28.0 } else { 54.0 };
        let bar_h = if compact { 84.0 } else { 88.0 };
        ui.rounded(cx, bar_y, cw, bar_h, 12.0, theme::CARD());
        ui.border(cx, bar_y, cw, bar_h, 12.0, theme::BORDER());

        let placeholder = "Search 50 native mods by name, category, or feature...";
        let query_is_placeholder =
            self.catalog.query.is_empty() && self.catalog.editing != Some("search");
        let display_query = if query_is_placeholder {
            placeholder.to_owned()
        } else {
            self.catalog.query.clone()
        };

        if compact {
            let clear_w = if self.catalog.query.is_empty() {
                0.0
            } else {
                68.0
            };
            let s_w = (cw - 16.0 - if clear_w > 0.0 { clear_w + 6.0 } else { 0.0 }).max(80.0);
            let srect = [cx + 8.0, bar_y + 7.0, s_w, 34.0];
            ui.rounded(srect[0], srect[1], srect[2], srect[3], 8.0, theme::CARD_INNER());
            ui.border(
                srect[0],
                srect[1],
                srect[2],
                srect[3],
                8.0,
                if self.catalog.editing == Some("search") {
                    theme::CRIMSON_BRIGHT()
                } else {
                    theme::BORDER()
                },
            );
            let qfit = ui.fit_text(Face::LauncherRegular, 13.0, &display_query, s_w - 22.0);
            ui.text(
                Face::LauncherRegular,
                13.0,
                srect[0] + 11.0,
                text_vcenter(srect[1], srect[3], 13.0),
                if query_is_placeholder {
                    theme::DIM()
                } else {
                    theme::TEXT()
                },
                &qfit,
            );
            self.targets.push((srect, Action::Field("search")));
            if clear_w > 0.0 {
                styled_button(
                    ui,
                    self.targets,
                    [srect[0] + s_w + 6.0, bar_y + 7.0, clear_w, 34.0],
                    "Clear",
                    Action::ClearModSearch,
                    BtnStyle::Secondary,
                    true,
                );
            }
        } else {
            let btn_pack_w = 148.0;
            let btn_reset_w = 128.0;
            let clear_w = if self.catalog.query.is_empty() {
                0.0
            } else {
                74.0
            };
            let extra_clear = if clear_w > 0.0 { clear_w + 8.0 } else { 0.0 };
            let input_w = (cw - btn_pack_w - btn_reset_w - extra_clear - 32.0).max(160.0);
            let srect = [cx + 8.0, bar_y + 8.0, input_w, 36.0];
            ui.rounded(srect[0], srect[1], srect[2], srect[3], 9.0, theme::CARD_INNER());
            ui.border(
                srect[0],
                srect[1],
                srect[2],
                srect[3],
                9.0,
                if self.catalog.editing == Some("search") {
                    theme::CRIMSON_BRIGHT()
                } else {
                    theme::BORDER()
                },
            );
            let qfit = ui.fit_text(Face::LauncherRegular, 13.5, &display_query, input_w - 28.0);
            ui.text(
                Face::LauncherRegular,
                13.5,
                srect[0] + 14.0,
                text_vcenter(srect[1], srect[3], 13.5),
                if query_is_placeholder {
                    theme::DIM()
                } else {
                    theme::TEXT()
                },
                &qfit,
            );
            self.targets.push((srect, Action::Field("search")));

            let mut bx = srect[0] + input_w + 8.0;
            if clear_w > 0.0 {
                styled_button(
                    ui,
                    self.targets,
                    [bx, bar_y + 8.0, clear_w, 36.0],
                    "Clear",
                    Action::ClearModSearch,
                    BtnStyle::Secondary,
                    true,
                );
                bx += clear_w + 8.0;
            }
            styled_button(
                ui,
                self.targets,
                [bx, bar_y + 8.0, btn_pack_w, 36.0],
                "Enable PvP + Perf",
                Action::EnableModPack,
                BtnStyle::Primary,
                true,
            );
            bx += btn_pack_w + 8.0;
            styled_button(
                ui,
                self.targets,
                [bx, bar_y + 8.0, btn_reset_w, 36.0],
                "Reset Defaults",
                Action::ResetModDefaults,
                BtnStyle::CrimsonOutline,
                true,
            );
        }

        // Filter row inside command card
        let filt_y = bar_y + if compact { 46.0 } else { 50.0 };
        let filt_w = (cw - 16.0 - (filters.len() - 1) as f32 * 5.0) / filters.len() as f32;
        for (idx, &(code, label)) in filters.iter().enumerate() {
            let fx = cx + 8.0 + idx as f32 * (filt_w + 5.0);
            let frect = [fx, filt_y, filt_w, 30.0];
            let active = self.catalog.mod_filter == code;
            let hovered = point_in(ui.cursor, frect);
            ui.rounded(
                frect[0],
                frect[1],
                frect[2],
                frect[3],
                7.5,
                if active {
                    theme::CRIMSON_BADGE()
                } else if hovered {
                    theme::HOVER()
                } else {
                    theme::CARD_INNER()
                },
            );
            ui.border(
                frect[0],
                frect[1],
                frect[2],
                frect[3],
                7.5,
                if active {
                    theme::CRIMSON_BRIGHT()
                } else {
                    theme::BORDER()
                },
            );
            draw_centered_text(
                ui,
                Face::LauncherMedium,
                if compact { 11.5 } else { 12.5 },
                frect,
                if active {
                    theme::ON_CRIMSON()
                } else {
                    theme::MUTED()
                },
                label,
            );
            self.targets.push((frect, Action::ModFilter(code)));
        }

        // Filter the 50 mods live
        let q = self.catalog.query.trim().to_ascii_lowercase();
        let filt = self.catalog.mod_filter;
        let visible: Vec<_> = CATALOG
            .iter()
            .filter(|m| {
                let on = self.catalog.extras.mod_catalog.is_on(m.id);
                let cat_ok = match filt {
                    1 => on,
                    2 => !on,
                    3 => m.category == VoltModCategory::PvpUtility,
                    4 => m.category == VoltModCategory::PvpHud,
                    5 => m.category == VoltModCategory::Visual,
                    6 => m.category == VoltModCategory::Performance,
                    7 => m.category == VoltModCategory::Additional,
                    _ => true,
                };
                if !cat_ok {
                    return false;
                }
                if q.is_empty() {
                    return true;
                }
                m.title.to_ascii_lowercase().contains(&q)
                    || m.slug.contains(&q)
                    || m.category.label().to_ascii_lowercase().contains(&q)
                    || m.summary.to_ascii_lowercase().contains(&q)
            })
            .collect();

        let top = bar_y + bar_h + 10.0;
        let bottom = cy + ch - 4.0;
        let view_h = (bottom - top).max(0.0);
        if view_h < 1.0 {
            return;
        }

        let native_installed = self.catalog.bundled_zoom_installed;
        let native_enabled = self.catalog.bundled_zoom_enabled;
        let native_visible = (q.is_empty() || "horizon zoom official native bundled".contains(&q))
            && match filt { 0 | 3 => true, 1 => native_enabled, 2 => !native_enabled, _ => false };
        if visible.is_empty() && !native_visible {
            ui.rounded(cx, top, cw, view_h, 14.0, theme::CARD());
            ui.border(cx, top, cw, view_h, 14.0, theme::BORDER());
            draw_centered_text(
                ui,
                Face::LauncherMedium,
                16.5,
                [cx + 20.0, top + view_h * 0.5 - 18.0, cw - 40.0, 24.0],
                theme::TEXT(),
                "No native mods match your current filter",
            );
            draw_centered_text(
                ui,
                Face::LauncherRegular,
                13.0,
                [cx + 20.0, top + view_h * 0.5 + 8.0, cw - 40.0, 20.0],
                theme::MUTED(),
                "Try switching back to 'All' or clearing your search query.",
            );
            return;
        }

        let cols = if compact {
            1usize
        } else if cw >= 920.0 {
            3usize
        } else {
            2usize
        };
        let gap = 10.0;
        let card_w = (cw - (cols.saturating_sub(1) as f32) * gap) / cols as f32;
        let card_h = if compact { 82.0 } else { 88.0 };
        let row_step = card_h + gap;
        let total_rows = (visible.len() + usize::from(native_visible)).div_ceil(cols);
        let content_h = (total_rows as f32 * row_step - gap).max(0.0);
        let max_scroll = (content_h - view_h).max(0.0);
        self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
        *self.list_view = Some([top, bottom, max_scroll]);

        let born_age = self.catalog.mod_born.elapsed().as_secs_f32();
        for i in 0..(visible.len() + usize::from(native_visible)) {
            let row = i / cols;
            let col_idx = i % cols;
            let x = cx + col_idx as f32 * (card_w + gap);
            let y = top + row as f32 * row_step - self.catalog.scroll;
            if native_visible && i == 0 {
                let rect = [x, y, card_w, card_h];
                self.list_targets.push((rect, Action::InstallBundledZoom));
                if y + card_h < top - 4.0 || y > bottom + 4.0 { continue; }
                let start = ui.batch_mut().vertices.len();
                ui.rounded(x, y, card_w, card_h, 11.0, theme::CARD());
                ui.border(x, y, card_w, card_h, 11.0, theme::BORDER());
                let button_w = if native_installed { 92.0 } else { 98.0 };
                let title_w = (card_w - button_w - 30.0).max(30.0);
                let title = ui.fit_text(Face::LauncherMedium, 14.5, "Horizon Zoom 2.0.0", title_w);
                ui.text(Face::LauncherMedium, 14.5, x + 12.0, y + 10.0, theme::TEXT(), &title);
                let state = if self.catalog.status.starts_with("Could not change Horizon Zoom:") {
                    self.catalog.status.as_str()
                } else if native_installed {
                    if native_enabled { "Installed · On in game" } else { "Installed · Off in game" }
                } else { "Bundled · not installed" };
                let state = ui.fit_text(Face::LauncherRegular, 11.0, state, title_w);
                ui.text(Face::LauncherRegular, 11.0, x + 12.0, y + 30.0, theme::DIM(), &state);
                let info = ui.fit_text(Face::LauncherRegular, 11.0, "Hold Z · add a Z touch button in Controls Studio", card_w - 24.0);
                ui.text(Face::LauncherRegular, 11.0, x + 12.0, y + 52.0, theme::MUTED(), &info);
                let b = [x + card_w - button_w - 10.0, y + 8.0, button_w, 29.0];
                let target_start = self.targets.len();
                styled_button(ui, self.targets, b, if native_installed { "Remove" } else { "Install" },
                    Action::InstallBundledZoom, BtnStyle::Secondary, true);
                if let Some(clipped) = super::gesture::clip_target_y(b, top, bottom) {
                    self.targets[target_start].0 = clipped;
                } else { self.targets.truncate(target_start); }
                let bounds = [ui.to_batch(cx), ui.to_batch(top), ui.to_batch(cx + cw), ui.to_batch(bottom)];
                ui.batch_mut().clip_since(start, bounds[0], bounds[1], bounds[2], bounds[3]);
                continue;
            }
            let m = visible[i - usize::from(native_visible)];
            let sw_rect = [x + card_w - 58.0, y + 12.0, 46.0, 24.0];
            let act = Action::ToggleVoltMod(m.id);
            self.list_targets.push((sw_rect, act));

            if y + card_h < top - 4.0 || y > bottom + 4.0 {
                continue;
            }

            let card_v_start = ui.batch_mut().vertices.len();
            let is_on = self.catalog.extras.mod_catalog.is_on(m.id);
            let card_rect = [x, y, card_w, card_h];
            let hovered = point_in(ui.cursor, card_rect);

            ui.rounded(
                x,
                y,
                card_w,
                card_h,
                11.0,
                if is_on {
                    hex(0x131922, 248)
                } else if hovered {
                    theme::HOVER()
                } else {
                    theme::CARD()
                },
            );
            ui.border(
                x,
                y,
                card_w,
                card_h,
                11.0,
                if is_on {
                    hex(0x2DD4BF, 175)
                } else if hovered {
                    theme::BORDER_CRIMSON()
                } else {
                    theme::BORDER()
                },
            );
            if is_on {
                ui.rounded(x + 1.0, y + 12.0, 3.0, card_h - 24.0, 1.5, hex(0x2DD4BF, 255));
            }

            // Custom 8x8 Pixel-Art Mod Logo (horizontal-run merged for crisp zero-lag quads)
            let logo_sz = 44.0f32;
            let lx = x + 12.0;
            let ly = y + (card_h - logo_sz) * 0.5;
            let spec = mod_logo_spec(m.id);
            ui.rounded(lx - 2.0, ly - 2.0, logo_sz + 4.0, logo_sz + 4.0, 9.0, hex(spec.bg, 255));
            ui.border(
                lx - 2.0,
                ly - 2.0,
                logo_sz + 4.0,
                logo_sz + 4.0,
                9.0,
                hex(spec.primary, if is_on { 190 } else { 95 }),
            );
            let has_upstream_icon = horizon_client::volt_mods::upstream_icon(m.id)
                .is_some_and(|(name, _)| ui.atlas_sprite(name, lx, ly, logo_sz, logo_sz));
            let cell = logo_sz / 8.0;
            let pri_col = hex(spec.primary, 255);
            let acc_col = hex(spec.accent, 255);
            for (ry_i, row_str) in spec.rows.iter().enumerate().filter(|_| !has_upstream_icon) {
                let bytes = row_str.as_bytes();
                let mut rx_i = 0usize;
                while rx_i < bytes.len().min(8) {
                    let b = bytes[rx_i];
                    let pcol = match b {
                        b'#' => Some(pri_col),
                        b'+' | b'&' => Some(acc_col),
                        _ => None,
                    };
                    if let Some(col) = pcol {
                        let start = rx_i;
                        rx_i += 1;
                        while rx_i < bytes.len().min(8) && bytes[rx_i] == b {
                            rx_i += 1;
                        }
                        let px = lx + start as f32 * cell;
                        let py = ly + ry_i as f32 * cell;
                        let pw = (rx_i - start) as f32 * cell;
                        ui.rounded(px, py, pw, cell, 0.0, col);
                    } else {
                        rx_i += 1;
                    }
                }
            }

            // Title + Category badge + Summary
            let tx = lx + logo_sz + 12.0;
            let title_max = (card_w - (tx - x) - 66.0).max(56.0);
            let tfit = ui.fit_text(Face::LauncherMedium, 14.5, m.title, title_max);
            ui.text(Face::LauncherMedium, 14.5, tx, y + 11.0, theme::TEXT(), &tfit);

            let cat_txt = format!("{} • v{}", m.category.short_label(), m.version);
            let cfit = ui.fit_text(Face::LauncherRegular, 11.0, &cat_txt, title_max);
            ui.text(
                Face::LauncherRegular,
                11.0,
                tx,
                y + 30.0,
                if is_on {
                    hex(0x5EEAD4, 240)
                } else {
                    theme::DIM()
                },
                &cfit,
            );

            let sum_max = (card_w - (tx - x) - 14.0).max(60.0);
            let sfit = ui.fit_text(Face::LauncherRegular, 12.0, m.summary, sum_max);
            ui.text(Face::LauncherRegular, 12.0, tx, y + 50.0, theme::MUTED(), &sfit);

            // Clip touch regions as well as vertices; a partially scrolled row must
            // not intercept the tabs, search field, or footer outside the viewport.
            let target_start = self.targets.len();
            toggle(ui, self.targets, sw_rect, is_on, act);
            if let Some(rect) = super::gesture::clip_target_y(sw_rect, top, bottom) {
                self.targets[target_start].0 = rect;
            } else {
                self.targets.truncate(target_start);
            }
            if let Some(rect) = super::gesture::clip_target_y(card_rect, top, bottom) {
                self.targets.push((rect, act));
            }

            // Smooth staggered entrance animation
            let delay = (i.min(12) as f32) * 0.018;
            if born_age < delay + 0.22 {
                let raw_t = ((born_age - delay) / 0.22).clamp(0.0, 1.0);
                let ease = ease_out_cubic(raw_t);
                ui.animate_vertices(
                    card_v_start,
                    0.0,
                    (1.0 - ease) * 12.0,
                    0.975 + 0.025 * ease,
                    [x + card_w * 0.5, y + card_h * 0.5],
                    ease,
                );
            }
            let bounds = [ui.to_batch(cx), ui.to_batch(top), ui.to_batch(cx + cw), ui.to_batch(bottom)];
            ui.batch_mut().clip_since(card_v_start, bounds[0], bounds[1], bounds[2], bounds[3]);
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
            ui.text(Face::LauncherMedium, 24.0, cx, cy, theme::TEXT(), "Installed Content");
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
                ui.rounded(col_x, top, col_w, card_h, 13.0, theme::CARD());
                ui.border(col_x, top, col_w, card_h, 13.0, theme::BORDER_CRIMSON());

                let count = self.catalog.local.iter().filter(|e| e.kind == kind).count();
                let header_label = format!("{} ({count})", kind.label());
                let hfit = ui.fit_text(Face::LauncherMedium, 16.5, &header_label, col_w - 60.0);
                ui.text(Face::LauncherMedium, 16.5, col_x + 16.0, top + 12.0, theme::TEXT(), &hfit);

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
                ui.rounded(col_x + 12.0, top + 46.0, col_w - 24.0, 1.0, 0.0, theme::BORDER());

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
                        Face::LauncherRegular,
                        13.0,
                        [col_x + 14.0, inner_top + 20.0, col_w - 28.0, 28.0],
                        theme::MUTED(),
                        "Section collapsed — tap > to expand",
                    );
                    continue;
                }

                if count == 0 {
                    // Empty dropzone box inside column (matching 05-installed.png)
                    let dz = [col_x + 14.0, inner_top, col_w - 28.0, inner_h];
                    ui.rounded(dz[0], dz[1], dz[2], dz[3], 10.0, theme::BG());
                    ui.border(dz[0], dz[1], dz[2], dz[3], 10.0, theme::BORDER());
                    let mid_y = dz[1] + dz[3] * 0.5;
                    draw_page_icon(ui, Page::Installed, dz[0] + dz[2] * 0.5, mid_y - 26.0, theme::TEXT());
                    draw_centered_text(
                        ui,
                        Face::LauncherRegular,
                        13.5,
                        [dz[0] + 8.0, mid_y - 6.0, dz[2] - 16.0, 22.0],
                        theme::TEXT(),
                        "Tap Import to add files",
                    );
                    draw_centered_text(
                        ui,
                        Face::LauncherRegular,
                        12.5,
                        [dz[0] + 8.0, mid_y + 18.0, dz[2] - 16.0, 20.0],
                        theme::MUTED(),
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
                            theme::CARD_INNER(),
                        );
                        ui.border(
                            col_x + 12.0,
                            iy,
                            col_w - 24.0,
                            item_h - 6.0,
                            9.0,
                            theme::BORDER(),
                        );
                        let tfit = ui.fit_text(Face::LauncherMedium, 13.5, &item.title, col_w - 40.0);
                        ui.text(Face::LauncherMedium, 13.5, col_x + 20.0, iy + 6.0, theme::TEXT(), &tfit);
                        if kind == Kind::Mods && !cfg!(target_pointer_width = "64") {
                            ui.text(
                                Face::LauncherRegular,
                                12.0,
                                col_x + 20.0,
                                iy + 34.0,
                                theme::MUTED(),
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
            ui.rounded(cx, fy, cw, footer_h, 10.0, theme::CARD());
            ui.border(cx, fy, cw, footer_h, 10.0, theme::BORDER());
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
            let fitted = ui.fit_text(Face::LauncherRegular, 13.5, msg, cw - 32.0);
            ui.text(
                Face::LauncherRegular,
                13.5,
                cx + 16.0,
                text_vcenter(fy, footer_h, 13.5),
                theme::TEXT(),
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
                    ui.rounded(cx, y, cw, row_height - 8.0, 10.0, theme::CARD());
                    ui.border(cx, y, cw, row_height - 8.0, 10.0, theme::BORDER());
                    let tfit = ui.fit_text(Face::LauncherMedium, 15.0, &item.title, cw - 24.0);
                    ui.text(Face::LauncherMedium, 15.0, cx + 12.0, y + 10.0, theme::TEXT(), &tfit);
                    if kind == Kind::Mods && !cfg!(target_pointer_width = "64") {
                        ui.text(
                            Face::LauncherRegular,
                            13.0,
                            cx + 12.0,
                            y + 54.0,
                            theme::MUTED(),
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

        let vulkan_driver_val = match &c.custom_vulkan_driver_name {
            Some(name) if !name.trim().is_empty() => format!("Custom: {name}"),
            _ if c.custom_vulkan_driver.is_some() => "Custom .so active".to_owned(),
            _ => "Built-in Supporter".to_owned(),
        };

        let left_items: [(&str, String, Action, Option<bool>, bool); 11] = [
            ("Appearance", if c.launcher_dark { "Dark" } else { "Light" }.to_owned(), Action::ToggleAppearance, None, false),
            ("Accent color", format!("#{:06X}", c.launcher_accent), Action::CycleAccent, None, false),
            (
                "Rendering",
                c.renderer.label().to_owned(),
                Action::Renderer,
                None,
                true,
            ),
            (
                "Vulkan driver",
                vulkan_driver_val,
                Action::ImportVulkanDriver,
                None,
                false,
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

        let right_items: [(&str, String, Action, Option<bool>, bool); 8] = [
            (
                "Profile",
                format!("Profile {}", c.profile),
                Action::Profile,
                None,
                true,
            ),
            (
                "Reset GPU driver",
                if c.custom_vulkan_driver.is_some() {
                    "Switch to Built-in".to_owned()
                } else {
                    "Using Built-in".to_owned()
                },
                Action::ResetVulkanDriver,
                None,
                false,
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
            ui.rounded(cx, cy, cw, import_h, 14.0, theme::HERO_BG());
            ui.rounded(
                cx + 8.0,
                cy + 8.0,
                cw * 0.46,
                import_h - 16.0,
                12.0,
                theme::HERO_GLOW_SOFT(),
            );
            ui.border(cx, cy, cw, import_h, 14.0, theme::BORDER_CRIMSON());

            let btn_grid_w = (cw * 0.45).clamp(256.0, 344.0);
            let btn_x = cx + cw - btn_grid_w - 16.0;
            let half_btn_w = (btn_grid_w - 8.0) * 0.5;
            let text_max = (btn_x - cx - 32.0).max(140.0);

            ui.text(
                Face::LauncherMedium,
                18.0,
                cx + 20.0,
                cy + 11.0,
                theme::TEXT(),
                "Controls Studio",
            );
            if text_max >= 260.0 {
                let badge_x = cx + 162.0;
                ui.rounded(badge_x, cy + 12.0, 128.0, 22.0, 6.0, theme::CRIMSON_BADGE());
                ui.border(badge_x, cy + 12.0, 128.0, 22.0, 6.0, theme::BORDER_CRIMSON());
                draw_centered_text(
                    ui,
                    Face::LauncherMedium,
                    11.0,
                    [badge_x, cy + 12.0, 128.0, 22.0],
                    theme::CRIMSON_BRIGHT(),
                    "Pojav • Zalith • FCL",
                );
            }

            let active_fit = ui.fit_text(Face::LauncherMedium, 13.5, &active_ctrl_summary, text_max);
            ui.text(
                Face::LauncherMedium,
                13.5,
                cx + 20.0,
                cy + 41.0,
                if self.catalog.active_controls.is_some() {
                    theme::CRIMSON_BRIGHT()
                } else {
                    theme::TEXT()
                },
                &active_fit,
            );
            let sub_hint =
                "Built-in Pojav, Zalith, FCL & Claw presets + visual button editor & .json/.zip import";
            let sub_fit = ui.fit_text(Face::LauncherRegular, 12.5, sub_hint, text_max);
            ui.text(
                Face::LauncherRegular,
                12.5,
                cx + 20.0,
                cy + 65.0,
                theme::MUTED(),
                &sub_fit,
            );

            let third_btn_w = (btn_grid_w - 12.0) / 3.0;
            styled_button(
                ui,
                self.targets,
                [btn_x, cy + 13.0, half_btn_w, 34.0],
                "Controls Studio",
                Action::Studio(super::controls_studio::StudioAction::Open),
                BtnStyle::Primary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x + half_btn_w + 8.0, cy + 13.0, half_btn_w, 34.0],
                "Import File",
                Action::ImportControl,
                BtnStyle::CrimsonOutline,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x, cy + 53.0, third_btn_w, 31.0],
                &saved_label,
                Action::ControlLayouts,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x + third_btn_w + 6.0, cy + 53.0, third_btn_w, 31.0],
                "Apply All",
                Action::ApplyControlsAllProfiles,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x + (third_btn_w + 6.0) * 2.0, cy + 53.0, third_btn_w, 31.0],
                "Reset",
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

            ui.rounded(cx, cards_y, col_w, cards_h, 14.0, theme::CARD());
            ui.border(cx, cards_y, col_w, cards_h, 14.0, theme::BORDER());
            ui.text(
                Face::LauncherMedium,
                17.0,
                cx + 20.0,
                cards_y + 12.0,
                theme::TEXT(),
                "Display & Graphics",
            );

            let rx = cx + col_w + gap;
            ui.rounded(rx, cards_y, col_w, cards_h, 14.0, theme::CARD());
            ui.border(rx, cards_y, col_w, cards_h, 14.0, theme::BORDER());
            ui.text(
                Face::LauncherMedium,
                17.0,
                rx + 20.0,
                cards_y + 12.0,
                theme::TEXT(),
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
                    let row_rect = [base_x + 10.0, y + 1.0, col_w - 20.0, row_h - 4.0];
                    if point_in(ui.cursor, row_rect) {
                        ui.rounded(row_rect[0], row_rect[1], row_rect[2], row_rect[3], 7.0, theme::HOVER());
                        ui.rounded(row_rect[0] + 1.0, row_rect[1] + 6.0, 2.5, row_rect[3] - 12.0, 1.2, theme::CRIMSON_BRIGHT());
                    }
                    let lbl_fit =
                        ui.fit_text(Face::LauncherRegular, 14.0, label, col_w - ctrl_w - 40.0);
                    ui.text(
                        Face::LauncherRegular,
                        14.0,
                        base_x + 20.0,
                        text_vcenter(y, row_h - 4.0, 14.0),
                        if point_in(ui.cursor, row_rect) { theme::TEXT() } else { theme::MUTED() },
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
                            theme::BORDER(),
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
                let lines = wrap_lines(ui, Face::LauncherRegular, 13.0, status, col_w, 2);
                for (li, line) in lines.iter().enumerate() {
                    ui.text(
                        Face::LauncherRegular,
                        13.0,
                        cx + 4.0,
                        btn_y + 2.0 + li as f32 * 18.0,
                        if self.error.is_some() {
                            theme::CRIMSON_BRIGHT()
                        } else {
                            theme::MUTED()
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
            let third_btn = (cw - 32.0 - 12.0) / 3.0;
            let stu_btn_r = [cx + 16.0, iy + 68.0, half_btn, 36.0];
            let imp_btn_r = [cx + 16.0 + half_btn + 8.0, iy + 68.0, half_btn, 36.0];
            let sav_btn_r = [cx + 16.0, iy + 110.0, third_btn, 32.0];
            let all_btn_r = [cx + 16.0 + third_btn + 6.0, iy + 110.0, third_btn, 32.0];
            let rst_btn_r = [cx + 16.0 + (third_btn + 6.0) * 2.0, iy + 110.0, third_btn, 32.0];
            self.list_targets.push((
                stu_btn_r,
                Action::Studio(super::controls_studio::StudioAction::Open),
            ));
            self.list_targets.push((imp_btn_r, Action::ImportControl));
            self.list_targets.push((sav_btn_r, Action::ControlLayouts));
            self.list_targets
                .push((all_btn_r, Action::ApplyControlsAllProfiles));
            self.list_targets.push((rst_btn_r, Action::ResetControls));

            if iy >= top - 4.0 && iy + import_card_h <= bottom + 4.0 {
                ui.rounded(cx, iy, cw, import_card_h, 12.0, theme::HERO_BG());
                ui.border(cx, iy, cw, import_card_h, 12.0, theme::BORDER_CRIMSON());
                ui.text(
                    Face::LauncherMedium,
                    16.5,
                    cx + 16.0,
                    iy + 10.0,
                    theme::TEXT(),
                    "Controls Studio",
                );
                let act_fit =
                    ui.fit_text(Face::LauncherRegular, 12.5, &active_ctrl_summary, cw - 32.0);
                ui.text(
                    Face::LauncherRegular,
                    12.5,
                    cx + 16.0,
                    iy + 38.0,
                    if self.catalog.active_controls.is_some() {
                        theme::CRIMSON_BRIGHT()
                    } else {
                        theme::MUTED()
                    },
                    &act_fit,
                );
                styled_button(
                    ui,
                    self.targets,
                    stu_btn_r,
                    "Controls Studio",
                    Action::Studio(super::controls_studio::StudioAction::Open),
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    imp_btn_r,
                    "Import File",
                    Action::ImportControl,
                    BtnStyle::CrimsonOutline,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    sav_btn_r,
                    &saved_label,
                    Action::ControlLayouts,
                    BtnStyle::Secondary,
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
                    "Reset",
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
                ui.text(Face::LauncherRegular, 13.5, cx + 2.0, y, theme::MUTED(), label);
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

    fn draw_volt_settings_page(&mut self, ui: &mut Ui, area: [f32; 4], compact: bool) {
        use horizon_client::extras::Item;
        use horizon_client::volt_features::VoltBoostPreset;

        let [cx, cy, cw, ch] = area;
        let c = &self.catalog.config;
        let ex = &self.catalog.extras;

        let render_scale_val = if self.catalog.editing == Some("render_scale") {
            self.catalog.render_scale_text.clone()
        } else {
            format!("{}%", c.render_scale.percent())
        };
        let vulkan_driver_val = match &c.custom_vulkan_driver_name {
            Some(name) if !name.trim().is_empty() => format!("Custom: {name}"),
            _ if c.custom_vulkan_driver.is_some() => "Custom .so active".to_owned(),
            _ => "Built-in Supporter".to_owned(),
        };

        let left_items: Vec<(&str, String, Action, Option<bool>, bool)> = vec![
            (
                "Rendering engine",
                c.renderer.label().to_owned(),
                Action::Renderer,
                None,
                true,
            ),
            (
                "Vulkan driver",
                vulkan_driver_val,
                Action::ImportVulkanDriver,
                None,
                false,
            ),
            (
                "Client resolution",
                render_scale_val,
                Action::RenderScale,
                None,
                true,
            ),
            (
                "Auto-DRS dynamic scale",
                Item::VoltAutoDrs.value(ex),
                Action::VoltExtra(Item::VoltAutoDrs),
                None,
                false,
            ),
            (
                "Display refresh rate",
                c.device.display_label(),
                Action::DisplayHz,
                None,
                true,
            ),
            (
                "ADPF CPU/GPU boost",
                String::new(),
                Action::PerformanceHints,
                Some(c.device.performance_hints),
                false,
            ),
            (
                "big.LITTLE prime affinity",
                String::new(),
                Action::VoltExtra(Item::VoltThreadBoost),
                Some(ex.thread_boost),
                false,
            ),
            (
                "Volt Reflex low-latency",
                String::new(),
                Action::VoltExtra(Item::VoltReflex),
                Some(ex.volt_reflex),
                false,
            ),
            (
                "Shader & cache warmup",
                String::new(),
                Action::VoltExtra(Item::VoltPrewarm),
                Some(ex.prewarm_pipelines),
                false,
            ),
            (
                "Dynamic menu / idle FPS",
                Item::VoltDynamicFps.value(ex),
                Action::VoltExtra(Item::VoltDynamicFps),
                None,
                false,
            ),
            (
                "Menu backdrop saver",
                Item::VoltMenuBg.value(ex),
                Action::VoltExtra(Item::VoltMenuBg),
                None,
                false,
            ),
            (
                "Mobile thermal guard",
                String::new(),
                Action::VoltExtra(Item::VoltThermalGuard),
                Some(ex.thermal_guard),
                false,
            ),
            (
                "Audio mixer voice cap",
                Item::VoltAudioCap.value(ex),
                Action::VoltExtra(Item::VoltAudioCap),
                None,
                false,
            ),
            (
                "Aggressive RAM compactor",
                String::new(),
                Action::VoltExtra(Item::VoltMemorySaver),
                Some(ex.memory_saver),
                false,
            ),
            (
                "Keep screen awake",
                String::new(),
                Action::VoltExtra(Item::VoltKeepAwake),
                Some(ex.keep_screen_awake),
                false,
            ),
            (
                "Live performance HUD",
                Item::VoltPerfHud.value(ex),
                Action::VoltExtra(Item::VoltPerfHud),
                None,
                false,
            ),
            (
                "Perf HUD corner",
                Item::VoltPerfHudCorner.value(ex),
                Action::VoltExtra(Item::VoltPerfHudCorner),
                None,
                false,
            ),
        ];

        let right_items: Vec<(&str, String, Action, Option<bool>, bool)> = vec![
            (
                "Smart leaf & canopy cull",
                String::new(),
                Action::VoltExtra(Item::VoltCullLeaves),
                Some(ex.cull_leaves),
                false,
            ),
            (
                "Frustum-first chunk mesh",
                String::new(),
                Action::VoltExtra(Item::VoltFrustumMesh),
                Some(ex.frustum_priority_mesh),
                false,
            ),
            (
                "Velocity lookahead stream",
                String::new(),
                Action::VoltExtra(Item::VoltLookaheadStream),
                Some(ex.lookahead_stream),
                false,
            ),
            (
                "Adaptive water sort",
                String::new(),
                Action::VoltExtra(Item::VoltFastTranslucency),
                Some(ex.fast_translucency),
                false,
            ),
            (
                "Cave sky & cloud cull",
                String::new(),
                Action::VoltExtra(Item::VoltCaveSkyCull),
                Some(ex.cave_sky_cull),
                false,
            ),
            (
                "Chunk GPU upload cap",
                Item::VoltChunkUploadCap.value(ex),
                Action::VoltExtra(Item::VoltChunkUploadCap),
                None,
                false,
            ),
            (
                "Fast weather columns",
                String::new(),
                Action::VoltExtra(Item::VoltFastWeather),
                Some(ex.fast_weather),
                false,
            ),
            (
                "Distant horizon LOD boost",
                String::new(),
                Action::VoltExtra(Item::VoltDistantLod),
                Some(ex.distant_lod_boost),
                false,
            ),
            (
                "Max rendered entities",
                Item::VoltEntityCap.value(ex),
                Action::VoltExtra(Item::VoltEntityCap),
                None,
                false,
            ),
            (
                "Dropped item cap",
                Item::VoltItemCap.value(ex),
                Action::VoltExtra(Item::VoltItemCap),
                None,
                false,
            ),
            (
                "Block-entity cap + occlude",
                Item::VoltBlockEntityCap.value(ex),
                Action::VoltExtra(Item::VoltBlockEntityCap),
                None,
                false,
            ),
            (
                "Sign & hologram text cull",
                String::new(),
                Action::VoltExtra(Item::VoltCullSignText),
                Some(ex.cull_sign_text),
                false,
            ),
            (
                "Entity contact shadows",
                Item::VoltFastShadows.value(ex),
                Action::VoltExtra(Item::VoltFastShadows),
                None,
                false,
            ),
            (
                "Distant mob pose throttle",
                String::new(),
                Action::VoltExtra(Item::VoltDistantMobThrottle),
                Some(ex.distant_mob_throttle),
                false,
            ),
            (
                "Multiplayer hub lag shield",
                String::new(),
                Action::VoltExtra(Item::VoltHubLagShield),
                Some(ex.hub_lag_shield),
                false,
            ),
            (
                "Particle budget & cutoff",
                Item::VoltParticleBudget.value(ex),
                Action::VoltExtra(Item::VoltParticleBudget),
                None,
                false,
            ),
            (
                "TNT / firework / break shield",
                String::new(),
                Action::VoltExtra(Item::VoltChaosShield),
                Some(ex.chaos_particle_shield),
                false,
            ),
            (
                "Smart 10 Hz animated atlas",
                String::new(),
                Action::VoltExtra(Item::VoltAnimThrottle),
                Some(ex.anim_throttle),
                false,
            ),
            (
                "Glint & armor-trim saver",
                Item::VoltGlintMode.value(ex),
                Action::VoltExtra(Item::VoltGlintMode),
                None,
                false,
            ),
        ];

        let rss_mib = horizon_threads::resident_ram_mib().unwrap_or(0);
        let total_mib = horizon_client::extras::detect_total_ram_mib().unwrap_or(0);
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let abi = if cfg!(target_arch = "aarch64") {
            "arm64-v8a"
        } else {
            "armeabi-v7a"
        };
        let active_n = ex.active_volt_optimizations();
        let hw_summary = format!(
            "RAM: {rss_mib} / {total_mib} MiB  •  {cores} Cores ({abi})  •  {active_n} Boosts Active"
        );

        let presets = [
            (VoltBoostPreset::Off, "OFF"),
            (VoltBoostPreset::Balanced, "Balanced"),
            (VoltBoostPreset::Competitive, "PvP"),
            (VoltBoostPreset::UltraFps, "Ultra FPS"),
            (VoltBoostPreset::BatterySaver, "Battery"),
            (VoltBoostPreset::LowRam, "Low-RAM"),
        ];

        if !compact {
            let hero_h = 104.0;
            ui.rounded(cx, cy, cw, hero_h, 14.0, theme::HERO_BG());
            ui.rounded(
                cx + 8.0,
                cy + 8.0,
                cw * 0.48,
                hero_h - 16.0,
                12.0,
                theme::HERO_GLOW_SOFT(),
            );
            ui.border(cx, cy, cw, hero_h, 14.0, theme::BORDER_CRIMSON());

            ui.text(
                Face::LauncherMedium,
                18.5,
                cx + 20.0,
                cy + 11.0,
                theme::TEXT(),
                "Volt Performance & Optimization Suite",
            );
            let hw_fit = ui.fit_text(Face::LauncherRegular, 13.0, &hw_summary, (cw - 320.0).max(160.0));
            ui.text(
                Face::LauncherRegular,
                13.0,
                cx + 20.0,
                cy + 36.0,
                theme::MUTED(),
                &hw_fit,
            );

            let act_w = 132.0;
            styled_button(
                ui,
                self.targets,
                [cx + cw - act_w * 2.0 - 24.0, cy + 12.0, act_w, 34.0],
                "Auto-Tune",
                Action::VoltExtra(Item::VoltAutoOptimize),
                BtnStyle::Primary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [cx + cw - act_w - 16.0, cy + 12.0, act_w, 34.0],
                "Purge RAM",
                Action::VoltExtra(Item::VoltPurgeRam),
                BtnStyle::CrimsonOutline,
                true,
            );

            let pill_gap = 8.0;
            let pill_w = ((cw - 32.0 - pill_gap * 5.0) / 6.0).max(64.0);
            for (pi, (preset, name)) in presets.iter().enumerate() {
                let px = cx + 16.0 + pi as f32 * (pill_w + pill_gap);
                let selected = ex.volt_preset == *preset;
                styled_button(
                    ui,
                    self.targets,
                    [px, cy + 60.0, pill_w, 33.0],
                    name,
                    Action::VoltPreset(*preset),
                    if selected {
                        BtnStyle::Primary
                    } else {
                        BtnStyle::Secondary
                    },
                    true,
                );
            }

            let gap = 16.0;
            let col_w = (cw - gap) * 0.5;
            let cards_y = cy + hero_h + 10.0;
            let cards_h = (ch - hero_h - 12.0).max(150.0);

            ui.rounded(cx, cards_y, col_w, cards_h, 14.0, theme::CARD());
            ui.border(cx, cards_y, col_w, cards_h, 14.0, theme::BORDER());
            ui.text(
                Face::LauncherMedium,
                16.5,
                cx + 20.0,
                cards_y + 12.0,
                theme::TEXT(),
                "Launcher, Hardware, CPU & RAM",
            );

            let rx = cx + col_w + gap;
            ui.rounded(rx, cards_y, col_w, cards_h, 14.0, theme::CARD());
            ui.border(rx, cards_y, col_w, cards_h, 14.0, theme::BORDER());
            ui.text(
                Face::LauncherMedium,
                16.5,
                rx + 20.0,
                cards_y + 12.0,
                theme::TEXT(),
                "In-World Terrain, Entities & Particles",
            );

            let list_top = cards_y + 46.0;
            let list_bottom = cards_y + cards_h - 8.0;
            let row_h = 46.0;
            let max_rows = left_items.len().max(right_items.len());
            let max_scroll = (max_rows as f32 * row_h - (list_bottom - list_top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([list_top, list_bottom, max_scroll]);

            for (slice, base_x) in [(&left_items[..], cx), (&right_items[..], rx)] {
                for (i, (label, val, act, sw, dropdown)) in slice.iter().enumerate() {
                    let y = list_top + i as f32 * row_h - self.catalog.scroll;
                    let ctrl_w = (col_w * 0.46).clamp(124.0, 214.0);
                    let ctrl_rect = [base_x + col_w - ctrl_w - 16.0, y + 5.0, ctrl_w, 34.0];
                    self.list_targets.push((ctrl_rect, *act));
                    if y < list_top - 4.0 || y + row_h - 4.0 > list_bottom {
                        continue;
                    }
                    let row_rect = [base_x + 10.0, y + 1.0, col_w - 20.0, row_h - 4.0];
                    if point_in(ui.cursor, row_rect) {
                        ui.rounded(row_rect[0], row_rect[1], row_rect[2], row_rect[3], 7.0, theme::HOVER());
                        ui.rounded(row_rect[0] + 1.0, row_rect[1] + 6.0, 2.5, row_rect[3] - 12.0, 1.2, theme::CRIMSON_BRIGHT());
                    }
                    let lbl_fit =
                        ui.fit_text(Face::LauncherRegular, 13.5, label, col_w - ctrl_w - 36.0);
                    ui.text(
                        Face::LauncherRegular,
                        13.5,
                        base_x + 18.0,
                        text_vcenter(y, row_h - 4.0, 13.5),
                        if point_in(ui.cursor, row_rect) { theme::TEXT() } else { theme::MUTED() },
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
                            base_x + 18.0,
                            y + row_h - 2.0,
                            col_w - 34.0,
                            1.0,
                            0.0,
                            theme::BORDER(),
                        );
                    }
                }
            }
        } else {
            let top = cy;
            let bottom = cy + ch;
            let all_items: Vec<_> = left_items.into_iter().chain(right_items).collect();
            let hero_h = 152.0;
            let row_h = 66.0;
            let total_h = hero_h + 12.0 + all_items.len() as f32 * row_h;
            let max_scroll = (total_h - (bottom - top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([top, bottom, max_scroll]);

            let iy = top - self.catalog.scroll;
            let half_btn = (cw - 32.0 - 8.0) * 0.5;
            let auto_r = [cx + 16.0, iy + 58.0, half_btn, 36.0];
            let purge_r = [cx + 16.0 + half_btn + 8.0, iy + 58.0, half_btn, 36.0];
            let next_preset_r = [cx + 16.0, iy + 102.0, cw - 32.0, 36.0];
            self.list_targets
                .push((auto_r, Action::VoltExtra(Item::VoltAutoOptimize)));
            self.list_targets
                .push((purge_r, Action::VoltExtra(Item::VoltPurgeRam)));
            self.list_targets
                .push((next_preset_r, Action::VoltExtra(Item::VoltPreset)));

            if iy >= top - 4.0 && iy + hero_h <= bottom + 4.0 {
                ui.rounded(cx, iy, cw, hero_h, 12.0, theme::HERO_BG());
                ui.border(cx, iy, cw, hero_h, 12.0, theme::BORDER_CRIMSON());
                ui.text(
                    Face::LauncherMedium,
                    16.5,
                    cx + 16.0,
                    iy + 10.0,
                    theme::TEXT(),
                    "Volt Settings & Boost",
                );
                let hw_fit = ui.fit_text(Face::LauncherRegular, 12.0, &hw_summary, cw - 32.0);
                ui.text(Face::LauncherRegular, 12.0, cx + 16.0, iy + 34.0, theme::MUTED(), &hw_fit);
                styled_button(ui, self.targets, auto_r, "Auto-Tune", Action::VoltExtra(Item::VoltAutoOptimize), BtnStyle::Primary, true);
                styled_button(ui, self.targets, purge_r, "Purge RAM", Action::VoltExtra(Item::VoltPurgeRam), BtnStyle::CrimsonOutline, true);
                styled_button(
                    ui,
                    self.targets,
                    next_preset_r,
                    &format!("Profile: {} (Tap to Cycle)", ex.volt_preset.label()),
                    Action::VoltExtra(Item::VoltPreset),
                    BtnStyle::Secondary,
                    true,
                );
            }

            for (i, (label, val, act, sw, dropdown)) in all_items.iter().enumerate() {
                let y = top + hero_h + 12.0 + i as f32 * row_h - self.catalog.scroll;
                let ctrl_rect = [cx, y + 24.0, cw, 36.0];
                self.list_targets.push((ctrl_rect, *act));
                if y < top || y + row_h - 4.0 > bottom {
                    continue;
                }
                ui.text(Face::LauncherRegular, 13.5, cx + 2.0, y, theme::MUTED(), label);
                if let Some(on) = sw {
                    toggle(ui, self.targets, ctrl_rect, *on, *act);
                } else {
                    let text = if *dropdown {
                        format!("{val}   v")
                    } else {
                        val.clone()
                    };
                    styled_button(ui, self.targets, ctrl_rect, &text, *act, BtnStyle::Secondary, false);
                }
            }
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
        ui.rounded(x, y, w, h, 8.0, theme::CARD_INNER());
        ui.border(
            x,
            y,
            w,
            h,
            8.0,
            if focused {
                theme::CRIMSON_BRIGHT()
            } else {
                theme::BORDER()
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
            theme::DIM()
        } else {
            theme::TEXT()
        };
        let fitted = ui.fit_text(Face::LauncherMedium, 14.5, &display, (w - 24.0).max(16.0));
        ui.text(
            Face::LauncherMedium,
            14.5,
            x + 12.0,
            text_vcenter(y, h, 14.5),
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
        let header_h = if compact { 34.0 } else { 38.0 };
        ui.text(
            Face::LauncherMedium,
            if compact { 19.5 } else { 24.0 },
            cx + 4.0,
            cy + 2.0,
            theme::TEXT(),
            "Account & Custom Skin Wardrobe",
        );
        if !compact {
            let chip_w = (cw * 0.34).clamp(190.0, 300.0);
            draw_account_chip(
                ui,
                self.targets,
                [cx + cw - chip_w, cy, chip_w, 36.0],
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
            let fitted = ui.fit_text(Face::LauncherMedium, 12.5, status, cw);
            ui.text(
                Face::LauncherMedium,
                12.5,
                cx + 4.0,
                cy + ch - 18.0,
                if self.error.is_some() {
                    theme::CRIMSON_BRIGHT()
                } else {
                    theme::MUTED()
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
        ui.rounded(x, y, w, h, 14.0, theme::CARD());
        ui.rounded(
            x + w * 0.14,
            y + h * 0.14,
            w * 0.72,
            h * 0.58,
            16.0,
            theme::HERO_GLOW_SOFT(),
        );
        ui.border(x, y, w, h, 14.0, theme::BORDER_CRIMSON());

        let badge_w = (ui.measure(Face::LauncherMedium, 11.0, acc_badge) + 18.0).min(96.0);
        ui.rounded(x + 14.0, y + 12.0, badge_w, 22.0, 6.0, theme::CRIMSON_BADGE());
        ui.border(x + 14.0, y + 12.0, badge_w, 22.0, 6.0, theme::BORDER_CRIMSON());
        draw_centered_text(
            ui,
            Face::LauncherMedium,
            11.0,
            [x + 14.0, y + 12.0, badge_w, 22.0],
            theme::CRIMSON_BRIGHT(),
            acc_badge,
        );

        let nfit = ui.fit_text(
            Face::LauncherMedium,
            14.5,
            acc_name,
            (w - badge_w - 94.0).max(36.0),
        );
        ui.text(
            Face::LauncherMedium,
            14.5,
            x + 20.0 + badge_w,
            text_vcenter(y + 12.0, 22.0, 14.5),
            theme::TEXT(),
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
            self.preview_vel_yaw,
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
            Face::LauncherRegular,
            11.5,
            &format!("Skin: {skin_lbl}  •  Cape: {cape_lbl}"),
            w - 24.0,
        );
        draw_centered_text(
            ui,
            Face::LauncherRegular,
            11.5,
            [x + 12.0, y + h - 26.0, w - 24.0, 20.0],
            theme::MUTED(),
            &summary,
        );
    }

    fn draw_accounts_auth_card(&mut self, ui: &mut Ui, rect: [f32; 4]) {
        use crate::volt_accounts::AccountProvider;
        let [x, y, w, h] = rect;
        ui.rounded(x, y, w, h, 14.0, theme::CARD());
        ui.border(x, y, w, h, 14.0, theme::BORDER_CRIMSON());

        ui.text(
            Face::LauncherMedium,
            15.5,
            x + 16.0,
            y + 12.0,
            theme::TEXT(),
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
                    theme::CRIMSON_SOFT()
                } else {
                    theme::CARD_INNER()
                },
            );
            ui.border(
                row_rect[0],
                row_rect[1],
                row_rect[2],
                row_rect[3],
                8.0,
                if selected {
                    theme::CRIMSON()
                } else {
                    theme::BORDER()
                },
            );
            let tag = format!("[{}] {}", acc.provider.badge(), acc.name);
            let tfit = ui.fit_text(Face::LauncherMedium, 12.5, &tag, row_w - 20.0);
            ui.text(
                Face::LauncherMedium,
                12.5,
                row_rect[0] + 10.0,
                text_vcenter(row_rect[1], row_rect[3], 12.5),
                if selected {
                    theme::ON_CRIMSON()
                } else {
                    theme::TEXT()
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
        ui.rounded(x + 14.0, form_top, w - 28.0, 30.0, 8.0, theme::RAIL());
        ui.border(x + 14.0, form_top, w - 28.0, 30.0, 8.0, theme::BORDER());
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
                ui.rounded(trect[0], trect[1], trect[2], trect[3], 6.0, theme::CRIMSON());
            }
            draw_centered_text(
                ui,
                Face::LauncherMedium,
                12.0,
                trect,
                if active { theme::ON_CRIMSON() } else { theme::MUTED() },
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
                    Face::LauncherRegular,
                    11.5,
                    "Supports full offline skin & cape equipping + server sync.",
                    w - 28.0,
                );
                ui.text(Face::LauncherRegular, 11.5, x + 14.0, fy + 84.0, theme::MUTED(), &note);
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
                    Face::LauncherRegular,
                    12.0,
                    "Official Microsoft / Xbox Live login for Hypixel and online-mode Java servers using device code.",
                    w - 28.0,
                    3,
                );
                for (li, line) in lines.iter().enumerate() {
                    ui.text(
                        Face::LauncherRegular,
                        12.0,
                        x + 14.0,
                        fy + li as f32 * 17.0,
                        theme::MUTED(),
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
        ui.rounded(x, y, w, h, 14.0, theme::CARD());
        ui.border(x, y, w, h, 14.0, theme::BORDER_CRIMSON());

        ui.text(
            Face::LauncherMedium,
            15.5,
            x + 16.0,
            y + 12.0,
            theme::TEXT(),
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
        let sync_fit = ui.fit_text(Face::LauncherRegular, 12.0, sync_sub, (w - 86.0).max(60.0));
        ui.text(Face::LauncherRegular, 12.0, x + 16.0, y + 38.0, theme::MUTED(), &sync_fit);
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
            Face::LauncherMedium,
            12.5,
            x + 16.0,
            y + 106.0,
            theme::CRIMSON_BRIGHT(),
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
            Face::LauncherMedium,
            12.5,
            x + 16.0,
            y + 198.0,
            theme::CRIMSON_BRIGHT(),
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
    pub(super) fn is_animating(&self) -> bool {
        let page_idx = Page::ALL
            .iter()
            .position(|&p| p == self.catalog.page)
            .unwrap_or(0) as f32;
        if (self.sidebar_anim - page_idx).abs() > 0.005 {
            return true;
        }
        if self
            .page_anim
            .is_some_and(|(ts, _)| ts.elapsed().as_secs_f32() < 0.28)
        {
            return true;
        }
        if self
            .modal_anim
            .is_some_and(|ts| ts.elapsed().as_secs_f32() < 0.24)
        {
            return true;
        }
        if self
            .feedback
            .is_some_and(|(_, ts)| ts.elapsed().as_secs_f32() < 0.24)
        {
            return true;
        }
        if self.preview_target_yaw.is_some() || self.preview_vel_yaw.abs() > 0.4 {
            return true;
        }
        if self.worker.is_some()
            || self.catalog.animating_icons_or_cards()
            || self.update_prompt.as_ref().is_some_and(|p| {
                matches!(
                    p.state,
                    crate::volt_updater::UpdateModalState::Downloading { .. }
                )
            })
        {
            return true;
        }
        has_active_toggle_anims()
    }

    fn tick_animations(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().clamp(0.0, 0.08);
        self.last_frame = now;

        // Detect page changes from any trigger (sidebar tap, character tap, controller, install completion)
        if self.catalog.page != self.prev_rendered_page {
            let old_idx = Page::ALL
                .iter()
                .position(|&p| p == self.prev_rendered_page)
                .unwrap_or(0) as f32;
            let new_idx = Page::ALL
                .iter()
                .position(|&p| p == self.catalog.page)
                .unwrap_or(0) as f32;
            let dir = if new_idx >= old_idx { 1.0 } else { -1.0 };
            self.page_anim = Some((now, dir));
            self.prev_rendered_page = self.catalog.page;
        }

        // Smoothly glide sidebar / dock active selection pill
        let target_idx = Page::ALL
            .iter()
            .position(|&p| p == self.catalog.page)
            .unwrap_or(0) as f32;
        let alpha = 1.0 - (-dt * 18.0).exp();
        self.sidebar_anim += (target_idx - self.sidebar_anim) * alpha;
        if (target_idx - self.sidebar_anim).abs() < 0.003 {
            self.sidebar_anim = target_idx;
        }

        // Detect modal pop-in transitions (Chooser / Microsoft Login / Update Prompt / Volt Boost)
        let modal_open = self.chooser.is_some()
            || self.login.visible
            || self.update_prompt.is_some()
            || self.catalog.volt_boost_modal
            || self.controls_studio.open;
        if modal_open && !self.modal_was_open {
            self.modal_anim = Some(now);
        }
        self.modal_was_open = modal_open;

        // Smooth 3D character Flip target + rotational inertia
        let dragging_preview = (self.mouse_down || self.drag.is_some())
            && self.drag.as_ref().is_some_and(|d| d.moved);
        if let Some(target_yaw) = self.preview_target_yaw {
            let mut diff = (target_yaw - self.preview_yaw + 180.0).rem_euclid(360.0) - 180.0;
            if diff.abs() < 0.4 {
                self.preview_yaw = target_yaw.rem_euclid(360.0);
                self.preview_vel_yaw = 0.0;
                self.preview_target_yaw = None;
            } else {
                let step_alpha = 1.0 - (-dt * 11.5).exp();
                let delta = diff * step_alpha;
                self.preview_yaw = (self.preview_yaw + delta).rem_euclid(360.0);
                self.preview_vel_yaw = (delta / dt.max(0.004)).clamp(-320.0, 320.0);
                diff -= delta;
                if diff.abs() < 0.4 {
                    self.preview_yaw = target_yaw.rem_euclid(360.0);
                    self.preview_target_yaw = None;
                }
            }
        } else if !dragging_preview {
            if self.preview_vel_yaw.abs() > 0.3 {
                self.preview_yaw =
                    (self.preview_yaw + self.preview_vel_yaw * dt).rem_euclid(360.0);
                self.preview_vel_yaw *= (-dt * 6.0).exp();
            } else {
                self.preview_vel_yaw = 0.0;
            }
        }
    }

    pub(super) fn draw_pages(&mut self) -> Result<()> {
        self.tick_animations();
        theme::set(self.catalog.config.launcher_dark, self.catalog.config.launcher_accent);
        ACTIVE_FEEDBACK.with(|fb| *fb.borrow_mut() = self.feedback);
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
        // Draw the scene across the surface; controls use the visible window
        // rect so temporarily visible system bars never cover buttons.
        let full_w = ui.w;
        let full_h = ui.h;
        let scale = ui.density / ui.batch.scale;
        ui.batch.fill(&gui.atlas, 0.0, 0.0, full_w * scale, full_h * scale, theme::BG());
        draw_launcher_scene(&mut ui, full_w, full_h);
        let scene_end = ui.batch_mut().vertices.len();
        let mut offset = [0.0f32; 2];
        if let Some([left, top, right, bottom]) = crate::native_ime::launcher_visible_bounds() {
            let density = ui.density;
            let x0 = (left as f32 / density).min(full_w);
            let y0 = (top as f32 / density).min(full_h);
            let x1 = (right as f32 / density).min(full_w);
            let y1 = (bottom as f32 / density).min(full_h);
            if x1 - x0 >= 120.0 && y1 - y0 >= 120.0 {
                offset = [x0, y0];
                ui.w = x1 - x0;
                ui.h = y1 - y0;
            }
        }
        let (w, h) = (ui.w, ui.h);
        ui.cursor = (self.cursor.0 as f32 / ui.density - offset[0],
                     self.cursor.1 as f32 / ui.density - offset[1]);

        let compact = w < 760.0;
        let account_status = self
            .volt_accounts
            .active()
            .map(|a| format!("{} • {}", a.name, a.provider.badge()))
            .unwrap_or_else(|| self.login.status(&self.catalog.config.name));
        let idle_phase = self.launch_start.elapsed().as_secs_f32();
        let content_area;
        let page_v_start;
        {
            let mut ctx = DrawCtx {
                home: &self.home,
                install: &self.install,
                worker: &self.worker,
                error: &self.error,
                catalog: &mut self.catalog,
                home_tab: self.home_tab,
                targets: &mut self.targets,
                list_targets: &mut self.list_targets,
                list_view: &mut self.list_view,
                login: &self.login,
                volt_accounts: &self.volt_accounts,
                preview_yaw: self.preview_yaw,
                preview_pitch: self.preview_pitch,
                preview_vel_yaw: self.preview_vel_yaw,
                sidebar_anim: self.sidebar_anim,
                idle_phase,
                preview_rect: &mut self.preview_rect,
            };
            content_area = if compact {
                ctx.draw_compact_header(&mut ui, w, &account_status);
                [12.0, 102.0, (w - 24.0).max(120.0), (h - 148.0).max(120.0)]
            } else {
                let rail_w = 196.0;
                ctx.draw_sidebar(&mut ui, 12.0, 12.0, rail_w, (h - 24.0).max(260.0));
                let x = 12.0 + rail_w + 16.0;
                let cw = (w - rail_w - 40.0).max(240.0);
                ctx.draw_topbar(&mut ui, [x, 14.0, cw, 38.0], &account_status);
                [x, 66.0, cw, (h - 80.0).max(120.0)]
            };

            page_v_start = ui.batch_mut().vertices.len();
            let page = ctx.catalog.page;
            match page {
                Page::Play | Page::Mods => ctx.draw_instance_page(&mut ui, content_area, compact, &account_status),
                Page::Account => {
                    ctx.draw_account_page(&mut ui, content_area, compact, &account_status)
                }
                Page::Packs | Page::Shaders => {
                    ctx.draw_catalog_page(&mut ui, content_area, compact, page, &account_status)
                }
                Page::Installed => {
                    ctx.draw_installed_page(&mut ui, content_area, compact, &account_status)
                }
                Page::VoltSettings => ctx.draw_volt_settings_page(&mut ui, content_area, compact),
                Page::Settings => {
                    ctx.draw_settings_page(&mut ui, content_area, compact, &account_status)
                }
            }
        }
        if let Some((anim_start, dir)) = self.page_anim {
            let el = anim_start.elapsed().as_secs_f32();
            if el < 0.25 {
                let ease = ease_out_cubic(el / 0.25);
                ui.animate_vertices(
                    page_v_start,
                    0.0,
                    dir * (1.0 - ease) * 18.0,
                    0.985 + 0.015 * ease,
                    [
                        content_area[0] + content_area[2] * 0.5,
                        content_area[1] + content_area[3] * 0.5,
                    ],
                    ease,
                );
            } else {
                self.page_anim = None;
            }
        }

        if compact {
            let status = self.error.as_deref().unwrap_or(&self.catalog.status);
            if !status.is_empty() {
                let sfit = ui.fit_text(Face::LauncherRegular, 12.0, status, (w - 196.0).max(40.0));
                ui.text(Face::LauncherRegular, 12.0, 14.0, h - 32.0, theme::MUTED(), &sfit);
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

        let modal_v_start = ui.batch_mut().vertices.len();
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
        } else if !self.login.visible
            && let Some(prompt) = &mut self.update_prompt
        {
            self.targets.clear();
            self.list_targets.clear();
            self.list_view = None;
            draw_update_prompt(&mut ui, &mut self.targets, prompt);
        } else if !self.login.visible && self.controls_studio.open {
            self.targets.clear();
            self.list_targets.clear();
            self.list_view = None;
            self.controls_studio.draw(&mut ui, &mut self.targets);
        } else if !self.login.visible && self.catalog.volt_boost_modal {
            self.targets.clear();
            self.list_targets.clear();
            self.list_view = draw_volt_boost_modal(
                &mut ui,
                &mut self.targets,
                &mut self.list_targets,
                &mut self.catalog,
            );
        }
        if (self.login.visible
            || self.chooser.is_some()
            || self.update_prompt.is_some()
            || self.catalog.volt_boost_modal
            || self.controls_studio.open)
            && let Some(anim_start) = self.modal_anim
        {
            let el = anim_start.elapsed().as_secs_f32();
            if el < 0.20 {
                let ease = ease_out_cubic(el / 0.20);
                // Fade backdrop smoothly while keeping full-bleed bounds; pop-scale modal card!
                let backdrop_end = (modal_v_start + 6).min(ui.batch_mut().vertices.len());
                for v in &mut ui.batch_mut().vertices[modal_v_start..backdrop_end] {
                    let base_a = (v.color >> 24) as f32;
                    let new_a = (base_a * ease).round().clamp(0.0, 255.0) as u32;
                    v.color = (v.color & 0x00ff_ffff) | (new_a << 24);
                }
                ui.animate_vertices(
                    backdrop_end,
                    0.0,
                    (1.0 - ease) * 14.0,
                    0.92 + 0.08 * ease,
                    [w * 0.5, h * 0.5],
                    ease,
                );
            } else {
                self.modal_anim = None;
            }
        }
        if let Some([top, bottom, maximum]) = self.list_view
            && maximum > 0.0
            && bottom > top
        {
            let scroll = if self.catalog.volt_boost_modal {
                self.catalog.volt_modal_scroll
            } else {
                self.chooser
                    .as_ref()
                    .map_or(self.catalog.scroll, |c| c.scroll)
            };
            let height = bottom - top;
            let thumb = (height * height / (height + maximum)).max(20.0).min(height);
            let y = top + (height - thumb) * (scroll / maximum).clamp(0.0, 1.0);
            ui.rounded(w - 8.0, top, 4.0, height, 2.0, theme::BORDER());
            ui.rounded(w - 8.0, y, 4.0, thumb, 2.0, theme::CRIMSON());
        }
        let caret_visible = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .is_ok_and(|t| t.as_millis() % 1000 < 500);
        if caret_visible && let Some(name) = self.catalog.editing {
            let (value, font_sz, pad_x) = match name {
                "name" => (self.catalog.config.name.clone(), 15.5, 14.0),
                "server" => (self.catalog.config.server.clone(), 15.5, 12.0),
                "render_scale" => (self.catalog.render_scale_text.clone(), 15.5, 12.0),
                "offline_name" => (self.catalog.offline_name_input.clone(), 14.5, 12.0),
                "ely_user" => (self.catalog.ely_user_input.clone(), 14.5, 12.0),
                "ely_pass" => (
                    "*".repeat(self.catalog.ely_pass_input.chars().count()),
                    14.5,
                    12.0,
                ),
                "ely_totp" => (self.catalog.ely_totp_input.clone(), 14.5, 12.0),
                "skin_query" => (self.catalog.skin_query_input.clone(), 14.5, 12.0),
                _ => (self.catalog.query.clone(), 14.5, 14.0),
            };
            let face = if name == "search" {
                Face::LauncherRegular
            } else {
                Face::LauncherMedium
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
                        theme::CRIMSON_BRIGHT(),
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
                theme::CRIMSON_BRIGHT(),
            );
            self.cursor = (
                f64::from((x + bw / 2.0 + offset[0]) * ui.density),
                f64::from((y + bh / 2.0 + offset[1]) * ui.density),
            );
        }
        if self.mouse_down || self.drag.as_ref().is_some_and(|drag| !drag.moved) {
            let point = [
                self.cursor.0 as f32 / ui.density - offset[0],
                self.cursor.1 as f32 / ui.density - offset[1],
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
                    horizon_client::hud::rgba(0, 155, 230, 44),
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
                horizon_client::hud::rgba(0, 155, 230, 44),
            );
        }
        crate::documents::draw_ui(&mut ui);
        // Geometry and touch targets share the same translation. The scene is
        // deliberately excluded so it still covers the whole Android surface.
        if offset != [0.0, 0.0] {
            let dx = ui.to_batch(offset[0]);
            let dy = ui.to_batch(offset[1]);
            ui.batch_mut().translate_since(scene_end, dx, dy);
            for (rect, _) in self.targets.iter_mut().chain(self.list_targets.iter_mut()) {
                rect[0] += offset[0];
                rect[1] += offset[1];
            }
            if let Some(rect) = &mut self.preview_rect {
                rect[0] += offset[0];
                rect[1] += offset[1];
            }
            if let Some(view) = &mut self.list_view {
                view[0] += offset[1];
                view[1] += offset[1];
            }
        }
        ui.finish();

        if !r.render(&game::loading_frame(&b))? && !b.patches.is_empty() {
            // A suspended/lost surface skips the frame, including its glyph
            // uploads. Rebuild them next time instead of caching invisible text.
            horizon_client::ui_theme::reset_atlas();
        }
        Ok(())
    }

    pub(super) fn activate(&mut self, _events: &ActiveEventLoop) -> Result<()> {
        if self.session_worker.is_some() || self.pending_play {
            return Ok(());
        }
        let Some(r) = &self.renderer else {
            return Ok(());
        };
        let Some(gui) = &self.gui else {
            return Ok(());
        };
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        let ui = Ui::mobile(&gui.atlas, &mut b, (0.0, 0.0));
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
        let now = std::time::Instant::now();
        if let Some(act) = action
            && !matches!(act, Action::Page(_))
            && self.transition_guard_until.is_some_and(|until| now < until)
        {
            self.focus = None;
            return Ok(());
        }
        let prev_view = (
            self.catalog.page,
            self.chooser.is_some(),
            self.login.visible,
            self.catalog.volt_boost_modal,
            self.controls_studio.open,
            self.update_prompt.is_some(),
        );
        self.feedback = action.map(|action| (action, now));
        self.focus = None;
        if action.is_some() {
            crate::native_ime::play_click_sound();
        }
        match action {
            Some(Action::ToggleAppearance) => {
                self.catalog.config.launcher_dark = !self.catalog.config.launcher_dark;
                self.catalog.config.save(&self.home)?;
            }
            Some(Action::CycleAccent) => {
                const COLORS: [u32; 8] = [0x6b4bff, 0x356cf6, 0x0e9fbd, 0x18a977, 0xe6ab32, 0xf26c5b, 0xd755ae, 0x8b5cf6];
                let current = self.catalog.config.launcher_accent;
                let index = COLORS.iter().position(|&c| c == current).map_or(0, |i| (i + 1) % COLORS.len());
                self.catalog.config.launcher_accent = COLORS[index];
                self.catalog.config.save(&self.home)?;
            }
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
                self.page_anim = Some((std::time::Instant::now(), 0.45));
                self.catalog.browse(&self.home);
            }
            Some(Action::ShaderSource(origin)) => {
                self.catalog.config.shader_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.page_anim = Some((std::time::Instant::now(), 0.45));
                self.catalog.browse(&self.home);
            }
            Some(Action::PackSource(origin)) => {
                self.catalog.config.pack_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.page_anim = Some((std::time::Instant::now(), 0.45));
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
                self.page_anim = Some((std::time::Instant::now(), 0.40));
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
                        self.login.visible = false;
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
                    // Smoothly spin character toward back so user watches their 3D cape billow into view!
                    self.preview_target_yaw = Some(162.0);
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
                let target = if normalized.abs() < 90.0 { 162.0 } else { 342.0 };
                self.preview_target_yaw = Some(target);
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
                if self.catalog.page == Page::Account {
                    self.login.visible = false;
                }
                self.volt_accounts
                    .sync_msa_accounts(&crate::account::Accounts::load(), false);
            }
            Some(Action::LoginLink) => self.login.open_link(),
            Some(Action::HomeTab(tab)) => {
                if tab == HomeTab::Multiplayer {
                    let home = self.catalog.config.game_home(&self.home);
                    self.catalog.saved_servers = crate::server_list::List::load(&home).servers();
                } else if tab == HomeTab::Screenshots {
                    self.catalog.refresh_screenshots(&self.home);
                }
                self.home_tab = tab;
                self.catalog.page = Page::Play;
                self.catalog.scroll = 0.0;
                self.catalog.editing = None;
                if crate::platform::active() { crate::platform::hide(); }
            }
            Some(Action::Page(page)) => {
                if page == Page::Play { self.home_tab = HomeTab::Overview; }
                self.catalog.volt_boost_modal = false;
                self.controls_studio.open = false;
                self.catalog.editing = None;
                if crate::platform::active() {
                    crate::platform::hide();
                }
                self.catalog.page = page;
                self.catalog.scroll = 0.0;
                self.error = None;
                self.catalog.offset = 0;
                if page == Page::Mods {
                    self.catalog.query.clear();
                    self.catalog.mod_born = std::time::Instant::now();
                }
                self.catalog.browse(&self.home);
            }
            Some(Action::SaveServer) => {
                let home = self.catalog.config.game_home(&self.home);
                let address = self.catalog.config.server.trim().to_owned();
                let result = crate::server_list::List::try_load(&home).and_then(|mut list| {
                    if list.servers().iter().any(|server| server.ip.eq_ignore_ascii_case(&address)) {
                        return Err("This server is already saved.".into());
                    }
                    list.add(&address, &address)?;
                    list.save(&home)
                });
                self.catalog.status = match result {
                    Ok(()) => {
                        self.catalog.saved_servers = crate::server_list::List::load(&home).servers();
                        "Server saved to this instance.".into()
                    }
                    Err(error) => error,
                };
            }
            Some(Action::ConnectServer(index)) => {
                if let Some(server) = self.catalog.saved_servers.get(index) {
                    self.catalog.config.server = server.ip.clone();
                    self.catalog.config.save(&self.home)?;
                    self.pending_play = true;
                    self.needs_redraw = true;
                } else {
                    self.catalog.status = "Server list changed; please choose again.".into();
                }
            }
            Some(Action::Play) => {
                self.pending_play = true;
                self.needs_redraw = true;
            }
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
                horizon_client::android_runtime::configure_graphics_api(mode.graphics_api());
                self.chooser = None;
            }
            Some(Action::ImportVulkanDriver) => {
                if let Err(err) = crate::documents::begin_vulkan_driver(self.home.clone()) {
                    self.catalog.status = err;
                }
            }
            Some(Action::ResetVulkanDriver) => {
                self.catalog.config.custom_vulkan_driver = None;
                self.catalog.config.custom_vulkan_driver_name = None;
                self.catalog.config.save(&self.home)?;
                horizon_client::android_runtime::configure_custom_vulkan_driver(None);
                self.catalog.status =
                    "Switched back to Volt's built-in Vulkan Supporter driver.".into();
            }
            Some(Action::InstallUpdate) => {
                if let Some(prompt) = &mut self.update_prompt
                    && !matches!(
                        prompt.state,
                        crate::volt_updater::UpdateModalState::Downloading { .. }
                    )
                {
                    prompt.state = crate::volt_updater::begin_download(&self.home, &prompt.info);
                }
            }
            Some(Action::CancelUpdate) => {
                if let Some(prompt) = self.update_prompt.take() {
                    if let crate::volt_updater::UpdateModalState::Downloading { cancel, .. } =
                        &prompt.state
                    {
                        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                    crate::volt_updater::mark_update_seen(&self.home, &prompt.info.tag_name);
                }
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
                if self.home_tab == HomeTab::Multiplayer {
                    let home = self.catalog.config.game_home(&self.home);
                    self.catalog.saved_servers = crate::server_list::List::load(&home).servers();
                } else if self.home_tab == HomeTab::Screenshots {
                    self.catalog.refresh_screenshots(&self.home);
                }
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
                if let Err(err) = std::thread::Builder::new()
                    .name("volt-diag-export".into())
                    .stack_size(4 * 1024 * 1024)
                    .spawn(move || {
                        let result = crate::diagnostics::export(&home).map(|p| {
                            p.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned()
                        });
                        let _ = sender.send(result);
                    })
                {
                    self.diagnostic_export = None;
                    self.catalog.status = format!("Diagnostic export failed: {err}");
                }
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
            Some(Action::OpenVoltBoost) => {
                self.catalog.volt_boost_modal = true;
                self.catalog.volt_modal_scroll = 0.0;
            }
            Some(Action::CloseVoltBoost) => {
                self.catalog.volt_boost_modal = false;
            }
            Some(Action::VoltPreset(preset)) => {
                self.catalog.extras.apply_volt_preset(preset);
                self.catalog.extras.distant_lod_boost = false;
                horizon_threads::set_thread_boost(self.catalog.extras.thread_boost);
                self.configure_screen();
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                self.catalog.extras.save(&game_dir);
                let mut gfx = horizon_client::graphics::Graphics::load(&game_dir);
                if gfx.lod_distance != 0 {
                    gfx.lod_distance = 0;
                    gfx.save(&game_dir);
                }
                self.catalog.status = format!(
                    "Volt Boost profile set to {} ({} optimizations active).",
                    preset.label(),
                    self.catalog.extras.active_volt_optimizations()
                );
            }
            Some(Action::VoltExtra(item)) => {
                item.cycle(&mut self.catalog.extras, false);
                self.catalog.extras.purge_ram_requested = false;
                horizon_threads::set_thread_boost(self.catalog.extras.thread_boost);
                self.configure_screen();
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                self.catalog.extras.save(&game_dir);
                if matches!(
                    item,
                    horizon_client::extras::Item::VoltPreset
                        | horizon_client::extras::Item::VoltAutoOptimize
                ) {
                    let mut gfx = horizon_client::graphics::Graphics::load(&game_dir);
                    if gfx.lod_distance != 0 {
                        gfx.lod_distance = 0;
                        gfx.save(&game_dir);
                    }
                }
                if item == horizon_client::extras::Item::VoltPurgeRam {
                    let rss = horizon_threads::resident_ram_mib().unwrap_or(0);
                    self.catalog.status =
                        format!("Purged unused allocator pages (Resident RAM: {rss} MiB).");
                } else if item == horizon_client::extras::Item::VoltAutoOptimize {
                    self.catalog.status = format!(
                        "Auto-tuned device to {} ({} optimizations active).",
                        self.catalog.extras.volt_preset.label(),
                        self.catalog.extras.active_volt_optimizations()
                    );
                }
            }
            Some(Action::RemoveBundled(i)) => self.catalog.remove_bundled(&self.home, i),
            Some(Action::HorizonModBrowser(show)) => {
                self.catalog.horizon_mod_browser = show;
                self.catalog.query.clear();
                self.catalog.offset = 0;
                self.catalog.scroll = 0.0;
                self.catalog.results.clear();
                self.catalog.editing = None;
                if crate::platform::active() { crate::platform::hide(); }
                self.catalog.browse(&self.home);
            }
            Some(Action::InstallBundledZoom) => {
                let game_dir = self.catalog.config.game_home(&self.home);
                let install = !self.catalog.bundled_zoom_installed;
                match horizon_client::bundled_native::set_zoom_installed(&game_dir, install) {
                    Ok(()) => {
                        self.catalog.bundled_zoom_installed = install;
                        self.catalog.bundled_zoom_enabled = install;
                        if install { self.catalog.bundled_mods.insert("zoom".into()); }
                        else { self.catalog.bundled_mods.remove("zoom"); }
                        self.catalog.status = if install {
                            "Horizon Zoom installed from the APK. Hold Z; add a Z button in Controls Studio. Settings: in-game Mods.".into()
                        } else { "Horizon Zoom removed from this profile; its settings are kept.".into() };
                    }
                    Err(e) => self.catalog.status = format!("Could not change Horizon Zoom: {e}"),
                }
            }
            Some(Action::ToggleVoltMod(id)) => {
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                let mut gfx = horizon_client::graphics::Graphics::load(&game_dir);
                let next = !self.catalog.extras.mod_catalog.is_on(id);
                horizon_client::volt_mods::apply_mod_to_engine(
                    id,
                    next,
                    &mut self.catalog.extras,
                    &mut gfx,
                );
                self.catalog.extras.save(&game_dir);
                gfx.save(&game_dir);
                self.catalog.status = format!(
                    "{} {} ({}/{} catalog switches on).",
                    id.meta().title,
                    if next { "enabled" } else { "disabled" },
                    self.catalog.extras.mod_catalog.enabled_count(),
                    horizon_client::volt_mods::TOTAL_VOLT_MODS
                );
            }
            Some(Action::ModFilter(code)) => {
                self.catalog.mod_filter = code;
                self.catalog.scroll = 0.0;
                self.catalog.mod_born = std::time::Instant::now();
            }
            Some(Action::ClearModSearch) => {
                self.catalog.query.clear();
                self.catalog.editing = None;
                if crate::platform::active() {
                    crate::platform::hide();
                }
                self.catalog.scroll = 0.0;
            }
            Some(Action::EnableModPack) => {
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                let mut gfx = horizon_client::graphics::Graphics::load(&game_dir);
                for m in &horizon_client::volt_mods::CATALOG {
                    if matches!(
                        m.category,
                        horizon_client::volt_mods::VoltModCategory::Performance
                            | horizon_client::volt_mods::VoltModCategory::PvpUtility
                            | horizon_client::volt_mods::VoltModCategory::PvpHud
                    ) && m.id != horizon_client::volt_mods::VoltModId::InventoryHudPlus
                    {
                        horizon_client::volt_mods::apply_mod_to_engine(
                            m.id,
                            true,
                            &mut self.catalog.extras,
                            &mut gfx,
                        );
                    }
                }
                self.catalog.extras.save(&game_dir);
                gfx.save(&game_dir);
                self.catalog.status = format!(
                    "Enabled PvP + Performance switches ({}/{} catalog switches on).",
                    self.catalog.extras.mod_catalog.enabled_count(),
                    horizon_client::volt_mods::TOTAL_VOLT_MODS
                );
            }
            Some(Action::ResetModDefaults) => {
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                let mut gfx = horizon_client::graphics::Graphics::load(&game_dir);
                self.catalog.extras.mod_catalog =
                    horizon_client::volt_mods::VoltModCatalogState::default();
                for m in &horizon_client::volt_mods::CATALOG {
                    horizon_client::volt_mods::apply_mod_to_engine(
                        m.id,
                        m.default_on,
                        &mut self.catalog.extras,
                        &mut gfx,
                    );
                }
                self.catalog.extras.save(&game_dir);
                gfx.save(&game_dir);
                self.catalog.status = format!(
                    "Reset native mods to default ({}/{} active).",
                    self.catalog.extras.mod_catalog.enabled_count(),
                    horizon_client::volt_mods::TOTAL_VOLT_MODS
                );
            }
            Some(Action::Studio(sact)) => {
                let profile_home = self.catalog.config.game_home(&self.home);
                if let Some(msg) =
                    self.controls_studio
                        .handle_action(sact, &self.home, &profile_home)
                {
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
                    self.catalog.status = msg;
                } else if matches!(sact, super::controls_studio::StudioAction::SavePreset) {
                    self.catalog.refresh_controls(&self.home);
                }
            }
            None => {}
        }
        let next_view = (
            self.catalog.page,
            self.chooser.is_some(),
            self.login.visible,
            self.catalog.volt_boost_modal,
            self.controls_studio.open,
            self.update_prompt.is_some(),
        );
        if next_view != prev_view {
            self.targets.clear();
            self.list_targets.clear();
            self.drag = None;
            self.mouse_down = false;
            self.pending_activate = None;
            self.transition_guard_until =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(180));
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

    pub(super) fn touch_event(&mut self, _events: &ActiveEventLoop, t: Touch) -> Result<()> {
        self.focus = None;
        self.pending_focus = None;
        self.needs_redraw = true;
        match t.phase {
            TouchPhase::Started => {
                if self.drag.is_none() {
                    self.cursor = (t.location.x, t.location.y);
                    if self.controls_studio.open {
                        let density = self.ui_density() as f64;
                        let _ = self.controls_studio.pointer_down(
                            (t.location.x / density) as f32,
                            (t.location.y / density) as f32,
                        );
                    }
                    self.drag = Some(super::gesture::Drag::with_id(
                        t.id,
                        t.location.x,
                        t.location.y,
                    ));
                }
            }
            TouchPhase::Moved => {
                if self
                    .drag
                    .as_ref()
                    .is_none_or(|d| d.pointer_id != Some(t.id))
                {
                    return Ok(());
                }
                self.cursor = (t.location.x, t.location.y);
                let density = self.ui_density() as f64;
                if self.controls_studio.open
                    && self.controls_studio.pointer_move(
                        (t.location.x / density) as f32,
                        (t.location.y / density) as f32,
                    )
                {
                    if let Some(drag) = &mut self.drag {
                        let _ = drag.update_xy(t.location.x, t.location.y, density);
                    }
                    return Ok(());
                }
                if let Some(drag) = &mut self.drag {
                    let (dx, dy) = drag.update_xy(t.location.x, t.location.y, density);
                    if drag.moved {
                        let [ox, oy] = drag.origin_xy();
                        let ux = (ox / density) as f32;
                        let uy = (oy / density) as f32;
                        let in_preview = !self.login.visible
                            && self.chooser.is_none()
                            && self.update_prompt.is_none()
                            && !self.catalog.volt_boost_modal
                            && !self.controls_studio.open
                            && self.preview_rect.is_some_and(|[rx, ry, rw, rh]| {
                                ux >= rx && ux <= rx + rw && uy >= ry && uy <= ry + rh
                            });
                        if in_preview {
                            self.preview_target_yaw = None;
                            self.preview_yaw =
                                (self.preview_yaw + dx * 0.9).rem_euclid(360.0);
                            self.preview_vel_yaw = (dx * 42.0).clamp(-360.0, 360.0);
                            self.preview_pitch =
                                (self.preview_pitch - dy * 0.45).clamp(-28.0, 28.0);
                        } else if let Some(prompt) = &mut self.update_prompt {
                            prompt.scroll = (prompt.scroll + dy).max(0.0);
                        } else if self.catalog.volt_boost_modal {
                            self.catalog.volt_modal_scroll =
                                (self.catalog.volt_modal_scroll + dy).max(0.0);
                        } else if let Some(chooser) = &mut self.chooser {
                            chooser.scroll = (chooser.scroll + dy).max(0.0);
                        } else if !self.controls_studio.open
                            && self
                                .list_view
                                .is_some_and(|view| oy / density >= view[0] as f64)
                        {
                            self.catalog.scroll = (self.catalog.scroll + dy).max(0.0);
                        }
                    }
                }
            }
            TouchPhase::Ended => {
                if self
                    .drag
                    .as_ref()
                    .is_some_and(|d| d.pointer_id == Some(t.id))
                {
                    let studio_dragged = self.controls_studio.pointer_up();
                    self.cursor = (t.location.x, t.location.y);
                    if let Some(drag) = self.drag.take()
                        && !drag.moved
                        && !studio_dragged
                    {
                        self.request_activate();
                    }
                }
            }
            TouchPhase::Cancelled => {
                if self
                    .drag
                    .as_ref()
                    .is_some_and(|d| d.pointer_id == Some(t.id) || d.pointer_id.is_none())
                {
                    let _ = self.controls_studio.pointer_up();
                    self.drag = None;
                }
            }
        }
        Ok(())
    }

    pub(super) fn handle_escape(&mut self) {
        if let Some(prompt) = self.update_prompt.take() {
            if let crate::volt_updater::UpdateModalState::Downloading { cancel, .. } =
                &prompt.state
            {
                cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            crate::volt_updater::mark_update_seen(&self.home, &prompt.info.tag_name);
            return;
        }
        if self.login.visible {
            self.login.hide();
            crate::platform::hide();
            self.catalog.editing = None;
            return;
        }
        if self.controls_studio.open {
            if self.controls_studio.bind_picker_open {
                self.controls_studio.bind_picker_open = false;
            } else {
                self.controls_studio.open = false;
                self.controls_studio.dragging = None;
                self.controls_studio.editing_label = false;
                crate::platform::hide();
            }
            return;
        }
        if self.catalog.volt_boost_modal {
            self.catalog.volt_boost_modal = false;
            return;
        }
        if self.chooser.take().is_some() {
            return;
        }
        crate::platform::hide();
        self.catalog.editing = None;
        let _ = self.catalog.config.save(&self.home);
    }

    pub(super) fn type_key(&mut self, key: KeyEvent) {
        if key.physical_key == PhysicalKey::Code(KeyCode::Escape)
            || key.logical_key
                == winit::keyboard::Key::Named(winit::keyboard::NamedKey::BrowserBack)
        {
            self.pending_escape = true;
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

fn wrap_modal_lines(ui: &Ui, lines: &[String], font_sz: f32, max_w: f32) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for raw in lines {
        let is_bullet = raw.starts_with("• ");
        let mut rest = raw.as_str();
        let mut first = true;
        while !rest.is_empty() {
            let prefix = if !first && is_bullet { "   " } else { "" };
            let avail = (max_w - if !first && is_bullet { 14.0 } else { 0.0 }).max(80.0);
            let fitted = ui.fit_text(Face::LauncherRegular, font_sz, rest, avail);
            if fitted == rest || !fitted.ends_with("...") {
                out.push((format!("{prefix}{rest}"), is_bullet && first));
                break;
            }
            let min_step = rest.chars().next().map_or(1, char::len_utf8);
            let mut cut_target = fitted
                .trim_end_matches("...")
                .len()
                .max(min_step)
                .min(rest.len());
            while cut_target < rest.len() && !rest.is_char_boundary(cut_target) {
                cut_target += 1;
            }
            let split_at = rest[..cut_target]
                .rfind(' ')
                .filter(|&i| i > 8)
                .unwrap_or(cut_target)
                .max(min_step);
            let head = rest[..split_at].trim_end();
            out.push((format!("{prefix}{head}"), is_bullet && first));
            rest = rest[split_at..].trim_start();
            first = false;
        }
    }
    out
}

fn draw_update_prompt(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    prompt: &mut crate::volt_updater::UpdatePrompt,
) {
    let (w, h) = (ui.w, ui.h);
    ui.rounded(0.0, 0.0, w, h, 0.0, theme::BACKDROP());
    let mw = (w - 44.0).clamp(300.0, 580.0);
    let mh = (h - 36.0).clamp(250.0, 396.0);
    let mx = (w - mw) * 0.5;
    let my = (h - mh) * 0.5;

    ui.rounded(mx, my, mw, mh, 16.0, theme::CARD());
    ui.rounded(mx + 6.0, my + 6.0, mw - 12.0, 62.0, 12.0, theme::HERO_BG());
    ui.border(mx, my, mw, mh, 16.0, theme::BORDER_CRIMSON());

    ui.text(
        Face::LauncherMedium,
        20.0,
        mx + 22.0,
        my + 15.0,
        theme::TEXT(),
        "Update Available",
    );

    let ver_badge_w = 82.0;
    let ver_badge_x = mx + mw - ver_badge_w - 20.0;
    ui.rounded(
        ver_badge_x,
        my + 15.0,
        ver_badge_w,
        24.0,
        7.0,
        theme::CRIMSON_BADGE(),
    );
    ui.border(
        ver_badge_x,
        my + 15.0,
        ver_badge_w,
        24.0,
        7.0,
        theme::BORDER_CRIMSON(),
    );
    draw_centered_text(
        ui,
        Face::LauncherMedium,
        12.5,
        [ver_badge_x, my + 15.0, ver_badge_w, 24.0],
        theme::CRIMSON_BRIGHT(),
        &prompt.info.display_version,
    );

    let size_mb = if prompt.info.size_bytes > 0 {
        format!(" · {:.1} MB", prompt.info.size_bytes as f64 / (1024.0 * 1024.0))
    } else {
        String::new()
    };
    let subtitle = format!(
        "Matched build for your device: {}{}",
        prompt.info.abi_label, size_mb
    );
    let sub_fit = ui.fit_text(Face::LauncherRegular, 12.5, &subtitle, mw - 44.0);
    ui.text(
        Face::LauncherRegular,
        12.5,
        mx + 22.0,
        my + 43.0,
        theme::MUTED(),
        &sub_fit,
    );

    ui.text(
        Face::LauncherMedium,
        14.0,
        mx + 22.0,
        my + 76.0,
        theme::CRIMSON_BRIGHT(),
        "What's New",
    );

    let box_x = mx + 20.0;
    let box_y = my + 98.0;
    let box_w = mw - 40.0;
    let box_h = (mh - 170.0).max(80.0);
    ui.rounded(box_x, box_y, box_w, box_h, 11.0, theme::CARD_INNER());
    ui.border(box_x, box_y, box_w, box_h, 11.0, theme::BORDER());

    let wrapped = wrap_modal_lines(ui, &prompt.info.whats_new, 13.0, box_w - 28.0);
    let line_h = 21.0;
    let max_scroll = ((wrapped.len() as f32) * line_h + 14.0 - box_h).max(0.0);
    prompt.scroll = prompt.scroll.clamp(0.0, max_scroll);

    for (idx, (line, highlight)) in wrapped.iter().enumerate() {
        let ly = box_y + 8.0 + idx as f32 * line_h - prompt.scroll;
        if ly < box_y + 3.0 || ly + 17.0 > box_y + box_h - 3.0 {
            continue;
        }
        ui.text(
            if *highlight { Face::LauncherMedium } else { Face::LauncherRegular },
            13.0,
            box_x + 14.0,
            ly,
            if *highlight { theme::TEXT() } else { theme::MUTED() },
            line,
        );
    }

    if max_scroll > 1.0 {
        let track_h = (box_h - 12.0).max(18.0);
        let thumb_h = (track_h * (box_h / (box_h + max_scroll))).clamp(18.0, track_h);
        let thumb_y =
            box_y + 6.0 + (track_h - thumb_h) * (prompt.scroll / max_scroll).clamp(0.0, 1.0);
        ui.rounded(box_x + box_w - 7.0, thumb_y, 3.0, thumb_h, 1.5, theme::CRIMSON());
    }

    let footer_y = my + mh - 56.0;
    match &prompt.state {
        crate::volt_updater::UpdateModalState::Downloading { progress, .. } => {
            let snap = progress.lock().unwrap_or_else(|e| e.into_inner()).clone();
            let pct = if snap.total > 0 {
                (snap.downloaded as f32 / snap.total as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let cancel_w = 112.0;
            let bar_w = (mw - 40.0 - cancel_w - 14.0).max(120.0);
            let bar_x = mx + 20.0;
            let dl_mb = snap.downloaded as f64 / (1024.0 * 1024.0);
            let tot_mb = snap.total as f64 / (1024.0 * 1024.0);
            let label = if snap.total > 0 {
                format!(
                    "Downloading {} APK... {:.0}% ({:.1} / {:.1} MB)",
                    prompt.info.abi_label,
                    pct * 100.0,
                    dl_mb,
                    tot_mb
                )
            } else {
                format!("Downloading {} APK... {:.1} MB", prompt.info.abi_label, dl_mb)
            };
            let fit_lbl = ui.fit_text(Face::LauncherMedium, 12.0, &label, bar_w);
            ui.text(Face::LauncherMedium, 12.0, bar_x, footer_y + 2.0, theme::TEXT(), &fit_lbl);
            ui.rounded(bar_x, footer_y + 24.0, bar_w, 12.0, 6.0, theme::CARD_INNER());
            ui.border(bar_x, footer_y + 24.0, bar_w, 12.0, 6.0, theme::BORDER());
            if pct > 0.01 {
                ui.rounded(
                    bar_x,
                    footer_y + 24.0,
                    (bar_w * pct).max(10.0),
                    12.0,
                    6.0,
                    theme::CRIMSON(),
                );
            }
            styled_button(
                ui,
                targets,
                [mx + mw - 20.0 - cancel_w, footer_y + 4.0, cancel_w, 38.0],
                "Cancel",
                Action::CancelUpdate,
                BtnStyle::Secondary,
                true,
            );
        }
        other => {
            if let crate::volt_updater::UpdateModalState::Error(err) = other {
                let max_err_w = (mw - 290.0).max(110.0);
                let err_fit = ui.fit_text(Face::LauncherRegular, 11.5, err, max_err_w);
                ui.text(
                    Face::LauncherRegular,
                    11.5,
                    mx + 22.0,
                    footer_y + 12.0,
                    theme::CRIMSON_BRIGHT(),
                    &err_fit,
                );
            } else {
                ui.text(
                    Face::LauncherRegular,
                    11.5,
                    mx + 22.0,
                    footer_y + 12.0,
                    theme::DIM(),
                    "This reminder appears once.",
                );
            }
            let btn_w = 116.0;
            let install_x = mx + mw - 20.0 - btn_w;
            let cancel_x = install_x - 10.0 - btn_w;
            styled_button(
                ui,
                targets,
                [cancel_x, footer_y + 4.0, btn_w, 38.0],
                "Cancel",
                Action::CancelUpdate,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                targets,
                [install_x, footer_y + 4.0, btn_w, 38.0],
                "Install",
                Action::InstallUpdate,
                BtnStyle::Primary,
                true,
            );
        }
    }
}

fn draw_volt_boost_modal(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    list_targets: &mut Vec<([f32; 4], Action)>,
    catalog: &mut Catalog,
) -> Option<[f32; 3]> {
    use horizon_client::extras::Item;
    use horizon_client::volt_features::VoltBoostPreset;

    let (w, h) = (ui.w, ui.h);
    let scale = ui.density / ui.batch.scale;
    ui.batch
        .fill(ui.atlas, 0.0, 0.0, w * scale, h * scale, theme::BACKDROP());

    let mw = (w - 32.0).clamp(300.0, 740.0);
    let mh = (h - 28.0).clamp(240.0, 500.0);
    let mx = ((w - mw) * 0.5).round();
    let my = ((h - mh) * 0.5).round();

    ui.rounded(mx, my, mw, mh, 16.0, theme::CARD());
    ui.rounded(mx + 8.0, my + 8.0, mw * 0.55, 86.0, 12.0, theme::HERO_GLOW_SOFT());
    ui.border(mx, my, mw, mh, 16.0, theme::BORDER_CRIMSON());

    let active_n = catalog.extras.active_volt_optimizations();
    let rss_mib = horizon_threads::resident_ram_mib().unwrap_or(0);
    ui.text(
        Face::LauncherMedium,
        19.0,
        mx + 20.0,
        my + 12.0,
        theme::TEXT(),
        "Volt Boost — Performance Hub",
    );
    let sub = format!("{active_n} Optimizations Active  •  Resident RAM: {rss_mib} MiB");
    let sub_fit = ui.fit_text(Face::LauncherRegular, 12.5, &sub, (mw - 260.0).max(120.0));
    ui.text(Face::LauncherRegular, 12.5, mx + 20.0, my + 37.0, theme::MUTED(), &sub_fit);

    styled_button(
        ui,
        targets,
        [mx + mw - 236.0, my + 13.0, 106.0, 32.0],
        "Auto-Tune",
        Action::VoltExtra(Item::VoltAutoOptimize),
        BtnStyle::Primary,
        true,
    );
    styled_button(
        ui,
        targets,
        [mx + mw - 122.0, my + 13.0, 104.0, 32.0],
        "Purge RAM",
        Action::VoltExtra(Item::VoltPurgeRam),
        BtnStyle::CrimsonOutline,
        true,
    );

    let presets = [
        (VoltBoostPreset::Off, "OFF"),
        (VoltBoostPreset::Balanced, "Balanced"),
        (VoltBoostPreset::Competitive, "PvP"),
        (VoltBoostPreset::UltraFps, "Ultra"),
        (VoltBoostPreset::BatterySaver, "Battery"),
        (VoltBoostPreset::LowRam, "Low-RAM"),
    ];
    let p_gap = 6.0;
    let p_w = ((mw - 36.0 - p_gap * 5.0) / 6.0).max(42.0);
    for (pi, (preset, label)) in presets.iter().enumerate() {
        let px = mx + 18.0 + pi as f32 * (p_w + p_gap);
        styled_button(
            ui,
            targets,
            [px, my + 58.0, p_w, 30.0],
            label,
            Action::VoltPreset(*preset),
            if catalog.extras.volt_preset == *preset {
                BtnStyle::Primary
            } else {
                BtnStyle::Secondary
            },
            true,
        );
    }

    let items: [Item; 28] = [
        Item::VoltPerfHud,
        Item::VoltAutoDrs,
        Item::VoltCullLeaves,
        Item::VoltFrustumMesh,
        Item::VoltLookaheadStream,
        Item::VoltFastTranslucency,
        Item::VoltCaveSkyCull,
        Item::VoltChunkUploadCap,
        Item::VoltFastWeather,
        Item::VoltDistantLod,
        Item::VoltEntityCap,
        Item::VoltItemCap,
        Item::VoltBlockEntityCap,
        Item::VoltCullSignText,
        Item::VoltFastShadows,
        Item::VoltDistantMobThrottle,
        Item::VoltHubLagShield,
        Item::VoltParticleBudget,
        Item::VoltParticleFrustum,
        Item::VoltChaosShield,
        Item::VoltAnimThrottle,
        Item::VoltActiveAnimCull,
        Item::VoltGlintMode,
        Item::VoltDynamicFps,
        Item::VoltMenuBg,
        Item::VoltThermalGuard,
        Item::VoltReflex,
        Item::VoltMemorySaver,
    ];

    let list_top = my + 96.0;
    let footer_y = my + mh - 46.0;
    let list_bottom = footer_y - 8.0;
    ui.rounded(
        mx + 14.0,
        list_top - 2.0,
        mw - 28.0,
        (list_bottom - list_top + 4.0).max(40.0),
        10.0,
        theme::CARD_INNER(),
    );
    ui.border(
        mx + 14.0,
        list_top - 2.0,
        mw - 28.0,
        (list_bottom - list_top + 4.0).max(40.0),
        10.0,
        theme::BORDER(),
    );

    let two_cols = mw >= 540.0;
    let row_h = 40.0;
    let rows_count = if two_cols {
        items.len().div_ceil(2)
    } else {
        items.len()
    };
    let max_scroll = (rows_count as f32 * row_h - (list_bottom - list_top)).max(0.0);
    catalog.volt_modal_scroll = catalog.volt_modal_scroll.clamp(0.0, max_scroll);
    let col_w = if two_cols {
        (mw - 44.0) * 0.5
    } else {
        mw - 36.0
    };

    for (idx, &item) in items.iter().enumerate() {
        let (r_idx, c_idx) = if two_cols {
            (idx / 2, idx % 2)
        } else {
            (idx, 0)
        };
        let rx = mx + 20.0 + c_idx as f32 * (col_w + 8.0);
        let ry = list_top + 4.0 + r_idx as f32 * row_h - catalog.volt_modal_scroll;
        let ctrl_w = (col_w * 0.44).clamp(96.0, 148.0);
        let ctrl_rect = [rx + col_w - ctrl_w - 6.0, ry + 3.0, ctrl_w, 30.0];
        list_targets.push((ctrl_rect, Action::VoltExtra(item)));
        if ry < list_top - 2.0 || ry + row_h - 4.0 > list_bottom {
            continue;
        }
        let lbl_fit = ui.fit_text(Face::LauncherRegular, 13.0, item.caption(), col_w - ctrl_w - 14.0);
        ui.text(
            Face::LauncherRegular,
            13.0,
            rx + 4.0,
            text_vcenter(ry, row_h - 4.0, 13.0),
            theme::TEXT(),
            &lbl_fit,
        );
        if let Some(on) = item.switch(&catalog.extras) {
            toggle(ui, targets, ctrl_rect, on, Action::VoltExtra(item));
        } else {
            styled_button(
                ui,
                targets,
                ctrl_rect,
                &item.value(&catalog.extras),
                Action::VoltExtra(item),
                BtnStyle::Secondary,
                false,
            );
        }
    }

    styled_button(
        ui,
        targets,
        [mx + 18.0, footer_y + 2.0, 174.0, 36.0],
        "All Volt Settings",
        Action::Page(Page::VoltSettings),
        BtnStyle::CrimsonOutline,
        true,
    );
    styled_button(
        ui,
        targets,
        [mx + mw - 122.0, footer_y + 2.0, 104.0, 36.0],
        "Done",
        Action::CloseVoltBoost,
        BtnStyle::Primary,
        true,
    );
    Some([list_top, list_bottom, max_scroll])
}

