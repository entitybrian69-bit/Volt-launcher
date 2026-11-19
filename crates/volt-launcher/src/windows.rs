#[path = "../../horizon-client/src/android/login.rs"]
mod login;
mod pages;
#[path = "../../horizon-client/src/android/photo_cache.rs"]
mod photo_cache;
mod skin_preview {
    include!("../../horizon-client/src/launcher_ui/picture_hash.rs");
}
#[path = "../../../launcher/src/server_list.rs"]
mod server_list;
struct Servers {
    rows: Vec<server_list::Server>,
    address: String,
    editing: bool,
    scroll: f32,
    note: Option<String>,
}
struct Worlds {
    dir: PathBuf,
    names: Vec<String>,
    scroll: f32,
    view: Option<pages::PhotoView>,
}

mod chooser {
    use crate::pages::{Action, button};
    use horizon_client::{
        graphics::GraphicsApi,
        ui_theme::{Face, Ui},
    };
    include!("../../horizon-client/src/launcher_ui/chooser.rs");
    impl Chooser {
        pub(super) fn renderers(active: GraphicsApi, device: Option<String>) -> Self {
            Self {
                title: "Rendering mode",
                help: "Select the graphics API for the next start.",
                device,
                scroll: 0.0,
                choices: GraphicsApi::choices()
                    .iter()
                    .map(|a| {
                        (
                            a.label().to_string(),
                            String::new(),
                            Action::ChooseRenderer(*a),
                            active == *a,
                        )
                    })
                    .collect(),
            }
        }
    }
}
fn profile_dir(home: &std::path::Path, profile: u32) -> PathBuf {
    if profile == 0 {
        home.into()
    } else {
        home.join("profiles").join(format!("profile-{profile}"))
    }
}
#[path = "../../../launcher/src/skins.rs"]
mod skins;
mod account {
    pub const SERVICE: &str = "dev.volt.launcher.pc";
    include!("../../../launcher/src/account/protocol.rs");
}
mod proxy {
    pub fn agent(builder: ureq::AgentBuilder, url: &str) -> ureq::AgentBuilder {
        use horizon_port_pack::net;
        if net::direct(url) {
            return builder;
        }
        match net::scheme_of(url)
            .and_then(net::env_for)
            .and_then(|p| net::parse(&p))
        {
            Some(proxy) => builder.proxy(proxy),
            None => builder,
        }
    }
}
mod backend {
    use std::path::PathBuf;
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub fn data_dir() -> PathBuf {
        std::env::var_os("HORIZON_LAUNCHER_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA missing"))
                    .join("Volt Launcher")
            })
    }
}
mod platform {
    pub fn copy_login_code(text: &str) -> bool {
        arboard::Clipboard::new()
            .and_then(|mut c| c.set_text(text))
            .is_ok()
    }
    pub fn open_login_url(text: &str) -> Result<(), String> {
        let uri = url::Url::parse(text).map_err(|e| e.to_string())?;
        if uri.scheme() != "https" {
            return Err("Expected HTTPS login address".into());
        }
        open::that(uri.as_str()).map_err(|e| e.to_string())
    }
}
use horizon_client::{
    Result,
    gfx::Renderer,
    graphics::{Graphics, GraphicsApi},
    hud::HudBatch,
    launcher_render::{UiResources, loading_frame, loading_renderer},
    official_install::{self, InstallProgress},
    ui_theme::Ui,
};
use pages::Action;
use std::{
    path::PathBuf,
    process::{Child, Command},
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

type Progress = Arc<Mutex<Option<InstallProgress>>>;
fn prepare_and_launch(
    home: PathBuf,
    profile: u32,
    server: Option<String>,
    progress: Progress,
) -> Result<Child> {
    let id = account::client_id();
    let mut accounts = account::Accounts::load();
    let pass = account::launch(
        &account::Endpoints::live(),
        id.as_deref(),
        account::store(),
        &mut accounts,
    )
    .map_err(|e| e.to_string())?;
    accounts.save()?;
    let engine = std::env::current_exe()?.with_file_name("horizon-client.exe");
    if !engine.is_file() {
        return Err("The sibling horizon-client.exe is missing.".into());
    }
    if !official_install::ready(&home) {
        if pass.owner.is_none() {
            return Err("Verify ownership online before downloading Minecraft.".into());
        }
        // Only allowlisted generated tables ship with the app. Mojang assets are installed per player.
        let tables = engine.with_file_name("engine-data.zip");
        if !tables.is_file() {
            return Err("Missing engine-data.zip beside Volt Launcher.".into());
        }
        std::fs::copy(tables, home.join("engine-data.zip"))?;
        official_install::install_with_progress(&home, |p| *progress.lock().unwrap() = Some(p))?;
    }
    let game_dir = profile_dir(&home, profile);
    let mut cmd = Command::new(engine);
    cmd.current_dir(&game_dir)
        .arg("--home")
        .arg(&home)
        .arg("--game-dir")
        .arg(&game_dir)
        .arg("--resources")
        .arg(
            home.join(".local/horizon-client")
                .join(horizon_core::VANILLA_VERSION),
        );
    if let Some(server) = server {
        cmd.arg("--server").arg(server);
    }
    let mut session = None;
    if let Some((player, secrets)) = pass.session {
        let (player, secrets) = account::fresh(
            &account::Endpoints::live(),
            id.as_deref().unwrap_or(""),
            &player,
            secrets,
        )?;
        accounts.add(account::store(), player.clone(), &secrets)?;
        accounts.save()?;
        let file = account::write_session_file(&home.join("sessions"), &player, &secrets)?;
        cmd.arg("--session-file").arg(&file);
        session = Some(file);
    } else if let Some(name) = accounts.name_arg() {
        // Only an explicitly selected offline name, after the same owner gate above.
        cmd.arg("--name").arg(name).envs(accounts.look_env());
    } else {
        return Err(pass
            .offline
            .unwrap_or_else(|| "Sign in again to start this profile.".into())
            .into());
    }
    let child = cmd.spawn();
    if child.is_err()
        && let Some(file) = session
    {
        let _ = std::fs::remove_file(file);
    }
    Ok(child?)
}

struct App {
    home: PathBuf,
    gui: UiResources,
    renderer: Option<Renderer>,
    window: Option<Arc<Window>>,
    login: login::Login,
    settings: bool,
    worlds: Option<Worlds>,
    servers: Option<Servers>,
    editing_name: bool,
    offline_name: String,
    photos: photo_cache::PhotoCache,
    chooser: Option<chooser::Chooser>,
    profile: u32,
    api: GraphicsApi,
    start: Instant,
    pointer: (f32, f32),
    density: f32,
    targets: Vec<([f32; 4], Action)>,
    message: Option<String>,
    fatal: Option<String>,
    ready: bool,
    worker: Option<mpsc::Receiver<std::result::Result<Child, String>>>,
    progress: Progress,
    child: Option<(Child, Instant)>,
    seconds: u64,
    smoke: bool,
    smoke_step: usize,
}
impl App {
    fn play(&mut self, server: Option<String>) {
        if self.worker.is_some() || self.child.is_some() {
            return;
        }
        if !self.login.signed_in && !account::Accounts::load().has_owner(account::store()) {
            self.login.visible = true;
            return;
        }
        self.message = Some("Checking Minecraft ownership…".into());
        let (tx, rx) = mpsc::channel();
        self.worker = Some(rx);
        let home = self.home.clone();
        let progress = self.progress.clone();
        let profile = self.profile;
        std::thread::spawn(move || {
            let _ = tx.send(
                prepare_and_launch(home, profile, server, progress).map_err(|e| e.to_string()),
            );
        });
    }
    fn action(&mut self, a: Action) {
        use Action::*;
        if self.worker.is_some() {
            return;
        }
        pages::feedback(a);
        let id = account::client_id().unwrap_or_default();
        match a {
            HomeTab(pages::HomeTab::Worlds) => {
                let dir = profile_dir(&self.home, self.profile).join("saves");
                self.worlds = Some(Worlds {
                    names: pages::world_folders_in(&dir),
                    dir,
                    scroll: 0.,
                    view: None,
                });
            }
            HomeTab(pages::HomeTab::Overview) => {
                self.worlds = None;
                self.servers = None;
            }
            HomeTab(pages::HomeTab::Multiplayer) => {
                self.servers = Some(Servers {
                    rows: server_list::List::load(&profile_dir(&self.home, self.profile)).servers(),
                    address: String::new(),
                    scroll: 0.,
                    editing: false,
                    note: None,
                });
            }
            Field("server") => {
                if let Some(s) = &mut self.servers {
                    s.editing = true;
                    s.note = Some("Type a server address, then press Enter to save.".into());
                }
            }
            SaveServer => {
                if let Some(s) = &mut self.servers {
                    let dir = profile_dir(&self.home, self.profile);
                    let address = s.address.trim();
                    let result = server_list::List::try_load(&dir).and_then(|mut l| {
                        if l.servers()
                            .iter()
                            .any(|srv| srv.ip.eq_ignore_ascii_case(address))
                        {
                            return Err("Server already saved.".into());
                        }
                        l.add(address, address)?;
                        l.save(&dir)?;
                        s.rows = l.servers();
                        Ok(())
                    });
                    s.editing = false;
                    s.note = Some(
                        result
                            .err()
                            .unwrap_or_else(|| "Saved to this instance.".into()),
                    );
                }
            }
            ConnectServer(i) => {
                if let Some(ip) = self
                    .servers
                    .as_ref()
                    .and_then(|s| s.rows.get(i))
                    .map(|s| s.ip.clone())
                {
                    self.play(Some(ip));
                }
            }
            Field("name") => self.editing_name = true,
            FinishNameEdit => {
                let mut ac = account::Accounts::load();
                let result = ac
                    .play_as(
                        account::store(),
                        account::Player::Offline(self.offline_name.clone()),
                    )
                    .and_then(|_| ac.save());
                self.editing_name = false;
                self.login.refresh_status();
                if let Err(e) = result {
                    self.login.hide();
                    self.message = Some(e);
                    self.offline_name = account::Accounts::load().name_arg().unwrap_or("").into();
                }
            }
            ViewWorld(i) => {
                if let Some(worlds) = &mut self.worlds
                    && let Some(name) = worlds.names.get(i)
                {
                    let path = worlds.dir.join(name).join("icon.png");
                    worlds.view = Some(pages::PhotoView {
                        title: name.clone(),
                        key: photo_cache::file_key(&path),
                        source: photo_cache::Source::File(path),
                    });
                }
            }
            ClosePhoto => {
                if let Some(worlds) = &mut self.worlds {
                    worlds.view = None;
                }
            }
            Play => self.play(None),
            Page(pages::Page::Account) => self.login.visible = true,
            Page(pages::Page::Settings) => self.settings = true,
            Page(pages::Page::Play) => self.settings = false,
            Discord => {
                self.message =
                    platform::open_login_url(horizon_client::discord_presence::DISCORD_INVITE)
                        .err();
            }
            Profile => self.chooser = Some(chooser::Chooser::profiles(&self.home, self.profile)),
            Dismiss => self.chooser = None,
            ChooseProfile(profile) => {
                let dir = profile_dir(&self.home, profile);
                match std::fs::create_dir_all(&dir).and_then(|_| {
                    std::fs::write(self.home.join("selected-profile"), profile.to_string())
                }) {
                    Ok(()) => {
                        self.profile = profile;
                        self.api = Graphics::load(&dir).api;
                    }
                    Err(e) => self.message = Some(e.to_string()),
                }
                self.chooser = None;
            }
            Renderer => {
                self.chooser = Some(chooser::Chooser::renderers(
                    self.api,
                    self.renderer.as_ref().map(|r| {
                        format!(
                            "GPU: {} / {} {}",
                            r.info.name, r.info.driver, r.info.driver_info
                        )
                    }),
                ))
            }
            ChooseRenderer(next) => {
                let dir = profile_dir(&self.home, self.profile);
                let mut g = Graphics::load(&dir);
                g.api = next;
                g.save(&dir);
                self.api = Graphics::load(&dir).api;
                if self.api != next {
                    self.message = Some("Could not save renderer setting".into());
                }
                self.chooser = None;
            }
            SignIn => self.login.start(&id),
            DeviceSignIn => self.login.start_device(&id),
            SignOut => self.login.sign_out(),
            CloseLogin | CancelLogin => {
                self.editing_name = false;
                self.login.close();
            }
            LoginLink => self.login.open_link(),
            CopyLoginCode => self.login.copy_code(),
            ReviewApp => self.login.review_app(),
            _ => self.message = Some("This Volt screen has not been ported to Windows yet.".into()),
        }
    }
    /// Exercise navigation through the actual hit rectangles after each present. No credentials,
    /// network, writes to settings or invocation of Play are needed for this startup check.
    fn smoke_frame(&mut self) -> Result<bool> {
        use pages::{HomeTab, Page};
        let path = [
            Action::Page(Page::Account),
            Action::CloseLogin,
            Action::Profile,
            Action::Dismiss,
            Action::HomeTab(HomeTab::Worlds),
            Action::HomeTab(HomeTab::Overview),
            Action::HomeTab(HomeTab::Multiplayer),
            Action::HomeTab(HomeTab::Overview),
            Action::Page(Page::Settings),
            Action::Renderer,
            Action::Dismiss,
            Action::Page(Page::Play),
        ];
        if let Some(&next) = path.get(self.smoke_step) {
            let &(rect, _) = self
                .targets
                .iter()
                .find(|(_, a)| *a == next)
                .ok_or_else(|| format!("Missing Volt hit target at step {}", self.smoke_step))?;
            let actual = self
                .hit(rect[0] + rect[2] / 2., rect[1] + rect[3] / 2.)
                .ok_or("Hit rectangle missing")?;
            if actual != next {
                return Err(
                    format!("Obstructed Volt hit target at step {}", self.smoke_step).into(),
                );
            }
            self.action(actual);
            self.smoke_step += 1;
            Ok(false)
        } else if self.targets.iter().any(|(_, a)| *a == Action::Play) {
            self.smoke = false;
            Ok(true)
        } else {
            Err("Did not return to Volt home".into())
        }
    }
    fn hit(&self, x: f32, y: f32) -> Option<Action> {
        self.targets
            .iter()
            .find(|([l, t, w, h], _)| x >= *l && x < l + w && y >= *t && y < t + h)
            .map(|(_, a)| *a)
    }
    fn draw(&mut self) -> Result<bool> {
        self.login.poll();
        if let Some(rx) = &self.worker {
            match rx.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {}
                result => {
                    self.worker = None;
                    *self.progress.lock().unwrap() = None;
                    self.ready = official_install::ready(&self.home);
                    match result.unwrap_or_else(|_| Err("Launcher worker stopped".into())) {
                        Ok(c) => {
                            self.child = Some((c, Instant::now()));
                            self.message = None;
                        }
                        Err(e) => self.message = Some(e),
                    }
                }
            }
        }
        if let Some((child, start)) = &mut self.child
            && let Some(exit) = child.try_wait()?
        {
            self.seconds += start.elapsed().as_secs();
            std::fs::write(self.home.join("play-seconds"), self.seconds.to_string())?;
            self.child = None;
            if !exit.success() {
                self.message = Some(format!("Minecraft exited: {exit}"));
            }
        }
        let r = self.renderer.as_mut().ok_or("Renderer missing")?;
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        let scale = b.scale;
        let mut ui = Ui::new(
            &self.gui.atlas,
            &mut b,
            (self.pointer.0 / scale, self.pointer.1 / scale),
        );
        self.density = ui.density;
        self.targets.clear();
        self.photos.poll();
        if let Some(servers) = &mut self.servers {
            pages::servers_page(
                &mut ui,
                &mut self.targets,
                self.start.elapsed().as_secs_f32(),
                servers,
                &mut self.photos,
            );
        } else if let Some(worlds) = &mut self.worlds {
            pages::worlds_page(
                &mut ui,
                &mut self.targets,
                self.start.elapsed().as_secs_f32(),
                worlds,
                &mut self.photos,
            );
        } else {
            pages::draw(
                &mut ui,
                &mut self.targets,
                self.start.elapsed().as_secs_f32(),
                self.settings,
                self.api,
                &self.login,
                &self.offline_name,
                self.seconds,
                self.ready,
                *self.progress.lock().unwrap(),
                self.message.as_deref(),
            );
        }
        if let Some(c) = &mut self.chooser {
            c.draw(&mut ui, &mut self.targets);
        }
        if self.login.visible {
            self.login.draw(
                &mut ui,
                &mut self.targets,
                &self.offline_name,
                self.editing_name,
            );
        }
        ui.finish();
        let shown = r.render(&loading_frame(&b))?;
        if !shown {
            horizon_client::ui_theme::reset_atlas();
            self.gui.atlas.font.reset_cache();
        }
        Ok(shown)
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let make = || -> Result<_> {
            let w = Arc::new(
                el.create_window(
                    Window::default_attributes()
                        .with_title("Volt Launcher")
                        .with_window_icon(horizon_client::window::icon())
                        .with_inner_size(LogicalSize::new(1100., 720.))
                        .with_min_inner_size(LogicalSize::new(740., 520.)),
                )?,
            );
            let r = loading_renderer(w.clone(), false, Some(&self.gui), None, self.api)?;
            Ok((w, r))
        };
        match make() {
            Ok((w, r)) => {
                w.request_redraw();
                self.window = Some(w);
                self.renderer = Some(r);
            }
            Err(e) => {
                self.fatal = Some(e.to_string());
                el.exit();
            }
        }
    }
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        el.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(33),
        ));
    }
    fn new_events(&mut self, _: &ActiveEventLoop, cause: winit::event::StartCause) {
        if matches!(cause, winit::event::StartCause::ResumeTimeReached { .. })
            && let Some(w) = &self.window
        {
            w.request_redraw();
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => {
                if s.width > 0
                    && s.height > 0
                    && let Some(r) = &mut self.renderer
                {
                    r.resize(s.width, s.height);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y * 38.0,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32 / self.density,
                };
                if let Some(c) = &mut self.chooser {
                    c.scroll -= dy;
                } else if let Some(servers) = &mut self.servers {
                    servers.scroll -= dy;
                } else if let Some(worlds) = &mut self.worlds
                    && worlds.view.is_none()
                {
                    worlds.scroll -= dy;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = (position.x as f32, position.y as f32)
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                ..
            } => {
                let (x, y) = (self.pointer.0 / self.density, self.pointer.1 / self.density);
                if let Some(a) = self.hit(x, y) {
                    self.action(a);
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.logical_key != Key::Named(NamedKey::Escape)
                    && (self.login.visible && self.editing_name
                        || !self.login.visible
                            && self.servers.as_ref().is_some_and(|s| s.editing)) =>
            {
                if event.logical_key == Key::Named(NamedKey::Enter) {
                    self.action(if self.editing_name {
                        Action::FinishNameEdit
                    } else {
                        Action::SaveServer
                    });
                } else {
                    let (field, max) = if self.editing_name {
                        (&mut self.offline_name, 16)
                    } else {
                        (&mut self.servers.as_mut().unwrap().address, 255)
                    };
                    if event.logical_key == Key::Named(NamedKey::Backspace) {
                        field.pop();
                    } else if let Some(text) = event.text {
                        for c in text.chars().filter(|c| !c.is_control()) {
                            if field.len() + c.len_utf8() <= max {
                                field.push(c);
                            }
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                if self.login.visible {
                    self.login.close();
                    self.editing_name = false;
                } else if self.servers.is_some() {
                    self.servers = None;
                } else if let Some(worlds) = &mut self.worlds {
                    if worlds.view.take().is_none() {
                        self.worlds = None;
                    }
                } else if self.chooser.take().is_none() {
                    self.login.close();
                    self.settings = false;
                }
            }
            WindowEvent::RedrawRequested => match self.draw() {
                Ok(true) if self.smoke => match self.smoke_frame() {
                    Ok(true) => el.exit(),
                    Ok(false) => {}
                    Err(e) => {
                        self.fatal = Some(e.to_string());
                        el.exit();
                    }
                },
                Ok(_) => {}
                Err(e) => {
                    self.fatal = Some(e.to_string());
                    el.exit();
                }
            },
            _ => {}
        }
    }
}
fn run() -> Result<()> {
    let home = backend::data_dir();
    std::fs::create_dir_all(&home)?;
    let _lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(home.join("volt-launcher.lock"))?;
    _lock
        .try_lock()
        .map_err(|e| format!("Cannot lock Volt home (another launcher may be running): {e}"))?;
    let profile = std::fs::read_to_string(home.join("selected-profile"))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let gui = UiResources::load(&home.join("installer"), &home.join("installer"))?;
    let mut app = App {
        ready: official_install::ready(&home),
        api: Graphics::load(&profile_dir(&home, profile)).api,
        profile,
        chooser: None,
        seconds: std::fs::read_to_string(home.join("play-seconds"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        home,
        gui,
        renderer: None,
        window: None,
        login: login::Login::new(),
        settings: false,
        photos: photo_cache::PhotoCache::new(),
        worlds: None,
        servers: None,
        editing_name: false,
        offline_name: account::Accounts::load().name_arg().unwrap_or("").into(),
        start: Instant::now(),
        pointer: (0., 0.),
        density: 1.,
        targets: vec![],
        message: None,
        fatal: None,
        worker: None,
        progress: Default::default(),
        child: None,
        smoke_step: 0,
        smoke: std::env::args().any(|a| a == "--smoke-frame"),
    };
    EventLoop::new()?.run_app(&mut app)?;
    if let Some(e) = app.fatal {
        return Err(e.into());
    }
    if app.smoke {
        return Err("Volt frame test closed before completing".into());
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        let _ = std::fs::write(
            backend::data_dir().join("launcher-error.txt"),
            e.to_string(),
        );
        std::process::exit(1);
    }
}
