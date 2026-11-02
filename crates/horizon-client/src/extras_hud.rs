//! HUD overlays in the corners and 3D viewport: Volt's minimal feature-rich
//! minimap, 3D waypoint beams/markers/edge chevrons, Accurate Block Placement
//! crosshair preview, and mods' corner panels.
use crate::{
    extras::Corner,
    hud::{HudAtlas, HudBatch, Rect, rgba},
    volt_features::{MinimapFrame, ProjectedWaypoint},
};

/// The gap between an overlay and the screen edge or the next overlay.
const MARGIN: f32 = 2.0;
const BACKGROUND: u32 = 0x7800_0000;

/// Where the next overlay goes in each corner.
pub struct Corners {
    width: f32,
    height: f32,
    /// Distance from the corner's top or bottom edge, per `Corner::ALL`.
    used: [f32; 4],
}
impl Corners {
    pub fn reserve(&mut self, corner: Corner, height: f32) {
        if let Some(i) = Corner::ALL.iter().position(|c| *c == corner) {
            self.used[i] = self.used[i].max(height);
        }
    }
    /// `top_right` reserves the status effect icons' rows.
    pub fn new(width: f32, height: f32, top_right: f32) -> Self {
        Self {
            width,
            height,
            used: [0.0, top_right, 0.0, 0.0],
        }
    }
    /// The top-left position of a `w` x `h` box in `corner`, below (or
    /// above) what the corner already holds.
    pub fn place(&mut self, corner: Corner, w: f32, h: f32) -> (f32, f32) {
        let i = Corner::ALL.iter().position(|c| *c == corner).unwrap_or(0);
        let edge = self.used[i] + MARGIN;
        self.used[i] = edge + h;
        let x = if corner.left() {
            MARGIN
        } else {
            self.width - MARGIN - w
        };
        let y = if corner.top() {
            edge
        } else {
            self.height - edge - h
        };
        (x, y)
    }
}

/// Entity dot rendered on the Volt Minimap radar.
#[derive(Clone, Copy, Debug)]
pub struct RadarDot {
    pub dx: f32,
    pub dy: f32,
    pub rgb: [u8; 3],
    pub radius: f32,
}

/// Waypoint pin rendered on the Volt Minimap.
#[derive(Clone, Debug)]
pub struct MinimapPin {
    pub dx: f32,
    pub dy: f32,
    pub rgb: [u8; 3],
    pub clamped: bool,
}

/// Full visual specification for drawing the Volt Minimap and its HUD readouts.
pub struct MinimapLook {
    pub corner: Corner,
    pub size: f32,
    pub image: Rect,
    pub arrow: f32,
    pub north: Option<(f32, f32)>,
    pub round: bool,
    pub frame: MinimapFrame,
    pub opacity: u8,
    pub chunk_grid: bool,
    pub blocks_per_pixel: f32,
    pub player_world_xz: (f64, f64),
    pub rotate_yaw: Option<f32>,
    pub radar_dots: Vec<RadarDot>,
    pub waypoint_pins: Vec<MinimapPin>,
    pub info_lines: Vec<String>,
}

fn white_uv(atlas: &HudAtlas) -> [[f32; 2]; 4] {
    let r = atlas.white;
    [[r.u0, r.v0], [r.u1, r.v0], [r.u1, r.v1], [r.u0, r.v1]]
}

fn draw_diamond(batch: &mut HudAtlasBatch<'_>, cx: f32, cy: f32, rad: f32, color: u32) {
    batch.batch.quad(
        [
            [cx, cy - rad],
            [cx + rad, cy],
            [cx, cy + rad],
            [cx - rad, cy],
        ],
        white_uv(batch.atlas),
        color,
    );
}

struct HudAtlasBatch<'a> {
    batch: &'a mut HudBatch,
    atlas: &'a HudAtlas,
}

