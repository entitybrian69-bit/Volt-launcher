// Horizon glue. This is an integration preview, not complete Punchy renderer parity.
use super::*;
use glam::{EulerRot, Mat4, Quat, Vec3};
use punchy_engine::player::Playback;
use punchy_engine::{Clip, Pose, actions as action, phases, resources};
use std::{cell::RefCell, collections::{BTreeMap, VecDeque}, sync::Arc};

pub(super) const ID: &str = "punchy";
pub(super) const TITLE: &str = "Punchy";
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
#[derive(Default)]
struct State {
    attempted: bool,
    source: Option<resources::Resources>,
    clips: BTreeMap<String, Arc<Clip>>,
    machine: action::ActionMachine,
    consume: phases::ConsumeController,
    mining: phases::MiningController,
    tick: i64,
    playback_id: u64,
    stack_keys: Vec<ItemStack>,
    forced_stack: Option<(usize, ItemStack)>,
    consume_hand: action::Hand,
    active: Option<Playback>,
    previous_swings: u32,
    use_down: bool,
    successful_use: VecDeque<action::Hand>,
    item_kinds: [Option<String>; 2],
    mining_attempt: bool,
    offhand_item: bool,
    inspect: bool,
    last_items: [String; 2],
    error: Option<String>,
    main_left: bool,
    render_alpha: f32,
    enabled: bool,
    tuning: Option<punchy_engine::tuning::Tuning>,
    status: String,
    render_count: u64,
}
struct Catalog<'a> {
    state: &'a State,
}
impl action::ClipCatalog for Catalog<'_> {
    fn resolve(&self, name: &str, hand: action::Hand) -> Option<action::ClipMeta> {
        let left = self.state.main_left != (hand == action::Hand::Off);
        let side = if left { "left" } else { "right" };
        let handed = format!("{name}_{side}");
        let aliases: &[&str] = match name {
            "use_drink" => &["use_drink2", "drink_loop", "drink_loop2"],
            "use_shield" => &["use_shield"],
            "mining" => &[
                "mining_foward",
                "mining_forward",
                "mining_sideways",
                "mining_digging",
            ],
            "inspect" => &["new_inspect_right", "new_inspect_left"],
            _ => &[],
        };
        let mapped = tuning_animation(self.state, hand, name);
        let custom = mapped
            .as_ref()
            .and_then(|m| m.name.as_deref())
            .and_then(|n| {
                punchy_engine::tuning::handed_candidates(n, left)
                    .into_iter()
                    .find_map(|n| self.state.clips.get(&n))
            });
        let clip = custom
            .or_else(|| self.state.clips.get(&handed))
            .or_else(|| self.state.clips.get(name))
            .or_else(|| aliases.iter().find_map(|n| self.state.clips.get(*n)))?;
        Some(action::ClipMeta {
            name: clip.name.clone(),
            length: clip.length,
            hold_on_last_frame: clip.hold_on_last_frame,
            dual_handed: punchy_engine::policy::dual_handed(
                clip,
                0.,
                self.state.offhand_item,
                self.state.offhand_item,
            ),
        })
    }
}
fn load(state: &mut State) {
    if state.attempted {
        return;
    }
    state.attempted = true;
    let result = resources::embedded().and_then(|r| {
        let clips = punchy_engine::load_clips(&r.animations)?;
        Ok((r, clips))
    });
    match result {
        Ok((source, clips)) => {
            state.status = format!("{} clips loaded", clips.len());
            state.clips = clips.into_iter().map(|(k, v)| (k, Arc::new(v))).collect();
            state.tuning = punchy_engine::tuning::Tuning::parse(&source.tuning).ok();
            state.source = Some(source);
            state.error = None;
        }
        Err(e) => {
            state.status = "Bundled animations could not load".into();
            state.error = Some(e);
        }
    }
}
fn enabled(game: &Game<'_>) -> bool {
    game.native_on(ID)
}
/// The player's animation speed in Punchy's own units: 1.0 = the mod's
/// normal pace, lower is slower, 0.1 the slowest. Stored as set.
fn ui_speed(game: &Game<'_>) -> f32 {
    let stored = game
        .native_setting(ID, "speed")
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|s| s.is_finite())
        .unwrap_or(1.0);
    // Builds before the 1.0-normal scale stored the engine value (0.5–5.5,
    // 5.5 = normal); a stored value above the player scale is one of those
    // and simply means "normal" again.
    if stored > 1.0 { 1.0 } else { stored.clamp(0.1, 1.0) }
}
fn global_speed(game: &Game<'_>) -> f32 {
    // The engine's configured scale puts normal pace at 5.5: ordinary
    // attack clips play at `speed ×` and the use/hold clips at `1 ÷ speed`,
    // so only 5.5 gives the mod's intended pace on both paths. The player
    // dials 1.0 = normal; map it onto the engine scale here, once, so both
    // paths stay consistent at every setting.
    (ui_speed(game) * 5.5).clamp(0.5, 5.5)
}
fn hand(h: Hand) -> action::Hand {
    if h == Hand::Main {
        action::Hand::Main
    } else {
        action::Hand::Off
    }
}
/// A float setting with a default, clamped to `lo..=hi`.
fn tweak_value(game: &Game<'_>, key: &str, default: f32, lo: f32, hi: f32) -> f32 {
    game.native_setting(ID, key)
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(default)
        .clamp(lo, hi)
}
/// The player's per-hand viewmodel tweak (Punchy's hand settings): size and
/// a view-space offset, for the physical `left`/right hand. Stored per
/// logical hand (main/off), like the settings page edits them.
fn hand_tweak(game: &Game<'_>, left: bool) -> Mat4 {
    let logical = if game.settings.main_hand_left != left {
        "off"
    } else {
        "main"
    };
    let size = tweak_value(game, &format!("hand_{logical}_size"), 1., 0.7, 1.3);
    let x = tweak_value(game, &format!("hand_{logical}_x"), 0., -0.25, 0.25);
    let y = tweak_value(game, &format!("hand_{logical}_y"), 0., -0.25, 0.25);
    let z = tweak_value(game, &format!("hand_{logical}_z"), 0., -0.25, 0.25);
    if size == 1. && x == 0. && y == 0. && z == 0. {
        return Mat4::IDENTITY;
    }
    // View space: +x sideways, +y up, -z forward (further from the camera).
    Mat4::from_translation(Vec3::new(x, y, -z)) * Mat4::from_scale(Vec3::splat(size))
}
fn tuning_animation(
    state: &State,
    hand: action::Hand,
    action_name: &str,
) -> Option<punchy_engine::tuning::Animation> {
    let index = hand.index();
    let item = &state.last_items[index];
    let context = punchy_engine::tuning::ItemContext {
        id: item,
        kind: state.item_kinds[index].as_deref(),
        ..Default::default()
    };
    state.tuning.as_ref()?.animation_for(
        &context,
        action_name,
        state.main_left != (hand == action::Hand::Off),
    )
}

