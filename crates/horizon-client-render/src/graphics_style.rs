//! Horizon & Volt Split-Pane Frosted-Glass Settings Design System (Image 1).
//! Shared by `graphics_ui` (`Video Settings` / `Graphics`), `volt_settings_ui`
//! (`Volt Settings`), and `volt_mods` (`Mod Menu`).

use crate::{
    hud::{HudAtlas, HudBatch},
    lang,
    text::Text,
    ui,
};
use horizon_theme::BLOCKS_DARK as B;

/// What a setting costs in frame time, shown beside its description in the bottom dock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cost {
    None,
    Low,
    Medium,
    High,
    /// Turning it on makes frames faster (culling, caching, batching).
    Saves,
}

impl Cost {
    pub fn label(self) -> &'static str {
        match self {
            Cost::None => "No cost",
            Cost::Low => "Low cost",
            Cost::Medium => "Medium cost",
            Cost::High => "High cost",
            Cost::Saves => "Saves time",
        }
    }

    pub fn rgb(self) -> u32 {
        match self {
            Cost::None => B.ink_soft,
            Cost::Low | Cost::Saves => B.success,
            Cost::Medium => B.warn,
            Cost::High => B.danger,
        }
    }
}

/// `0xRRGGBB` + alpha in `HudBatch`'s ABGR u32 color encoding.
pub const fn hex(c: u32, a: u8) -> u32 {
    (a as u32) << 24 | (c & 0xff) << 16 | (c >> 8 & 0xff) << 8 | (c >> 16 & 0xff)
}

/// Frosted dark glass plate & bottom description dock.
pub const GLASS: u32 = hex(B.page, 146);
/// 1 px subtle glass edge.
pub const EDGE: u32 = hex(B.ink, 30);
/// Group sub-panel inside the plate.
pub const GROUP: u32 = hex(B.ink, 13);
/// Hovered or active row highlight.
pub const HOVER: u32 = hex(B.ink, 22);
/// Open sidebar tab background.
pub const OPEN: u32 = hex(B.ink, 30);
/// Divider rules.
pub const HEAD_RULE: u32 = hex(B.ink, 34);
pub const ROW_RULE: u32 = hex(B.ink, 16);
/// Sunk field / stepper well.
pub const WELL: u32 = hex(B.outline, 118);
/// Secondary button, slider track & handle.
pub const BUTTON: u32 = hex(B.ink, 22);
pub const BUTTON_HOVER: u32 = hex(B.ink, 44);
pub const TRACK: u32 = hex(B.ink, 68);
pub const HANDLE: u32 = hex(B.ink, 255);
/// Primary Done button (Horizon/Volt violet).
pub const FACE: u32 = hex(B.face, 255);
pub const FACE_HOVER: u32 = hex(0x6a50e6, 255);
/// Warm amber Undo button (Image 1).
pub const UNDO_FACE: u32 = hex(0x6b4c1e, 235);
pub const UNDO_HOVER: u32 = hex(0x886126, 250);
/// Tick-box off border & shadow outline.
const BOX_EDGE: u32 = hex(B.ink_soft, 96);
const OUTLINE: u32 = hex(B.outline, 255);
pub const ACCENT: u32 = hex(B.accent, 255);
pub const WHITE: u32 = u32::MAX;
pub const MUTED: u32 = hex(B.muted, 255);
pub const SOFT: u32 = hex(B.ink_soft, 255);
pub const GREY: u32 = hex(0xaaaaaa, 255);
pub const VEIL: u32 = hex(B.page, 88);

/// Layout constants in GUI pixels (Image 1).
pub const TITLED: f32 = 300.0;
pub const ROW: f32 = 20.0;
pub const GROUP_ROW: f32 = 14.0;
pub const PAGE_ROW: f32 = 16.0;
pub const GROUP_GAP: f32 = 10.0;
pub const PAGE_GAP: f32 = 6.0;
pub const CONTROL_W: f32 = 112.0;
pub const CONTROL_H: f32 = 16.0;
pub const SEARCH_H: f32 = 16.0;
pub const SCROLLER_W: f32 = 6.0;
pub const DOCK_H: f32 = 50.0;
pub const BUTTON_H: f32 = 20.0;
pub const INSET: f32 = 10.0;

pub fn fill(batch: &mut HudBatch, atlas: &HudAtlas, [x, y, w, h]: [f32; 4], color: u32) {
    if w > 0.0 && h > 0.0 {
        batch.fill(atlas, x, y, w, h, color);
    }
}

pub fn ring(batch: &mut HudBatch, atlas: &HudAtlas, [x, y, w, h]: [f32; 4], color: u32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    batch.fill(atlas, x, y, w, 1.0, color);
    batch.fill(atlas, x, y + h - 1.0, w, 1.0, color);
    if h > 2.0 {
        batch.fill(atlas, x, y + 1.0, 1.0, h - 2.0, color);
        batch.fill(atlas, x + w - 1.0, y + 1.0, 1.0, h - 2.0, color);
    }
}

