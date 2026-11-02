//! Pure-Rust multi-version wire protocol translator for Minecraft Java Edition `1.20.1`..`26.3`
//! (protocols `763`..`777`).
//!
//! Horizon's integrated singleplayer server communicates over [`crate::connection::Pipe`] at
//! [`crate::PROTOCOL_VERSION`] (`777`) with zero translation overhead. External TCP multiplayer
//! connections running an older [`horizon_core::game_version::GameVersion`] (`763`..`776`) attach
//! a wire state via [`crate::connection::Connection::set_wire_protocol`], which translates packet
//! IDs and wire payloads cleanly between the server's protocol and Horizon's internal `777` model.

use crate::{
    Error, PROTOCOL_VERSION, Result, State,
    buf::{Reader, Writer, write_varint},
    ids,
};
use horizon_world::nbt::{Limits, Tag, read_network, write_network};

/// Unknown/unmodeled wire packet sentinel returned to `Clientbound::decode` so unmodeled legacy
/// packets (such as `1.20.1` `update_enabled_features` in Play or `recipe`) are cleanly ignored.
pub const UNMODELED_WIRE_ID: i32 = -1;

/// Encodes `State` into an `AtomicU8` slot shared between [`crate::connection::ConnReader`] and
/// [`crate::connection::ConnWriter`].
#[inline]
pub const fn state_to_u8(state: State) -> u8 {
    match state {
        State::Handshake => 0,
        State::Status => 1,
        State::Login => 2,
        State::Configuration => 3,
        State::Play => 4,
    }
}

/// Decodes `State` from an `AtomicU8` slot.
#[inline]
pub const fn state_from_u8(v: u8) -> State {
    match v {
        0 => State::Handshake,
        1 => State::Status,
        2 => State::Login,
        3 => State::Configuration,
        _ => State::Play,
    }
}

/// Returns the ordered list of `Configuration` clientbound packet names for `protocol` (`764`..`777`).
pub fn config_clientbound_names(protocol: i32) -> Vec<&'static str> {
    if protocol < 764 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(21);
    if protocol >= 766 {
        out.push("cookie_request");
    }
    out.extend_from_slice(&[
        "custom_payload",
        "disconnect",
        "finish_configuration",
        "keep_alive",
        "ping",
    ]);
    if protocol >= 766 {
        out.push("reset_chat");
    }
    out.push("registry_data");
    if protocol == 764 {
        out.push("resource_pack");
    } else {
        out.push("resource_pack_pop");
        out.push("resource_pack_push");
    }
    if protocol >= 775 {
        out.push("post_effects");
    }
    if protocol >= 766 {
        out.push("store_cookie");
        out.push("transfer");
    }
    out.push("update_enabled_features");
    out.push("update_tags");
    if protocol >= 766 {
        out.push("select_known_packs");
    }
    if protocol >= 767 {
        out.push("custom_report_details");
        out.push("server_links");
    }
    if protocol >= 771 {
        out.push("clear_dialog");
        out.push("show_dialog");
    }
    if protocol >= 773 {
        out.push("code_of_conduct");
    }
    out
}

/// Returns the ordered list of `Configuration` serverbound packet names for `protocol` (`764`..`777`).
pub fn config_serverbound_names(protocol: i32) -> Vec<&'static str> {
    if protocol < 764 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(10);
    out.push("client_information");
    if protocol >= 766 {
        out.push("cookie_response");
    }
    out.extend_from_slice(&[
        "custom_payload",
        "finish_configuration",
        "keep_alive",
        "pong",
        "resource_pack",
    ]);
    if protocol >= 766 {
        out.push("select_known_packs");
    }
    if protocol >= 771 {
        out.push("custom_click_action");
    }
    if protocol >= 773 {
        out.push("accept_code_of_conduct");
    }
    out
}

