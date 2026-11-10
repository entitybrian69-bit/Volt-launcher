//! Android's native entry point. The simulation and renderer are shared with desktop.
#![cfg(target_os = "android")]

#[allow(dead_code, clippy::collapsible_match)]
// Shared vanilla match structure and desktop harnesses.
mod game;
#[path = "android/launcher.rs"]
mod launcher;
// Share the desktop launcher's actual search, compatibility checks, downloads,
// dependency resolution and Java content porting; GPUI itself is desktop-only.
// These files are formatted in the launcher's separate Rust 2021 workspace.
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../launcher/src/account.rs"]
mod account;
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../launcher/src/browser.rs"]
mod browser;
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../launcher/src/server_list.rs"]
mod server_list;
#[path = "android/dns.rs"]
mod android_dns;
#[path = "android/catalog.rs"]
mod catalog;
#[path = "android/device_preferences.rs"]
mod device_preferences;
#[path = "android/device_runtime.rs"]
mod device_runtime;
#[path = "android/diagnostics.rs"]
mod diagnostics;
#[path = "android/discord_sdk.rs"]
mod discord_sdk;
#[path = "android/display.rs"]
mod display;
#[path = "android/document_bridge.rs"]
mod document_bridge;
#[path = "android/document_provider.rs"]
mod document_provider;
#[path = "android/document_store.rs"]
mod document_store;
#[path = "android/documents.rs"]
mod documents;
#[path = "android/exit_reports.rs"]
mod exit_reports;
#[path = "android/gallery.rs"]
mod gallery;
#[path = "android/native_ime.rs"]
mod native_ime;
#[path = "android/performance.rs"]
mod performance;
#[path = "android/platform.rs"]
mod platform;
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../launcher/src/proxy.rs"]
mod proxy;
#[path = "android/search_queue.rs"]
mod search_queue;
#[path = "android/secure_store.rs"]
mod secure_store;
#[path = "android/shader_catalog.rs"]
mod shader_catalog;
#[path = "android/skin_preview.rs"]
mod skin_preview;
#[path = "android/volt_accounts.rs"]
mod volt_accounts;
#[path = "android/volt_updater.rs"]
mod volt_updater;
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../launcher/src/skins.rs"]
mod skins;
// The shared account protocol needs only the launcher's version and data path.
mod backend {
    pub const VERSION: &str = if crate::PUBLIC_BUILD {
        "0.9.9"
    } else {
        "0.9.9-dev"
    };
    static HOME: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    pub fn initialize(home: std::path::PathBuf) {
        let _ = HOME.set(home);
    }
    pub fn game_home() -> std::path::PathBuf {
        data_dir()
    }
    pub fn game_mod_api_level() -> Option<u32> {
        Some(horizon_mods::api_level::LEVEL)
    }
    pub fn data_dir() -> std::path::PathBuf {
        HOME.get().expect("android_main sets account home").clone()
    }
}

use std::cell::RefCell;
pub(crate) fn keyboard_mouse_input() -> bool {
    horizon_client::android_runtime::active_input() == catalog::InputHost::KeyboardMouse
}
pub(crate) fn configure_input_mode(mode: catalog::InputHost) {
    horizon_client::android_runtime::configure_input(mode);
}
pub(crate) fn touch_visible() -> bool {
    horizon_client::android_runtime::active_input() == catalog::InputHost::Touch
}
pub(crate) fn touch_input(touch: bool) {
    horizon_client::android_runtime::note_input(if touch {
        catalog::InputHost::Touch
    } else {
        catalog::InputHost::KeyboardMouse
    });
}
pub(crate) fn save_input_mode() {
    APP.with(|app| {
        if let Some(home) = app.borrow().as_ref().and_then(|a| a.internal_data_path()) {
            let mut config = catalog::Config::load(&home);
            let mode = horizon_client::android_runtime::input_mode();
            if config.input_host != mode {
                config.input_host = mode;
                if let Err(e) = config.save(&home) {
                    log::warn!("Saving input mode: {e}");
                }
            }
        }
    });
}
pub(crate) const PUBLIC_BUILD: bool = match option_env!("HORIZON_ANDROID_PUBLIC") {
    Some(value) => value.len() == 1 && value.as_bytes()[0] == b'1',
    None => false,
};

#[unsafe(no_mangle)]
fn android_on_create(state: &winit::platform::android::activity::OnCreateState<'_>) {
    init_logging();
    horizon_protocol::install_system_srv_lookup(android_dns::query_srv);
    if let Err(error) =
        unsafe { native_ime::initialize(state.vm_as_ptr().cast(), state.activity_as_ptr().cast()) }
    {
        log::error!("Android system keyboard initialization: {error}");
    }
}
use winit::{
    event_loop::EventLoop,
    platform::android::{EventLoopBuilderExtAndroid, activity::AndroidApp},
};

