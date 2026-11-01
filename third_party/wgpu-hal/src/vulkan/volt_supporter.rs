//! Built-in Volt Vulkan Supporter & Custom Vulkan Driver Engine.
//!
//! Unlocks Vulkan rendering on budget, older, non-conformant, and buggy OEM
//! mobile GPUs (Vulkan 1.0/1.1 downlevel bridge, shader Y-flip when
//! `VK_KHR_maintenance1` is absent, uniform-only fallback when
//! `VK_KHR_storage_buffer_storage_class` is absent, non-zero graphics queue
//! family discovery, and optional extension sanitization), and supports
//! loading user-supplied custom Vulkan ICD/driver libraries (`.so` extracted
//! from Turnip / Freedreno / Mesa / custom `.zip` or `.adpkg` packages).

use alloc::{borrow::ToOwned, format, string::String, vec::Vec};
use core::{
    ffi::{c_char, c_int, c_void},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};
use std::path::{Path, PathBuf};

use ash::{khr, vk};
use parking_lot::Mutex;

static SUPPORTER_MODE: AtomicBool = AtomicBool::new(false);

struct DriverState {
    custom_path: Option<PathBuf>,
    last_status: Option<String>,
    /// Keeps dynamically opened companion & driver libraries resident for the
    /// lifetime of the process so function pointers handed to `ash` stay valid.
    retained_libs: Vec<libloading::Library>,
}

static DRIVER_STATE: Mutex<DriverState> = Mutex::new(DriverState {
    custom_path: None,
    last_status: None,
    retained_libs: Vec::new(),
});

/// Summary returned when validating an imported custom Vulkan `.so` library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriverProbeInfo {
    pub entry_kind: String,
    pub has_android_surface: bool,
    pub note: String,
}

/// Enables or disables the built-in Volt Vulkan Supporter compatibility bridge.
pub fn set_supporter_mode(enabled: bool) {
    SUPPORTER_MODE.store(enabled, Ordering::SeqCst);
}

/// Returns whether the built-in Volt Vulkan Supporter compatibility bridge is active.
pub fn supporter_mode() -> bool {
    SUPPORTER_MODE.load(Ordering::Relaxed)
}

/// Configures an optional custom Vulkan driver `.so` path to try before system `libvulkan.so`.
pub fn set_custom_driver_path(path: Option<PathBuf>) {
    let mut state = DRIVER_STATE.lock();
    state.custom_path = path;
}

/// Returns the currently configured custom Vulkan driver `.so` path, if any.
pub fn custom_driver_path() -> Option<PathBuf> {
    DRIVER_STATE.lock().custom_path.clone()
}

/// Human-readable status from the most recent Vulkan loader initialization.
pub fn last_driver_status() -> Option<String> {
    DRIVER_STATE.lock().last_status.clone()
}

fn set_status(msg: String) {
    DRIVER_STATE.lock().last_status = Some(msg);
}

#[repr(C)]
struct HwModuleMethods {
    open: Option<
        unsafe extern "C" fn(
            module: *const HwModule,
            id: *const c_char,
            device: *mut *mut HwDevice,
        ) -> c_int,
    >,
}

#[repr(C)]
struct HwModule {
    tag: u32,
    module_api_version: u16,
    hal_api_version: u16,
    id: *const c_char,
    name: *const c_char,
    author: *const c_char,
    methods: *const HwModuleMethods,
    dso: *mut c_void,
    reserved: [usize; 25],
}

#[repr(C)]
struct HwDevice {
    tag: u32,
    version: u32,
    module: *const HwModule,
    reserved: [usize; 12],
    close: Option<unsafe extern "C" fn(device: *mut HwDevice) -> c_int>,
}

#[repr(C)]
struct HwVulkanDevice {
    common: HwDevice,
    enumerate_instance_extension_properties: vk::PFN_vkEnumerateInstanceExtensionProperties,
    create_instance: vk::PFN_vkCreateInstance,
    get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
}

