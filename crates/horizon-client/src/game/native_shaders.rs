//! Horizon's own shader packs in the game (docs/shaders.md): the pick in `shaderpacks/` (a
//! folder with `horizon-shaders.toml`, or a `.hshader`; `game/shader_packs.rs`) runs once it
//! passes `horizon_shader_api::check`. The pack is looked at twice a second: a saved file is checked and the pack rebuilt, and a
//! pack that fails is refused with its problems in the chat while the last good one keeps
//! drawing.

use std::{
    hash::{Hash, Hasher},
    path::Path,
    time::{Duration, Instant},
};

use horizon_client::text::Text;
use horizon_shader_api::{Pack, check};

use super::Game;

const LOOK_EVERY: Duration = Duration::from_millis(500);
const SLIDER_REBUILD_DEBOUNCE: Duration = Duration::from_millis(100);
const RED: u32 = 0xFF5555;

/// Synthetic `picked` identifier used when built-in Volt Shaders are enabled.
pub(super) const VOLT_SHADERS_PICKED: &str = "__volt_shaders__";

#[derive(Default)]
pub(super) struct NativeShaders {
    /// The pack's name in `shaderpacks/`, of either kind (`game/shader_packs.rs`); empty: none.
    pub picked: String,
    /// The pack's files when it was last read (`fingerprint`).
    stamp: Option<u64>,
    looked: Option<Instant>,
    /// Active `VoltShadersConfig::fingerprint()` currently compiled on the GPU.
    pub volt_fingerprint: Option<u64>,
    /// Last instant a Volt Shaders specialization pipeline was rebuilt.
    pub volt_built_at: Option<Instant>,
    /// Whether the current resource pack's sprite-layer wave/water table is uploaded.
    pub volt_layers_uploaded: bool,
}

impl NativeShaders {
    pub fn picking(name: &str) -> Self {
        Self {
            picked: name.to_owned(),
            ..Self::default()
        }
    }
}

impl Game<'_> {
    /// Builds the pack at `path` when its files changed since it was last read.
    pub(super) fn tick_native_shader_pack(&mut self, path: &Path) {
        let now = Instant::now();
        let state = &mut self.native_shaders;
        if state.looked.is_some_and(|t| now - t < LOOK_EVERY) {
            return;
        }
        state.looked = Some(now);
        let stamp = fingerprint(path);
        if stamp == state.stamp {
            return;
        }
        let reload = state.stamp.is_some();
        state.stamp = stamp;
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        let result = Pack::read(path).map_err(|e| vec![e]).and_then(|pack| {
            let report = check(&pack)
                .map_err(|p| p.iter().map(ToString::to_string).collect::<Vec<String>>())?;
            // The player's values (the pack's Shader Options page), else the defaults.
            let options = &pack.manifest.options;
            let values = horizon_client::shader_pack_list::load_values(path, options);
            let constants = options.iter().map(|o| o.id.clone()).zip(values).collect();
            let labels = renderer
                .set_native_shader_pack(Some((report, constants)))
                .map_err(|e| vec![e])?;
            Ok((pack.manifest, labels))
        });
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        match result {
            Ok((m, labels)) => {
                log::info!(
                    "shader pack {} {}: {} in {} ms",
                    m.id,
                    m.version,
                    labels.join(", "),
                    now.elapsed().as_millis()
                );
                if reload {
                    self.system_message(Text::literal(format!("Shader pack {} reloaded", m.name)));
                }
            }
            Err(problems) => {
                log::warn!("shader pack {name} refused: {}", problems.join("; "));
                self.system_message(Text::colored(format!("Shader pack {name} refused:"), RED));
                for p in problems {
                    self.system_message(Text::colored(p, RED));
                }
            }
        }
    }

    /// Compiles and updates the built-in native WGSL Volt Shaders pack and 4-bit sprite-layer
    /// wave/water lookup table whenever `settings.extras.volt_shaders` or resource packs change.
    pub(super) fn tick_volt_shaders(&mut self) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        if !self.native_shaders.volt_layers_uploaded {
            let kinds = horizon_shader_api::volt_shaders::pack_sprite_layer_kinds(
                self.pack
                    .render
                    .sprite_layers
                    .iter()
                    .map(|(name, &layer)| (name.as_str(), layer)),
            );
            renderer.set_native_layer_kinds(&kinds);
            self.native_shaders.volt_layers_uploaded = true;
        }
        let cfg = &self.settings.extras.volt_shaders;
        let fp = cfg.fingerprint();
        if self.native_shaders.volt_fingerprint == Some(fp) {
            return;
        }
        let now = Instant::now();
        if self.dragging
            && self
                .native_shaders
                .volt_built_at
                .is_some_and(|last| now.saturating_duration_since(last) < SLIDER_REBUILD_DEBOUNCE)
        {
            return;
        }
        let started = Instant::now();
        match cfg
            .build_runtime()
            .and_then(|(prepared, constants)| renderer.set_native_shader_pack(Some((prepared, constants))))
        {
            Ok(labels) => {
                self.native_shaders.volt_fingerprint = Some(fp);
                self.native_shaders.volt_built_at = Some(now);
                log::info!(
                    "Volt Shaders ({}): {} in {} ms",
                    cfg.profile.label(),
                    labels.join(", "),
                    started.elapsed().as_millis()
                );
            }
            Err(err) => {
                self.native_shaders.volt_fingerprint = Some(fp);
                self.native_shaders.volt_built_at = Some(now);
                log::warn!("Volt Shaders build failed: {err}");
            }
        }
    }
}

/// A hash of the names, sizes and modification times of the pack's files and of its saved\n/// option values (`None` when the pack is unreadable).
fn fingerprint(path: &Path) -> Option<u64> {
    fn walk(path: &Path, hasher: &mut std::hash::DefaultHasher) -> Option<()> {
        let meta = std::fs::metadata(path).ok()?;
        path.hash(hasher);
        if meta.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(path)
                .ok()?
                .flatten()
                .map(|e| e.path())
                .collect();
            entries.sort();
            for entry in entries {
                walk(&entry, hasher)?;
            }
        } else {
            meta.len().hash(hasher);
            meta.modified().ok()?.hash(hasher);
        }
        Some(())
    }
    let mut hasher = std::hash::DefaultHasher::new();
    walk(path, &mut hasher)?;
    // The player's option values, when there are any.
    let values = horizon_client::shader_pack_list::values_path(path);
    let _ = walk(&values, &mut hasher);
    Some(hasher.finish())
}
