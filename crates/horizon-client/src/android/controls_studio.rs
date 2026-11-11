//! Visual **Control Editor** for Volt Launcher, 1:1 with PojavLauncher's
//! customcontrols editor: a full-screen canvas showing the live layout, a
//! floating cogwheel opening the context menu (Add button / Add button
//! drawer / Add joystick / Load / Save / Select default controlmap /
//! Export controlmap), DELETE + CLONE chips on the selected button, a
//! draggable resize handle, and a right-hand edit panel with Name, Size,
//! the 2x2 keycode Mapping grid, Toggleable / Mouse pass-thru / Swipeable,
//! Background image / color, Stroke color / width %, Corner radius %,
//! Button Opacity % and In-game / In-menus visibility.

use super::pages::{Action, theme};
use horizon_client::{
    osk::Output,
    touch::{
        BuiltInPreset, Control, CustomButton, Settings,
        import::{
            ImportedControlLayout, PresetEntry, apply_to_profile, list_presets, load_preset,
            save_preset_and_apply,
        },
        layout::{ImportedMeta, MAX_CUSTOM_BUTTONS, Placement},
    },
    ui_theme::{Face, Ui, color::hex},
};
use std::path::{Path, PathBuf};

/// The Pojav reference canvas the Size fields are measured in (dp).
const REF_H: f32 = 390.0;

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
    /// The floating cogwheel: opens / closes the context menu.
    ToggleContextMenu,
    AddButton,
    /// Adds a Pojav-style drawer (an expandable group of sub-buttons).
    AddDrawer,
    /// Context menu "Add joystick": shows the movement stick again.
    AddJoystick,
    /// Context menu "Load": pick a saved controlmap to edit.
    CtxLoad,
    /// Context menu "Select default controlmap": pick what the game uses.
    CtxSelectDefault,
    /// A row in the Load / Select-default list.
    PickPreset(usize),
    SavePreset,
    ExportJson,
    /// DELETE / CLONE chips on the selected button.
    DuplicateSelected,
    DeleteSelected,
    /// Adds a sub-button to the selected drawer.
    AddDrawerSub,
    /// Detaches the selected sub-button from its drawer.
    DetachFromDrawer,
    EditLabel,
    /// The W / H fields of the Size row (true = width).
    EditSize(bool),
    /// Selects one of the four Pojav keycode slots and opens the picker.
    BindSlot(usize),
    /// A row of the keycode radio list.
    PickBinding(usize),
    /// Keycode picker paging (false = previous page, true = next).
    PickerPage(bool),
    ToggleHoldLatch,
    TogglePassthrough,
    ToggleSwipeable,
    ToggleInGame,
    ToggleInMenu,
    /// `Background image >`: opens the PNG list from `button-images/`.
    OpenBgImage,
    PickBgImage(usize),
    /// Opens the Android document picker to import a PNG into the list.
    ImportBgImage,
    /// `Background color >` / `Stroke color >` open the swatch picker.
    OpenBgColor,
    OpenStrokeColor,
    PickColor(usize),
    ClearColor,
    /// Joystick: W also sprints after a forward flick (Pojav forwardLock).
    ToggleForwardLock,
    /// Joystick: absolute D-pad style vs. free analog (Pojav `absolute`).
    ToggleDpadStyle,
    /// The top-right MOUSE pill (the in-game virtual mouse toggle preview).
    MousePill,
    /// Tap outside an open dialog: closes it (Pojav behavior).
    CloseOverlay,
}

/// The swatches of the Background / Stroke color pickers.
const STUDIO_PALETTE: [[u8; 3]; 24] = [
    [0, 0, 0],
    [34, 34, 48],
    [64, 64, 72],
    [120, 128, 140],
    [255, 255, 255],
    [220, 20, 60],
    [255, 68, 96],
    [255, 120, 150],
    [46, 204, 113],
    [22, 120, 60],
    [52, 152, 219],
    [26, 60, 130],
    [241, 196, 15],
    [230, 126, 34],
    [155, 89, 182],
    [96, 40, 140],
    [26, 188, 156],
    [0, 96, 100],
    [255, 235, 205],
    [160, 82, 45],
    [105, 105, 105],
    [25, 25, 25],
    [250, 128, 114],
    [127, 255, 212],
];

/// Pojav keycode names for the Mapping slots and the radio-list picker:
/// the SPECIAL_* rows first, then the full keyboard, exactly the names
/// PojavLauncher shows (`SPACE`, `LEFT_SHIFT`, `KP_1`, ...).
fn keycode_entries() -> &'static [(String, &'static str)] {
    use std::sync::OnceLock;
    static ENTRIES: OnceLock<Vec<(String, &'static str)>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let mut out: Vec<(String, &'static str)> = vec![
            ("UNKNOWN".into(), ""),
            ("SPECIAL_KEYBOARD".into(), "special:keyboard"),
            ("SPECIAL_GUI".into(), "special:gui_toggle"),
            ("SPECIAL_PRI".into(), "special:mouse_pri"),
            ("SPECIAL_SEC".into(), "special:mouse_sec"),
            ("SPECIAL_MOUSE_MID".into(), "special:mouse_mid"),
            ("SPECIAL_MOUSE".into(), "special:virtual_mouse"),
            ("SPECIAL_SCROLL_UP".into(), "special:scroll_up"),
            ("SPECIAL_SCROLL_DOWN".into(), "special:scroll_down"),
            ("SPECIAL_MENU".into(), "special:menu"),
        ];
        for &(token, _) in horizon_client::keybinds::available_keys() {
            if token == "key.keyboard.unknown" {
                continue;
            }
            let name = token.strip_prefix("key.keyboard.").unwrap_or(token);
            // Keep the picker at Pojav's GLFW set: skip exotic USB-HID keys.
            if name.starts_with("world")
                || name.starts_with("international")
                || name.starts_with("lang")
                || name.contains("memory")
                || matches!(
                    name,
                    "help"
                        | "select"
                        | "again"
                        | "undo"
                        | "cut"
                        | "copy"
                        | "paste"
                        | "find"
                        | "keypad.hash"
                        | "keypad.clear"
                        | "keypad.clear.entry"
                        | "keypad.left.parenthesis"
                        | "keypad.right.parenthesis"
                        | "keypad.backspace"
                )
            {
                continue;
            }
            if let Some(f) = name.strip_prefix('f')
                && f.parse::<u32>().is_ok_and(|n| n > 12)
            {
                continue;
            }
            let mut disp = name.to_ascii_uppercase().replace('.', "_");
            if let Some(rest) = disp.strip_prefix("KEYPAD_") {
                disp = format!("KP_{rest}");
            }
            out.push((disp, token));
        }
        out
    })
}

/// The Pojav display name of a bound token (for the Mapping slots).
fn pojav_name(token: &str) -> String {
    if token.is_empty() || token == "key.keyboard.unknown" {
        return "UNKNOWN".into();
    }
    if let Some((disp, _)) = keycode_entries().iter().find(|(_, t)| *t == token) {
        return disp.clone();
    }
    if let Some(rest) = token.strip_prefix("action:") {
        return rest.to_ascii_uppercase().replace('.', "_");
    }
    if token.starts_with("special:drawer:") {
        return "DRAWER".into();
    }
    token.to_ascii_uppercase().replace('.', "_")
}

/// Which value a live slider drag writes.
#[derive(Clone, Copy, PartialEq)]
enum Slider {
    StrokeWidth,
    CornerRadius,
    Opacity,
    ColorAlpha,
    JoystickSize,
    BuiltinSize,
}

pub(crate) struct ControlsStudio {
    pub open: bool,
    pub settings: Settings,
    pub selected: Option<StudioTarget>,
    /// Kept for the launcher's Back handler: any open dialog.
    pub bind_picker_open: bool,
    pub editing_label: bool,
    pub label_draft: String,
    /// Which of the four Pojav keycode slots the picker writes (0..=3).
    pub bind_slot: usize,
    /// `Some(true)` while the Size W field is being typed, `Some(false)` for H.
    pub editing_size: Option<bool>,
    pub size_draft: String,
    pub canvas_rect: [f32; 4],
    pub dragging: Option<(StudioTarget, [f32; 2])>,
    pub status_hint: String,
    /// The cogwheel context menu is open.
    context_menu: bool,
    /// `Some(select_default)` while the Load / Select-default list is open.
    load_menu: Option<bool>,
    /// `Some(stroke)` while the Background / Stroke color picker is open.
    color_menu: Option<bool>,
    /// The `Background image >` list is open.
    image_menu: bool,
    image_files: Vec<String>,
    /// Decoded PNG previews: `None` caches a failed load.
    images: std::collections::HashMap<String, Option<(u64, u32, u32, Vec<u8>)>>,
    /// Saved controlmaps for the Load / Select-default list.
    presets: Vec<PresetEntry>,
    picker_page: usize,
    /// Slider tracks of the current frame: `(rect, kind)` in dp.
    slider_rects: Vec<([f32; 4], Slider)>,
    slider_drag: Option<Slider>,
    /// The selected button's resize handle (dp) and an active resize drag.
    handle_rect: Option<[f32; 4]>,
    resizing: bool,
    /// The edit panel / chrome rects of the current frame (dp), so taps on
    /// them neither drag canvas buttons nor clear the selection.
    panel_rect: Option<[f32; 4]>,
    chrome_rects: Vec<[f32; 4]>,
    /// The profile whose `horizon-touch.json` is being edited.
    profile_home: PathBuf,
    preset_counter: u32,
}

impl Default for ControlsStudio {
    fn default() -> Self {
        Self {
            open: false,
            settings: Settings::default(),
            selected: None,
            bind_picker_open: false,
            editing_label: false,
            label_draft: String::new(),
            bind_slot: 0,
            editing_size: None,
            size_draft: String::new(),
            canvas_rect: [0.0; 4],
            dragging: None,
            status_hint: String::new(),
            context_menu: false,
            load_menu: None,
            color_menu: None,
            image_menu: false,
            image_files: Vec::new(),
            images: std::collections::HashMap::new(),
            presets: Vec::new(),
            picker_page: 0,
            slider_rects: Vec::new(),
            slider_drag: None,
            handle_rect: None,
            resizing: false,
            panel_rect: None,
            chrome_rects: Vec::new(),
            profile_home: PathBuf::new(),
            preset_counter: 1,
        }
    }
}