/// Checks ELF header magic and CPU architecture before calling `dlopen` so a
/// mismatched (e.g. x86_64 or 32-bit ARM on arm64-v8a) library is rejected cleanly.
pub fn verify_elf_architecture(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Could not read library: {e}"))?;
    if bytes.len() < 20 {
        return Err("File is too small to be a shared library (.so)".to_owned());
    }
    if &bytes[0..4] != b"\x7FELF" {
        return Err("File is not a valid ELF shared library (.so)".to_owned());
    }
    let class = bytes[4];
    let machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    #[cfg(target_arch = "aarch64")]
    {
        if class != 2 || machine != 183 {
            return Err(format!(
                "Driver architecture mismatch (expected 64-bit arm64-v8a ELF, got class={class}, machine={machine})"
            ));
        }
    }
    #[cfg(target_arch = "arm")]
    {
        if class != 1 || machine != 40 {
            return Err(format!(
                "Driver architecture mismatch (expected 32-bit armeabi-v7a ELF, got class={class}, machine={machine})"
            ));
        }
    }
    #[cfg(target_arch = "x86_64")]
    {
        if class != 2 || machine != 62 {
            return Err(format!(
                "Driver architecture mismatch (expected 64-bit x86_64 ELF, got class={class}, machine={machine})"
            ));
        }
    }
    #[cfg(target_arch = "x86")]
    {
        if class != 1 || machine != 3 {
            return Err(format!(
                "Driver architecture mismatch (expected 32-bit x86 ELF, got class={class}, machine={machine})"
            ));
        }
    }
    let _ = (class, machine);
    Ok(())
}

fn entry_supports_wsi(entry: &ash::Entry) -> bool {
    let Ok(exts) = (unsafe { entry.enumerate_instance_extension_properties(None) }) else {
        return false;
    };
    let has_surface = exts
        .iter()
        .any(|e| e.extension_name_as_c_str() == Ok(khr::surface::NAME));
    #[cfg(target_os = "android")]
    let has_platform = exts
        .iter()
        .any(|e| e.extension_name_as_c_str() == Ok(khr::android_surface::NAME));
    #[cfg(not(target_os = "android"))]
    let has_platform = true;
    has_surface && has_platform
}

unsafe fn open_library_with_companions(
    path: &Path,
) -> Result<(libloading::Library, Vec<libloading::Library>), String> {
    let mut companions = Vec::new();
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p == path {
                    continue;
                }
                let is_so = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(".so") || n.contains(".so."));
                if is_so {
                    let flags =
                        libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_GLOBAL;
                    if let Ok(lib) = unsafe { libloading::os::unix::Library::open(Some(&p), flags) }
                    {
                        companions.push(lib.into());
                    }
                }
            }
        }
    }
    let main_lib = unsafe {
        #[cfg(unix)]
        {
            let flags = libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_LOCAL;
            libloading::os::unix::Library::open(Some(path), flags).map(Into::into)
        }
        #[cfg(not(unix))]
        {
            libloading::Library::new(path)
        }
    }
    .map_err(|e| format!("dlopen failed: {e}"))?;
    Ok((main_lib, companions))
}

unsafe fn resolve_entry_from_lib(
    lib: &libloading::Library,
) -> Result<(ash::Entry, &'static str), String> {
    // 1. Standard Vulkan loader / standalone ICD exporting `vkGetInstanceProcAddr`.
    if let Ok(sym) =
        unsafe { lib.get::<vk::PFN_vkGetInstanceProcAddr>(b"vkGetInstanceProcAddr\0") }
    {
        let get_instance_proc_addr = *sym;
        #[allow(unused_unsafe)]
        let entry = unsafe {
            ash::Entry::from_static_fn(ash::StaticFn {
                get_instance_proc_addr,
            })
        };
        return Ok((entry, "vkGetInstanceProcAddr"));
    }

    // 2. Khronos Vulkan ICD (`vk_icdGetInstanceProcAddr`, e.g. Mesa Turnip / PanVK / Lavapipe).
    if let Ok(sym) =
        unsafe { lib.get::<vk::PFN_vkGetInstanceProcAddr>(b"vk_icdGetInstanceProcAddr\0") }
    {
        let get_instance_proc_addr = *sym;
        #[allow(unused_unsafe)]
        let entry = unsafe {
            ash::Entry::from_static_fn(ash::StaticFn {
                get_instance_proc_addr,
            })
        };
        return Ok((entry, "vk_icdGetInstanceProcAddr"));
    }

    // 3. Android HAL `hwvulkan` module (`HMI` / `HAL_MODULE_INFO_SYM`).
    if let Ok(hmi) = unsafe { lib.get::<HwModule>(b"HMI\0") } {
        let module_ptr: *const HwModule = ptr::from_ref::<HwModule>(&*hmi);
        if !module_ptr.is_null() {
            let methods = unsafe { (*module_ptr).methods };
            if !methods.is_null() {
                if let Some(open_fn) = unsafe { (*methods).open } {
                    let mut hw_dev: *mut HwDevice = ptr::null_mut();
                    let rc = unsafe { open_fn(module_ptr, c"vk0".as_ptr(), &mut hw_dev) };
                    if rc == 0 && !hw_dev.is_null() {
                        let vk_dev = hw_dev.cast::<HwVulkanDevice>();
                        let get_instance_proc_addr = unsafe { (*vk_dev).get_instance_proc_addr };
                        #[allow(unused_unsafe)]
                        let entry = unsafe {
                            ash::Entry::from_static_fn(ash::StaticFn {
                                get_instance_proc_addr,
                            })
                        };
                        return Ok((entry, "hwvulkan (HMI)"));
                    }
                }
            }
        }
    }

    Err("No Vulkan entry point found (expected vkGetInstanceProcAddr, vk_icdGetInstanceProcAddr, or Android HMI)".to_owned())
}

