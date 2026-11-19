//! Verified, non-executing installation of compatible content in a single profile.
use horizon_client::content_files;
use horizon_mods::index::{self, Index, Kind};
use std::{fs, io::Read, path::Path};
#[derive(Clone)]
pub(super) struct Entry {
    pub id: String,
    pub title: String,
    pub summary: String,
    hash: String,
}
fn version<'a>(idx: &Index, p: &'a index::Project) -> Option<&'a index::Version> {
    idx.installable(
        p,
        horizon_core::VANILLA_VERSION,
        horizon_mods::api_level::LEVEL,
    )
    .filter(|v| v.requires.is_empty())
}
pub(super) fn rows(idx: Index, kind: &Kind) -> Vec<Entry> {
    idx.projects
        .iter()
        .filter(|p| p.kind == *kind)
        .filter_map(|p| {
            let v = version(idx, p)?;
            Some(Entry {
                id: p.id.clone(),
                title: format!("{} {}", p.name, v.version),
                summary: p.summary.clone(),
                hash: v.file.sha256.clone(),
            })
        })
        .collect()
}
pub(super) fn install_from<R: Read>(
    idx: &Index,
    home: &Path,
    kind: &Kind,
    entry: Entry,
    download: impl FnOnce(&str) -> Result<R, String>,
) -> Result<String, String> {
    // Refresh revocations and bind the click to the version/hash the player saw.
    let p = idx
        .project(&entry.id)
        .filter(|p| &p.kind == kind)
        .ok_or("Project no longer available")?;
    let v = version(idx, p)
        .filter(|v| v.file.sha256 == entry.hash)
        .ok_or("Project changed or was revoked; refresh the catalog")?;
    let content = match kind {
        Kind::Mod => content_files::Kind::Mods,
        Kind::Shader => content_files::Kind::Shaders,
        Kind::ResourcePack => content_files::Kind::Packs,
        _ => return Err("Unsupported package kind".into()),
    };
    let url = url::Url::parse(&v.file.url).map_err(|_| "Invalid release URL")?;
    let segments: Vec<_> = url.path_segments().into_iter().flatten().collect();
    if url.scheme() != "https"
        || url.host_str() != Some("github.com")
        || !matches!(&segments[..], [_, _, "releases", "download", _, _])
    {
        return Err("Expected a GitHub release asset".into());
    }
    // Prefix the already-validated id so Windows device names (e.g. CON) cannot be destinations.
    let name = format!(
        "volt-{}.{}",
        p.id,
        kind.extension().ok_or("Unknown extension")?
    );
    let dir = home.join(content.folder());
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    use std::os::windows::fs::MetadataExt;
    if fs::symlink_metadata(&dir)
        .map_err(|e| e.to_string())?
        .file_attributes()
        & 0x400
        != 0
    {
        return Err("Refusing a redirected content folder".into());
    }
    let path = dir.join(&name);
    if path.exists() {
        return Err("Already installed (existing file kept)".into());
    }
    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).map_err(|e| e.to_string())?;
    let stage = dir.join(format!(".volt-{:032x}.partial", u128::from_le_bytes(nonce)));
    let mut out = fs::File::create_new(&stage).map_err(|e| e.to_string())?;
    let result = (|| {
        let n = std::io::copy(&mut download(&v.file.url)?.take(v.file.size + 1), &mut out)
            .map_err(|e| e.to_string())?;
        out.sync_all().map_err(|e| e.to_string())?;
        drop(out);
        if n != v.file.size {
            return Err("Download size mismatch".into());
        }
        content_files::validate(&stage, content, &name, || false).map_err(|e| e.to_string())?;
        index::check_file(idx, p, v, &stage)?;
        // Atomic, fail-if-exists on NTFS. Do not replace an existing player file.
        fs::hard_link(&stage, path).map_err(|e| e.to_string())?;
        Ok(format!("Installed {}. Enable it in game settings.", p.name))
    })();
    let _ = fs::remove_file(stage);
    result
}
