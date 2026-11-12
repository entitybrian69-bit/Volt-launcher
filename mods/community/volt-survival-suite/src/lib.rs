//! Volt Survival Suite — a small, practical Horizon client mod.
//!
//! Commands:
//!   /kit      Toggle the live HUD (choice persists across launches).
//!   /where    Print exact coordinates and dimension.
//!   /inspect  Inspect the first block in the player's line of sight.
//!   /time     Print the current world day and time.
use horizon_mod_sdk::{
    Mod, Registration, chat,
    game::{self, Clock, Player},
    hud::{self, Corner, Line},
    storage,
};

const HUD_ENABLED_KEY: &str = "hud_enabled";
const INSPECT_REACH: f32 = 6.0;

struct SurvivalSuite {
    hud_enabled: bool,
    last_panel: Vec<String>,
}

fn facing(yaw: f32) -> &'static str {
    let quadrant = (yaw / 90.0 + 0.5).floor() as i32 & 3;
    ["South (+Z)", "West (-X)", "North (-Z)", "East (+X)"][quadrant as usize]
}

fn world_time(clock: &Clock) -> String {
    let day = clock.day_time / 24_000 + 1;
    let ticks = clock.day_time % 24_000;
    let hour = (ticks / 1_000 + 6) % 24;
    let minute = ticks % 1_000 * 60 / 1_000;
    format!("Day {day}  {hour:02}:{minute:02}")
}

fn dimension_name(id: &str) -> &str {
    id.strip_prefix("minecraft:").unwrap_or(id)
}

fn panel_lines(player: &Player, clock: Option<&Clock>) -> Vec<String> {
    let mut lines = vec![
        format!("XYZ  {:.1}  {:.1}  {:.1}", player.x, player.y, player.z),
        format!("Facing  {}", facing(player.yaw)),
        format!("Health  {:.1}/20   Food  {}/20", player.health, player.food),
        format!("Dimension  {}", dimension_name(&player.dimension)),
    ];
    if let Some(clock) = clock {
        lines.push(world_time(clock));
    }
    if let Some(hit) = game::raycast(INSPECT_REACH) {
        lines.push(format!(
            "Looking at  {}  ({}, {}, {})",
            hit.block, hit.block_x, hit.block_y, hit.block_z
        ));
    }
    lines
}

impl Mod for SurvivalSuite {
    fn init() -> Result<(Self, Registration), String> {
        let hud_enabled = storage::get(HUD_ENABLED_KEY)
            .and_then(|bytes| bytes.first().copied())
            .map(|value| value != 0)
            .unwrap_or(true);

        let registration = Registration::new()
            .command("kit")
            .command("where")
            .command("inspect")
            .command("time")
            .on_tick();

        Ok((
            Self {
                hud_enabled,
                last_panel: Vec::new(),
            },
            registration,
        ))
    }

    fn tick(&mut self, _tick: u64) {
        let Some(player) = game::get_player().filter(|_| self.hud_enabled) else {
            if !self.last_panel.is_empty() {
                hud::clear();
                self.last_panel.clear();
            }
            return;
        };

        let lines = panel_lines(&player, game::get_clock().as_ref());
        if lines == self.last_panel {
            return;
        }

        let rendered: Vec<Line> = lines
            .iter()
            .map(|text| Line {
                text: text.clone(),
                color: 0xE8F1FF,
            })
            .collect();
        hud::set_panel(Corner::TopLeft, &rendered);
        self.last_panel = lines;
    }

    fn command(&mut self, name: &str, _args: &str) -> Result<Option<String>, String> {
        match name {
            "kit" => {
                self.hud_enabled = !self.hud_enabled;
                let value = [u8::from(self.hud_enabled)];
                if let Err(error) = storage::set(HUD_ENABLED_KEY, &value) {
                    return Err(format!("Could not save HUD setting: {error}"));
                }
                if !self.hud_enabled {
                    hud::clear();
                    self.last_panel.clear();
                }
                Ok(Some(format!(
                    "Survival HUD {}",
                    if self.hud_enabled { "enabled" } else { "disabled" }
                )))
            }
            "where" => {
                let player = game::get_player().ok_or("You must be in a world")?;
                chat::show(&format!(
                    "XYZ: {:.2}, {:.2}, {:.2} | {}",
                    player.x,
                    player.y,
                    player.z,
                    player.dimension
                ));
                Ok(None)
            }
            "inspect" => {
                let hit = game::raycast(INSPECT_REACH)
                    .ok_or("No block in sight within 6 blocks")?;
                chat::show(&format!(
                    "{} at {}, {}, {}",
                    hit.block, hit.block_x, hit.block_y, hit.block_z
                ));
                Ok(None)
            }
            "time" => {
                let clock = game::get_clock().ok_or("World clock unavailable")?;
                chat::show(&world_time(&clock));
                Ok(None)
            }
            _ => Err("Unknown Survival Suite command".into()),
        }
    }
}

horizon_mod_sdk::export_mod!(SurvivalSuite);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_matches_vanilla_yaw() {
        assert_eq!(facing(0.0), "South (+Z)");
        assert_eq!(facing(90.0), "West (-X)");
        assert_eq!(facing(-180.0), "North (-Z)");
        assert_eq!(facing(-90.0), "East (+X)");
    }

    #[test]
    fn clock_starts_at_six_am() {
        let clock = Clock {
            game_time: 0,
            day_time: 0,
        };
        assert_eq!(world_time(&clock), "Day 1  06:00");
    }

    #[test]
    fn clock_rolls_over_to_next_day() {
        let clock = Clock {
            game_time: 24_000,
            day_time: 24_000,
        };
        assert_eq!(world_time(&clock), "Day 2  06:00");
    }

    #[test]
    fn dimension_prefix_is_removed_for_display() {
        assert_eq!(dimension_name("minecraft:the_nether"), "the_nether");
        assert_eq!(dimension_name("custom:moon"), "custom:moon");
    }
}
