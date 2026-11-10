# `.hnmod` on Android: decision and evidence

## Decision

**Android never executes downloaded native code. Ever.** Native mods reach
the Android client only by being compiled into the APK at build time (the
weave pipeline); everything else a user can download at runtime is data
(`.hmod` content mods, shader packs, resource packs).

## Why

1. **Platform reality.** An `.hnmod` is Rust source compiled against a
   specific engine revision (upstream documents the format at
   playhorizon.dev/mods: "compiled-in, Java never runs"). There is no
   stable ABI to dlopen against, and Android has no compiler on device.
2. **Security.** Executing downloaded native code would void every
   guarantee the content validators give (`docs/COMPAT-AUDIT.md` §2) and
   would also violate Play-store-style policy on dynamic code loading.
3. **Hook-surface drift.** Upstream `NATIVE_HOOKS_API` is 80 (PC nightly
   411); this fork exposes 40. A downloaded `.hnmod` built for upstream
   would not even link. The weave pipeline pins each mod by SHA-256 and
   verifies its hooks against the fork (`tools/native_mods/compat.py`,
   `check_hooks`) before any of its code is built.

## The strongest route within that constraint (what we ship)

| Route | Status | Evidence |
|---|---|---|
| Weave pipeline: 36 registry `.hnmod`s compiled into both Android ABIs (voicechat removed on owner request) | **Shipped** | `WEAVE_SET` in `.github/workflows/android-apk.yml`; `WOVEN` in `crates/horizon-client-render/src/bundled_native.rs`; CI verifies the expected set (`PY_VERIFY`). |
| Bundled official mods (Zoom, FPS Overlay, Keystrokes, Minimap) | **Shipped** | `BUNDLED` in `bundled_native.rs`; bridges in `game/official_*_bridge.rs`. |
| In-client built-ins (NEA, Subtle Effects, Player Animations, plus the earlier Carry Block/AppleSkin/etc. set; Punchy moved to a vendored woven `.hnmod`) | **Shipped** | `game/volt_builtins.rs` and `docs/BUILTIN-PARITY.md`. |
| Content-only `.hmod` import on device | **Shipped** | `content_files.rs` `Kind::Mods` accepts manifest + content; proven in CI by the real `validate()` test on `biomes_unseen.hmod`. |
| Runtime download-and-run of `.hnmod` on Android | **Rejected by design** | See above. |

## Per-platform can/cannot table

| Capability | PC | Android |
|---|---|---|
| Run `.hmod` WebAssembly mods | Yes | Yes |
| Import content-only `.hmod` | Yes | Yes (Import button) |
| Run woven/bundled native mods | Yes (compiled in) | Yes (compiled in, both ABIs) |
| Add a new `.hnmod` without rebuilding | Yes, by building from source against the engine (developer workflow) | **No** — requires the next APK build; by design |
| Shader packs (data-only) | Yes | Yes |
| JVM/Java mods (`.jar`, `.mrpack`) | No (out of scope for Horizon) | No |

Adding a native mod to the Android set is a pull request: pin the release
hash in `WEAVE_SET`, add the `WOVEN` row, and let CI prove the weave. This
is intentionally a build-time step, not a runtime one.

## Grand build #12: data-driven hook expansion (wave 1)

The fork cannot port 375 upstream hook-API levels in one pass, so coverage
grows from evidence: `.github/workflows/native-mod-probe.yml` builds every
registry candidate against the real engine on both ABIs and the failures
name exactly what is missing. Wave 1 (9 candidates, pinned snapshot
`63daa89`):

| Candidate | Probe verdict | Action |
|---|---|---|
| armorhud 1.0.0 | Green, both ABIs | **Promoted** into `WEAVE_SET` |
| lookat 0.2.1 | Green, both ABIs | **Promoted** into `WEAVE_SET` |
| blockscope 0.3.0 | Green, both ABIs | **Promoted** into `WEAVE_SET` |
| elytratrails 0.1.2 | One missing target: `native_visuals::native_frame_visuals` | **Hook ported** (upstream API 46 line path, `game/native_visuals.rs`), probe green on both ABIs, **promoted** into `WEAVE_SET` |
| chat_animation 0.1.0 | Upstream's old ten-argument `ui::slider` | **Shimmed** to `ui::slider_dragging`, probe green; stays probe-only — its raw targets are render internals the signing path correctly refuses |
| treefall 0.3.0 | Calls `destroy_progress` as `&self` (its pinned rev had it so) | Needs a read-only attribute value path; next wave |
| freecam 0.1.0 | `E_UNSUPPORTED_CFG` on the fork's `mapped_key` (android cfg in body) | Needs a cfg-free refactor of `mapped_key`; next wave |
| movement_status 0.1.0 | Its pinned mixin compiler fails to install in the probe | Infra, not hooks; investigating |
| basic_hud 0.5.0 | Non-hook transformations; needs its exact pinned Horizon source | Not unlockable by hook porting |

Bundles that pin a Horizon revision without a hook-API level are now
accepted when — and only when — **every** injection verifies against the
served hook manifest (`native-hooks.json`): named hooks by (name, version),
explicit targets by exact match against served hook points. Engine
internals stay closed; the mixin compiler's strict AST matching still
judges every application (`tools/native_mods/compat.py::check_hooks`,
tested in `test_compat.py`).
