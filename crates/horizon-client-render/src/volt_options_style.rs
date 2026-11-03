//! Volt Launcher's obsidian-dark and crimson-red theme & animation engine for
//! in-game Option screens (`OptionsUi`, `GraphicsUi`, `PackScreen`, and mod
//! settings). Keeps widget bounds and controller/touch targets identical while
//! replacing vanilla stone sprites with animated Volt cards, crimson fill-bar
//! sliders, sleek search fields, and smooth page transitions.

use crate::{
    hud::{HudAtlas, HudBatch, rgba},
    text::{Style, Text},
    ui,
};
use std::time::Instant;

/// Convert `0xRRGGBB` + `alpha` into `HudBatch`'s packed ABGR `u32`.
pub const fn hex_rgba(rgb: u32, a: u8) -> u32 {
    let r = ((rgb >> 16) & 0xFF) as u8;
    let g = ((rgb >> 8) & 0xFF) as u8;
    let b = (rgb & 0xFF) as u8;
    (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | (r as u32)
}

/// Linearly interpolate two packed ABGR `HudBatch` colors.
pub fn lerp_color(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ca = a.to_le_bytes();
    let cb = b.to_le_bytes();
    let mix = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    rgba(
        mix(ca[0], cb[0]),
        mix(ca[1], cb[1]),
        mix(ca[2], cb[2]),
        mix(ca[3], cb[3]),
    )
}

/// Scale the alpha channel of a packed ABGR `HudBatch` color.
pub fn scale_alpha(color: u32, factor: f32) -> u32 {
    let [r, g, b, a] = color.to_le_bytes();
    let na = (f32::from(a) * factor.clamp(0.0, 1.0)).round() as u8;
    rgba(r, g, b, na)
}

// Exact Volt Launcher palette (`crates/horizon-client/src/android/launcher_pages.rs`).
pub const BG_HEX: u32 = 0x07070A;
pub const BACKDROP_HEX: u32 = 0x030305;
pub const RAIL_HEX: u32 = 0x0C0C12;
pub const CARD_HEX: u32 = 0x111118;
pub const CARD_INNER_HEX: u32 = 0x181822;
pub const HOVER_HEX: u32 = 0x232331;
pub const HERO_BG_HEX: u32 = 0x19070D;
pub const HERO_HOVER_HEX: u32 = 0x280B16;
pub const BORDER_HEX: u32 = 0x262635;
pub const BORDER_SUBTLE_HEX: u32 = 0x1B1B26;
pub const BORDER_CRIMSON_HEX: u32 = 0x6D1729;
pub const CRIMSON_HEX: u32 = 0xDC143C;
pub const CRIMSON_BRIGHT_HEX: u32 = 0xFF2E54;
pub const CRIMSON_BADGE_HEX: u32 = 0x3B0D18;
pub const ONLINE_HEX: u32 = 0x22C55E;
pub const TEXT_HEX: u32 = 0xF4F4F8;
pub const MUTED_HEX: u32 = 0x9898AA;
pub const DIM_HEX: u32 = 0x68687A;

pub const TEXT_COLOR: u32 = hex_rgba(TEXT_HEX, 255);
pub const MUTED_COLOR: u32 = hex_rgba(MUTED_HEX, 255);
pub const DIM_COLOR: u32 = hex_rgba(DIM_HEX, 255);
pub const CRIMSON_COLOR: u32 = hex_rgba(CRIMSON_HEX, 255);
pub const CRIMSON_BRIGHT_COLOR: u32 = hex_rgba(CRIMSON_BRIGHT_HEX, 255);

/// Visual role of an option button so hero pages, primary actions, and ON/OFF
/// toggles are immediately scannable at a glance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonTone {
    /// Standard setting or navigation button.
    Standard,
    /// Featured Volt screen button (`Volt Settings`, `Volt Boost`, `Volt Shaders`, `Graphics`).
    Hero,
    /// Primary confirmation / `Done` action.
    Primary,
    /// Boolean or stateful option currently switched ON.
    ActiveOn,
    /// Boolean option currently switched OFF.
    ActiveOff,
}

