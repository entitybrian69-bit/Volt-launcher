//! SAF selection, bounded background imports and explicit collision choices.
use crate::launcher::pages::theme;
use horizon_client::{
    Result,
    content_files::{self, Kind},
    hud::{HudAtlas, HudBatch},
    ui_theme::{Face, Ui},
};
use jni::{
    JNIEnv,
    objects::{JObject, JString},
};
use std::{
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

enum Target {
    Content {
        kind: Kind,
        home: PathBuf,
    },
    Controls {
        root_home: PathBuf,
        profile_home: PathBuf,
    },
    Cosmetic {
        is_cape: bool,
        dest_path: PathBuf,
    },
    VulkanDriver {
        home: PathBuf,
    },
}
struct Request {
    target: Target,
    cache: PathBuf,
}
struct Conflict {
    kind: Kind,
    staged: PathBuf,
    target: PathBuf,
}
#[derive(Clone)]
pub struct Notice {
    pub text: String,
    pub changed: bool,
    pub controls_changed: bool,
    pub cosmetic_imported: Option<(bool, PathBuf)>,
    pub driver_imported: Option<(String, PathBuf)>,
    pub kind: Option<Kind>,
}
#[derive(Default)]
struct State {
    request: Option<Request>,
    working: bool,
    conflict: Option<Conflict>,
    notice: Option<Notice>,
    hits: Vec<([f32; 4], Choice)>,
    toast: Option<(String, std::time::Instant)>,
    cursor: (f32, f32),
}
#[derive(Clone, Copy)]
pub enum Choice {
    KeepBoth,
    Replace,
    Cancel,
}
static STATE: Mutex<State> = Mutex::new(State {
    request: None,
    working: false,
    conflict: None,
    notice: None,
    hits: Vec::new(),
    toast: None,
    cursor: (0.0, 0.0),
});
static CANCEL: AtomicBool = AtomicBool::new(false);
pub fn busy() -> bool {
    let s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.request.is_some() || s.working || s.conflict.is_some()
}
pub fn conflict() -> bool {
    STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .conflict
        .is_some()
}
fn notice(text: impl Into<String>, changed: bool) {
    let text = text.into();
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.toast = Some((text.clone(), std::time::Instant::now()));
    s.notice = Some(Notice {
        text,
        changed,
        controls_changed: false,
        cosmetic_imported: None,
        driver_imported: None,
        kind: None,
    });
}
fn imported(text: impl Into<String>, kind: Kind) {
    let text = text.into();
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.toast = Some((text.clone(), std::time::Instant::now()));
    s.notice = Some(Notice {
        text,
        changed: true,
        controls_changed: false,
        cosmetic_imported: None,
        driver_imported: None,
        kind: Some(kind),
    });
}
fn controls_imported(text: impl Into<String>) {
    let text = text.into();
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.toast = Some((text.clone(), std::time::Instant::now()));
    s.notice = Some(Notice {
        text,
        changed: false,
        controls_changed: true,
        cosmetic_imported: None,
        driver_imported: None,
        kind: None,
    });
}
fn cosmetic_done(text: impl Into<String>, is_cape: bool, path: PathBuf) {
    let text = text.into();
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.toast = Some((text.clone(), std::time::Instant::now()));
    s.notice = Some(Notice {
        text,
        changed: false,
        controls_changed: false,
        cosmetic_imported: Some((is_cape, path)),
        driver_imported: None,
        kind: None,
    });
}
fn driver_done(text: impl Into<String>, label: String, path: PathBuf) {
    let text = text.into();
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.toast = Some((text.clone(), std::time::Instant::now()));
    s.notice = Some(Notice {
        text,
        changed: false,
        controls_changed: false,
        cosmetic_imported: None,
        driver_imported: Some((label, path)),
        kind: None,
    });
}
pub fn poll() -> Option<Notice> {
    if crate::document_provider::take_changes() {
        return Some(Notice {
            text: "Content files changed. Reload packs to apply.".into(),
            changed: true,
            controls_changed: false,
            cosmetic_imported: None,
            driver_imported: None,
            kind: None,
        });
    }
    STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .notice
        .take()
}
pub fn begin(kind: Kind, home: PathBuf) -> std::result::Result<(), String> {
    if busy() {
        return Err("Finish or cancel the current import first".into());
    }
    CANCEL.store(false, Ordering::Relaxed);
    crate::platform::hide();
    let cache = crate::platform::cache_dir()
        .ok_or("Could not access the Android import cache")?
        .join("content-imports");
    STATE.lock().unwrap_or_else(|e| e.into_inner()).request = Some(Request {
        target: Target::Content { kind, home },
        cache,
    });
    if !crate::native_ime::pick_document() {
        picker_failed("Android document picker is unavailable");
        return Err("Android document picker is unavailable".into());
    }
    notice(format!("Choose a {} file", kind.label()), false);
    Ok(())
}
pub fn begin_controls(root_home: PathBuf, profile_home: PathBuf) -> std::result::Result<(), String> {
    if busy() {
        return Err("Finish or cancel the current import first".into());
    }
    CANCEL.store(false, Ordering::Relaxed);
    crate::platform::hide();
    let cache = crate::platform::cache_dir()
        .ok_or("Could not access the Android import cache")?
        .join("content-imports");
    STATE.lock().unwrap_or_else(|e| e.into_inner()).request = Some(Request {
        target: Target::Controls {
            root_home,
            profile_home,
        },
        cache,
    });
    if !crate::native_ime::pick_document() {
        picker_failed("Android document picker is unavailable");
        return Err("Android document picker is unavailable".into());
    }
    notice(
        "Choose a Java launcher control file (.json or .zip)",
        false,
    );
    Ok(())
}
pub fn begin_cosmetic(is_cape: bool, dest_path: PathBuf) -> std::result::Result<(), String> {
    if busy() {
        return Err("Finish or cancel the current import first".into());
    }
    CANCEL.store(false, Ordering::Relaxed);
    crate::platform::hide();
    let cache = crate::platform::cache_dir()
        .ok_or("Could not access the Android import cache")?
        .join("content-imports");
    STATE.lock().unwrap_or_else(|e| e.into_inner()).request = Some(Request {
        target: Target::Cosmetic { is_cape, dest_path },
        cache,
    });
    if !crate::native_ime::pick_document() {
        picker_failed("Android document picker is unavailable");
        return Err("Android document picker is unavailable".into());
    }
    let label = if is_cape { "cape PNG" } else { "skin PNG (64x64 or 64x32)" };
    notice(format!("Choose a {label} image"), false);
    Ok(())
}
pub fn begin_vulkan_driver(home: PathBuf) -> std::result::Result<(), String> {
    if busy() {
        return Err("Finish or cancel the current import first".into());
    }
    CANCEL.store(false, Ordering::Relaxed);
    crate::platform::hide();
    let cache = crate::platform::cache_dir()
        .ok_or("Could not access the Android import cache")?
        .join("content-imports");
    STATE.lock().unwrap_or_else(|e| e.into_inner()).request = Some(Request {
        target: Target::VulkanDriver { home },
        cache,
    });
    if !crate::native_ime::pick_document() {
        picker_failed("Android document picker is unavailable");
        return Err("Android document picker is unavailable".into());
    }
    notice(
        "Choose a custom Vulkan driver (.zip, .adpkg, or .so)",
        false,
    );
    Ok(())
}
pub fn picker_failed(error: &str) {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).request = None;
    notice(error, false);
}
/// Called by the approved Activity result bridge, never from a game tick.
/// Keep a conventional JNI export as well as explicit registration: Android
/// may restore a picker result before the IME bridge has been initialized.
#[unsafe(export_name = "Java_dev_horizon_android_HorizonActivity_nativeDocumentResult")]
pub extern "system" fn document_result(
    mut env: JNIEnv<'_>,
    activity: JObject<'_>,
    request: i32,
    result: i32,
    uri: JString<'_>,
) {
    if request != 4101 {
        return;
    }
    let pending = STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .request
        .take();
    let Some(pending) = pending else {
        notice("Import interrupted. Choose the file again.", false);
        return;
    };
    if result != -1 || uri.is_null() {
        notice("Import cancelled", false);
        return;
    }
    let setup = (|| -> jni::errors::Result<_> {
        let text: String = env.get_string(&uri)?.into();
        let activity = env.new_global_ref(activity)?;
        Ok((env.get_java_vm()?, activity, text))
    })();
    let Ok((vm, activity, uri)) = setup else {
        let _ = env.exception_clear();
        notice("Could not read the selected document", false);
        return;
    };
    if !uri.starts_with("content://") || uri.len() > 8192 {
        notice("Unsupported document URI", false);
        return;
    }
    STATE.lock().unwrap_or_else(|e| e.into_inner()).working = true;
    let spawned = std::thread::Builder::new()
        .name("Content import".into())
        .spawn(move || {
            let result = (|| -> Result<()> {
                let mut env = vm.attach_current_thread()?;
                let fallback = match &pending.target {
                    Target::Content {
                        kind: Kind::Mods, ..
                    } => "import.hmod",
                    Target::Content { .. } => "import.zip",
                    Target::Controls { .. } => "controls.json",
                    Target::Cosmetic { is_cape: true, .. } => "cape.png",
                    Target::Cosmetic { is_cape: false, .. } => "skin.png",
                    Target::VulkanDriver { .. } => "driver.zip",
                };
                let opened =
                    crate::document_bridge::open(&mut env, activity.as_obj(), &uri, fallback);
                if opened.is_err() {
                    let _ = env.exception_clear();
                }
                let (input, name) = opened.map_err(|e| e.to_string())?;
                stage(pending, input, name)
            })();
            STATE.lock().unwrap_or_else(|e| e.into_inner()).working = false;
            if let Err(error) = result {
                notice(format!("Import failed: {error}"), false);
            }
        });
    if let Err(error) = spawned {
        STATE.lock().unwrap_or_else(|e| e.into_inner()).working = false;
        notice(format!("Import failed: {error}"), false);
    }
}
fn stage(request: Request, mut input: std::fs::File, name: String) -> Result<()> {
    std::fs::create_dir_all(&request.cache)?;
    if std::fs::symlink_metadata(&request.cache)?
        .file_type()
        .is_symlink()
    {
        return Err("Refusing a redirected import cache".into());
    }
    for entry in std::fs::read_dir(&request.cache)?.flatten() {
        let entry_name = entry.file_name().to_string_lossy().into_owned();
        if entry_name.starts_with(".import-")
            && entry_name.ends_with(".part")
            && entry.file_type()?.is_file()
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    if let Target::Controls {
        root_home,
        profile_home,
    } = request.target
    {
        return stage_controls(&request.cache, &root_home, &profile_home, input, &name);
    }
    if let Target::Cosmetic { is_cape, dest_path } = request.target {
        return stage_cosmetic(is_cape, &dest_path, input, &name);
    }
    if let Target::VulkanDriver { home } = request.target {
        return stage_vulkan_driver(&request.cache, &home, input, &name);
    }
    let Target::Content { kind, home } = request.target else {
        unreachable!()
    };
    if !content_files::plain_name(&name) || !kind.accepts(&name) {
        return Err("Selected file has the wrong extension for this folder".into());
    }
    let dir = home.join(kind.folder());
    std::fs::create_dir_all(&dir)?;
    if std::fs::symlink_metadata(&dir)?.file_type().is_symlink() {
        return Err("Refusing a redirected content directory".into());
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let staged = request.cache.join(format!(".import-{stamp}.part"));
    let result = (|| -> Result<()> {
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)?;
        content_files::copy_bounded(&mut input, &mut output, |bytes| {
            notice(
                format!("Importing {}: {} KiB", kind.label(), bytes / 1024),
                false,
            );
            !CANCEL.load(Ordering::Relaxed)
        })?;
        output.sync_all()?;
        drop(output);
        notice("Checking archive...", false);
        let native_shader = content_files::validate(&staged, kind, &name, || {
            CANCEL.load(Ordering::Relaxed)
        })?;
        let name = if kind == Kind::Mods {
            format!("{}.hmod", horizon_mods::Package::read(&staged)?.manifest.id)
        } else if native_shader || name.to_ascii_lowercase().ends_with(".hshader") {
            let pack = horizon_shader_api::Pack::read(&staged)?;
            horizon_shader_api::check(&pack).map_err(|e| format!("Invalid shader pack: {e:?}"))?;
            let mut path = PathBuf::from(name);
            path.set_extension("hshader");
            path.to_string_lossy().into_owned()
        } else {
            name
        };
        if !content_files::plain_name(&name) {
            return Err("Package has an invalid ID or filename".into());
        }
        if CANCEL.load(Ordering::Relaxed) {
            return Err("Import cancelled".into());
        }
        let target = dir.join(name);
        if target.exists() {
            STATE.lock().unwrap_or_else(|e| e.into_inner()).conflict = Some(Conflict {
                kind,
                staged: staged.clone(),
                target,
            });
            notice(
                "File already exists: choose Keep both, Replace or Cancel",
                false,
            );
        } else {
            std::fs::rename(&staged, &target)?;
            imported(
                format!(
                    "Imported {} into {}. Enable it in game settings.",
                    target.file_name().unwrap_or_default().to_string_lossy(),
                    kind.label()
                ),
                kind,
            );
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(staged);
    }
    result
}

fn stage_controls(
    cache: &std::path::Path,
    root_home: &std::path::Path,
    profile_home: &std::path::Path,
    mut input: std::fs::File,
    name: &str,
) -> Result<()> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let staged = cache.join(format!(".import-controls-{stamp}.part"));
    let result = (|| -> Result<()> {
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)?;
        content_files::copy_bounded(&mut input, &mut output, |bytes| {
            notice(
                format!("Importing control layout: {} KiB", bytes / 1024),
                false,
            );
            !CANCEL.load(Ordering::Relaxed)
        })?;
        output.sync_all()?;
        drop(output);
        if CANCEL.load(Ordering::Relaxed) {
            return Err("Import cancelled".into());
        }
        notice("Converting Java launcher controls...", false);
        let parsed = horizon_client::touch::import::import_file(&staged, name)?;
        horizon_client::touch::import::save_preset_and_apply(root_home, profile_home, &parsed)?;
        let extras = match (parsed.meta.drawers > 0, parsed.meta.has_joystick) {
            (true, true) => format!(", {} drawers, joystick", parsed.meta.drawers),
            (true, false) => format!(", {} drawers", parsed.meta.drawers),
            (false, true) => ", joystick".into(),
            (false, false) => String::new(),
        };
        controls_imported(format!(
            "Imported \"{}\" ({} • {} buttons{}) — active now!",
            parsed.meta.name, parsed.meta.format, parsed.meta.buttons, extras
        ));
        Ok(())
    })();
    let _ = std::fs::remove_file(&staged);
    result
}
pub fn resolve(choice: Choice) {
    let c = STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .conflict
        .take();
    let Some(mut c) = c else {
        return;
    };
    if matches!(choice, Choice::Cancel) {
        let _ = std::fs::remove_file(c.staged);
        notice("Import cancelled", false);
        return;
    }
    let result = (|| -> Result<()> {
        if matches!(choice, Choice::Replace)
            && std::fs::symlink_metadata(&c.target)
                .is_ok_and(|m| !m.is_file() || m.file_type().is_symlink())
        {
            return Err("Cannot replace a folder or symbolic link".into());
        }
        if matches!(choice, Choice::KeepBoth) {
            if c.kind == Kind::Mods {
                return Err(
                    "A mod ID cannot be installed twice; choose Replace to update it".into(),
                );
            }
            let mut stem = c
                .target
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            while stem.len() > 150 {
                stem.pop();
            }
            let ext = c
                .target
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let dir = c
                .target
                .parent()
                .ok_or("Invalid content folder")?
                .to_owned();
            for i in 1..=10000 {
                let candidate = dir.join(format!("{stem} ({i}).{ext}"));
                if !candidate.exists() {
                    c.target = candidate;
                    break;
                }
                if i == 10000 {
                    return Err("Too many filename conflicts".into());
                }
            }
        }
        std::fs::rename(&c.staged, &c.target)?;
        Ok(())
    })();
    match result {
        Ok(()) => imported(
            "Import completed. Enable it in Installed or game settings.",
            c.kind,
        ),
        Err(e) => {
            let _ = std::fs::remove_file(c.staged);
            notice(format!("Import failed: {e}"), false);
        }
    }
}
pub fn cancel() {
    CANCEL.store(true, Ordering::Relaxed);
    STATE.lock().unwrap_or_else(|e| e.into_inner()).request = None;
    if conflict() {
        resolve(Choice::Cancel);
    } else {
        notice("Import cancelled", false);
    }
}
pub fn tap(x: f32, y: f32) -> bool {
    if !busy() {
        return false;
    }
    let choice = STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .hits
        .iter()
        .find(|(r, _)| x >= r[0] && x <= r[0] + r[2] && y >= r[1] && y <= r[1] + r[3])
        .map(|(_, c)| *c);
    if let Some(choice) = choice {
        if matches!(choice, Choice::Cancel) {
            cancel();
        } else {
            resolve(choice);
        }
    }
    true
}
pub fn draw(batch: &mut HudBatch, atlas: &HudAtlas) {
    let mut ui = Ui::mobile(atlas, batch, (0.0, 0.0));
    draw_ui(&mut ui);
    ui.finish();
}
/// Launcher overlays must use its bundled font, including before game fonts exist.
pub fn draw_ui(ui: &mut Ui<'_>) {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.hits.clear();
    if s.conflict.is_none() && !s.working {
        // The launcher already shows the result in its status line. Keep only
        // actionable overlays (progress/cancel and a name-conflict choice).
        return;
    }
    let (name, choices) = if let Some(c) = &s.conflict {
        (
            c.target
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            if c.kind == Kind::Mods {
                vec![("Replace", Choice::Replace), ("Cancel", Choice::Cancel)]
            } else {
                vec![
                    ("Keep both", Choice::KeepBoth),
                    ("Replace", Choice::Replace),
                    ("Cancel", Choice::Cancel),
                ]
            },
        )
    } else {
        (
            s.toast.as_ref().map(|v| v.0.clone()).unwrap_or_default(),
            vec![("Cancel", Choice::Cancel)],
        )
    };
    let w = 460.0f32.min(ui.w - 24.0);
    let vertical = w < 320.0;
    let height = if vertical {
        98.0 + choices.len() as f32 * 44.0
    } else {
        152.0
    };
    let x = (ui.w - w) / 2.0;
    let y = ((ui.h - height) / 2.0).max(0.0);
    ui.rounded(0.0, 0.0, ui.w, ui.h, 0.0, theme::BACKDROP);
    ui.rounded(x, y, w, height, 14.0, theme::CARD);
    ui.border(x, y, w, height, 14.0, theme::BORDER_CRIMSON);
    ui.text(
        Face::Medium,
        19.0,
        x + 18.0,
        y + 14.0,
        theme::TEXT,
        if s.conflict.is_some() {
            "File exists"
        } else {
            "Importing file"
        },
    );
    let name = ui.fit_text(Face::Regular, 14.5, &name, w - 36.0);
    ui.text(Face::Regular, 14.5, x + 18.0, y + 48.0, theme::MUTED, &name);
    let width = (w - 36.0) / choices.len() as f32;
    for (i, (label, choice)) in choices.into_iter().enumerate() {
        let rect = if vertical {
            [x + 18.0, y + 88.0 + i as f32 * 44.0, w - 36.0, 38.0]
        } else {
            [x + 18.0 + i as f32 * width, y + 96.0, width - 6.0, 38.0]
        };
        let is_primary = matches!(choice, Choice::Replace | Choice::KeepBoth) && i == 0;
        ui.rounded(
            rect[0],
            rect[1],
            rect[2],
            rect[3],
            8.0,
            if is_primary {
                theme::CRIMSON
            } else {
                theme::CARD_INNER
            },
        );
        ui.border(
            rect[0],
            rect[1],
            rect[2],
            rect[3],
            8.0,
            if is_primary {
                theme::CRIMSON_BRIGHT
            } else {
                theme::BORDER
            },
        );
        let tw = ui.measure(Face::Medium, 14.5, label);
        ui.text(
            Face::Medium,
            14.5,
            rect[0] + ((rect[2] - tw) * 0.5).max(8.0),
            rect[1] + 8.5,
            if is_primary {
                theme::ON_CRIMSON
            } else {
                theme::TEXT
            },
            label,
        );
        s.hits.push((
            [
                rect[0] * ui.density,
                rect[1] * ui.density,
                rect[2] * ui.density,
                rect[3] * ui.density,
            ],
            choice,
        ));
    }
}
pub fn handle_event(event: &winit::event::WindowEvent) -> bool {
    use winit::{
        event::{ElementState, MouseButton, TouchPhase, WindowEvent},
        keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    };
    if !busy() {
        return false;
    }
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            STATE.lock().unwrap_or_else(|e| e.into_inner()).cursor =
                (position.x as f32, position.y as f32);
            true
        }
        WindowEvent::MouseInput {
            button: MouseButton::Left,
            state,
            ..
        } => {
            if *state == ElementState::Released {
                let (x, y) = STATE.lock().unwrap_or_else(|e| e.into_inner()).cursor;
                tap(x, y);
            }
            true
        }
        WindowEvent::Touch(t) => {
            if t.phase == TouchPhase::Ended {
                tap(t.location.x as f32, t.location.y as f32);
            }
            true
        }
        WindowEvent::KeyboardInput { event, .. } => {
            if event.state == ElementState::Pressed {
                if event.physical_key == PhysicalKey::Code(KeyCode::Escape)
                    || event.logical_key == Key::Named(NamedKey::BrowserBack)
                {
                    cancel();
                } else if event.physical_key == PhysicalKey::Code(KeyCode::Enter) && conflict() {
                    let native = STATE
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .conflict
                        .as_ref()
                        .is_some_and(|c| c.kind == Kind::Mods);
                    resolve(if native {
                        Choice::Cancel
                    } else {
                        Choice::KeepBoth
                    });
                }
            }
            true
        }
        _ => false,
    }
}

