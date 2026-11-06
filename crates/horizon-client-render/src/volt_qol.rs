//! Unified pure-Rust configurations, data structures, and HUD/screen engines for:
//! - **Volt QoL** (`#2` Zoom & Freelook, `#3` World Inspector & Spawn-Light Overlay,
//!   `#8` Volt Blueprint 3D Hologram, `#9` Volt Cam & Replay Ring, `#15` Sound
//!   Physics Lite & Muffler, `#17` Interactive World Atlas, `#18` Volt JEI Recipe
//!   Browser, `#25` First-Person Hand & Viewmodel Customizer)
//! - **Volt Chat** (`#21` 3D Positional Proximity Voice Chat UDP LAN/SVC engine +
//!   Chat Timestamps, Spam Stacker, Quick-Chat Bar & Mention Ping)
//! - **Volt Visuals** (`#19` Seamless Connected Glass/Textures, Better Grass Sides,
//!   3D Dropped Items)
//! - **Volt Boost+** (`#6` Coplanar Greedy Quad Merge, Cave Occlusion, Mesh Pool,
//!   Offscreen Entity Skip, Smart Frame Pacer)
//! - **Volt Mods** (24 built-in pure-Rust native gameplay/PvP/survival/tech mods)

use std::collections::{HashMap, HashSet, VecDeque};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

use crate::hud::{self, HudAtlas, HudBatch};

// ============================================================================
// 1. VOLT QOL CONFIG (`Page::VoltQol`)
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BlueprintPreset {
    #[default]
    StarterCabin,
    Watchtower,
    CropFarm9x9,
    PortalShrine,
    MobFarmDeck,
    WorldClipboard,
}

