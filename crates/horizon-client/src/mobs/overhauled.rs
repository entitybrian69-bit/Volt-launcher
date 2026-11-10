//! Overhauled Entities, built into Volt Launcher: a full-body animation
//! overhaul for EVERY living entity, the player included, applied over the
//! vanilla pose after `anim::setup`. Walking gains weight and follow-
//! through (eased double-frequency gait harmonics), bodies roll and
//! counter-roll into steps, idle entities breathe, glance around and
//! fidget, zombies reach and sway, creepers tiptoe, lean and tremble as
//! they swell, spiders wave their legs, chickens peck as they strut,
//! wolves and cats wag and curl their tails. On top of the procedural
//! layer ride the keyframed clips (`overhauled_clips`): idle emotes, the
//! mid-air flip, the sprint-fall roll, the sneak sit, the three water
//! entries and the full-moon wolf howl. Players swim with a real stroke,
//! lean into sprints, settle into a sword stance, tremble holding a drawn
//! bow, brace behind shields and windmill for balance on ledges; horses
//! swish and stomp, cats loaf, wolves play-bow and hop, withers drift
//! their three heads, happy ghasts sway. Every entity gets a stable
//! personality (phase, amplitude, tempo) so herds never move in lockstep.
//! Purely visual: movement, timing and hitboxes never change, so it works
//! on any server.
//!
//! Behavioral parity with Actions & Stuff (Oreville Studios, Bedrock) and
//! in the spirit of FreshLX's Fresh Animations — reimplemented from their
//! documented behaviour as native procedural animation; no upstream assets
//! or code are used. Powered by the hidden `emf`/`etf` engine layers.
//! Credits in THIRD_PARTY_NOTICES.md.
use super::anim::{Anim, Poses, State};
use super::{Mob, emf, etf, overhauled_clips as clips};
use horizon_core::geometry::trig::Mth;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const ID: &str = "overhauled_entities";
pub const NAME: &str = "Overhauled Entities";
pub const VERSION: &str = "1.3.0";
pub const DESCRIPTION: &str = "Animation overhaul for every entity, the player included: \
weighted walks, swim strokes, sword stances, ledge balancing, dives, flips, rolls, sits, \
idle emotes, death limps, fishing poses, wolf howls and lay-downs, cat loafs, horse \
stomps, wither head drift, glow-squid pulses, reel yanks, shrinking food and a rare \
pink dolphin. Plus rain splashes, boat wakes, richer explosions, XP-orb trails, \
material-coloured mining sparks, item swap sounds, a totem pop-up dial and an on-fire \
overlay. Purely visual, works on any server.";

static ENABLED: AtomicBool = AtomicBool::new(true);
static PLAYERS: AtomicBool = AtomicBool::new(true);
static MOBS: AtomicBool = AtomicBool::new(true);
static RAIN: AtomicBool = AtomicBool::new(true);
static SWAP: AtomicBool = AtomicBool::new(true);
/// Totem pop-up mode: 0 = full, 1 = subtle (shrunk), 2 = hidden.
static TOTEM: AtomicUsize = AtomicUsize::new(0);
/// First-person food shrink: set by the game each frame (defaults off so
/// the pure `item_pose` fixtures never drift in tests).
static FP_SHRINK: AtomicBool = AtomicBool::new(false);
/// 0 = Subtle, 1 = Balanced, 2 = Dramatic.
static INTENSITY: AtomicUsize = AtomicUsize::new(1);
/// Set while the mod's row in Options > Mods is switched off entirely.
static SUSPENDED: AtomicBool = AtomicBool::new(false);
/// Fed once a frame by the game bridge: whether it is a full-moon night
/// (wolves howl), whether the own player stands on a ledge, and whether
/// the own player holds crouch (the synced pose covers everyone else).
static NIGHT_MOON: AtomicBool = AtomicBool::new(false);
static OWN_EDGE: AtomicBool = AtomicBool::new(false);
static OWN_CROUCH: AtomicBool = AtomicBool::new(false);
/// Entities with a fishing hook in the water right now, and whether the
/// own player's line has a bite (fed by the game bridge each tick).
static FISHING: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());
static OWN_BITE: AtomicBool = AtomicBool::new(false);

pub const INTENSITY_NAMES: [&str; 3] = ["Subtle", "Balanced", "Dramatic"];
const INTENSITY_SCALE: [f32; 3] = [0.6, 1.0, 1.45];

pub fn set_suspended(off: bool) {
    SUSPENDED.store(off, Ordering::Relaxed);
}
pub fn set_env(night_moon: bool) {
    NIGHT_MOON.store(night_moon, Ordering::Relaxed);
}
pub fn set_own_edge(edge: bool) {
    OWN_EDGE.store(edge, Ordering::Relaxed);
}
pub fn set_own_crouch(crouch: bool) {
    OWN_CROUCH.store(crouch, Ordering::Relaxed);
}
pub fn set_fishing(ids: &[u32], own_bite: bool) {
    let mut f = FISHING.lock().unwrap();
    f.clear();
    f.extend_from_slice(ids);
    OWN_BITE.store(own_bite, Ordering::Relaxed);
}
fn is_fishing(id: u32) -> bool {
    FISHING.lock().unwrap().contains(&id)
}

/// Whether the on-fire screen overlay should draw (the HUD asks).
pub fn fire_overlay_on() -> bool {
    active() && PLAYERS.load(Ordering::Relaxed)
}

/// The totem pop-up dial for the item-activation overlay: 0 = full,
/// 1 = subtle (shrunk), 2 = hidden. Full whenever the mod is off.
pub fn totem_mode() -> usize {
    if active() { TOTEM.load(Ordering::Relaxed).min(2) } else { 0 }
}
pub const TOTEM_NAMES: [&str; 3] = ["Full", "Subtle", "Hidden"];

/// First-person food shrink gate: the game feeds it each frame from the
/// player-animation flag; defaults off so pure pose fixtures never move.
pub fn set_fp_shrink(on: bool) {
    FP_SHRINK.store(on, Ordering::Relaxed);
}
pub fn fp_shrink_on() -> bool {
    FP_SHRINK.load(Ordering::Relaxed)
}

/// The rare-variant tint for a dolphin: about 1 in 100 dolphins is pink,
/// stable per entity id. White (no tint) while the mod or its mob layer
/// is off. **Named substitution:** a procedural body tint stands in for
/// the reference pack's authored variant texture — no third-party art.
pub fn dolphin_tint(id: u32) -> u32 {
    if !active() || !MOBS.load(Ordering::Relaxed) {
        return 0xFFFF_FFFF;
    }
    if etf::entity_random(id, 800) < 0.01 {
        // Pink RGB (255, 175, 201), packed ABGR like every body tint.
        0xFFC9_AFFF
    } else {
        0xFFFF_FFFF
    }
}

/// The glow squid's rhythmic light pulse: raises the packed block-light
/// byte on a slow per-entity cycle, leaving the sky bits (and everything
/// above them) untouched. Returns the light unchanged while the mod or
/// its mob layer is off, and never darkens below the scene light.
pub fn glow_pulse(id: u32, age: f32, light: u32) -> u32 {
    if !active() || !MOBS.load(Ordering::Relaxed) {
        return light;
    }
    let per = emf::personality(id);
    let s = 0.5 + 0.5 * (age / 20.0 * per.tempo * 1.2 * std::f32::consts::TAU + per.phase).msin();
    let block = (150.0 + 90.0 * s) as u32;
    let block = block.min(240).max(light & 0xFF);
    (light & !0xFFu32) | block
}

