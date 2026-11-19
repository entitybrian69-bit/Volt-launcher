/// What the full-screen photo viewer shows, over every launcher page.
#[derive(Clone)]
pub(crate) struct PhotoView {
    pub title: String,
    /// The photo cache key (stable per picture version).
    pub key: String,
    pub source: crate::photo_cache::Source,
}

/// The world folders under `saves`, sorted, as the Worlds grid shows them.
pub(super) fn world_folders_in(saves: &std::path::Path) -> Vec<String> {
    let mut worlds: Vec<String> = std::fs::read_dir(saves)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    worlds.sort_unstable();
    worlds
}

/// One gallery thumbnail: the letterboxed picture when decoded, a quiet
/// placeholder tile while loading or when there is no picture.
fn draw_photo_thumb(
    ui: &mut Ui,
    photos: &mut crate::photo_cache::PhotoCache,
    slot: usize,
    key: &str,
    source: &crate::photo_cache::Source,
    rect: [f32; 4],
    glyph: u8,
) {
    use crate::photo_cache::Fit;
    let [x, y, w, h] = rect;
    ui.rounded(x, y, w, h, 8.0, theme::HERO_BG());
    let mut drawn = false;
    if let Some(photo) = photos.get(key, source, Fit::Thumb) {
        let fade = (photo.loaded_at.elapsed().as_secs_f32() / 0.22).clamp(0.0, 1.0);
        let alpha = (ease_out_cubic(fade) * 255.0).round() as u8;
        let (cw, ch) = Fit::Thumb.canvas();
        drawn = ui.draw_dynamic_rgba(slot, photo.hash, cw, ch, &photo.rgba, x, y, w, h, alpha);
    }
    if !drawn {
        // Placeholder: the tab's glyph, dim, centred.
        draw_tile_icon(
            ui,
            glyph,
            x + w * 0.5,
            y + h * 0.5,
            horizon_client::ui_theme::color::hex(0xc9cfdb, 70),
        );
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}

pub(super) fn draw_photo_viewer(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    photos: &mut crate::photo_cache::PhotoCache,
    view: &PhotoView,
    w: f32,
    h: f32,
) {
    use crate::photo_cache::Fit;
    use horizon_client::ui_theme::color::hex;
    ui.rounded(0.0, 0.0, w, h, 0.0, hex(0x05070b, 240));
    let (cw, ch) = Fit::Full.canvas();
    let avail_w = (w - 24.0).max(64.0);
    let avail_h = (h - 86.0).max(64.0);
    let scale = (avail_w / cw as f32).min(avail_h / ch as f32);
    let (dw, dh) = (cw as f32 * scale, ch as f32 * scale);
    let (dx, dy) = ((w - dw) * 0.5, 56.0 + (avail_h - dh) * 0.5);
    let mut drawn = false;
    if let Some(photo) = photos.get(&view.key, &view.source, Fit::Full) {
        let fade = (photo.loaded_at.elapsed().as_secs_f32() / 0.25).clamp(0.0, 1.0);
        let alpha = (ease_out_cubic(fade) * 255.0).round() as u8;
        drawn = ui.draw_dynamic_rgba(28, photo.hash, cw, ch, &photo.rgba, dx, dy, dw, dh, alpha);
    }
    if !drawn {
        let text = if photos.failed(&view.key, Fit::Full) {
            "This picture can't be read."
        } else {
            "Loading picture..."
        };
        draw_centered_text(
            ui,
            Face::LauncherRegular,
            14.0,
            [0.0, dy, w, dh],
            theme::MUTED(),
            text,
        );
    }
    let back = [12.0, 10.0, 92.0, 36.0];
    ui.rounded(back[0], back[1], back[2], back[3], 18.0, theme::GLASS());
    ui.border(
        back[0],
        back[1],
        back[2],
        back[3],
        18.0,
        theme::GLASS_BORDER(),
    );
    draw_centered_text(
        ui,
        Face::LauncherMedium,
        13.0,
        back,
        theme::TEXT(),
        "< Back",
    );
    targets.push((back, Action::ClosePhoto));
    let title = ui.fit_text(
        Face::LauncherMedium,
        14.0,
        &view.title,
        (w - 130.0).max(60.0),
    );
    ui.text(
        Face::LauncherMedium,
        14.0,
        back[0] + back[2] + 12.0,
        19.0,
        theme::TEXT(),
        &title,
    );
    // Any tap outside the picture closes the viewer too.
    targets.push(([0.0, 0.0, w, h], Action::ClosePhoto));
}

pub(super) fn world_grid(
    ui: &mut Ui,
    area: [f32; 4],
    saves: &std::path::Path,
    worlds: &[String],
    photos: &mut crate::photo_cache::PhotoCache,
    targets: &mut Vec<([f32; 4], Action)>,
    scroll: &mut f32,
) -> [f32; 3] {
    let [x, y, w, h] = area;
    ui.text(
        Face::LauncherMedium,
        15.0,
        x + 8.0,
        y + 8.0,
        theme::TEXT(),
        "Worlds",
    );
    let summary = if worlds.is_empty() {
        "No local world folders found".to_owned()
    } else {
        format!(
            "{} local worlds \u{2022} Tap one to see its picture",
            worlds.len()
        )
    };
    let summary = ui.fit_text(Face::LauncherRegular, 12.5, &summary, w - 16.0);
    ui.text(
        Face::LauncherRegular,
        12.5,
        x + 8.0,
        y + 35.0,
        theme::MUTED(),
        &summary,
    );
    let top = y + 60.0;
    let bottom = y + h;
    let view_h = (bottom - top).max(0.0);
    // A thumbnail grid: each world card carries its icon.png.
    let cols = ((w / 190.0).floor() as usize).max(1);
    let gap = 10.0;
    let card_w = (w - 16.0 - (cols.saturating_sub(1) as f32) * gap) / cols as f32;
    let thumb_h = (card_w * 9.0 / 16.0).min(130.0);
    let card_h = thumb_h + 26.0;
    let rows = worlds.len().div_ceil(cols);
    let maximum = (rows as f32 * (card_h + gap) - gap - view_h).max(0.0);
    *scroll = scroll.clamp(0.0, maximum);

    let start = ui.batch_mut().vertices.len();
    for (i, world) in worlds.iter().enumerate() {
        let cx = x + 8.0 + (i % cols) as f32 * (card_w + gap);
        let cy = top + (i / cols) as f32 * (card_h + gap) - *scroll;
        if cy + card_h < top || cy > bottom {
            continue;
        }
        let card = [cx, cy, card_w, card_h];
        ui.rounded(
            card[0],
            card[1],
            card[2],
            card[3],
            10.0,
            theme::GLASS_INNER(),
        );
        let icon = saves.join(world).join("icon.png");
        draw_photo_thumb(
            ui,
            photos,
            4 + (i % 24),
            &crate::photo_cache::file_key(&icon),
            &crate::photo_cache::Source::File(icon),
            [cx + 3.0, cy + 3.0, card_w - 6.0, thumb_h - 3.0],
            0,
        );
        let label = ui.fit_text(Face::LauncherMedium, 12.0, world, card_w - 16.0);
        ui.text(
            Face::LauncherMedium,
            12.0,
            cx + 8.0,
            cy + thumb_h + 4.0,
            theme::TEXT(),
            &label,
        );
        if cy >= top - 4.0 && cy + card_h <= bottom + 4.0 {
            targets.push((card, Action::ViewWorld(i)));
        }
    }
    if worlds.is_empty() && view_h > 24.0 {
        ui.text(
            Face::LauncherRegular,
            12.5,
            x + 12.0,
            top + 6.0,
            theme::MUTED(),
            "Create a world in the game and it appears here.",
        );
    }
    let bounds = [
        ui.to_batch(x),
        ui.to_batch(top),
        ui.to_batch(x + w),
        ui.to_batch(bottom),
    ];
    ui.batch_mut()
        .clip_since(start, bounds[0], bounds[1], bounds[2], bounds[3]);

    [top, bottom, maximum]
}
