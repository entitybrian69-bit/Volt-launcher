//! Consume and mining v2 phase controllers from the shipped Punchy 3.1b classes.
//!
//! The controllers separate immediate effects from arbitration requests. Apply
//! `effects` immediately in order, enqueue `request` with its priority, and call
//! `commit` only after the request won and the indicated clip actually started.
//! A rejected/blocked request must never advance the controller's phase.
//!
//! Game-dependent decisions are explicit inputs: fully resolved clip/entry
//! identity, blacklists, stack equality, equip cancellation, and mining target
//! recency. Stack tokens must represent item + components; mining equality must
//! ignore damage, enchantments and components outside the minecraft namespace,
//! exactly as MiningStateMachine.hasItemChanged does. This module does not
//! resolve selectors/tags, copy ItemStacks or manipulate arm snapshot ownership.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Hand {
    #[default]
    Main,
    Off,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    None,
    Single,
    Start,
    Loop,
    LoopEndPending,
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipSpec {
    pub name: String,
    pub length: f32,
    /// PunchyAnimationManager.clipSpeedMultiplier, not the effective wall rate.
    pub speed_multiplier: f32,
    pub hold: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    /// Changes when the selected tuning entry object changes, even if names do
    /// not. The original state machines compare entry references by identity.
    pub entry_id: u64,
    pub start: ClipSpec,
    pub looping: ClipSpec,
    pub end: ClipSpec,
    pub skip_blend_in: bool,
    pub skip_blend_out: bool,
    pub stop_immediate: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConsumeSelection {
    Phases(Selection),
    Single {
        clip: ClipSpec,
        skip_blend_in: bool,
        skip_blend_out: bool,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Playback {
    pub clip: Option<String>,
    pub id: u64,
    pub playing: bool,
    pub blending: bool,
    pub paused: bool,
    pub frozen: bool,
    pub time: f32,
    pub expression_time: f32,
}

impl Playback {
    pub fn playing_clip(&self, clip: &ClipSpec) -> bool {
        self.playing && self.clip.as_deref() == Some(clip.name.as_str())
    }
    pub fn current_clip(&self, clip: &ClipSpec) -> bool {
        self.clip.as_deref() == Some(clip.name.as_str())
    }
    pub fn near_end(&self, clip: &ClipSpec, lead: f32) -> bool {
        !self.playing_clip(clip) || clip.length <= 0.0 || self.time >= (clip.length - lead).max(0.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ownership {
    pub clip: String,
    pub playback_id: u64,
}
impl Ownership {
    pub fn matches(&self, playback: &Playback) -> bool {
        playback.id == self.playback_id
            && playback.clip.as_deref() == Some(self.clip.as_str())
            && (playback.playing || playback.blending)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayStyle {
    /// playAtTime(..., skipBlendIn=true), with a cold origin.
    Immediate,
    /// playVariationAtTime with smooth real-time entrance; a continuing action
    /// preserves its playback ID and expression clock.
    Variation { continuing: bool },
    /// playWithBlendFromCurrentDynamic, smooth .06-second mining entrance.
    DynamicFromCurrent,
    /// playAtTimeRebasedFromCurrentReturnToOrigin, return speed 4.5.
    RebasedReturn { speed: f32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayRequest {
    pub clip: ClipSpec,
    pub hand: Hand,
    pub looping: bool,
    pub hold: bool,
    pub speed_multiplier: f32,
    pub blend_seconds: f32,
    pub return_blend_seconds: Option<f32>,
    pub stop_after_current: bool,
    pub start_time: f32,
    pub expression_time: f32,
    pub style: PlayStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Play(PlayRequest),
    /// Re-attempt every requested tick. A .25-second lead is only a request
    /// lead, never permission to skip the last .25 seconds of the start clip.
    Handoff {
        clip: ClipSpec,
        hand: Hand,
        speed_multiplier: f32,
        tick_seconds: f32,
        blend_seconds: f32,
        fallback: PlayRequest,
    },
    FinishCurrent {
        release_hold: bool,
    },
    StopImmediate {
        blend_seconds: f32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub priority: u8,
    pub deferred: bool,
    pub phase: Phase,
    pub expected: Option<Ownership>,
    pub command: Command,
}
impl Request {
    pub fn may_apply(&self, playback: &Playback) -> bool {
        self.expected
            .as_ref()
            .is_none_or(|owner| owner.matches(playback))
    }
    pub fn target(&self) -> Option<&ClipSpec> {
        match &self.command {
            Command::Play(p) => Some(&p.clip),
            Command::Handoff { clip, .. } => Some(clip),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    ClearUseTracker,
    ClearSourceHand,
    SetSourceHand(Hand),
    SetForcedStack {
        hand: Hand,
        token: u64,
    },
    ClearForcedStack,
    RequestRenderedSync,
    CancelOffhandTransition,
    TriggerOffhandPickup,
    TriggerImmediateMainEquip,
    ResetPoseImmediate,
    SetLooping(bool),
    SetSpeedMultiplier(f32),
    SetBlendOutDurationNext(f32),
    ClearPersistentMining,
    SetPersistentMining,
    /// Selection has no valid v2 phases. The caller must run the legacy
    /// playPunch path using its normal resolver, not manufacture phase clips.
    LegacyMiningFallback,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Output {
    pub request: Option<Request>,
    pub effects: Vec<Effect>,
}

/// PoseHandler.handoffToLoopAtCurrentTick's boundary and overflow arithmetic.
/// The caller supplies deltas computed with the actual clip's anim_speed
/// timeline and effective-rate policy. `target_delta_for_overflow_seconds`
/// is only used for blended consume handoffs, whose overflow is converted back
/// to real seconds and then into the target clip's animation clock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HandoffDecision {
    Wait,
    Fallback,
    Start {
        time: f32,
        expression_time: f32,
        consumes_tick: bool,
    },
}

pub fn exact_handoff(
    playback: &Playback,
    source_length: f32,
    source_delta: f32,
    target_length: f32,
    tick_seconds: f32,
    blend_seconds: f32,
    target_delta_for_overflow_seconds: impl FnOnce(f32) -> f32,
) -> HandoffDecision {
    if !playback.playing {
        return HandoffDecision::Fallback;
    }
    if playback.clip.is_none() || playback.paused || playback.frozen || source_length <= 0.0 {
        return HandoffDecision::Wait;
    }
    let mut overflow = playback.time + source_delta - source_length;
    if overflow < -1.0e-4 {
        return HandoffDecision::Wait;
    }
    if blend_seconds > 0.0 {
        let seconds = overflow.max(0.0) * tick_seconds / source_delta.max(1.0e-6);
        overflow = target_delta_for_overflow_seconds(seconds);
    }
    if target_length > 0.0 {
        overflow = overflow.rem_euclid(target_length);
    } else {
        overflow = 0.0;
    }
    HandoffDecision::Start {
        time: overflow,
        expression_time: overflow,
        consumes_tick: true,
    }
}

#[derive(Clone, Debug)]
pub struct ConsumeInput {
    pub player_present: bool,
    pub use_down: bool,
    pub using_item: bool,
    pub using_consume: bool,
    pub hand: Hand,
    /// Full ItemStack equality token; zero denotes ItemStack.EMPTY.
    pub using_stack: u64,
    pub current_hand_stack: u64,
    pub opposite_hand_stack: u64,
    pub blacklisted: bool,
    pub equip_in_progress: bool,
    pub equip_abort_allowed: bool,
    pub selection: Option<ConsumeSelection>,
    pub animation_speed: f32,
    pub playback: Playback,
}
impl Default for ConsumeInput {
    fn default() -> Self {
        Self {
            player_present: true,
            use_down: false,
            using_item: false,
            using_consume: false,
            hand: Hand::Main,
            using_stack: 0,
            current_hand_stack: 0,
            opposite_hand_stack: 0,
            blacklisted: false,
            equip_in_progress: false,
            equip_abort_allowed: true,
            selection: None,
            animation_speed: 5.5,
            playback: Playback::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConsumeController {
    phase: Phase,
    owner: Option<Ownership>,
    active_clip: Option<ClipSpec>,
    active_hand: Option<Hand>,
    selection: Option<Selection>,
    last_stack: u64,
    forced_stack: u64,
    forced_hand: Option<Hand>,
    finishing_result: u64,
    was_consuming: bool,
    finishing: bool,
    swap_pending: bool,
    swapped: bool,
    skip_blend_out: bool,
    equip_blocked: bool,
    use_suppressed: bool,
}

impl ConsumeController {
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn equip_blocked(&self) -> bool {
        self.equip_blocked
    }
    pub fn use_suppressed(&self) -> bool {
        self.use_suppressed
    }
    pub fn finishing(&self) -> bool {
        self.finishing
    }
    pub fn forced_stack(&self) -> Option<(Hand, u64)> {
        self.forced_hand
            .zip((self.forced_stack != 0).then_some(self.forced_stack))
    }
    pub fn owns(&self, p: &Playback) -> bool {
        self.owner.as_ref().is_some_and(|owner| owner.matches(p))
    }

    pub fn commit(&mut self, request: &Request, playback: &Playback) -> bool {
        let Some(clip) = request.target() else {
            return false;
        };
        if !playback.playing_clip(clip) {
            return false;
        }
        self.active_clip = Some(clip.clone());
        self.owner = Some(Ownership {
            clip: clip.name.clone(),
            playback_id: playback.id,
        });
        self.phase = request.phase;
        if let Command::Play(p) = &request.command {
            self.skip_blend_out = p.return_blend_seconds == Some(0.0);
        }
        true
    }

    pub fn tick(&mut self, i: &ConsumeInput) -> Output {
        let mut out = Output::default();
        if !i.player_present {
            *self = Self::default();
            out.effects.push(Effect::ClearForcedStack);
            return out;
        }
        if !i.use_down {
            self.use_suppressed = false;
        }
        let consuming = i.using_item && i.use_down && i.using_consume;
        if self.finishing {
            if consuming {
                self.finishing = false;
                self.swap_pending = false;
                self.swapped = false;
                self.active_clip = None;
                self.owner = None;
                self.skip_blend_out = false;
                self.reset_phases();
                out.effects.push(Effect::ClearSourceHand);
            }
            if self.finishing && self.finishing_stack_changed(i.current_hand_stack) {
                self.cancel(&i.playback, &mut out);
                self.finishing = false;
                self.swap_pending = false;
                self.swapped = false;
                self.active_clip = None;
                self.owner = None;
                self.skip_blend_out = false;
                self.reset_phases();
                out.effects.push(Effect::ClearSourceHand);
                self.clear_forced(&mut out);
                self.equip_blocked = false;
            }
            out.effects.push(Effect::ClearUseTracker);
            if self.owns(&i.playback) {
                if self.swap_pending
                    && !self.swapped
                    && self.active_clip.as_ref().is_some_and(|clip| {
                        clip.length <= 0.0 || i.playback.time >= clip.length - 1.0e-4
                    })
                {
                    self.clear_forced(&mut out);
                    out.effects.push(Effect::RequestRenderedSync);
                    self.swapped = true;
                }
                self.equip_blocked = true;
                return out;
            }
            self.finishing = false;
            self.swap_pending = false;
            self.swapped = false;
            self.active_clip = None;
            self.owner = None;
            self.active_hand = None;
            self.skip_blend_out = false;
            self.reset_phases();
            out.effects.push(Effect::ClearSourceHand);
            self.clear_forced(&mut out);
            self.use_suppressed = true;
        }
        if consuming && self.was_consuming && self.active_hand.is_some_and(|h| h != i.hand) {
            self.cancel(&i.playback, &mut out);
            out.effects.push(Effect::ClearSourceHand);
            self.was_consuming = false;
            self.finishing = false;
            self.swap_pending = false;
            self.swapped = false;
            self.active_clip = None;
            self.owner = None;
            self.active_hand = None;
            self.skip_blend_out = false;
            self.use_suppressed = false;
            self.equip_blocked = false;
            self.reset_phases();
            self.clear_forced(&mut out);
            out.effects
                .extend([Effect::CancelOffhandTransition, Effect::RequestRenderedSync]);
            if i.hand == Hand::Off {
                out.effects.push(Effect::TriggerOffhandPickup);
            }
            return out;
        }
        if consuming && i.equip_in_progress && !i.equip_abort_allowed {
            return out;
        }
        if consuming {
            out.effects.push(Effect::ClearUseTracker);
            self.use_suppressed = true;
            self.equip_blocked = true;
            if i.blacklisted {
                if self.was_consuming {
                    self.cancel(&i.playback, &mut out);
                    out.effects.push(Effect::ClearSourceHand);
                }
                self.was_consuming = false;
                self.finishing = false;
                self.active_clip = None;
                self.owner = None;
                self.active_hand = None;
                self.equip_blocked = false;
                self.reset_phases();
                self.clear_forced(&mut out);
                return out;
            }
            match i.selection.as_ref() {
                Some(ConsumeSelection::Phases(selection)) => {
                    self.tick_phases(i, selection, &mut out)
                }
                Some(ConsumeSelection::Single {
                    clip,
                    skip_blend_in,
                    skip_blend_out,
                }) => {
                    if !self.was_consuming
                        || self
                            .active_clip
                            .as_ref()
                            .is_none_or(|old| old.name != clip.name)
                        || !i.playback.playing
                    {
                        // The original refreshes forcedRenderStack before the
                        // continuing equality check; use the refreshed token.
                        self.set_forced(i.hand, i.using_stack, &mut out);
                        let continuing = self.was_consuming
                            && self.active_hand == Some(i.hand)
                            && matches!(self.phase, Phase::None | Phase::Single)
                            && self.owns(&i.playback);
                        out.request = Some(self.consume_play(
                            i,
                            clip,
                            Phase::Single,
                            (*skip_blend_in, *skip_blend_out),
                            continuing,
                            0.25,
                        ));
                    }
                    if self.phase != Phase::None && self.phase != Phase::Single {
                        self.reset_phases();
                    }
                }
                None => {
                    if self.phase != Phase::None {
                        self.reset_phases();
                    }
                }
            }
            self.was_consuming = true;
            self.finishing = false;
            self.swap_pending = false;
            self.swapped = false;
            self.active_hand = Some(i.hand);
        } else {
            if self.was_consuming {
                let swapped = self.forced_stack != 0
                    && self.forced_stack != i.current_hand_stack
                    && self.forced_stack == i.opposite_hand_stack;
                let completed = !i.using_item && i.use_down && !swapped;
                if self.phase != Phase::None
                    && self.phase != Phase::Single
                    && self.selection.is_some()
                {
                    if self.owns(&i.playback) {
                        let selection = self.selection.as_ref().unwrap().clone();
                        let mut release = i.clone();
                        release.hand = self.active_hand.unwrap_or(i.hand);
                        out.request = Some(self.consume_play(
                            &release,
                            &selection.end,
                            Phase::End,
                            (selection.skip_blend_in, selection.skip_blend_out),
                            false,
                            0.1,
                        ));
                        out.request.as_mut().unwrap().expected = self.owner.clone();
                    }
                    self.begin_finishing(completed, i.current_hand_stack);
                } else if completed {
                    if self.owns(&i.playback) {
                        out.request = Some(
                            self.stop_request(Command::FinishCurrent { release_hold: true }, false),
                        );
                    }
                    self.begin_finishing(true, i.current_hand_stack);
                } else {
                    self.cancel(&i.playback, &mut out);
                    self.finishing = false;
                    self.swap_pending = false;
                    self.swapped = false;
                    self.equip_blocked = false;
                    self.reset_phases();
                    self.clear_forced(&mut out);
                    if swapped {
                        out.effects
                            .extend([Effect::CancelOffhandTransition, Effect::RequestRenderedSync]);
                    }
                }
            }
            self.was_consuming = false;
            if !self.finishing {
                self.swap_pending = false;
                self.swapped = false;
                self.equip_blocked = false;
                self.reset_phases();
                self.clear_forced(&mut out);
            }
        }
        out
    }

    fn tick_phases(&mut self, i: &ConsumeInput, selection: &Selection, out: &mut Output) {
        let restart = matches!(self.phase, Phase::None | Phase::Single)
            || self
                .selection
                .as_ref()
                .is_none_or(|old| old.entry_id != selection.entry_id)
            || !self.owns(&i.playback)
            || self.last_stack != i.using_stack;
        self.skip_blend_out = selection.skip_blend_out;
        if restart {
            self.selection = Some(selection.clone());
            self.set_forced(i.hand, i.using_stack, out);
            out.request = Some(self.consume_play(
                i,
                &selection.start,
                Phase::Start,
                (selection.skip_blend_in, selection.skip_blend_out),
                false,
                0.25,
            ));
        } else {
            match self.phase {
                Phase::Start if i.playback.near_end(&selection.start, 0.25) => {
                    let fallback = match self
                        .consume_play(
                            i,
                            &selection.looping,
                            Phase::Loop,
                            (selection.skip_blend_in, selection.skip_blend_out),
                            false,
                            0.1,
                        )
                        .command
                    {
                        Command::Play(p) => p,
                        _ => unreachable!(),
                    };
                    out.request = Some(Request {
                        priority: 80,
                        deferred: true,
                        phase: Phase::Loop,
                        expected: self.owner.clone(),
                        command: Command::Handoff {
                            clip: selection.looping.clone(),
                            hand: i.hand,
                            speed_multiplier: consume_multiplier(
                                &selection.looping,
                                i.animation_speed,
                            ),
                            tick_seconds: 0.05,
                            blend_seconds: 0.1,
                            fallback,
                        },
                    });
                }
                Phase::Loop if !i.playback.playing_clip(&selection.looping) => {
                    out.request = Some(self.consume_play(
                        i,
                        &selection.looping,
                        Phase::Loop,
                        (selection.skip_blend_in, selection.skip_blend_out),
                        false,
                        0.1,
                    ));
                }
                Phase::Loop => out.effects.push(Effect::SetLooping(true)),
                Phase::End
                    if !i.playback.playing_clip(&selection.end)
                        && !i.playback.current_clip(&selection.end) =>
                {
                    self.reset_phases()
                }
                _ => {}
            }
        }
        self.last_stack = i.using_stack;
    }

    fn consume_play(
        &self,
        i: &ConsumeInput,
        clip: &ClipSpec,
        phase: Phase,
        skip: (bool, bool),
        continuing: bool,
        blend: f32,
    ) -> Request {
        let (skip_in, skip_out) = skip;
        let same = continuing && self.owns(&i.playback) && i.playback.playing;
        let cold = !i.playback.playing && !i.playback.blending;
        let skip = cold && skip_in && matches!(phase, Phase::None | Phase::Single | Phase::Start);
        Request {
            priority: 80,
            deferred: true,
            phase,
            expected: None,
            command: Command::Play(PlayRequest {
                clip: clip.clone(),
                hand: i.hand,
                looping: phase == Phase::Loop,
                hold: matches!(phase, Phase::None | Phase::Single) && clip.hold,
                speed_multiplier: consume_multiplier(clip, i.animation_speed),
                blend_seconds: if skip { 0.0 } else { blend },
                return_blend_seconds: Some(if skip_out { 0.0 } else { 0.25 }),
                stop_after_current: matches!(phase, Phase::Start | Phase::End),
                start_time: if same { i.playback.time } else { 0.0 },
                expression_time: if same {
                    i.playback.expression_time
                } else {
                    0.0
                },
                style: if skip {
                    PlayStyle::Immediate
                } else {
                    PlayStyle::Variation { continuing: same }
                },
            }),
        }
    }
    fn stop_request(&self, command: Command, deferred: bool) -> Request {
        Request {
            priority: 80,
            deferred,
            phase: self.phase,
            expected: self.owner.clone(),
            command,
        }
    }
    fn cancel(&self, p: &Playback, out: &mut Output) {
        if self.owns(p) {
            out.request = Some(self.stop_request(
                Command::StopImmediate {
                    blend_seconds: if self.skip_blend_out { 0.0 } else { 0.25 },
                },
                false,
            ));
        }
    }
    fn set_forced(&mut self, hand: Hand, stack: u64, out: &mut Output) {
        out.effects.push(Effect::SetSourceHand(hand));
        if stack != 0 {
            self.forced_stack = stack;
            self.forced_hand = Some(hand);
            out.effects
                .push(Effect::SetForcedStack { hand, token: stack });
        }
    }
    fn clear_forced(&mut self, out: &mut Output) {
        self.forced_stack = 0;
        self.forced_hand = None;
        out.effects.push(Effect::ClearForcedStack);
    }
    fn reset_phases(&mut self) {
        self.phase = Phase::None;
        self.selection = None;
        self.last_stack = 0;
    }
    fn begin_finishing(&mut self, completed: bool, stack: u64) {
        self.finishing = true;
        self.swap_pending = completed;
        self.finishing_result = if completed { stack } else { 0 };
        self.swapped = false;
        self.equip_blocked = true;
        self.use_suppressed = true;
    }
    fn finishing_stack_changed(&self, current: u64) -> bool {
        let reference = if self.swap_pending {
            self.finishing_result
        } else {
            self.forced_stack
        };
        (self.swap_pending || reference != 0) && reference != current
    }
}

fn consume_multiplier(clip: &ClipSpec, animation_speed: f32) -> f32 {
    if matches!(clip.name.as_str(), "use_eat" | "use_drink") {
        1.0
    } else {
        1.0 / animation_speed.max(0.01)
    }
}

#[derive(Clone, Debug)]
pub struct MiningInput {
    pub player_present: bool,
    pub tick: i64,
    pub raw_mining: bool,
    pub triggered: bool,
    pub attack_down: bool,
    pub aiming_trackable_block: bool,
    pub recent_target_90ms: bool,
    pub recent_chain_180ms: bool,
    /// Comparison token following hasItemChanged's ignored-component policy.
    pub stack: u64,
    pub functional_swap_to_offhand: bool,
    pub blacklisted: bool,
    pub hand_out_active: bool,
    pub equip_in_progress: bool,
    pub equip_is_hand_in: bool,
    pub should_abort_equip: bool,
    pub equip_abort_allowed: bool,
    pub selection: Option<Selection>,
    pub playback: Playback,
}
impl Default for MiningInput {
    fn default() -> Self {
        Self {
            player_present: true,
            tick: 0,
            raw_mining: false,
            triggered: false,
            attack_down: false,
            aiming_trackable_block: false,
            recent_target_90ms: false,
            recent_chain_180ms: false,
            stack: 0,
            functional_swap_to_offhand: false,
            blacklisted: false,
            hand_out_active: false,
            equip_in_progress: false,
            equip_is_hand_in: false,
            should_abort_equip: true,
            equip_abort_allowed: true,
            selection: None,
            playback: Playback::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MiningController {
    phase: Phase,
    selection: Option<Selection>,
    active_clip: Option<String>,
    last_stack: u64,
    last_v2_stack: u64,
    was_mining: bool,
    min_stop_tick: i64,
    stop_grace_ticks: u8,
    last_v2_trigger: i64,
    start_speed: f32,
    pending_tool_equip: bool,
}
impl Default for MiningController {
    fn default() -> Self {
        Self {
            phase: Phase::None,
            selection: None,
            active_clip: None,
            last_stack: 0,
            last_v2_stack: 0,
            was_mining: false,
            min_stop_tick: -1,
            stop_grace_ticks: 0,
            last_v2_trigger: -1,
            start_speed: 1.0,
            pending_tool_equip: false,
        }
    }
}

impl MiningController {
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn start_speed(&self) -> f32 {
        self.start_speed
    }
    pub fn was_mining(&self) -> bool {
        self.was_mining
    }
    pub fn commit(&mut self, request: &Request, playback: &Playback, tick: i64) -> bool {
        let Some(clip) = request.target() else {
            return false;
        };
        if !playback.playing_clip(clip) {
            return false;
        }
        self.active_clip = Some(clip.name.clone());
        self.phase = request.phase;
        if request.phase == Phase::Start {
            self.min_stop_tick = tick + 2;
        }
        true
    }

    pub fn tick(&mut self, i: &MiningInput) -> Output {
        let mut out = Output::default();
        if !i.player_present {
            *self = Self::default();
            out.effects.push(Effect::ClearPersistentMining);
            return out;
        }
        if i.hand_out_active {
            self.clear_all();
            out.effects.push(Effect::ClearPersistentMining);
            return out;
        }
        if i.blacklisted {
            if self.was_mining
                && self.active_clip.as_deref() == i.playback.clip.as_deref()
                && self.active_clip.is_some()
            {
                let skip = self.selection.as_ref().is_some_and(|s| s.skip_blend_out);
                if skip {
                    out.effects.push(Effect::SetBlendOutDurationNext(0.0));
                }
                let immediate =
                    self.selection.as_ref().is_some_and(|s| s.stop_immediate) && !i.attack_down;
                out.request = Some(self.stop_request(if immediate {
                    Command::StopImmediate { blend_seconds: 0.3 }
                } else {
                    Command::FinishCurrent {
                        release_hold: false,
                    }
                }));
            }
            self.clear_all();
            out.effects.push(Effect::ClearPersistentMining);
            return out;
        }
        let mut mining = self.keep_active(i);
        if i.triggered {
            mining = true;
        }
        if self.pending_tool_equip {
            out.effects.push(Effect::ClearPersistentMining);
            if i.equip_in_progress {
                if !i.equip_is_hand_in || !mining || !i.should_abort_equip {
                    return out;
                }
                self.pending_tool_equip = false;
                if !i.equip_abort_allowed {
                    return out;
                }
            }
            self.pending_tool_equip = false;
        }
        if i.functional_swap_to_offhand {
            return out;
        }
        if self.was_mining && self.last_stack != i.stack {
            self.pending_tool_equip = true;
            out.effects.extend([
                Effect::TriggerImmediateMainEquip,
                Effect::ClearPersistentMining,
            ]);
            self.clear_all();
            self.pending_tool_equip = true;
            return out;
        }
        if (mining || i.triggered)
            && i.equip_in_progress
            && i.should_abort_equip
            && !i.equip_abort_allowed
        {
            return out;
        }
        let active_selection = i.selection.as_ref().or(self.selection.as_ref()).cloned();
        if !mining
            && !i.triggered
            && self.phase != Phase::None
            && active_selection.as_ref().is_some_and(|s| s.stop_immediate)
            && !i.attack_down
        {
            let skip = active_selection.as_ref().is_some_and(|s| s.skip_blend_out);
            out.request = Some(self.stop_request(Command::StopImmediate {
                blend_seconds: if skip { 0.0 } else { 0.3 },
            }));
            out.effects
                .extend([Effect::SetLooping(false), Effect::ClearPersistentMining]);
            self.clear_all();
            return out;
        }
        if !mining && !i.triggered && self.phase == Phase::None {
            out.effects.push(Effect::ClearPersistentMining);
            self.was_mining = false;
            return out;
        }
        out.effects.push(Effect::SetPersistentMining);
        self.tick_phases(i, mining, &mut out);
        self.was_mining = mining;
        out
    }

    fn tick_phases(&mut self, i: &MiningInput, mining: bool, out: &mut Output) {
        if i.triggered {
            self.register_trigger(i.tick);
        }
        if self.phase == Phase::None && i.selection.is_none() {
            if mining {
                out.effects.push(Effect::SetLooping(true));
                if !self.was_mining || !i.playback.playing || self.last_stack != i.stack {
                    out.effects.push(Effect::LegacyMiningFallback);
                }
            }
            return;
        }
        let changed = self.phase == Phase::None
            || self.last_v2_stack != i.stack
            || self.selection.as_ref().map(|s| s.entry_id)
                != i.selection.as_ref().map(|s| s.entry_id);
        if changed {
            let Some(s) = i.selection.as_ref() else {
                self.reset_phases();
                return;
            };
            self.selection = Some(s.clone());
            out.request = Some(self.start_request(&s.start));
            self.last_stack = i.stack;
            self.last_v2_stack = i.stack;
            return;
        }
        if let Some(s) = i.selection.as_ref() {
            self.selection = Some(s.clone());
        }
        let Some(s) = self.selection.clone() else {
            return;
        };
        self.last_stack = i.stack;
        self.last_v2_stack = i.stack;
        if self.phase == Phase::End && (mining || i.triggered) {
            out.effects.push(Effect::ResetPoseImmediate);
            out.request = Some(self.start_request(&s.start));
            return;
        }
        match self.phase {
            Phase::Start => {
                if i.playback.current_clip(&s.start) {
                    out.effects.push(Effect::SetSpeedMultiplier(
                        s.start.speed_multiplier * self.start_speed,
                    ));
                }
                if i.playback.near_end(&s.start, 0.25) {
                    if i.attack_down && mining {
                        out.request = Some(self.loop_request(&s.looping, &i.playback, true));
                    } else {
                        out.request = Some(self.end_request(&s.end, &i.playback));
                    }
                }
            }
            Phase::Loop => {
                if !i.attack_down || !mining {
                    out.effects.push(Effect::SetLooping(false));
                    out.request = Some(self.end_request(&s.end, &i.playback));
                } else if !i.playback.playing_clip(&s.looping) {
                    out.request = Some(self.loop_request(&s.looping, &i.playback, false));
                } else {
                    out.effects.push(Effect::SetLooping(true));
                }
            }
            Phase::LoopEndPending => {
                if i.triggered {
                    if !mining {
                        return;
                    }
                    out.effects.push(Effect::ResetPoseImmediate);
                    out.request = Some(self.start_request(&s.start));
                } else if i.playback.playing_clip(&s.looping) {
                    out.effects.push(Effect::SetLooping(false));
                } else {
                    out.request = Some(self.end_request(&s.end, &i.playback));
                }
            }
            Phase::End if !i.playback.playing_clip(&s.end) && !i.playback.current_clip(&s.end) => {
                self.reset_phases()
            }
            _ => {}
        }
    }
    fn keep_active(&mut self, i: &MiningInput) -> bool {
        if i.raw_mining {
            self.stop_grace_ticks = 0;
            true
        } else if self.was_mining && self.min_stop_tick >= 0 && i.tick < self.min_stop_tick {
            true
        } else if self.was_mining
            && i.attack_down
            && (i.aiming_trackable_block && self.stop_grace_ticks < 2
                || i.recent_target_90ms && self.stop_grace_ticks < 1
                || i.recent_chain_180ms && self.stop_grace_ticks < 2)
        {
            self.stop_grace_ticks += 1;
            true
        } else {
            false
        }
    }
    pub fn register_trigger(&mut self, tick: i64) {
        let target = if self.last_v2_trigger >= 0 {
            let interval = (tick - self.last_v2_trigger).max(1);
            let n = (1.0 - (interval - 1) as f32 / 5.0).clamp(0.0, 1.0);
            1.0 + n * n * (2.35 - 1.0)
        } else {
            1.0
        };
        self.start_speed = (self.start_speed + 0.45 * (target - self.start_speed)).clamp(1.0, 2.35);
        self.last_v2_trigger = tick;
    }
    fn start_request(&self, clip: &ClipSpec) -> Request {
        Request {
            priority: 60,
            deferred: true,
            phase: Phase::Start,
            expected: None,
            command: Command::Play(PlayRequest {
                clip: clip.clone(),
                hand: Hand::Main,
                looping: false,
                hold: false,
                speed_multiplier: clip.speed_multiplier * self.start_speed,
                blend_seconds: 0.0,
                return_blend_seconds: None,
                stop_after_current: false,
                start_time: 0.0,
                expression_time: 0.0,
                style: PlayStyle::Immediate,
            }),
        }
    }
    fn loop_request(&self, clip: &ClipSpec, p: &Playback, exact: bool) -> Request {
        let play = PlayRequest {
            clip: clip.clone(),
            hand: Hand::Main,
            looping: true,
            hold: false,
            speed_multiplier: clip.speed_multiplier,
            blend_seconds: if p.playing { 0.06 } else { 0.0 },
            return_blend_seconds: None,
            stop_after_current: false,
            start_time: 0.0,
            expression_time: 0.0,
            style: if p.playing {
                PlayStyle::DynamicFromCurrent
            } else {
                PlayStyle::Immediate
            },
        };
        Request {
            priority: 60,
            deferred: true,
            phase: Phase::Loop,
            expected: None,
            command: if exact {
                Command::Handoff {
                    clip: clip.clone(),
                    hand: Hand::Main,
                    speed_multiplier: clip.speed_multiplier,
                    tick_seconds: 0.05,
                    blend_seconds: 0.0,
                    fallback: play,
                }
            } else {
                Command::Play(play)
            },
        }
    }
    fn end_request(&self, clip: &ClipSpec, p: &Playback) -> Request {
        // Source's easing/blendDuration arguments (.08 smooth/.12 back) are
        // unused inside transitionToMiningV2End: the effective path rebases
        // from current with rebaseReturnSpeedNext=4.5, or skips entrance.
        Request {
            priority: 60,
            deferred: true,
            phase: Phase::End,
            expected: None,
            command: Command::Play(PlayRequest {
                clip: clip.clone(),
                hand: Hand::Main,
                looping: false,
                hold: false,
                speed_multiplier: clip.speed_multiplier,
                blend_seconds: 0.0,
                return_blend_seconds: None,
                stop_after_current: false,
                start_time: 0.0,
                expression_time: 0.0,
                style: if p.playing {
                    PlayStyle::RebasedReturn { speed: 4.5 }
                } else {
                    PlayStyle::Immediate
                },
            }),
        }
    }
    fn stop_request(&self, command: Command) -> Request {
        Request {
            priority: 60,
            deferred: false,
            phase: self.phase,
            expected: None,
            command,
        }
    }
    fn reset_phases(&mut self) {
        self.phase = Phase::None;
        self.selection = None;
        self.active_clip = None;
        self.last_v2_stack = 0;
        self.last_v2_trigger = -1;
        self.start_speed = 1.0;
    }
    fn clear_all(&mut self) {
        self.reset_phases();
        self.was_mining = false;
        self.last_stack = 0;
        self.min_stop_tick = -1;
        self.stop_grace_ticks = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn clip(name: &str, length: f32) -> ClipSpec {
        ClipSpec {
            name: name.into(),
            length,
            speed_multiplier: 1.0,
            hold: false,
        }
    }
    fn selection() -> Selection {
        Selection {
            entry_id: 1,
            start: clip("start", 1.0),
            looping: clip("loop", 2.0),
            end: clip("end", 0.5),
            skip_blend_in: false,
            skip_blend_out: false,
            stop_immediate: false,
        }
    }
    fn playback(name: &str, id: u64, time: f32) -> Playback {
        Playback {
            clip: Some(name.into()),
            id,
            playing: true,
            time,
            ..Default::default()
        }
    }
    fn consume() -> ConsumeInput {
        ConsumeInput {
            use_down: true,
            using_item: true,
            using_consume: true,
            using_stack: 10,
            current_hand_stack: 10,
            selection: Some(ConsumeSelection::Phases(selection())),
            ..Default::default()
        }
    }
    fn commit_consume(c: &mut ConsumeController, r: &Request, id: u64) {
        assert!(c.commit(r, &playback(&r.target().unwrap().name, id, 0.0)));
    }
    fn mining() -> MiningInput {
        MiningInput {
            tick: 10,
            raw_mining: true,
            attack_down: true,
            stack: 10,
            selection: Some(selection()),
            ..Default::default()
        }
    }

    #[test]
    fn consume_phase_changes_only_after_winning_arbitration() {
        let mut c = ConsumeController::default();
        let i = consume();
        let r = c.tick(&i).request.unwrap();
        assert_eq!(c.phase(), Phase::None);
        assert!(
            matches!(&r.command, Command::Play(p) if p.blend_seconds == 0.25 && !p.looping && p.stop_after_current)
        );
        commit_consume(&mut c, &r, 1);
        assert_eq!(c.phase(), Phase::Start);
    }
    #[test]
    fn consume_lead_requests_handoff_at_exact_quarter_second_boundary() {
        let mut c = ConsumeController::default();
        let mut i = consume();
        let r = c.tick(&i).request.unwrap();
        commit_consume(&mut c, &r, 1);
        i.playback = playback("start", 1, 0.74999);
        assert!(c.tick(&i).request.is_none());
        i.playback.time = 0.75;
        let r = c.tick(&i).request.unwrap();
        assert!(matches!(
            r.command,
            Command::Handoff {
                blend_seconds: 0.1,
                tick_seconds: 0.05,
                ..
            }
        ));
        assert_eq!(c.phase(), Phase::Start);
    }
    #[test]
    fn consume_cold_skip_does_not_remove_blend_from_existing_playback() {
        let mut i = consume();
        if let Some(ConsumeSelection::Phases(s)) = &mut i.selection {
            s.skip_blend_in = true;
        }
        let mut c = ConsumeController::default();
        assert!(
            matches!(c.tick(&i).request.unwrap().command, Command::Play(p) if p.style == PlayStyle::Immediate && p.blend_seconds == 0.0)
        );
        let mut c = ConsumeController::default();
        i.playback = playback("unrelated", 5, 0.3);
        assert!(
            matches!(c.tick(&i).request.unwrap().command, Command::Play(p) if matches!(p.style, PlayStyle::Variation { .. }) && p.blend_seconds == 0.25)
        );
    }
    #[test]
    fn consume_stack_change_restarts_and_hand_change_interrupts() {
        let mut c = ConsumeController::default();
        let mut i = consume();
        let r = c.tick(&i).request.unwrap();
        commit_consume(&mut c, &r, 1);
        i.playback = playback("start", 1, 0.4);
        i.using_stack = 11;
        assert_eq!(c.tick(&i).request.unwrap().phase, Phase::Start);
        i.hand = Hand::Off;
        let out = c.tick(&i);
        assert!(matches!(
            out.request.unwrap().command,
            Command::StopImmediate {
                blend_seconds: 0.25
            }
        ));
        assert!(out.effects.contains(&Effect::TriggerOffhandPickup));
        assert!(!c.equip_blocked());
    }
    #[test]
    fn completed_consume_swaps_result_on_last_frame_and_waits_through_blend() {
        let mut c = ConsumeController::default();
        let mut i = consume();
        let r = c.tick(&i).request.unwrap();
        commit_consume(&mut c, &r, 1);
        i.playback = playback("start", 1, 0.3);
        i.using_item = false;
        i.using_consume = false;
        i.current_hand_stack = 20;
        let r = c.tick(&i).request.unwrap();
        assert_eq!(r.phase, Phase::End);
        assert!(c.finishing());
        commit_consume(&mut c, &r, 2);
        i.playback = playback("end", 2, 0.49989);
        assert!(!c.tick(&i).effects.contains(&Effect::RequestRenderedSync));
        i.playback.time = 0.49991;
        assert!(c.tick(&i).effects.contains(&Effect::RequestRenderedSync));
        i.playback.playing = false;
        i.playback.blending = true;
        assert!(c.tick(&i).request.is_none());
        assert!(c.equip_blocked());
        i.playback.blending = false;
        c.tick(&i);
        assert!(!c.equip_blocked());
        assert!(c.use_suppressed());
    }
    #[test]
    fn exact_handoff_waits_for_actual_tick_end_and_converts_consume_overflow() {
        let p = playback("start", 1, 0.75);
        assert_eq!(
            exact_handoff(&p, 1.0, 0.05, 2.0, 0.05, 0.1, |s| s),
            HandoffDecision::Wait
        );
        let p = playback("start", 1, 0.99);
        let HandoffDecision::Start {
            time,
            consumes_tick,
            ..
        } = exact_handoff(&p, 1.0, 0.05, 2.0, 0.05, 0.1, |s| s * 2.0)
        else {
            panic!("handoff");
        };
        assert!((time - 0.08).abs() < 1e-5);
        assert!(consumes_tick);
        let HandoffDecision::Start { time, .. } =
            exact_handoff(&p, 1.0, 0.05, 2.0, 0.05, 0.0, |_| {
                panic!("mining does not convert overflow")
            })
        else {
            panic!("handoff");
        };
        assert!((time - 0.04).abs() < 1e-5);
    }
    #[test]
    fn mining_start_then_exact_loop_then_rebased_end() {
        let mut c = MiningController::default();
        let mut i = mining();
        let r = c.tick(&i).request.unwrap();
        assert!(
            matches!(r.command, Command::Play(ref p) if p.style == PlayStyle::Immediate && p.blend_seconds == 0.0)
        );
        assert!(c.commit(&r, &playback("start", 1, 0.0), i.tick));
        i.tick += 1;
        i.playback = playback("start", 1, 0.75);
        let r = c.tick(&i).request.unwrap();
        assert!(matches!(
            r.command,
            Command::Handoff {
                blend_seconds: 0.0,
                ..
            }
        ));
        assert!(c.commit(&r, &playback("loop", 2, 0.0), i.tick));
        i.tick += 1;
        i.playback = playback("loop", 2, 0.1);
        i.attack_down = false;
        i.raw_mining = false;
        let r = c.tick(&i).request.unwrap();
        assert!(
            matches!(r.command, Command::Play(p) if p.style == PlayStyle::RebasedReturn { speed: 4.5 } && !p.looping)
        );
    }
    #[test]
    fn mining_retrigger_speed_has_source_response_curve() {
        let mut c = MiningController::default();
        c.register_trigger(10);
        assert_eq!(c.start_speed(), 1.0);
        c.register_trigger(11);
        assert!((c.start_speed() - 1.6075).abs() < 1e-6);
        c.register_trigger(17);
        assert!((c.start_speed() - 1.334125).abs() < 1e-6);
    }
    #[test]
    fn mining_minimum_two_ticks_then_two_target_grace_ticks() {
        let mut c = MiningController::default();
        let mut i = mining();
        let r = c.tick(&i).request.unwrap();
        c.commit(&r, &playback("start", 1, 0.0), i.tick);
        i.playback = playback("start", 1, 0.1);
        i.raw_mining = false;
        i.tick = 11;
        c.tick(&i);
        assert!(c.was_mining());
        i.aiming_trackable_block = true;
        i.tick = 12;
        c.tick(&i);
        assert!(c.was_mining());
        i.tick = 13;
        c.tick(&i);
        assert!(c.was_mining());
        i.tick = 14;
        c.tick(&i);
        assert!(!c.was_mining());
    }
    #[test]
    fn mining_stop_immediate_uses_point_three_return_and_skip_can_zero_it() {
        let mut c = MiningController::default();
        let mut i = mining();
        i.selection.as_mut().unwrap().stop_immediate = true;
        let r = c.tick(&i).request.unwrap();
        c.commit(&r, &playback("start", 1, 0.0), i.tick);
        i.tick = 12;
        i.raw_mining = false;
        i.attack_down = false;
        i.playback = playback("start", 1, 0.3);
        assert!(matches!(
            c.tick(&i).request.unwrap().command,
            Command::StopImmediate { blend_seconds: 0.3 }
        ));
    }
    #[test]
    fn mining_end_does_not_reset_while_same_clip_is_blending() {
        let mut c = MiningController::default();
        let mut i = mining();
        let r = c.tick(&i).request.unwrap();
        c.commit(&r, &playback("start", 1, 0.0), i.tick);
        i.tick = 12;
        i.raw_mining = false;
        i.attack_down = false;
        i.playback = playback("start", 1, 0.8);
        let r = c.tick(&i).request.unwrap();
        c.commit(&r, &playback("end", 2, 0.0), i.tick);
        i.tick = 13;
        i.playback = Playback {
            clip: Some("end".into()),
            id: 2,
            blending: true,
            ..Default::default()
        };
        assert!(c.tick(&i).request.is_none());
        assert_eq!(c.phase(), Phase::End);
        i.playback.clip = None;
        c.tick(&i);
        assert_eq!(c.phase(), Phase::None);
    }
}
