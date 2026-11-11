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
| ABIs | Host target | `arm64-v8a`, `armeabi-v7a`, `x86_64` and `x86`, all built and released by CI | Device coverage incl. emulators, ChromeOS and x86 handhelds (Starling/Horizon-Android parity). The weave runs for every ABI; the 32-bit ABIs carry no `mods` feature flag difference in the woven set. |
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

## Starling / Horizon-Android performance parity (2026-11)

Audit of the Starling `android` branch and the Horizon Android fork
against this repository; every item behaves as the original does.

| Item | State here | Notes |
|---|---|---|
| Native-window buffer scaling | Present (`android/display.rs`, 25-200 % scaler) | Adopted upstream's zero-size view clamp and its unconditional input-scale write, so touches stay mapped during fold/rotate resizes. |
| Per-frame upload budgets | Present (`gfx/upload_budget.rs`) | Same 2-8 ms / 8 MiB envelope. |
| ADPF performance hints | Present (`android/device_runtime.rs`) | Same hint lifecycle as HA 0.7.5. |
| Chunk-packet per-frame budget | Present | Same 4 ms/frame policy as the HA fork. |
| GPU cost by measurement | Present (`gfx/timing.rs` timestamp queries, read back late) | Backs the same measured-not-guessed tuning HA 0.7.5 shipped. |
| Indirect-draw offset handling | Present | Bypass/fix equivalent to HA 0.7.7. |
| Vulkan separate opaque/cutout | Present (cutout path in `gfx`) | HA's split was measured here too (`gpu_times`); kept where it wins. |
| Deferred GL world-renderer init | Not applicable | This engine is Vulkan-only by directive; the deferred init is a GL-backend fix. |
| Serious server support | Present | Vanilla-brand handshake, chat, skins/capes; same protocol scope as upstream. |
| Shader packs | Present (data-only packs, `docs/SHADER-AUDIT.md`) | Policy stricter than upstream: no downloaded native code. |
| Phone-compiled native mods | Intentionally different | No on-device compiler by policy (`docs/HNMOD-DECISION.md`); mods are woven into the APK by CI instead. |
| `immersive` default | Ours: on; upstream: off | Upstream turned it off over device-specific freeze reports; Volt keeps the immersive full-screen default and exposes the toggle in Settings. |
