//! Volt Replay's MP4 export: frames the renderer reads back are converted
//! to YUV, pushed through the device's hardware H.264 encoder and muxed
//! into an `.mp4`, entirely on-device (`libmediandk`, resolved at runtime
//! like the Android DNS bindings). Desktop builds drain the frames and
//! report that export is Android-only.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::time::Duration;

/// One read-back frame: tightly packed 4-byte pixels, row after row, with
/// the presentation time the playback clock assigned.
pub struct Frame {
    pub size: [u32; 2],
    /// The surface was BGRA: red and blue arrive swapped.
    pub bgra: bool,
    pub pts_us: i64,
    pub pixels: Vec<[u8; 4]>,
}

/// Box-averages `frame` down to `w`×`h` (both at most the source size);
/// the Volt Recorder's resolution setting. Runs on the encoder thread.
pub(super) fn downscale(frame: &Frame, w: u32, h: u32) -> Frame {
    let (sw, sh) = (frame.size[0] as usize, frame.size[1] as usize);
    let (w, h) = (w as usize, h as usize);
    if w == 0 || h == 0 || sw == 0 || sh == 0 || (w == sw && h == sh) {
        return Frame {
            size: frame.size,
            bgra: frame.bgra,
            pts_us: frame.pts_us,
            pixels: frame.pixels.clone(),
        };
    }
    let mut pixels = Vec::with_capacity(w * h);
    for oy in 0..h {
        // The source rows/columns this output pixel covers.
        let y0 = oy * sh / h;
        let y1 = (((oy + 1) * sh).div_ceil(h)).clamp(y0 + 1, sh);
        for ox in 0..w {
            let x0 = ox * sw / w;
            let x1 = (((ox + 1) * sw).div_ceil(w)).clamp(x0 + 1, sw);
            let mut sum = [0u32; 4];
            for y in y0..y1 {
                for x in x0..x1 {
                    let px = frame.pixels[y * sw + x];
                    for (acc, c) in sum.iter_mut().zip(px) {
                        *acc += u32::from(c);
                    }
                }
            }
            let n = ((y1 - y0) * (x1 - x0)) as u32;
            pixels.push(sum.map(|c| ((c + n / 2) / n) as u8));
        }
    }
    Frame {
        size: [w as u32, h as u32],
        bgra: frame.bgra,
        pts_us: frame.pts_us,
        pixels,
    }
}

/// The encoder thread and the channel the renderer's capture closure
/// feeds. Dropping it finalizes the file (bounded: the worker polls a stop
/// flag, so a forgotten sender clone cannot hang the join).
pub struct Exporter {
    sender: Option<SyncSender<Frame>>,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<String>>,
}

impl Exporter {
    pub fn start(path: PathBuf) -> Exporter {
        let (sender, receiver) = std::sync::mpsc::sync_channel::<Frame>(2);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let worker = std::thread::Builder::new()
            .name("volt-replay-export".into())
            .spawn(move || encode_worker(receiver, &stopped, path))
            .ok();
        Exporter {
            sender: worker.as_ref().map(|_| sender),
            stop,
            worker,
        }
    }
    /// A handle for the renderer's capture closure; `None` when the
    /// encoder thread could not start.
    pub fn sender(&self) -> Option<SyncSender<Frame>> {
        self.sender.clone()
    }
    /// Stops accepting frames, waits for the file to finalize and returns
    /// the one-line outcome for the HUD.
    pub fn finish(&mut self) -> String {
        self.stop.store(true, Ordering::Relaxed);
        self.sender = None;
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .unwrap_or_else(|_| "Export failed: encoder thread panicked.".to_string()),
            None => "Export never started.".to_string(),
        }
    }
}

impl Drop for Exporter {
    fn drop(&mut self) {
        if self.worker.is_some() {
            let note = self.finish();
            log::info!("Volt Replay export: {note}");
        }
    }
}

