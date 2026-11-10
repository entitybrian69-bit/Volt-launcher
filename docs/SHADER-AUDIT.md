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

## Volt Shaders 2.1 overhaul (owner bug report, grand build #11)

Owner report: Potato / "Super Low" render a full-white screen on device
(Mali-G57), the grade is "way too yellowish", and water/realism fall
short. Target look confirmed by the owner: **Complementary Reimagined**.

What changed:

- **Executed-value proof.** `crates/horizon-client-render/tests/volt_shader_exec.rs`
  compiles the shipped sky + material stages of every profile exactly as
  the renderer does and executes them on a host GPU (CI: lavapipe) over
  six scenes (noon, sunset, night, storm, cave, far-lands) and ten
  surface classes. It asserts no NaN/blow-out, that zero-post profiles
  stay inside 0..1, and that no scene comes back near-white — the
  white-screen class of bug is now machine-checked every run, and the
  per-profile channel means are printed as CI annotations.
- **NaN scrub.** Both stages clamp their colour (Vulkan `clamp` drops
  NaN) before the inline grade; a NaN on any path used to be able to
  paint white/garbage pixels on mobile GPUs.
- **Neutral daylight.** The permanent warm-sun white balance is gone:
  daylight is neutral (Complementary Reimagined), and the light only
  turns golden near the horizon (`golden_hour` term on the sun height).
  All preset `warmth` values are 0 and every preset now defaults to the
  Complementary Crisp style (BSL Soft stays selectable, softened).
- **Water.** Deep azure Complementary palette, clearer body (lower
  alpha), reflections track the actual sky colour with full Fresnel
  weight at grazing angles, and the sun glint is tighter (pow 120).