/// Returns the ordered list of `Play` clientbound packet names for `protocol` (`763`..`777`).
pub fn play_clientbound_names(protocol: i32) -> Vec<&'static str> {
    let mut out = Vec::with_capacity(148);
    out.push("bundle_delimiter");
    out.push("add_entity");
    if protocol <= 769 {
        out.push("add_experience_orb");
    }
    if protocol == 763 {
        out.push("add_player");
    }
    out.extend_from_slice(&[
        "animate",
        "award_stats",
        "block_changed_ack",
        "block_destruction",
        "block_entity_data",
        "block_event",
        "block_update",
        "boss_event",
        "change_difficulty",
    ]);
    if protocol >= 764 {
        out.push("chunk_batch_finished");
        out.push("chunk_batch_start");
    }
    out.extend_from_slice(&[
        "chunks_biomes",
        "clear_titles",
        "command_suggestions",
        "commands",
        "container_close",
        "container_set_content",
        "container_set_data",
        "container_set_slot",
    ]);
    if protocol >= 766 {
        out.push("cookie_request");
    }
    out.extend_from_slice(&[
        "cooldown",
        "custom_chat_completions",
        "custom_payload",
        "damage_event",
    ]);
    if protocol >= 773 {
        out.extend_from_slice(&[
            "debug/block_value",
            "debug/chunk_value",
            "debug/entity_value",
            "debug/event",
        ]);
    }
    if protocol >= 766 {
        out.push("debug_sample");
    }
    out.extend_from_slice(&["delete_chat", "disconnect", "disguised_chat", "entity_event"]);
    if protocol >= 768 {
        out.push("entity_position_sync");
    }
    out.push("explode");
    if protocol >= 775 {
        out.push("add_transient_block");
    }
    out.push("forget_level_chunk");
    out.push("game_event");
    if protocol >= 775 {
        out.push("game_rule_values");
    }
    if protocol >= 773 {
        out.push("game_test_highlight_pos");
    }
    out.extend_from_slice(&[
        "mount_screen_open",
        "hurt_animation",
        "initialize_border",
        "keep_alive",
        "level_chunk_with_light",
        "level_event",
        "level_particles",
        "light_update",
        "login",
    ]);
    if protocol >= 775 {
        out.push("low_disk_space_warning");
    }
    out.extend_from_slice(&[
        "map_item_data",
        "merchant_offers",
        "move_entity_pos",
        "move_entity_pos_rot",
    ]);
    if protocol >= 768 {
        out.push("move_minecart_along_track");
    }
    out.extend_from_slice(&[
        "move_entity_rot",
        "move_vehicle",
        "open_book",
        "open_screen",
        "open_sign_editor",
        "ping",
    ]);
    if protocol >= 764 {
        out.push("pong_response");
    }
    out.extend_from_slice(&[
        "place_ghost_recipe",
        "player_abilities",
        "player_chat",
        "player_combat_end",
        "player_combat_enter",
        "player_combat_kill",
        "player_info_remove",
        "player_info_update",
        "player_look_at",
        "player_position",
    ]);
    if protocol >= 768 {
        out.push("player_rotation");
        out.push("recipe_book_add");
        out.push("recipe_book_remove");
        out.push("recipe_book_settings");
    } else {
        out.push("recipe");
    }
    out.push("remove_entities");
    out.push("remove_mob_effect");
    if protocol >= 765 {
        out.push("reset_score");
    }
    if protocol <= 764 {
        out.push("resource_pack");
    } else {
        out.push("resource_pack_pop");
        out.push("resource_pack_push");
    }
    if protocol >= 775 {
        out.push("post_effects");
    }
    out.extend_from_slice(&[
        "respawn",
        "rotate_head",
        "section_blocks_update",
        "select_advancements_tab",
        "server_data",
        "set_action_bar_text",
        "set_border_center",
        "set_border_lerp_size",
        "set_border_size",
        "set_border_warning_delay",
        "set_border_warning_distance",
        "set_camera",
    ]);
    if protocol <= 767 {
        // Renamed from `set_carried_item` to `set_held_slot` in 1.21.2 (768).
        out.push("set_held_slot");
    }
    out.push("set_chunk_cache_center");
    out.push("set_chunk_cache_radius");
    if protocol >= 768 {
        out.push("set_cursor_item");
    }
    out.extend_from_slice(&[
        "set_default_spawn_position",
        "set_display_objective",
        "set_entity_data",
        "set_entity_link",
        "set_entity_motion",
        "set_equipment",
        "set_experience",
        "set_health",
    ]);
    if protocol >= 768 {
        out.push("set_held_slot");
    }
    out.push("set_objective");
    out.push("set_passengers");
    if protocol >= 768 {
        out.push("set_player_inventory");
    }
    out.extend_from_slice(&[
        "set_player_team",
        "set_score",
        "set_simulation_distance",
        "set_subtitle_text",
        "set_time",
        "set_title_text",
        "set_titles_animation",
        "sound_entity",
        "sound",
    ]);
    if protocol >= 764 {
        out.push("start_configuration");
    }
    out.push("stop_sound");
    if protocol >= 766 {
        out.push("store_cookie");
    }
    if protocol >= 775 {
        out.push("swing_animation");
    }
    out.extend_from_slice(&[
        "system_chat",
        "tab_list",
        "tag_query",
        "take_item_entity",
        "teleport_entity",
    ]);
    if protocol >= 770 {
        out.push("test_instance_block_status");
    }
    if protocol >= 765 {
        out.push("ticking_state");
        out.push("ticking_step");
    }
    if protocol >= 766 {
        out.push("transfer");
    }
    out.push("update_advancements");
    out.push("update_attributes");
    if protocol == 763 {
        out.push("update_enabled_features");
    }
    out.extend_from_slice(&["update_mob_effect", "update_recipes", "update_tags"]);
    if protocol >= 766 {
        out.push("projectile_power");
    }
    if protocol >= 767 {
        out.push("custom_report_details");
        out.push("server_links");
    }
    if protocol >= 771 {
        out.push("waypoint");
        out.push("clear_dialog");
        out.push("show_dialog");
    }
    out
}

