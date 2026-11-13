//! HDR metadata for the swapchain: the Vulkan calls the game crate, which
//! forbids unsafe code, can't make itself. wgpu sets no HDR metadata, and
//! without it a compositor may clip or tone-map everything above 1.0 (80 nits)
//! of an scRGB swapchain.

#[cfg(target_os = "linux")]
pub mod wayland;

/// Opens a Vulkan device with `VK_EXT_hdr_metadata` too, when the GPU offers
/// it, so [`set_metadata`] can tell the compositor the real peak brightness.
/// `None` leaves opening to wgpu.
#[cfg(not(target_os = "macos"))]
pub fn open_vulkan_device(
    adapter: &wgpu::Adapter,
    desc: &wgpu::DeviceDescriptor<'_>,
) -> Option<Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>> {
    if vulkan_supporter_mode() {
        return None;
    }
    use wgpu::hal::api::Vulkan;
    let open = {
        // Safety: only read and opened from here; dropped before wgpu takes it back.
        let hal = unsafe { adapter.as_hal::<Vulkan>() }?;
        let instance = hal.shared_instance().raw_instance();
        // Safety: the adapter's own physical device on its own instance.
        let offered =
            unsafe { instance.enumerate_device_extension_properties(hal.raw_physical_device()) }
                .ok()?
                .iter()
                .any(|e| e.extension_name_as_c_str() == Ok(ash::ext::hdr_metadata::NAME));
        if !offered {
            return None;
        }
        // Safety: what wgpu's own request_device does, with one more extension.
        unsafe {
            hal.open_with_callback(
                desc.required_features,
                &desc.required_limits,
                &desc.memory_hints,
                Some(Box::new(
                    |args: wgpu::hal::vulkan::CreateDeviceCallbackArgs| {
                        args.extensions.push(ash::ext::hdr_metadata::NAME);
                    },
                )),
            )
        }
        .ok()?
    };
    // Safety: opened from this adapter with exactly `desc`'s features.
    Some(unsafe { adapter.create_device_from_hal(open, desc) })
}

/// Tells the compositor, through `VK_EXT_hdr_metadata`, that the swapchain's
/// content is BT.709, peaks at `peak` nits and is no darker than `black`. Needs a
/// device from `open_vulkan_device` and a configured surface; does nothing otherwise.
pub fn set_metadata(
    device: &wgpu::Device,
    surface: &wgpu::Surface<'_>,
    paper_white: f32,
    peak: f32,
    black: f32,
) {
    #[cfg(not(target_os = "macos"))]
    {
        use wgpu::hal::api::Vulkan;
        // Safety: read-only access to handles wgpu keeps alive while these guards live.
        let (Some(hal_device), Some(hal_surface)) =
            (unsafe { device.as_hal::<Vulkan>() }, unsafe {
                surface.as_hal::<Vulkan>()
            })
        else {
            return;
        };
        if !hal_device
            .enabled_device_extensions()
            .contains(&ash::ext::hdr_metadata::NAME)
        {
            return;
        }
        let Some(swapchain) = hal_surface.raw_native_swapchain() else {
            return;
        };
        let xy = |x, y| ash::vk::XYColorEXT { x, y };
        let metadata = ash::vk::HdrMetadataEXT::default()
            .display_primary_red(xy(0.64, 0.33))
            .display_primary_green(xy(0.30, 0.60))
            .display_primary_blue(xy(0.15, 0.06))
            .white_point(xy(0.3127, 0.3290))
            .max_luminance(peak)
            .min_luminance(black)
            .max_content_light_level(peak)
            .max_frame_average_light_level(paper_white.min(peak));
        let loader = ash::ext::hdr_metadata::Device::new(
            hal_device.shared_instance().raw_instance(),
            hal_device.raw_device(),
        );
        // Safety: a live swapchain of this device, one metadata entry for it.
        unsafe { loader.set_hdr_metadata(&[swapchain], &[metadata]) };
    }
    #[cfg(target_os = "macos")]
    let _ = (device, surface, paper_white, peak, black);
}

/// Summary returned when validating an imported custom Vulkan `.so` driver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomVulkanDriverInfo {
    pub entry_kind: String,
    pub has_android_surface: bool,
    pub note: String,
}

/// Enables or disables the built-in Volt Vulkan Supporter compatibility bridge.
pub fn set_vulkan_supporter_mode(enabled: bool) {
    #[cfg(not(target_os = "macos"))]
    wgpu::hal::vulkan::volt_supporter::set_supporter_mode(enabled);
    #[cfg(target_os = "macos")]
    let _ = enabled;
}

/// Returns whether the built-in Volt Vulkan Supporter compatibility bridge is active.
pub fn vulkan_supporter_mode() -> bool {
    #[cfg(not(target_os = "macos"))]
    {
        wgpu::hal::vulkan::volt_supporter::supporter_mode()
    }
    #[cfg(target_os = "macos")]
    {
        false
    }
}

/// Sets or clears an optional custom Vulkan `.so` driver library path.
pub fn set_custom_vulkan_driver_path(path: Option<std::path::PathBuf>) {
    #[cfg(not(target_os = "macos"))]
    wgpu::hal::vulkan::volt_supporter::set_custom_driver_path(path);
    #[cfg(target_os = "macos")]
    let _ = path;
}

/// Returns the active custom Vulkan `.so` driver library path, if configured.
pub fn custom_vulkan_driver_path() -> Option<std::path::PathBuf> {
    #[cfg(not(target_os = "macos"))]
    {
        wgpu::hal::vulkan::volt_supporter::custom_driver_path()
    }
    #[cfg(target_os = "macos")]
    {
        None
    }
}

/// Returns the human-readable status from the latest Vulkan loader initialization.
pub fn last_vulkan_driver_status() -> Option<String> {
    #[cfg(not(target_os = "macos"))]
    {
        wgpu::hal::vulkan::volt_supporter::last_driver_status()
    }
    #[cfg(target_os = "macos")]
    {
        None
    }
}

/// Checks ELF header magic and target CPU architecture before loading a custom `.so`.
pub fn verify_custom_vulkan_driver_elf(path: &std::path::Path) -> Result<(), String> {
    #[cfg(not(target_os = "macos"))]
    {
        wgpu::hal::vulkan::volt_supporter::verify_elf_architecture(path)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = path;
        Err("Custom Vulkan drivers are not supported on macOS".to_owned())
    }
}

/// Probes a custom `.so` library for Vulkan ICD / HAL entry points.
pub fn inspect_custom_vulkan_driver(
    path: &std::path::Path,
) -> Result<CustomVulkanDriverInfo, String> {
    #[cfg(not(target_os = "macos"))]
    {
        let info = wgpu::hal::vulkan::volt_supporter::inspect_driver_library(path)?;
        Ok(CustomVulkanDriverInfo {
            entry_kind: info.entry_kind,
            has_android_surface: info.has_android_surface,
            note: info.note,
        })
    }
    #[cfg(target_os = "macos")]
    {
        let _ = path;
        Err("Custom Vulkan drivers are not supported on macOS".to_owned())
    }
}