/// Detect whether a localized/formatted option label represents an ON or OFF toggle.
pub fn infer_toggle_tone(plain: &str) -> ButtonTone {
    let trimmed = plain.trim();
    if let Some((_, rhs)) = trimmed.rsplit_once(':') {
        let v = rhs.trim();
        if v.eq_ignore_ascii_case("on")
            || v.eq_ignore_ascii_case("enabled")
            || v.eq_ignore_ascii_case("yes")
            || v.eq_ignore_ascii_case("true")
            || v == "Вкл"
            || v == "ВКЛ"
            || v == "Да"
        {
            return ButtonTone::ActiveOn;
        }
        if v.eq_ignore_ascii_case("off")
            || v.eq_ignore_ascii_case("disabled")
            || v.eq_ignore_ascii_case("no")
            || v.eq_ignore_ascii_case("false")
            || v == "Выкл"
            || v == "ВЫКЛ"
            || v == "Нет"
        {
            return ButtonTone::ActiveOff;
        }
    }
    ButtonTone::Standard
}

#[derive(Clone, Copy, Debug)]
struct SlotAnim {
    id: u64,
    hover: f32,
    slider: f32,
    slider_init: bool,
    pulse: f32,
}

/// Lightweight per-screen animation state for hover/press/slider interpolation
/// and page-navigation transitions.
#[derive(Clone, Debug)]
pub struct OptionAnim {
    started: Instant,
    last_frame: Option<Instant>,
    transition_at: Option<Instant>,
    /// `-1.0` = pop/back, `0.0` = initial open, `1.0` = push/deeper.
    transition_dir: f32,
    slots: Vec<SlotAnim>,
}

impl Default for OptionAnim {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            last_frame: None,
            transition_at: None,
            transition_dir: 0.0,
            slots: Vec::new(),
        }
    }
}

impl OptionAnim {
    const TRANSITION_SECS: f32 = 0.165;

    /// Advance frame delta (`0.0..=0.05`s) and decay button click pulses.
    pub fn begin_frame(&mut self) -> f32 {
        let now = Instant::now();
        let dt = self
            .last_frame
            .map_or(1.0 / 60.0, |prev| {
                now.saturating_duration_since(prev).as_secs_f32()
            })
            .clamp(0.0, 0.05);
        self.last_frame = Some(now);
        for s in &mut self.slots {
            if s.pulse > 0.0 {
                s.pulse = (s.pulse - dt * 6.5).max(0.0);
            }
        }
        if self.slots.len() > 160 {
            self.slots.drain(..self.slots.len() - 128);
        }
        dt
    }

    /// Monotonic elapsed seconds for subtle marquee scrolling & active glow.
    pub fn elapsed_secs(&self) -> f32 {
        self.started.elapsed().as_secs_f32()
    }

    /// Start a smooth page entrance (`dir = 0.0` open, `1.0` push, `-1.0` back).
    pub fn start_transition(&mut self, dir: f32) {
        self.transition_at = Some(Instant::now());
        self.transition_dir = dir;
        for s in &mut self.slots {
            s.hover = 0.0;
            s.slider_init = false;
        }
    }

    /// Immediately settle any active page transition (used by static jump/test entrypoints).
    pub fn settle_transition(&mut self) {
        self.transition_at = None;
    }

    /// Current `(dx, dy, alpha, underline_progress)` for the active page transition.
    pub fn transition_state(&mut self) -> (f32, f32, f32, f32) {
        let Some(at) = self.transition_at else {
            return (0.0, 0.0, 1.0, 1.0);
        };
        let raw = (at.elapsed().as_secs_f32() / Self::TRANSITION_SECS).clamp(0.0, 1.0);
        if raw >= 1.0 {
            self.transition_at = None;
            return (0.0, 0.0, 1.0, 1.0);
        }
        // Cubic ease-out: crisp immediate response with silky deceleration.
        let eased = 1.0 - (1.0 - raw).powi(3);
        let rem = 1.0 - eased;
        let (dx, dy) = if self.transition_dir.abs() < 0.1 {
            (0.0, 7.0 * rem)
        } else {
            (11.0 * self.transition_dir.signum() * rem, 0.0)
        };
        let alpha = (0.38 + 0.62 * eased).clamp(0.0, 1.0);
        (dx, dy, alpha, eased)
    }