impl ControlsStudio {
    pub fn open_for_profile(&mut self, profile_home: &Path) {
        self.settings = Settings::load_from(&profile_home.join("horizon-touch.json"));
        self.profile_home = profile_home.to_path_buf();
        self.open = true;
        self.close_overlays();
        self.dragging = None;
        self.resizing = false;
        self.slider_drag = None;
        self.selected = None;
        self.images.clear();
        self.status_hint.clear();
    }

    fn close_overlays(&mut self) {
        self.bind_picker_open = false;
        self.context_menu = false;
        self.load_menu = None;
        self.color_menu = None;
        self.image_menu = false;
        self.editing_label = false;
        self.editing_size = None;
        self.picker_page = 0;
    }

    /// Any dialog is open (drives the Back handler).
    fn overlay_open(&self) -> bool {
        self.bind_picker_open
            || self.context_menu
            || self.load_menu.is_some()
            || self.color_menu.is_some()
            || self.image_menu
    }

    /// The system Back button inside the editor: closes the open dialog
    /// first; with nothing open it saves to the profile and exits.
    /// Returns whether the editor is still open.
    pub fn back(&mut self) -> bool {
        if self.editing_label || self.editing_size.is_some() {
            self.editing_label = false;
            self.editing_size = None;
            crate::platform::hide();
            return true;
        }
        if self.overlay_open() {
            self.close_overlays();
            return true;
        }
        if self.selected.is_some() {
            self.selected = None;
            return true;
        }
        // Exit saves, like Pojav's "save and quit" choice: nothing is lost.
        self.hide_stock_when_custom();
        self.settings.sanitize();
        let _ = apply_to_profile(&self.profile_home.clone(), &self.settings);
        self.open = false;
        self.dragging = None;
        false
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

    /// The selected custom button's smaller side in Pojav reference pixels,
    /// the base of the Stroke width / Corner radius percentages.
    fn selected_min_side(&self) -> Option<f32> {
        let Some(StudioTarget::Custom(idx)) = self.selected else {
            return None;
        };
        let b = self.settings.custom_buttons.get(idx)?;
        let h = b.layouts[self.settings.mode.index()].size * REF_H;
        Some(h.min(h * b.width_ratio.clamp(0.2, 8.0)).max(1.0))
    }

    /// Writes a 0..=1 slider position into the dragged value.
    fn apply_slider(&mut self, kind: Slider, frac: f32) {
        let frac = frac.clamp(0.0, 1.0);
        match kind {
            Slider::StrokeWidth => {
                let Some(m) = self.selected_min_side() else {
                    return;
                };
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(b) = self.settings.custom_buttons.get_mut(idx)
                {
                    let pct = (frac * 100.0).round();
                    b.stroke_width = if pct < 1.0 {
                        None
                    } else {
                        Some((pct / 100.0 * m * 0.5).clamp(0.5, 12.0))
                    };
                    if b.stroke_width.is_some() && b.stroke_color.is_none() {
                        b.stroke_color = Some([255, 255, 255, 160]);
                    }
                    self.mark_customized();
                }
            }
            Slider::CornerRadius => {
                let Some(m) = self.selected_min_side() else {
                    return;
                };
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(b) = self.settings.custom_buttons.get_mut(idx)
                {
                    let pct = (frac * 100.0).round();
                    b.corner_radius = if pct < 1.0 {
                        None
                    } else {
                        Some((pct / 100.0 * m * 0.5).clamp(0.5, 60.0))
                    };
                    self.mark_customized();
                }
            }
            Slider::Opacity => {
                let pct = (frac * 100.0).round().clamp(10.0, 100.0);
                match self.selected {
                    Some(StudioTarget::Custom(idx)) => {
                        if let Some(b) = self.settings.custom_buttons.get_mut(idx) {
                            for p in &mut b.layouts {
                                p.opacity = pct / 100.0;
                            }
                            self.mark_customized();
                        }
                    }
                    Some(StudioTarget::Joystick) => {
                        for row in &mut self.settings.layouts {
                            row[Control::Move as usize].opacity = pct / 100.0;
                        }
                        self.mark_customized();
                    }
                    Some(StudioTarget::Builtin(idx)) if idx < 14 => {
                        for row in &mut self.settings.layouts {
                            row[idx].opacity = pct / 100.0;
                        }
                        self.mark_customized();
                    }
                    _ => {}
                }
            }
            Slider::ColorAlpha => {
                let Some(stroke) = self.color_menu else {
                    return;
                };
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(b) = self.settings.custom_buttons.get_mut(idx)
                {
                    let a = (frac * 255.0).round() as u8;
                    if stroke {
                        let mut c = b.stroke_color.unwrap_or([255, 255, 255, 160]);
                        c[3] = a;
                        b.stroke_color = Some(c);
                    } else {
                        let mut c = b.bg_color.unwrap_or([0, 0, 0, 115]);
                        c[3] = a;
                        b.bg_color = Some(c);
                    }
                    self.mark_customized();
                }
            }
            Slider::JoystickSize => {
                let size = 0.14 + frac * (0.45 - 0.14);
                for row in &mut self.settings.layouts {
                    row[Control::Move as usize].size = size;
                }
                self.mark_customized();
            }
            Slider::BuiltinSize => {
                if let Some(StudioTarget::Builtin(idx)) = self.selected
                    && idx < 14
                {
                    let size = 0.05 + frac * (0.35 - 0.05);
                    for row in &mut self.settings.layouts {
                        row[idx].size = size;
                    }
                    self.mark_customized();
                }
            }
        }
    }

    /// The slider track under `(ux, uy)`, if any (copied out so the caller
    /// can mutate `self` right away).
    fn slider_hit(&self, ux: f32, uy: f32, alpha_only: bool) -> Option<([f32; 4], Slider)> {
        self.slider_rects
            .iter()
            .copied()
            .find(|&(r, kind)| {
                (kind == Slider::ColorAlpha) == alpha_only
                    && ux >= r[0]
                    && ux <= r[0] + r[2]
                    && uy >= r[1]
                    && uy <= r[1] + r[3]
            })
    }

    /// Hit-tests `(ux, uy)` in dp: sliders first, then the resize handle,
    /// then the layout's buttons; a tap on empty canvas clears the selection.
    pub fn pointer_down(&mut self, ux: f32, uy: f32) -> bool {
        if !self.open {
            return false;
        }
        // Dialog taps go through the target list (rows + tap-outside-closes).
        if self.overlay_open() {
            // The color picker's Opacity slider stays live while it is open.
            if self.color_menu.is_some()
                && let Some((r, kind)) = self.slider_hit(ux, uy, true)
            {
                self.slider_drag = Some(kind);
                self.apply_slider(kind, (ux - r[0]) / r[2].max(1.0));
                return true;
            }
            return false;
        }
        if let Some((r, kind)) = self.slider_hit(ux, uy, false) {
            self.slider_drag = Some(kind);
            self.apply_slider(kind, (ux - r[0]) / r[2].max(1.0));
            return true;
        }
        if let Some(hr) = self.handle_rect
            && ux >= hr[0] - 4.0
            && ux <= hr[0] + hr[2] + 4.0
            && uy >= hr[1] - 4.0
            && uy <= hr[1] + hr[3] + 4.0
        {
            self.resizing = true;
            return true;
        }
        if let Some([px, py, pw, ph]) = self.panel_rect
            && ux >= px
            && ux <= px + pw
            && uy >= py
            && uy <= py + ph
        {
            return false;
        }
        for r in &self.chrome_rects {
            if ux >= r[0] && ux <= r[0] + r[2] && uy >= r[1] && uy <= r[1] + r[3] {
                return false;
            }
        }
        let [cx, cy, cw, ch] = self.canvas_rect;
        if cw <= 10.0 || ch <= 10.0 {
            return false;
        }
        let nx = ((ux - cx) / cw).clamp(0.0, 1.0);
        let ny = ((uy - cy) / ch).clamp(0.0, 1.0);
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
                let center =
                    self.settings.custom_buttons[idx].layouts[self.settings.mode.index()].center;
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

        // Empty canvas: close the edit panel, exactly like Pojav.
        self.selected = None;
        false
    }

    pub fn pointer_move(&mut self, ux: f32, uy: f32) -> bool {
        if let Some(kind) = self.slider_drag {
            let track = self
                .slider_rects
                .iter()
                .copied()
                .find(|(_, k)| *k == kind)
                .map(|(r, _)| r);
            if let Some(r) = track {
                self.apply_slider(kind, (ux - r[0]) / r[2].max(1.0));
            }
            return true;
        }
        let [cx, cy, cw, ch] = self.canvas_rect;
        if cw <= 10.0 || ch <= 10.0 {
            return false;
        }
        if self.resizing {
            let aspect = cw / ch;
            if let Some(target) = self.selected
                && let Some([rx, ry, _, _]) = self.target_norm_rect(target, aspect)
            {
                let bx = cx + rx * cw;
                let by = cy + ry * ch;
                let w_dp = (ux - bx).max(14.0);
                let h_dp = (uy - by).max(14.0);
                match target {
                    StudioTarget::Custom(idx) => {
                        if let Some(b) = self.settings.custom_buttons.get_mut(idx) {
                            let size = (h_dp / ch).clamp(0.025, 0.55);
                            for p in &mut b.layouts {
                                p.size = size;
                            }
                            b.width_ratio = (w_dp / h_dp).clamp(0.2, 8.0);
                            self.mark_customized();
                        }
                    }
                    StudioTarget::Joystick => {
                        let size = (h_dp / ch).clamp(0.14, 0.45);
                        for row in &mut self.settings.layouts {
                            row[Control::Move as usize].size = size;
                        }
                        self.mark_customized();
                    }
                    StudioTarget::Builtin(idx) if idx < 14 => {
                        let size = (h_dp / ch).clamp(0.05, 0.35);
                        for row in &mut self.settings.layouts {
                            row[idx].size = size;
                        }
                        self.mark_customized();
                    }
                    _ => {}
                }
            }
            return true;
        }
        let Some((target, [ox, oy])) = self.dragging else {
            return false;
        };
        let nx = ((ux - cx) / cw + ox).clamp(0.035, 0.965);
        let ny = ((uy - cy) / ch + oy).clamp(0.045, 0.955);
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
        let dragged = self.dragging.take().is_some();
        let resized = std::mem::take(&mut self.resizing);
        let slid = self.slider_drag.take().is_some();
        dragged || resized || slid
    }

    pub fn handle_osk(&mut self, out: &Output) -> bool {
        if !self.open {
            return false;
        }
        if let Some(width) = self.editing_size {
            match out {
                Output::Text(t) => {
                    for ch in t.chars() {
                        if ch.is_ascii_digit() && self.size_draft.chars().count() < 4 {
                            self.size_draft.push(ch);
                        }
                    }
                }
                Output::Backspace => {
                    self.size_draft.pop();
                }
                Output::Enter => {
                    self.commit_size_draft(width);
                    self.editing_size = None;
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

    fn commit_label_draft(&mut self) {
        if let Some(StudioTarget::Custom(idx)) = self.selected
            && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
        {
            let trimmed = self.label_draft.trim();
            btn.label = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            };
            self.mark_customized();
        }
    }

    /// Applies the typed Size field: W adjusts the width ratio, H the base
    /// size, both in the 844x390 Pojav reference space.
    fn commit_size_draft(&mut self, width: bool) {
        let Ok(px) = self.size_draft.trim().parse::<f32>() else {
            return;
        };
        let px = px.clamp(10.0, 400.0);
        if let Some(StudioTarget::Custom(idx)) = self.selected
            && let Some(b) = self.settings.custom_buttons.get_mut(idx)
        {
            if width {
                let h_px = (b.layouts[self.settings.mode.index()].size * REF_H).max(1.0);
                b.width_ratio = (px / h_px).clamp(0.2, 8.0);
            } else {
                let size = (px / REF_H).clamp(0.025, 0.55);
                for p in &mut b.layouts {
                    p.size = size;
                }
            }
            self.mark_customized();
            self.status_hint = format!("Size set to {px:.0} px.");
        }
    }

    /// Re-lists `button-images/*.png` of the edited profile.
    fn refresh_image_files(&mut self) {
        let dir = self.profile_home.join("button-images");
        let mut files: Vec<String> = std::fs::read_dir(&dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n.to_ascii_lowercase().ends_with(".png") && n.len() <= 96)
            .collect();
        files.sort();
        files.truncate(18);
        self.image_files = files;
    }

    /// Loads and caches a PNG preview from the profile's `button-images/`.
    fn image_preview(&mut self, name: &str) -> Option<(u64, u32, u32)> {
        if !self.images.contains_key(name) {
            let decoded = (|| {
                if name.contains(['/', '\\']) || name.contains("..") {
                    return None;
                }
                let path = self.profile_home.join("button-images").join(name);
                let bytes = std::fs::read(path).ok()?;
                if bytes.len() > 4 * 1024 * 1024 {
                    return None;
                }
                let (w, h, rgba) = horizon_client::hud::decode_bytes(&bytes).ok()?;
                if w == 0 || h == 0 {
                    return None;
                }
                const MAX: u32 = 96;
                let (ow, oh) = if w <= MAX && h <= MAX {
                    (w, h)
                } else {
                    let f = MAX as f32 / w.max(h) as f32;
                    (
                        ((w as f32 * f).round() as u32).max(1),
                        ((h as f32 * f).round() as u32).max(1),
                    )
                };
                let mut out = vec![0u8; (ow * oh * 4) as usize];
                for oy in 0..oh {
                    for ox in 0..ow {
                        let sx = (u64::from(ox) * u64::from(w) / u64::from(ow))
                            .min(u64::from(w) - 1) as u32;
                        let sy = (u64::from(oy) * u64::from(h) / u64::from(oh))
                            .min(u64::from(h) - 1) as u32;
                        let si = ((sy * w + sx) * 4) as usize;
                        let di = ((oy * ow + ox) * 4) as usize;
                        out[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
                    }
                }
                let mut hash = 0xcbf2_9ce4_8422_2325u64;
                for b in bytes.iter().step_by(64) {
                    hash = (hash ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3);
                }
                hash ^= u64::from(ow) << 32 | u64::from(oh);
                Some((hash, ow, oh, out))
            })();
            self.images.insert(name.to_owned(), decoded);
        }
        self.images
            .get(name)
            .and_then(|v| v.as_ref())
            .map(|(hash, w, h, _)| (*hash, *w, *h))
    }

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
        if self.editing_size.is_some() && !matches!(act, StudioAction::EditSize(_)) {
            self.editing_size = None;
            crate::platform::hide();
        }
        // Any action other than opening it puts the context menu away,
        // exactly like Pojav's menu closing when an entry is chosen.
        if !matches!(act, StudioAction::ToggleContextMenu) {
            self.context_menu = false;
        }
        match act {
            StudioAction::Open => {
                self.open_for_profile(profile_home);
                None
            }
            StudioAction::ApplyAndClose => {
                self.hide_stock_when_custom();
                self.settings.sanitize();
                match apply_to_profile(profile_home, &self.settings) {
                    Ok(()) => {
                        self.open = false;
                        self.close_overlays();
                        self.dragging = None;
                        let title = self
                            .settings
                            .imported_from
                            .as_ref()
                            .map(|m| m.name.clone())
                            .unwrap_or_else(|| "Volt Default".into());
                        Some(format!("Control editor saved \"{title}\" to active profile."))
                    }
                    Err(e) => {
                        self.status_hint = format!("Save failed: {e}");
                        None
                    }
                }
            }
            StudioAction::ToggleContextMenu => {
                let next = !self.context_menu;
                self.close_overlays();
                self.context_menu = next;
                None
            }
            StudioAction::CloseOverlay => {
                self.close_overlays();
                None
            }
            StudioAction::MousePill => {
                self.status_hint =
                    "MOUSE is the in-game virtual mouse toggle - bind SPECIAL_MOUSE to any button."
                        .into();
                None
            }
            StudioAction::AddJoystick => {
                if self.joystick_visible() {
                    self.status_hint = "The joystick is already on the canvas - drag it to move."
                        .into();
                } else {
                    self.settings.button_visibility[Control::Move as usize] = 1;
                    self.selected = Some(StudioTarget::Joystick);
                    self.mark_customized();
                    self.status_hint = "Joystick added - drag it anywhere, resize by its handle."
                        .into();
                }
                None
            }
            StudioAction::CtxLoad => {
                self.presets = list_presets(root_home);
                self.load_menu = Some(false);
                self.bind_picker_open = true;
                None
            }
            StudioAction::CtxSelectDefault => {
                self.presets = list_presets(root_home);
                self.load_menu = Some(true);
                self.bind_picker_open = true;
                None
            }
            StudioAction::PickPreset(i) => {
                let select_default = self.load_menu.unwrap_or(false);
                let entry = self.presets.get(i).cloned();
                self.close_overlays();
                if let Some(entry) = entry {
                    match load_preset(root_home, &entry.slug) {
                        Ok(s) => {
                            self.settings = s;
                            self.selected = None;
                            self.images.clear();
                            if select_default {
                                match apply_to_profile(profile_home, &self.settings) {
                                    Ok(()) => {
                                        self.status_hint = format!(
                                            "\"{}\" is now the default controlmap in game.",
                                            entry.meta.name
                                        );
                                    }
                                    Err(e) => self.status_hint = e,
                                }
                            } else {
                                self.status_hint =
                                    format!("Loaded \"{}\" into the editor.", entry.meta.name);
                            }
                        }
                        Err(e) => self.status_hint = e,
                    }
                }
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
                    label: Some("New".into()),
                    width_ratio: 1.0,
                    toggle: false,
                    swipeable: false,
                    passthrough: false,
                    in_game: true,
                    in_menu: false,
                    bg_color: Some([0, 0, 0, 115]),
                    stroke_color: None,
                    corner_radius: Some(8.0),
                    stroke_width: None,
                    background_image: None,
                    drawer_parent: None,
                };
                self.settings.custom_buttons.push(new_btn);
                let idx = self.settings.custom_buttons.len() - 1;
                self.selected = Some(StudioTarget::Custom(idx));
                self.mark_customized();
                self.status_hint =
                    "Button added - drag to place it, tap it to edit keybinds and looks.".into();
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
                    self.status_hint = "Cloned the selected button.".into();
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
                        self.selected = None;
                        self.mark_customized();
                        self.status_hint = "Removed button.".into();
                    }
                    Some(StudioTarget::Joystick) => {
                        self.settings.button_visibility[Control::Move as usize] = 2;
                        self.selected = None;
                        self.mark_customized();
                        self.status_hint =
                            "Joystick hidden (context menu > Add joystick restores it).".into();
                    }
                    _ => {}
                }
                None
            }
            StudioAction::BindSlot(slot) => {
                if matches!(self.selected, Some(StudioTarget::Custom(_))) {
                    self.bind_slot = slot.min(3);
                    self.bind_picker_open = true;
                    self.picker_page = 0;
                }
                None
            }
            StudioAction::PickerPage(next) => {
                let pages = keycode_entries().len().div_ceil(PICKER_PER_PAGE).max(1);
                if next {
                    self.picker_page = (self.picker_page + 1).min(pages - 1);
                } else {
                    self.picker_page = self.picker_page.saturating_sub(1);
                }
                None
            }
            StudioAction::PickBinding(i) => {
                if let (Some(StudioTarget::Custom(idx)), Some((disp, token))) =
                    (self.selected, keycode_entries().get(i).cloned())
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    if token.is_empty() {
                        // UNKNOWN clears the slot, exactly like Pojav.
                        if self.bind_slot == 0 {
                            if btn.extra_keys.is_empty() {
                                btn.key = "key.keyboard.unknown".into();
                            } else {
                                btn.key = btn.extra_keys.remove(0);
                            }
                        } else {
                            let at = self.bind_slot - 1;
                            if at < btn.extra_keys.len() {
                                btn.extra_keys.remove(at);
                            }
                        }
                        self.status_hint = format!("Slot {} cleared.", self.bind_slot + 1);
                    } else if self.bind_slot == 0 {
                        let old_auto = CustomButton::token_label(&btn.key);
                        if btn.label.as_deref().is_none_or(|l| l == old_auto || l == "New") {
                            btn.label = Some(CustomButton::token_label(token));
                        }
                        btn.key = token.to_owned();
                        self.status_hint = format!("Bound to {disp}.");
                    } else if btn.key != token && !btn.extra_keys.iter().any(|k| k == token) {
                        let at = self.bind_slot - 1;
                        if at < btn.extra_keys.len() {
                            btn.extra_keys[at] = token.to_owned();
                        } else if btn.extra_keys.len() < 3 {
                            btn.extra_keys.push(token.to_owned());
                        }
                        self.status_hint =
                            format!("Slot {} presses {disp} together.", self.bind_slot + 1);
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
                    self.status_hint = "Type the button name, then press Enter.".into();
                }
                None
            }
            StudioAction::EditSize(width) => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(b) = self.settings.custom_buttons.get(idx)
                {
                    let h_px = b.layouts[self.settings.mode.index()].size * REF_H;
                    let px = if width {
                        h_px * b.width_ratio.clamp(0.2, 8.0)
                    } else {
                        h_px
                    };
                    self.size_draft = format!("{px:.0}");
                    self.editing_size = Some(width);
                    crate::platform::show(&self.size_draft);
                    self.status_hint = "Type the size in px, then press Enter.".into();
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
            StudioAction::OpenBgImage => {
                self.refresh_image_files();
                self.image_menu = true;
                self.bind_picker_open = true;
                None
            }
            StudioAction::ImportBgImage => {
                let dir = self.profile_home.join("button-images");
                match crate::documents::begin_button_image(dir) {
                    Ok(()) => {
                        self.status_hint =
                            "Pick a PNG - it lands in the Background image list.".into();
                    }
                    Err(e) => self.status_hint = e,
                }
                None
            }
            StudioAction::PickBgImage(i) => {
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    if i == 0 {
                        btn.background_image = None;
                        self.status_hint = "Background image removed.".into();
                    } else if let Some(name) = self.image_files.get(i - 1) {
                        btn.background_image = Some(name.clone());
                        self.status_hint = format!("Background image: {name}.");
                    }
                    self.mark_customized();
                }
                self.close_overlays();
                None
            }
            StudioAction::OpenBgColor => {
                if matches!(self.selected, Some(StudioTarget::Custom(_))) {
                    self.color_menu = Some(false);
                    self.bind_picker_open = true;
                }
                None
            }
            StudioAction::OpenStrokeColor => {
                if matches!(self.selected, Some(StudioTarget::Custom(_))) {
                    self.color_menu = Some(true);
                    self.bind_picker_open = true;
                }
                None
            }
            StudioAction::PickColor(i) => {
                let Some(stroke) = self.color_menu else {
                    return None;
                };
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                    && let Some(rgb) = STUDIO_PALETTE.get(i)
                {
                    if stroke {
                        let a = btn.stroke_color.map_or(160, |c| c[3]);
                        btn.stroke_color = Some([rgb[0], rgb[1], rgb[2], a.max(40)]);
                        if btn.stroke_width.is_none() {
                            btn.stroke_width = Some(2.0);
                        }
                    } else {
                        let a = btn.bg_color.map_or(115, |c| c[3]);
                        btn.bg_color = Some([rgb[0], rgb[1], rgb[2], a.max(30)]);
                    }
                    self.mark_customized();
                }
                None
            }
            StudioAction::ClearColor => {
                let Some(stroke) = self.color_menu else {
                    return None;
                };
                if let Some(StudioTarget::Custom(idx)) = self.selected
                    && let Some(btn) = self.settings.custom_buttons.get_mut(idx)
                {
                    if stroke {
                        btn.stroke_color = None;
                        btn.stroke_width = None;
                    } else {
                        btn.bg_color = None;
                    }
                    self.mark_customized();
                }
                self.close_overlays();
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
                    bg_color: Some([0, 0, 0, 140]),
                    stroke_color: None,
                    corner_radius: Some(10.0),
                    stroke_width: None,
                    background_image: None,
                    drawer_parent: None,
                });
                let idx = self.settings.custom_buttons.len() - 1;
                self.selected = Some(StudioTarget::Custom(idx));
                self.mark_customized();
                self.status_hint =
                    "Drawer added - use + Sub-button in its panel to put keys inside.".into();
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
                    bg_color: Some([0, 0, 0, 115]),
                    stroke_color: None,
                    corner_radius: Some(8.0),
                    stroke_width: None,
                    background_image: None,
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
                    "Forward lock ON: flicking the stick forward sprints (Pojav forwardLock)."
                        .into()
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
                            format!("Saved \"{base}\" - it is the active controlmap now.");
                    }
                    Err(e) => {
                        self.status_hint = format!("Could not save controlmap: {e}");
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
                            "Exported volt-controls-export.json to the profile folder.".into();
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
            .fill(ui.atlas, 0.0, 0.0, w * scale, h * scale, hex(0x0B0D13, 255));

        // The whole screen is the canvas, exactly like Pojav's editor.
        self.canvas_rect = [0.0, 0.0, w, h];
        self.slider_rects.clear();
        self.handle_rect = None;
        self.panel_rect = None;
        self.chrome_rects.clear();
        let mut t_overlay: Vec<([f32; 4], Action)> = Vec::new();
        let mut t_main: Vec<([f32; 4], Action)> = Vec::new();

        // Subtle world guides: the crosshair and the hotbar silhouette.
        let (mid_x, mid_y) = (w * 0.5, h * 0.5);
        ui.rounded(mid_x - 7.0, mid_y - 0.6, 14.0, 1.2, 0.0, hex(0xFFFFFF, 70));
        ui.rounded(mid_x - 0.6, mid_y - 7.0, 1.2, 14.0, 0.0, hex(0xFFFFFF, 70));
        let hb_w = (w * 0.34).clamp(120.0, 280.0);
        let hb_h = (h * 0.065).clamp(14.0, 24.0);
        ui.rounded(mid_x - hb_w * 0.5, h - hb_h - 4.0, hb_w, hb_h, 4.0, hex(0x161822, 160));
        ui.border(mid_x - hb_w * 0.5, h - hb_h - 4.0, hb_w, hb_h, 4.0, hex(0xFFFFFF, 35));

        let aspect = w / h.max(1.0);
        self.draw_canvas_buttons(ui, aspect);

        // Hint text, top center (Pojav's two helper lines).
        if self.selected.is_none() && !self.overlay_open() {
            let l1 = "Touch the cogwheel icon to open the context menu";
            let l2 = "Tap a button to customize: edit, resize, remove, or change its keybinds.";
            let w1 = ui.measure(Face::LauncherRegular, 12.0, l1);
            let w2 = ui.measure(Face::LauncherRegular, 11.0, l2);
            ui.text(Face::LauncherRegular, 12.0, (w - w1) / 2.0, 10.0, hex(0xE8E8F0, 220), l1);
            ui.text(Face::LauncherRegular, 11.0, (w - w2) / 2.0, 26.0, hex(0xB8B8C4, 190), l2);
        }

        // Top-right MOUSE pill (the virtual mouse, like Pojav's editor).
        let mouse_r = [w - 76.0, 8.0, 66.0, 24.0];
        ui.rounded(mouse_r[0], mouse_r[1], mouse_r[2], mouse_r[3], 12.0, hex(0x1E202C, 200));
        ui.border(mouse_r[0], mouse_r[1], mouse_r[2], mouse_r[3], 12.0, hex(0xFFFFFF, 70));
        draw_label_centered(ui, mouse_r, 10.5, theme::TEXT(), "MOUSE");
        t_main.push((mouse_r, Action::Studio(StudioAction::MousePill)));
        self.chrome_rects.push(mouse_r);

        // Back chip, top-left: saves and exits (the system Back does the same).
        let back_r = [8.0, 8.0, 30.0, 24.0];
        ui.rounded(back_r[0], back_r[1], back_r[2], back_r[3], 12.0, hex(0x1E202C, 200));
        ui.border(back_r[0], back_r[1], back_r[2], back_r[3], 12.0, hex(0xFFFFFF, 70));
        draw_label_centered(ui, back_r, 12.0, theme::TEXT(), "\u{2039}");
        t_main.push((back_r, Action::Studio(StudioAction::ApplyAndClose)));
        self.chrome_rects.push(back_r);

        // The floating cogwheel, vertically centered at the left edge.
        let cog_r = [10.0, h * 0.5 - 18.0, 36.0, 36.0];
        self.draw_cogwheel(ui, cog_r);
        t_main.push((cog_r, Action::Studio(StudioAction::ToggleContextMenu)));
        self.chrome_rects.push(cog_r);

        // Selected-button chrome: DELETE / CLONE chips and the resize handle.
        if !self.overlay_open() {
            self.draw_selection_chrome(ui, &mut t_main, aspect);
        }

        // The right-hand edit panel when a button is selected.
        if self.selected.is_some() && !self.context_menu {
            self.draw_edit_panel(ui, &mut t_main, w, h);
        }

        // The cogwheel context menu, sliding in at the right edge.
        if self.context_menu {
            self.draw_context_menu(ui, &mut t_overlay, w, h);
        }

        // Dialogs: keycode picker, Load list, color picker, image picker.
        if self.bind_picker_open {
            if self.load_menu.is_some() {
                self.draw_preset_list(ui, &mut t_overlay, w, h);
            } else if self.color_menu.is_some() {
                self.draw_color_picker(ui, &mut t_overlay, w, h);
            } else if self.image_menu {
                self.draw_image_picker(ui, &mut t_overlay, w, h);
            } else {
                self.draw_bind_picker(ui, &mut t_overlay, w, h);
            }
        }

        // Status hint, bottom left.
        if !self.status_hint.is_empty() {
            let line = ui.fit_text(Face::LauncherRegular, 11.0, &self.status_hint, w - 110.0);
            ui.text(Face::LauncherRegular, 11.0, 10.0, h - 18.0, theme::MUTED(), &line);
        }

        // Dialog targets first: the first matching rect wins the hit test.
        targets.extend(t_overlay);
        targets.extend(t_main);
    }

    /// Draws every layout button on the full-screen canvas.
    fn draw_canvas_buttons(&mut self, ui: &mut Ui, aspect: f32) {
        let [_, _, cw, ch] = self.canvas_rect;

        // Movement joystick.
        if let Some([nx, ny, nw, nh]) = self.target_norm_rect(StudioTarget::Joystick, aspect) {
            let (rx, ry, rw, rh) = (nx * cw, ny * ch, nw * cw, nh * ch);
            let sel = self.selected == Some(StudioTarget::Joystick);
            let r = (rw.min(rh) * 0.5).clamp(8.0, 120.0);
            ui.rounded(rx, ry, rw, rh, r, hex(0x181A24, 150));
            ui.border(rx, ry, rw, rh, r, if sel { theme::CRIMSON_BRIGHT() } else { hex(0xD8DCE8, 95) });
            let knob = (rw.min(rh) * 0.36).max(10.0);
            ui.rounded(
                rx + (rw - knob) * 0.5,
                ry + (rh - knob) * 0.5,
                knob,
                knob,
                knob * 0.5,
                if sel { theme::CRIMSON() } else { hex(0xDC143C, 155) },
            );
        }

        // Built-in buttons when editing the stock Volt Default layout.
        if self.settings.custom_buttons.is_empty() {
            for control in Control::ALL {
                if control == Control::Move {
                    continue;
                }
                let target = StudioTarget::Builtin(control as usize);
                let Some([nx, ny, nw, nh]) = self.target_norm_rect(target, aspect) else {
                    continue;
                };
                let (rx, ry, rw, rh) = (nx * cw, ny * ch, nw * cw, nh * ch);
                let sel = self.selected == Some(target);
                ui.rounded(rx, ry, rw, rh, 6.0, hex(0x1E202C, 175));
                ui.border(rx, ry, rw, rh, 6.0, if sel { theme::CRIMSON_BRIGHT() } else { hex(0xFFFFFF, 85) });
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

        // Custom buttons: fill, image, stroke width, label - live preview.
        for idx in 0..self.settings.custom_buttons.len() {
            let target = StudioTarget::Custom(idx);
            let Some([nx, ny, nw, nh]) = self.target_norm_rect(target, aspect) else {
                continue;
            };
            let (rx, ry) = (nx * cw, ny * ch);
            let (rw, rh) = ((nw * cw).max(14.0), (nh * ch).max(14.0));
            let b = self.settings.custom_buttons[idx].clone();
            let p = b.layouts[self.settings.mode.index()];
            let sel = self.selected == Some(target);
            let bg = b.bg_color.unwrap_or([0, 0, 0, 115]);
            let alpha = ((f32::from(bg[3]) / 255.0) * p.opacity * 255.0)
                .round()
                .clamp(24.0, 250.0) as u8;
            let fill = hex(
                (u32::from(bg[0]) << 16) | (u32::from(bg[1]) << 8) | u32::from(bg[2]),
                alpha,
            );
            let rad = b.corner_radius.unwrap_or(0.0).min(rw * 0.48).min(rh * 0.48);
            if b.background_image.is_none() {
                ui.rounded(rx, ry, rw, rh, rad.max(1.0), fill);
            } else if let Some(name) = b.background_image.clone()
                && let Some((hash, iw, ih)) = self.image_preview(&name)
                && let Some((_, _, _, rgba)) =
                    self.images.get(&name).and_then(|v| v.as_ref()).cloned()
            {
                let slot = 4 + idx % 24;
                let a = (p.opacity * 255.0).round().clamp(40.0, 255.0) as u8;
                ui.draw_dynamic_rgba(slot, hash, iw, ih, &rgba, rx, ry, rw, rh, a);
            } else {
                ui.rounded(rx, ry, rw, rh, rad.max(1.0), fill);
            }
            let stroke_col = if sel {
                theme::CRIMSON_BRIGHT()
            } else if let Some(sc) = b.stroke_color {
                hex(
                    (u32::from(sc[0]) << 16) | (u32::from(sc[1]) << 8) | u32::from(sc[2]),
                    sc[3].max(70),
                )
            } else {
                hex(0xFFFFFF, 85)
            };
            ui.border(rx, ry, rw, rh, rad, stroke_col);
            // A thicker Pojav stroke reads as a second inner border.
            if b.stroke_width.is_some_and(|sw| sw >= 2.0) {
                ui.border(rx + 1.0, ry + 1.0, rw - 2.0, rh - 2.0, (rad - 1.0).max(0.0), stroke_col);
            }
            if sel {
                ui.border(rx - 1.5, ry - 1.5, rw + 3.0, rh + 3.0, rad + 1.5, theme::CRIMSON());
            }
            let lbl = b.label();
            let font_sz = (rh * 0.34).clamp(8.5, 13.0);
            draw_label_centered(ui, [rx, ry, rw, rh], font_sz, theme::TEXT(), &lbl);
        }
    }

    /// The gear button: ring, teeth and hub, drawn from primitives.
    fn draw_cogwheel(&mut self, ui: &mut Ui, r: [f32; 4]) {
        let [x, y, w, h] = r;
        let c = (x + w * 0.5, y + h * 0.5);
        ui.rounded(x, y, w, h, w * 0.5, hex(0x1E202C, 220));
        ui.border(x, y, w, h, w * 0.5, theme::CRIMSON());
        // Four teeth, N / S / E / W.
        let t = 5.0;
        ui.rounded(c.0 - t * 0.5, y + 3.0, t, 6.0, 1.5, hex(0xE8E8F0, 230));
        ui.rounded(c.0 - t * 0.5, y + h - 9.0, t, 6.0, 1.5, hex(0xE8E8F0, 230));
        ui.rounded(x + 3.0, c.1 - t * 0.5, 6.0, t, 1.5, hex(0xE8E8F0, 230));
        ui.rounded(x + w - 9.0, c.1 - t * 0.5, 6.0, t, 1.5, hex(0xE8E8F0, 230));
        // Ring and hub.
        let ring = w * 0.46;
        ui.border(c.0 - ring * 0.5, c.1 - ring * 0.5, ring, ring, ring * 0.5, hex(0xE8E8F0, 230));
        let hub = w * 0.16;
        ui.rounded(c.0 - hub * 0.5, c.1 - hub * 0.5, hub, hub, hub * 0.5, hex(0xE8E8F0, 230));
    }

    /// DELETE / CLONE chips over the selected button plus the teal resize
    /// handle at its bottom-right corner.
    fn draw_selection_chrome(
        &mut self,
        ui: &mut Ui,
        t_main: &mut Vec<([f32; 4], Action)>,
        aspect: f32,
    ) {
        let Some(target) = self.selected else { return };
        let Some([nx, ny, nw, nh]) = self.target_norm_rect(target, aspect) else {
            return;
        };
        let [_, _, cw, ch] = self.canvas_rect;
        let (rx, ry, rw, rh) = (nx * cw, ny * ch, (nw * cw).max(14.0), (nh * ch).max(14.0));

        // Chips above the button; below it when it sits at the top edge.
        let chip_w = 58.0;
        let chip_h = 20.0;
        let chips_y = if ry > chip_h + 8.0 { ry - chip_h - 5.0 } else { ry + rh + 5.0 };
        let total = chip_w * 2.0 + 6.0;
        let chips_x = (rx + rw * 0.5 - total * 0.5).clamp(4.0, cw - total - 4.0);
        let del_r = [chips_x, chips_y, chip_w, chip_h];
        let clone_r = [chips_x + chip_w + 6.0, chips_y, chip_w, chip_h];
        ui.rounded(del_r[0], del_r[1], del_r[2], del_r[3], 10.0, hex(0x5A1420, 235));
        ui.border(del_r[0], del_r[1], del_r[2], del_r[3], 10.0, theme::CRIMSON_BRIGHT());
        draw_label_centered(ui, del_r, 10.0, hex(0xFFD8DE, 255), "DELETE");
        t_main.push((del_r, Action::Studio(StudioAction::DeleteSelected)));
        self.chrome_rects.push(del_r);
        if matches!(target, StudioTarget::Custom(_)) {
            ui.rounded(clone_r[0], clone_r[1], clone_r[2], clone_r[3], 10.0, hex(0x14323C, 235));
            ui.border(clone_r[0], clone_r[1], clone_r[2], clone_r[3], 10.0, hex(0x2EE6A8, 220));
            draw_label_centered(ui, clone_r, 10.0, hex(0xCFF8EA, 255), "CLONE");
            t_main.push((clone_r, Action::Studio(StudioAction::DuplicateSelected)));
            self.chrome_rects.push(clone_r);
        }

        // The draggable resize handle (teal, like Pojav's).
        let hs = 13.0;
        let hr = [rx + rw - hs * 0.5, ry + rh - hs * 0.5, hs, hs];
        ui.rounded(hr[0], hr[1], hr[2], hr[3], 3.0, hex(0x2EE6A8, 240));
        ui.border(hr[0], hr[1], hr[2], hr[3], 3.0, hex(0x0B0D13, 255));
        self.handle_rect = Some(hr);
    }

    /// The right-slide context menu with Pojav's seven entries.
    fn draw_context_menu(
        &mut self,
        ui: &mut Ui,
        t_overlay: &mut Vec<([f32; 4], Action)>,
        w: f32,
        h: f32,
    ) {
        let mw = 212.0_f32.min(w * 0.46);
        let mx = w - mw;
        ui.batch.fill(
            ui.atlas,
            0.0,
            0.0,
            w * (ui.density / ui.batch.scale),
            h * (ui.density / ui.batch.scale),
            hex(0x000000, 90),
        );
        ui.rounded(mx, 0.0, mw, h, 0.0, hex(0x14161E, 250));
        ui.rounded(mx, 0.0, 2.0, h, 0.0, theme::CRIMSON());
        let rows: [(&str, StudioAction); 7] = [
            ("Add button", StudioAction::AddButton),
            ("Add button drawer", StudioAction::AddDrawer),
            ("Add joystick", StudioAction::AddJoystick),
            ("Load", StudioAction::CtxLoad),
            ("Save", StudioAction::SavePreset),
            ("Select default controlmap", StudioAction::CtxSelectDefault),
            ("Export controlmap", StudioAction::ExportJson),
        ];
        let row_h = ((h - 20.0) / rows.len() as f32).clamp(26.0, 42.0);
        let mut y = ((h - row_h * rows.len() as f32) * 0.5).max(10.0);
        for (label, act) in rows {
            let r = [mx, y, mw, row_h];
            let text = ui.fit_text(Face::LauncherRegular, 12.5, label, mw - 26.0);
            ui.text(
                Face::LauncherRegular,
                12.5,
                mx + 16.0,
                y + (row_h - 13.0) * 0.5,
                hex(0xE8E8F0, 240),
                &text,
            );
            ui.rounded(mx + 10.0, y + row_h - 1.0, mw - 20.0, 1.0, 0.0, hex(0xFFFFFF, 22));
            t_overlay.push((r, Action::Studio(act)));
            y += row_h;
        }
        // A tap left of the menu closes it, exactly like Pojav.
        t_overlay.push(([0.0, 0.0, w, h], Action::Studio(StudioAction::CloseOverlay)));
    }

    /// The right-hand edit panel for the selected target.
    fn draw_edit_panel(
        &mut self,
        ui: &mut Ui,
        t_main: &mut Vec<([f32; 4], Action)>,
        w: f32,
        h: f32,
    ) {
        let pw = 236.0_f32.min(w * 0.46);
        let px = w - pw - 6.0;
        let py = 6.0;
        let ph = h - 12.0;
        ui.rounded(px, py, pw, ph, 12.0, hex(0x14161E, 246));
        ui.border(px, py, pw, ph, 12.0, theme::BORDER());
        self.panel_rect = Some([px, py, pw, ph]);
        let x = px + 10.0;
        let iw = pw - 20.0;
        let mut y = py + 8.0;

        match self.selected {
            Some(StudioTarget::Custom(idx)) => {
                let Some(b) = self.settings.custom_buttons.get(idx).cloned() else {
                    return;
                };
                // Name field.
                let name_r = [x, y, iw, 24.0];
                ui.rounded(name_r[0], name_r[1], name_r[2], name_r[3], 6.0, hex(0x0B0D13, 255));
                ui.border(
                    name_r[0],
                    name_r[1],
                    name_r[2],
                    name_r[3],
                    6.0,
                    if self.editing_label { theme::CRIMSON_BRIGHT() } else { hex(0xFFFFFF, 60) },
                );
                let name = if self.editing_label {
                    format!("{}_", self.label_draft)
                } else {
                    b.label()
                };
                let name_fit = ui.fit_text(Face::LauncherRegular, 12.0, &name, iw - 16.0);
                ui.text(Face::LauncherRegular, 12.0, x + 8.0, y + 6.0, hex(0xE8E8F0, 245), &name_fit);
                t_main.push((name_r, Action::Studio(StudioAction::EditLabel)));
                y += 30.0;

                // Size: [W] x [H].
                ui.text(Face::LauncherRegular, 11.0, x, y + 4.0, theme::MUTED(), "Size");
                let fw = (iw - 46.0 - 14.0) / 2.0;
                let h_px = b.layouts[self.settings.mode.index()].size * REF_H;
                let w_px = h_px * b.width_ratio.clamp(0.2, 8.0);
                let w_txt = if self.editing_size == Some(true) {
                    format!("{}_", self.size_draft)
                } else {
                    format!("{w_px:.0}")
                };
                let h_txt = if self.editing_size == Some(false) {
                    format!("{}_", self.size_draft)
                } else {
                    format!("{h_px:.0}")
                };
                let wr = [x + 34.0, y, fw, 22.0];
                let hr2 = [x + 34.0 + fw + 14.0, y, fw, 22.0];
                for (r, txt, editing) in [
                    (wr, &w_txt, self.editing_size == Some(true)),
                    (hr2, &h_txt, self.editing_size == Some(false)),
                ] {
                    ui.rounded(r[0], r[1], r[2], r[3], 5.0, hex(0x0B0D13, 255));
                    ui.border(
                        r[0],
                        r[1],
                        r[2],
                        r[3],
                        5.0,
                        if editing { theme::CRIMSON_BRIGHT() } else { hex(0xFFFFFF, 60) },
                    );
                    draw_label_centered(ui, r, 11.0, hex(0xE8E8F0, 240), txt);
                }
                draw_label_centered(
                    ui,
                    [x + 34.0 + fw, y, 14.0, 22.0],
                    11.0,
                    theme::MUTED(),
                    "x",
                );
                t_main.push((wr, Action::Studio(StudioAction::EditSize(true))));
                t_main.push((hr2, Action::Studio(StudioAction::EditSize(false))));
                y += 28.0;

                let is_drawer = b.key.starts_with("special:drawer:");
                if is_drawer {
                    let add_r = [x, y, iw, 22.0];
                    ui.rounded(add_r[0], add_r[1], add_r[2], add_r[3], 6.0, theme::CRIMSON_BADGE());
                    ui.border(add_r[0], add_r[1], add_r[2], add_r[3], 6.0, theme::CRIMSON());
                    draw_label_centered(ui, add_r, 10.5, theme::TEXT(), "+ Sub-button");
                    t_main.push((add_r, Action::Studio(StudioAction::AddDrawerSub)));
                    y += 28.0;
                } else {
                    // Mapping: the Pojav 2x2 keycode grid.
                    ui.text(Face::LauncherRegular, 11.0, x, y, theme::MUTED(), "Mapping");
                    y += 15.0;
                    let slot_w = (iw - 6.0) / 2.0;
                    for slot in 0..4usize {
                        let col = slot % 2;
                        let row = slot / 2;
                        let r = [
                            x + col as f32 * (slot_w + 6.0),
                            y + row as f32 * 25.0,
                            slot_w,
                            21.0,
                        ];
                        let token: &str = if slot == 0 {
                            &b.key
                        } else {
                            b.extra_keys.get(slot - 1).map(String::as_str).unwrap_or("")
                        };
                        let disp = pojav_name(token);
                        ui.rounded(r[0], r[1], r[2], r[3], 5.0, hex(0x0B0D13, 255));
                        ui.border(r[0], r[1], r[2], r[3], 5.0, hex(0xFFFFFF, 55));
                        let fit = ui.fit_text(Face::LauncherRegular, 9.5, &disp, slot_w - 8.0);
                        draw_label_centered(ui, r, 9.5, hex(0xD8D8E4, 235), &fit);
                        t_main.push((r, Action::Studio(StudioAction::BindSlot(slot))));
                    }
                    y += 54.0;
                }

                // Toggles.
                for (label, on, act) in [
                    ("Toggleable", b.toggle, StudioAction::ToggleHoldLatch),
                    ("Mouse pass-thru", b.passthrough, StudioAction::TogglePassthrough),
                    ("Swipeable", b.swipeable, StudioAction::ToggleSwipeable),
                ] {
                    let r = draw_checkbox_row(ui, [x, y, iw, 19.0], label, on);
                    t_main.push((r, Action::Studio(act)));
                    y += 21.0;
                }
                y += 3.0;

                // Background image / Background color / Stroke color rows.
                let img_label = b.background_image.as_deref().unwrap_or("None");
                let r = draw_arrow_row(ui, [x, y, iw, 21.0], "Background image", img_label, None);
                t_main.push((r, Action::Studio(StudioAction::OpenBgImage)));
                y += 23.0;
                let r = draw_arrow_row(
                    ui,
                    [x, y, iw, 21.0],
                    "Background color",
                    "",
                    Some(b.bg_color.unwrap_or([0, 0, 0, 115])),
                );
                t_main.push((r, Action::Studio(StudioAction::OpenBgColor)));
                y += 23.0;
                let r = draw_arrow_row(
                    ui,
                    [x, y, iw, 21.0],
                    "Stroke color",
                    "",
                    Some(b.stroke_color.unwrap_or([255, 255, 255, 0])),
                );
                t_main.push((r, Action::Studio(StudioAction::OpenStrokeColor)));
                y += 25.0;

                // Percent sliders, Pojav-style.
                let m = self.selected_min_side().unwrap_or(50.0);
                let stroke_pct =
                    b.stroke_width.map_or(0.0, |sw| (sw / (m * 0.5) * 100.0).clamp(0.0, 100.0));
                let corner_pct = b
                    .corner_radius
                    .map_or(0.0, |cr| (cr / (m * 0.5) * 100.0).clamp(0.0, 100.0));
                let opacity_pct =
                    (b.layouts[self.settings.mode.index()].opacity * 100.0).clamp(10.0, 100.0);
                for (label, pct, kind) in [
                    ("Stroke width", stroke_pct, Slider::StrokeWidth),
                    ("Corner radius", corner_pct, Slider::CornerRadius),
                    ("Button Opacity", opacity_pct, Slider::Opacity),
                ] {
                    let track = draw_slider_row(ui, [x, y, iw, 26.0], label, pct);
                    self.slider_rects.push((track, kind));
                    y += 28.0;
                }

                // Visibility checkboxes.
                ui.text(Face::LauncherRegular, 11.0, x, y, theme::MUTED(), "Visibility");
                y += 14.0;
                let half = (iw - 6.0) / 2.0;
                let r1 = draw_checkbox_row(ui, [x, y, half, 19.0], "In-game", b.in_game);
                t_main.push((r1, Action::Studio(StudioAction::ToggleInGame)));
                let r2 =
                    draw_checkbox_row(ui, [x + half + 6.0, y, half, 19.0], "In menus", b.in_menu);
                t_main.push((r2, Action::Studio(StudioAction::ToggleInMenu)));
                y += 23.0;

                if b.drawer_parent.is_some() {
                    let r = [x, y, iw, 20.0];
                    ui.rounded(r[0], r[1], r[2], r[3], 6.0, hex(0x1E202C, 220));
                    ui.border(r[0], r[1], r[2], r[3], 6.0, hex(0xFFFFFF, 55));
                    draw_label_centered(ui, r, 10.0, theme::TEXT(), "Detach from drawer");
                    t_main.push((r, Action::Studio(StudioAction::DetachFromDrawer)));
                }
            }
            Some(StudioTarget::Joystick) => {
                ui.text(Face::LauncherMedium, 13.0, x, y + 2.0, hex(0xE8E8F0, 245), "Joystick");
                y += 24.0;
                let size = self.settings.layouts[self.settings.mode.index()]
                    [Control::Move as usize]
                    .size;
                let pct = ((size - 0.14) / (0.45 - 0.14) * 100.0).clamp(0.0, 100.0);
                let track = draw_slider_row(ui, [x, y, iw, 26.0], "Size", pct);
                self.slider_rects.push((track, Slider::JoystickSize));
                y += 28.0;
                let opacity = self.settings.layouts[self.settings.mode.index()]
                    [Control::Move as usize]
                    .opacity
                    * 100.0;
                let track =
                    draw_slider_row(ui, [x, y, iw, 26.0], "Opacity", opacity.clamp(10.0, 100.0));
                self.slider_rects.push((track, Slider::Opacity));
                y += 30.0;
                let r = draw_checkbox_row(
                    ui,
                    [x, y, iw, 19.0],
                    "Forward lock (sprint)",
                    self.settings.sprint_with_joystick,
                );
                t_main.push((r, Action::Studio(StudioAction::ToggleForwardLock)));
                y += 22.0;
                let dpad = self.settings.mode == horizon_client::touch::Mode::Dpad;
                let r = draw_checkbox_row(ui, [x, y, iw, 19.0], "D-pad style (absolute)", dpad);
                t_main.push((r, Action::Studio(StudioAction::ToggleDpadStyle)));
            }
            Some(StudioTarget::Builtin(idx)) if idx < 14 => {
                ui.text(Face::LauncherMedium, 13.0, x, y + 2.0, hex(0xE8E8F0, 245), "Stock button");
                y += 24.0;
                let size = self.settings.layouts[self.settings.mode.index()][idx].size;
                let pct = ((size - 0.05) / (0.35 - 0.05) * 100.0).clamp(0.0, 100.0);
                let track = draw_slider_row(ui, [x, y, iw, 26.0], "Size", pct);
                self.slider_rects.push((track, Slider::BuiltinSize));
                y += 28.0;
                let opacity = self.settings.layouts[self.settings.mode.index()][idx].opacity * 100.0;
                let track =
                    draw_slider_row(ui, [x, y, iw, 26.0], "Opacity", opacity.clamp(10.0, 100.0));
                self.slider_rects.push((track, Slider::Opacity));
                y += 30.0;
                let note = "Stock Volt controls: custom buttons replace these when you add any.";
                for line in wrap_text(ui, note, Face::LauncherRegular, 10.5, iw) {
                    ui.text(Face::LauncherRegular, 10.5, x, y, theme::MUTED(), &line);
                    y += 14.0;
                }
            }
            _ => {}
        }
    }

    /// The full keycode radio list, paged like a scrolled Pojav dialog.
    fn draw_bind_picker(
        &mut self,
        ui: &mut Ui,
        t_overlay: &mut Vec<([f32; 4], Action)>,
        w: f32,
        h: f32,
    ) {
        let scale = ui.density / ui.batch.scale;
        ui.batch
            .fill(ui.atlas, 0.0, 0.0, w * scale, h * scale, hex(0x000000, 150));
        let mw = (w * 0.74).clamp(300.0, 560.0);
        let mh = (h * 0.84).clamp(220.0, 430.0);
        let mx = (w - mw) * 0.5;
        let my = (h - mh) * 0.5;
        ui.rounded(mx, my, mw, mh, 12.0, hex(0x14161E, 252));
        ui.border(mx, my, mw, mh, 12.0, theme::CRIMSON());
        let title = format!("Choose a keycode - slot {}", self.bind_slot + 1);
        ui.text(Face::LauncherMedium, 13.0, mx + 14.0, my + 10.0, hex(0xE8E8F0, 250), &title);

        let entries = keycode_entries();
        let pages = entries.len().div_ceil(PICKER_PER_PAGE).max(1);
        let page = self.picker_page.min(pages - 1);
        let selected_token: Option<String> = if let Some(StudioTarget::Custom(idx)) = self.selected
        {
            self.settings.custom_buttons.get(idx).map(|b| {
                if self.bind_slot == 0 {
                    b.key.clone()
                } else {
                    b.extra_keys
                        .get(self.bind_slot - 1)
                        .cloned()
                        .unwrap_or_default()
                }
            })
        } else {
            None
        };

        let cols = 4usize;
        let gx = mx + 12.0;
        let gy = my + 32.0;
        let gw = mw - 24.0;
        let cell_w = (gw - (cols as f32 - 1.0) * 6.0) / cols as f32;
        let cell_h = 24.0;
        let rows = PICKER_PER_PAGE / cols;
        for (k, entry) in entries
            .iter()
            .enumerate()
            .skip(page * PICKER_PER_PAGE)
            .take(PICKER_PER_PAGE)
        {
            let (disp, token) = (&entry.0, entry.1);
            let at = k - page * PICKER_PER_PAGE;
            let col = at % cols;
            let row = at / cols;
            if row >= rows {
                break;
            }
            let r = [
                gx + col as f32 * (cell_w + 6.0),
                gy + row as f32 * (cell_h + 5.0),
                cell_w,
                cell_h,
            ];
            let active = selected_token.as_deref() == Some(token)
                || (token.is_empty()
                    && selected_token.as_deref().is_some_and(|t| {
                        t.is_empty() || t == "key.keyboard.unknown"
                    }));
            // The radio dot.
            let dot = 10.0;
            ui.rounded(r[0], r[1], r[2], r[3], 6.0, if active { theme::CRIMSON_BADGE() } else { hex(0x0B0D13, 240) });
            ui.border(r[0], r[1], r[2], r[3], 6.0, if active { theme::CRIMSON_BRIGHT() } else { hex(0xFFFFFF, 45) });
            ui.border(r[0] + 6.0, r[1] + (cell_h - dot) * 0.5, dot, dot, dot * 0.5, hex(0xD8D8E4, 200));
            if active {
                ui.rounded(r[0] + 8.5, r[1] + (cell_h - dot) * 0.5 + 2.5, 5.0, 5.0, 2.5, theme::CRIMSON_BRIGHT());
            }
            let fit = ui.fit_text(Face::LauncherRegular, 9.0, disp, cell_w - 24.0);
            ui.text(
                Face::LauncherRegular,
                9.0,
                r[0] + 20.0,
                r[1] + (cell_h - 9.0) * 0.5,
                hex(0xE0E0EA, 240),
                &fit,
            );
            t_overlay.push((r, Action::Studio(StudioAction::PickBinding(k))));
        }

        // Pager + close.
        let py = my + mh - 30.0;
        let prev_r = [mx + 12.0, py, 64.0, 22.0];
        let next_r = [mx + 82.0, py, 64.0, 22.0];
        let close_r = [mx + mw - 76.0, py, 64.0, 22.0];
        for (r, label, act) in [
            (prev_r, "< Prev", StudioAction::PickerPage(false)),
            (next_r, "Next >", StudioAction::PickerPage(true)),
            (close_r, "Close", StudioAction::CloseOverlay),
        ] {
            ui.rounded(r[0], r[1], r[2], r[3], 6.0, hex(0x1E202C, 230));
            ui.border(r[0], r[1], r[2], r[3], 6.0, hex(0xFFFFFF, 60));
            draw_label_centered(ui, r, 10.0, theme::TEXT(), label);
            t_overlay.push((r, Action::Studio(act)));
        }
        let page_txt = format!("{} / {pages}", page + 1);
        let tw = ui.measure(Face::LauncherRegular, 10.0, &page_txt);
        ui.text(Face::LauncherRegular, 10.0, mx + (mw - tw) * 0.5, py + 5.0, theme::MUTED(), &page_txt);

        t_overlay.push(([0.0, 0.0, w, h], Action::Studio(StudioAction::CloseOverlay)));
    }

    /// The Load / Select-default list of saved controlmaps.
    fn draw_preset_list(
        &mut self,
        ui: &mut Ui,
        t_overlay: &mut Vec<([f32; 4], Action)>,
        w: f32,
        h: f32,
    ) {
        let scale = ui.density / ui.batch.scale;
        ui.batch
            .fill(ui.atlas, 0.0, 0.0, w * scale, h * scale, hex(0x000000, 150));
        let mw = (w * 0.56).clamp(260.0, 420.0);
        let mh = (h * 0.8).clamp(200.0, 400.0);
        let mx = (w - mw) * 0.5;
        let my = (h - mh) * 0.5;
        ui.rounded(mx, my, mw, mh, 12.0, hex(0x14161E, 252));
        ui.border(mx, my, mw, mh, 12.0, theme::CRIMSON());
        let title = if self.load_menu == Some(true) {
            "Select default controlmap"
        } else {
            "Load controlmap"
        };
        ui.text(Face::LauncherMedium, 13.0, mx + 14.0, my + 10.0, hex(0xE8E8F0, 250), title);
        let mut y = my + 34.0;
        if self.presets.is_empty() {
            ui.text(
                Face::LauncherRegular,
                11.0,
                mx + 14.0,
                y,
                theme::MUTED(),
                "No saved controlmaps yet - Save one from the context menu.",
            );
        }
        for (i, entry) in self.presets.iter().take(9).enumerate() {
            let r = [mx + 10.0, y, mw - 20.0, 26.0];
            ui.rounded(r[0], r[1], r[2], r[3], 7.0, hex(0x0B0D13, 240));
            ui.border(r[0], r[1], r[2], r[3], 7.0, hex(0xFFFFFF, 45));
            let name = ui.fit_text(Face::LauncherRegular, 11.5, &entry.meta.name, r[2] - 90.0);
            ui.text(Face::LauncherRegular, 11.5, r[0] + 10.0, y + 7.0, hex(0xE8E8F0, 240), &name);
            let info = format!("{} btns", entry.meta.buttons);
            let tw = ui.measure(Face::LauncherRegular, 10.0, &info);
            ui.text(Face::LauncherRegular, 10.0, r[0] + r[2] - tw - 10.0, y + 8.0, theme::MUTED(), &info);
            t_overlay.push((r, Action::Studio(StudioAction::PickPreset(i))));
            y += 30.0;
        }
        t_overlay.push(([0.0, 0.0, w, h], Action::Studio(StudioAction::CloseOverlay)));
    }

    /// The Background / Stroke color picker: swatch grid plus alpha slider.
    fn draw_color_picker(
        &mut self,
        ui: &mut Ui,
        t_overlay: &mut Vec<([f32; 4], Action)>,
        w: f32,
        h: f32,
    ) {
        let stroke = self.color_menu == Some(true);
        let scale = ui.density / ui.batch.scale;
        ui.batch
            .fill(ui.atlas, 0.0, 0.0, w * scale, h * scale, hex(0x000000, 150));
        let mw = (w * 0.56).clamp(280.0, 420.0);
        let mh = (h * 0.76).clamp(210.0, 360.0);
        let mx = (w - mw) * 0.5;
        let my = (h - mh) * 0.5;
        ui.rounded(mx, my, mw, mh, 12.0, hex(0x14161E, 252));
        ui.border(mx, my, mw, mh, 12.0, theme::CRIMSON());
        let title = if stroke { "Stroke color" } else { "Background color" };
        ui.text(Face::LauncherMedium, 13.0, mx + 14.0, my + 10.0, hex(0xE8E8F0, 250), title);

        let current = if let Some(StudioTarget::Custom(idx)) = self.selected {
            self.settings.custom_buttons.get(idx).and_then(|b| {
                if stroke { b.stroke_color } else { b.bg_color }
            })
        } else {
            None
        };

        let cols = 6usize;
        let gx = mx + 14.0;
        let gy = my + 34.0;
        let cell = ((mw - 28.0 - (cols as f32 - 1.0) * 8.0) / cols as f32).min(34.0);
        for (i, rgb) in STUDIO_PALETTE.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let r = [
                gx + col as f32 * (cell + 8.0),
                gy + row as f32 * (cell + 8.0),
                cell,
                cell,
            ];
            let swatch = hex(
                (u32::from(rgb[0]) << 16) | (u32::from(rgb[1]) << 8) | u32::from(rgb[2]),
                255,
            );
            ui.rounded(r[0], r[1], r[2], r[3], 6.0, swatch);
            let active = current.is_some_and(|c| c[0] == rgb[0] && c[1] == rgb[1] && c[2] == rgb[2]);
            ui.border(
                r[0] - 1.0,
                r[1] - 1.0,
                r[2] + 2.0,
                r[3] + 2.0,
                7.0,
                if active { theme::CRIMSON_BRIGHT() } else { hex(0xFFFFFF, 70) },
            );
            t_overlay.push((r, Action::Studio(StudioAction::PickColor(i))));
        }
        let rows = STUDIO_PALETTE.len().div_ceil(cols) as f32;
        let mut y = gy + rows * (cell + 8.0) + 4.0;

        // Alpha slider.
        let alpha_pct = f32::from(current.map_or(255, |c| c[3])) / 255.0 * 100.0;
        let track = draw_slider_row(ui, [mx + 14.0, y, mw - 28.0, 26.0], "Opacity", alpha_pct);
        self.slider_rects.push((track, Slider::ColorAlpha));
        y += 32.0;

        let none_r = [mx + 14.0, y, (mw - 36.0) / 2.0, 22.0];
        let done_r = [mx + 22.0 + (mw - 36.0) / 2.0, y, (mw - 36.0) / 2.0, 22.0];
        ui.rounded(none_r[0], none_r[1], none_r[2], none_r[3], 6.0, hex(0x1E202C, 230));
        ui.border(none_r[0], none_r[1], none_r[2], none_r[3], 6.0, hex(0xFFFFFF, 60));
        draw_label_centered(ui, none_r, 10.0, theme::TEXT(), if stroke { "No stroke" } else { "Default" });
        t_overlay.push((none_r, Action::Studio(StudioAction::ClearColor)));
        ui.rounded(done_r[0], done_r[1], done_r[2], done_r[3], 6.0, theme::CRIMSON());
        draw_label_centered(ui, done_r, 10.0, theme::ON_CRIMSON(), "Done");
        t_overlay.push((done_r, Action::Studio(StudioAction::CloseOverlay)));

        t_overlay.push(([0.0, 0.0, w, h], Action::Studio(StudioAction::CloseOverlay)));
    }

    /// The `Background image >` picker listing `button-images/*.png`.
    fn draw_image_picker(
        &mut self,
        ui: &mut Ui,
        t_overlay: &mut Vec<([f32; 4], Action)>,
        w: f32,
        h: f32,
    ) {
        let scale = ui.density / ui.batch.scale;
        ui.batch
            .fill(ui.atlas, 0.0, 0.0, w * scale, h * scale, hex(0x000000, 150));
        let mw = (w * 0.56).clamp(280.0, 420.0);
        let mh = (h * 0.8).clamp(210.0, 400.0);
        let mx = (w - mw) * 0.5;
        let my = (h - mh) * 0.5;
        ui.rounded(mx, my, mw, mh, 12.0, hex(0x14161E, 252));
        ui.border(mx, my, mw, mh, 12.0, theme::CRIMSON());
        ui.text(Face::LauncherMedium, 13.0, mx + 14.0, my + 10.0, hex(0xE8E8F0, 250), "Background image");
        self.refresh_image_files();
        let mut y = my + 34.0;

        let import_r = [mx + 10.0, y, mw - 20.0, 24.0];
        ui.rounded(import_r[0], import_r[1], import_r[2], import_r[3], 7.0, theme::CRIMSON_BADGE());
        ui.border(import_r[0], import_r[1], import_r[2], import_r[3], 7.0, theme::CRIMSON());
        draw_label_centered(ui, import_r, 10.5, theme::TEXT(), "+ Import PNG");
        t_overlay.push((import_r, Action::Studio(StudioAction::ImportBgImage)));
        y += 28.0;

        let none_r = [mx + 10.0, y, mw - 20.0, 24.0];
        ui.rounded(none_r[0], none_r[1], none_r[2], none_r[3], 7.0, hex(0x0B0D13, 240));
        ui.border(none_r[0], none_r[1], none_r[2], none_r[3], 7.0, hex(0xFFFFFF, 45));
        ui.text(Face::LauncherRegular, 11.5, none_r[0] + 10.0, y + 6.0, hex(0xE8E8F0, 240), "None");
        t_overlay.push((none_r, Action::Studio(StudioAction::PickBgImage(0))));
        y += 28.0;

        if self.image_files.is_empty() {
            let note = "Put .png files into the profile's button-images folder, then reopen.";
            for line in wrap_text(ui, note, Face::LauncherRegular, 10.5, mw - 28.0) {
                ui.text(Face::LauncherRegular, 10.5, mx + 14.0, y, theme::MUTED(), &line);
                y += 14.0;
            }
        }
        let files = self.image_files.clone();
        for (i, name) in files.iter().enumerate() {
            if y + 26.0 > my + mh - 8.0 {
                break;
            }
            let r = [mx + 10.0, y, mw - 20.0, 24.0];
            ui.rounded(r[0], r[1], r[2], r[3], 7.0, hex(0x0B0D13, 240));
            ui.border(r[0], r[1], r[2], r[3], 7.0, hex(0xFFFFFF, 45));
            if let Some((hash, iw, ih)) = self.image_preview(name)
                && let Some((_, _, _, rgba)) =
                    self.images.get(name).and_then(|v| v.as_ref()).cloned()
            {
                ui.draw_dynamic_rgba(4 + (i % 24), hash, iw, ih, &rgba, r[0] + 4.0, y + 3.0, 18.0, 18.0, 255);
            }
            let fit = ui.fit_text(Face::LauncherRegular, 11.0, name, r[2] - 38.0);
            ui.text(Face::LauncherRegular, 11.0, r[0] + 28.0, y + 6.5, hex(0xE8E8F0, 240), &fit);
            t_overlay.push((r, Action::Studio(StudioAction::PickBgImage(i + 1))));
            y += 28.0;
        }
        t_overlay.push(([0.0, 0.0, w, h], Action::Studio(StudioAction::CloseOverlay)));
    }
}

/// Keycode picker entries per page (4 columns x 9 rows).
const PICKER_PER_PAGE: usize = 36;

fn draw_label_centered(ui: &mut Ui, rect: [f32; 4], size: f32, col: u32, text: &str) {
    let [x, y, w, h] = rect;
    let fitted = ui.fit_text(Face::LauncherMedium, size, text, w - 6.0);
    let tw = ui.measure(Face::LauncherMedium, size, &fitted);
    ui.text(
        Face::LauncherMedium,
        size,
        x + (w - tw) / 2.0,
        y + (h - size) / 2.0,
        col,
        &fitted,
    );
}

/// A `[label ......... [x]]` checkbox row. Returns its tap rect.
fn draw_checkbox_row(ui: &mut Ui, rect: [f32; 4], label: &str, on: bool) -> [f32; 4] {
    let [x, y, w, h] = rect;
    let box_s = 14.0;
    let bx = x + w - box_s - 2.0;
    let by = y + (h - box_s) / 2.0;
    let fit = ui.fit_text(Face::LauncherRegular, 11.0, label, w - box_s - 12.0);
    ui.text(Face::LauncherRegular, 11.0, x, y + (h - 11.0) / 2.0, hex(0xE0E0EA, 235), &fit);
    ui.rounded(bx, by, box_s, box_s, 3.0, if on { theme::CRIMSON() } else { hex(0x0B0D13, 255) });
    ui.border(bx, by, box_s, box_s, 3.0, if on { theme::CRIMSON_BRIGHT() } else { hex(0xFFFFFF, 80) });
    if on {
        // The check mark: two strokes.
        ui.rounded(bx + 3.0, by + 7.0, 3.5, 2.0, 1.0, theme::ON_CRIMSON());
        ui.rounded(bx + 5.5, by + 4.0, 2.0, 6.5, 1.0, theme::ON_CRIMSON());
    }
    rect
}

/// A `label  [value] >` row with an optional color swatch. Returns its rect.
fn draw_arrow_row(
    ui: &mut Ui,
    rect: [f32; 4],
    label: &str,
    value: &str,
    swatch: Option<[u8; 4]>,
) -> [f32; 4] {
    let [x, y, w, h] = rect;
    ui.rounded(x, y, w, h, 6.0, hex(0x0B0D13, 230));
    ui.border(x, y, w, h, 6.0, hex(0xFFFFFF, 40));
    let fit = ui.fit_text(Face::LauncherRegular, 11.0, label, w * 0.62);
    ui.text(Face::LauncherRegular, 11.0, x + 8.0, y + (h - 11.0) / 2.0, hex(0xE0E0EA, 235), &fit);
    let mut right = x + w - 16.0;
    ui.text(Face::LauncherMedium, 12.0, right + 4.0, y + (h - 12.0) / 2.0, theme::MUTED(), "\u{203A}");
    if let Some(c) = swatch {
        let s = h - 8.0;
        right -= s + 2.0;
        let col = hex((u32::from(c[0]) << 16) | (u32::from(c[1]) << 8) | u32::from(c[2]), c[3].max(60));
        ui.rounded(right, y + 4.0, s, s, 3.0, col);
        ui.border(right, y + 4.0, s, s, 3.0, hex(0xFFFFFF, 90));
    } else if !value.is_empty() {
        let fit = ui.fit_text(Face::LauncherRegular, 10.0, value, w * 0.3);
        let tw = ui.measure(Face::LauncherRegular, 10.0, &fit);
        right -= tw + 4.0;
        ui.text(Face::LauncherRegular, 10.0, right, y + (h - 10.0) / 2.0, theme::MUTED(), &fit);
    }
    rect
}

/// A `label [====|----] NN %` slider row. Returns the live track rect.
fn draw_slider_row(ui: &mut Ui, rect: [f32; 4], label: &str, pct: f32) -> [f32; 4] {
    let [x, y, w, _h] = rect;
    ui.text(Face::LauncherRegular, 10.5, x, y, theme::MUTED(), label);
    let pct_txt = format!("{pct:.0} %");
    let tw = ui.measure(Face::LauncherRegular, 10.5, &pct_txt);
    ui.text(Face::LauncherRegular, 10.5, x + w - tw, y, hex(0xE0E0EA, 235), &pct_txt);
    let ty = y + 14.0;
    let th = 10.0;
    let track = [x, ty - 3.0, w, th + 6.0];
    ui.rounded(x, ty + th / 2.0 - 2.0, w, 4.0, 2.0, hex(0x30323E, 255));
    let fill_w = (w * (pct / 100.0)).clamp(0.0, w);
    ui.rounded(x, ty + th / 2.0 - 2.0, fill_w.max(2.0), 4.0, 2.0, theme::CRIMSON());
    let kx = (x + fill_w - 5.0).clamp(x, x + w - 10.0);
    ui.rounded(kx, ty - 1.0, 10.0, 12.0, 5.0, hex(0xE8E8F0, 255));
    track
}

/// Greedy word wrap in dp for the given face / size.
fn wrap_text(ui: &Ui, text: &str, face: Face, size: f32, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let probe = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if ui.measure(face, size, &probe) > width && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
            line = word.to_owned();
        } else {
            line = probe;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
