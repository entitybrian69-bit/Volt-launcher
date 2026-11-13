//! What the player is doing, for Discord Rich Presence (`horizon_client::discord_presence`).
use super::*;
use horizon_client::discord_presence::{self as presence, Dimension, Play, Status};

/// Whether the last published status was in a world, for the Tier-1
/// webhook's started/stopped embeds.
static WEBHOOK_IN_WORLD: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

impl Game<'_> {
    /// Each frame: only a change reaches the presence thread.
    pub(super) fn update_discord_presence(&self) {
        let status = self.discord_status();
        if horizon_client::discord_webhook::configured() {
            let in_world = matches!(status, Status::World { .. });
            let was = WEBHOOK_IN_WORLD
                .swap(in_world, std::sync::atomic::Ordering::Relaxed);
            if in_world != was {
                horizon_client::discord_webhook::post(
                    &self.options.player_name(),
                    if in_world {
                        horizon_client::discord_webhook::Session::Started
                    } else {
                        horizon_client::discord_webhook::Session::Stopped
                    },
                );
            }
        }
        presence::set(status);
    }
    fn discord_status(&self) -> Status {
        if !self.settings.extras.discord_presence {
            return Status::Off;
        }
        if self.title.is_some() {
            return Status::Menus;
        }
        let play = if let Some(host) = &self.lan {
            Play::Hosting {
                players: 1 + host.hub.guests.len(),
            }
        } else if let Some(server) = &self.integrated {
            match (server.published(), &self.remote) {
                (Some(_), Some(remote)) => Play::Hosting {
                    players: remote.player_count(),
                },
                _ => Play::Singleplayer,
            }
        } else if self.remote.is_some() {
            Play::Multiplayer
        } else {
            Play::Singleplayer
        };
        Status::World {
            play,
            dimension: match self.dim {
                Dim::Overworld => Dimension::Overworld,
                Dim::Nether => Dimension::Nether,
                Dim::End => Dimension::End,
            },
            mode: self.mode,
        }
    }
}