    fn slot_mut(&mut self, id: u64) -> &mut SlotAnim {
        if let Some(pos) = self.slots.iter().position(|s| s.id == id) {
            return &mut self.slots[pos];
        }
        self.slots.push(SlotAnim {
            id,
            hover: 0.0,
            slider: 0.0,
            slider_init: false,
            pulse: 0.0,
        });
        self.slots.last_mut().expect("pushed above")
    }

    /// Smoothly approach target hover (`0.0` or `1.0`) and return `(hover_t, pulse_t)`.
    pub fn step_button(&mut self, id: u64, hovered: bool, dt: f32) -> (f32, f32) {
        let target = if hovered { 1.0 } else { 0.0 };
        let k = (1.0 - (-dt * 18.0).exp()).clamp(0.0, 1.0);
        let slot = self.slot_mut(id);
        slot.hover += (target - slot.hover) * k;
        if (slot.hover - target).abs() < 0.004 {
            slot.hover = target;
        }
        (slot.hover, slot.pulse)
    }

    /// Smoothly approach slider value and hover state.
    pub fn step_slider(
        &mut self,
        id: u64,
        value: f32,
        hovered: bool,
        dragging: bool,
        dt: f32,
    ) -> (f32, f32) {
        let value = value.clamp(0.0, 1.0);
        let hover_target = if hovered || dragging { 1.0 } else { 0.0 };
        let hk = (1.0 - (-dt * 18.0).exp()).clamp(0.0, 1.0);
        let sk = if dragging {
            1.0
        } else {
            (1.0 - (-dt * 24.0).exp()).clamp(0.0, 1.0)
        };
        let slot = self.slot_mut(id);
        slot.hover += (hover_target - slot.hover) * hk;
        if !slot.slider_init {
            slot.slider = value;
            slot.slider_init = true;
        } else {
            slot.slider += (value - slot.slider) * sk;
            if (slot.slider - value).abs() < 0.002 {
                slot.slider = value;
            }
        }
        (slot.slider, slot.hover)
    }

    /// Trigger a crimson ripple pulse on the clicked button slot.
    pub fn pulse(&mut self, id: u64) {
        self.slot_mut(id).pulse = 1.0;
    }
}

/// Apply `(dx, dy)` and `alpha` transition transform to vertices emitted since `start`.
pub fn apply_transition(batch: &mut HudBatch, start: usize, dx: f32, dy: f32, alpha: f32) {
    if dx == 0.0 && dy == 0.0 && alpha >= 0.999 {
        return;
    }
    for v in &mut batch.vertices[start..] {
        v.pos[0] += dx;
        v.pos[1] += dy;
        if alpha < 0.999 {
            v.color = scale_alpha(v.color, alpha);
        }
    }
}

/// Draw a 1px crisp inner-framed rectangle (`fill` + 4 border edges).
pub fn draw_card_box(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    (x, y, w, h): (f32, f32, f32, f32),
    fill: u32,
    border: u32,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    batch.fill(atlas, x, y, w, h, fill);
    if w >= 2.0 && h >= 2.0 {
        batch.fill(atlas, x, y, w, 1.0, border);
        batch.fill(atlas, x, y + h - 1.0, w, 1.0, border);
        batch.fill(atlas, x, y + 1.0, 1.0, h - 2.0, border);
        batch.fill(atlas, x + w - 1.0, y + 1.0, 1.0, h - 2.0, border);
    }
}

/// Full-screen Volt Launcher obsidian backdrop with subtle crimson header aura.
pub fn menu_backdrop(batch: &mut HudBatch, atlas: &HudAtlas, w: f32, h: f32, in_world: bool) {
    if in_world {
        batch.fill_gradient(
            atlas,
            (0.0, 0.0, w, h),
            hex_rgba(BG_HEX, 214),
            hex_rgba(BACKDROP_HEX, 232),
        );
    } else {
        batch.fill_gradient(
            atlas,
            (0.0, 0.0, w, h),
            hex_rgba(BG_HEX, 255),
            hex_rgba(RAIL_HEX, 255),
        );
    }
    // Subtle crimson ambient top wash matching Volt Launcher's hero header glow.
    let glow_h = h.min(48.0);
    batch.fill_gradient(
        atlas,
        (0.0, 0.0, w, glow_h),
        hex_rgba(CRIMSON_HEX, if in_world { 28 } else { 36 }),
        hex_rgba(CRIMSON_HEX, 0),
    );
}

