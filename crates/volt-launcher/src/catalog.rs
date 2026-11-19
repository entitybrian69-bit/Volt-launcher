//! Profile-scoped Windows browser for the original Horizon registry. No native
//! source compilation, Java execution or unverified fallback downloads.
pub(super) use crate::registry::Entry;
use crate::registry::{install_from, rows};
use horizon_mods::index::{Index, Kind};
use std::{io::Read, path::PathBuf, sync::mpsc, thread, time::Duration};
const INDEX: &str = "https://horizon-rust.github.io/horizon-mods/index.json";
const MAX_INDEX: u64 = 16 << 20;
fn get(url: &str) -> Result<Box<dyn Read>, String> {
    let agent = crate::proxy::agent(
        ureq::AgentBuilder::new()
            .https_only(true)
            .timeout(Duration::from_secs(90))
            .user_agent("VoltLauncher/0.1 (luciferstudios/Volt-launcherPc)"),
        url,
    )
    .build();
    agent
        .get(url)
        .call()
        .map(|r| r.into_reader() as Box<dyn Read>)
        .map_err(|e| format!("Registry request failed: {e}"))
}
fn fetch() -> Result<Index, String> {
    let mut text = String::new();
    get(INDEX)?
        .take(MAX_INDEX + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() as u64 > MAX_INDEX {
        return Err("Oversized registry index".into());
    }
    Index::parse(&text)
}
enum Reply {
    List(Vec<Entry>),
    Installed(String),
}
pub(super) struct Catalog {
    home: PathBuf,
    kind: Kind,
    pub label: &'static str,
    pub rows: Vec<Entry>,
    pub status: String,
    pub scroll: f32,
    pending: Option<mpsc::Receiver<Result<Reply, String>>>,
}
impl Catalog {
    pub fn new(home: PathBuf, page: crate::pages::Page) -> Self {
        use crate::pages::Page;
        let (kind, label) = match page {
            Page::Shaders => (Kind::Shader, "Shader packs"),
            Page::Packs => (Kind::ResourcePack, "Resource packs"),
            _ => (Kind::Mod, "Mods"),
        };
        Self {
            home,
            kind,
            label,
            rows: vec![],
            status: "Refresh to browse compatible Horizon registry projects.".into(),
            scroll: 0.,
            pending: None,
        }
    }
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }
    pub fn request(&mut self, i: Option<usize>) {
        if self.busy() {
            return;
        }
        let entry = match i {
            Some(i) => match self.rows.get(i) {
                Some(row) => Some(row.clone()),
                None => return,
            },
            None => None,
        };
        let (home, kind) = (self.home.clone(), self.kind.clone());
        let (tx, rx) = mpsc::channel();
        self.pending = Some(rx);
        self.status = if entry.is_some() {
            "Verifying download…"
        } else {
            "Loading registry…"
        }
        .into();
        thread::spawn(move || {
            let _ = tx.send(match entry {
                None => fetch().map(|idx| Reply::List(rows(idx, &kind))),
                Some(e) => fetch()
                    .and_then(|idx| install_from(&idx, &home, &kind, e, get))
                    .map(Reply::Installed),
            });
        });
    }
    pub fn poll(&mut self) {
        let Some(rx) = &self.pending else {
            return;
        };
        let result = match rx.try_recv() {
            Ok(r) => r,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(_) => Err("Registry worker stopped".into()),
        };
        self.pending = None;
        match result {
            Ok(Reply::List(r)) => {
                self.rows = r;
                self.scroll = 0.;
                self.status = format!("{} compatible projects", self.rows.len());
            }
            Ok(Reply::Installed(s)) | Err(s) => self.status = s,
        }
    }
}
