//! Structured actions, labels, tooltips, and section definitions for the
//! consolidated **Volt Settings** sub-pages (`VoltVisuals`, `VoltQol`,
//! `VoltChat`, `VoltMods`, `VoltBoost` `#6` Engine+, and `VoltShaders`
//! BSL + Complementary controls).

use crate::extras::Extras;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VoltAct {
    // --- BSL + Complementary Volt Shaders additions ---
    ShaderStyle,
    ShaderEyeComfort,
    ShaderCloudsMode,
    ShaderCloudCoverage,
    ShaderCloudShadows,
    ShaderGlowingOres,
    ShaderOreGlow,
    ShaderAutoPbr,
    ShaderWaterFoam,
    ShaderNightSkyPlus,
    ShaderRainbow,
    // --- #19 Volt Visuals ---
    VisualsCtmGlass,
    VisualsBetterGrass,
    VisualsDroppedItems3d,
    // --- #6 Volt Boost+ Vulkan Engine Passes ---
    BoostGreedyMerge,
    BoostCaveOcclusion,
    BoostMeshPool,
    BoostOffscreenSkip,
    BoostFramePacer,
    // --- Volt QoL (#2, #3, #8, #9, #15, #17, #18, #25) ---
    QolZoom,
    QolZoomFov,
    QolZoomSmooth,
    QolFreelook,
    QolInspector,
    QolSpawnLight,
    QolBlueprint,
    QolBlueprintPreset,
    QolBlueprintLayer,
    QolBlueprintChecklist,
    QolPhotoOverlays,
    QolCamRoll,
    QolLetterbox,
    QolReplay,
    QolReplayButton,
    QolCaveReverb,
    QolUnderwaterLpf,
    QolWallOcclusion,
    QolMufflePortal,
    QolMufflePiston,
    QolMuffleMinecart,
    QolMuffleExplosion,
    QolMuffleRain,
    QolAtlas,
    QolAtlasMode,
    QolAtlasGrid,
    QolVmCustom,
    QolVmX,
    QolVmY,
    QolVmZ,
    QolVmScale,
    QolVmRoll,
    QolVmSwing,
    QolVmClassic,
    QolVmTotem,
    QolVmReset,
    // --- Volt Chat (#21) ---
    ChatTimestamps,
    ChatCompactSpam,
    ChatMentionPing,
    ChatQuickBar,
    // --- Volt Mods (24 Built-in Native Mods) ---
    ModArmorHud,
    ModPotionHud,
    ModCpsHud,
    ModReachHud,
    ModFpsPill,
    ModKeystrokes,
    ModCrosshair,
    ModLowShieldFire,
    ModNoHurtcam,
    ModToggleSprint,
    ModAppleSkin,
    ModShulkerPreview,
    ModToolShield,
    ModAutoTool,
    ModTrajectory,
    ModDamagePopups,
    ModHitboxPlus,
    ModDamageDir,
    ModChatCalc,
    ModEnchantGuide,
    ModSlimeTech,
    ModDeathTracker,
    ModTimeWeather,
    ModOledAfk,
}

fn on_off(v: bool) -> &'static str {
    if v { "ON" } else { "OFF" }
}

fn step_u16(cur: u16, min: u16, max: u16, delta: u16, back: bool) -> u16 {
    if back {
        if cur <= min {
            max
        } else {
            cur.saturating_sub(delta).max(min)
        }
    } else if cur >= max {
        min
    } else {
        (cur + delta).min(max)
    }
}

fn step_i16(cur: i16, min: i16, max: i16, delta: i16, back: bool) -> i16 {
    if back {
        if cur <= min {
            max
        } else {
            (cur - delta).max(min)
        }
    } else if cur >= max {
        min
    } else {
        (cur + delta).min(max)
    }
}

