//! Runtime timeline with live interrupted sources and independent expression clocks.
use crate::{
    Clip, actions as action,
    attachments::{AttachmentFrame, sample_frame},
    skeleton::Skeleton,
};
use std::sync::Arc;
#[derive(Clone)]
pub struct Playback {
    pub from: Option<Box<Self>>,
    pub out_weight: f32,
    pub previous_out_weight: f32,
    pub id: u64,
    pub expression: f32,
    pub previous_expression: f32,
    pub return_fade: f32,
    pub skip_tick: bool,
    pub clip: Arc<Clip>,
    pub hand: action::Hand,
    pub previous: f32,
    pub time: f32,
    pub multiplier: f32,
    pub looping: bool,
    pub released: bool,
    pub hold: bool,
    pub fade: f32,
    pub stopping: bool,
    pub weight: f32,
    pub previous_weight: f32,
}

impl Playback {
    pub fn time_at(&self, alpha: f32) -> f32 {
        if self.time < self.previous {
            self.time
        } else {
            self.previous + (self.time - self.previous) * alpha.clamp(0., 1.)
        }
    }
    pub fn expression_at(&self, alpha: f32) -> f32 {
        self.previous_expression
            + (self.expression - self.previous_expression) * alpha.clamp(0., 1.)
    }
    pub fn advance(&mut self, speed: f32) {
        self.previous = self.time;
        self.previous_expression = self.expression;
        self.previous_weight = self.weight;
        self.previous_out_weight = self.out_weight;
        if let Some(from) = &mut self.from {
            from.advance(speed);
        }
        if !self.stopping && !std::mem::take(&mut self.skip_tick) {
            let delta =
                0.05 * crate::playback::rate(&self.clip, self.time, speed, self.multiplier, 1.);
            self.time += delta;
            self.expression += delta;
        }
        if (self.looping || self.clip.looped && !self.released)
            && !self.stopping
            && self.clip.length > 0.
        {
            self.time = self.time.rem_euclid(self.clip.length);
        } else if self.time >= self.clip.length {
            self.time = self.clip.length;
            if !self.hold && !self.stopping {
                self.stopping = true;
                self.return_fade = self.return_fade.max(0.001);
            }
        }
        self.weight = (self.weight + 0.05 / self.fade.max(0.001)).min(1.);
        if self.stopping {
            self.out_weight = (self.out_weight - 0.05 / self.return_fade.max(0.001)).max(0.);
        }
        if self.previous_weight >= 1. {
            self.from = None;
        }
    }
    pub fn frame(
        &self,
        skeleton: &Skeleton<'_>,
        left: bool,
        main_left: bool,
        offhand_item: bool,
        alpha: f32,
    ) -> AttachmentFrame {
        let time = self.time_at(alpha);
        let owns = (main_left != (self.hand == action::Hand::Off)) == left
            || crate::policy::dual_handed(&self.clip, time, offhand_item, offhand_item);
        let rest = sample_frame(skeleton, left, None, 1., true);
        let target = sample_frame(
            skeleton,
            left,
            Some((&self.clip, time, self.expression_at(alpha))),
            1.,
            owns,
        );
        let from = self.from.as_ref().map_or(rest, |p| {
            p.frame(skeleton, left, main_left, offhand_item, alpha)
        });
        let w = (self.previous_weight + (self.weight - self.previous_weight) * alpha).clamp(0., 1.);
        let entrance = w * w * (3. - 2. * w);
        let blended = from.blend(target, entrance);
        let out = self.previous_out_weight + (self.out_weight - self.previous_out_weight) * alpha;
        rest.blend(blended, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn playback(name: &str) -> Playback {
        let json = serde_json::json!({
            "animations": {
                (name): {
                    "animation_length": 1,
                    "loop": true,
                    "effects": "use_keyframes_v2",
                    "bones": {
                        "body": {
                            "position": {"0":[0,0,0],"1":[16,0,0]}
                        }
                    }
                }
            }
        })
        .to_string();
        let clip = Arc::new(
            crate::load_clips(json.as_bytes())
                .unwrap()
                .remove(name)
                .unwrap(),
        );
        Playback {
            from: None,
            out_weight: 1.,
            previous_out_weight: 1.,
            id: 1,
            clip,
            hand: action::Hand::Main,
            previous: 0.,
            time: 0.,
            expression: 0.,
            previous_expression: 0.,
            return_fade: 0.25,
            skip_tick: false,
            multiplier: 1.,
            looping: false,
            released: false,
            hold: false,
            fade: 0.2,
            stopping: false,
            weight: 1.,
            previous_weight: 1.,
        }
    }
    #[test]
    fn declared_loop_survives_endpoint_and_expression_clock_does_not_wrap() {
        let mut p = playback("test");
        p.time = 0.98;
        p.expression = 0.98;
        p.advance(1.);
        assert!((p.time - 0.03).abs() < 1e-5);
        assert!((p.expression - 1.03).abs() < 1e-5);
        assert!(!p.stopping);
        p.released = true;
        p.time = 0.98;
        p.advance(1.);
        assert_eq!(p.time, 1.);
        assert!(p.stopping);
        for _ in 0..5 {
            p.advance(1.);
        }
        assert_eq!(p.out_weight, 0.);
    }
    #[test]
    fn interruption_preserves_live_outgoing_pose_and_nested_transition_origin() {
        let resources = crate::resources::embedded().unwrap();
        let skeleton = Skeleton::new(&resources.bones).unwrap();
        let mut old = playback("old");
        old.time = 0.3;
        old.previous = 0.3;
        let expected = old.frame(&skeleton, false, false, false, 1.);
        let mut next = playback("next");
        next.time = 0.8;
        next.previous = 0.8;
        next.weight = 0.;
        next.previous_weight = 0.;
        next.from = Some(Box::new(old));
        assert!(
            next.frame(&skeleton, false, false, false, 1.)
                .body
                .abs_diff_eq(expected.body, 1e-6)
        );
        next.advance(1.);
        let outgoing = next.from.as_ref().unwrap();
        assert!((outgoing.time - 0.35).abs() < 1e-6);
        let current = next.frame(&skeleton, false, false, false, 1.);
        let from = outgoing.frame(&skeleton, false, false, false, 1.);
        let to = sample_frame(
            &skeleton,
            false,
            Some((&next.clip, next.time, next.expression)),
            1.,
            true,
        );
        assert!(current.body.abs_diff_eq(from.blend(to, 0.15625).body, 1e-6));
        let mut interrupt = playback("interrupt");
        interrupt.weight = 0.;
        interrupt.previous_weight = 0.;
        interrupt.from = Some(Box::new(next));
        assert!(
            interrupt
                .frame(&skeleton, false, false, false, 1.)
                .body
                .abs_diff_eq(current.body, 1e-6)
        );
    }
}
