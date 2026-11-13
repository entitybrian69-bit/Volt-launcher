//! PoseHandler.computeAnimDelta and resolveTimelineSpeed policies. A playback
//! multiplier is not a final rate: equip and consume have explicit overrides.
use crate::Clip;

pub fn equip(name: &str) -> bool {
    name == "hand_in"
        || name == "hand_out"
        || name.starts_with("hand_in_")
        || name.starts_with("hand_out_")
}
fn unhanded(name: &str) -> &str {
    name.strip_suffix("_left")
        .or_else(|| name.strip_suffix("_right"))
        .unwrap_or(name)
}
pub fn manager_multiplier(name: &str, configured: f32, is_swimming_clip: bool) -> f32 {
    match unhanded(name) {
        "use_bow"
        | "use_shield"
        | "use_crossbow"
        | "use_crossbow_shot"
        | "use_trident"
        | "use_trident_throw"
        | "use_trident_throw_loyalty"
        | "use_trident_throw_riptide"
        | "attack_mace"
        | "dry_out"
        | "hand_in"
        | "hand_out" => 1. / configured.max(0.01),
        "interact" => 5.4 / configured.max(0.01),
        _ if is_swimming_clip => 1. / configured.max(0.01),
        _ => 1.,
    }
}
fn numeric_prefix(s: &str) -> Option<f32> {
    let b = s.as_bytes();
    let mut i = usize::from(matches!(b.first(), Some(b'+' | b'-')));
    let start = i;
    while b.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    if i == start {
        return None;
    }
    if matches!(b.get(i), Some(b'.' | b',')) && b.get(i + 1).is_some_and(u8::is_ascii_digit) {
        i += 1;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
    }
    s[..i]
        .replace(',', ".")
        .parse::<f32>()
        .ok()
        .filter(|v| v.is_finite())
}
/// Last timeline speed up to `time`; clip static effects do not set this rate.
/// The engine retains a sorted set at one timestamp; distinct speeds at exactly
/// one time require the original insertion order and are not parity-certified.
pub fn timeline_speed(clip: &Clip, time: f32, configured: f32) -> Option<f32> {
    let mut speed = None;
    for (_, effects) in clip.timeline.iter().filter(|(t, _)| *t <= time) {
        for effect in effects {
            let normalized = effect.trim().to_lowercase();
            if let Some(s) = normalized
                .strip_prefix("anim_speed_")
                .and_then(numeric_prefix)
            {
                speed = Some(s.clamp(0.5, 5.5));
            }
        }
    }
    let ordinary = !equip(&clip.name)
        && !matches!(
            clip.name.as_str(),
            "use_eat" | "use_drink" | "falling_in_water"
        );
    speed.map(|s| {
        if ordinary && manager_multiplier(&clip.name, configured, false) == 1. {
            s * configured / 5.5
        } else {
            s
        }
    })
}
pub fn rate(clip: &Clip, time: f32, configured: f32, multiplier: f32, timeline_scale: f32) -> f32 {
    if let Some(speed) = timeline_speed(clip, time, configured) {
        return speed * timeline_scale;
    }
    if equip(&clip.name) {
        1.
    } else if matches!(
        clip.name.as_str(),
        "use_eat" | "use_drink" | "falling_in_water"
    ) {
        multiplier
    } else {
        configured * multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn clip(name: &str) -> Clip {
        crate::load_clips(
            format!(r#"{{"animations":{{"{name}":{{"animation_length":3}}}}}}"#).as_bytes(),
        )
        .unwrap()
        .remove(name)
        .unwrap()
    }
    #[test]
    fn equip_ignores_global_and_multiplier_while_consume_keeps_multiplier() {
        assert_eq!(rate(&clip("hand_in_sword"), 0., 5.5, 0.01, 1.), 1.);
        assert_eq!(rate(&clip("hand_out"), 0., 0.5, 20., 1.), 1.);
        assert_eq!(rate(&clip("use_eat"), 0., 5.5, 0.75, 1.), 0.75);
        assert_eq!(rate(&clip("use_drink_right"), 0., 5.5, 1., 1.), 5.5);
        assert!((rate(&clip("attack_1"), 0., 5.5, 1.12, 1.) - 6.16).abs() < 1e-6);
        assert!((manager_multiplier("use_bow_right", 5.5, false) * 5.5 - 1.).abs() < 1e-6);
    }
    #[test]
    fn timeline_speed_parses_source_numeric_prefix_and_time_boundaries() {
        let mut c = clip("attack_1");
        c.timeline = vec![
            (0., ["anim_speed_4,5x".into()].into()),
            (1., ["anim_speed_99".into()].into()),
            (2., ["anim_speed_bad".into()].into()),
        ];
        assert_eq!(timeline_speed(&c, -0.01, 5.5), None);
        assert_eq!(rate(&c, 0., 5.5, 12., 1.), 4.5);
        assert_eq!(rate(&c, 1., 5.5, 12., 1.), 5.5);
        assert_eq!(rate(&c, 2., 2.75, 12., 1.), 2.75);
        c.name = "hand_in_sword".into();
        assert_eq!(rate(&c, 0., 2.75, 12., 2.), 9.);
        assert_eq!(numeric_prefix(".5"), None);
        assert_eq!(numeric_prefix("+2.3.7"), Some(2.3));
    }
}
