//! Native resource installation from Mojang HTTPS endpoints. No Java or LAN server.
use crate::Result;
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

const CATALOG: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const LOCK: &str = include_str!("../../../research-lock.json");
static INSTALLATION: Mutex<()> = Mutex::new(());
static DOWNLOAD_CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Android supplies Context.getCacheDir(), keeping temporary downloads out of files/.
pub fn configure_download_cache_dir(directory: PathBuf) {
    let _ = DOWNLOAD_CACHE_DIR.set(directory.join("minecraft-downloads"));
}

fn download_cache_for(home: &Path, version_id: &str) -> PathBuf {
    DOWNLOAD_CACHE_DIR
        .get()
        .cloned()
        .unwrap_or_else(|| home.join(".local/downloads"))
        .join(version_id)
}

fn download_cache(home: &Path) -> PathBuf {
    download_cache_for(home, horizon_core::VANILLA_VERSION)
}

/// Earlier Android packages installed textures only. An audio-capable APK must
/// upgrade those resources before entering the game, not silently stay mute.
pub fn ready(home: &Path) -> bool {
    let root = home
        .join(".local/horizon-client")
        .join(horizon_core::VANILLA_VERSION);
    crate::resource_install::ready(home)
        && root.join("android-downloads-v2.json").is_file()
        && (!cfg!(feature = "audio") || root.join("audio-complete.json").is_file())
}

/// Returns `true` when both the base engine tables and `version`'s official Mojang
/// client assets (panorama, textures, splashes, sounds, and era worldgen) are prepared.
pub fn ready_for(home: &Path, version: horizon_core::game_version::GameVersion) -> bool {
    if !ready(home) {
        return false;
    }
    if version == horizon_core::game_version::GameVersion::V26_3 {
        return true;
    }
    let root = home.join(".local/horizon-client").join(version.id());
    root.join("horizon-resources.json").is_file() && root.join("volt-version-v1.json").is_file()
}

/// Discard redundant installer inputs only after the prepared installation is complete.
/// An interrupted download/preparation keeps its cache for the next attempt.
pub fn cleanup_download_cache(home: &Path) -> Result<()> {
    if !ready(home) {
        return Ok(());
    }
    let root = fs::canonicalize(home)?;
    let cache = root
        .join(".local/downloads")
        .join(horizon_core::VANILLA_VERSION);
    let metadata = match fs::symlink_metadata(&cache) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    // Verify the final absolute target before recursive removal. Reject aliases
    // of this path, including a redirected .local/downloads ancestor.
    if !metadata.is_dir() || metadata.file_type().is_symlink() || fs::canonicalize(&cache)? != cache
    {
        return Err("Refusing redirected installer cache path".into());
    }
    fs::remove_dir_all(cache)?;
    Ok(())
}

fn cleanup_system_download_cache() -> Result<()> {
    let Some(directory) = DOWNLOAD_CACHE_DIR.get() else {
        return Ok(());
    };
    if !directory.exists() {
        return Ok(());
    }
    let root = fs::canonicalize(directory)?;
    let cache = root.join(horizon_core::VANILLA_VERSION);
    let metadata = match fs::symlink_metadata(&cache) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() || fs::canonicalize(&cache)? != cache
    {
        return Err("Refusing redirected system installer cache".into());
    }
    fs::remove_dir_all(cache)?;
    Ok(())
}

/// Clean both old files/ downloads and the system cache when resources are complete.
pub fn cleanup_completed_downloads(home: &Path) -> Result<()> {
    let _guard = INSTALLATION.lock().unwrap_or_else(|e| e.into_inner());
    cleanup_completed_downloads_inner(home)
}
fn cleanup_completed_downloads_inner(home: &Path) -> Result<()> {
    if !ready(home) {
        return Ok(());
    }
    cleanup_download_cache(home)?;
    cleanup_system_download_cache()
}

fn official_url(text: &str) -> Result<()> {
    let url = url::Url::parse(text)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some_and(|p| p != 443)
        || !matches!(
            url.host_str(),
            Some(
                "piston-meta.mojang.com"
                    | "piston-data.mojang.com"
                    | "resources.download.minecraft.net"
                    | "launcher.mojang.com"
                    | "launchermeta.mojang.com"
            )
        )
    {
        return Err("Download URL is not an official Mojang HTTPS endpoint".into());
    }
    Ok(())
}

fn agent() -> ureq::Agent {
    // Thousands of sound objects share a host; retain pooled TLS connections.
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT
        .get_or_init(|| {
            ureq::AgentBuilder::new()
                .redirects(0)
                .timeout_connect(std::time::Duration::from_secs(30))
                .timeout_read(std::time::Duration::from_secs(120))
                .build()
        })
        .clone()
}

// Metadata requests need the same transient-network recovery as asset files.
// Retry on the installer worker, retaining completed files for the next attempt.
fn metadata_bytes(url: &str) -> Result<Vec<u8>> {
    official_url(url)?;
    for attempt in 0..3 {
        let result = (|| -> Result<Vec<u8>> {
            let mut bytes = Vec::new();
            agent()
                .get(url)
                .call()?
                .into_reader()
                .take(16 * 1024 * 1024)
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        })();
        if result.is_ok() || attempt == 2 {
            return result;
        }
        std::thread::sleep(std::time::Duration::from_millis(500 * (attempt + 1)));
    }
    unreachable!()
}
fn json(url: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&metadata_bytes(url)?)?)
}

fn hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut digest = Sha1::new();
    let mut buffer = [0; 65536];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

struct Download {
    url: String,
    sha1: String,
    size: u64,
    path: PathBuf,
}

impl Download {
    fn from(value: &Value, path: PathBuf) -> Result<Self> {
        let file = Self {
            url: value["url"].as_str().ok_or("Missing download URL")?.into(),
            sha1: value["sha1"]
                .as_str()
                .ok_or("Missing download hash")?
                .into(),
            size: value["size"].as_u64().ok_or("Missing download size")?,
            path,
        };
        official_url(&file.url)?;
        if file.sha1.len() != 40
            || !file.sha1.bytes().all(|c| c.is_ascii_hexdigit())
            || file.size > 1024 * 1024 * 1024
        {
            return Err("Invalid Mojang download metadata".into());
        }
        Ok(file)
    }
    fn get(&self) -> Result<()> {
        if self.path.is_file()
            && fs::metadata(&self.path)?.len() == self.size
            && hash(&self.path)? == self.sha1
        {
            return Ok(());
        }
        fs::create_dir_all(self.path.parent().ok_or("Missing download directory")?)?;
        let partial = self.path.with_extension("partial");
        for attempt in 0..3 {
            let result = (|| -> Result<()> {
                let mut input = agent()
                    .get(&self.url)
                    .call()?
                    .into_reader()
                    .take(self.size + 1);
                let mut output = fs::File::create(&partial)?;
                let count = std::io::copy(&mut input, &mut output)?;
                output.flush()?;
                drop(output);
                if count != self.size || hash(&partial)? != self.sha1 {
                    return Err("Mojang file checksum or size mismatch".into());
                }
                fs::rename(&partial, &self.path)?;
                Ok(())
            })();
            if result.is_ok() {
                return result;
            }
            let _ = fs::remove_file(&partial);
            if attempt == 2 {
                return result;
            }
            std::thread::sleep(std::time::Duration::from_millis(500 * (attempt + 1)));
        }
        unreachable!()
    }
}