/// Returns the ordered list of `Play` serverbound packet names for `protocol` (`763`..`777`).
pub fn play_serverbound_names(protocol: i32) -> Vec<&'static str> {
    let mut out = Vec::with_capacity(70);
    out.push("accept_teleportation");
    if protocol >= 775 {
        out.push("attack");
    }
    out.push("block_entity_tag_query");
    if protocol >= 768 {
        out.push("bundle_item_selected");
    }
    out.push("change_difficulty");
    if protocol >= 771 {
        out.push("change_game_mode");
    }
    out.push("chat_ack");
    out.push("chat_command");
    if protocol >= 766 {
        out.push("chat_command_signed");
    }
    out.push("chat");
    out.push("chat_session_update");
    if protocol >= 764 {
        out.push("chunk_batch_received");
    }
    out.push("client_command");
    if protocol >= 768 {
        out.push("client_tick_end");
    }
    out.push("client_information");
    out.push("command_suggestion");
    if protocol >= 764 {
        out.push("configuration_acknowledged");
    }
    out.extend_from_slice(&[
        "container_button_click",
        "container_click",
        "container_close",
    ]);
    if protocol >= 765 {
        out.push("container_slot_state_changed");
    }
    if protocol >= 766 {
        out.push("cookie_response");
    }
    out.push("custom_payload");
    if protocol >= 766 {
        out.push("debug_subscription_request");
    }
    out.extend_from_slice(&[
        "edit_book",
        "entity_tag_query",
        "interact",
        "jigsaw_generate",
        "keep_alive",
        "lock_difficulty",
        "move_player_pos",
        "move_player_pos_rot",
        "move_player_rot",
        "move_player_status_only",
        "move_vehicle",
        "paddle_boat",
    ]);
    if protocol >= 769 {
        out.push("pick_item_from_block");
        out.push("pick_item_from_entity");
    } else {
        out.push("pick_item");
    }
    if protocol >= 764 {
        out.push("ping_request");
    }
    out.extend_from_slice(&[
        "place_recipe",
        "player_abilities",
        "player_action",
        "player_command",
        "player_input",
    ]);
    if protocol >= 769 {
        out.push("player_loaded");
    }
    out.push("pong");
    if protocol >= 775 {
        out.push("punch");
    }
    out.extend_from_slice(&[
        "recipe_book_change_settings",
        "recipe_book_seen_recipe",
        "rename_item",
        "resource_pack",
        "seen_advancements",
        "select_trade",
        "set_beacon",
        "set_carried_item",
        "set_command_block",
        "set_command_minecart",
        "set_creative_mode_slot",
    ]);
    if protocol >= 775 {
        out.push("set_game_rule");
    }
    out.push("set_jigsaw_block");
    out.push("set_structure_block");
    if protocol >= 770 {
        out.push("set_test_block");
    }
    out.push("sign_update");
    if protocol >= 775 {
        out.push("spectator_action");
    } else {
        // Legacy `swing` packet (0x2F in 1.20.1, replaced by `punch` in 26.1).
        out.push("punch");
    }
    out.push("teleport_to_entity");
    if protocol >= 770 {
        out.push("test_instance_block_action");
    }
    out.push("use_item_on");
    out.push("use_item");
    if protocol >= 771 {
        out.push("custom_click_action");
    }
    out
}