impl BlueprintPreset {
    pub const ALL: [Self; 6] = [
        Self::StarterCabin,
        Self::Watchtower,
        Self::CropFarm9x9,
        Self::PortalShrine,
        Self::MobFarmDeck,
        Self::WorldClipboard,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::StarterCabin => "starter_cabin",
            Self::Watchtower => "watchtower",
            Self::CropFarm9x9 => "crop_farm",
            Self::PortalShrine => "portal_shrine",
            Self::MobFarmDeck => "mob_farm",
            Self::WorldClipboard => "clipboard",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::StarterCabin => "Cozy Oak Cabin (7x5x7)",
            Self::Watchtower => "Stone Watchtower (5x9x5)",
            Self::CropFarm9x9 => "Irrigated 9x9 Farm",
            Self::PortalShrine => "Obsidian Portal Frame",
            Self::MobFarmDeck => "Spawning Pad (9x4x9)",
            Self::WorldClipboard => "Copied World Region",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AtlasViewMode {
    #[default]
    Surface,
    HeightRelief,
    BiomeTint,
    SlimeChunks,
}

impl AtlasViewMode {
    pub const ALL: [Self; 4] = [
        Self::Surface,
        Self::HeightRelief,
        Self::BiomeTint,
        Self::SlimeChunks,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Surface => "surface",
            Self::HeightRelief => "relief",
            Self::BiomeTint => "biome",
            Self::SlimeChunks => "slime",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Surface => "Surface + Relief",
            Self::HeightRelief => "Elevation Contour",
            Self::BiomeTint => "Lush Biome Map",
            Self::SlimeChunks => "Slime Chunk Radar",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoltQolConfig {
    // #2 Zoom & 360 Freelook
    pub zoom_enabled: bool,
    pub zoom_fov_pct: u16,
    pub zoom_smooth: bool,
    pub freelook_enabled: bool,
    // #3 World Inspector & Mob-Spawn Light-0 Overlay
    pub world_inspector: bool,
    pub spawn_light_overlay: bool,
    // #8 Volt Blueprint
    pub blueprint_enabled: bool,
    pub blueprint_preset: BlueprintPreset,
    pub blueprint_layer: i16,
    pub blueprint_checklist: bool,
    // #9 Volt Cam & Replay Ring
    pub photo_mode_overlays: bool,
    pub cam_roll_deg: i16,
    pub cam_letterbox: bool,
    pub replay_buffer: bool,
    // #15 Sound Physics Lite & Muffler
    pub sound_cave_reverb: bool,
    pub sound_underwater_lpf: bool,
    pub sound_wall_occlusion: bool,
    pub muffle_portal_pct: u16,
    pub muffle_piston_pct: u16,
    pub muffle_minecart_pct: u16,
    pub muffle_explosion_pct: u16,
    pub muffle_rain_pct: u16,
    // #17 World Atlas
    pub atlas_enabled: bool,
    pub atlas_mode: AtlasViewMode,
    pub atlas_grid: bool,
    // #25 First-Person Hand & Viewmodel Customizer
    pub viewmodel_custom: bool,
    pub vm_offset_x: i16,
    pub vm_offset_y: i16,
    pub vm_offset_z: i16,
    pub vm_scale_pct: u16,
    pub vm_roll_deg: i16,
    pub vm_swing_speed_pct: u16,
    pub vm_classic_animations: bool,
    pub vm_totem_scale_pct: u16,
}

impl Default for VoltQolConfig {
    fn default() -> Self {
        Self {
            zoom_enabled: false,
            zoom_fov_pct: 28,
            zoom_smooth: true,
            freelook_enabled: false,
            world_inspector: false,
            spawn_light_overlay: false,
            blueprint_enabled: false,
            blueprint_preset: BlueprintPreset::StarterCabin,
            blueprint_layer: -1,
            blueprint_checklist: true,
            photo_mode_overlays: false,
            cam_roll_deg: 0,
            cam_letterbox: false,
            replay_buffer: false,
            sound_cave_reverb: false,
            sound_underwater_lpf: true,
            sound_wall_occlusion: false,
            muffle_portal_pct: 100,
            muffle_piston_pct: 100,
            muffle_minecart_pct: 100,
            muffle_explosion_pct: 100,
            muffle_rain_pct: 100,
            atlas_enabled: true,
            atlas_mode: AtlasViewMode::Surface,
            atlas_grid: true,
            viewmodel_custom: false,
            vm_offset_x: 0,
            vm_offset_y: 0,
            vm_offset_z: 0,
            vm_scale_pct: 100,
            vm_roll_deg: 0,
            vm_swing_speed_pct: 100,
            vm_classic_animations: false,
            vm_totem_scale_pct: 100,
        }
    }
}

// ============================================================================
// 2. VOLT VISUALS & VOLT BOOST+ CONFIGS (`#19` & `#6`)
// ============================================================================

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct VoltVisualsConfig {
    /// `#19` Seamless Connected Glass, Stained Glass & Panes (`mesher.rs` UV border trim).
    pub connected_glass: bool,
    /// `#19` Lush full-green Grass Block sides when adjacent to grass/foliage.
    pub better_grass_sides: bool,
    /// `#19` 3D Physics-resting / tumbling dropped item presentation.
    pub dropped_items_3d: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct VoltBoostPlusConfig {
    /// `#6` Coalesces adjacent identical coplanar opaque block faces in `mesher.rs`.
    pub greedy_quad_merge: bool,
    /// `#6` Culls sealed underground/indoor chunk sections occluded from camera.
    pub cave_chunk_occlusion: bool,
    /// `#6` Recycles chunk mesh allocations to eliminate GC/allocator frame spikes.
    pub zero_stutter_mesh_pool: bool,
    /// `#6` Skips distant block-entity animations & off-frustum entity pose updates.
    pub offscreen_entity_skip: bool,
    /// `#6` Smooths mobile frame pacing to reduce thermal spikes on Mali/Adreno GPUs.
    pub smart_frame_pacer: bool,
}

// ============================================================================
// 3. VOLT CHAT & 3D PROXIMITY VOICE CONFIG + UDP ENGINE (`#21`)
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoltChatConfig {
    pub voice_enabled: bool,
    pub voice_ptt_mode: bool,
    pub voice_vad_threshold: u16,
    pub voice_mic_gain_pct: u16,
    pub voice_output_gain_pct: u16,
    pub voice_distance_blocks: u16,
    pub voice_cave_muffle: bool,
    pub voice_hud_button: bool,
    pub voice_loopback_test: bool,
    pub voice_port: u16,
    // Chat enhancements
    pub chat_timestamps: bool,
    pub chat_compact_spam: bool,
    pub chat_mention_ping: bool,
    pub chat_quick_bar: bool,
}

impl Default for VoltChatConfig {
    fn default() -> Self {
        Self {
            voice_enabled: false,
            voice_ptt_mode: true,
            voice_vad_threshold: 22,
            voice_mic_gain_pct: 100,
            voice_output_gain_pct: 100,
            voice_distance_blocks: 48,
            voice_cave_muffle: true,
            voice_hud_button: true,
            voice_loopback_test: false,
            voice_port: 24454,
            chat_timestamps: false,
            chat_compact_spam: false,
            chat_mention_ping: true,
            chat_quick_bar: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VoicePeerState {
    pub name: String,
    pub pos: [f32; 3],
    pub level: f32,
    pub last_seen_s: f32,
    pub distance_m: f32,
}

/// ITU G.711 mu-law companding for ultra-low-CPU 16kHz voice frames (zero unsafe).
pub fn encode_mulaw(sample: i16) -> u8 {
    const BIAS: i32 = 0x84;
    const CLIP: i32 = 32635;
    let mut pcm = i32::from(sample);
    let sign = if pcm < 0 {
        pcm = -pcm;
        0x80u8
    } else {
        0x00u8
    };
    if pcm > CLIP {
        pcm = CLIP;
    }
    pcm += BIAS;
    let mut exp = 7u8;
    let mut mask = 0x4000i32;
    while exp > 0 && (pcm & mask) == 0 {
        exp -= 1;
        mask >>= 1;
    }
    let mantissa = ((pcm >> (exp + 3)) & 0x0F) as u8;
    !(sign | (exp << 4) | mantissa)
}

pub fn decode_mulaw(byte: u8) -> i16 {
    let u = !byte;
    let sign = (u & 0x80) != 0;
    let exp = ((u >> 4) & 0x07) as i32;
    let mantissa = (u & 0x0F) as i32;
    let mut sample = ((mantissa << 3) + 0x84) << exp;
    sample -= 0x84;
    if sign {
        (-sample).clamp(-32768, 32767) as i16
    } else {
        sample.clamp(-32768, 32767) as i16
    }
}

/// Safe-Rust non-blocking UDP 3D Positional Proximity Voice Chat engine.
/// Supports LAN broadcast (`255.255.255.255:port`) AND direct server relay
/// (`server_ip:port`, default `24454` matching Simple Voice Chat port).
pub struct VoltVoiceRuntime {
    socket: Option<UdpSocket>,
    bound_port: u16,
    pub ptt_held: bool,
    pub local_speaking: bool,
    pub local_mic_level: f32,
    pub peers: HashMap<u64, VoicePeerState>,
    pub muted_peers: HashSet<String>,
    pub mixed_pcm_out: VecDeque<[f32; 2]>,
    seq: u32,
    last_tx_s: f32,
}

impl Default for VoltVoiceRuntime {
    fn default() -> Self {
        Self {
            socket: None,
            bound_port: 0,
            ptt_held: false,
            local_speaking: false,
            local_mic_level: 0.0,
            peers: HashMap::new(),
            muted_peers: HashSet::new(),
            mixed_pcm_out: VecDeque::with_capacity(4096),
            seq: 0,
            last_tx_s: 0.0,
        }
    }
}

impl VoltVoiceRuntime {
    pub fn tick(
        &mut self,
        cfg: &VoltChatConfig,
        local_id: u64,
        local_name: &str,
        listener_pos: [f32; 3],
        listener_yaw_deg: f32,
        in_cave: bool,
        server_host: Option<&str>,
        mic_samples_16k: &[i16],
        now_s: f32,
    ) {
        if !cfg.voice_enabled {
            if self.socket.is_some() {
                self.socket = None;
                self.peers.clear();
                self.mixed_pcm_out.clear();
                self.local_speaking = false;
                self.local_mic_level = 0.0;
            }
            return;
        }

        if self.socket.is_none() || self.bound_port != cfg.voice_port {
            self.bound_port = cfg.voice_port;
            let sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, cfg.voice_port))
                .or_else(|_| UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)));
            if let Ok(s) = sock {
                let _ = s.set_nonblocking(true);
                let _ = s.set_broadcast(true);
                self.socket = Some(s);
            }
        }

        // Compute mic RMS & transmit 20ms frame (`VV01` packet) when PTT or VAD active
        let mic_gain = f32::from(cfg.voice_mic_gain_pct.clamp(0, 200)) / 100.0;
        let mut rms = 0.0f32;
        if !mic_samples_16k.is_empty() {
            let sum_sq: f32 = mic_samples_16k
                .iter()
                .map(|&s| {
                    let v = (f32::from(s) / 32768.0) * mic_gain;
                    v * v
                })
                .sum();
            rms = (sum_sq / mic_samples_16k.len() as f32).sqrt();
        } else if cfg.voice_ptt_mode && self.ptt_held {
            // Subtle carrier comfort indicator when holding PTT without hardware mic capture
            rms = 0.18 + (now_s * 9.0).sin().abs() * 0.14;
        }
        self.local_mic_level = (rms * 2.4).clamp(0.0, 1.0);
        let vad_gate = f32::from(cfg.voice_vad_threshold.clamp(5, 80)) / 200.0;
        self.local_speaking = if cfg.voice_ptt_mode {
            self.ptt_held
        } else {
            self.local_mic_level >= vad_gate
        };

        if self.local_speaking && (now_s - self.last_tx_s) >= 0.020 {
            self.last_tx_s = now_s;
            self.seq = self.seq.wrapping_add(1);
            let mut pkt = Vec::with_capacity(384);
            pkt.extend_from_slice(b"VV01");
            pkt.extend_from_slice(&local_id.to_le_bytes());
            pkt.extend_from_slice(&self.seq.to_le_bytes());
            for c in listener_pos {
                pkt.extend_from_slice(&c.to_le_bytes());
            }
            let name_bytes = local_name.as_bytes();
            let nlen = name_bytes.len().min(24) as u8;
            pkt.push(nlen);
            pkt.extend_from_slice(&name_bytes[..usize::from(nlen)]);
            let frame_len = mic_samples_16k.len().min(320);
            pkt.extend_from_slice(&(frame_len as u16).to_le_bytes());
            for &s in &mic_samples_16k[..frame_len] {
                let scaled = (f32::from(s) * mic_gain).clamp(-32767.0, 32767.0) as i16;
                pkt.push(encode_mulaw(scaled));
            }
            if let Some(sock) = &self.socket {
                let bcast = SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::BROADCAST,
                    cfg.voice_port.max(1024),
                ));
                let _ = sock.send_to(&pkt, bcast);
                if let Some(host) = server_host
                    && let Ok(ip) = host.split(':').next().unwrap_or("").parse::<Ipv4Addr>()
                {
                    let _ = sock.send_to(
                        &pkt,
                        SocketAddr::V4(SocketAddrV4::new(ip, cfg.voice_port.max(1024))),
                    );
                }
            }
            if cfg.voice_loopback_test && !mic_samples_16k.is_empty() {
                let out_gain = f32::from(cfg.voice_output_gain_pct.clamp(0, 200)) / 100.0;
                for &s in &mic_samples_16k[..frame_len] {
                    if self.mixed_pcm_out.len() < 8192 {
                        let f = (f32::from(s) / 32768.0) * mic_gain * out_gain * 0.6;
                        self.mixed_pcm_out.push_back([f, f]);
                    }
                }
            }
        }

        // Receive & 3D spatialize incoming peer voice packets
        let max_dist = f32::from(cfg.voice_distance_blocks.clamp(8, 128));
        let out_gain = f32::from(cfg.voice_output_gain_pct.clamp(0, 200)) / 100.0;
        let (yaw_s, yaw_c) = listener_yaw_deg.to_radians().sin_cos();

        if let Some(sock) = &self.socket {
            let mut buf = [0u8; 640];
            while let Ok((len, _addr)) = sock.recv_from(&mut buf) {
                if len < 31 || &buf[0..4] != b"VV01" {
                    continue;
                }
                let sender_id = u64::from_le_bytes([
                    buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11],
                ]);
                if sender_id == local_id && !cfg.voice_loopback_test {
                    continue;
                }
                let px = f32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
                let py = f32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
                let pz = f32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]);
                let nlen = usize::from(buf[28]);
                if 29 + nlen + 2 > len {
                    continue;
                }
                let peer_name = String::from_utf8_lossy(&buf[29..29 + nlen]).into_owned();
                if self.muted_peers.contains(&peer_name) {
                    continue;
                }
                let dx = px - listener_pos[0];
                let dy = py - listener_pos[1];
                let dz = pz - listener_pos[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist > max_dist {
                    continue;
                }
                let atten = (1.0 - (dist / max_dist)).powi(2)
                    * if cfg.voice_cave_muffle && in_cave && dist > 10.0 {
                        0.68
                    } else {
                        1.0
                    };
                // Right-ear vector in Minecraft camera coords
                let right_x = -yaw_c;
                let right_z = -yaw_s;
                let pan = if dist > 0.25 {
                    ((dx * right_x + dz * right_z) / dist).clamp(-0.85, 0.85)
                } else {
                    0.0
                };
                let gain_l = atten * out_gain * (1.0 - pan * 0.45);
                let gain_r = atten * out_gain * (1.0 + pan * 0.45);

                let audio_off = 29 + nlen;
                let audio_len =
                    usize::from(u16::from_le_bytes([buf[audio_off], buf[audio_off + 1]]));
                let payload = &buf[audio_off + 2..len.min(audio_off + 2 + audio_len)];
                let mut peak = 0.15f32;
                for &b in payload {
                    let s = f32::from(decode_mulaw(b)) / 32768.0;
                    peak = peak.max(s.abs() * 2.2);
                    if self.mixed_pcm_out.len() < 8192 {
                        // Upsample 16kHz -> ~48kHz (3x hold) for smooth playback mixing
                        let frame = [s * gain_l, s * gain_r];
                        self.mixed_pcm_out.push_back(frame);
                        self.mixed_pcm_out.push_back(frame);
                        self.mixed_pcm_out.push_back(frame);
                    }
                }
                self.peers.insert(
                    sender_id,
                    VoicePeerState {
                        name: peer_name,
                        pos: [px, py, pz],
                        level: peak.clamp(0.0, 1.0),
                        last_seen_s: now_s,
                        distance_m: dist,
                    },
                );
            }
        }
        self.peers.retain(|_, p| (now_s - p.last_seen_s) < 1.25);
    }
}

