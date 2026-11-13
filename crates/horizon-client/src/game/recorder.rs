//! Volt Recorder: live gameplay capture to MP4, on-device. Video frames
//! the renderer reads back (already composed with or without the HUD and
//! the touch controls — `Renderer::video_layers`) are downscaled to the
//! chosen resolution and pushed through the hardware H.264 encoder; the
//! mixer's output tap supplies game audio for an AAC track. A clip mode
//! keeps only the last 30 seconds — encoded, in RAM — and saves on demand.
//! Everything runs on one worker thread; when the recorder is off, nothing
//! here runs at all.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};

use super::video_export::Frame;

/// The resolution ladder (output height; width follows the screen aspect).
pub const HEIGHTS: [u32; 5] = [360, 480, 720, 1080, 1440];
/// The frame-rate ladder.
pub const RATES: [u32; 5] = [30, 60, 90, 120, 144];
/// The clip ring keeps this much tail.
pub const CLIP_SECONDS: u64 = 30;

/// Player-facing recorder settings, persisted as `config/volt_recorder.txt`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Settings {
    /// Output height, one of [`HEIGHTS`].
    pub height: u32,
    /// Capture rate, one of [`RATES`].
    pub fps: u32,
    /// Record the on-screen touch controls.
    pub controls: bool,
    /// Record the game HUD.
    pub hud: bool,
    /// Clip mode: keep only the last 30 s, saved when asked.
    pub clip: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            height: 720,
            fps: 60,
            controls: false,
            hud: true,
            clip: false,
        }
    }
}

impl Settings {
    pub fn load(home: &Path) -> Settings {
        let mut s = Settings::default();
        let Ok(text) = std::fs::read_to_string(home.join("config/volt_recorder.txt")) else {
            return s;
        };
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "height" => {
                    if let Ok(v) = value.parse() {
                        if HEIGHTS.contains(&v) {
                            s.height = v;
                        }
                    }
                }
                "fps" => {
                    if let Ok(v) = value.parse() {
                        if RATES.contains(&v) {
                            s.fps = v;
                        }
                    }
                }
                "controls" => s.controls = value == "true",
                "hud" => s.hud = value == "true",
                "clip" => s.clip = value == "true",
                _ => {}
            }
        }
        s
    }
    pub fn save(&self, home: &Path) {
        let dir = home.join("config");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(
            dir.join("volt_recorder.txt"),
            format!(
                "height={}\nfps={}\ncontrols={}\nhud={}\nclip={}\n",
                self.height, self.fps, self.controls, self.hud, self.clip
            ),
        );
    }
    /// The H.264 bitrate for this resolution and rate (bits per second):
    /// 0.09 bits per pixel per frame, the ballpark phone screen recorders
    /// use, with a floor so tiny settings stay watchable.
    pub fn bitrate(&self, width: u32) -> u32 {
        let bpp = 0.09f64;
        ((f64::from(width) * f64::from(self.height) * f64::from(self.fps) * bpp) as u32)
            .clamp(1_200_000, 50_000_000)
    }
    /// The video width for a screen aspect ratio, kept even.
    pub fn width_for(&self, screen: [u32; 2]) -> u32 {
        let aspect = if screen[1] == 0 {
            16.0 / 9.0
        } else {
            f64::from(screen[0]) / f64::from(screen[1])
        };
        (((f64::from(self.height) * aspect).round() as u32) & !1).max(64)
    }
    /// "~NN MB/min" for the settings page (video + the 128 kb/s AAC track).
    pub fn mb_per_min(&self, screen: [u32; 2]) -> f64 {
        let bits = f64::from(self.bitrate(self.width_for(screen))) + 128_000.0;
        bits * 60.0 / 8.0 / 1_000_000.0
    }
    /// Every ladder height records on every screen: the renderer scales
    /// each capture on the GPU to the requested output size, upsampling
    /// included, before it is read back.
    pub fn height_supported(height: u32, _screen: [u32; 2]) -> bool {
        HEIGHTS.contains(&height)
    }
}

