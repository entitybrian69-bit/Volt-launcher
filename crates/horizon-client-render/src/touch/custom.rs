//! User-added and imported Java launcher touch keys.
use super::*;
use layout::Placement;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_ratio() -> f32 {
    1.0
}

fn is_default_ratio(v: &f32) -> bool {
    (*v - 1.0).abs() < 0.0001
}

fn is_false(v: &bool) -> bool {
    !*v
}

fn is_true(v: &bool) -> bool {
    *v
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomButton {
    pub key: String,
    pub layouts: [Placement; 3],
    /// Additional simultaneous tokens pressed with `key` (e.g. Pojav 4-key combos).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_keys: Vec<String>,
    /// Explicit display caption from an imported control layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Width divided by height (`1.0` = square button).
    #[serde(default = "default_ratio", skip_serializing_if = "is_default_ratio")]
    pub width_ratio: f32,
    /// Latching toggle mode (`isToggle` / `autoKeep`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub toggle: bool,
    /// Finger sliding across swipeable buttons switches active keys (`isSwipeable`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub swipeable: bool,
    /// Dragging on this button also turns the camera (`passThruEnabled` / `movable`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub passthrough: bool,
    /// Shown during active gameplay (`displayInGame`).
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub in_game: bool,
    /// Shown on inventory/container screens (`displayInMenu`).
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub in_menu: bool,
    /// Imported fill RGBA `[r, g, b, a]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<[u8; 4]>,
    /// Imported border RGBA `[r, g, b, a]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<[u8; 4]>,
    /// Optional rounded corner radius in logical pixels (`0.0..=24.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f32>,
    /// Parent drawer group ID when this button belongs to an expandable drawer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drawer_parent: Option<u16>,
}

pub(super) struct CustomFinger {
    pub tokens: Vec<String>,
    pub button_index: Option<usize>,
    pub swipeable: bool,
    pub passthrough: bool,
    pub start: [f32; 2],
    pub previous: [f32; 2],
    pub dragged: bool,
}

impl CustomButton {
    pub fn new(key: String, index: usize) -> Self {
        Self {
            key,
            layouts: [Placement::new(
                0.68 + (index % 3) as f32 * 0.1,
                0.35 + (index / 3 % 3) as f32 * 0.12,
                0.105,
            ); 3],
            extra_keys: Vec::new(),
            label: None,
            width_ratio: 1.0,
            toggle: false,
            swipeable: false,
            passthrough: false,
            in_game: true,
            in_menu: true,
            bg_color: None,
            stroke_color: None,
            corner_radius: None,
            drawer_parent: None,
        }
    }

    pub fn valid_token(token: &str) -> bool {
        token == "key.keyboard.unknown"
            || matches!(
                token,
                "special:keyboard"
                    | "special:gui_toggle"
                    | "special:mouse_pri"
                    | "special:mouse_sec"
                    | "special:mouse_mid"
                    | "special:virtual_mouse"
                    | "special:scroll_up"
                    | "special:scroll_down"
                    | "special:menu"
            )
            || token
                .strip_prefix("special:drawer:")
                .is_some_and(|id| id.parse::<u16>().is_ok())
            || crate::keybinds::key_code(token).is_some()
            || token
                .strip_prefix("action:")
                .is_some_and(|name| crate::keybinds::MAPPINGS.iter().any(|m| m.0 == name))
    }

    pub fn sanitize(&mut self, imported: bool) {
        self.width_ratio = if self.width_ratio.is_finite() {
            self.width_ratio.clamp(0.2, 8.0)
        } else {
            1.0
        };
        if let Some(r) = &mut self.corner_radius {
            if r.is_finite() && *r > 0.25 {
                *r = r.clamp(0.5, 24.0);
            } else {
                self.corner_radius = None;
            }
        }
        self.extra_keys.retain(|k| Self::valid_token(k) && k != "key.keyboard.unknown");
        self.extra_keys.truncate(8);
        if let Some(label) = &mut self.label {
            if label.len() > 48 {
                let mut end = 48;
                while !label.is_char_boundary(end) {
                    end -= 1;
                }
                label.truncate(end);
            }
        }
        for p in &mut self.layouts {
            if imported {
                p.sanitize_imported();
            } else {
                p.sanitize();
            }
        }
    }

    pub fn label(&self) -> String {
        if let Some(custom) = self.label.as_deref().map(str::trim)
            && !custom.is_empty()
        {
            return custom.to_owned();
        }
        Self::token_label(&self.key)
    }

    pub fn token_label(token: &str) -> String {
        match token {
            "key.keyboard.unknown" => "Button".into(),
            "special:keyboard" => "KBD".into(),
            "special:gui_toggle" => "GUI".into(),
            "special:mouse_pri" => "PRI".into(),
            "special:mouse_sec" => "SEC".into(),
            "special:mouse_mid" => "MID".into(),
            "special:virtual_mouse" => "Mouse".into(),
            "special:scroll_up" => "Scrl+".into(),
            "special:scroll_down" => "Scrl-".into(),
            "special:menu" => "Menu".into(),
            _ if token.starts_with("special:drawer:") => "Drawer".into(),
            _ => {
                if let Some(name) = token.strip_prefix("action:") {
                    crate::text::Text::translate(name, &[]).plain()
                } else {
                    crate::keybinds::display_name(token).plain()
                }
            }
        }
    }

    fn is_one_shot_token(token: &str) -> bool {
        matches!(
            token,
            "special:keyboard"
                | "special:virtual_mouse"
                | "special:scroll_up"
                | "special:scroll_down"
                | "special:menu"
        )
    }
}

pub(super) fn choices(raw_keys: bool) -> Vec<(String, String)> {
    if raw_keys {
        crate::keybinds::available_keys()
            .iter()
            .map(|(name, _)| ((*name).into(), crate::keybinds::display_name(name).plain()))
            .collect()
    } else {
        crate::keybinds::MAPPINGS
            .iter()
            .filter(|m| m.1 != "key.keyboard.unknown")
            .map(|(name, _, _)| {
                (
                    format!("action:{name}"),
                    crate::text::Text::translate(name, &[]).plain(),
                )
            })
            .collect()
    }
}

impl TouchControls {
    pub(super) fn inventory_token(control: Control) -> Option<&'static str> {
        match control {
            Control::Sneak => Some("key.keyboard.left.shift"),
            Control::Attack => Some("action:key.attack"),
            Control::Use => Some("action:key.use"),
            _ => None,
        }
    }

    pub fn inventory_update(
        &mut self,
        id: u64,
        point: [f32; 2],
        started: bool,
        ended: bool,
    ) -> bool {
        if self.custom_fingers.contains_key(&id) {
            return self.custom_update(id, point, false, ended).is_some();
        }
        if started && self.custom_at(point).is_some() {
            return self.custom_update(id, point, true, false).is_some();
        }
        if started
            && let Some(token) = self.control_at(point).and_then(Self::inventory_token)
        {
            self.custom_fingers.insert(
                id,
                CustomFinger {
                    tokens: vec![token.into()],
                    button_index: None,
                    swipeable: false,
                    passthrough: false,
                    start: point,
                    previous: point,
                    dragged: false,
                },
            );
            self.fingers.insert(id, (Role::Ignore, point));
            self.sync_custom_tokens();
            return true;
        }
        false
    }

    pub(super) fn fit_placement(&self, mut p: Placement) -> Placement {
        p.size = (p.size * self.settings.button_scale)
            .min(self.aspect())
            .min(1.0);
        let half = [p.size / self.aspect() / 2.0, p.size / 2.0];
        for (i, half) in half.into_iter().enumerate() {
            p.center[i] = p.center[i].clamp(half, 1.0 - half);
        }
        p
    }

    pub(super) fn custom_placement(&self, index: usize) -> Placement {
        let mut p = self.settings.custom_buttons[index].layouts[self.settings.mode.index()];
        if self.settings.left_handed {
            p.center[0] = 1.0 - p.center[0];
        }
        self.fit_placement(p)
    }

    pub(super) fn custom_rect(&self, index: usize) -> [f32; 4] {
        let button = &self.settings.custom_buttons[index];
        if is_default_ratio(&button.width_ratio) {
            return self.custom_placement(index).rect(self.aspect());
        }
        let mut center = button.layouts[self.settings.mode.index()].center;
        if self.settings.left_handed {
            center[0] = 1.0 - center[0];
        }
        let raw_size = button.layouts[self.settings.mode.index()].size * self.settings.button_scale;
        let h = raw_size.min(1.0);
        let w = (h * button.width_ratio.clamp(0.2, 8.0) / self.aspect().max(0.1)).min(1.0);
        let cx = center[0].clamp(w / 2.0, 1.0 - w / 2.0);
        let cy = center[1].clamp(h / 2.0, 1.0 - h / 2.0);
        [cx - w / 2.0, cy - h / 2.0, w, h]
    }

    pub(super) fn custom_visible(&self, index: usize) -> bool {
        if self.editor.is_some() {
            return true;
        }
        let Some(button) = self.settings.custom_buttons.get(index) else {
            return false;
        };
        if self.controls_hidden && button.key != "special:gui_toggle" {
            return false;
        }
        if let Some(drawer_id) = button.drawer_parent
            && !self.open_drawers.contains(&drawer_id)
        {
            return false;
        }
        if self.inventory_mode {
            button.in_menu
        } else {
            button.in_game
        }
    }

    pub fn custom_at(&self, point: [f32; 2]) -> Option<usize> {
        (0..self.settings.custom_buttons.len())
            .rev()
            .find(|&i| self.custom_visible(i) && layout::contains(self.custom_rect(i), point))
    }

    fn sync_custom_tokens(&mut self) {
        let mut desired: Vec<String> = Vec::new();
        for tokens in self.toggled_custom.values() {
            for token in tokens {
                if !desired.iter().any(|existing| existing == token) {
                    desired.push(token.clone());
                }
            }
        }
        for finger in self.custom_fingers.values() {
            for token in &finger.tokens {
                if !desired.iter().any(|existing| existing == token) {
                    desired.push(token.clone());
                }
            }
        }
        for old in self.active_custom_tokens.iter().rev() {
            if !desired.iter().any(|next| next == old) {
                self.custom_events.push((old.clone(), false));
            }
        }
        for next in &desired {
            if !self.active_custom_tokens.iter().any(|old| old == next) {
                self.custom_events.push((next.clone(), true));
            }
        }
        self.active_custom_tokens = desired;
    }

    fn trigger_button_press(&mut self, index: usize) -> Vec<String> {
        let button = &self.settings.custom_buttons[index];
        let is_toggle = button.toggle;
        let mut all_tokens = Vec::with_capacity(1 + button.extra_keys.len());
        if button.key != "key.keyboard.unknown" {
            all_tokens.push(button.key.clone());
        }
        for extra in &button.extra_keys {
            if extra != "key.keyboard.unknown" && !all_tokens.iter().any(|t| t == extra) {
                all_tokens.push(extra.clone());
            }
        }
        let mut holdable = Vec::new();
        for token in all_tokens {
            if token == "special:gui_toggle" {
                self.controls_hidden = !self.controls_hidden;
                if self.controls_hidden {
                    self.toggled_custom.clear();
                }
            } else if let Some(drawer_id) = token
                .strip_prefix("special:drawer:")
                .and_then(|s| s.parse::<u16>().ok())
            {
                if !self.open_drawers.remove(&drawer_id) {
                    self.open_drawers.insert(drawer_id);
                } else {
                    let buttons = &self.settings.custom_buttons;
                    self.toggled_custom.retain(|&idx, _| {
                        buttons
                            .get(idx)
                            .and_then(|b| b.drawer_parent)
                            != Some(drawer_id)
                    });
                }
            } else if CustomButton::is_one_shot_token(&token) {
                self.custom_events.push((token.clone(), true));
                self.custom_events.push((token, false));
            } else {
                holdable.push(token);
            }
        }
        if is_toggle && !holdable.is_empty() {
            if self.toggled_custom.remove(&index).is_none() {
                self.toggled_custom.insert(index, holdable);
            }
            Vec::new()
        } else {
            holdable
        }
    }

    pub(super) fn custom_update(
        &mut self,
        id: u64,
        point: [f32; 2],
        started: bool,
        ended: bool,
    ) -> Option<[f32; 2]> {
        if self.custom_fingers.contains_key(&id) {
            if ended {
                self.custom_fingers.remove(&id);
                self.fingers.remove(&id);
                self.sync_custom_tokens();
                return Some([0.0; 2]);
            }
            let current_info = self
                .custom_fingers
                .get(&id)
                .map(|f| (f.swipeable, f.button_index));
            if let Some((true, prev_idx)) = current_info {
                let hovered = self.custom_at(point);
                if hovered != prev_idx {
                    if let Some(next_idx) = hovered
                        && self.settings.custom_buttons[next_idx].swipeable
                    {
                        let next_passthrough = self.settings.custom_buttons[next_idx].passthrough;
                        let next_tokens = self.trigger_button_press(next_idx);
                        if let Some(finger) = self.custom_fingers.get_mut(&id) {
                            finger.button_index = Some(next_idx);
                            finger.tokens = next_tokens;
                            finger.passthrough |= next_passthrough;
                        }
                        self.clicked = true;
                        self.sync_custom_tokens();
                    } else if hovered.is_none() {
                        if let Some(finger) = self.custom_fingers.get_mut(&id)
                            && !finger.tokens.is_empty()
                        {
                            finger.button_index = None;
                            finger.tokens.clear();
                            self.sync_custom_tokens();
                        }
                    }
                }
            }
            let aspect = self.aspect();
            let inventory = self.inventory_mode;
            if let Some(finger) = self.custom_fingers.get_mut(&id) {
                let delta = if finger.passthrough && !inventory {
                    finger.dragged |= ((point[0] - finger.start[0]) * aspect)
                        .hypot(point[1] - finger.start[1])
                        > 0.008;
                    if finger.dragged {
                        [point[0] - finger.previous[0], point[1] - finger.previous[1]]
                    } else {
                        [0.0; 2]
                    }
                } else {
                    [0.0; 2]
                };
                finger.previous = point;
                if let Some((_, p)) = self.fingers.get_mut(&id) {
                    *p = point;
                }
                return Some(delta);
            }
            return Some([0.0; 2]);
        }
        if started && let Some(index) = self.custom_at(point) {
            let button = &self.settings.custom_buttons[index];
            if button.key == "key.keyboard.unknown" && button.extra_keys.is_empty() {
                return None;
            }
            let swipeable = button.swipeable;
            let passthrough = button.passthrough;
            let tokens = self.trigger_button_press(index);
            self.custom_fingers.insert(
                id,
                CustomFinger {
                    tokens,
                    button_index: Some(index),
                    swipeable,
                    passthrough,
                    start: point,
                    previous: point,
                    dragged: false,
                },
            );
            self.fingers.insert(id, (Role::Ignore, point));
            self.sync_custom_tokens();
            self.clicked = true;
            return Some([0.0; 2]);
        }
        None
    }

    pub fn held_custom_tokens(&self) -> impl Iterator<Item = &str> {
        self.active_custom_tokens.iter().map(String::as_str)
    }

    pub fn key_events(&mut self) -> Vec<(String, bool)> {
        std::mem::take(&mut self.custom_events)
    }

    pub(super) fn clear_custom(&mut self) {
        self.custom_fingers.clear();
        self.toggled_custom.clear();
        self.sync_custom_tokens();
    }

    fn custom_button_pressed(&self, index: usize, button: &CustomButton) -> bool {
        if self.toggled_custom.contains_key(&index) {
            return true;
        }
        if let Some(drawer_id) = button
            .key
            .strip_prefix("special:drawer:")
            .and_then(|s| s.parse::<u16>().ok())
        {
            return self.open_drawers.contains(&drawer_id);
        }
        if self
            .custom_fingers
            .values()
            .any(|f| f.button_index == Some(index))
        {
            return true;
        }
        button.key != "key.keyboard.unknown"
            && self
                .active_custom_tokens
                .iter()
                .any(|token| token == &button.key)
    }

    pub(super) fn draw_custom(&self, b: &mut HudBatch, a: &HudAtlas) {
        for (i, button) in self.settings.custom_buttons.iter().enumerate() {
            if !self.custom_visible(i) {
                continue;
            }
            let p = self.settings.custom_buttons[i].layouts[self.settings.mode.index()];
            let norm = self.custom_rect(i);
            let rect = [
                norm[0] * b.width,
                norm[1] * b.height,
                norm[2] * b.width,
                norm[3] * b.height,
            ];
            let pressed = self.custom_button_pressed(i, button);
            let mut opacity = if self.editor.is_some() {
                p.opacity.max(0.25)
            } else if self.controls_hidden && button.key == "special:gui_toggle" {
                p.opacity.clamp(0.22, 0.55)
            } else {
                p.opacity
            };
            opacity = opacity.clamp(0.0, 1.0);
            if opacity <= 0.0 {
                continue;
            }
            let text_color = if button.bg_color.is_some()
                || button.stroke_color.is_some()
                || button.corner_radius.is_some()
            {
                let bg = button.bg_color.unwrap_or([18, 18, 24, 138]);
                let fill_alpha = ((f32::from(bg[3]) / 255.0) * opacity * 255.0).round() as u8;
                let fill = if pressed {
                    rgba(
                        bg[0].saturating_add(55).max(160),
                        bg[1].saturating_add(24),
                        bg[2].saturating_add(34),
                        fill_alpha.max((opacity * 175.0) as u8),
                    )
                } else {
                    rgba(bg[0], bg[1], bg[2], fill_alpha)
                };
                let stroke = if pressed {
                    rgba(255, 68, 96, (opacity * 235.0) as u8)
                } else if let Some(sc) = button.stroke_color
                    && sc[3] > 0
                {
                    let sa = ((f32::from(sc[3]) / 255.0) * opacity * 255.0).round() as u8;
                    rgba(sc[0], sc[1], sc[2], sa)
                } else {
                    rgba(255, 255, 255, (opacity * 95.0) as u8)
                };
                let rad = button
                    .corner_radius
                    .unwrap_or(0.0)
                    .min(rect[2] * 0.48)
                    .min(rect[3] * 0.48);
                if rad >= 2.0 {
                    draw_rounded_hud_box(b, a, rect, rad, fill, stroke);
                } else {
                    b.fill(a, rect[0], rect[1], rect[2], rect[3], fill);
                    let t = 1.0;
                    b.fill(a, rect[0], rect[1], rect[2], t, stroke);
                    b.fill(a, rect[0], rect[1] + rect[3] - t, rect[2], t, stroke);
                    b.fill(a, rect[0], rect[1], t, rect[3], stroke);
                    b.fill(a, rect[0] + rect[2] - t, rect[1], t, rect[3], stroke);
                }
                rgba(248, 248, 252, (opacity * 255.0) as u8)
            } else {
                let tint = if pressed {
                    rgba(170, 210, 150, (opacity * 255.0) as u8)
                } else {
                    rgba(255, 255, 255, (opacity * 255.0) as u8)
                };
                if !theme_button(
                    b,
                    a,
                    "button",
                    rect,
                    pressed,
                    rgba(255, 255, 255, (opacity * 255.0) as u8),
                    true,
                ) {
                    nine(b, a, "ui/button_borderless_light", 2.0, rect, tint);
                }
                rgba(25, 25, 25, (opacity * 255.0) as u8)
            };
            let label = button.label();
            let width = HudBatch::text_width(a, &label);
            let start = b.vertices.len();
            let shadow = button.bg_color.is_some()
                || button.stroke_color.is_some()
                || button.corner_radius.is_some();
            b.text(
                a,
                &label,
                rect[0] + (rect[2] - width) / 2.0,
                rect[1] + (rect[3] - 8.0) / 2.0,
                text_color,
                shadow,
            );
            let scale = ((rect[2] - 6.0).max(1.0) / width.max(1.0))
                .min(((rect[3] - 6.0).max(1.0) / 8.0).min(1.0));
            b.scale_since(
                start,
                rect[0] + rect[2] / 2.0,
                rect[1] + rect[3] / 2.0,
                scale,
                scale,
            );
        }
    }
}

