//! Profile-scoped view models for the original Volt captures and replay pages.
use crate::{
    pages::{Action, PhotoView, replay_cover_path},
    photo_cache,
};
use horizon_client::{Result, replay};
use std::path::PathBuf;
include!("../../horizon-client/src/launcher_ui/captures.rs");
pub(super) struct Media {
    pub home: PathBuf,
    pub replay_tab: bool,
    pub shots: Vec<String>,
    pub clips: Vec<(String, u64)>,
    pub replays: Vec<replay::Meta>,
    pub view: Option<PhotoView>,
    pub scroll: f32,
    remove: Option<PathBuf>,
}
impl Media {
    pub fn new(home: PathBuf, replay_tab: bool) -> Self {
        let (shots, clips) = capture_files(&home);
        let replays = replay::list(&home.join("replays"));
        Self {
            home,
            replay_tab,
            shots,
            clips,
            replays,
            view: None,
            scroll: 0.,
            remove: None,
        }
    }
    pub fn action(&mut self, a: Action) -> Result<Option<String>> {
        let pair = match a {
            Action::ViewScreenshot(i) => self
                .shots
                .get(i)
                .map(|n| (n.clone(), self.home.join("screenshots").join(n))),
            Action::ViewReplayCover(i) => self
                .replays
                .get(i)
                .map(|r| (r.file_name.clone(), replay_cover_path(&r.path))),
            Action::OpenCaptures => {
                let dir = self.home.join("recordings");
                std::fs::create_dir_all(&dir)?;
                open::that(dir)?;
                None
            }
            Action::DeleteReplay(i) => {
                self.remove = self.replays.get(i).map(|r| r.path.clone());
                return Ok(self
                    .remove
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned()));
            }
            Action::DeleteRecording(i) => {
                self.remove = self
                    .clips
                    .get(i)
                    .map(|(n, _)| self.home.join("recordings").join(n));
                return Ok(self.clips.get(i).map(|(n, _)| n.clone()));
            }
            Action::ConfirmRemove => {
                if let Some(p) = self.remove.take() {
                    std::fs::remove_file(p)?;
                    *self = Self::new(self.home.clone(), self.replay_tab);
                }
                None
            }
            _ => None,
        };
        if let Some((title, path)) = pair {
            self.view = Some(PhotoView {
                title,
                key: photo_cache::file_key(&path),
                source: photo_cache::Source::File(path),
            });
        }
        Ok(None)
    }
}