/// Receives frames until every sender is gone or `stop` is raised, then
/// finalizes; the `Ok` and `Err` notes both read well on the HUD.
fn encode_worker(receiver: Receiver<Frame>, stop: &AtomicBool, path: PathBuf) -> String {
    match encode(&receiver, stop, &path) {
        Ok(note) => note,
        Err(e) => {
            let _ = std::fs::remove_file(&path);
            format!("Export failed: {e}")
        }
    }
}

/// The next frame, honouring the stop flag; `None` means end of stream.
fn next_frame(receiver: &Receiver<Frame>, stop: &AtomicBool) -> Option<Frame> {
    loop {
        if stop.load(Ordering::Relaxed) {
            return None;
        }
        match receiver.recv_timeout(Duration::from_millis(250)) {
            Ok(frame) => return Some(frame),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return None,
        }
    }
}

#[cfg(not(target_os = "android"))]
fn encode(
    receiver: &Receiver<Frame>,
    stop: &AtomicBool,
    _path: &std::path::Path,
) -> Result<String, String> {
    // Drain so the renderer never blocks; MediaCodec is Android's.
    while next_frame(receiver, stop).is_some() {}
    Err("MP4 export needs Android's hardware encoder.".to_string())
}

#[cfg(target_os = "android")]
fn encode(
    receiver: &Receiver<Frame>,
    stop: &AtomicBool,
    path: &std::path::Path,
) -> Result<String, String> {
    media::encode(receiver, stop, path)
}

#[cfg(target_os = "android")]
pub(super) mod media {
    use super::{Frame, next_frame};
    use std::ffi::{CStr, c_char, c_int, c_void};
    use std::path::Path;
    use std::sync::atomic::AtomicBool;
    use std::sync::mpsc::Receiver;

    /// `AMediaCodecBufferInfo`.
    #[repr(C)]
    pub(in crate::game) struct BufferInfo {
        pub offset: i32,
        pub size: i32,
        pub presentation_time_us: i64,
        pub flags: u32,
    }