pub fn panel(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4]) {
    fill(batch, atlas, rect, GLASS);
    ring(batch, atlas, rect, EDGE);
}

pub fn group(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4]) {
    fill(batch, atlas, rect, GROUP);
    ring(batch, atlas, rect, ROW_RULE);
}

pub fn well(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4]) {
    fill(batch, atlas, rect, WELL);
}

pub fn scroller(batch: &mut HudBatch, atlas: &HudAtlas, track: [f32; 4], thumb: [f32; 4]) {
    fill(batch, atlas, track, ROW_RULE);
    fill(batch, atlas, thumb, TRACK);
}

pub fn scrollbar_thumb(top: f32, bottom: f32, content: f32, scroll: f32) -> Option<(f32, f32)> {
    let view = bottom - top;
    if content <= view || view < 16.0 {
        return None;
    }
    let max_bar = view - 6.0;
    let bar = ((view * view) / content).clamp(24.0_f32.min(max_bar), max_bar).floor();
    let max_scroll = (content - view).max(1.0);
    let ratio = (scroll / max_scroll).clamp(0.0, 1.0);
    let y = top + ((view - bar) * ratio).floor();
    Some((y, bar))
}

pub fn fit(atlas: &HudAtlas, text: &str, width: f32) -> String {
    if width <= 8.0 {
        return String::new();
    }
    if atlas.font.width(text) <= width {
        return text.to_owned();
    }
    let mut out = String::new();
    for c in text.chars() {
        if atlas.font.width(&format!("{out}{c}...")) > width {
            break;
        }
        out.push(c);
    }
    format!("{}...", out.trim_end())
}

pub fn wrap_lines(atlas: &HudAtlas, text: &str, width: f32) -> Vec<String> {
    let room = width.max(40.0);
    let mut lines = vec![String::new()];
    for word in text.split_whitespace() {
        let cur = lines.last().cloned().unwrap_or_default();
        let cand = if cur.is_empty() {
            word.to_owned()
        } else {
            format!("{cur} {word}")
        };
        if atlas.font.width(&cand) <= room || cur.is_empty() {
            if let Some(last) = lines.last_mut() {
                *last = cand;
            }
        } else {
            lines.push(word.to_owned());
        }
    }
    lines
}

pub fn checkbox(batch: &mut HudBatch, atlas: &HudAtlas, right: f32, y: f32, on: bool, live: bool) {
    let rect = [right - 12.0, y + 4.0, 12.0, 12.0];
    fill(batch, atlas, rect, WELL);
    let edge = match (on, live) {
        (true, true) => ACCENT,
        (true, false) => MUTED,
        (false, _) => BOX_EDGE,
    };
    ring(batch, atlas, rect, edge);
    if on {
        tick(batch, atlas, rect[0] + 1.0, rect[1] + 2.0, edge);
    }
}

fn tick(batch: &mut HudBatch, atlas: &HudAtlas, x: f32, y: f32, color: u32) {
    const TICK: [&str; 8] = [
        "         #",
        "        ##",
        "       ## ",
        "#     ##  ",
        "##   ##   ",
        " ## ##    ",
        "  ###     ",
        "   #      ",
    ];
    for (j, row) in TICK.iter().enumerate() {
        for (i, c) in row.chars().enumerate() {
            if c == '#' {
                let (px, py) = (x + i as f32, y + j as f32);
                fill(batch, atlas, [px + 1.0, py + 1.0, 1.0, 1.0], OUTLINE);
                fill(batch, atlas, [px, py, 1.0, 1.0], color);
            }
        }
    }
}

/// Animated pill toggle switch (`anim` in `0.0..=1.0`).
pub fn switch_pill(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    [x, y, w, h]: [f32; 4],
    anim: f32,
    on: bool,
) {
    let t = anim.clamp(0.0, 1.0);
    let track_col = if on {
        hex(0x1f9d8b, 255)
    } else {
        hex(0x283244, 255)
    };
    fill(batch, atlas, [x, y, w, h], track_col);
    ring(
        batch,
        atlas,
        [x, y, w, h],
        if on { hex(0x4de3c8, 255) } else { BOX_EDGE },
    );
    let knob_w = (h - 4.0).max(6.0);
    let travel = (w - 4.0 - knob_w).max(0.0);
    let kx = (x + 2.0 + travel * t).round();
    fill(batch, atlas, [kx, y + 2.0, knob_w, h - 4.0], WHITE);
}

pub fn button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    rect: [f32; 4],
    label: &Text,
    (hover, live, primary): (bool, bool, bool),
) {
    button_styled(batch, atlas, rect, label, (hover, live, primary, false));
}

pub fn undo_button(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4], label: &Text, hover: bool) {
    button_styled(batch, atlas, rect, label, (hover, true, false, true));
}