/// A running recording (or armed clip ring).
pub struct Recording {
    sender: Option<SyncSender<Frame>>,
    stop: Arc<AtomicBool>,
    /// Clip mode: raised to save the ring's last 30 s now.
    clip_now: Arc<AtomicBool>,
    /// Microseconds recorded so far (the worker updates it for the HUD).
    recorded_us: Arc<AtomicU64>,
    worker: Option<std::thread::JoinHandle<String>>,
    pub settings: Settings,
    pub started: std::time::Instant,
    /// Where the MP4(s) land.
    pub dir: PathBuf,
}

impl Recording {
    /// Starts the worker and the game-audio tap. The caller installs the
    /// renderer's `video_sink` with [`Recording::sender`] and sets
    /// `video_layers`/`video_interval` to match `settings`.
    pub fn start(dir: PathBuf, settings: Settings) -> Recording {
        // Deep enough to ride out encoder hiccups without dropping frames
        // at the renderer (pooled buffers keep the memory bounded).
        let (sender, frames) = std::sync::mpsc::sync_channel::<Frame>(8);
        let (audio_tx, audio_rx) = std::sync::mpsc::sync_channel(64);
        horizon_client::audio::tap::install(audio_tx);
        let stop = Arc::new(AtomicBool::new(false));
        let clip_now = Arc::new(AtomicBool::new(false));
        let recorded_us = Arc::new(AtomicU64::new(0));
        let (stopped, clip_flag, us) = (stop.clone(), clip_now.clone(), recorded_us.clone());
        let (out_dir, opts) = (dir.clone(), settings);
        let worker = std::thread::Builder::new()
            .name("volt-recorder".into())
            .spawn(move || {
                let note = work(&frames, &audio_rx, &stopped, &clip_flag, &us, &out_dir, opts);
                horizon_client::audio::tap::clear();
                note
            })
            .ok();
        if worker.is_none() {
            horizon_client::audio::tap::clear();
        }
        Recording {
            sender: worker.as_ref().map(|_| sender),
            stop,
            clip_now,
            recorded_us,
            worker,
            settings,
            started: std::time::Instant::now(),
            dir,
        }
    }
    /// The renderer's capture closure handle.
    pub fn sender(&self) -> Option<SyncSender<Frame>> {
        self.sender.clone()
    }
    /// Clip mode: save the last 30 seconds now (no-op otherwise).
    pub fn save_clip(&self) {
        self.clip_now.store(true, Ordering::Relaxed);
    }
    /// Seconds captured so far (clip mode: seconds currently in the ring).
    pub fn recorded_seconds(&self) -> f64 {
        self.recorded_us.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }
    /// Stops, finalizes the file and returns the one-line outcome.
    pub fn finish(&mut self) -> String {
        self.stop.store(true, Ordering::Relaxed);
        self.sender = None;
        horizon_client::audio::tap::clear();
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .unwrap_or_else(|_| "Recording failed: the encoder thread panicked.".to_string()),
            None => "The recorder never started.".to_string(),
        }
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        if self.worker.is_some() {
            let note = self.finish();
            log::info!("Volt Recorder: {note}");
        }
    }
}

