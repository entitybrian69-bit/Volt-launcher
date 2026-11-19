// Shared Volt presentation. Hosts supply action routing, not alternate artwork.
/// Launcher colors: the upstream glass calculations with Volt's deep-blue
/// night palette and a bright blue accent, as in the reference capture.
/// The scene is approximated in this mobile renderer; no backdrop blur is claimed.
pub(crate) mod theme {
    #![allow(non_snake_case)]
    use horizon_client::ui_theme::color::{hex, lerp};
    use std::cell::Cell;
    use horizon_theme::glass::{Glass, ACCENT, BLUR, TINT};
    #[derive(Clone, Copy)]
    struct Look { dark: bool, liquid: bool, glass: Glass }
    thread_local! { static LOOK: Cell<Option<Look>> = const { Cell::new(None) }; }
    thread_local! { static EMBER: Cell<f32> = const { Cell::new(0.5) }; }
    pub fn set(dark: bool, liquid: bool, accent: u32) {
        let accent = accent & 0xffffff;
        LOOK.with(|cell| {
            if cell.get().is_none_or(|look| look.dark != dark || look.liquid != liquid
                || look.glass.accent != accent)
            {
                cell.set(Some(Look { dark, liquid, glass: Glass::new(accent, TINT, BLUR) }));
            }
        });
    }
    fn look() -> Look { LOOK.with(|cell| cell.get().unwrap_or_else(|| Look {
        dark: true, liquid: true, glass: Glass::new(ACCENT, TINT, BLUR)
    })) }
    /// Liquid Black always reads as a dark look, whatever the appearance toggle says.
    pub fn dark() -> bool { let l = look(); l.dark || l.liquid }
    pub fn liquid() -> bool { look().liquid }
    /// Live silver shimmer (0..1) published by the moonlight scene each
    /// frame; the glass layers read it so panels subtly breathe with it.
    pub fn set_ember(value: f32) { EMBER.with(|cell| cell.set(value.clamp(0.0, 1.0))); }
    pub fn ember() -> f32 { EMBER.with(|cell| cell.get()) }
    /// Liquid Glass is strictly monochrome: every accent collapses to
    /// polished silver, whatever accent color Classic is set to.
    fn accent() -> u32 { if liquid() { return hex(0xdfe4ec, 255); } hex(look().glass.accent, 255) }
    fn choose(dark_value: u32, light_value: u32) -> u32 { if dark() { dark_value } else { light_value } }
    pub fn BG() -> u32 { if liquid() { return hex(0x060709, 255); } choose(hex(0x0c0e13,255), hex(0xedf1f0,255)) }
    pub fn BACKDROP() -> u32 { if liquid() { return hex(0x04060c, 225); } hex(0x070a1a, 215) }
    pub fn RAIL() -> u32 { if liquid() { return hex(0x0c0e13, 172); } choose(hex(0x15181f,205), hex(0xfcfdfd,225)) }
    pub fn CARD() -> u32 { if liquid() { return hex(0x11141b, 172); } choose(hex(0x1a1d25,215), hex(0xfcfdfd,230)) }
    pub fn CARD_INNER() -> u32 { if liquid() { return hex(0x191d26, 188); } choose(hex(0x272c36,225), hex(0xe7ecea,245)) }
    pub fn HOVER() -> u32 { if liquid() { return hex(0x272d3a, 212); } choose(hex(0x343a46,235), hex(0xd8e0dd,245)) }
    pub fn HERO_BG() -> u32 { CARD() }
    pub fn HERO_GLOW() -> u32 { lerp(CARD(), accent(), 0.22) }
    pub fn HERO_GLOW_SOFT() -> u32 { lerp(CARD(), accent(), 0.12) }
    pub fn BORDER() -> u32 { if liquid() { return hex(0xffffff, 34); } choose(hex(0xffffff,30), hex(0x3c4a44,34)) }
    pub fn BORDER_CRIMSON() -> u32 { lerp(BORDER(), accent(), 0.32) }
    pub fn CRIMSON() -> u32 { accent() }
    pub fn CRIMSON_BRIGHT() -> u32 {
        if liquid() { return hex(0xf1f4f9, 255); }
        let glass = look().glass;
        hex(if dark() { glass.accent_soft } else { glass.accent }, 255)
    }
    pub fn CRIMSON_SOFT() -> u32 { (accent() & 0x00ffffff) | (42u32 << 24) }
    pub fn CRIMSON_BADGE() -> u32 { choose(hex(0xffffff,22), hex(0x101a14,22)) }
    pub fn ON_CRIMSON() -> u32 {
        // Ink on a silver fill must be near-black in Liquid Glass.
        if liquid() { return hex(0x10141b, 255); }
        let a = look().glass.accent;
        let lum = 0.2126 * (a >> 16 & 255) as f32 + 0.7152 * (a >> 8 & 255) as f32
            + 0.0722 * (a & 255) as f32;
        if lum > 150.0 { hex(0x0c1a12, 255) } else { hex(look().glass.on_accent, 255) }
    }
    pub fn TEXT() -> u32 { choose(hex(0xf2f5f3,255), hex(0x151c18,255)) }
    pub fn MUTED() -> u32 { choose(hex(0xc6ccc9,255), hex(0x4b5551,255)) }
    pub fn DIM() -> u32 { choose(hex(0x8f969b,255), hex(0x68726d,255)) }
    pub fn ONLINE() -> u32 { if liquid() { return hex(0xe6ebf3, 255); } choose(hex(0x4ade80,255), hex(0x15803d,255)) }
    pub fn STATUS_BAD() -> u32 { choose(hex(0xf87171,255), hex(0xb91c1c,255)) }
    pub fn GLASS() -> u32 {
        if liquid() {
            // Near-clear smoked crystal; a cool silver shimmer published by
            // the moonlight scene rolls softly through every panel.
            let e = ember();
            return lerp(hex(0x0c0e13, 150), hex(0x1b1f28, 160), 0.5 * e);
        }
        if !dark() { return hex(0xffffff, 214); }
        // Neutral smoked glass per the chosen reference, faintly tinted by
        // the upstream accent-derived plate so accents still colour panels.
        lerp(hex(0x16191f, 202), {
            let glass = look().glass;
            hex(glass.plate, 202)
        }, 0.18)
    }
    pub fn GLASS_INNER() -> u32 { if liquid() { return hex(0xffffff, (13.0 + 9.0 * ember()) as u8); } choose(hex(0xffffff,14), hex(0x101a14,12)) }
    pub fn GLASS_BORDER() -> u32 { if liquid() { let e = ember(); return lerp(hex(0xffffff, (44.0 + 30.0 * e) as u8), hex(0xdfe6f2, 96), 0.45 * e); } choose(hex(0xffffff,36), hex(0x3c4a44,30)) }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum HomeTab { Overview, Worlds, Multiplayer, Screenshots, Replays, Console }

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
        Page::Controls => {
            // Gamepad: rounded body, D-pad cross left, two action dots right
            ui.border(cx - 8.5, cy - 5.5, 17.0, 11.0, 5.0, col);
            ui.rounded(cx - 6.5, cy - 1.0, 5.5, 2.0, 1.0, col);
            ui.rounded(cx - 4.75, cy - 2.75, 2.0, 5.5, 1.0, col);
            ui.rounded(cx + 2.0, cy - 2.75, 2.2, 2.2, 1.1, col);
            ui.rounded(cx + 4.5, cy + 0.55, 2.2, 2.2, 1.1, col);
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


/// Deterministic per-column terrain noise for the backdrop, 0..1.
fn terra_noise(column: i32, seed: u32) -> f32 {
    let mut x = (column as u32).wrapping_mul(0x9e37_79b9) ^ seed;
    x ^= x >> 16;
    x = x.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 13;
    (x & 0xff) as f32 / 255.0
}

/// Continuously animated live-world backdrop: an original blocky voxel
/// landscape at dusk — gradient sky, a pixel sun, drifting block clouds and
/// three parallax block-stepped ridges — all plain rectangles, so it stays
/// cheap on mobile GPUs. `phase` is seconds since launch.
fn draw_launcher_scene(ui: &mut Ui, w: f32, h: f32, phase: f32) {
    use horizon_client::ui_theme::color::{hex, lerp};
    if theme::liquid() {
        return draw_moonlight_scene(ui, w, h, phase);
    }
    let dark = theme::dark();
    // Sky wash with a warm band near the horizon.
    let stops: &[(f32, u32)] = if dark {
        &[(0.0, 0x111533), (0.40, 0x272e55), (0.62, 0x4a3c63), (0.78, 0x87565c), (1.0, 0xb06a4a)]
    } else {
        &[(0.0, 0x82b4f1), (0.45, 0x9fc6f5), (0.75, 0xc6def9), (1.0, 0xeaf3fe)]
    };
    let rows = 36;
    for row in 0..rows {
        let t = row as f32 / (rows - 1) as f32;
        let mut color = hex(stops[stops.len() - 1].1, 255);
        for pair in stops.windows(2) {
            let ((t0, c0), (t1, c1)) = (pair[0], pair[1]);
            if t >= t0 && t <= t1 {
                color = lerp(hex(c0, 255), hex(c1, 255), (t - t0) / (t1 - t0).max(0.001));
                break;
            }
        }
        ui.rounded(0.0, row as f32 * h / (rows - 1) as f32 - 1.0, w, h / (rows - 1) as f32 + 2.0, 0.0, color);
    }
    if dark {
        for i in 0..44 {
            let x = w * (((i * 73 + 19) % 101) as f32 / 101.0);
            let y = h * 0.42 * (((i * 37 + 11) % 64) as f32 / 64.0);
            let size = if i % 9 == 0 { 2.0 } else { 1.2 };
            ui.rounded(x, y, size, size, 0.0, hex(0xdfe4ff, if i % 3 == 0 { 150 } else { 85 }));
        }
    }
    // A blocky sun low over the ridge line, with a soft warm halo.
    let sun_x = w * 0.58;
    let sun_y = h * 0.55;
    let radius = h * 0.075;
    let grid = (radius / 3.0).max(2.0);
    let (core, rim) = if dark { (0xffe9a8, 0xff8a4e) } else { (0xfff6c8, 0xffd27a) };
    ui.rounded(sun_x - radius * 2.6, sun_y - radius * 1.6, radius * 5.2, radius * 3.2,
        radius * 1.6, hex(rim, if dark { 30 } else { 44 }));
    for band in 0..6 {
        let yy = (band as f32 + 0.5) / 3.0 - 1.0;
        let half = ((1.0 - yy * yy).max(0.0).sqrt() * radius / grid).round() * grid;
        if half <= 0.0 { continue; }
        let color = lerp(hex(core, 255), hex(rim, 255), (yy + 1.0) * 0.5);
        ui.rounded(sun_x - half, sun_y + yy * radius, half * 2.0, radius / 3.0 + 1.0, 0.0, color);
    }
    // Blocky clouds drifting on three bands; each is a few stacked rectangles.
    let cloud_body = if dark { hex(0x3c4368, 205) } else { hex(0xffffff, 235) };
    let cloud_glow = if dark { hex(0x8a5f6b, 150) } else { hex(0xdbe9fb, 235) };
    for k in 0..5u32 {
        let speed = 3.0 + k as f32 * 1.4;
        let span = w + 180.0;
        let x = span - ((phase * speed + k as f32 * 263.0) % span);
        let y = h * (0.10 + 0.085 * k as f32);
        let s = 1.0 + (k % 3) as f32 * 0.35;
        ui.rounded(x - 180.0 + 26.0 * s, y - 9.0 * s, 34.0 * s, 10.0 * s, 0.0, cloud_body);
        ui.rounded(x - 180.0, y, 86.0 * s, 12.0 * s, 0.0, cloud_body);
        ui.rounded(x - 180.0 + 10.0 * s, y + 12.0 * s, 62.0 * s, 5.0 * s, 0.0, cloud_glow);
    }
    // Three parallax ridges of discrete blocks; the front one keeps a grassy lip.
    let bs = (w / 46.0).max(8.0);
    let layers: [(f32, f32, f32, u32, Option<u32>); 3] = if dark {
        [
            (1.6, 0.56, 5.0, 0x2c3153, None),
            (3.4, 0.68, 4.0, 0x20253f, None),
            (6.5, 0.80, 3.0, 0x141828, Some(0x3c5137)),
        ]
    } else {
        [
            (1.6, 0.56, 5.0, 0x9ec49a, None),
            (3.4, 0.68, 4.0, 0x7db077, None),
            (6.5, 0.80, 3.0, 0x5d9c58, Some(0x6fb166)),
        ]
    };
    for (li, &(speed, base, amp, color, lip)) in layers.iter().enumerate() {
        let seed = 0x5eed_0001u32.wrapping_mul(li as u32 + 1);
        let scroll = phase * speed;
        let first = (scroll / bs).floor() as i32;
        let frac = scroll - first as f32 * bs;
        let columns = (w / bs) as i32 + 2;
        for i in 0..columns {
            let column = first + i;
            let n = (terra_noise(column, seed) + terra_noise(column + 1, seed)) * 0.5;
            let blocks = (n * amp).round();
            let y_top = h * base - blocks * bs;
            let x = i as f32 * bs - frac;
            ui.rounded(x, y_top, bs + 0.5, (h - y_top).max(0.0), 0.0, hex(color, 255));
            if let Some(lip_color) = lip {
                ui.rounded(x, y_top, bs + 0.5, bs * 0.55, 0.0, hex(lip_color, 255));
            }
        }
    }
    // A quiet dim layer keeps the frosted panels readable over the scene.
    ui.rounded(0.0, 0.0, w, h, 0.0, if dark { hex(0x090b12, 128) } else { hex(0xffffff, 56) });
}


/// Smoothstep for the scene loop choreography.
fn sstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0).max(1e-4)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The Liquid Glass scene: a desaturated, moonlit voxel mountain night in
/// pure monochrome — black depths, silver ridges, white stars. An ~18s loop:
/// drifting cloud wisps, twinkling stars, a breathing moon halo and one
/// shooting star per loop. Publishes a gentle silver shimmer through
/// `theme::set_ember` so the glass panels above subtly breathe with it.
fn draw_moonlight_scene(ui: &mut Ui, w: f32, h: f32, phase: f32) {
    use horizon_client::ui_theme::color::{hex, lerp};
    let lt = phase % 18.0;
    let shimmer = (0.5
        + 0.28 * (phase * 0.37).sin()
        + 0.14 * (phase * 1.31 + 0.8).sin()
        + 0.08 * (phase * 3.7).sin())
    .clamp(0.0, 1.0);
    theme::set_ember(shimmer);

    // Night sky: pure black up high, the faintest blue-grey toward the
    // ridge line — dark with depth, never flat.
    let stops: [(f32, u32); 4] =
        [(0.0, 0x020304), (0.5, 0x050608), (0.8, 0x090a0e), (1.0, 0x0c0d11)];
    let rows = 20;
    for row in 0..rows {
        let ty = row as f32 / (rows - 1) as f32;
        let mut color = hex(stops[3].1, 255);
        for pair in stops.windows(2) {
            let ((t0, c0), (t1, c1)) = (pair[0], pair[1]);
            if ty >= t0 && ty <= t1 {
                color = lerp(hex(c0, 255), hex(c1, 255), (ty - t0) / (t1 - t0).max(0.001));
                break;
            }
        }
        ui.rounded(0.0, row as f32 * h / (rows - 1) as f32 - 1.0, w,
            h / (rows - 1) as f32 + 2.0, 0.0, color);
    }
    // Stars on individual twinkle clocks, cool white.
    for i in 0..60u32 {
        let x = w * (((i * 73 + 19) % 101) as f32 / 101.0);
        let y = h * 0.56 * (((i * 37 + 11) % 64) as f32 / 64.0);
        let tw = 0.55 + 0.45 * (phase * (0.5 + (i % 7) as f32 * 0.21) + i as f32 * 1.37).sin();
        let alpha = (if i % 3 == 0 { 165.0 } else { 95.0 }) * tw;
        let size = if i % 9 == 0 { 2.2 } else { 1.3 };
        ui.rounded(x, y, size, size, 0.0, hex(0xe8ecf4, alpha.max(0.0) as u8));
    }
    // One shooting star per loop, streaking down-right at ~11s.
    let shoot = sstep(11.0, 11.12, lt) * (1.0 - sstep(11.55, 11.85, lt));
    if shoot > 0.01 {
        let t01 = ((lt - 11.0) / 0.85).clamp(0.0, 1.0);
        let (sx, sy) = (w * (0.12 + 0.30 * t01), h * (0.10 + 0.14 * t01));
        for seg in 0..6 {
            let back = seg as f32 * 0.035;
            let (px, py) = (w * (0.12 + 0.30 * (t01 - back).max(0.0)),
                h * (0.10 + 0.14 * (t01 - back).max(0.0)));
            let a = (shoot * (150.0 - seg as f32 * 24.0)).max(0.0);
            ui.rounded(px, py, 2.2, 2.2, 1.1, hex(0xf2f5fa, a as u8));
        }
        ui.rounded(sx, sy, 2.8, 2.8, 1.4, hex(0xffffff, (200.0 * shoot) as u8));
    }
    // Crescent moon with a halo that slowly breathes.
    let (mx, my, mr) = (w * 0.70, h * 0.19, h * 0.048);
    let halo = mr * (2.3 + 0.45 * (phase * 0.5).sin());
    ui.rounded(mx - halo, my - halo, halo * 2.0, halo * 2.0, halo,
        hex(0xaeb6c6, (10.0 + 12.0 * shimmer) as u8));
    ui.rounded(mx - mr, my - mr, mr * 2.0, mr * 2.0, mr, hex(0xe9edf5, 225));
    ui.rounded(mx - mr + mr * 0.60, my - mr - mr * 0.20, mr * 2.0, mr * 2.0, mr,
        hex(0x040507, 255));
    // Thin silver cloud wisps drifting on their own slow clocks.
    for i in 0..4u32 {
        let speed = 2.2 + i as f32 * 1.1;
        let span = w + 360.0;
        let x = ((phase * speed + i as f32 * 420.0) % span) - 180.0;
        let y = h * (0.16 + 0.11 * i as f32) + (phase * 0.2 + i as f32).sin() * 3.0;
        let cw = 170.0 + 50.0 * i as f32;
        let ch = 9.0 + 2.5 * i as f32;
        ui.rounded(x, y, cw, ch, ch * 0.5, hex(0x9ba4b6, (9 + i * 2) as u8));
        ui.rounded(x + cw * 0.2, y - ch * 0.45, cw * 0.55, ch * 0.8, ch * 0.4,
            hex(0xaab2c2, 7));
    }
    // Three parallax ranges of blocky mountains, each ridge top kissed by
    // a thin line of moonlight that brightens with the shimmer.
    let layers: [(f32, f32, u32, u32, u8); 3] = [
        (1.4, 0.58, 0xb7c3_0003, 0x0d0f14, 70),
        (2.6, 0.68, 0xb7c3_0017, 0x08090d, 48),
        (4.2, 0.78, 0xb7c3_0031, 0x040406, 30),
    ];
    for (speed, base, seed, color, rim_a) in layers {
        let bs = (w / 46.0).max(8.0);
        let scroll = phase * speed;
        let first = (scroll / bs).floor() as i32;
        let frac = scroll - first as f32 * bs;
        for i in 0..((w / bs) as i32 + 2) {
            let column = first + i;
            let n = (terra_noise(column, seed) + terra_noise(column + 1, seed)) * 0.5;
            let y_top = h * base - (n * 4.0).round() * bs * 0.55;
            let x = i as f32 * bs - frac;
            ui.rounded(x, y_top, bs + 0.5, (h - y_top).max(0.0), 0.0, hex(color, 255));
            // Moonlit ridge line.
            let rim = (rim_a as f32 * (0.6 + 0.4 * shimmer)) as u8;
            ui.rounded(x, y_top, bs + 0.5, 1.6, 0.0, hex(0x6f7a8e, rim));
        }
    }
    // Deepen the foreground so the dock floats over true black.
    ui.rounded(0.0, h * 0.82, w, h * 0.18, 0.0, hex(0x020203, 150));
    // A final whisper of dim keeps text crisp over the brightest ridges.
    ui.rounded(0.0, 0.0, w, h, 0.0, hex(0x020308, 64));
}

/// Tap feedback for lobby slabs: briefly dips the vertices drawn since
/// `v_start` toward the slab centre, matching the pill/button behaviour.
fn press_scale(ui: &mut Ui, v_start: usize, rect: [f32; 4], action: Action) {
    let press_t = ACTIVE_FEEDBACK.with(|fb| {
        fb.borrow().and_then(|(act, ts)| {
            let el = ts.elapsed().as_secs_f32();
            (act == action && el < 0.22).then_some(el / 0.22)
        })
    });
    if let Some(t) = press_t {
        let dip = (t * std::f32::consts::PI).sin() * (1.0 - t * 0.45);
        ui.animate_vertices(v_start, 0.0, 0.0, 1.0 - 0.05 * dip,
            [rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5], 1.0);
    }
}

/// Headset glyph for the lobby Help button.
fn draw_headset_icon(ui: &mut Ui, cx: f32, cy: f32, col: u32) {
    ui.border(cx - 7.5, cy - 7.5, 15.0, 14.5, 7.0, col);
    ui.rounded(cx - 9.0, cy - 1.0, 4.5, 8.0, 2.0, col);
    ui.rounded(cx + 4.5, cy - 1.0, 4.5, 8.0, 2.0, col);
    ui.rounded(cx + 1.5, cy + 6.2, 6.0, 2.2, 1.1, col);
}

/// Minimal line icons for home tiles that are not launcher pages, built from
/// the same primitives as the page icons.
fn draw_tile_icon(ui: &mut Ui, kind: u8, cx: f32, cy: f32, col: u32) {
    match kind {
        // Worlds: a globe ring with an equator.
        0 => {
            ui.border(cx - 8.0, cy - 8.0, 16.0, 16.0, 8.0, col);
            ui.rounded(cx - 8.0, cy - 1.0, 16.0, 2.0, 1.0, col);
            ui.rounded(cx - 1.0, cy - 8.0, 2.0, 16.0, 1.0, col);
        }
        // Multiplayer: two heads side by side.
        1 => {
            ui.rounded(cx - 8.0, cy - 6.0, 7.0, 7.0, 3.5, col);
            ui.rounded(cx + 1.0, cy - 6.0, 7.0, 7.0, 3.5, col);
            ui.rounded(cx - 8.5, cy + 3.0, 8.0, 5.0, 2.5, col);
            ui.rounded(cx + 0.5, cy + 3.0, 8.0, 5.0, 2.5, col);
        }
        // Screenshots: a camera body with a lens dot.
        2 => {
            ui.border(cx - 8.5, cy - 6.0, 17.0, 12.5, 4.0, col);
            ui.rounded(cx - 2.5, cy - 2.2, 5.0, 5.0, 2.5, col);
            ui.rounded(cx - 6.0, cy - 8.5, 5.0, 3.0, 1.5, col);
        }
        // Logs: three text lines.
        3 => {
            for (i, lw) in [16.0, 12.0, 14.0].iter().enumerate() {
                ui.rounded(cx - 8.0, cy - 6.0 + i as f32 * 5.5, *lw, 2.4, 1.2, col);
            }
        }
        // Replays: a filmstrip frame with sprocket holes.
        5 => {
            ui.border(cx - 9.0, cy - 6.5, 18.0, 13.0, 3.0, col);
            for i in 0..3 {
                let hy = cy - 5.0 + i as f32 * 4.0;
                ui.rounded(cx - 7.0, hy, 2.2, 2.2, 1.1, col);
                ui.rounded(cx + 4.8, hy, 2.2, 2.2, 1.1, col);
            }
            ui.rounded(cx - 2.2, cy - 3.0, 4.4, 6.0, 1.4, col);
        }
        // Boost/settings bolt: a simple two-part lightning mark.
        _ => {
            ui.rounded(cx - 1.0, cy - 8.0, 4.5, 8.5, 1.5, col);
            ui.rounded(cx - 3.5, cy - 0.5, 4.5, 8.5, 1.5, col);
            ui.rounded(cx - 5.0, cy - 1.2, 10.0, 2.4, 1.2, col);
        }
    }
}

/// The Liquid Glass home: moonlit vista behind, chrome VOLT brand
/// top-left, glass account chip and the Discord button top-right, a
/// destination rail down the left edge, a gallery rail (Worlds / Server /
/// Shots / Replay) down the right edge, the chrome-rimmed PLAY pill at
/// the middle-bottom, and a spaced chip row along the bottom-left.
/// Strictly monochrome.
fn liquid_home(ui: &mut Ui, area: [f32; 4], account_status: &str, secs: u64,
    installed: bool, progress: Option<horizon_client::official_install::InstallProgress>,
    error: Option<&str>, targets: &mut Vec<([f32; 4], Action)>) {
    use horizon_client::official_install::InstallStage;
    use horizon_client::ui_theme::color::hex;
    let [ax, ay, w, h] = area;
    let installing = progress.is_some();
    let compact = w < 760.0;
    let m = if compact { 12.0 } else { 18.0 };
    let shimmer = theme::ember();

    // ---- Brand, top-left: V logo + VOLT, playtime whispered under it ----
    ui.volt_logo(ax + m, ay + m, 34.0, 34.0);
    ui.text(Face::Display, 21.0, ax + m + 44.0, ay + m + 4.0, hex(0xf2f4f8, 255), "VOLT");
    let time = if secs == 0 { "0 h".to_owned() }
        else if secs < 3600 { format!("{} min", (secs / 60).max(1)) }
        else { format!("{} h", secs / 3600) };
    ui.text(Face::LauncherRegular, 10.5, ax + m + 44.0, ay + m + 26.0, theme::DIM(),
        &format!("26.3 · {time} played"));

    // ---- Account chip, top-right ----
    let (name, badge) = account_status.split_once(" • ")
        .map(|(n, b)| (n.to_owned(), b.to_owned()))
        .unwrap_or_else(|| (account_status.to_owned(), "Offline".to_owned()));
    let chip_h = 44.0;
    let chip_w = (ui.measure(Face::LauncherMedium, 13.5, &name) + chip_h + 26.0)
        .min(w * 0.30)
        .max(132.0);
    let chip = [ax + w - m - chip_w, ay + m, chip_w, chip_h];
    ui.rounded(chip[0], chip[1], chip[2], chip[3], chip_h * 0.5, theme::GLASS());
    ui.border(chip[0], chip[1], chip[2], chip[3], chip_h * 0.5, theme::GLASS_BORDER());
    ui.rounded(chip[0] + 6.0, chip[1] + 6.0, chip_h - 12.0, chip_h - 12.0,
        (chip_h - 12.0) * 0.5, theme::GLASS_INNER());
    let initial = name.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('V');
    draw_centered_text(ui, Face::LauncherMedium, 15.0,
        [chip[0] + 6.0, chip[1] + 6.0, chip_h - 12.0, chip_h - 12.0],
        hex(0xf2f4f8, 255), &initial.to_string());
    let text_x = chip[0] + chip_h + 4.0;
    let name_fit = ui.fit_text(Face::LauncherMedium, 13.5, &name, chip_w - chip_h - 16.0);
    ui.text(Face::LauncherMedium, 13.5, text_x, chip[1] + 7.0, theme::TEXT(), &name_fit);
    let badge_fit = ui.fit_text(Face::LauncherRegular, 10.5, &badge, chip_w - chip_h - 16.0);
    ui.text(Face::LauncherRegular, 10.5, text_x, chip[1] + 25.0, theme::DIM(), &badge_fit);
    targets.push((chip, Action::Page(Page::Account)));

    // ---- Discord, moved up: a round glass button under the account chip ----
    let fab = 44.0;
    let help = [ax + w - m - fab, chip[1] + chip_h + 12.0, fab, fab];
    {
        let v_start = ui.batch_mut().vertices.len();
        ui.rounded(help[0], help[1], help[2], help[3], fab * 0.5, theme::GLASS());
        ui.border(help[0], help[1], help[2], help[3], fab * 0.5, theme::GLASS_BORDER());
        draw_headset_icon(ui, help[0] + fab * 0.5, help[1] + fab * 0.5, hex(0xe8ecf4, 230));
        press_scale(ui, v_start, help, Action::Discord);
        targets.push((help, Action::Discord));
    }

    // ---- The hero: the chrome-rimmed glass PLAY pill, middle-bottom ----
    let play_h = if compact { 64.0 } else { 76.0 };
    let play_w = if compact { 236.0 } else { 300.0 };
    let py0 = ay + h - m - play_h;
    let px0 = ax + (w - play_w) * 0.5;

    // ---- Left rail: the launcher's destinations, flush down the left edge ----
    let rail_items: [(&str, Page, Action); 7] = [
        ("Home", Page::Play, Action::Page(Page::Play)),
        ("Instances", Page::Installed, Action::Profile),
        ("Browse", Page::Mods, Action::Page(Page::Mods)),
        ("Packs", Page::Packs, Action::Page(Page::Packs)),
        ("Shaders", Page::Shaders, Action::Page(Page::Shaders)),
        ("Controls", Page::Controls, Action::Page(Page::Controls)),
        ("Settings", Page::Settings, Action::Page(Page::Settings)),
    ];
    let rail_top = ay + m + 54.0;
    let rail_bottom = ay + h - m;
    let rail_w = if compact { 50.0 } else { 64.0 };
    let step = ((rail_bottom - rail_top - 14.0) / rail_items.len() as f32)
        .min(64.0)
        .max(34.0);
    let labels = !compact && step >= 46.0;
    let rail_h = (step * rail_items.len() as f32 + 14.0).min(rail_bottom - rail_top);
    let rail = [ax + 10.0, rail_top, rail_w, rail_h];
    ui.rounded(rail[0], rail[1], rail[2], rail[3], 18.0, theme::GLASS());
    ui.border(rail[0], rail[1], rail[2], rail[3], 18.0, theme::GLASS_BORDER());
    for (i, (label, page, action)) in rail_items.iter().enumerate() {
        let rect = [rail[0] + 3.0, rail[1] + 7.0 + i as f32 * step, rail_w - 6.0, step];
        if rect[1] + rect[3] > rail[1] + rail[3] {
            break;
        }
        let active = i == 0;
        let cx = rect[0] + rect[2] * 0.5;
        let cy = rect[1] + if labels { step * 0.5 - 7.0 } else { step * 0.5 };
        if active {
            ui.rounded(rect[0] + 2.0, rect[1] + 2.0, rect[2] - 4.0, rect[3] - 4.0, 12.0,
                hex(0xffffff, (18.0 + 12.0 * shimmer) as u8));
            ui.rounded(rail[0] - 1.0, cy - 9.0, 3.0, 18.0, 1.5, hex(0xf2f5fa, 235));
        }
        draw_page_icon(ui, *page, cx, cy,
            if active { hex(0xffffff, 255) } else { hex(0xc9cfdb, 185) });
        if labels {
            draw_centered_text(ui, Face::LauncherMedium, 9.5,
                [rect[0], cy + 11.0, rect[2], 12.0],
                if active { hex(0xf2f5fa, 235) } else { hex(0xc9cfdb, 165) }, label);
        }
        targets.push((rect, *action));
    }

    // ---- Right rail: the play-space galleries, flush down the right edge ----
    let gallery_items: [(&str, u8, Action); 4] = [
        ("Worlds", 0, Action::HomeTab(HomeTab::Worlds)),
        ("Server", 1, Action::HomeTab(HomeTab::Multiplayer)),
        ("Shots", 2, Action::HomeTab(HomeTab::Screenshots)),
        ("Replay", 5, Action::HomeTab(HomeTab::Replays)),
    ];
    let gal_top = help[1] + fab + 12.0;
    let gal_bottom = ay + h - m;
    let gal_step = ((gal_bottom - gal_top - 14.0) / gallery_items.len() as f32)
        .min(68.0)
        .max(34.0);
    let gal_labels = !compact && gal_step >= 46.0;
    let gal_h = (gal_step * gallery_items.len() as f32 + 14.0).min(gal_bottom - gal_top);
    let gal = [ax + w - 10.0 - rail_w, gal_top, rail_w, gal_h];
    ui.rounded(gal[0], gal[1], gal[2], gal[3], 18.0, theme::GLASS());
    ui.border(gal[0], gal[1], gal[2], gal[3], 18.0, theme::GLASS_BORDER());
    for (i, (label, glyph, action)) in gallery_items.iter().enumerate() {
        let rect = [gal[0] + 3.0, gal[1] + 7.0 + i as f32 * gal_step, rail_w - 6.0, gal_step];
        if rect[1] + rect[3] > gal[1] + gal[3] {
            break;
        }
        let cx = rect[0] + rect[2] * 0.5;
        let cy = rect[1] + if gal_labels { gal_step * 0.5 - 7.0 } else { gal_step * 0.5 };
        let v_start = ui.batch_mut().vertices.len();
        draw_tile_icon(ui, *glyph, cx, cy, hex(0xc9cfdb, 185));
        if gal_labels {
            draw_centered_text(ui, Face::LauncherMedium, 9.5,
                [rect[0], cy + 11.0, rect[2], 12.0], hex(0xc9cfdb, 165), label);
        }
        press_scale(ui, v_start, rect, *action);
        targets.push((rect, *action));
    }
    let slab = [px0, py0, play_w, play_h];
    if let Some(progress) = progress {
        ui.rounded(slab[0], slab[1], slab[2], slab[3], play_h * 0.5, theme::GLASS());
        ui.border(slab[0], slab[1], slab[2], slab[3], play_h * 0.5,
            hex(0xffffff, 150));
        let index = InstallStage::ALL.iter().position(|s| *s == progress.stage).unwrap_or(0);
        let part = if progress.total > 0 { progress.done as f32 / progress.total as f32 } else { 0.0 };
        let fraction = ((index as f32 + part.clamp(0.0, 1.0)) / InstallStage::ALL.len() as f32)
            .clamp(0.0, 1.0);
        draw_centered_text(ui, Face::LauncherMedium, 10.0,
            [slab[0], py0 + 10.0, play_w, 12.0], theme::DIM(), "INSTALLING");
        draw_centered_text(ui, Face::Display, 20.0,
            [slab[0], py0 + 24.0, play_w, 24.0], theme::TEXT(),
            &format!("{:.0}%", fraction * 100.0));
        let bar = [slab[0] + 22.0, py0 + play_h - 18.0, play_w - 44.0, 4.0];
        ui.rounded(bar[0], bar[1], bar[2], bar[3], 2.0, hex(0xffffff, 40));
        ui.rounded(bar[0], bar[1], (bar[2] * fraction).max(4.0), bar[3], 2.0,
            hex(0xf2f5fa, 255));
    } else {
        let v_start = ui.batch_mut().vertices.len();
        // Soft white aura breathing with the moon shimmer.
        let halo = 6.0 + 5.0 * shimmer;
        ui.rounded(slab[0] - halo, slab[1] - halo, slab[2] + 2.0 * halo,
            slab[3] + 2.0 * halo, (play_h + 2.0 * halo) * 0.5,
            hex(0xdfe6f2, (14.0 + 14.0 * shimmer) as u8));
        // Clear glass body.
        ui.rounded(slab[0], slab[1], slab[2], slab[3], play_h * 0.5, theme::GLASS());
        // Chrome rim: a bright outer ring and a quieter inner one.
        ui.border(slab[0], slab[1], slab[2], slab[3], play_h * 0.5,
            hex(0xffffff, (150.0 + 70.0 * shimmer) as u8));
        ui.border(slab[0] + 2.5, slab[1] + 2.5, slab[2] - 5.0, slab[3] - 5.0,
            (play_h - 5.0) * 0.5, hex(0xffffff, 46));
        // Glossy highlight sweeping the top of the capsule.
        ui.rounded(slab[0] + play_h * 0.28, slab[1] + 5.0,
            slab[2] - play_h * 0.56, play_h * 0.34, play_h * 0.17,
            hex(0xffffff, 34));
        let label = if installed { "PLAY" }
            else if error.is_some() { "RETRY" } else { "DOWNLOAD" };
        let font = if compact { 24.0 } else { 28.0 };
        draw_centered_text(ui, Face::Display, font, slab, hex(0xffffff, 255), label);
        press_scale(ui, v_start, slab, Action::Play);
        targets.insert(0, (slab, Action::Play));
    }

    // Error line floating above the pill.
    if !installing && let Some(error) = error {
        let e = ui.fit_text(Face::LauncherRegular, 11.5, error, w - 2.0 * m - 40.0);
        let ew = ui.measure(Face::LauncherRegular, 11.5, &e);
        ui.text(Face::LauncherRegular, 11.5, ax + (w - ew) * 0.5, py0 - 20.0,
            theme::STATUS_BAD(), &e);
    }

    // ---- Quiet utility chips: a spaced row along the bottom, between
    // the left rail and the PLAY pill, never stacked or overlapping ----
    let chips: [(&str, Action); 3] = [
        ("Logs", Action::HomeTab(HomeTab::Console)),
        ("Files", Action::Page(Page::Installed)),
        ("Account", Action::Page(Page::Account)),
    ];
    let chip_small_h = 28.0;
    let mut chip_x = rail[0] + rail_w + 14.0;
    let chip_y = ay + h - m - chip_small_h;
    for (label, action) in chips.iter() {
        let cw = ui.measure(Face::LauncherMedium, 11.0, label) + 24.0;
        if chip_x + cw > px0 - 12.0 {
            break;
        }
        let rect = [chip_x, chip_y, cw, chip_small_h];
        ui.rounded(rect[0], rect[1], rect[2], rect[3], chip_small_h * 0.5, theme::GLASS());
        ui.border(rect[0], rect[1], rect[2], rect[3], chip_small_h * 0.5, hex(0xffffff, 26));
        draw_centered_text(ui, Face::LauncherMedium, 11.0, rect, hex(0xc9cfdb, 200), label);
        targets.push((rect, *action));
        chip_x += cw + 10.0;
    }
}
