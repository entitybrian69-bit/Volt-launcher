//! Decoded pictures for the launcher's galleries: screenshots, world icons,
//! server favicons, replay covers and catalog gallery shots.
//!
//! Every picture is decoded off-thread and letterboxed onto a fixed-size
//! transparent canvas, so a dynamic atlas slot keeps one width and height and
//! updates stay cheap patches. Sources are files, raw PNG bytes (servers.dat
//! favicons) and HTTPS addresses (registry and Modrinth gallery shots).
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;

/// The two canvas classes a picture can be prepared at.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fit {
    /// Card thumbnails in the gallery grids.
    Thumb,
    /// The full-screen viewer's picture.
    Full,
}
impl Fit {
    pub fn canvas(self) -> (u32, u32) {
        match self {
            Fit::Thumb => (192, 108),
            Fit::Full => (896, 504),
        }
    }
    /// How many finished entries of this class stay cached.
    fn capacity(self) -> usize {
        match self {
            Fit::Thumb => 48,
            Fit::Full => 2,
        }
    }
}

/// Where a picture's bytes come from.
#[derive(Clone)]
pub enum Source {
    File(PathBuf),
    Bytes(Vec<u8>),
    Https(String),
}

/// A decoded, letterboxed picture ready for `Ui::draw_dynamic_rgba`.
pub struct Photo {
    pub hash: u64,
    pub rgba: Vec<u8>,
    pub loaded_at: std::time::Instant,
}

type Key = (String, bool);
type Decoded = Option<(u64, Vec<u8>)>;

pub struct PhotoCache {
    entries: HashMap<Key, Option<Photo>>,
    /// Insertion order per class, for the eviction caps.
    order: Vec<Key>,
    pending: std::collections::HashSet<Key>,
    tx: mpsc::Sender<(Key, Decoded)>,
    rx: mpsc::Receiver<(Key, Decoded)>,
}

/// A file's cache key: its path and size+mtime, so an overwritten
/// screenshot or world icon redecodes instead of showing the stale frame.
pub fn file_key(path: &std::path::Path) -> String {
    let stamp = std::fs::metadata(path)
        .ok()
        .map(|m| {
            let t = m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("{}:{t}", m.len())
        })
        .unwrap_or_default();
    format!("file:{}@{stamp}", path.display())
}

impl PhotoCache {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            entries: HashMap::new(),
            order: Vec::new(),
            pending: std::collections::HashSet::new(),
            tx,
            rx,
        }
    }

    /// Drain finished decodes; true when anything new arrived.
    pub fn poll(&mut self) -> bool {
        let mut fresh = false;
        while let Ok((key, decoded)) = self.rx.try_recv() {
            self.pending.remove(&key);
            let photo = decoded.map(|(hash, rgba)| Photo {
                hash,
                rgba,
                loaded_at: std::time::Instant::now(),
            });
            let full = key.1;
            self.entries.insert(key.clone(), photo);
            self.order.push(key);
            // Evict the oldest finished entries of the same class past the cap.
            let cap = if full { Fit::Full } else { Fit::Thumb }.capacity();
            let mut count = self.order.iter().filter(|k| k.1 == full).count();
            while count > cap {
                let Some(pos) = self.order.iter().position(|k| k.1 == full) else {
                    break;
                };
                let old = self.order.remove(pos);
                self.entries.remove(&old);
                count -= 1;
            }
            fresh = true;
        }
        fresh
    }

    /// Whether any decode is still running (keep redrawing while true).
    pub fn busy(&self) -> bool {
        !self.pending.is_empty()
    }

    /// The decoded picture for `source` at `fit`, starting the decode on the
    /// first ask. None while loading or when the picture can't be read.
    pub fn get(&mut self, key: &str, source: &Source, fit: Fit) -> Option<&Photo> {
        let full = fit == Fit::Full;
        let k: Key = (key.to_owned(), full);
        if !self.entries.contains_key(&k) && !self.pending.contains(&k) && self.pending.len() < 6 {
            let tx = self.tx.clone();
            let source = source.clone();
            let send_key = k.clone();
            let (cw, ch) = fit.canvas();
            if std::thread::Builder::new()
                .name("volt-photo-decode".into())
                .stack_size(2 * 1024 * 1024)
                .spawn(move || {
                    let decoded = decode(&source, cw, ch);
                    let _ = tx.send((send_key, decoded));
                })
                .is_ok()
            {
                self.pending.insert(k.clone());
            }
        }
        self.entries.get(&k).and_then(|p| p.as_ref())
    }

    /// Whether `key` finished decoding and failed (no picture to show).
    pub fn failed(&self, key: &str, fit: Fit) -> bool {
        matches!(self.entries.get(&(key.to_owned(), fit == Fit::Full)), Some(None))
    }
}