/// Recessed dark obsidian scrollable list well between `top` and `bottom`.
pub fn list_well(batch: &mut HudBatch, atlas: &HudAtlas, top: f32, bottom: f32, in_world: bool) {
    if bottom <= top {
        return;
    }
    let w = batch.width;
    batch.fill_gradient(
        atlas,
        (0.0, top, w, bottom - top),
        hex_rgba(BACKDROP_HEX, if in_world { 168 } else { 228 }),
        hex_rgba(BG_HEX, if in_world { 184 } else { 236 }),
    );
}

/// Solid dark obsidian header (`0..top`) and footer (`bottom..h`) rails drawn
/// behind header titles, search inputs, and footer buttons.
pub fn list_rail_bands(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    top: f32,
    bottom: f32,
    in_world: bool,
) {
    if bottom <= top {
        return;
    }
    let (w, h) = (batch.width, batch.height);
    let rail_alpha = if in_world { 226 } else { 248 };
    if top > 0.0 {
        batch.fill(atlas, 0.0, 0.0, w, top, hex_rgba(RAIL_HEX, rail_alpha));
        batch.fill_gradient(
            atlas,
            (0.0, 0.0, w, top.min(26.0)),
            hex_rgba(CRIMSON_HEX, 26),
            hex_rgba(CRIMSON_HEX, 0),
        );
    }
    if bottom < h {
        batch.fill(
            atlas,
            0.0,
            bottom,
            w,
            h - bottom,
            hex_rgba(RAIL_HEX, rail_alpha),
        );
    }
}

/// Crisp `1px` top/bottom crimson horizon divider lines and subtle inner list shadows.
pub fn list_rail_dividers(batch: &mut HudBatch, atlas: &HudAtlas, top: f32, bottom: f32) {
    if bottom <= top {
        return;
    }
    let w = batch.width;
    let border = hex_rgba(BORDER_HEX, 255);
    batch.fill(atlas, 0.0, top - 1.0, w, 1.0, border);
    batch.fill(atlas, 0.0, bottom, w, 1.0, border);

    // Centered crimson horizon accent along both header and footer dividers.
    let span = (w * 0.46).clamp(120.0, 340.0);
    let cx = (w / 2.0).floor();
    let x0 = cx - (span / 2.0).floor();
    let mid = (span * 0.45).floor();
    for y in [top - 1.0, bottom] {
        batch.fill(atlas, x0, y, span, 1.0, hex_rgba(BORDER_CRIMSON_HEX, 230));
        batch.fill(
            atlas,
            cx - (mid / 2.0).floor(),
            y,
            mid,
            1.0,
            hex_rgba(CRIMSON_HEX, 245),
        );
    }
    // Soft inner shadow just inside the list viewport for depth.
    let shadow_h = (bottom - top).min(5.0);
    batch.fill_gradient(
        atlas,
        (0.0, top, w, shadow_h),
        rgba(0, 0, 0, 110),
        rgba(0, 0, 0, 0),
    );
    batch.fill_gradient(
        atlas,
        (0.0, bottom - shadow_h, w, shadow_h),
        rgba(0, 0, 0, 0),
        rgba(0, 0, 0, 95),
    );
}

/// Header & footer rails with crisp dark borders and centered crimson horizon glow.
pub fn list_rails(batch: &mut HudBatch, atlas: &HudAtlas, top: f32, bottom: f32, in_world: bool) {
    list_rail_bands(batch, atlas, top, bottom, in_world);
    list_rail_dividers(batch, atlas, top, bottom);
}

/// Page title with an animated Volt crimson accent bar beneath it.
pub fn draw_page_title(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    title: &Text,
    y: f32,
    underline_progress: f32,
) {
    let w = batch.width;
    let tw = atlas.font.width_text(title);
    let tx = ((w - tw) / 2.0).floor();
    batch.text_styled(atlas, title, tx, y, TEXT_COLOR, true);
    let bar_full = (tw * 0.55).clamp(24.0, 68.0);
    let bar_w = (bar_full * underline_progress.clamp(0.0, 1.0)).round();
    if bar_w >= 2.0 {
        let bx = ((w - bar_w) / 2.0).floor();
        batch.fill(atlas, bx, y + 10.0, bar_w, 1.0, CRIMSON_COLOR);
    }
}