/// A file name like `volt-2026-10-31-17305.mp4`, from the wall clock.
fn file_name(prefix: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Civil date from days since epoch (Howard Hinnant's algorithm).
    let days = (now / 86_400) as i64;
    let secs = now % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let (d, m) = (doy - (153 * mp + 2) / 5 + 1, if mp < 10 { mp + 3 } else { mp - 9 });
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{prefix}-{y:04}-{m:02}-{d:02}-{:02}{:02}{:02}.mp4",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

#[cfg(not(target_os = "android"))]
fn work(
    frames: &Receiver<Frame>,
    audio: &Receiver<horizon_client::audio::tap::Chunk>,
    stop: &AtomicBool,
    _clip_now: &AtomicBool,
    _recorded_us: &AtomicU64,
    _dir: &Path,
    _settings: Settings,
) -> String {
    // Drain so the renderer and the mixer never block.
    while !stop.load(Ordering::Relaxed) {
        let got = frames.recv_timeout(std::time::Duration::from_millis(250));
        while audio.try_recv().is_ok() {}
        if matches!(got, Err(std::sync::mpsc::RecvTimeoutError::Disconnected)) {
            break;
        }
    }
    "Recording needs Android's hardware encoder.".to_string()
}

#[cfg(target_os = "android")]
fn work(
    frames: &Receiver<Frame>,
    audio: &Receiver<horizon_client::audio::tap::Chunk>,
    stop: &AtomicBool,
    clip_now: &AtomicBool,
    recorded_us: &AtomicU64,
    dir: &Path,
    settings: Settings,
) -> String {
    match android::record(frames, audio, stop, clip_now, recorded_us, dir, settings) {
        Ok(note) => note,
        Err(e) => format!("Recording failed: {e}"),
    }
}

#[cfg(target_os = "android")]
mod android {
    use super::super::video_export::{Frame, downscale, media};
    use super::Settings;
    use std::collections::VecDeque;
    use std::ffi::c_void;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::mpsc::{Receiver, RecvTimeoutError};
    use std::time::Duration;

    /// One encoded sample held for the muxer (streaming: until both tracks
    /// exist; clip ring: the whole 30 s tail).
    struct Packet {
        data: Vec<u8>,
        pts_us: i64,
        flags: u32,
        key: bool,
    }

    /// `AMEDIACODEC_BUFFER_FLAG_KEY_FRAME` (a.k.a. sync sample).
    const FLAG_KEY: u32 = 1;

    /// Hands a consumed frame's pixel vector back to the renderer's pool,
    /// so the render thread never allocates multi-megabyte buffers
    /// mid-game while recording.
    fn recycle(pixels: Vec<[u8; 4]>) {
        if pixels.capacity() == 0 {
            return;
        }
        let mut pool = horizon_client::gfx::video_pixel_pool()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if pool.len() < 8 {
            pool.push(pixels);
        }
    }

    /// One MediaCodec encoder plus its pending output packets and track
    /// bookkeeping.
    struct Track {
        codec: *mut c_void,
        /// The output format once INFO_FORMAT_CHANGED delivered it; kept
        /// alive until the worker ends (the muxer reads it on addTrack).
        format: *mut c_void,
        /// Held encoded samples (pre-start, or the clip ring).
        packets: VecDeque<Packet>,
        /// The muxer track index once added.
        track: isize,
        saw_eos: bool,
    }

    impl Track {
        fn new(codec: *mut c_void) -> Track {
            Track {
                codec,
                format: std::ptr::null_mut(),
                packets: VecDeque::new(),
                track: -1,
                saw_eos: false,
            }
        }
        /// Pulls every ready output packet into `packets`.
        fn drain(&mut self, lib: &media::Lib, timeout_us: i64) -> Result<(), String> {
            loop {
                let mut info = media::BufferInfo {
                    offset: 0,
                    size: 0,
                    presentation_time_us: 0,
                    flags: 0,
                };
                let got = unsafe { (lib.dq_out)(self.codec, &mut info, timeout_us) };
                match got {
                    media::INFO_TRY_AGAIN => return Ok(()),
                    media::INFO_FORMAT_CHANGED => {
                        if self.format.is_null() {
                            self.format = unsafe { (lib.out_fmt)(self.codec) };
                        }
                    }
                    media::INFO_BUFFERS_CHANGED => {}
                    i if i >= 0 => {
                        let i = i as usize;
                        if info.flags & media::FLAG_CODEC_CONFIG == 0 && info.size > 0 {
                            let mut cap = 0usize;
                            let buf = unsafe { (lib.out_buf)(self.codec, i, &mut cap) };
                            if !buf.is_null() {
                                let at = info.offset.max(0) as usize;
                                let len = (info.size as usize).min(cap.saturating_sub(at));
                                let data = unsafe {
                                    std::slice::from_raw_parts(buf.add(at), len)
                                }
                                .to_vec();
                                self.packets.push_back(Packet {
                                    data,
                                    pts_us: info.presentation_time_us,
                                    flags: info.flags,
                                    key: info.flags & FLAG_KEY != 0,
                                });
                            }
                        }
                        unsafe { (lib.rel_out)(self.codec, i, false) };
                        if info.flags & media::FLAG_END_OF_STREAM != 0 {
                            self.saw_eos = true;
                            return Ok(());
                        }
                    }
                    _ => return Err(format!("dequeueOutputBuffer said {got}")),
                }
            }
        }
        /// An input buffer index, draining while the encoder is full.
        fn input_slot(&mut self, lib: &media::Lib) -> Result<usize, String> {
            for _ in 0..200 {
                let idx = unsafe { (lib.dq_in)(self.codec, 20_000) };
                if idx >= 0 {
                    return Ok(idx as usize);
                }
                self.drain(lib, 0)?;
            }
            Err("the encoder stopped accepting input".to_string())
        }
        /// Queues an end-of-stream mark and waits for it to come back.
        fn finish(&mut self, lib: &media::Lib, last_pts: i64) -> Result<(), String> {
            let idx = self.input_slot(lib)?;
            let rc = unsafe {
                (lib.q_in)(
                    self.codec,
                    idx,
                    0,
                    0,
                    last_pts.max(0) as u64,
                    media::FLAG_END_OF_STREAM,
                )
            };
            if rc != 0 {
                return Err(format!("queueInputBuffer (EOS) said {rc}"));
            }
            for _ in 0..200 {
                self.drain(lib, 50_000)?;
                if self.saw_eos {
                    return Ok(());
                }
            }
            Err("the encoder never flushed".to_string())
        }
    }

    /// Opens the H.264 encoder for `w`×`h` at the configured rate.
    fn open_video(
        lib: &media::Lib,
        w: usize,
        h: usize,
        settings: &Settings,
        bitrate: u32,
    ) -> Result<(Track, bool, usize, usize), String> {
        for color in media::COLOR_FORMATS {
            let c = unsafe { (lib.create)(c"video/avc".as_ptr()) };
            if c.is_null() {
                return Err("this device has no H.264 encoder".to_string());
            }
            let fmt = unsafe { (lib.fmt_new)() };
            unsafe {
                (lib.fmt_str)(fmt, c"mime".as_ptr(), c"video/avc".as_ptr());
                (lib.fmt_i32)(fmt, c"width".as_ptr(), w as i32);
                (lib.fmt_i32)(fmt, c"height".as_ptr(), h as i32);
                (lib.fmt_i32)(fmt, c"color-format".as_ptr(), color);
                (lib.fmt_i32)(fmt, c"bitrate".as_ptr(), bitrate as i32);
                (lib.fmt_i32)(fmt, c"frame-rate".as_ptr(), settings.fps as i32);
                // Clip trimming cuts at sync samples: one every second.
                (lib.fmt_i32)(fmt, c"i-frame-interval".as_ptr(), 1);
            }
            let ok = unsafe {
                (lib.configure)(
                    c,
                    fmt,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    media::CONFIGURE_ENCODE,
                ) == 0
                    && (lib.start)(c) == 0
            };
            unsafe { (lib.fmt_del)(fmt) };
            if ok {
                // The encoder's real input layout; most devices pad it.
                let (mut stride, mut slice) = (w as i32, h as i32);
                let in_fmt = unsafe { (lib.in_fmt)(c) };
                if !in_fmt.is_null() {
                    unsafe {
                        (lib.fmt_get)(in_fmt, c"stride".as_ptr(), &mut stride);
                        (lib.fmt_get)(in_fmt, c"slice-height".as_ptr(), &mut slice);
                        (lib.fmt_del)(in_fmt);
                    }
                }
                return Ok((
                    Track::new(c),
                    color == 21,
                    stride.max(w as i32) as usize,
                    slice.max(h as i32) as usize,
                ));
            }
            unsafe { (lib.delete)(c) };
        }
        Err("the H.264 encoder rejected both YUV layouts".to_string())
    }

    /// Opens the AAC-LC encoder for the mixer's rate and channel count.
    fn open_audio(lib: &media::Lib, rate: u32, channels: u32) -> Option<Track> {
        let c = unsafe { (lib.create)(c"audio/mp4a-latm".as_ptr()) };
        if c.is_null() {
            return None;
        }
        let fmt = unsafe { (lib.fmt_new)() };
        unsafe {
            (lib.fmt_str)(fmt, c"mime".as_ptr(), c"audio/mp4a-latm".as_ptr());
            (lib.fmt_i32)(fmt, c"sample-rate".as_ptr(), rate as i32);
            (lib.fmt_i32)(fmt, c"channel-count".as_ptr(), channels as i32);
            (lib.fmt_i32)(fmt, c"bitrate".as_ptr(), 128_000);
            // AACObjectLC.
            (lib.fmt_i32)(fmt, c"aac-profile".as_ptr(), 2);
        }
        let ok = unsafe {
            (lib.configure)(
                c,
                fmt,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                media::CONFIGURE_ENCODE,
            ) == 0
                && (lib.start)(c) == 0
        };
        unsafe { (lib.fmt_del)(fmt) };
        if ok {
            Some(Track::new(c))
        } else {
            unsafe { (lib.delete)(c) };
            None
        }
    }

    /// The open muxer of a streaming (non-clip) recording.
    struct Stream {
        muxer: *mut c_void,
        fd: i32,
        v_track: isize,
        a_track: isize,
    }

    impl Stream {
        /// Opens the file and registers both tracks. Call once the video
        /// format exists (and the audio format, when there is audio).
        fn open(
            lib: &media::Lib,
            path: &Path,
            video: &Track,
            audio: Option<&Track>,
        ) -> Result<Stream, String> {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
            let fd = {
                use std::os::fd::IntoRawFd;
                file.into_raw_fd()
            };
            let muxer = unsafe { (lib.mux_new)(fd, media::MUXER_MPEG_4) };
            if muxer.is_null() {
                unsafe { media::close(fd) };
                return Err("the MP4 muxer would not open the file".to_string());
            }
            let v_track = unsafe { (lib.mux_track)(muxer, video.format) };
            if v_track < 0 {
                unsafe {
                    (lib.mux_del)(muxer);
                    media::close(fd);
                }
                return Err("the muxer rejected the video track".to_string());
            }
            let a_track = match audio {
                Some(a) if !a.format.is_null() => unsafe { (lib.mux_track)(muxer, a.format) },
                _ => -1,
            };
            if unsafe { (lib.mux_start)(muxer) } != 0 {
                unsafe {
                    (lib.mux_del)(muxer);
                    media::close(fd);
                }
                return Err("the muxer would not start".to_string());
            }
            Ok(Stream {
                muxer,
                fd,
                v_track,
                a_track,
            })
        }
        /// Writes and forgets everything both tracks have held back.
        fn flush(&self, lib: &media::Lib, video: &mut Track, audio: Option<&mut Track>) {
            for p in video.packets.drain(..) {
                self.write(lib, self.v_track, &p);
            }
            if self.a_track >= 0 {
                if let Some(a) = audio {
                    for p in a.packets.drain(..) {
                        self.write(lib, self.a_track, &p);
                    }
                }
            } else if let Some(a) = audio {
                // Audio opened too late for this file: drop it.
                a.packets.clear();
            }
        }
        fn write(&self, lib: &media::Lib, track: isize, p: &Packet) {
            let info = media::BufferInfo {
                offset: 0,
                size: p.data.len() as i32,
                presentation_time_us: p.pts_us.max(0),
                flags: p.flags & FLAG_KEY,
            };
            unsafe { (lib.mux_write)(self.muxer, track as usize, p.data.as_ptr(), &info) };
        }
        fn close(self, lib: &media::Lib) {
            unsafe {
                (lib.mux_stop)(self.muxer);
                (lib.mux_del)(self.muxer);
                media::close(self.fd);
            }
        }
    }

    /// Writes the held packets of both tracks into one MP4. `base_pts`
    /// subtracts so the file starts at zero (the clip ring's start).
    fn mux_out(
        lib: &media::Lib,
        path: &Path,
        video: &mut Track,
        audio: Option<&mut Track>,
        base_pts: i64,
    ) -> Result<(), String> {
        if video.format.is_null() {
            return Err("no encoded video arrived".to_string());
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
        let fd = {
            use std::os::fd::IntoRawFd;
            file.into_raw_fd()
        };
        let muxer = unsafe { (lib.mux_new)(fd, media::MUXER_MPEG_4) };
        if muxer.is_null() {
            unsafe { media::close(fd) };
            return Err("the MP4 muxer would not open the file".to_string());
        }
        let finish = |muxer: *mut c_void, fd, started: bool, r: Result<(), String>| {
            unsafe {
                if started {
                    (lib.mux_stop)(muxer);
                }
                (lib.mux_del)(muxer);
                media::close(fd);
            }
            r
        };
        let v_track = unsafe { (lib.mux_track)(muxer, video.format) };
        if v_track < 0 {
            return finish(muxer, fd, false, Err("the muxer rejected the video track".into()));
        }
        let a_track = match &audio {
            Some(a) if !a.format.is_null() && !a.packets.is_empty() => {
                unsafe { (lib.mux_track)(muxer, a.format) }
            }
            _ => -1,
        };
        if unsafe { (lib.mux_start)(muxer) } != 0 {
            return finish(muxer, fd, false, Err("the muxer would not start".into()));
        }
        let mut write = |track: isize, p: &Packet| {
            let info = media::BufferInfo {
                offset: 0,
                size: p.data.len() as i32,
                presentation_time_us: (p.pts_us - base_pts).max(0),
                flags: p.flags & FLAG_KEY,
            };
            unsafe { (lib.mux_write)(muxer, track as usize, p.data.as_ptr(), &info) };
        };
        for p in &video.packets {
            write(v_track, p);
        }
        if a_track >= 0 && let Some(a) = audio {
            for p in a.packets.iter().filter(|p| p.pts_us >= base_pts) {
                write(a_track, p);
            }
        }
        finish(muxer, fd, true, Ok(()))
    }

    pub(super) fn record(
        frames: &Receiver<Frame>,
        audio_rx: &Receiver<horizon_client::audio::tap::Chunk>,
        stop: &AtomicBool,
        clip_now: &AtomicBool,
        recorded_us: &AtomicU64,
        dir: &Path,
        settings: Settings,
    ) -> Result<String, String> {
        // The first frame fixes the source size and the output aspect.
        let first = loop {
            if stop.load(Ordering::Relaxed) {
                return Err("stopped before any frame arrived".to_string());
            }
            match frames.recv_timeout(Duration::from_millis(250)) {
                Ok(f) => break f,
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("no frames arrived".to_string());
                }
            }
        };
        let src = first.size;
        let out_h = settings.height.min(src[1] & !1).max(64) & !1;
        let out_w = (settings.width_for(src).min(src[0] & !1)).max(64) & !1;
        let lib = media::Lib::open()?;
        let (mut video, semi_planar, stride, slice) = open_video(
            &lib,
            out_w as usize,
            out_h as usize,
            &settings,
            settings.bitrate(out_w),
        )?;
        let close_codec = |t: &mut Track| unsafe {
            (lib.stop)(t.codec);
            (lib.delete)(t.codec);
            if !t.format.is_null() {
                (lib.fmt_del)(t.format);
            }
        };
        // The AAC track opens lazily on the first mixed chunk (its rate and
        // channel count); silence stays a video-only file.
        let mut audio: Option<Track> = None;
        let mut audio_pts_samples: u64 = 0;
        let mut audio_rate = 48_000u32;
        let mut audio_channels = 2u32;
        let mut pcm_pending: VecDeque<i16> = VecDeque::new();
        let clip_mode = settings.clip;
        let ring_us = super::CLIP_SECONDS as i64 * 1_000_000;
        let mut clips_saved = 0u32;
        let mut frames_in = 0u64;
        let mut last_pts = 0i64;
        let mut pending = Some(first);
        let mut result: Result<(), String> = Ok(());
        // Streaming mode writes into the muxer as packets appear; it opens
        // once the video format exists and the audio has had a moment to
        // deliver its first chunk (so the AAC track is not raced out).
        let mut stream: Option<Stream> = None;
        let stream_path = dir.join(super::file_name("volt-rec"));

        let mut input_gone = false;
        'record: loop {
            let next = match pending.take() {
                Some(f) => Some(f),
                None => match frames.recv_timeout(Duration::from_millis(100)) {
                    Ok(f) => Some(f),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => {
                        input_gone = true;
                        None
                    }
                },
            };
            if next.is_none() && (stop.load(Ordering::Relaxed) || input_gone) {
                break;
            }
            // Game audio: convert to interleaved i16 and feed the AAC
            // encoder in input-buffer-sized bites.
            while let Ok(chunk) = audio_rx.try_recv() {
                if audio.is_none() {
                    audio_rate = chunk.rate.max(8000);
                    audio_channels = chunk.channels.clamp(1, 2);
                    audio = open_audio(&lib, audio_rate, audio_channels);
                }
                if audio.is_some() {
                    let step = chunk.channels.max(1) as usize;
                    let keep = audio_channels as usize;
                    for frame in chunk.samples.chunks(step) {
                        for c in 0..keep {
                            let s = frame.get(c.min(step - 1)).copied().unwrap_or(0.0);
                            pcm_pending.push_back((s.clamp(-1., 1.) * 32767.0) as i16);
                        }
                    }
                }
            }
            if let Some(a) = &mut audio {
                while !pcm_pending.is_empty() {
                    let idx = match a.input_slot(&lib) {
                        Ok(i) => i,
                        Err(e) => {
                            result = Err(e);
                            break 'record;
                        }
                    };
                    let mut cap = 0usize;
                    let buf = unsafe { (lib.in_buf)(a.codec, idx, &mut cap) };
                    if buf.is_null() || cap < 2 {
                        break;
                    }
                    let samples = (cap / 2).min(pcm_pending.len());
                    let out = unsafe { std::slice::from_raw_parts_mut(buf.cast::<u8>(), cap) };
                    for i in 0..samples {
                        let s = pcm_pending.pop_front().unwrap_or(0);
                        out[i * 2..i * 2 + 2].copy_from_slice(&s.to_le_bytes());
                    }
                    let pts =
                        audio_pts_samples * 1_000_000 / u64::from(audio_rate * audio_channels);
                    audio_pts_samples += samples as u64;
                    let rc = unsafe { (lib.q_in)(a.codec, idx, 0, samples * 2, pts, 0) };
                    if rc != 0 {
                        result = Err(format!("audio queueInputBuffer said {rc}"));
                        break 'record;
                    }
                    if let Err(e) = a.drain(&lib, 0) {
                        result = Err(e);
                        break 'record;
                    }
                    // Half-full input is fine; wait for more PCM.
                    if pcm_pending.len() < cap / 2 {
                        break;
                    }
                }
            }
            // Video.
            if let Some(frame) = next {
                if frame.size != src || (frames_in > 0 && frame.pts_us <= last_pts) {
                    recycle(frame.pixels);
                    continue;
                }
                let scaled = if out_w == src[0] && out_h == src[1] {
                    frame
                } else {
                    let small = downscale(&frame, out_w, out_h);
                    recycle(frame.pixels);
                    small
                };
                let idx = match video.input_slot(&lib) {
                    Ok(i) => i,
                    Err(e) => {
                        result = Err(e);
                        break;
                    }
                };
                let mut cap = 0usize;
                let buf = unsafe { (lib.in_buf)(video.codec, idx, &mut cap) };
                let needed = stride * slice * 3 / 2;
                if buf.is_null() || cap < needed {
                    result = Err("the video input buffer is too small".to_string());
                    break;
                }
                let out = unsafe { std::slice::from_raw_parts_mut(buf, cap) };
                super::super::video_export::media::fill_yuv(
                    out,
                    &scaled,
                    out_w as usize,
                    out_h as usize,
                    stride,
                    slice,
                    semi_planar,
                );
                let rc = unsafe {
                    (lib.q_in)(video.codec, idx, 0, needed, scaled.pts_us.max(0) as u64, 0)
                };
                if rc != 0 {
                    result = Err(format!("queueInputBuffer said {rc}"));
                    break;
                }
                if let Err(e) = video.drain(&lib, 0) {
                    result = Err(e);
                    break;
                }
                last_pts = scaled.pts_us;
                frames_in += 1;
                recycle(scaled.pixels);
            }
            // Clip mode: trim the ring to the last 30 s, cutting at a sync
            // sample so the saved file always starts decodable.
            if clip_mode {
                loop {
                    let cut = video
                        .packets
                        .iter()
                        .enumerate()
                        .skip(1)
                        .find(|(_, p)| p.key && last_pts - p.pts_us >= ring_us)
                        .map(|(i, _)| i);
                    match cut {
                        Some(i) => {
                            video.packets.drain(..i);
                        }
                        None => break,
                    }
                }
                let base = video.packets.front().map_or(last_pts, |p| p.pts_us);
                if let Some(a) = &mut audio {
                    while a.packets.front().is_some_and(|p| p.pts_us < base - 500_000) {
                        a.packets.pop_front();
                    }
                }
                recorded_us.store((last_pts - base).max(0) as u64, Ordering::Relaxed);
                if clip_now.swap(false, Ordering::Relaxed) && !video.packets.is_empty() {
                    let path = dir.join(super::file_name("volt-clip"));
                    match mux_out(&lib, &path, &mut video, audio.as_mut(), base) {
                        Ok(()) => {
                            clips_saved += 1;
                            log::info!("Volt Recorder: clip saved to {}", path.display());
                        }
                        Err(e) => log::warn!("Volt Recorder: clip failed: {e}"),
                    }
                }
            } else {
                recorded_us.store(last_pts.max(0) as u64, Ordering::Relaxed);
                if stream.is_none()
                    && !video.format.is_null()
                    && (audio.as_ref().is_some_and(|a| !a.format.is_null())
                        || last_pts > 700_000)
                {
                    match Stream::open(&lib, &stream_path, &video, audio.as_ref()) {
                        Ok(s) => stream = Some(s),
                        Err(e) => {
                            result = Err(e);
                            break;
                        }
                    }
                }
                if let Some(s) = &stream {
                    s.flush(&lib, &mut video, audio.as_mut());
                }
            }
        }

        // Flush both encoders, then write what remains.
        if result.is_ok() && frames_in > 0 {
            result = video.finish(&lib, last_pts);
            if let (Ok(()), Some(a)) = (&result, &mut audio) {
                let _ = a.finish(&lib, last_pts);
            }
        }
        let outcome = match (&result, frames_in) {
            (Err(e), _) => Err(e.clone()),
            (Ok(()), 0) => Err("no frames arrived".to_string()),
            (Ok(()), _) if clip_mode => {
                // Stopping the ring saves one last clip of what's left.
                let base = video.packets.front().map_or(0, |p| p.pts_us);
                if video.packets.is_empty() {
                    if clips_saved > 0 {
                        Ok(format!("Saved {clips_saved} clip(s)."))
                    } else {
                        Err("nothing was in the clip ring".to_string())
                    }
                } else {
                    let path = dir.join(super::file_name("volt-clip"));
                    mux_out(&lib, &path, &mut video, audio.as_mut(), base)
                        .map(|()| format!("Saved {} ({} clips).", path.display(), clips_saved + 1))
                }
            }
            (Ok(()), _) => match stream
                .take()
                .map(Ok)
                .unwrap_or_else(|| Stream::open(&lib, &stream_path, &video, audio.as_ref()))
            {
                Ok(s) => {
                    s.flush(&lib, &mut video, audio.as_mut());
                    s.close(&lib);
                    Ok(format!(
                        "Saved {} ({} frames, {:.1} s, {:.0} fps avg).",
                        stream_path.display(),
                        frames_in,
                        last_pts as f64 / 1_000_000.0,
                        frames_in as f64 / (last_pts as f64 / 1_000_000.0).max(0.001)
                    ))
                }
                Err(e) => Err(e),
            },
        };
        if let Some(s) = stream.take() {
            s.close(&lib);
        }
        close_codec(&mut video);
        if let Some(mut a) = audio {
            close_codec(&mut a);
        }
        if outcome.is_err() && !clip_mode {
            let _ = std::fs::remove_file(&stream_path);
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_and_estimates() {
        let dir = std::env::temp_dir().join(format!("volt-rec-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let s = Settings {
            height: 1080,
            fps: 90,
            controls: true,
            hud: false,
            clip: true,
        };
        s.save(&dir);
        assert_eq!(Settings::load(&dir), s);
        // 1080p at the 16:9 default aspect.
        assert_eq!(s.width_for([2400, 1080]), 2400);
        assert!(s.mb_per_min([1920, 1080]) > 10.0);
        // Every ladder height is recordable: the GPU scales the capture.
        assert!(Settings::height_supported(720, [2400, 1080]));
        assert!(Settings::height_supported(1440, [1920, 1080]));
        assert!(!Settings::height_supported(1000, [1920, 1080]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_names_are_sortable() {
        let name = file_name("volt-rec");
        assert!(name.starts_with("volt-rec-20"));
        assert!(name.ends_with(".mp4"));
    }
}
