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
| Weave pipeline: 34 registry `.hnmod`s compiled into both Android ABIs | **Shipped** | `WEAVE_SET` in `.github/workflows/android-apk.yml`; `WOVEN` in `crates/horizon-client-render/src/bundled_native.rs`; CI verifies the expected set (`PY_VERIFY`). |
| Bundled official mods (Zoom, FPS Overlay, Keystrokes, Minimap) | **Shipped** | `BUNDLED` in `bundled_native.rs`; bridges in `game/official_*_bridge.rs`. |
| In-client built-ins (NEA, Subtle Effects, Punchy, Player Animations, plus the earlier Carry Block/AppleSkin/etc. set) | **Shipped** | `game/volt_builtins.rs` and `docs/BUILTIN-PARITY.md`. |
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
