# Overhauled Entities — Parity Ledger

Mission: behavioral parity with **Actions & Stuff v2.0** (Oreville Studios,
Bedrock) as native Volt/Horizon content. Authored assets only — zero bytes
from the Oreville pack (C1). This file is the source of truth across
sessions.

## Session log / decisions

| Date | Decision / event | By |
|---|---|---|
| 2026-11-09 | Pre-contract baseline shipped: procedural "Overhauled Entities" built-in (humanoid gait/idle, zombie reach, quadruped graze, tail wags, creeper tiptoe, spider legs, chicken peck, villager, enderman, ambient; player+mob+intensity toggles). Release v0.9.9-build.ad7e04a. This baseline is NOT counted as A&S parity; it is the foundation T1/T2 build on. | agent |
| 2026-11-09 | Operating contract v1 received; mission redefined from "Fresh Animations-inspired" to "Actions & Stuff v2.0 behavioral parity". Phase 0 opened. | owner |
| 2026-11-09 | [OOB] Light Overlay stale-overlay fix v2 (read-level force, commit 5c18dad) — IMPLEMENTED-UNVERIFIED, awaiting owner device run. Not part of this mission; tracked in BUILTIN-PARITY.md. | agent |
| 2026-11-09 | PHASE 0 ACCEPTED (pending version-source citation + OOB tagging, both resolved below). Gate decisions: **Q1** scope = sections A–E; F splits into F-NOW (holstered weapons, sword stance, wolf moon-howl — rows added below, per-row decision at Phase 1, no implementation before Phase 1 sign-off) and F-BACKLOG (death-animation system, Witch/Strider reworks, placeable items/props/plushies, Vibrant Visuals, seasonal/Pale Garden — deferred). **Q2** defaults screenshot incoming; UNKNOWN defaults tied to BLOCKED rows marked DEFERRED-WITH-PARENT. **Q3** first person = extend Punchy; single FP owner, no parallel layer. **Q4** Moving Vegetation approved as NAMED SUBSTITUTION for the grass-sway row (D2). **Q5** A14 faces DEFER — no shader-hacked eyes (C2); B8 items/armour DEFER — needs separate scoping pass; C11 Wither SUBSTITUTE motion-only, label IMPLEMENTED-PARTIAL; C13 babies DEFER; E12 Morphs DEFER (model replacement + server grief vector). | owner |

| 2026-11-09 | **OWNER OVERRIDE**: "Do all the phase and work in one go. No stopping before the completion of every single task. Provide me apk." Logged as blanket sign-off: Phase 1 signed, T1+T2+T3 built in one pass, no per-tier gates, deliverable = release APK. Immutable constraints (C1–C6) unchanged. Results in "One-go build" section below. | owner |
| 2026-11-09 | PHASE 1 conditional acceptance processed. **Blocker resolved as (a)**: deliverable = compiled-in native row, not a distributable hmod — **accepted deviation from the original ask, reason: hmod format is data-only** (and `.hnmod` is source-form; see Packaging, citations added). Confirmations: hot-apply logged as NAMED DEVIATION (superset, approved); Volt-extra toggles (players/mobs/intensity) KEPT as labeled Volt extras (named deviation); item-QoL rides T3 (approved, no T4). F-NOW: FN1 holstered weapons DEFERRED (un-sized; reopen as scoped candidate only after v1 ships), FN2 sword stance APPROVED → T1, FN3 wolf moon-howl APPROVED → T2. One-pass audit of the official 1.5 source completed (corrections + new rows below). | owner |

## Named deviations (approved, per C2)

| # | Deviation | Approved |
|---|---|---|
| ND1 | Deliverable is a compiled-in native mod row, not a distributable hmod. Reason: hmod format is data-only; `.hnmod` is Rust source (not a loadable binary). | 2026-11-09 |
| ND2 | Hot-apply is instant (statics read per frame) — superset of the reference's apply-on-menu-close. | 2026-11-09 |
| ND3 | Volt-only extra toggles kept (`players`, `mobs`, `intensity`), clearly labeled as Volt extras — additions beyond the 1:1 mirrored set. | 2026-11-09 |
| ND4 | D2 grass sway satisfied by Volt Shaders Moving Vegetation (named substitution, Q4). | 2026-11-09 |

## Packaging note (for Phase 1, logged early per §8)

Animations require native code, and nothing downloadable can carry it on
this engine — cited (blocker resolution, option (a)):

1. **Spec:** `docs/HNMOD-DECISION.md` — "Android never executes
   downloaded native code. Ever." with the per-platform can/cannot table
   and the security rationale. This is Volt's recorded design decision.
2. **Format fact:** a `.hnmod` is Rust *source* compiled against a pinned
   engine revision (upstream: playhorizon.dev/mods, "compiled-in"); it is
   not a binary — there is nothing to `dlopen`, sideloaded or not. The
   engine contains no runtime native loader (grep `dlopen|libloading`
   across `crates/`: zero hits).