/// The game's interact-entity path reports a click on a mob: cats may
/// settle into the loaf, wolves into the lay (50%, purely visual).
pub fn note_interact(m: &Mob) {
    if !active() || !MOBS.load(Ordering::Relaxed) {
        return;
    }
    let family = match m.kind.anim {
        Anim::Feline => clips::Family::Cat,
        Anim::Wolf => clips::Family::Wolf,
        _ => return,
    };
    clips::interact_toggle(m.id, family, m.age as f32 / 20.0);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flags {
    pub enabled: bool,
    pub players: bool,
    pub mobs: bool,
    pub rain: bool,
    pub swap: bool,
    pub totem: usize,
    pub intensity: usize,
}
pub fn flags() -> Flags {
    Flags {
        enabled: ENABLED.load(Ordering::Relaxed),
        players: PLAYERS.load(Ordering::Relaxed),
        mobs: MOBS.load(Ordering::Relaxed),
        rain: RAIN.load(Ordering::Relaxed),
        swap: SWAP.load(Ordering::Relaxed),
        totem: TOTEM.load(Ordering::Relaxed).min(2),
        intensity: INTENSITY.load(Ordering::Relaxed).min(2),
    }
}
pub fn set_flags(f: Flags) {
    ENABLED.store(f.enabled, Ordering::Relaxed);
    PLAYERS.store(f.players, Ordering::Relaxed);
    MOBS.store(f.mobs, Ordering::Relaxed);
    RAIN.store(f.rain, Ordering::Relaxed);
    SWAP.store(f.swap, Ordering::Relaxed);
    TOTEM.store(f.totem.min(2), Ordering::Relaxed);
    INTENSITY.store(f.intensity.min(2), Ordering::Relaxed);
}
pub fn active() -> bool {
    ENABLED.load(Ordering::Relaxed) && !SUSPENDED.load(Ordering::Relaxed)
}

/// What the entity holds, read from the synced main-hand item id: the
/// stance, swing style and tremble layers key off it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    None,
    Sword,
    Axe,
    Pickaxe,
    Shovel,
    Hoe,
    Bow,
    Crossbow,
    Trident,
}
pub fn tool_of(item: Option<&str>) -> Tool {
    let Some(id) = item else { return Tool::None };
    let n = id.rsplit(':').next().unwrap_or(id);
    if n.ends_with("_sword") {
        Tool::Sword
    } else if n.ends_with("_axe") {
        Tool::Axe
    } else if n.ends_with("_pickaxe") {
        Tool::Pickaxe
    } else if n.ends_with("_shovel") {
        Tool::Shovel
    } else if n.ends_with("_hoe") {
        Tool::Hoe
    } else if n == "crossbow" {
        Tool::Crossbow
    } else if n == "bow" {
        Tool::Bow
    } else if n == "trident" {
        Tool::Trident
    } else {
        Tool::None
    }
}

/// What the motion routines read, extracted so each is a pure function of
/// plain values (and unit-testable without building a `Mob`).
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    /// Seconds including the partial tick, scaled by the personality tempo.
    pub t: f32,
    /// The vanilla walk-animation position (`s.pos`).
    pub pos: f32,
    /// The vanilla walk-animation speed, 0..1.
    pub speed: f32,
    /// Master amplitude: the intensity setting times the personality.
    pub k: f32,
    /// The entity's stable phase, so herds desynchronise.
    pub phase: f32,
    /// 1 when standing still, fading to 0 once walking.
    pub idle: f32,
    /// The vanilla attack-swing progress, 0..1 (0 = not swinging).
    pub swing: f32,
    /// How many swings this entity has thrown (for attack variation).
    pub swing_index: u32,
    /// Bobbing in water: the swim-stroke overlay gates on it.
    pub in_water: bool,
    /// Using an item right now (a drawn bow trembles).
    pub using: bool,
    /// Holding a shield up: the brace stance.
    pub blocking: bool,
    /// The own player standing over a ledge: the balance windmill.
    pub on_edge: bool,
    /// A swelling creeper's tension, 0..1: the fuse tremble.
    pub tense: f32,
    /// What the main hand holds.
    pub tool: Tool,
    /// A fishing hook is out: the rod-holding pose.
    pub fishing: bool,
    /// The own player's line has a bite: the rod jerks.
    pub bite: bool,
}

/// The post-pass over the finished vanilla pose, for every mob kind.
pub fn apply(m: &Mob, s: &State, p: &mut Poses) {
    if !active() {
        return;
    }
    let f = flags();
    let player = matches!(m.kind.anim, Anim::Humanoid { .. });
    if (player && !f.players) || (!player && !f.mobs) {
        return;
    }
    let per = emf::personality(m.id);
    if m.death_time > 0 {
        // Dying: the limp collapse layer over the vanilla tip-over, and
        // nothing else.
        let t01 = (m.death_time as f32 + s.partial) / 20.0;
        death_limp(p, t01, INTENSITY_SCALE[f.intensity] * per.amp, per.phase);
        return;
    }
    let family = match m.kind.anim {
        Anim::Humanoid { .. }
        | Anim::Zombie
        | Anim::Drowned
        | Anim::ZombieVillager
        | Anim::ZombifiedPiglin
        | Anim::Skeleton
        | Anim::Witch
        | Anim::Illager
        | Anim::Piglin => clips::Family::Humanoid,
        Anim::Wolf => clips::Family::Wolf,
        Anim::Feline => clips::Family::Cat,
        _ => clips::Family::Other,
    };
    // The own player's crouch comes from the keyboard; everyone else's
    // from the synced entity pose (5 = crouching in the vanilla order).
    let crouching = if m.own {
        OWN_CROUCH.load(Ordering::Relaxed)
    } else {
        m.extra.synced_pose == 5
    };
    let mut k = INTENSITY_SCALE[f.intensity] * per.amp;
    if m.extra.fall_flying {
        // Elytra flight keeps its vanilla pose: damp everything additive.
        k *= 0.25;
    }
    let idle = (1.0 - s.speed * 3.0).clamp(0.0, 1.0);
    let sig = clips::tick(&clips::Obs {
        id: m.id,
        now: s.age / 20.0,
        swing: s.swing,
        in_water: m.fluids.in_water,
        fall: m.fall_distance as f32,
        crouching,
        idle: idle > 0.6,
        family,
        howl_ok: NIGHT_MOON.load(Ordering::Relaxed),
    });
    let mut mo = Motion {
        t: s.age / 20.0 * per.tempo * if s.baby { 1.15 } else { 1.0 },
        pos: s.pos,
        speed: s.speed.min(1.0),
        k,
        phase: per.phase,
        idle,
        swing: s.swing,
        swing_index: sig.swing_index,
        in_water: m.fluids.in_water,
        using: m.extra.using_item > 0,
        blocking: m.extra.blocking,
        on_edge: m.own && OWN_EDGE.load(Ordering::Relaxed),
        tense: (m.swell as f32 / 30.0).clamp(0.0, 1.0),
        tool: tool_of(m.extra.main_hand.as_deref()),
        fishing: is_fishing(m.id),
        bite: m.own && OWN_BITE.load(Ordering::Relaxed),
    };
    // A sustained clip (sit, loaf, lay) owns the pose: damp the idle
    // windows so a loafing cat does not also graze or play.
    if let Some((c, _, _)) = sig.clip
        && clips::sustained(c)
    {
        mo.idle *= 0.25;
    }
    match m.kind.anim {
        Anim::Humanoid { zombie_arms } => humanoid(p, &mo, zombie_arms),
        Anim::Zombie | Anim::Drowned | Anim::ZombieVillager | Anim::ZombifiedPiglin => {
            humanoid(p, &mo, true);
        }
        Anim::Skeleton | Anim::Witch | Anim::Illager | Anim::Piglin => humanoid(p, &mo, false),
        Anim::Villager => villager(p, &mo),
        Anim::Enderman => enderman(p, &mo),
        Anim::Quadruped
        | Anim::Sheep
        | Anim::PolarBear
        | Anim::Panda
        | Anim::Goat
        | Anim::Hoglin
        | Anim::Llama => quadruped(p, &mo),
        Anim::Horse => {
            quadruped(p, &mo);
            equine(p, &mo);
        }
        Anim::Wolf => {
            quadruped(p, &mo);
            tail_wag(p, &mo);
            wolf(p, &mo);
        }
        Anim::Feline => {
            quadruped(p, &mo);
            tail_wag(p, &mo);
            cat(p, &mo);
        }
        Anim::Fox => {
            quadruped(p, &mo);
            tail_wag(p, &mo);
        }
        Anim::Creeper => creeper(p, &mo),
        Anim::Spider => spider(p, &mo),
        Anim::Chicken => chicken(p, &mo),
        Anim::Wither => wither(p, &mo),
        Anim::HappyGhast => happy_ghast_sway(p, &mo),
        _ => ambient(p, &mo),
    }
    // The keyframed clips ride on top of everything: idle emotes fade
    // with the idle factor (so walking out of one never pops), triggered
    // clips (dives, flips, rolls, sits, howls) play at full strength.
    if let Some((c, t, emote)) = sig.clip {
        let fade = if emote { mo.idle } else { 1.0 };
        let scale = (0.85 + 0.15 * mo.k.min(2.0)) * fade;
        if scale > 0.01 {
            clips::apply_clip(p, c, t, scale);
        }
    }
}