thread_local! {
    static APP: RefCell<Option<AndroidApp>> = const { RefCell::new(None) };
}

pub(crate) fn android_event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    APP.with(|app| {
        EventLoop::builder()
            .with_android_app(
                app.borrow()
                    .as_ref()
                    .expect("android_main sets APP")
                    .clone(),
            )
            .build()
    })
}

/// Development overrides for testing both of Horizon's single-player paths.
pub(crate) fn android_arguments() -> horizon_client::Result<Vec<String>> {
    let home = APP.with(|app| app.borrow().as_ref().unwrap().internal_data_path().unwrap());
    let config = catalog::Config::load(&home);
    let mut args = vec![
        "--resources".into(),
        home.join(".local/horizon-client/26.3")
            .to_string_lossy()
            .into_owned(),
    ];
    if !PUBLIC_BUILD {
        args.extend([
            "--control-socket".into(),
            home.join("control.json").to_string_lossy().into_owned(),
        ]);
    }
    if let Some(session) = crate::launcher::session_argument() {
        args.extend([
            "--session-file".into(),
            session.to_string_lossy().into_owned(),
        ]);
    } else if !config.name.is_empty() {
        // Offline identity must never override a licensed session's profile.
        args.extend(["--name".into(), config.name.clone()]);
    }
    if !config.replay.is_empty() {
        // Volt Replay: watch the chosen recording instead of joining.
        args.extend(["--replay".into(), config.replay.clone()]);
    } else if !config.server.is_empty() {
        args.extend(["--server".into(), config.server.clone()]);
    }
    if !PUBLIC_BUILD {
        match std::fs::read(home.join("android-launch.json")) {
            Ok(bytes) => args.extend(serde_json::from_slice::<Vec<String>>(&bytes)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(args)
}

fn init_logging() {
    let trace_gles = !PUBLIC_BUILD && std::env::var_os("HORIZON_GLES_TRACE").is_some();
    let mut logger = android_logger::Config::default()
        .with_max_level(if trace_gles {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        })
        .with_tag("Horizon");
    if trace_gles {
        logger = logger.with_filter(
            android_logger::FilterBuilder::new()
                .parse("info,wgpu_hal::gles=trace,wgpu_core::device=debug")
                .build(),
        );
    }
    struct LocalLogger(android_logger::AndroidLogger);
    impl log::Log for LocalLogger {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            log::Log::enabled(&self.0, metadata)
        }
        fn log(&self, record: &log::Record<'_>) {
            if !self.enabled(record.metadata()) {
                return;
            }
            log::Log::log(&self.0, record);
            diagnostics::log_line(record.level(), record.target(), &record.args().to_string());
        }
        fn flush(&self) {
            diagnostics::request_flush();
        }
    }
    static LOGGER: std::sync::OnceLock<LocalLogger> = std::sync::OnceLock::new();
    let logger = LOGGER.get_or_init(|| LocalLogger(android_logger::AndroidLogger::new(logger)));
    if log::set_logger(logger).is_ok() {
        log::set_max_level(if trace_gles {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        });
    }
}

fn apk_engine_stamp() -> Option<String> {
    let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
    for line in maps.lines() {
        if !line.contains("libhorizon_android") && !line.contains("/base.apk") {
            continue;
        }
        let raw_path = line.split_whitespace().nth(5)?;
        let fs_path = raw_path
            .split_once("!/")
            .map_or(raw_path, |(apk, _)| apk);
        let meta = std::fs::metadata(fs_path).ok()?;
        let mtime = meta
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos();
        return Some(format!(
            "{}:{}:{}:{}:{}",
            backend::VERSION,
            option_env!("HORIZON_ANDROID_BUILD_COMMIT").unwrap_or("none"),
            fs_path,
            meta.len(),
            mtime,
        ));
    }
    None
}

static ENGINE_DATA_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
static ENGINE_DATA_DONE: (std::sync::Mutex<bool>, std::sync::Condvar) =
    (std::sync::Mutex::new(false), std::sync::Condvar::new());
static ACTIVE_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static MAIN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn ensure_engine_data(app: &AndroidApp, home: &std::path::Path) {
    struct DoneGuard;
    impl Drop for DoneGuard {
        fn drop(&mut self) {
            let (lock, cvar) = &ENGINE_DATA_DONE;
            *lock.lock().unwrap_or_else(|e| e.into_inner()) = true;
            cvar.notify_all();
        }
    }
    let _done = DoneGuard;
    let _guard = ENGINE_DATA_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let zip_path = home.join("engine-data.zip");
    let stamp_path = home.join("engine-data.stamp");
    let stamp = apk_engine_stamp();
    let up_to_date = stamp.as_deref().is_some_and(|expected| {
        zip_path.metadata().is_ok_and(|m| m.len() > 1024)
            && std::fs::read_to_string(&stamp_path).ok().as_deref() == Some(expected)
    });
    if up_to_date {
        return;
    }
    let Some(mut data) = app.asset_manager().open(c"engine-data.zip") else {
        return;
    };
    let copied = (|| -> std::io::Result<()> {
        let partial = home.join("engine-data.partial");
        let mut file = std::fs::File::create(&partial)?;
        std::io::copy(&mut data, &mut file)?;
        drop(file);
        std::fs::rename(&partial, &zip_path)?;
        if let Some(stamp) = &stamp {
            let _ = std::fs::write(&stamp_path, stamp);
        }
        Ok(())
    })();
    if let Err(error) = copied {
        log::error!("Cannot unpack engine data: {error}");
    }
}

pub(crate) fn engine_data_ready() -> bool {
    *ENGINE_DATA_DONE.0.lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) fn wait_for_engine_data() {
    let (lock, cvar) = &ENGINE_DATA_DONE;
    let mut done = lock.lock().unwrap_or_else(|e| e.into_inner());
    while !*done {
        done = cvar.wait(done).unwrap_or_else(|e| e.into_inner());
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    let generation = ACTIVE_GENERATION.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
    let _main_guard = MAIN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use winit::platform::android::activity::input::Axis;
    for axis in [
        Axis::Vscroll,
        Axis::Hscroll,
        Axis::RelativeX,
        Axis::RelativeY,
    ] {
        app.enable_motion_axis(axis);
    }
    init_logging();
    // Rich Presence goes through the Discord app on this phone; before any game start.
    discord_sdk::register();
    let Some(home) = app.internal_data_path() else {
        log::error!("Android supplied no application data directory");
        return;
    };
    backend::initialize(home.clone());
    if let Err(error) = diagnostics::initialize(&home, serde_json::json!({})) {
        log::error!("Cannot initialize local diagnostics: {error}");
    }
    if let Ok(vm) = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) } {
        // Keep the main native loop thread permanently attached so periodic JNI calls
        // never churn ART AttachCurrentThread/DetachCurrentThread per frame.
        let _ = vm.attach_current_thread_permanently();
        secure_store::initialize(vm);
    }
    let app_for_engine = app.clone();
    APP.with(|slot| *slot.borrow_mut() = Some(app));
    let Some(cache) = platform::cache_dir() else {
        log::error!("Android supplied no application cache directory");
        return;
    };
    horizon_client::android_runtime::configure_cache_dir(cache.clone());
    horizon_client::official_install::configure_download_cache_dir(cache);
    horizon_client::android_runtime::install_clipboard(
        horizon_client::android_runtime::ClipboardHooks {
            get: platform::clipboard_get,
            set: platform::clipboard_set,
        },
    );
    if let Err(error) =
        std::fs::create_dir_all(&home).and_then(|()| std::env::set_current_dir(&home))
    {
        log::error!("Cannot open {}: {error}", home.display());
        return;
    }
    #[cfg(debug_assertions)]
    if secure_store::development_probe() {
        std::process::exit(0);
    }
    horizon_client::crash::install(home.clone(), "client");
    exit_reports::install(&home);
    let _ = std::fs::remove_file(home.join("android-error.txt"));
    // Unpack `engine-data.zip` (~38 MB) on a background thread so `android_event_loop()`
    // starts draining NativeActivity lifecycle/input commands immediately on first launch.
    {
        *ENGINE_DATA_DONE.0.lock().unwrap_or_else(|e| e.into_inner()) = false;
        let home_bg = home.clone();
        let app_fallback = app_for_engine.clone();
        if std::thread::Builder::new()
            .name("volt-engine-unpack".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || ensure_engine_data(&app_for_engine, &home_bg))
            .is_err()
        {
            ensure_engine_data(&app_fallback, &home);
        }
    }
    let mut launcher = launcher::Launcher::new(home.clone());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        android_event_loop()
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })
            .and_then(|events| events.run_app(&mut launcher).map_err(Into::into))
    }))
    .unwrap_or_else(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .map(str::to_owned)
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unexpected native panic".into());
        Err(format!("Android runtime panic: {message}").into())
    });
    if let Err(error) = result {
        log::error!("Horizon: {error}");
        diagnostics::record_error(&error.to_string());
        horizon_client::crash::report_error(&home, "client", &*error);
        let _ = std::fs::write(home.join("android-error.txt"), error.to_string());
    }
    drop(launcher);
    diagnostics::finish();
    if ACTIVE_GENERATION.load(std::sync::atomic::Ordering::Acquire) == generation {
        platform::finish();
    }
    APP.with(|slot| *slot.borrow_mut() = None);
}
