//! One-time ABI-matched Volt Launcher update reminder & PackageInstaller downloader.
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

const RELEASES_LIST_URL: &str =
    "https://api.github.com/repos/entitybrian69-bit/Volt-launcher/releases?per_page=5";
const RELEASES_LATEST_URL: &str =
    "https://api.github.com/repos/entitybrian69-bit/Volt-launcher/releases/latest";
const SEEN_FILE: &str = "volt-seen-updates.json";
const MAX_APK_BYTES: u64 = 256 * 1024 * 1024;

static INSTALLER_STATUS: OnceLock<Mutex<Option<String>>> = OnceLock::new();

pub fn note_installer_result(code: &str) {
    let slot = INSTALLER_STATUS.get_or_init(|| Mutex::new(None));
    let msg = if code == "OK" {
        "Android package update prompt opened.".to_owned()
    } else if code == "ALLOW_SOURCE" {
        "Enable 'Allow from this source' — update will resume automatically.".to_owned()
    } else if let Some(err) = code.strip_prefix("ERR:") {
        format!("Could not start APK update: {err}")
    } else {
        code.to_owned()
    };
    *slot.lock().unwrap_or_else(|e| e.into_inner()) = Some(msg);
}

pub fn take_installer_status() -> Option<String> {
    let slot = INSTALLER_STATUS.get()?;
    slot.lock().unwrap_or_else(|e| e.into_inner()).take()
}

#[derive(Clone, Debug)]
pub struct UpdateInfo {
    pub tag_name: String,
    pub display_version: String,
    pub abi: String,
    pub abi_label: String,
    pub asset_name: String,
    pub download_url: String,
    pub size_bytes: u64,
    pub whats_new: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
}

pub enum UpdateModalState {
    Ready,
    Downloading {
        progress: Arc<Mutex<DownloadProgress>>,
        cancel: Arc<AtomicBool>,
        done_rx: mpsc::Receiver<Result<PathBuf, String>>,
    },
    Error(String),
}

pub struct UpdatePrompt {
    pub info: UpdateInfo,
    pub state: UpdateModalState,
    pub scroll: f32,
}

#[derive(Default, Serialize, Deserialize)]
struct SeenStore {
    #[serde(default)]
    seen_tags: Vec<String>,
}

impl SeenStore {
    fn load(home: &Path) -> Self {
        std::fs::read(home.join(SEEN_FILE))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn contains(&self, tag: &str) -> bool {
        self.seen_tags.iter().any(|t| t == tag)
    }

    fn mark_seen(&mut self, home: &Path, tag: &str) {
        if !self.contains(tag) {
            self.seen_tags.push(tag.to_owned());
            if self.seen_tags.len() > 64 {
                let drop_n = self.seen_tags.len() - 64;
                self.seen_tags.drain(0..drop_n);
            }
        }
        let part = home.join("volt-seen-updates.part");
        if let Ok(bytes) = serde_json::to_vec_pretty(self)
            && std::fs::write(&part, bytes).is_ok()
        {
            let _ = std::fs::rename(&part, home.join(SEEN_FILE));
        }
    }
}

pub fn mark_update_seen(home: &Path, tag: &str) {
    let mut store = SeenStore::load(home);
    store.mark_seen(home, tag);
}

pub fn select_device_abi(supported_abis: &[String]) -> (&'static str, &'static str) {
    if supported_abis
        .iter()
        .any(|a| a.eq_ignore_ascii_case("arm64-v8a"))
    {
        ("arm64-v8a", "64-bit ARM64")
    } else if supported_abis
        .iter()
        .any(|a| a.eq_ignore_ascii_case("armeabi-v7a") || a.eq_ignore_ascii_case("armeabi"))
    {
        ("armeabi-v7a", "32-bit ARMv7")
    } else if cfg!(target_arch = "aarch64") {
        ("arm64-v8a", "64-bit ARM64")
    } else {
        ("armeabi-v7a", "32-bit ARMv7")
    }
}

pub fn parse_semver(raw: &str) -> Option<(u32, u32, u32)> {
    let trimmed = raw.trim().trim_start_matches(['v', 'V']);
    let core = trimmed.split(['-', '+', ' ']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u32>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u32>().ok()?;
    Some((major, minor, patch))
}

fn tag_commit_suffix(tag: &str) -> Option<&str> {
    let (_, sha) = tag.rsplit_once("-build.")?;
    let clean = sha.trim();
    (!clean.is_empty()).then_some(clean)
}

pub fn is_newer_release(
    installed_version: &str,
    installed_commit: Option<&str>,
    remote_tag: &str,
) -> bool {
    let Some(inst) = parse_semver(installed_version) else {
        return false;
    };
    let Some(rem) = parse_semver(remote_tag) else {
        return false;
    };
    if rem > inst {
        return true;
    }
    if rem < inst {
        return false;
    }
    if let (Some(local_sha), Some(remote_sha)) = (
        installed_commit.map(str::trim).filter(|s| s.len() >= 7),
        tag_commit_suffix(remote_tag).filter(|s| s.len() >= 7),
    ) {
        return !local_sha.starts_with(remote_sha) && !remote_sha.starts_with(local_sha);
    }
    false
}

pub fn format_whats_new(body: &str, fallback_title: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw_line in body.lines() {
        let mut line = raw_line.trim();
        if line.is_empty() || line.starts_with("---") || line.starts_with("```") {
            continue;
        }
        while let Some(rest) = line.strip_prefix('#') {
            line = rest.trim_start();
        }
        let cleaned = line.replace("**", "").replace('`', "");
        let cleaned = cleaned.trim();
        if cleaned.is_empty() {
            continue;
        }
        if let Some(bullet) = cleaned
            .strip_prefix("- ")
            .or_else(|| cleaned.strip_prefix("* "))
            .or_else(|| cleaned.strip_prefix("• "))
        {
            out.push(format!("• {}", bullet.trim()));
        } else {
            out.push(cleaned.to_owned());
        }
        if out.len() >= 18 {
            break;
        }
    }
    if out.is_empty() {
        out.push(format!("• {fallback_title}"));
        out.push("• Performance, compatibility, and stability improvements.".into());
    }
    out
}

pub fn spawn_startup_check(home: PathBuf) -> mpsc::Receiver<UpdateInfo> {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("volt-update-check".into())
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(900));
            if let Some(info) = check_once_blocking(&home) {
                let _ = tx.send(info);
            }
        })
        .ok();
    rx
}

