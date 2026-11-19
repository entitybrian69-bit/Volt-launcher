/// The cover picture saved beside a replay (`<file>.png`).
pub(crate) fn replay_cover_path(replay: &std::path::Path) -> std::path::PathBuf {
    let mut name = replay.as_os_str().to_owned();
    name.push(".png");
    std::path::PathBuf::from(name)
}

pub(super) fn captures_grid(
    ui: &mut Ui,
    area: [f32; 4],
    shots: &[String],
    clips: &[(String, u64)],
    folder: &std::path::Path,
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
        "Captures",
    );
    styled_button(
        ui,
        targets,
        [x + w - 118.0, y + 4.0, 110.0, 30.0],
        "Open folder",
        Action::OpenCaptures,
        BtnStyle::CrimsonOutline,
        true,
    );
    let summary = if clips.is_empty() && shots.is_empty() {
        "No captures yet. Screenshots and Volt Recorder clips land here.".to_owned()
    } else {
        format!(
            "{} recording(s) \u{2022} {} screenshot(s) \u{2022} Tap a picture to see it full screen",
            clips.len(),
            shots.len()
        )
    };
    let summary = ui.fit_text(Face::LauncherRegular, 12.5, &summary, w - 16.0);
    ui.text(
        Face::LauncherRegular,
        12.5,
        x + 8.0,
        y + 34.0,
        theme::MUTED(),
        &summary,
    );
    let top = y + 58.0;
    let bottom = y + h;
    let clip_h = 40.0;
    // Recorder clips keep their rows; the screenshots below are a
    // thumbnail grid of the actual pictures.
    let cols = ((w / 190.0).floor() as usize).max(1);
    let gap = 10.0;
    let card_w = (w - 16.0 - (cols.saturating_sub(1) as f32) * gap) / cols as f32;
    let thumb_h = (card_w * 9.0 / 16.0).min(130.0);
    let rows = shots.len().div_ceil(cols);
    let grid_h = (rows as f32 * (thumb_h + gap)).max(0.0);
    let total = clips.len() as f32 * clip_h + grid_h;
    let maximum = (total - (bottom - top).max(0.0)).max(0.0);
    *scroll = (*scroll).clamp(0.0, maximum);
    let start = ui.batch_mut().vertices.len();
    for (i, (name, size)) in clips.iter().enumerate() {
        let row_y = top + i as f32 * clip_h - *scroll;
        if row_y + clip_h < top || row_y > bottom {
            continue;
        }
        let row = [x + 8.0, row_y, w - 16.0, 34.0];
        ui.rounded(row[0], row[1], row[2], row[3], 10.0, theme::GLASS_INNER());
        let label = ui.fit_text(Face::LauncherMedium, 12.5, name, row[2] - 150.0);
        ui.text(
            Face::LauncherMedium,
            12.5,
            row[0] + 10.0,
            row[1] + 3.0,
            theme::TEXT(),
            &label,
        );
        let mib = *size as f32 / (1024.0 * 1024.0);
        let sub = format!("Volt Recorder \u{2022} {mib:.1} MiB");
        ui.text(
            Face::LauncherRegular,
            11.0,
            row[0] + 10.0,
            row[1] + 18.0,
            theme::MUTED(),
            &sub,
        );
        let del = [row[0] + row[2] - 50.0, row[1] + 3.0, 42.0, 28.0];
        styled_button(
            ui,
            targets,
            del,
            "Del",
            Action::DeleteRecording(i),
            BtnStyle::Secondary,
            true,
        );
    }
    let shots_top = top + clips.len() as f32 * clip_h;
    for (i, shot) in shots.iter().enumerate() {
        let cx = x + 8.0 + (i % cols) as f32 * (card_w + gap);
        let cy = shots_top + (i / cols) as f32 * (thumb_h + gap) - *scroll;
        if cy + thumb_h < top || cy > bottom {
            continue;
        }
        let path = folder.join(shot);
        let rect = [cx, cy, card_w, thumb_h];
        draw_photo_thumb(
            ui,
            photos,
            4 + (i % 24),
            &crate::photo_cache::file_key(&path),
            &crate::photo_cache::Source::File(path),
            rect,
            2,
        );
        if cy >= top - 4.0 && cy + thumb_h <= bottom + 4.0 {
            targets.push((rect, Action::ViewScreenshot(i)));
        }
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
pub(super) fn replay_rows(
    ui: &mut Ui,
    area: [f32; 4],
    replays: &[horizon_client::replay::Meta],
    replay_dir: &std::path::Path,
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
        "Volt Replays",
    );
    let summary = if replays.is_empty() {
        "No replays yet. Turn on Replay Recording in Volt Settings > Volt QoL, play, and your sessions appear here.".to_owned()
    } else {
        format!(
            "{} recorded sessions \u{2022} Tap one to watch it",
            replays.len()
        )
    };
    let summary = ui.fit_text(Face::LauncherRegular, 12.5, &summary, w - 16.0);
    ui.text(
        Face::LauncherRegular,
        12.5,
        x + 8.0,
        y + 34.0,
        theme::MUTED(),
        &summary,
    );
    let top = y + 58.0;
    let bottom = y + h;
    let max = (replays.len() as f32 * 54.0 - (bottom - top).max(0.0)).max(0.0);
    *scroll = (*scroll).clamp(0.0, max);
    let start = ui.batch_mut().vertices.len();
    for (i, replay) in replays.iter().enumerate() {
        let row_y = top + i as f32 * 54.0 - *scroll;
        let row = [x + 8.0, row_y, w - 16.0, 48.0];
        if row_y < top || row_y + 48.0 > bottom {
            continue;
        }
        ui.rounded(row[0], row[1], row[2], row[3], 12.0, theme::GLASS_INNER());
        // The session's cover picture, when one was saved beside it.
        let cover = replay_cover_path(&replay_dir.join(&replay.file_name));
        let tile = [row[0] + 5.0, row_y + 5.0, 68.0, 38.0];
        if cover.is_file() {
            draw_photo_thumb(
                ui,
                photos,
                4 + (i % 24),
                &crate::photo_cache::file_key(&cover),
                &crate::photo_cache::Source::File(cover),
                tile,
                5,
            );
            targets.push((tile, Action::ViewReplayCover(i)));
        } else {
            ui.rounded(tile[0], tile[1], tile[2], tile[3], 8.0, theme::HERO_BG());
            draw_tile_icon(
                ui,
                5,
                tile[0] + tile[2] * 0.5,
                tile[1] + tile[3] * 0.5,
                horizon_client::ui_theme::color::hex(0xc9cfdb, 90),
            );
        }
        let title = if replay.server.is_empty() {
            &replay.file_name
        } else {
            &replay.server
        };
        let title = ui.fit_text(Face::LauncherMedium, 13.0, title, row[2] - 280.0);
        ui.text(
            Face::LauncherMedium,
            13.0,
            row[0] + 82.0,
            row[1] + 4.0,
            theme::TEXT(),
            &title,
        );
        let mib = replay.size as f32 / (1024.0 * 1024.0);
        let sub = format!(
            "{} \u{2022} {} \u{2022} {:.1} MiB",
            horizon_client::replay::format_time(replay.duration_ms),
            replay.name,
            mib,
        );
        let sub = ui.fit_text(Face::LauncherRegular, 11.5, &sub, row[2] - 200.0);
        ui.text(
            Face::LauncherRegular,
            11.5,
            row[0] + 12.0,
            row[1] + 25.0,
            theme::MUTED(),
            &sub,
        );
        let watch = [row[0] + row[2] - 184.0, row[1] + 8.0, 70.0, 32.0];
        styled_button(
            ui,
            targets,
            watch,
            "Watch",
            Action::WatchReplay(i),
            BtnStyle::Primary,
            true,
        );
        let export = [row[0] + row[2] - 106.0, row[1] + 8.0, 48.0, 32.0];
        styled_button(
            ui,
            targets,
            export,
            "MP4",
            Action::ExportReplay(i),
            BtnStyle::Secondary,
            true,
        );
        let del = [row[0] + row[2] - 50.0, row[1] + 8.0, 42.0, 32.0];
        styled_button(
            ui,
            targets,
            del,
            "Del",
            Action::DeleteReplay(i),
            BtnStyle::Secondary,
            true,
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
    [top, bottom, max]
}
