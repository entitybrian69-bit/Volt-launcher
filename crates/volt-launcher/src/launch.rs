//! Ownership, verified installation, and transfer to the sibling engine, off the UI thread.
use crate::{account, profile_dir};
use horizon_client::{
    Result,
    official_install::{self, InstallProgress},
};
use std::{
    path::PathBuf,
    process::{Child, Command},
    sync::{Arc, Mutex},
};
pub(super) enum Destination {
    Home,
    Server(String),
    Replay(PathBuf, bool),
}
pub(super) type Progress = Arc<Mutex<Option<InstallProgress>>>;
pub(super) fn prepare_and_launch(
    home: PathBuf,
    profile: u32,
    destination: Destination,
    progress: Progress,
) -> Result<Child> {
    let id = account::client_id();
    let mut accounts = account::Accounts::load();
    let pass = account::launch(
        &account::Endpoints::live(),
        id.as_deref(),
        account::store(),
        &mut accounts,
    )
    .map_err(|e| e.to_string())?;
    accounts.save()?;
    let engine = std::env::current_exe()?.with_file_name("horizon-client.exe");
    if !engine.is_file() {
        return Err("The sibling horizon-client.exe is missing.".into());
    }
    if !official_install::ready(&home) {
        if pass.owner.is_none() {
            return Err("Verify ownership online before downloading Minecraft.".into());
        }
        // Only allowlisted generated tables ship with the app. Mojang assets are installed per player.
        let tables = engine.with_file_name("engine-data.zip");
        if !tables.is_file() {
            return Err("Missing engine-data.zip beside Volt Launcher.".into());
        }
        std::fs::copy(tables, home.join("engine-data.zip"))?;
        official_install::install_with_progress(&home, |p| *progress.lock().unwrap() = Some(p))?;
    }
    let game_dir = profile_dir(&home, profile);
    let mut cmd = Command::new(engine);
    cmd.current_dir(&game_dir)
        .arg("--home")
        .arg(&home)
        .arg("--game-dir")
        .arg(&game_dir)
        .arg("--resources")
        .arg(
            home.join(".local/horizon-client")
                .join(horizon_core::VANILLA_VERSION),
        );
    match destination {
        Destination::Home => {}
        Destination::Server(ip) => {
            cmd.arg("--server").arg(ip);
        }
        Destination::Replay(path, export) => {
            cmd.arg("--replay").arg(path);
            if export {
                cmd.args(["--replay-export", "1"]);
            }
        }
    }
    let mut session = None;
    if let Some((player, secrets)) = pass.session {
        let (player, secrets) = account::fresh(
            &account::Endpoints::live(),
            id.as_deref().unwrap_or(""),
            &player,
            secrets,
        )?;
        accounts.add(account::store(), player.clone(), &secrets)?;
        accounts.save()?;
        let file = account::write_session_file(&home.join("sessions"), &player, &secrets)?;
        cmd.arg("--session-file").arg(&file);
        session = Some(file);
    } else if let Some(name) = accounts.name_arg() {
        // Only an explicitly selected offline name, after the same owner gate above.
        cmd.arg("--name").arg(name).envs(accounts.look_env());
    } else {
        return Err(pass
            .offline
            .unwrap_or_else(|| "Sign in again to start this profile.".into())
            .into());
    }
    let child = cmd.spawn();
    if child.is_err()
        && let Some(file) = session
    {
        let _ = std::fs::remove_file(file);
    }
    Ok(child?)
}
