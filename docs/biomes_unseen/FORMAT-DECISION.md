# Biomes Unseen: `.hmod` vs `.hnmod` — format decision

**Decision: ship Biomes Unseen as a `.hmod` content mod.** No `.hnmod` is
shipped: it adds nothing this mod needs, and on Android it could not even be
installed (evidence below). Volt Launcher is an Android launcher, so Android
support decides.

## What each format is, from this repository's source

### `.hmod` — a content package the game loads at runtime

- Discovery and loading: the game scans `<game home>/horizon-mods/*.hmod` at
  startup — `crates/horizon-client/src/game/mods.rs` (`content_root`, around
  line 191) over `crates/horizon-client-render/src/mod_list.rs` (`DIR =
  "horizon-mods"`, beside `options.txt`). On Android the game home is the
  app's working directory (`crates/horizon-client/src/android_main.rs`,
  `set_current_dir(&home)` around line 377), so the same path works there.
- Validation, never execution: `horizon_mods::Package::read` +
  `Content::read` (`crates/horizon-mods/src/content.rs`) check every file
  path, namespace and declaration (`data/<ns>/horizon/{block,item,mob_effect}`
  schemas, tags, assets). A `.hmod` carries **data only** — there is no code
  in it to run.
- What it can carry: blocks/items/mob effects from vanilla templates,
  tags, recipes-level data files, client assets (blockstates, models,
  textures, lang, `items/` client item definitions) and — since the
  worldgen-api patch — **registered biomes with climate placement**
  (`crates/horizon-client/src/mod_content.rs`: `WORLDGEN_BUNDLES` includes
  `worldgen/biome`, `ClimateEntry`/`append_climates` write the multi-noise
  preset tables in `worldgen/presets.json`; `append_creative` places mod
  items into `creative-inventory.json`).
- Android install UI exists today: the launcher's Mods page imports a local
  `.hmod` or downloads one over HTTPS (32 MiB cap, manifest-checked before
  install) into `<game>/horizon-mods/` —
  `crates/horizon-client/src/android/catalog.rs` (`import_native`, around
  line 743) and `crates/horizon-client/src/android/launcher_pages.rs`
  ("Import .hmod", "native .hmod mods run on Android", around lines
  1794–1904). The `horizon-mods` folder is also exposed as "Mods" through the
  Android document provider (`crates/horizon-client/src/android/document_store.rs:70`),
  so a file manager can drop the file in directly.

### `.hnmod` — a source-only native mixin bundle that must be compiled in

- "Source-only native mixin bundles. Reading a bundle never executes its
  code" — `crates/horizon-mods/src/index/native.rs:1`. The bundle holds Rust
  and TOML sources plus `mixins/<id>/data/` files only (same file, lines
  ~69–118); its manifest is `{schema: 1, id, version, game, horizon, mixins,
  hooks?}` pinning **exact engine revisions**. There is no slot for game
  registries, worldgen data or client assets: biomes cannot ride in a
  `.hnmod` at all.
- It runs only when the game binary was **rebuilt with it**:
  `crates/horizon-client-render/src/mod_list.rs` (`NATIVE_DIR`,
  `NOT_BUILT = "Installed, not built into this game: build the game with it
  in the launcher..."`).
- Android never compiles or executes downloaded code: "This is not a generic
  `.hnmod` loader and never executes downloaded code" —
  `crates/horizon-client-render/src/bundled_native.rs:2`. On Android a native
  mod works only when its exact source was **woven into the APK at CI build
  time** (the "Weave registry mods" jobs in
  `.github/workflows/android-apk.yml`); installing other `.hnmod` sources
  just reports "no compatible Volt APK build is available yet"
  (`crates/horizon-client/src/android/catalog.rs`, `install`).

## Comparison

| Criterion | `.hmod` | `.hnmod` |
|---|---|---|
| **Android support** | Loads at runtime on any Volt APK; import UI + document provider exist | Only if baked into the APK at CI time; user installation impossible |
| Biome / worldgen data | Yes (`WORLDGEN_BUNDLES`, climate placement, preset tables) | No — carries Rust/TOML sources only |
| Blocks, items, assets | Yes (validated declarations + client assets) | No |
| Native code hooks | No | Yes (the only thing it adds) |
| Stability / safety | Data-validated, nothing executes | Compiled against exact pinned engine revisions |
| Install steps | Download, import in launcher, enable | Rebuild and reinstall the whole game |

Biomes Unseen is 100% data: 44 biomes, 176 template blocks, 29 items, tags,
features and client assets. It needs **no native code hook**, so a `.hnmod`
would add nothing the `.hmod` cannot do — and would strictly lose Android
installability. Per the decision rule, only the `.hmod` ships, as the single
primary release artifact (`biomes_unseen.hmod` on the GitHub release, hashed
in `SHA256SUMS.txt`).

## Android install steps (the shipped flow)

1. On the device, open the GitHub release and copy the `biomes_unseen.hmod`
   asset link (or download the file).
2. Volt Launcher > **Mods** > paste the HTTPS URL and tap **Import .hmod**
   (or import the downloaded file; a file manager can also place it in the
   "Mods" folder the launcher exposes). The launcher verifies the package
   manifest before installing it to `<game>/horizon-mods/biomes_unseen.hmod`.
3. Enable **Biomes Unseen** in Installed (or in the game under Options >
   Mods — content mods are off until turned on, `mod_list.rs`), then create
   a new world. The APK itself does **not** bundle the mod.

## Verification

The `Worldgen mod tests (host)` CI job loads the real `biomes_unseen.hmod`
through `Package::read`/`Content::read` and proves: 44 biomes registered in
the multi-noise preset tables (39 `minecraft:overworld`, 5
`minecraft:the_nether`), every placement decoded, and — since round 10 —
every block and item carrying a blockstate/model/texture chain, an
`items/<name>.json` client item definition the engine decoder accepts, an
English name, and a seat in the creative category and search tabs
(`biomes_unseen_hmod_ships_textures_models_lang_and_creative_entries`).