pub(super) fn tick(game: &mut Game<'_>) {
    if !game.native_window() {
        return;
    }
    if !enabled(game) || game.player.is_none() {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.active = None;
            s.machine = Default::default();
            s.consume = Default::default();
            s.mining = Default::default();
            s.forced_stack = None;
            s.successful_use.clear();
            s.mining_attempt = false;
            s.previous_swings = game.swings_started;
        });
        return;
    }
    let using = game.used_hand();
    let items = [game.held().cloned(), game.offhand.clone()];
    let use_down = game.native_mapping("key.use").is_some_and(|m| m.0);
    let attack_down = game.native_mapping("key.attack").is_some_and(|m| m.0);
    let speed = global_speed(game);
    STATE.with(|cell| {
        let mut s=cell.borrow_mut();load(&mut s);
        if s.source.is_none() {return;}
        s.main_left=game.settings.main_hand_left;
        s.offhand_item=items[1].is_some();
        let mut input=action::Input { animation_speed:speed, use_down,
            mining:game.mining.is_some(),
            entity_target:game.pick_entity_hit().flatten().is_some(),
            block_target:game.pick().is_some(),
            inspect:std::mem::take(&mut s.inspect),
            ..Default::default() };
        let changed=game.swings_started!=s.previous_swings;
        s.previous_swings=game.swings_started;
        let mining_attempt=std::mem::take(&mut s.mining_attempt);
        input.attack=(changed && !input.mining && !mining_attempt).then_some(hand(game.swing.attack_hand()));
        input.mining_trigger=mining_attempt || changed && input.mining;
        let use_hand=hand(using.unwrap_or(Hand::Main));
        let was_use_down=s.use_down;
        s.use_down=use_down;
        for (i,stack) in items.iter().enumerate() {
            input.hands[i].kind=stack.as_ref().map_or(action::ItemKind::Empty,|v|action::ItemKind::from_id(&v.item));
            input.hands[i].crossbow_charged=stack.as_ref().and_then(|v|v.data()).is_some_and(|d|!d.charged.is_empty());
            let id=stack.as_ref().map_or("",|v|v.item.as_ref()).to_owned();
            s.item_kinds[i] = stack.as_ref().and_then(|stack| {
                game.items()
                    .get(&stack.item)
                    .filter(|info| info.block.is_some())
                    .map(|_| "FULL_BLOCK".to_owned())
            });
            if id!=s.last_items[i] {
                input.hotbar_changed|=i==0;
                input.equip=Some(action::EquipEvent {hand:if i==0 {action::Hand::Main}else{action::Hand::Off},entering:!id.is_empty()});
                s.last_items[i]=id;
            }
        }
        let input_driven=tuning_animation(&s,use_hand,"useItem").is_some_and(|e|e.trigger_on_input);
        input.use_trigger=s.successful_use.pop_front().or_else(|| (use_down && !was_use_down && input_driven).then_some(use_hand));
        if let Some(h)=using {
            let i=usize::from(h==Hand::Off);
            if let Some(stack)=items[i].as_ref() {
                let owner=game.player_owner(Some(stack));
                let mut kind=input.hands[i].kind;
                if kind==action::ItemKind::Other {
                    let components=game.pack.item_renderer.models.components(&horizon_client::item_render::stack_view(stack));
                    let mut animation=components.get("minecraft:consumable").map_or(horizon_world::consumable::UseAnimation::None,horizon_client::first_person::consumable_animation);
                    game.native_use_animation(stack,&mut animation);
                    kind=match animation {horizon_world::consumable::UseAnimation::Eat=>action::ItemKind::Eat,
                        horizon_world::consumable::UseAnimation::Drink=>action::ItemKind::Drink,_=>kind};
                }
                input.using=Some(action::UseState {hand:hand(h),kind,
                    elapsed_ticks:owner.use_duration-owner.use_item_remaining_ticks-1,
                    charge_duration_ticks:owner.crossbow_charge_duration as i32,..Default::default()});
            }
        }
        if s.tuning.is_some() {
            let h=input.attack.unwrap_or(action::Hand::Main);
            if let Some(entry)=tuning_animation(&s,h,"attack") {
                input.attack_options.custom_entry_present=true;
                input.attack_options.combo=if entry.variations.is_empty(){entry.name.into_iter().collect()}else{entry.variations};
                input.attack_options.critical_clip=entry.critical;
                input.attack_options.trigger_on_input=entry.trigger_on_input;
                input.attack_options.skip_blend_in=entry.skip_blend_in;
            }
        }
        input.has_bow_projectile_or_creative=game.creative || game.projectile_slot("minecraft:bow").is_some();
        input.bow_shot_configured=s.tuning.as_ref().is_some_and(|t|s.last_items.iter().enumerate().any(|(i,id)|t.animation(id,"use_bow_shot",s.main_left!=(i==1)).is_some()));
        if let Some(p)=&s.active {
            input.playback_active=!p.stopping;
            input.playback_clip=Some(p.clip.name.clone());input.playback_hand=Some(p.hand);
        }

        let phase_request = phase_tick(&mut s, &mut input, &items, speed, attack_down);

        // Split ownership to let the catalog borrow clip data during arbitration.
        let mut machine=std::mem::take(&mut s.machine);
        let intent=machine.tick(&input,&Catalog{state:&s});s.machine=machine;
        let core_won = intent.is_some();
        if let Some(intent)=intent {
            match intent.operation {
                action::Operation::Play(p) => {
                    if let Some(clip)=s.clips.get(&p.clip.name).cloned() {

                        s.status=format!("Playing {}",clip.name);
                        s.playback_id=s.playback_id.wrapping_add(1);
                        let from=s.active.take().map(Box::new);
                        s.active=Some(Playback{from,out_weight:1.,previous_out_weight:1.,id:s.playback_id,expression:p.start_time,previous_expression:p.start_time,return_fade:0.28,skip_tick:false,clip,hand:p.hand,previous:p.start_time,time:p.start_time,
                            multiplier:p.speed_multiplier,looping:p.looping,released:false,hold:p.hold_on_last_frame,
                            fade:p.blend_seconds.unwrap_or(0.2).max(0.001),stopping:false,
                            weight:if p.skip_blend_in {1.0}else{0.0},previous_weight:if p.skip_blend_in {1.}else{0.}});
                    }
                }
                action::Operation::Stop{hand,mode} => {
                    if let Some(p)=s.active.as_mut().filter(|p|p.hand==hand) {
                        p.looping=false;p.released=true;p.hold=false;
                        if let action::StopMode::BlendOut(seconds)=mode {p.stopping=true;p.return_fade=seconds.max(0.001);}
                    }
                }
            }
        }
        if !core_won {
            if let Some((consume, request)) = phase_request { apply_phase(&mut s, consume, &request, speed); }
        }
        if let Some(p)=s.active.as_mut() {
            p.advance(speed);
            if p.stopping && p.out_weight<=0. {s.active=None;}
        }
        if let Some(path)=std::env::var_os("PUNCHY_DIAGNOSTICS") {
            let value=serde_json::json!({"loaded":s.source.is_some(),"resource_source":"embedded","clips":s.clips.len(),"error":s.error,
                "active":s.active.as_ref().map(|p|serde_json::json!({"name":p.clip.name,"time":p.time,"weight":p.weight,"speed":punchy_engine::playback::rate(&p.clip,p.time,speed,p.multiplier,1.)})),
                "swings":s.previous_swings,"mining":input.mining,"consume_phase":format!("{:?}",s.consume.phase()),"mining_phase":format!("{:?}",s.mining.phase())});
            if let Ok(bytes)=serde_json::to_vec_pretty(&value) {let _=std::fs::write(path,bytes);}
        }
    });
}

