//! Tier-1 Discord presence: "started/stopped playing" embeds to a Discord
//! webhook the owner configures. Fully optional — without a configured
//! webhook nothing runs and nothing is sent. The webhook URL comes from the
//! launcher's settings through `VOLT_DISCORD_WEBHOOK` (or the same
//! environment variable on desktop); only the player's display name and the
//! session event ever leave the device, over one short-lived thread per
//! event so the render loop never blocks.
use std::sync::Mutex;
use std::time::Duration;

/// Crimson, like the launcher.
const EMBED_COLOR: u32 = 0xDC14_3C;

/// The launcher-configured URL; on desktop the `VOLT_DISCORD_WEBHOOK`
/// environment variable is the fallback.
static CONFIGURED_URL: Mutex<Option<String>> = Mutex::new(None);

/// Hands the launcher's configured webhook URL to the game, called before
/// a session starts. An empty or non-Discord URL turns the feature off.
pub fn configure(url: &str) {
    let url = url.trim();
    *CONFIGURED_URL.lock().unwrap() = if url.is_empty() {
        None
    } else {
        Some(url.to_owned())
    };
}

fn webhook_url() -> Option<String> {
    let url = CONFIGURED_URL
        .lock()
        .unwrap()
        .clone()
        .or_else(|| std::env::var("VOLT_DISCORD_WEBHOOK").ok())?;
    let url = url.trim().to_owned();
    // Only ever talk to a real Discord webhook endpoint.
    url.starts_with("https://discord.com/api/webhooks/")
        .then_some(url)
}

/// Whether a webhook is configured; cheap enough for callers to gate on.
pub fn configured() -> bool {
    webhook_url().is_some()
}

/// A session event worth an embed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Session {
    Started,
    Stopped,
}

/// Posts the event in the background; errors are logged and dropped —
/// presence must never affect play.
pub fn post(player: &str, event: Session) {
    let Some(url) = webhook_url() else {
        return;
    };
    let player = if player.trim().is_empty() {
        "A player".to_owned()
    } else {
        player.trim().to_owned()
    };
    let text = match event {
        Session::Started => format!("🎮 **{player}** started playing Volt Launcher"),
        Session::Stopped => format!("🏁 **{player}** stopped playing"),
    };
    let body = serde_json::json!({
        "username": "Volt Launcher",
        "embeds": [{
            "description": text,
            "color": EMBED_COLOR,
        }],
    });
    let spawned = std::thread::Builder::new()
        .name("volt-webhook".into())
        .spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout(Duration::from_secs(8))
                .build();
            if let Err(error) = agent
                .post(&url)
                .set("Content-Type", "application/json")
                .send_string(&body.to_string())
            {
                log::debug!("Volt: Discord webhook: {error}");
            }
        });
    if let Err(error) = spawned {
        log::debug!("Volt: Discord webhook thread: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only Discord webhook endpoints are accepted; anything else stays
    /// unconfigured and sends nothing.
    #[test]
    fn rejects_non_discord_urls() {
        // The variable is unset in tests: nothing is configured.
        assert!(!configured());
        post("name", Session::Started); // Must be a quiet no-op.
    }
}
