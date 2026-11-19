use horizon_client::{
    graphics::GraphicsApi,
    official_install::InstallProgress,
    ui_theme::{Face, Ui},
};
include!("../../horizon-client/src/launcher_ui/page.rs");
include!("../../horizon-client/src/launcher_ui/home.rs");
include!("../../horizon-client/src/launcher_ui/console.rs");
include!("../../horizon-client/src/launcher_ui/classic.rs");
include!("../../horizon-client/src/launcher_ui/gallery.rs");
include!("../../horizon-client/src/launcher_ui/servers.rs");
include!("../../horizon-client/src/launcher_ui/media.rs");
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Action {
    Play,
    Logs,
    Account,
    OpenVoltBoost,
    ToggleHomeStyle,
    Studio(crate::controls_studio::StudioAction),
    Page(Page),
    Profile,
    ViewWorld(usize),
    OpenCaptures,
    ViewScreenshot(usize),
    DeleteRecording(usize),
    WatchReplay(usize),
    ExportReplay(usize),
    ViewReplayCover(usize),
    DeleteReplay(usize),
    ConfirmRemove,
    SaveServer,
    ConnectServer(usize),
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
    offline_name: &str,
    seconds: u64,
    ready: bool,
    progress: Option<InstallProgress>,
    message: Option<&str>,
    avatar: &mut crate::Avatar,
) {
    avatar.rect = None;
    let (w, h) = (ui.w, ui.h);
    draw_launcher_scene(ui, w, h, phase);
    if settings {
        ui.text(Face::Display, 24., 24., 24., theme::TEXT(), "Settings");
        button(
            ui,
            targets,
            [24., 154., 360., 52.],
            if theme::liquid() {
                "Home style: Liquid Glass"
            } else {
                "Home style: Classic"
            },
            Action::ToggleHomeStyle,
            false,
        );
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
    } else if !theme::liquid() {
        avatar.rect = Some(classic_home(
            ui,
            [0., 0., w, h],
            &login.status(offline_name),
            login.profile.as_deref().unwrap_or(offline_name).to_owned(),
            seconds,
            ready,
            progress,
            message,
            targets,
            |ui, rect| {
                crate::skin_preview::draw_player_3d(
                    ui,
                    rect,
                    avatar.yaw,
                    avatar.pitch,
                    phase,
                    0.,
                    2,
                    3,
                    None,
                    None,
                    avatar.uuid,
                )
            },
        ));
    } else {
        liquid_home(
            ui,
            [0., 0., w, h],
            &login.status(offline_name),
            seconds,
            ready,
            progress,
            message,
            targets,
        );
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
    let (x, y) = tab_header(ui, targets, "Worlds");
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

fn tab_header(ui: &mut Ui, targets: &mut Vec<([f32; 4], Action)>, title: &str) -> (f32, f32) {
    let (x, y) = (16., 14.);
    let back = [x, y, 92., 38.];
    ui.rounded(x, y, 92., 38., 19., theme::GLASS());
    ui.border(x, y, 92., 38., 19., theme::GLASS_BORDER());
    draw_centered_text(ui, Face::LauncherMedium, 13., back, theme::TEXT(), "< Home");
    targets.push((back, Action::HomeTab(HomeTab::Overview)));
    ui.text(Face::Display, 21., x + 106., y + 5., theme::TEXT(), title);
    (x, y)
}
pub(super) fn servers_page(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    phase: f32,
    servers: &mut crate::Servers,
    photos: &mut crate::photo_cache::PhotoCache,
) {
    let (w, h) = (ui.w, ui.h);
    draw_launcher_scene(ui, w, h, phase);
    let (x, y) = tab_header(ui, targets, "Multiplayer");
    server_rows(
        ui,
        [x, y + 50., w - 2. * x, h - 2. * y - 50.],
        &servers.address,
        &servers.rows,
        photos,
        &mut servers.scroll,
        targets,
        None,
    );
    if let Some(note) = &servers.note {
        ui.text(
            Face::LauncherRegular,
            12.,
            x + 230.,
            y + 12.,
            theme::MUTED(),
            note,
        );
    }
}

pub(super) fn media_page(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    phase: f32,
    media: &mut crate::media::Media,
    photos: &mut crate::photo_cache::PhotoCache,
) {
    let (w, h) = (ui.w, ui.h);
    draw_launcher_scene(ui, w, h, phase);
    let (x, y) = tab_header(
        ui,
        targets,
        if media.replay_tab {
            "Replays"
        } else {
            "Captures"
        },
    );
    let area = [x, y + 50., w - 2. * x, h - 2. * y - 50.];
    if media.replay_tab {
        replay_rows(
            ui,
            area,
            &media.replays,
            &media.home.join("replays"),
            photos,
            targets,
            &mut media.scroll,
        );
    } else {
        captures_grid(
            ui,
            area,
            &media.shots,
            &media.clips,
            &media.home.join("screenshots"),
            photos,
            targets,
            &mut media.scroll,
        );
    }
    if let Some(view) = &media.view {
        targets.clear();
        draw_photo_viewer(ui, targets, photos, view, w, h);
    }
}

pub(super) fn logs_page(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    phase: f32,
    summary: &str,
) {
    let (w, h) = (ui.w, ui.h);
    draw_launcher_scene(ui, w, h, phase);
    let (x, y) = tab_header(ui, targets, "Launcher log");
    console_panel(
        ui,
        [x, y + 50., w - 2. * x, h - 2. * y - 50.],
        targets,
        summary,
    );
}