pub fn draw_minimap(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    corners: &mut Corners,
    look: &MinimapLook,
) {
    let size = look.size;
    let pad = match look.frame {
        MinimapFrame::CleanMinimal => 1.0,
        MinimapFrame::VoltCrimson | MinimapFrame::ClassicDark => 2.0,
    };
    let box_size = size + pad * 2.0;
    let (bx, by) = corners.place(look.corner, box_size, box_size);
    let (x, y) = (bx + pad, by + pad);
    let (cx, cy) = (x + size * 0.5, y + size * 0.5);
    let alpha = ((u16::from(look.opacity.clamp(20, 100)) * 255) / 100) as u8;

    // 1. Frame / bezel behind map:
    if !look.round {
        match look.frame {
            MinimapFrame::VoltCrimson => {
                batch.fill(atlas, bx, by, box_size, box_size, rgba(220, 38, 38, 225));
                batch.fill(
                    atlas,
                    bx + 1.0,
                    by + 1.0,
                    box_size - 2.0,
                    box_size - 2.0,
                    rgba(11, 12, 16, 235),
                );
            }
            MinimapFrame::ClassicDark => {
                batch.fill(atlas, bx, by, box_size, box_size, rgba(82, 82, 91, 220));
                batch.fill(
                    atlas,
                    bx + 1.0,
                    by + 1.0,
                    box_size - 2.0,
                    box_size - 2.0,
                    rgba(12, 12, 14, 235),
                );
            }
            MinimapFrame::CleanMinimal => {
                batch.fill(atlas, bx, by, box_size, box_size, rgba(0, 0, 0, 130));
            }
        }
    } else {
        // Subtle 24-segment circular bezel ring for Round mode:
        let ring_col = match look.frame {
            MinimapFrame::VoltCrimson => rgba(220, 38, 38, 220),
            MinimapFrame::ClassicDark => rgba(82, 82, 91, 210),
            MinimapFrame::CleanMinimal => rgba(0, 0, 0, 140),
        };
        let r_in = size * 0.5 - 0.5;
        let r_out = size * 0.5 + pad;
        let uv = white_uv(atlas);
        let segs = 28;
        for i in 0..segs {
            let a0 = (i as f32 / segs as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / segs as f32) * std::f32::consts::TAU;
            let (s0, c0) = a0.sin_cos();
            let (s1, c1) = a1.sin_cos();
            batch.quad(
                [
                    [cx + c0 * r_in, cy + s0 * r_in],
                    [cx + c0 * r_out, cy + s0 * r_out],
                    [cx + c1 * r_out, cy + s1 * r_out],
                    [cx + c1 * r_in, cy + s1 * r_in],
                ],
                uv,
                ring_col,
            );
        }
    }

    // 2. Terrain map sprite:
    batch.rect_uv(x, y, size, size, look.image, rgba(255, 255, 255, alpha));

    // 3. Optional 16x16 chunk boundary grid (north-up mode or non-rotated):
    if look.chunk_grid && look.rotate_yaw.is_none() {
        let scale = size / crate::minimap::SIZE as f32;
        let bpp = look.blocks_per_pixel.max(0.1);
        let chunk_px = 16.0 / bpp * scale;
        if chunk_px >= 4.0 {
            let half_blocks = (crate::minimap::SIZE as f64 * 0.5) * f64::from(bpp);
            let min_wx = (look.player_world_xz.0 - half_blocks).floor() as i32;
            let max_wx = (look.player_world_xz.0 + half_blocks).ceil() as i32;
            let min_wz = (look.player_world_xz.1 - half_blocks).floor() as i32;
            let max_wz = (look.player_world_xz.1 + half_blocks).ceil() as i32;
            let grid_col = rgba(255, 255, 255, 42);
            let half = size * 0.5;
            let mut gx = min_wx.div_euclid(16) * 16;
            while gx <= max_wx {
                let dx = (f64::from(gx) - look.player_world_xz.0) as f32 / bpp * scale;
                if dx > -half && dx < half {
                    let span = if look.round {
                        (half * half - dx * dx).max(0.0).sqrt()
                    } else {
                        half
                    };
                    batch.fill(atlas, cx + dx, cy - span, 0.75, span * 2.0, grid_col);
                }
                gx += 16;
            }
            let mut gz = min_wz.div_euclid(16) * 16;
            while gz <= max_wz {
                let dy = (f64::from(gz) - look.player_world_xz.1) as f32 / bpp * scale;
                if dy > -half && dy < half {
                    let span = if look.round {
                        (half * half - dy * dy).max(0.0).sqrt()
                    } else {
                        half
                    };
                    batch.fill(atlas, cx - span, cy + dy, span * 2.0, 0.75, grid_col);
                }
                gz += 16;
            }
        }
    }

    // 4. Live Entity Radar dots:
    for dot in &look.radar_dots {
        let px = cx + dot.dx;
        let py = cy + dot.dy;
        let r = dot.radius;
        batch.fill(
            atlas,
            px - r - 0.6,
            py - r - 0.6,
            (r + 0.6) * 2.0,
            (r + 0.6) * 2.0,
            rgba(0, 0, 0, 195),
        );
        batch.fill(
            atlas,
            px - r,
            py - r,
            r * 2.0,
            r * 2.0,
            rgba(dot.rgb[0], dot.rgb[1], dot.rgb[2], 245),
        );
    }

    // 5. Waypoint pins on the minimap:
    {
        let mut ab = HudAtlasBatch { batch, atlas };
        for pin in &look.waypoint_pins {
            let px = cx + pin.dx;
            let py = cy + pin.dy;
            let rad = if pin.clamped { 2.6 } else { 3.6 };
            draw_diamond(&mut ab, px, py, rad + 1.0, rgba(0, 0, 0, 220));
            draw_diamond(
                &mut ab,
                px,
                py,
                rad,
                rgba(pin.rgb[0], pin.rgb[1], pin.rgb[2], 255),
            );
        }
    }

    // 6. Player arrow at centre:
    if let Some(r) = atlas.sprites.get("map/decorations/player").copied() {
        let (s, c) = (look.arrow + 180.0).to_radians().sin_cos();
        let p = |dx: f32, dy: f32| [cx + dx * c - dy * s, cy + dx * s + dy * c];
        let h = 4.0;
        batch.quad(
            [p(-h, h), p(h, h), p(h, -h), p(-h, -h)],
            [[r.u0, r.v0], [r.u1, r.v0], [r.u1, r.v1], [r.u0, r.v1]],
            u32::MAX,
        );
    } else {
        let mut ab = HudAtlasBatch { batch, atlas };
        draw_diamond(&mut ab, cx, cy, 3.0, rgba(255, 255, 255, 255));
    }

    // 7. Compass cardinal indicators (N in crimson accent, E/S/W subtle):
    let reach = size * 0.5 - 5.5;
    let cardinals: [(&str, (f32, f32), u32); 4] = if let Some((nx, ny)) = look.north {
        // Orthogonal right-handed screen basis from North = (nx, ny):
        // East = (-ny, nx), South = (-nx, -ny), West = (ny, -nx).
        [
            ("N", (nx, ny), rgba(248, 113, 113, 255)),
            ("E", (-ny, nx), rgba(228, 228, 231, 195)),
            ("S", (-nx, -ny), rgba(228, 228, 231, 195)),
            ("W", (ny, -nx), rgba(228, 228, 231, 195)),
        ]
    } else {
        [
            ("N", (0.0, -1.0), rgba(248, 113, 113, 255)),
            ("E", (1.0, 0.0), rgba(228, 228, 231, 180)),
            ("S", (0.0, 1.0), rgba(228, 228, 231, 180)),
            ("W", (-1.0, 0.0), rgba(228, 228, 231, 180)),
        ]
    };
    let show_all_cardinals = size >= 84.0;
    for (i, (label, ( vx, vy), col)) in cardinals.into_iter().enumerate() {
        if i > 0 && !show_all_cardinals {
            continue;
        }
        let tw = HudBatch::text_width(atlas, label);
        let tx = (cx + vx * reach - tw * 0.5).round();
        let ty = (cy + vy * reach - 4.0).round();
        batch.text(atlas, label, tx, ty, col, true);
    }

    // 8. Optional compact readout pill lines stacked right next to the minimap:
    if !look.info_lines.is_empty() {
        let text_w = look
            .info_lines
            .iter()
            .map(|l| HudBatch::text_width(atlas, l))
            .fold(0.0f32, f32::max);
        let pill_w = (text_w + 8.0).max(box_size);
        let pill_h = look.info_lines.len() as f32 * 10.0 + 3.0;
        let (px, py) = corners.place(look.corner, pill_w, pill_h);
        batch.fill(atlas, px, py, pill_w, pill_h, rgba(11, 12, 16, 185));
        if look.frame == MinimapFrame::VoltCrimson {
            batch.fill(atlas, px, py, 1.5, pill_h, rgba(220, 38, 38, 220));
        }
        for (i, line) in look.info_lines.iter().enumerate() {
            let lw = HudBatch::text_width(atlas, line);
            let tx = (px + (pill_w - lw) * 0.5).floor();
            let ty = py + 2.0 + i as f32 * 10.0;
            batch.text(atlas, line, tx, ty, rgba(244, 244, 245, 245), true);
        }
    }
}