fn relative(name: &str) -> Result<&Path> {
    let path = Path::new(name);
    if path
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
        || name.contains('\\')
    {
        return Err("Unsafe asset path".into());
    }
    Ok(path)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InstallStage {
    #[default]
    Metadata,
    Download,
    Extract,
    WorldData,
    Controls,
    Finalize,
}
impl InstallStage {
    pub const ALL: [Self; 6] = [
        Self::Metadata,
        Self::Download,
        Self::Extract,
        Self::WorldData,
        Self::Controls,
        Self::Finalize,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Metadata => "Checking Minecraft version",
            Self::Download => "Downloading game resources and sound",
            Self::Extract => "Unpacking verified files",
            Self::WorldData => "Preparing worlds and registries",
            Self::Controls => "Downloading touch controls",
            Self::Finalize => "Finishing installation",
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct InstallProgress {
    pub stage: InstallStage,
    pub done: u64,
    pub total: u64,
}

pub fn install(home: &Path, progress: impl Fn(u64, u64) + Sync) -> Result<()> {
    install_with_progress(home, |p| {
        if p.stage == InstallStage::Download {
            progress(p.done, p.total)
        }
    })
}
/// Metadata may be reissued, but the engine's game version and JARs must match.
fn validate_manifest(bytes: &[u8], checksum: &str, version: &str, pins: &Value) -> Result<Value> {
    if format!("{:x}", Sha1::digest(bytes)) != checksum {
        return Err("Version metadata checksum mismatch".into());
    }
    let manifest: Value = serde_json::from_slice(bytes)?;
    if manifest["id"].as_str() != Some(version)
        || ["client", "server"].iter().any(|side| {
            let pinned = pins[format!("{side}_sha1")].as_str();
            pinned.is_none() || manifest["downloads"][side]["sha1"].as_str() != pinned
        })
    {
        return Err("Minecraft game files differ from this engine's pinned version".into());
    }
    Ok(manifest)
}

pub fn install_with_progress(home: &Path, progress: impl Fn(InstallProgress) + Sync) -> Result<()> {
    let _guard = INSTALLATION.lock().unwrap_or_else(|e| e.into_inner());
    let report = |stage, done, total| progress(InstallProgress { stage, done, total });
    report(InstallStage::Metadata, 0, 0);
    let version = horizon_core::VANILLA_VERSION;
    let cache = download_cache(home);
    let legacy = home.join(".local/downloads").join(version);
    if cache != legacy && legacy.is_dir() && !cache.exists() {
        let root = fs::canonicalize(home)?;
        if fs::canonicalize(&legacy)? != root.join(".local/downloads").join(version) {
            return Err("Refusing redirected legacy installer cache".into());
        }
        fs::create_dir_all(cache.parent().ok_or("Missing cache parent")?)?;
        fs::rename(&legacy, &cache)?;
    }
    fs::create_dir_all(&cache)?;
    let pins: Value = serde_json::from_str(LOCK)?;
    let pins = &pins["minecraft"];
    let catalog = json(CATALOG)?;
    let entry = catalog["versions"]
        .as_array()
        .and_then(|vs| vs.iter().find(|v| v["id"] == version))
        .ok_or("Pinned Minecraft version is not listed by Mojang")?;
    let checksum = entry["sha1"]
        .as_str()
        .ok_or("Missing version metadata hash")?;
    // Mojang may reissue metadata while Minecraft 26.3's game jars are unchanged.
    // The downloaded JSON is still verified against the catalogue's SHA-1,
    // and the actual client/server jar hashes remain engine-pinned.
    let manifest_url = entry["url"]
        .as_str()
        .ok_or("Missing version metadata URL")?;
    official_url(manifest_url)?;
    let bytes = metadata_bytes(manifest_url)?;
    let manifest = validate_manifest(&bytes, checksum, version, pins)?;
    let index = Download::from(&manifest["assetIndex"], cache.join("asset-index.json"))?;
    index.get()?;
    let index: Value = serde_json::from_reader(fs::File::open(&index.path)?)?;
    let objects = index["objects"]
        .as_object()
        .ok_or("Missing asset objects")?;
    let client = Download::from(&manifest["downloads"]["client"], cache.join("client.jar"))?;
    let mut files = vec![client];
    let mut assets = Vec::new();
    let mut seen = HashSet::new();
    for (name, object) in objects {
        if !cfg!(feature = "audio")
            && (name.starts_with("minecraft/sounds/") || name == "minecraft/sounds.json")
        {
            continue;
        }
        relative(name)?;
        let checksum = object["hash"].as_str().ok_or("Missing asset hash")?;
        if checksum.len() != 40 || !checksum.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("Invalid asset hash".into());
        }
        let path = cache.join("objects").join(checksum);
        assets.push((name, path.clone()));
        if seen.insert(checksum) {
            files.push(Download::from(&serde_json::json!({"url":format!("https://resources.download.minecraft.net/{}/{checksum}", &checksum[..2]), "sha1":checksum, "size":object["size"]}), path)?);
        }
    }
    let total: u64 = files.iter().map(|f| f.size).sum();
    log::info!(
        "Minecraft {version}: downloading {} files directly from Mojang ({} MiB)",
        files.len(),
        total / 1048576
    );
    report(InstallStage::Download, 0, total);
    let next = AtomicUsize::new(0);
    let done = AtomicU64::new(0);
    let error = Mutex::new(None);
    std::thread::scope(|scope| {
        for idx in 0..8 {
            let _ = std::thread::Builder::new()
                .name(format!("volt-dl-{idx}"))
                .stack_size(4 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    loop {
                        if error.lock().unwrap().is_some() {
                            break;
                        }
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(file) = files.get(i) else { break };
                        if let Err(e) = file.get() {
                            *error.lock().unwrap() = Some(e.to_string());
                            break;
                        }
                        report(
                            InstallStage::Download,
                            done.fetch_add(file.size, Ordering::Relaxed) + file.size,
                            total,
                        );
                    }
                });
        }
    });
    if let Some(error) = error.into_inner().unwrap() {
        return Err(error.into());
    }
    let versions = home.join(".local/horizon-client");
    fs::create_dir_all(&versions)?;
    let stage = versions.join("official.partial");
    if stage.exists() {
        fs::remove_dir_all(&stage)?;
    }
    fs::create_dir(&stage)?;
    let result = (|| -> Result<()> {
        report(InstallStage::Extract, 0, 0);
        let mut zip = zip::ZipArchive::new(fs::File::open(&files[0].path)?)?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            if !(entry.name().starts_with("assets/") || entry.name().starts_with("data/"))
                || entry.is_dir()
            {
                continue;
            }
            let name = entry.enclosed_name().ok_or("Unsafe client resource path")?;
            if entry.is_symlink() {
                return Err("Symlink in client resources".into());
            }
            let path = stage.join(name);
            fs::create_dir_all(path.parent().ok_or("Missing resource parent")?)?;
            std::io::copy(&mut entry, &mut fs::File::create(path)?)?;
        }
        for (name, from) in assets {
            let path = stage.join("assets").join(relative(name)?);
            fs::create_dir_all(path.parent().ok_or("Missing asset parent")?)?;
            fs::copy(from, path)?;
        }
        crate::resource_install::extract(&home.join("engine-data.zip"), &stage)?;
        report(InstallStage::WorldData, 0, 0);
        crate::android_resources::rebuild(&stage)?;
        report(InstallStage::Controls, 0, 0);
        crate::android_resources::controls_with_progress(&stage, |done, total| {
            report(InstallStage::Controls, done, total)
        })?;
        report(InstallStage::Finalize, 0, 0);
        fs::write(stage.join("android-downloads-v2.json"), b"{\"schema\":2}")?;
        // Android can evict cache files under storage pressure. The verified
        // index was already decoded above; finalization must not reopen it.
        fs::write(stage.join("asset-index.json"), serde_json::to_vec(&index)?)
            .map_err(|error| format!("Could not save prepared asset index: {error}"))?;
        if !stage.join("horizon-resources.json").is_file() {
            return Err("APK has no engine data for this version".into());
        }
        if cfg!(feature = "audio") {
            fs::write(stage.join("audio-complete.json"), b"{}")?;
        }
        let destination = versions.join(version);
        let backup = versions.join(format!("{version}.previous"));
        if backup.exists() && !destination.exists() {
            fs::rename(&backup, &destination)?;
        }
        if backup.exists() {
            fs::remove_dir_all(&backup)?;
        }
        if destination.exists() {
            fs::rename(&destination, &backup)?;
        }
        if let Err(error) = fs::rename(&stage, &destination) {
            if backup.exists() {
                let _ = fs::rename(&backup, &destination);
            }
            return Err(format!("Could not activate prepared Minecraft resources: {error}").into());
        }
        if backup.exists() {
            let _ = fs::remove_dir_all(&backup);
        }
        Ok(())
    })();
    if stage.exists() {
        let _ = fs::remove_dir_all(stage);
    }
    if result.is_ok() {
        drop(index);
        drop(files);
        horizon_threads::release_free_memory();
        let destination = versions.join(version);
        prewarm_first_launch_caches(home, &destination);
    }
    // `install_with_progress` holds INSTALLATION for the entire transaction,
    // so use the inner helper here rather than reacquiring the non-reentrant
    // mutex through `cleanup_completed_downloads`.
    if result.is_ok()
        && let Err(error) = cleanup_completed_downloads_inner(home)
    {
        // The installation guard is still held here; do not lock it again.
        // Resources were committed successfully; failed housekeeping must not
        // turn a working installation into a reported installation failure.
        log::warn!("Could not clean completed Minecraft downloads: {error}");
    }
    result
}