3. **Data-only `.hmod`:** `crates/horizon-mods/src/manifest.rs:33`
   (`#[serde(deny_unknown_fields)]` Manifest) — id/name/version/api/
   capabilities/namespaces + content files; no code-carrying field, and
   `content.rs` validation accepts asset/data kinds only.
4. **ABI drift:** the fork pins NATIVE_HOOKS_API 40 vs upstream 80 — even
   a hypothetical binary hnmod would not link
   (`tools/native_mods/compat.py` `check_hooks`).

So: the owner's (b) hypothesis (dlopen from app-private storage is
technically possible on sideloaded Android) is true of Android but moot
here — the mod format ships source, the engine has no loader, and the
spec forbids adding one. Deviation ND1 logged. Data-only parts (authored
variant textures) can still ship as `.hmod` content.

## Version mapping (RESOLVED, cited — anchors the table)

Official versioning is 1.x only; there is no official "2.0". The
fan-site "v2.0" headline feature set is **official Actions & Stuff 1.5**:

- Primary: Oreville's own guide, "Actions & Stuff 1.5" —
  <https://guides.orevillestudios.com/1-5-23fd70150598817e90c9f038f425f025>
  — lists Expressions (blinking, eyes, eyebrows), Happy Ghast/Ghastling,
  Diving/Cannon Ball/Belly Flop water entries, Boat Wakes, Wolf Laying &
  Cat Loafing, Wither & Glow Squid effect revamps.
- Primary: @OrevilleStudios announcement of the same 1.5 list —
  <https://x.com/OrevilleStudios/status/1940886133371527613>
- Changelog line 1.7 → 1.13 (1.13 = Nov 2, 2026), no 2.0 anywhere:
  <https://gleaks.pro/resources/actions-stuff.3422/updates>
- "v2.0" label appears only on APK/fan sites (actionandstuffs.com,
  actionsandstuff.org, actionandstuff.io), attached to the 1.5-era
  features above.

Scope per Q1: sections A–E (the 1.5-era set + core baseline), plus the
F-NOW rows below pending per-row decision.

## Phase 0 inventory

Status values: NOT-STARTED / IN-PROGRESS / IMPLEMENTED / VERIFIED / BLOCKED.
Verdicts: NATIVE (Horizon hook path exists today; procedural/keyframe work
only), EQUIVALENT-WORK (feasible; needs a new small engine hook and/or
authored assets), BLOCKED (Class A/C; owner verdict required).
"Baseline" = already visibly present from the pre-contract build (distinct
from A&S parity, which needs the richer motion + the rubric pass).

### A. Player — third person

| # | Feature | What it changes | Bedrock mechanism | Verdict | Status | Measurability (device rubric) |
|---|---|---|---|---|---|---|
| A1 | Walk: bounce, arm swing, shoulder shift, head bob, weighted steps | Full walk cycle replaced | entity anim (player.entity.json) | NATIVE | IN-PROGRESS (baseline) | F5, walk 10 s on flat ground: visible vertical bounce per step, counter-phase arm swing, head bob. Pass = all 3 visible at 2 m. |
| A2 | Run: forward lean, stronger cycle | Sprint variant | entity anim | NATIVE | NOT-STARTED | Sprint: torso leans forward vs walking; readable side-on. |
| A3 | Sneak: proper crouch cycle | Sneak variant | entity anim | NATIVE | NOT-STARTED | Sneak-walk: distinct lowered gait, no foot slide. |
| A4 | Swim: arm strokes + leg kicks, surface vs underwater variants | Swim cycle | entity anim | NATIVE | NOT-STARTED | Swim at surface then dive: two distinct stroke cycles. |
| A5 | Water entry: dive / belly-flop / cannonball by entry style | Context pose on water impact | anim controller | EQUIVALENT-WORK (fall-state detection) | NOT-STARTED | Jump into water from sprint, from standstill, from height: 3 distinct entry poses. |
| A6 | Jump + landing impact | Airborne + land poses | anim controller | NATIVE | IN-PROGRESS (gesture spring baseline) | Jump from 3 blocks: visible crouch-absorb on landing. |
| A7 | Idle: breathing, weight shift, occasional head movement | Idle layer | entity anim | NATIVE | IN-PROGRESS (baseline) | Stand still 30 s at F5: visible breathing + at least one glance/shift. |
| A8 | Idle emotes: stretch, look-around, arm-cross (v2.0) | Random idle clips | anim controller | NATIVE (keyframe layer exists) | NOT-STARTED | Stand still 90 s: at least one full emote plays. |
| A9 | Edge balancing (walking on block edges) | Balance pose near edges | Molang edge check | EQUIVALENT-WORK (edge probe) | NOT-STARTED | Walk along a 1-wide bridge: arms-out balance pose engages. |
| A10 | Eating: item to mouth, bites, chewing; golden-apple / chorus specials | Eat cycle + item motion | anim + item model | NATIVE pose (NEA baseline) + EQUIVALENT-WORK (item shrink) | IN-PROGRESS | Eat steak at F5: hand-to-mouth cycle; special foods differ. Item shrink: separate check. |
| A11 | Drinking: bottle tilt, visible liquid depletion | Drink cycle + model state | anim + geometry | Pose NATIVE; liquid depletion = named substitution candidate (C2) | NOT-STARTED | Drink potion: tilt cycle visible. Liquid level: owner verdict row. |
| A12 | Flip (crouch mid-air), Sit (sneak 5 s + look down), Roll (sprint-fall ≥4) | Player-triggered stunts | anim controller + input | NATIVE (keyframe clips + input triggers) | NOT-STARTED | Perform each trigger: clip plays, returns clean to normal pose. |
| A13 | Player death animations | Death pose cycle | anim | EQUIVALENT-WORK (death-time layer) | NOT-STARTED | Die on flat ground: body animates instead of vanilla tip-over. |
| A14 | Facial expressions: blinking, eye tracking, eyebrows (official 1.5 headline) | Face overlay on any skin | render controller + UV swap | BLOCKED — CLASS A | **DEFERRED (owner, 2026-11-09; no shader-hacked eyes per C2)** | n/a while deferred. |
| A15 | First-person animation suite | FP hands/items re-animated | FP anim set | DECIDED (Q3, 2026-11-09): extend vendored Punchy; single FP owner, no parallel layer | NOT-STARTED | FP rows verified inside Punchy's coverage checklist. |