/// Prepared HUD draw item for one 3D world waypoint.
pub struct WorldWaypointDraw {
    pub projected: ProjectedWaypoint,
    pub rgb: [u8; 3],
    pub title: String,
    pub subtitle: Option<String>,
    pub show_beam: bool,
    pub show_label: bool,
    pub show_edge_arrow: bool,
    pub scale: f32,
}

pub fn draw_world_waypoints(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    waypoints: &[WorldWaypointDraw],
) {
    let uv = white_uv(atlas);
    for wp in waypoints {
        let [r, g, b] = wp.rgb;
        let p = &wp.projected;
        if p.on_screen {
            // 1. Vertical translucent beacon light column:
            if wp.show_beam && p.distance <= 320.0 {
                let fade = (1.0 - (p.distance as f32 / 320.0)).clamp(0.18, 1.0);
                let half_outer = (2.8 * wp.scale * fade).clamp(1.2, 4.5);
                let half_inner = (1.1 * wp.scale * fade).clamp(0.6, 2.0);
                let h = (p.beam_bottom_y - p.beam_top_y).max(0.0);
                if h > 2.0 {
                    batch.fill(
                        atlas,
                        p.screen_x - half_outer,
                        p.beam_top_y,
                        half_outer * 2.0,
                        h,
                        rgba(r, g, b, (52.0 * fade) as u8),
                    );
                    batch.fill(
                        atlas,
                        p.screen_x - half_inner,
                        p.beam_top_y,
                        half_inner * 2.0,
                        h,
                        rgba(255, 255, 255, (88.0 * fade) as u8),
                    );
                }
            }

            // 2. Floating diamond icon + shadowed nameplate:
            let rad = 4.2 * wp.scale;
            {
                let mut ab = HudAtlasBatch { batch, atlas };
                draw_diamond(&mut ab, p.screen_x, p.screen_y, rad + 1.2, rgba(0, 0, 0, 210));
                draw_diamond(&mut ab, p.screen_x, p.screen_y, rad, rgba(r, g, b, 245));
                draw_diamond(
                    &mut ab,
                    p.screen_x,
                    p.screen_y,
                    (rad * 0.42).max(1.2),
                    rgba(255, 255, 255, 220),
                );
            }

            if wp.show_label && !wp.title.is_empty() {
                let tw = HudBatch::text_width(atlas, &wp.title);
                let sw = wp
                    .subtitle
                    .as_deref()
                    .map_or(0.0, |s| HudBatch::text_width(atlas, s));
                let box_w = tw.max(sw) + 8.0;
                let lines = if wp.subtitle.is_some() { 2.0 } else { 1.0 };
                let box_h = lines * 10.0 + 2.0;
                let bx = (p.screen_x - box_w * 0.5).floor();
                let by = (p.screen_y - rad - box_h - 3.0).floor().max(2.0);
                batch.fill(atlas, bx, by, box_w, box_h, rgba(10, 11, 15, 185));
                batch.fill(atlas, bx, by, box_w, 1.0, rgba(r, g, b, 220));
                batch.text(
                    atlas,
                    &wp.title,
                    (p.screen_x - tw * 0.5).floor(),
                    by + 2.0,
                    rgba(255, 255, 255, 250),
                    true,
                );
                if let Some(sub) = &wp.subtitle {
                    batch.text(
                        atlas,
                        sub,
                        (p.screen_x - sw * 0.5).floor(),
                        by + 12.0,
                        rgba(212, 212, 216, 235),
                        true,
                    );
                }
            }
        } else if wp.show_edge_arrow {
            // Sleek rotated directional chevron at the screen edge:
            let (ex, ey) = p.edge_pos;
            let (s, c) = p.edge_angle.sin_cos();
            let rot = |lx: f32, ly: f32| [ex + lx * c - ly * s, ey + lx * s + ly * c];
            let sz = 5.0 * wp.scale;
            batch.quad(
                [
                    rot(sz + 1.2, 0.0),
                    rot(-sz - 0.8, -sz * 0.78 - 0.8),
                    rot(-sz * 0.45, 0.0),
                    rot(-sz - 0.8, sz * 0.78 + 0.8),
                ],
                uv,
                rgba(0, 0, 0, 205),
            );
            batch.quad(
                [
                    rot(sz, 0.0),
                    rot(-sz, -sz * 0.72),
                    rot(-sz * 0.35, 0.0),
                    rot(-sz, sz * 0.72),
                ],
                uv,
                rgba(r, g, b, 245),
            );
        }
    }
}

