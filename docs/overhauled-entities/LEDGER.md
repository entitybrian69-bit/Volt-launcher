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
| 2026-11-09 | Light Overlay stale-overlay fix v2 (read-level force, commit 5c18dad) — IMPLEMENTED-UNVERIFIED, awaiting owner device run. Not part of this mission; tracked in BUILTIN-PARITY.md. | agent |

## Packaging note (for Phase 1, logged early per §8)

Animations require native code. Android never executes downloaded native
code (standing Volt directive), so a literal downloadable `.hmod` cannot
carry animation logic. The delivery vehicle is the existing compiled-in
native mod row "Overhauled Entities" (same UX as an hmod row in Options >
Mods). Data-only parts (authored textures/variants) can ship as `.hmod`
content. Formal decision at Phase 1 gate.

## Version-scope question (gate Q1, Class B)

The official Oreville changelog numbers releases 1.0 → 1.13 (1.13 =
Nov 2026). "v2.0" is the fan/APK-site branding for the 2025 facial-
expressions era (blink/eye-tracking/eyebrows, Happy Ghast, Wither rework).
Inventory below covers the full 2.0-era set and lists post-2.0 (1.11–1.13)
features separately so nothing is silently dropped (C3). Owner to confirm
scope at sign-off.

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
| A14 | Facial expressions: blinking, eye tracking, eyebrows (v2.0 headline) | Face overlay on any skin | render controller + UV swap | BLOCKED — CLASS A (no skin-face UV/overlay hook in Horizon) | BLOCKED | n/a until verdict. Smallest substitute: engine hook drawing an authored eye-overlay quad on the head part, skin-tone-agnostic. Owner verdict required. |
| A15 | First-person animation suite | FP hands/items re-animated | FP anim set | EQUIVALENT-WORK; overlaps vendored Punchy port (223 clips) | NOT-STARTED | Owner decision: extend Punchy coverage vs. parallel A&S-style FP set. Gate Q3. |

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
| B8 | 3D item models in hand / custom armour / costumes | Replaces item+armour visuals | BLOCKED — CLASS A (no custom in-hand item/armour geometry pipeline) + heavy authored-asset volume | BLOCKED | Owner verdict. |

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
| D2 | Grass/foliage sway | NATIVE — overlaps Volt Shaders "Moving Vegetation" (already shipped, off by default) | IN-PROGRESS (existing feature) | Enable: grass sways. Owner decides if the A&S row maps to it (gate Q4). |
| D3 | Better rain particles / splashes | NATIVE particles | NOT-STARTED | Rain: ground splash particles beyond vanilla. |
| D4 | Weather/biome visual reactions ("world reacts to weather") | CLASS B — not observable from public info what exactly changes | BLOCKED-B | Owner describes the reference behavior; rubric then defined. |

### E. Settings toggles (1:1 rows; defaults mostly not public → gate Q2)

| # | Toggle | Semantics (public info) | Default | Verdict |
|---|---|---|---|---|
| E1 | Texture Style subpack (Full Experience 16x / 32x vanilla+3D / 64x anims-only) | Selects texture overhaul level | Full Experience (confidence 70%) | Texture overhaul itself = CLASS A/asset-volume; toggle row BLOCKED with it |
| E2 | 3D Tools | 3D tool models on/off | UNKNOWN (B) | With B8 |
| E3 | Enable Expressions | Blink/eye-track/eyebrows on/off | ON (confidence 80%) | With A14 |
| E4 | Armour Visibility | first-person arms / vanilla / hidden | UNKNOWN (B) | With B8 |
| E5 | Tool Enchantment Glint | outline / glint / both / vanilla | UNKNOWN (B) | EQUIVALENT-WORK (render tint path) |
| E6 | Enchantment Effect Colour | outline colour picker | UNKNOWN (B) | With E5 |
| E7 | Totem of Undying Opacity | FP totem translucency | UNKNOWN (B) | EQUIVALENT-WORK |
| E8 | Costumes | leather+trim = themed costumes | UNKNOWN (B) | With B8 |
| E9 | Item Swap Sounds | sound on hotbar switch | UNKNOWN (B) | NATIVE (vanilla sound event) |
| E10 | Better Rain Particles | custom vs vanilla rain | UNKNOWN (B) | With D3 |
| E11 | Axolotl Colour (bucket) | held-bucket variant colour | Pink (B) | With C14 pipeline |
| E12 | Morphs (custom 3D player characters) | replaces player model | n/a | BLOCKED — CLASS A (player geometry replacement) |
| E13 | Legacy Baby Chick Model | old vs new chick | UNKNOWN (B) | With C13 |
| E14 | Entity Biome Variants | natural spawn vs nametag-only | UNKNOWN (B) | With C14 |
| Volt | Player animations / Mob animations / Intensity (pre-contract Volt rows) | Volt-only additions, not in A&S | ON/ON/Balanced | Keep? Owner call at Phase 1 (they are extra toggles, not 1:1) |

### F. Post-2.0 features (listed so nothing is silently dropped; default OUT-OF-SCOPE pending gate Q1)

| Feature | Version |
|---|---|
| Mob + player death animations, directional/water/lava/fall deaths, death effects + sounds | 1.13 |
| Holstered weapons (hotbar carried on back/belt) | 1.13 |
| Sword combat stance; armour shimmer; enchant outline on armour/elytra | 1.13 |
| Wolf full-moon howl; Witch/Strider model rework | 1.13 |
| Seasonal (Halloween) + Pale Garden mob variants | 1.13 |
| Placeable items, props (weapon rack, tanning rack, nets, hoops), plushies | 1.11–1.12 |
| Vibrant Visuals integration | 1.8 |

## Gate questions (batched, §8)

1. **Scope (B):** "v2.0" = the facial-expressions-era feature set (sections A–E) — confirm, or include the 1.11–1.13 rows in section F?
2. **Defaults (B):** You own the pack — read the real defaults for E1–E14 off the device settings screen (one screenshot of the pack settings page suffices).
3. **First person (decision):** extend the vendored Punchy port's coverage, or build a parallel A&S-style FP set? Recommendation: extend Punchy — one FP animation owner, no layer fights.
4. **D2 mapping:** does Volt Shaders' Moving Vegetation satisfy the A&S grass-sway row, as a named substitution (C2)?
5. **Class A verdicts needed:** A14 faces, B8 3D items/armour, C11 Wither model, C13 baby models, E12 Morphs — cut / substitute / defer each.

## Known-good / known-bad devices (Class D)

| Device | Status |
|---|---|
| (owner's Mali-G57 MC2 / Helio G96 device) | reference device; no entries yet |

## Side quest (not this mission): block X-cross overlay

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
