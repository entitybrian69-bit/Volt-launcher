use horizon_client::{
    graphics::GraphicsApi,
    official_install::InstallProgress,
    ui_theme::{Face, Ui},
};
include!("../../horizon-client/src/launcher_ui/page.rs");
include!("../../horizon-client/src/launcher_ui/home.rs");
include!("../../horizon-client/src/launcher_ui/gallery.rs");
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Action {
    Play,
    Page(Page),
    Profile,
    ViewWorld(usize),
    ClosePhoto,
    HomeTab(HomeTab),
    Discord,
    Renderer,
    Dismiss,
    ChooseProfile(u32),
    ChooseRenderer(GraphicsApi),
    SignIn,
    DeviceSignIn,
    SignOut,
    CloseLogin,
    CancelLogin,
    LoginLink,
    CopyLoginCode,
    ReviewApp,
    Field(&'static str),
    FinishNameEdit,
}
thread_local! {static ACTIVE_FEEDBACK:std::cell::RefCell<Option<(Action,std::time::Instant)>> = const{std::cell::RefCell::new(None)};}
pub fn feedback(a: Action) {
    ACTIVE_FEEDBACK.with(|f| *f.borrow_mut() = Some((a, std::time::Instant::now())));
}
pub fn draw(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    phase: f32,
    settings: bool,
    api: GraphicsApi,
    login: &crate::login::Login,
    seconds: u64,
    ready: bool,
    progress: Option<InstallProgress>,
    message: Option<&str>,
) {
    let (w, h) = (ui.w, ui.h);
    draw_launcher_scene(ui, w, h, phase);
    if settings {
        ui.text(Face::Display, 24., 24., 24., theme::TEXT(), "Settings");
        button(
            ui,
            targets,
            [24., 90., 360., 52.],
            &format!("Graphics API (next start): {}", api.label()),
            Action::Renderer,
            false,
        );
        button(
            ui,
            targets,
            [24., h - 76., 180., 52.],
            "Home",
            Action::Page(Page::Play),
            true,
        );
    } else {
        liquid_home(
            ui,
            [0., 0., w, h],
            &login.status(""),
            seconds,
            ready,
            progress,
            message,
            targets,
        );
    }
    if login.visible {
        login.draw(ui, targets, "", false);
    }
}

pub(super) fn worlds_page(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    phase: f32,
    worlds: &mut crate::Worlds,
    photos: &mut crate::photo_cache::PhotoCache,
) {
    let (w, h) = (ui.w, ui.h);
    draw_launcher_scene(ui, w, h, phase);
    let (x, y) = (16., 14.);
    let back = [x, y, 92., 38.];
    ui.rounded(x, y, 92., 38., 19., theme::GLASS());
    ui.border(x, y, 92., 38., 19., theme::GLASS_BORDER());
    draw_centered_text(ui, Face::LauncherMedium, 13., back, theme::TEXT(), "< Home");
    targets.push((back, Action::HomeTab(HomeTab::Overview)));
    ui.text(
        Face::Display,
        21.,
        x + 106.,
        y + 5.,
        theme::TEXT(),
        "Worlds",
    );
    world_grid(
        ui,
        [x, y + 50., w - 2. * x, h - 2. * y - 50.],
        &worlds.dir,
        &worlds.names,
        photos,
        targets,
        &mut worlds.scroll,
    );
    if let Some(view) = &worlds.view {
        targets.clear();
        draw_photo_viewer(ui, targets, photos, view, w, h);
    }
}
