# Built-in mod parity

Feature-by-feature status for the five built-ins added by the grand
upgrade, against the exact upstream mod each one recreates. Status values:

- **DONE** — behaviour recreated 1:1 within our engine's model.
- **ADAPTED** — the idea ships, with a documented deviation and the reason.
- **UNSUPPORTED** — not shipped, with the reason.

All five appear in Options > Mods with a toggle and a settings page, on PC
and Android alike; settings persist (per-key files under
`.horizon-mod-data/<id>/`) and apply live. Every built-in is zero-cost when
its row is toggled off (a suspended flag short-circuits all of its code
paths). Licensing details are in `THIRD_PARTY_NOTICES.md`.

---

## 1. Accurate Block Placement Reborn (woven native mod)

**Upstream identity:** `accurate_block_placement` 1.0.0 by Lutziano888
(MIT), the Horizon port of Accurate Block Placement: Reborn (MIT, by
Clayborn, Flourick, schwar and KadTheHunter). **Not a reimplementation** —
the real mod is woven into the Android client, pinned by SHA-256
`e631745f6d267360de6eb51995ee995bf1ab3d64393d287b7b93c332b85939ce`
(release tag `accurate_block_placement-1.0.0`).

| Feature | Status | Notes |
|---|---|---|
| Hold right-click places on each **new** aimed spot instantly | DONE | Upstream code, exact behaviour. |
| No repeat placement on the same spot | DONE | Upstream code. |
| Single click keeps the vanilla delay unless the view turns | DONE | Upstream code. |
| Backwards bridging, one block per step | DONE | Upstream code. |
| Interactable blocks unaffected | DONE | Upstream code. |
| Settings: on/off, toggle keybind (unbound default), "Allowed on servers" | DONE | Upstream settings page. |
| Android touch | DONE | Hold-to-place works through the remote input path (`remote.rs` `remote_handle_input`). |

## 2. Not Enough Animations → `mobs/nea.rs`

**Upstream identity:** Not Enough Animations by tr7zw (custom license — no
code used). **Implementation:** port of CayMac's MIT Horizon native mod
`not_enough_animations` (github.com/CayMac97/not-enough-animations), which
already recreated it for this engine. Applied after the vanilla humanoid
pose in `mobs/anim.rs::setup`.

| Upstream feature | Status | Notes |
|---|---|---|
| Eating/drinking third-person arm + chew | DONE | Ease-in over 4 ticks, chew oscillation, head nod; uses either hand; bow/crossbow/trident/shield/brush/spyglass/goat-horn/spear excluded. |
| Two-handed map holding | DONE | Two arms when the map is alone or mirrored, one otherwise; breathing idle. |
| Shield body rotation | UNSUPPORTED | The CayMac port does not carry it and our body-yaw pipeline derives from movement only; a correct port needs a new engine hook on body-yaw resolution. Deliberately skipped rather than shipped wrong. |
| Hide offhand during bow use | UNSUPPORTED | First-person feature overlapping Punchy's hand control; not in the CayMac port. |
| Boat rowing arms | DONE | Rowing cycle while the boat moves; local boat state fed per frame, other players use the passenger flag. |
| Horse riding rein hold | UNSUPPORTED | Needs mount-type data on remote mobs that the fork does not sync yet. |
| Compass/clock/spyglass hold poses | UNSUPPORTED | Not in the CayMac port; item-specific hold poses would conflict with vanilla's pose table. |
| Ladder/vine climbing hand-over-hand | DONE | Position-phased cycle, legs alternate; local climbable detection via the `minecraft:climbable` tag. |
| Crawling in tight spaces | UNSUPPORTED | Vanilla crawl pose already exists in the engine; the extra NEA polish is not in the CayMac port. |
| Smooth transitions between states | ADAPTED | Eating eases in/out; the other states switch per-frame like the CayMac port, without global cross-fade. |
| Per-animation toggles | DONE | Eating, maps, climbing, boats, plus a master switch and a toggle keybind. |

## 3. Subtle Effects → `subtle_effects.rs` (+ hooks in `animate_tick.rs`, bridge frame pass)