/// Compact touch-friendly `[+WP]` quick-pin button on the HUD.
pub fn waypoint_hud_button_rect(screen_w: f32, _screen_h: f32) -> [f32; 4] {
    let w = 34.0;
    let h = 16.0;
    [(screen_w * 0.5 + 98.0).min(screen_w - w - 4.0), 4.0, w, h]
}

pub fn draw_waypoint_hud_button(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4]) {
    let [x, y, w, h] = rect;
    batch.fill(atlas, x, y, w, h, rgba(220, 38, 38, 215));
    batch.fill(atlas, x + 1.0, y + 1.0, w - 2.0, h - 2.0, rgba(14, 15, 20, 225));
    let label = "+WP";
    let tw = HudBatch::text_width(atlas, label);
    batch.text(
        atlas,
        label,
        (x + (w - tw) * 0.5).floor(),
        y + 4.0,
        rgba(255, 255, 255, 250),
        true,
    );
}

/// Touch-friendly `[REV]` and `[LOCK]` quick-action pills for Accurate Block Placement.
pub fn placement_hud_button_rects(screen_w: f32, _screen_h: f32) -> ([f32; 4], [f32; 4]) {
    let rev_w = 31.0;
    let lock_w = 42.0;
    let h = 16.0;
    let rev_x = (screen_w * 0.5 - 136.0).max(75.0);
    let lock_x = rev_x + rev_w + 3.0;
    ([rev_x, 4.0, rev_w, h], [lock_x, 4.0, lock_w, h])
}

