//! Android wiring of the pinned official Horizon FPS, Keystrokes and Minimap.
use super::{Game, native_hooks::{NativeMod, NativeMinimap}};
use horizon_client::{hud::HudBatch, extras_hud::Corners};

#[cfg(target_os = "android")]
impl Game<'_> {
    fn bundled_hud_active(&self, id: &str) -> bool {
        self.native_on(id) && self.settings.mods.installed.iter().any(|m| m.id == id && m.native.is_some())
    }
    pub(super) fn bundled_hud_list(&self, mods: &mut Vec<NativeMod>) {
        use super::{official_fps_overlay as fps, official_keystrokes as keys, official_minimap as map};
        fps::reset(); keys::reset(); map::reset();
        let Some(home) = Self::options_path().parent().map(std::path::Path::to_path_buf) else { return; };
        for (id, name, version, description) in [
            (fps::ID, fps::NAME, fps::VERSION, fps::DESCRIPTION),
            (keys::ID, keys::NAME, keys::VERSION, keys::DESCRIPTION),
            (map::ID, map::NAME, map::VERSION, map::DESCRIPTION),
        ] {
            if horizon_client::bundled_native::installed(&home, id) && !mods.iter().any(|m| m.id == id) {
                mods.push(NativeMod { id: id.into(), title: format!("{name} {version}"),
                    description: description.into(), keys: Vec::new() });
            }
        }
    }
    pub(super) fn bundled_hud_settings(&self, id: &str, buttons: &mut Vec<String>) {
        use super::{official_fps_overlay as fps, official_keystrokes as keys, official_minimap as map};
        match id {
            fps::ID => buttons.extend(fps::settings(|k| self.native_setting(id, k)).labels()),
            keys::ID => buttons.extend(keys::settings(|k| self.native_setting(id, k)).labels()),
            map::ID => buttons.extend(map::settings(|k| self.native_setting(id, k)).labels()),
            _ => {}
        }
    }
    pub(super) fn bundled_hud_click(&mut self, id: &str, button: usize) {
        use super::{official_fps_overlay as fps, official_keystrokes as keys, official_minimap as map};
        match id {
            fps::ID => {
                let mut s = fps::settings(|k| self.native_setting(id, k));
                s.click(button); fps::set_settings(s);
                for (k, v) in s.stored() { self.native_set_setting(id, k, &v); }
            }
            keys::ID => {
                let mut s = keys::settings(|k| self.native_setting(id, k));
                s.click(button); keys::set_settings(s);
                for (k, v) in s.stored() { self.native_set_setting(id, k, &v); }
            }
            map::ID => {
                let mut s = map::settings(|k| self.native_setting(id, k));
                s.click(button); map::set_settings(s);
                for (k, v) in s.stored() { self.native_set_setting(id, k, &v); }
            }
            _ => {}
        }
    }
    pub(super) fn bundled_minimap(&self, result: &mut Option<NativeMinimap>) {
        use super::official_minimap as map;
        use horizon_client::extras::Corner;
        if result.is_some() || !self.bundled_hud_active(map::ID) { return; }
        let s = map::settings(|k| self.native_setting(map::ID, k));
        let (size, blocks_per_pixel) = s.view();
        *result = Some(NativeMinimap {
            corner: [Corner::TopLeft, Corner::TopRight, Corner::BottomLeft, Corner::BottomRight][s.corner],
            size, blocks_per_pixel, rotate: s.rotate, round: s.round,
        });
    }
    pub(super) fn bundled_hud_draw(&mut self, batch: &mut HudBatch, corners: &mut Corners) {
        use horizon_client::extras::Corner;
        use super::{official_fps_overlay as fps, official_keystrokes as keys};
        let corner_of = |i| [Corner::TopLeft, Corner::TopRight, Corner::BottomLeft, Corner::BottomRight][i];
        if self.bundled_hud_active(fps::ID) {
            let s = fps::settings(|k| self.native_setting(fps::ID, k));
            let corner = corner_of(s.corner);
            if self.native_corner_shown(corner) {
                let text = s.text(self.native_frame_rate());
                let w = self.native_text_width(&text) + 4.0;
                let (x, y) = corners.place(corner, w, 12.0);
                self.native_fill(batch, [x, y, w, 12.0], fps::BACKGROUND);
                self.native_text(batch, &text, [x + 2.0, y + 2.0], fps::INK, true);
            }
        }
        if self.bundled_hud_active(keys::ID) {
            let s = keys::settings(|k| self.native_setting(keys::ID, k));
            let corner = corner_of(s.corner);
            if self.native_corner_shown(corner) {
                let mut down = |name: &str| self.native_mapping(name).unwrap_or_default();
                let view = keys::View { moves: keys::MOVES.map(&mut down), jump: down("key.jump").0,
                    attack: down("key.attack").0, use_: down("key.use").0, cps: None };
                let view = keys::View { cps: s.cps.then(|| self.native_clicks()), ..view };
                let (w, h) = keys::SIZE;
                let (x, y) = corners.place(corner, w, h);
                for shape in keys::shapes(&view, &|t| self.native_text_width(t)) {
                    match shape {
                        keys::Shape::Fill([dx, dy, w, h], argb) => self.native_fill(batch, [x + dx, y + dy, w, h], argb),
                        keys::Shape::Text(text, [dx, dy], rgb, shadow) => self.native_text(batch, &text, [x + dx, y + dy], rgb, shadow),
                    }
                }
            }
        }
    }
}
#[cfg(not(target_os = "android"))]
impl Game<'_> {
    pub(super) fn bundled_hud_list(&self, _: &mut Vec<NativeMod>) {}
    pub(super) fn bundled_hud_settings(&self, _: &str, _: &mut Vec<String>) {}
    pub(super) fn bundled_hud_click(&mut self, _: &str, _: usize) {}
    pub(super) fn bundled_minimap(&self, _: &mut Option<NativeMinimap>) {}
    pub(super) fn bundled_hud_draw(&mut self, _: &mut HudBatch, _: &mut Corners) {}
}