/// Pre-builds `behaviours-v2.bin` right after installation while 0 MB of GPU/world
/// textures are resident, eliminating ~400 MB of cold-start peak RSS on the very first PLAY tap.
pub fn prewarm_first_launch_caches(home: &Path, root: &Path) {
    horizon_world::level::Behaviours::set_cache_dir(home.join(".horizon-cache"));
    let home_root = root.to_path_buf();
    let _ = std::thread::Builder::new()
        .name("volt-prewarm".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            if let Ok(registry) = horizon_world::level::data::load_registry(&home_root)
                && horizon_world::level::Behaviours::load_cached(&registry, &home_root).is_none()
                && let Ok(file) = fs::File::open(home_root.join("block-behaviours.json"))
                && let Ok(behaviour_json) =
                    serde_json::from_reader::<_, Value>(std::io::BufReader::new(file))
            {
                let _ = horizon_items::blocks::BlockFacts::load_with_behaviours(
                    &home_root,
                    &registry,
                    &behaviour_json,
                );
            }
            horizon_threads::release_free_memory();
        })
        .and_then(|h| h.join().map_err(|_| std::io::Error::other("prewarm panic")));
    horizon_threads::release_free_memory();
}

/// Writes `bytes` to `path` after unlinking any existing hardlink so the base `26.3`
/// installation's shared inodes are never mutated in place.
fn write_unlinked(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = fs::remove_file(path);
    fs::write(path, bytes)
}

fn copy_unlinked(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = fs::remove_file(to);
    fs::hard_link(from, to).or_else(|_| fs::copy(from, to).map(|_| ()))
}

fn hardlink_or_copy_tree(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for item in fs::read_dir(src)? {
        let entry = item?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || entry.file_name() == "horizon-sprites.cache" {
            continue;
        }
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            hardlink_or_copy_tree(&entry.path(), &target)?;
        } else if file_type.is_file() {
            let src_path = entry.path();
            fs::hard_link(&src_path, &target)
                .or_else(|_| fs::copy(&src_path, &target).map(|_| ()))?;
        }
    }
    Ok(())
}

fn png_header_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    (w > 0 && h > 0).then_some((w, h))
}

fn png_file_size(path: &Path) -> Option<(u32, u32)> {
    let mut header = [0u8; 24];
    fs::File::open(path).ok()?.read_exact(&mut header).ok()?;
    png_header_size(&header)
}