### B. Combat & tools

| # | Feature | What it changes | Verdict | Status | Measurability |
|---|---|---|---|---|---|
| B1 | Sword: multi-angle strikes (horizontal/diagonal/overhead cycle) | Swing variety by combo counter | NATIVE (attack_anim + swing counter) | NOT-STARTED | Swing 4× at F5: at least 3 distinct arcs. |
| B2 | Axe: slower wind-up, heavy chop, follow-through | Weight differentiation | NATIVE | NOT-STARTED | Axe vs sword swings look distinct side-by-side. |
| B3 | Bow: draw pose + string tension | Draw cycle | NATIVE (vanilla bow stage textures carry tension) | NOT-STARTED | Draw bow 2 s: body pose + staged draw visible. |
| B4 | Shield: brace before block | Guard pose | NATIVE | NOT-STARTED | Raise shield: brace pose reads at F5. |
| B5 | Tool-specific mining arcs (pickaxe down-strike, shovel scoop, hoe rhythm) | Per-tool swing | NATIVE | NOT-STARTED | Mine with 3 tools: 3 distinct arcs. |
| B6 | Mining sparks scaled by tool material + block | Particles | NATIVE (subtle_effects pattern) | NOT-STARTED | Mine stone with wood vs diamond pick: spark count/colour differs. |
| B7 | Fishing rod: cast, line tension, reel, catch reaction | Rod cycle | EQUIVALENT-WORK (fishing state is client-visible; fishing_line hook exists) | NOT-STARTED | Cast + catch: all 4 phases animate. |
| B8 | 3D item models in hand / custom armour / costumes | Replaces item+armour visuals | BLOCKED — CLASS A | **DEFERRED (owner, 2026-11-09; separate scoping pass required before any implementation)** | n/a while deferred. |

### C. Mobs