/// Centered section heading for `OptionsUi` lists (replaces bold-underlined white text).
pub fn draw_section_header(batch: &mut HudBatch, atlas: &HudAtlas, label: &Text, y: f32) {
    let w = batch.width;
    // Strip legacy underline so headings look crisp & modern instead of noisy.
    let clean = Text::literal(label.plain()).with_style(&Style {
        bold: Some(true),
        ..Style::default()
    });
    let tw = atlas.font.width_text(&clean);
    let tx = ((w - tw) / 2.0).floor();
    let wing = 22.0;
    let wy = y + 4.0;
    if tx > wing + 10.0 {
        batch.fill(
            atlas,
            tx - wing - 6.0,
            wy,
            wing,
            1.0,
            hex_rgba(BORDER_CRIMSON_HEX, 210),
        );
        batch.fill(atlas, tx - 8.0, wy, 2.0, 1.0, CRIMSON_COLOR);
        batch.fill(
            atlas,
            tx + tw + 6.0,
            wy,
            wing,
            1.0,
            hex_rgba(BORDER_CRIMSON_HEX, 210),
        );
        batch.fill(atlas, tx + tw + 6.0, wy, 2.0, 1.0, CRIMSON_COLOR);
    }
    batch.text_styled(atlas, &clean, tx, y, TEXT_COLOR, true);
}

/// Centered label that smoothly marquee-scrolls when wider than `w` instead of
/// chopping words mid-character (`"Low-RAM Resc"` -> full smooth scroll).
pub fn centered_or_marquee_text(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    text: &Text,
    (x, y, w, h): (f32, f32, f32, f32),
    color: u32,
    elapsed_secs: f32,
) {
    if w <= 2.0 {
        return;
    }
    let resolved = text.resolved();
    let tw = atlas.font.width_text(&resolved);
    let ty = y + ((h - 8.0) / 2.0).floor();
    if tw <= w {
        batch.text_styled(
            atlas,
            &resolved,
            x + ((w - tw) / 2.0).floor(),
            ty,
            color,
            true,
        );
        return;
    }
    let overflow = tw - w;
    // Smooth cosine back-and-forth with gentle dwell at both ends (like vanilla's
    // AbstractWidget.renderScrollingString, zero allocations).
    let speed = (overflow / 26.0).clamp(1.2, 4.5);
    let wave = 0.5 - 0.5 * (elapsed_secs * std::f32::consts::PI / speed).cos();
    let shift = (overflow * wave).round();
    let start = batch.vertices.len();
    batch.text_styled(atlas, &resolved, x - shift, ty, color, true);
    batch.clip_since(start, x, y, x + w, y + h);
}

