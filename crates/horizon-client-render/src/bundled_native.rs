//! Activation of reviewed native code already compiled into the Android APK.
//! This is not a generic `.hnmod` loader and never executes downloaded code.
use std::{fs, io, path::Path};

/// The only registry bundle compiled into the current Android APK.
pub fn is_bundled_zoom(id: &str, version: &str, sha256: &str) -> bool {
    id == "zoom" && version == "2.0.0"
        && sha256 == "49fa2d7d635225c53304fbf1477606c0677eb23d94dac75b730f6c3ba065f127"
}

const ZOOM_MARKER: &str = "volt-bundled-zoom-2.0.0.installed";

/// The marker belongs to this profile and this exact bundled implementation.
pub fn zoom_installed(home: &Path) -> bool {
    home.join(super::mod_list::NATIVE_DIR).join(ZOOM_MARKER).is_file()
}

/// Installs or removes the bundled Zoom for the selected profile. Settings
/// remain in place when removing it. Removal never deletes an `.hnmod` file.
pub fn set_zoom_installed(home: &Path, install: bool) -> io::Result<()> {
    let dir = home.join(super::mod_list::NATIVE_DIR);
    let marker = dir.join(ZOOM_MARKER);
    if !install {
        return match fs::remove_file(marker) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        };
    }
    fs::create_dir_all(home)?;
    fs::create_dir_all(&dir)?;
    // Preserve unknown lines as well as every other mod's settings and keys.
    let options = home.join(super::mod_list::FILE);
    let old = match fs::read_to_string(&options) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };
    let mut updated = String::new();
    for line in old.lines() {
        if line.split_once(':').is_some_and(|(k, _)| k.trim() == "mod.zoom") {
            continue;
        }
        updated.push_str(line);
        updated.push('\n');
    }
    updated.push_str("mod.zoom:true\n");
    let options_tmp = home.join("horizon-mods.txt.volt-partial");
    fs::write(&options_tmp, updated)?;
    fs::rename(options_tmp, options)?;
    let temp = dir.join("volt-bundled-zoom-2.0.0.partial");
    fs::write(&temp, b"Horizon Zoom 2.0.0; bundled in Volt APK\n")?;
    fs::rename(temp, marker)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prebuilt_match_is_exact() {
        let hash = "49fa2d7d635225c53304fbf1477606c0677eb23d94dac75b730f6c3ba065f127";
        assert!(is_bundled_zoom("zoom", "2.0.0", hash));
        assert!(!is_bundled_zoom("zoom", "2.0.1", hash));
        assert!(!is_bundled_zoom("other", "2.0.0", hash));
        assert!(!is_bundled_zoom("zoom", "2.0.0", "bad"));
    }
    #[test]
    fn profile_install_remove_preserves_other_mods_and_keys() {
        let dir = std::env::temp_dir().join(format!("volt-bundled-zoom-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(super::super::mod_list::FILE),
            "mod.other:true\nunknown:keep me\nmod.zoom:false\nkey.zoom.zoom:key.keyboard.x\n").unwrap();
        assert!(!zoom_installed(&dir));
        set_zoom_installed(&dir, true).unwrap();
        assert!(zoom_installed(&dir));
        let list = super::super::mod_list::ModList::load(&dir);
        assert!(fs::read_to_string(dir.join(super::super::mod_list::FILE)).unwrap().contains("unknown:keep me"));
        assert!(list.enabled.contains("other") && list.enabled.contains("zoom"));
        assert!(!list.disabled.contains("zoom"));
        assert_eq!(list.keys["zoom"]["zoom"], "key.keyboard.x");
        set_zoom_installed(&dir, false).unwrap();
        set_zoom_installed(&dir, false).unwrap();
        assert!(!zoom_installed(&dir));
        assert_eq!(super::super::mod_list::ModList::load(&dir), list);
        fs::remove_dir_all(&dir).unwrap();
    }
}
