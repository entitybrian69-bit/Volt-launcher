//! The look of Volt's graphics settings screens (`Simple`, `Advanced`, `HDR`),
//! matching the Volt Launcher obsidian-dark and crimson-red UI theme with
//! animated hover/press feedback, crimson fill-bar sliders, and cost badges.
use crate::{
    hud::{HudAtlas, HudBatch},
    lang,
    text::{Style as TextStyle, Text},
    volt_options_style::{
        self as volt, BORDER_CRIMSON_HEX, BORDER_HEX, ButtonTone, CRIMSON_BADGE_HEX, CRIMSON_COLOR,
        MUTED_COLOR, TEXT_COLOR, hex_rgba,
    },
};

/// What a setting costs in frame time, shown beside its description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cost {
    None,
    Low,
    Medium,
    High,
    /// Turning it on makes frames faster (culling).
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
}

pub struct Style {
    /// Width of a setting's button or slider.
    pub control_width: f32,
    /// Space between the control and its description.
    pub gap: f32,
    /// Widest description column; narrower on small screens.
    pub max_text_width: f32,
    /// Vertical distance between setting rows.
    pub row_height: f32,
    pub header_height: f32,
    /// Top and bottom bands holding the title and the buttons.
    pub band: f32,
    pub title_color: u32,
    pub header_color: u32,
    pub description_color: u32,
    /// Colour of each `Cost`, in its order.
    pub cost_colors: [u32; 5],
}

pub const STYLE: Style = Style {
    control_width: 150.0,
    gap: 8.0,
    max_text_width: 250.0,
    row_height: 25.0,
    header_height: 16.0,
    band: 33.0,
    title_color: TEXT_COLOR,
    header_color: TEXT_COLOR,
    description_color: MUTED_COLOR,
    cost_colors: [
        hex_rgba(0x9898AA, 255),
        hex_rgba(0x4ADE80, 255),
        hex_rgba(0xFBBF24, 255),
        hex_rgba(0xFF4D6D, 255),
        hex_rgba(0x22C55E, 255),
    ],
};