| # | Feature | Verdict | Status | Measurability |
|---|---|---|---|---|
| C1 | Full re-animation pass, all ~60 Anim families (walk + idle per family, A&S-grade richness) | NATIVE (additive + keyframe layers over every family) | IN-PROGRESS (baseline covers ~14 families) | Per-family spot checks in Phase 3 checklist. |
| C2 | Cow/sheep graze, tail swish | NATIVE | IN-PROGRESS (baseline) | Idle cow 60 s: graze dip + tail motion. |
| C3 | Chicken: peck, wing scatter when approached | NATIVE (peck baseline; flee-flutter needs velocity gate) | IN-PROGRESS | Walk at chicken: wings flare while it scatters. |
| C4 | Wolf: wag (happy), tail-down (scared), stretch, yawn, ear perks, sit polish | NATIVE (mood from tail/sit state where synced; else Class B per-state) | IN-PROGRESS (wag baseline) | Tamed wolf: wag when looked at; stretch/yawn within 2 min idle. |
| C5 | Cat: loaf idle, wake-stretch | NATIVE | NOT-STARTED | Idle cat 2 min: loaf pose occurs. |
| C6 | Horse: gallop cycle, mane/tail flow, idle stomp, rear, ear flicks; mounted gaits | NATIVE/EQUIVALENT-WORK (mane/tail part availability per model) | NOT-STARTED | Gallop + idle horse checks. |
| C7 | Skeleton: rattle, precise aim, bow draw, retreat | NATIVE | NOT-STARTED | Skeleton tracking player: aim + draw read clearly. |
| C8 | Creeper: trailer-inspired motion | NATIVE (tiptoe baseline) | IN-PROGRESS | Stalking creeper reads "nervous" at 5 m. |
| C9 | Villager: blink + facial expressions | Face = BLOCKED with A14; head/arm life NATIVE (baseline) | PART-BLOCKED | Non-face motion: idle villager animates. Face: owner verdict. |
| C10 | Happy Ghast support | NATIVE (Anim::HappyGhast exists) | NOT-STARTED | Happy Ghast idle/fly cycle visibly animated. |
| C11 | Wither visual rework | Motion NATIVE; model rework = CLASS A (authored geometry pipeline) | PART-BLOCKED | Motion pass only until verdict. |
| C12 | Glow Squid rhythmic light pulse | EQUIVALENT-WORK (emissive pulse in render path) | NOT-STARTED | Glow squid pulses ~1 Hz in dark water. |
| C13 | Baby-mob redesigned models (Tiny Takeover: wolf/cat/pig/cow/chick) | BLOCKED — CLASS A (mob geometry replacement) | BLOCKED | Owner verdict. |
| C14 | Entity biome variants (authored textures, m.extra.variant hook exists) | EQUIVALENT-WORK | NOT-STARTED | Named/biome variant renders a distinct authored skin. |
| C15 | Nametag easter eggs (Baby/Cherry/Pirate/Maid) | EQUIVALENT-WORK (nametag-keyed variant) | NOT-STARTED | Name a mob: variant applies. |

### D. Environment

| # | Feature | Verdict | Status | Measurability |
|---|---|---|---|---|
| D1 | Ambient drifting particles in open areas | NATIVE | NOT-STARTED | Stand in plains: occasional motes drift. |
| D2 | Grass/foliage sway | NAMED SUBSTITUTION (approved 2026-11-09, Q4): Volt Shaders "Moving Vegetation" satisfies this row | IMPLEMENTED (pre-existing) | Enable Volt Shaders + Moving Vegetation: grass sways. |
| D3 | Better rain particles / splashes | NATIVE particles | NOT-STARTED | Rain: ground splash particles beyond vanilla. |
| D4 | Weather/biome visual reactions ("world reacts to weather") | CLASS B — not observable from public info what exactly changes | BLOCKED-B | Owner describes the reference behavior; rubric then defined. |
| D5 | Boat wakes: particle trails behind moving boats (official 1.5) | NATIVE (particles from boat velocity) | NOT-STARTED | Row a boat: foam trail particles behind the stern. |

### E. Settings toggles (1:1 rows; defaults mostly not public → gate Q2)

| # | Toggle | Semantics (public info) | Default | Verdict |
|---|---|---|---|---|
| E1 | Texture Style subpack (16x/32x/64x) | Selects texture overhaul level | DEFERRED-WITH-PARENT (B8/texture overhaul) | deferred |
| E2 | 3D Tools | 3D tool models on/off | DEFERRED-WITH-PARENT (B8) | deferred |
| E3 | Enable Expressions | Blink/eye-track/eyebrows on/off | DEFERRED-WITH-PARENT (A14) | deferred |
| E4 | Armour Visibility | first-person arms / vanilla / hidden | DEFERRED-WITH-PARENT (B8) | deferred |
| E5 | Tool Enchantment Glint | outline / glint / both / vanilla | TBD (screenshot) | EQUIVALENT-WORK (render tint path) |
| E6 | Enchantment Effect Colour | outline colour picker | TBD (screenshot) | With E5 |
| E7 | Totem of Undying Opacity | FP totem translucency | TBD (screenshot) | EQUIVALENT-WORK |
| E8 | Costumes | leather+trim = themed costumes | DEFERRED-WITH-PARENT (B8) | deferred |
| E9 | Item Swap Sounds | sound on hotbar switch | TBD (screenshot) | NATIVE (vanilla sound event) |
| E10 | Better Rain Particles | custom vs vanilla rain | TBD (screenshot) | With D3 |
| E11 | Axolotl Colour (bucket) | held-bucket variant colour | TBD (screenshot) | With C14 pipeline |
| E12 | Morphs (custom 3D player characters) | replaces player model | DEFERRED (owner, 2026-11-09: model replacement + server grief vector) | deferred |
| E13 | Legacy Baby Chick Model | old vs new chick | DEFERRED-WITH-PARENT (C13) | deferred |
| E14 | Entity Biome Variants | natural spawn vs nametag-only | TBD (screenshot) | With C14 |
| Volt | Player animations / Mob animations / Intensity (pre-contract Volt rows) | Volt-only additions, not in A&S | ON/ON/Balanced | Keep? Owner call at Phase 1 (they are extra toggles, not 1:1) |