impl VoltAct {
    pub fn caption(self) -> &'static str {
        match self {
            Self::ShaderStyle => "Shader Style",
            Self::ShaderEyeComfort => "Eye-Comfort Rolloff",
            Self::ShaderCloudsMode => "3D Sky Clouds",
            Self::ShaderCloudCoverage => "Cloud Coverage",
            Self::ShaderCloudShadows => "Cloud Ground Shadows",
            Self::ShaderGlowingOres => "Glowing Ores (PBR)",
            Self::ShaderOreGlow => "Ore Glow Strength",
            Self::ShaderAutoPbr => "Integrated Auto-PBR",
            Self::ShaderWaterFoam => "Water Foam & Caustics",
            Self::ShaderNightSkyPlus => "Milky Way & Meteors",
            Self::ShaderRainbow => "Post-Rain Rainbow",
            Self::VisualsCtmGlass => "Connected Glass (CTM)",
            Self::VisualsBetterGrass => "Better Grass Sides",
            Self::VisualsDroppedItems3d => "3D Dropped Items",
            Self::BoostGreedyMerge => "Coplanar Quad Merge",
            Self::BoostCaveOcclusion => "Sealed Cave Occlusion",
            Self::BoostMeshPool => "Zero-Stutter Mesh Pool",
            Self::BoostOffscreenSkip => "Offscreen Entity Skip",
            Self::BoostFramePacer => "Smart Frame Pacer",
            Self::QolZoom => "Optic Zoom (Hold C)",
            Self::QolZoomFov => "Zoom Magnification",
            Self::QolZoomSmooth => "Cinematic Smooth Zoom",
            Self::QolFreelook => "360\u{00b0} Freelook Orbit",
            Self::QolInspector => "World Inspector HUD",
            Self::QolSpawnLight => "Spawn Light-0 Overlay",
            Self::QolBlueprint => "3D Blueprint Hologram",
            Self::QolBlueprintPreset => "Blueprint Template",
            Self::QolBlueprintLayer => "Blueprint Y-Slice",
            Self::QolBlueprintChecklist => "Material Checklist HUD",
            Self::QolPhotoOverlays => "Volt Cam Framing Grid",
            Self::QolCamRoll => "Camera Dutch Tilt",
            Self::QolLetterbox => "Cinema Letterbox Bars",
            Self::QolReplay => "Replay Recording",
            Self::QolReplayButton => "On-Screen Replay Button",
            Self::QolCaveReverb => "Cave Echo Reverb",
            Self::QolUnderwaterLpf => "Underwater Muffle LPF",
            Self::QolWallOcclusion => "Solid Wall Sound Muffle",
            Self::QolMufflePortal => "Portal Sound Volume",
            Self::QolMufflePiston => "Piston/Redstone Volume",
            Self::QolMuffleMinecart => "Minecart Sound Volume",
            Self::QolMuffleExplosion => "Explosion Peak Volume",
            Self::QolMuffleRain => "Rain Sound Volume",
            Self::QolAtlas => "Auto-Map World Chunks",
            Self::QolAtlasMode => "World Atlas Layer",
            Self::QolAtlasGrid => "World Atlas Chunk Grid",
            Self::QolVmCustom => "Custom Hand Viewmodel",
            Self::QolVmX => "Hand Offset X",
            Self::QolVmY => "Hand Offset Y",
            Self::QolVmZ => "Hand Depth Z",
            Self::QolVmScale => "Hand Item Scale",
            Self::QolVmRoll => "Hand Tilt Angle",
            Self::QolVmSwing => "Swing Animation Speed",
            Self::QolVmClassic => "1.7 Classic Eat & Guard",
            Self::QolVmTotem => "Totem Pop Scale",
            Self::QolVmReset => "Reset Viewmodel",
            Self::ChatTimestamps => "Chat Timestamps",
            Self::ChatCompactSpam => "Compact Duplicate Chat",
            Self::ChatMentionPing => "Name Mention Highlight",
            Self::ChatQuickBar => "Quick-Chat Macro Bar",
            Self::ModArmorHud => "Armor & Durability HUD",
            Self::ModPotionHud => "Compact Potion Timers",
            Self::ModCpsHud => "Live CPS Counter",
            Self::ModReachHud => "Attack Reach Display",
            Self::ModFpsPill => "Minimalist FPS/ms Pill",
            Self::ModKeystrokes => "Keystrokes Overlay",
            Self::ModCrosshair => "Custom Crosshair Pro",
            Self::ModLowShieldFire => "Low Shield & Low Fire",
            Self::ModNoHurtcam => "Reduced Hurtcam Shake",
            Self::ModToggleSprint => "Toggle Sprint/Sneak HUD",
            Self::ModAppleSkin => "AppleSkin Food Preview",
            Self::ModShulkerPreview => "Shulker & Bundle Peek",
            Self::ModToolShield => "Tool-Break Protection",
            Self::ModAutoTool => "Best-Tool Slot Hint",
            Self::ModTrajectory => "Projectile Trajectory Arc",
            Self::ModDamagePopups => "Floating Damage Numbers",
            Self::ModHitboxPlus => "Tactical Hitbox Vectors",
            Self::ModDamageDir => "Damage Direction Arc",
            Self::ModChatCalc => "Inline Chat Calculator",
            Self::ModEnchantGuide => "Enchantment Max Guide",
            Self::ModSlimeTech => "Slime & Despawn Tech HUD",
            Self::ModDeathTracker => "Death & Chest Coords Log",
            Self::ModTimeWeather => "Client Time & Weather",
            Self::ModOledAfk => "OLED AFK Power Saver",
        }
    }

    pub fn value(self, e: &Extras) -> String {
        match self {
            Self::ShaderStyle => e.volt_shaders.style.label().into(),
            Self::ShaderEyeComfort => on_off(e.volt_shaders.eye_comfort).into(),
            Self::ShaderCloudsMode => e.volt_shaders.clouds_mode.label().into(),
            Self::ShaderCloudCoverage => format!("{}%", e.volt_shaders.cloud_coverage),
            Self::ShaderCloudShadows => on_off(e.volt_shaders.cloud_shadows).into(),
            Self::ShaderGlowingOres => on_off(e.volt_shaders.glowing_ores).into(),
            Self::ShaderOreGlow => format!("{}%", e.volt_shaders.ore_glow),
            Self::ShaderAutoPbr => on_off(e.volt_shaders.auto_pbr).into(),
            Self::ShaderWaterFoam => on_off(e.volt_shaders.water_foam).into(),
            Self::ShaderNightSkyPlus => on_off(e.volt_shaders.night_sky_plus).into(),
            Self::ShaderRainbow => on_off(e.volt_shaders.post_rain_rainbow).into(),
            Self::VisualsCtmGlass => on_off(e.volt_visuals.connected_glass).into(),
            Self::VisualsBetterGrass => on_off(e.volt_visuals.better_grass_sides).into(),
            Self::VisualsDroppedItems3d => on_off(e.volt_visuals.dropped_items_3d).into(),
            Self::BoostGreedyMerge => on_off(e.volt_boost_plus.greedy_quad_merge).into(),
            Self::BoostCaveOcclusion => on_off(e.volt_boost_plus.cave_chunk_occlusion).into(),
            Self::BoostMeshPool => on_off(e.volt_boost_plus.zero_stutter_mesh_pool).into(),
            Self::BoostOffscreenSkip => on_off(e.volt_boost_plus.offscreen_entity_skip).into(),
            Self::BoostFramePacer => on_off(e.volt_boost_plus.smart_frame_pacer).into(),
            Self::QolZoom => on_off(e.volt_qol.zoom_enabled || e.zoom).into(),
            Self::QolZoomFov => format!("{}% FOV", e.volt_qol.zoom_fov_pct),
            Self::QolZoomSmooth => on_off(e.volt_qol.zoom_smooth).into(),
            Self::QolFreelook => on_off(e.volt_qol.freelook_enabled).into(),
            Self::QolInspector => on_off(e.volt_qol.world_inspector).into(),
            Self::QolSpawnLight => on_off(e.volt_qol.spawn_light_overlay).into(),
            Self::QolBlueprint => on_off(e.volt_qol.blueprint_enabled).into(),
            Self::QolBlueprintPreset => e.volt_qol.blueprint_preset.label().into(),
            Self::QolBlueprintLayer => {
                if e.volt_qol.blueprint_layer < 0 {
                    "All Layers".into()
                } else {
                    format!("Layer Y+{}", e.volt_qol.blueprint_layer)
                }
            }
            Self::QolBlueprintChecklist => on_off(e.volt_qol.blueprint_checklist).into(),
            Self::QolPhotoOverlays => on_off(e.volt_qol.photo_mode_overlays).into(),
            Self::QolCamRoll => format!("{}\u{00b0}", e.volt_qol.cam_roll_deg),
            Self::QolLetterbox => on_off(e.volt_qol.cam_letterbox).into(),
            Self::QolReplay => on_off(e.volt_qol.replay_buffer).into(),
            Self::QolReplayButton => on_off(e.volt_qol.replay_button).into(),
            Self::QolCaveReverb => on_off(e.volt_qol.sound_cave_reverb).into(),
            Self::QolUnderwaterLpf => on_off(e.volt_qol.sound_underwater_lpf).into(),
            Self::QolWallOcclusion => on_off(e.volt_qol.sound_wall_occlusion).into(),
            Self::QolMufflePortal => format!("{}%", e.volt_qol.muffle_portal_pct),
            Self::QolMufflePiston => format!("{}%", e.volt_qol.muffle_piston_pct),
            Self::QolMuffleMinecart => format!("{}%", e.volt_qol.muffle_minecart_pct),
            Self::QolMuffleExplosion => format!("{}%", e.volt_qol.muffle_explosion_pct),
            Self::QolMuffleRain => format!("{}%", e.volt_qol.muffle_rain_pct),
            Self::QolAtlas => on_off(e.volt_qol.atlas_enabled).into(),
            Self::QolAtlasMode => e.volt_qol.atlas_mode.label().into(),
            Self::QolAtlasGrid => on_off(e.volt_qol.atlas_grid).into(),
            Self::QolVmCustom => on_off(e.volt_qol.viewmodel_custom).into(),
            Self::QolVmX => format!("{:+}", e.volt_qol.vm_offset_x),
            Self::QolVmY => format!("{:+}", e.volt_qol.vm_offset_y),
            Self::QolVmZ => format!("{:+}", e.volt_qol.vm_offset_z),
            Self::QolVmScale => format!("{}%", e.volt_qol.vm_scale_pct),
            Self::QolVmRoll => format!("{}\u{00b0}", e.volt_qol.vm_roll_deg),
            Self::QolVmSwing => format!("{}%", e.volt_qol.vm_swing_speed_pct),
            Self::QolVmClassic => on_off(e.volt_qol.vm_classic_animations).into(),
            Self::QolVmTotem => format!("{}%", e.volt_qol.vm_totem_scale_pct),
            Self::QolVmReset => "Tap to Reset".into(),
            Self::ChatTimestamps => on_off(e.volt_chat.chat_timestamps).into(),
            Self::ChatCompactSpam => on_off(e.volt_chat.chat_compact_spam).into(),
            Self::ChatMentionPing => on_off(e.volt_chat.chat_mention_ping).into(),
            Self::ChatQuickBar => on_off(e.volt_chat.chat_quick_bar).into(),
            Self::ModArmorHud => on_off(e.volt_mods.armor_hud).into(),
            Self::ModPotionHud => on_off(e.volt_mods.potion_hud).into(),
            Self::ModCpsHud => on_off(e.volt_mods.cps_hud).into(),
            Self::ModReachHud => on_off(e.volt_mods.reach_hud).into(),
            Self::ModFpsPill => on_off(e.volt_mods.fps_ping_pill).into(),
            Self::ModKeystrokes => on_off(e.volt_mods.keystrokes_hud).into(),
            Self::ModCrosshair => e.volt_mods.custom_crosshair.label().into(),
            Self::ModLowShieldFire => on_off(e.volt_mods.low_shield_fire).into(),
            Self::ModNoHurtcam => on_off(e.volt_mods.no_hurtcam).into(),
            Self::ModToggleSprint => on_off(e.volt_mods.toggle_sprint).into(),
            Self::ModAppleSkin => on_off(e.volt_mods.appleskin_hud).into(),
            Self::ModShulkerPreview => on_off(e.volt_mods.shulker_preview).into(),
            Self::ModToolShield => on_off(e.volt_mods.tool_break_shield).into(),
            Self::ModAutoTool => on_off(e.volt_mods.auto_tool_hint).into(),
            Self::ModTrajectory => on_off(e.volt_mods.trajectory_arc).into(),
            Self::ModDamagePopups => on_off(e.volt_mods.damage_popups).into(),
            Self::ModHitboxPlus => on_off(e.volt_mods.hitbox_plus).into(),
            Self::ModDamageDir => on_off(e.volt_mods.damage_direction_hud).into(),
            Self::ModChatCalc => on_off(e.volt_mods.chat_calculator).into(),
            Self::ModEnchantGuide => on_off(e.volt_mods.enchant_guide).into(),
            Self::ModSlimeTech => on_off(e.volt_mods.slime_tech_hud).into(),
            Self::ModDeathTracker => on_off(e.volt_mods.death_tracker).into(),
            Self::ModTimeWeather => e.volt_mods.time_weather.label().into(),
            Self::ModOledAfk => on_off(e.volt_mods.oled_afk_saver).into(),
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::ShaderStyle => "Switches overall BSL Soft Cinema, Complementary Reimagined, Volt Balanced, or Pastel Dream atmosphere.",
            Self::ShaderEyeComfort => "BSL soft highlight shoulder & velvet shadow lift for comfortable zero-glare sessions.",
            Self::ShaderCloudsMode => "Selects multi-layer BSL Puffy 3D clouds, Complementary Reimagined voxel clouds, or soft Cirrus.",
            Self::ShaderCloudCoverage => "Adjusts 3D sky cloud density and sunlit cloud shadow coverage.",
            Self::ShaderCloudShadows => "Projects drifting sunlight & cool cloud shadow patches across outdoor landscapes.",
            Self::ShaderGlowingOres => "Complementary-style selective ore vein emission: diamond, emerald, gold, redstone & amethyst veins glow!",
            Self::ShaderOreGlow => "Controls HDR emission brightness of glowing ore veins inside caves.",
            Self::ShaderAutoPbr => "Integrated Auto-PBR specular glints & Fresnel reflections on metals, gems, ice, polished stone & glass.",
            Self::ShaderWaterFoam => "Adds shimmering underwater caustics and bubbly white shoreline/wave-crest foam.",
            Self::ShaderNightSkyPlus => "Renders a subtle Milky Way nebula band and occasional night shooting stars.",
            Self::ShaderRainbow => "Displays a natural 42-degree optical rainbow arc during sun-showers and clearing rain.",
            Self::VisualsCtmGlass => "Removes interior border seams between adjacent glass and stained-glass blocks on any resource pack.",
            Self::VisualsBetterGrass => "Extends lush green grass textures down the sides of grassy slopes and meadows.",
            Self::VisualsDroppedItems3d => "Gives dropped items a crisp 3D tumbling and ground-resting visual presentation.",
            Self::BoostGreedyMerge => "Eliminates hidden interior translucent/partial coplanar faces to cut GPU overdraw.",
            Self::BoostCaveOcclusion => "Skips unseen sealed underground cavities completely enclosed by solid stone.",
            Self::BoostMeshPool => "Expands multi-worker SectionMesh buffer recycling to eliminate chunk-meshing frame spikes.",
            Self::BoostOffscreenSkip => "Skips animation updates for distant block entities and off-frustum mobs.",
            Self::BoostFramePacer => "Smooths frame delivery intervals to prevent mobile GPU thermal throttling.",
            Self::QolZoom => "Enables smooth spyglass-free optic zoom (Key C or touch Zoom) with proportional look scaling.",
            Self::QolZoomFov => "Target field-of-view percentage when zooming in.",
            Self::QolZoomSmooth => "Smoothly interpolates FOV transitions instead of snapping instantly.",
            Self::QolFreelook => "Orbit camera 360 degrees around your player without changing movement heading.",
            Self::QolInspector => "Jade/WTHIT top-center HUD card showing looked-at block/entity info, crop growth & HP.",
            Self::QolSpawnLight => "Highlights nearby blocks where hostile mobs can spawn (Amber = Night, Red = Always).",
            Self::QolBlueprint => "Projects a 3D translucent build hologram in front of your anchor position.",
            Self::QolBlueprintPreset => "Selects the active 3D blueprint structure template.",
            Self::QolBlueprintLayer => "View all Y layers at once or slice one horizontal build layer at a time.",
            Self::QolBlueprintChecklist => "Shows live required block counts for the active blueprint.",
            Self::QolPhotoOverlays => "Displays rule-of-thirds framing guides for cinematic screenshots.",
            Self::QolCamRoll => "Applies a cinematic Dutch-angle camera roll tilt.",
            Self::QolLetterbox => "Adds 21:9 cinema letterbox bars to the top and bottom of the screen.",
            Self::QolReplay => "Records multiplayer and world sessions as Volt Replays. Watch them from the launcher's Replays page: timeline, speeds, pause, and a free camera that shows you in third person.",
            Self::QolReplayButton => "Shows a floating Start / Pause / End recording button in the HUD. Drag it anywhere; hold it, then drag up-down for size and left-right for opacity. Applies when you next join a world or server.",
            Self::QolCaveReverb => "Adds subtle natural acoustic echo when deep inside enclosed stone caves.",
            Self::QolUnderwaterLpf => "Applies a warm low-pass muffle filter when the camera is submerged underwater.",
            Self::QolWallOcclusion => "Softens positional sounds originating behind solid walls.",
            Self::QolMufflePortal => "Individual volume cap for Nether & End portal hums.",
            Self::QolMufflePiston => "Individual volume cap for pistons, dispensers, and droppers.",
            Self::QolMuffleMinecart => "Individual volume cap for rolling minecarts.",
            Self::QolMuffleExplosion => "Individual volume cap for loud TNT and creeper explosions.",
            Self::QolMuffleRain => "Individual volume cap for outdoor rain weather loops.",
            Self::QolAtlas => "Automatically records visited chunk terrain into the persistent full-screen World Atlas.",
            Self::QolAtlasMode => "Switches World Atlas between Surface, Elevation Relief, Lush Biome, and Slime Chunk Radar.",
            Self::QolAtlasGrid => "Overlays 16x16 chunk boundary lines on the World Atlas.",
            Self::QolVmCustom => "Enables custom first-person hand X/Y/Z position, scale, tilt, and swing speed.",
            Self::QolVmX => "Shifts first-person held items horizontally.",
            Self::QolVmY => "Shifts first-person held items vertically.",
            Self::QolVmZ => "Shifts first-person held items forward or backward.",
            Self::QolVmScale => "Scales first-person held items (compact PvP items at 65-80%).",
            Self::QolVmRoll => "Tilts first-person held items around the forward axis.",
            Self::QolVmSwing => "Adjusts arm mining/attack swing speed without affecting actual attack cooldown.",
            Self::QolVmClassic => "Enables smooth 1.7-style eating and sword/shield guard posture.",
            Self::QolVmTotem => "Scales the Totem of Undying pop animation so it never blocks your screen.",
            Self::QolVmReset => "Restores all first-person hand & viewmodel sliders to vanilla defaults.",
            Self::ChatTimestamps => "Prefixes incoming chat messages with [HH:MM] timestamps.",
            Self::ChatCompactSpam => "Stacks identical consecutive chat messages into compact (x2), (x3) counters.",
            Self::ChatMentionPing => "Highlights messages mentioning your username with a subtle chime.",
            Self::ChatQuickBar => "Adds 1-tap Quick-Chat buttons (GG, Share Coords, OMW, Help) on the chat screen.",
            _ => "Built-in pure-Rust native Volt Mod with zero overhead when disabled.",
        }
    }

    pub fn trigger(self, e: &mut Extras, back: bool) {
        match self {
            Self::ShaderStyle => {
                e.volt_shaders.style = e.volt_shaders.style.cycle(back);
                e.volt_shaders.mark_custom();
            }
            Self::ShaderEyeComfort => {
                e.volt_shaders.eye_comfort = !e.volt_shaders.eye_comfort;
                e.volt_shaders.mark_custom();
            }
            Self::ShaderCloudsMode => {
                e.volt_shaders.clouds_mode = e.volt_shaders.clouds_mode.cycle(back);
                e.volt_shaders.mark_custom();
            }
            Self::ShaderCloudCoverage => {
                e.volt_shaders.cloud_coverage =
                    step_u16(e.volt_shaders.cloud_coverage, 40, 160, 15, back);
                e.volt_shaders.mark_custom();
            }
            Self::ShaderCloudShadows => {
                e.volt_shaders.cloud_shadows = !e.volt_shaders.cloud_shadows;
                e.volt_shaders.mark_custom();
            }
            Self::ShaderGlowingOres => {
                e.volt_shaders.glowing_ores = !e.volt_shaders.glowing_ores;
                e.volt_shaders.mark_custom();
            }
            Self::ShaderOreGlow => {
                e.volt_shaders.ore_glow = step_u16(e.volt_shaders.ore_glow, 40, 200, 20, back);
                e.volt_shaders.mark_custom();
            }
            Self::ShaderAutoPbr => {
                e.volt_shaders.auto_pbr = !e.volt_shaders.auto_pbr;
                e.volt_shaders.mark_custom();
            }
            Self::ShaderWaterFoam => {
                e.volt_shaders.water_foam = !e.volt_shaders.water_foam;
                e.volt_shaders.mark_custom();
            }
            Self::ShaderNightSkyPlus => {
                e.volt_shaders.night_sky_plus = !e.volt_shaders.night_sky_plus;
                e.volt_shaders.mark_custom();
            }
            Self::ShaderRainbow => {
                e.volt_shaders.post_rain_rainbow = !e.volt_shaders.post_rain_rainbow;
                e.volt_shaders.mark_custom();
            }
            Self::VisualsCtmGlass => {
                e.volt_visuals.connected_glass = !e.volt_visuals.connected_glass
            }
            Self::VisualsBetterGrass => {
                e.volt_visuals.better_grass_sides = !e.volt_visuals.better_grass_sides
            }
            Self::VisualsDroppedItems3d => {
                e.volt_visuals.dropped_items_3d = !e.volt_visuals.dropped_items_3d
            }
            Self::BoostGreedyMerge => {
                e.volt_boost_plus.greedy_quad_merge = !e.volt_boost_plus.greedy_quad_merge
            }
            Self::BoostCaveOcclusion => {
                e.volt_boost_plus.cave_chunk_occlusion = !e.volt_boost_plus.cave_chunk_occlusion
            }
            Self::BoostMeshPool => {
                e.volt_boost_plus.zero_stutter_mesh_pool = !e.volt_boost_plus.zero_stutter_mesh_pool
            }
            Self::BoostOffscreenSkip => {
                e.volt_boost_plus.offscreen_entity_skip = !e.volt_boost_plus.offscreen_entity_skip
            }
            Self::BoostFramePacer => {
                e.volt_boost_plus.smart_frame_pacer = !e.volt_boost_plus.smart_frame_pacer
            }
            Self::QolZoom => {
                let on = !(e.volt_qol.zoom_enabled || e.zoom);
                e.volt_qol.zoom_enabled = on;
                e.zoom = on;
            }
            Self::QolZoomFov => {
                e.volt_qol.zoom_fov_pct = step_u16(e.volt_qol.zoom_fov_pct, 12, 56, 4, back)
            }
            Self::QolZoomSmooth => e.volt_qol.zoom_smooth = !e.volt_qol.zoom_smooth,
            Self::QolFreelook => e.volt_qol.freelook_enabled = !e.volt_qol.freelook_enabled,
            Self::QolInspector => e.volt_qol.world_inspector = !e.volt_qol.world_inspector,
            Self::QolSpawnLight => e.volt_qol.spawn_light_overlay = !e.volt_qol.spawn_light_overlay,
            Self::QolBlueprint => e.volt_qol.blueprint_enabled = !e.volt_qol.blueprint_enabled,
            Self::QolBlueprintPreset => {
                e.volt_qol.blueprint_preset = e.volt_qol.blueprint_preset.cycle(back)
            }
            Self::QolBlueprintLayer => {
                e.volt_qol.blueprint_layer = step_i16(e.volt_qol.blueprint_layer, -1, 8, 1, back)
            }
            Self::QolBlueprintChecklist => {
                e.volt_qol.blueprint_checklist = !e.volt_qol.blueprint_checklist
            }
            Self::QolPhotoOverlays => {
                e.volt_qol.photo_mode_overlays = !e.volt_qol.photo_mode_overlays
            }
            Self::QolCamRoll => {
                e.volt_qol.cam_roll_deg = step_i16(e.volt_qol.cam_roll_deg, -30, 30, 5, back)
            }
            Self::QolLetterbox => e.volt_qol.cam_letterbox = !e.volt_qol.cam_letterbox,
            Self::QolReplay => e.volt_qol.replay_buffer = !e.volt_qol.replay_buffer,
            Self::QolReplayButton => e.volt_qol.replay_button = !e.volt_qol.replay_button,
            Self::QolCaveReverb => e.volt_qol.sound_cave_reverb = !e.volt_qol.sound_cave_reverb,
            Self::QolUnderwaterLpf => {
                e.volt_qol.sound_underwater_lpf = !e.volt_qol.sound_underwater_lpf
            }
            Self::QolWallOcclusion => {
                e.volt_qol.sound_wall_occlusion = !e.volt_qol.sound_wall_occlusion
            }
            Self::QolMufflePortal => {
                e.volt_qol.muffle_portal_pct =
                    step_u16(e.volt_qol.muffle_portal_pct, 0, 100, 20, back)
            }
            Self::QolMufflePiston => {
                e.volt_qol.muffle_piston_pct =
                    step_u16(e.volt_qol.muffle_piston_pct, 0, 100, 20, back)
            }
            Self::QolMuffleMinecart => {
                e.volt_qol.muffle_minecart_pct =
                    step_u16(e.volt_qol.muffle_minecart_pct, 0, 100, 20, back)
            }
            Self::QolMuffleExplosion => {
                e.volt_qol.muffle_explosion_pct =
                    step_u16(e.volt_qol.muffle_explosion_pct, 0, 100, 20, back)
            }
            Self::QolMuffleRain => {
                e.volt_qol.muffle_rain_pct = step_u16(e.volt_qol.muffle_rain_pct, 0, 100, 20, back)
            }
            Self::QolAtlas => e.volt_qol.atlas_enabled = !e.volt_qol.atlas_enabled,
            Self::QolAtlasMode => e.volt_qol.atlas_mode = e.volt_qol.atlas_mode.cycle(back),
            Self::QolAtlasGrid => e.volt_qol.atlas_grid = !e.volt_qol.atlas_grid,
            Self::QolVmCustom => e.volt_qol.viewmodel_custom = !e.volt_qol.viewmodel_custom,
            Self::QolVmX => e.volt_qol.vm_offset_x = step_i16(e.volt_qol.vm_offset_x, -60, 60, 10, back),
            Self::QolVmY => e.volt_qol.vm_offset_y = step_i16(e.volt_qol.vm_offset_y, -60, 60, 10, back),
            Self::QolVmZ => e.volt_qol.vm_offset_z = step_i16(e.volt_qol.vm_offset_z, -60, 60, 10, back),
            Self::QolVmScale => {
                e.volt_qol.vm_scale_pct = step_u16(e.volt_qol.vm_scale_pct, 50, 150, 10, back)
            }
            Self::QolVmRoll => {
                e.volt_qol.vm_roll_deg = step_i16(e.volt_qol.vm_roll_deg, -30, 30, 5, back)
            }
            Self::QolVmSwing => {
                e.volt_qol.vm_swing_speed_pct =
                    step_u16(e.volt_qol.vm_swing_speed_pct, 50, 180, 15, back)
            }
            Self::QolVmClassic => {
                e.volt_qol.vm_classic_animations = !e.volt_qol.vm_classic_animations
            }
            Self::QolVmTotem => {
                e.volt_qol.vm_totem_scale_pct =
                    step_u16(e.volt_qol.vm_totem_scale_pct, 30, 110, 10, back)
            }
            Self::QolVmReset => {
                e.volt_qol.vm_offset_x = 0;
                e.volt_qol.vm_offset_y = 0;
                e.volt_qol.vm_offset_z = 0;
                e.volt_qol.vm_scale_pct = 100;
                e.volt_qol.vm_roll_deg = 0;
                e.volt_qol.vm_swing_speed_pct = 100;
                e.volt_qol.vm_totem_scale_pct = 100;
            }
            Self::ChatTimestamps => e.volt_chat.chat_timestamps = !e.volt_chat.chat_timestamps,
            Self::ChatCompactSpam => e.volt_chat.chat_compact_spam = !e.volt_chat.chat_compact_spam,
            Self::ChatMentionPing => e.volt_chat.chat_mention_ping = !e.volt_chat.chat_mention_ping,
            Self::ChatQuickBar => e.volt_chat.chat_quick_bar = !e.volt_chat.chat_quick_bar,
            Self::ModArmorHud => e.volt_mods.armor_hud = !e.volt_mods.armor_hud,
            Self::ModPotionHud => e.volt_mods.potion_hud = !e.volt_mods.potion_hud,
            Self::ModCpsHud => e.volt_mods.cps_hud = !e.volt_mods.cps_hud,
            Self::ModReachHud => e.volt_mods.reach_hud = !e.volt_mods.reach_hud,
            Self::ModFpsPill => e.volt_mods.fps_ping_pill = !e.volt_mods.fps_ping_pill,
            Self::ModKeystrokes => e.volt_mods.keystrokes_hud = !e.volt_mods.keystrokes_hud,
            Self::ModCrosshair => {
                e.volt_mods.custom_crosshair = e.volt_mods.custom_crosshair.cycle(back)
            }
            Self::ModLowShieldFire => e.volt_mods.low_shield_fire = !e.volt_mods.low_shield_fire,
            Self::ModNoHurtcam => e.volt_mods.no_hurtcam = !e.volt_mods.no_hurtcam,
            Self::ModToggleSprint => e.volt_mods.toggle_sprint = !e.volt_mods.toggle_sprint,
            Self::ModAppleSkin => e.volt_mods.appleskin_hud = !e.volt_mods.appleskin_hud,
            Self::ModShulkerPreview => e.volt_mods.shulker_preview = !e.volt_mods.shulker_preview,
            Self::ModToolShield => e.volt_mods.tool_break_shield = !e.volt_mods.tool_break_shield,
            Self::ModAutoTool => e.volt_mods.auto_tool_hint = !e.volt_mods.auto_tool_hint,
            Self::ModTrajectory => e.volt_mods.trajectory_arc = !e.volt_mods.trajectory_arc,
            Self::ModDamagePopups => e.volt_mods.damage_popups = !e.volt_mods.damage_popups,
            Self::ModHitboxPlus => e.volt_mods.hitbox_plus = !e.volt_mods.hitbox_plus,
            Self::ModDamageDir => {
                e.volt_mods.damage_direction_hud = !e.volt_mods.damage_direction_hud
            }
            Self::ModChatCalc => e.volt_mods.chat_calculator = !e.volt_mods.chat_calculator,
            Self::ModEnchantGuide => e.volt_mods.enchant_guide = !e.volt_mods.enchant_guide,
            Self::ModSlimeTech => e.volt_mods.slime_tech_hud = !e.volt_mods.slime_tech_hud,
            Self::ModDeathTracker => e.volt_mods.death_tracker = !e.volt_mods.death_tracker,
            Self::ModTimeWeather => e.volt_mods.time_weather = e.volt_mods.time_weather.cycle(back),
            Self::ModOledAfk => e.volt_mods.oled_afk_saver = !e.volt_mods.oled_afk_saver,
        }
    }
}