/// Weighted humanoid gait + idle life. `zombie` adds the reaching sway.
pub fn humanoid(p: &mut Poses, mo: &Motion, zombie: bool) {
    let k = mo.k;
    // Follow-through: a double-frequency bounce over the vanilla swing,
    // so steps land with weight instead of a rigid metronome.
    let bounce = emf::gait_bounce(mo.pos, mo.speed, mo.phase * 0.0);
    emf::add(p, "right_arm", 3, emf::capped(0.14 * k * bounce, 0.35));
    emf::add(p, "left_arm", 3, emf::capped(-0.14 * k * bounce, 0.35));
    emf::add(p, "right_leg", 3, emf::capped(-0.10 * k * bounce, 0.3));
    emf::add(p, "left_leg", 3, emf::capped(0.10 * k * bounce, 0.3));
    // Body roll into each step, head counter-rolling to stay level.
    let roll = (mo.pos * 0.6662).msin() * mo.speed;
    emf::add(p, "body", 5, emf::capped(0.05 * k * roll, 0.12));
    emf::add(p, "head", 5, emf::capped(-0.035 * k * roll, 0.1));
    emf::add(p, "body", 4, emf::capped(0.06 * k * roll, 0.14));
    // The sprint lean: shoulders pitch into a fast run.
    emf::add(p, "body", 3, 0.10 * k * ((mo.speed - 0.65).max(0.0) / 0.35));
    // Idle life: breathing arms, a slow head wander, a tiny weight shift.
    let idle = mo.idle * k;
    emf::add(p, "right_arm", 5, 0.025 * idle * emf::osc(mo.t, 0.21, mo.phase));
    emf::add(p, "left_arm", 5, -0.025 * idle * emf::osc(mo.t, 0.21, mo.phase + 0.4));
    emf::add(p, "head", 3, 0.045 * idle * emf::drift(mo.t, 0.07, mo.phase));
    emf::add(p, "head", 4, 0.08 * idle * emf::drift(mo.t, 0.045, mo.phase + 2.1));
    emf::add(p, "body", 5, 0.012 * idle * emf::osc(mo.t, 0.09, mo.phase + 1.0));
    if zombie {
        // The undead reach: raised arms wander and spread, stronger while
        // chasing, never enough to break an attack swing.
        let reach = 0.5 + 0.5 * mo.speed;
        emf::add(p, "right_arm", 3, emf::capped(0.11 * k * reach * emf::drift(mo.t, 0.33, mo.phase), 0.3));
        emf::add(p, "left_arm", 3, emf::capped(0.11 * k * reach * emf::drift(mo.t, 0.33, mo.phase + 2.6), 0.3));
        emf::add(p, "right_arm", 4, 0.06 * k * emf::drift(mo.t, 0.21, mo.phase + 1.2));
        emf::add(p, "left_arm", 4, -0.06 * k * emf::drift(mo.t, 0.21, mo.phase + 4.0));
    }
    swim_stroke(p, mo);
    stance(p, mo);
    attack_variation(p, mo);
    edge_balance(p, mo);
}

/// The swim overlay: a real alternating front-crawl stroke and a flutter
/// kick, over the vanilla water pose, whenever a humanoid moves in water.
pub fn swim_stroke(p: &mut Poses, mo: &Motion) {
    if !mo.in_water || mo.speed < 0.03 {
        return;
    }
    let k = mo.k;
    let sw = mo.speed;
    let st = mo.t * 2.1 * PI * 2.0 + mo.phase;
    emf::add(p, "right_arm", 3, -0.55 * k * sw * (st.msin() * 0.5 + 0.5));
    emf::add(p, "left_arm", 3, -0.55 * k * sw * ((st + PI).msin() * 0.5 + 0.5));
    emf::add(p, "right_arm", 5, 0.3 * k * sw * (st + 0.8).msin());
    emf::add(p, "left_arm", 5, -0.3 * k * sw * (st + 0.8 + PI).msin());
    let kick = (mo.t * 5.0 * PI * 2.0 + mo.phase).msin() * 0.35 * k * sw;
    emf::add(p, "right_leg", 3, kick);
    emf::add(p, "left_leg", 3, -kick);
    emf::add(p, "body", 5, 0.08 * k * sw * (st * 0.5).msin());
}

