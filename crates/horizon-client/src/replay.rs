//! Volt Replay: records a multiplayer session's packet stream and plays it
//! back through the same event pipeline a live server connection uses, so
//! the whole engine replays it — world, entities, chat, everything.
//!
//! The file (`replays/*.vrpl`) is a fixed plain header (so the duration can
//! be patched in on finish and the launcher can list files cheaply) followed
//! by a gzip stream of records. Each record carries the phase
//! (configuration clientbound / play clientbound / the recorder's own
//! serverbound movement), the absolute time since the recording began, the
//! packet id and the raw payload — exactly the bytes the connection carried,
//! so playback decodes with the same protocol code as live play.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const MAGIC: &[u8; 8] = b"VOLTRPL1";
/// A clientbound configuration-phase packet (registries, tags).
pub const PHASE_CONFIG: u8 = 0;
/// A clientbound play packet.
pub const PHASE_PLAY: u8 = 1;
/// A serverbound movement packet: the recorder's own position.
pub const PHASE_SELF: u8 = 2;
/// A single packet payload larger than this is corrupt, not a replay.
const MAX_PAYLOAD: u32 = 16 * 1024 * 1024;

fn write_var(out: &mut Vec<u8>, mut v: u32) {
    loop {
        if v < 0x80 {
            out.push(v as u8);
            return;
        }
        out.push((v & 0x7f) as u8 | 0x80);
        v >>= 7;
    }
}

fn read_var(input: &mut impl Read) -> Option<u32> {
    let mut v = 0u32;
    let mut shift = 0u32;
    loop {
        let mut byte = [0u8; 1];
        input.read_exact(&mut byte).ok()?;
        v |= u32::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Some(v);
        }
        shift += 7;
        if shift >= 35 {
            return None;
        }
    }
}

/// What the fixed header carries; `list` reads it without decompressing.
#[derive(Clone)]
pub struct Meta {
    pub path: PathBuf,
    pub file_name: String,
    pub size: u64,
    pub duration_ms: u32,
    /// Unix seconds when the recording began.
    pub created: u64,
    pub uuid: u128,
    pub name: String,
    pub server: String,
}

/// At most `limit` bytes of `s`, cut on a character boundary.
fn clip(s: &str, limit: usize) -> &[u8] {
    let mut end = s.len().min(limit);
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s.as_bytes()[..end]
}

fn write_header(file: &mut File, created: u64, uuid: u128, name: &str, server: &str) -> std::io::Result<()> {
    let mut head = Vec::with_capacity(64);
    head.extend_from_slice(MAGIC);
    head.extend_from_slice(&0u32.to_le_bytes()); // duration, patched on finish
    head.extend_from_slice(&created.to_le_bytes());
    head.extend_from_slice(&uuid.to_le_bytes());
    let name = clip(name, 64);
    head.extend_from_slice(&(name.len() as u16).to_le_bytes());
    head.extend_from_slice(name);
    let server = clip(server, 255);
    head.extend_from_slice(&(server.len() as u16).to_le_bytes());
    head.extend_from_slice(server);
    file.write_all(&head)
}

fn read_header(input: &mut impl Read) -> Option<(u32, u64, u128, String, String)> {
    let mut magic = [0u8; 8];
    input.read_exact(&mut magic).ok()?;
    if &magic != MAGIC {
        return None;
    }
    let mut n4 = [0u8; 4];
    let mut n8 = [0u8; 8];
    let mut n16 = [0u8; 16];
    input.read_exact(&mut n4).ok()?;
    let duration = u32::from_le_bytes(n4);
    input.read_exact(&mut n8).ok()?;
    let created = u64::from_le_bytes(n8);
    input.read_exact(&mut n16).ok()?;
    let uuid = u128::from_le_bytes(n16);
    let mut text = |limit: usize| -> Option<String> {
        let mut n2 = [0u8; 2];
        input.read_exact(&mut n2).ok()?;
        let len = usize::from(u16::from_le_bytes(n2));
        if len > limit {
            return None;
        }
        let mut bytes = vec![0u8; len];
        input.read_exact(&mut bytes).ok()?;
        String::from_utf8(bytes).ok()
    };
    let name = text(64)?;
    let server = text(255)?;
    Some((duration, created, uuid, name, server))
}