pub fn draw_placement_hud_buttons(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    screen_w: f32,
    screen_h: f32,
    reversed: bool,
    lock_label: Option<&str>,
) {
    let (rev_rect, lock_rect) = placement_hud_button_rects(screen_w, screen_h);
    let draw_pill = |batch: &mut HudBatch, rect: [f32; 4], text: &str, border: u32, fill: u32| {
        let [x, y, w, h] = rect;
        batch.fill(atlas, x, y, w, h, border);
        batch.fill(atlas, x + 1.0, y + 1.0, w - 2.0, h - 2.0, fill);
        let tw = HudBatch::text_width(atlas, text);
        batch.text(
            atlas,
            text,
            (x + (w - tw) * 0.5).floor(),
            y + 4.0,
            rgba(255, 255, 255, 250),
            true,
        );
    };
    draw_pill(
        batch,
        rev_rect,
        "REV",
        if reversed {
            rgba(245, 158, 11, 235)
        } else {
            rgba(220, 38, 38, 200)
        },
        if reversed {
            rgba(120, 53, 15, 230)
        } else {
            rgba(14, 15, 20, 225)
        },
    );
    let lock_caption = match lock_label {
        Some("North") => "LCK:N",
        Some("South") => "LCK:S",
        Some("East") => "LCK:E",
        Some("West") => "LCK:W",
        Some("Up") => "LCK:U",
        Some("Down") => "LCK:D",
        Some("Top Half") => "LCK:T",
        Some("Bottom Half") => "LCK:B",
        Some(_) => "LOCKED",
        None => "LOCK",
    };
    draw_pill(
        batch,
        lock_rect,
        lock_caption,
        if lock_label.is_some() {
            rgba(251, 191, 36, 240)
        } else {
            rgba(220, 38, 38, 200)
        },
        if lock_label.is_some() {
            rgba(120, 53, 15, 230)
        } else {
            rgba(14, 15, 20, 225)
        },
    );
}

