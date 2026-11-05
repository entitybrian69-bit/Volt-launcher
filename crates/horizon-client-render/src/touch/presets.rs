//! Built-in touch control presets for **Volt Controls Studio** and in-game Touch Settings.
//! Ready to select with zero external files required:
//! - **Volt Default** (clean native analog/crosshair layout)
//! - **Pojav Classic** (swipeable WASD + PRI/SEC/MOUSE/GUI/INV/DEBUG/CHAT/TAB/3RD)
//! - **Zalith Layout** (analog left stick + sleek crimson-outlined rounded action cluster)
//! - **Fold Craft (FCL)** (analog stick + FCL quick-bar, wheel scroll & dual mouse triggers)
//! - **4-Finger PvP Claw** (top-corner index triggers + bottom-thumb stick/jump/crouch)

use super::{
    CustomButton,
    layout::{Control, ImportedMeta, Mode, Placement, Settings},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltInPreset {
    VoltDefault,
    PojavClassic,
    ZalithModern,
    FoldCraft,
    PvpClaw,
}

impl BuiltInPreset {
    pub const ALL: [Self; 5] = [
        Self::VoltDefault,
        Self::PojavClassic,
        Self::ZalithModern,
        Self::FoldCraft,
        Self::PvpClaw,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::VoltDefault => "volt-default",
            Self::PojavClassic => "pojav-classic",
            Self::ZalithModern => "zalith-modern",
            Self::FoldCraft => "fold-craft-fcl",
            Self::PvpClaw => "pvp-4finger-claw",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::VoltDefault => "Volt Default",
            Self::PojavClassic => "Pojav Classic",
            Self::ZalithModern => "Zalith Layout",
            Self::FoldCraft => "Fold Craft (FCL)",
            Self::PvpClaw => "4-Finger PvP Claw",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::VoltDefault => "Volt",
            Self::PojavClassic => "Pojav",
            Self::ZalithModern => "Zalith",
            Self::FoldCraft => "FCL",
            Self::PvpClaw => "4F Claw",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Self::VoltDefault => "Native analog stick + clean right-thumb actions",
            Self::PojavClassic => "Swipeable WASD + PRI / SEC / MOUSE / GUI / DEBUG / 3RD",
            Self::ZalithModern => "Analog stick + sleek rounded crimson-accented buttons",
            Self::FoldCraft => "FCL layout with hotbar scroll, PRI/SEC & quick utility bar",
            Self::PvpClaw => "Top index PRI/SEC triggers + thumb stick, jump & crouch",
        }
    }

    /// Detect which built-in preset matches `settings` (or `None` when custom/imported).
    pub fn detect(settings: &Settings) -> Option<Self> {
        let Some(meta) = &settings.imported_from else {
            return if settings.custom_buttons.is_empty() {
                Some(Self::VoltDefault)
            } else {
                None
            };
        };
        Self::ALL.into_iter().find(|p| p.label() == meta.name)
    }

    /// Build a complete, ready-to-play `Settings` instance for this preset.
    pub fn build(self) -> Settings {
        match self {
            Self::VoltDefault => {
                let mut s = Settings::default();
                s.analog_movement = true;
                s.sanitize();
                s
            }
            Self::PojavClassic => build_pojav_classic(),
            Self::ZalithModern => build_zalith_modern(),
            Self::FoldCraft => build_fold_craft(),
            Self::PvpClaw => build_pvp_claw(),
        }
    }
}