**Upstream identity:** Subtle Effects by MincraftEinstein (All Rights
Reserved). **Implementation:** clean-room from the documented feature list;
vanilla particle sprites only (constraint: features needing custom sprites
or sounds are unsupported by policy).

| Upstream feature | Status | Notes |
|---|---|---|
| Torch/candle flame sparks | DONE | `flame_spark` beside every vanilla torch/candle flame spawn (incl. soul/copper torches via the same arm): a 3-spark ember fan at 90% odds, so the rare animate-tick visits read as a clear burst (runtime-proven in CI: `subtle_effects_burst_in_the_live_animate_tick`). |
| Campfire embers | DONE | `campfire_sparks` over lit campfires: a 3-ember column plus a frequent arcing lava spark, beyond vanilla's pops. |
| Glowstone dust | DONE | Three gold `dust` motes per sampled tick drifting off glowstone faces (60% odds), a visible shimmer. |
| Redstone-block dust | DONE | Red `dust` from `minecraft:redstone_block`. (Upstream also covers ores; ours is exact-block only to keep the animate-tick lookup cheap — ADAPTED in scope.) |
| Burning-entity sparks | DONE | Embers + smoke within the mob's bounding size, mobs within 32 blocks, tick rate. |
| Slime trails | DONE | `item_slime` blobs under moving grounded slimes. |
| Particle density/culling controls | ADAPTED | One Low/Medium/High density setting scaling all spawn odds; the engine's own `max_particles` budget does the culling. |
| Sleeping Z's, frosty breath, drowning bubbles | UNSUPPORTED | Need custom sprites (no custom assets by policy). |
| Allay/vex magic, shear fluff, item-rarity beams | UNSUPPORTED | Custom sprites/shaders. |
| Heartbeat at ≤3 hearts | UNSUPPORTED | Needs a sound asset. |
| Falling-block landing poof, egg splat, drip splash extras | UNSUPPORTED | Need block-entity/projectile event hooks; deferred rather than half-done. |
| Night-vision fade, biome rain colour, composter/anvil effects, flame geysers, `/subtle_effects` command | UNSUPPORTED | Out of scope for a first release; the command adds nothing over the settings page. |

## 4. Punchy → vendored native mod (Android weave)