fn index_of(list: &[&str], target: &str) -> Option<i32> {
    list.iter().position(|n| *n == target).map(|i| i as i32)
}

/// Translates an incoming `(wire_id, wire_payload)` on `protocol` (`763`..`776`) into Horizon's
/// internal `777` `(id, payload)` representation.
pub fn translate_clientbound(
    protocol: i32,
    state: State,
    wire_id: i32,
    payload: &[u8],
) -> Result<(i32, Vec<u8>)> {
    if protocol == PROTOCOL_VERSION {
        return Ok((wire_id, payload.to_vec()));
    }
    match state {
        State::Handshake | State::Status | State::Login => Ok((wire_id, payload.to_vec())),
        State::Configuration => {
            let names = config_clientbound_names(protocol);
            let Some(&name) = usize::try_from(wire_id).ok().and_then(|i| names.get(i)) else {
                return Ok((UNMODELED_WIRE_ID, Vec::new()));
            };
            let Some(internal_id) = index_of(ids::configuration::clientbound::NAMES, name) else {
                return Ok((UNMODELED_WIRE_ID, Vec::new()));
            };
            let adapted = adapt_config_clientbound(protocol, name, payload)?;
            Ok((internal_id, adapted))
        }
        State::Play => {
            let names = play_clientbound_names(protocol);
            let Some(&name) = usize::try_from(wire_id).ok().and_then(|i| names.get(i)) else {
                return Ok((UNMODELED_WIRE_ID, Vec::new()));
            };
            let Some(internal_id) = index_of(ids::play::clientbound::NAMES, name) else {
                return Ok((UNMODELED_WIRE_ID, Vec::new()));
            };
            let adapted = match adapt_play_clientbound(protocol, name, payload) {
                Ok(bytes) => bytes,
                Err(_) => return Ok((UNMODELED_WIRE_ID, Vec::new())),
            };
            Ok((internal_id, adapted))
        }
    }
}

/// Translates an outgoing `777` framed body (`VarInt(internal_id) + payload`) into the wire body
/// for `protocol` (`763`..`776`). Returns `Ok(None)` when the packet does not exist in `protocol`
/// (e.g. `ClientTickEnd` on `1.20.1`..`1.21.1` or `PlayerLoaded` on `1.20.1`..`1.21.3`), so the
/// connection silently skips packets unknown to older servers.
pub fn translate_serverbound(
    protocol: i32,
    state: State,
    body: &[u8],
) -> Result<Option<Vec<u8>>> {
    if protocol == PROTOCOL_VERSION {
        return Ok(Some(body.to_vec()));
    }
    match state {
        State::Handshake | State::Status | State::Login => Ok(Some(body.to_vec())),
        State::Configuration => {
            let mut r = Reader::new(body);
            let internal_id = r.varint()?;
            let payload = r.rest();
            let Some(&name) = usize::try_from(internal_id)
                .ok()
                .and_then(|i| ids::configuration::serverbound::NAMES.get(i))
            else {
                return Ok(None);
            };
            let wire_names = config_serverbound_names(protocol);
            let Some(wire_id) = index_of(&wire_names, name) else {
                return Ok(None);
            };
            let adapted = adapt_serverbound_payload(protocol, name, payload)?;
            let mut out = Vec::with_capacity(5 + adapted.len());
            write_varint(&mut out, wire_id);
            out.extend_from_slice(&adapted);
            Ok(Some(out))
        }
        State::Play => {
            let mut r = Reader::new(body);
            let internal_id = r.varint()?;
            let payload = r.rest();
            let Some(&name) = usize::try_from(internal_id)
                .ok()
                .and_then(|i| ids::play::serverbound::NAMES.get(i))
            else {
                return Ok(None);
            };
            let wire_names = play_serverbound_names(protocol);
            let Some(wire_id) = index_of(&wire_names, name) else {
                return Ok(None);
            };
            let adapted = adapt_serverbound_payload(protocol, name, payload)?;
            let mut out = Vec::with_capacity(5 + adapted.len());
            write_varint(&mut out, wire_id);
            out.extend_from_slice(&adapted);
            Ok(Some(out))
        }
    }
}