/// Curated palette of bindable tokens exposed in Controls Studio's visual key picker.
pub const STUDIO_BINDINGS: &[(&str, &str)] = &[
    ("special:mouse_pri", "Attack / Left Click (PRI)"),
    ("special:mouse_sec", "Use / Right Click (SEC)"),
    ("special:mouse_mid", "Pick Block (MID)"),
    ("special:virtual_mouse", "Virtual Mouse Toggle"),
    ("special:gui_toggle", "Hide HUD (GUI)"),
    ("special:keyboard", "On-Screen Keyboard"),
    ("special:menu", "Pause Menu (ESC)"),
    ("special:scroll_up", "Hotbar Scroll Up"),
    ("special:scroll_down", "Hotbar Scroll Down"),
    ("key.keyboard.space", "Jump (Space)"),
    ("key.keyboard.left.shift", "Sneak / Crouch (Shift)"),
    ("key.keyboard.left.control", "Sprint (Ctrl)"),
    ("key.keyboard.e", "Inventory (E)"),
    ("key.keyboard.f", "Swap Offhand (F)"),
    ("key.keyboard.q", "Drop Item (Q)"),
    ("key.keyboard.t", "Open Chat (T)"),
    ("key.keyboard.tab", "Player List (Tab)"),
    ("key.keyboard.f3", "Debug Overlay (F3)"),
    ("key.keyboard.f5", "Perspective (F5)"),
    ("key.keyboard.z", "Horizon Zoom (Z)"),
    ("key.keyboard.b", "New Waypoint (B)"),
    ("key.keyboard.w", "Move Forward (W)"),
    ("key.keyboard.a", "Strafe Left (A)"),
    ("key.keyboard.s", "Move Back (S)"),
    ("key.keyboard.d", "Strafe Right (D)"),
    ("key.keyboard.1", "Hotbar Slot 1"),
    ("key.keyboard.2", "Hotbar Slot 2"),
    ("key.keyboard.3", "Hotbar Slot 3"),
    ("key.keyboard.4", "Hotbar Slot 4"),
    ("key.keyboard.5", "Hotbar Slot 5"),
];

#[allow(clippy::too_many_arguments)]
fn btn(
    key: &str,
    label: &str,
    x: f32,
    y: f32,
    size: f32,
    width_ratio: f32,
    opacity: f32,
    toggle: bool,
    swipeable: bool,
    passthrough: bool,
    in_game: bool,
    in_menu: bool,
    bg: [u8; 4],
    stroke: [u8; 4],
    radius: Option<f32>,
) -> CustomButton {
    let p = Placement {
        center: [x, y],
        size,
        opacity,
    };
    CustomButton {
        key: key.to_owned(),
        layouts: [p; 3],
        extra_keys: Vec::new(),
        label: Some(label.to_owned()),
        width_ratio,
        toggle,
        swipeable,
        passthrough,
        in_game,
        in_menu,
        bg_color: Some(bg),
        stroke_color: Some(stroke),
        corner_radius: radius,
        drawer_parent: None,
    }
}

fn finalize_preset(
    name: &str,
    format: &str,
    joystick: Option<(Placement, bool)>,
    buttons: Vec<CustomButton>,
) -> Settings {
    let mut s = Settings::default();
    s.button_visibility = [2; 14];
    let has_joystick = if let Some((stick, sprint_forward)) = joystick {
        s.button_visibility[Control::Move as usize] = 1;
        s.analog_movement = true;
        s.sprint_with_joystick = sprint_forward;
        for row in &mut s.layouts {
            row[Control::Move as usize] = stick;
        }
        true
    } else {
        s.button_visibility[Control::Move as usize] = 2;
        false
    };
    s.mode = Mode::Crosshair;
    s.split_controls = true;
    s.crosshair_gestures = false;
    s.inventory_controls = true;
    s.offhand_button = false;
    s.shield_button = false;
    let count = buttons.len();
    s.custom_buttons = buttons;
    s.imported_from = Some(ImportedMeta {
        name: name.to_owned(),
        format: format.to_owned(),
        buttons: count,
        drawers: 0,
        has_joystick,
    });
    s.sanitize();
    s
}

