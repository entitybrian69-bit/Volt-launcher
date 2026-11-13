//! Android integration of the bundled official Horizon Zoom source.
//! Hook-point definitions stay untouched so external mixins remain hookable.
use super::{Game, native_hooks::{NativeMod, NativeZoom}};

#[cfg(target_os = "android")]
impl Game<'_> {
    fn bundled_zoom_active(&self) -> bool {
        use super::official_zoom as zoom;
        self.settings.mods.installed.iter().any(|m| m.id == zoom::ID && m.native.is_some())
            && self.native_on(zoom::ID)
    }
    pub(super) fn bundled_zoom_list(&self, mods: &mut Vec<NativeMod>) {
        use super::official_zoom as zoom;
        zoom::reset();
        if Self::options_path().parent().is_some_and(horizon_client::bundled_native::zoom_installed)
            && !mods.iter().any(|m| m.id == zoom::ID)
        {
            mods.push(NativeMod {
                id: zoom::ID.into(), title: format!("Horizon Zoom {}", zoom::VERSION),
                description: zoom::DESCRIPTION.into(),
                keys: vec![[zoom::KEY.into(), "Zoom key".into(), zoom::DEFAULT_KEY.into()]],
            });
        }
    }
    pub(super) fn bundled_zoom_key(&mut self, key: &str, pressed: bool) {
        use super::official_zoom as zoom;
        if !self.bundled_zoom_active() && (pressed || zoom::view().is_none()) { return; }
        if key != self.native_mod_key(zoom::ID, zoom::KEY, zoom::DEFAULT_KEY) { return; }
        let playing = self.native_playing();
        let game = &*self;
        zoom::key(playing, pressed, |name| game.native_setting(zoom::ID, name));
    }
    pub(super) fn bundled_zoom_wheel(&self, steps: f32, claimed: &mut bool) {
        if !*claimed && self.bundled_zoom_active() && super::official_zoom::wheel(steps) {
            *claimed = true;
        }
    }
    pub(super) fn bundled_zoom_view(&self, result: &mut Option<NativeZoom>) {
        if result.is_none() && self.bundled_zoom_active()
            && let Some((factor, smooth, turn)) = super::official_zoom::view()
        { *result = Some(NativeZoom { factor, smooth, turn }); }
    }
    pub(super) fn bundled_zoom_settings(&self, id: &str, buttons: &mut Vec<String>) {
        use super::official_zoom as zoom;
        if id == zoom::ID {
            buttons.extend(zoom::settings(|key| self.native_setting(zoom::ID, key)).labels());
        }
    }
    pub(super) fn bundled_zoom_click(&mut self, id: &str, button: usize) {
        use super::official_zoom as zoom;
        if id == zoom::ID {
            let mut settings = zoom::settings(|key| self.native_setting(zoom::ID, key));
            settings.click(button);
            zoom::set_settings(settings);
            for (key, value) in settings.stored() { self.native_set_setting(zoom::ID, key, &value); }
        }
    }
}

#[cfg(not(target_os = "android"))]
impl Game<'_> {
    pub(super) fn bundled_zoom_list(&self, _: &mut Vec<NativeMod>) {}
    pub(super) fn bundled_zoom_key(&mut self, _: &str, _: bool) {}
    pub(super) fn bundled_zoom_wheel(&self, _: f32, _: &mut bool) {}
    pub(super) fn bundled_zoom_view(&self, _: &mut Option<NativeZoom>) {}
    pub(super) fn bundled_zoom_settings(&self, _: &str, _: &mut Vec<String>) {}
    pub(super) fn bundled_zoom_click(&mut self, _: &str, _: usize) {}
}
