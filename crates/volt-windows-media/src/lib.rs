//! Windows Media Foundation adapter. All COM objects live on the calling encoder thread.
#![cfg(windows)]

use std::{fs, path::Path};
use windows::{
    Win32::{Media::MediaFoundation::*, System::Com::*},
    core::{GUID, HSTRING},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Runtime;
impl Runtime {
    fn start() -> windows::core::Result<Self> {
        // Called on the dedicated export worker, not the UI apartment.
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
            if let Err(err) = MFStartup(MF_VERSION, MFSTARTUP_FULL) {
                CoUninitialize();
                return Err(err);
            }
        }
        Ok(Self)
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            let _ = MFShutdown();
            CoUninitialize();
        }
    }
}

fn video_type(w: u32, h: u32, fps: u32, subtype: &GUID) -> windows::core::Result<IMFMediaType> {
    unsafe {
        let ty = MFCreateMediaType()?;
        ty.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        ty.SetGUID(&MF_MT_SUBTYPE, subtype)?;
        ty.SetUINT64(&MF_MT_FRAME_SIZE, (u64::from(w) << 32) | u64::from(h))?;
        ty.SetUINT64(&MF_MT_FRAME_RATE, (u64::from(fps) << 32) | 1)?;
        ty.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1 << 32) | 1)?;
        ty.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
        Ok(ty)
    }
}

fn sample(data: &[u8], time: i64) -> windows::core::Result<IMFSample> {
    // Caller validated length against the checked u32 NV12 buffer size.
    unsafe {
        let buffer = MFCreateMemoryBuffer(data.len() as u32)?;
        let mut dst = std::ptr::null_mut();
        buffer.Lock(&mut dst, None, None)?;
        // MF allocated at least data.len() writable bytes. No fallible operations while locked.
        std::ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());
        buffer.Unlock()?;
        buffer.SetCurrentLength(data.len() as u32)?;
        let sample = MFCreateSample()?;
        sample.AddBuffer(&buffer)?;
        sample.SetSampleTime(time)?;
        Ok(sample)
    }
}

/// Encode a replay's NV12 frames (presentation timestamps in microseconds). The sink
/// writer selects an installed H.264 encoder. Never publish a partial MP4 or erase
/// a previous successful export if the codec, filesystem or capture fails.
pub fn write_mp4(
    path: &Path,
    w: u32,
    h: u32,
    frames: impl Iterator<Item = (i64, Vec<u8>)>,
) -> Result<u64> {
    write_mp4_at_rate(path, w, h, 30, 12_000_000, frames)
}

/// Live recording uses the GPU capture rate and the profile's resolution-dependent bitrate.
pub fn write_mp4_at_rate(
    path: &Path,
    w: u32,
    h: u32,
    fps: u32,
    bitrate: u32,
    frames: impl Iterator<Item = (i64, Vec<u8>)>,
) -> Result<u64> {
    if !(1..=240).contains(&fps) || bitrate == 0 {
        return Err("invalid H.264 encoding rate".into());
    }
    let bytes = w
        .checked_mul(h)
        .and_then(|n| n.checked_add(n / 2))
        .filter(|_| w >= 64 && h >= 64 && w % 2 == 0 && h % 2 == 0)
        .ok_or("invalid NV12 dimensions")?;
    let path = std::path::absolute(path)?;
    let parent = path.parent().ok_or("missing output folder")?;
    fs::create_dir_all(parent)?;
    // An exclusively created directory protects concurrent exports to this name.
    let staging = path.with_extension(format!("{}.exporting", std::process::id()));
    fs::create_dir(&staging)?;
    let tmp = staging.join("video.mp4");
    let result = (|| -> Result<u64> {
        let _runtime = Runtime::start()?;
        let count = unsafe {
            let writer = MFCreateSinkWriterFromURL(&HSTRING::from(tmp.as_os_str()), None, None)?;
            let output = video_type(w, h, fps, &MFVideoFormat_H264)?;
            output.SetUINT32(&MF_MT_AVG_BITRATE, bitrate)?;
            let stream = writer.AddStream(&output)?;
            let input = video_type(w, h, fps, &MFVideoFormat_NV12)?;
            input.SetUINT32(&MF_MT_DEFAULT_STRIDE, w)?;
            writer.SetInputMediaType(stream, &input, None)?;
            writer.BeginWriting()?;
            let mut previous: Option<(i64, IMFSample)> = None;
            let mut origin = None;
            let mut count = 0;
            let mut duration = 10_000_000 / i64::from(fps);
            for (pts, data) in frames {
                if data.len() != bytes as usize {
                    return Err("invalid NV12 frame length".into());
                }
                let time = pts
                    .checked_sub(*origin.get_or_insert(pts))
                    .and_then(|t| t.checked_mul(10))
                    .ok_or("replay timestamp out of range")?;
                if let Some((last, frame)) = previous.take() {
                    if time <= last {
                        return Err("replay clock did not advance".into());
                    }
                    duration = time - last;
                    frame.SetSampleDuration(duration)?;
                    writer.WriteSample(stream, &frame)?;
                }
                previous = Some((time, sample(&data, time)?));
                count += 1;
            }
            let (_, frame) = previous.ok_or("no frames arrived")?;
            frame.SetSampleDuration(duration)?;
            writer.WriteSample(stream, &frame)?;
            writer.Finalize()?;
            count
        }; // Close the sink's file handle before rename on NTFS.
        fs::rename(&tmp, path)?;
        Ok(count)
    })();
    let _ = fs::remove_file(tmp);
    let _ = fs::remove_dir(staging);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_replay_can_be_read_and_failed_reexport_preserves_it() -> Result<()> {
        let root = std::env::temp_dir().join(format!("volt media 測試 {}", std::process::id()));
        fs::create_dir(&root)?;
        let path = root.join("replay.mp4");
        let size = 160 * 120 * 3 / 2;
        let frames = (0..8).map(|i| (i * 40_000, vec![80 + i as u8; size]));
        assert_eq!(
            write_mp4_at_rate(&path, 160, 120, 25, 1_200_000, frames)?,
            8
        );
        let original = fs::read(&path)?;
        assert!(write_mp4(&path, 160, 120, [(0, vec![0; 8])].into_iter()).is_err());
        assert_eq!(fs::read(&path)?, original);
        assert_eq!(fs::read_dir(&root)?.count(), 1);
        {
            let _runtime = Runtime::start()?;
            unsafe {
                let reader = MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), None)?;
                let stream = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
                let ty = reader.GetNativeMediaType(stream, 0)?;
                assert_eq!(ty.GetGUID(&MF_MT_SUBTYPE)?, MFVideoFormat_H264);
                assert_eq!(ty.GetUINT64(&MF_MT_FRAME_SIZE)?, (160 << 32) | 120);
                let decoded = MFCreateMediaType()?;
                decoded.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
                decoded.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12)?;
                reader.SetCurrentMediaType(stream, None, &decoded)?;
                let mut times = Vec::new();
                loop {
                    let (mut flags, mut frame) = (0, None);
                    reader.ReadSample(stream, 0, None, Some(&mut flags), None, Some(&mut frame))?;
                    if let Some(frame) = frame {
                        assert!(frame.GetTotalLength()? >= size as u32);
                        times.push(frame.GetSampleTime()?);
                    }
                    if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                        break;
                    }
                }
                assert_eq!(times, (0..8).map(|i| i * 400_000).collect::<Vec<_>>());
            }
        }
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