/// Held-item stances: the sword ready pose, the drawn-bow tremble and the
/// shield brace.
pub fn stance(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    if mo.tool == Tool::Sword && mo.swing <= 0.0 {
        // The ready stance: the sword arm rides a little raised and in.
        emf::add(p, "right_arm", 3, -0.2 * k);
        emf::add(p, "right_arm", 5, -0.05 * k);
        emf::add(p, "body", 4, -0.06 * k);
    }
    if matches!(mo.tool, Tool::Bow | Tool::Crossbow) && mo.using {
        // Holding a draw is work: a fine, fast tremble in both arms.
        let tr = 0.02 * k * (mo.t * 37.0).msin() + 0.012 * k * (mo.t * 53.0 + 1.3).msin();
        emf::add(p, "right_arm", 3, tr);
        emf::add(p, "left_arm", 3, -tr);
        emf::add(p, "head", 5, 0.4 * tr);
    }
    if mo.blocking {
        // Braced behind the shield: hunched shoulders, chin down.
        emf::add(p, "body", 3, 0.09 * k);
        emf::add(p, "head", 3, 0.12 * k);
    }
    if mo.fishing && mo.swing <= 0.0 {
        // The rod out over the water: the rod arm up and forward, the
        // body turned a touch, patient.
        emf::add(p, "right_arm", 3, -0.85 * k);
        emf::add(p, "right_arm", 5, -0.12 * k);
        emf::add(p, "body", 4, -0.08 * k);
        if mo.bite {
            // A bite! The rod tip jerks and the whole arm fights it.
            let jerk = (mo.t * 8.0 * PI * 2.0).msin() * 0.14 * k;
            emf::add(p, "right_arm", 3, jerk);
            emf::add(p, "head", 3, 0.5 * jerk);
        }
    }
}

/// The dying collapse: limbs go slack and spread, the head lolls — an
/// additive limp over the vanilla death tip-over, easing in across the
/// twenty death ticks. Part names cover humanoids and quadrupeds alike
/// (absent parts are a no-op).
pub fn death_limp(p: &mut Poses, t01: f32, k: f32, phase: f32) {
    let u = t01.clamp(0.0, 1.0);
    let e = u * u * (3.0 - 2.0 * u);
    let kk = k * e;
    emf::add(p, "right_arm", 3, 0.45 * kk);
    emf::add(p, "left_arm", 3, 0.38 * kk);
    emf::add(p, "right_arm", 5, 0.35 * kk);
    emf::add(p, "left_arm", 5, -0.35 * kk);
    emf::add(p, "head", 5, 0.4 * kk * (0.6 + 0.4 * (phase).msin()));
    emf::add(p, "head", 3, 0.2 * kk);
    emf::add(p, "right_leg", 3, 0.2 * kk);
    emf::add(p, "left_leg", 3, -0.15 * kk);
    for leg in ["right_front_leg", "left_front_leg", "right_hind_leg", "left_hind_leg"] {
        emf::add(p, leg, 3, 0.3 * kk);
    }
    emf::add(p, "tail", 3, 0.3 * kk);
    emf::add(p, "real_tail", 3, 0.3 * kk);
    emf::add(p, "body", 5, 0.1 * kk);
}

/// Attack variation: each swing picks a different lateral path, axes chop
/// with more shoulder, picks and shovels drive with the body.
pub fn attack_variation(p: &mut Poses, mo: &Motion) {
    if mo.swing <= 0.0 || mo.swing >= 1.0 {
        return;
    }
    let k = mo.k;
    let env = (mo.swing * PI).msin();
    let path = [-0.16f32, 0.0, 0.16][(mo.swing_index % 3) as usize];
    emf::add(p, "right_arm", 4, path * k * env);
    match mo.tool {
        Tool::Axe => {
            emf::add(p, "body", 3, 0.12 * k * env);
            emf::add(p, "right_arm", 5, 0.1 * k * env);
        }
        Tool::Pickaxe | Tool::Shovel => emf::add(p, "body", 3, 0.08 * k * env),
        _ => {}
    }
}

/// Ledge balancing (own player): arms windmill out, the torso wobbles.
pub fn edge_balance(p: &mut Poses, mo: &Motion) {
    if !mo.on_edge || mo.idle < 0.3 {
        return;
    }
    let k = mo.k * mo.idle;
    let w = emf::drift(mo.t, 0.9, mo.phase);
    emf::add(p, "right_arm", 5, (0.9 + 0.25 * w) * k);
    emf::add(p, "left_arm", 5, (-0.9 + 0.25 * w) * k);
    emf::add(p, "body", 5, 0.06 * k * w);
    emf::add(p, "head", 5, -0.04 * k * w);
}

/// Weighted quadruped gait: leg follow-through, a trotting body bob, an
/// idle grazing head dip and the occasional ear-era tail flick.
pub fn quadruped(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let bounce = emf::gait_bounce(mo.pos, mo.speed, 0.0);
    for (part, sign) in [
        ("right_hind_leg", 1.0f32),
        ("left_hind_leg", -1.0),
        ("right_front_leg", -1.0),
        ("left_front_leg", 1.0),
    ] {
        emf::add(p, part, 3, emf::capped(0.09 * k * sign * bounce, 0.28));
    }
    // The trot bob: the body pitches gently with each stride pair.
    emf::add(p, "body", 3, emf::capped(0.035 * k * (mo.pos * 1.3324).msin() * mo.speed, 0.09));
    // Idle grazing: a slow, eased head dip that comes and goes.
    let idle = mo.idle * k;
    let graze = (emf::osc(mo.t, 0.05, mo.phase) - 0.55).max(0.0) / 0.45;
    emf::add(p, "head", 3, 0.38 * idle * graze * graze);
    emf::add(p, "head", 4, 0.07 * idle * emf::drift(mo.t, 0.06, mo.phase + 1.4));
    // A tail flick, on models that have one.
    emf::add(p, "tail", 4, 0.2 * idle * emf::drift(mo.t, 0.5, mo.phase).max(0.0));
}

/// A wagging tail for wolves, cats and foxes: eager while moving, a lazy
/// curl-sway at rest. (The wolf model names its articulated tail
/// `real_tail`, the cat its segments `tail1`/`tail2`: all written,
/// unknown parts are a no-op.)
pub fn tail_wag(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let eager = 0.25 + 0.75 * mo.speed;
    let wag = emf::capped(0.28 * k * eager * emf::osc(mo.t, 1.9, mo.phase), 0.45);
    let curl = 0.06 * k * mo.idle * emf::osc(mo.t, 0.23, mo.phase + 0.8);
    for tail in ["tail", "real_tail", "tail1", "tail2"] {
        emf::add(p, tail, 4, wag);
        emf::add(p, tail, 3, curl);
    }
}

/// Wolf idle windows: the play bow, two happy hops and ear flicks.
pub fn wolf(p: &mut Poses, mo: &Motion) {
    let idle = mo.idle * mo.k;
    // The play bow: chest down, front legs stretched, tail up.
    let bow = ((emf::osc(mo.t, 0.031, mo.phase + 4.2) - 0.75).max(0.0) / 0.25) * idle;
    emf::add(p, "upper_body", 3, 0.45 * bow);
    emf::add(p, "right_front_leg", 3, -0.7 * bow);
    emf::add(p, "left_front_leg", 3, -0.7 * bow);
    emf::add(p, "real_head", 3, -0.25 * bow);
    emf::add(p, "head", 3, -0.25 * bow);
    emf::add(p, "real_tail", 3, -0.5 * bow);
    emf::add(p, "tail", 3, -0.5 * bow);
    // Happy hops: a short double-bounce window.
    let gate = ((emf::osc(mo.t, 0.027, mo.phase + 1.1) - 0.85).max(0.0) / 0.15) * idle;
    let hop = gate * (mo.t * 7.0 * PI * 2.0).msin().max(0.0);
    emf::add(p, "right_front_leg", 3, -0.8 * hop);
    emf::add(p, "left_front_leg", 3, -0.8 * hop);
    emf::add(p, "upper_body", 3, -0.25 * hop);
    // Ear flicks (a no-op on models without ear parts).
    let flick = ((emf::osc(mo.t, 0.11, mo.phase + 2.7) - 0.9).max(0.0) / 0.1) * idle;
    emf::add(p, "left_ear", 5, -0.5 * flick * (mo.t * 15.0).msin());
    emf::add(p, "right_ear", 5, 0.5 * flick * (mo.t * 15.0 + 1.0).msin());
}