### F-NOW (pulled into scope per Q1; per-row decision at Phase 1 gate; no implementation before Phase 1 sign-off)

| # | Feature | Version | Verdict | Status |
|---|---|---|---|---|
| FN1 | Holstered weapons (hotbar carried on back/belt) | 1.13 | EQUIVALENT-WORK | **DEFERRED (owner, 2026-11-09: un-sized; reopen as scoped candidate after v1 ships)** |
| FN2 | Sword combat stance | 1.13 | NATIVE (held-sword pose layer) | APPROVED → T1 |
| FN3 | Wolf full-moon howl | 1.13 | NATIVE (pose clip + moon/time gate; howl *sound* = authored-asset question, flagged) | APPROVED → T2 |

### F-BACKLOG (deferred by owner, 2026-11-09; logged, not dropped)

| Feature | Version |
|---|---|
| Mob + player death-animation system (directional/water/lava/fall, death effects + sounds) | 1.13 |
| Witch/Strider model reworks; armour shimmer; enchant outline on armour/elytra | 1.13 |
| Seasonal (Halloween) + Pale Garden mob variants | 1.13 |
| Placeable items, props (weapon rack, tanning rack, nets, hoops), plushies | 1.11–1.12 |
| Vibrant Visuals integration | 1.8 |

## Official-1.5 source audit (one pass, 2026-11-09; both chunks of the guide read)

Source: <https://guides.orevillestudios.com/1-5-23fd70150598817e90c9f038f425f025>

**Corrections to existing rows (no renumbering):**

- **C11 Wither**: official scope is *effects*, not geometry — spawn
  animation with visible skulls, phase-1 glowing eyes, phase-2 blue aura +
  particles + crack texture, pixel-accurate texture, 3rd-person throwing
  animation. Motion + particle + emissive parts are implementable under
  the approved motion-only substitute; texture parts group with the
  deferred asset pipeline. Target label stays IMPLEMENTED-PARTIAL.
- **A5 water entry**: official triggers — leap into water = dive; crouch
  at dive start = Cannon Ball; crouch midway (body flat) = Belly Flop;
  bubble particles on impact. Rubric updated to these exact triggers.
- **C4 Wolf**: interact on tamed wolf = 50% lay down; 4 laying idles
  (look around, ear wiggles, curious, ear scratch); dozes off with zZz
  particles, wake on interact; 3 standing idles (ear scratch, play bow,
  2 happy-hop variants). Own-player interactions are client-visible;
  other players' interactions = Class B note.
- **C5 Cat**: interact = 50% LOAF MODE, 3 loaf idles, doze with zZz,
  wake on interact. Same Class B note.
- **C10 Happy Ghast**: + Ghastling; 3 idles; wiggly tentacles. Facial
  parts ride A14 (deferred). Harness/balloon designs = asset content.
- **E1 subpack semantics** (official): since 1.5, Full Experience = 64x,
  Custom (texture-pack compatible) = 16x, Vanilla = 32x.

**New rows (added, not renumbered):**

| # | Feature | Verdict | Status |
|---|---|---|---|
| D6 | XP orb particle trail | NATIVE (particles) | NOT-STARTED (T3) |
| D7 | On-fire screen overlay (player burning) | EQUIVALENT-WORK (HUD overlay quad) | NOT-STARTED (T3) |
| D8 | Explosion particle revamp (scale-accurate) | NATIVE (particles) | NOT-STARTED (T3) |
| C16 | Rare natural variants + 1.5 nametag set (Baby Enderman 0.5% / Brian zombie 0.05% / Pink Dolphin 1% / Dapper Iron Golem / Oiia Oiia Cat / balloon + Saddleless tags / Husk Mummy-Dusty 3%) | EQUIVALENT-WORK — rides the C14 authored-texture pipeline | NOT-STARTED |
| X1 | 1.5 texture/asset content aggregate: 60 double-slab designs, 3D mushroom/dripstone model assignments, stone-brick variants, Disc 5 glows, smithing-rune emissives, held-item models (Dried Ghast etc.), kelp/glyph/panorama touch-ups, multi-lead locators | DEFERRED-WITH-PARENT (B8/E1 texture overhaul) | deferred |

## Gate Q&A — all answered 2026-11-09 (see session log). Open follow-ups:

- Q2 defaults screenshot: PENDING (owner, this session) → fills the E-table TBDs.
- X-cross retest on v0.9.9-build.11b1353: PENDING (owner) → VERIFIED or triage before Phase 1 code.

## Hook scoping (required before T1 starts; estimates, verified at build)

