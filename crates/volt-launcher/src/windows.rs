#[path = "../../horizon-client/src/android/login.rs"]
mod login;
mod pages;
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
fn prepare_and_launch(home: PathBuf, progress: Progress) -> Result<Child> {
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
    let (player, secrets) = pass
        .session
        .ok_or("A signed-in Minecraft profile is required.")?;
    // Renew once more after a potentially long install; session files have a short lifetime.
    let (player, secrets) = account::fresh(
        &account::Endpoints::live(),
        id.as_deref().unwrap_or(""),
        &player,
        secrets,
    )?;
    accounts.add(account::store(), player.clone(), &secrets)?;
    accounts.save()?;
    let file = account::write_session_file(&home.join("sessions"), &player, &secrets)?;
    let child = Command::new(engine)
        .current_dir(&home)
        .arg("--home")
        .arg(&home)
        .arg("--game-dir")
        .arg(&home)
        .arg("--resources")
        .arg(
            home.join(".local/horizon-client")
                .join(horizon_core::VANILLA_VERSION),
        )
        .arg("--session-file")
        .arg(&file)
        .spawn();
    if child.is_err() {
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
}
impl App {
    fn play(&mut self) {
        if self.worker.is_some() || self.child.is_some() {
            return;
        }
        if !self.login.signed_in {
            self.login.visible = true;
            return;
        }
        self.message = Some("Checking Minecraft ownership…".into());
        let (tx, rx) = mpsc::channel();
        self.worker = Some(rx);
        let home = self.home.clone();
        let progress = self.progress.clone();
        std::thread::spawn(move || {
            let _ = tx.send(prepare_and_launch(home, progress).map_err(|e| e.to_string()));
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
            Play => self.play(),
            Page(pages::Page::Account) => self.login.visible = true,
            Page(pages::Page::Settings) => self.settings = true,
            Page(pages::Page::Play) => self.settings = false,
            Renderer => {
                let choices = GraphicsApi::choices();
                let at = choices.iter().position(|a| *a == self.api).unwrap_or(0);
                let next = choices[(at + 1) % choices.len()];
                let mut g = Graphics::load(&self.home);
                g.api = next;
                g.save(&self.home);
                self.api = Graphics::load(&self.home).api;
                if self.api != next {
                    self.message = Some("Could not save renderer setting".into());
                }
            }
            SignIn => self.login.start(&id),
            DeviceSignIn => self.login.start_device(&id),
            SignOut => self.login.sign_out(),
            CloseLogin | CancelLogin => self.login.close(),
            LoginLink => self.login.open_link(),
            CopyLoginCode => self.login.copy_code(),
            ReviewApp => self.login.review_app(),
            _ => self.message = Some("This Volt screen has not been ported to Windows yet.".into()),
        }
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
        pages::draw(
            &mut ui,
            &mut self.targets,
            self.start.elapsed().as_secs_f32(),
            self.settings,
            self.api,
            &self.login,
            self.seconds,
            self.ready,
            *self.progress.lock().unwrap(),
            self.message.as_deref(),
        );
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
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = (position.x as f32, position.y as f32)
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                ..
            } => {
                let (x, y) = (self.pointer.0 / self.density, self.pointer.1 / self.density);
                if let Some((_, a)) = self
                    .targets
                    .iter()
                    .find(|([l, t, w, h], _)| x >= *l && x < l + w && y >= *t && y < t + h)
                {
                    self.action(*a);
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                self.login.close();
                self.settings = false;
            }
            WindowEvent::RedrawRequested => match self.draw() {
                Ok(true) if self.smoke => el.exit(),
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
    let gui = UiResources::load(&home.join("installer"), &home.join("installer"))?;
    let mut app = App {
        ready: official_install::ready(&home),
        api: Graphics::load(&home).api,
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
        start: Instant::now(),
        pointer: (0., 0.),
        density: 1.,
        targets: vec![],
        message: None,
        fatal: None,
        worker: None,
        progress: Default::default(),
        child: None,
        smoke: std::env::args().any(|a| a == "--smoke-frame"),
    };
    EventLoop::new()?.run_app(&mut app)?;
    if let Some(e) = app.fatal {
        return Err(e.into());
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
