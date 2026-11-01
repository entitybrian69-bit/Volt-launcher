//! Import touch control layouts from Java Android Minecraft launchers:
//! - PojavLauncher / Amethyst / MojoLauncher / Zalith v1 (`mControlDataList`, `mDrawerDataList`, `mJoystickDataList`)
//! - Fold Craft Launcher / FCL (`viewGroups`, `buttonList`, `directionList`)
//! - Zalith Launcher 2 (`layers`, `normalButtons`, `directionButtons`)
//! - Volt / Horizon native touch JSON (`horizon-touch.json`)
//! - `.zip` control packs wrapping any supported JSON layout

use super::{
    CustomButton,
    layout::{Control, ImportedMeta, MAX_CUSTOM_BUTTONS, Mode, Placement, Settings},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

const REF_WIDTH_DP: f32 = 844.0;
const REF_HEIGHT_DP: f32 = 390.0;
const MAX_CONTROL_JSON_BYTES: usize = 4 * 1024 * 1024;
const MAX_PRESETS: usize = 24;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetEntry {
    pub slug: String,
    pub meta: ImportedMeta,
}

#[derive(Clone, Debug)]
pub struct ImportedControlLayout {
    pub settings: Settings,
    pub meta: ImportedMeta,
}

pub fn import_file(path: &Path, display_name: &str) -> Result<ImportedControlLayout, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("Could not read control file: {e}"))?;
    if meta.len() > 16 * 1024 * 1024 {
        return Err("Control package exceeds the 16 MiB limit".into());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("Could not read control file: {e}"))?;
    import_bytes(&bytes, display_name)
}

pub fn import_bytes(bytes: &[u8], display_name: &str) -> Result<ImportedControlLayout, String> {
    if bytes.is_empty() {
        return Err("Selected control file is empty".into());
    }
    if bytes.starts_with(b"PK\x03\x04") {
        return import_zip(bytes, display_name);
    }
    if bytes.len() > MAX_CONTROL_JSON_BYTES {
        return Err("Control JSON exceeds the 4 MiB limit".into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "Control file is not valid UTF-8 JSON or ZIP archive".to_owned())?;
    let trimmed = text.trim_start_matches('\u{feff}').trim();
    let value: Value =
        serde_json::from_str(trimmed).map_err(|e| format!("Invalid control JSON: {e}"))?;
    import_json_value(&value, clean_layout_title(display_name))
}

fn import_zip(bytes: &[u8], display_name: &str) -> Result<ImportedControlLayout, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| format!("Invalid control ZIP archive: {e}"))?;
    if archive.len() > 256 {
        return Err("Control ZIP archive contains too many entries".into());
    }
    let mut manifest_title: Option<String> = None;
    let mut candidates: Vec<(u8, String, Value)> = Vec::new();

    for idx in 0..archive.len() {
        let mut entry = archive
            .by_index(idx)
            .map_err(|e| format!("Corrupt ZIP entry: {e}"))?;
        if entry.is_dir() || entry.size() > MAX_CONTROL_JSON_BYTES as u64 {
            continue;
        }
        let name = entry.name().to_owned();
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".json") && !lower.ends_with(".control") && !lower.ends_with(".fcl") {
            continue;
        }
        let mut buf = Vec::with_capacity(entry.size().min(65536) as usize);
        entry
            .by_ref()
            .take(MAX_CONTROL_JSON_BYTES as u64 + 1)
            .read_to_end(&mut buf)
            .map_err(|e| format!("Could not extract {name}: {e}"))?;
        if buf.len() > MAX_CONTROL_JSON_BYTES {
            continue;
        }
        let Ok(text) = std::str::from_utf8(&buf) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}').trim())
        else {
            continue;
        };
        let file_stem = Path::new(&name)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        if matches!(
            file_stem.to_ascii_lowercase().as_str(),
            "manifest.json" | "info.json" | "pack.json"
        ) {
            if let Some(title) = json
                .get("name")
                .or_else(|| json.get("title"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                manifest_title = Some(title.to_owned());
            }
        }
        let score = layout_score(&json);
        if score > 0 {
            candidates.push((score, file_stem.into_owned(), json));
        }
    }

    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    let Some((_, entry_name, best)) = candidates.into_iter().next() else {
        return Err(
            "No supported PojavLauncher, Zalith, FCL or Volt control JSON found in ZIP".into(),
        );
    };
    let title = manifest_title.unwrap_or_else(|| {
        let outer = clean_layout_title(display_name);
        if outer.is_empty() || outer.eq_ignore_ascii_case("import") {
            clean_layout_title(&entry_name)
        } else {
            outer
        }
    });
    import_json_value(&best, title)
}

fn layout_score(v: &Value) -> u8 {
    let Some(obj) = v.as_object() else {
        return 0;
    };
    if obj.contains_key("mControlDataList")
        || obj.contains_key("mDrawerDataList")
        || obj.contains_key("mJoystickDataList")
    {
        return 100;
    }
    if obj.contains_key("viewGroups")
        || obj.contains_key("buttonList")
        || obj.contains_key("directionList")
    {
        return 95;
    }
    if obj.contains_key("layers")
        || obj.contains_key("controlLayers")
        || obj.contains_key("normalButtons")
        || obj.contains_key("directionButtons")
    {
        return 90;
    }
    if obj.contains_key("layouts")
        && (obj.contains_key("custom_buttons") || obj.contains_key("button_visibility"))
    {
        return 85;
    }
    0
}

fn clean_layout_title(raw: &str) -> String {
    let file = Path::new(raw)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| raw.to_owned());
    let mut stem = file.as_str();
    for ext in [".json", ".zip", ".control", ".fcl", ".txt"] {
        if stem.len() > ext.len() && stem[stem.len() - ext.len()..].eq_ignore_ascii_case(ext) {
            stem = &stem[..stem.len() - ext.len()];
            break;
        }
    }
    let cleaned = stem.trim();
    if cleaned.is_empty() {
        "Imported Controls".into()
    } else {
        truncate_chars(cleaned, 44)
    }
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

fn import_json_value(v: &Value, title: String) -> Result<ImportedControlLayout, String> {
    let Some(obj) = v.as_object() else {
        return Err("Control layout root must be a JSON object".into());
    };
    if obj.contains_key("mControlDataList")
        || obj.contains_key("mDrawerDataList")
        || obj.contains_key("mJoystickDataList")
    {
        return import_pojav(v, title);
    }
    if obj.contains_key("viewGroups")
        || obj.contains_key("buttonList")
        || obj.contains_key("directionList")
    {
        return import_fcl(v, title);
    }
    if obj.contains_key("layers")
        || obj.contains_key("controlLayers")
        || obj.contains_key("normalButtons")
        || obj.contains_key("directionButtons")
    {
        return import_zalith(v, title);
    }
    if obj.contains_key("layouts")
        && (obj.contains_key("custom_buttons") || obj.contains_key("button_visibility"))
    {
        return import_native(v, title);
    }
    Err(
        "Unrecognized control format. Expected PojavLauncher, Amethyst, Mojo, Zalith, FCL or Volt JSON."
            .into(),
    )
}

fn import_native(v: &Value, title: String) -> Result<ImportedControlLayout, String> {
    let mut settings: Settings =
        serde_json::from_value(v.clone()).map_err(|e| format!("Invalid Volt touch JSON: {e}"))?;
    let drawers = settings
        .custom_buttons
        .iter()
        .filter(|b| b.key.starts_with("special:drawer:"))
        .count();
    let has_joystick = settings.button_visibility[Control::Move as usize] != 2;
    let meta = settings.imported_from.clone().unwrap_or(ImportedMeta {
        name: title,
        format: "Volt / Horizon".into(),
        buttons: settings.custom_buttons.len(),
        drawers,
        has_joystick,
    });
    settings.imported_from = Some(meta.clone());
    settings.sanitize();
    Ok(ImportedControlLayout { settings, meta })
}

// ---------------------------------------------------------------------------
// 1. PojavLauncher / Amethyst / MojoLauncher / Zalith v1 (`ControlLayout`)
// ---------------------------------------------------------------------------

