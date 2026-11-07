//! Activation of reviewed native code already compiled into the Android APK.
//! This is not a generic `.hnmod` loader and never executes downloaded code.
use std::{fs, io, path::Path};

/// Exact source bundles whose code is compiled into the current Android APK.
pub const BUNDLED: &[(&str, &str, &str)] = &[
    ("zoom", "2.0.0", "49fa2d7d635225c53304fbf1477606c0677eb23d94dac75b730f6c3ba065f127"),
    ("fps_overlay", "2.0.0", "bedb670e2490ce8239906c86eabd3a0c131ef5037b2937607231bdbdba38c171"),
    ("keystrokes", "2.0.0", "bed34d2f8049e92c52cd7c91c0c4c8c9d165faaf0183500cf4021d9e04783a7d"),
    ("minimap", "2.0.0", "6f5dbca0699562b088a04b3f5abdc1b7faf8da51257a96da0a4ed639c6644ed9"),
];

/// Official registry mods woven into the engine library at APK build time
/// (`.github/workflows/android-apk.yml`, `weave` job). They register on the
/// in-game Mods page through `native_mods` and are on until the player turns
/// them off (`mod_list::ModList::disabled`); nothing is installed per profile.
pub const WOVEN: &[(&str, &str)] = &[
    ("jei", "0.1.0"),
    ("horizon_physics", "0.15.1"),
    ("jade", "26.3.5"),
    ("veinminer", "0.3.2"),
    ("appleskin", "1.0.0"),
    ("status_effect_hud", "1.0.0"),
    ("toggle_sprint", "1.0.0"),
    ("inventory_profiles", "1.0.0"),
    ("shulker_peek", "0.1.0"),
    ("mousetricks", "0.2.0"),
    ("itemnew", "0.1.1"),
    ("eatinganimation", "0.1.0"),
    ("enchantmenthints", "0.1.0"),
    ("light_overlay", "1.0.0"),
    ("right_click_crops", "1.0.0"),
    ("smartf3", "0.1.0"),
    ("armor_status", "1.0.0"),
    ("crosshair_indicator", "1.0.0"),
    ("hitbox_plus", "1.3.0"),
    ("invisible_frames", "0.1.1"),
    ("freelook", "0.3.1"),
    ("voicechat", "1.0.0"),
    ("worldedit", "1.2.1"),
    ("horizonmatica", "0.1.1"),
    ("visuality_horizon", "1.0.2"),
    ("held_item_info", "0.1.0"),
    ("carryblock", "0.1.2"),
    ("chatheads", "0.1.0"),
    ("fast_items", "0.1.0"),
    ("customhud", "1.6.0"),
    ("coords_hud", "2.0.0"),
    ("biomecompass", "0.2.2"),
    ("skins_3d", "1.1.0"),
];

pub fn is_bundled(id: &str, version: &str, sha256: &str) -> bool {
    BUNDLED.iter().any(|&(i, v, h)| (id, version, sha256) == (i, v, h))
}
pub fn supports(id: &str) -> bool { BUNDLED.iter().any(|&(i, _, _)| i == id) }
fn marker(home: &Path, id: &str) -> Option<std::path::PathBuf> {
    let (id, version, _) = BUNDLED.iter().find(|&&(i, _, _)| i == id)?;
    Some(home.join(super::mod_list::NATIVE_DIR).join(format!("volt-bundled-{id}-{version}.installed")))
}
/// The marker belongs to this profile and exact bundled implementation.
pub fn installed(home: &Path, id: &str) -> bool {
    marker(home, id).is_some_and(|p| p.is_file())
}
pub fn zoom_installed(home: &Path) -> bool { installed(home, "zoom") }
pub fn set_zoom_installed(home: &Path, install: bool) -> io::Result<()> {
    set_installed(home, "zoom", install)
}

/// Installs or removes only a compiled-in mod. User settings are retained.
pub fn set_installed(home: &Path, id: &str, install: bool) -> io::Result<()> {
    let marker = marker(home, id).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "unknown bundled mod"))?;
    let dir = marker.parent().expect("marker has a parent");
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
        if line.split_once(':').is_some_and(|(k, _)| k.trim().strip_prefix("mod.") == Some(id)) {
            continue;
        }
        updated.push_str(line);
        updated.push('\n');
    }
    updated.push_str(&format!("mod.{id}:true\n"));
    let options_tmp = home.join("horizon-mods.txt.volt-partial");
    fs::write(&options_tmp, updated)?;
    fs::rename(options_tmp, options)?;
    let temp = marker.with_extension("partial");
    fs::write(&temp, format!("Horizon {id}; bundled in Volt APK\n"))?;
    fs::rename(temp, marker)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prebuilt_match_is_exact() {
        let hash = "49fa2d7d635225c53304fbf1477606c0677eb23d94dac75b730f6c3ba065f127";
        assert!(is_bundled("zoom", "2.0.0", hash));
        assert!(!is_bundled("zoom", "2.0.1", hash));
        assert!(!is_bundled("other", "2.0.0", hash));
        assert!(!is_bundled("zoom", "2.0.0", "bad"));
        assert!(supports("fps_overlay"));
        assert!(!supports("../escape"));
        for &(id, version, hash) in BUNDLED {
            assert!(is_bundled(id, version, hash));
        }
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
        set_installed(&dir, "fps_overlay", true).unwrap();
        assert!(installed(&dir, "fps_overlay"));
        assert!(!installed(&dir, "minimap"));
        assert!(set_installed(&dir, "../escape", true).is_err());
        let list = super::super::mod_list::ModList::load(&dir);
        assert!(fs::read_to_string(dir.join(super::super::mod_list::FILE)).unwrap().contains("unknown:keep me"));
        assert!(list.enabled.contains("other") && list.enabled.contains("zoom") && list.enabled.contains("fps_overlay"));
        assert!(!list.disabled.contains("zoom"));
        assert_eq!(list.keys["zoom"]["zoom"], "key.keyboard.x");
        set_zoom_installed(&dir, false).unwrap();
        set_zoom_installed(&dir, false).unwrap();
        assert!(!zoom_installed(&dir));
        assert_eq!(super::super::mod_list::ModList::load(&dir), list);
        fs::remove_dir_all(&dir).unwrap();
    }
}
