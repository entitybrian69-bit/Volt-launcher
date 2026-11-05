//! Mobile launcher state. Browser operations reuse the desktop backend unchanged.
use crate::browser::{self, Kind, Project, Query, Source, installed::Installed};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
};
pub fn source_only_mod(project: &Project) -> bool {
    project.source == browser::SourceId::Horizon
        && project.loaders.iter().any(|loader| loader == "hnmod")
}

/// Temporary dev registration published as Minecraft Nintendo Switch by PrismarineJS.
/// Account tokens and client secrets are never bundled.
pub const MICROSOFT_CLIENT_ID: &str = "00000000441cc96b";

/// Vulkan, Volt Vulkan Supporter, and OpenGL ES rendering paths.
/// Defaults to `VulkanCompatibility` (`GraphicsApi::Auto`), which tries native
/// Vulkan first, then Volt Vulkan Supporter for legacy GPUs, then OpenGL ES.
#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RendererMode {
    Vulkan,
    VulkanSupporter,
    OpenGl,
    #[default]
    VulkanCompatibility,
}
impl RendererMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Vulkan => "Vulkan / GPU batching",
            Self::VulkanSupporter => "Vulkan Supporter / legacy GPUs",
            Self::OpenGl => "OpenGL ES / fallback",
            Self::VulkanCompatibility => "Vulkan / compatibility (auto)",
        }
    }

    pub fn graphics_api(self) -> horizon_client::graphics::GraphicsApi {
        match self {
            Self::Vulkan => horizon_client::graphics::GraphicsApi::Vulkan,
            Self::VulkanSupporter => horizon_client::graphics::GraphicsApi::VulkanSupporter,
            Self::OpenGl => horizon_client::graphics::GraphicsApi::OpenGl,
            Self::VulkanCompatibility => horizon_client::graphics::GraphicsApi::Auto,
        }
    }
}

fn default_pack_source() -> crate::shader_catalog::Origin {
    crate::shader_catalog::Origin::Modrinth
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub profile: u32,
    pub proxy: Option<String>,
    pub name: String,
    pub server: String,
    #[serde(skip)]
    pub microsoft_client_id: String,
    pub renderer: RendererMode,
    pub render_scale: crate::display::RenderScale,
    pub orientation: Orientation,
    #[serde(flatten)]
    pub screen: crate::display::ScreenPreferences,
    pub input_host: InputHost,
    pub keypad_controls: bool,
    pub record_performance: bool,
    #[serde(flatten)]
    pub device: crate::device_preferences::Settings,
    pub shader_source: crate::shader_catalog::Origin,
    pub mod_source: crate::shader_catalog::Origin,
    #[serde(default = "default_pack_source")]
    pub pack_source: crate::shader_catalog::Origin,
    #[serde(default)]
    pub custom_vulkan_driver: Option<String>,
    #[serde(default)]
    pub custom_vulkan_driver_name: Option<String>,
}

pub use horizon_client::android_gamepad::InputMode as InputHost;

