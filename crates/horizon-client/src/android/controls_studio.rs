//! Visual **Controls Studio** for Volt Launcher.
//! Lets players pick ready-to-play built-in layouts (**Volt Default**, **Pojav Classic**,
//! **Zalith Layout**, **Fold Craft / FCL**, **4-Finger PvP Claw**) or customize every button
//! visually on a live 16:9 canvas (drag, snap-to-grid, resize, width ratio, opacity, corner
//! radius, custom text labels, toggle/hold, camera passthrough, swipeable, key/mouse rebind,
//! duplicate, save preset, and `.json`/`.zip` import/export).

use super::pages::{Action, theme};
use horizon_client::{
    osk::Output,
    touch::{
        BuiltInPreset, Control, CustomButton, Settings,
        import::{
            ImportedControlLayout, apply_to_profile, eval_position_formula, save_preset_and_apply,
        },
        layout::{ImportedMeta, MAX_CUSTOM_BUTTONS, Placement},
        presets::STUDIO_BINDINGS,
    },
    ui_theme::{Face, Ui, color::hex},
};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StudioTarget {
    Joystick,
    Builtin(usize),
    Custom(usize),
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum StudioAction {
    Open,
    ApplyAndClose,
    Cancel,
    Preset(BuiltInPreset),
    Select(StudioTarget),
    AddButton,
    DuplicateSelected,
    DeleteSelected,
    ToggleJoystick,
    ToggleSnap,
    CycleBinding(bool),
    ToggleBindPicker,
    PickBinding(usize),
    EditLabel,
    AdjustSize(i8),
    AdjustWidth(i8),
    AdjustOpacity(i8),
    AdjustRadius(i8),
    ToggleHoldLatch,
    TogglePassthrough,
    ToggleSwipeable,
    ToggleInGame,
    ToggleInMenu,
    /// Switches the inspector between its Main and Style/Advanced pages.
    InspectorPage(u8),
    /// Selects one of the four Pojav-style keycode slots and opens the picker.
    BindSlot(usize),
    /// Clears the active extra-key slot (slots 2-4).
    ClearSlot,
    CycleBgColor(bool),
    AdjustBgAlpha(i8),
    CycleStrokeColor(bool),
    AdjustStrokeAlpha(i8),
    /// Adds a Pojav-style drawer (an expandable group of sub-buttons).
    AddDrawer,
    /// Adds a sub-button to the selected drawer.
    AddDrawerSub,
    /// Detaches the selected sub-button from its drawer.
    DetachFromDrawer,
    /// Joystick: W also sprints after a forward flick (Pojav forwardLock).
    ToggleForwardLock,
    /// Joystick: absolute D-pad style vs. free analog (Pojav `absolute`).
    ToggleDpadStyle,
    /// Type a Pojav dynamic-position formula for X / Y.
    EditFormulaX,
    EditFormulaY,
    SavePreset,
    ExportJson,
}

/// The Pojav editor's color choices, cycled through for fills and strokes.
const STUDIO_PALETTE: [[u8; 3]; 12] = [
    [0, 0, 0],
    [34, 34, 48],
    [255, 255, 255],
    [220, 20, 60],
    [255, 68, 96],
    [46, 204, 113],
    [52, 152, 219],
    [241, 196, 15],
    [230, 126, 34],
    [155, 89, 182],
    [26, 188, 156],
    [120, 128, 140],
];

fn cycle_palette(current: Option<[u8; 4]>, back: bool, default_alpha: u8) -> [u8; 4] {
    let (rgb, alpha) = match current {
        Some(c) => ([c[0], c[1], c[2]], c[3]),
        None => (STUDIO_PALETTE[0], default_alpha),
    };
    let at = STUDIO_PALETTE.iter().position(|p| *p == rgb).unwrap_or(0);
    let len = STUDIO_PALETTE.len();
    let next = if back { (at + len - 1) % len } else { (at + 1) % len };
    [
        STUDIO_PALETTE[next][0],
        STUDIO_PALETTE[next][1],
        STUDIO_PALETTE[next][2],
        alpha,
    ]
}

pub(crate) struct ControlsStudio {
    pub open: bool,
    pub settings: Settings,
    pub selected: Option<StudioTarget>,
    pub snap_grid: bool,
    pub bind_picker_open: bool,
    pub editing_label: bool,
    pub label_draft: String,
    /// 0 = Main inspector page, 1 = Style / Advanced.
    pub inspector_page: u8,
    /// Which of the four Pojav keycode slots the picker writes (0..=3).
    pub bind_slot: usize,
    /// `Some(horizontal)` while a dynamic-position formula is being typed.
    pub editing_formula: Option<bool>,
    pub formula_draft: String,
    pub canvas_rect: [f32; 4],
    pub dragging: Option<(StudioTarget, [f32; 2])>,
    pub status_hint: String,
    preset_counter: u32,
}

impl Default for ControlsStudio {
    fn default() -> Self {
        Self {
            open: false,
            settings: Settings::default(),
            selected: None,
            snap_grid: true,
            bind_picker_open: false,
            editing_label: false,
            label_draft: String::new(),
            inspector_page: 0,
            bind_slot: 0,
            editing_formula: None,
            formula_draft: String::new(),
            canvas_rect: [0.0; 4],
            dragging: None,
            status_hint: String::new(),
            preset_counter: 1,
        }
    }
}

impl ControlsStudio {
    pub fn open_for_profile(&mut self, profile_home: &Path) {
        self.settings = Settings::load_from(&profile_home.join("horizon-touch.json"));
        self.open = true;
        self.bind_picker_open = false;
        self.editing_label = false;
        self.inspector_page = 0;
        self.bind_slot = 0;
        self.editing_formula = None;
        self.dragging = None;
        self.select_default_target();
        self.status_hint =
            "Tap a preset above or drag any button on the canvas to customize.".into();
    }

    fn select_default_target(&mut self) {
        if !self.settings.custom_buttons.is_empty() {
            self.selected = Some(StudioTarget::Custom(0));
        } else if self.joystick_visible() {
            self.selected = Some(StudioTarget::Joystick);
        } else {
            self.selected = Some(StudioTarget::Builtin(Control::Jump as usize));
        }
    }

    pub fn joystick_visible(&self) -> bool {
        self.settings.button_visibility[Control::Move as usize] != 2
    }

    fn mark_customized(&mut self) {
        let base_name = self
            .settings
            .imported_from
            .as_ref()
            .map(|m| m.name.as_str())
            .unwrap_or("Volt Custom");
        let custom_name = if base_name.ends_with(" (Custom)") {
            base_name.to_owned()
        } else if BuiltInPreset::ALL.iter().any(|p| p.label() == base_name) {
            format!("{base_name} (Custom)")
        } else {
            base_name.to_owned()
        };
        self.settings.imported_from = Some(ImportedMeta {
            name: custom_name,
            format: "Volt Controls Studio".into(),
            buttons: self.settings.custom_buttons.len(),
            drawers: self.drawer_count(),
            has_joystick: self.joystick_visible(),
        });
    }

    /// Applying a layout with custom buttons makes the stock MCPE touch
    /// controls vanish (visibility 2), exactly like an imported layout. The
    /// pause button stays when no button binds Escape / menu, so the game
    /// can always be paused.
    fn hide_stock_when_custom(&mut self) {
        if self.settings.custom_buttons.is_empty() {
            return;
        }
        let move_idx = Control::Move as usize;
        let move_vis = self.settings.button_visibility[move_idx];
        self.settings.button_visibility = [2; 14];
        self.settings.button_visibility[move_idx] = move_vis;
        let has_menu = self.settings.custom_buttons.iter().any(|b| {
            std::iter::once(&b.key)
                .chain(b.extra_keys.iter())
                .any(|tok| matches!(tok.as_str(), "key.keyboard.escape" | "special:menu"))
        });
        if !has_menu {
            self.settings.button_visibility[Control::Pause as usize] = 1;
        }
    }

    fn drawer_count(&self) -> usize {
        self.settings
            .custom_buttons
            .iter()
            .filter(|b| b.key.starts_with("special:drawer:"))
            .count()
    }

    /// Returns normalized `[x, y, w, h]` within `0..1` for a target at `aspect`.
    fn target_norm_rect(&self, target: StudioTarget, aspect: f32) -> Option<[f32; 4]> {
        let aspect = aspect.max(0.5);
        let mode_idx = self.settings.mode.index();
        match target {
            StudioTarget::Joystick => {
                if !self.joystick_visible() {
                    return None;
                }
                Some(self.settings.layouts[mode_idx][Control::Move as usize].rect(aspect))
            }
            StudioTarget::Builtin(idx) => {
                if idx >= 14 || idx == Control::Move as usize {
                    return None;
                }
                if !self.settings.custom_buttons.is_empty()
                    && self.settings.button_visibility[idx] == 2
                {
                    return None;
                }
                Some(self.settings.layouts[mode_idx][idx].rect(aspect))
            }
            StudioTarget::Custom(idx) => {
                let b = self.settings.custom_buttons.get(idx)?;
                let p = b.layouts[mode_idx];
                let h = p.size;
                let w = (p.size * b.width_ratio.clamp(0.25, 6.0)) / aspect;
                Some([p.center[0] - w * 0.5, p.center[1] - h * 0.5, w, h])
            }
        }
    }

    /// Hit-tests `(ux, uy)` on the studio canvas and starts dragging if a control was hit.
    pub fn pointer_down(&mut self, ux: f32, uy: f32) -> bool {
        if !self.open || self.bind_picker_open {
            return false;
        }
        let [cx, cy, cw, ch] = self.canvas_rect;
        if cw <= 10.0 || ch <= 10.0 || ux < cx || ux > cx + cw || uy < cy || uy > cy + ch {
            return false;
        }
        let nx = (ux - cx) / cw;
        let ny = (uy - cy) / ch;
        let aspect = cw / ch;

        // Topmost custom button wins first
        for idx in (0..self.settings.custom_buttons.len()).rev() {
            let target = StudioTarget::Custom(idx);
            if let Some([rx, ry, rw, rh]) = self.target_norm_rect(target, aspect)
                && nx >= rx
                && nx <= rx + rw
                && ny >= ry
                && ny <= ry + rh
            {
                let center = self.settings.custom_buttons[idx].layouts[self.settings.mode.index()].center;
                self.selected = Some(target);
                self.dragging = Some((target, [center[0] - nx, center[1] - ny]));
                return true;
            }
        }

        // Built-in buttons when visible (e.g. Volt Default layout)
        if self.settings.custom_buttons.is_empty() {
            for control in Control::ALL {
                if control == Control::Move {
                    continue;
                }
                let target = StudioTarget::Builtin(control as usize);
                if let Some([rx, ry, rw, rh]) = self.target_norm_rect(target, aspect)
                    && nx >= rx
                    && nx <= rx + rw
                    && ny >= ry
                    && ny <= ry + rh
                {
                    let center =
                        self.settings.layouts[self.settings.mode.index()][control as usize].center;
                    self.selected = Some(target);
                    self.dragging = Some((target, [center[0] - nx, center[1] - ny]));
                    return true;
                }
            }
        }

        // Movement joystick
        if let Some([rx, ry, rw, rh]) = self.target_norm_rect(StudioTarget::Joystick, aspect)
            && nx >= rx
            && nx <= rx + rw
            && ny >= ry
            && ny <= ry + rh
        {
            let center =
                self.settings.layouts[self.settings.mode.index()][Control::Move as usize].center;
            self.selected = Some(StudioTarget::Joystick);
            self.dragging = Some((StudioTarget::Joystick, [center[0] - nx, center[1] - ny]));
            return true;
        }

        false
    }

    pub fn pointer_move(&mut self, ux: f32, uy: f32) -> bool {
        let Some((target, [ox, oy])) = self.dragging else {
            return false;
        };
        let [cx, cy, cw, ch] = self.canvas_rect;
        if cw <= 10.0 || ch <= 10.0 {
            return false;
        }
        let mut nx = ((ux - cx) / cw + ox).clamp(0.035, 0.965);
        let mut ny = ((uy - cy) / ch + oy).clamp(0.045, 0.955);
        if self.snap_grid {
            nx = (nx * 40.0).round() / 40.0;
            ny = (ny * 40.0).round() / 40.0;
            nx = nx.clamp(0.035, 0.965);
            ny = ny.clamp(0.045, 0.955);
        }
        match target {
            StudioTarget::Joystick => {
                for row in &mut self.settings.layouts {
                    row[Control::Move as usize].center = [nx, ny];
                }
            }
            StudioTarget::Builtin(idx) if idx < 14 => {
                for row in &mut self.settings.layouts {
                    row[idx].center = [nx, ny];
                }
            }
            StudioTarget::Custom(idx) => {
                if let Some(btn) = self.settings.custom_buttons.get_mut(idx) {
                    for p in &mut btn.layouts {
                        p.center = [nx, ny];
                    }
                    self.mark_customized();
                }
            }
            _ => {}
        }
        true
    }

    pub fn pointer_up(&mut self) -> bool {
        self.dragging.take().is_some()
    }

    pub fn handle_osk(&mut self, out: &Output) -> bool {
        if !self.open {
            return false;
        }
        if let Some(horizontal) = self.editing_formula {
            match out {
                Output::Text(t) => {
                    for ch in t.chars() {
                        if !ch.is_control() && self.formula_draft.chars().count() < 64 {
                            self.formula_draft.push(ch);
                        }
                    }
                }
                Output::Backspace => {
                    self.formula_draft.pop();
                }
                Output::Enter => {
                    self.apply_formula_draft(horizontal);
                    self.editing_formula = None;
                    crate::platform::hide();
                }
                _ => {}
            }
            return true;
        }
        if !self.editing_label {
            return false;
        }
        match out {
            Output::Text(s) => {
                for ch in s.chars() {
                    if !ch.is_control() && self.label_draft.chars().count() < 14 {
                        self.label_draft.push(ch);
                    }
                }
                self.commit_label_draft();
            }
            Output::Backspace => {
                self.label_draft.pop();
                self.commit_label_draft();
            }
            Output::Enter => {
                self.commit_label_draft();
                self.editing_label = false;
                crate::platform::hide();
            }
            _ => {}
        }
        true
    }

    /// Evaluates the typed Pojav formula in the 844x390 reference space and
    /// moves the selected button there, exactly like an imported layout.
    fn apply_formula_draft(&mut self, horizontal: bool) {
        let expr = self.formula_draft.trim().to_owned();
        if expr.is_empty() {
            return;
        }
        let Some(StudioTarget::Custom(idx)) = self.selected else {
            return;
        };
        let mode_idx = self.settings.mode.index();
        let Some(btn) = self.settings.custom_buttons.get_mut(idx) else {
            return;
        };
        let aspect = 16.0 / 9.0;
        let h_norm = btn.layouts[mode_idx].size;
        let w_norm = h_norm * btn.width_ratio.clamp(0.25, 6.0) / aspect;
        match eval_position_formula(&expr, w_norm, h_norm, horizontal) {
            Some(center) => {
                for p in &mut btn.layouts {
                    if horizontal {
                        p.center[0] = center;
                    } else {
                        p.center[1] = center;
                    }
                }
                self.mark_customized();
                self.status_hint = format!(
                    "Placed via formula: {} = {expr}",
                    if horizontal { "X" } else { "Y" }
                );
            }
            None => {
                self.status_hint =
                    "Formula not understood - try ${right} - ${margin} * 2 style.".into();
            }
        }
    }

    fn commit_label_draft(&mut self) {
        if let Some(StudioTarget::Custom(idx)) = self.selected
            && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
        {
            let trimmed = self.label_draft.trim();
            btn.label = (!trimmed.is_empty()).then(|| trimmed.to_owned());
            self.mark_customized();
        }
    }

    /// Ensures native Volt Default buttons can be edited freely as `CustomButton`s when the user
    /// clicks `+ Add` on an empty custom layout while keeping existing buttons if desired.
    pub fn handle_action(
        &mut self,
        act: StudioAction,
        root_home: &Path,
        profile_home: &Path,
    ) -> Option<String> {
        if self.editing_label && !matches!(act, StudioAction::EditLabel) {
            self.editing_label = false;
            crate::platform::hide();
        }
        if self.editing_formula.is_some()
            && !matches!(act, StudioAction::EditFormulaX | StudioAction::EditFormulaY)
        {
            self.editing_formula = None;
            crate::platform::hide();
        }
        match act {
            StudioAction::Open => {
                self.open_for_profile(profile_home);
                None
            }
            StudioAction::Cancel => {
                self.open = false;
                self.bind_picker_open = false;
                self.dragging = None;
                None
            }
            StudioAction::ApplyAndClose => {
                self.hide_stock_when_custom();
                self.settings.sanitize();
                match apply_to_profile(profile_home, &self.settings) {
                    Ok(()) => {
                        self.open = false;
                        self.bind_picker_open = false;
                        self.dragging = None;
                        let title = self
                            .settings
                            .imported_from
                            .as_ref()
                            .map(|m| m.name.clone())
                            .unwrap_or_else(|| "Volt Default".into());
                        Some(format!("Controls Studio applied \"{title}\" to active profile."))
                    }
                    Err(e) => {
                        self.status_hint = format!("Save failed: {e}");
                        None
                    }
                }
            }
            StudioAction::Preset(preset) => {
                let prev = self.settings.clone();
                let mut next = preset.build();
                next.sensitivity = prev.sensitivity;
                next.sensitivity_horizontal = prev.sensitivity_horizontal;
                next.sensitivity_vertical = prev.sensitivity_vertical;
                next.virtual_mouse_sensitivity = prev.virtual_mouse_sensitivity;
                next.gyroscope = prev.gyroscope;
                next.gyro_sensitivity = prev.gyro_sensitivity;
                self.settings = next;
                self.bind_picker_open = false;
                self.select_default_target();
                self.status_hint = format!("Loaded {}: {}", preset.label(), preset.subtitle());
                None
            }
            StudioAction::Select(target) => {
                self.selected = Some(target);
                self.bind_picker_open = false;
                self.bind_slot = 0;
                None
            }
            StudioAction::ToggleSnap => {
                self.snap_grid = !self.snap_grid;
                self.status_hint = if self.snap_grid {
                    "Grid snap ON (2.5% increments).".into()
                } else {
                    "Freeform pixel drag ON.".into()
                };
                None
            }
            StudioAction::ToggleJoystick => {
                let show = !self.joystick_visible();
                self.settings.button_visibility[Control::Move as usize] = if show { 1 } else { 2 };
                if show {
                    self.selected = Some(StudioTarget::Joystick);
                } else if self.selected == Some(StudioTarget::Joystick) {
                    self.select_default_target();
                }
                self.mark_customized();
                None
            }
            StudioAction::AddButton => {
                if self.settings.custom_buttons.len() >= MAX_CUSTOM_BUTTONS {
                    self.status_hint = "Maximum custom button limit reached.".into();
                    return None;
                }
                let n = self.settings.custom_buttons.len();
                let offset = ((n % 5) as f32 - 2.0) * 0.04;
                let p = Placement {
                    center: [(0.50 + offset).clamp(0.15, 0.85), (0.50 + offset).clamp(0.18, 0.82)],
                    size: 0.115,
                    opacity: 0.76,
                };
                let new_btn = CustomButton {
                    key: "special:mouse_pri".into(),
                    layouts: [p; 3],
                    extra_keys: Vec::new(),
                    label: Some("BTN".into()),
                    width_ratio: 1.15,
                    toggle: false,
                    swipeable: false,
                    passthrough: false,
                    in_game: true,
                    in_menu: false,
                    bg_color: Some([24, 18, 28, 155]),
                    stroke_color: Some([225, 48, 82, 150]),
                    corner_radius: Some(8.0),
                    drawer_parent: None,
                };
                self.settings.custom_buttons.push(new_btn);
                let idx = self.settings.custom_buttons.len() - 1;
                self.selected = Some(StudioTarget::Custom(idx));
                self.mark_customized();
                self.status_hint =
                    "Added custom button — drag it on the canvas and pick its keybind on the right."
                        .into();
                None
            }
            StudioAction::DuplicateSelected => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(src) = self.settings.custom_buttons.get(idx).cloned()
                    && self.settings.custom_buttons.len() < MAX_CUSTOM_BUTTONS
                {
                    let mut clone = src;
                    for p in &mut clone.layouts {
                        p.center[0] = (p.center[0] + 0.055).clamp(0.05, 0.95);
                        p.center[1] = (p.center[1] + 0.055).clamp(0.06, 0.94);
                    }
                    self.settings.custom_buttons.push(clone);
                    let new_idx = self.settings.custom_buttons.len() - 1;
                    self.selected = Some(StudioTarget::Custom(new_idx));
                    self.mark_customized();
                    self.status_hint = "Duplicated selected button.".into();
                }
                None
            }
            StudioAction::DeleteSelected => {
                match self.selected {
                    Some(StudioTarget::Custom(idx)) if idx < self.settings.custom_buttons.len() => {
                        let removed = self.settings.custom_buttons.remove(idx);
                        // A deleted drawer takes its sub-buttons with it (Pojav behavior).
                        if let Some(id) = removed
                            .key
                            .strip_prefix("special:drawer:")
                            .and_then(|v| v.parse::<u16>().ok())
                        {
                            self.settings
                                .custom_buttons
                                .retain(|b| b.drawer_parent != Some(id));
                        }
                        if self.settings.custom_buttons.is_empty() {
                            self.select_default_target();
                        } else {
                            let next = idx.min(self.settings.custom_buttons.len() - 1);
                            self.selected = Some(StudioTarget::Custom(next));
                        }
                        self.mark_customized();
                        self.status_hint = "Removed button.".into();
                    }
                    Some(StudioTarget::Joystick) => {
                        self.settings.button_visibility[Control::Move as usize] = 2;
                        self.select_default_target();
                        self.mark_customized();
                        self.status_hint = "Hidden movement joystick (tap Stick ON to restore).".into();
                    }
                    _ => {}
                }
                None
            }
            StudioAction::CycleBinding(back) => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    let cur = STUDIO_BINDINGS
                        .iter()
                        .position(|(k, _)| *k == btn.key)
                        .unwrap_or(0);
                    let len = STUDIO_BINDINGS.len();
                    let next = if back {
                        (cur + len - 1) % len
                    } else {
                        (cur + 1) % len
                    };
                    let (token, _) = STUDIO_BINDINGS[next];
                    let old_auto = CustomButton::token_label(&btn.key);
                    if btn.label.as_deref().is_none_or(|l| l == old_auto || l == "BTN") {
                        btn.label = Some(CustomButton::token_label(token));
                    }
                    btn.key = token.to_owned();
                    self.mark_customized();
                }
                None
            }
            StudioAction::ToggleBindPicker => {
                if matches!(self.selected, Some(StudioTarget::Custom(_))) {
                    self.bind_picker_open = !self.bind_picker_open;
                    if self.bind_picker_open {
                        self.bind_slot = 0;
                    }
                }
                None
            }
            StudioAction::InspectorPage(pg) => {
                self.inspector_page = pg.min(1);
                self.bind_picker_open = false;
                None
            }
            StudioAction::BindSlot(slot) => {
                if matches!(self.selected, Some(StudioTarget::Custom(_))) {
                    self.bind_slot = slot.min(3);
                    self.bind_picker_open = true;
                    self.status_hint = format!(
                        "Pick the key for slot {} (all slots press together, like Pojav).",
                        self.bind_slot + 1
                    );
                }
                None
            }
            StudioAction::ClearSlot => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                    && self.bind_slot > 0
                    && btn.extra_keys.len() >= self.bind_slot
                {
                    btn.extra_keys.remove(self.bind_slot - 1);
                    self.bind_picker_open = false;
                    self.mark_customized();
                    self.status_hint = format!("Cleared key slot {}.", self.bind_slot + 1);
                }
                None
            }
            StudioAction::PickBinding(b_idx) => {
                if let (Some(StudioTarget::Custom(idx)), Some(&(token, desc))) =
                    (self.selected, STUDIO_BINDINGS.get(b_idx))
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    if self.bind_slot == 0 {
                        let old_auto = CustomButton::token_label(&btn.key);
                        if btn.label.as_deref().is_none_or(|l| l == old_auto || l == "BTN") {
                            btn.label = Some(CustomButton::token_label(token));
                        }
                        btn.key = token.to_owned();
                        self.status_hint = format!("Bound button to {desc}.");
                    } else if btn.key != token && !btn.extra_keys.iter().any(|k| k == token) {
                        let at = self.bind_slot - 1;
                        if at < btn.extra_keys.len() {
                            btn.extra_keys[at] = token.to_owned();
                        } else if btn.extra_keys.len() < 3 {
                            btn.extra_keys.push(token.to_owned());
                        }
                        self.status_hint =
                            format!("Slot {} presses {desc} together.", self.bind_slot + 1);
                    } else {
                        self.status_hint = "That key is already on this button.".into();
                    }
                    self.bind_picker_open = false;
                    self.mark_customized();
                }
                None
            }
            StudioAction::EditLabel => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get(idx)
                {
                    self.label_draft = btn.label();
                    self.editing_label = true;
                    crate::platform::show(&self.label_draft);
                    self.status_hint = "Type custom button label on keyboard, then press Enter.".into();
                }
                None
            }
            StudioAction::AdjustSize(dir) => {
                let delta = f32::from(dir) * 0.012;
                match self.selected {
                    Some(StudioTarget::Joystick) => {
                        for row in &mut self.settings.layouts {
                            let p = &mut row[Control::Move as usize];
                            p.size = (p.size + delta * 1.5).clamp(0.14, 0.45);
                        }
                    }
                    Some(StudioTarget::Builtin(idx)) if idx < 14 => {
                        for row in &mut self.settings.layouts {
                            let p = &mut row[idx];
                            p.size = (p.size + delta).clamp(0.05, 0.35);
                        }
                    }
                    Some(StudioTarget::Custom(idx)) => {
                        if let Some(btn) = self.settings.custom_buttons.get_mut(idx) {
                            for p in &mut btn.layouts {
                                p.size = (p.size + delta).clamp(0.045, 0.36);
                            }
                            self.mark_customized();
                        }
                    }
                    _ => {}
                }
                None
            }
            StudioAction::AdjustWidth(dir) => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    btn.width_ratio = (btn.width_ratio + f32::from(dir) * 0.10).clamp(0.50, 3.20);
                    self.mark_customized();
                }
                None
            }
            StudioAction::AdjustOpacity(dir) => {
                let delta = f32::from(dir) * 0.08;
                match self.selected {
                    Some(StudioTarget::Joystick) => {
                        for row in &mut self.settings.layouts {
                            let p = &mut row[Control::Move as usize];
                            p.opacity = (p.opacity + delta).clamp(0.15, 1.0);
                        }
                    }
                    Some(StudioTarget::Builtin(idx)) if idx < 14 => {
                        for row in &mut self.settings.layouts {
                            let p = &mut row[idx];
                            p.opacity = (p.opacity + delta).clamp(0.15, 1.0);
                        }
                    }
                    Some(StudioTarget::Custom(idx)) => {
                        if let Some(btn) = self.settings.custom_buttons.get_mut(idx) {
                            for p in &mut btn.layouts {
                                p.opacity = (p.opacity + delta).clamp(0.15, 1.0);
                            }
                            self.mark_customized();
                        }
                    }
                    _ => {}
                }
                None
            }
            StudioAction::AdjustRadius(dir) => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    let cur = btn.corner_radius.unwrap_or(0.0);
                    let next = (cur + f32::from(dir) * 2.0).clamp(0.0, 22.0);
                    btn.corner_radius = (next >= 1.0).then_some(next);
                    self.mark_customized();
                }
                None
            }
            StudioAction::ToggleHoldLatch => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    btn.toggle = !btn.toggle;
                    self.mark_customized();
                }
                None
            }
            StudioAction::TogglePassthrough => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    btn.passthrough = !btn.passthrough;
                    self.mark_customized();
                }
                None
            }
            StudioAction::ToggleSwipeable => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    btn.swipeable = !btn.swipeable;
                    self.mark_customized();
                }
                None
            }
            StudioAction::ToggleInGame => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    btn.in_game = !btn.in_game;
                    self.mark_customized();
                }
                None
            }
            StudioAction::ToggleInMenu => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    btn.in_menu = !btn.in_menu;
                    self.mark_customized();
                }
                None
            }
            StudioAction::CycleBgColor(back) => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    btn.bg_color = Some(cycle_palette(btn.bg_color, back, 150));
                    self.mark_customized();
                }
                None
            }
            StudioAction::AdjustBgAlpha(dir) => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    let mut c = btn.bg_color.unwrap_or([22, 22, 30, 150]);
                    c[3] = (i16::from(c[3]) + i16::from(dir) * 16).clamp(0, 255) as u8;
                    btn.bg_color = Some(c);
                    self.mark_customized();
                }
                None
            }
            StudioAction::CycleStrokeColor(back) => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    btn.stroke_color = Some(cycle_palette(btn.stroke_color, back, 150));
                    self.mark_customized();
                }
                None
            }
            StudioAction::AdjustStrokeAlpha(dir) => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    let mut c = btn.stroke_color.unwrap_or([255, 255, 255, 150]);
                    c[3] = (i16::from(c[3]) + i16::from(dir) * 16).clamp(0, 255) as u8;
                    btn.stroke_color = Some(c);
                    self.mark_customized();
                }
                None
            }
            StudioAction::AddDrawer => {
                if self.settings.custom_buttons.len() >= MAX_CUSTOM_BUTTONS {
                    self.status_hint = "Maximum custom button limit reached.".into();
                    return None;
                }
                let id = self
                    .settings
                    .custom_buttons
                    .iter()
                    .filter_map(|b| {
                        b.key
                            .strip_prefix("special:drawer:")
                            .and_then(|v| v.parse::<u16>().ok())
                    })
                    .max()
                    .unwrap_or(0)
                    .saturating_add(1);
                let p = Placement {
                    center: [0.92, 0.26],
                    size: 0.10,
                    opacity: 0.8,
                };
                self.settings.custom_buttons.push(CustomButton {
                    key: format!("special:drawer:{id}"),
                    layouts: [p; 3],
                    extra_keys: Vec::new(),
                    label: Some("\u{2261}".into()),
                    width_ratio: 1.0,
                    toggle: false,
                    swipeable: false,
                    passthrough: false,
                    in_game: true,
                    in_menu: false,
                    bg_color: Some([24, 18, 28, 165]),
                    stroke_color: Some([225, 48, 82, 160]),
                    corner_radius: Some(10.0),
                    drawer_parent: None,
                });
                let idx = self.settings.custom_buttons.len() - 1;
                self.selected = Some(StudioTarget::Custom(idx));
                self.mark_customized();
                self.status_hint =
                    "Drawer added - tap + Sub-button to put keys inside it (Pojav drawers).".into();
                None
            }
            StudioAction::AddDrawerSub => {
                if self.settings.custom_buttons.len() >= MAX_CUSTOM_BUTTONS {
                    self.status_hint = "Maximum custom button limit reached.".into();
                    return None;
                }
                let Some(StudioTarget::Custom(idx)) = self.selected else {
                    return None;
                };
                let Some(parent) = self.settings.custom_buttons.get(idx) else {
                    return None;
                };
                let Some(id) = parent
                    .key
                    .strip_prefix("special:drawer:")
                    .and_then(|v| v.parse::<u16>().ok())
                else {
                    return None;
                };
                let mode_idx = self.settings.mode.index();
                let base = parent.layouts[mode_idx];
                let siblings = self
                    .settings
                    .custom_buttons
                    .iter()
                    .filter(|b| b.drawer_parent == Some(id))
                    .count();
                let p = Placement {
                    center: [
                        base.center[0],
                        (base.center[1] + 0.115 * (siblings + 1) as f32).clamp(0.06, 0.94),
                    ],
                    size: base.size,
                    opacity: base.opacity,
                };
                self.settings.custom_buttons.push(CustomButton {
                    key: "special:mouse_pri".into(),
                    layouts: [p; 3],
                    extra_keys: Vec::new(),
                    label: Some("SUB".into()),
                    width_ratio: 1.0,
                    toggle: false,
                    swipeable: false,
                    passthrough: false,
                    in_game: true,
                    in_menu: false,
                    bg_color: Some([24, 18, 28, 155]),
                    stroke_color: Some([225, 48, 82, 150]),
                    corner_radius: Some(8.0),
                    drawer_parent: Some(id),
                });
                let new_idx = self.settings.custom_buttons.len() - 1;
                self.selected = Some(StudioTarget::Custom(new_idx));
                self.mark_customized();
                self.status_hint =
                    "Sub-button added - it shows when the drawer is tapped in game.".into();
                None
            }
            StudioAction::DetachFromDrawer => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                    && btn.drawer_parent.is_some()
                {
                    btn.drawer_parent = None;
                    self.mark_customized();
                    self.status_hint = "Button detached from its drawer.".into();
                }
                None
            }
            StudioAction::ToggleForwardLock => {
                self.settings.sprint_with_joystick = !self.settings.sprint_with_joystick;
                self.mark_customized();
                self.status_hint = if self.settings.sprint_with_joystick {
                    "Forward lock ON: flicking the stick forward sprints (Pojav forwardLock).".into()
                } else {
                    "Forward lock OFF.".into()
                };
                None
            }
            StudioAction::ToggleDpadStyle => {
                use horizon_client::touch::Mode;
                self.settings.mode = if self.settings.mode == Mode::Dpad {
                    Mode::Crosshair
                } else {
                    Mode::Dpad
                };
                self.mark_customized();
                self.status_hint = if self.settings.mode == Mode::Dpad {
                    "D-pad style: 8-way absolute movement (Pojav absolute joystick).".into()
                } else {
                    "Analog stick: free 360\u{b0} movement.".into()
                };
                None
            }
            StudioAction::EditFormulaX | StudioAction::EditFormulaY => {
                if matches!(self.selected, Some(StudioTarget::Custom(_))) {
                    let horizontal = matches!(act, StudioAction::EditFormulaX);
                    if self.editing_formula == Some(horizontal) {
                        self.editing_formula = None;
                        crate::platform::hide();
                    } else {
                        self.formula_draft.clear();
                        self.editing_formula = Some(horizontal);
                        crate::platform::show("");
                        self.status_hint =
                            "Type a Pojav formula - e.g. ${right} - ${margin} * 2 - then Enter."
                                .into();
                    }
                }
                None
            }
            StudioAction::SavePreset => {
                self.hide_stock_when_custom();
                self.settings.sanitize();
                let base = self
                    .settings
                    .imported_from
                    .as_ref()
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| {
                        let n = self.preset_counter;
                        self.preset_counter += 1;
                        format!("Volt Studio #{n}")
                    });
                let meta = ImportedMeta {
                    name: base.clone(),
                    format: "Volt Controls Studio".into(),
                    buttons: self.settings.custom_buttons.len(),
                    drawers: self.drawer_count(),
                    has_joystick: self.joystick_visible(),
                };
                self.settings.imported_from = Some(meta.clone());
                let layout = ImportedControlLayout {
                    settings: self.settings.clone(),
                    meta,
                };
                match save_preset_and_apply(root_home, profile_home, &layout) {
                    Ok(_) => {
                        self.status_hint =
                            format!("Saved \"{base}\" to Saved Presets & active profile.");
                    }
                    Err(e) => {
                        self.status_hint = format!("Could not save preset: {e}");
                    }
                }
                None
            }
            StudioAction::ExportJson => {
                self.settings.sanitize();
                let out_path = profile_home.join("volt-controls-export.json");
                match std::fs::create_dir_all(profile_home)
                    .and_then(|()| self.settings.save_to(&out_path))
                {
                    Ok(()) => {
                        self.status_hint =
                            "Exported volt-controls-export.json to profile folder.".into();
                    }
                    Err(e) => {
                        self.status_hint = format!("Export failed: {e}");
                    }
                }
                None
            }
        }
    }

    pub fn draw(&mut self, ui: &mut Ui, targets: &mut Vec<([f32; 4], Action)>) {
        let (w, h) = (ui.w, ui.h);
        let scale = ui.density / ui.batch.scale;
        ui.batch
            .fill(ui.atlas, 0.0, 0.0, w * scale, h * scale, theme::BACKDROP());

        let pad = 10.0;
        let mx = pad;
        let my = pad;
        let mw = (w - pad * 2.0).max(300.0);
        let mh = (h - pad * 2.0).max(220.0);

        ui.rounded(mx, my, mw, mh, 14.0, theme::BG());
        ui.border(mx, my, mw, mh, 14.0, theme::BORDER_CRIMSON());

        // Top Studio bar: title + 5 built-in preset chips + Import + Apply & Close
        let top_h = 42.0;
        ui.rounded(mx + 4.0, my + 4.0, mw - 8.0, top_h, 10.0, theme::RAIL());
        ui.text(
            Face::LauncherMedium,
            14.0,
            mx + 14.0,
            my + 13.0,
            theme::CRIMSON_BRIGHT(),
            "CONTROLS STUDIO",
        );

        let active_preset = BuiltInPreset::detect(&self.settings);
        let chips_x = mx + 148.0;
        let right_btns_w = 198.0;
        let chips_avail = (mw - 148.0 - right_btns_w - 14.0).max(160.0);
        let chip_gap = 5.0;
        let chip_w = ((chips_avail - chip_gap * 4.0) / 5.0).clamp(38.0, 96.0);
        for (i, preset) in BuiltInPreset::ALL.into_iter().enumerate() {
            let cx = chips_x + i as f32 * (chip_w + chip_gap);
            let rect = [cx, my + 8.0, chip_w, 30.0];
            let is_active = active_preset == Some(preset);
            studio_btn(
                ui,
                targets,
                rect,
                preset.short_label(),
                Action::Studio(StudioAction::Preset(preset)),
                if is_active {
                    StudioBtn::Crimson
                } else {
                    StudioBtn::Dark
                },
            );
        }

        let cancel_r = [mx + mw - 190.0, my + 8.0, 68.0, 30.0];
        let apply_r = [mx + mw - 116.0, my + 8.0, 106.0, 30.0];
        studio_btn(
            ui,
            targets,
            cancel_r,
            "Cancel",
            Action::Studio(StudioAction::Cancel),
            StudioBtn::Dark,
        );
        studio_btn(
            ui,
            targets,
            apply_r,
            "Apply & Save",
            Action::Studio(StudioAction::ApplyAndClose),
            StudioBtn::Crimson,
        );

        // Split body: Left Interactive 16:9 Canvas + Right Inspector Panel
        let body_y = my + top_h + 10.0;
        let body_h = (mh - top_h - 16.0).max(150.0);
        let inspector_w = (mw * 0.31).clamp(196.0, 262.0);
        let canvas_area_w = (mw - inspector_w - 22.0).max(140.0);
        let canvas_area_x = mx + 8.0;
        let toolbar_h = 32.0;
        let canvas_max_h = (body_h - toolbar_h - 24.0).max(100.0);

        // Fit 16:9 viewport cleanly inside canvas_area_w x canvas_max_h
        let target_aspect = 16.0 / 9.0;
        let (cw, ch) = if canvas_area_w / canvas_max_h > target_aspect {
            (canvas_max_h * target_aspect, canvas_max_h)
        } else {
            (canvas_area_w, (canvas_area_w / target_aspect).max(90.0))
        };
        let cx = (canvas_area_x + (canvas_area_w - cw) * 0.5).round();
        let cy = (body_y + toolbar_h + 4.0 + (canvas_max_h - ch) * 0.5).round();
        self.canvas_rect = [cx, cy, cw, ch];

        // Canvas action toolbar above the 16:9 preview
        let tb_gap = 5.0;
        let tb_items: [(&str, StudioAction, bool); 6] = [
            ("+ Button", StudioAction::AddButton, false),
            ("Clone", StudioAction::DuplicateSelected, false),
            ("Delete", StudioAction::DeleteSelected, false),
            (
                if self.joystick_visible() {
                    "Stick: ON"
                } else {
                    "Stick: OFF"
                },
                StudioAction::ToggleJoystick,
                self.joystick_visible(),
            ),
            (
                if self.snap_grid {
                    "Snap: ON"
                } else {
                    "Snap: OFF"
                },
                StudioAction::ToggleSnap,
                self.snap_grid,
            ),
            ("Save Preset", StudioAction::SavePreset, false),
        ];
        let tb_w = ((canvas_area_w - tb_gap * 5.0) / 6.0).max(42.0);
        for (i, (lbl, sact, hi)) in tb_items.into_iter().enumerate() {
            let rx = canvas_area_x + i as f32 * (tb_w + tb_gap);
            studio_btn(
                ui,
                targets,
                [rx, body_y, tb_w, 28.0],
                lbl,
                Action::Studio(sact),
                if hi {
                    StudioBtn::Outline
                } else {
                    StudioBtn::Dark
                },
            );
        }

        // Draw 16:9 simulated Minecraft HUD canvas
        ui.rounded(cx, cy, cw, ch, 10.0, hex(0x0B0D13, 255));
        ui.border(cx, cy, cw, ch, 10.0, theme::BORDER());

        if self.snap_grid {
            let grid_col = hex(0xFFFFFF, 13);
            for step in 1..10 {
                let gx = cx + cw * (step as f32 * 0.1);
                let gy = cy + ch * (step as f32 * 0.1);
                ui.rounded(gx, cy + 2.0, 1.0, ch - 4.0, 0.0, grid_col);
                ui.rounded(cx + 2.0, gy, cw - 4.0, 1.0, 0.0, grid_col);
            }
        }

        // Center crosshair + bottom hotbar reference guide
        let mid_x = cx + cw * 0.5;
        let mid_y = cy + ch * 0.5;
        ui.rounded(mid_x - 6.0, mid_y - 0.5, 12.0, 1.2, 0.0, hex(0xFFFFFF, 95));
        ui.rounded(mid_x - 0.5, mid_y - 6.0, 1.2, 12.0, 0.0, hex(0xFFFFFF, 95));

        let hb_w = (cw * 0.34).clamp(90.0, 210.0);
        let hb_h = (ch * 0.075).clamp(12.0, 22.0);
        let hb_x = mid_x - hb_w * 0.5;
        let hb_y = cy + ch - hb_h - 4.0;
        ui.rounded(hb_x, hb_y, hb_w, hb_h, 4.0, hex(0x161822, 195));
        ui.border(hb_x, hb_y, hb_w, hb_h, 4.0, hex(0xFFFFFF, 45));

        let aspect = cw / ch;

        // Draw Joystick if visible
        if let Some([nx, ny, nw, nh]) = self.target_norm_rect(StudioTarget::Joystick, aspect) {
            let rx = cx + nx * cw;
            let ry = cy + ny * ch;
            let rw = nw * cw;
            let rh = nh * ch;
            let sel = self.selected == Some(StudioTarget::Joystick);
            let outer_r = (rw.min(rh) * 0.5).clamp(8.0, 48.0);
            ui.rounded(rx, ry, rw, rh, outer_r, hex(0x181A24, 150));
            ui.border(
                rx,
                ry,
                rw,
                rh,
                outer_r,
                if sel {
                    theme::CRIMSON_BRIGHT()
                } else {
                    hex(0xD8DCE8, 95)
                },
            );
            let knob = (rw.min(rh) * 0.36).max(10.0);
            ui.rounded(
                rx + (rw - knob) * 0.5,
                ry + (rh - knob) * 0.5,
                knob,
                knob,
                knob * 0.5,
                if sel {
                    theme::CRIMSON()
                } else {
                    hex(0xDC143C, 155)
                },
            );
        }

        // Draw built-in buttons when editing Volt Default (no custom buttons)
        if self.settings.custom_buttons.is_empty() {
            for control in Control::ALL {
                if control == Control::Move {
                    continue;
                }
                let target = StudioTarget::Builtin(control as usize);
                if let Some([nx, ny, nw, nh]) = self.target_norm_rect(target, aspect) {
                    let rx = cx + nx * cw;
                    let ry = cy + ny * ch;
                    let rw = nw * cw;
                    let rh = nh * ch;
                    let sel = self.selected == Some(target);
                    ui.rounded(rx, ry, rw, rh, 6.0, hex(0x1E202C, 175));
                    ui.border(
                        rx,
                        ry,
                        rw,
                        rh,
                        6.0,
                        if sel {
                            theme::CRIMSON_BRIGHT()
                        } else {
                            hex(0xFFFFFF, 85)
                        },
                    );
                    let short = match control {
                        Control::Sprint => "RUN",
                        Control::Jump => "JMP",
                        Control::Sneak => "SNK",
                        Control::Attack => "ATK",
                        Control::Use => "USE",
                        Control::Chat => "CHT",
                        Control::Pause => "II",
                        Control::Settings => "CFG",
                        Control::Inventory => "INV",
                        Control::Debug => "F3",
                        Control::Perspective => "F5",
                        Control::Offhand => "OFF",
                        Control::Shield => "SHD",
                        Control::Move => "",
                    };
                    draw_label_centered(ui, [rx, ry, rw, rh], 10.5, theme::TEXT(), short);
                }
            }
        }

        // Draw all CustomButtons
        for idx in 0..self.settings.custom_buttons.len() {
            let target = StudioTarget::Custom(idx);
            let Some([nx, ny, nw, nh]) = self.target_norm_rect(target, aspect) else {
                continue;
            };
            let b = &self.settings.custom_buttons[idx];
            let p = b.layouts[self.settings.mode.index()];
            let rx = cx + nx * cw;
            let ry = cy + ny * ch;
            let rw = (nw * cw).max(14.0);
            let rh = (nh * ch).max(14.0);
            let sel = self.selected == Some(target);
            let bg = b.bg_color.unwrap_or([22, 22, 30, 150]);
            let alpha = ((f32::from(bg[3]) / 255.0) * p.opacity * 255.0)
                .round()
                .clamp(38.0, 240.0) as u8;
            let fill = hex(
                (u32::from(bg[0]) << 16) | (u32::from(bg[1]) << 8) | u32::from(bg[2]),
                alpha,
            );
            let rad = b
                .corner_radius
                .unwrap_or(3.0)
                .min(rw * 0.48)
                .min(rh * 0.48);
            ui.rounded(rx, ry, rw, rh, rad, fill);
            let border_col = if sel {
                theme::CRIMSON_BRIGHT()
            } else if let Some(sc) = b.stroke_color {
                hex(
                    (u32::from(sc[0]) << 16) | (u32::from(sc[1]) << 8) | u32::from(sc[2]),
                    sc[3].max(90),
                )
            } else {
                hex(0xFFFFFF, 95)
            };
            ui.border(rx, ry, rw, rh, rad, border_col);
            if sel {
                ui.border(
                    rx - 1.5,
                    ry - 1.5,
                    rw + 3.0,
                    rh + 3.0,
                    rad + 1.5,
                    theme::CRIMSON(),
                );
            }
            let lbl = b.label();
            let font_sz = (rh * 0.34).clamp(8.5, 13.0);
            draw_label_centered(ui, [rx, ry, rw, rh], font_sz, theme::TEXT(), &lbl);
        }

        // Status hint line under canvas
        let hint_y = body_y + body_h - 16.0;
        let fit_hint = ui.fit_text(Face::LauncherRegular, 11.0, &self.status_hint, canvas_area_w);
        ui.text(
            Face::LauncherRegular,
            11.0,
            canvas_area_x + 2.0,
            hint_y,
            theme::MUTED(),
            &fit_hint,
        );

        // Right-hand Inspector Panel
        let ix = mx + mw - inspector_w - 8.0;
        let iy = body_y;
        let ih = body_h;
        ui.rounded(ix, iy, inspector_w, ih, 12.0, theme::CARD());
        ui.border(ix, iy, inspector_w, ih, 12.0, theme::BORDER());

        self.draw_inspector(ui, targets, [ix, iy, inspector_w, ih]);

        // Optional Visual Keybind Picker Overlay over the canvas
        if self.bind_picker_open {
            self.draw_bind_picker(ui, targets, [cx, cy, cw, ch]);
        }
    }

    fn draw_inspector(
        &self,
        ui: &mut Ui,
        targets: &mut Vec<([f32; 4], Action)>,
        area: [f32; 4],
    ) {
        let [ix, iy, iw, ih] = area;
        let pad = 10.0;
        let inner_w = iw - pad * 2.0;
        let mut y = iy + 8.0;

        match self.selected {
            Some(StudioTarget::Custom(idx)) if idx < self.settings.custom_buttons.len() => {
                let b = &self.settings.custom_buttons[idx];
                let p = b.layouts[self.settings.mode.index()];
                let is_drawer = b.key.starts_with("special:drawer:");
                let title = format!("Button #{}: {}", idx + 1, b.label());
                let fit_t = ui.fit_text(Face::LauncherMedium, 12.5, &title, inner_w);
                ui.text(Face::LauncherMedium, 12.5, ix + pad, y, theme::TEXT(), &fit_t);
                y += 20.0;

                // Main | Style page chips (the Pojav editor's two columns).
                let half = (inner_w - 6.0) * 0.5;
                for (pg, cap) in [(0u8, "Main"), (1u8, "Style & Pos")] {
                    let rx = ix + pad + f32::from(pg) * (half + 6.0);
                    studio_btn(
                        ui,
                        targets,
                        [rx, y, half, 22.0],
                        cap,
                        Action::Studio(StudioAction::InspectorPage(pg)),
                        if self.inspector_page == pg {
                            StudioBtn::Crimson
                        } else {
                            StudioBtn::Dark
                        },
                    );
                }
                y += 27.0;

                if self.inspector_page == 0 {
                    // Binding selector row: [<] [Pick Keybind...] [>]
                    let bind_name = STUDIO_BINDINGS
                        .iter()
                        .find(|(k, _)| *k == b.key)
                        .map(|(_, d)| *d)
                        .unwrap_or(b.key.as_str());
                    studio_btn(
                        ui,
                        targets,
                        [ix + pad, y, 26.0, 26.0],
                        "<",
                        Action::Studio(StudioAction::CycleBinding(true)),
                        StudioBtn::Dark,
                    );
                    studio_btn(
                        ui,
                        targets,
                        [ix + pad + 30.0, y, inner_w - 60.0, 26.0],
                        bind_name,
                        Action::Studio(StudioAction::ToggleBindPicker),
                        StudioBtn::Outline,
                    );
                    studio_btn(
                        ui,
                        targets,
                        [ix + pad + inner_w - 26.0, y, 26.0, 26.0],
                        ">",
                        Action::Studio(StudioAction::CycleBinding(false)),
                        StudioBtn::Dark,
                    );
                    y += 30.0;

                    // Pojav's 4 keycode slots: all press together.
                    if !is_drawer {
                        let quarter = (inner_w - 12.0) * 0.25;
                        for slot in 0..4usize {
                            let cap = if slot == 0 {
                                CustomButton::token_label(&b.key)
                            } else {
                                b.extra_keys
                                    .get(slot - 1)
                                    .map(|k| CustomButton::token_label(k))
                                    .unwrap_or_else(|| "+".into())
                            };
                            let rx = ix + pad + slot as f32 * (quarter + 4.0);
                            studio_btn(
                                ui,
                                targets,
                                [rx, y, quarter, 22.0],
                                &cap,
                                Action::Studio(StudioAction::BindSlot(slot)),
                                if self.bind_picker_open && self.bind_slot == slot {
                                    StudioBtn::Crimson
                                } else if slot == 0 || b.extra_keys.len() >= slot {
                                    StudioBtn::Outline
                                } else {
                                    StudioBtn::Dark
                                },
                            );
                        }
                        y += 26.0;
                    }

                    // Custom Label button
                    let lbl_caption = if self.editing_label {
                        format!("Typing: {}_", self.label_draft)
                    } else {
                        format!("Label: \"{}\"", b.label())
                    };
                    studio_btn(
                        ui,
                        targets,
                        [ix + pad, y, inner_w, 25.0],
                        &lbl_caption,
                        Action::Studio(StudioAction::EditLabel),
                        if self.editing_label {
                            StudioBtn::Crimson
                        } else {
                            StudioBtn::Dark
                        },
                    );
                    y += 30.0;

                    for (caption, dec, inc) in [
                        (
                            format!("Size {}%", (p.size * 100.0).round() as i32),
                            StudioAction::AdjustSize(-1),
                            StudioAction::AdjustSize(1),
                        ),
                        (
                            format!("Width {:.1}x", b.width_ratio),
                            StudioAction::AdjustWidth(-1),
                            StudioAction::AdjustWidth(1),
                        ),
                    ] {
                        if y + 25.0 > iy + ih - 58.0 {
                            break;
                        }
                        draw_stepper_row(ui, targets, [ix + pad, y, inner_w, 23.0], &caption, dec, inc);
                        y += 26.0;
                    }

                    // Compact toggle chips: Latch/Hold, Aim Pass, Swipe, Game/Menu
                    if y + 25.0 <= iy + ih - 30.0 {
                        studio_btn(
                            ui,
                            targets,
                            [ix + pad, y, half, 24.0],
                            if b.toggle { "Mode: Toggle" } else { "Mode: Hold" },
                            Action::Studio(StudioAction::ToggleHoldLatch),
                            if b.toggle {
                                StudioBtn::Outline
                            } else {
                                StudioBtn::Dark
                            },
                        );
                        studio_btn(
                            ui,
                            targets,
                            [ix + pad + half + 6.0, y, half, 24.0],
                            if b.passthrough { "Aim: Pass" } else { "Aim: Lock" },
                            Action::Studio(StudioAction::TogglePassthrough),
                            if b.passthrough {
                                StudioBtn::Outline
                            } else {
                                StudioBtn::Dark
                            },
                        );
                        y += 28.0;
                    }
                    if y + 24.0 <= iy + ih - 30.0 {
                        let third = (inner_w - 8.0) / 3.0;
                        studio_btn(
                            ui,
                            targets,
                            [ix + pad, y, third, 23.0],
                            "Swipe",
                            Action::Studio(StudioAction::ToggleSwipeable),
                            if b.swipeable {
                                StudioBtn::Outline
                            } else {
                                StudioBtn::Dark
                            },
                        );
                        studio_btn(
                            ui,
                            targets,
                            [ix + pad + third + 4.0, y, third, 23.0],
                            "InGame",
                            Action::Studio(StudioAction::ToggleInGame),
                            if b.in_game {
                                StudioBtn::Outline
                            } else {
                                StudioBtn::Dark
                            },
                        );
                        studio_btn(
                            ui,
                            targets,
                            [ix + pad + (third + 4.0) * 2.0, y, third, 23.0],
                            "InMenu",
                            Action::Studio(StudioAction::ToggleInMenu),
                            if b.in_menu {
                                StudioBtn::Outline
                            } else {
                                StudioBtn::Dark
                            },
                        );
                    }
                } else {
                    // ---- Style & Position page (Pojav bgColor / strokeColor /
                    // cornerRadius / opacity / dynamic formulas / drawers) ----
                    for (caption, dec, inc) in [
                        (
                            format!("Alpha {}%", (p.opacity * 100.0).round() as i32),
                            StudioAction::AdjustOpacity(-1),
                            StudioAction::AdjustOpacity(1),
                        ),
                        (
                            format!("Corner {:.0}px", b.corner_radius.unwrap_or(0.0)),
                            StudioAction::AdjustRadius(-1),
                            StudioAction::AdjustRadius(1),
                        ),
                    ] {
                        draw_stepper_row(ui, targets, [ix + pad, y, inner_w, 23.0], &caption, dec, inc);
                        y += 26.0;
                    }

                    // Fill color swatch row + alpha.
                    let bg = b.bg_color.unwrap_or([22, 22, 30, 150]);
                    draw_color_row(
                        ui,
                        targets,
                        [ix + pad, y, inner_w, 23.0],
                        "Fill",
                        bg,
                        StudioAction::CycleBgColor(true),
                        StudioAction::CycleBgColor(false),
                    );
                    y += 26.0;
                    draw_stepper_row(
                        ui,
                        targets,
                        [ix + pad, y, inner_w, 23.0],
                        &format!("Fill alpha {}", bg[3]),
                        StudioAction::AdjustBgAlpha(-1),
                        StudioAction::AdjustBgAlpha(1),
                    );
                    y += 26.0;

                    // Stroke color swatch row + alpha.
                    let sc = b.stroke_color.unwrap_or([255, 255, 255, 95]);
                    draw_color_row(
                        ui,
                        targets,
                        [ix + pad, y, inner_w, 23.0],
                        "Stroke",
                        sc,
                        StudioAction::CycleStrokeColor(true),
                        StudioAction::CycleStrokeColor(false),
                    );
                    y += 26.0;
                    draw_stepper_row(
                        ui,
                        targets,
                        [ix + pad, y, inner_w, 23.0],
                        &format!("Stroke alpha {}", sc[3]),
                        StudioAction::AdjustStrokeAlpha(-1),
                        StudioAction::AdjustStrokeAlpha(1),
                    );
                    y += 26.0;

                    // Pojav dynamic-position formulas.
                    for (horizontal, act) in
                        [(true, StudioAction::EditFormulaX), (false, StudioAction::EditFormulaY)]
                    {
                        if y + 24.0 > iy + ih - 60.0 {
                            break;
                        }
                        let editing = self.editing_formula == Some(horizontal);
                        let cap = if editing {
                            format!(
                                "{}: {}_",
                                if horizontal { "X" } else { "Y" },
                                self.formula_draft
                            )
                        } else {
                            format!(
                                "{} formula (e.g. ${{right}} - ${{margin}})",
                                if horizontal { "X" } else { "Y" }
                            )
                        };
                        studio_btn(
                            ui,
                            targets,
                            [ix + pad, y, inner_w, 23.0],
                            &cap,
                            Action::Studio(act),
                            if editing { StudioBtn::Crimson } else { StudioBtn::Dark },
                        );
                        y += 26.0;
                    }

                    // Drawer controls: parent -> add sub, child -> detach, else new drawer.
                    if y + 24.0 <= iy + ih - 60.0 {
                        let (cap, act) = if is_drawer {
                            ("+ Sub-button in drawer", StudioAction::AddDrawerSub)
                        } else if b.drawer_parent.is_some() {
                            ("Detach from drawer", StudioAction::DetachFromDrawer)
                        } else {
                            ("+ New drawer group", StudioAction::AddDrawer)
                        };
                        studio_btn(
                            ui,
                            targets,
                            [ix + pad, y, inner_w, 23.0],
                            cap,
                            Action::Studio(act),
                            StudioBtn::Outline,
                        );
                    }
                }
            }
            Some(StudioTarget::Joystick) => {
                ui.text(
                    Face::LauncherMedium,
                    13.0,
                    ix + pad,
                    y,
                    theme::TEXT(),
                    "Movement Joystick",
                );
                y += 24.0;
                let p = self.settings.layouts[self.settings.mode.index()][Control::Move as usize];
                draw_stepper_row(
                    ui,
                    targets,
                    [ix + pad, y, inner_w, 25.0],
                    &format!("Size {}%", (p.size * 100.0).round() as i32),
                    StudioAction::AdjustSize(-1),
                    StudioAction::AdjustSize(1),
                );
                y += 30.0;
                draw_stepper_row(
                    ui,
                    targets,
                    [ix + pad, y, inner_w, 25.0],
                    &format!("Alpha {}%", (p.opacity * 100.0).round() as i32),
                    StudioAction::AdjustOpacity(-1),
                    StudioAction::AdjustOpacity(1),
                );
                y += 30.0;
                let half = (inner_w - 6.0) * 0.5;
                studio_btn(
                    ui,
                    targets,
                    [ix + pad, y, half, 24.0],
                    if self.settings.sprint_with_joystick {
                        "FwdLock ON"
                    } else {
                        "FwdLock OFF"
                    },
                    Action::Studio(StudioAction::ToggleForwardLock),
                    if self.settings.sprint_with_joystick {
                        StudioBtn::Outline
                    } else {
                        StudioBtn::Dark
                    },
                );
                studio_btn(
                    ui,
                    targets,
                    [ix + pad + half + 6.0, y, half, 24.0],
                    if self.settings.mode == horizon_client::touch::Mode::Dpad {
                        "D-pad 8-way"
                    } else {
                        "Analog 360"
                    },
                    Action::Studio(StudioAction::ToggleDpadStyle),
                    if self.settings.mode == horizon_client::touch::Mode::Dpad {
                        StudioBtn::Outline
                    } else {
                        StudioBtn::Dark
                    },
                );
                y += 30.0;
                ui.text(
                    Face::LauncherRegular,
                    11.5,
                    ix + pad,
                    y,
                    theme::MUTED(),
                    "Drag the joystick on the canvas",
                );
                ui.text(
                    Face::LauncherRegular,
                    11.5,
                    ix + pad,
                    y + 16.0,
                    theme::MUTED(),
                    "or tap + Button to add keys.",
                );
            }
            Some(StudioTarget::Builtin(idx)) if idx < 14 => {
                let ctrl = Control::ALL[idx];
                let fit = ui.fit_text(Face::LauncherMedium, 12.5, ctrl.name(), inner_w);
                ui.text(Face::LauncherMedium, 12.5, ix + pad, y, theme::TEXT(), &fit);
                y += 24.0;
                let p = self.settings.layouts[self.settings.mode.index()][idx];
                draw_stepper_row(
                    ui,
                    targets,
                    [ix + pad, y, inner_w, 25.0],
                    &format!("Size {}%", (p.size * 100.0).round() as i32),
                    StudioAction::AdjustSize(-1),
                    StudioAction::AdjustSize(1),
                );
                y += 30.0;
                draw_stepper_row(
                    ui,
                    targets,
                    [ix + pad, y, inner_w, 25.0],
                    &format!("Alpha {}%", (p.opacity * 100.0).round() as i32),
                    StudioAction::AdjustOpacity(-1),
                    StudioAction::AdjustOpacity(1),
                );
            }
            _ => {
                ui.text(
                    Face::LauncherRegular,
                    12.0,
                    ix + pad,
                    y + 8.0,
                    theme::MUTED(),
                    "Tap any control on canvas.",
                );
            }
        }

        // Bottom quick import/export row inside inspector
        let bot_y = iy + ih - 32.0;
        let half = (inner_w - 6.0) * 0.5;
        studio_btn(
            ui,
            targets,
            [ix + pad, bot_y, half, 25.0],
            "Import File",
            Action::ImportControl,
            StudioBtn::Dark,
        );
        studio_btn(
            ui,
            targets,
            [ix + pad + half + 6.0, bot_y, half, 25.0],
            "Export JSON",
            Action::Studio(StudioAction::ExportJson),
            StudioBtn::Dark,
        );
    }

    fn draw_bind_picker(
        &self,
        ui: &mut Ui,
        targets: &mut Vec<([f32; 4], Action)>,
        canvas: [f32; 4],
    ) {
        let [cx, cy, cw, ch] = canvas;
        ui.rounded(cx, cy, cw, ch, 10.0, hex(0x090A10, 242));
        ui.border(cx, cy, cw, ch, 10.0, theme::BORDER_CRIMSON());
        let title = if self.bind_slot == 0 {
            "Select Key / Mouse Binding".to_owned()
        } else {
            format!("Select Key for Slot {} (pressed together)", self.bind_slot + 1)
        };
        ui.text(
            Face::LauncherMedium,
            13.0,
            cx + 12.0,
            cy + 8.0,
            theme::TEXT(),
            &title,
        );
        if self.bind_slot > 0 {
            studio_btn(
                ui,
                targets,
                [cx + cw - 140.0, cy + 6.0, 62.0, 24.0],
                "Clear",
                Action::Studio(StudioAction::ClearSlot),
                StudioBtn::Dark,
            );
        }
        studio_btn(
            ui,
            targets,
            [cx + cw - 70.0, cy + 6.0, 62.0, 24.0],
            "Close",
            Action::Studio(StudioAction::ToggleBindPicker),
            StudioBtn::Outline,
        );

        let cols = 5usize;
        let rows = STUDIO_BINDINGS.len().div_ceil(cols);
        let gx = cx + 10.0;
        let gy = cy + 34.0;
        let gw = cw - 20.0;
        let gh = (ch - 42.0).max(60.0);
        let gap = 4.0;
        let cell_w = ((gw - gap * (cols as f32 - 1.0)) / cols as f32).max(32.0);
        let cell_h = ((gh - gap * (rows as f32 - 1.0)) / rows as f32).clamp(18.0, 30.0);

        let cur_key = match self.selected {
            Some(StudioTarget::Custom(i)) => self
                .settings
                .custom_buttons
                .get(i)
                .map(|b| {
                    if self.bind_slot == 0 {
                        b.key.as_str()
                    } else {
                        b.extra_keys
                            .get(self.bind_slot - 1)
                            .map(String::as_str)
                            .unwrap_or("")
                    }
                })
                .unwrap_or(""),
            _ => "",
        };

        for (idx, &(token, desc)) in STUDIO_BINDINGS.iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;
            let rx = gx + col as f32 * (cell_w + gap);
            let ry = gy + row as f32 * (cell_h + gap);
            let short = desc
                .split('(')
                .next()
                .unwrap_or(desc)
                .trim();
            studio_btn(
                ui,
                targets,
                [rx, ry, cell_w, cell_h],
                short,
                Action::Studio(StudioAction::PickBinding(idx)),
                if token == cur_key {
                    StudioBtn::Crimson
                } else {
                    StudioBtn::Dark
                },
            );
        }
    }
}