/// Volt Launcher crimson-dark interactive button card.
/// Records widget bounds and cursor state identically to `ui::button`.
#[allow(clippy::too_many_arguments)]
pub fn button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    (x, y, w, h): (f32, f32, f32, f32),
    label: &Text,
    active: bool,
    hover: bool,
    tone: ButtonTone,
    (hover_t, pulse_t): (f32, f32),
    elapsed_secs: f32,
) {
    crate::widgets::record(&label.plain(), [x, y, w, h], active);
    crate::widgets::widget_cursor(hover, active);

    let hover_t = if active { hover_t.clamp(0.0, 1.0) } else { 0.0 };
    let pulse_t = if active { pulse_t.clamp(0.0, 1.0) } else { 0.0 };

    let (idle_bg, hover_bg, idle_border, hover_border) = if !active {
        (
            hex_rgba(0x0B0B10, 220),
            hex_rgba(0x0B0B10, 220),
            hex_rgba(BORDER_SUBTLE_HEX, 210),
            hex_rgba(BORDER_SUBTLE_HEX, 210),
        )
    } else {
        match tone {
            ButtonTone::Hero => (
                hex_rgba(HERO_BG_HEX, 245),
                hex_rgba(HERO_HOVER_HEX, 252),
                hex_rgba(BORDER_CRIMSON_HEX, 255),
                hex_rgba(CRIMSON_BRIGHT_HEX, 255),
            ),
            ButtonTone::Primary => (
                hex_rgba(0x1B0D14, 248),
                hex_rgba(CRIMSON_HEX, 245),
                hex_rgba(BORDER_CRIMSON_HEX, 255),
                hex_rgba(CRIMSON_BRIGHT_HEX, 255),
            ),
            ButtonTone::ActiveOn => (
                hex_rgba(0x160E15, 244),
                hex_rgba(0x26121E, 250),
                hex_rgba(BORDER_CRIMSON_HEX, 235),
                hex_rgba(CRIMSON_BRIGHT_HEX, 255),
            ),
            ButtonTone::ActiveOff | ButtonTone::Standard => (
                hex_rgba(CARD_HEX, 242),
                hex_rgba(HOVER_HEX, 250),
                hex_rgba(BORDER_HEX, 245),
                hex_rgba(CRIMSON_HEX, 255),
            ),
        }
    };

    let mut bg = lerp_color(idle_bg, hover_bg, hover_t);
    if pulse_t > 0.001 {
        bg = lerp_color(bg, hex_rgba(CRIMSON_HEX, 235), pulse_t * 0.42);
    }
    let border = lerp_color(
        idle_border,
        hover_border,
        (hover_t + pulse_t * 0.6).clamp(0.0, 1.0),
    );
    draw_card_box(batch, atlas, (x, y, w, h), bg, border);

    // Subtle top specular highlight inside active cards.
    if active && w > 4.0 && h > 4.0 {
        batch.fill(
            atlas,
            x + 1.0,
            y + 1.0,
            w - 2.0,
            1.0,
            rgba(255, 255, 255, if hover_t > 0.3 { 18 } else { 9 }),
        );
    }

    // Left crimson accent bar: always present on Hero & ActiveOn, expands smoothly on hover!
    if active && h >= 6.0 && w >= 12.0 {
        let base_bar: f32 = match tone {
            ButtonTone::Hero | ButtonTone::ActiveOn => 1.0,
            ButtonTone::Primary => 0.65,
            _ => 0.0,
        };
        let bar_t = base_bar.max(hover_t);
        if bar_t > 0.02 {
            let max_bh = (h - 4.0).max(2.0);
            let bh = (max_bh * bar_t).max(2.0).round();
            let by = y + ((h - bh) / 2.0).floor();
            let bar_col = if hover_t > 0.25 || tone == ButtonTone::Hero {
                CRIMSON_BRIGHT_COLOR
            } else {
                CRIMSON_COLOR
            };
            batch.fill(atlas, x + 1.0, by, 2.0, bh, bar_col);
        }

        // Crisp status pip on the right edge of ON/OFF switches (for wide option buttons).
        if w >= 90.0 {
            match tone {
                ButtonTone::ActiveOn => {
                    let py = y + ((h - 4.0) / 2.0).floor();
                    batch.fill(
                        atlas,
                        x + w - 7.0,
                        py,
                        3.0,
                        4.0,
                        hex_rgba(ONLINE_HEX, 245),
                    );
                }
                ButtonTone::ActiveOff => {
                    let py = y + ((h - 4.0) / 2.0).floor();
                    batch.fill(
                        atlas,
                        x + w - 7.0,
                        py,
                        3.0,
                        4.0,
                        hex_rgba(DIM_HEX, 150),
                    );
                }
                _ => {}
            }
        }
    }

    let text_color = if !active {
        DIM_COLOR
    } else if hover_t > 0.4 || matches!(tone, ButtonTone::Hero | ButtonTone::Primary) {
        rgba(255, 255, 255, 255)
    } else if tone == ButtonTone::ActiveOff {
        lerp_color(hex_rgba(0xD4D4E0, 255), TEXT_COLOR, hover_t)
    } else {
        TEXT_COLOR
    };

    let pad_right = if active && w >= 90.0 && matches!(tone, ButtonTone::ActiveOn | ButtonTone::ActiveOff) {
        10.0
    } else {
        5.0
    };
    let pad_left = 6.0;
    centered_or_marquee_text(
        batch,
        atlas,
        label,
        (x + pad_left, y, (w - pad_left - pad_right).max(2.0), h),
        text_color,
        elapsed_secs,
    );
}