/// Vanilla two-tone spawn egg palette used in `1.20.1`..`1.21.4` before `1.21.5`'s
/// shaped spawn-egg textures.
const CLASSIC_SPAWN_EGG_COLORS: &[(&str, u32, u32)] = &[
    ("allay", 0x00DAFF, 0x00ADFF),
    ("armadillo", 0xAD716D, 0x824848),
    ("axolotl", 0xFBC1E3, 0xA62D74),
    ("bat", 0x4C3E30, 0x0F0F0F),
    ("bee", 0xEDC343, 0x43241B),
    ("blaze", 0xF6B201, 0xFFF87E),
    ("bogged", 0x8A9C72, 0x314D1B),
    ("breeze", 0xAF94DF, 0x9166DF),
    ("camel", 0xFCC369, 0xCB9337),
    ("cat", 0xEFC88E, 0x957256),
    ("cave_spider", 0x0C424E, 0xA80E0E),
    ("chicken", 0xA1A1A1, 0xFF0000),
    ("cod", 0xC1A76A, 0xE5C48B),
    ("cow", 0x443626, 0xA1A1A1),
    ("creaking", 0x5F5F5F, 0xFC7812),
    ("creeper", 0x0DA70B, 0x000000),
    ("dolphin", 0x223B4D, 0xF9F9F9),
    ("donkey", 0x534539, 0x867566),
    ("drowned", 0x8FF1D7, 0x799C65),
    ("elder_guardian", 0xCECCBA, 0x747693),
    ("ender_dragon", 0x1C1C1C, 0xE079FA),
    ("enderman", 0x161616, 0x000000),
    ("endermite", 0x161616, 0x6E6E6E),
    ("evoker", 0x959B9B, 0x1E1C1A),
    ("fox", 0xD5B69F, 0xCC6920),
    ("frog", 0xD07444, 0xFFC77C),
    ("ghast", 0xF9F9F9, 0xBCBCBC),
    ("glow_squid", 0x095656, 0x85F1BC),
    ("goat", 0xA5947C, 0x55493E),
    ("guardian", 0x5A8272, 0xF17D30),
    ("hoglin", 0xC66E55, 0x5F6464),
    ("horse", 0xC09E7D, 0xEEE500),
    ("husk", 0x797061, 0xE6CC94),
    ("iron_golem", 0xDBCDC2, 0x74A332),
    ("llama", 0xC09E7D, 0x995F40),
    ("magma_cube", 0x340000, 0xFCFC00),
    ("mooshroom", 0xA00F10, 0xB7B7B7),
    ("mule", 0x1B0200, 0x51331D),
    ("ocelot", 0xEFDE7D, 0x564434),
    ("panda", 0xE7E7E7, 0x1B1B22),
    ("parrot", 0x0DA70B, 0xFF0000),
    ("phantom", 0x43518A, 0x88FF00),
    ("pig", 0xF0A5A2, 0xDB635F),
    ("piglin", 0x995F40, 0xF9F3A4),
    ("piglin_brute", 0x592A10, 0xF9F3A4),
    ("pillager", 0x532F36, 0x959B9B),
    ("polar_bear", 0xEEEEDE, 0xD5D6CD),
    ("pufferfish", 0xF6B201, 0x37C3F2),
    ("rabbit", 0x995F40, 0x734831),
    ("ravager", 0x757470, 0x5B5049),
    ("salmon", 0xA00F10, 0x0E8474),
    ("sheep", 0xE7E7E7, 0xFFB5B5),
    ("shulker", 0x946794, 0x4D3852),
    ("silverfish", 0x6E6E6E, 0x303030),
    ("skeleton", 0xC1C1C1, 0x494949),
    ("skeleton_horse", 0x68684F, 0xE5E5D8),
    ("slime", 0x51A03E, 0x7EBF6E),
    ("sniffer", 0x871E09, 0x25AB70),
    ("snow_golem", 0xD9F2F2, 0x81A4A4),
    ("spider", 0x342D27, 0xA80E0E),
    ("squid", 0x223B4D, 0x708899),
    ("stray", 0x617677, 0xDDEAEA),
    ("strider", 0x9C3436, 0x4D494D),
    ("tadpole", 0x6D533D, 0x160A00),
    ("trader_llama", 0xEAA430, 0x456296),
    ("tropical_fish", 0xEF6915, 0xFFF9EF),
    ("turtle", 0xE7E7E7, 0x00AFAF),
    ("vex", 0x7A90A4, 0xE8EDF1),
    ("villager", 0x563C33, 0xBD8B72),
    ("vindicator", 0x959B9B, 0x275E61),
    ("wandering_trader", 0x456296, 0xEAA430),
    ("warden", 0x0F4649, 0x39D6E0),
    ("witch", 0x340000, 0x51A03E),
    ("wither", 0x141414, 0x4D72A0),
    ("wither_skeleton", 0x141414, 0x474D4D),
    ("wolf", 0xD7D3D3, 0xCEAF96),
    ("zoglin", 0xC66E55, 0xE6E6E6),
    ("zombie", 0x00AFAF, 0x799C65),
    ("zombie_horse", 0x315234, 0x97C284),
    ("zombie_villager", 0x563C33, 0x799C65),
    ("zombified_piglin", 0xEA9393, 0x4C7129),
];

fn bake_classic_spawn_eggs(
    item_tex_dir: &Path,
    base_png: &[u8],
    overlay_png: &[u8],
) -> Result<()> {
    let (bw, bh, base_rgba) = crate::hud::decode_bytes(base_png)?;
    let (ow, oh, over_rgba) = crate::hud::decode_bytes(overlay_png)?;
    if bw == 0 || bh == 0 || (bw, bh) != (ow, oh) {
        return Ok(());
    }
    let mul = |c: u8, tint: u8| ((u32::from(c) * u32::from(tint) + 127) / 255) as u8;
    let rgb = |hex: u32| {
        [
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        ]
    };
    let mut pixels = vec![0u8; base_rgba.len()];
    let mut encoded = Vec::with_capacity(1024);
    for &(mob, primary, secondary) in CLASSIC_SPAWN_EGG_COLORS {
        let p = rgb(primary);
        let s = rgb(secondary);
        for ((out, b), o) in pixels
            .chunks_exact_mut(4)
            .zip(base_rgba.chunks_exact(4))
            .zip(over_rgba.chunks_exact(4))
        {
            let br = mul(b[0], p[0]);
            let bg = mul(b[1], p[1]);
            let bb = mul(b[2], p[2]);
            let ba = b[3];

            let or = mul(o[0], s[0]);
            let og = mul(o[1], s[1]);
            let ob = mul(o[2], s[2]);
            let oa = u32::from(o[3]);

            if oa == 0 {
                out.copy_from_slice(&[br, bg, bb, ba]);
            } else if oa == 255 || ba == 0 {
                out.copy_from_slice(&[or, og, ob, o[3].max(ba)]);
            } else {
                let inv = 255 - oa;
                out[0] = ((u32::from(or) * oa + u32::from(br) * inv + 127) / 255) as u8;
                out[1] = ((u32::from(og) * oa + u32::from(bg) * inv + 127) / 255) as u8;
                out[2] = ((u32::from(ob) * oa + u32::from(bb) * inv + 127) / 255) as u8;
                out[3] = o[3].max(ba);
            }
        }
        encoded.clear();
        {
            let mut enc = png::Encoder::new(&mut encoded, bw, bh);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            let mut writer = enc.write_header()?;
            writer.write_image_data(&pixels)?;
        }
        let target = item_tex_dir.join(format!("{mob}_spawn_egg.png"));
        write_unlinked(&target, &encoded)?;
        let _ = fs::remove_file(item_tex_dir.join(format!("{mob}_spawn_egg.png.mcmeta")));
    }
    Ok(())
}