fn button_styled(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    rect: [f32; 4],
    label: &Text,
    (hover, live, primary, warn): (bool, bool, bool, bool),
) {
    let [x, y, w, h] = rect;
    let face = if !live {
        WELL
    } else if warn {
        if hover { UNDO_HOVER } else { UNDO_FACE }
    } else if primary {
        if hover { FACE_HOVER } else { FACE }
    } else if hover {
        BUTTON_HOVER
    } else {
        BUTTON
    };
    fill(batch, atlas, rect, face);
    if hover && live {
        ring(batch, atlas, rect, WHITE);
    } else {
        ring(batch, atlas, rect, EDGE);
    }
    let color = if live { WHITE } else { MUTED };
    let ty = y + ((h - 8.0) / 2.0).floor();
    fitted_centered(batch, atlas, label, [x + 3.0, ty, w - 6.0], color);
}

fn fitted_centered(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    text: &Text,
    [left, y, width]: [f32; 3],
    color: u32,
) {
    let plain = text.plain();
    let fitted = fit(atlas, &plain, width);
    ui::centered_text(batch, atlas, &Text::literal(fitted), left, y, width, color);
}

pub fn slider(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    rect: [f32; 4],
    value: &Text,
    position: f32,
    (hover, live): (bool, bool),
) {
    let [x, y, w, h] = rect;
    let color = if live { WHITE } else { MUTED };
    fitted_centered(batch, atlas, value, [x + 2.0, y + 1.0, w - 4.0], color);
    let line = y + h - 3.0;
    let at = x + (position.clamp(0.0, 1.0) * (w - 4.0)).round();
    fill(batch, atlas, [x, line, w, 2.0], TRACK);
    fill(
        batch,
        atlas,
        [x, line, (at - x).max(0.0), 2.0],
        if live { ACCENT } else { MUTED },
    );
    let handle = [at, line - 3.0, 4.0, 8.0];
    fill(batch, atlas, handle, if live { HANDLE } else { MUTED });
    if hover && live {
        ring(batch, atlas, [at - 1.0, line - 4.0, 6.0, 10.0], ACCENT);
    }
}

pub fn cycler(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    rect: [f32; 4],
    value: &Text,
    (hover, live): (bool, bool),
) {
    let [x, y, w, h] = rect;
    well(batch, atlas, rect);
    if hover && live {
        ring(batch, atlas, rect, WHITE);
    }
    let ty = y + ((h - 8.0) / 2.0).floor();
    let arrows = if hover && live { WHITE } else { MUTED };
    batch.text_styled(atlas, &Text::literal("<"), x + 3.0, ty, arrows, true);
    batch.text_styled(atlas, &Text::literal(">"), x + w - 8.0, ty, arrows, true);
    let color = match (live, hover) {
        (false, _) => MUTED,
        (true, true) => WHITE,
        (true, false) => SOFT,
    };
    fitted_centered(batch, atlas, value, [x + 9.0, ty, w - 18.0], color);
}

pub fn cost_color(cost: Cost) -> u32 {
    hex(cost.rgb(), 255)
}

/// Legacy compatibility struct retained so any external callers referencing `STYLE` compile.
pub struct Style {
    pub control_width: f32,
    pub gap: f32,
    pub max_text_width: f32,
    pub row_height: f32,
    pub header_height: f32,
    pub band: f32,
    pub title_color: u32,
    pub header_color: u32,
    pub description_color: u32,
    pub cost_colors: [u32; 5],
}

const fn opaque(r: u8, g: u8, b: u8) -> u32 {
    0xFF00_0000 | (b as u32) << 16 | (g as u32) << 8 | r as u32
}

pub const STYLE: Style = Style {
    control_width: 150.0,
    gap: 8.0,
    max_text_width: 250.0,
    row_height: 25.0,
    header_height: 16.0,
    band: 33.0,
    title_color: u32::MAX,
    header_color: opaque(255, 255, 85),
    description_color: opaque(170, 170, 170),
    cost_colors: [
        opaque(120, 200, 120),
        opaque(170, 220, 120),
        opaque(240, 200, 90),
        opaque(240, 110, 90),
        opaque(85, 255, 85),
    ],
};

impl Style {
    pub fn description(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        x: f32,
        y: f32,
        cost: Cost,
        text: &str,
    ) {
        let tag = Text::literal(lang::get(cost.label()));
        batch.text_styled(atlas, &tag, x, y, self.cost_colors[cost as usize], true);
        let indent = atlas.font.width_text(&tag) + 4.0;
        let text = lang::get(text);
        batch.text_styled(atlas, &Text::literal(text), x + indent, y, GREY, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cost_has_a_label_and_a_colour() {
        let all = [Cost::None, Cost::Low, Cost::Medium, Cost::High, Cost::Saves];
        assert_eq!(all.len(), STYLE.cost_colors.len());
        for (i, cost) in all.into_iter().enumerate() {
            assert_eq!(cost as usize, i);
            assert!(!cost.label().is_empty());
            assert_ne!(cost_color(cost), 0);
        }
    }
}