pub const QOL_SECTIONS: &[(&str, &[VoltAct])] = &[
    (
        "Zoom & 360\u{00b0} Freelook (#2)",
        &[
            VoltAct::QolZoom,
            VoltAct::QolZoomFov,
            VoltAct::QolZoomSmooth,
            VoltAct::QolFreelook,
        ],
    ),
    (
        "World Inspector & Spawn-Light Overlay (#3)",
        &[VoltAct::QolInspector, VoltAct::QolSpawnLight],
    ),
    (
        "World Atlas & Chunk Mapper (#17)",
        &[
            VoltAct::QolAtlas,
            VoltAct::QolAtlasMode,
            VoltAct::QolAtlasGrid,
        ],
    ),
    (
        "Volt Blueprint 3D Hologram (#8)",
        &[
            VoltAct::QolBlueprint,
            VoltAct::QolBlueprintPreset,
            VoltAct::QolBlueprintLayer,
            VoltAct::QolBlueprintChecklist,
        ],
    ),
    (
        "First-Person Hand & Viewmodel (#25)",
        &[
            VoltAct::QolVmCustom,
            VoltAct::QolVmScale,
            VoltAct::QolVmX,
            VoltAct::QolVmY,
            VoltAct::QolVmZ,
            VoltAct::QolVmRoll,
            VoltAct::QolVmSwing,
            VoltAct::QolVmClassic,
            VoltAct::QolVmTotem,
            VoltAct::QolVmReset,
        ],
    ),
    (
        "Sound Physics Lite & Per-Sound Muffler (#15)",
        &[
            VoltAct::QolCaveReverb,
            VoltAct::QolUnderwaterLpf,
            VoltAct::QolWallOcclusion,
            VoltAct::QolMufflePortal,
            VoltAct::QolMufflePiston,
            VoltAct::QolMuffleMinecart,
            VoltAct::QolMuffleExplosion,
            VoltAct::QolMuffleRain,
        ],
    ),
    (
        "Volt Cam Photo Mode & Replay (#9)",
        &[
            VoltAct::QolPhotoOverlays,
            VoltAct::QolCamRoll,
            VoltAct::QolLetterbox,
            VoltAct::QolReplay,
            VoltAct::QolReplayButton,
        ],
    ),
];

