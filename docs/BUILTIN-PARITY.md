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
| Torch/candle flame sparks | DONE | `flame_spark` beside every vanilla torch/candle flame spawn (incl. soul/copper torches via the same arm). |
| Campfire embers | DONE | `campfire_sparks` over lit campfires, beyond vanilla's lava pops. |
| Glowstone dust | DONE | Gold `dust` particles drifting off sampled glowstone faces. |
| Redstone-block dust | DONE | Red `dust` from `minecraft:redstone_block`. (Upstream also covers ores; ours is exact-block only to keep the animate-tick lookup cheap — ADAPTED in scope.) |
| Burning-entity sparks | DONE | Embers + smoke within the mob's bounding size, mobs within 32 blocks, tick rate. |
| Slime trails | DONE | `item_slime` blobs under moving grounded slimes. |
| Particle density/culling controls | ADAPTED | One Low/Medium/High density setting scaling all spawn odds; the engine's own `max_particles` budget does the culling. |
| Sleeping Z's, frosty breath, drowning bubbles | UNSUPPORTED | Need custom sprites (no custom assets by policy). |
| Allay/vex magic, shear fluff, item-rarity beams | UNSUPPORTED | Custom sprites/shaders. |
| Heartbeat at ≤3 hearts | UNSUPPORTED | Needs a sound asset. |
| Falling-block landing poof, egg splat, drip splash extras | UNSUPPORTED | Need block-entity/projectile event hooks; deferred rather than half-done. |
| Night-vision fade, biome rain colour, composter/anvil effects, flame geysers, `/subtle_effects` command | UNSUPPORTED | Out of scope for a first release; the command adds nothing over the settings page. |

## 4. Punchy → `punchy.rs` (+ matrix hook in `game.rs` first-person path)

**Upstream identity:** Punchy! by PunchyDevGuy (All Rights Reserved).
**Implementation:** clean-room procedural state machine; one extra matrix
on the vanilla first-person hand transform.

| Upstream feature | Status | Notes |
|---|---|---|
| Livelier first-person hand motion (procedural) | DONE | Walk/sprint sway, idle breathing, jump drift — damped, bounded, intensity setting. |
| Landing impact | DONE | Spring-damped dip scaled by fall speed. |
| Varied consecutive attack animations | ADAPTED | Three-variant rotation offsets over the vanilla swing rather than fully re-authored swing animations (the vanilla swing path stays untouched by directive). |
| Visible hands / off-hand | ADAPTED | Optional "show empty off hand" (off by default); upstream always shows both. |
| 3D in-hand item models with physics | UNSUPPORTED | Our item renderer draws baked item models; physics-simulated items need a renderer feature, not a mod. |
| Two-handed spear, water-exit shake, climb/crawl first-person animations | UNSUPPORTED | Each needs state the first-person path doesn't expose yet; deferred. |
| Triggerable keybind animations | ADAPTED | Covered by the Player Animations built-in's gesture key (shared API) rather than duplicated here. |
| Camera motion | UNSUPPORTED | By performance directive, camera shake stays out. |

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
| Third-person playback | DONE | On the local player's body. |
| First-person playback | UNSUPPORTED | Our first-person hands are matrix-posed, not keyframe-rigged; Punchy covers first-person motion. |
| Playback on other players | UNSUPPORTED | Would need a network channel; no protocol changes by directive. |
| API for other mods | DONE | `play_keyframes(index, now)` is public to the other built-ins. |
| Built-in gestures + keybind | DONE | Wave, Point, Salute, Nod, Shake head, Clap; gesture key + selection in settings. (This part is ours; upstream ships no gestures.) |

---

## Test coverage

Each logic module has unit tests compiled into `horizon-client --lib` and
run by the CI worldgen-test job filter: `nea::tests`,
`player_gestures::tests`, `subtle_effects::tests`, `punchy::tests` — pose
math, gating flags, keyframe sampling, spring/landing behaviour, settings
round-trips. The woven ABP mod is verified by the weave job's expected-set
check.