    #[link(name = "dl")]
    unsafe extern "C" {
        fn dlopen(name: *const c_char, flags: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        pub(in crate::game) fn close(fd: c_int) -> c_int;
    }

    /// `off_t` in Bionic: 32-bit on armeabi-v7a, 64-bit on arm64-v8a.
    #[cfg(target_pointer_width = "64")]
    pub(in crate::game) type OffT = i64;
    #[cfg(target_pointer_width = "32")]
    pub(in crate::game) type OffT = i32;

    // Every signature matches media/NdkMediaCodec.h, NdkMediaFormat.h and
    // NdkMediaMuxer.h in the NDK (API 21+).
    pub(in crate::game) type CreateEncoder = unsafe extern "C" fn(*const c_char) -> *mut c_void;
    pub(in crate::game) type Configure =
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, u32) -> c_int;
    pub(in crate::game) type Simple = unsafe extern "C" fn(*mut c_void) -> c_int;
    pub(in crate::game) type DequeueIn = unsafe extern "C" fn(*mut c_void, i64) -> isize;
    pub(in crate::game) type GetBuf = unsafe extern "C" fn(*mut c_void, usize, *mut usize) -> *mut u8;
    pub(in crate::game) type QueueIn = unsafe extern "C" fn(*mut c_void, usize, OffT, usize, u64, u32) -> c_int;
    pub(in crate::game) type DequeueOut = unsafe extern "C" fn(*mut c_void, *mut BufferInfo, i64) -> isize;
    pub(in crate::game) type ReleaseOut = unsafe extern "C" fn(*mut c_void, usize, bool) -> c_int;
    pub(in crate::game) type GetFormat = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
    pub(in crate::game) type FormatNew = unsafe extern "C" fn() -> *mut c_void;
    pub(in crate::game) type FormatSetString = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char);
    pub(in crate::game) type FormatSetInt = unsafe extern "C" fn(*mut c_void, *const c_char, i32);
    pub(in crate::game) type FormatGetInt = unsafe extern "C" fn(*mut c_void, *const c_char, *mut i32) -> bool;
    pub(in crate::game) type MuxerNew = unsafe extern "C" fn(c_int, c_int) -> *mut c_void;
    pub(in crate::game) type MuxerAddTrack = unsafe extern "C" fn(*mut c_void, *mut c_void) -> isize;
    pub(in crate::game) type MuxerWrite =
        unsafe extern "C" fn(*mut c_void, usize, *const u8, *const BufferInfo) -> c_int;

    pub(in crate::game) const CONFIGURE_ENCODE: u32 = 1;
    pub(in crate::game) const FLAG_CODEC_CONFIG: u32 = 2;
    pub(in crate::game) const FLAG_END_OF_STREAM: u32 = 4;
    pub(in crate::game) const INFO_TRY_AGAIN: isize = -1;
    pub(in crate::game) const INFO_FORMAT_CHANGED: isize = -2;
    pub(in crate::game) const INFO_BUFFERS_CHANGED: isize = -3;
    /// `COLOR_FormatYUV420SemiPlanar` (NV12) and `...Planar` (I420): tried
    /// in that order, whichever the device's encoder accepts.
    pub(in crate::game) const COLOR_FORMATS: [i32; 2] = [21, 19];
    pub(in crate::game) const MUXER_MPEG_4: c_int = 0;

    pub(in crate::game) struct Lib {
        pub(in crate::game) create: CreateEncoder,
        pub(in crate::game) configure: Configure,
        pub(in crate::game) start: Simple,
        pub(in crate::game) stop: Simple,
        pub(in crate::game) delete: Simple,
        pub(in crate::game) dq_in: DequeueIn,
        pub(in crate::game) in_buf: GetBuf,
        pub(in crate::game) q_in: QueueIn,
        pub(in crate::game) dq_out: DequeueOut,
        pub(in crate::game) out_buf: GetBuf,
        pub(in crate::game) rel_out: ReleaseOut,
        pub(in crate::game) out_fmt: GetFormat,
        pub(in crate::game) in_fmt: GetFormat,
        pub(in crate::game) fmt_new: FormatNew,
        pub(in crate::game) fmt_del: Simple,
        pub(in crate::game) fmt_str: FormatSetString,
        pub(in crate::game) fmt_i32: FormatSetInt,
        pub(in crate::game) fmt_get: FormatGetInt,
        pub(in crate::game) mux_new: MuxerNew,
        pub(in crate::game) mux_track: MuxerAddTrack,
        pub(in crate::game) mux_start: Simple,
        pub(in crate::game) mux_stop: Simple,
        pub(in crate::game) mux_del: Simple,
        pub(in crate::game) mux_write: MuxerWrite,
    }

    impl Lib {
        pub(in crate::game) fn open() -> Result<Lib, String> {
            unsafe {
                let lib = dlopen(c"libmediandk.so".as_ptr(), 2);
                if lib.is_null() {
                    return Err("libmediandk.so is unavailable".to_string());
                }
                let sym = |name: &CStr| -> Result<*mut c_void, String> {
                    let p = dlsym(lib, name.as_ptr());
                    if p.is_null() {
                        Err(format!("{} is missing", name.to_string_lossy()))
                    } else {
                        Ok(p)
                    }
                };
                Ok(Lib {
                    create: std::mem::transmute::<*mut c_void, CreateEncoder>(sym(
                        c"AMediaCodec_createEncoderByType",
                    )?),
                    configure: std::mem::transmute::<*mut c_void, Configure>(sym(
                        c"AMediaCodec_configure",
                    )?),
                    start: std::mem::transmute::<*mut c_void, Simple>(sym(c"AMediaCodec_start")?),
                    stop: std::mem::transmute::<*mut c_void, Simple>(sym(c"AMediaCodec_stop")?),
                    delete: std::mem::transmute::<*mut c_void, Simple>(sym(c"AMediaCodec_delete")?),
                    dq_in: std::mem::transmute::<*mut c_void, DequeueIn>(sym(
                        c"AMediaCodec_dequeueInputBuffer",
                    )?),
                    in_buf: std::mem::transmute::<*mut c_void, GetBuf>(sym(
                        c"AMediaCodec_getInputBuffer",
                    )?),
                    q_in: std::mem::transmute::<*mut c_void, QueueIn>(sym(
                        c"AMediaCodec_queueInputBuffer",
                    )?),
                    dq_out: std::mem::transmute::<*mut c_void, DequeueOut>(sym(
                        c"AMediaCodec_dequeueOutputBuffer",
                    )?),
                    out_buf: std::mem::transmute::<*mut c_void, GetBuf>(sym(
                        c"AMediaCodec_getOutputBuffer",
                    )?),
                    rel_out: std::mem::transmute::<*mut c_void, ReleaseOut>(sym(
                        c"AMediaCodec_releaseOutputBuffer",
                    )?),
                    out_fmt: std::mem::transmute::<*mut c_void, GetFormat>(sym(
                        c"AMediaCodec_getOutputFormat",
                    )?),
                    in_fmt: std::mem::transmute::<*mut c_void, GetFormat>(sym(
                        c"AMediaCodec_getInputFormat",
                    )?),
                    fmt_new: std::mem::transmute::<*mut c_void, FormatNew>(sym(
                        c"AMediaFormat_new",
                    )?),
                    fmt_del: std::mem::transmute::<*mut c_void, Simple>(sym(
                        c"AMediaFormat_delete",
                    )?),
                    fmt_str: std::mem::transmute::<*mut c_void, FormatSetString>(sym(
                        c"AMediaFormat_setString",
                    )?),
                    fmt_i32: std::mem::transmute::<*mut c_void, FormatSetInt>(sym(
                        c"AMediaFormat_setInt32",
                    )?),
                    fmt_get: std::mem::transmute::<*mut c_void, FormatGetInt>(sym(
                        c"AMediaFormat_getInt32",
                    )?),
                    mux_new: std::mem::transmute::<*mut c_void, MuxerNew>(sym(c"AMediaMuxer_new")?),
                    mux_track: std::mem::transmute::<*mut c_void, MuxerAddTrack>(sym(
                        c"AMediaMuxer_addTrack",
                    )?),
                    mux_start: std::mem::transmute::<*mut c_void, Simple>(sym(
                        c"AMediaMuxer_start",
                    )?),
                    mux_stop: std::mem::transmute::<*mut c_void, Simple>(sym(c"AMediaMuxer_stop")?),
                    mux_del: std::mem::transmute::<*mut c_void, Simple>(sym(
                        c"AMediaMuxer_delete",
                    )?),
                    mux_write: std::mem::transmute::<*mut c_void, MuxerWrite>(sym(
                        c"AMediaMuxer_writeSampleData",
                    )?),
                })
            }
        }
    }

    /// The encoder, its layout and the muxer's state while frames stream in.
    struct Encoder<'a> {
        lib: &'a Lib,
        codec: *mut c_void,
        muxer: *mut c_void,
        /// NV12 (true) or I420.
        semi_planar: bool,
        width: usize,
        height: usize,
        stride: usize,
        slice: usize,
        track: isize,
        started: bool,
        saw_eos: bool,
    }

    impl Encoder<'_> {
        /// Moves every ready output packet into the muxer. `timeout_us`
        /// greater than zero waits (the end-of-stream drain).
        fn drain(&mut self, timeout_us: i64) -> Result<(), String> {
            loop {
                let mut info = BufferInfo {
                    offset: 0,
                    size: 0,
                    presentation_time_us: 0,
                    flags: 0,
                };
                let got = unsafe { (self.lib.dq_out)(self.codec, &mut info, timeout_us) };
                match got {
                    INFO_TRY_AGAIN => return Ok(()),
                    INFO_FORMAT_CHANGED => {
                        let fmt = unsafe { (self.lib.out_fmt)(self.codec) };
                        self.track = unsafe { (self.lib.mux_track)(self.muxer, fmt) };
                        unsafe { (self.lib.fmt_del)(fmt) };
                        if self.track < 0 {
                            return Err("the muxer rejected the video track".to_string());
                        }
                        if unsafe { (self.lib.mux_start)(self.muxer) } != 0 {
                            return Err("the muxer would not start".to_string());
                        }
                        self.started = true;
                    }
                    INFO_BUFFERS_CHANGED => {}
                    i if i >= 0 => {
                        let i = i as usize;
                        if self.started && info.flags & FLAG_CODEC_CONFIG == 0 && info.size > 0 {
                            let mut cap = 0usize;
                            let buf = unsafe { (self.lib.out_buf)(self.codec, i, &mut cap) };
                            if !buf.is_null() {
                                unsafe {
                                    (self.lib.mux_write)(self.muxer, self.track as usize, buf,
                                        &info);
                                }
                            }
                        }
                        unsafe { (self.lib.rel_out)(self.codec, i, false) };
                        if info.flags & FLAG_END_OF_STREAM != 0 {
                            self.saw_eos = true;
                            return Ok(());
                        }
                    }
                    _ => return Err(format!("dequeueOutputBuffer said {got}")),
                }
            }
        }

        /// An input buffer index, draining outputs while the encoder is full.
        fn input_slot(&mut self) -> Result<usize, String> {
            for _ in 0..200 {
                let idx = unsafe { (self.lib.dq_in)(self.codec, 20_000) };
                if idx >= 0 {
                    return Ok(idx as usize);
                }
                self.drain(0)?;
            }
            Err("the encoder stopped accepting input".to_string())
        }

        /// Converts the frame to the encoder's YUV layout and queues it.
        fn feed(&mut self, frame: &Frame) -> Result<(), String> {
            let idx = self.input_slot()?;
            let mut cap = 0usize;
            let buf = unsafe { (self.lib.in_buf)(self.codec, idx, &mut cap) };
            if buf.is_null() {
                return Err("getInputBuffer returned nothing".to_string());
            }
            let (w, h, stride, slice) = (self.width, self.height, self.stride, self.slice);
            // The luma plane plus either chroma layout fits in 3/2 luma sizes.
            let needed = stride * slice * 3 / 2;
            if cap < needed {
                return Err(format!("input buffer holds {cap}, frame needs {needed}"));
            }
            let out = unsafe { std::slice::from_raw_parts_mut(buf, cap) };
            fill_yuv(out, frame, w, h, stride, slice, self.semi_planar);
            let rc = unsafe {
                (self.lib.q_in)(self.codec, idx, 0, needed, frame.pts_us.max(0) as u64, 0)
            };
            if rc != 0 {
                return Err(format!("queueInputBuffer said {rc}"));
            }
            self.drain(0)
        }

        /// Queues the end-of-stream mark and waits until it comes back out.
        fn finish(&mut self, last_pts: i64) -> Result<(), String> {
            let idx = self.input_slot()?;
            let rc = unsafe {
                (self.lib.q_in)(
                    self.codec,
                    idx,
                    0,
                    0,
                    last_pts.max(0) as u64,
                    FLAG_END_OF_STREAM,
                )
            };
            if rc != 0 {
                return Err(format!("queueInputBuffer (EOS) said {rc}"));
            }
            for _ in 0..200 {
                self.drain(50_000)?;
                if self.saw_eos {
                    return Ok(());
                }
            }
            Err("the encoder never flushed its last frames".to_string())
        }
    }

    /// Converts `frame`'s packed pixels (cropped to `w`×`h`) into the
    /// encoder's YUV 4:2:0 layout inside `out` (BT.601 limited range).
    pub(in crate::game) fn fill_yuv(
        out: &mut [u8],
        frame: &Frame,
        w: usize,
        h: usize,
        stride: usize,
        slice: usize,
        semi_planar: bool,
    ) {
        let src_w = frame.size[0] as usize;
        let (ri, bi) = if frame.bgra { (2usize, 0usize) } else { (0usize, 2usize) };
        let pixels = &frame.pixels;
        // One luma row from the packed source into `dst` (stride-wide).
        let luma_row = |row: usize, dst: &mut [u8]| {
            let src = &pixels[row * src_w..row * src_w + w];
            for (d, px) in dst[..w].iter_mut().zip(src) {
                let (r, g, b) = (px[ri] as i32, px[1] as i32, px[bi] as i32);
                *d = (((66 * r + 129 * g + 25 * b + 128) >> 8) + 16).clamp(0, 255) as u8;
            }
        };
        // One chroma value pair for the subsampled pixel at (`row`, `col`).
        let uv_at = |row: usize, col: usize| {
            let px = pixels[row * 2 * src_w + col * 2];
            let (r, g, b) = (px[ri] as i32, px[1] as i32, px[bi] as i32);
            let u = (((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128).clamp(0, 255) as u8;
            let v = (((112 * r - 94 * g - 18 * b + 128) >> 8) + 128).clamp(0, 255) as u8;
            (u, v)
        };
        // The convert was the recorder thread's bottleneck above 720p (one
        // scalar pass per pixel): band the disjoint luma and chroma rows
        // across a few cores. Small frames stay single-threaded.
        let bands = if h >= 540 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                .clamp(1, 4)
        } else {
            1
        };
        let (y_plane, chroma) = out.split_at_mut(stride * slice);
        let c_h = h / 2;
        let cs = stride / 2;
        let plane = cs * slice.div_ceil(2);
        std::thread::scope(|scope| {
            let y_rows = h.div_ceil(bands);
            for (band, chunk) in y_plane[..stride * h].chunks_mut(stride * y_rows).enumerate() {
                let mut work = move || {
                    for (r, dst) in chunk.chunks_mut(stride).enumerate() {
                        luma_row(band * y_rows + r, dst);
                    }
                };
                if bands > 1 {
                    scope.spawn(work);
                } else {
                    work();
                }
            }
            if c_h == 0 {
                return;
            }
            let c_rows = c_h.div_ceil(bands);
            if semi_planar {
                for (band, chunk) in chroma[..stride * c_h].chunks_mut(stride * c_rows).enumerate()
                {
                    let mut work = move || {
                        for (r, dst) in chunk.chunks_mut(stride).enumerate() {
                            let row = band * c_rows + r;
                            for col in 0..w / 2 {
                                let (u, v) = uv_at(row, col);
                                dst[col * 2] = u;
                                dst[col * 2 + 1] = v;
                            }
                        }
                    };
                    if bands > 1 {
                        scope.spawn(work);
                    } else {
                        work();
                    }
                }
            } else {
                let (u_plane, v_plane) = chroma.split_at_mut(plane);
                let u_chunks = u_plane[..cs * c_h].chunks_mut(cs * c_rows);
                let v_chunks = v_plane[..cs * c_h].chunks_mut(cs * c_rows);
                for (band, (uc, vc)) in u_chunks.zip(v_chunks).enumerate() {
                    let mut work = move || {
                        for (r, (ud, vd)) in uc.chunks_mut(cs).zip(vc.chunks_mut(cs)).enumerate() {
                            let row = band * c_rows + r;
                            for col in 0..w / 2 {
                                let (u, v) = uv_at(row, col);
                                ud[col] = u;
                                vd[col] = v;
                            }
                        }
                    };
                    if bands > 1 {
                        scope.spawn(work);
                    } else {
                        work();
                    }
                }
            }
        });
    }

    pub(super) fn encode(
        receiver: &Receiver<Frame>,
        stop: &AtomicBool,
        path: &Path,
    ) -> Result<String, String> {
        let Some(first) = next_frame(receiver, stop) else {
            return Err("no frames arrived".to_string());
        };
        // Encoders want even dimensions: the odd edge row/column is cropped.
        let size = first.size;
        let (w, h) = ((size[0] & !1) as usize, (size[1] & !1) as usize);
        if w < 64 || h < 64 {
            return Err("the frame is too small to encode".to_string());
        }
        let lib = Lib::open()?;
        let mut codec = std::ptr::null_mut();
        let mut semi_planar = true;
        for color in COLOR_FORMATS {
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
                (lib.fmt_i32)(fmt, c"bitrate".as_ptr(), 12_000_000);
                (lib.fmt_i32)(fmt, c"frame-rate".as_ptr(), 30);
                (lib.fmt_i32)(fmt, c"i-frame-interval".as_ptr(), 2);
            }
            let ok = unsafe {
                (lib.configure)(
                    c,
                    fmt,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    CONFIGURE_ENCODE,
                ) == 0
                    && (lib.start)(c) == 0
            };
            unsafe { (lib.fmt_del)(fmt) };
            if ok {
                codec = c;
                semi_planar = color == 21;
                break;
            }
            unsafe { (lib.delete)(c) };
        }
        if codec.is_null() {
            return Err("the H.264 encoder rejected both YUV layouts".to_string());
        }
        // The encoder's real row layout; most devices pad the stride.
        let (mut stride, mut slice) = (w as i32, h as i32);
        let in_fmt = unsafe { (lib.in_fmt)(codec) };
        if !in_fmt.is_null() {
            unsafe {
                (lib.fmt_get)(in_fmt, c"stride".as_ptr(), &mut stride);
                (lib.fmt_get)(in_fmt, c"slice-height".as_ptr(), &mut slice);
                (lib.fmt_del)(in_fmt);
            }
        }
        let stride = (stride.max(w as i32)) as usize;
        let slice = (slice.max(h as i32)) as usize;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
        let fd = {
            use std::os::fd::IntoRawFd;
            file.into_raw_fd()
        };
        let muxer = unsafe { (lib.mux_new)(fd, MUXER_MPEG_4) };
        if muxer.is_null() {
            unsafe {
                (lib.stop)(codec);
                (lib.delete)(codec);
                close(fd);
            }
            return Err("the MP4 muxer would not open the file".to_string());
        }
        let mut enc = Encoder {
            lib: &lib,
            codec,
            muxer,
            semi_planar,
            width: w,
            height: h,
            stride,
            slice,
            track: -1,
            started: false,
            saw_eos: false,
        };
        let mut frames = 0u64;
        let mut last_pts = 0i64;
        let mut result: Result<(), String> = Ok(());
        let mut pending = Some(first);
        loop {
            let Some(frame) = pending.take().or_else(|| next_frame(receiver, stop)) else {
                break;
            };
            // The window was resized or time stood still: skip the frame.
            if frame.size != size || (frames > 0 && frame.pts_us <= last_pts) {
                continue;
            }
            if let Err(e) = enc.feed(&frame) {
                result = Err(e);
                break;
            }
            last_pts = frame.pts_us;
            frames += 1;
        }
        if result.is_ok() && frames > 0 {
            result = enc.finish(last_pts);
        }
        let started = enc.started;
        unsafe {
            (lib.stop)(codec);
            (lib.delete)(codec);
            if started {
                (lib.mux_stop)(muxer);
            }
            (lib.mux_del)(muxer);
            close(fd);
        }
        result?;
        if frames == 0 {
            return Err("no frames arrived".to_string());
        }
        Ok(format!(
            "Saved {} ({} frames, {:.1} s).",
            path.display(),
            frames,
            last_pts as f64 / 1_000_000.0
        ))
    }
}