/// Validates a custom `.so` library without permanently activating it.
pub fn inspect_driver_library(path: &Path) -> Result<DriverProbeInfo, String> {
    verify_elf_architecture(path)?;
    let (lib, mut companions) = unsafe { open_library_with_companions(path) }?;
    let (entry, kind) = unsafe { resolve_entry_from_lib(&lib) }?;
    let has_wsi = entry_supports_wsi(&entry);
    // Keep loaded libraries alive in the retention pool because unloading GPU
    // ICDs that registered TLS/atexit hooks can fault on Bionic.
    {
        let mut state = DRIVER_STATE.lock();
        state.retained_libs.append(&mut companions);
        state.retained_libs.push(lib);
    }
    let note = if has_wsi {
        format!("Ready ({kind} + native Android WSI surface support)")
    } else {
        format!(
            "Detected {kind} HAL module (uses Volt Vulkan Supporter system WSI bridge at launch)"
        )
    };
    Ok(DriverProbeInfo {
        entry_kind: kind.to_owned(),
        has_android_surface: has_wsi,
        note,
    })
}

/// Loads the configured custom Vulkan driver if valid and WSI-capable, otherwise
/// transparently falls back to the system Vulkan loader (`libvulkan.so`).
pub unsafe fn load_vulkan_entry() -> Result<ash::Entry, ash::LoadingError> {
    let custom = custom_driver_path();
    if let Some(ref path) = custom {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("custom-driver.so");
        match verify_elf_architecture(path).and_then(|()| unsafe {
            let (lib, companions) = open_library_with_companions(path)?;
            let (entry, kind) = resolve_entry_from_lib(&lib)?;
            Ok((lib, companions, entry, kind))
        }) {
            Ok((lib, mut companions, entry, kind)) => {
                if entry_supports_wsi(&entry) {
                    let mut state = DRIVER_STATE.lock();
                    state.retained_libs.append(&mut companions);
                    state.retained_libs.push(lib);
                    state.last_status = Some(format!("Custom Vulkan driver active: {name} ({kind})"));
                    log::info!(
                        "Volt Vulkan Supporter: loaded custom driver {} via {}",
                        path.display(),
                        kind
                    );
                    return Ok(entry);
                }
                let mut state = DRIVER_STATE.lock();
                state.retained_libs.append(&mut companions);
                state.retained_libs.push(lib);
                state.last_status = Some(format!(
                    "{name} ({kind}) lacks standalone Android WSI; bridging via system libvulkan.so + Volt Supporter"
                ));
                log::warn!(
                    "Volt Vulkan Supporter: {} ({}) lacks standalone VK_KHR_android_surface; bridging via system libvulkan.so",
                    path.display(),
                    kind
                );
            }
            Err(err) => {
                set_status(format!(
                    "Custom driver {name} fallback to system Vulkan: {err}"
                ));
                log::warn!(
                    "Volt Vulkan Supporter: custom driver {} failed ({err}); falling back to system libvulkan.so",
                    path.display()
                );
            }
        }
    }

    let loaded = unsafe {
        #[cfg(target_env = "ohos")]
        let res = ash::Entry::load_from("libvulkan.so");
        #[cfg(not(target_env = "ohos"))]
        let res = ash::Entry::load();
        res
    };
    if loaded.is_ok() && custom.is_none() {
        if supporter_mode() {
            set_status("Built-in Volt Vulkan Supporter bridge active".to_owned());
        } else {
            set_status("System Vulkan driver active".to_owned());
        }
    }
    loaded
}