/// Cat idle windows: the loaf (legs folded under, tail curled around)
/// and a dozing head nod inside it.
pub fn cat(p: &mut Poses, mo: &Motion) {
    let idle = mo.idle * mo.k;
    let loaf = (((emf::osc(mo.t, 0.016, mo.phase + 3.3) - 0.62).max(0.0) / 0.38).min(1.0)) * idle;
    for leg in ["right_front_leg", "left_front_leg", "right_hind_leg", "left_hind_leg"] {
        emf::add(p, leg, 3, 1.25 * loaf);
    }
    emf::add(p, "body", 3, 0.1 * loaf);
    emf::add(p, "head", 3, 0.22 * loaf * (0.6 + 0.4 * emf::osc(mo.t, 0.11, mo.phase)));
    emf::add(p, "tail1", 4, 0.8 * loaf);
    emf::add(p, "tail2", 4, 0.9 * loaf);
}

/// Horse-family idle extras: the tail swish, a head toss and a front-hoof
/// stomp. (The articulated head group is `head_parts` on equine models.)
pub fn equine(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let idle = mo.idle * k;
    emf::add(p, "tail", 4, 0.35 * k * emf::drift(mo.t, 0.6, mo.phase));
    let toss = ((emf::osc(mo.t, 0.05, mo.phase + 0.9) - 0.86).max(0.0) / 0.14) * idle;
    emf::add(p, "head_parts", 3, -0.3 * toss);
    emf::add(p, "head_parts", 5, 0.15 * toss * (mo.t * 9.0).msin());
    let stomp = ((emf::osc(mo.t, 0.041, mo.phase + 2.2) - 0.88).max(0.0) / 0.12) * idle;
    emf::add(p, "right_front_leg", 3, -0.55 * stomp * (mo.t * 6.0 * PI).msin().max(0.0));
}

/// The creeper tiptoe: a nervous forward lean that grows with speed, legs
/// stepping lighter, the head tilting as it stalks — and a fuse tremble
/// that builds as it swells.
pub fn creeper(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let lean = mo.speed * (0.8 + 0.2 * (mo.pos * 1.3324).msin());
    emf::add(p, "body", 3, emf::capped(0.09 * k * lean, 0.16));
    let bounce = emf::gait_bounce(mo.pos, mo.speed, PI / 2.0);
    for (part, sign) in [
        ("right_hind_leg", 1.0f32),
        ("left_hind_leg", -1.0),
        ("right_front_leg", -1.0),
        ("left_front_leg", 1.0),
    ] {
        emf::add(p, part, 3, emf::capped(0.07 * k * sign * bounce, 0.2));
    }
    emf::add(p, "head", 5, 0.05 * k * emf::drift(mo.t, 0.11, mo.phase));
    emf::add(p, "head", 3, 0.04 * k * mo.idle * emf::drift(mo.t, 0.08, mo.phase + 2.0));
    if mo.tense > 0.0 {
        let tr = mo.tense * k;
        emf::add(p, "body", 5, 0.05 * tr * (mo.t * 31.0).msin());
        emf::add(p, "head", 5, 0.04 * tr * (mo.t * 43.0).msin());
    }
}

/// Spider idle leg waves: each leg lifts and circles in turn while the
/// spider rests, the way real spiders test the air.
pub fn spider(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let idle = mo.idle * k;
    for (i, n) in ["hind", "middle_hind", "middle_front", "front"].iter().enumerate() {
        let leg_phase = mo.phase + i as f32 * 1.9;
        // One leg at a time: the wave gates on the positive lobe only.
        let wave = (emf::osc(mo.t, 0.13, leg_phase) - 0.72).max(0.0) / 0.28;
        let lift = wave * (mo.t * 6.0 + leg_phase).msin() * 0.22 * idle;
        emf::add(p, &format!("right_{n}_leg"), 5, lift);
        emf::add(p, &format!("left_{n}_leg"), 5, -lift);
    }
    emf::add(p, "body", 3, 0.03 * idle * emf::osc(mo.t, 0.35, mo.phase));
    emf::add(p, "head", 3, 0.05 * idle * emf::drift(mo.t, 0.1, mo.phase + 1.1));
}

/// The chicken strut: the head pecks forward with each step, wings
/// flutter half-open at rest, the body bobbing with the walk.
pub fn chicken(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let peck = ((mo.pos * 1.3324).msin()).max(0.0) * mo.speed;
    emf::add(p, "head", 3, emf::capped(0.3 * k * peck, 0.4));
    emf::add(p, "body", 3, emf::capped(0.05 * k * (mo.pos * 1.3324 + PI).msin() * mo.speed, 0.1));
    let idle = mo.idle * k;
    // An idle ruffle: both wings flick out briefly, now and then.
    let ruffle = (emf::osc(mo.t, 0.09, mo.phase) - 0.8).max(0.0) / 0.2;
    let flick = ruffle * (mo.t * 9.0).msin().abs() * 0.5 * idle;
    emf::add(p, "right_wing", 5, flick);
    emf::add(p, "left_wing", 5, -flick);
    emf::add(p, "head", 4, 0.1 * idle * emf::drift(mo.t, 0.17, mo.phase + 0.7));
}

/// Villagers: a thoughtful head nod and wander, the crossed arms rising
/// with a slow breath, the nose twitching now and then.
pub fn villager(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    let idle = mo.idle * k;
    emf::add(p, "head", 3, 0.05 * idle * emf::drift(mo.t, 0.08, mo.phase));
    emf::add(p, "head", 4, 0.09 * idle * emf::drift(mo.t, 0.05, mo.phase + 1.9));
    emf::add(p, "arms", 3, 0.03 * idle * emf::osc(mo.t, 0.2, mo.phase));
    let twitch = (emf::osc(mo.t, 0.21, mo.phase + 3.0) - 0.9).max(0.0) / 0.1;
    emf::add(p, "nose", 5, 0.12 * idle * twitch * (mo.t * 11.0).msin());
    emf::add(p, "body", 5, 0.045 * k * (mo.pos * 0.6662).msin() * mo.speed);
}