fn adapt_config_clientbound(protocol: i32, name: &str, payload: &[u8]) -> Result<Vec<u8>> {
    match name {
        // In 1.20.2..1.20.4 (764..765), Disconnect reason in Configuration was JSON String;
        // 1.20.3+ Play switched to NBT, and 1.20.5+ Configuration also uses NBT.
        "disconnect" if protocol <= 765 => json_string_payload_to_nbt(payload),
        // In 1.20.2..1.20.4 (764..765), RegistryData sent a single compound Nbt instead of
        // per-registry entries; handle unmodeled single-compound form gracefully as empty list.
        "registry_data" if protocol <= 765 => {
            let mut w = Writer::new();
            w.string("minecraft:dimension_type");
            w.varint(0);
            Ok(w.data)
        }
        _ => Ok(payload.to_vec()),
    }
}

fn adapt_play_clientbound(protocol: i32, name: &str, payload: &[u8]) -> Result<Vec<u8>> {
    match name {
        "login" => adapt_play_login(protocol, payload),
        "respawn" => adapt_play_respawn(protocol, payload),
        "player_position" if protocol < 768 => adapt_legacy_player_position(payload),
        "disconnect" if protocol <= 764 => json_string_payload_to_nbt(payload),
        "system_chat" if protocol <= 764 => adapt_legacy_system_chat(payload),
        "add_entity" if protocol < 773 => adapt_legacy_add_entity(payload),
        _ => Ok(payload.to_vec()),
    }
}

fn adapt_serverbound_payload(protocol: i32, name: &str, payload: &[u8]) -> Result<Vec<u8>> {
    match name {
        // `particle_status: VarInt` at the end of ClientInformation was added in 1.21.2 (768).
        "client_information" if protocol < 768 => {
            let mut r = Reader::new(payload);
            let language = r.string(16)?;
            let view_distance = r.i8()?;
            let chat_visibility = r.varint()?;
            let chat_colors = r.bool()?;
            let model_customisation = r.u8()?;
            let main_hand = r.varint()?;
            let text_filtering = r.bool()?;
            let allows_listing = r.bool()?;
            let mut w = Writer::new();
            w.string(&language);
            w.i8(view_distance);
            w.varint(chat_visibility);
            w.bool(chat_colors);
            w.u8(model_customisation);
            w.varint(main_hand);
            w.bool(text_filtering);
            w.bool(allows_listing);
            Ok(w.data)
        }
        // In < 768 (pre-1.21.2), the trailing byte of MovePlayer* is `on_ground: bool` (bit 0 only).
        "move_player_pos" | "move_player_pos_rot" | "move_player_rot" | "move_player_status_only"
            if protocol < 768 && !payload.is_empty() =>
        {
            let mut out = payload.to_vec();
            if let Some(last) = out.last_mut() {
                *last &= 1;
            }
            Ok(out)
        }
        // Legacy `swing` packet (0x2F in 1.20.1..1.21.11) takes `hand: VarInt` (0 main, 1 offhand).
        "punch" if protocol < 775 => {
            let mut w = Writer::new();
            w.varint(0);
            Ok(w.data)
        }
        _ => Ok(payload.to_vec()),
    }
}

fn skip_named_root_nbt(r: &mut Reader) -> Result<()> {
    let tag_id = r.u8()?;
    if tag_id == 0 {
        return Ok(());
    }
    let name_len = usize::from(r.u16()?);
    let _ = r.bytes(name_len)?;
    let rest = r.rest();
    let mut synthetic = Vec::with_capacity(1 + rest.len());
    synthetic.push(tag_id);
    synthetic.extend_from_slice(rest);
    let (_, used) = read_network(&synthetic, Limits::default())
        .map_err(|e| Error::new(format!("legacy NBT: {e}")))?;
    let payload_used = used.saturating_sub(1);
    let _ = r.bytes(payload_used)?;
    Ok(())
}

