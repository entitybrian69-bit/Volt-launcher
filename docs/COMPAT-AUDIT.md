# Volt Launcher mod-compatibility audit

An audit of every path a mod can take into the client, every point where one
can be rejected, and what each rejection says. File references are to this
repository at the commit that adds this document.

## 1. Loading paths

### 1.1 `.hmod` packages (WebAssembly and content mods)

| Stage | File | What it does |
|---|---|---|
| Container read | `crates/horizon-mods/src/manifest.rs` (`Package::read`) | Opens the zip, reads `manifest.toml` via `Manifest::parse_tolerant`, extracts the optional `mod.wasm` and the `content/` tree. |
| Manifest parse | `crates/horizon-mods/src/manifest.rs` (`Manifest::parse_tolerant`) | Tolerant TOML parse: known fields are `id`, `name`, `version`, `description`, `authors`, `api`, `side`, `capabilities`, `namespaces`, `experimental`; unknown fields are stripped with a warning instead of failing. |
| Content index | `crates/horizon-mods/src/index.rs`, `crates/horizon-mods/src/content.rs` (`Content::check`) | Indexes `content/<kind>/...` files; validates namespace ownership and JSON schemas; unknown content kinds are skipped with a warning, never an error. |
| Import gate (Android) | `crates/horizon-client/src/content_files.rs` (`validate`, `Kind::Mods` arm, ~line 160) | A package imports as a mod when it has a manifest and **either** `mod.wasm` **or** content files. Content-only `.hmod`s (like `biomes_unseen.hmod`) are accepted. |
| Runtime load | `crates/horizon-client/src/game/mods.rs` | Starts WebAssembly mods, collects content mods, and writes the per-mod load report. |
| Content build | `crates/horizon-client/src/mod_content.rs` (`build`) | Materialises content registries (blocks, items, biomes, worldgen, sounds, creative entries) into the engine's prepared root. |

### 1.2 Native mods (`.hnmod`)

| Stage | File | What it does |
|---|---|---|
| Registry index | `crates/horizon-client/src/index/native.rs` | Schema-1 index of native mods (id, version, SHA-256, source). |
| Bundled/woven set | `crates/horizon-client-render/src/bundled_native.rs` | `BUNDLED` (4 official mods) and `WOVEN` (35 woven mods: 34 registry mods incl. `accurate_block_placement`, plus the vendored `punchy` 0.1.2) compiled into the Android client by the weave pipeline. |
| Weave pipeline | `.github/workflows/android-apk.yml` (`WEAVE_SET`), `tools/native_mods/compat.py` | CI fetches each pinned `.hnmod` by hash, checks its hooks against the fork (`check_hooks`, line ~348), applies `FORK_SOURCE_SHIMS` (lines ~240–305), and compiles the sources into the client. `tools/native_mods/build_set.py` (line ~67) rejects untrusted mixin pins. |
| In-client built-ins | `crates/horizon-client/src/game/volt_builtins.rs` + library modules | Four mods compiled directly into the client (see `docs/BUILTIN-PARITY.md`); no download, no weave. |

Android never loads native code at runtime; see `docs/HNMOD-DECISION.md`.

## 2. Rejection points and their messages

`.hmod` structural validation is unchanged by the tolerant loader; these are
the hard failures:

| # | Where | Condition | Outcome |
|---|---|---|---|
| 1 | `manifest.rs` | Missing `manifest.toml` | Error: package is not a mod. |
| 2 | `manifest.rs` | `id` fails the id regex | Error. |
| 3 | `manifest.rs` | `api` not in the supported list | Tolerated for content-only packages (capabilities ⊆ {Registries, Storage}, side ≠ Server); otherwise error. A future-api package with `mod.wasm` loads content-only: the wasm is dropped **with a warning**, never a crash. |
| 4 | `manifest.rs` | `capabilities` includes Registries with api < 0.2, or Server side on api 0.3 constraints | Error (API negotiation). |
| 5 | `manifest.rs` | `mod.wasm` over 32 MiB | Error. |
| 6 | `manifest.rs` | Content-only package without the Registries capability | Error. |
| 7 | `content.rs` (`check`) | Content file outside the mod's `namespaces` | Error (namespace ownership). |
| 8 | `content.rs` (`check`) | Malformed JSON / schema violation in a known registry file | Error. |
| 9 | `content.rs` (`check`) | Caps exceeded: 65 536 files, 16 MiB per file, 512 MiB total | Error. |
| 10 | `mod_content.rs` (~line 540) | A block's `sounds` id is not a registered block | Error at world preparation (this is what broke Biomes Unseen v1.0.1's `minecraft:grass` sound id; fixed in v1.0.2). |
| 11 | `mod_content.rs` (~line 434) | Block template missing/unknown | Error at world preparation. |

**Warnings (never fatal):** unknown manifest fields, unknown content kinds,
future-api wasm dropped. All of them surface per mod in the Mods screen and
in the plain-text report `volt-mods-report.txt` (written by
`crates/horizon-client/src/game/mods.rs`, format
`mod <id> (<file>): warning: <text>`), stored next to the game's options
file; the exact path is printed in the report header.

## 3. Shader pipeline inventory

Data-only; no shader pack can execute code.

| Piece | File |
|---|---|
| Pack discovery/selection | `crates/horizon-client/src/game/shader_packs.rs`, `crates/horizon-client-render/src/shader_pack_list.rs` |
| Android catalog + UI | `crates/horizon-client/src/android/shader_catalog.rs`, `crates/horizon-client-render/src/shader_options.rs` |
| WGSL shader API | `crates/horizon-shader-api` (validated by `cargo test -p horizon-shader-api` in CI) |
| Built-in Volt shaders | `crates/horizon-shaders` |
| Native shader hook | `crates/horizon-client/src/game/native_shaders.rs` |

Ours-vs-upstream gaps are in `docs/SHADER-AUDIT.md`.

## 4. Built-in mod patterns

Three patterns exist; new built-ins should pick the cheapest that fits:

1. **Woven `.hnmod`** (35 mods): 34 registry pins in `WEAVE_SET` plus the vendored `punchy` (`VENDORED_SET`, rebuilt deterministically from `tools/native_mods/vendored/punchy/` by `pack_vendored.py` and pinned by sha256 in `vendored/punchy.toml`); registry pins are a hash in `WEAVE_SET`
   (android-apk.yml) + `WOVEN` row (`bundled_native.rs`) + the workflow's
   `PY_VERIFY` expected set. Rows render automatically in the Mods screen
   (`crates/horizon-client/src/android/launcher_pages.rs`, ~line 2697).
2. **Bundled official source** (Zoom, HUD mods): logic module + a
   `bundled_<x>_*` bridge (`official_zoom_bridge.rs` pattern) called from
   `game/native_hooks.rs`.
3. **In-client built-in** (NEA, Subtle Effects, Player Animations):
   logic in a library module with atomics + unit tests, one shared bridge
   `game/volt_builtins.rs` for rows/settings/keys/persistence, per-frame
   feed beside `native_frame` in `game/extras.rs`. Works identically on PC
   and Android because nothing is platform-gated.