pub(super) fn key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_playing() || !enabled(game) {
        return;
    }
    if key == game.native_mod_key(ID, "inspect", "key.keyboard.f6") {
        STATE.with(|s| s.borrow_mut().inspect = true);
    }
    if key == game.native_mod_key(ID, "reload", "key.keyboard.f7") {
        STATE.with(|s| *s.borrow_mut() = State::default());
    }
}
pub(super) fn settings(game: &Game<'_>, buttons: &mut Vec<String>) {
    buttons.push(format!("Animation speed: {:.1}", ui_speed(game)));
    buttons.push("Reload animations".into());
    buttons.push(format!(
        "Hand poses: {}",
        if game.native_setting(ID, "hand_poses").as_deref() == Some("horizon") {
            "Horizon"
        } else {
            "Punchy"
        }
    ));
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        load(&mut s);
        buttons.push(s.error.clone().unwrap_or_else(|| s.status.clone()));
    });
    // Punchy's hand viewmodel settings: size and view-space offsets per hand.
    let sel = selected_hand(game);
    buttons.push(format!(
        "Edit hand: {}",
        if sel == "off" { "Off hand" } else { "Main hand" }
    ));
    buttons.push(format!(
        "Hand size: {:.0}%",
        tweak_value(game, &format!("hand_{sel}_size"), 1., 0.7, 1.3) * 100.
    ));
    for (label, axis) in [("sideways", "x"), ("height", "y"), ("distance", "z")] {
        buttons.push(format!(
            "Hand {label}: {:+.2}",
            tweak_value(game, &format!("hand_{sel}_{axis}"), 0., -0.25, 0.25)
        ));
    }
    buttons.push("Reset this hand".into());
}
fn selected_hand(game: &Game<'_>) -> &'static str {
    if game.native_setting(ID, "hand_sel").as_deref() == Some("off") {
        "off"
    } else {
        "main"
    }
}
/// Steps `key` through `lo..=hi` by `step`, wrapping back to `lo` past `hi`.
fn cycle_tweak(game: &Game<'_>, key: &str, default: f32, lo: f32, hi: f32, step: f32) {
    let current = tweak_value(game, key, default, lo, hi);
    let next = if current + step > hi + step / 2. {
        lo
    } else {
        current + step
    };
    game.native_set_setting(ID, key, &format!("{next:.2}"));
}
pub(super) fn click(game: &Game<'_>, button: usize) {
    match button {
        0 => {
            let speed = ui_speed(game);
            game.native_set_setting(
                ID,
                "speed",
                &if speed >= 0.95 {
                    "0.1".into()
                } else {
                    format!("{:.1}", speed + 0.1)
                },
            );
        }
        1 => STATE.with(|s| *s.borrow_mut() = State::default()),
        2 => game.native_set_setting(
            ID,
            "hand_poses",
            if game.native_setting(ID, "hand_poses").as_deref() == Some("horizon") {
                "punchy"
            } else {
                "horizon"
            },
        ),
        // 3 is the status line; 4.. are the hand viewmodel rows.
        4 => game.native_set_setting(
            ID,
            "hand_sel",
            if selected_hand(game) == "off" {
                "main"
            } else {
                "off"
            },
        ),
        5 => {
            let sel = selected_hand(game);
            cycle_tweak(game, &format!("hand_{sel}_size"), 1., 0.7, 1.3, 0.1);
        }
        6 | 7 | 8 => {
            let sel = selected_hand(game);
            let axis = ["x", "y", "z"][button - 6];
            cycle_tweak(game, &format!("hand_{sel}_{axis}"), 0., -0.25, 0.25, 0.05);
        }
        9 => {
            let sel = selected_hand(game);
            game.native_set_setting(ID, &format!("hand_{sel}_size"), "1");
            for axis in ["x", "y", "z"] {
                game.native_set_setting(ID, &format!("hand_{sel}_{axis}"), "0");
            }
        }
        _ => {}
    }
}

// Authored skeletal delta calibrated against the same bone's rest transform.
// This preserves Horizon's item renderer and equipment coordinates; the full
// Punchy item mesh/physics/attachment renderer is a separate unfinished port.
fn bone_delta(s: &State, name: &str, alpha: f32) -> Mat4 {
    let Some(p) = &s.active else {
        return Mat4::IDENTITY;
    };
    let Some(base) = s.source.as_ref().and_then(|r| r.bones.get(name)) else {
        return Mat4::IDENTITY;
    };
    let time = if p.time < p.previous {
        p.time
    } else {
        p.previous + (p.time - p.previous) * alpha
    };
    let expression = p.previous_expression + (p.expression - p.previous_expression) * alpha;
    let sample = p.clip.sample(name, time, expression);
    let weight = p.previous_weight + (p.weight - p.previous_weight) * alpha;
    let sample = Pose::default().blend(sample, weight);
    let mut rest = base.matrix();
    rest.w_axis = (base.position / 16.).extend(1.);
    let posed = if p.clip.effect_active("use_keyframes_v2", time) {
        let rot = base.rotation + sample.euler;
        Mat4::from_scale_rotation_translation(
            sample.scale,
            Quat::from_euler(
                EulerRot::ZYX,
                rot.z.to_radians(),
                rot.y.to_radians(),
                rot.x.to_radians(),
            ),
            (base.position + sample.position) / 16.,
        )
    } else {
        rest * sample.matrix()
    };
    posed * rest.inverse()
}
fn hand_delta(s: &State, left: bool, alpha: f32) -> Mat4 {
    let Some(p) = &s.active else {
        return Mat4::IDENTITY;
    };
    let active_left = s.main_left != (p.hand == action::Hand::Off);
    if left != active_left
        && !punchy_engine::policy::dual_handed(&p.clip, p.time, s.offhand_item, s.offhand_item)
    {
        return Mat4::IDENTITY;
    }
    let side = if left { "left" } else { "right" };
    // Minecraft's authored model axes use X/Y opposite to first-person view.
    let flip = Mat4::from_scale(Vec3::new(-1., -1., 1.));
    flip * bone_delta(s, "camera", alpha)
        * bone_delta(s, "body", alpha)
        * bone_delta(s, &format!("{side}_arm"), alpha)
        * flip
}
pub(super) fn arms(game: &mut Game<'_>, alpha: f64, out: &mut Vec<Vertex>) {
    if !enabled(game)
        || game.spectator()
        || game.ui.hide_gui
        || game.sleeping.is_some()
        || !game.camera_type.first_person()
        || game.own_invisible()
        || game
            .player
            .as_ref()
            .is_none_or(|p| p.health.dead_or_dying())
        || game.screen == Screen::Dead
    {
        return;
    }
    let items = rendered_items(game);
    if items.iter().flatten().any(|s| maps::map_id(s).is_some()) {
        return;
    }
    let shown = game.hands_to_render();
    let a = alpha as f32;
    let eye = game.eye_position().unwrap_or_default();
    let light = game.world.light(
        eye[0].floor() as i32,
        eye[1].floor() as i32,
        eye[2].floor() as i32,
    );
    let packed = (u32::from(light >> 4) * 16) | ((u32::from(light & 15) * 16) << 8);
    let world = game.sway.to_world(a, &game.view);
    STATE.with(|cell| {
        let s = cell.borrow();
        if s.source.is_none() {
            return;
        }
        for (i, item) in items.iter().enumerate() {
            let dual = s.active.as_ref().is_some_and(|p| {
                punchy_engine::policy::dual_handed(&p.clip, p.time, s.offhand_item, s.offhand_item)
            });
            let hidden = i == 1
                && s.active.as_ref().is_some_and(|p| {
                    punchy_engine::policy::hide_empty_offhand(&p.clip, p.time, s.offhand_item)
                });
            if hidden {
                continue;
            }
            if (!shown[i] && !dual) || (item.is_none() && (i == 0 || !dual)) {
                continue;
            }
            let left = game.settings.main_hand_left != (i == 1);
            let m = world * hand_tweak(game, left) * arm_pose(left, 0., 0.);
            game.first_person_arm(left, m, packed, out);
        }
    });
}

pub(super) fn begin(game: &mut Game<'_>, alpha: f64) {
    STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        s.enabled = enabled(game) && game.native_window();
        s.render_alpha = (alpha as f32).clamp(0., 1.);
        s.main_left = game.settings.main_hand_left;
        let deltas = [
            hand_tweak(game, false) * hand_delta(&s, false, s.render_alpha),
            hand_tweak(game, true) * hand_delta(&s, true, s.render_alpha),
        ];
        let active = if let Some(p) = &s.active {
            let left = s.main_left != (p.hand == action::Hand::Off);
            let dual = punchy_engine::policy::dual_handed(&p.clip, p.time, s.offhand_item, s.offhand_item);
            [!left || dual, left || dual]
        } else {
            [false, false]
        };
        let mut authored = [None, None];
        if s.enabled && game.native_setting(ID, "hand_poses").as_deref() != Some("horizon") {
            if let (Some(source), Some(tuning)) = (&s.source, &s.tuning) {
                if let Ok(skeleton) = punchy_engine::skeleton::Skeleton::new(&source.bones)
                {
                    for left in [false, true] {
                        let i = usize::from(left);
                        let h = usize::from(s.main_left != left);
                        let id = &s.last_items[h];
                        let owns = s.active.as_ref().is_some_and(|p| {
                            (s.main_left != (p.hand == action::Hand::Off)) == left
                                || punchy_engine::policy::dual_handed(&p.clip, p.time, s.offhand_item, s.offhand_item)
                        });
                        let animation = s.active.as_ref().map(|p| {
                            let t = if p.time < p.previous {
                                p.time
                            } else {
                                p.previous + (p.time - p.previous) * s.render_alpha
                            };
                            (&*p.clip, t, p.previous_expression+(p.expression-p.previous_expression)*s.render_alpha)
                        });
                        let weight = s.active.as_ref().map_or(1., |p| {
                            p.previous_weight + (p.weight - p.previous_weight) * s.render_alpha
                        });
                        let context = punchy_engine::tuning::ItemContext {
                            id,
                            kind: s.item_kinds[h].as_deref(),
                            ..Default::default()
                        };
                        let arm = tuning
                            .transform_for(&context, left, true)
                            .unwrap_or_default();
                        let item = tuning
                            .transform_for(&context, left, false)
                            .unwrap_or_default();
                        let suppress_tuning = animation.is_some_and(|(c, t, _)| {
                            c.effect_active("disable_arm_tuning", t)
                                || (c.effect_active("use_keyframes_v2", t)
                                    && !c.effect_active("keep_arm_tuning", t))
                        });
                        let factor = if owns && suppress_tuning { 1. - weight } else { 1. };
                        let frame=s.active.as_ref().map_or_else(
                            ||punchy_engine::attachments::sample_frame(&skeleton,left,None,1.,true),
                            |p|p.frame(&skeleton,left,s.main_left,s.offhand_item,s.render_alpha));
                        let mut pose = punchy_engine::attachments::attach(
                            frame,
                            left,
                            &arm,
                            &item,
                            factor,
                            if owns { 1. - weight } else { 1. },
                            id.ends_with(":chest")
                                || id.ends_with(":piston")
                                || id.ends_with(":sticky_piston"),
                        );
                        let slim = game.player_model.as_ref().is_some_and(|p| p.slim);
                        if let Some(model) = game.res.player_arm(slim, left).as_ref() {
                            let base = Mat4::from_translation(
                                Vec3::new(model.pose[0], model.pose[1], model.pose[2]) / 16.,
                            ) * Mat4::from_rotation_z(if left { -0.1 } else { 0.1 });
                            pose.arm *= base.inverse();
                            authored[i] = Some(pose);
                        }
                    }
                }
            }
        }
        s.render_count=s.render_count.saturating_add(1);
        if let Some(path)=std::env::var_os("PUNCHY_RENDER_DIAGNOSTICS") {
            let value=serde_json::json!({"frames":s.render_count,"authored_poses":authored.iter().flatten().count(),"active":active,
                "arm_matrices":authored.map(|p|p.map(|p|p.arm.to_cols_array())),"item_matrices":authored.map(|p|p.map(|p|p.item.to_cols_array()))});
            if let Ok(bytes)=serde_json::to_vec_pretty(&value){let _=std::fs::write(path,bytes);}
        }
        punchy_engine::horizon_pose::set_frame(punchy_engine::horizon_pose::Frame {
            authored,
            enabled: s.enabled,
            deltas,
            active,
        });
    });
}