fn draw_rounded_hud_box(
    b: &mut HudBatch,
    a: &HudAtlas,
    rect: [f32; 4],
    radius: f32,
    fill: u32,
    stroke: u32,
) {
    let [x, y, w, h] = rect;
    let r = radius.floor().clamp(2.0, (w.min(h) * 0.48).max(2.0)) as i32;
    let rf = r as f32;
    let body_h = (h - 2.0 * rf).max(0.0);
    if body_h > 0.0 {
        b.fill(a, x, y + rf, w, body_h, fill);
        b.fill(a, x, y + rf, 1.0, body_h, stroke);
        b.fill(a, x + w - 1.0, y + rf, 1.0, body_h, stroke);
    }
    for row in 0..r {
        let dy = rf - (row as f32 + 0.5);
        let dx = (rf * rf - dy * dy).max(0.0).sqrt();
        let inset = (rf - dx).round().clamp(0.0, w * 0.48);
        let span = (w - 2.0 * inset).max(1.0);
        let ty = y + row as f32;
        let by = y + h - 1.0 - row as f32;
        b.fill(a, x + inset, ty, span, 1.0, fill);
        b.fill(a, x + inset, by, span, 1.0, fill);
        if row == 0 {
            b.fill(a, x + inset, ty, span, 1.0, stroke);
            b.fill(a, x + inset, by, span, 1.0, stroke);
        } else {
            b.fill(a, x + inset, ty, 1.5, 1.0, stroke);
            b.fill(a, x + inset + span - 1.5, ty, 1.5, 1.0, stroke);
            b.fill(a, x + inset, by, 1.5, 1.0, stroke);
            b.fill(a, x + inset + span - 1.5, by, 1.5, 1.0, stroke);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_modifier_and_mouse_hold_independently_and_cancel_cleanly() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.inventory_mode = true;
        t.settings.mode = Mode::Crosshair;
        for (id, control, token) in [
            (1, Control::Sneak, "key.keyboard.left.shift"),
            (2, Control::Attack, "action:key.attack"),
            (3, Control::Use, "action:key.use"),
        ] {
            assert!(t.visible(control));
            assert!(t.inventory_update(id, t.placement(control).center, true, false));
            assert_eq!(t.key_events(), [(token.into(), true)]);
        }
        t.clear_world_gesture();
        assert_eq!(t.held_custom_tokens().count(), 3);
        assert_eq!(t.poll_state(), PadState::default());
        assert!(t.inventory_update(2, [0.0, 0.0], false, true));
        assert_eq!(t.key_events(), [("action:key.attack".into(), false)]);
        t.clear();
        let released = t.key_events();
        assert!(released.contains(&("key.keyboard.left.shift".into(), false)));
        assert!(released.contains(&("action:key.use".into(), false)));
    }
    #[test]
    fn inventory_overlay_preserves_configured_visibility_and_placements() {
        let mut touch = TouchControls::default();
        for mode in Mode::ALL {
            touch.settings.mode = mode;
            touch.settings.button_visibility[Control::Attack as usize] = 2;
            let before: Vec<_> = Control::ALL
                .into_iter()
                .map(|c| (c, touch.visible(c), touch.placement(c).center))
                .collect();
            touch.inventory_mode = true;
            for (control, visible, center) in before {
                assert_eq!(touch.visible(control), visible);
                assert_eq!(touch.placement(control).center, center);
            }
            touch.inventory_mode = false;
        }
    }
    #[test]
    fn unassigned_button_emits_no_input() {
        let mut touch = TouchControls::default();
        touch.viewport(1920, 1080);
        touch
            .settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.unknown".into(), 0));
        let point = touch.custom_placement(0).center;
        touch.update(1, point, true, false);
        assert!(touch.key_events().is_empty());
    }
    #[test]
    fn custom_keys_hold_through_motion_and_shared_keys_release_only_after_last_finger() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.z".into(), 0));
        let p = t.custom_placement(0).center;
        t.update(1, p, true, false);
        t.update(2, p, true, false);
        assert_eq!(t.key_events(), [("key.keyboard.z".into(), true)]);
        t.update(1, [0.99, 0.1], false, false);
        assert!(t.key_events().is_empty());
        t.update(1, p, false, true);
        assert!(t.key_events().is_empty());
        t.update(2, p, false, true);
        assert_eq!(t.key_events(), [("key.keyboard.z".into(), false)]);
    }
    #[test]
    fn cancellation_releases_all_custom_keys_and_scaling_preserves_saved_sizes() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.z".into(), 0));
        let stored = t.settings.custom_buttons[0].layouts[0].size;
        t.settings.button_scale = 2.0;
        assert_eq!(t.custom_placement(0).size, stored * 2.0);
        t.update(1, t.custom_placement(0).center, true, false);
        t.key_events();
        t.clear();
        assert_eq!(t.key_events(), [("key.keyboard.z".into(), false)]);
        assert_eq!(t.settings.custom_buttons[0].layouts[0].size, stored);
    }

    #[test]
    fn toggle_combo_passthrough_swipe_and_drawer_buttons_work_end_to_end() {
        let mut t = TouchControls::default();
        t.viewport(2400, 1080);

        // 0: Toggle + 2-key combo (F3 + B)
        let mut combo = CustomButton::new("key.keyboard.f3".into(), 0);
        combo.extra_keys = vec!["key.keyboard.b".into()];
        combo.toggle = true;
        combo.layouts = [Placement {
            center: [0.2, 0.2],
            size: 0.1,
            opacity: 0.7,
        }; 3];

        // 1: Passthrough PRI button
        let mut pri = CustomButton::new("action:key.attack".into(), 1);
        pri.passthrough = true;
        pri.layouts = [Placement {
            center: [0.8, 0.6],
            size: 0.14,
            opacity: 0.7,
        }; 3];

        // 2 & 3: Swipeable hotbar 1 -> 2
        let mut hb1 = CustomButton::new("key.keyboard.1".into(), 2);
        hb1.swipeable = true;
        hb1.layouts = [Placement {
            center: [0.4, 0.85],
            size: 0.1,
            opacity: 0.7,
        }; 3];
        let mut hb2 = CustomButton::new("key.keyboard.2".into(), 3);
        hb2.swipeable = true;
        hb2.layouts = [Placement {
            center: [0.5, 0.85],
            size: 0.1,
            opacity: 0.7,
        }; 3];

        // 4 & 5: Drawer parent + child
        let mut drawer = CustomButton::new("special:drawer:1".into(), 4);
        drawer.layouts = [Placement {
            center: [0.1, 0.5],
            size: 0.1,
            opacity: 0.7,
        }; 3];
        let mut child = CustomButton::new("key.keyboard.g".into(), 5);
        child.drawer_parent = Some(1);
        child.in_menu = false;
        child.layouts = [Placement {
            center: [0.2, 0.5],
            size: 0.1,
            opacity: 0.7,
        }; 3];

        t.settings.custom_buttons = vec![combo, pri, hb1, hb2, drawer, child];

        // Toggle latches on finger lift and unlatches on second tap
        t.update(1, [0.2, 0.2], true, false);
        assert_eq!(
            t.key_events(),
            [
                ("key.keyboard.f3".into(), true),
                ("key.keyboard.b".into(), true)
            ]
        );
        t.update(1, [0.2, 0.2], false, true);
        assert!(t.key_events().is_empty());
        assert_eq!(t.held_custom_tokens().count(), 2);
        t.update(2, [0.2, 0.2], true, false);
        assert_eq!(
            t.key_events(),
            [
                ("key.keyboard.b".into(), false),
                ("key.keyboard.f3".into(), false)
            ]
        );
        t.update(2, [0.2, 0.2], false, true);

        // Passthrough Attack rotates camera while holding attack
        assert_eq!(t.update(3, [0.8, 0.6], true, false), [0.0, 0.0]);
        assert_eq!(t.key_events(), [("action:key.attack".into(), true)]);
        let look = t.update(3, [0.85, 0.55], false, false);
        assert!(look[0] > 0.04 && look[1] < -0.04);
        t.update(3, [0.85, 0.55], false, true);
        assert_eq!(t.key_events(), [("action:key.attack".into(), false)]);

        // Swipeable switches from 1 -> 2 seamlessly
        t.update(4, [0.4, 0.85], true, false);
        assert_eq!(t.key_events(), [("key.keyboard.1".into(), true)]);
        t.update(4, [0.5, 0.85], false, false);
        assert_eq!(
            t.key_events(),
            [
                ("key.keyboard.1".into(), false),
                ("key.keyboard.2".into(), true)
            ]
        );
        t.update(4, [0.5, 0.85], false, true);
        assert_eq!(t.key_events(), [("key.keyboard.2".into(), false)]);

        // Drawer hides child until expanded, and respects in_menu
        assert_eq!(t.custom_at([0.2, 0.5]), None);
        t.update(5, [0.1, 0.5], true, false);
        t.update(5, [0.1, 0.5], false, true);
        assert_eq!(t.custom_at([0.2, 0.5]), Some(5));
        t.inventory_mode = true;
        assert_eq!(t.custom_at([0.2, 0.5]), None);
    }
}