| Hook | Tier | Files touched | Rough LOC (impl + test) | Risk |
|---|---|---|---|---|
| H1 swing-variety counter | T1 | `mobs.rs` (AnimState +1 field, increment at attack_anim start — single site), `mobs/anim.rs` (State +1 field in `state()`), `overhauled.rs` (consume) | 25 + 15 | LOW |
| H2 edge probe | T1 | `game/volt_builtins.rs` (frame: 4 neighbor `game.world.block` + facts queries under feet — same API light_overlay glue uses), `overhauled.rs` (static + `Motion.on_edge`, pure classify fn) | 35 + 25 | LOW-MED (own player only) |
| H3 water-entry event | T1 | `game/volt_builtins.rs` (prev-frame in-water/fall/crouch tracker → queue clip), `mobs/overhauled_clips.rs` (trigger API) | 30 + 15 | LOW-MED (player fluid flag verified at build; fallback = block-at-eye fluid check, same API) |
| H4 moon-phase/time read | T2 | `game/volt_builtins.rs` (world time/day → static), `overhauled.rs` (gate) | 10 + 10 | LOW |
| H5 glow-squid emissive pulse | T2 | `mobs/render.rs` (emissive intensity scaled by etf phase for GlowSquid) | 15 + 10 (+20 if its emissive layer is absent — verified at build) | LOW |
| H6 clips module (not an engine hook) | T1 | `mobs/overhauled_clips.rs` NEW (sampler reuse, tables, zero-ends + budget tests) | ~300 | LOW (pattern exists in player_gestures) |
| H7 local-state feed (sprint/sneak/airborne/fall/sit-timer for A12) | T1 | `game/volt_builtins.rs` | 40 + 10 | LOW |
| H8 pet-interact trigger (C4/C5 lay/loaf) | T2 | game interact-entity path (site located at T2 start) → `overhauled.rs` per-mob state map (etf-seeded chance) | 40 + 15 | MED (locating the use-entity client path; other players' interactions = Class B) |
| H9 on-fire HUD overlay | T3 | `extras_hud.rs` (overlay quad while burning) | 30 + 10 | LOW-MED |
| H10 particle batch (XP trail, explosion, rain splash, boat wakes, ambient motes, mining sparks) | T3 | `overhauled_fx.rs` NEW + hook sites in `animate_tick.rs` / game frame (subtle_effects pattern) | ~120 + 30 | LOW (API proven) |
| (dropped) FN1 item-on-body render | — | deferred with FN1 | — | — |

## One-go build (2026-11-09, owner override) — per-row outcomes

Code: `mobs/overhauled_clips.rs` (NEW: keyframe clip engine + per-entity
trigger state, 9 unit tests), `mobs/overhauled.rs` v1.1.0 (swim stroke,
sprint lean, tool stances, swing variation, ledge balance, wolf/cat/
equine/wither/happy-ghast idle families, creeper fuse tremble, clip
integration), `game/volt_builtins.rs` (env feed, own-player triggers,
tick-rate particle pass, "Better rain particles" toggle). All statuses
below are IMPLEMENTED-UNVERIFIED until the owner's on-device pass
(Phase 3 checklist) upgrades them to VERIFIED; CI unit tests are the
only evidence attached at ship time.

| Row | Outcome | Mechanism |
|---|---|---|
| A1/A7 | IMPLEMENTED-UNVERIFIED | baseline gait/idle kept; follow-through + personality layers |
| A2 | IMPLEMENTED-UNVERIFIED | sprint body lean (speed-gated) |
| A3 | IMPLEMENTED-PARTIAL | vanilla crouch pose + gait layers persist while sneaking; no dedicated sneak cycle |
| A4 | IMPLEMENTED-UNVERIFIED | alternating front-crawl stroke + flutter kick overlay in water |
| A5 | IMPLEMENTED-UNVERIFIED | water-entry edge on `fluids.in_water` + tracked fall: dive / cannonball (crouch early) / belly flop (crouch at entry); bubble burst on impact. Remote players: crouch via synced pose — mid-air crouch may not sync on all servers (noted limitation) |
| A6 | IMPLEMENTED (baseline) | gesture spring landing kept |
| A8 | IMPLEMENTED-UNVERIFIED | stretch / look-around / arm-cross on a deterministic per-entity idle schedule |
| A9 | IMPLEMENTED-UNVERIFIED | own-player ledge probe (block-below + 2 open sides) → balance windmill |
| A10 | IMPLEMENTED-PARTIAL | NEA eat/drink pose layer only; item shrink NOT SHIPPED (item-render pipeline) |
| A11 | IMPLEMENTED-PARTIAL | tilt pose via NEA; liquid depletion NOT SHIPPED (geometry states) |
| A12 | IMPLEMENTED-UNVERIFIED | flip = crouch mid-air; sit = sneak 5 s + look down (held until sneak release); roll = landing a sprint fall ≥3 blocks. **Named limitation (C2): no root-motion hook — flip/roll are tuck-pose clips, not full 360° body rotations** |
| A13 | NOT SHIPPED | death-anim system rides F-BACKLOG (1.13 content) |
| B1 | IMPLEMENTED-UNVERIFIED | swing counter (rising-edge, no engine hook needed) cycles 3 lateral paths |
| B2/B5 | IMPLEMENTED-UNVERIFIED | axe chop drives body+shoulder; pick/shovel drive body; keyed off synced main-hand item |
| B3 | IMPLEMENTED-UNVERIFIED | drawn bow/crossbow fine tremble (vanilla staged draw kept) |
| B4 | IMPLEMENTED-UNVERIFIED | shield brace hunch while blocking |
| B6 | IMPLEMENTED-PARTIAL | sparks fly from the mined face while breaking; NOT scaled by tool material (flat rate) |
| B7 | NOT SHIPPED | fishing rod cycle — no cheap client fishing-state path found this pass |
| C1 | IMPLEMENTED-PARTIAL | families covered: humanoid(+5 undead/arbalist kinds), villager, enderman, quadruped group, horse, wolf, cat, fox, creeper, spider, chicken, wither, happy ghast, ambient fallback for the rest |
| C2/C3/C8 | IMPLEMENTED (baseline kept) | graze/tail, peck/ruffle, tiptoe + NEW fuse tremble (C8) |
| C4 | IMPLEMENTED-PARTIAL | play bow + happy hops + ear flicks as idle windows; full-moon howl clip (FN3). Interact-to-lay trigger NOT SHIPPED (H8 pending) |
| C5 | IMPLEMENTED-PARTIAL | loaf (legs folded, tail curled, doze nod) as idle window; interact trigger NOT SHIPPED (H8) |
| C6 | IMPLEMENTED-PARTIAL | tail swish, head toss, idle stomp; mane flow needs model parts not present |
| C7 | IMPLEMENTED-UNVERIFIED | drawn-bow tremble applies to skeletons (synced main hand + using) |
| C10 | IMPLEMENTED-PARTIAL | slow body sway; tentacle parts not separately articulated in the model |
| C11 | IMPLEMENTED-PARTIAL (as approved) | three heads drift/scan, ribcage pulse, tail sway; effects/texture parts stay deferred |
| C12 | IMPLEMENTED-PARTIAL | glow twinkle particles near glow squids; render-path emissive pulse NOT SHIPPED (C5 risk on shared render path) |
| C14/C15/C16 | NOT SHIPPED | authored-texture variant pipeline — asset work, unchanged verdict |
| D1 | IMPLEMENTED-UNVERIFIED | sparse air motes near the player (air-block checked) |
| D3/E10 | IMPLEMENTED-UNVERIFIED | rain impact splashes on open-sky surfaces, "Better rain particles" toggle (default ON — assumption until the defaults screenshot lands). Limitation: global weather flag, not biome-filtered |
| D5 | IMPLEMENTED-UNVERIFIED | stern wake + hull bubbles while under way, oar splashes while paddling, idle hull foam |
| D6 | IMPLEMENTED-UNVERIFIED | twinkle trail on XP orbs (teal Glow particle — colour differs from the reference's green; named approximation) |
| D7 | NOT SHIPPED | on-fire HUD overlay (H9) — HUD hook not landed this pass |
| D8 | NOT SHIPPED | explosion revamp — explosion event hook not scoped-safe this pass |
| E5/E6/E7 | NOT SHIPPED | glint/totem render-pipeline work |
| E9 | NOT SHIPPED | no confirmed UI-sound API on the client path |
| E11/E14 | DEFERRED-WITH-PARENT | unchanged |
| FN2 | IMPLEMENTED-UNVERIFIED | sword ready stance while a sword is held |
| FN3 | IMPLEMENTED-UNVERIFIED | howl clip on full-moon nights (day_time: night window + day-0 moon phase); howl *sound* not shipped (authored asset) |
| H1–H7, H10 | LANDED | as scoped (H1 needed no engine hook; H10 lives in volt_builtins, not a separate fx file) |
| H5, H8, H9 | NOT LANDED | listed above against their rows |

**Honest remaining list after v1.1 (superseded by the v1.2 pass below):**
A13 death anims, B7 fishing, B6 material scaling, H8 interact triggers,
D7 fire overlay, D8 explosions, E5/E6/E7 glint+totem, E9 swap sounds,
A10/A11 item-render parts, C14/C15/C16/X1 asset pipeline, render-path
glow pulse (C12 full), A14/B8/C13/E12 owner-deferred rows, FN1,
F-BACKLOG.

## v1.2.0 one-go pass (owner one-go override; this build)

Second completion sweep over the v1.1 remaining list. Everything below
is CI-tested where a unit test can reach it, and IMPLEMENTED-UNVERIFIED
on-device until the owner's device pass.

| Row | Status after v1.2 | What shipped |
|---|---|---|
| A13 | IMPLEMENTED-UNVERIFIED | death limp layer: while `death_time > 0`, limbs slacken and spread, head lolls, eased over the 20 death ticks, additive over the vanilla tip-over (players and mobs). Unit-tested (ease-in from zero, movement, budget). Note: a *limp*, not the 1.13 authored death-pose cycle — named scope (C2) |
| B6 | IMPLEMENTED-UNVERIFIED | spark count now scales with the actual break rate (`progress/ticks`), 1–5 per tick — fast tools on soft blocks spray, slow tools trickle. Colour variation still NOT shipped (named residue) |
| B7 | IMPLEMENTED-PARTIAL | rod-holding pose (rod arm up/forward) for every entity with a hook out — own player via `Game.hook`, remote players via `remote_hooks.owner`; bite jerk (8 Hz rod fight) on the own line via `Hook.nibble`. Unit-tested. Cast/reel sub-phases NOT separately animated (named residue) |
| H8 / C4 / C5 | IMPLEMENTED-UNVERIFIED | interact-to-loaf (cats) / interact-to-lay (wolves) landed: `interact_mob` → `note_interact` → stable 50% roll (etf-seeded per entity+count) settles the pet into a sustained loaf/sphinx-lay clip; the next interact wakes it. Idle windows damped while sustained. Unit-tested (determinism, wake, family gating). Named approximation (C2): fires on any mob interact incl. feeding |
| D7 | IMPLEMENTED-UNVERIFIED | on-fire screen overlay: flickering flame tongues + orange wash over the lower screen, first person only, gated on `fire_overlay_on()` (mod + players layer) and the own model burning. Flat-colour HUD quads off the atlas white texel — zero new assets. Limitation: draws inside `draw_extras` (hidden with the HUD by F1/hide-gui) |
| D8 | IMPLEMENTED-UNVERIFIED | explosion burst layered over the vanilla particles: radius-scaled spherical spray (10 + 6·radius, cap 40) of poof / large-smoke / ember particles, vanilla sprites only, gated on `active()` |
| C12 | IMPLEMENTED-UNVERIFIED | render-path glow pulse landed: glow squid whole-body block-light byte pulses 150–240 on a slow per-squid cycle (emf tempo/phase), sky bits untouched, never darkens below scene light, zero cost for every other species (one species check). Unit-tested (suspended = unchanged, sky preserved, moving, never darkens) |
| E9 | IMPLEMENTED-UNVERIFIED | "Item swap sounds" toggle (default ON) + soft page-turn flick on hotbar slot change. **Named substitution (C2): the vanilla `item.book.page_turn` event at low volume stands in for the pack's authored swap sounds — no third-party audio ships (C1)** |

Engine hooks this pass (all minimal, named per C5): `fishing.rs`
`Hook.nibble` + `RemoteHook.owner` → `pub(super)` (visibility only, no
behavior change); `creatures.rs` `interact_mob` one `note_interact`
call; `explosion.rs` post-vanilla particle burst; `render.rs` one
glow-squid light branch; `extras.rs` overlay call at `draw_extras` top;
`extras_hud.rs` new `draw_fire_overlay`. Shared paths touched: mob
render light path, explosion particles, HUD extras — all behind
`active()` gates, zero cost while the mod is off.

**Honest remaining list (not silently dropped):** E5/E6 enchant-glint
colour/strength, E7 totem opacity (item render pipeline), A10/A11
item-render parts (shrinking food, liquid depletion), C14/C15/C16/X1
authored-texture variant pipeline (C1: needs original art, no
third-party bytes), B6 spark colour by material, B7 cast/reel
sub-phases, A13 authored death-pose cycle (F-BACKLOG), A14/B8/C13/E12
owner-deferred rows, FN1, F-BACKLOG. E-toggle defaults stay provisional
until the owner's defaults screenshot arrives. On-device verification
(Class D) remains owner-side for every row above.

## Known-good / known-bad devices (Class D)

| Device | Status |
|---|---|
| (owner's Mali-G57 MC2 / Helio G96 device) | reference device; no entries yet |

## OUT-OF-BAND fixes (tagged [OOB]; not phase work, excluded from phase audits)

### [OOB] Block X-cross overlay

- Diagnosis: woven `light_overlay` mod, persisted `active=true`, no F7 on
  touch. Fix v1 (frame scrub, ad7e04a) — owner reports FAIL on device.
  Root-cause candidate for v1 failure: frame-order race vs the mod's
  first cached settings read.
- Fix v2 (5c18dad): read-level force in `native_setting` — stale flag can
  never load regardless of order. IMPLEMENTED-UNVERIFIED.
- If crosses survive v2 on-device: light_overlay is eliminated; next
  suspect is real texture/atlas corruption — owner to report whether the
  marks are clean red/yellow X lines on top of dark blocks (overlay) or
  garbled texture pixels (atlas), plus a screenshot if possible.