pub const CHAT_SECTIONS: &[(&str, &[VoltAct])] = &[
    (
        "Chat Enhancements & Quick Macros",
        &[
            VoltAct::ChatTimestamps,
            VoltAct::ChatCompactSpam,
            VoltAct::ChatMentionPing,
            VoltAct::ChatQuickBar,
        ],
    ),
];

pub const MODS_SECTIONS: &[(&str, &[VoltAct])] = &[
    (
        "Combat & PvP HUD Mods",
        &[
            VoltAct::ModArmorHud,
            VoltAct::ModPotionHud,
            VoltAct::ModCpsHud,
            VoltAct::ModReachHud,
            VoltAct::ModFpsPill,
            VoltAct::ModKeystrokes,
            VoltAct::ModCrosshair,
            VoltAct::ModLowShieldFire,
            VoltAct::ModNoHurtcam,
            VoltAct::ModToggleSprint,
        ],
    ),
    (
        "Survival & Inventory Mods",
        &[
            VoltAct::ModAppleSkin,
            VoltAct::ModShulkerPreview,
            VoltAct::ModToolShield,
            VoltAct::ModAutoTool,
        ],
    ),
    (
        "World & Combat Visual Mods",
        &[
            VoltAct::ModTrajectory,
            VoltAct::ModDamagePopups,
            VoltAct::ModHitboxPlus,
            VoltAct::ModDamageDir,
        ],
    ),
    (
        "Technical, Utility & Atmosphere Mods",
        &[
            VoltAct::ModChatCalc,
            VoltAct::ModEnchantGuide,
            VoltAct::ModSlimeTech,
            VoltAct::ModDeathTracker,
            VoltAct::ModTimeWeather,
            VoltAct::ModOledAfk,
        ],
    ),
];

pub const BOOST_PLUS_ITEMS: &[VoltAct] = &[
    VoltAct::BoostGreedyMerge,
    VoltAct::BoostCaveOcclusion,
    VoltAct::BoostMeshPool,
    VoltAct::BoostOffscreenSkip,
    VoltAct::BoostFramePacer,
];