impl Style {
    /// Width of the description column on a screen `screen_width` wide.
    pub fn text_width(&self, screen_width: f32) -> f32 {
        (screen_width - self.control_width - self.gap - 24.0).clamp(120.0, self.max_text_width)
    }
    /// Left edge of a row's control, centring control and description.
    pub fn row_x(&self, screen_width: f32) -> f32 {
        let total = self.control_width + self.gap + self.text_width(screen_width);
        ((screen_width - total) / 2.0).floor()
    }
    pub fn background(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        top: f32,
        bottom: f32,
        in_world: bool,
    ) {
        let (w, h) = (batch.width, batch.height);
        volt::menu_backdrop(batch, atlas, w, h, in_world);
        volt::list_well(batch, atlas, top, bottom, in_world);
    }
    pub fn frame(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        top: f32,
        bottom: f32,
        in_world: bool,
    ) {
        volt::list_rails(batch, atlas, top, bottom, in_world);
    }
    pub fn title(&self, batch: &mut HudBatch, atlas: &HudAtlas, text: &Text) {
        self.title_animated(batch, atlas, text, 1.0);
    }
    pub fn title_animated(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        text: &Text,
        underline_progress: f32,
    ) {
        volt::draw_page_title(batch, atlas, text, 11.0, underline_progress);
    }
    pub fn header(&self, batch: &mut HudBatch, atlas: &HudAtlas, text: &Text, x: f32, y: f32) {
        let row_w = self.control_width + self.gap + self.text_width(batch.width);
        batch.fill(atlas, x, y + 3.0, 3.0, 9.0, CRIMSON_COLOR);
        let bold = Text::literal(text.plain()).with_style(&TextStyle {
            bold: Some(true),
            ..TextStyle::default()
        });
        let tw = atlas.font.width_text(&bold);
        batch.text_styled(atlas, &bold, x + 7.0, y + 4.0, self.header_color, true);
        let rule_x = x + 12.0 + tw;
        if rule_x + 8.0 < x + row_w {
            batch.fill(
                atlas,
                rule_x,
                y + 8.0,
                x + row_w - rule_x,
                1.0,
                hex_rgba(BORDER_CRIMSON_HEX, 185),
            );
        }
    }
    pub fn button(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        (x, y, w): (f32, f32, f32),
        label: &Text,
        hover: bool,
        active: bool,
    ) {
        let tone = volt::infer_toggle_tone(&label.plain());
        volt::button(
            batch,
            atlas,
            (x, y, w, 20.0),
            label,
            active,
            hover,
            tone,
            (if hover { 1.0 } else { 0.0 }, 0.0),
            0.0,
        );
    }
    #[allow(clippy::too_many_arguments)]
    pub fn button_animated(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        (x, y, w): (f32, f32, f32),
        label: &Text,
        hover: bool,
        active: bool,
        tone: ButtonTone,
        anim_state: (f32, f32),
        elapsed_secs: f32,
    ) {
        volt::button(
            batch,
            atlas,
            (x, y, w, 20.0),
            label,
            active,
            hover,
            tone,
            anim_state,
            elapsed_secs,
        );
    }
    #[allow(clippy::too_many_arguments)]
    pub fn slider(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        (x, y, w): (f32, f32, f32),
        label: &Text,
        value: f32,
        hover: bool,
        active: bool,
        dragging: bool,
    ) {
        volt::slider(
            batch,
            atlas,
            (x, y, w),
            label,
            value,
            value,
            hover && active,
            active,
            dragging && active,
            if hover || dragging { 1.0 } else { 0.0 },
            0.0,
        );
    }
    #[allow(clippy::too_many_arguments)]
    pub fn slider_animated(
        &self,
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
        volt::slider(
            batch,
            atlas,
            (x, y, w),
            label,
            value,
            shown_value,
            hover && active,
            active,
            dragging && active,
            hover_t,
            elapsed_secs,
        );
    }
    /// The cost pill badge and the description beside a control, wrapped to two lines.
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
        let tag_w = atlas.font.width_text(&tag);
        let (pill_bg, pill_border) = match cost {
            Cost::None => (hex_rgba(0x14141D, 225), hex_rgba(BORDER_HEX, 220)),
            Cost::Low => (hex_rgba(0x0E2418, 230), hex_rgba(0x1E5436, 235)),
            Cost::Medium => (hex_rgba(0x281C0B, 230), hex_rgba(0x6B4814, 235)),
            Cost::High => (
                hex_rgba(CRIMSON_BADGE_HEX, 240),
                hex_rgba(BORDER_CRIMSON_HEX, 245),
            ),
            Cost::Saves => (hex_rgba(0x0C2A1A, 235), hex_rgba(0x1F6E43, 240)),
        };
        volt::draw_card_box(
            batch,
            atlas,
            (x, y - 1.0, tag_w + 6.0, 10.0),
            pill_bg,
            pill_border,
        );
        batch.text_styled(
            atlas,
            &tag,
            x + 3.0,
            y,
            self.cost_colors[cost as usize],
            true,
        );
        let indent = tag_w + 10.0;
        let text = lang::get(text);
        let lines = wrap(atlas, &text, self.text_width(batch.width), indent);
        for (i, line) in lines.iter().take(2).enumerate() {
            let lx = if i == 0 { x + indent } else { x };
            batch.text_styled(
                atlas,
                &Text::literal(line.clone()),
                lx,
                y + i as f32 * 10.0,
                self.description_color,
                true,
            );
        }
    }
}

/// Greedy word wrap; the first line is `indent` narrower.
fn wrap(atlas: &HudAtlas, text: &str, width: f32, indent: f32) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in text.split_whitespace() {
        let room = if lines.len() == 1 {
            width - indent
        } else {
            width
        };
        let line = lines.last().expect("one line");
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if atlas.font.width(&candidate) <= room || line.is_empty() {
            *lines.last_mut().expect("one line") = candidate;
        } else {
            lines.push(word.to_owned());
        }
    }
    lines
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
        }
    }
}