fn read_legacy_spawn_info(protocol: i32, r: &mut Reader, w: &mut Writer) -> Result<()> {
    // dimension_type: Ident in <= 765 (1.20.1..1.20.4), VarInt in >= 766 (1.20.5+).
    let dim_type_id = if protocol <= 765 {
        let dim_type = r.string(32767)?;
        match dim_type.as_str() {
            "minecraft:the_nether" => 2,
            "minecraft:the_end" => 3,
            "minecraft:overworld_caves" => 1,
            _ => 0,
        }
    } else {
        r.varint()?
    };
    let dimension = r.string(32767)?;
    let seed = r.i64()?;
    let game_type = if protocol <= 774 {
        i32::from(r.u8()?)
    } else {
        r.varint()?
    };
    let prev_game_type: Option<i32> = if protocol <= 774 {
        let raw = r.i8()?;
        (raw >= 0).then_some(i32::from(raw))
    } else {
        let raw = r.varint()?;
        (raw != 0).then_some(raw - 1)
    };
    let debug = r.bool()?;
    let flat = r.bool()?;
    let has_last_death = r.bool()?;
    let last_death = if has_last_death {
        Some((r.string(32767)?, r.i64()?))
    } else {
        None
    };
    let portal_cooldown = r.varint()?;
    let sea_level = if protocol >= 768 { r.varint()? } else { 63 };

    w.varint(dim_type_id);
    w.string(&dimension);
    w.i64(seed);
    w.varint(game_type);
    w.varint(prev_game_type.map_or(0, |g| g + 1));
    w.bool(debug);
    w.bool(flat);
    w.bool(last_death.is_some());
    if let Some((dim, pos)) = last_death {
        w.string(&dim);
        w.i64(pos);
    }
    w.varint(portal_cooldown);
    w.varint(sea_level);
    Ok(())
}

fn adapt_play_login(protocol: i32, payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    let player_id = r.i32()?;
    let hardcore = r.bool()?;
    if protocol == 763 {
        // 1.20.1 (763) field order before SpawnInfo extraction in 1.20.2:
        // player_id, hardcore, game_type(u8), prev_game_type(i8), levels(List<Ident>),
        // registry_holder(named NBT), dimension_type(Ident), dimension(Ident),
        // seed(i64), max_players(VarInt), chunk_radius(VarInt), simulation_distance(VarInt),
        // reduced_debug_info(bool), show_death_screen(bool), debug(bool), flat(bool),
        // last_death(Opt<GlobalPos>), portal_cooldown(VarInt).
        let game_type = i32::from(r.u8()?);
        let prev_raw = r.i8()?;
        let level_count = r.count(256)?;
        let mut levels = Vec::with_capacity(level_count);
        for _ in 0..level_count {
            levels.push(r.string(32767)?);
        }
        skip_named_root_nbt(&mut r)?;
        let dim_type = r.string(32767)?;
        let dim_type_id = match dim_type.as_str() {
            "minecraft:the_nether" => 2,
            "minecraft:the_end" => 3,
            "minecraft:overworld_caves" => 1,
            _ => 0,
        };
        let dimension = r.string(32767)?;
        let seed = r.i64()?;
        let max_players = r.varint()?;
        let chunk_radius = r.varint()?;
        let simulation_distance = r.varint()?;
        let reduced_debug_info = r.bool()?;
        let show_death_screen = r.bool()?;
        let debug = r.bool()?;
        let flat = r.bool()?;
        let has_last_death = r.bool()?;
        let last_death = if has_last_death {
            Some((r.string(32767)?, r.i64()?))
        } else {
            None
        };
        let portal_cooldown = r.varint()?;

        let mut w = Writer::new();
        w.i32(player_id);
        w.bool(hardcore);
        w.varint(levels.len() as i32);
        for lvl in &levels {
            w.string(lvl);
        }
        w.varint(max_players);
        w.varint(chunk_radius);
        w.varint(simulation_distance);
        w.bool(reduced_debug_info);
        w.bool(show_death_screen);
        w.bool(false); // limited_crafting
        w.varint(dim_type_id);
        w.string(&dimension);
        w.i64(seed);
        w.varint(game_type);
        w.varint(if prev_raw >= 0 { i32::from(prev_raw) + 1 } else { 0 });
        w.bool(debug);
        w.bool(flat);
        w.bool(last_death.is_some());
        if let Some((dim, pos)) = last_death {
            w.string(&dim);
            w.i64(pos);
        }
        w.varint(portal_cooldown);
        w.varint(63); // sea_level
        w.bool(false); // online_mode
        w.bool(false); // enforces_secure_chat
        return Ok(w.data);
    }

    // 764..=776 (1.20.2..26.2)
    let level_count = r.count(256)?;
    let mut levels = Vec::with_capacity(level_count);
    for _ in 0..level_count {
        levels.push(r.string(32767)?);
    }
    let max_players = r.varint()?;
    let chunk_radius = r.varint()?;
    let simulation_distance = r.varint()?;
    let reduced_debug_info = r.bool()?;
    let show_death_screen = r.bool()?;
    let limited_crafting = r.bool()?;

    let mut w = Writer::new();
    w.i32(player_id);
    w.bool(hardcore);
    w.varint(levels.len() as i32);
    for lvl in &levels {
        w.string(lvl);
    }
    w.varint(max_players);
    w.varint(chunk_radius);
    w.varint(simulation_distance);
    w.bool(reduced_debug_info);
    w.bool(show_death_screen);
    w.bool(limited_crafting);
    read_legacy_spawn_info(protocol, &mut r, &mut w)?;
    let online_mode = if protocol >= 775 && r.remaining() >= 2 {
        r.bool()?
    } else {
        false
    };
    let enforces_secure_chat = if protocol >= 766 && r.remaining() >= 1 {
        r.bool()?
    } else {
        false
    };
    w.bool(online_mode);
    w.bool(enforces_secure_chat);
    Ok(w.data)
}

