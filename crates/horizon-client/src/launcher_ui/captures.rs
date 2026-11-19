pub fn capture_files(home: &std::path::Path) -> (Vec<String>, Vec<(String, u64)>) {
    let mut shots: Vec<String> = std::fs::read_dir(home.join("screenshots"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .filter(|s| s.to_ascii_lowercase().ends_with(".png"))
                .map(str::to_owned)
        })
        .collect();
    shots.sort_unstable_by(|a, b| b.cmp(a));
    let mut clips: Vec<(String, u64)> = std::fs::read_dir(home.join("recordings"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            let name = entry
                .file_name()
                .to_str()
                .filter(|s| s.to_ascii_lowercase().ends_with(".mp4"))
                .map(str::to_owned)?;
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            Some((name, size))
        })
        .collect();
    clips.sort_unstable_by(|a, b| b.0.cmp(&a.0));
    (shots, clips)
}