/// Endermen: a slow, menacing sway, long arms drifting independently.
pub fn enderman(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    emf::add(p, "body", 4, 0.05 * k * emf::drift(mo.t, 0.05, mo.phase));
    emf::add(p, "right_arm", 5, 0.04 * k * emf::osc(mo.t, 0.16, mo.phase));
    emf::add(p, "left_arm", 5, -0.04 * k * emf::osc(mo.t, 0.16, mo.phase + 1.2));
    emf::add(p, "head", 3, 0.04 * k * mo.idle * emf::drift(mo.t, 0.09, mo.phase + 0.5));
    let bounce = emf::gait_bounce(mo.pos, mo.speed, 0.0);
    emf::add(p, "right_leg", 3, emf::capped(-0.08 * k * bounce, 0.25));
    emf::add(p, "left_leg", 3, emf::capped(0.08 * k * bounce, 0.25));
}

/// The wither: its three heads drift and scan independently, the ribcage
/// pulses with the hover, the tail sways.
pub fn wither(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    emf::add(p, "left_head", 4, 0.35 * k * emf::drift(mo.t, 0.09, mo.phase + 1.0));
    emf::add(p, "left_head", 3, 0.18 * k * emf::drift(mo.t, 0.07, mo.phase + 2.0));
    emf::add(p, "right_head", 4, 0.35 * k * emf::drift(mo.t, 0.08, mo.phase + 4.1));
    emf::add(p, "right_head", 3, 0.18 * k * emf::drift(mo.t, 0.06, mo.phase + 5.2));
    emf::add(p, "center_head", 3, 0.06 * k * emf::drift(mo.t, 0.05, mo.phase));
    emf::add(p, "ribcage", 3, 0.05 * k * emf::osc(mo.t, 0.22, mo.phase));
    emf::add(p, "tail", 3, 0.25 * k * emf::drift(mo.t, 0.13, mo.phase + 0.7));
}

/// The happy ghast: a slow, floaty whole-body sway as it drifts.
pub fn happy_ghast_sway(p: &mut Poses, mo: &Motion) {
    let k = mo.k;
    emf::add(p, "body", 3, 0.07 * k * emf::drift(mo.t, 0.07, mo.phase));
    emf::add(p, "body", 5, 0.09 * k * emf::drift(mo.t, 0.055, mo.phase + 2.0));
}