pub(super) fn ready(game: &Game<'_>) -> bool {
    enabled(game) && STATE.with(|cell| cell.borrow().source.is_some())
}
pub(super) fn item_pose(
    left: bool,
    height: f32,
    attack: f32,
    pose: horizon_client::first_person::ItemPose,
) -> Mat4 {
    if let Some(pose) = punchy_engine::horizon_pose::authored(left) {
        return pose.item;
    }
    let swing = if punchy_engine::horizon_pose::suppress_swing(left) {
        0.
    } else {
        attack
    };
    punchy_engine::horizon_pose::transform(
        left,
        horizon_client::first_person::item_pose(left, height, swing, pose),
    )
}
pub(super) fn arm_pose(left: bool, height: f32, attack: f32) -> Mat4 {
    if let Some(pose) = punchy_engine::horizon_pose::authored(left) {
        return pose.arm;
    }
    let swing = if punchy_engine::horizon_pose::suppress_swing(left) {
        0.
    } else {
        attack
    };
    punchy_engine::horizon_pose::transform(
        left,
        horizon_client::first_person::player_arm_pose(left, height, swing),
    )
}

pub(super) fn authored(left: bool) -> bool {
    punchy_engine::horizon_pose::authored(left).is_some()
}

// Called only after a successful client-side interaction, including offhand.
pub(super) fn used(game: &Game<'_>, used_hand: Hand, success: bool) {
    if success && game.native_window() && enabled(game) {
        STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            if state.successful_use.len() >= 32 {
                state.successful_use.pop_front();
            }
            state.successful_use.push_back(hand(used_hand));
        });
    }
}