**Upstream identity:** Punchy! by PunchyDevGuy (All Rights Reserved);
Rust port by the Horizon community
([CihuyAkz/horizon-android-more-mods](https://github.com/CihuyAkz/horizon-android-more-mods),
`mods/official/punchy/`, MIT glue around the ARR engine).
**Implementation:** the earlier clean-room `punchy.rs` built-in was
removed. The real port is vendored at `tools/native_mods/vendored/punchy/`,
packed deterministically by `tools/native_mods/pack_vendored.py` into
`punchy-rust-0.1.2.hnmod`, and woven into Android builds exactly like the
registry set (verified extract, pinned mixins, single compiler pass). The
engine exposes the hooks the port binds to: `game/item_use.rs::use_pose`
and the `native_render_hand` / `native_use_animation` no-op hooks in
`game/native_hooks.rs`.

| Area | Status | Notes |
|---|---|---|
| First-person combat/movement/interaction animations | DONE (Android) | The port's own animation core: 223 clips, easing, crossfades, tuning template. |
| Eat/drink/bow/trident/brush hand poses | DONE | `use_pose` picks the pose from remaining/duration/partial ticks (unit tested). |
| Desktop runtime-proof build | ABSENT | The weave happens only in the Android APK job; the desktop test build carries the hooks but no Punchy code. |
| Port completeness | PARTIAL (upstream) | The port's own README marks itself a preview: editor, pack mixing, locomotion, physics and some renderers are not ported. |

## 5. Player Animations → `mobs/player_gestures.rs`

**Upstream identity:** playerAnimator by KosmX (MIT) — a keyframe animation
*library*, not an end-user feature mod. **Implementation:** our own
keyframe layer in its spirit (original keyframe data), plus a user-facing
gesture set so the library is usable out of the box.

| Library capability | Status | Notes |
|---|---|---|
| Keyframed per-part channels | DONE | `(time, radians)` keys per part axis. |
| Interpolation/easing | ADAPTED | Smoothstep easing between keys (playerAnimator also offers molang expressions — out of scope). |
| Layering over vanilla pose with clean entry/exit | DONE | Additive channels; every clip starts and ends at zero, so there is no snap (tested). |
| Third-person playback | DONE | On the local player's body; while a clip plays from first person the camera auto-borrows the front third-person view and restores it after (`volt_builtins_frame` + `still_playing`), so gestures are always visible. |
| First-person playback | UNSUPPORTED | Our first-person hands are matrix-posed, not keyframe-rigged; Punchy covers first-person motion. |
| Playback on other players | UNSUPPORTED | Would need a network channel; no protocol changes by directive. |
| API for other mods | DONE | `play_keyframes(index, now)` is public to the other built-ins. |
| Built-in gestures + keybind | DONE | Wave, Point, Salute, Nod, Shake head, Clap; gesture key + selection in settings. (This part is ours; upstream ships no gestures.) |

## 6. Overhauled Entities → `mobs/overhauled.rs` (+ `mobs/emf.rs`, `mobs/etf.rs`)

**Upstream identity:** inspired by FreshLX's Fresh Animations / Fresh
Moves resource packs (All Rights Reserved CEM keyframe data) and powered
in upstream Minecraft by Traben's Entity Model Features (EMF) and Entity
Texture Features (ETF). **Implementation:** fully clean-room native
procedural animation — no upstream CEM data, assets or code. One visible
mod row ("Overhauled Entities", default ON); the EMF/ETF counterparts are
internal engine layers with no row or settings anywhere, by design.

| Area | Status | Notes |
|---|---|---|
| Full-body walk overhaul (player + humanoids) | DONE | Eased double-frequency gait follow-through on arms/legs, body roll + sway, head counter-roll; additive over the vanilla pose, hitboxes untouched. |
| Idle life (all entities) | DONE | Breathing arms, layered-noise head wander, weight shifts; `ambient` fallback covers every living model via part-name no-op writes. |
| Zombie family reaches | DONE | Drifting raised-arm reach for Zombie/Drowned/Zombie Villager/Zombified Piglin and `Humanoid { zombie_arms }`. |
| Quadruped gait + grazing + tail flicks | DONE | Cows, pigs, sheep, bears, pandas, goats, hoglins, horses, llamas; gated slow graze dip while idle. |
| Tail wags (wolf/cat/fox) | DONE | Eager wag scaling with speed, lazy curl at rest. |
| Creeper tiptoe, spider leg waves, chicken pecks, villager nods, enderman sway | DONE | Per-family routines matched to each model's part names. |
| Per-entity personality (EMF/ETF layer) | DONE | Stable SplitMix64 phase/amplitude/tempo per entity id, so herds never sync (`etf::entity_random` avalanche-tested). |
| Keyframe-exact Fresh Animations clips | UNSUPPORTED | The upstream packs' CEM keyframes are ARR data; shipping them would violate the licensing directive. The procedural recreation targets the same visible behaviours. |
| ETF texture features (blinking, random skins, emissives) | UNSUPPORTED / already-vanilla | Need third-party texture assets (none ship by policy); emissive eyes are already vanilla-parity in `mobs/render.rs`. |
| Settings | DONE | Master toggle, player/mob switches, Subtle/Balanced/Dramatic intensity; all persisted per profile. |

---

## Test coverage

Each logic module has unit tests compiled into `horizon-client --lib` and
run by the CI worldgen-test job filter: `nea::tests`,
`player_gestures::tests`, `subtle_effects::tests`, `overhauled::tests`,
`emf::tests`, `etf::tests` — pose
math, gating flags, keyframe sampling, spring/landing behaviour, additive
rotation budgets, randomizer stability and settings
round-trips. The woven ABP mod is verified by the weave job's expected-set
check.

---

## Woven-mod behaviour note: Light Overlay

The woven `light_overlay` registry mod persists its `active` flag across
sessions, and its only toggle is the F7 key — which touch devices do not
have. A once-enabled overlay therefore came back on every launch as
"random X crosses on the blocks". Volt keeps the mod but makes the overlay
session-only, enforced at the read level: `native_hooks::native_setting`
returns "false" for `light_overlay`/`active` until the first in-session
write of that key, so no frame-order race can resurrect a stale flag. The
in-session toggle still works unchanged.
