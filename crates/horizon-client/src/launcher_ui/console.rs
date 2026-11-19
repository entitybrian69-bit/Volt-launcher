// Shared Android/PC diagnostics panel. Hosts supply the local snapshot on Refresh.
fn console_panel(ui: &mut Ui, area: [f32; 4], targets: &mut Vec<([f32; 4], Action)>, status: &str) {
    let [x, y, w, h] = area;
    ui.text(
        Face::LauncherMedium,
        15.0,
        x + 8.0,
        y + 8.0,
        theme::TEXT(),
        "Launcher diagnostics",
    );
    let status = if status.is_empty() {
        "Tap Refresh to read the latest local diagnostic summary."
    } else {
        status
    };
    let status = ui.fit_text(Face::LauncherRegular, 12.5, status, w - 16.0);
    ui.text(
        Face::LauncherRegular,
        12.5,
        x + 8.0,
        y + 38.0,
        theme::MUTED(),
        &status,
    );
    if h >= 100.0 {
        styled_button(
            ui,
            targets,
            [x + 8.0, y + 64.0, 100.0, 32.0],
            "Refresh",
            Action::Logs,
            BtnStyle::Secondary,
            true,
        );
    }
}