fn phase_hand(hand: action::Hand) -> phases::Hand {
    if hand == action::Hand::Main {
        phases::Hand::Main
    } else {
        phases::Hand::Off
    }
}
fn action_hand(hand: phases::Hand) -> action::Hand {
    if hand == phases::Hand::Main {
        action::Hand::Main
    } else {
        action::Hand::Off
    }
}
fn phase_playback(s: &State) -> phases::Playback {
    s.active
        .as_ref()
        .map_or_else(Default::default, |p| phases::Playback {
            clip: Some(p.clip.name.clone()),
            id: p.id,
            playing: !p.stopping,
            blending: p.stopping || p.weight < 1.,
            time: p.time,
            expression_time: p.expression,
            ..Default::default()
        })
}
fn phase_selection(
    s: &State,
    kind: &str,
    hand: action::Hand,
    speed: f32,
) -> Option<phases::Selection> {
    let item = &s.last_items[hand.index()];
    let entry = tuning_animation(s, hand, kind)?;
    let resolve = |name: &str| {
        let (_, clip) = entry.resolve(name, &s.clips)?;
        Some(phases::ClipSpec {
            name: clip.name.clone(),
            length: clip.length,
            speed_multiplier: punchy_engine::playback::manager_multiplier(&clip.name, speed, false),
            hold: clip.hold_on_last_frame,
        })
    };
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    item.hash(&mut hash);
    kind.hash(&mut hash);
    entry.raw.to_string().hash(&mut hash);
    Some(phases::Selection {
        entry_id: hash.finish(),
        start: resolve(entry.start.as_deref()?)?,
        looping: resolve(entry.looping.as_deref()?)?,
        end: resolve(entry.end.as_deref()?)?,
        skip_blend_in: entry.skip_blend_in,
        skip_blend_out: entry.skip_blend_out,
        stop_immediate: entry.stop_immediate,
    })
}
fn stack_token(s: &mut State, stack: Option<&ItemStack>, mining: bool) -> u64 {
    let Some(stack) = stack else {
        return 0;
    };
    let mut key = stack.clone();
    key.count = 1;
    if mining {
        key.damage = 0;
        if let Some(extra) = key.extra.as_mut() {
            extra.enchantments.clear();
            extra.other.retain(|(id, _)| id.starts_with("minecraft:"));
        }
    }
    if let Some(index) = s.stack_keys.iter().position(|k| *k == key) {
        return index as u64 + 1;
    }
    s.stack_keys.push(key);
    s.stack_keys.len() as u64
}
fn phase_effects(s: &mut State, effects: Vec<phases::Effect>) {
    for effect in effects {
        match effect {
            phases::Effect::ClearUseTracker => s.successful_use.clear(),
            phases::Effect::SetForcedStack { hand, token } => {
                s.forced_stack = token
                    .checked_sub(1)
                    .and_then(|i| s.stack_keys.get(i as usize))
                    .cloned()
                    .map(|v| (action_hand(hand).index(), v));
            }
            phases::Effect::ClearForcedStack => s.forced_stack = None,
            phases::Effect::ResetPoseImmediate => s.active = None,
            phases::Effect::SetLooping(value) => {
                if let Some(p) = &mut s.active {
                    p.looping = value;
                    p.released = !value;
                }
            }
            phases::Effect::SetSpeedMultiplier(value) => {
                if let Some(p) = &mut s.active {
                    p.multiplier = value;
                }
            }
            phases::Effect::SetBlendOutDurationNext(value) => {
                if let Some(p) = &mut s.active {
                    p.return_fade = value;
                }
            }
            // Source-hand and rendered-sync notifications are represented by the
            // playback hand and the forced stack at this renderer boundary.
            _ => {}
        }
    }
}
fn phase_tick(
    s: &mut State,
    input: &mut action::Input,
    items: &[Option<ItemStack>; 2],
    speed: f32,
    attack_down: bool,
) -> Option<(bool, phases::Request)> {
    s.tick += 1;
    let playback = phase_playback(s);
    let mining_selection =
        phase_selection(s, "mining", action::Hand::Main, speed);
    let mining_stack = stack_token(s, items[0].as_ref(), true);
    let mining = s.mining.tick(&phases::MiningInput {
        tick: s.tick,
        raw_mining: input.mining,
        triggered: input.mining_trigger,
        attack_down,
        aiming_trackable_block: input.block_target,
        stack: mining_stack,
        selection: mining_selection.clone(),
        playback: playback.clone(),
        ..Default::default()
    });
    let using = input.using;
    if let Some(u) = using {
        s.consume_hand = u.hand;
    }
    let h = s.consume_hand.index();
    let kind = using.map(|u| u.kind);
    let consume_kind = match kind {
        Some(action::ItemKind::Eat) => Some("eating"),
        Some(action::ItemKind::Drink) => Some("drinking"),
        _ => None,
    };
    let selection = consume_kind
        .and_then(|kind| phase_selection(s, kind, s.consume_hand, speed));
    let tokens = [
        stack_token(s, items[0].as_ref(), false),
        stack_token(s, items[1].as_ref(), false),
    ];
    let consume = s.consume.tick(&phases::ConsumeInput {
        use_down: input.use_down,
        using_item: using.is_some(),
        using_consume: consume_kind.is_some(),
        hand: phase_hand(s.consume_hand),
        using_stack: tokens[h],
        current_hand_stack: tokens[h],
        opposite_hand_stack: tokens[1 - h],
        animation_speed: speed,
        selection: selection.clone().map(phases::ConsumeSelection::Phases),
        playback: playback.clone(),
        ..Default::default()
    });
    if selection.is_some() {
        input.using = None;
        input.use_trigger = None;
    }
    if mining_selection.is_some() {
        input.mining_trigger = false;
        input.mining = false;
    }
    if s.consume.equip_blocked() {
        input.equip = None;
    }
    phase_effects(s, mining.effects);
    phase_effects(s, consume.effects);
    let request = consume
        .request
        .filter(|r| r.may_apply(&playback))
        .map(|r| (true, r))
        .or_else(|| {
            mining
                .request
                .filter(|r| r.may_apply(&playback))
                .map(|r| (false, r))
        });
    input.external_priority = request.as_ref().map(|(_, r)| r.priority);
    request
}
fn apply_phase(s: &mut State, consume: bool, request: &phases::Request, speed: f32) {
    let mut consume_tick = false;
    let play = match &request.command {
        phases::Command::Play(p) => Some(p.clone()),
        phases::Command::Handoff {
            clip,
            hand,
            speed_multiplier,
            tick_seconds,
            blend_seconds,
            fallback,
        } => {
            let playback = phase_playback(s);
            let Some(target) = s.clips.get(&clip.name) else {
                return;
            };
            let source_length = s.active.as_ref().map_or(0., |p| p.clip.length);
            let source_delta = s.active.as_ref().map_or(0., |p| {
                tick_seconds
                    * punchy_engine::playback::rate(&p.clip, p.time, speed, p.multiplier, 1.)
            });
            match phases::exact_handoff(
                &playback,
                source_length,
                source_delta,
                clip.length,
                *tick_seconds,
                *blend_seconds,
                |seconds| {
                    seconds
                        * punchy_engine::playback::rate(target, 0., speed, *speed_multiplier, 1.)
                },
            ) {
                phases::HandoffDecision::Wait => return,
                phases::HandoffDecision::Fallback => Some(fallback.clone()),
                phases::HandoffDecision::Start {
                    time,
                    expression_time,
                    consumes_tick,
                } => {
                    consume_tick = consumes_tick;
                    let mut p = fallback.clone();
                    p.clip = clip.clone();
                    p.hand = *hand;
                    p.looping = true;
                    p.hold = false;
                    p.speed_multiplier = *speed_multiplier;
                    p.blend_seconds = *blend_seconds;
                    p.start_time = time;
                    p.expression_time = expression_time;
                    p.stop_after_current = false;
                    Some(p)
                }
            }
        }
        phases::Command::FinishCurrent { release_hold } => {
            if let Some(p) = &mut s.active {
                p.looping = false;
                p.released = true;
                if *release_hold {
                    p.hold = false;
                }
            }
            None
        }
        phases::Command::StopImmediate { blend_seconds } => {
            if let Some(p) = &mut s.active {
                p.stopping = true;
                p.return_fade = blend_seconds.max(0.001);
            }
            None
        }
    };
    if let Some(play) = play {
        let Some(clip) = s.clips.get(&play.clip.name).cloned() else {
            return;
        };
        s.playback_id = s.playback_id.wrapping_add(1);
        let from = s.active.take().map(Box::new);
        s.active = Some(Playback {
            from,
            out_weight: 1.,
            previous_out_weight: 1.,
            id: s.playback_id,
            clip,
            hand: action_hand(play.hand),
            previous: play.start_time,
            time: play.start_time,
            expression: play.expression_time,
            previous_expression: play.expression_time,
            multiplier: play.speed_multiplier,
            looping: play.looping,
            released: play.stop_after_current,
            hold: play.hold,
            fade: play.blend_seconds.max(0.001),
            return_fade: play.return_blend_seconds.unwrap_or(0.28),
            skip_tick: consume_tick,
            stopping: false,
            weight: if play.blend_seconds <= 0. { 1. } else { 0. },
            previous_weight: if play.blend_seconds <= 0. { 1. } else { 0. },
        });
        let playback = phase_playback(s);
        if consume {
            s.consume.commit(request, &playback);
        } else {
            s.mining.commit(request, &playback, s.tick);
        }
    }
}

pub(super) fn rendered_items(game: &Game<'_>) -> [Option<ItemStack>; 2] {
    let mut items = game.visible_hand_items();
    STATE.with(|s| {
        if let Some((hand, stack)) = &s.borrow().forced_stack {
            items[*hand] = Some(stack.clone());
        }
    });
    items
}

// Capture the target before creative prediction removes the block. Source
// records start attempts too; relying only on Game.mining loses instant breaks.
pub(super) fn mining_attempt(game: &Game<'_>) {
    if game.native_window() && enabled(game) && !game.spectator() {
        STATE.with(|s| s.borrow_mut().mining_attempt = true);
    }
}