fn stage_cosmetic(
    is_cape: bool,
    dest_path: &std::path::Path,
    mut input: std::fs::File,
    name: &str,
) -> Result<()> {
    use std::io::Read as _;
    let mut bytes = Vec::new();
    input
        .by_ref()
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("Image file is too large (max 4 MB)".into());
    }
    let (w, h, rgba) = horizon_client::hud::decode_bytes(&bytes)
        .or_else(|_| {
            image::load_from_memory(&bytes)
                .map(|img| {
                    let rgba8 = img.to_rgba8();
                    let (iw, ih) = rgba8.dimensions();
                    (iw, ih, rgba8.into_raw())
                })
                .map_err(|_| ())
        })
        .map_err(|_| "Selected file is not a valid image")?;
    let normalized_rgba = if is_cape {
        horizon_client::skins::process_cape(w, h, &rgba)
            .ok_or("Selected image could not be processed as a cape")?
    } else {
        horizon_client::skins::process_legacy_skin(w, h, &rgba)
            .ok_or("Skin image must be 64x64 or 64x32")?
    };
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let (out_w, out_h, out_slice) = if is_cape {
        (64u32, 32u32, &normalized_rgba[..64 * 32 * 4])
    } else {
        (64u32, 64u32, normalized_rgba.as_slice())
    };
    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, out_w, out_h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| format!("PNG encode failed: {e}"))?;
        writer
            .write_image_data(out_slice)
            .map_err(|e| format!("PNG write failed: {e}"))?;
    }
    let tmp = dest_path.with_extension("png.tmp");
    std::fs::write(&tmp, &png_bytes)?;
    std::fs::rename(&tmp, dest_path)?;
    let kind_label = if is_cape { "Cape" } else { "Skin" };
    cosmetic_done(
        format!("{kind_label} equipped from {name}"),
        is_cape,
        dest_path.to_path_buf(),
    );
    Ok(())
}