#[derive(Clone, Copy)]
enum StudioBtn {
    Crimson,
    Outline,
    Dark,
}

fn draw_label_centered(ui: &mut Ui, rect: [f32; 4], size: f32, col: u32, text: &str) {
    let [x, y, w, h] = rect;
    let fitted = ui.fit_text(Face::LauncherMedium, size, text, (w - 6.0).max(6.0));
    let tw = ui.measure(Face::LauncherMedium, size, &fitted);
    let ty = y + (h - (size * 1.42).round()) * 0.5;
    ui.text(
        Face::LauncherMedium,
        size,
        x + ((w - tw) * 0.5).max(2.0),
        ty,
        col,
        &fitted,
    );
}

fn studio_btn(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    label: &str,
    action: Action,
    style: StudioBtn,
) {
    let [x, y, w, h] = rect;
    let (fill, border, text_col) = match style {
        StudioBtn::Crimson => (theme::CRIMSON(), theme::CRIMSON_BRIGHT(), theme::ON_CRIMSON()),
        StudioBtn::Outline => (theme::CRIMSON_BADGE(), theme::CRIMSON(), theme::TEXT()),
        StudioBtn::Dark => (theme::CARD_INNER(), theme::BORDER(), theme::TEXT()),
    };
    ui.rounded(x, y, w, h, 7.0, fill);
    ui.border(x, y, w, h, 7.0, border);
    draw_label_centered(ui, rect, 11.0, text_col, label);
    targets.push((rect, action));
}

