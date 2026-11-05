// Horizon's official FPS counter, formerly the WebAssembly mod and before
// that a built-in extra: the game's frame counter (frames in the last second,
// the slowest frame and the 1% low of the last 5 seconds) in a box in a
// corner, drawn as the extra drew it. Its settings are the WebAssembly mod's
// files (`horizon-mods/.horizon-mod-data/fps_overlay/corner`, `lows`), so a
// player keeps them. This file is both the mod's Cargo library (its tests)
// and the module the bundle adds to the game.

/// The mod's id (its settings folder), name and version.
pub const ID: &str = "fps_overlay";
pub const NAME: &str = "FPS Counter";
pub const VERSION: &str = "2.0.0";
pub const DESCRIPTION: &str = "Frames per second in a corner of the screen, with the slowest frame and the 1% low if you like. Horizon's own.";
/// The extra's box: translucent black, white text with a shadow.
pub const BACKGROUND: u32 = 0x7800_0000;
pub const INK: u32 = 0xFF_FFFF;
/// The corners, as the settings files name them and the page labels them.
pub const CORNERS: [(&str, &str); 4] = [
    ("top_left", "Top left"),
    ("top_right", "Top right"),
    ("bottom_left", "Bottom left"),
    ("bottom_right", "Bottom right"),
];

/// The settings, as the WebAssembly FPS Counter kept them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Index in `CORNERS`.
    pub corner: usize,
    /// Show the slowest frame and the 1% low too.
    pub lows: bool,
}

impl Settings {
    /// From the stored values; the extra's defaults (top left, no lows) for
    /// the ones missing.
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let corner = get("corner")
            .and_then(|c| CORNERS.iter().position(|k| k.0 == c.trim()))
            .unwrap_or(0);
        let lows = get("lows").is_some_and(|v| v.trim() == "true");
        Self { corner, lows }
    }
    pub fn labels(&self) -> Vec<String> {
        let on = |b: bool| if b { "ON" } else { "OFF" };
        vec![
            format!("Corner: {}", CORNERS[self.corner].1),
            format!("Minimum and 1% low: {}", on(self.lows)),
        ]
    }
    pub fn click(&mut self, i: usize) {
        match i {
            0 => self.corner = (self.corner + 1) % CORNERS.len(),
            1 => self.lows = !self.lows,
            _ => {}
        }
    }
    pub fn stored(&self) -> [(&'static str, String); 2] {
        let lows = if self.lows { "true" } else { "false" };
        [
            ("corner", CORNERS[self.corner].0.to_owned()),
            ("lows", lows.to_owned()),
        ]
    }
    /// The box's text for a reading (fps, min, 1% low).
    pub fn text(&self, (fps, min, low): (u32, u32, u32)) -> String {
        if self.lows {
            format!("{fps} fps  min {min}  1% {low}")
        } else {
            format!("{fps} fps")
        }
    }
}

thread_local! {
    /// The settings, read once from the files, then kept.
    static SETTINGS: std::cell::Cell<Option<Settings>> = const { std::cell::Cell::new(None) };
}

/// Clear profile-local cached settings when a game starts.
pub fn reset() { SETTINGS.with(|s| s.set(None)); }

/// The settings, read once through `get` (one file per key).
pub fn settings(get: impl Fn(&str) -> Option<String>) -> Settings {
    SETTINGS.with(|s| {
        let found = s.get().unwrap_or_else(|| Settings::read(get));
        s.set(Some(found));
        found
    })
}

/// Replaces the settings (the settings page changed them).
pub fn set_settings(settings: Settings) {
    SETTINGS.with(|s| s.set(Some(settings)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        |key| pairs.iter().find(|p| p.0 == key).map(|p| p.1.to_owned())
    }

    #[test]
    fn reads_and_writes_the_webassembly_mods_settings() {
        let s = Settings::read(stored(&[]));
        assert_eq!(
            s,
            Settings {
                corner: 0,
                lows: false
            }
        );
        assert_eq!(s.text((60, 40, 30)), "60 fps");
        let mut s = Settings::read(stored(&[("corner", "bottom_right\n"), ("lows", "true")]));
        assert_eq!(
            s,
            Settings {
                corner: 3,
                lows: true
            }
        );
        assert_eq!(s.text((60, 40, 30)), "60 fps  min 40  1% 30");
        assert_eq!(
            s.labels(),
            ["Corner: Bottom right", "Minimum and 1% low: ON"]
        );
        s.click(0);
        s.click(1);
        assert_eq!(
            s.stored(),
            [("corner", "top_left".into()), ("lows", "false".into())]
        );
    }
}
