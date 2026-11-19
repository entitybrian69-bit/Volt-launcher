//! Windows adapter for the shared Controls Studio image-import action.
use std::{
    fs,
    io::{Read, Write},
    os::windows::process::CommandExt,
    path::PathBuf,
    process::Command,
};

pub(super) fn begin_button_image(dir: PathBuf) -> Result<(), String> {
    // Inbox Windows PowerShell supplies the standard file dialog without an extra runtime.
    // Neither the selected filename nor a user directory is interpolated into shell code.
    let shell = PathBuf::from(std::env::var_os("SystemRoot").ok_or("No Windows directory")?)
        .join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
    let output = Command::new(shell).creation_flags(0x0800_0000)
        .args(["-NoProfile", "-NonInteractive", "-STA", "-Command", r#"
$ErrorActionPreference='Stop'; Add-Type -AssemblyName System.Windows.Forms
$d=[Windows.Forms.OpenFileDialog]::new(); $d.Filter='PNG images (*.png)|*.png'
try { if ($d.ShowDialog() -eq 'OK') { [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); [Console]::Write($d.FileName) } } finally { $d.Dispose() }
"#]).output().map_err(|_| "Could not start Windows file picker")?;
    if !output.status.success() {
        return Err("Windows file picker failed".into());
    }
    let path = PathBuf::from(String::from_utf8(output.stdout).map_err(|_| "Invalid file name")?);
    if path.as_os_str().is_empty() {
        return Err("Image import canceled".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(&path)
        .and_then(|f| f.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes))
        .map_err(|_| "Could not read image")?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("Button PNG must be at most 4 MiB".into());
    }
    let (w, h) =
        image::ImageReader::with_format(std::io::Cursor::new(&bytes), image::ImageFormat::Png)
            .into_dimensions()
            .map_err(|_| "Not a PNG image")?;
    if w.max(h) > 2048 {
        return Err("Button PNG dimensions exceed 2048 pixels".into());
    }
    horizon_client::hud::decode_bytes(&bytes).map_err(|_| "Invalid PNG image")?;
    let name = path.file_name().ok_or("Missing filename")?;
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("png"))
    {
        return Err("Choose a .png file".into());
    }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(name);
    let mut file =
        fs::File::create_new(&dest).map_err(|e| format!("Cannot import button image: {e}"))?;
    if let Err(e) = file.write_all(&bytes) {
        drop(file);
        let _ = fs::remove_file(dest);
        return Err(e.to_string());
    }
    Ok(())
}