fn merge_version_lang(
    stage_lang_path: &Path,
    ver_lang_bytes: &[u8],
    version: horizon_core::game_version::GameVersion,
) -> Result<()> {
    let mut base_map: serde_json::Map<String, Value> = fs::read(stage_lang_path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    if let Ok(ver_map) = serde_json::from_slice::<serde_json::Map<String, Value>>(ver_lang_bytes) {
        for (k, v) in ver_map {
            if k.starts_with("options.horizon") {
                continue;
            }
            base_map.insert(k, v);
        }
    }
    if version < horizon_core::game_version::GameVersion::V1_20_3 {
        if let Some(grass_name) = base_map.get("block.minecraft.grass").cloned() {
            base_map.insert("block.minecraft.short_grass".into(), grass_name);
        } else {
            base_map.insert(
                "block.minecraft.short_grass".into(),
                Value::String("Grass".into()),
            );
        }
    }
    if version < horizon_core::game_version::GameVersion::V1_20_5 {
        if let Some(scute_name) = base_map.get("item.minecraft.scute").cloned() {
            base_map.insert("item.minecraft.turtle_scute".into(), scute_name);
        } else {
            base_map.insert(
                "item.minecraft.turtle_scute".into(),
                Value::String("Scute".into()),
            );
        }
    }
    write_unlinked(stage_lang_path, &serde_json::to_vec(&base_map)?)?;
    Ok(())
}

fn patch_era_worldgen(
    stage: &Path,
    version: horizon_core::game_version::GameVersion,
) -> Result<()> {
    use horizon_core::game_version::GameVersion;
    if version < GameVersion::V1_21 {
        let sets_path = stage.join("worldgen/structure_sets.json");
        if let Ok(bytes) = fs::read(&sets_path)
            && let Ok(mut map) = serde_json::from_slice::<serde_json::Map<String, Value>>(&bytes)
            && map.remove("minecraft:trial_chambers").is_some()
        {
            write_unlinked(&sets_path, &serde_json::to_vec(&map)?)?;
        }
    }
    if version < GameVersion::V1_21_4 {
        let presets_path = stage.join("worldgen/world_presets.json");
        if let Ok(bytes) = fs::read(&presets_path)
            && let Ok(mut map) = serde_json::from_slice::<serde_json::Map<String, Value>>(&bytes)
        {
            let mut changed = false;
            for val in map.values_mut() {
                if let Some(s) = val.as_str()
                    && s.contains("minecraft:pale_garden")
                {
                    *val = Value::String(
                        s.replace("\"minecraft:pale_garden\"", "\"minecraft:dark_forest\""),
                    );
                    changed = true;
                }
            }
            if changed {
                write_unlinked(&presets_path, &serde_json::to_vec(&map)?)?;
            }
        }
    }
    if version < GameVersion::V1_21_5 {
        const SPRING_FEATURES: &[&str] = &[
            "minecraft:patch_leaf_litter",
            "minecraft:wildflowers_birch_forest",
            "minecraft:wildflowers_meadow",
            "minecraft:patch_firefly_bush_near_water",
            "minecraft:patch_firefly_bush_near_water_swamp",
            "minecraft:patch_firefly_bush_swamp",
            "minecraft:patch_dry_grass_badlands",
            "minecraft:patch_dry_grass_desert",
            "minecraft:patch_bush",
            "minecraft:fallen_oak_tree",
            "minecraft:fallen_birch_tree",
            "minecraft:fallen_super_birch_tree",
            "minecraft:fallen_jungle_tree",
            "minecraft:fallen_spruce_tree",
        ];
        let biomes_path = stage.join("worldgen/biomes.json");
        if let Ok(bytes) = fs::read(&biomes_path)
            && let Ok(mut map) = serde_json::from_slice::<serde_json::Map<String, Value>>(&bytes)
        {
            let mut any_changed = false;
            for val in map.values_mut() {
                let Some(s) = val.as_str() else { continue };
                if !SPRING_FEATURES.iter().any(|f| s.contains(f)) {
                    continue;
                }
                if let Ok(mut biome_doc) = serde_json::from_str::<Value>(s)
                    && let Some(steps) = biome_doc
                        .get_mut("features")
                        .and_then(|f| f.as_array_mut())
                {
                    for step in steps {
                        if let Some(arr) = step.as_array_mut() {
                            arr.retain(|item| {
                                item.as_str()
                                    .is_none_or(|id| !SPRING_FEATURES.contains(&id))
                            });
                        }
                    }
                    if let Ok(updated) = serde_json::to_string(&biome_doc) {
                        *val = Value::String(updated);
                        any_changed = true;
                    }
                }
            }
            if any_changed {
                write_unlinked(&biomes_path, &serde_json::to_vec(&map)?)?;
            }
        }
    }
    Ok(())
}

/// Installs `version`'s official Mojang client assets on top of the base engine tables,
/// reusing already-downloaded SHA-1 verified sound/icon objects from `26.3` so switching
/// to `1.20.1`..`26.2` downloads only `client.jar` + version-specific assets.
pub fn install_version_with_progress(
    home: &Path,
    version: horizon_core::game_version::GameVersion,
    progress: impl Fn(InstallProgress) + Sync,
) -> Result<()> {
    if !ready(home) {
        install_with_progress(home, &progress)?;
    }
    if version == horizon_core::game_version::GameVersion::V26_3 || ready_for(home, version) {
        return Ok(());
    }
    let _guard = INSTALLATION.lock().unwrap_or_else(|e| e.into_inner());
    let report = |stage, done, total| progress(InstallProgress { stage, done, total });
    report(InstallStage::Metadata, 0, 0);

    let ver_id = version.id();
    let versions = home.join(".local/horizon-client");
    let base_root = versions.join(horizon_core::VANILLA_VERSION);
    let cache = download_cache_for(home, ver_id);
    fs::create_dir_all(&cache)?;

    let catalog = json(CATALOG)?;
    let entry = catalog["versions"]
        .as_array()
        .and_then(|vs| vs.iter().find(|v| v["id"].as_str() == Some(ver_id)))
        .ok_or_else(|| format!("Minecraft {ver_id} is not listed by Mojang"))?;
    let checksum = entry["sha1"]
        .as_str()
        .ok_or("Missing version metadata hash")?;
    let manifest_url = entry["url"]
        .as_str()
        .ok_or("Missing version metadata URL")?;
    official_url(manifest_url)?;
    let bytes = metadata_bytes(manifest_url)?;
    if format!("{:x}", Sha1::digest(&bytes)) != checksum {
        return Err("Version metadata checksum mismatch".into());
    }
    let manifest: Value = serde_json::from_slice(&bytes)?;
    if manifest["id"].as_str() != Some(ver_id) {
        return Err("Mojang version manifest ID mismatch".into());
    }

    let index_dl = Download::from(&manifest["assetIndex"], cache.join("asset-index.json"))?;
    index_dl.get()?;
    let index: Value = serde_json::from_reader(fs::File::open(&index_dl.path)?)?;
    let objects = index["objects"]
        .as_object()
        .ok_or("Missing asset objects")?;

    // Build O(1) lookup of SHA-1 hashes already installed in `.local/horizon-client/26.3`.
    let mut base_hashes = std::collections::HashMap::<String, String>::new();
    if let Ok(base_idx_bytes) = fs::read(base_root.join("asset-index.json"))
        && let Ok(base_idx) = serde_json::from_slice::<Value>(&base_idx_bytes)
        && let Some(base_objs) = base_idx["objects"].as_object()
    {
        for (k, v) in base_objs {
            if let Some(h) = v["hash"].as_str() {
                base_hashes.insert(k.clone(), h.to_owned());
            }
        }
    }

    let client = Download::from(&manifest["downloads"]["client"], cache.join("client.jar"))?;
    let mut files = vec![client];
    let mut changed_assets = Vec::<(String, PathBuf)>::new();
    let mut seen = HashSet::new();
    for (name, object) in objects {
        if !cfg!(feature = "audio")
            && (name.starts_with("minecraft/sounds/") || name == "minecraft/sounds.json")
        {
            continue;
        }
        let rel = relative(name)?;
        let obj_hash = object["hash"].as_str().ok_or("Missing asset hash")?;
        if obj_hash.len() != 40 || !obj_hash.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("Invalid asset hash".into());
        }
        if base_hashes.get(name).map(String::as_str) == Some(obj_hash)
            && base_root.join("assets").join(rel).is_file()
        {
            continue;
        }
        let path = cache.join("objects").join(obj_hash);
        changed_assets.push((name.clone(), path.clone()));
        if seen.insert(obj_hash.to_owned()) {
            files.push(Download::from(
                &serde_json::json!({
                    "url": format!(
                        "https://resources.download.minecraft.net/{}/{obj_hash}",
                        &obj_hash[..2]
                    ),
                    "sha1": obj_hash,
                    "size": object["size"]
                }),
                path,
            )?);
        }
    }

    let total: u64 = files.iter().map(|f| f.size).sum();
    log::info!(
        "Minecraft {ver_id}: downloading {} version-specific files from Mojang ({} MiB)",
        files.len(),
        total / 1048576
    );
    report(InstallStage::Download, 0, total);
    let next = AtomicUsize::new(0);
    let done = AtomicU64::new(0);
    let error = Mutex::new(None);
    std::thread::scope(|scope| {
        for idx in 0..8 {
            let _ = std::thread::Builder::new()
                .name(format!("volt-vdl-{idx}"))
                .stack_size(4 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    loop {
                        if error.lock().unwrap().is_some() {
                            break;
                        }
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(file) = files.get(i) else { break };
                        if let Err(e) = file.get() {
                            *error.lock().unwrap() = Some(e.to_string());
                            break;
                        }
                        report(
                            InstallStage::Download,
                            done.fetch_add(file.size, Ordering::Relaxed) + file.size,
                            total,
                        );
                    }
                });
        }
    });
    if let Some(error) = error.into_inner().unwrap() {
        return Err(error.into());
    }

    let stage = versions.join(format!("{ver_id}.partial"));
    if stage.exists() {
        fs::remove_dir_all(&stage)?;
    }
    let result = (|| -> Result<()> {
        report(InstallStage::Extract, 0, 0);
        hardlink_or_copy_tree(&base_root, &stage)?;

        let mut zip = zip::ZipArchive::new(fs::File::open(&files[0].path)?)?;
        let mut jar_mcmetas = std::collections::HashMap::<String, Vec<u8>>::new();
        let mut jar_pngs = Vec::<(String, Vec<u8>)>::new();
        let mut spawn_egg_base = None::<Vec<u8>>;
        let mut spawn_egg_overlay = None::<Vec<u8>>;
        let mut legacy_grass_png = None::<Vec<u8>>;
        let mut legacy_scute_png = None::<Vec<u8>>;
        let mut ver_en_us = None::<Vec<u8>>;

        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            if entry.is_dir() || entry.is_symlink() {
                continue;
            }
            let Some(enclosed) = entry.enclosed_name() else {
                continue;
            };
            let name = enclosed.to_string_lossy().replace('\\', "/");
            if name == "assets/minecraft/lang/en_us.json" {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                ver_en_us = Some(buf);
                continue;
            }
            if name.starts_with("assets/minecraft/textures/gui/title/")
                || name.starts_with("assets/minecraft/texts/")
                || (version < horizon_core::game_version::GameVersion::V1_20_5
                    && name == "assets/minecraft/textures/gui/options_background.png")
            {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                write_unlinked(&stage.join(&name), &buf)?;
                continue;
            }
            const TEX_DIRS: &[&str] = &[
                "assets/minecraft/textures/block/",
                "assets/minecraft/textures/item/",
                "assets/minecraft/textures/environment/",
                "assets/minecraft/textures/mob_effect/",
                "assets/minecraft/textures/painting/",
                "assets/minecraft/textures/colormap/",
                "assets/minecraft/textures/misc/",
            ];
            if TEX_DIRS.iter().any(|p| name.starts_with(p)) {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                if name == "assets/minecraft/textures/item/spawn_egg.png" {
                    spawn_egg_base = Some(buf.clone());
                } else if name == "assets/minecraft/textures/item/spawn_egg_overlay.png" {
                    spawn_egg_overlay = Some(buf.clone());
                } else if name == "assets/minecraft/textures/block/grass.png" {
                    legacy_grass_png = Some(buf.clone());
                } else if name == "assets/minecraft/textures/item/scute.png" {
                    legacy_scute_png = Some(buf.clone());
                }
                if name.ends_with(".png.mcmeta") {
                    jar_mcmetas.insert(name, buf);
                } else if name.ends_with(".png") {
                    jar_pngs.push((name, buf));
                }
            }
        }

        for (name, buf) in jar_pngs {
            let dst = stage.join(&name);
            let Some((nw, nh)) = png_header_size(&buf) else {
                continue;
            };
            let compatible = match png_file_size(&dst) {
                None => true,
                Some((ow, oh)) => {
                    (nw, nh) == (ow, oh)
                        || ((name.starts_with("assets/minecraft/textures/block/")
                            || name.starts_with("assets/minecraft/textures/item/"))
                            && nw == ow
                            && nh % nw == 0)
                }
            };
            if compatible {
                write_unlinked(&dst, &buf)?;
                let mcmeta_key = format!("{name}.mcmeta");
                let mcmeta_dst = stage.join(&mcmeta_key);
                if let Some(meta_bytes) = jar_mcmetas.get(&mcmeta_key) {
                    write_unlinked(&mcmeta_dst, meta_bytes)?;
                } else {
                    let _ = fs::remove_file(&mcmeta_dst);
                }
            }
        }

        if version < horizon_core::game_version::GameVersion::V1_20_3
            && let Some(grass) = &legacy_grass_png
        {
            write_unlinked(
                &stage.join("assets/minecraft/textures/block/short_grass.png"),
                grass,
            )?;
            let _ = fs::remove_file(
                stage.join("assets/minecraft/textures/block/short_grass.png.mcmeta"),
            );
        }
        if version < horizon_core::game_version::GameVersion::V1_20_5
            && let Some(scute) = &legacy_scute_png
        {
            write_unlinked(
                &stage.join("assets/minecraft/textures/item/turtle_scute.png"),
                scute,
            )?;
            let _ = fs::remove_file(
                stage.join("assets/minecraft/textures/item/turtle_scute.png.mcmeta"),
            );
        }
        if version < horizon_core::game_version::GameVersion::V1_21_5
            && let (Some(base_egg), Some(over_egg)) = (&spawn_egg_base, &spawn_egg_overlay)
        {
            bake_classic_spawn_eggs(
                &stage.join("assets/minecraft/textures/item"),
                base_egg,
                over_egg,
            )?;
        }
        if let Some(en_us_bytes) = &ver_en_us {
            merge_version_lang(
                &stage.join("assets/minecraft/lang/en_us.json"),
                en_us_bytes,
                version,
            )?;
        }

        for (name, from) in &changed_assets {
            if name == "minecraft/lang/en_us.json" {
                continue;
            }
            if name.starts_with("minecraft/textures/gui/title/")
                || name.starts_with("minecraft/sounds/")
                || name == "minecraft/sounds.json"
                || name.starts_with("minecraft/icons/")
                || (name.starts_with("minecraft/lang/") && name != "minecraft/lang/ru_ru.json")
            {
                let dst = stage.join("assets").join(relative(name)?);
                copy_unlinked(from, &dst)?;
            }
        }

        report(InstallStage::WorldData, 0, 0);
        patch_era_worldgen(&stage, version)?;

        report(InstallStage::Finalize, 0, 0);
        write_unlinked(
            &stage.join("asset-index.json"),
            &serde_json::to_vec(&index)?,
        )?;
        write_unlinked(
            &stage.join("volt-version-v1.json"),
            serde_json::json!({
                "schema": 1,
                "version": ver_id,
                "protocol": version.protocol(),
            })
            .to_string()
            .as_bytes(),
        )?;

        let destination = versions.join(ver_id);
        let backup = versions.join(format!("{ver_id}.previous"));
        if backup.exists() {
            let _ = fs::remove_dir_all(&backup);
        }
        if destination.exists() {
            fs::rename(&destination, &backup)?;
        }
        if let Err(err) = fs::rename(&stage, &destination) {
            if backup.exists() {
                let _ = fs::rename(&backup, &destination);
            }
            return Err(
                format!("Could not activate Minecraft {ver_id} resources: {err}").into(),
            );
        }
        if backup.exists() {
            let _ = fs::remove_dir_all(&backup);
        }
        Ok(())
    })();

    if stage.exists() {
        let _ = fs::remove_dir_all(&stage);
    }
    if result.is_ok() {
        let _ = fs::remove_dir_all(&cache);
        horizon_threads::release_free_memory();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CacheHome(PathBuf);
    impl CacheHome {
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for CacheHome {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn cache_home() -> CacheHome {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = fs::canonicalize(std::env::temp_dir()).unwrap();
        let path = base.join(format!(
            "horizon-installer-cache-test-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        let home = CacheHome(path);
        let cache = home
            .path()
            .join(".local/downloads")
            .join(horizon_core::VANILLA_VERSION);
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("client.jar"), b"download input").unwrap();
        fs::create_dir_all(home.path().join("saves")).unwrap();
        fs::write(home.path().join("saves/keep"), b"player world").unwrap();
        home
    }

    fn mark_cache_install_ready(home: &Path) {
        let root = home
            .join(".local/horizon-client")
            .join(horizon_core::VANILLA_VERSION);
        fs::create_dir_all(&root).unwrap();
        for marker in [
            "horizon-resources.json",
            "android-downloads-v2.json",
            "audio-complete.json",
        ] {
            fs::write(root.join(marker), b"{}").unwrap();
        }
        fs::write(root.join("prepared-data"), b"runtime data").unwrap();
    }

    #[test]
    fn cleanup_waits_until_installation_releases_its_inputs() {
        let home = cache_home();
        mark_cache_install_ready(home.path());
        let guard = INSTALLATION.lock().unwrap();
        let path = home.path().to_owned();
        let (tx, rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let result = cleanup_completed_downloads(&path).map_err(|e| e.to_string());
            tx.send(result).unwrap();
        });
        assert!(
            rx.recv_timeout(std::time::Duration::from_millis(30))
                .is_err()
        );
        assert!(
            home.path()
                .join(".local/downloads")
                .join(horizon_core::VANILLA_VERSION)
                .join("client.jar")
                .is_file()
        );
        drop(guard);
        rx.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap()
            .unwrap();
        worker.join().unwrap();
        assert!(home.path().join("saves/keep").is_file());
    }

    #[test]
    fn completed_install_discards_only_its_download_cache() {
        let home = cache_home();
        mark_cache_install_ready(home.path());
        cleanup_download_cache(home.path()).unwrap();
        assert!(
            !home
                .path()
                .join(".local/downloads")
                .join(horizon_core::VANILLA_VERSION)
                .exists()
        );
        assert!(home.path().join("saves/keep").is_file());
        assert!(
            home.path()
                .join(".local/horizon-client")
                .join(horizon_core::VANILLA_VERSION)
                .join("prepared-data")
                .is_file()
        );
        cleanup_download_cache(home.path()).unwrap();
    }

    #[test]
    fn interrupted_install_keeps_inputs_for_retry() {
        let home = cache_home();
        cleanup_download_cache(home.path()).unwrap();
        assert!(
            home.path()
                .join(".local/downloads")
                .join(horizon_core::VANILLA_VERSION)
                .join("client.jar")
                .is_file()
        );
    }

    #[test]
    fn android_cache_location_is_disposable_after_preparation() {
        let home = cache_home();
        let android_cache = home.path().join("android-cache");
        configure_download_cache_dir(android_cache.clone());
        let cache = download_cache(home.path());
        assert_eq!(
            cache,
            android_cache
                .join("minecraft-downloads")
                .join(horizon_core::VANILLA_VERSION)
        );
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("client.jar"), b"download").unwrap();
        cleanup_completed_downloads(home.path()).unwrap();
        assert!(cache.is_dir());
        mark_cache_install_ready(home.path());
        cleanup_completed_downloads(home.path()).unwrap();
        assert!(!cache.exists());
        assert!(
            !home
                .path()
                .join(".local/downloads")
                .join(horizon_core::VANILLA_VERSION)
                .exists()
        );
        assert!(home.path().join("saves/keep").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn redirected_cache_cannot_remove_a_different_directory() {
        let home = cache_home();
        mark_cache_install_ready(home.path());
        let cache = home
            .path()
            .join(".local/downloads")
            .join(horizon_core::VANILLA_VERSION);
        fs::remove_dir_all(&cache).unwrap();
        std::os::unix::fs::symlink(home.path().join("saves"), &cache).unwrap();
        assert!(cleanup_download_cache(home.path()).is_err());
        assert!(home.path().join("saves/keep").is_file());
    }
    fn manifest_fixture() -> (Value, Value) {
        let pins = serde_json::json!({
            "manifest_sha1": "old-metadata-hash",
            "client_sha1": "client-jar-hash",
            "server_sha1": "server-jar-hash"
        });
        let manifest = serde_json::json!({
            "id": "test-version",
            "downloads": {
                "client": {"sha1": "client-jar-hash"},
                "server": {"sha1": "server-jar-hash"}
            },
            "releaseTime": "new-metadata"
        });
        (manifest, pins)
    }
    fn check_manifest(manifest: &Value, pins: &Value) -> Result<Value> {
        let bytes = serde_json::to_vec(manifest)?;
        let checksum = format!("{:x}", Sha1::digest(&bytes));
        validate_manifest(&bytes, &checksum, "test-version", pins)
    }
    #[test]
    fn republished_metadata_keeps_the_pinned_game_usable() {
        let (manifest, pins) = manifest_fixture();
        assert!(check_manifest(&manifest, &pins).is_ok());
        let mut tampered = serde_json::to_vec(&manifest).unwrap();
        let checksum = format!("{:x}", Sha1::digest(&tampered));
        tampered.push(b' ');
        assert!(validate_manifest(&tampered, &checksum, "test-version", &pins).is_err());
    }
    #[test]
    fn metadata_cannot_replace_either_jar_or_the_game_version() {
        let (manifest, pins) = manifest_fixture();
        for side in ["client", "server"] {
            let mut changed = manifest.clone();
            changed["downloads"][side]["sha1"] = Value::String("other-jar".into());
            assert!(check_manifest(&changed, &pins).is_err());
            let mut missing = pins.clone();
            missing
                .as_object_mut()
                .unwrap()
                .remove(&format!("{side}_sha1"));
            changed["downloads"][side]
                .as_object_mut()
                .unwrap()
                .remove("sha1");
            assert!(check_manifest(&changed, &missing).is_err());
        }
        let mut changed = manifest;
        changed["id"] = Value::String("other-version".into());
        assert!(check_manifest(&changed, &pins).is_err());
    }
    #[test]
    fn only_mojang_https_is_accepted() {
        assert!(official_url("https://piston-data.mojang.com/objects/game.jar").is_ok());
        for url in [
            "http://piston-data.mojang.com/game.jar",
            "https://piston-data.mojang.com.evil.test/game.jar",
            "https://piston-data.mojang.com@evil.test/game.jar",
            "http://127.0.0.1:8123/game.zip",
        ] {
            assert!(official_url(url).is_err());
        }
    }
    #[test]
    fn asset_paths_cannot_escape_installation() {
        assert!(relative("minecraft/lang/en_us.json").is_ok());
        for name in [
            "../escape",
            "/absolute",
            "minecraft/../escape",
            "minecraft\\..\\escape",
        ] {
            assert!(relative(name).is_err());
        }
    }

    #[test]
    fn version_overlay_unlinks_hardlinks_and_patches_era_worldgen() {
        use horizon_core::game_version::GameVersion;
        let home = cache_home();
        mark_cache_install_ready(home.path());
        assert!(ready_for(home.path(), GameVersion::V26_3));
        assert!(!ready_for(home.path(), GameVersion::V1_20_1));

        let base = home.path().join(".local/horizon-client/26.3");
        fs::create_dir_all(base.join("worldgen")).unwrap();
        fs::write(
            base.join("worldgen/structure_sets.json"),
            r#"{"minecraft:villages":"{}","minecraft:trial_chambers":"{}"}"#,
        )
        .unwrap();
        fs::write(
            base.join("worldgen/world_presets.json"),
            r#"{"minecraft:normal":"{\"biome\":\"minecraft:pale_garden\"}"}"#,
        )
        .unwrap();
        fs::write(
            base.join("worldgen/biomes.json"),
            r#"{"minecraft:forest":"{\"features\":[[\"minecraft:trees_birch_and_oak\",\"minecraft:patch_leaf_litter\",\"minecraft:fallen_oak_tree\"]]}"}"#,
        )
        .unwrap();

        let stage = home.path().join(".local/horizon-client/1.20.1");
        hardlink_or_copy_tree(&base, &stage).unwrap();
        patch_era_worldgen(&stage, GameVersion::V1_20_1).unwrap();
        write_unlinked(&stage.join("volt-version-v1.json"), b"{\"schema\":1}").unwrap();

        assert!(ready_for(home.path(), GameVersion::V1_20_1));

        // 26.3 base files must remain 100% untouched despite hardlinking!
        let base_sets = fs::read_to_string(base.join("worldgen/structure_sets.json")).unwrap();
        assert!(base_sets.contains("minecraft:trial_chambers"));
        let base_presets = fs::read_to_string(base.join("worldgen/world_presets.json")).unwrap();
        assert!(base_presets.contains("minecraft:pale_garden"));

        // 1.20.1 overlaid files must exclude Trial Chambers, Pale Garden, and 1.21.5 Leaf Litter/Fallen Trees!
        let ver_sets = fs::read_to_string(stage.join("worldgen/structure_sets.json")).unwrap();
        assert!(!ver_sets.contains("minecraft:trial_chambers"));
        assert!(ver_sets.contains("minecraft:villages"));
        let ver_presets = fs::read_to_string(stage.join("worldgen/world_presets.json")).unwrap();
        assert!(!ver_presets.contains("minecraft:pale_garden"));
        assert!(ver_presets.contains("minecraft:dark_forest"));
        let ver_biomes = fs::read_to_string(stage.join("worldgen/biomes.json")).unwrap();
        assert!(!ver_biomes.contains("minecraft:patch_leaf_litter"));
        assert!(!ver_biomes.contains("minecraft:fallen_oak_tree"));
        assert!(ver_biomes.contains("minecraft:trees_birch_and_oak"));
    }
}