/// Read one file's header; None when it is not a Volt replay.
pub fn read_meta(path: &Path) -> Option<Meta> {
    let file = File::open(path).ok()?;
    let size = file.metadata().ok()?.len();
    let mut input = BufReader::new(file);
    let (duration_ms, created, uuid, name, server) = read_header(&mut input)?;
    Some(Meta {
        file_name: path.file_name()?.to_string_lossy().into_owned(),
        path: path.to_owned(),
        size,
        duration_ms,
        created,
        uuid,
        name,
        server,
    })
}

/// Every replay in the folder, newest first.
pub fn list(dir: &Path) -> Vec<Meta> {
    let mut out: Vec<Meta> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "vrpl"))
        .filter_map(|e| read_meta(&e.path()))
        .collect();
    out.sort_by(|a, b| b.created.cmp(&a.created));
    out
}

/// `12:34` / `1:02:34` for the lists and the playback bar.
pub fn format_time(ms: u32) -> String {
    let s = ms / 1000;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

struct Active {
    out: flate2::write::GzEncoder<File>,
    path: PathBuf,
    started: Instant,
    last_ms: u32,
}

/// Shared between the network thread (clientbound frames) and the game
/// thread (the player's own movement packets); a write failure quietly
/// stops the recording rather than the session.
pub struct Recorder {
    inner: Mutex<Option<Active>>,
}

impl Recorder {
    pub fn create(dir: &Path, server: &str, name: &str, uuid: u128) -> Option<Arc<Recorder>> {
        std::fs::create_dir_all(dir).ok()?;
        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();
        let label: String = server
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
            .take(48)
            .collect();
        let path = dir.join(format!("{label}-{created}.vrpl"));
        let mut file = File::create(&path).ok()?;
        write_header(&mut file, created, uuid, name, server).ok()?;
        Some(Arc::new(Recorder {
            inner: Mutex::new(Some(Active {
                out: flate2::write::GzEncoder::new(file, flate2::Compression::fast()),
                path,
                started: Instant::now(),
                last_ms: 0,
            })),
        }))
    }
    pub fn record(&self, phase: u8, id: i32, payload: &[u8]) {
        if id < 0 {
            return;
        }
        let mut guard = self.inner.lock().unwrap();
        let Some(active) = guard.as_mut() else { return };
        let at = active.started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32;
        active.last_ms = at;
        let mut head = Vec::with_capacity(16);
        head.push(phase);
        write_var(&mut head, at);
        write_var(&mut head, id as u32);
        write_var(&mut head, payload.len() as u32);
        if active
            .out
            .write_all(&head)
            .and_then(|()| active.out.write_all(payload))
            .is_err()
        {
            *guard = None;
        }
    }
    /// Close the stream and patch the duration into the header.
    pub fn finish(&self) {
        let Some(active) = self.inner.lock().unwrap().take() else { return };
        let (path, duration) = (active.path.clone(), active.last_ms);
        let closed = active.out.finish().map(|mut f| f.flush());
        if closed.is_ok()
            && let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(&path)
            && file.seek(SeekFrom::Start(8)).is_ok()
        {
            let _ = file.write_all(&duration.to_le_bytes());
        }
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        self.finish();
    }
}

/// Reads a replay's records back, in order.
pub struct Feed {
    input: flate2::read::GzDecoder<BufReader<File>>,
    pub meta: Meta,
}

impl Feed {
    pub fn open(path: &Path) -> Option<Feed> {
        let meta = read_meta(path)?;
        let mut input = BufReader::new(File::open(path).ok()?);
        read_header(&mut input)?;
        Some(Feed {
            input: flate2::read::GzDecoder::new(input),
            meta,
        })
    }
    /// The next record: (phase, milliseconds since start, packet id, payload).
    pub fn next(&mut self) -> Option<(u8, u32, i32, Vec<u8>)> {
        let mut phase = [0u8; 1];
        self.input.read_exact(&mut phase).ok()?;
        let at = read_var(&mut self.input)?;
        let id = read_var(&mut self.input)?;
        let len = read_var(&mut self.input)?;
        if len > MAX_PAYLOAD || id > i32::MAX as u32 {
            return None;
        }
        let mut payload = vec![0u8; len as usize];
        self.input.read_exact(&mut payload).ok()?;
        Some((phase[0], at, id as i32, payload))
    }
}

/// `seek_ms` empty.
pub const NO_SEEK: u32 = u32::MAX;

/// Playback control, shared between the feeder thread and the game's
/// replay bar. Times are milliseconds into the recording.
pub struct Control {
    pub paused: AtomicBool,
    /// 25..=800, percent of real time.
    pub speed_pct: AtomicU32,
    /// Where playback currently is; the feeder publishes it.
    pub time_ms: AtomicU32,
    pub duration_ms: AtomicU32,
    /// A requested jump, `NO_SEEK` when none. Backwards jumps restart the
    /// stream and fast-forward, like the original Replay Mod's seeking.
    pub seek_ms: AtomicU32,
    pub finished: AtomicBool,
    /// The viewer asked to leave the replay.
    pub exit: AtomicBool,
    /// True: the camera rides the recorded point of view. False: free
    /// camera, with the recorder's body shown as its own player.
    pub pov: AtomicBool,
}

impl Control {
    pub fn new(duration_ms: u32) -> Arc<Control> {
        Arc::new(Control {
            paused: AtomicBool::new(false),
            speed_pct: AtomicU32::new(100),
            time_ms: AtomicU32::new(0),
            duration_ms: AtomicU32::new(duration_ms),
            seek_ms: AtomicU32::new(NO_SEEK),
            finished: AtomicBool::new(false),
            exit: AtomicBool::new(false),
            pov: AtomicBool::new(true),
        })
    }
    pub fn request_seek(&self, ms: u32) {
        let duration = self.duration_ms.load(Ordering::Relaxed);
        self.seek_ms.store(ms.min(duration), Ordering::Relaxed);
        self.finished.store(false, Ordering::Relaxed);
    }
}

/// The recorder's own movement, rebuilt from its recorded serverbound
/// packets during playback: a rolling state the move packets update.
#[derive(Clone, Copy, Default)]
pub struct SelfPose {
    pub position: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub known: bool,
}

/// A small pose history the game interpolates the recorded view from.
pub type PoseBuffer = VecDeque<(u32, [f64; 3], f32, f32)>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_record_and_feed() {
        let dir = std::env::temp_dir().join(format!("volt-replay-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let recorder = Recorder::create(&dir, "play.example.org:25565", "Steve", 7).unwrap();
        recorder.record(PHASE_CONFIG, 5, b"reg");
        recorder.record(PHASE_PLAY, 44, &[1, 2, 3, 4]);
        recorder.record(PHASE_SELF, 29, &[9; 300]);
        recorder.finish();
        let listed = list(&dir);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Steve");
        assert_eq!(listed[0].uuid, 7);
        assert_eq!(listed[0].server, "play.example.org:25565");
        let mut feed = Feed::open(&listed[0].path).unwrap();
        let (phase, _, id, payload) = feed.next().unwrap();
        assert_eq!((phase, id, payload.as_slice()), (PHASE_CONFIG, 5, &b"reg"[..]));
        let (phase, _, id, payload) = feed.next().unwrap();
        assert_eq!((phase, id, payload.len()), (PHASE_PLAY, 44, 4));
        let (phase, _, id, payload) = feed.next().unwrap();
        assert_eq!((phase, id, payload.len()), (PHASE_SELF, 29, 300));
        assert!(feed.next().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[test]
    fn format_time_reads_like_a_clock() {
        assert_eq!(format_time(0), "0:00");
        assert_eq!(format_time(754_000), "12:34");
        assert_eq!(format_time(3_754_000), "1:02:34");
    }
}
