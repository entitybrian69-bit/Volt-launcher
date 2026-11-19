//! Microsoft account sign-in (docs/microsoft-account.md): the OAuth device-code flow, then Xbox
//! Live, XSTS and the Minecraft services, the game ownership check and the profile. Secrets (the
//! Microsoft refresh token, the Minecraft access token, the last ownership answer) live in the OS
//! keychain; accounts.json keeps only names and UUIDs. The game gets the session in a file it
//! reads once (`--session-file`), never on its command line. Tokens never reach logs, errors or
//! output.
//!
//! The owner gate ([`launch`]): the launcher starts the game only while an account in it owns
//! Minecraft: Java Edition, as other launchers do. Offline names play behind such an account.
//! The game binary itself, its harnesses and the servers don't go through the launcher's gate.
//!
//! The Azure app (client ID) is Horizon's own and is configuration: `HORIZON_MSA_CLIENT_ID` at
//! build time, or at run time to override it. Without one, sign-in says it is not configured.
pub const SERVICE: &str = "dev.playhorizon.launcher";
include!("account/protocol.rs");

#[cfg(not(target_os = "android"))]
include!("account/java_player.rs");

#[cfg(test)]
#[path = "account/tests.rs"]
mod tests;