fn check_once_blocking(home: &Path) -> Option<UpdateInfo> {
    let mut seen = SeenStore::load(home);
    let abis = crate::native_ime::device_supported_abis();
    let (target_abi, abi_label) = select_device_abi(&abis);
    let installed_ver = crate::native_ime::app_version_name();
    let installed_commit = option_env!("HORIZON_ANDROID_BUILD_COMMIT");

    let agent = ureq::AgentBuilder::new()
        .https_only(true)
        .user_agent(&format!(
            "VoltLauncher/{} (Android; {})",
            crate::backend::VERSION,
            target_abi
        ))
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(Duration::from_secs(12))
        .build();

    let releases = fetch_releases(&agent)?;
    for rel in releases {
        if rel.get("draft").and_then(|v| v.as_bool()) == Some(true) {
            continue;
        }
        let Some(tag) = rel.get("tag_name").and_then(|v| v.as_str()) else {
            continue;
        };
        if !is_newer_release(&installed_ver, installed_commit, tag) {
            continue;
        }
        if seen.contains(tag) {
            return None;
        }
        let Some(assets) = rel.get("assets").and_then(|v| v.as_array()) else {
            continue;
        };
        let matched = assets.iter().find_map(|asset| {
            let name = asset.get("name")?.as_str()?;
            let lower = name.to_ascii_lowercase();
            if !lower.ends_with(".apk") || !lower.contains(target_abi) {
                return None;
            }
            let url = asset.get("browser_download_url")?.as_str()?;
            if !url.starts_with("https://") {
                return None;
            }
            let size = asset.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
            Some((name.to_owned(), url.to_owned(), size))
        });
        let Some((asset_name, download_url, size_bytes)) = matched else {
            continue;
        };
        let title = rel
            .get("name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(tag);
        let body = rel.get("body").and_then(|v| v.as_str()).unwrap_or("");
        let display_version = parse_semver(tag)
            .map(|(ma, mi, pa)| format!("v{ma}.{mi}.{pa}"))
            .unwrap_or_else(|| tag.to_owned());

        // Persist immediately so this release reminder only appears ONCE and never nags again!
        seen.mark_seen(home, tag);

        return Some(UpdateInfo {
            tag_name: tag.to_owned(),
            display_version,
            abi: target_abi.to_owned(),
            abi_label: abi_label.to_owned(),
            asset_name,
            download_url,
            size_bytes,
            whats_new: format_whats_new(body, title),
        });
    }
    None
}

fn fetch_releases(agent: &ureq::Agent) -> Option<Vec<serde_json::Value>> {
    if let Ok(resp) = agent
        .get(RELEASES_LIST_URL)
        .set("Accept", "application/vnd.github+json")
        .call()
    {
        let mut buf = Vec::new();
        if resp
            .into_reader()
            .take(512 * 1024)
            .read_to_end(&mut buf)
            .is_ok()
            && let Ok(serde_json::Value::Array(arr)) = serde_json::from_slice(&buf)
            && !arr.is_empty()
        {
            return Some(arr);
        }
    }
    let resp = agent
        .get(RELEASES_LATEST_URL)
        .set("Accept", "application/vnd.github+json")
        .call()
        .ok()?;
    let mut buf = Vec::new();
    resp.into_reader()
        .take(256 * 1024)
        .read_to_end(&mut buf)
        .ok()?;
    let val: serde_json::Value = serde_json::from_slice(&buf).ok()?;
    Some(vec![val])
}

pub fn begin_download(home: &Path, info: &UpdateInfo) -> UpdateModalState {
    mark_update_seen(home, &info.tag_name);
    let progress = Arc::new(Mutex::new(DownloadProgress {
        downloaded: 0,
        total: info.size_bytes,
    }));
    let cancel = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();
    let dl_url = info.download_url.clone();
    let abi = info.abi.clone();
    let updates_dir = home.join("updates");
    let prog_clone = Arc::clone(&progress);
    let cancel_clone = Arc::clone(&cancel);

    let spawned = std::thread::Builder::new()
        .name("volt-apk-download".into())
        .spawn(move || {
            let res = download_apk_blocking(&updates_dir, &abi, &dl_url, &prog_clone, &cancel_clone);
            let _ = tx.send(res);
        });
    if let Err(err) = spawned {
        return UpdateModalState::Error(format!("Could not start download thread: {err}"));
    }
    UpdateModalState::Downloading {
        progress,
        cancel,
        done_rx: rx,
    }
}

fn download_apk_blocking(
    updates_dir: &Path,
    abi: &str,
    url: &str,
    progress: &Mutex<DownloadProgress>,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(updates_dir).map_err(|e| format!("Updates folder error: {e}"))?;
    if let Ok(entries) = std::fs::read_dir(updates_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                let _ = std::fs::remove_file(p);
            }
        }
    }
    let stage = updates_dir.join(format!("volt-update-{abi}.apk.part"));
    let dest = updates_dir.join(format!("volt-update-{abi}.apk"));

    let agent = ureq::AgentBuilder::new()
        .https_only(true)
        .user_agent(&format!(
            "VoltLauncher/{} (Android; {abi})",
            crate::backend::VERSION
        ))
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(120))
        .build();

    let resp = agent
        .get(url)
        .call()
        .map_err(|e| format!("Download request failed: {e}"))?;
    if let Some(len) = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        && len > 0
    {
        progress.lock().unwrap_or_else(|e| e.into_inner()).total = len;
    }

    let mut reader = resp.into_reader();
    let mut out = std::fs::File::create(&stage).map_err(|e| format!("Write error: {e}"))?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut total_written = 0u64;
    let mut magic = [0u8; 4];

    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = std::fs::remove_file(&stage);
            return Err("CANCELLED".into());
        }
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("Network read error: {e}"))?;
        if n == 0 {
            break;
        }
        if total_written < 4 {
            let start = total_written as usize;
            let copy_len = (4 - start).min(n);
            magic[start..start + copy_len].copy_from_slice(&buf[..copy_len]);
        }
        total_written += n as u64;
        if total_written > MAX_APK_BYTES {
            let _ = std::fs::remove_file(&stage);
            return Err("Downloaded APK exceeds 256 MiB safety limit.".into());
        }
        out.write_all(&buf[..n])
            .map_err(|e| format!("Disk write error: {e}"))?;
        progress
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .downloaded = total_written;
    }
    out.sync_all()
        .map_err(|e| format!("Disk sync error: {e}"))?;
    drop(out);

    if total_written < 1024 || magic != [b'P', b'K', 3, 4] {
        let _ = std::fs::remove_file(&stage);
        return Err("Downloaded file is not a valid Android APK archive.".into());
    }
    std::fs::rename(&stage, &dest).map_err(|e| format!("Finalize APK error: {e}"))?;
    Ok(dest)
}
