use horizon_client::{
    graphics::GraphicsApi,
    official_install::InstallProgress,
    ui_theme::{Face, Ui},
};
include!("../../horizon-client/src/launcher_ui/page.rs");
include!("../../horizon-client/src/launcher_ui/home.rs");
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Action {
    Play,
    Page(Page),
    Profile,
    HomeTab(HomeTab),
    Discord,
    Renderer,
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