/// A `[<] [label + color swatch] [>]` row for the Pojav-style color pickers.
fn draw_color_row(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    label: &str,
    color: [u8; 4],
    dec: StudioAction,
    inc: StudioAction,
) {
    let [x, y, w, h] = rect;
    let btn_w = 26.0;
    studio_btn(ui, targets, [x, y, btn_w, h], "<", Action::Studio(dec), StudioBtn::Dark);
    let mid_r = [x + btn_w + 4.0, y, (w - btn_w * 2.0 - 8.0).max(24.0), h];
    ui.rounded(mid_r[0], mid_r[1], mid_r[2], mid_r[3], 6.0, theme::BG());
    let swatch = hex(
        (u32::from(color[0]) << 16) | (u32::from(color[1]) << 8) | u32::from(color[2]),
        255,
    );
    ui.rounded(mid_r[0] + 4.0, y + 4.0, h - 8.0, h - 8.0, 4.0, swatch);
    ui.border(mid_r[0] + 4.0, y + 4.0, h - 8.0, h - 8.0, 4.0, hex(0xFFFFFF, 90));
    let caption = format!(
        "{label} #{:02X}{:02X}{:02X}",
        color[0], color[1], color[2]
    );
    draw_label_centered(ui, [mid_r[0] + h, y, mid_r[2] - h, h], 10.5, theme::MUTED(), &caption);
    studio_btn(
        ui,
        targets,
        [x + w - btn_w, y, btn_w, h],
        ">",
        Action::Studio(inc),
        StudioBtn::Dark,
    );
}

fn draw_stepper_row(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    caption: &str,
    dec: StudioAction,
    inc: StudioAction,
) {
    let [x, y, w, h] = rect;
    let btn_w = 26.0;
    studio_btn(
        ui,
        targets,
        [x, y, btn_w, h],
        "-",
        Action::Studio(dec),
        StudioBtn::Dark,
    );
    let mid_r = [x + btn_w + 4.0, y, (w - btn_w * 2.0 - 8.0).max(24.0), h];
    ui.rounded(mid_r[0], mid_r[1], mid_r[2], mid_r[3], 6.0, theme::BG());
    draw_label_centered(ui, mid_r, 10.5, theme::MUTED(), caption);
    studio_btn(
        ui,
        targets,
        [x + w - btn_w, y, btn_w, h],
        "+",
        Action::Studio(inc),
        StudioBtn::Dark,
    );
}
