# Shader pipeline audit: this fork vs upstream

Verified against the real upstream sources (github.com/horizon-rust/horizon
and github.com/starling-client/starling) in November 2026 — not against the
earlier assumption that upstream had removed packs pending a rewrite. That
assumption was **wrong**: upstream's Shader API 0.2 is back, with pack
support (0.2.43 tonemap work, 0.2.49 identity pass, 0.2.54 modpack
shaderpacks), `shader_ui.rs`, the `golden_hour_0_2` pack, additional
translate passes, `core_shader.rs` and `textures.rs`.

## Ours

| Piece | File(s) | State |
|---|---|---|
| Volt Shaders (built-in, WGSL) | `crates/horizon-shaders`, `crates/horizon-shader-api` | Shipped; validated in CI (`cargo test -p horizon-shader-api`). Off by default, Potato→Custom presets, Moving Vegetation, BSL/Complementary-like look. |
| Pack folder + selection UI | `game/shader_packs.rs`, `crates/horizon-client-render/src/shader_pack_list.rs`, `android/shader_catalog.rs`, `crates/horizon-client-render/src/shader_options.rs` | Shipped on Android and PC; same folder semantics. |
| Iris-format zip reading | `game/shader_packs.rs` | Shipped (data-only translation to our pipeline; no execution). |
| Native shader hook | `game/native_shaders.rs` | Shipped. |
| Failure handling | render pipeline | A pack that fails to compile falls back to the default renderer; the client does not crash. |

## Upstream (horizon-rust/horizon)

| Piece | Present upstream | Present here | Gap assessment |
|---|---|---|---|
| Shader API 0.2 | Yes | Yes (WGSL `horizon-shader-api`) | Compatible intent, divergent engine revs (hooks 411 vs 40) — wholesale merge is **not** safe; cherry-picks only. |
| `shader_ui.rs` screen | Yes | Our own selection + settings screens | Ours stays (Volt UI directives). |
| `golden_hour_0_2` pack | Yes | Not imported | Candidate cherry-pick; data-only, low risk. Not yet imported — listed as outstanding. |
| Extra translate passes (~10) | Yes | Partially (our pipeline covers the common ones) | Audit-first, partial merge; the engine divergence makes blind ports risky. Outstanding. |
| `core_shader.rs` / `textures.rs` refactors | Yes | No | Structural upstream refactors; adopting them would churn our render crate for no user-visible gain right now. Deliberately skipped. |

## starling-client/starling

Branch `android` exists; its shader state has **not** been examined in
depth. Unverified — listed honestly as such.

## Policy (unchanged)

- Shader packs are data-only. There is no execution path in a pack and
  none will be added.
- Compile/link failures surface per-shader in the UI and the log, then
  fall back to the default renderer. Never a crash.
- Engine validation is never weakened to make a pack load.