/// Everything else alive gets the ambient layer: breathing and a wandering
/// gaze on whatever parts the model actually has.
pub fn ambient(p: &mut Poses, mo: &Motion) {
    let idle = mo.idle * mo.k;
    emf::add(p, "head", 3, 0.04 * idle * emf::drift(mo.t, 0.07, mo.phase));
    emf::add(p, "head", 4, 0.06 * idle * emf::drift(mo.t, 0.045, mo.phase + 2.3));
    emf::add(p, "body", 3, 0.015 * idle * emf::osc(mo.t, 0.18, mo.phase));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobs::Part;
    use std::sync::Mutex;

    /// Tests that mutate the global flag statics serialize on this lock
    /// (the test harness runs tests in parallel).
    static FLAGS_LOCK: Mutex<()> = Mutex::new(());

    fn humanoid_poses() -> Poses {
        let part = |name: &str| Part {
            name: name.into(),
            pose: [0.0; 9],
            polygons: Vec::new(),
            children: Vec::new(),
        };
        let mut root = part("body");
        root.children = ["head", "right_arm", "left_arm", "right_leg", "left_leg"]
            .map(part)
            .into();
        Poses::new(&root)
    }
    fn quad_poses() -> Poses {
        let part = |name: &str| Part {
            name: name.into(),
            pose: [0.0; 9],
            polygons: Vec::new(),
            children: Vec::new(),
        };
        let mut root = part("body");
        root.children = [
            "head",
            "right_hind_leg",
            "left_hind_leg",
            "right_front_leg",
            "left_front_leg",
            "tail",
        ]
        .map(part)
        .into();
        Poses::new(&root)
    }
    fn motion(t: f32, pos: f32, speed: f32) -> Motion {
        Motion {
            t,
            pos,
            speed,
            k: 1.0,
            phase: 0.7,
            idle: (1.0 - speed * 3.0).clamp(0.0, 1.0),
            swing: 0.0,
            swing_index: 0,
            in_water: false,
            using: false,
            blocking: false,
            on_edge: false,
            tense: 0.0,
            tool: Tool::None,
            fishing: false,
            bite: false,
        }
    }

    #[test]
    fn walking_humanoids_gain_visible_follow_through() {
        let mut p = humanoid_poses();
        humanoid(&mut p, &motion(3.0, 2.2, 1.0), false);
        let moved = ["right_arm", "left_arm", "right_leg", "left_leg", "body"]
            .iter()
            .map(|n| p.get(n)[3].abs() + p.get(n)[4].abs() + p.get(n)[5].abs())
            .sum::<f32>();
        assert!(moved > 0.1, "a walking pose visibly changes: {moved}");
        // Arms counter-phase: follow-through pulls them apart, not together.
        let right = p.get("right_arm")[3];
        let left = p.get("left_arm")[3];
        assert_ne!(right, left);
    }

    #[test]
    fn idle_humanoids_breathe_and_glance() {
        let mut p = humanoid_poses();
        humanoid(&mut p, &motion(5.3, 0.0, 0.0), false);
        let head = p.get("head");
        assert!(head[3].abs() + head[4].abs() > 0.004, "the gaze wanders");
        assert!(p.get("right_arm")[5].abs() > 0.001, "arms breathe");
        // And the idle life actually moves over time (not a frozen offset).
        let mut later = humanoid_poses();
        humanoid(&mut later, &motion(9.1, 0.0, 0.0), false);
        assert_ne!(p.get("head")[4], later.get("head")[4]);
    }

    #[test]
    fn zombies_reach_more_than_skeletons() {
        let mut z = humanoid_poses();
        humanoid(&mut z, &motion(4.0, 1.0, 0.6), true);
        let mut s = humanoid_poses();
        humanoid(&mut s, &motion(4.0, 1.0, 0.6), false);
        let arm = |p: &mut Poses| p.get("right_arm")[3].abs() + p.get("left_arm")[3].abs();
        assert!(arm(&mut z) != arm(&mut s), "the undead reach is its own layer");
    }

    #[test]
    fn swimmers_stroke_and_kick() {
        let mut dry = humanoid_poses();
        humanoid(&mut dry, &motion(3.0, 1.5, 0.8), false);
        let mut wet = humanoid_poses();
        let mut mo = motion(3.0, 1.5, 0.8);
        mo.in_water = true;
        humanoid(&mut wet, &mo, false);
        let arms = |p: &mut Poses| p.get("right_arm")[3].abs() + p.get("left_arm")[3].abs();
        assert_ne!(arms(&mut dry), arms(&mut wet), "water adds the stroke");
        // The stroke alternates: both arms never share the same x rotation.
        let right = wet.get("right_arm")[3];
        let left = wet.get("left_arm")[3];
        assert_ne!(right, left);
        // And it stays within a sane budget everywhere.
        for ti in 0..40 {
            let mut p = humanoid_poses();
            let mut mo = motion(ti as f32 * 0.21, 2.0, 1.0);
            mo.in_water = true;
            humanoid(&mut p, &mo, false);
            for n in ["head", "body", "right_arm", "left_arm", "right_leg", "left_leg"] {
                for a in 3..6 {
                    assert!(p.get(n)[a].abs() < 1.2, "{n}[{a}] swim budget");
                }
            }
        }
    }

    #[test]
    fn stances_follow_the_held_tool() {
        let mut sword = humanoid_poses();
        let mut mo = motion(2.0, 0.0, 0.0);
        mo.tool = Tool::Sword;
        humanoid(&mut sword, &mo, false);
        let mut bare = humanoid_poses();
        humanoid(&mut bare, &motion(2.0, 0.0, 0.0), false);
        assert!(
            sword.get("right_arm")[3] < bare.get("right_arm")[3],
            "the sword arm rides raised"
        );
        // A drawn bow trembles; an undrawn one does not.
        let mut drawn = humanoid_poses();
        let mut mo = motion(2.0, 0.0, 0.0);
        mo.tool = Tool::Bow;
        mo.using = true;
        humanoid(&mut drawn, &mo, false);
        let mut held = humanoid_poses();
        mo.using = false;
        humanoid(&mut held, &mo, false);
        assert_ne!(drawn.get("right_arm")[3], held.get("right_arm")[3]);
        // Blocking hunches the body.
        let mut block = humanoid_poses();
        let mut mo = motion(2.0, 0.0, 0.0);
        mo.blocking = true;
        humanoid(&mut block, &mo, false);
        assert!(block.get("body")[3] > bare.get("body")[3]);
    }

    #[test]
    fn swings_vary_by_index_and_tool() {
        let swing_at = |index: u32, tool: Tool| {
            let mut p = humanoid_poses();
            let mut mo = motion(2.0, 0.0, 0.0);
            mo.swing = 0.5;
            mo.swing_index = index;
            mo.tool = tool;
            humanoid(&mut p, &mo, false);
            (p.get("right_arm")[4], p.get("body")[3])
        };
        let (a, _) = swing_at(0, Tool::None);
        let (b, _) = swing_at(1, Tool::None);
        let (c, _) = swing_at(2, Tool::None);
        assert!(a != c && a != b, "three swing paths cycle: {a} {b} {c}");
        let (_, bare_body) = swing_at(1, Tool::None);
        let (_, axe_body) = swing_at(1, Tool::Axe);
        assert!(axe_body > bare_body, "the axe chop drives the body");
    }

    #[test]
    fn ledge_balancing_windmills_the_arms() {
        let mut p = humanoid_poses();
        let mut mo = motion(2.0, 0.0, 0.0);
        mo.on_edge = true;
        humanoid(&mut p, &mo, false);
        assert!(p.get("right_arm")[5] > 0.5, "arms out");
        assert!(p.get("left_arm")[5] < -0.5, "both of them");
        // Off the ledge, nothing of the sort.
        let mut q = humanoid_poses();
        humanoid(&mut q, &motion(2.0, 0.0, 0.0), false);
        assert!(q.get("right_arm")[5].abs() < 0.1);
    }

    #[test]
    fn tool_of_classifies_the_held_item() {
        assert_eq!(tool_of(Some("minecraft:diamond_sword")), Tool::Sword);
        assert_eq!(tool_of(Some("minecraft:golden_axe")), Tool::Axe);
        assert_eq!(tool_of(Some("minecraft:iron_pickaxe")), Tool::Pickaxe);
        assert_eq!(tool_of(Some("minecraft:bow")), Tool::Bow);
        assert_eq!(tool_of(Some("minecraft:crossbow")), Tool::Crossbow);
        assert_eq!(tool_of(Some("minecraft:trident")), Tool::Trident);
        assert_eq!(tool_of(Some("minecraft:cobblestone")), Tool::None);
        assert_eq!(tool_of(None), Tool::None);
    }

    #[test]
    fn every_channel_stays_within_the_additive_budget() {
        // Sweep phases and speeds: no single routine may bend any part
        // more than half a radian, so vanilla poses stay recognisable.
        for ti in 0..40 {
            let t = ti as f32 * 0.37;
            for (pos, speed) in [(0.0, 0.0), (1.3, 0.4), (4.1, 1.0), (7.7, 1.0)] {
                for zombie in [false, true] {
                    let mut p = humanoid_poses();
                    humanoid(&mut p, &motion(t, pos, speed), zombie);
                    for n in ["head", "body", "right_arm", "left_arm", "right_leg", "left_leg"] {
                        for a in 3..6 {
                            assert!(
                                p.get(n)[a].abs() < 0.5,
                                "{n}[{a}] within budget at t={t} pos={pos}: {}",
                                p.get(n)[a]
                            );
                        }
                    }
                }
                let mut q = quad_poses();
                quadruped(&mut q, &motion(t, pos, speed));
                tail_wag(&mut q, &motion(t, pos, speed));
                for n in ["head", "body", "tail", "right_hind_leg", "left_front_leg"] {
                    for a in 3..6 {
                        assert!(q.get(n)[a].abs() < 0.75, "{n}[{a}] quadruped budget");
                    }
                }
                // The idle-window families stay within their own budgets.
                for (i, routine) in
                    [wolf as fn(&mut Poses, &Motion), cat, equine].iter().enumerate()
                {
                    let mut w = quad_poses();
                    routine(&mut w, &motion(t, pos, speed));
                    for n in ["head", "body", "tail", "right_front_leg", "left_front_leg"] {
                        for a in 3..6 {
                            assert!(w.get(n)[a].abs() < 1.6, "{i}:{n}[{a}] idle-window budget");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn quadrupeds_graze_and_wag_when_idle() {
        // Find a moment the slow graze gate is open.
        let grazed = (0..600).any(|i| {
            let mut q = quad_poses();
            quadruped(&mut q, &motion(i as f32 * 0.25, 0.0, 0.0));
            q.get("head")[3] > 0.1
        });
        assert!(grazed, "the grazing dip happens within a couple of minutes");
        let mut q = quad_poses();
        quadruped(&mut q, &motion(2.0, 0.0, 0.0));
        tail_wag(&mut q, &motion(2.0, 0.0, 0.0));
        assert!(q.get("tail")[4].abs() > 0.001, "the tail is alive");
    }

    #[test]
    fn idle_windows_eventually_open_for_each_family() {
        let wolf_bowed = (0..2400).any(|i| {
            let mut q = quad_poses();
            wolf(&mut q, &motion(i as f32 * 0.25, 0.0, 0.0));
            q.get("right_front_leg")[3].abs() > 0.1
        });
        assert!(wolf_bowed, "a wolf play-bows or hops within ten minutes");
        let cat_loafed = (0..2400).any(|i| {
            let mut q = quad_poses();
            cat(&mut q, &motion(i as f32 * 0.25, 0.0, 0.0));
            q.get("right_front_leg")[3] > 0.4
        });
        assert!(cat_loafed, "a cat loafs within ten minutes");
        let horse_moved = (0..2400).any(|i| {
            let mut q = quad_poses();
            equine(&mut q, &motion(i as f32 * 0.25, 0.0, 0.0));
            q.get("right_front_leg")[3].abs() > 0.1 || q.get("tail")[4].abs() > 0.1
        });
        assert!(horse_moved, "a horse swishes or stomps within ten minutes");
    }

    #[test]
    fn creepers_tremble_as_they_swell() {
        let mut calm = quad_poses();
        creeper(&mut calm, &motion(3.0, 0.0, 0.0));
        let mut fused = quad_poses();
        let mut mo = motion(3.0, 0.0, 0.0);
        mo.tense = 1.0;
        creeper(&mut fused, &mo);
        assert_ne!(calm.get("body")[5], fused.get("body")[5], "the fuse trembles");
    }

    #[test]
    fn per_entity_personality_desynchronises_the_herd() {
        let a = emf::personality(10);
        let b = emf::personality(11);
        let mut pa = quad_poses();
        let mut pb = quad_poses();
        let mo = |per: emf::Personality| {
            let mut m = motion(0.0, 0.0, 0.0);
            m.t = 6.0 * per.tempo;
            m.k = per.amp;
            m.phase = per.phase;
            m.idle = 1.0;
            m
        };
        quadruped(&mut pa, &mo(a));
        quadruped(&mut pb, &mo(b));
        assert_ne!(
            pa.get("head")[4],
            pb.get("head")[4],
            "two idle animals hold different poses"
        );
    }

    #[test]
    fn flags_round_trip_and_gate() {
        let _g = FLAGS_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = flags();
        set_flags(Flags {
            enabled: true,
            players: false,
            mobs: true,
            rain: false,
            swap: false,
            totem: 1,
            intensity: 2,
        });
        assert_eq!(
            flags(),
            Flags {
                enabled: true,
                players: false,
                mobs: true,
                rain: false,
                swap: false,
                totem: 1,
                intensity: 2,
            }
        );
        set_flags(Flags {
            enabled: true,
            players: true,
            mobs: true,
            rain: true,
            swap: true,
            totem: 9,
            intensity: 9,
        });
        assert_eq!(flags().intensity, 2, "intensity clamps to the names table");
        assert_eq!(flags().totem, 2, "the totem dial clamps to its names table");
        set_suspended(true);
        assert!(!active(), "the row toggle suspends everything");
        set_suspended(false);
        assert!(active());
        set_flags(before);
    }

    #[test]
    fn dolphin_tint_and_totem_and_shrink_gates() {
        let _g = FLAGS_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = flags();
        let mut f = flags();
        f.enabled = true;
        f.mobs = true;
        f.totem = 1;
        set_flags(f);
        set_suspended(false);
        // About 1 in 100 dolphins goes pink, stable per id.
        let pink = (0..3000u32).find(|&id| dolphin_tint(id) != 0xFFFF_FFFF);
        let pink = pink.expect("a pink dolphin exists in three thousand");
        assert_eq!(dolphin_tint(pink), 0xFFC9_AFFF, "stable per entity");
        let rate = (0..3000u32).filter(|&id| dolphin_tint(id) != 0xFFFF_FFFF).count();
        assert!((10..=80).contains(&rate), "about 1%, got {rate}/3000");
        // Off while suspended.
        set_suspended(true);
        assert_eq!(dolphin_tint(pink), 0xFFFF_FFFF, "suspended: vanilla");
        assert_eq!(totem_mode(), 0, "suspended: full totem");
        set_suspended(false);
        assert_eq!(totem_mode(), 1, "the dial reads back");
        // The first-person shrink gate is frame-fed and defaults off.
        assert!(!fp_shrink_on(), "defaults off: pose fixtures never move");
        set_fp_shrink(true);
        assert!(fp_shrink_on());
        set_fp_shrink(false);
        set_flags(before);
    }

    #[test]
    fn dying_bodies_go_limp_within_budget() {
        let mut p = humanoid_poses();
        death_limp(&mut p, 0.0, 1.0, 0.7);
        let start: f32 = (3..6).map(|a| p.get("right_arm")[a].abs()).sum();
        assert!(start < 1e-6, "the limp eases in from nothing");
        let mut q = humanoid_poses();
        death_limp(&mut q, 1.0, 1.0, 0.7);
        assert!(q.get("right_arm")[3] > 0.2, "arms slacken");
        assert!(q.get("head")[5].abs() > 0.02, "the head lolls");
        for t in 0..20 {
            let mut r = humanoid_poses();
            death_limp(&mut r, t as f32 / 19.0, 1.45, 2.0);
            for n in ["head", "body", "right_arm", "left_arm", "right_leg", "left_leg"] {
                for a in 3..6 {
                    assert!(r.get(n)[a].abs() < 1.0, "{n}[{a}] limp budget");
                }
            }
        }
    }

    #[test]
    fn glow_pulse_brightens_only_while_the_mob_layer_is_on() {
        let _g = FLAGS_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = flags();
        let sky = 0x60u32 << 8;
        let dim = sky | 0x20;
        set_suspended(true);
        assert_eq!(glow_pulse(7, 100.0, dim), dim, "suspended: untouched");
        set_suspended(false);
        let mut f = flags();
        f.enabled = true;
        f.mobs = true;
        set_flags(f);
        let lit = glow_pulse(7, 100.0, dim);
        assert_eq!(lit >> 8, dim >> 8, "sky bits preserved");
        assert!((lit & 0xFF) >= 150 && (lit & 0xFF) <= 240, "block light pulses bright");
        // The pulse actually moves over time.
        let a = glow_pulse(7, 40.0, dim) & 0xFF;
        let b = glow_pulse(7, 47.0, dim) & 0xFF;
        assert_ne!(a, b, "a living pulse, not a constant");
        // And it never darkens an already-bright squid.
        let bright = sky | 0xF0;
        assert_eq!(glow_pulse(7, 40.0, bright) & 0xFF, 0xF0);
        set_flags(before);
    }

    #[test]
    fn fishing_raises_the_rod_arm_and_bites_jerk_it() {
        let mut idle_pose = humanoid_poses();
        humanoid(&mut idle_pose, &motion(2.0, 0.0, 0.0), false);
        let mut rod = humanoid_poses();
        let mut mo = motion(2.0, 0.0, 0.0);
        mo.fishing = true;
        humanoid(&mut rod, &mo, false);
        assert!(
            rod.get("right_arm")[3] < idle_pose.get("right_arm")[3] - 0.5,
            "the rod arm rides up"
        );
        let mut mo2 = motion(2.03, 0.0, 0.0);
        mo2.fishing = true;
        let mut rod2 = humanoid_poses();
        humanoid(&mut rod2, &mo2, false);
        let mut bite = humanoid_poses();
        mo2.bite = true;
        humanoid(&mut bite, &mo2, false);
        let delta = (bite.get("right_arm")[3] - rod2.get("right_arm")[3]).abs();
        assert!(delta > 0.02, "the bite jerks the rod arm: {delta}");
        // The feed round-trips.
        set_fishing(&[42], true);
        assert!(is_fishing(42));
        assert!(!is_fishing(43));
        set_fishing(&[], false);
        assert!(!is_fishing(42));
    }

    #[test]
    fn frame_feeds_round_trip() {
        set_env(true);
        assert!(NIGHT_MOON.load(Ordering::Relaxed));
        set_env(false);
        assert!(!NIGHT_MOON.load(Ordering::Relaxed));
        set_own_edge(true);
        assert!(OWN_EDGE.load(Ordering::Relaxed));
        set_own_edge(false);
        set_own_crouch(true);
        assert!(OWN_CROUCH.load(Ordering::Relaxed));
        set_own_crouch(false);
    }
}