/// Volt Launcher crimson fill-bar slider (`20px` high).
/// Preserves exact `Kind::Slider { value, track: [x + 4.0, w - 8.0] }` geometry.
#[allow(clippy::too_many_arguments)]
pub fn slider(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    (x, y, w): (f32, f32, f32),
    label: &Text,
    value: f32,
    shown_value: f32,
    hover: bool,
    active: bool,
    dragging: bool,
    hover_t: f32,
    elapsed_secs: f32,
) {
    let h = 20.0;
    let value = value.clamp(0.0, 1.0);
    let shown = shown_value.clamp(0.0, 1.0);
    let kind = crate::widgets::Kind::Slider {
        value,
        track: [x + 4.0, w - 8.0],
    };
    crate::widgets::record_as(&label.plain(), [x, y, w, h], active, kind);
    if active {
        crate::widgets::target_as([x, y, w, h], kind);
    }
    crate::widgets::widget_cursor(hover, active);

    let hover_t = if active { hover_t.clamp(0.0, 1.0) } else { 0.0 };
    let bg = if !active {
        hex_rgba(0x0B0B10, 220)
    } else {
        lerp_color(hex_rgba(CARD_HEX, 244), hex_rgba(CARD_INNER_HEX, 250), hover_t)
    };
    let border = if !active {
        hex_rgba(BORDER_SUBTLE_HEX, 210)
    } else if dragging {
        CRIMSON_BRIGHT_COLOR
    } else {
        lerp_color(hex_rgba(BORDER_HEX, 245), CRIMSON_COLOR, hover_t)
    };
    draw_card_box(batch, atlas, (x, y, w, h), bg, border);

    // Thumb geometry matches `ui::slider_value` (`x + (v * (w - 8.0)).round()`, 8px wide).
    let travel = (w - 8.0).max(0.0);
    let tx = x + (shown * travel).round();
    let fill_w = ((tx + 4.0) - (x + 1.0)).clamp(0.0, (w - 2.0).max(0.0));

    if active && fill_w > 0.5 {
        let fill_top = lerp_color(
            hex_rgba(0x6B0F24, 155),
            hex_rgba(0xA81332, 195),
            hover_t,
        );
        let fill_bot = lerp_color(
            hex_rgba(0x470A18, 175),
            hex_rgba(0x7A0E25, 215),
            hover_t,
        );
        batch.fill_gradient(atlas, (x + 1.0, y + 1.0, fill_w, h - 2.0), fill_top, fill_bot);
    }

    // Bottom 2px precision progress rail.
    if w > 6.0 {
        batch.fill(
            atlas,
            x + 2.0,
            y + h - 3.0,
            w - 4.0,
            2.0,
            hex_rgba(0x222230, if active { 230 } else { 130 }),
        );
        if active && fill_w > 1.0 {
            batch.fill(
                atlas,
                x + 2.0,
                y + h - 3.0,
                fill_w.min(w - 4.0),
                2.0,
                if dragging || hover_t > 0.35 {
                    CRIMSON_BRIGHT_COLOR
                } else {
                    CRIMSON_COLOR
                },
            );
        }
    }

    // Sleek 6px crimson thumb centered on `tx + 4.0`.
    let thumb_col = if !active {
        hex_rgba(0x3A3A4A, 220)
    } else if dragging || hover_t > 0.35 {
        CRIMSON_BRIGHT_COLOR
    } else {
        CRIMSON_COLOR
    };
    batch.fill(atlas, tx + 1.0, y + 2.0, 6.0, h - 4.0, thumb_col);
    if active {
        batch.fill(
            atlas,
            tx + 3.0,
            y + 5.0,
            2.0,
            h - 10.0,
            rgba(255, 255, 255, if dragging { 245 } else { 190 }),
        );
    }

    let text_color = if !active {
        DIM_COLOR
    } else if hover_t > 0.3 || dragging {
        rgba(255, 255, 255, 255)
    } else {
        TEXT_COLOR
    };
    centered_or_marquee_text(
        batch,
        atlas,
        label,
        (x + 6.0, y, (w - 12.0).max(2.0), h),
        text_color,
        elapsed_secs,
    );
}