fn build_pojav_classic() -> Settings {
    let bg = [16, 16, 22, 145];
    let stroke = [225, 230, 240, 105];
    let rad = Some(4.0);
    let top_y = 0.065;
    let top_h = 0.082;
    let buttons = vec![
        // Top utility strip
        btn("special:menu", "ESC", 0.055, top_y, top_h, 1.35, 0.72, false, false, false, true, true, bg, stroke, rad),
        btn("special:gui_toggle", "GUI", 0.145, top_y, top_h, 1.35, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.f3", "DEBUG", 0.245, top_y, top_h, 1.55, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.f5", "3RD", 0.345, top_y, top_h, 1.35, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.t", "CHAT", 0.438, top_y, top_h, 1.45, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.tab", "TAB", 0.532, top_y, top_h, 1.35, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("special:keyboard", "KB", 0.620, top_y, top_h, 1.25, 0.72, false, false, false, true, true, bg, stroke, rad),
        btn("special:virtual_mouse", "MOUSE", 0.935, top_y, top_h, 1.55, 0.78, false, false, false, true, true, bg, stroke, rad),
        // Bottom-left swipeable WASD + center Sneak + Sprint
        btn("key.keyboard.w", "▲", 0.145, 0.585, 0.125, 1.0, 0.72, false, true, false, true, false, bg, stroke, rad),
        btn("key.keyboard.a", "◀", 0.072, 0.725, 0.125, 1.0, 0.72, false, true, false, true, false, bg, stroke, rad),
        btn("key.keyboard.left.shift", "◇", 0.145, 0.725, 0.120, 1.0, 0.72, true, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.d", "▶", 0.218, 0.725, 0.125, 1.0, 0.72, false, true, false, true, false, bg, stroke, rad),
        btn("key.keyboard.s", "▼", 0.145, 0.865, 0.125, 1.0, 0.72, false, true, false, true, false, bg, stroke, rad),
        btn("key.keyboard.left.control", "CTRL", 0.062, 0.445, 0.088, 1.35, 0.70, true, false, false, true, false, bg, stroke, rad),
        // Right-hand Pojav combat & inventory cluster
        btn("special:mouse_pri", "PRI", 0.785, 0.635, 0.145, 1.15, 0.76, false, false, true, true, true, bg, stroke, rad),
        btn("special:mouse_sec", "SEC", 0.915, 0.525, 0.145, 1.15, 0.76, false, false, true, true, true, bg, stroke, rad),
        btn("key.keyboard.space", "JUMP", 0.908, 0.795, 0.150, 1.15, 0.76, false, false, true, true, false, bg, stroke, rad),
        btn("key.keyboard.e", "INV", 0.765, 0.885, 0.095, 1.35, 0.74, false, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.f", "OFF", 0.675, 0.765, 0.092, 1.15, 0.70, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.q", "DROP", 0.935, 0.335, 0.088, 1.30, 0.68, false, false, false, true, true, bg, stroke, rad),
    ];
    finalize_preset("Pojav Classic", "Volt Studio (Pojav)", None, buttons)
}

fn build_zalith_modern() -> Settings {
    let bg = [22, 18, 26, 152];
    let stroke = [225, 48, 82, 145];
    let accent_bg = [46, 18, 28, 168];
    let rad = Some(10.0);
    let stick = Placement {
        center: [0.155, 0.735],
        size: 0.31,
        opacity: 0.72,
    };
    let top_y = 0.065;
    let top_h = 0.078;
    let buttons = vec![
        btn("special:menu", "ESC", 0.055, top_y, top_h, 1.35, 0.74, false, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.f3", "F3", 0.135, top_y, top_h, 1.20, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.f5", "F5", 0.208, top_y, top_h, 1.20, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.t", "CHAT", 0.292, top_y, top_h, 1.45, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.tab", "TAB", 0.382, top_y, top_h, 1.30, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("special:gui_toggle", "HUD", 0.845, top_y, top_h, 1.30, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("special:virtual_mouse", "CURSOR", 0.935, top_y, top_h, 1.55, 0.78, false, false, false, true, true, accent_bg, stroke, rad),
        // Right ergonomic arc
        btn("special:mouse_pri", "ATK", 0.790, 0.625, 0.150, 1.05, 0.80, false, false, true, true, true, accent_bg, stroke, rad),
        btn("special:mouse_sec", "USE", 0.915, 0.505, 0.145, 1.05, 0.80, false, false, true, true, true, accent_bg, stroke, rad),
        btn("key.keyboard.space", "JUMP", 0.910, 0.785, 0.155, 1.05, 0.80, false, false, true, true, false, bg, stroke, rad),
        btn("key.keyboard.left.shift", "SNEAK", 0.785, 0.845, 0.112, 1.22, 0.76, true, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.e", "BAG", 0.680, 0.895, 0.092, 1.30, 0.75, false, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.f", "SWAP", 0.695, 0.715, 0.095, 1.18, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.q", "DROP", 0.930, 0.315, 0.086, 1.25, 0.70, false, false, false, true, true, bg, stroke, rad),
    ];
    finalize_preset("Zalith Layout", "Volt Studio (Zalith)", Some((stick, true)), buttons)
}

fn build_fold_craft() -> Settings {
    let bg = [20, 24, 30, 148];
    let stroke = [180, 195, 220, 120];
    let rad = Some(8.0);
    let stick = Placement {
        center: [0.150, 0.740],
        size: 0.30,
        opacity: 0.70,
    };
    let top_y = 0.065;
    let top_h = 0.078;
    let buttons = vec![
        btn("special:menu", "ESC", 0.055, top_y, top_h, 1.30, 0.72, false, false, false, true, true, bg, stroke, rad),
        btn("special:gui_toggle", "GUI", 0.135, top_y, top_h, 1.30, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.f3", "F3", 0.210, top_y, top_h, 1.18, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.f5", "F5", 0.278, top_y, top_h, 1.18, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.t", "CHAT", 0.358, top_y, top_h, 1.40, 0.72, false, false, false, true, false, bg, stroke, rad),
        btn("special:keyboard", "IME", 0.440, top_y, top_h, 1.25, 0.72, false, false, false, true, true, bg, stroke, rad),
        btn("special:virtual_mouse", "MOUSE", 0.935, top_y, top_h, 1.50, 0.76, false, false, false, true, true, bg, stroke, rad),
        // FCL hotbar wheel buttons + left sprint
        btn("key.keyboard.left.control", "RUN", 0.068, 0.455, 0.090, 1.25, 0.70, true, false, false, true, false, bg, stroke, rad),
        btn("special:scroll_up", "▲SL", 0.855, 0.315, 0.082, 1.20, 0.68, false, false, false, true, true, bg, stroke, rad),
        btn("special:scroll_down", "▼SL", 0.935, 0.315, 0.082, 1.20, 0.68, false, false, false, true, true, bg, stroke, rad),
        // FCL right combat & utility diamond
        btn("special:mouse_pri", "LMB", 0.785, 0.640, 0.145, 1.10, 0.76, false, false, true, true, true, bg, stroke, rad),
        btn("special:mouse_sec", "RMB", 0.915, 0.520, 0.145, 1.10, 0.76, false, false, true, true, true, bg, stroke, rad),
        btn("key.keyboard.space", "JUMP", 0.910, 0.795, 0.148, 1.10, 0.76, false, false, true, true, false, bg, stroke, rad),
        btn("key.keyboard.left.shift", "SNK", 0.785, 0.845, 0.108, 1.15, 0.74, true, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.e", "INV", 0.680, 0.895, 0.092, 1.30, 0.74, false, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.f", "OFF", 0.690, 0.720, 0.090, 1.15, 0.70, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.q", "Q", 0.775, 0.475, 0.086, 1.05, 0.68, false, false, false, true, true, bg, stroke, rad),
    ];
    finalize_preset("Fold Craft (FCL)", "Volt Studio (FCL)", Some((stick, true)), buttons)
}

fn build_pvp_claw() -> Settings {
    let bg = [20, 14, 20, 148];
    let trigger_bg = [62, 16, 28, 172];
    let stroke = [245, 58, 92, 160];
    let rad = Some(12.0);
    let stick = Placement {
        center: [0.165, 0.755],
        size: 0.32,
        opacity: 0.72,
    };
    let buttons = vec![
        // Left index trigger (top-left) — Attack / Left-click with aim passthrough
        btn("special:mouse_pri", "ATTACK", 0.115, 0.205, 0.185, 1.45, 0.82, false, false, true, true, true, trigger_bg, stroke, rad),
        btn("key.keyboard.left.control", "SPRINT", 0.265, 0.185, 0.098, 1.30, 0.75, true, false, false, true, false, bg, stroke, rad),
        // Right index trigger (top-right) — Use / Place / Block / Pearl with aim passthrough
        btn("special:mouse_sec", "USE / PLACE", 0.885, 0.205, 0.185, 1.45, 0.82, false, false, true, true, true, trigger_bg, stroke, rad),
        btn("key.keyboard.f", "OFFHAND", 0.730, 0.185, 0.098, 1.35, 0.75, false, false, false, true, false, bg, stroke, rad),
        // Top-center compact utility strip
        btn("special:menu", "ESC", 0.405, 0.062, 0.072, 1.25, 0.70, false, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.f5", "F5", 0.478, 0.062, 0.072, 1.15, 0.70, false, false, false, true, false, bg, stroke, rad),
        btn("key.keyboard.t", "CHAT", 0.552, 0.062, 0.072, 1.30, 0.70, false, false, false, true, false, bg, stroke, rad),
        btn("special:virtual_mouse", "MOUSE", 0.635, 0.062, 0.072, 1.40, 0.72, false, false, false, true, true, bg, stroke, rad),
        // Right thumb (bottom-right) — Jump, Crouch, Inventory, Drop, Zoom
        btn("key.keyboard.space", "JUMP", 0.885, 0.735, 0.175, 1.15, 0.80, false, false, true, true, false, bg, stroke, rad),
        btn("key.keyboard.left.shift", "CROUCH", 0.755, 0.835, 0.125, 1.22, 0.76, true, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.e", "INV", 0.755, 0.645, 0.105, 1.20, 0.74, false, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.q", "DROP", 0.915, 0.485, 0.090, 1.20, 0.70, false, false, false, true, true, bg, stroke, rad),
        btn("key.keyboard.z", "ZOOM", 0.650, 0.765, 0.090, 1.20, 0.70, false, false, false, true, false, bg, stroke, rad),
    ];
    finalize_preset("4-Finger PvP Claw", "Volt Studio (Claw)", Some((stick, true)), buttons)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_preset_builds_valid_round_tripping_settings() {
        for preset in BuiltInPreset::ALL {
            let s = preset.build();
            assert_eq!(BuiltInPreset::detect(&s), Some(preset));
            for b in &s.custom_buttons {
                assert!(
                    CustomButton::valid_token(&b.key),
                    "invalid token {} in {preset:?}",
                    b.key
                );
            }
            let json = serde_json::to_vec(&s).expect("serialize preset");
            let mut decoded: Settings = serde_json::from_slice(&json).expect("deserialize preset");
            decoded.sanitize();
            assert_eq!(decoded.custom_buttons.len(), s.custom_buttons.len());
            assert_eq!(BuiltInPreset::detect(&decoded), Some(preset));
        }
    }

    #[test]
    fn every_studio_binding_is_a_valid_custom_button_token() {
        for &(token, label) in STUDIO_BINDINGS {
            assert!(!label.is_empty());
            assert!(
                CustomButton::valid_token(token),
                "studio binding {token} rejected by CustomButton::valid_token"
            );
        }
    }
}
