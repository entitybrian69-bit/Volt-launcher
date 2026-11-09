# PC ↔ Android parity

What is identical across platforms, what differs, and why. File references
are to this repository.

## Shared by construction (same code, no platform gates)

| Area | Where | Notes |
|---|---|---|
| `.hmod` loaders and validation | `crates/horizon-mods/*` | One crate for both platforms; identical errors and warnings. |
| Content build into the prepared root | `crates/horizon-client/src/mod_content.rs` | Same registries, same caps, same failure text. |
| Import gate | `crates/horizon-client/src/content_files.rs` | Same `validate()` used by the Android importer and by CI's host-side test on `biomes_unseen.hmod`. |
| Per-mod load report | `crates/horizon-client/src/game/mods.rs` | `volt-mods-report.txt` beside the options file on both platforms. |
| In-client built-ins | `crates/horizon-client/src/{mobs/nea.rs, mobs/player_gestures.rs, subtle_effects.rs}` + `game/volt_builtins.rs` | Deliberately **not** `#[cfg(target_os = "android")]`-gated, unlike the older bundled-zoom bridge: rows, settings, keys and persistence are the same on PC. |
| Shader packs (data-only) and the shaderpacks folder | `game/shader_packs.rs`, `shader_pack_list.rs` | Same formats and selection logic. |
| Engine validation (never weakened) | throughout | Identical on both platforms. |

## Platform differences (each with a reason)

| Difference | PC | Android | Why |
|---|---|---|---|
| Native-mod acquisition | Build from source (developer workflow) | Compiled into the APK only (35 woven + 4 bundled + built-ins) | No on-device compiler; no dynamic native code by policy (`docs/HNMOD-DECISION.md`). |
| ABIs | Host target | `arm64-v8a` **and** `armeabi-v7a`, both built and released by CI | Device coverage. The weave runs for both ABIs; v7a carries no `mods` feature flag difference in the woven set. |
| Punchy (first-person animations) | **Absent** — desktop/test builds carry only the no-op hooks (`native_render_hand`, `native_use_animation`, `item_use::use_pose`) | Woven in as the vendored `punchy-rust-0.1.2.hnmod` | The port is Android-weave-only; nothing downloads or executes at runtime on either platform. |
| Bundled official Zoom/HUD bridges | No-op stubs | Active | Those mods' assets ship in the APK; PC users build them natively. The newer `volt_builtins.rs` pattern removes this asymmetry for all future built-ins. |
| Renderer backend | Vulkan (no OpenGL fallback, by directive) | Vulkan via the loader, Adreno/Mali handling | Device constraints (Naga/WGSL limits, `SPRITE_ARRAYS = 8`). |
| Capability scaling | Full | Resolution scaler 25–200 %, particle budget clamp when `ema_frame_ms > 28` (`game/extras.rs`), LOD and Volt Boost options | Thermal and GPU limits on Mali-G57-class devices. The built-ins honour the same budget: Subtle Effects has a density setting and all four are zero-cost when off. |

## Device capability detection

- Frame-time EMA drives the particle clamp and thermal behaviour
  (`game/extras.rs`).
- Resolution scaling and upscaling ratios are user-visible settings with
  safe defaults; shader quality presets choose by device class.
- All fallbacks degrade visuals only; none change validation or gameplay.