/// Sleek Volt obsidian-crimson EditBox wrapper (preserves `EditBox::draw` caret,
/// selection, `widgets::target_as(Kind::Field)`, and `preedit::report_caret`).
pub fn draw_edit_box(batch: &mut HudBatch, atlas: &HudAtlas, edit: &mut ui::EditBox) {
    let (x, y, w, h) = (edit.x, edit.y, edit.width, edit.height);
    let bg = if edit.focused {
        hex_rgba(CARD_INNER_HEX, 250)
    } else {
        hex_rgba(0x0E0E15, 242)
    };
    let border = if edit.focused {
        CRIMSON_COLOR
    } else {
        hex_rgba(BORDER_HEX, 245)
    };
    draw_card_box(batch, atlas, (x, y, w, h), bg, border);
    // Left crimson search accent bar + bottom focus underline.
    batch.fill(
        atlas,
        x + 1.0,
        y + 2.0,
        2.0,
        (h - 4.0).max(1.0),
        if edit.focused {
            CRIMSON_BRIGHT_COLOR
        } else {
            hex_rgba(BORDER_CRIMSON_HEX, 230)
        },
    );
    if edit.focused && w > 4.0 {
        batch.fill(atlas, x + 1.0, y + h - 1.0, w - 2.0, 1.0, CRIMSON_BRIGHT_COLOR);
    }
    edit.draw_custom_framed(batch, atlas, None);
}

/// Volt Launcher crimson scrollbar matching `ui::scrollbar_thumb` geometry (`6px` wide at `x`).
pub fn scrollbar(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    top: f32,
    bottom: f32,
    content: f32,
    scroll: f32,
    active_or_hover: bool,
) {
    let Some((bar_y, bar_h)) = ui::scrollbar_thumb(top, bottom, content, scroll) else {
        return;
    };
    draw_card_box(
        batch,
        atlas,
        (x, top, 6.0, bottom - top),
        hex_rgba(0x0A0A10, 225),
        hex_rgba(0x1F1F2D, 230),
    );
    let thumb = if active_or_hover {
        CRIMSON_BRIGHT_COLOR
    } else {
        CRIMSON_COLOR
    };
    batch.fill(atlas, x + 1.0, bar_y, 4.0, bar_h, thumb);
    if bar_h >= 6.0 {
        batch.fill(
            atlas,
            x + 1.0,
            bar_y,
            4.0,
            1.0,
            rgba(255, 160, 180, if active_or_hover { 220 } else { 150 }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_rgba_packs_little_endian_abgr_channels_for_hud_batch() {
        let crimson = hex_rgba(CRIMSON_HEX, 255);
        assert_eq!(crimson, rgba(0xDC, 0x14, 0x3C, 255));
        assert_eq!(crimson.to_le_bytes(), [0xDC, 0x14, 0x3C, 255]);
    }

    #[test]
    fn toggle_tone_detects_english_and_russian_on_off_suffixes() {
        assert_eq!(infer_toggle_tone("Enable Shaders: ON"), ButtonTone::ActiveOn);
        assert_eq!(infer_toggle_tone("Entity Culling: On"), ButtonTone::ActiveOn);
        assert_eq!(infer_toggle_tone("VSync: OFF"), ButtonTone::ActiveOff);
        assert_eq!(infer_toggle_tone("Smart Leaf Cull: Off"), ButtonTone::ActiveOff);
        assert_eq!(infer_toggle_tone("Шейдеры: Вкл"), ButtonTone::ActiveOn);
        assert_eq!(infer_toggle_tone("Шейдеры: Выкл"), ButtonTone::ActiveOff);
        assert_eq!(infer_toggle_tone("Boost Profile: Balanced"), ButtonTone::Standard);
    }

    #[test]
    fn option_anim_settles_and_interpolates_cleanly() {
        let mut anim = OptionAnim::default();
        assert_eq!(anim.transition_state(), (0.0, 0.0, 1.0, 1.0));
        anim.start_transition(1.0);
        let (dx, dy, alpha, _) = anim.transition_state();
        assert!(dx > 0.0 && dy == 0.0 && alpha >= 0.38);
        anim.settle_transition();
        assert_eq!(anim.transition_state(), (0.0, 0.0, 1.0, 1.0));

        let dt = anim.begin_frame();
        let (h1, _) = anim.step_button(42, true, dt);
        assert!(h1 > 0.0 && h1 <= 1.0);
        anim.pulse(42);
        let (_, p) = anim.step_button(42, true, dt);
        assert!(p > 0.0);
    }
}