// ============================================================================
// 4. VOLT MODS CONFIG (24 Built-In Pure-Rust Native Mods, all OFF by default)
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CrosshairStyle {
    #[default]
    Vanilla,
    PrecisionDot,
    CrispPlus,
    CircleRing,
    PvpChevron,
}

impl CrosshairStyle {
    pub const ALL: [Self; 5] = [
        Self::Vanilla,
        Self::PrecisionDot,
        Self::CrispPlus,
        Self::CircleRing,
        Self::PvpChevron,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::PrecisionDot => "dot",
            Self::CrispPlus => "crisp_plus",
            Self::CircleRing => "circle",
            Self::PvpChevron => "chevron",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::PrecisionDot => "Precision Dot",
            Self::CrispPlus => "Crisp Plus",
            Self::CircleRing => "Tactical Ring",
            Self::PvpChevron => "PvP Chevron",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ClientTimeWeather {
    #[default]
    ServerLive,
    SunnyNoon,
    GoldenSunset,
    StarryMidnight,
    CozyRain,
}

impl ClientTimeWeather {
    pub const ALL: [Self; 5] = [
        Self::ServerLive,
        Self::SunnyNoon,
        Self::GoldenSunset,
        Self::StarryMidnight,
        Self::CozyRain,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ServerLive => "live",
            Self::SunnyNoon => "noon",
            Self::GoldenSunset => "sunset",
            Self::StarryMidnight => "midnight",
            Self::CozyRain => "rain",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ServerLive => "Server Live",
            Self::SunnyNoon => "Sunny Noon",
            Self::GoldenSunset => "Golden Sunset",
            Self::StarryMidnight => "Starry Night",
            Self::CozyRain => "Cozy Rain",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct VoltModsConfig {
    // Combat & PvP HUDs
    pub armor_hud: bool,
    pub potion_hud: bool,
    pub cps_hud: bool,
    pub reach_hud: bool,
    pub fps_ping_pill: bool,
    pub keystrokes_hud: bool,
    pub custom_crosshair: CrosshairStyle,
    pub low_shield_fire: bool,
    pub no_hurtcam: bool,
    pub toggle_sprint: bool,
    // Survival & Inventory
    pub appleskin_hud: bool,
    pub shulker_preview: bool,
    pub tool_break_shield: bool,
    pub auto_tool_hint: bool,
    // World & Combat Visuals
    pub trajectory_arc: bool,
    pub damage_popups: bool,
    pub hitbox_plus: bool,
    pub damage_direction_hud: bool,
    // Utility, Technical & Atmosphere
    pub chat_calculator: bool,
    pub enchant_guide: bool,
    pub slime_tech_hud: bool,
    pub death_tracker: bool,
    pub time_weather: ClientTimeWeather,
    pub oled_afk_saver: bool,
}

impl VoltModsConfig {
    pub fn active_count(&self) -> usize {
        let bools = [
            self.armor_hud,
            self.potion_hud,
            self.cps_hud,
            self.reach_hud,
            self.fps_ping_pill,
            self.keystrokes_hud,
            self.custom_crosshair != CrosshairStyle::Vanilla,
            self.low_shield_fire,
            self.no_hurtcam,
            self.toggle_sprint,
            self.appleskin_hud,
            self.shulker_preview,
            self.tool_break_shield,
            self.auto_tool_hint,
            self.trajectory_arc,
            self.damage_popups,
            self.hitbox_plus,
            self.damage_direction_hud,
            self.chat_calculator,
            self.enchant_guide,
            self.slime_tech_hud,
            self.death_tracker,
            self.time_weather != ClientTimeWeather::ServerLive,
            self.oled_afk_saver,
        ];
        bools.into_iter().filter(|&v| v).count()
    }
}

/// Evaluates an inline chat calculator expression (e.g. `"= 64*27 + 12"` -> `Some(1740.0)`).
pub fn eval_chat_math(input: &str) -> Option<f64> {
    let expr = input.trim().strip_prefix('=')?.trim();
    if expr.is_empty() {
        return None;
    }
    fn parse_expr(s: &mut &str) -> Option<f64> {
        let mut lhs = parse_term(s)?;
        loop {
            *s = s.trim_start();
            if let Some(rest) = s.strip_prefix('+') {
                *s = rest;
                lhs += parse_term(s)?;
            } else if let Some(rest) = s.strip_prefix('-') {
                *s = rest;
                lhs -= parse_term(s)?;
            } else {
                break;
            }
        }
        Some(lhs)
    }
    fn parse_term(s: &mut &str) -> Option<f64> {
        let mut lhs = parse_factor(s)?;
        loop {
            *s = s.trim_start();
            if let Some(rest) = s.strip_prefix('*') {
                *s = rest;
                lhs *= parse_factor(s)?;
            } else if let Some(rest) = s.strip_prefix('/') {
                *s = rest;
                let rhs = parse_factor(s)?;
                if rhs.abs() < 1e-12 {
                    return None;
                }
                lhs /= rhs;
            } else {
                break;
            }
        }
        Some(lhs)
    }
    fn parse_factor(s: &mut &str) -> Option<f64> {
        *s = s.trim_start();
        if let Some(rest) = s.strip_prefix('(') {
            *s = rest;
            let v = parse_expr(s)?;
            *s = s.trim_start().strip_prefix(')')?;
            return Some(v);
        }
        if let Some(rest) = s.strip_prefix('-') {
            *s = rest;
            return Some(-parse_factor(s)?);
        }
        let end = s
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(s.len());
        if end == 0 {
            return None;
        }
        let val: f64 = s[..end].parse().ok()?;
        *s = &s[end..];
        Some(val)
    }
    let mut slice = expr;
    let out = parse_expr(&mut slice)?;
    if slice.trim().is_empty() && out.is_finite() {
        Some(out)
    } else {
        None
    }
}

/// Deterministic Minecraft Java slime-chunk formula (`Random` seeded by chunk coords + world seed `0` fallback).
pub fn is_slime_chunk(cx: i32, cz: i32, seed: i64) -> bool {
    let term1 = (cx as i64).wrapping_mul(cx as i64).wrapping_mul(0x4c1906);
    let term2 = (cx as i64).wrapping_mul(0x5ac0db);
    let term3 = (cz as i64).wrapping_mul(cz as i64).wrapping_mul(0x4307a7);
    let term4 = (cz as i64).wrapping_mul(0x5f24f);
    let mut rnd = seed
        .wrapping_add(term1)
        .wrapping_add(term2)
        .wrapping_add(term3)
        .wrapping_add(term4)
        ^ 0x3ad8025f;
    rnd = (rnd ^ 0x5DEECE66D) & ((1i64 << 48) - 1);
    let bits = loop {
        rnd = (rnd.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & ((1i64 << 48) - 1);
        let b = (rnd >> 17) as i32;
        let v = b % 10;
        if b - v + 9 >= 0 {
            break v;
        }
    };
    bits == 0
}

// ============================================================================
// 5. PERSISTENT INTERACTIVE WORLD ATLAS (`#17` — `Page::VoltAtlas`)
// ============================================================================

#[derive(Clone, Debug)]
pub struct AtlasChunkTile {
    /// Packed RGB `[r, g, b]` per `4x4` sub-cell (`4x4 = 16` cells per chunk -> tiny 64B footprint!).
    pub rgb: [[u8; 3]; 16],
    pub height: [i16; 16],
}

#[derive(Clone, Debug)]
pub struct WorldAtlasState {
    pub tiles: HashMap<(i32, i32), AtlasChunkTile>,
    pub pan_x: f32,
    pub pan_z: f32,
    pub zoom: f32,
    pub player_pos: [f32; 3],
    pub player_yaw: f32,
    pub dragging: bool,
    pub drag_moved: bool,
    pub last_cursor: [f32; 2],
    pub hover_world: [i32; 2],
    pub pending_waypoint: Option<[i32; 3]>,
}

impl Default for WorldAtlasState {
    fn default() -> Self {
        Self {
            tiles: HashMap::new(),
            pan_x: 0.0,
            pan_z: 0.0,
            zoom: 2.0,
            player_pos: [0.0; 3],
            player_yaw: 0.0,
            dragging: false,
            drag_moved: false,
            last_cursor: [0.0; 2],
            hover_world: [0; 2],
            pending_waypoint: None,
        }
    }
}

impl WorldAtlasState {
    pub fn center_on_player(&mut self) {
        self.pan_x = self.player_pos[0];
        self.pan_z = self.player_pos[2];
    }

    pub fn record_tile(&mut self, cx: i32, cz: i32, tile: AtlasChunkTile) {
        if self.tiles.len() >= 4096 && !self.tiles.contains_key(&(cx, cz)) {
            // Keep memory bounded on ultra-long journeys
            let px = (self.player_pos[0] / 16.0) as i32;
            let pz = (self.player_pos[2] / 16.0) as i32;
            self.tiles
                .retain(|&(kx, kz), _| (kx - px).abs() <= 48 && (kz - pz).abs() <= 48);
        }
        self.tiles.insert((cx, cz), tile);
    }

    pub fn draw_screen(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        cfg: &VoltQolConfig,
        waypoints: &[(String, [f64; 3], [u8; 3])],
    ) {
        let w = batch.width;
        let h = batch.height;
        let map_x = 12.0;
        let map_y = 30.0;
        let map_w = (w - 24.0).max(160.0);
        let map_h = (h - 68.0).max(90.0);

        batch.fill(atlas, map_x - 1.0, map_y - 1.0, map_w + 2.0, map_h + 2.0, hud::rgba(214, 42, 62, 220));
        batch.fill(atlas, map_x, map_y, map_w, map_h, hud::rgba(10, 13, 20, 244));

        let cx_scr = map_x + map_w * 0.5;
        let cy_scr = map_y + map_h * 0.5;
        let cell_px = (4.0 * self.zoom).clamp(2.0, 24.0);
        let chunk_px = cell_px * 4.0;

        let half_chunks_x = ((map_w * 0.5) / chunk_px).ceil() as i32 + 1;
        let half_chunks_z = ((map_h * 0.5) / chunk_px).ceil() as i32 + 1;
        let center_cx = (self.pan_x / 16.0).floor() as i32;
        let center_cz = (self.pan_z / 16.0).floor() as i32;

        for dz in -half_chunks_z..=half_chunks_z {
            for dx in -half_chunks_x..=half_chunks_x {
                let tcx = center_cx + dx;
                let tcz = center_cz + dz;
                let world_x0 = (tcx * 16) as f32;
                let world_z0 = (tcz * 16) as f32;
                let sx0 = cx_scr + (world_x0 - self.pan_x) * self.zoom;
                let sy0 = cy_scr + (world_z0 - self.pan_z) * self.zoom;
                if sx0 + chunk_px <= map_x
                    || sx0 >= map_x + map_w
                    || sy0 + chunk_px <= map_y
                    || sy0 >= map_y + map_h
                {
                    continue;
                }

                if let Some(tile) = self.tiles.get(&(tcx, tcz)) {
                    for sub_z in 0..4 {
                        for sub_x in 0..4 {
                            let idx = sub_z * 4 + sub_x;
                            let px = (sx0 + sub_x as f32 * cell_px).max(map_x);
                            let py = (sy0 + sub_z as f32 * cell_px).max(map_y);
                            let pw = ((sx0 + (sub_x + 1) as f32 * cell_px).min(map_x + map_w) - px).max(0.0);
                            let ph = ((sy0 + (sub_z + 1) as f32 * cell_px).min(map_y + map_h) - py).max(0.0);
                            if pw <= 0.1 || ph <= 0.1 {
                                continue;
                            }
                            let mut c = tile.rgb[idx];
                            let h_val = tile.height[idx];
                            match cfg.atlas_mode {
                                AtlasViewMode::Surface => {
                                    let prev_h = if sub_z > 0 {
                                        tile.height[(sub_z - 1) * 4 + sub_x]
                                    } else {
                                        h_val
                                    };
                                    let slope = ((h_val - prev_h) * 14).clamp(-36, 36);
                                    c[0] = (i32::from(c[0]) + i32::from(slope)).clamp(8, 255) as u8;
                                    c[1] = (i32::from(c[1]) + i32::from(slope)).clamp(8, 255) as u8;
                                    c[2] = (i32::from(c[2]) + i32::from(slope)).clamp(8, 255) as u8;
                                }
                                AtlasViewMode::HeightRelief => {
                                    let norm = ((h_val + 64).clamp(0, 256) as f32 / 256.0).clamp(0.0, 1.0);
                                    c = [
                                        (38.0 + norm * 205.0) as u8,
                                        (84.0 + (1.0 - (norm - 0.45).abs() * 1.8).max(0.0) * 150.0) as u8,
                                        (168.0 - norm * 120.0).max(28.0) as u8,
                                    ];
                                }
                                AtlasViewMode::BiomeTint => {
                                    c = [
                                        (u16::from(c[0]) * 9 / 10).min(255) as u8,
                                        (u16::from(c[1]) * 11 / 10).min(255) as u8,
                                        (u16::from(c[2]) * 10 / 10).min(255) as u8,
                                    ];
                                }
                                AtlasViewMode::SlimeChunks => {
                                    if is_slime_chunk(tcx, tcz, 0) {
                                        c = [68, 232, 96];
                                    } else {
                                        c = [c[0] / 3 + 12, c[1] / 3 + 14, c[2] / 3 + 20];
                                    }
                                }
                            }
                            batch.fill(atlas, px, py, pw, ph, hud::rgba(c[0], c[1], c[2], 255));
                        }
                    }
                } else if cfg.atlas_mode == AtlasViewMode::SlimeChunks && is_slime_chunk(tcx, tcz, 0)
                {
                    let px = sx0.max(map_x);
                    let py = sy0.max(map_y);
                    let pw = ((sx0 + chunk_px).min(map_x + map_w) - px).max(0.0);
                    let ph = ((sy0 + chunk_px).min(map_y + map_h) - py).max(0.0);
                    if pw > 0.5 && ph > 0.5 {
                        batch.fill(atlas, px, py, pw, ph, hud::rgba(42, 148, 66, 145));
                    }
                }

                if cfg.atlas_grid && chunk_px >= 10.0 && sx0 >= map_x && sx0 < map_x + map_w {
                    batch.fill(atlas, sx0, map_y, 0.75, map_h, hud::rgba(255, 255, 255, 22));
                }
                if cfg.atlas_grid && chunk_px >= 10.0 && sy0 >= map_y && sy0 < map_y + map_h {
                    batch.fill(atlas, map_x, sy0, map_w, 0.75, hud::rgba(255, 255, 255, 22));
                }
            }
        }

        // Draw Waypoints on Atlas
        for (name, pos, col) in waypoints {
            let wx = cx_scr + (pos[0] as f32 - self.pan_x) * self.zoom;
            let wz = cy_scr + (pos[2] as f32 - self.pan_z) * self.zoom;
            if wx >= map_x + 4.0 && wx <= map_x + map_w - 4.0 && wz >= map_y + 4.0 && wz <= map_y + map_h - 4.0 {
                batch.fill(atlas, wx - 3.5, wz - 3.5, 7.0, 7.0, hud::rgba(0, 0, 0, 220));
                batch.fill(atlas, wx - 2.5, wz - 2.5, 5.0, 5.0, hud::rgba(col[0], col[1], col[2], 255));
                batch.text(atlas, name, wx + 5.0, wz - 4.0, hud::rgba(255, 255, 255, 240), true);
            }
        }

        // Player arrow marker
        let px = cx_scr + (self.player_pos[0] - self.pan_x) * self.zoom;
        let pz = cy_scr + (self.player_pos[2] - self.pan_z) * self.zoom;
        if px >= map_x && px <= map_x + map_w && pz >= map_y && pz <= map_y + map_h {
            batch.fill(atlas, px - 4.0, pz - 4.0, 8.0, 8.0, hud::rgba(255, 42, 68, 255));
            batch.fill(atlas, px - 2.0, pz - 2.0, 4.0, 4.0, hud::rgba(255, 255, 255, 255));
        }

        // Top HUD status bar inside Atlas
        let hc_x = self.hover_world[0].div_euclid(16);
        let hc_z = self.hover_world[1].div_euclid(16);
        let slime_tag = if is_slime_chunk(hc_x, hc_z, 0) {
            " [SLIME CHUNK]"
        } else {
            ""
        };
        let info = format!(
            "Mode: {}  |  Zoom: {:.1}x  |  Cursor: X {} Z {} (Chunk {}, {}){}  |  Tap Map = Pin Waypoint",
            cfg.atlas_mode.label(),
            self.zoom,
            self.hover_world[0],
            self.hover_world[1],
            hc_x,
            hc_z,
            slime_tag
        );
        batch.text(
            atlas,
            &info,
            map_x + 4.0,
            map_y + 4.0,
            hud::rgba(244, 246, 255, 245),
            true,
        );

        // Bottom mini toolbar buttons (`Mode`, `Zoom -`, `Zoom +`, `Center`, `+ Pin Center`)
        let btn_y = map_y + map_h + 5.0;
        for (i, lbl) in ["Mode", "Zoom -", "Zoom +", "Center", "+ Pin WP"].iter().enumerate() {
            let bx = map_x + i as f32 * 64.0;
            batch.fill(atlas, bx, btn_y, 60.0, 18.0, hud::rgba(32, 36, 52, 235));
            batch.fill(atlas, bx, btn_y, 60.0, 1.0, hud::rgba(218, 48, 68, 220));
            let tw = atlas.font.width(lbl);
            batch.text(
                atlas,
                lbl,
                bx + (60.0 - tw) * 0.5,
                btn_y + 5.0,
                hud::rgba(250, 250, 255, 255),
                true,
            );
        }
    }

    pub fn pointer_down(
        &mut self,
        cfg: &mut VoltQolConfig,
        (w, h): (f32, f32),
        (px, py): (f32, f32),
    ) -> (bool, bool) {
        let map_x = 12.0;
        let map_y = 30.0;
        let map_w = (w - 24.0).max(160.0);
        let map_h = (h - 68.0).max(90.0);
        let btn_y = map_y + map_h + 5.0;
        if py >= btn_y && py <= btn_y + 18.0 {
            let idx = ((px - map_x) / 64.0).floor() as i32;
            match idx {
                0 => {
                    cfg.atlas_mode = cfg.atlas_mode.cycle(false);
                    return (true, true);
                }
                1 => {
                    self.zoom = (self.zoom * 0.75).clamp(0.5, 8.0);
                    return (false, true);
                }
                2 => {
                    self.zoom = (self.zoom * 1.35).clamp(0.5, 8.0);
                    return (false, true);
                }
                3 => {
                    self.center_on_player();
                    return (false, true);
                }
                4 => {
                    self.pending_waypoint = Some([
                        self.pan_x.round() as i32,
                        self.player_pos[1].round() as i32,
                        self.pan_z.round() as i32,
                    ]);
                    return (true, true);
                }
                _ => {}
            }
        }
        if px >= map_x && px <= map_x + map_w && py >= map_y && py <= map_y + map_h {
            self.dragging = true;
            self.drag_moved = false;
            self.last_cursor = [px, py];
            self.update_hover((w, h), (px, py));
        }
        (false, false)
    }

    pub fn pointer_move(&mut self, (w, h): (f32, f32), (px, py): (f32, f32)) {
        self.update_hover((w, h), (px, py));
        if self.dragging {
            let dx = px - self.last_cursor[0];
            let dy = py - self.last_cursor[1];
            if dx.abs() + dy.abs() > 3.5 {
                self.drag_moved = true;
            }
            self.last_cursor = [px, py];
            self.pan_x -= dx / self.zoom.max(0.25);
            self.pan_z -= dy / self.zoom.max(0.25);
        }
    }

    pub fn pointer_up(&mut self, (w, h): (f32, f32), (px, py): (f32, f32)) -> bool {
        let was_drag = self.dragging;
        let moved = self.drag_moved;
        self.dragging = false;
        self.drag_moved = false;
        if was_drag && !moved {
            self.update_hover((w, h), (px, py));
            self.pending_waypoint = Some([
                self.hover_world[0],
                self.player_pos[1].round() as i32,
                self.hover_world[1],
            ]);
            return true;
        }
        false
    }

    pub fn scroll(&mut self, lines: f32) {
        let factor = if lines > 0.0 { 1.22 } else { 0.82 };
        self.zoom = (self.zoom * factor).clamp(0.5, 8.0);
    }

    fn update_hover(&mut self, (w, h): (f32, f32), (px, py): (f32, f32)) {
        let map_x = 12.0;
        let map_y = 30.0;
        let map_w = (w - 24.0).max(160.0);
        let map_h = (h - 68.0).max(90.0);
        let cx_scr = map_x + map_w * 0.5;
        let cy_scr = map_y + map_h * 0.5;
        let wx = self.pan_x + (px - cx_scr) / self.zoom.max(0.25);
        let wz = self.pan_z + (py - cy_scr) / self.zoom.max(0.25);
        self.hover_world = [wx.floor() as i32, wz.floor() as i32];
    }
}

// ============================================================================
// 6. VOLT BLUEPRINT TEMPLATES (`#8`) & VOLT JEI CATALOG (`#18`)
// ============================================================================

#[derive(Clone, Debug)]
pub struct BlueprintBlock {
    pub rel: [i32; 3],
    pub block_key: &'static str,
    pub color: [u8; 3],
}

pub fn preset_blueprint_blocks(preset: BlueprintPreset) -> Vec<BlueprintBlock> {
    let mut out = Vec::new();
    match preset {
        BlueprintPreset::StarterCabin => {
            for y in 0..5 {
                for z in 0..7 {
                    for x in 0..7 {
                        let edge = x == 0 || x == 6 || z == 0 || z == 6;
                        let corner = (x == 0 || x == 6) && (z == 0 || z == 6);
                        if y == 0 {
                            out.push(BlueprintBlock { rel: [x - 3, y, z - 3], block_key: "cobblestone", color: [132, 136, 142] });
                        } else if y == 4 {
                            out.push(BlueprintBlock { rel: [x - 3, y, z - 3], block_key: "oak_stairs", color: [178, 138, 82] });
                        } else if corner {
                            out.push(BlueprintBlock { rel: [x - 3, y, z - 3], block_key: "oak_log", color: [112, 84, 48] });
                        } else if edge {
                            if z == 0 && x == 3 && (y == 1 || y == 2) {
                                continue; // Doorway
                            }
                            if y == 2 && (x == 3 || z == 3) {
                                out.push(BlueprintBlock { rel: [x - 3, y, z - 3], block_key: "glass_pane", color: [158, 228, 255] });
                            } else {
                                out.push(BlueprintBlock { rel: [x - 3, y, z - 3], block_key: "oak_planks", color: [192, 152, 92] });
                            }
                        }
                    }
                }
            }
        }
        BlueprintPreset::Watchtower => {
            for y in 0i32..9 {
                for z in -2i32..=2 {
                    for x in -2i32..=2 {
                        let edge = x.abs() == 2 || z.abs() == 2;
                        if y == 0 || y == 7 {
                            out.push(BlueprintBlock { rel: [x, y, z], block_key: "stone_bricks", color: [148, 152, 158] });
                        } else if y == 8 && edge && ((x + z) & 1 == 0) {
                            out.push(BlueprintBlock { rel: [x, y, z], block_key: "stone_brick_wall", color: [168, 172, 178] });
                        } else if y < 7 && edge && !(z == -2 && x == 0 && y <= 2) {
                            out.push(BlueprintBlock { rel: [x, y, z], block_key: "stone_bricks", color: [142, 146, 154] });
                        }
                    }
                }
            }
        }
        BlueprintPreset::CropFarm9x9 => {
            for z in -4i32..=4 {
                for x in -4i32..=4 {
                    if x == 0 && z == 0 {
                        out.push(BlueprintBlock { rel: [x, 0, z], block_key: "water", color: [52, 148, 248] });
                    } else {
                        out.push(BlueprintBlock { rel: [x, 0, z], block_key: "farmland", color: [128, 86, 52] });
                        out.push(BlueprintBlock { rel: [x, 1, z], block_key: "wheat", color: [218, 198, 72] });
                    }
                }
            }
        }
        BlueprintPreset::PortalShrine => {
            for y in 0i32..5 {
                for x in -1i32..=2 {
                    if y == 0 || y == 4 || x == -1 || x == 2 {
                        out.push(BlueprintBlock { rel: [x, y, 0], block_key: "obsidian", color: [68, 38, 108] });
                    }
                }
            }
        }
        BlueprintPreset::MobFarmDeck | BlueprintPreset::WorldClipboard => {
            for z in -4i32..=4 {
                for x in -4i32..=4 {
                    if x.abs() <= 1 && z.abs() <= 1 {
                        continue; // Drop chute
                    }
                    out.push(BlueprintBlock { rel: [x, 0, z], block_key: "deepslate_tiles", color: [78, 82, 92] });
                    if x.abs() == 4 || z.abs() == 4 {
                        out.push(BlueprintBlock { rel: [x, 1, z], block_key: "tinted_glass", color: [118, 92, 142] });
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mulaw_codec_roundtrips_speech_waveform_accurately() {
        for sample in [-24000i16, -4200, -320, 0, 320, 4200, 24000] {
            let enc = encode_mulaw(sample);
            let dec = decode_mulaw(enc);
            let tol = (i32::from(sample).abs() / 16).max(32);
            assert!(
                (i32::from(sample) - i32::from(dec)).abs() <= tol,
                "sample {sample} decoded as {dec}"
            );
        }
    }

    #[test]
    fn chat_calculator_evaluates_stack_arithmetic() {
        assert_eq!(eval_chat_math("= 64*27+12"), Some(1740.0));
        assert_eq!(eval_chat_math("= (128 / 8) * 3"), Some(48.0));
        assert_eq!(eval_chat_math("hello world"), None);
    }

    #[test]
    fn volt_mods_default_all_off() {
        let m = VoltModsConfig::default();
        assert_eq!(m.active_count(), 0);
    }
}