fn validate_driver_elf(path: &std::path::Path) -> std::result::Result<(), String> {
    use std::io::Read as _;
    let mut f = std::fs::File::open(path).map_err(|e| format!("Could not read driver .so: {e}"))?;
    let mut hdr = [0u8; 20];
    let n = f
        .read(&mut hdr)
        .map_err(|e| format!("Could not inspect driver header: {e}"))?;
    if n < 20 || hdr[..4] != [0x7F, b'E', b'L', b'F'] {
        return Err("Selected driver library is not a valid Android ELF .so binary.".into());
    }
    let want_class = if cfg!(target_pointer_width = "64") {
        2u8
    } else {
        1u8
    };
    if hdr[4] != want_class {
        return Err(if want_class == 2 {
            "Selected Vulkan driver is 32-bit, but this device is running 64-bit Volt Launcher."
                .into()
        } else {
            "Selected Vulkan driver is 64-bit, but this device is running 32-bit Volt Launcher."
                .into()
        });
    }
    Ok(())
}

fn stage_vulkan_driver(
    cache: &std::path::Path,
    home: &std::path::Path,
    mut input: std::fs::File,
    name: &str,
) -> Result<()> {
    use std::io::Read as _;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let staged = cache.join(format!(".import-vkdriver-{stamp}.part"));
    let result = (|| -> Result<()> {
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)?;
        content_files::copy_bounded(&mut input, &mut output, |bytes| {
            notice(
                format!("Importing Vulkan driver: {} KiB", bytes / 1024),
                false,
            );
            !CANCEL.load(Ordering::Relaxed)
        })?;
        output.sync_all()?;
        drop(output);
        if CANCEL.load(Ordering::Relaxed) {
            return Err("Import cancelled".into());
        }

        let driver_dir = home.join("drivers").join("vulkan");
        std::fs::create_dir_all(&driver_dir)?;
        if let Ok(entries) = std::fs::read_dir(&driver_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }

        let mut magic = [0u8; 4];
        {
            let mut f = std::fs::File::open(&staged)?;
            let _ = f.read(&mut magic)?;
        }

        let (driver_label, primary_so) = if magic == [b'P', b'K', 3, 4] {
            let zip_file = std::fs::File::open(&staged)?;
            let mut archive = zip::ZipArchive::new(zip_file)
                .map_err(|e| format!("Invalid Vulkan driver package (.zip/.adpkg): {e}"))?;
            let mut meta_label: Option<String> = None;
            let mut meta_lib: Option<String> = None;
            let mut extracted_sos: Vec<PathBuf> = Vec::new();

            for idx in 0..archive.len() {
                let mut entry = archive
                    .by_index(idx)
                    .map_err(|e| format!("Driver archive read error: {e}"))?;
                if entry.is_dir() {
                    continue;
                }
                let Some(file_name) = PathBuf::from(entry.name())
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                else {
                    continue;
                };
                if file_name.eq_ignore_ascii_case("meta.json") {
                    let mut raw = Vec::new();
                    entry.by_ref().take(64 * 1024).read_to_end(&mut raw)?;
                    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&raw) {
                        meta_label = v
                            .get("name")
                            .and_then(|s| s.as_str())
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned);
                        meta_lib = v
                            .get("libraryName")
                            .or_else(|| v.get("library"))
                            .and_then(|s| s.as_str())
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned);
                    }
                    continue;
                }
                if file_name.to_ascii_lowercase().ends_with(".so")
                    && content_files::plain_name(&file_name)
                {
                    let out_path = driver_dir.join(&file_name);
                    let mut out_file = std::fs::File::create(&out_path)?;
                    std::io::copy(&mut entry.take(128 * 1024 * 1024), &mut out_file)?;
                    out_file.sync_all()?;
                    extracted_sos.push(out_path);
                }
            }

            if extracted_sos.is_empty() {
                return Err("Archive does not contain any Vulkan .so driver libraries.".into());
            }
            let chosen = meta_lib
                .as_deref()
                .and_then(|want| {
                    extracted_sos
                        .iter()
                        .find(|p| {
                            p.file_name()
                                .is_some_and(|n| n.to_string_lossy() == want)
                        })
                        .cloned()
                })
                .or_else(|| {
                    extracted_sos
                        .iter()
                        .find(|p| {
                            let lower = p
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_ascii_lowercase();
                            lower.contains("vulkan") || lower.contains("turnip")
                        })
                        .cloned()
                })
                .unwrap_or_else(|| extracted_sos[0].clone());
            validate_driver_elf(&chosen)?;
            let label = meta_label.unwrap_or_else(|| {
                PathBuf::from(name)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "Custom Vulkan Driver".into())
            });
            (label, chosen)
        } else if magic == [0x7F, b'E', b'L', b'F'] {
            validate_driver_elf(&staged)?;
            let clean_name = PathBuf::from(name)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .filter(|n| n.ends_with(".so") && content_files::plain_name(n))
                .unwrap_or_else(|| "libvulkan_custom.so".into());
            let dest = driver_dir.join(&clean_name);
            std::fs::copy(&staged, &dest)?;
            (clean_name, dest)
        } else {
            return Err("Choose a Turnip/AdrenoTools .zip/.adpkg package or an ELF .so library.".into());
        };

        driver_done(
            format!("Custom Vulkan driver \"{driver_label}\" imported & active!"),
            driver_label,
            primary_so,
        );
        Ok(())
    })();
    let _ = std::fs::remove_file(&staged);
    result
}


