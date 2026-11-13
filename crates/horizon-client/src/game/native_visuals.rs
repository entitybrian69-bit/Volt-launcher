//! Frame-local visual requests from woven native mods: the world-space line
//! path of upstream's `native_visuals` module (native hook API 46), ported so
//! mods like Elytra Trails can draw per-frame gizmo lines. The upstream rim
//! and colored-light budgets (`budget_visuals`, `dynamic_light`) are not
//! ported yet; a bundle that needs those still pins its exact Horizon source.
use horizon_client::gizmos;
use horizon_client::gizmos::Line;

use super::Game;

/// Most world-space lines one frame accepts from mods' visual injections,
/// mirroring upstream's per-mod outline budget (`RIM_EDGES_PER_MOD`): a
/// runaway request cannot grow the frame's vertex buffer without bound.
pub(super) const LINES_PER_FRAME: usize = 4096;

impl Game<'_> {
    /// Frame-local requests never persist after a mod is removed or a world
    /// changes: woven mods' HEAD injections push this frame's world-space
    /// `Line`s into `lines`, and they draw with the frame's debug gizmos.
    /// `alpha` is the frame's interpolation fraction, as upstream passes it
    /// (`native_visuals::Game::native_frame_visuals`). Like `native_tick`,
    /// the body is the extension point; the caller scrubs what was pushed
    /// with `scrub_mod_lines` before it reaches the vertex buffer.
    pub(super) fn native_frame_visuals(&self, alpha: f64, lines: &mut Vec<gizmos::Line>) {
        let _ = (alpha, lines);
    }
}

/// Keeps a frame's mod-pushed lines drawable: past `from`, only finite
/// geometry with a finite positive width survives, and at most
/// `LINES_PER_FRAME` of it, so no injection can poison or flood the frame.
/// Order is preserved.
pub(super) fn scrub_mod_lines(lines: &mut Vec<Line>, from: usize) {
    let from = from.min(lines.len());
    let mut kept = from;
    for i in from..lines.len() {
        if kept - from >= LINES_PER_FRAME {
            break;
        }
        let l = &lines[i];
        if l.start.iter().chain(&l.end).all(|v| v.is_finite())
            && l.width.is_finite()
            && l.width > 0.0
        {
            lines.swap(kept, i);
            kept += 1;
        }
    }
    lines.truncate(kept);
}

#[cfg(test)]
mod tests {
    use super::{LINES_PER_FRAME, scrub_mod_lines};
    use horizon_client::gizmos::Line;

    fn line(x: f64, width: f32) -> Line {
        Line {
            start: [x, 64.0, 0.0],
            end: [x, 65.0, 0.0],
            color: 0xFFFF_0000,
            width,
        }
    }

    #[test]
    fn earlier_frame_lines_are_untouched_and_bad_mod_lines_drop() {
        let mut lines = vec![line(f64::NAN, 1.0), line(1.0, 1.0)];
        let from = lines.len();
        lines.push(line(2.0, 1.0));
        lines.push(line(f64::NAN, 1.0));
        lines.push(line(3.0, f32::NAN));
        lines.push(line(4.0, 0.0));
        lines.push(line(5.0, -1.0));
        lines.push(line(6.0, 2.5));
        scrub_mod_lines(&mut lines, from);
        // The frame's own lines stay, even a NaN one: only mod pushes are ours to police.
        assert_eq!(lines.len(), from + 2);
        assert_eq!(lines[from].start[0], 2.0);
        assert_eq!(lines[from + 1].start[0], 6.0);
    }

    #[test]
    fn a_flood_is_capped_at_the_frame_budget_in_order() {
        let mut lines: Vec<Line> = (0..LINES_PER_FRAME + 100)
            .map(|i| line(i as f64, 1.0))
            .collect();
        scrub_mod_lines(&mut lines, 0);
        assert_eq!(lines.len(), LINES_PER_FRAME);
        assert_eq!(lines[LINES_PER_FRAME - 1].start[0], (LINES_PER_FRAME - 1) as f64);
    }

    #[test]
    fn a_from_past_the_end_is_safe() {
        let mut lines = vec![line(1.0, 1.0)];
        scrub_mod_lines(&mut lines, 5);
        assert_eq!(lines.len(), 1);
    }
}