/// Reads, decodes and letterboxes one picture onto a `cw` x `ch` transparent
/// canvas. Runs on the decode thread.
fn decode(source: &Source, cw: u32, ch: u32) -> Decoded {
    let bytes: Vec<u8> = match source {
        Source::File(path) => {
            let meta = std::fs::metadata(path).ok()?;
            if !meta.is_file() || meta.len() > 24 * 1024 * 1024 {
                return None;
            }
            std::fs::read(path).ok()?
        }
        Source::Bytes(bytes) => bytes.clone(),
        Source::Https(url) => {
            if !url.starts_with("https://") {
                return None;
            }
            use std::io::Read;
            let mut bytes = Vec::new();
            ureq::AgentBuilder::new()
                .https_only(true)
                .timeout_connect(std::time::Duration::from_secs(12))
                .timeout_read(std::time::Duration::from_secs(30))
                .build()
                .get(url)
                .call()
                .ok()?
                .into_reader()
                .take(8 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .ok()?;
            if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
                return None;
            }
            bytes
        }
    };
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(6144);
    limits.max_image_height = Some(6144);
    limits.max_alloc = Some(160 << 20);
    reader.limits(limits);
    let decoded = reader.decode().ok()?;
    let (iw, ih) = (decoded.width().max(1), decoded.height().max(1));
    // Fit inside the canvas, keeping the aspect; small pictures (64 px world
    // icons, favicons) scale up with the blocky nearest filter they deserve.
    let scale = (cw as f32 / iw as f32).min(ch as f32 / ih as f32);
    let (dw, dh) = (
        ((iw as f32 * scale).round() as u32).clamp(1, cw),
        ((ih as f32 * scale).round() as u32).clamp(1, ch),
    );
    let filter = if scale >= 1.0 {
        image::imageops::FilterType::Nearest
    } else {
        image::imageops::FilterType::Triangle
    };
    let resized = decoded.resize_exact(dw, dh, filter).to_rgba8();
    let mut canvas = vec![0u8; (cw * ch * 4) as usize];
    let (ox, oy) = ((cw - dw) / 2, (ch - dh) / 2);
    for row in 0..dh {
        let src = (row * dw * 4) as usize;
        let dst = (((row + oy) * cw + ox) * 4) as usize;
        canvas[dst..dst + (dw * 4) as usize].copy_from_slice(&resized.as_raw()[src..src + (dw * 4) as usize]);
    }
    let hash = crate::skin_preview::fnv1a(&canvas);
    Some((hash, canvas))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn letterboxes_onto_the_exact_canvas_and_caches_by_stamp() {
        // A 2x1 red PNG lands centred on the 192x108 thumb canvas.
        let mut png = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png, 2, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[255, 0, 0, 255, 255, 0, 0, 255]).unwrap();
        }
        let (hash, canvas) = decode(&Source::Bytes(png), 192, 108).unwrap();
        assert_eq!(canvas.len(), 192 * 108 * 4);
        assert_ne!(hash, 0);
        // The centre pixel is opaque red, the corner transparent.
        let centre = ((54 * 192 + 96) * 4) as usize;
        assert_eq!(&canvas[centre..centre + 4], &[255, 0, 0, 255]);
        assert_eq!(canvas[3], 0);
        let missing = file_key(std::path::Path::new("/nonexistent/volt-test.png"));
        assert!(missing.ends_with('@'));
    }
}