fn adapt_play_respawn(protocol: i32, payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    let mut w = Writer::new();
    if protocol == 763 {
        let dim_type = r.string(32767)?;
        let dim_type_id = match dim_type.as_str() {
            "minecraft:the_nether" => 2,
            "minecraft:the_end" => 3,
            "minecraft:overworld_caves" => 1,
            _ => 0,
        };
        let dimension = r.string(32767)?;
        let seed = r.i64()?;
        let game_type = i32::from(r.u8()?);
        let prev_raw = r.i8()?;
        let debug = r.bool()?;
        let flat = r.bool()?;
        let keep = r.i8()?;
        let has_last_death = r.bool()?;
        let last_death = if has_last_death {
            Some((r.string(32767)?, r.i64()?))
        } else {
            None
        };
        let portal_cooldown = r.varint()?;
        w.varint(dim_type_id);
        w.string(&dimension);
        w.i64(seed);
        w.varint(game_type);
        w.varint(if prev_raw >= 0 { i32::from(prev_raw) + 1 } else { 0 });
        w.bool(debug);
        w.bool(flat);
        w.bool(last_death.is_some());
        if let Some((dim, pos)) = last_death {
            w.string(&dim);
            w.i64(pos);
        }
        w.varint(portal_cooldown);
        w.varint(63);
        w.i8(keep);
        return Ok(w.data);
    }
    read_legacy_spawn_info(protocol, &mut r, &mut w)?;
    let keep = r.i8()?;
    w.i8(keep);
    Ok(w.data)
}

fn adapt_legacy_player_position(payload: &[u8]) -> Result<Vec<u8>> {
    // 763..=767: x(f64), y(f64), z(f64), yaw(f32), pitch(f32), relatives(u8), teleport(VarInt),
    // (+ optional dismount_vehicle bool in 763, ignored in 764+).
    let mut r = Reader::new(payload);
    let x = r.f64()?;
    let y = r.f64()?;
    let z = r.f64()?;
    let yaw = r.f32()?;
    let pitch = r.f32()?;
    let relatives = i32::from(r.u8()?);
    let teleport = r.varint()?;

    let mut w = Writer::new();
    w.varint(teleport);
    w.f64(x);
    w.f64(y);
    w.f64(z);
    w.f64(0.0);
    w.f64(0.0);
    w.f64(0.0);
    w.f32(yaw);
    w.f32(pitch);
    w.i32(relatives);
    Ok(w.data)
}

fn json_string_to_tag(json_str: &str) -> Tag {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str)
        && let Some(s) = v.as_str()
    {
        return Tag::String(s.into());
    }
    Tag::String(json_str.into())
}

fn json_string_payload_to_nbt(payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    let text = r.string(262144)?;
    let tag = json_string_to_tag(&text);
    write_network(&tag, Limits::default()).map_err(|e| Error::new(format!("NBT encode: {e}")))
}

fn adapt_legacy_system_chat(payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    let text = r.string(262144)?;
    let overlay = r.bool()?;
    let mut w = Writer::new();
    w.nbt(&json_string_to_tag(&text));
    w.bool(overlay);
    Ok(w.data)
}