/// Crosshair quadrant reticle & orientation pill for Accurate Block Placement.
pub fn draw_placement_hud_badge(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    screen_w: f32,
    screen_h: f32,
    badge: &str,
    quadrant_ticks: bool,
    locked: bool,
) {
    let cx = (screen_w * 0.5).floor();
    let cy = (screen_h * 0.5).floor();
    if quadrant_ticks {
        let tick_col = if locked {
            rgba(251, 191, 36, 175)
        } else {
            rgba(239, 68, 68, 165)
        };
        let d = 7.0;
        batch.fill(atlas, cx - d, cy - d, 2.0, 2.0, tick_col);
        batch.fill(atlas, cx + d - 1.0, cy - d, 2.0, 2.0, tick_col);
        batch.fill(atlas, cx - d, cy + d - 1.0, 2.0, 2.0, tick_col);
        batch.fill(atlas, cx + d - 1.0, cy + d - 1.0, 2.0, 2.0, tick_col);
    }
    let tw = HudBatch::text_width(atlas, badge);
    let pw = tw + 8.0;
    let ph = 11.0;
    let px = (cx - pw * 0.5).floor();
    let py = cy + 12.0;
    let accent = if locked {
        rgba(245, 158, 11, 230)
    } else {
        rgba(220, 38, 38, 225)
    };
    batch.fill(atlas, px, py, pw, ph, rgba(10, 11, 15, 185));
    batch.fill(atlas, px, py, 1.5, ph, accent);
    batch.text(atlas, badge, px + 4.0, py + 2.0, rgba(250, 250, 250, 245), true);
}

/// A box of text lines (a mod's HUD panel): `(text, 0xRRGGBB)` each.
pub fn draw_lines(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    corners: &mut Corners,
    corner: Corner,
    lines: &[(&str, u32)],
) {
    draw_lines_iter(batch, atlas, corners, corner, lines.iter().copied());
}

/// Draw retained mod lines without building a temporary vector each frame.
pub fn draw_lines_iter<'a>(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    corners: &mut Corners,
    corner: Corner,
    lines: impl ExactSizeIterator<Item = (&'a str, u32)> + Clone,
) {
    if lines.len() == 0 {
        return;
    }
    let w = lines
        .clone()
        .map(|(text, _)| HudBatch::text_width(atlas, text))
        .fold(0.0, f32::max)
        + 4.0;
    let h = 10.0 * lines.len() as f32 + 2.0;
    let (x, y) = corners.place(corner, w, h);
    batch.fill(atlas, x, y, w, h, BACKGROUND);
    for (i, (text, color)) in lines.enumerate() {
        let [_, r, g, b] = color.to_be_bytes();
        let y = y + 2.0 + 10.0 * i as f32;
        batch.text(atlas, text, x + 2.0, y, rgba(r, g, b, 255), true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_stack_away_from_their_edge() {
        let mut c = Corners::new(400.0, 300.0, 26.0);
        assert_eq!(c.place(Corner::TopLeft, 50.0, 12.0), (2.0, 2.0));
        assert_eq!(c.place(Corner::TopLeft, 70.0, 20.0), (2.0, 16.0));
        // Top right starts under the effect icons.
        assert_eq!(c.place(Corner::TopRight, 50.0, 12.0), (348.0, 28.0));
        assert_eq!(c.place(Corner::BottomRight, 50.0, 12.0), (348.0, 286.0));
        assert_eq!(c.place(Corner::BottomRight, 50.0, 10.0), (348.0, 274.0));
        assert_eq!(c.place(Corner::BottomLeft, 10.0, 10.0), (2.0, 288.0));
    }
}
