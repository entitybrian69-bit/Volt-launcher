//! Timeline ownership rules from PunchyAnimationManager and PoseHandler.
use crate::Clip;

fn last(clip: &Clip, effects: &[&str], time: f32) -> Option<f32> {
    clip.timeline
        .iter()
        .filter(|(t, e)| *t <= time && effects.iter().any(|name| e.contains(*name)))
        .map(|(t, _)| *t)
        .max_by(f32::total_cmp)
}

/// `offhand_context` also includes a visible outgoing offhand equip pose.
pub fn dual_handed(clip: &Clip, time: f32, offhand_item: bool, offhand_context: bool) -> bool {
    if !clip.effect_active("dual_handed", time) {
        return clip.effect_active("dual_handed_if_has_off_item", time) && offhand_context;
    }
    if !offhand_item {
        return true;
    }
    let start = last(
        clip,
        &["start_here_if_off_hand_item", "start_if_offhand_item"],
        time,
    );
    let stop = last(clip, &["stop_if_offhand_item"], time);
    match (start, stop) {
        (_, None) => true,
        (Some(start), Some(stop)) => start > stop,
        (None, Some(_)) => false,
    }
}

pub fn hide_empty_offhand(clip: &Clip, time: f32, offhand_item: bool) -> bool {
    if offhand_item {
        return false;
    }
    let hide = last(clip, &["hide_offhand_if_empty"], time);
    let show = last(clip, &["show_offhand_if_empty"], time);
    hide.is_some_and(|h| show.is_none_or(|s| h > s))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_windows_respect_start_stop_alias_and_equal_time_stop() {
        let clips = crate::load_clips(br#"{"animations":{"window":{"effects":"dual_handed","timeline":{"1":"stop_if_offhand_item","2":"start_if_offhand_item","3":["start_here_if_off_hand_item","stop_if_offhand_item"]}},"delayed":{"timeline":{"1":"dual_handed_if_has_off_item"}}}}"#).unwrap();
        let clip = &clips["window"];
        for (time, expected) in [
            (0., true),
            (1., false),
            (1.99, false),
            (2., true),
            (3., false),
        ] {
            assert_eq!(dual_handed(clip, time, true, true), expected, "{time}");
            assert!(dual_handed(clip, time, false, false));
        }
        assert!(!dual_handed(&clips["delayed"], 0.99, true, true));
        assert!(dual_handed(&clips["delayed"], 1., true, true));
        assert!(!dual_handed(&clips["delayed"], 1., false, false));
    }
    #[test]
    fn bundled_inspect_reveals_and_hides_empty_offhand_at_authored_times() {
        let clips = crate::load_clips(&crate::resources::embedded().unwrap().animations).unwrap();
        let clip = &clips["new_inspect_right"];
        assert!(hide_empty_offhand(clip, 0., false));
        assert!(hide_empty_offhand(clip, 3.79, false));
        assert!(!hide_empty_offhand(clip, 3.8, false));
        assert!(!hide_empty_offhand(clip, 9.03, false));
        assert!(hide_empty_offhand(clip, 9.05, false));
        assert!(!hide_empty_offhand(clip, 9.05, true));
    }
}