fn adapt_legacy_add_entity(payload: &[u8]) -> Result<Vec<u8>> {
    // < 773 (1.20.1..1.21.8): id(VarInt), uuid(Uuid), kind(VarInt), x/y/z(f64*3),
    // pitch(i8), yaw(i8), head_yaw(i8), data(VarInt), vx(i16), vy(i16), vz(i16).
    // >= 773 (1.21.9+): id, uuid, kind, x/y/z, velocity(LpVec3), pitch, yaw, head_yaw, data.
    let mut r = Reader::new(payload);
    let id = r.varint()?;
    let uuid = r.uuid()?;
    let kind = r.varint()?;
    let x = r.f64()?;
    let y = r.f64()?;
    let z = r.f64()?;
    let pitch = r.i8()?;
    let yaw = r.i8()?;
    let head_yaw = r.i8()?;
    let data = r.varint()?;
    let vx = f64::from(r.i16()?) / 8000.0;
    let vy = f64::from(r.i16()?) / 8000.0;
    let vz = f64::from(r.i16()?) / 8000.0;

    let mut w = Writer::new();
    w.varint(id);
    w.uuid(uuid);
    w.varint(kind);
    w.f64(x);
    w.f64(y);
    w.f64(z);
    <crate::buf::LpVec3 as crate::buf::Codec<[f64; 3]>>::write(&[vx, vy, vz], &mut w);
    w.i8(pitch);
    w.i8(yaw);
    w.i8(head_yaw);
    w.varint(data);
    Ok(w.data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v777_packet_tables_match_ids_module_1_to_1() {
        assert_eq!(
            config_clientbound_names(777).as_slice(),
            ids::configuration::clientbound::NAMES
        );
        assert_eq!(
            config_serverbound_names(777).as_slice(),
            ids::configuration::serverbound::NAMES
        );
        assert_eq!(
            play_clientbound_names(777).as_slice(),
            ids::play::clientbound::NAMES
        );
        assert_eq!(
            play_serverbound_names(777).as_slice(),
            ids::play::serverbound::NAMES
        );
    }

    #[test]
    fn v1_20_1_protocol_763_packet_ids_match_mojang_spec() {
        let cb = play_clientbound_names(763);
        assert_eq!(cb.len(), 111);
        assert_eq!(index_of(&cb, "bundle_delimiter"), Some(0x00));
        assert_eq!(index_of(&cb, "add_entity"), Some(0x01));
        assert_eq!(index_of(&cb, "keep_alive"), Some(0x23));
        assert_eq!(index_of(&cb, "level_chunk_with_light"), Some(0x24));
        assert_eq!(index_of(&cb, "login"), Some(0x28));
        assert_eq!(index_of(&cb, "player_position"), Some(0x3C));
        assert_eq!(index_of(&cb, "system_chat"), Some(0x64));
        assert_eq!(index_of(&cb, "update_tags"), Some(0x6E));

        let sb = play_serverbound_names(763);
        assert_eq!(sb.len(), 51);
        assert_eq!(index_of(&sb, "accept_teleportation"), Some(0x00));
        assert_eq!(index_of(&sb, "keep_alive"), Some(0x12));
        assert_eq!(index_of(&sb, "move_player_pos"), Some(0x14));
        assert_eq!(index_of(&sb, "move_player_pos_rot"), Some(0x15));
        assert_eq!(index_of(&sb, "punch"), Some(0x2F)); // legacy `swing`
        assert_eq!(index_of(&sb, "use_item_on"), Some(0x31));
        assert_eq!(index_of(&sb, "use_item"), Some(0x32));
    }

    #[test]
    fn suppresses_newer_serverbound_packets_on_older_protocols() {
        // ClientTickEnd does not exist in 1.20.1 (763) or 1.21.1 (767)
        let tick_end = crate::packets::play::Serverbound::ClientTickEnd(crate::packets::Empty).encode();
        assert_eq!(translate_serverbound(763, State::Play, &tick_end).unwrap(), None);
        assert_eq!(translate_serverbound(767, State::Play, &tick_end).unwrap(), None);
        assert!(translate_serverbound(768, State::Play, &tick_end).unwrap().is_some());
    }

    #[test]
    fn adapts_legacy_player_position_to_777() {
        let mut legacy = Writer::new();
        legacy.f64(12.5);
        legacy.f64(64.0);
        legacy.f64(-8.25);
        legacy.f32(90.0);
        legacy.f32(15.0);
        legacy.u8(0);
        legacy.varint(42);
        let (id, payload) = translate_clientbound(763, State::Play, 0x3C, &legacy.data).unwrap();
        let decoded = crate::packets::play::Clientbound::decode(id, &payload).unwrap();
        match decoded {
            crate::packets::play::Clientbound::PlayerPosition(p) => {
                assert_eq!(p.teleport, 42);
                assert_eq!(p.position, [12.5, 64.0, -8.25]);
                assert_eq!(p.yaw, 90.0);
                assert_eq!(p.pitch, 15.0);
            }
            other => panic!("unexpected packet: {other:?}"),
        }
    }
}
