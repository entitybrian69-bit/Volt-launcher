/// Chunky "Minecraft slab" panel for the lobby home, per the chosen
/// reference: a notched pixel silhouette with a dark outline, a soft drop
/// shadow, a two-tone fill and a bright bevel along the top edge.
fn pixel_panel(ui: &mut Ui, rect: [f32; 4], top: u32, bottom: u32) {
    use horizon_client::ui_theme::color::{hex, lerp};
    let [x, y, w, h] = rect;
    let n = 5.0_f32.min(w * 0.12).min(h * 0.2); // corner notch size
    let b = 2.5; // outline thickness
    // Soft drop shadow (single pass so overlaps never double up).
    ui.rounded(x + 2.0, y + 4.0, w - 2.0, h, 3.0, hex(0x04060a, 95));
    // Notched outline silhouette: two overlapping rectangles.
    let outline = hex(0x10131a, 240);
    ui.rounded(x + n, y, w - 2.0 * n, h, 0.0, outline);
    ui.rounded(x, y + n, w, h - 2.0 * n, 0.0, outline);
    // Fill, inset by the outline, same notched union.
    let (fx, fy, fw, fh) = (x + b, y + b, w - 2.0 * b, h - 2.0 * b);
    ui.rounded(fx + n, fy, fw - 2.0 * n, fh, 0.0, bottom);
    ui.rounded(fx, fy + n, fw, fh - 2.0 * n, 0.0, bottom);
    // Brighter top half for the two-tone slab look.
    let split = fh * 0.46;
    ui.rounded(fx + n, fy, fw - 2.0 * n, split, 0.0, top);
    if split > n {
        ui.rounded(fx, fy + n, fw, split - n, 0.0, top);
    }
    // Bevel highlight along the top edge.
    ui.rounded(
        fx + n,
        fy,
        fw - 2.0 * n,
        2.5,
        0.0,
        lerp(top, hex(0xffffff, 255), 0.32),
    );
}

/// A small filled "play" triangle with its left edge at `x`, centred on `cy`.
fn draw_play_triangle(ui: &mut Ui, x: f32, cy: f32, size: f32, color: u32) {
    let white = ui.atlas_ref().white;
    let uv = [(white.u0 + white.u1) * 0.5, (white.v0 + white.v1) * 0.5];
    let points = [
        [ui.to_batch(x), ui.to_batch(cy - size * 0.55)],
        [ui.to_batch(x), ui.to_batch(cy + size * 0.55)],
        [ui.to_batch(x + size), ui.to_batch(cy)],
    ];
    for pos in points {
        ui.batch_mut()
            .vertices
            .push(horizon_client::hud::HudVertex {
                pos,
                uv,
                color,
                depth: 0.0,
                glint_uv: [0.0, 0.0],
            });
    }
}

