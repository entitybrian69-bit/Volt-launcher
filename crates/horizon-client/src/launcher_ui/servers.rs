pub(super) fn server_rows(
    ui: &mut Ui,
    area: [f32; 4],
    address: &str,
    servers: &[crate::server_list::Server],
    photos: &mut crate::photo_cache::PhotoCache,
    scroll: &mut f32,
    targets: &mut Vec<([f32; 4], Action)>,
    mut list_targets: Option<&mut Vec<([f32; 4], Action)>>,
) -> [f32; 3] {
    let [x, y, w, h] = area;
    ui.text(
        Face::LauncherMedium,
        15.0,
        x + 8.0,
        y + 8.0,
        theme::TEXT(),
        "Multiplayer",
    );
    ui.text(
        Face::LauncherRegular,
        12.5,
        x + 8.0,
        y + 34.0,
        theme::MUTED(),
        "Select a saved server or enter an address",
    );
    let r = [x + 8.0, y + 56.0, (w - 104.0).max(80.0).min(380.0), 38.0];
    ui.rounded(r[0], r[1], r[2], r[3], 10.0, theme::CARD_INNER());
    let value = if address.is_empty() {
        "Server address (optional)"
    } else {
        address
    };
    let value = ui.fit_text(Face::LauncherRegular, 13.0, value, r[2] - 24.0);
    ui.text(
        Face::LauncherRegular,
        13.0,
        r[0] + 12.0,
        text_vcenter(r[1], r[3], 13.0),
        theme::TEXT(),
        &value,
    );
    targets.push((r, Action::Field("server")));
    if w >= 200.0 {
        styled_button(
            ui,
            targets,
            [r[0] + r[2] + 8.0, r[1], 80.0, r[3]],
            "Save",
            Action::SaveServer,
            BtnStyle::Secondary,
            true,
        );
    }

    let top = y + 104.0;
    let bottom = y + h;
    let max = (servers.len() as f32 * 54.0 - (bottom - top).max(0.0)).max(0.0);
    *scroll = (*scroll).clamp(0.0, max);

    if servers.is_empty() && bottom > top + 20.0 {
        ui.text(
            Face::LauncherRegular,
            12.5,
            x + 8.0,
            top + 5.0,
            theme::MUTED(),
            "No saved servers in this instance.",
        );
    }
    let start = ui.batch_mut().vertices.len();
    for (i, server) in servers.iter().enumerate() {
        let row_y = top + i as f32 * 54.0 - *scroll;
        let row = [x + 8.0, row_y, w - 16.0, 48.0];
        if let Some(list) = list_targets.as_deref_mut() {
            list.push((row, Action::ConnectServer(i)));
        }
        if row_y < top || row_y + 48.0 > bottom {
            continue;
        }
        ui.rounded(row[0], row[1], row[2], row[3], 12.0, theme::GLASS_INNER());
        // The server's favicon from servers.dat, decoded off-thread.
        let icon = [row[0] + 6.0, row[1] + 6.0, 36.0, 36.0];
        ui.rounded(icon[0], icon[1], icon[2], icon[3], 8.0, theme::HERO_BG());
        let mut has_icon = false;
        if let Some(bytes) = &server.icon {
            let key = format!("srv:{:x}", crate::skin_preview::fnv1a(bytes));
            if let Some(photo) = photos.get(
                &key,
                &crate::photo_cache::Source::Bytes(bytes.clone()),
                crate::photo_cache::Fit::Thumb,
            ) {
                let (cw, ch) = crate::photo_cache::Fit::Thumb.canvas();
                // The favicon is square: draw the letterboxed
                // 16:9 canvas wider than the tile, centred, so
                // the picture itself lands square in the tile.
                let dw = icon[3] * cw as f32 / ch as f32;
                has_icon = ui.draw_dynamic_rgba(
                    4 + (i % 24),
                    photo.hash,
                    cw,
                    ch,
                    &photo.rgba,
                    icon[0] + (icon[2] - dw) * 0.5,
                    icon[1],
                    dw,
                    icon[3],
                    255,
                );
            }
        }
        if !has_icon {
            draw_tile_icon(
                ui,
                1,
                icon[0] + 18.0,
                icon[1] + 18.0,
                horizon_client::ui_theme::color::hex(0xc9cfdb, 90),
            );
        }
        let tx0 = row[0] + 52.0;
        let title = ui.fit_text(Face::LauncherMedium, 13.0, &server.name, row[2] - 64.0);
        let address = ui.fit_text(Face::LauncherRegular, 11.5, &server.ip, row[2] - 64.0);
        ui.text(
            Face::LauncherMedium,
            13.0,
            tx0,
            row[1] + 4.0,
            theme::TEXT(),
            &title,
        );
        ui.text(
            Face::LauncherRegular,
            11.5,
            tx0,
            row[1] + 25.0,
            theme::MUTED(),
            &address,
        );
        targets.push((row, Action::ConnectServer(i)));
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