fn import_pojav(v: &Value, title: String) -> Result<ImportedControlLayout, String> {
    let version = v.get("version").and_then(Value::as_i64).unwrap_or(5);
    let scaled_at = v
        .get("scaledAt")
        .and_then(Value::as_f64)
        .map(|s| s as f32)
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(100.0);

    let (ref_w, ref_h, dp_px, px_scale) = if version <= 2 {
        (1920.0f32, 1080.0f32, 2.75f32, 1.0f32)
    } else {
        (REF_WIDTH_DP, REF_HEIGHT_DP, 1.0f32, 1.0f32 / 2.75f32)
    };
    let legacy_lwjgl2 = version == 1;

    let mut custom_buttons: Vec<CustomButton> = Vec::new();
    let mut drawer_count = 0usize;

    if let Some(list) = v.get("mControlDataList").and_then(Value::as_array) {
        for item in list {
            if custom_buttons.len() >= MAX_CUSTOM_BUTTONS {
                break;
            }
            if let Some(btn) = parse_pojav_button(
                item,
                ref_w,
                ref_h,
                dp_px,
                px_scale,
                scaled_at,
                legacy_lwjgl2,
                None,
                None,
            ) {
                custom_buttons.push(btn);
            }
        }
    }

    if let Some(drawers) = v.get("mDrawerDataList").and_then(Value::as_array) {
        for (idx, drawer) in drawers.iter().enumerate() {
            if custom_buttons.len() >= MAX_CUSTOM_BUTTONS {
                break;
            }
            let drawer_id = (idx + 1).min(u16::MAX as usize) as u16;
            let props = drawer.get("properties").unwrap_or(drawer);
            let (parent_x, parent_y, parent_w, parent_h) =
                pojav_box(props, ref_w, ref_h, dp_px, px_scale, scaled_at);
            if let Some(mut parent_btn) = parse_pojav_button(
                props,
                ref_w,
                ref_h,
                dp_px,
                px_scale,
                scaled_at,
                legacy_lwjgl2,
                None,
                Some(format!("special:drawer:{drawer_id}")),
            ) {
                if parent_btn.label.as_deref().unwrap_or_default().is_empty() {
                    parent_btn.label = Some("≡".into());
                }
                custom_buttons.push(parent_btn);
                drawer_count += 1;
            }

            let orientation = drawer_orientation(drawer.get("orientation"));
            if let Some(children) = drawer.get("buttonProperties").and_then(Value::as_array) {
                for (child_idx, child) in children.iter().enumerate() {
                    if custom_buttons.len() >= MAX_CUSTOM_BUTTONS {
                        break;
                    }
                    let override_pos = match orientation {
                        DrawerDir::Free => None,
                        dir => {
                            let step = (child_idx + 1) as f32;
                            let margin = 2.0 * dp_px;
                            let (dx, dy) = match dir {
                                DrawerDir::Down => (0.0, (parent_h + margin) * step),
                                DrawerDir::Up => (0.0, -(parent_h + margin) * step),
                                DrawerDir::Right => ((parent_w + margin) * step, 0.0),
                                DrawerDir::Left => (-(parent_w + margin) * step, 0.0),
                                DrawerDir::Free => (0.0, 0.0),
                            };
                            Some((parent_x + dx, parent_y + dy, parent_w, parent_h))
                        }
                    };
                    if let Some(mut child_btn) = parse_pojav_button(
                        child,
                        ref_w,
                        ref_h,
                        dp_px,
                        px_scale,
                        scaled_at,
                        legacy_lwjgl2,
                        override_pos,
                        None,
                    ) {
                        child_btn.drawer_parent = Some(drawer_id);
                        custom_buttons.push(child_btn);
                    }
                }
            }
        }
    }

    let mut joystick: Option<(Placement, bool, bool)> = None;
    if let Some(sticks) = v.get("mJoystickDataList").and_then(Value::as_array)
        && let Some(stick) = sticks.first()
    {
        let (jx, jy, jw, jh) = pojav_box(stick, ref_w, ref_h, dp_px, px_scale, scaled_at);
        let size = (jh.max(jw) / ref_h).clamp(0.12, 0.45);
        let cx = ((jx + jw * 0.5) / ref_w).clamp(0.05, 0.95);
        let cy = ((jy + jh * 0.5) / ref_h).clamp(0.05, 0.95);
        let opacity = normalize_opacity(stick.get("opacity").and_then(Value::as_f64));
        let forward_lock = stick
            .get("forwardLock")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let absolute = stick
            .get("absolute")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        joystick = Some((
            Placement {
                center: [cx, cy],
                size,
                opacity,
            },
            forward_lock,
            absolute,
        ));
    }

    if custom_buttons.is_empty() && joystick.is_none() {
        return Err("Pojav control file contains no buttons or joysticks".into());
    }

    let layout_name = v
        .get("name")
        .or_else(|| v.get("mName"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| truncate_chars(s, 44))
        .unwrap_or(title);

    let format_name = if v.get("amethystVersion").is_some() {
        "Amethyst"
    } else if v.get("mojoVersion").is_some() {
        "MojoLauncher"
    } else {
        "PojavLauncher"
    };

    Ok(assemble_imported_settings(
        layout_name,
        format_name.into(),
        custom_buttons,
        drawer_count,
        joystick,
    ))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DrawerDir {
    Down,
    Left,
    Up,
    Right,
    Free,
}

fn drawer_orientation(v: Option<&Value>) -> DrawerDir {
    if let Some(s) = v.and_then(Value::as_str) {
        return match s.trim().to_ascii_uppercase().as_str() {
            "DOWN" => DrawerDir::Down,
            "LEFT" => DrawerDir::Left,
            "UP" => DrawerDir::Up,
            "RIGHT" => DrawerDir::Right,
            _ => DrawerDir::Free,
        };
    }
    if let Some(n) = v.and_then(Value::as_i64) {
        return match n {
            0 => DrawerDir::Down,
            1 => DrawerDir::Left,
            2 => DrawerDir::Up,
            3 => DrawerDir::Right,
            _ => DrawerDir::Free,
        };
    }
    DrawerDir::Free
}

fn pojav_box(
    obj: &Value,
    ref_w: f32,
    ref_h: f32,
    dp_px: f32,
    px_scale: f32,
    scaled_at: f32,
) -> (f32, f32, f32, f32) {
    let raw_w = json_f32(obj.get("width")).unwrap_or(50.0).max(8.0) * dp_px;
    let raw_h = json_f32(obj.get("height")).unwrap_or(50.0).max(8.0) * dp_px;
    let vars = ExprVars {
        screen_w: ref_w,
        screen_h: ref_h,
        btn_w: raw_w,
        btn_h: raw_h,
        dp_px,
        px_scale,
        preferred_scale: scaled_at,
    };
    let x = obj
        .get("dynamicX")
        .and_then(Value::as_str)
        .and_then(|s| eval_dynamic_expr(s, &vars, ref_w))
        .or_else(|| json_f32(obj.get("x")).map(|x| if x <= 1.0 && x >= 0.0 && raw_w < ref_w * 0.5 && obj.get("dynamicX").is_none() && x.fract() != 0.0 { x * ref_w } else { x }))
        .unwrap_or(ref_w * 0.5 - raw_w * 0.5);
    let y = obj
        .get("dynamicY")
        .and_then(Value::as_str)
        .and_then(|s| eval_dynamic_expr(s, &vars, ref_h))
        .or_else(|| json_f32(obj.get("y")).map(|y| if y <= 1.0 && y >= 0.0 && raw_h < ref_h * 0.5 && obj.get("dynamicY").is_none() && y.fract() != 0.0 { y * ref_h } else { y }))
        .unwrap_or(ref_h * 0.5 - raw_h * 0.5);
    (x, y, raw_w, raw_h)
}

#[allow(clippy::too_many_arguments)]
fn parse_pojav_button(
    obj: &Value,
    ref_w: f32,
    ref_h: f32,
    dp_px: f32,
    px_scale: f32,
    scaled_at: f32,
    legacy_lwjgl2: bool,
    override_box: Option<(f32, f32, f32, f32)>,
    override_token: Option<String>,
) -> Option<CustomButton> {
    let mut tokens: Vec<String> = Vec::new();
    if let Some(tok) = override_token {
        tokens.push(tok);
    } else {
        if let Some(arr) = obj.get("keycodes").and_then(Value::as_array) {
            for entry in arr {
                if let Some(code) = entry.as_i64()
                    && code != 0
                    && let Some(mapped) = map_java_keycode(code as i32, legacy_lwjgl2)
                    && !tokens.iter().any(|t| t == &mapped)
                {
                    tokens.push(mapped);
                }
            }
        }
        if tokens.is_empty()
            && let Some(single) = obj.get("keycode").and_then(Value::as_i64)
            && single != 0
            && let Some(mapped) = map_java_keycode(single as i32, legacy_lwjgl2)
        {
            tokens.push(mapped);
        }
    }
    if tokens.is_empty() {
        return None;
    }

    let (x, y, bw, bh) =
        override_box.unwrap_or_else(|| pojav_box(obj, ref_w, ref_h, dp_px, px_scale, scaled_at));
    let norm_h = (bh / ref_h).clamp(0.025, 0.55);
    let width_ratio = (bw / bh.max(1.0)).clamp(0.2, 8.0);
    let cx = ((x + bw * 0.5) / ref_w).clamp(0.01, 0.99);
    let cy = ((y + bh * 0.5) / ref_h).clamp(0.01, 0.99);
    let opacity = normalize_opacity(obj.get("opacity").and_then(Value::as_f64));

    let raw_name = obj
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let label = if raw_name.is_empty() {
        None
    } else {
        Some(truncate_chars(raw_name, 32))
    };

    let toggle = obj
        .get("isToggle")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let swipeable = obj
        .get("isSwipeable")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let passthrough = obj
        .get("passThruEnabled")
        .or_else(|| obj.get("passthroughEnabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let in_game = obj
        .get("displayInGame")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let in_menu = obj
        .get("displayInMenu")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let bg_color = parse_argb_color(obj.get("bgColor")).or(Some([0, 0, 0, 115]));
    let stroke_width = json_f32(obj.get("strokeWidth")).unwrap_or(0.0);
    let stroke_color = if stroke_width > 0.0 {
        parse_argb_color(obj.get("strokeColor"))
    } else {
        parse_argb_color(obj.get("strokeColor")).filter(|c| c[3] > 0)
    };

    let primary = tokens.remove(0);
    let placement = Placement {
        center: [cx, cy],
        size: norm_h,
        opacity,
    };
    Some(CustomButton {
        key: primary,
        layouts: [placement; 3],
        extra_keys: tokens,
        label,
        width_ratio,
        toggle,
        swipeable,
        passthrough,
        in_game,
        in_menu,
        bg_color,
        stroke_color,
        drawer_parent: None,
    })
}

// ---------------------------------------------------------------------------
// 2. Fold Craft Launcher / FCL (`viewGroups` / `buttonList` / `directionList`)
// ---------------------------------------------------------------------------

fn import_fcl(v: &Value, title: String) -> Result<ImportedControlLayout, String> {
    let mut custom_buttons: Vec<CustomButton> = Vec::new();
    let mut joystick: Option<(Placement, bool, bool)> = None;
    let mut drawer_count = 0usize;

    let mut groups: Vec<(Option<u16>, &Value, bool)> = Vec::new();
    if let Some(arr) = v.get("viewGroups").and_then(Value::as_array) {
        let mut group_id_map: std::collections::BTreeMap<String, u16> =
            std::collections::BTreeMap::new();
        for (idx, g) in arr.iter().enumerate() {
            if let Some(id) = g.get("id").and_then(Value::as_str) {
                group_id_map.insert(id.to_owned(), (idx + 1) as u16);
            }
        }
        for (idx, g) in arr.iter().enumerate() {
            let visible_by_default = g
                .get("visibility")
                .and_then(Value::as_str)
                .is_none_or(|vis| !vis.eq_ignore_ascii_case("INVISIBLE"));
            let gid = (idx + 1) as u16;
            let data = g.get("viewData").unwrap_or(g);
            groups.push(((!visible_by_default).then_some(gid), data, visible_by_default));
        }
        // Parse buttons with group bindings
        for (drawer_parent, data, visible_by_default) in &groups {
            if !visible_by_default {
                drawer_count += 1;
            }
            parse_fcl_view_data(
                data,
                *drawer_parent,
                &group_id_map,
                &mut custom_buttons,
                &mut joystick,
            );
        }
    } else {
        let empty = std::collections::BTreeMap::new();
        parse_fcl_view_data(v, None, &empty, &mut custom_buttons, &mut joystick);
    }

    if custom_buttons.is_empty() && joystick.is_none() {
        return Err("FCL control file contains no buttons or direction pads".into());
    }

    let layout_name = v
        .get("manifest")
        .and_then(|m| m.get("name"))
        .or_else(|| v.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| truncate_chars(s, 44))
        .unwrap_or(title);

    Ok(assemble_imported_settings(
        layout_name,
        "Fold Craft (FCL)".into(),
        custom_buttons,
        drawer_count,
        joystick,
    ))
}

fn parse_fcl_view_data(
    data: &Value,
    drawer_parent: Option<u16>,
    group_id_map: &std::collections::BTreeMap<String, u16>,
    custom_buttons: &mut Vec<CustomButton>,
    joystick: &mut Option<(Placement, bool, bool)>,
) {
    if let Some(buttons) = data.get("buttonList").and_then(Value::as_array) {
        for btn in buttons {
            if custom_buttons.len() >= MAX_CUSTOM_BUTTONS {
                break;
            }
            if let Some(mut parsed) = parse_fcl_button(btn, group_id_map) {
                parsed.drawer_parent = drawer_parent;
                custom_buttons.push(parsed);
            }
        }
    }
    if joystick.is_none()
        && let Some(dirs) = data.get("directionList").and_then(Value::as_array)
        && let Some(dir) = dirs.first()
    {
        let base = dir.get("baseInfo").unwrap_or(dir);
        let (cx, cy, _, norm_h, opacity, _, _) = parse_fcl_base_info(base, 0.28);
        let style_dpad = dir
            .get("styleType")
            .and_then(Value::as_str)
            .is_some_and(|s| s.to_ascii_uppercase().contains("BUTTON"));
        let sneak_or_sprint = dir
            .get("event")
            .and_then(|e| e.get("sneak"))
            .and_then(Value::as_bool)
            .unwrap_or(true);
        *joystick = Some((
            Placement {
                center: [cx, cy],
                size: norm_h.clamp(0.14, 0.45),
                opacity,
            },
            sneak_or_sprint,
            style_dpad,
        ));
    }
}

fn parse_fcl_base_info(base: &Value, fallback_h: f32) -> (f32, f32, f32, f32, f32, bool, bool) {
    let size_type = base
        .get("sizeType")
        .and_then(Value::as_str)
        .unwrap_or("ABSOLUTE");
    let raw_w = json_f32(base.get("width")).unwrap_or(50.0).max(1.0);
    let raw_h = json_f32(base.get("height")).unwrap_or(50.0).max(1.0);

    let (norm_w, norm_h) = if size_type.eq_ignore_ascii_case("PERCENTAGE") {
        let w = if raw_w > 1.0 { raw_w / 1000.0 } else { raw_w };
        let h = if raw_h > 1.0 { raw_h / 1000.0 } else { raw_h };
        (w.clamp(0.02, 0.8), h.clamp(0.025, 0.55))
    } else {
        // FCL stores ABSOLUTE sizes either as dp*10 (e.g. 500 = 50.0dp) or plain dp (50)
        let w_dp = if raw_w > 160.0 { raw_w / 10.0 } else { raw_w };
        let h_dp = if raw_h > 160.0 { raw_h / 10.0 } else { raw_h };
        (
            (w_dp / REF_WIDTH_DP).clamp(0.02, 0.8),
            (h_dp / REF_HEIGHT_DP).clamp(0.025, 0.55),
        )
    };
    let norm_h = if norm_h.is_finite() {
        norm_h
    } else {
        fallback_h
    };
    let rx = json_f32(base.get("xPosition").or_else(|| base.get("x"))).unwrap_or(500.0);
    let ry = json_f32(base.get("yPosition").or_else(|| base.get("y"))).unwrap_or(500.0);
    let tx = if rx > 1.0 { rx / 1000.0 } else { rx }.clamp(0.0, 1.0);
    let ty = if ry > 1.0 { ry / 1000.0 } else { ry }.clamp(0.0, 1.0);
    let left = (1.0 - norm_w) * tx;
    let top = (1.0 - norm_h) * ty;
    let cx = (left + norm_w * 0.5).clamp(0.01, 0.99);
    let cy = (top + norm_h * 0.5).clamp(0.01, 0.99);

    let alpha = json_f32(base.get("alpha").or_else(|| base.get("opacity")))
        .map(|a| if a > 1.0 { a / 100.0 } else { a })
        .unwrap_or(0.75)
        .clamp(0.05, 1.0);

    let (in_game, in_menu) = parse_visibility_mode(base.get("visibilityType"));
    let width_ratio = ((norm_w * REF_WIDTH_DP) / (norm_h * REF_HEIGHT_DP).max(1.0)).clamp(0.2, 8.0);
    (cx, cy, width_ratio, norm_h, alpha, in_game, in_menu)
}

fn parse_fcl_button(
    btn: &Value,
    group_id_map: &std::collections::BTreeMap<String, u16>,
) -> Option<CustomButton> {
    let base = btn.get("baseInfo").unwrap_or(btn);
    let (cx, cy, width_ratio, norm_h, opacity, in_game, in_menu) = parse_fcl_base_info(base, 0.12);
    let event = btn.get("event").unwrap_or(btn);
    let press = event.get("pressEvent").unwrap_or(event);

    let mut tokens: Vec<String> = Vec::new();
    if let Some(groups) = press
        .get("bindViewGroup")
        .or_else(|| event.get("bindViewGroup"))
        .and_then(Value::as_array)
    {
        for g in groups {
            if let Some(id_str) = g.as_str()
                && let Some(&gid) = group_id_map.get(id_str)
            {
                tokens.push(format!("special:drawer:{gid}"));
            }
        }
    }
    if let Some(codes) = press
        .get("outputKeycodes")
        .or_else(|| event.get("outputKeycodes"))
        .or_else(|| btn.get("keycodes"))
        .and_then(Value::as_array)
    {
        for entry in codes {
            if let Some(tok) = parse_fcl_or_zalith_key(entry)
                && !tokens.iter().any(|t| t == &tok)
            {
                tokens.push(tok);
            }
        }
    }
    if tokens.is_empty() {
        return None;
    }

    let toggle = press
        .get("autoKeep")
        .or_else(|| event.get("autoKeep"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let passthrough = press
        .get("movable")
        .or_else(|| press.get("pointerFollow"))
        .or_else(|| event.get("pointerFollow"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let label = btn
        .get("text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| truncate_chars(s, 32));

    let style = btn.get("style");
    let bg_color = style
        .and_then(|s| parse_argb_color(s.get("fillColor").or_else(|| s.get("bgColor"))))
        .or(Some([0, 0, 0, 115]));
    let stroke_color = style.and_then(|s| parse_argb_color(s.get("strokeColor")));

    let primary = tokens.remove(0);
    let placement = Placement {
        center: [cx, cy],
        size: norm_h,
        opacity,
    };
    Some(CustomButton {
        key: primary,
        layouts: [placement; 3],
        extra_keys: tokens,
        label,
        width_ratio,
        toggle,
        swipeable: false,
        passthrough,
        in_game,
        in_menu,
        bg_color,
        stroke_color,
        drawer_parent: None,
    })
}

// ---------------------------------------------------------------------------
// 3. Zalith Launcher 2 (`layers` / `normalButtons` / `directionButtons`)
// ---------------------------------------------------------------------------

fn import_zalith(v: &Value, title: String) -> Result<ImportedControlLayout, String> {
    let mut custom_buttons: Vec<CustomButton> = Vec::new();
    let mut joystick: Option<(Placement, bool, bool)> = None;
    let mut drawer_count = 0usize;

    let layers = v
        .get("layers")
        .or_else(|| v.get("controlLayers"))
        .and_then(Value::as_array);

    if let Some(layers) = layers {
        for (idx, layer) in layers.iter().enumerate() {
            let hidden = layer
                .get("hide")
                .or_else(|| layer.get("defaultHide"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let drawer_parent = hidden.then_some((idx + 1) as u16);
            if hidden {
                drawer_count += 1;
            }
            parse_zalith_layer(layer, drawer_parent, &mut custom_buttons, &mut joystick);
        }
    } else {
        parse_zalith_layer(v, None, &mut custom_buttons, &mut joystick);
    }

    if custom_buttons.is_empty() && joystick.is_none() {
        return Err("Zalith control file contains no active buttons or joysticks".into());
    }

    let layout_name = v
        .get("info")
        .and_then(|i| i.get("name"))
        .or_else(|| v.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| truncate_chars(s, 44))
        .unwrap_or(title);

    Ok(assemble_imported_settings(
        layout_name,
        "Zalith Launcher".into(),
        custom_buttons,
        drawer_count,
        joystick,
    ))
}

fn parse_zalith_layer(
    layer: &Value,
    drawer_parent: Option<u16>,
    custom_buttons: &mut Vec<CustomButton>,
    joystick: &mut Option<(Placement, bool, bool)>,
) {
    if let Some(buttons) = layer
        .get("normalButtons")
        .or_else(|| layer.get("buttons"))
        .and_then(Value::as_array)
    {
        for btn in buttons {
            if custom_buttons.len() >= MAX_CUSTOM_BUTTONS {
                break;
            }
            if let Some(mut parsed) = parse_zalith_button(btn) {
                parsed.drawer_parent = drawer_parent;
                custom_buttons.push(parsed);
            }
        }
    }
    if joystick.is_none()
        && let Some(sticks) = layer
            .get("directionButtons")
            .or_else(|| layer.get("joysticks"))
            .and_then(Value::as_array)
        && let Some(stick) = sticks.first()
    {
        let (cx, cy, _, norm_h) = parse_zalith_rect(stick, 0.28);
        let opacity = normalize_opacity(stick.get("alpha").or_else(|| stick.get("opacity")).and_then(Value::as_f64));
        *joystick = Some((
            Placement {
                center: [cx, cy],
                size: norm_h.clamp(0.14, 0.45),
                opacity,
            },
            true,
            false,
        ));
    }
}

fn parse_zalith_rect(obj: &Value, fallback_h: f32) -> (f32, f32, f32, f32) {
    let normalize_coord = |raw: f32, dp_axis: f32| -> f32 {
        if raw > 1000.0 {
            raw / 10000.0
        } else if raw > 100.0 {
            raw / 1000.0
        } else if raw > 1.0 {
            // Could be percentage 0..100 or dp
            (raw / 100.0).min(raw / dp_axis)
        } else {
            raw
        }
    };
    let rw = json_f32(obj.get("width")).unwrap_or(50.0);
    let rh = json_f32(obj.get("height")).unwrap_or(50.0);
    let norm_w = if obj.get("isPercentageSize").and_then(Value::as_bool) == Some(false)
        || (rw <= 250.0 && rh <= 250.0 && rw > 1.0)
    {
        (rw / REF_WIDTH_DP).clamp(0.02, 0.8)
    } else {
        normalize_coord(rw, REF_WIDTH_DP).clamp(0.02, 0.8)
    };
    let norm_h = if obj.get("isPercentageSize").and_then(Value::as_bool) == Some(false)
        || (rw <= 250.0 && rh <= 250.0 && rh > 1.0)
    {
        (rh / REF_HEIGHT_DP).clamp(0.025, 0.55)
    } else {
        normalize_coord(rh, REF_HEIGHT_DP).clamp(0.025, 0.55)
    };
    let norm_h = if norm_h.is_finite() { norm_h } else { fallback_h };
    let rx = json_f32(obj.get("x")).unwrap_or(0.5);
    let ry = json_f32(obj.get("y")).unwrap_or(0.5);
    let tx = (if rx > 1000.0 {
        rx / 10000.0
    } else if rx > 1.0 {
        rx / 100.0
    } else {
        rx
    })
    .clamp(0.0, 1.0);
    let ty = (if ry > 1000.0 {
        ry / 10000.0
    } else if ry > 1.0 {
        ry / 100.0
    } else {
        ry
    })
    .clamp(0.0, 1.0);

    let left = (1.0 - norm_w) * tx;
    let top = (1.0 - norm_h) * ty;
    let cx = (left + norm_w * 0.5).clamp(0.01, 0.99);
    let cy = (top + norm_h * 0.5).clamp(0.01, 0.99);
    let width_ratio = ((norm_w * REF_WIDTH_DP) / (norm_h * REF_HEIGHT_DP).max(1.0)).clamp(0.2, 8.0);
    (cx, cy, width_ratio, norm_h)
}

fn parse_zalith_button(btn: &Value) -> Option<CustomButton> {
    let (cx, cy, width_ratio, norm_h) = parse_zalith_rect(btn, 0.12);
    let mut tokens: Vec<String> = Vec::new();
    if let Some(events) = btn.get("clickEvents").and_then(Value::as_array) {
        for ev in events {
            let key_val = ev.get("key").or_else(|| ev.get("keycode")).unwrap_or(ev);
            if let Some(tok) = parse_fcl_or_zalith_key(key_val)
                && !tokens.iter().any(|t| t == &tok)
            {
                tokens.push(tok);
            }
        }
    }
    if let Some(keys) = btn.get("keycodes").and_then(Value::as_array) {
        for k in keys {
            if let Some(tok) = parse_fcl_or_zalith_key(k)
                && !tokens.iter().any(|t| t == &tok)
            {
                tokens.push(tok);
            }
        }
    }
    if tokens.is_empty() {
        return None;
    }

    let opacity = normalize_opacity(
        btn.get("alpha")
            .or_else(|| btn.get("opacity"))
            .and_then(Value::as_f64),
    );
    let (in_game, in_menu) = parse_visibility_mode(btn.get("visibilityType"));
    let toggle = btn
        .get("isToggle")
        .or_else(|| btn.get("toggle"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let swipeable = btn
        .get("isSwipeable")
        .or_else(|| btn.get("swipeable"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let passthrough = btn
        .get("isPointerFollow")
        .or_else(|| btn.get("passThruEnabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let label = btn
        .get("text")
        .or_else(|| btn.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| truncate_chars(s, 32));

    let primary = tokens.remove(0);
    let placement = Placement {
        center: [cx, cy],
        size: norm_h,
        opacity,
    };
    Some(CustomButton {
        key: primary,
        layouts: [placement; 3],
        extra_keys: tokens,
        label,
        width_ratio,
        toggle,
        swipeable,
        passthrough,
        in_game,
        in_menu,
        bg_color: Some([0, 0, 0, 115]),
        stroke_color: None,
        drawer_parent: None,
    })
}

// ---------------------------------------------------------------------------
// Shared Settings Assembler & Keycode / Expression Utilities
// ---------------------------------------------------------------------------

fn assemble_imported_settings(
    name: String,
    format: String,
    custom_buttons: Vec<CustomButton>,
    drawers: usize,
    joystick: Option<(Placement, bool, bool)>,
) -> ImportedControlLayout {
    let mut settings = Settings::default();
    let has_wasd = custom_buttons.iter().any(|b| {
        matches!(
            b.key.as_str(),
            "key.keyboard.w"
                | "key.keyboard.a"
                | "key.keyboard.s"
                | "key.keyboard.d"
                | "action:key.forward"
                | "action:key.left"
                | "action:key.back"
                | "action:key.right"
        )
    });
    let has_attack_or_use = custom_buttons.iter().any(|b| {
        std::iter::once(&b.key)
            .chain(b.extra_keys.iter())
            .any(|tok| {
                matches!(
                    tok.as_str(),
                    "action:key.attack"
                        | "action:key.use"
                        | "special:mouse_pri"
                        | "special:mouse_sec"
                )
            })
    });
    let has_escape_or_menu = custom_buttons.iter().any(|b| {
        std::iter::once(&b.key)
            .chain(b.extra_keys.iter())
            .any(|tok| matches!(tok.as_str(), "key.keyboard.escape" | "special:menu"))
    });

    // Hide duplicate built-in HUD buttons so only the imported Java layout appears.
    settings.button_visibility = [2; 14];
    let has_joystick = if let Some((stick_place, forward_sprint, absolute_dpad)) = joystick {
        settings.button_visibility[Control::Move as usize] = 1;
        settings.sprint_with_joystick = forward_sprint;
        for row in &mut settings.layouts {
            row[Control::Move as usize] = stick_place;
        }
        settings.mode = if absolute_dpad {
            Mode::Dpad
        } else {
            Mode::Crosshair
        };
        true
    } else if has_wasd {
        settings.button_visibility[Control::Move as usize] = 2;
        settings.mode = Mode::Crosshair;
        false
    } else {
        // Keep built-in movement stick when layout neither defines WASD nor a joystick.
        settings.button_visibility[Control::Move as usize] = 1;
        settings.mode = Mode::Crosshair;
        true
    };

    if !has_escape_or_menu {
        settings.button_visibility[Control::Pause as usize] = 1;
    }

    settings.split_controls = true;
    settings.crosshair_gestures = !has_attack_or_use;
    settings.inventory_controls = true;
    settings.offhand_button = false;
    settings.shield_button = false;

    let meta = ImportedMeta {
        name,
        format,
        buttons: custom_buttons.len(),
        drawers,
        has_joystick,
    };
    settings.custom_buttons = custom_buttons;
    settings.imported_from = Some(meta.clone());
    settings.sanitize();
    ImportedControlLayout { settings, meta }
}

fn parse_visibility_mode(v: Option<&Value>) -> (bool, bool) {
    match v
        .and_then(Value::as_str)
        .unwrap_or("ALWAYS")
        .trim()
        .to_ascii_uppercase()
        .as_str()
    {
        "IN_GAME" | "INGAME" | "GAME" => (true, false),
        "MENU" | "IN_MENU" | "INMENU" | "GUI" => (false, true),
        _ => (true, true),
    }
}

fn json_f32(v: Option<&Value>) -> Option<f32> {
    v.and_then(|val| {
        val.as_f64()
            .map(|n| n as f32)
            .or_else(|| val.as_str()?.trim().parse::<f32>().ok())
    })
    .filter(|n| n.is_finite())
}

fn normalize_opacity(raw: Option<f64>) -> f32 {
    let v = raw.unwrap_or(0.75) as f32;
    if !v.is_finite() {
        return 0.75;
    }
    if v > 1.0 {
        (v / 100.0).clamp(0.05, 1.0)
    } else {
        v.clamp(0.05, 1.0)
    }
}

fn parse_argb_color(v: Option<&Value>) -> Option<[u8; 4]> {
    let val = v?;
    if let Some(n) = val.as_i64() {
        let u = n as i32 as u32;
        return Some([
            ((u >> 16) & 0xFF) as u8,
            ((u >> 8) & 0xFF) as u8,
            (u & 0xFF) as u8,
            ((u >> 24) & 0xFF) as u8,
        ]);
    }
    let s = val.as_str()?.trim().trim_start_matches('#');
    if s.len() == 8
        && let Ok(u) = u32::from_str_radix(s, 16)
    {
        return Some([
            ((u >> 16) & 0xFF) as u8,
            ((u >> 8) & 0xFF) as u8,
            (u & 0xFF) as u8,
            ((u >> 24) & 0xFF) as u8,
        ]);
    }
    if s.len() == 6
        && let Ok(u) = u32::from_str_radix(s, 16)
    {
        return Some([
            ((u >> 16) & 0xFF) as u8,
            ((u >> 8) & 0xFF) as u8,
            (u & 0xFF) as u8,
            180,
        ]);
    }
    None
}

fn parse_fcl_or_zalith_key(v: &Value) -> Option<String> {
    if let Some(code) = v.as_i64() {
        let code = code as i32;
        // FCL special mouse/scroll codes (1000..=1006)
        return match code {
            1000 => Some("action:key.attack".into()),
            1001 => Some("action:key.pickItem".into()),
            1002 => Some("action:key.use".into()),
            1003 => Some("special:scroll_up".into()),
            1004 => Some("special:scroll_down".into()),
            1005 => Some("special:virtual_mouse".into()),
            1006 => Some("special:keyboard".into()),
            _ => map_java_keycode(code, false),
        };
    }
    let raw = v.as_str()?.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(code) = raw.parse::<i32>() {
        return map_java_keycode(code, false);
    }
    if CustomButton::valid_token(raw) {
        return Some(raw.to_owned());
    }
    let upper = raw.to_ascii_uppercase();
    let stripped = upper
        .strip_prefix("GLFW_KEY_")
        .or_else(|| upper.strip_prefix("KEY_"))
        .unwrap_or(&upper);
    Some(
        match stripped {
            "MOUSE_BUTTON_LEFT" | "MOUSE_LEFT" | "MOUSE_PRI" | "LEFT_CLICK" | "ATTACK" => {
                "action:key.attack"
            }
            "MOUSE_BUTTON_RIGHT" | "MOUSE_RIGHT" | "MOUSE_SEC" | "RIGHT_CLICK" | "USE" => {
                "action:key.use"
            }
            "MOUSE_BUTTON_MIDDLE" | "MOUSE_MIDDLE" | "MOUSE_MID" | "PICK_ITEM" => {
                "action:key.pickItem"
            }
            "SCROLL_UP" | "MOUSE_SCROLL_UP" => "special:scroll_up",
            "SCROLL_DOWN" | "MOUSE_SCROLL_DOWN" => "special:scroll_down",
            "VIRTUAL_MOUSE" | "MOUSE_MODE" => "special:virtual_mouse",
            "KEYBOARD" | "OPEN_KEYBOARD" | "IME" => "special:keyboard",
            "GUI" | "TOGGLE_CTRL" | "HIDE_CONTROLS" => "special:gui_toggle",
            "MENU" | "GAME_MENU" => "special:menu",
            "SPACE" => "key.keyboard.space",
            "ESCAPE" | "ESC" => "key.keyboard.escape",
            "ENTER" | "RETURN" => "key.keyboard.enter",
            "TAB" => "key.keyboard.tab",
            "BACKSPACE" => "key.keyboard.backspace",
            "LEFT_SHIFT" | "LSHIFT" | "SHIFT" => "key.keyboard.left.shift",
            "RIGHT_SHIFT" | "RSHIFT" => "key.keyboard.right.shift",
            "LEFT_CONTROL" | "LCTRL" | "CTRL" => "key.keyboard.left.control",
            "RIGHT_CONTROL" | "RCTRL" => "key.keyboard.right.control",
            "LEFT_ALT" | "LALT" | "ALT" => "key.keyboard.left.alt",
            "RIGHT_ALT" | "RALT" => "key.keyboard.right.alt",
            "UP" => "key.keyboard.up",
            "DOWN" => "key.keyboard.down",
            "LEFT" => "key.keyboard.left",
            "RIGHT" => "key.keyboard.right",
            "SLASH" => "key.keyboard.slash",
            "GRAVE_ACCENT" => "key.keyboard.grave.accent",
            s if s.len() == 1 && s.as_bytes()[0].is_ascii_alphanumeric() => {
                return Some(format!("key.keyboard.{}", s.to_ascii_lowercase()));
            }
            s if s.starts_with('F')
                && s[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n)) =>
            {
                return Some(format!("key.keyboard.{}", s.to_ascii_lowercase()));
            }
            _ => return None,
        }
        .into(),
    )
}

/// Maps PojavLauncher / Amethyst / Mojo / GLFW / legacy LWJGL2 keycodes to Volt tokens.
pub fn map_java_keycode(code: i32, legacy_lwjgl2: bool) -> Option<String> {
    if code < 0 {
        return Some(
            match code {
                -1 => "special:keyboard",
                -2 => "special:gui_toggle",
                -3 => "action:key.attack",
                -4 => "action:key.use",
                -5 => "special:virtual_mouse",
                -6 => "action:key.pickItem",
                -7 => "special:scroll_up",
                -8 => "special:scroll_down",
                -9 => "special:menu",
                _ => return None,
            }
            .into(),
        );
    }
    if legacy_lwjgl2 {
        return map_lwjgl2_keycode(code).map(str::to_owned);
    }
    map_glfw_keycode(code).map(str::to_owned)
}

fn map_glfw_keycode(code: i32) -> Option<&'static str> {
    Some(match code {
        32 => "key.keyboard.space",
        39 => "key.keyboard.apostrophe",
        44 => "key.keyboard.comma",
        45 => "key.keyboard.minus",
        46 => "key.keyboard.period",
        47 => "key.keyboard.slash",
        48 => "key.keyboard.0",
        49 => "key.keyboard.1",
        50 => "key.keyboard.2",
        51 => "key.keyboard.3",
        52 => "key.keyboard.4",
        53 => "key.keyboard.5",
        54 => "key.keyboard.6",
        55 => "key.keyboard.7",
        56 => "key.keyboard.8",
        57 => "key.keyboard.9",
        59 => "key.keyboard.semicolon",
        61 => "key.keyboard.equal",
        65 => "key.keyboard.a",
        66 => "key.keyboard.b",
        67 => "key.keyboard.c",
        68 => "key.keyboard.d",
        69 => "key.keyboard.e",
        70 => "key.keyboard.f",
        71 => "key.keyboard.g",
        72 => "key.keyboard.h",
        73 => "key.keyboard.i",
        74 => "key.keyboard.j",
        75 => "key.keyboard.k",
        76 => "key.keyboard.l",
        77 => "key.keyboard.m",
        78 => "key.keyboard.n",
        79 => "key.keyboard.o",
        80 => "key.keyboard.p",
        81 => "key.keyboard.q",
        82 => "key.keyboard.r",
        83 => "key.keyboard.s",
        84 => "key.keyboard.t",
        85 => "key.keyboard.u",
        86 => "key.keyboard.v",
        87 => "key.keyboard.w",
        88 => "key.keyboard.x",
        89 => "key.keyboard.y",
        90 => "key.keyboard.z",
        91 => "key.keyboard.left.bracket",
        92 => "key.keyboard.backslash",
        93 => "key.keyboard.right.bracket",
        96 => "key.keyboard.grave.accent",
        161 => "key.keyboard.world.1",
        256 => "key.keyboard.escape",
        257 => "key.keyboard.enter",
        258 => "key.keyboard.tab",
        259 => "key.keyboard.backspace",
        260 => "key.keyboard.insert",
        261 => "key.keyboard.delete",
        262 => "key.keyboard.right",
        263 => "key.keyboard.left",
        264 => "key.keyboard.down",
        265 => "key.keyboard.up",
        266 => "key.keyboard.page.up",
        267 => "key.keyboard.page.down",
        268 => "key.keyboard.home",
        269 => "key.keyboard.end",
        280 => "key.keyboard.caps.lock",
        281 => "key.keyboard.scroll.lock",
        282 => "key.keyboard.num.lock",
        283 => "key.keyboard.print.screen",
        284 => "key.keyboard.pause",
        290 => "key.keyboard.f1",
        291 => "key.keyboard.f2",
        292 => "key.keyboard.f3",
        293 => "key.keyboard.f4",
        294 => "key.keyboard.f5",
        295 => "key.keyboard.f6",
        296 => "key.keyboard.f7",
        297 => "key.keyboard.f8",
        298 => "key.keyboard.f9",
        299 => "key.keyboard.f10",
        300 => "key.keyboard.f11",
        301 => "key.keyboard.f12",
        302 => "key.keyboard.f13",
        303 => "key.keyboard.f14",
        304 => "key.keyboard.f15",
        305 => "key.keyboard.f16",
        306 => "key.keyboard.f17",
        307 => "key.keyboard.f18",
        308 => "key.keyboard.f19",
        309 => "key.keyboard.f20",
        310 => "key.keyboard.f21",
        311 => "key.keyboard.f22",
        312 => "key.keyboard.f23",
        313 => "key.keyboard.f24",
        320 => "key.keyboard.keypad.0",
        321 => "key.keyboard.keypad.1",
        322 => "key.keyboard.keypad.2",
        323 => "key.keyboard.keypad.3",
        324 => "key.keyboard.keypad.4",
        325 => "key.keyboard.keypad.5",
        326 => "key.keyboard.keypad.6",
        327 => "key.keyboard.keypad.7",
        328 => "key.keyboard.keypad.8",
        329 => "key.keyboard.keypad.9",
        330 => "key.keyboard.keypad.period",
        331 => "key.keyboard.keypad.divide",
        332 => "key.keyboard.keypad.multiply",
        333 => "key.keyboard.keypad.subtract",
        334 => "key.keyboard.keypad.add",
        335 => "key.keyboard.keypad.enter",
        336 => "key.keyboard.keypad.equal",
        340 => "key.keyboard.left.shift",
        341 => "key.keyboard.left.control",
        342 => "key.keyboard.left.alt",
        343 => "key.keyboard.left.win",
        344 => "key.keyboard.right.shift",
        345 => "key.keyboard.right.control",
        346 => "key.keyboard.right.alt",
        347 => "key.keyboard.right.win",
        348 => "key.keyboard.menu",
        _ => return None,
    })
}

fn map_lwjgl2_keycode(code: i32) -> Option<&'static str> {
    Some(match code {
        1 => "key.keyboard.escape",
        2 => "key.keyboard.1",
        3 => "key.keyboard.2",
        4 => "key.keyboard.3",
        5 => "key.keyboard.4",
        6 => "key.keyboard.5",
        7 => "key.keyboard.6",
        8 => "key.keyboard.7",
        9 => "key.keyboard.8",
        10 => "key.keyboard.9",
        11 => "key.keyboard.0",
        12 => "key.keyboard.minus",
        13 => "key.keyboard.equal",
        14 => "key.keyboard.backspace",
        15 => "key.keyboard.tab",
        16 => "key.keyboard.q",
        17 => "key.keyboard.w",
        18 => "key.keyboard.e",
        19 => "key.keyboard.r",
        20 => "key.keyboard.t",
        21 => "key.keyboard.y",
        22 => "key.keyboard.u",
        23 => "key.keyboard.i",
        24 => "key.keyboard.o",
        25 => "key.keyboard.p",
        26 => "key.keyboard.left.bracket",
        27 => "key.keyboard.right.bracket",
        28 => "key.keyboard.enter",
        29 => "key.keyboard.left.control",
        30 => "key.keyboard.a",
        31 => "key.keyboard.s",
        32 => "key.keyboard.d",
        33 => "key.keyboard.f",
        34 => "key.keyboard.g",
        35 => "key.keyboard.h",
        36 => "key.keyboard.j",
        37 => "key.keyboard.k",
        38 => "key.keyboard.l",
        39 => "key.keyboard.semicolon",
        40 => "key.keyboard.apostrophe",
        41 => "key.keyboard.grave.accent",
        42 => "key.keyboard.left.shift",
        43 => "key.keyboard.backslash",
        44 => "key.keyboard.z",
        45 => "key.keyboard.x",
        46 => "key.keyboard.c",
        47 => "key.keyboard.v",
        48 => "key.keyboard.b",
        49 => "key.keyboard.n",
        50 => "key.keyboard.m",
        51 => "key.keyboard.comma",
        52 => "key.keyboard.period",
        53 => "key.keyboard.slash",
        54 => "key.keyboard.right.shift",
        56 => "key.keyboard.left.alt",
        57 => "key.keyboard.space",
        58 => "key.keyboard.caps.lock",
        59 => "key.keyboard.f1",
        60 => "key.keyboard.f2",
        61 => "key.keyboard.f3",
        62 => "key.keyboard.f4",
        63 => "key.keyboard.f5",
        64 => "key.keyboard.f6",
        65 => "key.keyboard.f7",
        66 => "key.keyboard.f8",
        67 => "key.keyboard.f9",
        68 => "key.keyboard.f10",
        87 => "key.keyboard.f11",
        88 => "key.keyboard.f12",
        157 => "key.keyboard.right.control",
        184 => "key.keyboard.right.alt",
        199 => "key.keyboard.home",
        200 => "key.keyboard.up",
        201 => "key.keyboard.page.up",
        203 => "key.keyboard.left",
        205 => "key.keyboard.right",
        207 => "key.keyboard.end",
        208 => "key.keyboard.down",
        209 => "key.keyboard.page.down",
        210 => "key.keyboard.insert",
        211 => "key.keyboard.delete",
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// PojavLauncher Dynamic Coordinate Expression Parser (`${right} - ${margin} * 2 - 50dp`)
// ---------------------------------------------------------------------------

struct ExprVars {
    screen_w: f32,
    screen_h: f32,
    btn_w: f32,
    btn_h: f32,
    dp_px: f32,
    px_scale: f32,
    preferred_scale: f32,
}

fn eval_dynamic_expr(input: &str, vars: &ExprVars, axis_len: f32) -> Option<f32> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.len() > 512 {
        return None;
    }
    let mut parser = ExprParser {
        bytes: trimmed.as_bytes(),
        pos: 0,
        vars,
        axis_len,
        depth: 0,
    };
    let val = parser.parse_add_sub()?;
    parser.skip_ws();
    if parser.pos == parser.bytes.len() && val.is_finite() {
        Some(val)
    } else {
        None
    }
}

struct ExprParser<'a> {
    bytes: &'a [u8],
    pos: usize,
    vars: &'a ExprVars,
    axis_len: f32,
    depth: u8,
}

impl ExprParser<'_> {
    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn parse_add_sub(&mut self) -> Option<f32> {
        let mut left = self.parse_mul_div()?;
        loop {
            self.skip_ws();
            let Some(&op) = self.bytes.get(self.pos) else {
                break;
            };
            if op != b'+' && op != b'-' {
                break;
            }
            self.pos += 1;
            let right = self.parse_mul_div()?;
            left = if op == b'+' {
                left + right
            } else {
                left - right
            };
        }
        Some(left)
    }

    fn parse_mul_div(&mut self) -> Option<f32> {
        let mut left = self.parse_unary()?;
        loop {
            self.skip_ws();
            let Some(&op) = self.bytes.get(self.pos) else {
                break;
            };
            if op != b'*' && op != b'/' && op != b'%' {
                break;
            }
            self.pos += 1;
            let right = self.parse_unary()?;
            left = match op {
                b'*' => left * right,
                b'/' => {
                    if right.abs() < 1e-6 {
                        0.0
                    } else {
                        left / right
                    }
                }
                _ => {
                    if right.abs() < 1e-6 {
                        0.0
                    } else {
                        left % right
                    }
                }
            };
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<f32> {
        self.skip_ws();
        match self.bytes.get(self.pos).copied() {
            Some(b'+') => {
                self.pos += 1;
                self.parse_unary()
            }
            Some(b'-') => {
                self.pos += 1;
                self.parse_unary().map(|v| -v)
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Option<f32> {
        self.skip_ws();
        let b = *self.bytes.get(self.pos)?;
        if b == b'(' {
            if self.depth >= 16 {
                return None;
            }
            self.depth += 1;
            self.pos += 1;
            let val = self.parse_add_sub()?;
            self.skip_ws();
            if self.bytes.get(self.pos) != Some(&b')') {
                return None;
            }
            self.pos += 1;
            self.depth -= 1;
            return Some(self.apply_unit(val));
        }
        if b == b'$' && self.bytes.get(self.pos + 1) == Some(&b'{') {
            self.pos += 2;
            let start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos] != b'}' {
                self.pos += 1;
            }
            if self.bytes.get(self.pos) != Some(&b'}') {
                return None;
            }
            let name = std::str::from_utf8(&self.bytes[start..self.pos])
                .ok()?
                .trim();
            self.pos += 1;
            let val = match name.to_ascii_lowercase().as_str() {
                "width" | "screen_width" => self.vars.screen_w,
                "height" | "screen_height" => self.vars.screen_h,
                "half_width" => self.vars.screen_w * 0.5,
                "half_height" => self.vars.screen_h * 0.5,
                "right" => (self.vars.screen_w - self.vars.btn_w).max(0.0),
                "bottom" => (self.vars.screen_h - self.vars.btn_h).max(0.0),
                "top" | "left" => 0.0,
                "margin" => 2.0 * self.vars.dp_px,
                "preferred_scale" | "scale" => self.vars.preferred_scale,
                "btn_width" | "button_width" => self.vars.btn_w,
                "btn_height" | "button_height" => self.vars.btn_h,
                _ => return None,
            };
            return Some(self.apply_unit(val));
        }
        let start = self.pos;
        while let Some(&c) = self.bytes.get(self.pos) {
            if c.is_ascii_digit() || c == b'.' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return None;
        }
        let num = std::str::from_utf8(&self.bytes[start..self.pos])
            .ok()?
            .parse::<f32>()
            .ok()?;
        Some(self.apply_unit(num))
    }

    fn apply_unit(&mut self, value: f32) -> f32 {
        let rest = &self.bytes[self.pos..];
        if rest.len() >= 3 && rest[..3].eq_ignore_ascii_case(b"dip") {
            self.pos += 3;
            value * self.vars.dp_px
        } else if rest.len() >= 2
            && (rest[..2].eq_ignore_ascii_case(b"dp") || rest[..2].eq_ignore_ascii_case(b"sp"))
        {
            self.pos += 2;
            value * self.vars.dp_px
        } else if rest.len() >= 2 && rest[..2].eq_ignore_ascii_case(b"px") {
            self.pos += 2;
            value * self.vars.px_scale
        } else if rest.first() == Some(&b'%')
            && !rest.get(1).is_some_and(|c| c.is_ascii_digit())
        {
            self.pos += 1;
            value * self.axis_len / 100.0
        } else {
            value
        }
    }
}

// ---------------------------------------------------------------------------
// Preset Library & Per-Profile Persistence
// ---------------------------------------------------------------------------

fn presets_dir(root_home: &Path) -> PathBuf {
    root_home.join("imported-controls")
}

fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
        if slug.len() >= 40 {
            break;
        }
    }
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "controls".into()
    } else {
        trimmed.to_owned()
    }
}

pub fn save_preset_and_apply(
    root_home: &Path,
    profile_home: &Path,
    imported: &ImportedControlLayout,
) -> Result<String, String> {
    let dir = presets_dir(root_home);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Could not create preset folder: {e}"))?;
    if std::fs::symlink_metadata(&dir)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("Refusing a redirected control preset directory".into());
    }
    let slug = slugify(&imported.meta.name);
    let preset_path = dir.join(format!("{slug}.preset.json"));
    imported
        .settings
        .save_to(&preset_path)
        .map_err(|e| format!("Could not save control preset: {e}"))?;
    prune_presets(&dir, &preset_path);
    apply_to_profile(profile_home, &imported.settings)?;
    Ok(slug)
}

fn prune_presets(dir: &Path, keep: &Path) {
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if !p.file_name()?.to_str()?.ends_with(".preset.json") {
                return None;
            }
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((modified, p))
        })
        .collect();
    if files.len() <= MAX_PRESETS {
        return;
    }
    files.sort_by_key(|(t, _)| *t);
    for (_, path) in files.into_iter().take(MAX_PRESETS.min(8)) {
        if path != keep {
            let _ = std::fs::remove_file(path);
        }
    }
}

pub fn list_presets(root_home: &Path) -> Vec<PresetEntry> {
    let dir = presets_dir(root_home);
    let Ok(meta) = std::fs::symlink_metadata(&dir) else {
        return Vec::new();
    };
    if meta.file_type().is_symlink() {
        return Vec::new();
    }
    let mut items: Vec<(std::time::SystemTime, PresetEntry)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let file_name = entry.file_name().to_str()?.to_owned();
            let slug = file_name.strip_suffix(".preset.json")?.to_owned();
            let bytes = std::fs::read(entry.path()).ok()?;
            let settings: Settings = serde_json::from_slice(&bytes).ok()?;
            let imported = settings.imported_from?;
            let modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::UNIX_EPOCH);
            Some((modified, PresetEntry { slug, meta: imported }))
        })
        .collect();
    items.sort_by(|a, b| b.0.cmp(&a.0));
    items.into_iter().map(|(_, entry)| entry).collect()
}

pub fn load_preset(root_home: &Path, slug: &str) -> Result<Settings, String> {
    if slug.is_empty()
        || !slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("Invalid control preset name".into());
    }
    let path = presets_dir(root_home).join(format!("{slug}.preset.json"));
    if !path.is_file() {
        return Err("Control preset file is no longer available".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let mut settings: Settings =
        serde_json::from_slice(&bytes).map_err(|e| format!("Corrupt preset: {e}"))?;
    settings.sanitize();
    Ok(settings)
}

pub fn active_meta(profile_home: &Path) -> Option<ImportedMeta> {
    let bytes = std::fs::read(profile_home.join("horizon-touch.json")).ok()?;
    let settings: Settings = serde_json::from_slice(&bytes).ok()?;
    settings.imported_from
}

pub fn apply_to_profile(profile_home: &Path, settings: &Settings) -> Result<(), String> {
    std::fs::create_dir_all(profile_home).map_err(|e| e.to_string())?;
    if std::fs::symlink_metadata(profile_home)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("Refusing a redirected profile directory".into());
    }
    let mut clean = settings.clone();
    clean.sanitize();
    clean
        .save_to(&profile_home.join("horizon-touch.json"))
        .map_err(|e| format!("Could not write touch settings: {e}"))
}

pub fn apply_to_all_profiles(root_home: &Path, settings: &Settings) -> Result<usize, String> {
    let mut count = 0usize;
    apply_to_profile(root_home, settings)?;
    count += 1;
    let profiles_dir = root_home.join("profiles");
    if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir()
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|n| n.starts_with("profile-"))
            {
                apply_to_profile(&p, settings)?;
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn reset_profile(profile_home: &Path) -> Result<(), String> {
    apply_to_profile(profile_home, &Settings::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::touch::TouchControls;
    use std::io::Write;

    #[test]
    fn pojav_v8_layout_expressions_macros_drawers_and_joystick_import_cleanly() {
        let pojav = serde_json::json!({
            "version": 8,
            "scaledAt": 100.0,
            "mControlDataList": [
                {
                    "name": "GUI",
                    "keycodes": [-2, 0, 0, 0],
                    "dynamicX": "${margin}",
                    "dynamicY": "${top} + ${margin}",
                    "width": 80.0,
                    "height": 30.0,
                    "opacity": 0.8,
                    "isToggle": false,
                    "passThruEnabled": false,
                    "displayInGame": true,
                    "displayInMenu": true,
                    "bgColor": 1291845632
                },
                {
                    "name": "PRI",
                    "keycodes": [-3, 0, 0, 0],
                    "dynamicX": "${right} - ${margin}",
                    "dynamicY": "${bottom} - ${margin} * 2 - 50dp",
                    "width": 65.0,
                    "height": 65.0,
                    "opacity": 75.0,
                    "isToggle": false,
                    "passThruEnabled": true,
                    "displayInGame": true,
                    "displayInMenu": true
                },
                {
                    "name": "SEC",
                    "keycodes": [-4, 0, 0, 0],
                    "dynamicX": "${right} - 80dp",
                    "dynamicY": "${height} * 0.45",
                    "width": 60.0,
                    "height": 60.0,
                    "opacity": 0.7,
                    "passThruEnabled": true,
                    "displayInGame": true,
                    "displayInMenu": true
                },
                {
                    "name": "Sneak",
                    "keycodes": [340, 0, 0, 0],
                    "dynamicX": "20dp",
                    "dynamicY": "${bottom} - 120dp",
                    "width": 50.0,
                    "height": 50.0,
                    "isToggle": true,
                    "displayInGame": true,
                    "displayInMenu": true
                },
                {
                    "name": "Hitbox",
                    "keycodes": [292, 66, 0, 0],
                    "dynamicX": "${width} * 0.5 - 25dp",
                    "dynamicY": "${margin}",
                    "width": 50.0,
                    "height": 30.0,
                    "displayInGame": true,
                    "displayInMenu": false
                },
                {
                    "name": "Esc",
                    "keycodes": [256, 0, 0, 0],
                    "dynamicX": "${right} - ${margin}",
                    "dynamicY": "${margin}",
                    "width": 50.0,
                    "height": 30.0,
                    "displayInGame": true,
                    "displayInMenu": true
                }
            ],
            "mDrawerDataList": [
                {
                    "orientation": "DOWN",
                    "properties": {
                        "name": "Extras",
                        "keycodes": [0, 0, 0, 0],
                        "dynamicX": "100dp",
                        "dynamicY": "${margin}",
                        "width": 60.0,
                        "height": 30.0,
                        "displayInGame": true,
                        "displayInMenu": false
                    },
                    "buttonProperties": [
                        {
                            "name": "3rd",
                            "keycodes": [294, 0, 0, 0],
                            "width": 60.0,
                            "height": 30.0,
                            "displayInGame": true,
                            "displayInMenu": false
                        }
                    ]
                }
            ],
            "mJoystickDataList": [
                {
                    "dynamicX": "24dp",
                    "dynamicY": "${bottom} - 140dp",
                    "width": 120.0,
                    "height": 120.0,
                    "opacity": 0.65,
                    "forwardLock": true,
                    "absolute": false
                }
            ]
        });

        let imported = import_bytes(&serde_json::to_vec(&pojav).unwrap(), "PvP_Claw.json").unwrap();
        assert_eq!(imported.meta.name, "PvP_Claw");
        assert_eq!(imported.meta.format, "PojavLauncher");
        assert_eq!(imported.meta.drawers, 1);
        assert!(imported.meta.has_joystick);
        assert_eq!(imported.settings.custom_buttons.len(), 8);
        assert!(imported.settings.inventory_controls);
        assert!(!imported.settings.crosshair_gestures);
        assert!(imported.settings.sprint_with_joystick);
        assert_eq!(
            imported.settings.button_visibility[Control::Move as usize],
            1
        );
        assert_eq!(
            imported.settings.button_visibility[Control::Jump as usize],
            2
        );

        let mut touch = TouchControls {
            settings: imported.settings.clone(),
            loaded: true,
            ..Default::default()
        };
        touch.viewport(2400, 1080);

        // Tap PRI button & drag -> emits attack AND rotates camera via passthrough
        let pri_center = touch.settings.custom_buttons[1].layouts[0].center;
        touch.update(1, pri_center, true, false);
        assert_eq!(touch.key_events(), [("action:key.attack".into(), true)]);
        let look = touch.update(1, [pri_center[0] - 0.04, pri_center[1]], false, false);
        assert!(look[0] < -0.02);
        touch.update(1, pri_center, false, true);
        assert_eq!(touch.key_events(), [("action:key.attack".into(), false)]);

        // Tap Hitbox (F3 + B) -> emits F3 then B in order
        let hb_center = touch.settings.custom_buttons[4].layouts[0].center;
        touch.update(2, hb_center, true, false);
        assert_eq!(
            touch.key_events(),
            [
                ("key.keyboard.f3".into(), true),
                ("key.keyboard.b".into(), true)
            ]
        );
        touch.update(2, hb_center, false, true);
        assert_eq!(
            touch.key_events(),
            [
                ("key.keyboard.b".into(), false),
                ("key.keyboard.f3".into(), false)
            ]
        );

        // Tap GUI toggle -> hides other buttons while keeping GUI button reachable
        let gui_center = touch.settings.custom_buttons[0].layouts[0].center;
        touch.update(3, gui_center, true, false);
        touch.update(3, gui_center, false, true);
        assert!(touch.controls_hidden);
        assert_eq!(touch.custom_at(pri_center), None);
        assert_eq!(touch.custom_at(gui_center), Some(0));
    }

    #[test]
    fn fcl_and_zalith_zip_archives_import_and_roundtrip_presets() {
        let fcl = serde_json::json!({
            "manifest": { "name": "FCL Bedwars" },
            "viewGroups": [
                {
                    "id": "main",
                    "visibility": "VISIBLE",
                    "viewData": {
                        "buttonList": [
                            {
                                "text": "Attack",
                                "baseInfo": {
                                    "xPosition": 900,
                                    "yPosition": 650,
                                    "width": 600,
                                    "height": 600,
                                    "sizeType": "ABSOLUTE",
                                    "visibilityType": "ALWAYS"
                                },
                                "event": {
                                    "pressEvent": {
                                        "autoKeep": false,
                                        "movable": true,
                                        "outputKeycodes": [1000]
                                    }
                                }
                            },
                            {
                                "text": "Jump",
                                "baseInfo": {
                                    "xPosition": 880,
                                    "yPosition": 850,
                                    "width": 500,
                                    "height": 500,
                                    "sizeType": "ABSOLUTE",
                                    "visibilityType": "IN_GAME"
                                },
                                "event": {
                                    "outputKeycodes": [32]
                                }
                            }
                        ],
                        "directionList": [
                            {
                                "styleType": "ROCKER",
                                "baseInfo": {
                                    "xPosition": 120,
                                    "yPosition": 750,
                                    "width": 1200,
                                    "height": 1200,
                                    "sizeType": "ABSOLUTE"
                                }
                            }
                        ]
                    }
                }
            ]
        });
        let fcl_res = import_bytes(&serde_json::to_vec(&fcl).unwrap(), "bedwars.json").unwrap();
        assert_eq!(fcl_res.meta.name, "FCL Bedwars");
        assert_eq!(fcl_res.meta.format, "Fold Craft (FCL)");
        assert_eq!(fcl_res.settings.custom_buttons.len(), 2);
        assert!(fcl_res.settings.custom_buttons[0].passthrough);
        assert!(!fcl_res.settings.custom_buttons[1].in_menu);

        // Wrap a Zalith 2 layout inside a real ZIP archive
        let zalith = serde_json::json!({
            "info": { "name": "Zalith Speedrun" },
            "layers": [
                {
                    "hide": false,
                    "normalButtons": [
                        {
                            "text": "Swap",
                            "x": 8200,
                            "y": 4000,
                            "width": 50,
                            "height": 50,
                            "visibilityType": "IN_GAME",
                            "clickEvents": [{ "key": "GLFW_KEY_F" }]
                        },
                        {
                            "text": "Mouse",
                            "x": 500,
                            "y": 500,
                            "width": 60,
                            "height": 32,
                            "visibilityType": "IN_MENU",
                            "clickEvents": [{ "key": "VIRTUAL_MOUSE" }]
                        }
                    ]
                }
            ]
        });
        let mut zip_bytes = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(Cursor::new(&mut zip_bytes));
            let opts = zip::write::SimpleFileOptions::default();
            writer.start_file("manifest.json", opts).unwrap();
            writer
                .write_all(br#"{"name":"Zalith Pack"}"#)
                .unwrap();
            writer.start_file("controls/layout.json", opts).unwrap();
            writer
                .write_all(&serde_json::to_vec(&zalith).unwrap())
                .unwrap();
            writer.finish().unwrap();
        }
        let zip_res = import_bytes(&zip_bytes, "speedrun.zip").unwrap();
        assert_eq!(zip_res.meta.format, "Zalith Launcher");
        assert_eq!(zip_res.settings.custom_buttons.len(), 2);
        assert_eq!(zip_res.settings.custom_buttons[0].key, "key.keyboard.f");
        assert_eq!(
            zip_res.settings.custom_buttons[1].key,
            "special:virtual_mouse"
        );
    }
}