/// Original classic lobby. Each host supplies its selected player's avatar and receives
/// the drag rectangle; buttons, metrics and install progress use the same layout.
fn classic_home(
    ui: &mut Ui,
    area: [f32; 4],
    account_status: &str,
    skin_sub: String,
    play_seconds: u64,
    installed: bool,
    progress: Option<horizon_client::official_install::InstallProgress>,
    error: Option<&str>,
    targets: &mut Vec<([f32; 4], Action)>,
    draw_avatar: impl FnOnce(&mut Ui, [f32; 4]),
) -> [f32; 4] {
    use horizon_client::official_install::InstallStage;
    use horizon_client::ui_theme::color::hex;
    let [ax, ay, w, h] = area;
    let compact = w < 760.0;
    let m = if compact { 10.0 } else { 16.0 };

    // ---- Brand, flush to the top-left corner ----
    ui.volt_logo(ax + 2.0, ay + m, 32.0, 32.0);
    ui.text(
        Face::Display,
        19.0,
        ax + 42.0,
        ay + m + 1.0,
        theme::TEXT(),
        "VOLT",
    );
    ui.text(
        Face::LauncherRegular,
        10.5,
        ax + 42.0,
        ay + m + 22.0,
        theme::MUTED(),
        "Minecraft 26.3 · Native Rust",
    );

    // ---- Quiet icon squares, top-right ----
    let sq = if compact { 38.0 } else { 42.0 };
    let squares: [(Page, Action); 5] = [
        (Page::Controls, Action::Page(Page::Controls)),
        (Page::Installed, Action::Page(Page::Installed)),
        (Page::VoltSettings, Action::Page(Page::VoltSettings)),
        (Page::Account, Action::Page(Page::Account)),
        (Page::Settings, Action::Page(Page::Settings)),
    ];
    for (i, (page, action)) in squares.iter().enumerate() {
        // Flush to the screen's right edge, where the rail below joins.
        let rect = [
            ax + w - (squares.len() - i) as f32 * (sq + 8.0) + 8.0,
            ay + m,
            sq,
            sq,
        ];
        ui.rounded(rect[0], rect[1], rect[2], rect[3], 10.0, theme::GLASS());
        ui.border(
            rect[0],
            rect[1],
            rect[2],
            rect[3],
            10.0,
            theme::GLASS_BORDER(),
        );
        draw_page_icon(
            ui,
            *page,
            rect[0] + sq * 0.5,
            rect[1] + sq * 0.5,
            theme::TEXT(),
        );
        targets.push((rect, *action));
    }

    // ---- Bottom bar metrics (the character is sized around them) ----
    let bar_h = if compact { 64.0 } else { 76.0 };
    let bar_y = ay + h - m - bar_h;

    // ---- The star of the show: the rotatable 3D character ----
    let char_w = (w * 0.34).clamp(170.0, 360.0);
    let char_top = ay + m + sq + 4.0;
    let char_h = (bar_y - 10.0 - char_top).max(60.0);
    let char_rect = [ax + (w - char_w) * 0.5, char_top, char_w, char_h];
    draw_avatar(ui, char_rect);

    // ---- Pixel-art destination slabs, left column ----
    let card_w = (w * 0.24).clamp(164.0, 236.0);
    let gap = 10.0;
    let cards_top = ay + m + 50.0;
    let region = (bar_y - 10.0 - cards_top).max(120.0);
    let card_h = ((region - 2.0 * gap) / 3.0).clamp(44.0, 84.0);
    let cards: [(&str, String, Page, u32, u32, Action); 3] = [
        (
            "MODS",
            "Built into Volt".to_owned(),
            Page::Mods,
            hex(0xd8455f, 255),
            hex(0xa32a44, 255),
            Action::Page(Page::Mods),
        ),
        (
            "MY SKIN",
            skin_sub,
            Page::Account,
            hex(0x3f7bea, 255),
            hex(0x2a54c0, 255),
            Action::Page(Page::Account),
        ),
        (
            "SHADERS",
            "Volt Visuals".to_owned(),
            Page::Shaders,
            hex(0x58b956, 255),
            hex(0x398f3f, 255),
            Action::Page(Page::Shaders),
        ),
    ];
    for (i, (title, sub, page, top, bottom, action)) in cards.iter().enumerate() {
        // Flush to the screen's left edge.
        let rect = [ax, cards_top + i as f32 * (card_h + gap), card_w, card_h];
        if rect[1] + rect[3] > bar_y - 6.0 {
            break;
        }
        let v_start = ui.batch_mut().vertices.len();
        pixel_panel(ui, rect, *top, *bottom);
        let cy = rect[1] + card_h * 0.5;
        draw_page_icon(ui, *page, rect[0] + 26.0, cy, hex(0xffffff, 235));
        if card_h >= 56.0 {
            ui.text(
                Face::Display,
                15.0,
                rect[0] + 46.0,
                cy - 17.0,
                hex(0xffffff, 255),
                title,
            );
            let sub = ui.fit_text(Face::LauncherRegular, 10.5, sub, card_w - 60.0);
            ui.text(
                Face::LauncherRegular,
                10.5,
                rect[0] + 46.0,
                cy + 2.0,
                hex(0xffffff, 190),
                &sub,
            );
        } else {
            ui.text(
                Face::Display,
                14.5,
                rect[0] + 46.0,
                text_vcenter(rect[1], card_h, 14.5),
                hex(0xffffff, 255),
                title,
            );
        }
        press_scale(ui, v_start, rect, *action);
        targets.push((rect, *action));
    }

    // ---- Right rail: the quieter destinations as small chips, on a
    // glass rail that runs flush to the screen edge and joins the
    // top icon row with no gap, so the two bars read as one piece.
    let chips: [(&str, u8, Action); 8] = [
        ("Worlds", 0, Action::HomeTab(HomeTab::Worlds)),
        ("Controls", 6, Action::Page(Page::Controls)),
        ("Servers", 1, Action::HomeTab(HomeTab::Multiplayer)),
        ("Replays", 2, Action::HomeTab(HomeTab::Replays)),
        ("Shots", 2, Action::HomeTab(HomeTab::Screenshots)),
        ("Logs", 3, Action::HomeTab(HomeTab::Console)),
        ("Packs", 5, Action::Page(Page::Packs)),
        ("Boost", 4, Action::OpenVoltBoost),
    ];
    let chip_w = 96.0;
    let chip_h = 27.0;
    let rail_w = chip_w + 14.0;
    let rail_x = ax + w - rail_w;
    let chip_x = rail_x + 7.0;
    let mut chip_y = ay + m + sq + 8.0;
    let shown = chips
        .iter()
        .enumerate()
        .take_while(|(i, _)| chip_y + (*i as f32) * (chip_h + 6.0) + chip_h <= bar_y - 6.0)
        .count();
    if shown > 0 {
        // The rail starts at the top icon row's baseline, so the
        // vertical bar connects straight into the horizontal one.
        let rail_top = ay + m + sq;
        let rail_h = (shown as f32) * (chip_h + 6.0) + 10.0;
        ui.rounded(rail_x, rail_top, rail_w, rail_h, 14.0, theme::GLASS());
        ui.border(
            rail_x,
            rail_top,
            rail_w,
            rail_h,
            14.0,
            theme::GLASS_BORDER(),
        );
    }
    for (label, glyph, action) in chips.into_iter().take(shown) {
        let rect = [chip_x, chip_y, chip_w, chip_h];
        ui.rounded(
            rect[0],
            rect[1],
            rect[2],
            rect[3],
            chip_h * 0.5,
            theme::GLASS(),
        );
        ui.border(
            rect[0],
            rect[1],
            rect[2],
            rect[3],
            chip_h * 0.5,
            theme::GLASS_BORDER(),
        );
        if glyph == 5 {
            draw_page_icon(
                ui,
                Page::Packs,
                rect[0] + 16.0,
                rect[1] + chip_h * 0.5,
                theme::MUTED(),
            );
        } else if glyph == 6 {
            draw_page_icon(
                ui,
                Page::Controls,
                rect[0] + 16.0,
                rect[1] + chip_h * 0.5,
                theme::MUTED(),
            );
        } else {
            draw_tile_icon(
                ui,
                glyph,
                rect[0] + 16.0,
                rect[1] + chip_h * 0.5,
                theme::MUTED(),
            );
        }
        ui.text(
            Face::LauncherMedium,
            11.5,
            rect[0] + 29.0,
            text_vcenter(rect[1], chip_h, 11.5),
            theme::TEXT(),
            label,
        );
        targets.push((rect, action));
        chip_y += chip_h + 6.0;
    }

    // ---- Bottom centre: ONE big red Play slab, perfectly centred
    // (the profile selector and Game Files slabs are gone by request;
    // profiles stay reachable from Instances and the files page from
    // the top-right squares).
    let play_w = if compact { 210.0 } else { 260.0 };
    let px0 = ax + (w - play_w) * 0.5;
    if let Some(progress) = progress {
        let slab = [px0, bar_y, play_w, bar_h];
        pixel_panel(ui, slab, hex(0xd8455f, 255), hex(0xa32a44, 255));
        let index = InstallStage::ALL
            .iter()
            .position(|s| *s == progress.stage)
            .unwrap_or(0);
        let part = if progress.total > 0 {
            progress.done as f32 / progress.total as f32
        } else {
            0.0
        };
        let fraction = ((index as f32 + part.clamp(0.0, 1.0)) / InstallStage::ALL.len() as f32)
            .clamp(0.0, 1.0);
        draw_centered_text(
            ui,
            Face::LauncherMedium,
            10.0,
            [slab[0], bar_y + 8.0, play_w, 12.0],
            hex(0x3d0812, 220),
            "INSTALLING",
        );
        draw_centered_text(
            ui,
            Face::Display,
            19.0,
            [slab[0], bar_y + 20.0, play_w, 24.0],
            hex(0x3d0812, 255),
            &format!("{:.0}%", fraction * 100.0),
        );
        let bar = [slab[0] + 16.0, bar_y + bar_h - 16.0, play_w - 32.0, 5.0];
        ui.rounded(bar[0], bar[1], bar[2], bar[3], 2.5, hex(0x3d0812, 130));
        ui.rounded(
            bar[0],
            bar[1],
            (bar[2] * fraction).max(5.0),
            bar[3],
            2.5,
            hex(0xffe3e9, 255),
        );
    } else {
        let slab = [px0, bar_y, play_w, bar_h];
        let v_start = ui.batch_mut().vertices.len();
        pixel_panel(ui, slab, hex(0xd8455f, 255), hex(0xa32a44, 255));
        let label = if installed {
            "Play"
        } else if error.is_some() {
            "Retry"
        } else {
            "Download"
        };
        let ink = hex(0xffffff, 255);
        let font = 20.0;
        let text_w = ui.measure(Face::Display, font, label);
        let tri = font * 0.62;
        let line_cy = bar_y + bar_h * 0.5 - 7.0;
        let start_x = px0 + (play_w - text_w - tri - 9.0) * 0.5;
        draw_play_triangle(ui, start_x, line_cy, tri, ink);
        ui.text(
            Face::Display,
            font,
            start_x + tri + 9.0,
            text_vcenter(line_cy - 20.0, 40.0, font),
            ink,
            label,
        );
        let secs = play_seconds;
        let time = if secs == 0 {
            "0 h".to_owned()
        } else if secs < 3600 {
            format!("{} min", (secs / 60).max(1))
        } else {
            format!("{} h", secs / 3600)
        };
        draw_centered_text(
            ui,
            Face::LauncherMedium,
            9.5,
            [px0, bar_y + bar_h - 20.0, play_w, 12.0],
            hex(0xffd2da, 220),
            &time,
        );
        press_scale(ui, v_start, slab, Action::Play);
        targets.insert(0, (slab, Action::Play));
    }

    // Error line floating above the bar.
    if progress.is_none()
        && let Some(error) = error
    {
        let e = ui.fit_text(Face::LauncherRegular, 11.5, error, w - 2.0 * m - 40.0);
        let ew = ui.measure(Face::LauncherRegular, 11.5, &e);
        ui.text(
            Face::LauncherRegular,
            11.5,
            ax + (w - ew) * 0.5,
            bar_y - 22.0,
            theme::CRIMSON_BRIGHT(),
            &e,
        );
    }

    // ---- Bottom-left: account chip ----
    let acc_max = (px0 - ax - 12.0).min(w * 0.3);
    if acc_max >= 44.0 {
        let acc_w = (ui.measure(Face::LauncherMedium, 12.0, account_status) + 48.0).min(acc_max);
        // Flush to the screen's left edge.
        let acc = [ax, ay + h - m - 40.0, acc_w, 40.0];
        ui.rounded(acc[0], acc[1], acc[2], acc[3], 20.0, theme::GLASS());
        ui.border(acc[0], acc[1], acc[2], acc[3], 20.0, theme::GLASS_BORDER());
        draw_page_icon(
            ui,
            Page::Account,
            acc[0] + 20.0,
            acc[1] + 20.0,
            theme::MUTED(),
        );
        if acc_w >= 72.0 {
            let fit = ui.fit_text(Face::LauncherMedium, 12.0, account_status, acc_w - 48.0);
            ui.text(
                Face::LauncherMedium,
                12.0,
                acc[0] + 36.0,
                text_vcenter(acc[1], 40.0, 12.0),
                theme::TEXT(),
                &fit,
            );
        }
        targets.push((acc, Action::Account));
    }

    // ---- Bottom-right: Help → the Volt Discord ----
    let help = [ax + w - 96.0, ay + h - m - 40.0, 96.0, 40.0];
    ui.rounded(help[0], help[1], help[2], help[3], 12.0, theme::GLASS());
    ui.border(
        help[0],
        help[1],
        help[2],
        help[3],
        12.0,
        theme::GLASS_BORDER(),
    );
    draw_headset_icon(ui, help[0] + 22.0, help[1] + 20.0, theme::TEXT());
    ui.text(
        Face::LauncherMedium,
        12.5,
        help[0] + 38.0,
        text_vcenter(help[1], 40.0, 12.5),
        theme::TEXT(),
        "Help",
    );
    targets.push((help, Action::Discord));
    char_rect
}
