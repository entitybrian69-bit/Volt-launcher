#![cfg_attr(windows, windows_subsystem = "windows")]
#![forbid(unsafe_code)]

#[cfg(windows)]
include!("windows.rs");

#[cfg(not(windows))]
fn main() {
    eprintln!("Volt Launcher PC requires Windows x64.");
    std::process::exit(1);
}
