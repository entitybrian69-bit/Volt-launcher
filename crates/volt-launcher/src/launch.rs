//! Ownership, verified installation, and transfer to the sibling engine, off the UI thread.
use crate::{account, profile_dir};
use horizon_client::{
    Result,
    official_install::{self, InstallProgress},
};
use std::{
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
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
    let licensed_test = std::env::var("VOLT_QA_LICENSED_PLAY").as_deref() == Ok("1");
    if licensed_test
        && (pass.owner.is_none()
            || pass.session.is_none()
            || !matches!(&destination, Destination::Home))
    {
        return Err(
            "This acceptance run requires Play with a live, entitled Microsoft session.".into(),
        );
    }
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
    // The console-subsystem engine owns its own game window. Don't flash a second
    // console window when Play starts it from Explorer/the GUI subsystem.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW)
        .current_dir(&game_dir)
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
    if licensed_test {
        // Observe a normal interactive game (no autostart, synthetic terrain or frame limit).
        cmd.stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("--smoke-report")
            .arg(home.join("licensed-world.json"));
    }
    cmd.env_remove("HORIZON_SESSION_FILE");
    if let Some((player, secrets)) = pass.session {
        let (player, secrets) = account::fresh(
            &account::Endpoints::live(),
            id.as_deref().unwrap_or(""),
            &player,
            secrets,
        )?;
        accounts.add(account::store(), player.clone(), &secrets)?;
        accounts.save()?;
        let mut child = cmd.arg("--session-stdin").stdin(Stdio::piped()).spawn()?;
        let result = child
            .stdin
            .take()
            .ok_or_else(|| "Session pipe not available".to_owned())
            .and_then(|pipe| account::write_game_session(pipe, &player, &secrets));
        if let Err(error) = result {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error.into());
        }
        return Ok(child);
    } else if let Some(name) = accounts.name_arg() {
        // Only an explicitly selected offline name, after the same owner gate above.
        cmd.arg("--name").arg(name).envs(accounts.look_env());
    } else {
        return Err(pass
            .offline
            .unwrap_or_else(|| "Sign in again to start this profile.".into())
            .into());
    }
    Ok(cmd.spawn()?)
}