/// Android ActivityInfo orientations, independent of the device's natural axis.
#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Orientation {
    #[default]
    Landscape,
    Portrait,
    ReverseLandscape,
    ReversePortrait,
    Auto,
}
impl Orientation {
    pub const ALL: [Self; 5] = [
        Self::Landscape,
        Self::Portrait,
        Self::ReverseLandscape,
        Self::ReversePortrait,
        Self::Auto,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Landscape => "Landscape (sensor)",
            Self::Portrait => "Portrait (sensor)",
            Self::ReverseLandscape => "Landscape (reversed)",
            Self::ReversePortrait => "Portrait (reversed)",
            Self::Auto => "Automatic rotation",
        }
    }
    pub fn android_value(self) -> i32 {
        match self {
            Self::Landscape => 6,
            Self::Portrait => 7,
            Self::ReverseLandscape => 8,
            Self::ReversePortrait => 9,
            Self::Auto => 10,
        }
    }
}
impl Config {
    pub fn load(home: &Path) -> Self {
        let value: serde_json::Value = std::fs::read(home.join("android-launcher.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let mut config: Self = serde_json::from_value(value.clone()).unwrap_or_default();
        if value.get("renderer").is_none() {
            config.renderer = RendererMode::VulkanCompatibility;
        }
        // Ignore saved app registrations while this dev build uses the fixed ID.
        config.microsoft_client_id =
            crate::account::client_id().unwrap_or_else(|| MICROSOFT_CLIENT_ID.into());
        if value.get("renderer").is_none() && value["gpu_batching"].as_bool() == Some(true) {
            config.renderer = RendererMode::Vulkan;
        }
        if config.pack_source == crate::shader_catalog::Origin::Horizon {
            config.pack_source = crate::shader_catalog::Origin::Modrinth;
        }
        let legacy = std::fs::read(config.game_home(home).join("horizon-touch.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .unwrap_or_default();
        config.device.migrate_missing(&value, &legacy);
        config
    }
    pub fn save(&self, home: &Path) -> Result<(), String> {
        let part = home.join("android-launcher.partial");
        std::fs::write(
            &part,
            serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        std::fs::rename(part, home.join("android-launcher.json")).map_err(|e| e.to_string())
    }
    pub fn profile_home(&self, home: &Path) -> PathBuf {
        if self.profile == 0 {
            home.into()
        } else {
            home.join("profiles")
                .join(format!("profile-{}", self.profile))
        }
    }
    pub fn game_home(&self, home: &Path) -> PathBuf {
        // Legacy modpack selections are ignored; their files remain untouched.
        self.profile_home(home)
    }
}
#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Play,
    Account,
    Mods,
    Packs,
    Shaders,
    Installed,
    VoltSettings,
    Settings,
}
impl Page {
    pub const ALL: [Self; 8] = [
        Self::Play,
        Self::Account,
        Self::Mods,
        Self::Packs,
        Self::Shaders,
        Self::Installed,
        Self::VoltSettings,
        Self::Settings,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Play => "Play",
            Self::Account => "Account",
            Self::Mods => "Mods",
            Self::Packs => "Packs",
            Self::Shaders => "Shaders",
            Self::Installed => "Installed",
            Self::VoltSettings => "Volt Settings",
            Self::Settings => "Settings",
        }
    }
    pub fn is_catalog(self) -> bool {
        matches!(self, Self::Packs | Self::Shaders)
    }
}
#[derive(Clone)]
pub struct ProjectIcon {
    pub hash: u64,
    pub rgba: Vec<u8>,
    pub loaded_at: std::time::Instant,
}

enum Reply {
    Done(String, horizon_client::content_files::Kind),
}
pub struct Catalog {
    pub local: Vec<LocalItem>,
    pub sections: [bool; 3],
    pub pending_delete: Option<LocalItem>,
    downloaded: Vec<(browser::SourceId, String)>,
    downloading: Option<(browser::SourceId, String)>,
    progress: Arc<Mutex<f32>>,
    failed: Option<(browser::SourceId, String)>,
    pub page: Page,
    pub query: String,
    pub results: Vec<Project>,
    pub results_born: std::time::Instant,
    pub status: String,
    pub offset: usize,
    pub scroll: f32,
    pub mod_filter: u8,
    pub bundled_zoom_installed: bool,
    pub bundled_zoom_enabled: bool,
    pub mod_born: std::time::Instant,
    pub volt_boost_modal: bool,
    pub volt_modal_scroll: f32,
    pub extras: horizon_client::extras::Extras,
    pub config: Config,
    pub active_controls: Option<horizon_client::touch::layout::ImportedMeta>,
    pub control_presets: Vec<horizon_client::touch::import::PresetEntry>,
    pub editing: Option<&'static str>,
    pub render_scale_text: String,
    pub account_tab: crate::volt_accounts::AccountProvider,
    pub offline_name_input: String,
    pub ely_user_input: String,
    pub ely_pass_input: String,
    pub ely_totp_input: String,
    pub skin_query_input: String,
    icon_cache: std::collections::HashMap<String, Option<ProjectIcon>>,
    icon_pending: std::collections::HashSet<String>,
    icon_tx: mpsc::Sender<(String, Option<(u64, Vec<u8>)>)>,
    icon_rx: mpsc::Receiver<(String, Option<(u64, Vec<u8>)>)>,
    worker: Option<mpsc::Receiver<Result<Reply, String>>>,
    search_worker: Option<mpsc::Receiver<Result<Vec<Project>, String>>>,
    searches: crate::search_queue::Queue<
        (Page, crate::shader_catalog::Origin, String, usize),
        Vec<Project>,
    >,
}
#[derive(Clone)]
pub struct LocalItem {
    pub kind: horizon_client::content_files::Kind,
    pub file: String,
    pub title: String,
    pub enabled: bool,
}
impl Catalog {
    pub fn new(home: &Path) -> Self {
        let config = Config::load(home);
        let (icon_tx, icon_rx) = mpsc::channel();
        let mut result = Self {
            local: vec![],
            sections: [true, true, true],
            pending_delete: None,
            downloaded: vec![],
            downloading: None,
            progress: Arc::new(Mutex::new(0.0)),
            failed: None,
            page: Page::Play,
            query: String::new(),
            results: vec![],
            results_born: std::time::Instant::now(),
            status: String::new(),
            offset: 0,
            scroll: 0.0,
            mod_filter: 0,
            bundled_zoom_installed: false,
            bundled_zoom_enabled: false,
            mod_born: std::time::Instant::now(),
            volt_boost_modal: false,
            volt_modal_scroll: 0.0,
            extras: horizon_client::extras::Extras::default(),
            config,
            active_controls: None,
            control_presets: vec![],
            editing: None,
            render_scale_text: String::new(),
            account_tab: crate::volt_accounts::AccountProvider::Offline,
            offline_name_input: String::new(),
            ely_user_input: String::new(),
            ely_pass_input: String::new(),
            ely_totp_input: String::new(),
            skin_query_input: String::new(),
            icon_cache: std::collections::HashMap::new(),
            icon_pending: std::collections::HashSet::new(),
            icon_tx,
            icon_rx,
            worker: None,
            search_worker: None,
            searches: crate::search_queue::Queue::new(),
        };
        result.refresh(home);
        result
    }
    pub fn show_installed(&mut self, kind: Option<horizon_client::content_files::Kind>) {
        self.searches.cancel_interest();
        self.page = Page::Installed;
        self.scroll = 0.0;
        self.editing = None;
        self.sections = [false; 3];
        if let Some(i) = horizon_client::content_files::Kind::ALL
            .iter()
            .position(|k| Some(*k) == kind)
        {
            self.sections[i] = true;
        }
    }
    pub fn refresh(&mut self, home: &Path) {
        let game = self.config.game_home(home);
        let installed = Installed::load(&game);
        self.downloaded = installed
            .entries
            .iter()
            .filter(|e| game.join(&e.file).is_file())
            .map(|e| (e.source, e.project.clone()))
            .collect();
        let on = horizon_client::mod_list::ModList::load(&game);
        self.bundled_zoom_installed = horizon_client::bundled_native::zoom_installed(&game);
        self.bundled_zoom_enabled = self.bundled_zoom_installed && !on.disabled.contains("zoom");
        let settings = horizon_client::settings::Settings::load(&game.join("options.txt"));
        self.extras = settings.extras.clone();
        horizon_client::volt_mods::sync_catalog_from_engine(&mut self.extras, &settings.horizon);
        horizon_threads::set_thread_boost(self.extras.thread_boost);
        self.local.clear();
        for kind in horizon_client::content_files::Kind::ALL {
            let dir = game.join(kind.folder());
            if std::fs::symlink_metadata(&dir).is_ok_and(|m| m.file_type().is_symlink()) {
                continue;
            }
            for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                if !entry.file_type().is_ok_and(|t| t.is_file()) {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                if !horizon_client::content_files::plain_name(&name) || !kind.accepts(&name) {
                    continue;
                }
                let enabled = match kind {
                    horizon_client::content_files::Kind::Packs => {
                        settings.resource_packs.contains(&format!("file/{name}"))
                    }
                    horizon_client::content_files::Kind::Shaders => {
                        settings.horizon.shader_pack == name
                    }
                    horizon_client::content_files::Kind::Mods => name
                        .strip_suffix(".hmod")
                        .is_some_and(|id| on.enabled.contains(id)),
                };
                self.local.push(LocalItem {
                    kind,
                    file: name.clone(),
                    title: name,
                    enabled,
                });
            }
        }
        self.local.sort_by_key(|e| e.title.to_lowercase());
        self.refresh_controls(home);
    }
    pub fn refresh_controls(&mut self, home: &Path) {
        let game = self.config.game_home(home);
        self.active_controls = horizon_client::touch::import::active_meta(&game);
        self.control_presets = horizon_client::touch::import::list_presets(home);
    }
    pub fn busy(&self) -> bool {
        self.worker.is_some()
    }
    pub fn download_label(&self, project: &Project) -> Option<String> {
        if self
            .downloading
            .as_ref()
            .is_some_and(|(source, id)| *source == project.source && *id == project.id)
        {
            return Some(format!(
                "{:.0}%",
                *self.progress.lock().unwrap_or_else(|e| e.into_inner()) * 100.0
            ));
        }
        if self
            .downloaded
            .iter()
            .any(|(source, id)| *source == project.source && *id == project.id)
            || (project.kind == Kind::Mod
                && self.local.iter().any(|item| {
                    item.kind == horizon_client::content_files::Kind::Mods
                        && item.file == format!("{}.hmod", project.id)
                }))
        {
            return Some("Installed".into());
        }
        None
    }
    pub fn install_button(&self, project: &Project) -> &'static str {
        if self
            .failed
            .as_ref()
            .is_some_and(|(source, id)| *source == project.source && *id == project.id)
        {
            "Retry"
        } else {
            "Install"
        }
    }
    fn job(&mut self, status: &str, job: impl FnOnce() -> Result<Reply, String> + Send + 'static) {
        if self.busy() {
            return;
        }
        self.status = status.into();
        let (tx, rx) = mpsc::channel();
        self.worker = Some(rx);
        if let Err(err) = std::thread::Builder::new()
            .name("volt-catalog-job".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                let _ = tx.send(job());
            })
        {
            self.worker = None;
            self.status = format!("Could not start launcher worker: {err}");
        }
    }
    pub fn poll(&mut self, home: &Path) {
        let reply = self.worker.as_ref().and_then(|rx| match rx.try_recv() {
            Ok(v) => Some(v),
            Err(mpsc::TryRecvError::Disconnected) => Some(Err("Launcher worker stopped".into())),
            Err(_) => None,
        });
        if let Some(reply) = reply {
            self.worker = None;
            let downloading = self.downloading.take();
            if reply.is_err() {
                self.failed = downloading;
            }
            match reply {
                Ok(Reply::Done(s, kind)) => {
                    self.status = s;
                    self.show_installed(Some(kind));
                }
                Err(e) => self.status = e,
            }
            self.refresh(home);
        }
        while let Ok((url, decoded)) = self.icon_rx.try_recv() {
            self.icon_pending.remove(&url);
            let entry = decoded.map(|(hash, rgba)| ProjectIcon {
                hash,
                rgba,
                loaded_at: std::time::Instant::now(),
            });
            self.icon_cache.insert(url, entry);
        }
        let result = self
            .search_worker
            .as_ref()
            .and_then(|rx| match rx.try_recv() {
                Ok(v) => Some(v),
                Err(mpsc::TryRecvError::Disconnected) => Some(Err("Search worker stopped".into())),
                Err(_) => None,
            });
        if let Some(result) = result {
            self.search_worker = None;
            let visible = self.searches.finish(result.as_ref().ok().cloned());
            if visible && self.page.is_catalog() && !self.busy() {
                match result {
                    Ok(items) => self.show_results(items, home),
                    Err(error) => self.status = error,
                }
            }
        }
        if self.page.is_catalog() && !self.results.is_empty() {
            self.queue_result_icons(home);
        }
        self.start_search();
    }
    pub fn search(&mut self, home: &Path) {
        self.request_search(home, true);
    }
    pub fn browse(&mut self, home: &Path) {
        self.searches.cancel_interest();
        if self.page.is_catalog() {
            self.request_search(home, false);
        }
    }
    fn request_search(&mut self, home: &Path, refresh: bool) {
        if !self.page.is_catalog() {
            return;
        }
        if self.page == Page::Mods && self.query.trim().starts_with("https://") {
            self.searches.cancel_interest();
            if refresh {
                self.import_native(home);
            } else {
                self.results.clear();
                self.status = "Press Import .hmod to download this HTTPS link".into();
            }
            return;
        }
        self.scroll = 0.0;
        let origin = match self.page {
            Page::Packs => self.config.pack_source,
            Page::Shaders => self.config.shader_source,
            Page::Mods => self.config.mod_source,
            _ => crate::shader_catalog::Origin::Horizon,
        };
        let key = (self.page, origin, self.query.trim().to_owned(), self.offset);
        if let Some(items) = self.searches.request(key, refresh) {
            self.show_results(items, home);
        } else {
            self.results.clear();
            self.status = "Searching... You can switch tabs while results load.".into();
        }
        self.start_search();
    }
    fn show_results(&mut self, items: Vec<Project>, home: &Path) {
        use crate::shader_catalog::Origin;
        self.status = if self.page == Page::Shaders
            && items.is_empty()
            && self.config.shader_source == Origin::Horizon
        {
            "No Horizon shader packs match this search. Try another query or import a .hshader."
                .into()
        } else if self.page == Page::Mods && self.config.mod_source != Origin::Horizon {
            format!(
                "{} Java mods ({}) - browsing; this Rust client runs native .hmod mods",
                items.len(),
                self.config.mod_source.label()
            )
        } else if self.page == Page::Mods {
            let native = items.iter().filter(|p| source_only_mod(p)).count();
            format!(
                "{} .hmod / {} source-only .hnmod (not supported yet)",
                items.len() - native,
                native
            )
        } else {
            let active_origin = match self.page {
                Page::Packs => self.config.pack_source,
                Page::Shaders => self.config.shader_source,
                _ => Origin::Modrinth,
            };
            format!(
                "{} {} results - select Install to download",
                items.len(),
                active_origin.label()
            )
        };
        self.results = items;
        self.results_born = std::time::Instant::now();
        self.queue_result_icons(home);
    }
    pub fn project_icon(&self, project: &Project) -> Option<&ProjectIcon> {
        let url = project.icon.as_deref().filter(|u| !u.is_empty())?;
        self.icon_cache.get(url).and_then(|entry| entry.as_ref())
    }
    pub fn animating_icons_or_cards(&self) -> bool {
        if !self.icon_pending.is_empty() || self.search_worker.is_some() || self.worker.is_some() {
            return true;
        }
        if self.page.is_catalog() && !self.results.is_empty() {
            if self.results_born.elapsed().as_secs_f32() < 0.65 {
                return true;
            }
            for p in &self.results {
                if let Some(icon) = self.project_icon(p)
                    && icon.loaded_at.elapsed().as_secs_f32() < 0.30
                {
                    return true;
                }
            }
        }
        false
    }
    fn queue_result_icons(&mut self, home: &Path) {
        if self.icon_cache.len() > 256 {
            self.icon_cache.clear();
        }
        let urls: Vec<String> = self
            .results
            .iter()
            .filter_map(|p| p.icon.as_deref())
            .filter(|u| !u.is_empty())
            .map(str::to_owned)
            .collect();
        for url in urls {
            if self.icon_pending.len() >= 6 {
                break;
            }
            if self.icon_cache.contains_key(&url) || self.icon_pending.contains(&url) {
                continue;
            }
            let url_key = url.clone();
            let tx = self.icon_tx.clone();
            let home_buf = home.to_path_buf();
            if std::thread::Builder::new()
                .name("volt-icon-fetch".into())
                .stack_size(4 * 1024 * 1024)
                .spawn(move || {
                    let decoded = fetch_and_round_icon(&home_buf, &url);
                    let _ = tx.send((url, decoded));
                })
                .is_ok()
            {
                self.icon_pending.insert(url_key);
            }
        }
    }
    fn start_search(&mut self) {
        if self.busy() || self.search_worker.is_some() {
            return;
        }
        let Some((page, origin, text, offset)) = self.searches.take_next() else {
            return;
        };
        let (tx, rx) = mpsc::channel();
        self.search_worker = Some(rx);
        if let Err(err) = std::thread::Builder::new()
            .name("volt-catalog-search".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                let result = run_catalog_query(page, origin, &text, offset);
                let _ = tx.send(result);
            })
        {
            self.search_worker = None;
            self.status = format!("Search unavailable: {err}");
        }
    }
    pub fn import_native(&mut self, home: &Path) {
        let url = self.query.trim().to_owned();
        let game = self.config.game_home(home);
        self.job("Downloading sandboxed .hmod over HTTPS...",move||{
            use std::io::Read;
            let mut bytes=Vec::new();
            ureq::AgentBuilder::new().https_only(true).timeout_connect(std::time::Duration::from_secs(20)).timeout_read(std::time::Duration::from_secs(90)).build().get(&url).call().map_err(|e|e.to_string())?.into_reader().take(32*1024*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
            if bytes.len()>32*1024*1024 {return Err("Horizon mod exceeds 32 MiB".into());}
            let dir=game.join("horizon-mods");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
            let stage=dir.join("import.partial");std::fs::write(&stage,bytes).map_err(|e|e.to_string())?;
            let result=(||{
                let package=horizon_mods::Package::read(&stage)?;
                let destination=dir.join(format!("{}.hmod",package.manifest.id));
                if destination.exists(){std::fs::copy(&destination,destination.with_extension("hmod.previous")).map_err(|e|e.to_string())?;}
                std::fs::rename(&stage,destination).map_err(|e|e.to_string())?;
                Ok(Reply::Done(format!("Installed {}. Enable it in Installed after reviewing its capabilities in game Mods.",package.manifest.name), horizon_client::content_files::Kind::Mods))
            })();
            let _=std::fs::remove_file(stage);result
        });
    }
    pub fn install(&mut self, home: &Path, index: usize) {
        let Some(project) = self.results.get(index).cloned() else {
            return;
        };
        if source_only_mod(&project) {
            self.status = "This .hnmod needs a separately compiled client. Android support is planned; sandboxed .hmod mods can be installed now.".into();
            return;
        }
        if self.busy() || self.download_label(&project).is_some() {
            return;
        }
        let game = self.config.game_home(home);
        if project.kind != Kind::ResourcePack
            && !(project.kind == Kind::Shader
                && matches!(
                    project.source,
                    browser::SourceId::Horizon
                        | browser::SourceId::Modrinth
                        | browser::SourceId::CurseForge
                ))
            && !(cfg!(target_pointer_width = "64")
                && project.kind == Kind::Mod
                && project.source == browser::SourceId::Horizon)
        {
            return;
        }
        self.downloading = Some((project.source, project.id.clone()));
        self.failed = None;
        *self.progress.lock().unwrap_or_else(|e| e.into_inner()) = 0.0;
        let progress = self.progress.clone();
        let status = if project.kind == Kind::Shader
            && matches!(
                project.source,
                browser::SourceId::Modrinth | browser::SourceId::CurseForge
            ) {
            "Downloading shader pack ZIP..."
        } else {
            "Checking compatibility and downloading..."
        };
        self.job(status, move || {
            std::fs::create_dir_all(&game).map_err(|e| e.to_string())?;
            let source: Box<dyn Source> = if project.source == browser::SourceId::CurseForge {
                Box::new(browser::curseforge::CurseForge::volt())
            } else {
                browser::source(project.source).ok_or("Unsupported download source")?
            };
            let count = if project.kind == Kind::Shader
                && matches!(
                    project.source,
                    browser::SourceId::Modrinth | browser::SourceId::CurseForge
                )
            {
                crate::shader_catalog::install(source.as_ref(), &project, &game, &|_name, p| {
                    if p.is_finite() {
                        *progress.lock().unwrap_or_else(|e| e.into_inner()) = p.clamp(0.0, 1.0);
                    }
                })?
            } else {
                browser::installed::install(source.as_ref(), &project, true, &game, &|_name, p| {
                    if p.is_finite() {
                        *progress.lock().unwrap_or_else(|e| e.into_inner()) = p.clamp(0.0, 1.0);
                    }
                })?
                .len()
            };
            Ok(Reply::Done(
                format!("Installed {count} item(s). Open Play to load this profile."),
                match project.kind {
                    Kind::Mod => horizon_client::content_files::Kind::Mods,
                    Kind::Shader => horizon_client::content_files::Kind::Shaders,
                    _ => horizon_client::content_files::Kind::Packs,
                },
            ))
        });
    }
    pub fn toggle_local(&mut self, home: &Path, index: usize) {
        let Some(item) = self.local.get(index).cloned() else {
            return;
        };
        let game = self.config.game_home(home);
        let path = game.join("options.txt");
        let mut settings = horizon_client::settings::Settings::load(&path);
        match item.kind {
            horizon_client::content_files::Kind::Mods => {
                if !cfg!(target_pointer_width = "64") {
                    self.status = "Sandboxed code mods require a 64-bit APK".into();
                    return;
                }
                let mut on = horizon_client::mod_list::ModList::load(&game);
                if let Some(id) = item.file.strip_suffix(".hmod") {
                    if item.enabled {
                        on.enabled.remove(id);
                    } else {
                        on.enabled.insert(id.to_owned());
                    }
                }
                on.save(&game);
            }
            horizon_client::content_files::Kind::Packs => {
                let id = format!("file/{}", item.file);
                if item.enabled {
                    settings.resource_packs.retain(|p| p != &id);
                } else {
                    settings.resource_packs.push(id);
                }
                settings.save(&path);
            }
            horizon_client::content_files::Kind::Shaders => {
                settings.horizon.shader_pack = if item.enabled {
                    String::new()
                } else {
                    item.file
                };
                settings.save(&path);
            }
        }
        self.status = "Selection saved for the next game launch".into();
        self.refresh(home);
    }
    pub fn remove_local(&mut self, home: &Path) {
        let Some(item) = self.pending_delete.take() else {
            return;
        };
        let game = self.config.game_home(home);
        let result = (|| -> Result<(), String> {
            if !horizon_client::content_files::plain_name(&item.file) {
                return Err("Invalid content filename".into());
            }
            let dir = game.join(item.kind.folder());
            if std::fs::symlink_metadata(&dir)
                .map_err(|e| e.to_string())?
                .file_type()
                .is_symlink()
            {
                return Err("Refusing a redirected content directory".into());
            }
            let target = dir.join(&item.file);
            let meta = std::fs::symlink_metadata(&target).map_err(|e| e.to_string())?;
            if !meta.is_file() || meta.file_type().is_symlink() {
                return Err(
                    "Only an explicitly selected regular add-on file can be removed".into(),
                );
            }
            std::fs::remove_file(target).map_err(|e| e.to_string())?;
            let relative = format!("{}/{}", item.kind.folder(), item.file);
            let mut installed = Installed::load(&game);
            installed.entries.retain(|e| e.file != relative);
            installed.save(&game)?;
            let path = game.join("options.txt");
            let mut settings = horizon_client::settings::Settings::load(&path);
            if item.kind == horizon_client::content_files::Kind::Packs {
                settings
                    .resource_packs
                    .retain(|p| p != &format!("file/{}", item.file));
            }
            if item.kind == horizon_client::content_files::Kind::Shaders
                && settings.horizon.shader_pack == item.file
            {
                settings.horizon.shader_pack.clear();
            }
            settings.save(&path);
            if item.kind == horizon_client::content_files::Kind::Mods {
                let mut on = horizon_client::mod_list::ModList::load(&game);
                if let Some(id) = item.file.strip_suffix(".hmod") {
                    on.enabled.remove(id);
                }
                on.save(&game);
            }
            Ok(())
        })();
        self.status = match result {
            Ok(()) => format!("Removed {}", item.title),
            Err(e) => e,
        };
        self.refresh(home);
    }
    pub fn edit(&mut self, out: horizon_client::osk::Output, home: &Path) {
        use horizon_client::osk::Output;
        let field = match self.editing {
            Some("name") => &mut self.config.name,
            Some("server") => &mut self.config.server,
            Some("render_scale") => &mut self.render_scale_text,
            Some("client_id") => &mut self.config.microsoft_client_id,
            Some("offline_name") => &mut self.offline_name_input,
            Some("ely_user") => &mut self.ely_user_input,
            Some("ely_pass") => &mut self.ely_pass_input,
            Some("ely_totp") => &mut self.ely_totp_input,
            Some("skin_query") => &mut self.skin_query_input,
            _ => &mut self.query,
        };
        match out {
            Output::Text(s) => field.push_str(&s),
            Output::Backspace => {
                field.pop();
            }
            Output::Enter | Output::Close => {
                if self.editing == Some("render_scale") {
                    match crate::display::RenderScale::parse(&self.render_scale_text) {
                        Ok(scale) => {
                            self.config.render_scale = scale;
                            crate::display::set_scale(scale);
                        }
                        Err(error) => {
                            self.status = error.into();
                            return;
                        }
                    }
                }
                self.editing = None;
                let _ = self.config.save(home);
                if self.page.is_catalog() {
                    self.offset = 0;
                    self.search(home);
                }
            }
        }
    }
}

fn run_single_source_query(
    page: Page,
    origin: crate::shader_catalog::Origin,
    text: &str,
    offset: usize,
    limit: usize,
) -> Result<Vec<Project>, String> {
    use crate::shader_catalog::Origin;
    let kind = match page {
        Page::Mods => Kind::Mod,
        Page::Shaders => Kind::Shader,
        _ => Kind::ResourcePack,
    };
    let mut query = Query::new(text, kind);
    query.offset = offset;
    query.limit = limit;
    match origin {
        Origin::Horizon => browser::horizon::Registry.search(&query),
        Origin::Modrinth => {
            query.everything = true;
            if page == Page::Shaders {
                browser::modrinth::search_shader_packs(&query)
            } else {
                browser::modrinth::Modrinth.search(&query)
            }
        }
        Origin::CurseForge => {
            query.everything = true;
            let cf = browser::curseforge::CurseForge::volt();
            if page == Page::Shaders {
                cf.search_shader_packs(&query)
            } else {
                cf.search(&query)
            }
        }
        Origin::Both => Ok(Vec::new()),
    }
}

fn run_catalog_query(
    page: Page,
    origin: crate::shader_catalog::Origin,
    text: &str,
    offset: usize,
) -> Result<Vec<Project>, String> {
    use crate::shader_catalog::Origin;
    if origin != Origin::Both {
        return run_single_source_query(page, origin, text, offset, 12);
    }
    let half_offset = offset / 2;
    let (mr_res, cf_res) = std::thread::scope(|s| {
        let mr = s.spawn(|| run_single_source_query(page, Origin::Modrinth, text, half_offset, 8));
        let cf =
            s.spawn(|| run_single_source_query(page, Origin::CurseForge, text, half_offset, 8));
        (
            mr.join()
                .unwrap_or_else(|_| Err("Modrinth worker panicked".into())),
            cf.join()
                .unwrap_or_else(|_| Err("CurseForge worker panicked".into())),
        )
    });
    match (mr_res, cf_res) {
        (Ok(mr), Ok(cf)) => {
            let mut merged = Vec::with_capacity(12);
            let max_len = mr.len().max(cf.len());
            for idx in 0..max_len {
                if let Some(item) = mr.get(idx)
                    && merged.len() < 12
                {
                    merged.push(item.clone());
                }
                if let Some(item) = cf.get(idx)
                    && merged.len() < 12
                {
                    merged.push(item.clone());
                }
                if merged.len() >= 12 {
                    break;
                }
            }
            Ok(merged)
        }
        (Ok(mut mr), Err(_)) => {
            mr.truncate(12);
            Ok(mr)
        }
        (Err(_), Ok(mut cf)) => {
            cf.truncate(12);
            Ok(cf)
        }
        (Err(e1), Err(e2)) => Err(format!("Modrinth: {e1} | CurseForge: {e2}")),
    }
}

fn fetch_and_round_icon(home: &Path, url: &str) -> Option<(u64, Vec<u8>)> {
    let decoded = if let Some(cached_path) = browser::images::cached(home, url, 80)
        .or_else(|| browser::images::fetch(home, url, 80).ok())
    {
        image::open(cached_path).ok()?
    } else if url.starts_with("https://") {
        use std::io::Read as _;
        let mut bytes = Vec::new();
        ureq::AgentBuilder::new()
            .https_only(true)
            .user_agent(&format!(
                "VoltLauncher/{} (github.com/entitybrian69-bit/Volt-launcher)",
                crate::backend::VERSION
            ))
            .timeout_connect(std::time::Duration::from_secs(12))
            .timeout_read(std::time::Duration::from_secs(20))
            .build()
            .get(url)
            .call()
            .ok()?
            .into_reader()
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .ok()?;
        if bytes.is_empty() || bytes.len() > 4 * 1024 * 1024 {
            return None;
        }
        let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .ok()?;
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(2048);
        limits.max_image_height = Some(2048);
        limits.max_alloc = Some(32 << 20);
        reader.limits(limits);
        reader.decode().ok()?
    } else {
        return None;
    };

    let resized = decoded
        .resize_exact(64, 64, image::imageops::FilterType::Triangle)
        .to_rgba8();
    let mut rgba = resized.into_raw();

    // 4x4 supersampled rounded-corner mask (radius = 10.5 out of 64.0) for crisp anti-aliased edges.
    let radius = 10.5f32;
    let r2 = radius * radius;
    for y in 0..64usize {
        for x in 0..64usize {
            let mut inside = 0u32;
            for sy in 0..4usize {
                for sx in 0..4usize {
                    let px = x as f32 + (sx as f32 + 0.5) * 0.25;
                    let py = y as f32 + (sy as f32 + 0.5) * 0.25;
                    let cx = if px < radius {
                        radius - px
                    } else if px > 64.0 - radius {
                        px - (64.0 - radius)
                    } else {
                        0.0
                    };
                    let cy = if py < radius {
                        radius - py
                    } else if py > 64.0 - radius {
                        py - (64.0 - radius)
                    } else {
                        0.0
                    };
                    if cx * cx + cy * cy <= r2 {
                        inside += 1;
                    }
                }
            }
            if inside < 16 {
                let idx = (y * 64 + x) * 4 + 3;
                rgba[idx] = ((u32::from(rgba[idx]) * inside + 8) / 16) as u8;
            }
        }
    }
    let hash = crate::skin_preview::fnv1a(&rgba);
    Some((hash, rgba))
}
