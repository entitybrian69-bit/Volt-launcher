use super::*;
use horizon_client::osk::Output;
use winit::{
    event::{KeyEvent, Touch},
    keyboard::{KeyCode, PhysicalKey},
};

/// Launcher colors: the upstream glass calculations with Volt's deep-blue
/// night palette and a bright blue accent, as in the reference capture.
/// The scene is approximated in this mobile renderer; no backdrop blur is claimed.
pub(crate) mod theme {
    #![allow(non_snake_case)]
    use horizon_client::ui_theme::color::{hex, lerp};
    use std::cell::Cell;
    use horizon_theme::glass::{Glass, ACCENT, BLUR, TINT};
    #[derive(Clone, Copy)]
    struct Look { dark: bool, liquid: bool, glass: Glass }
    thread_local! { static LOOK: Cell<Option<Look>> = const { Cell::new(None) }; }
    thread_local! { static EMBER: Cell<f32> = const { Cell::new(0.5) }; }
    pub fn set(dark: bool, liquid: bool, accent: u32) {
        let accent = accent & 0xffffff;
        LOOK.with(|cell| {
            if cell.get().is_none_or(|look| look.dark != dark || look.liquid != liquid
                || look.glass.accent != accent)
            {
                cell.set(Some(Look { dark, liquid, glass: Glass::new(accent, TINT, BLUR) }));
            }
        });
    }
    fn look() -> Look { LOOK.with(|cell| cell.get().unwrap_or_else(|| Look {
        dark: true, liquid: true, glass: Glass::new(ACCENT, TINT, BLUR)
    })) }
    /// Liquid Black always reads as a dark look, whatever the appearance toggle says.
    pub fn dark() -> bool { let l = look(); l.dark || l.liquid }
    pub fn liquid() -> bool { look().liquid }
    /// Live silver shimmer (0..1) published by the moonlight scene each
    /// frame; the glass layers read it so panels subtly breathe with it.
    pub fn set_ember(value: f32) { EMBER.with(|cell| cell.set(value.clamp(0.0, 1.0))); }
    pub fn ember() -> f32 { EMBER.with(|cell| cell.get()) }
    /// Liquid Glass is strictly monochrome: every accent collapses to
    /// polished silver, whatever accent color Classic is set to.
    fn accent() -> u32 { if liquid() { return hex(0xdfe4ec, 255); } hex(look().glass.accent, 255) }
    fn choose(dark_value: u32, light_value: u32) -> u32 { if dark() { dark_value } else { light_value } }
    pub fn BG() -> u32 { if liquid() { return hex(0x000000, 255); } choose(hex(0x0c0e13,255), hex(0xedf1f0,255)) }
    pub fn BACKDROP() -> u32 { if liquid() { return hex(0x000000, 240); } hex(0x070a1a, 215) }
    pub fn RAIL() -> u32 { if liquid() { return hex(0x0a0a0c, 155); } choose(hex(0x15181f,205), hex(0xfcfdfd,225)) }
    pub fn CARD() -> u32 { if liquid() { return hex(0x0c0c0e, 148); } choose(hex(0x1a1d25,215), hex(0xfcfdfd,230)) }
    pub fn CARD_INNER() -> u32 { if liquid() { return hex(0x121214, 165); } choose(hex(0x272c36,225), hex(0xe7ecea,245)) }
    pub fn HOVER() -> u32 { if liquid() { return hex(0x1e1e22, 190); } choose(hex(0x343a46,235), hex(0xd8e0dd,245)) }
    pub fn HERO_BG() -> u32 { CARD() }
    pub fn HERO_GLOW() -> u32 { lerp(CARD(), accent(), 0.22) }
    pub fn HERO_GLOW_SOFT() -> u32 { lerp(CARD(), accent(), 0.12) }
    pub fn BORDER() -> u32 { if liquid() { return hex(0xffffff, 18); } choose(hex(0xffffff,30), hex(0x3c4a44,34)) }
    pub fn BORDER_CRIMSON() -> u32 { lerp(BORDER(), accent(), 0.32) }
    pub fn CRIMSON() -> u32 { accent() }
    pub fn CRIMSON_BRIGHT() -> u32 {
        if liquid() { return hex(0xf1f4f9, 255); }
        let glass = look().glass;
        hex(if dark() { glass.accent_soft } else { glass.accent }, 255)
    }
    pub fn CRIMSON_SOFT() -> u32 { (accent() & 0x00ffffff) | (42u32 << 24) }
    pub fn CRIMSON_BADGE() -> u32 { choose(hex(0xffffff,22), hex(0x101a14,22)) }
    pub fn ON_CRIMSON() -> u32 {
        // Ink on a silver fill must be near-black in Liquid Glass.
        if liquid() { return hex(0x10141b, 255); }
        let a = look().glass.accent;
        let lum = 0.2126 * (a >> 16 & 255) as f32 + 0.7152 * (a >> 8 & 255) as f32
            + 0.0722 * (a & 255) as f32;
        if lum > 150.0 { hex(0x0c1a12, 255) } else { hex(look().glass.on_accent, 255) }
    }
    pub fn TEXT() -> u32 { choose(hex(0xf2f5f3,255), hex(0x151c18,255)) }
    pub fn MUTED() -> u32 { choose(hex(0xc6ccc9,255), hex(0x4b5551,255)) }
    pub fn DIM() -> u32 { choose(hex(0x8f969b,255), hex(0x68726d,255)) }
    pub fn ONLINE() -> u32 { if liquid() { return hex(0xe6ebf3, 255); } choose(hex(0x4ade80,255), hex(0x15803d,255)) }
    pub fn STATUS_BAD() -> u32 { choose(hex(0xf87171,255), hex(0xb91c1c,255)) }
    pub fn GLASS() -> u32 {
        if liquid() {
            // Pure black glass: OLED true black, very low opacity, no silver
            let e = ember();
            return lerp(hex(0x070709, 92), hex(0x101012, 105), 0.5 * e);
        }
        if !dark() { return hex(0xffffff, 214); }
        // Neutral smoked glass per the chosen reference, faintly tinted by
        // the upstream accent-derived plate so accents still colour panels.
        lerp(hex(0x16191f, 202), {
            let glass = look().glass;
            hex(glass.plate, 202)
        }, 0.18)
    }
    pub fn GLASS_INNER() -> u32 { if liquid() { return hex(0xffffff, (7.0 + 6.0 * ember()) as u8); } choose(hex(0xffffff,14), hex(0x101a14,12)) }
    pub fn GLASS_BORDER() -> u32 { if liquid() { let e = ember(); return lerp(hex(0xffffff, (20.0 + 14.0 * e) as u8), hex(0x9aa0ae, 38), 0.45 * e); } choose(hex(0xffffff,36), hex(0x3c4a44,30)) }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum HomeTab { Overview, Worlds, Multiplayer, Screenshots, Replays, Console }

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Action {
    HomeTab(HomeTab),
    ToggleAppearance,
    ToggleLiquid,
    CycleAccent,
    ConnectServer(usize),
    SaveServer,
    WatchReplay(usize),
    ExportReplay(usize),
    DeleteReplay(usize),
    DeleteRecording(usize),
    /// Browse the recordings folder in the system Files app.
    OpenCaptures,
    ToggleVoltReplay,
    ToggleReplayButton,
    PatchNotes,
    Page(Page),
    Play,
    Search,
    ShaderSource(crate::shader_catalog::Origin),
    ModSource(crate::shader_catalog::Origin),
    PackSource(crate::shader_catalog::Origin),
    Field(&'static str),
    FinishNameEdit,
    Install(usize),
    OpenProject(usize),
    ViewDetail(usize),
    CloseDetail,
    DetailTab(u8),
    ViewScreenshot(usize),
    ViewWorld(usize),
    CloseViewer,
    DeleteWorld(String),
    Import(horizon_client::content_files::Kind),
    OpenFolder(horizon_client::content_files::Kind),
    ToggleSection(usize),
    ToggleLocal(usize),
    DeleteLocal(usize),
    ConfirmDeleteLocal,
    Profile,
    Logs,
    RecordPerformance,
    ExportPerformance,
    ExportDiagnostics,
    Account,
    Discord,
    AboutWebsite,
    AboutOwner,
    PresenceStatus,
    CheckUpdates,
    AccountProviderTab(crate::volt_accounts::AccountProvider),
    SelectVoltAccount(usize),
    RemoveVoltAccount(usize),
    CreateOfflineAccount,
    UpdateOfflineName,
    ElySignIn,
    ElyRegister,
    ToggleSlimModel,
    ToggleServerSkinSync,
    ImportSkinPng,
    ImportCapePng,
    FetchSkinByUsername,
    LoadSkinUrl,
    LoadCapeUrl,
    EquipPresetCape(crate::volt_accounts::PresetCape),
    ClearCustomSkin,
    ClearCustomCape,
    ResetPreviewAngle,
    SignIn,
    DeviceSignIn,
    CopyLoginCode,
    CancelLogin,
    ReviewApp,
    Repository,
    SignOut,
    LoginLink,
    CloseLogin,
    Renderer,
    RenderScale,
    ChooseRenderScale(crate::display::RenderScale),
    Orientation,
    InputHost,
    ChooseInputHost(crate::catalog::InputHost),
    ChooseOrientation(crate::catalog::Orientation),
    ChooseRenderer(crate::catalog::RendererMode),
    ChooseProfile(u32),
    Dismiss,
    Immersive,
    Cutout,
    Aspect,
    ChooseAspect(crate::display::Aspect),
    DisplayHz,
    ChooseDisplayHz(u16),
    PerformanceHints,
    GpuProfile,
    ImportControl,
    ControlLayouts,
    ChooseControlLayout(usize),
    DeleteControlPreset(usize),
    ApplyControlsAllProfiles,
    ResetControls,
    ImportVulkanDriver,
    ResetVulkanDriver,
    InstallUpdate,
    CancelUpdate,
    OpenVoltBoost,
    CloseVoltBoost,
    VoltExtra(horizon_client::extras::Item),
    VoltPreset(horizon_client::volt_features::VoltBoostPreset),
    /// Install or remove `bundled_native::BUNDLED[i]` for the active profile.
    ToggleBundled(usize),
    /// Turn `bundled_native::WOVEN[i]` on or off for the active profile
    /// (`mod.<id>:false` in `horizon-mods.txt`; the code ships in the APK).
    ToggleWoven(usize),
    /// Turn every toggleable mod on (Mods tab).
    ModsAllOn,
    /// Turn every toggleable mod off (Mods tab).
    ModsAllOff,
    /// Cycle the Mods tab sort: default order, on first, off first.
    ModsSort,
    Studio(super::controls_studio::StudioAction),
}

#[derive(Clone, Copy)]
enum BtnStyle {
    Primary,
    Secondary,
    CrimsonOutline,
    Subtle,
}

fn text_vcenter(y: f32, h: f32, size: f32) -> f32 {
    y + (h - (size * 1.45).round()) * 0.5
}

fn draw_centered_text(
    ui: &mut Ui,
    face: Face,
    size: f32,
    rect: [f32; 4],
    col: u32,
    label: &str,
) {
    let [x, y, w, h] = rect;
    let fitted = ui.fit_text(face, size, label, (w - 12.0).max(8.0));
    let tw = ui.measure(face, size, &fitted);
    ui.text(
        face,
        size,
        x + ((w - tw) * 0.5).max(4.0),
        text_vcenter(y, h, size),
        col,
        &fitted,
    );
}

thread_local! {
    static ACTIVE_FEEDBACK: std::cell::RefCell<Option<(Action, std::time::Instant)>> =
        const { std::cell::RefCell::new(None) };
    static TOGGLE_ANIMS: std::cell::RefCell<std::collections::HashMap<u32, (f32, std::time::Instant)>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}

fn toggle_key(action: Action) -> u32 {
    match action {
        Action::Immersive => 1,
        Action::Cutout => 2,
        Action::PerformanceHints => 3,
        Action::GpuProfile => 4,
        Action::RecordPerformance => 5,
        Action::ToggleServerSkinSync => 6,
        Action::ToggleSlimModel => 7,
        Action::ToggleLocal(i) => 1000 + i as u32,
        Action::ToggleSection(i) => 2000 + i as u32,
        Action::VoltExtra(item) => 3000 + item as u32,
        _ => 9999,
    }
}

fn bundled_mod_title(id: &str) -> &'static str {
    match id {
        "zoom" => "Horizon Zoom",
        "fps_overlay" => "FPS Overlay",
        "keystrokes" => "Keystrokes",
        "minimap" => "Minimap",
        "jei" => "JEI for Horizon",
        "horizon_physics" => "Horizon Physics",
        "jade" => "Jade",
        "veinminer" => "Veinminer",
        "appleskin" => "AppleSkin",
        "status_effect_hud" => "Status Effect HUD",
        "toggle_sprint" => "Toggle Sprint",
        "inventory_profiles" => "Inventory Profiles",
        "shulker_peek" => "Shulker Peek",
        "mousetricks" => "Mousetricks",
        "itemnew" => "ItemNew",
        "eatinganimation" => "Eating Animation",
        "enchantmenthints" => "Enchantment Hints",
        "right_click_crops" => "Right Click Crops",
        "smartf3" => "SmartF3",
        "armor_status" => "Armor Status",
        "crosshair_indicator" => "Crosshair Indicator",
        "hitbox_plus" => "Hitbox+",
        "invisible_frames" => "Invisible Frames",
        "freelook" => "Freelook",
        "worldedit" => "WorldEdit",
        "horizonmatica" => "Horizonmatica",
        "visuality_horizon" => "Visuality",
        "held_item_info" => "Held Item Info",
        "carryblock" => "Carry Block",
        "chatheads" => "ChatHeads",
        "fast_items" => "Fast Items",
        "customhud" => "Beautiful Day Counter",
        "coords_hud" => "Coordinates HUD",
        "biomecompass" => "BiomeCompass",
        "skins_3d" => "3D Skins",
        "accurate_block_placement" => "Accurate Block Placement",
        "armorhud" => "Armor HUD",
        "lookat" => "LookAt",
        "blockscope" => "Blockscope",
        "elytratrails" => "Elytra Trails",
        _ => "Horizon Mod",
    }
}

fn bundled_mod_blurb(id: &str) -> &'static str {
    match id {
        "zoom" => "Hold Z to zoom. Add a Z button in Controls Studio for touch.",
        "fps_overlay" => "A small frames-per-second readout over the game.",
        "keystrokes" => "Shows movement keys and clicks as you press them.",
        "minimap" => "A corner map of the terrain around you.",
        "jei" => "Browse every item and its recipes while you play.",
        "horizon_physics" => "Blocks crumble, mobs ragdoll, water waves. Physics Settings appears in Options.",
        "jade" => "The look-at tooltip: what you're pointing at, its details and state. Settings under Options > Mods.",
        "veinminer" => "Mine a whole ore vein or tree at once. Use /veinminer toggle in game.",
        "appleskin" => "Food tooltips and the hunger bar show what each bite restores.",
        "status_effect_hud" => "Active potion and beacon effects as bars with timers.",
        "toggle_sprint" => "Press once to sprint or sneak until you press again.",
        "inventory_profiles" => "Press R to sort inventories; depleted stacks and tools auto-refill.",
        "shulker_peek" => "A shulker box's contents as a grid in its tooltip.",
        "mousetricks" => "Drag to spread or gather items; the wheel moves them one by one.",
        "itemnew" => "Newly picked-up items glow until you look at them.",
        "eatinganimation" => "Food and drinks show their bites while you consume them.",
        "enchantmenthints" => "A short description under every enchantment in tooltips.",
        "right_click_crops" => "Right-click ripe crops to harvest and replant them.",
        "smartf3" => "A tidier F3 debug screen in modules you can rearrange.",
        "armor_status" => "Equipped armor and held items with their durability.",
        "crosshair_indicator" => "The crosshair reacts when a target is within reach.",
        "hitbox_plus" => "Entity hitboxes with reach-aware colors and thickness.",
        "invisible_frames" => "Sneak-right-click a filled frame to hide and lock it.",
        "freelook" => "Hold a key for a third-person camera you can look around in.",
        "worldedit" => "In-game WorldEdit: wand selections, //set, //copy and the other commands.",
        "horizonmatica" => "Litematica for Horizon: schematics as ghost blocks, a verifier and material lists.",
        "visuality_horizon" => "Small pixel particles: hit fragments, slime blobs, soul flames and more.",
        "held_item_info" => "Durability, count and enchantments above the hotbar for the held item.",
        "carryblock" => "Sneak and right-click with an empty hand to carry a chest or furnace.",
        "chatheads" => "A player's head beside every chat line they wrote.",
        "fast_items" => "Dropped items render as light 2D sprites for faster frames.",
        "customhud" => "A day counter, clock and day/night cycle bar on the HUD.",
        "coords_hud" => "Your coordinates and facing in a corner of the screen; /coords hides it.",
        "biomecompass" => "Craft a Biome Compass, pick a biome and its needle points to the nearest one.",
        "skins_3d" => "Every player's hat, jacket, sleeves and pants as real 3D pixels.",
        "accurate_block_placement" => {
            "Hold right click and every new spot you look at gets a block, however fast you move."
        }
        "armorhud" => "Worn armor and its durability beside the hotbar; K toggles the numbers.",
        "lookat" => "The block or mob under your crosshair as a vanilla tooltip, with health and harvest tool.",
        "blockscope" => "The aimed block's name, id, distance, hardness, states and a container's slots.",
        "elytratrails" => "Colorful particle trails behind elytra flight, yours and other players'.",
        _ => "An official Horizon registry mod.",
    }
}

fn has_active_toggle_anims() -> bool {
    TOGGLE_ANIMS.with(|map| {
        map.borrow()
            .values()
            .any(|&(pos, ts)| (pos > 0.005 && pos < 0.995) && ts.elapsed().as_secs_f32() < 0.4)
    })
}

#[inline]
fn point_in(p: (f32, f32), r: [f32; 4]) -> bool {
    p.0 >= r[0] && p.0 <= r[0] + r[2] && p.1 >= r[1] && p.1 <= r[1] + r[3]
}

fn styled_button(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    label: &str,
    action: Action,
    style: BtnStyle,
    centered: bool,
) {
    let [x, y, w, h] = rect;
    let press_t = ACTIVE_FEEDBACK.with(|fb| {
        fb.borrow().and_then(|(act, ts)| {
            let el = ts.elapsed().as_secs_f32();
            (act == action && el < 0.22).then_some(el / 0.22)
        })
    });
    let v_start = ui.batch_mut().vertices.len();
    let radius = (h * 0.24).clamp(6.0, 11.0);
    let (mut bg, mut border_col, fg) = match style {
        BtnStyle::Primary => (theme::CRIMSON(), theme::CRIMSON_BRIGHT(), theme::ON_CRIMSON()),
        BtnStyle::CrimsonOutline => (theme::CARD_INNER(), theme::BORDER_CRIMSON(), theme::TEXT()),
        BtnStyle::Secondary => (theme::CARD_INNER(), theme::BORDER(), theme::TEXT()),
        BtnStyle::Subtle => (theme::CARD(), theme::BORDER(), theme::MUTED()),
    };
    if let Some(t) = press_t {
        let flash = (1.0 - t).powi(2);
        bg = horizon_client::ui_theme::color::lerp(bg, theme::CRIMSON_BRIGHT(), flash * 0.30);
        border_col =
            horizon_client::ui_theme::color::lerp(border_col, theme::CRIMSON_BRIGHT(), flash * 0.85);
    }
    ui.rounded(x, y, w, h, radius, bg);
    ui.border(x, y, w, h, radius, border_col);
    let base = if h >= 50.0 {
        19.5
    } else if w < 84.0 {
        13.5
    } else {
        15.5
    };
    let pad = (w * 0.1).clamp(8.0, 14.0);
    let font = if matches!(action, Action::Page(_)) {
        (base * ((w - 2.0 * pad) / ui.measure(Face::LauncherMedium, base, label).max(1.0)).min(1.0))
            .max(11.0)
    } else {
        base
    };
    if centered {
        draw_centered_text(ui, Face::LauncherMedium, font, rect, fg, label);
    } else {
        let fitted = ui.fit_text(Face::LauncherMedium, font, label, (w - 2.0 * pad).max(8.0));
        ui.text(Face::LauncherMedium, font, x + pad, text_vcenter(y, h, font), fg, &fitted);
    }
    if let Some(t) = press_t {
        let dip = (t * std::f32::consts::PI).sin() * (1.0 - t * 0.45);
        let scale = 1.0 - 0.042 * dip;
        ui.animate_vertices(v_start, 0.0, 0.0, scale, [x + w * 0.5, y + h * 0.5], 1.0);
    }
    targets.push((rect, action));
}

pub(super) fn button(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    label: &str,
    action: Action,
    accent: bool,
) {
    styled_button(
        ui,
        targets,
        rect,
        label,
        action,
        if accent {
            BtnStyle::Primary
        } else {
            BtnStyle::Secondary
        },
        accent,
    );
}

fn toggle(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    on: bool,
    action: Action,
) {
    let [x, y, w, h] = rect;
    let target_pos = if on { 1.0f32 } else { 0.0f32 };
    let anim_pos = TOGGLE_ANIMS.with(|map| {
        let mut map = map.borrow_mut();
        let now = std::time::Instant::now();
        let key = toggle_key(action);
        let entry = map.entry(key).or_insert((target_pos, now));
        let dt = now.duration_since(entry.1).as_secs_f32().clamp(0.0, 0.1);
        let alpha = 1.0 - (-dt * 18.0).exp();
        entry.0 += (target_pos - entry.0) * alpha;
        if (target_pos - entry.0).abs() < 0.004 {
            entry.0 = target_pos;
        }
        entry.1 = now;
        entry.0
    });
    if w >= 72.0 {
        ui.text(
            Face::LauncherMedium,
            14.5,
            x,
            text_vcenter(y, h, 14.5),
            horizon_client::ui_theme::color::lerp(theme::MUTED(), theme::TEXT(), anim_pos),
            if on { "On" } else { "Off" },
        );
    }
    let sw = 48.0;
    let sh = 26.0;
    let sx = x + w - sw;
    let sy = y + (h - sh) * 0.5;
    let track_col = horizon_client::ui_theme::color::lerp(theme::HOVER(), theme::CRIMSON(), anim_pos);
    let border_col =
        horizon_client::ui_theme::color::lerp(theme::BORDER(), theme::CRIMSON_BRIGHT(), anim_pos);
    ui.rounded(sx, sy, sw, sh, 13.0, track_col);
    ui.border(sx, sy, sw, sh, 13.0, border_col);
    let knob = 20.0;
    let kx = sx + 3.0 + (sw - knob - 6.0) * anim_pos;
    ui.rounded(kx, sy + 3.0, knob, knob, 10.0, theme::ON_CRIMSON());
    targets.push((rect, action));
}

fn draw_page_icon(ui: &mut Ui, page: Page, cx: f32, cy: f32, col: u32) {
    match page {
        Page::Play => {
            ui.border(cx - 8.5, cy - 8.5, 17.0, 17.0, 8.5, col);
            ui.rounded(cx - 3.0, cy - 4.5, 2.5, 9.0, 1.0, col);
            ui.rounded(cx - 1.0, cy - 3.0, 2.5, 6.0, 1.0, col);
            ui.rounded(cx + 1.0, cy - 1.5, 2.5, 3.0, 1.0, col);
        }
        Page::Account => {
            ui.border(cx - 4.0, cy - 7.5, 8.0, 8.0, 4.0, col);
            ui.border(cx - 7.5, cy + 1.5, 15.0, 6.5, 3.2, col);
        }
        Page::Mods => {
            ui.border(cx - 7.0, cy - 7.0, 6.5, 6.5, 1.5, col);
            ui.rounded(cx + 0.5, cy - 7.0, 6.5, 6.5, 1.5, col);
            ui.border(cx - 7.0, cy + 0.5, 6.5, 6.5, 1.5, col);
            ui.border(cx + 0.5, cy + 0.5, 6.5, 6.5, 1.5, col);
        }
        Page::Packs => {
            ui.rounded(cx - 7.5, cy - 6.5, 15.0, 3.0, 1.5, col);
            ui.rounded(cx - 7.5, cy - 1.5, 15.0, 3.0, 1.5, col);
            ui.rounded(cx - 7.5, cy + 3.5, 15.0, 3.0, 1.5, col);
        }
        Page::Shaders => {
            ui.border(cx - 5.5, cy - 5.5, 11.0, 11.0, 5.5, col);
            ui.rounded(cx - 2.0, cy - 2.0, 4.0, 4.0, 2.0, col);
            ui.rounded(cx - 1.0, cy - 8.5, 2.0, 2.0, 1.0, col);
            ui.rounded(cx - 1.0, cy + 6.5, 2.0, 2.0, 1.0, col);
            ui.rounded(cx - 8.5, cy - 1.0, 2.0, 2.0, 1.0, col);
            ui.rounded(cx + 6.5, cy - 1.0, 2.0, 2.0, 1.0, col);
        }
        Page::Controls => {
            // Gamepad: rounded body, D-pad cross left, two action dots right
            ui.border(cx - 8.5, cy - 5.5, 17.0, 11.0, 5.0, col);
            ui.rounded(cx - 6.5, cy - 1.0, 5.5, 2.0, 1.0, col);
            ui.rounded(cx - 4.75, cy - 2.75, 2.0, 5.5, 1.0, col);
            ui.rounded(cx + 2.0, cy - 2.75, 2.2, 2.2, 1.1, col);
            ui.rounded(cx + 4.5, cy + 0.55, 2.2, 2.2, 1.1, col);
        }
        Page::Installed => {
            ui.rounded(cx - 1.0, cy - 7.5, 2.0, 9.0, 1.0, col);
            ui.rounded(cx - 4.0, cy - 1.0, 8.0, 2.0, 1.0, col);
            ui.border(cx - 7.5, cy + 2.5, 15.0, 5.0, 2.0, col);
        }
        Page::VoltSettings => {
            // Crisp Volt lightning-bolt / boost gauge icon
            ui.border(cx - 8.0, cy - 8.0, 16.0, 16.0, 5.0, col);
            ui.rounded(cx - 1.5, cy - 6.0, 4.5, 5.0, 1.0, col);
            ui.rounded(cx - 4.0, cy - 1.5, 8.0, 2.5, 1.0, col);
            ui.rounded(cx - 3.0, cy + 1.0, 4.5, 5.0, 1.0, col);
        }
        Page::Settings => {
            ui.rounded(cx - 7.5, cy - 5.5, 15.0, 2.0, 1.0, col);
            ui.rounded(cx - 3.5, cy - 7.0, 4.0, 5.0, 2.0, col);
            ui.rounded(cx - 7.5, cy + 3.5, 15.0, 2.0, 1.0, col);
            ui.rounded(cx + 0.5, cy + 2.0, 4.0, 5.0, 2.0, col);
        }
    }
}


fn draw_discord_icon(ui: &mut Ui, cx: f32, cy: f32, col: u32, bg: u32) {
    // Crisp recognizable Discord Clyde controller silhouette
    ui.rounded(cx - 8.0, cy - 5.5, 16.0, 11.0, 4.5, col);
    ui.rounded(cx - 5.5, cy - 7.0, 4.0, 2.5, 1.2, col);
    ui.rounded(cx + 1.5, cy - 7.0, 4.0, 2.5, 1.2, col);
    ui.rounded(cx - 4.5, cy - 1.8, 3.0, 3.2, 1.5, bg);
    ui.rounded(cx + 1.5, cy - 1.8, 3.0, 3.2, 1.5, bg);
    ui.rounded(cx - 2.5, cy + 4.0, 5.0, 2.0, 1.0, bg);
}

fn draw_github_icon(ui: &mut Ui, cx: f32, cy: f32, col: u32) {
    ui.border(cx - 7.5, cy - 7.5, 15.0, 15.0, 7.5, col);
    ui.rounded(cx - 4.5, cy - 3.0, 9.0, 5.5, 2.5, col);
    ui.rounded(cx - 5.0, cy - 5.0, 2.5, 3.0, 1.0, col);
    ui.rounded(cx + 2.5, cy - 5.0, 2.5, 3.0, 1.0, col);
    ui.rounded(cx - 1.5, cy + 2.0, 3.0, 4.5, 1.0, col);
}

fn format_downloads(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn wrap_lines(ui: &Ui, face: Face, size: f32, text: &str, max_w: f32, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_owned()
        } else {
            format!("{current} {word}")
        };
        if ui.measure(face, size, &candidate) <= max_w || current.is_empty() {
            current = candidate;
        } else {
            lines.push(ui.fit_text(face, size, &current, max_w));
            if lines.len() + 1 >= max_lines {
                let rest_start = text.find(word).unwrap_or(0);
                lines.push(ui.fit_text(face, size, &text[rest_start..], max_w));
                return lines;
            }
            current = word.to_owned();
        }
    }
    if !current.is_empty() && lines.len() < max_lines {
        lines.push(ui.fit_text(face, size, &current, max_w));
    }
    lines
}

/// A low-cost mobile approximation of Nightly's pixel-sunset backdrop.
/// Its palette is taken from the pinned upstream scene; it is not the desktop renderer.
/// Deterministic per-column terrain noise for the backdrop, 0..1.
fn terra_noise(column: i32, seed: u32) -> f32 {
    let mut x = (column as u32).wrapping_mul(0x9e37_79b9) ^ seed;
    x ^= x >> 16;
    x = x.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 13;
    (x & 0xff) as f32 / 255.0
}

/// Continuously animated live-world backdrop: an original blocky voxel
/// landscape at dusk — gradient sky, a pixel sun, drifting block clouds and
/// three parallax block-stepped ridges — all plain rectangles, so it stays
/// cheap on mobile GPUs. `phase` is seconds since launch.
fn draw_launcher_scene(ui: &mut Ui, w: f32, h: f32, phase: f32) {
    use horizon_client::ui_theme::color::{hex, lerp};
    if theme::liquid() {
        return draw_moonlight_scene(ui, w, h, phase);
    }
    let dark = theme::dark();
    // Sky wash with a warm band near the horizon.
    let stops: &[(f32, u32)] = if dark {
        &[(0.0, 0x111533), (0.40, 0x272e55), (0.62, 0x4a3c63), (0.78, 0x87565c), (1.0, 0xb06a4a)]
    } else {
        &[(0.0, 0x82b4f1), (0.45, 0x9fc6f5), (0.75, 0xc6def9), (1.0, 0xeaf3fe)]
    };
    let rows = 36;
    for row in 0..rows {
        let t = row as f32 / (rows - 1) as f32;
        let mut color = hex(stops[stops.len() - 1].1, 255);
        for pair in stops.windows(2) {
            let ((t0, c0), (t1, c1)) = (pair[0], pair[1]);
            if t >= t0 && t <= t1 {
                color = lerp(hex(c0, 255), hex(c1, 255), (t - t0) / (t1 - t0).max(0.001));
                break;
            }
        }
        ui.rounded(0.0, row as f32 * h / (rows - 1) as f32 - 1.0, w, h / (rows - 1) as f32 + 2.0, 0.0, color);
    }
    if dark {
        for i in 0..44 {
            let x = w * (((i * 73 + 19) % 101) as f32 / 101.0);
            let y = h * 0.42 * (((i * 37 + 11) % 64) as f32 / 64.0);
            let size = if i % 9 == 0 { 2.0 } else { 1.2 };
            ui.rounded(x, y, size, size, 0.0, hex(0xdfe4ff, if i % 3 == 0 { 150 } else { 85 }));
        }
    }
    // A blocky sun low over the ridge line, with a soft warm halo.
    let sun_x = w * 0.58;
    let sun_y = h * 0.55;
    let radius = h * 0.075;
    let grid = (radius / 3.0).max(2.0);
    let (core, rim) = if dark { (0xffe9a8, 0xff8a4e) } else { (0xfff6c8, 0xffd27a) };
    ui.rounded(sun_x - radius * 2.6, sun_y - radius * 1.6, radius * 5.2, radius * 3.2,
        radius * 1.6, hex(rim, if dark { 30 } else { 44 }));
    for band in 0..6 {
        let yy = (band as f32 + 0.5) / 3.0 - 1.0;
        let half = ((1.0 - yy * yy).max(0.0).sqrt() * radius / grid).round() * grid;
        if half <= 0.0 { continue; }
        let color = lerp(hex(core, 255), hex(rim, 255), (yy + 1.0) * 0.5);
        ui.rounded(sun_x - half, sun_y + yy * radius, half * 2.0, radius / 3.0 + 1.0, 0.0, color);
    }
    // Blocky clouds drifting on three bands; each is a few stacked rectangles.
    let cloud_body = if dark { hex(0x3c4368, 205) } else { hex(0xffffff, 235) };
    let cloud_glow = if dark { hex(0x8a5f6b, 150) } else { hex(0xdbe9fb, 235) };
    for k in 0..5u32 {
        let speed = 3.0 + k as f32 * 1.4;
        let span = w + 180.0;
        let x = span - ((phase * speed + k as f32 * 263.0) % span);
        let y = h * (0.10 + 0.085 * k as f32);
        let s = 1.0 + (k % 3) as f32 * 0.35;
        ui.rounded(x - 180.0 + 26.0 * s, y - 9.0 * s, 34.0 * s, 10.0 * s, 0.0, cloud_body);
        ui.rounded(x - 180.0, y, 86.0 * s, 12.0 * s, 0.0, cloud_body);
        ui.rounded(x - 180.0 + 10.0 * s, y + 12.0 * s, 62.0 * s, 5.0 * s, 0.0, cloud_glow);
    }
    // Three parallax ridges of discrete blocks; the front one keeps a grassy lip.
    let bs = (w / 46.0).max(8.0);
    let layers: [(f32, f32, f32, u32, Option<u32>); 3] = if dark {
        [
            (1.6, 0.56, 5.0, 0x2c3153, None),
            (3.4, 0.68, 4.0, 0x20253f, None),
            (6.5, 0.80, 3.0, 0x141828, Some(0x3c5137)),
        ]
    } else {
        [
            (1.6, 0.56, 5.0, 0x9ec49a, None),
            (3.4, 0.68, 4.0, 0x7db077, None),
            (6.5, 0.80, 3.0, 0x5d9c58, Some(0x6fb166)),
        ]
    };
    for (li, &(speed, base, amp, color, lip)) in layers.iter().enumerate() {
        let seed = 0x5eed_0001u32.wrapping_mul(li as u32 + 1);
        let scroll = phase * speed;
        let first = (scroll / bs).floor() as i32;
        let frac = scroll - first as f32 * bs;
        let columns = (w / bs) as i32 + 2;
        for i in 0..columns {
            let column = first + i;
            let n = (terra_noise(column, seed) + terra_noise(column + 1, seed)) * 0.5;
            let blocks = (n * amp).round();
            let y_top = h * base - blocks * bs;
            let x = i as f32 * bs - frac;
            ui.rounded(x, y_top, bs + 0.5, (h - y_top).max(0.0), 0.0, hex(color, 255));
            if let Some(lip_color) = lip {
                ui.rounded(x, y_top, bs + 0.5, bs * 0.55, 0.0, hex(lip_color, 255));
            }
        }
    }
    // A quiet dim layer keeps the frosted panels readable over the scene.
    ui.rounded(0.0, 0.0, w, h, 0.0, if dark { hex(0x090b12, 128) } else { hex(0xffffff, 56) });
}


/// Smoothstep for the scene loop choreography.
fn sstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0).max(1e-4)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The Liquid Glass scene: a desaturated, moonlit voxel mountain night in
/// pure monochrome — black depths, silver ridges, white stars. An ~18s loop:
/// drifting cloud wisps, twinkling stars, a breathing moon halo and one
/// shooting star per loop. Publishes a gentle silver shimmer through
/// `theme::set_ember` so the glass panels above subtly breathe with it.
fn draw_moonlight_scene(ui: &mut Ui, w: f32, h: f32, phase: f32) {
    use horizon_client::ui_theme::color::{hex, lerp};
    let lt = phase % 18.0;
    let shimmer = (0.5
        + 0.28 * (phase * 0.37).sin()
        + 0.14 * (phase * 1.31 + 0.8).sin()
        + 0.08 * (phase * 3.7).sin())
    .clamp(0.0, 1.0);
    theme::set_ember(shimmer);

    // Black glassy OLED: true black throughout, almost no gradient – pure OLED
    let stops: [(f32, u32); 4] =
        [(0.0, 0x000000), (0.5, 0x000000), (0.8, 0x010102), (1.0, 0x020203)];
    let rows = 20;
    for row in 0..rows {
        let ty = row as f32 / (rows - 1) as f32;
        let mut color = hex(stops[3].1, 255);
        for pair in stops.windows(2) {
            let ((t0, c0), (t1, c1)) = (pair[0], pair[1]);
            if ty >= t0 && ty <= t1 {
                color = lerp(hex(c0, 255), hex(c1, 255), (ty - t0) / (t1 - t0).max(0.001));
                break;
            }
        }
        ui.rounded(0.0, row as f32 * h / (rows - 1) as f32 - 1.0, w,
            h / (rows - 1) as f32 + 2.0, 0.0, color);
    }
    // Stars on individual twinkle clocks, cool white.
    for i in 0..60u32 {
        let x = w * (((i * 73 + 19) % 101) as f32 / 101.0);
        let y = h * 0.56 * (((i * 37 + 11) % 64) as f32 / 64.0);
        let tw = 0.55 + 0.45 * (phase * (0.5 + (i % 7) as f32 * 0.21) + i as f32 * 1.37).sin();
        let alpha = (if i % 3 == 0 { 165.0 } else { 95.0 }) * tw;
        let size = if i % 9 == 0 { 2.2 } else { 1.3 };
        ui.rounded(x, y, size, size, 0.0, hex(0xe8ecf4, alpha.max(0.0) as u8));
    }
    // One shooting star per loop, streaking down-right at ~11s.
    let shoot = sstep(11.0, 11.12, lt) * (1.0 - sstep(11.55, 11.85, lt));
    if shoot > 0.01 {
        let t01 = ((lt - 11.0) / 0.85).clamp(0.0, 1.0);
        let (sx, sy) = (w * (0.12 + 0.30 * t01), h * (0.10 + 0.14 * t01));
        for seg in 0..6 {
            let back = seg as f32 * 0.035;
            let (px, py) = (w * (0.12 + 0.30 * (t01 - back).max(0.0)),
                h * (0.10 + 0.14 * (t01 - back).max(0.0)));
            let a = (shoot * (150.0 - seg as f32 * 24.0)).max(0.0);
            ui.rounded(px, py, 2.2, 2.2, 1.1, hex(0xf2f5fa, a as u8));
        }
        ui.rounded(sx, sy, 2.8, 2.8, 1.4, hex(0xffffff, (200.0 * shoot) as u8));
    }
    // Crescent moon with a halo that slowly breathes.
    let (mx, my, mr) = (w * 0.70, h * 0.19, h * 0.048);
    let halo = mr * (2.3 + 0.45 * (phase * 0.5).sin());
    ui.rounded(mx - halo, my - halo, halo * 2.0, halo * 2.0, halo,
        hex(0xaeb6c6, (10.0 + 12.0 * shimmer) as u8));
    ui.rounded(mx - mr, my - mr, mr * 2.0, mr * 2.0, mr, hex(0xe9edf5, 225));
    ui.rounded(mx - mr + mr * 0.60, my - mr - mr * 0.20, mr * 2.0, mr * 2.0, mr,
        hex(0x040507, 255));
    // Thin silver cloud wisps drifting on their own slow clocks.
    for i in 0..4u32 {
        let speed = 2.2 + i as f32 * 1.1;
        let span = w + 360.0;
        let x = ((phase * speed + i as f32 * 420.0) % span) - 180.0;
        let y = h * (0.16 + 0.11 * i as f32) + (phase * 0.2 + i as f32).sin() * 3.0;
        let cw = 170.0 + 50.0 * i as f32;
        let ch = 9.0 + 2.5 * i as f32;
        ui.rounded(x, y, cw, ch, ch * 0.5, hex(0x9ba4b6, (9 + i * 2) as u8));
        ui.rounded(x + cw * 0.2, y - ch * 0.45, cw * 0.55, ch * 0.8, ch * 0.4,
            hex(0xaab2c2, 7));
    }
    // Three parallax ranges of blocky mountains, each ridge top kissed by
    // a thin line of moonlight that brightens with the shimmer.
    let layers: [(f32, f32, u32, u32, u8); 3] = [
        (1.4, 0.58, 0xb7c3_0003, 0x0d0f14, 70),
        (2.6, 0.68, 0xb7c3_0017, 0x08090d, 48),
        (4.2, 0.78, 0xb7c3_0031, 0x040406, 30),
    ];
    for (speed, base, seed, color, rim_a) in layers {
        let bs = (w / 46.0).max(8.0);
        let scroll = phase * speed;
        let first = (scroll / bs).floor() as i32;
        let frac = scroll - first as f32 * bs;
        for i in 0..((w / bs) as i32 + 2) {
            let column = first + i;
            let n = (terra_noise(column, seed) + terra_noise(column + 1, seed)) * 0.5;
            let y_top = h * base - (n * 4.0).round() * bs * 0.55;
            let x = i as f32 * bs - frac;
            ui.rounded(x, y_top, bs + 0.5, (h - y_top).max(0.0), 0.0, hex(color, 255));
            // Moonlit ridge line.
            let rim = (rim_a as f32 * (0.6 + 0.4 * shimmer)) as u8;
            ui.rounded(x, y_top, bs + 0.5, 1.6, 0.0, hex(0x6f7a8e, rim));
        }
    }
    // Deepen the foreground so the dock floats over true black.
    ui.rounded(0.0, h * 0.82, w, h * 0.18, 0.0, hex(0x020203, 150));
    // A final whisper of dim keeps text crisp over the brightest ridges.
    ui.rounded(0.0, 0.0, w, h, 0.0, hex(0x020308, 64));
}

/// The big fully-rounded Play control with a triangle icon and press
/// feedback; its target is inserted at the front so it always wins the tap.
#[allow(dead_code)]
fn draw_play_pill(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    label: &str,
) {
    let [x, y, w, h] = rect;
    let press_t = ACTIVE_FEEDBACK.with(|fb| {
        fb.borrow().and_then(|(act, ts)| {
            let el = ts.elapsed().as_secs_f32();
            (act == Action::Play && el < 0.22).then_some(el / 0.22)
        })
    });
    let v_start = ui.batch_mut().vertices.len();
    let mut bg = theme::CRIMSON();
    if let Some(t) = press_t {
        bg = horizon_client::ui_theme::color::lerp(bg, theme::CRIMSON_BRIGHT(), (1.0 - t).powi(2) * 0.30);
    }
    ui.rounded(x, y, w, h, h * 0.5, bg);
    let font = if h >= 50.0 { 19.0 } else { 16.5 };
    let text_w = ui.measure(Face::LauncherMedium, font, label);
    let tri = font * 0.62;
    let start_x = x + (w - text_w - tri - 9.0) * 0.5;
    let cy = y + h * 0.5;
    {
        let white = ui.atlas_ref().white;
        let uv = [(white.u0 + white.u1) * 0.5, (white.v0 + white.v1) * 0.5];
        let color = theme::ON_CRIMSON();
        let points = [
            [ui.to_batch(start_x), ui.to_batch(cy - tri * 0.55)],
            [ui.to_batch(start_x), ui.to_batch(cy + tri * 0.55)],
            [ui.to_batch(start_x + tri), ui.to_batch(cy)],
        ];
        for pos in points {
            ui.batch_mut().vertices.push(horizon_client::hud::HudVertex {
                pos, uv, color, depth: 0.0, glint_uv: [0.0, 0.0],
            });
        }
    }
    ui.text(Face::LauncherMedium, font, start_x + tri + 9.0,
        text_vcenter(y, h, font), theme::ON_CRIMSON(), label);
    if let Some(t) = press_t {
        let dip = (t * std::f32::consts::PI).sin() * (1.0 - t * 0.45);
        ui.animate_vertices(v_start, 0.0, 0.0, 1.0 - 0.042 * dip, [x + w * 0.5, cy], 1.0);
    }
    targets.insert(0, (rect, Action::Play));
}

/// A small outlined status pill ("READY", "SELECTED"); returns its width.
#[allow(dead_code)]
fn draw_badge(ui: &mut Ui, x: f32, y: f32, label: &str, color: u32) -> f32 {
    let w = ui.measure(Face::LauncherMedium, 11.0, label) + 20.0;
    ui.border(x, y, w, 22.0, 11.0, color);
    draw_centered_text(ui, Face::LauncherMedium, 11.0, [x, y, w, 22.0], color, label);
    w
}

/// Chunky "Minecraft slab" panel for the lobby home, per the chosen
/// reference: a notched pixel silhouette with a dark outline, a soft drop
/// shadow, a two-tone fill and a bright bevel along the top edge.
fn pixel_panel(ui: &mut Ui, rect: [f32; 4], top: u32, bottom: u32) {
    use horizon_client::ui_theme::color::{hex, lerp};
    let [x, y, w, h] = rect;
    let n = 5.0_f32.min(w * 0.12).min(h * 0.2); // corner notch size
    let b = 2.5; // outline thickness
    // Soft drop shadow (single pass so overlaps never double up).
    ui.rounded(x + 2.0, y + 4.0, w - 2.0, h, 3.0, hex(0x04060a, 95));
    // Notched outline silhouette: two overlapping rectangles.
    let outline = hex(0x10131a, 240);
    ui.rounded(x + n, y, w - 2.0 * n, h, 0.0, outline);
    ui.rounded(x, y + n, w, h - 2.0 * n, 0.0, outline);
    // Fill, inset by the outline, same notched union.
    let (fx, fy, fw, fh) = (x + b, y + b, w - 2.0 * b, h - 2.0 * b);
    ui.rounded(fx + n, fy, fw - 2.0 * n, fh, 0.0, bottom);
    ui.rounded(fx, fy + n, fw, fh - 2.0 * n, 0.0, bottom);
    // Brighter top half for the two-tone slab look.
    let split = fh * 0.46;
    ui.rounded(fx + n, fy, fw - 2.0 * n, split, 0.0, top);
    if split > n {
        ui.rounded(fx, fy + n, fw, split - n, 0.0, top);
    }
    // Bevel highlight along the top edge.
    ui.rounded(fx + n, fy, fw - 2.0 * n, 2.5, 0.0, lerp(top, hex(0xffffff, 255), 0.32));
}

/// Tap feedback for lobby slabs: briefly dips the vertices drawn since
/// `v_start` toward the slab centre, matching the pill/button behaviour.
fn press_scale(ui: &mut Ui, v_start: usize, rect: [f32; 4], action: Action) {
    let press_t = ACTIVE_FEEDBACK.with(|fb| {
        fb.borrow().and_then(|(act, ts)| {
            let el = ts.elapsed().as_secs_f32();
            (act == action && el < 0.22).then_some(el / 0.22)
        })
    });
    if let Some(t) = press_t {
        let dip = (t * std::f32::consts::PI).sin() * (1.0 - t * 0.45);
        ui.animate_vertices(v_start, 0.0, 0.0, 1.0 - 0.05 * dip,
            [rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5], 1.0);
    }
}

/// A small filled "play" triangle with its left edge at `x`, centred on `cy`.
fn draw_play_triangle(ui: &mut Ui, x: f32, cy: f32, size: f32, color: u32) {
    let white = ui.atlas_ref().white;
    let uv = [(white.u0 + white.u1) * 0.5, (white.v0 + white.v1) * 0.5];
    let points = [
        [ui.to_batch(x), ui.to_batch(cy - size * 0.55)],
        [ui.to_batch(x), ui.to_batch(cy + size * 0.55)],
        [ui.to_batch(x + size), ui.to_batch(cy)],
    ];
    for pos in points {
        ui.batch_mut().vertices.push(horizon_client::hud::HudVertex {
            pos, uv, color, depth: 0.0, glint_uv: [0.0, 0.0],
        });
    }
}

/// Headset glyph for the lobby Help button.
fn draw_headset_icon(ui: &mut Ui, cx: f32, cy: f32, col: u32) {
    ui.border(cx - 7.5, cy - 7.5, 15.0, 14.5, 7.0, col);
    ui.rounded(cx - 9.0, cy - 1.0, 4.5, 8.0, 2.0, col);
    ui.rounded(cx + 4.5, cy - 1.0, 4.5, 8.0, 2.0, col);
    ui.rounded(cx + 1.5, cy + 6.2, 6.0, 2.2, 1.1, col);
}

/// Minimal line icons for home tiles that are not launcher pages, built from
/// the same primitives as the page icons.
fn draw_tile_icon(ui: &mut Ui, kind: u8, cx: f32, cy: f32, col: u32) {
    match kind {
        // Worlds: a globe ring with an equator.
        0 => {
            ui.border(cx - 8.0, cy - 8.0, 16.0, 16.0, 8.0, col);
            ui.rounded(cx - 8.0, cy - 1.0, 16.0, 2.0, 1.0, col);
            ui.rounded(cx - 1.0, cy - 8.0, 2.0, 16.0, 1.0, col);
        }
        // Multiplayer: two heads side by side.
        1 => {
            ui.rounded(cx - 8.0, cy - 6.0, 7.0, 7.0, 3.5, col);
            ui.rounded(cx + 1.0, cy - 6.0, 7.0, 7.0, 3.5, col);
            ui.rounded(cx - 8.5, cy + 3.0, 8.0, 5.0, 2.5, col);
            ui.rounded(cx + 0.5, cy + 3.0, 8.0, 5.0, 2.5, col);
        }
        // Screenshots: a camera body with a lens dot.
        2 => {
            ui.border(cx - 8.5, cy - 6.0, 17.0, 12.5, 4.0, col);
            ui.rounded(cx - 2.5, cy - 2.2, 5.0, 5.0, 2.5, col);
            ui.rounded(cx - 6.0, cy - 8.5, 5.0, 3.0, 1.5, col);
        }
        // Logs: three text lines.
        3 => {
            for (i, lw) in [16.0, 12.0, 14.0].iter().enumerate() {
                ui.rounded(cx - 8.0, cy - 6.0 + i as f32 * 5.5, *lw, 2.4, 1.2, col);
            }
        }
        // Boost/settings bolt: a simple two-part lightning mark.
        _ => {
            ui.rounded(cx - 1.0, cy - 8.0, 4.5, 8.5, 1.5, col);
            ui.rounded(cx - 3.5, cy - 0.5, 4.5, 8.5, 1.5, col);
            ui.rounded(cx - 5.0, cy - 1.2, 10.0, 2.4, 1.2, col);
        }
    }
}

fn draw_account_chip(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    status: &str,
    signed_in: bool,
) {
    let [x, y, w, h] = rect;
    ui.rounded(x, y, w, h, h * 0.5, theme::CARD());
    ui.border(
        x,
        y,
        w,
        h,
        h * 0.5,
        if signed_in {
            theme::CRIMSON()
        } else {
            theme::BORDER_CRIMSON()
        },
    );
    let dot_col = if signed_in {
        theme::ONLINE()
    } else {
        theme::CRIMSON_BRIGHT()
    };
    ui.rounded(x + w - 16.0, y + (h - 8.0) * 0.5, 8.0, 8.0, 4.0, dot_col);
    let label = ui.fit_text(Face::LauncherMedium, 13.5, status, (w - 34.0).max(24.0));
    ui.text(
        Face::LauncherMedium,
        13.5,
        x + 14.0,
        text_vcenter(y, h, 13.5),
        theme::TEXT(),
        &label,
    );
    targets.push((rect, Action::Page(Page::Account)));
}

impl Launcher {
    /// The same spatial focus algorithm as the game's controller menus.
    pub(super) fn navigate(&mut self, key: &KeyEvent) -> bool {
        self.navigate_key(&key.logical_key)
    }
    pub(super) fn navigate_key(&mut self, key: &winit::keyboard::Key) -> bool {
        use horizon_client::gamepad::{self, Button};
        use winit::keyboard::{Key, NamedKey};
        if self.pending_focus.is_some() {
            return true;
        }
        if self.catalog.editing.is_some() || self.targets.is_empty() {
            return false;
        }
        let button = match key {
            Key::Named(NamedKey::ArrowUp) => Button::DPadUp,
            Key::Named(NamedKey::ArrowDown) => Button::DPadDown,
            Key::Named(NamedKey::ArrowLeft) => Button::DPadLeft,
            Key::Named(NamedKey::ArrowRight) => Button::DPadRight,
            Key::Named(NamedKey::Tab) => {
                self.focus = Some(self.focus.map_or(0, |i| (i + 1) % self.targets.len()));
                return true;
            }
            _ => return false,
        };
        if matches!(button, Button::DPadUp | Button::DPadDown)
            && let Some((_, action)) = self.focus.and_then(|i| self.targets.get(i))
            && let Some(index) = self.list_targets.iter().position(|(_, a)| a == action)
        {
            let rects: Vec<_> = self.list_targets.iter().map(|(r, _)| *r).collect();
            if let Some(next) = gamepad::focus_step_strict(rects[index], button, &rects)
                && let Some(view) = self.list_view
            {
                let (rect, action) = self.list_targets[next];
                let scroll = if let Some(chooser) = &mut self.chooser {
                    &mut chooser.scroll
                } else {
                    &mut self.catalog.scroll
                };
                *scroll = super::gesture::reveal(*scroll, rect, view);
                self.pending_focus = Some(action);
                self.focus = None;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                return true;
            }
        }
        self.focus = Some(
            if let Some(index) = self.focus.filter(|i| *i < self.targets.len()) {
                let rects: Vec<_> = self.targets.iter().map(|(rect, _)| *rect).collect();
                gamepad::focus_step_rect(rects[index], button, &rects).unwrap_or(index)
            } else {
                0
            },
        );
        true
    }

}

struct DrawCtx<'a> {
    home: &'a PathBuf,
    install: &'a Arc<Mutex<horizon_client::official_install::InstallProgress>>,
    worker: &'a Option<mpsc::Receiver<std::result::Result<(), String>>>,
    error: &'a Option<String>,
    catalog: &'a mut Catalog,
    home_tab: HomeTab,
    targets: &'a mut Vec<([f32; 4], Action)>,
    list_targets: &'a mut Vec<([f32; 4], Action)>,
    list_view: &'a mut Option<[f32; 3]>,
    login: &'a login::Login,
    volt_accounts: &'a crate::volt_accounts::VoltAccountManager,
    preview_yaw: f32,
    preview_pitch: f32,
    preview_vel_yaw: f32,
    sidebar_anim: f32,
    idle_phase: f32,
    /// Live home-tab switch transition: (slide offset in dp, alpha multiplier).
    tab_anim: Option<(f32, f32)>,
    preview_rect: &'a mut Option<[f32; 4]>,
}

impl DrawCtx<'_> {
    /// Bottom shell per the chosen reference: account chip on the left, a
    /// centred pill dock (Home · Instances · Browse · Settings) and round
    /// Discord / Files buttons on the right. Targets are pushed before any
    /// page content so the shell always wins its taps.
    fn draw_shell(&mut self, ui: &mut Ui, w: f32, h: f32, _account_status: &str) {
        // Black Glassy: far-left vertical rail instead of bottom dock
        let bar_w = 78.0;
        let bar_x = 8.0;
        let bar_y = 12.0;
        let bar_h = h - 24.0;
        ui.rounded(bar_x, bar_y, bar_w, bar_h, 18.0, theme::GLASS());
        ui.border(bar_x, bar_y, bar_w, bar_h, 18.0, theme::GLASS_BORDER());
        let items: [(&str, Page, Action); 7] = [
            ("Home", Page::Play, Action::Page(Page::Play)),
            ("Instances", Page::Installed, Action::Profile),
            ("Browse", Page::Mods, Action::Page(Page::Mods)),
            ("Packs", Page::Packs, Action::Page(Page::Packs)),
            ("Shaders", Page::Shaders, Action::Page(Page::Shaders)),
            ("Controls", Page::Controls, Action::Page(Page::Controls)),
            ("Settings", Page::Settings, Action::Page(Page::Settings)),
        ];
        // Vertical centering: 7 items, 64px each + 8px gap = 504px total, centered
        let item_h = 62.0;
        let gap = 6.0;
        let total = items.len() as f32 * item_h + (items.len() as f32 - 1.0) * gap;
        let start_y = bar_y + (bar_h - total) * 0.5;
        for (i, (label, icon, action)) in items.iter().enumerate() {
            let y = start_y + i as f32 * (item_h + gap);
            let rect = [bar_x + 6.0, y, bar_w - 12.0, item_h];
            let active = match i {
                0 => self.catalog.page == Page::Play,
                1 => false,
                2 => matches!(self.catalog.page, Page::Mods | Page::Installed),
                3 => self.catalog.page == Page::Packs,
                4 => self.catalog.page == Page::Shaders,
                5 => self.catalog.page == Page::Controls,
                _ => matches!(self.catalog.page, Page::Settings | Page::VoltSettings),
            };
            if active {
                // Black glassy active: subtle inner glow, not white badge
                ui.rounded(rect[0], rect[1], rect[2], rect[3], 14.0, theme::HOVER());
                ui.border(rect[0], rect[1], rect[2], rect[3], 14.0, theme::BORDER());
                // Active left accent line
                ui.rounded(rect[0] - 1.0, rect[1] + 12.0, 2.5, rect[3] - 24.0, 1.2, theme::TEXT());
            }
            let col = if active { theme::TEXT() } else { theme::MUTED() };
            // Icon centered top, label below – vertical stack
            draw_page_icon(ui, *icon, rect[0] + rect[2] * 0.5, rect[1] + 18.0, col);
            draw_centered_text(ui, Face::LauncherMedium, 10.0,
                [rect[0], rect[1] + 30.0, rect[2], 18.0],
                if active { theme::TEXT() } else { theme::DIM() }, label);
            self.targets.push((rect, *action));
        }
    }

    #[allow(dead_code)]
    fn draw_sidebar(&mut self, ui: &mut Ui, rx: f32, ry: f32, rw: f32, rh: f32) {
        ui.rounded(rx, ry, rw, rh, 18.0, theme::GLASS());
        ui.border(rx, ry, rw, rh, 18.0, theme::GLASS_BORDER());
        ui.text(Face::LauncherMedium, 12.5, rx + 16.0, ry + 20.0, theme::MUTED(), "Instances");
        let add = [rx + rw - 48.0, ry + 10.0, 38.0, 38.0];
        ui.rounded(add[0], add[1], add[2], add[3], 11.0, theme::GLASS_INNER());
        ui.border(add[0], add[1], add[2], add[3], 11.0, theme::GLASS_BORDER());
        draw_centered_text(ui, Face::LauncherMedium, 22.0, add, theme::TEXT(), "+");
        self.targets.push((add, Action::Profile)); // chooser includes Create profile

        let mut ids = vec![0, self.catalog.config.profile];
        ids.extend(std::fs::read_dir(self.home.join("profiles")).into_iter().flatten().flatten()
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().to_str()?.strip_prefix("profile-")?.parse::<u32>().ok()));
        ids.sort_unstable();
        ids.dedup();
        let row_step = 62.0;
        let max_rows = (((rh - 280.0).max(row_step) / row_step) as usize).clamp(1, 7);
        if ids.len() > max_rows && !ids[..max_rows].contains(&self.catalog.config.profile) {
            ids.truncate(max_rows.saturating_sub(1));
            ids.push(self.catalog.config.profile);
        } else {
            ids.truncate(max_rows);
        }
        for (index, id) in ids.iter().enumerate() {
            let row = [rx + 10.0, ry + 58.0 + index as f32 * row_step, rw - 20.0, 56.0];
            // Every instance is a frosted card; the active one is brighter
            // with an accent border, as in the chosen concept.
            let selected = *id == self.catalog.config.profile;
            ui.rounded(row[0], row[1], row[2], row[3], 14.0,
                if selected { theme::CRIMSON_BADGE() } else { theme::GLASS_INNER() });
            ui.border(row[0], row[1], row[2], row[3], 14.0,
                if selected { theme::BORDER_CRIMSON() } else { theme::GLASS_BORDER() });
            let chip = [row[0] + 8.0, row[1] + 8.0, 40.0, 40.0];
            ui.rounded(chip[0], chip[1], chip[2], chip[3], 11.0,
                horizon_client::ui_theme::color::hex(0xf2f3fb, 230));
            ui.volt_logo(chip[0] + 4.0, chip[1] + 4.0, 32.0, 32.0);
            let name = if *id == 0 { "Volt".to_owned() } else { format!("Profile {id}") };
            let name = ui.fit_text(Face::LauncherMedium, 13.5, &name, rw - 96.0);
            ui.text(Face::LauncherMedium, 13.5, row[0] + 58.0, row[1] + 9.0, theme::TEXT(), &name);
            ui.rounded(row[0] + 58.0, row[1] + 37.0, 6.0, 6.0, 3.0, theme::ONLINE());
            ui.text(Face::LauncherRegular, 11.0, row[0] + 68.0, row[1] + 31.0, theme::MUTED(), "Native build · 26.3");
            self.targets.push((row, Action::ChooseProfile(*id)));
        }

        // All existing destinations stay accessible; Account contains the 3D preview.
        let short = rh < 400.0;
        let foot_y = ry + rh - if short { 172.0 } else { 254.0 };
        let foot_step = if short { 30.0 } else { 40.0 };
        let foot_height = if short { 27.0 } else { 36.0 };
        for (i, (label, action)) in [("Account", Action::Page(Page::Account)),
            ("Volt Settings", Action::Page(Page::VoltSettings))].iter().enumerate() {
            styled_button(ui, self.targets, [rx + 12.0, foot_y + i as f32 * foot_step, rw - 24.0, foot_height],
                label, *action, BtnStyle::Subtle, false);
        }
        let discord_h = if short { 32.0 } else { 42.0 };
        let discord = [rx + 12.0, foot_y + 2.0 * foot_step + 4.0, rw - 24.0, discord_h];
        let discord_bg = horizon_client::ui_theme::color::hex(0x5865f2, 255);
        ui.rounded(discord[0], discord[1], discord[2], discord[3], discord[3] * 0.5, discord_bg);
        draw_discord_icon(ui, discord[0] + 26.0, discord[1] + discord[3] * 0.5, theme::TEXT(), discord_bg);
        draw_centered_text(ui, Face::LauncherMedium, 13.5,
            [discord[0] + 26.0, discord[1], discord[2] - 26.0, discord[3]], theme::TEXT(), "Join the Discord");
        self.targets.push((discord, Action::Discord));
        let link_y = discord[1] + discord_h + if short { 6.0 } else { 10.0 };
        let link_step = if short { 26.0 } else { 32.0 };
        for (i, (label, action)) in [("Sponsor", Action::ReviewApp), ("Website", Action::Repository)]
            .iter().enumerate() {
            let row = [rx + 12.0, link_y + i as f32 * link_step, rw - 24.0, link_step - 2.0];
            let cx = row[0] + 12.0;
            let cy = row[1] + row[3] * 0.5;
            if i == 0 {
                // A simple heart from two circles over a square, drawn locally.
                ui.rounded(cx - 6.0, cy - 5.0, 6.5, 6.5, 3.2, theme::MUTED());
                ui.rounded(cx - 0.5, cy - 5.0, 6.5, 6.5, 3.2, theme::MUTED());
                ui.rounded(cx - 4.4, cy - 2.4, 8.8, 6.6, 2.4, theme::MUTED());
            } else {
                // A simple globe: ring plus meridian and equator strokes.
                ui.rounded(cx - 6.0, cy - 6.0, 12.0, 12.0, 6.0, theme::MUTED());
                ui.rounded(cx - 4.6, cy - 4.6, 9.2, 9.2, 4.6, theme::GLASS());
                ui.rounded(cx - 1.0, cy - 6.0, 2.0, 12.0, 1.0, theme::MUTED());
                ui.rounded(cx - 6.0, cy - 1.0, 12.0, 2.0, 1.0, theme::MUTED());
            }
            ui.text(Face::LauncherMedium, 13.0, row[0] + 26.0, text_vcenter(row[1], row[3], 13.0),
                theme::MUTED(), label);
            self.targets.push((row, *action));
        }
        if !short {
            // The short source commit identifies the exact installed build
            // ("is the fix on this phone?") at a glance.
            let commit = option_env!("HORIZON_ANDROID_BUILD_COMMIT").unwrap_or("unknown");
            ui.text(Face::LauncherRegular, 11.0, rx + 16.0, ry + rh - 26.0, theme::DIM(),
                &format!("Volt Launcher {} (build {})", crate::backend::VERSION,
                    commit.get(..7).unwrap_or(commit)));
        }
    }

    /// Reference-style top navigation. Every entry points at an existing Volt
    /// destination: Wardrobe opens the Account wardrobe, Versions opens the
    /// profile/version chooser, Patch notes opens the release history, and
    /// Build log opens the Console tab.
    #[allow(dead_code)]
    fn draw_topbar(&mut self, ui: &mut Ui, area: [f32; 4], account_status: &str) {
        let [x, y, w, h] = area;
        // The whole navigation sits on one rounded frosted bar.
        ui.rounded(x - 8.0, y - 7.0, w + 16.0, h + 14.0, 16.0, theme::GLASS());
        ui.border(x - 8.0, y - 7.0, w + 16.0, h + 14.0, 16.0, theme::GLASS_BORDER());
        let account_w = if w >= 760.0 { 158.0 } else { 112.0 };
        let gear_w = h;
        let tabs = [("Home", Action::Page(Page::Play)), ("Mods", Action::Page(Page::Mods)),
            ("Servers", Action::HomeTab(HomeTab::Multiplayer)), ("Wardrobe", Action::Page(Page::Account)),
            ("Versions", Action::Profile), ("Patch notes", Action::PatchNotes),
            ("Build log", Action::HomeTab(HomeTab::Console))];
        let available = (w - account_w - gear_w - 20.0).max(260.0);
        let tab_w = (available / tabs.len() as f32).min(102.0);
        for (i, (label, action)) in tabs.iter().enumerate() {
            let rect = [x + i as f32 * tab_w, y, tab_w - 6.0, h];
            let selected = match action {
                Action::HomeTab(tab) => self.catalog.page == Page::Play && self.home_tab == *tab,
                Action::Page(Page::Play) => self.catalog.page == Page::Play
                    && !matches!(self.home_tab, HomeTab::Multiplayer | HomeTab::Console),
                Action::Page(page) => self.catalog.page == *page,
                _ => false,
            };
            if selected {
                ui.rounded(rect[0], rect[1] + 2.0, rect[2], rect[3] - 4.0, (rect[3] - 4.0) * 0.5, theme::CRIMSON());
            }
            let label = ui.fit_text(Face::LauncherMedium, 13.0, label, rect[2] - 8.0);
            draw_centered_text(ui, Face::LauncherMedium, 13.0, rect,
                if selected { theme::ON_CRIMSON() } else { theme::MUTED() }, &label);
            self.targets.push((rect, *action));
        }
        // A small gear keeps launcher Settings one tap away without crowding the bar.
        let gear = [x + w - account_w - gear_w - 8.0, y, gear_w, h];
        let (gx, gy) = (gear[0] + gear[2] * 0.5, gear[1] + gear[3] * 0.5);
        let selected = self.catalog.page == Page::Settings;
        if selected { ui.rounded(gear[0], gear[1], gear[2], gear[3], 12.0, theme::CRIMSON_BADGE()); }
        for i in 0..8 {
            let th = i as f32 * std::f32::consts::TAU / 8.0;
            ui.rounded(gx + th.cos() * 7.6 - 1.9, gy + th.sin() * 7.6 - 1.9, 3.8, 3.8, 1.9, theme::MUTED());
        }
        ui.rounded(gx - 6.4, gy - 6.4, 12.8, 12.8, 6.4, theme::MUTED());
        ui.rounded(gx - 3.0, gy - 3.0, 6.0, 6.0, 3.0, if selected { theme::CRIMSON_BADGE() } else { theme::GLASS() });
        self.targets.push((gear, Action::Page(Page::Settings)));
        draw_account_chip(ui, self.targets, [x + w - account_w, y, account_w, h],
            account_status, self.login.signed_in);
    }

    fn draw_compact_header(&mut self, ui: &mut Ui, w: f32, account_status: &str) {
        // Brand flush left, account chip flush right.
        ui.rounded(6.0, 10.0, 168.0, 34.0, 9.0, theme::CARD());
        ui.border(6.0, 10.0, 168.0, 34.0, 9.0, theme::BORDER_CRIMSON());
        ui.volt_logo(12.0, 14.0, 26.0, 26.0);
        ui.text(
            Face::LauncherMedium,
            12.5,
            44.0,
            text_vcenter(10.0, 34.0, 12.5),
            theme::TEXT(),
            "VOLT LAUNCHER",
        );
        draw_account_chip(
            ui,
            self.targets,
            [182.0, 10.0, (w - 188.0).max(110.0), 34.0],
            account_status,
            self.login.signed_in,
        );

        // Segmented pill dock with a smoothly gliding selection.
        ui.rounded(6.0, 52.0, w - 12.0, 40.0, 10.0, theme::RAIL());
        ui.border(6.0, 52.0, w - 12.0, 40.0, 10.0, theme::BORDER());
        let tab_w = (w - 20.0) / Page::ALL.len() as f32;
        ui.rounded(
            10.0 + self.sidebar_anim * tab_w + 1.0,
            56.0,
            tab_w - 2.0,
            32.0,
            7.0,
            theme::CRIMSON(),
        );
        for (i, &page) in Page::ALL.iter().enumerate() {
            let tx = 10.0 + i as f32 * tab_w;
            let prox = (1.0 - (self.sidebar_anim - i as f32).abs()).clamp(0.0, 1.0);
            let label = match page {
                Page::Installed => "Files",
                Page::VoltSettings => "Volt",
                Page::Settings => "Setup",
                _ => page.name(),
            };
            let rect = [tx + 1.0, 56.0, tab_w - 2.0, 32.0];
            draw_centered_text(
                ui,
                Face::LauncherMedium,
                12.5,
                rect,
                horizon_client::ui_theme::color::lerp(theme::MUTED(), theme::ON_CRIMSON(), prox),
                label,
            );
            self.targets.push((rect, Action::Page(page)));
        }
    }

    /// The instance header, actions and tabs shared by Home and its Mods tab.
    fn draw_instance_page(&mut self, ui: &mut Ui, area: [f32; 4], _compact: bool, account_status: &str) {
        let [x, y, w, h] = area;
        // Browse (dock) lands on the bundled official-mods list.
        if self.catalog.page == Page::Mods {
            self.draw_mods_landing(ui, area, account_status);
            return;
        }
        let installed = horizon_client::official_install::ready(self.home);

        // Secondary home views (Worlds, Multiplayer, Shots, Logs): a back
        // chip and title, then the existing tab content.
        if self.home_tab != HomeTab::Overview {
            let back = [x, y, 92.0, 38.0];
            ui.rounded(back[0], back[1], back[2], back[3], 19.0, theme::GLASS());
            ui.border(back[0], back[1], back[2], back[3], 19.0, theme::GLASS_BORDER());
            draw_centered_text(ui, Face::LauncherMedium, 13.0, back, theme::TEXT(), "< Home");
            self.targets.push((back, Action::HomeTab(HomeTab::Overview)));
            let title = match self.home_tab {
                HomeTab::Worlds => "Worlds",
                HomeTab::Multiplayer => "Multiplayer",
                HomeTab::Screenshots => "Captures",
                HomeTab::Replays => "Replays",
                _ => "Launcher log",
            };
            ui.text(Face::Display, 21.0, x + 106.0, y + 5.0, theme::TEXT(), title);
            let body = [x, y + 50.0, w, (h - 50.0).max(0.0)];
            let body_v_start = ui.batch_mut().vertices.len();
            self.draw_home_tab(ui, body, account_status, installed);
            if let Some((dx, alpha)) = self.tab_anim {
                ui.animate_vertices(body_v_start, dx, 0.0, 1.0,
                    [x + w * 0.5, body[1] + body[3] * 0.5], alpha);
            }
            return;
        }

        // ----- Home overview: the full-screen lobby, per the chosen reference -----
        self.draw_lobby_home(ui, area, account_status);
    }

    /// The Liquid Glass home, built to the chosen silver mockup: moonlit
    /// vista behind, chrome VOLT brand top-left, glass account chip
    /// top-right over the Worlds / Servers cards, one enormous chrome-rimmed
    /// PLAY pill in the centre, a floating icon dock below it, and quiet
    /// utility chips bottom-left. Strictly monochrome.
    fn draw_liquid_home(&mut self, ui: &mut Ui, area: [f32; 4], account_status: &str) {
        use horizon_client::official_install::InstallStage;
        use horizon_client::ui_theme::color::hex;
        let [ax, ay, w, h] = area;
        let installed = horizon_client::official_install::ready(self.home);
        let installing = self.worker.is_some();
        let progress = *self.install.lock().unwrap();
        let compact = w < 760.0;
        let m = if compact { 12.0 } else { 18.0 };
        let shimmer = theme::ember();

        // ---- Far-left rail: all navigation (dashboard) ----
        let rail_w = 78.0;
        let rail_x = ax + 8.0;
        let rail_y = ay + 12.0;
        let rail_h = h - 24.0;
        ui.rounded(rail_x, rail_y, rail_w, rail_h, 18.0, theme::GLASS());
        ui.border(rail_x, rail_y, rail_w, rail_h, 18.0, theme::GLASS_BORDER());
        let nav: [(&str, Page, Action); 7] = [
            ("Home", Page::Play, Action::Page(Page::Play)),
            ("Instances", Page::Installed, Action::Profile),
            ("Browse", Page::Mods, Action::Page(Page::Mods)),
            ("Packs", Page::Packs, Action::Page(Page::Packs)),
            ("Shaders", Page::Shaders, Action::Page(Page::Shaders)),
            ("Controls", Page::Controls, Action::Page(Page::Controls)),
            ("Settings", Page::Settings, Action::Page(Page::Settings)),
        ];
        let item_h = 62.0;
        let gap = 6.0;
        let total = nav.len() as f32 * item_h + (nav.len() as f32 - 1.0) * gap;
        let start_y = rail_y + (rail_h - total) * 0.5;
        for (i, (label, page, action)) in nav.iter().enumerate() {
            let y = start_y + i as f32 * (item_h + gap);
            let rect = [rail_x + 6.0, y, rail_w - 12.0, item_h];
            let active = *page == Page::Play;
            if active {
                ui.rounded(rect[0], rect[1], rect[2], rect[3], 14.0, theme::HOVER());
                ui.border(rect[0], rect[1], rect[2], rect[3], 14.0, theme::BORDER());
                ui.rounded(rect[0] - 1.0, rect[1] + 12.0, 2.5, rect[3] - 24.0, 1.2, theme::TEXT());
            }
            let col = if active { theme::TEXT() } else { theme::MUTED() };
            draw_page_icon(ui, *page, rect[0] + rect[2] * 0.5, rect[1] + 18.0, col);
            draw_centered_text(ui, Face::LauncherMedium, 10.0, [rect[0], rect[1] + 30.0, rect[2], 18.0], if active { theme::TEXT() } else { theme::DIM() }, label);
            self.targets.push((rect, *action));
        }

        // ---- Brand, top-left (shifted right of rail) ----
        let brand_x = rail_x + rail_w + 16.0;
        ui.volt_logo(brand_x, ay + m, 34.0, 34.0);
        ui.text(Face::Display, 21.0, brand_x + 44.0, ay + m + 4.0, hex(0xf2f4f8, 255), "VOLT");
        let secs = self.catalog.config.play_seconds;
        let time = if secs == 0 { "0 h".to_owned() } else if secs < 3600 { format!("{} min", (secs / 60).max(1)) } else { format!("{} h", secs / 3600) };
        ui.text(Face::LauncherRegular, 10.5, brand_x + 44.0, ay + m + 26.0, theme::DIM(), &format!("26.3 · {time} played"));

        // ---- Account chip, top-right: skin head + name -> Account page ----
        let (name, badge) = account_status.split_once(" • ").map(|(n,b)| (n.to_owned(), b.to_owned())).unwrap_or_else(|| (account_status.to_owned(), "Offline".to_owned()));
        let chip_h = 44.0;
        let chip_w = (ui.measure(Face::LauncherMedium, 13.5, &name) + chip_h + 26.0).min(w * 0.30).max(132.0);
        let chip = [ax + w - m - chip_w, ay + m, chip_w, chip_h];
        ui.rounded(chip[0], chip[1], chip[2], chip[3], chip_h * 0.5, theme::GLASS());
        ui.border(chip[0], chip[1], chip[2], chip[3], chip_h * 0.5, theme::GLASS_BORDER());
        // Skin head avatar (small dark glass square with initial fallback, will show skin texture when available)
        ui.rounded(chip[0] + 6.0, chip[1] + 6.0, chip_h - 12.0, chip_h - 12.0, (chip_h - 12.0) * 0.5, theme::CARD_INNER());
        let initial = name.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('V');
        draw_centered_text(ui, Face::LauncherMedium, 15.0, [chip[0] + 6.0, chip[1] + 6.0, chip_h - 12.0, chip_h - 12.0], hex(0xf2f4f8, 255), &initial.to_string());
        // Try to overlay tiny skin head if we have a custom skin texture (future: replace initial with head crop)
        // For now the slot is ready for skin_preview::draw_player_3d head crop.
        let text_x = chip[0] + chip_h + 4.0;
        let name_fit = ui.fit_text(Face::LauncherMedium, 13.5, &name, chip_w - chip_h - 16.0);
        ui.text(Face::LauncherMedium, 13.5, text_x, chip[1] + 7.0, theme::TEXT(), &name_fit);
        let badge_fit = ui.fit_text(Face::LauncherRegular, 10.5, &badge, chip_w - chip_h - 16.0);
        ui.text(Face::LauncherRegular, 10.5, text_x, chip[1] + 25.0, theme::DIM(), &badge_fit);
        self.targets.push((chip, Action::Page(Page::Account)));

        // Dashboard: Worlds/Servers removed per request — clean center-focus only

        // Black glassy home is center-focused: no floating bottom dock (navigation is far-left rail)
        let _bar_h = if compact { 54.0 } else { 60.0 };

        // ---- The hero: Play at middle-bottom, 3D player above it ----
        let play_h = if compact { 68.0 } else { 82.0 };
        // Move Play to middle-bottom as requested (arrow down in markup)
        let py0 = ay + h - m - play_h - 22.0;
        let play_w = if compact { 236.0 } else { 320.0 };
        // center between rails (left 78 + right 88) so Play doesn't overlap sidebars
        let rail_w2 = 78.0;
        let r_rail_w2 = 88.0;
        let content_w = (w - rail_w2 - r_rail_w2 - 32.0).max(play_w + 20.0);
        let px0 = ax + rail_w2 + 16.0 + (content_w - play_w)*0.5;
        let slab = [px0, py0, play_w, play_h];
        // ---- 3D player character above Play (dashboard spec) ----
        let char_w = (w * 0.26).clamp(150.0, 260.0);
        let char_h = (h * 0.40).clamp(160.0, 300.0).min((py0 - (ay + m + 56.0)).max(100.0));
        if char_h >= 100.0 {
            let char_x = ax + rail_w2 + 16.0 + (content_w - char_w)*0.5;
            let char_y = (py0 - char_h - 18.0).max(ay + m + 52.0);
            let char_rect = [char_x, char_y, char_w, char_h];
            let default_uuid = self.volt_accounts.active().map(|a| a.uuid_u128()).unwrap_or(0);
            crate::skin_preview::draw_player_3d(ui, char_rect, self.preview_yaw, self.preview_pitch, self.idle_phase, self.preview_vel_yaw, 2, 3, self.volt_accounts.active_skin_preview(), self.volt_accounts.active_cape_preview(), default_uuid);
            *self.preview_rect = Some(char_rect);
        }
        if installing {
            ui.rounded(slab[0], slab[1], slab[2], slab[3], play_h * 0.5, theme::GLASS());
            ui.border(slab[0], slab[1], slab[2], slab[3], play_h * 0.5,
                hex(0xffffff, 150));
            let index = InstallStage::ALL.iter().position(|s| *s == progress.stage).unwrap_or(0);
            let part = if progress.total > 0 { progress.done as f32 / progress.total as f32 } else { 0.0 };
            let fraction = ((index as f32 + part.clamp(0.0, 1.0)) / InstallStage::ALL.len() as f32)
                .clamp(0.0, 1.0);
            draw_centered_text(ui, Face::LauncherMedium, 10.0,
                [slab[0], py0 + 10.0, play_w, 12.0], theme::DIM(), "INSTALLING");
            draw_centered_text(ui, Face::Display, 20.0,
                [slab[0], py0 + 24.0, play_w, 24.0], theme::TEXT(),
                &format!("{:.0}%", fraction * 100.0));
            let bar = [slab[0] + 22.0, py0 + play_h - 18.0, play_w - 44.0, 4.0];
            ui.rounded(bar[0], bar[1], bar[2], bar[3], 2.0, hex(0xffffff, 40));
            ui.rounded(bar[0], bar[1], (bar[2] * fraction).max(4.0), bar[3], 2.0,
                hex(0xf2f5fa, 255));
        } else {
            let v_start = ui.batch_mut().vertices.len();
            // Black glassy: subtle dark aura, no white chrome
            let halo = 4.0 + 3.0 * shimmer;
            ui.rounded(slab[0] - halo, slab[1] - halo, slab[2] + 2.0 * halo,
                slab[3] + 2.0 * halo, (play_h + 2.0 * halo) * 0.5,
                hex(0x111114, (10.0 + 10.0 * shimmer) as u8));
            // Pure black glass body – OLED true black
            ui.rounded(slab[0], slab[1], slab[2], slab[3], play_h * 0.5, theme::CARD());
            // Black glass rim: very subtle highlight, not white chrome
            ui.border(slab[0], slab[1], slab[2], slab[3], play_h * 0.5,
                hex(0xffffff, (22.0 + 12.0 * shimmer) as u8));
            ui.border(slab[0] + 2.0, slab[1] + 2.0, slab[2] - 4.0, slab[3] - 4.0,
                (play_h - 4.0) * 0.5, hex(0xffffff, 12));
            // Subtle inner highlight – dark, not white
            ui.rounded(slab[0] + play_h * 0.28, slab[1] + 5.0,
                slab[2] - play_h * 0.56, play_h * 0.28, play_h * 0.14,
                hex(0xffffff, 10));
            let label = if installed { "PLAY" }
                else if self.error.is_some() { "RETRY" } else { "DOWNLOAD" };
            let font = if compact { 24.0 } else { 28.0 };
            draw_centered_text(ui, Face::Display, font, slab, theme::TEXT(), label);
            press_scale(ui, v_start, slab, Action::Play);
            self.targets.insert(0, (slab, Action::Play));
        }

        // Error line floating above the pill.
        if !installing && let Some(error) = self.error.as_deref() {
            let e = ui.fit_text(Face::LauncherRegular, 11.5, error, w - 2.0 * m - 40.0);
            let ew = ui.measure(Face::LauncherRegular, 11.5, &e);
            ui.text(Face::LauncherRegular, 11.5, ax + (w - ew) * 0.5, py0 - 20.0,
                theme::STATUS_BAD(), &e);
        }

        // Bottom-left cramped tabs removed — replaced by dedicated right rail per request
        // Keep only minimal quiet chips if space allows (hidden when right rail present)

        // Help: Discord moved up near top-right, below account chip (per markup)
        let fab = 44.0;
        let help = [ax + w - m - fab, ay + m + chip_h + 16.0, fab, fab];
        ui.rounded(help[0], help[1], help[2], help[3], fab * 0.5, theme::GLASS());
        ui.border(help[0], help[1], help[2], help[3], fab * 0.5, theme::GLASS_BORDER());
        draw_headset_icon(ui, help[0] + fab * 0.5, help[1] + fab * 0.5, theme::MUTED());
        self.targets.push((help, Action::Discord));

        // ---- Right rail: Worlds / Server / Screenshots / Replay (mirror of left rail) ----
        let r_rail_w = 88.0;
        let r_rail_x = ax + w - r_rail_w - 8.0;
        let r_rail_y = ay + 12.0;
        let r_rail_h = h - 24.0;
        ui.rounded(r_rail_x, r_rail_y, r_rail_w, r_rail_h, 18.0, theme::GLASS());
        ui.border(r_rail_x, r_rail_y, r_rail_w, r_rail_h, 18.0, theme::GLASS_BORDER());
        let right_nav: [(&str, HomeTab, Action); 4] = [
            ("Worlds", HomeTab::Worlds, Action::HomeTab(HomeTab::Worlds)),
            ("Servers", HomeTab::Multiplayer, Action::HomeTab(HomeTab::Multiplayer)),
            ("Shots", HomeTab::Screenshots, Action::HomeTab(HomeTab::Screenshots)),
            ("Replays", HomeTab::Replays, Action::HomeTab(HomeTab::Replays)),
        ];
        let r_item_h = 68.0;
        let r_gap = 8.0;
        let r_total = r_item_h * right_nav.len() as f32 + r_gap * (right_nav.len() as f32 - 1.0);
        let r_start = r_rail_y + (r_rail_h - r_total) * 0.5;
        for (i, (label, _tab, action)) in right_nav.iter().enumerate() {
            let y = r_start + i as f32 * (r_item_h + r_gap);
            let rect = [r_rail_x + 6.0, y, r_rail_w - 12.0, r_item_h];
            let active = self.home_tab == *_tab && self.catalog.page == Page::Play;
            if active {
                ui.rounded(rect[0], rect[1], rect[2], rect[3], 12.0, theme::HOVER());
                ui.border(rect[0], rect[1], rect[2], rect[3], 12.0, theme::BORDER());
                ui.rounded(rect[0] - 1.0, rect[1] + 14.0, 2.2, rect[3] - 28.0, 1.0, theme::TEXT());
            }
            // Icon placeholder + label centered
            draw_page_icon(ui, Page::Play, rect[0] + rect[2] * 0.5, rect[1] + 20.0, if active { theme::TEXT() } else { theme::MUTED() });
            draw_centered_text(ui, Face::LauncherMedium, 10.0, [rect[0], rect[1] + 34.0, rect[2], 18.0], if active { theme::TEXT() } else { theme::DIM() }, label);
            self.targets.push((rect, *action));
        }
    }

    /// The lobby home: the player's 3D character takes centre stage over
    /// the live world, chunky pixel-art destination slabs flush against the
    /// left edge, the icon squares and their connected chip rail flush
    /// against the right edge, and one big centred red Play slab (with
    /// playtime). The dock shell is intentionally absent here.
    fn draw_lobby_home(&mut self, ui: &mut Ui, area: [f32; 4], account_status: &str) {
        if theme::liquid() {
            return self.draw_liquid_home(ui, area, account_status);
        }
        use horizon_client::official_install::InstallStage;
        use horizon_client::ui_theme::color::hex;
        let [ax, ay, w, h] = area;
        let installed = horizon_client::official_install::ready(self.home);
        let installing = self.worker.is_some();
        let progress = *self.install.lock().unwrap();
        let compact = w < 760.0;
        let m = if compact { 10.0 } else { 16.0 };


        // ---- Brand, flush to the top-left corner ----
        ui.volt_logo(ax + 2.0, ay + m, 32.0, 32.0);
        ui.text(Face::Display, 19.0, ax + 42.0, ay + m + 1.0, theme::TEXT(), "VOLT");
        ui.text(Face::LauncherRegular, 10.5, ax + 42.0, ay + m + 22.0, theme::MUTED(),
            "Minecraft 26.3 · Native Rust");

        // ---- Quiet icon squares, top-right ----
        let sq = if compact { 38.0 } else { 42.0 };
        let squares: [(Page, Action); 5] = [
            (Page::Controls, Action::Page(Page::Controls)),
            (Page::Installed, Action::Page(Page::Installed)),
            (Page::VoltSettings, Action::Page(Page::VoltSettings)),
            (Page::Account, Action::Page(Page::Account)),
            (Page::Settings, Action::Page(Page::Settings)),
        ];
        for (i, (page, action)) in squares.iter().enumerate() {
            // Flush to the screen's right edge, where the rail below joins.
            let rect = [ax + w - (squares.len() - i) as f32 * (sq + 8.0) + 8.0, ay + m, sq, sq];
            ui.rounded(rect[0], rect[1], rect[2], rect[3], 10.0, theme::GLASS());
            ui.border(rect[0], rect[1], rect[2], rect[3], 10.0, theme::GLASS_BORDER());
            draw_page_icon(ui, *page, rect[0] + sq * 0.5, rect[1] + sq * 0.5, theme::TEXT());
            self.targets.push((rect, *action));
        }

        // ---- Bottom bar metrics (the character is sized around them) ----
        let bar_h = if compact { 64.0 } else { 76.0 };
        let bar_y = ay + h - m - bar_h;

        // ---- The star of the show: the rotatable 3D character ----
        let char_w = (w * 0.34).clamp(170.0, 360.0);
        let char_top = ay + m + sq + 4.0;
        let char_h = (bar_y - 10.0 - char_top).max(60.0);
        let char_rect = [ax + (w - char_w) * 0.5, char_top, char_w, char_h];
        let default_uuid = self.volt_accounts.active().map(|a| a.uuid_u128()).unwrap_or(0);
        crate::skin_preview::draw_player_3d(
            ui,
            char_rect,
            self.preview_yaw,
            self.preview_pitch,
            self.idle_phase,
            self.preview_vel_yaw,
            2,
            3,
            self.volt_accounts.active_skin_preview(),
            self.volt_accounts.active_cape_preview(),
            default_uuid,
        );
        *self.preview_rect = Some(char_rect);

        // ---- Pixel-art destination slabs, left column ----
        let card_w = (w * 0.24).clamp(164.0, 236.0);
        let gap = 10.0;
        let cards_top = ay + m + 50.0;
        let region = (bar_y - 10.0 - cards_top).max(120.0);
        let card_h = ((region - 2.0 * gap) / 3.0).clamp(44.0, 84.0);
        let skin_sub = self
            .volt_accounts
            .active()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| self.catalog.config.name.clone());
        let cards: [(&str, String, Page, u32, u32, Action); 3] = [
            ("MODS", "Built into Volt".to_owned(), Page::Mods,
                hex(0xd8455f, 255), hex(0xa32a44, 255), Action::Page(Page::Mods)),
            ("MY SKIN", skin_sub, Page::Account,
                hex(0x3f7bea, 255), hex(0x2a54c0, 255), Action::Page(Page::Account)),
            ("SHADERS", "Volt Visuals".to_owned(), Page::Shaders,
                hex(0x58b956, 255), hex(0x398f3f, 255), Action::Page(Page::Shaders)),
        ];
        for (i, (title, sub, page, top, bottom, action)) in cards.iter().enumerate() {
            // Flush to the screen's left edge.
            let rect = [ax, cards_top + i as f32 * (card_h + gap), card_w, card_h];
            if rect[1] + rect[3] > bar_y - 6.0 {
                break;
            }
            let v_start = ui.batch_mut().vertices.len();
            pixel_panel(ui, rect, *top, *bottom);
            let cy = rect[1] + card_h * 0.5;
            draw_page_icon(ui, *page, rect[0] + 26.0, cy, hex(0xffffff, 235));
            if card_h >= 56.0 {
                ui.text(Face::Display, 15.0, rect[0] + 46.0, cy - 17.0, hex(0xffffff, 255), title);
                let sub = ui.fit_text(Face::LauncherRegular, 10.5, sub, card_w - 60.0);
                ui.text(Face::LauncherRegular, 10.5, rect[0] + 46.0, cy + 2.0, hex(0xffffff, 190), &sub);
            } else {
                ui.text(Face::Display, 14.5, rect[0] + 46.0,
                    text_vcenter(rect[1], card_h, 14.5), hex(0xffffff, 255), title);
            }
            press_scale(ui, v_start, rect, *action);
            self.targets.push((rect, *action));
        }

        // ---- Right rail: the quieter destinations as small chips, on a
        // glass rail that runs flush to the screen edge and joins the
        // top icon row with no gap, so the two bars read as one piece.
        let chips: [(&str, u8, Action); 8] = [
            ("Worlds", 0, Action::HomeTab(HomeTab::Worlds)),
            ("Controls", 6, Action::Page(Page::Controls)),
            ("Servers", 1, Action::HomeTab(HomeTab::Multiplayer)),
            ("Replays", 2, Action::HomeTab(HomeTab::Replays)),
            ("Shots", 2, Action::HomeTab(HomeTab::Screenshots)),
            ("Logs", 3, Action::HomeTab(HomeTab::Console)),
            ("Packs", 5, Action::Page(Page::Packs)),
            ("Boost", 4, Action::OpenVoltBoost),
        ];
        let chip_w = 96.0;
        let chip_h = 27.0;
        let rail_w = chip_w + 14.0;
        let rail_x = ax + w - rail_w;
        let chip_x = rail_x + 7.0;
        let mut chip_y = ay + m + sq + 8.0;
        let shown = chips
            .iter()
            .enumerate()
            .take_while(|(i, _)| chip_y + (*i as f32) * (chip_h + 6.0) + chip_h <= bar_y - 6.0)
            .count();
        if shown > 0 {
            // The rail starts at the top icon row's baseline, so the
            // vertical bar connects straight into the horizontal one.
            let rail_top = ay + m + sq;
            let rail_h = (shown as f32) * (chip_h + 6.0) + 10.0;
            ui.rounded(rail_x, rail_top, rail_w, rail_h, 14.0, theme::GLASS());
            ui.border(rail_x, rail_top, rail_w, rail_h, 14.0, theme::GLASS_BORDER());
        }
        for (label, glyph, action) in chips.into_iter().take(shown) {
            let rect = [chip_x, chip_y, chip_w, chip_h];
            ui.rounded(rect[0], rect[1], rect[2], rect[3], chip_h * 0.5, theme::GLASS());
            ui.border(rect[0], rect[1], rect[2], rect[3], chip_h * 0.5, theme::GLASS_BORDER());
            if glyph == 5 {
                draw_page_icon(ui, Page::Packs, rect[0] + 16.0, rect[1] + chip_h * 0.5, theme::MUTED());
            } else if glyph == 6 {
                draw_page_icon(ui, Page::Controls, rect[0] + 16.0, rect[1] + chip_h * 0.5, theme::MUTED());
            } else {
                draw_tile_icon(ui, glyph, rect[0] + 16.0, rect[1] + chip_h * 0.5, theme::MUTED());
            }
            ui.text(Face::LauncherMedium, 11.5, rect[0] + 29.0,
                text_vcenter(rect[1], chip_h, 11.5), theme::TEXT(), label);
            self.targets.push((rect, action));
            chip_y += chip_h + 6.0;
        }

        // ---- Bottom centre: ONE big red Play slab, perfectly centred
        // (the profile selector and Game Files slabs are gone by request;
        // profiles stay reachable from Instances and the files page from
        // the top-right squares).
        let play_w = if compact { 210.0 } else { 260.0 };
        let px0 = ax + (w - play_w) * 0.5;
        if installing {
            let slab = [px0, bar_y, play_w, bar_h];
            pixel_panel(ui, slab, hex(0xd8455f, 255), hex(0xa32a44, 255));
            let index = InstallStage::ALL.iter().position(|s| *s == progress.stage).unwrap_or(0);
            let part = if progress.total > 0 { progress.done as f32 / progress.total as f32 } else { 0.0 };
            let fraction = ((index as f32 + part.clamp(0.0, 1.0)) / InstallStage::ALL.len() as f32)
                .clamp(0.0, 1.0);
            draw_centered_text(ui, Face::LauncherMedium, 10.0,
                [slab[0], bar_y + 8.0, play_w, 12.0], hex(0x3d0812, 220), "INSTALLING");
            draw_centered_text(ui, Face::Display, 19.0,
                [slab[0], bar_y + 20.0, play_w, 24.0], hex(0x3d0812, 255),
                &format!("{:.0}%", fraction * 100.0));
            let bar = [slab[0] + 16.0, bar_y + bar_h - 16.0, play_w - 32.0, 5.0];
            ui.rounded(bar[0], bar[1], bar[2], bar[3], 2.5, hex(0x3d0812, 130));
            ui.rounded(bar[0], bar[1], (bar[2] * fraction).max(5.0), bar[3], 2.5, hex(0xffe3e9, 255));
        } else {
            let slab = [px0, bar_y, play_w, bar_h];
            let v_start = ui.batch_mut().vertices.len();
            pixel_panel(ui, slab, hex(0xd8455f, 255), hex(0xa32a44, 255));
            let label = if installed { "Play" } else if self.error.is_some() { "Retry" } else { "Download" };
            let ink = hex(0xffffff, 255);
            let font = 20.0;
            let text_w = ui.measure(Face::Display, font, label);
            let tri = font * 0.62;
            let line_cy = bar_y + bar_h * 0.5 - 7.0;
            let start_x = px0 + (play_w - text_w - tri - 9.0) * 0.5;
            draw_play_triangle(ui, start_x, line_cy, tri, ink);
            ui.text(Face::Display, font, start_x + tri + 9.0,
                text_vcenter(line_cy - 20.0, 40.0, font), ink, label);
            let secs = self.catalog.config.play_seconds;
            let time = if secs == 0 { "0 h".to_owned() }
                else if secs < 3600 { format!("{} min", (secs / 60).max(1)) }
                else { format!("{} h", secs / 3600) };
            draw_centered_text(ui, Face::LauncherMedium, 9.5,
                [px0, bar_y + bar_h - 20.0, play_w, 12.0], hex(0xffd2da, 220), &time);
            press_scale(ui, v_start, slab, Action::Play);
            self.targets.insert(0, (slab, Action::Play));
        }

        // Error line floating above the bar.
        if !installing && let Some(error) = self.error.as_deref() {
            let e = ui.fit_text(Face::LauncherRegular, 11.5, error, w - 2.0 * m - 40.0);
            let ew = ui.measure(Face::LauncherRegular, 11.5, &e);
            ui.text(Face::LauncherRegular, 11.5, ax + (w - ew) * 0.5, bar_y - 22.0,
                theme::CRIMSON_BRIGHT(), &e);
        }

        // ---- Bottom-left: account chip ----
        let acc_max = (px0 - ax - 12.0).min(w * 0.3);
        if acc_max >= 44.0 {
            let acc_w = (ui.measure(Face::LauncherMedium, 12.0, account_status) + 48.0).min(acc_max);
            // Flush to the screen's left edge.
            let acc = [ax, ay + h - m - 40.0, acc_w, 40.0];
            ui.rounded(acc[0], acc[1], acc[2], acc[3], 20.0, theme::GLASS());
            ui.border(acc[0], acc[1], acc[2], acc[3], 20.0, theme::GLASS_BORDER());
            draw_page_icon(ui, Page::Account, acc[0] + 20.0, acc[1] + 20.0, theme::MUTED());
            if acc_w >= 72.0 {
                let fit = ui.fit_text(Face::LauncherMedium, 12.0, account_status, acc_w - 48.0);
                ui.text(Face::LauncherMedium, 12.0, acc[0] + 36.0,
                    text_vcenter(acc[1], 40.0, 12.0), theme::TEXT(), &fit);
            }
            self.targets.push((acc, Action::Account));
        }

        // ---- Bottom-right: Help → the Volt Discord ----
        let help = [ax + w - 96.0, ay + h - m - 40.0, 96.0, 40.0];
        ui.rounded(help[0], help[1], help[2], help[3], 12.0, theme::GLASS());
        ui.border(help[0], help[1], help[2], help[3], 12.0, theme::GLASS_BORDER());
        draw_headset_icon(ui, help[0] + 22.0, help[1] + 20.0, theme::TEXT());
        ui.text(Face::LauncherMedium, 12.5, help[0] + 38.0,
            text_vcenter(help[1], 40.0, 12.5), theme::TEXT(), "Help");
        self.targets.push((help, Action::Discord));
    }

    fn draw_fullscreen_viewer(&mut self, ui: &mut Ui, area: [f32; 4]) {
        use horizon_client::ui_theme::color::hex;
        let [x, y, w, h] = area;
        // True black fullscreen
        ui.rounded(x, y, w, h, 12.0, theme::CARD());
        ui.border(x, y, w, h, 12.0, theme::BORDER());
        // Back pill
        let back = [x + 12.0, y + 10.0, 84.0, 36.0];
        ui.rounded(back[0], back[1], back[2], back[3], 18.0, theme::GLASS());
        ui.border(back[0], back[1], back[2], back[3], 18.0, theme::GLASS_BORDER());
        draw_centered_text(ui, Face::LauncherMedium, 13.0, back, theme::TEXT(), "← Back");
        self.targets.push((back, Action::CloseViewer));
        // Title: show file name if screenshot viewer
        let title = if let Some(idx) = self.catalog.viewer_shot {
            self.catalog.screenshots.get(idx).cloned().unwrap_or_else(|| format!("Photo {idx}"))
        } else if let Some(idx) = self.catalog.viewer_world {
            // world viewer
            let worlds: Vec<String> = std::fs::read_dir(self.catalog.config.game_home(&self.home).join("saves"))
                .into_iter().flatten().flatten().filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().into_owned()).collect();
            worlds.get(idx).cloned().unwrap_or_else(|| format!("World {idx}"))
        } else { "Photo".to_owned() };
        ui.text(Face::LauncherMedium, 14.0, x + 110.0, y + 18.0, theme::TEXT(), &ui.fit_text(Face::LauncherMedium, 14.0, &title, w - 130.0));
        // Photo area – black glass with image placeholder
        let photo = [x + 14.0, y + 56.0, w - 28.0, h - 70.0];
        ui.rounded(photo[0], photo[1], photo[2], photo[3], 14.0, theme::GLASS_INNER());
        // Gradient placeholder to look like actual photo
        let mid_x = photo[0] + photo[2]*0.5;
        let mid_y = photo[1] + photo[3]*0.5;
        ui.rounded(mid_x - photo[2]*0.35, mid_y - 36.0, photo[2]*0.7, 72.0, 10.0, theme::HERO_GLOW_SOFT());
        draw_centered_text(ui, Face::LauncherRegular, 13.0, [photo[0], mid_y - 8.0, photo[2], 20.0], theme::MUTED(), "Fullscreen photo – pinch to zoom in game");
        draw_centered_text(ui, Face::LauncherRegular, 11.0, [photo[0], mid_y + 18.0, photo[2], 16.0], theme::DIM(), "Actual screenshot rendered from files when available");
        // Simulate image border
        ui.border(photo[0], photo[1], photo[2], photo[3], 14.0, theme::GLASS_BORDER());
    }

    fn draw_home_tab(&mut self, ui: &mut Ui, area: [f32; 4], account_status: &str, installed: bool) {
        use horizon_client::ui_theme::color::hex;
        let [x, y, w, h] = area;
        // Fullscreen viewer takes over all tabs
        if self.catalog.viewer_shot.is_some() || self.catalog.viewer_world.is_some() {
            self.draw_fullscreen_viewer(ui, area);
            return;
        }
        match self.home_tab {
            HomeTab::Overview => {
                // Separate compact cards rather than an empty full-height rectangle.
                let gap = 12.0;
                let half = (w - gap) * 0.5;
                let card_h = h.min(132.0);
                for r in [[x, y, half, card_h], [x + half + gap, y, half, card_h]] {
                    ui.rounded(r[0], r[1], r[2], r[3], 12.0, theme::GLASS());
                    ui.border(r[0], r[1], r[2], r[3], 12.0, theme::GLASS_BORDER());
                }
                ui.text(Face::LauncherMedium, 15.0, x + 14.0, y + 12.0, theme::TEXT(), "Instance");
                let state = self.error.as_deref().unwrap_or(if installed { "Game files ready · 26.3" } else { "Game files need to be downloaded" });
                let state = ui.fit_text(Face::LauncherRegular, 12.5, state, half - 28.0);
                ui.text(Face::LauncherRegular, 12.5, x + 14.0, y + 42.0, theme::MUTED(), &state);
                let account = ui.fit_text(Face::LauncherRegular, 12.5, account_status, half - 28.0);
                if card_h >= 95.0 {
                    ui.text(Face::LauncherRegular, 12.5, x + 14.0, y + 67.0, theme::MUTED(), &account);
                    self.targets.push(([x + 8.0, y + 62.0, half - 16.0, 30.0], Action::Page(Page::Account)));
                }
                let dx = x + half + gap;
                ui.text(Face::LauncherMedium, 15.0, dx + 14.0, y + 12.0, theme::TEXT(), "Native mods");
                let note = format!("{} of 50 switches on", self.catalog.extras.mod_catalog.enabled_count());
                ui.text(Face::LauncherRegular, 12.5, dx + 14.0, y + 42.0, theme::MUTED(), &note);
                if card_h >= 95.0 {
                    let note = ui.fit_text(Face::LauncherRegular, 11.5, "Some native features are still in development", half - 28.0);
                    ui.text(Face::LauncherRegular, 11.5, dx + 14.0, y + 67.0, theme::MUTED(), &note);
                }
                self.targets.push(([dx, y, half, card_h], Action::Page(Page::Mods)));
                // Quick links keep Shaders and Files one tap away now that the
                // top bar follows the reference entries.
                let link_y = y + card_h + 14.0;
                let link_h = (h - card_h - 14.0).clamp(0.0, 72.0);
                if link_h >= 48.0 {
                    for (i, (label, note, action)) in [
                        ("Shaders", "Volt Shaders & shader packs", Action::Page(Page::Shaders)),
                        ("Files", "Imported content and folders", Action::Page(Page::Installed)),
                    ].iter().enumerate() {
                        let r = [x + i as f32 * (half + gap), link_y, half, link_h];
                        ui.rounded(r[0], r[1], r[2], r[3], 12.0, theme::GLASS());
                        ui.border(r[0], r[1], r[2], r[3], 12.0, theme::GLASS_BORDER());
                        ui.text(Face::LauncherMedium, 14.0, r[0] + 14.0, r[1] + 10.0, theme::TEXT(), label);
                        let note = ui.fit_text(Face::LauncherRegular, 11.5, note, half - 28.0);
                        ui.text(Face::LauncherRegular, 11.5, r[0] + 14.0, r[1] + 34.0, theme::MUTED(), &note);
                        self.targets.push((r, *action));
                    }
                }
            }
            HomeTab::Worlds => {
                ui.text(Face::LauncherMedium, 15.0, x + 8.0, y + 8.0, theme::TEXT(), "Worlds");
                let mut worlds: Vec<String> = std::fs::read_dir(self.catalog.config.game_home(self.home).join("saves"))
                    .into_iter().flatten().flatten().filter(|entry| entry.path().is_dir())
                    .map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
                worlds.sort_unstable();
                let summary = if worlds.is_empty() { "No local world folders found".to_owned() }
                    else { format!("{} worlds · Tap a photo to fullscreen · Back to return", worlds.len()) };
                let summary = ui.fit_text(Face::LauncherRegular, 12.5, &summary, w - 16.0);
                ui.text(Face::LauncherRegular, 12.5, x + 8.0, y + 35.0, theme::MUTED(), &summary);
                if worlds.is_empty() {
                    ui.text(Face::LauncherRegular, 12.5, x+8.0, y+58.0, theme::DIM(), "Create a world inside the game – it will appear here.");
                } else {
                    let cols = if w < 600.0 { 1 } else { 2 };
                    let gap = 10.0;
                    let card_w = (w - 16.0 - gap*(cols as f32 -1.0))/cols as f32;
                    let card_h = 118.0;
                    let top = y + 60.0;
                    let bottom = y + h;
                    let total_rows = worlds.len().div_ceil(cols);
                    let content_h = total_rows as f32 * (card_h+gap);
                    let max = (content_h - (bottom-top).max(0.0)).max(0.0);
                    self.catalog.scroll = self.catalog.scroll.clamp(0.0, max);
                    *self.list_view = Some([top, bottom, max]);
                    let start = ui.batch_mut().vertices.len();
                    for (i, world) in worlds.iter().enumerate() {
                        let col = (i % cols) as f32;
                        let row = (i / cols) as f32;
                        let cx2 = x + 8.0 + col*(card_w+gap);
                        let cy2 = top + row*(card_h+gap) - self.catalog.scroll;
                        if cy2 + card_h < top || cy2 > bottom { continue; }
                        let rect = [cx2, cy2, card_w, card_h];
                        // Photo placeholder with gradient
                        ui.rounded(rect[0], rect[1], rect[2], rect[3], 12.0, theme::CARD());
                        ui.border(rect[0], rect[1], rect[2], rect[3], 12.0, theme::BORDER());
                        // Image area top 72px faux photo
                        ui.rounded(rect[0]+4.0, rect[1]+4.0, rect[2]-8.0, 68.0, 8.0, theme::GLASS_INNER());
                        // simulated terrain stripes
                        ui.rounded(rect[0]+10.0, rect[1]+14.0, rect[2]-20.0, 18.0, 6.0, theme::HERO_GLOW_SOFT());
                        ui.rounded(rect[0]+10.0, rect[1]+38.0, rect[2]-20.0, 10.0, 4.0, hex(0x5a7d3a,100));
                        let label = ui.fit_text(Face::LauncherMedium, 13.0, world, rect[2]-16.0);
                        ui.text(Face::LauncherMedium, 13.0, rect[0]+8.0, rect[1]+80.0, theme::TEXT(), &label);
                        ui.text(Face::LauncherRegular, 10.5, rect[0]+8.0, rect[1]+98.0, theme::MUTED(), "Tap photo → fullscreen");
                        self.targets.push((rect, Action::ViewWorld(i)));
                    }
                    let bounds = [ui.to_batch(x), ui.to_batch(top), ui.to_batch(x+w), ui.to_batch(bottom)];
                    ui.batch_mut().clip_since(start,bounds[0],bounds[1],bounds[2],bounds[3]);
                }
            }
            HomeTab::Multiplayer => {
                ui.text(Face::LauncherMedium, 15.0, x + 8.0, y + 8.0, theme::TEXT(), "Servers");
                ui.text(Face::LauncherRegular, 12.5, x + 8.0, y + 34.0, theme::MUTED(), "Select a saved server – photo cards, tap to connect, back returns");
                let r = [x + 8.0, y + 56.0, (w - 104.0).max(80.0).min(380.0), 38.0];
                ui.rounded(r[0], r[1], r[2], r[3], 10.0, theme::CARD_INNER());
                let value = if self.catalog.config.server.is_empty() { "Server address (optional)" } else { &self.catalog.config.server };
                let value = ui.fit_text(Face::LauncherRegular, 13.0, value, r[2] - 24.0);
                ui.text(Face::LauncherRegular, 13.0, r[0] + 12.0, text_vcenter(r[1], r[3], 13.0), theme::TEXT(), &value);
                self.targets.push((r, Action::Field("server")));
                if w >= 200.0 {
                    styled_button(ui, self.targets, [r[0] + r[2] + 8.0, r[1], 80.0, r[3]], "Save", Action::SaveServer, BtnStyle::Secondary, true);
                }
                let servers = &self.catalog.saved_servers;
                let top = y + 104.0;
                let bottom = y + h;
                let cols = if w < 520.0 { 1 } else { 2 };
                let gap = 10.0;
                let card_w = (w - 16.0 - gap*(cols as f32 -1.0))/cols as f32;
                let card_h = 84.0;
                let total_rows = servers.len().div_ceil(cols.max(1));
                let content_h = total_rows as f32 * (card_h+gap);
                let max = (content_h - (bottom-top).max(0.0)).max(0.0);
                self.catalog.scroll = self.catalog.scroll.clamp(0.0, max);
                *self.list_view = Some([top, bottom, max]);
                if servers.is_empty() && bottom > top + 20.0 {
                    ui.text(Face::LauncherRegular, 12.5, x + 8.0, top + 5.0, theme::MUTED(), "No saved servers – add one above. Photos show banner.");
                }
                let start = ui.batch_mut().vertices.len();
                for (i, server) in servers.iter().enumerate() {
                    let col = (i % cols) as f32;
                    let row = (i / cols) as f32;
                    let sx = x + 8.0 + col*(card_w+gap);
                    let sy = top + row*(card_h+gap) - self.catalog.scroll;
                    if sy + card_h < top || sy > bottom { continue; }
                    let rr = [sx, sy, card_w, card_h];
                    ui.rounded(rr[0], rr[1], rr[2], rr[3], 12.0, theme::GLASS_INNER());
                    // faux banner photo
                    ui.rounded(rr[0]+5.0, rr[1]+6.0, 42.0, 42.0, 8.0, theme::HERO_BG());
                    ui.rounded(rr[0]+10.0, rr[1]+14.0, 32.0, 18.0, 5.0, theme::CRIMSON_BADGE());
                    let title = ui.fit_text(Face::LauncherMedium, 13.0, &server.name, rr[2] - 70.0);
                    let address = ui.fit_text(Face::LauncherRegular, 11.0, &server.ip, rr[2] - 70.0);
                    ui.text(Face::LauncherMedium, 13.0, rr[0] + 56.0, rr[1] + 8.0, theme::TEXT(), &title);
                    ui.text(Face::LauncherRegular, 11.0, rr[0] + 56.0, rr[1] + 28.0, theme::MUTED(), &address);
                    ui.text(Face::LauncherRegular, 10.0, rr[0] + 56.0, rr[1] + 50.0, theme::DIM(), "Tap photo area → connect");
                    self.targets.push((rr, Action::ConnectServer(i)));
                }
                let bounds = [ui.to_batch(x), ui.to_batch(top), ui.to_batch(x+w), ui.to_batch(bottom)];
                ui.batch_mut().clip_since(start,bounds[0],bounds[1],bounds[2],bounds[3]);
            }
            HomeTab::Screenshots => {
                ui.text(Face::LauncherMedium, 15.0, x + 8.0, y + 8.0, theme::TEXT(), "Screenshots");
                styled_button(ui, self.targets, [x + w - 118.0, y + 4.0, 110.0, 30.0], "Open folder", Action::OpenCaptures, BtnStyle::CrimsonOutline, true);
                let shots = self.catalog.screenshots.clone();
                let clips = self.catalog.recordings.clone();
                let summary = if clips.is_empty() && shots.is_empty() {
                    "No captures yet. Screenshots appear as photos here.".to_owned()
                } else {
                    format!("{} recording(s) \u{2022} {} screenshot(s) \u{00b7 Tap photo \u{2192} fullscreen, Back returns", clips.len(), shots.len())
                };
                let summary = ui.fit_text(Face::LauncherRegular, 12.5, &summary, w - 16.0);
                ui.text(Face::LauncherRegular, 12.5, x + 8.0, y + 34.0, theme::MUTED(), &summary);
                let top = y + 58.0;
                let bottom = y + h;
                let cols = if w < 560.0 { 2 } else { 3 };
                let gap = 10.0;
                let card_w = (w - 16.0 - gap*(cols as f32 -1.0))/cols as f32;
                let card_h = 98.0;
                let rec_h = if clips.is_empty() { 0.0 } else { clips.len() as f32 * 40.0 + 8.0 };
                let grid_top = top + rec_h;
                let total_rows = shots.len().div_ceil(cols.max(1));
                let content_h = rec_h + total_rows as f32 * (card_h+gap);
                let max = (content_h - (bottom - grid_top + rec_h).max(0.0)).max(0.0);
                self.catalog.scroll = self.catalog.scroll.clamp(0.0, max);
                *self.list_view = Some([top, bottom, max]);
                let start = ui.batch_mut().vertices.len();
                for (i, (name, size)) in clips.iter().enumerate() {
                    let row_y = top + i as f32 * 40.0 - self.catalog.scroll;
                    if row_y + 40.0 < top || row_y > bottom { continue; }
                    let row = [x + 8.0, row_y, w - 16.0, 34.0];
                    ui.rounded(row[0], row[1], row[2], row[3], 10.0, theme::GLASS_INNER());
                    let label = ui.fit_text(Face::LauncherMedium, 12.5, name, row[2] - 150.0);
                    ui.text(Face::LauncherMedium, 12.5, row[0] + 10.0, row[1] + 3.0, theme::TEXT(), &label);
                    let mib = *size as f32 / (1024.0 * 1024.0);
                    ui.text(Face::LauncherRegular, 11.0, row[0] + 10.0, row[1] + 18.0, theme::MUTED(), &format!("Volt Recorder \u{2022} {mib:.1} MiB"));
                    let del = [row[0]+row[2]-50.0, row[1]+3.0, 42.0, 28.0];
                    styled_button(ui, self.targets, del, "Del", Action::DeleteRecording(i), BtnStyle::Secondary, true);
                }
                for (i, shot) in shots.iter().enumerate() {
                    let col = (i % cols) as f32;
                    let row = (i / cols) as f32;
                    let sx = x + 8.0 + col*(card_w+gap);
                    let sy = grid_top + row*(card_h+gap) - self.catalog.scroll;
                    if sy + card_h < top || sy > bottom { continue; }
                    let rect = [sx, sy, card_w, card_h];
                    ui.rounded(rect[0], rect[1], rect[2], rect[3], 10.0, theme::CARD());
                    ui.border(rect[0], rect[1], rect[2], rect[3], 10.0, theme::BORDER());
                    ui.rounded(rect[0]+4.0, rect[1]+4.0, rect[2]-8.0, 64.0, 8.0, theme::GLASS_INNER());
                    ui.rounded(rect[0]+12.0, rect[1]+16.0, rect[2]-24.0, 14.0, 6.0, theme::HERO_GLOW_SOFT());
                    ui.rounded(rect[0]+12.0, rect[1]+36.0, rect[2]*0.45, 8.0, 4.0, hex(0x6aa84f,120));
                    let label = ui.fit_text(Face::LauncherRegular, 10.5, shot, rect[2]-10.0);
                    ui.text(Face::LauncherRegular, 10.5, rect[0]+5.0, rect[1]+74.0, theme::MUTED(), &label);
                    self.targets.push((rect, Action::ViewScreenshot(i)));
                }
                let bounds = [ui.to_batch(x), ui.to_batch(top), ui.to_batch(x+w), ui.to_batch(bottom)];
                ui.batch_mut().clip_since(start,bounds[0],bounds[1],bounds[2],bounds[3]);
            }
            HomeTab::Replays => {
                ui.text(Face::LauncherMedium, 15.0, x + 8.0, y + 8.0, theme::TEXT(), "Replays");
                let replays = self.catalog.replays.clone();
                let summary = if replays.is_empty() {
                    "No replays yet. Record a session – thumbnail photos appear here.".to_owned()
                } else {
                    format!("{} replays \u{2022} Photo cards, tap fullscreen, Back returns", replays.len())
                };
                let summary = ui.fit_text(Face::LauncherRegular, 12.5, &summary, w - 16.0);
                ui.text(Face::LauncherRegular, 12.5, x + 8.0, y + 34.0, theme::MUTED(), &summary);
                let top = y + 58.0;
                let bottom = y + h;
                let cols = if w < 560.0 { 1 } else { 2 };
                let gap = 10.0;
                let card_w = (w - 16.0 - gap*(cols as f32 -1.0))/cols as f32;
                let card_h = 108.0;
                let total_rows = replays.len().div_ceil(cols.max(1));
                let content_h = total_rows as f32 * (card_h+gap);
                let max = (content_h - (bottom-top).max(0.0)).max(0.0);
                self.catalog.scroll = self.catalog.scroll.clamp(0.0, max);
                *self.list_view = Some([top, bottom, max]);
                let start = ui.batch_mut().vertices.len();
                for (i, replay) in replays.iter().enumerate() {
                    let col = (i % cols) as f32;
                    let row = (i / cols) as f32;
                    let sx = x + 8.0 + col*(card_w+gap);
                    let sy = top + row*(card_h+gap) - self.catalog.scroll;
                    if sy + card_h < top || sy > bottom { continue; }
                    let rr = [sx, sy, card_w, card_h];
                    ui.rounded(rr[0], rr[1], rr[2], rr[3], 12.0, theme::GLASS_INNER());
                    // thumbnail faux photo
                    ui.rounded(rr[0]+6.0, rr[1]+6.0, rr[2]-12.0, 52.0, 8.0, theme::HERO_BG());
                    ui.rounded(rr[0]+12.0, rr[1]+16.0, rr[2]-24.0, 24.0, 6.0, theme::CRIMSON_BADGE());
                    let title = if replay.server.is_empty() { replay.file_name.clone() } else { replay.server.clone() };
                    let title = ui.fit_text(Face::LauncherMedium, 12.5, &title, rr[2]-14.0);
                    ui.text(Face::LauncherMedium, 12.5, rr[0]+8.0, rr[1]+64.0, theme::TEXT(), &title);
                    let mib = replay.size as f32 / (1024.0*1024.0);
                    let sub = format!("{} \u{2022} {} \u{2022} {:.1} MiB \u{00b7} Tap photo \u{2192} fullscreen", horizon_client::replay::format_time(replay.duration_ms), replay.name, mib);
                    let sub = ui.fit_text(Face::LauncherRegular, 10.5, &sub, rr[2]-14.0);
                    ui.text(Face::LauncherRegular, 10.5, rr[0]+8.0, rr[1]+84.0, theme::MUTED(), &sub);
                    let watch = [rr[0]+rr[2]-174.0, rr[1]+64.0, 60.0, 28.0];
                    styled_button(ui, self.targets, watch, "Watch", Action::WatchReplay(i), BtnStyle::Primary, true);
                    let export = [rr[0]+rr[2]-108.0, rr[1]+64.0, 44.0, 28.0];
                    styled_button(ui, self.targets, export, "MP4", Action::ExportReplay(i), BtnStyle::Secondary, true);
                    let del = [rr[0]+rr[2]-58.0, rr[1]+64.0, 36.0, 28.0];
                    styled_button(ui, self.targets, del, "Del", Action::DeleteReplay(i), BtnStyle::Secondary, true);
                    let photo_area = [rr[0]+6.0, rr[1]+6.0, rr[2]-12.0, 52.0];
                    self.targets.push((photo_area, Action::ViewScreenshot(i % 1000))); // reuse viewer for replay thumb
                }
                let bounds = [ui.to_batch(x), ui.to_batch(top), ui.to_batch(x+w), ui.to_batch(bottom)];
                ui.batch_mut().clip_since(start,bounds[0],bounds[1],bounds[2],bounds[3]);
            }
            HomeTab::Console => {
                ui.text(Face::LauncherMedium, 15.0, x + 8.0, y + 8.0, theme::TEXT(), "Launcher diagnostics");
                let status = if self.catalog.status.is_empty() { "Tap Refresh to read the latest local diagnostic summary." } else { &self.catalog.status };
                let status = ui.fit_text(Face::LauncherRegular, 12.5, status, w - 16.0);
                ui.text(Face::LauncherRegular, 12.5, x + 8.0, y + 38.0, theme::MUTED(), &status);
                if h >= 100.0 {
                    styled_button(ui, self.targets, [x + 8.0, y + 64.0, 100.0, 32.0], "Refresh", Action::Logs, BtnStyle::Secondary, true);
                }
            }
        }
    }

    fn draw_catalog_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        page: Page,
        account_status: &str,
    ) {
        let [cx, cy, cw, ch] = area;
        let kind = match page {
            Page::Mods => horizon_client::content_files::Kind::Mods,
            Page::Shaders => horizon_client::content_files::Kind::Shaders,
            _ => horizon_client::content_files::Kind::Packs,
        };
        let title = match page {
            Page::Mods => "Horizon Mods",
            Page::Shaders => "Shader Packs",
            _ => "Resource Packs",
        };
        use crate::shader_catalog::Origin;
        use horizon_client::ui_theme::color::hex;
        let hint = match page {
            Page::Mods if self.catalog.config.mod_source != Origin::Horizon => {
                "Java mods (Modrinth / CurseForge) are browse-only; native .hmod mods run on Android"
            }
            Page::Mods => "Search Horizon mods or paste an HTTPS .hmod link",
            Page::Shaders if self.catalog.config.shader_source != Origin::Horizon => {
                "Modrinth & CurseForge shader ZIPs / experimental compatibility"
            }
            Page::Shaders => "Horizon catalog / native .hshader & experimental ZIP support",
            _ => "Modrinth & CurseForge verified resource packs / enable in Installed",
        };
        let _origins: &[Origin] = if page == Page::Packs {
            &[Origin::Modrinth, Origin::CurseForge, Origin::Both]
        } else {
            &[
                Origin::Horizon,
                Origin::Modrinth,
                Origin::CurseForge,
                Origin::Both,
            ]
        };
        let _active_origin = match page {
            Page::Mods => self.catalog.config.mod_source,
            Page::Shaders => self.catalog.config.shader_source,
            _ => self.catalog.config.pack_source,
        };
        let _origin_action = |o: Origin| match page {
            Page::Mods => Action::ModSource(o),
            Page::Shaders => Action::ShaderSource(o),
            _ => Action::PackSource(o),
        };

        // Detail page: About/Screenshots/Download with Back
        if let Some(idx) = self.catalog.detail_index {
            if let Some(proj) = self.catalog.results.get(idx).cloned() {
                let back = [cx, cy, 88.0, 34.0];
                ui.rounded(back[0], back[1], back[2], back[3], 17.0, theme::GLASS());
                ui.border(back[0], back[1], back[2], back[3], 17.0, theme::GLASS_BORDER());
                draw_centered_text(ui, Face::LauncherMedium, 12.0, back, theme::TEXT(), "\u2190 Back");
                self.targets.push((back, Action::CloseDetail));
                ui.text(Face::LauncherMedium, 22.0, cx+100.0, cy+4.0, theme::TEXT(), &ui.fit_text(Face::LauncherMedium, 22.0, &proj.title, cw-120.0));
                ui.text(Face::LauncherRegular, 12.5, cx+100.0, cy+32.0, theme::MUTED(), &ui.fit_text(Face::LauncherRegular, 12.5, &proj.summary, cw-120.0));
                // Tabs: About / Screenshots / Download
                let tab_y = cy + 52.0;
                let tab_w = 108.0;
                for (i, label) in ["About","Screenshots","Download"].iter().enumerate() {
                    let r = [cx + i as f32*(tab_w+8.0), tab_y, tab_w, 32.0];
                    let active = self.catalog.detail_tab == i as u8;
                    if active {
                        ui.rounded(r[0], r[1], r[2], r[3], 16.0, theme::HOVER());
                        ui.border(r[0], r[1], r[2], r[3], 16.0, theme::BORDER());
                    } else {
                        ui.rounded(r[0], r[1], r[2], r[3], 16.0, theme::GLASS());
                        ui.border(r[0], r[1], r[2], r[3], 16.0, theme::GLASS_BORDER());
                    }
                    draw_centered_text(ui, Face::LauncherMedium, 12.0, r, if active{theme::TEXT()}else{theme::MUTED()}, label);
                    self.targets.push((r, Action::DetailTab(i as u8)));
                }
                let body = [cx, tab_y+44.0, cw, ch - (tab_y+44.0 - cy) - 48.0];
                // Error banner for Horizon failures
                if let Some((src,id)) = &self.catalog.failed.clone() {
                    if *src==proj.source && *id==proj.id {
                        let msg = if self.catalog.status.is_empty() { "Download failed – check connection or try again.".to_owned() } else { self.catalog.status.clone() };
                        ui.rounded(body[0], body[1], body[2], 36.0, 10.0, theme::CARD());
                        ui.border(body[0], body[1], body[2], 36.0, 10.0, hex(0xff4444, 80));
                        ui.text(Face::LauncherMedium, 12.0, body[0]+12.0, body[1]+10.0, hex(0xff6b6b,255), &ui.fit_text(Face::LauncherMedium, 12.0, &msg, body[2]-24.0));
                    }
                }
                if self.catalog.detail_tab==0 {
                    ui.rounded(body[0], body[1]+ (if self.catalog.failed.is_some(){40.0}else{0.0}), body[2], 140.0, 12.0, theme::CARD());
                    ui.border(body[0], body[1]+ (if self.catalog.failed.is_some(){40.0}else{0.0}), body[2], 140.0, 12.0, theme::BORDER());
                    ui.text(Face::LauncherMedium, 14.0, body[0]+14.0, body[1]+ (if self.catalog.failed.is_some(){50.0}else{10.0}), theme::TEXT(), &proj.title);
                    ui.text(Face::LauncherRegular, 12.5, body[0]+14.0, body[1]+ (if self.catalog.failed.is_some(){72.0}else{32.0}), theme::MUTED(), &format!("by {} · {} downloads · {}", proj.author, proj.downloads, proj.source.label()));
                    let desc = if proj.summary.is_empty() { "No description available." } else { &proj.summary };
                    for (li, line) in wrap_lines(ui, Face::LauncherRegular, 12.0, desc, body[2]-28.0, 4).iter().enumerate() {
                        ui.text(Face::LauncherRegular, 12.0, body[0]+14.0, body[1]+ (if self.catalog.failed.is_some(){96.0}else{56.0}) + li as f32*15.0, theme::MUTED(), line);
                    }
                    // View project page external
                    styled_button(ui, self.targets, [body[0]+14.0, body[1]+ (if self.catalog.failed.is_some(){160.0}else{120.0}), 140.0, 34.0], "Open website", Action::OpenProject(idx), BtnStyle::Secondary, true);
                } else if self.catalog.detail_tab==1 {
                    // Screenshots faux photos
                    let cols = if cw < 500.0 {2} else {3};
                    let gap=10.0;
                    let cw2=(body[2]- gap*(cols as f32 -1.0))/cols as f32;
                    for i in 0..6 {
                        let col=(i%cols) as f32;
                        let row=(i/cols) as f32;
                        let r=[body[0]+col*(cw2+gap), body[1]+ (if self.catalog.failed.is_some(){40.0}else{0.0}) + row*88.0, cw2, 78.0];
                        ui.rounded(r[0], r[1], r[2], r[3], 10.0, theme::GLASS_INNER());
                        ui.rounded(r[0]+6.0, r[1]+6.0, r[2]-12.0, 46.0, 8.0, theme::HERO_GLOW_SOFT());
                        draw_centered_text(ui, Face::LauncherRegular, 10.0, [r[0], r[1]+52.0, r[2], 14.0], theme::DIM(), &format!("Screenshot {}", i+1));
                        self.targets.push((r, Action::ViewScreenshot(i)));
                    }
                } else {
                    // Download tab
                    let btn = [body[0]+14.0, body[1]+ (if self.catalog.failed.is_some(){40.0}else{0.0}) + 12.0, 200.0, 44.0];
                    let downloading = self.catalog.downloading.is_some_and(|(s,id)| s==proj.source && id==proj.id);
                    if downloading {
                        ui.rounded(btn[0], btn[1], btn[2], btn[3], 10.0, theme::GLASS());
                        draw_centered_text(ui, Face::LauncherMedium, 13.0, btn, theme::MUTED(), "Downloading…");
                    } else if self.catalog.failed.is_some_and(|(s,id)| s==proj.source && id==proj.id) {
                        styled_button(ui, self.targets, btn, "Retry download", Action::Install(idx), BtnStyle::Primary, true);
                        // Show error explicitly
                        let err = if self.catalog.status.is_empty() { "Horizon download failed – no downloadable ZIP or checksum error." } else { &self.catalog.status };
                        ui.text(Face::LauncherRegular, 11.5, body[0]+14.0, btn[1]+54.0, hex(0xff6b6b,255), &ui.fit_text(Face::LauncherRegular, 11.5, err, body[2]-28.0));
                    } else {
                        styled_button(ui, self.targets, btn, "Download & Install", Action::Install(idx), BtnStyle::Primary, true);
                        ui.text(Face::LauncherRegular, 11.5, body[0]+14.0, btn[1]+54.0, theme::MUTED(), "Will appear in Installed after download.");
                    }
                }
                // Footer back note
                ui.rounded(cx, cy+ch-44.0, cw, 44.0, 11.0, theme::CARD());
                ui.border(cx, cy+ch-44.0, cw, 44.0, 11.0, theme::BORDER());
                draw_centered_text(ui, Face::LauncherRegular, 12.0, [cx, cy+ch-44.0, cw, 44.0], theme::MUTED(), "Tip: Back returns to list · About shows info, Screenshots shows photos, Download installs");
                return;
            } else {
                self.catalog.detail_index=None;
            }
        }

        // Black glassy: clean header — title + hint only, no origin loader pills (Forge/Fabric/All/Installed/Popular removed)
        if !compact {
            ui.text(Face::LauncherMedium, 24.0, cx, cy, theme::TEXT(), title);
            let acc_w = 186.0;
            let hint_max = (cw - acc_w - 32.0).max(120.0);
            let fitted_hint = ui.fit_text(Face::LauncherRegular, 13.0, hint, hint_max);
            ui.text(Face::LauncherRegular, 13.0, cx, cy + 30.0, theme::MUTED(), &fitted_hint);
            draw_account_chip(ui, self.targets, [cx + cw - acc_w, cy + 4.0, acc_w, 38.0], account_status, self.login.signed_in);
            // origin selector removed per request
        } else {
            ui.text(Face::LauncherMedium, 18.0, cx, cy, theme::TEXT(), title);
            // compact origin selector removed — black glassy clean header
        }

        // Unified Command Toolbar Card (Search + Search CTA + Open folder + Import file)
        let bar_y = cy + if compact { 66.0 } else { 54.0 };
        let bar_h = if compact { 88.0 } else { 52.0 };
        ui.rounded(cx, bar_y, cw, bar_h, 12.0, theme::CARD());
        ui.border(cx, bar_y, cw, bar_h, 12.0, theme::BORDER());

        let search_btn_label = if page == Page::Mods && self.catalog.query.trim().starts_with("https://") {
            "Import .hmod"
        } else {
            "Search"
        };
        let placeholder = match page {
            Page::Mods => "Search Horizon mods or paste .hmod URL...",
            Page::Shaders => "Search shader packs...",
            _ => "Search resource packs...",
        };
        let display_query = if self.catalog.query.is_empty() && self.catalog.editing != Some("search") {
            placeholder.to_owned()
        } else {
            self.catalog.query.clone()
        };
        let query_is_placeholder =
            self.catalog.query.is_empty() && self.catalog.editing != Some("search");

        if compact {
            let s_w = cw - 122.0;
            let srect = [cx + 8.0, bar_y + 8.0, s_w, 34.0];
            ui.rounded(srect[0], srect[1], srect[2], srect[3], 8.0, theme::CARD_INNER());
            ui.border(
                srect[0],
                srect[1],
                srect[2],
                srect[3],
                8.0,
                if self.catalog.editing == Some("search") {
                    theme::CRIMSON()
                } else {
                    theme::BORDER()
                },
            );
            let qfit = ui.fit_text(Face::LauncherRegular, 13.5, &display_query, s_w - 24.0);
            ui.text(
                Face::LauncherRegular,
                13.5,
                srect[0] + 12.0,
                text_vcenter(srect[1], srect[3], 13.5),
                if query_is_placeholder {
                    theme::DIM()
                } else {
                    theme::TEXT()
                },
                &qfit,
            );
            self.targets.push((srect, Action::Field("search")));

            styled_button(
                ui,
                self.targets,
                [cx + cw - 106.0, bar_y + 8.0, 98.0, 34.0],
                search_btn_label,
                Action::Search,
                BtnStyle::Primary,
                true,
            );
            let half = (cw - 24.0) * 0.5;
            styled_button(
                ui,
                self.targets,
                [cx + 8.0, bar_y + 48.0, half, 32.0],
                "Open folder",
                Action::OpenFolder(kind),
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [cx + 16.0 + half, bar_y + 48.0, half, 32.0],
                "Import file",
                Action::Import(kind),
                BtnStyle::CrimsonOutline,
                true,
            );
        } else {
            let btn_folder_w = 124.0;
            let btn_import_w = 124.0;
            let btn_search_w = 118.0;
            let input_w = (cw - btn_folder_w - btn_import_w - btn_search_w - 44.0).max(160.0);
            let srect = [cx + 8.0, bar_y + 7.0, input_w, 38.0];
            ui.rounded(srect[0], srect[1], srect[2], srect[3], 9.0, theme::CARD_INNER());
            ui.border(
                srect[0],
                srect[1],
                srect[2],
                srect[3],
                9.0,
                if self.catalog.editing == Some("search") {
                    theme::CRIMSON()
                } else {
                    theme::BORDER()
                },
            );
            let qfit = ui.fit_text(Face::LauncherRegular, 14.0, &display_query, input_w - 28.0);
            ui.text(
                Face::LauncherRegular,
                14.0,
                srect[0] + 14.0,
                text_vcenter(srect[1], srect[3], 14.0),
                if query_is_placeholder {
                    theme::DIM()
                } else {
                    theme::TEXT()
                },
                &qfit,
            );
            self.targets.push((srect, Action::Field("search")));

            let mut bx = srect[0] + input_w + 8.0;
            styled_button(
                ui,
                self.targets,
                [bx, bar_y + 7.0, btn_search_w, 38.0],
                search_btn_label,
                Action::Search,
                BtnStyle::Primary,
                true,
            );
            bx += btn_search_w + 8.0;
            styled_button(
                ui,
                self.targets,
                [bx, bar_y + 7.0, btn_folder_w, 38.0],
                "Open folder",
                Action::OpenFolder(kind),
                BtnStyle::Secondary,
                true,
            );
            bx += btn_folder_w + 8.0;
            styled_button(
                ui,
                self.targets,
                [bx, bar_y + 7.0, btn_import_w, 38.0],
                "Import file",
                Action::Import(kind),
                BtnStyle::CrimsonOutline,
                true,
            );
        }

        // Catalog viewport + bottom pagination bar
        let footer_h = 44.0;
        let top = bar_y + bar_h + 12.0;
        let bottom = cy + ch - footer_h - 10.0;
        let view_h = (bottom - top).max(80.0);

        if self.catalog.results.is_empty() {
            // Atmospheric Empty State Card (matching 04-shaders.png)
            ui.rounded(cx, top, cw, view_h, 14.0, theme::CARD());
            ui.border(cx, top, cw, view_h, 14.0, theme::BORDER());

            let mid_x = cx + cw * 0.5;
            let mid_y = top + view_h * 0.5;

            // Layered crimson diamond/stack emblem
            ui.rounded(mid_x - 36.0, mid_y - 68.0, 72.0, 48.0, 12.0, theme::HERO_GLOW_SOFT());
            for layer in 0..3 {
                let ly = mid_y - 64.0 + layer as f32 * 11.0;
                ui.border(mid_x - 24.0, ly, 48.0, 16.0, 4.0, theme::CRIMSON_BRIGHT());
            }

            let empty_title = if page == Page::Shaders
                && self.catalog.config.shader_source == crate::shader_catalog::Origin::Horizon
            {
                "No native shader packs published yet"
            } else if !self.catalog.status.is_empty() {
                &self.catalog.status
            } else {
                "No packages listed yet"
            };
            draw_centered_text(
                ui,
                Face::LauncherMedium,
                18.0,
                [cx + 20.0, mid_y - 8.0, cw - 40.0, 26.0],
                theme::TEXT(),
                empty_title,
            );
            let empty_sub = match page {
                Page::Shaders => "Import a .hshader or experimental shaders ZIP to get started.",
                Page::Mods => "Paste an HTTPS .hmod URL above or import a local .hmod package.",
                _ => "Search above or import a resource pack ZIP from your device.",
            };
            draw_centered_text(
                ui,
                Face::LauncherRegular,
                13.5,
                [cx + 20.0, mid_y + 20.0, cw - 40.0, 22.0],
                theme::MUTED(),
                empty_sub,
            );

            let import_cta = match page {
                Page::Shaders => "Import .hshader / ZIP",
                Page::Mods => "Import .hmod",
                _ => "Import pack ZIP",
            };
            if mid_y + 52.0 + 40.0 <= bottom - 6.0 {
                styled_button(
                    ui,
                    self.targets,
                    [mid_x - 104.0, mid_y + 52.0, 208.0, 40.0],
                    import_cta,
                    Action::Import(kind),
                    BtnStyle::Primary,
                    true,
                );
            }
        } else {
            let cols = if compact {
                1usize
            } else if page == Page::Packs && cw >= 860.0 {
                3usize
            } else {
                2usize
            };
            let gap = 12.0;
            let card_w = (cw - (cols.saturating_sub(1) as f32) * gap) / cols as f32;
            let card_h = if compact {
                122.0
            } else if cols == 3 {
                168.0
            } else {
                156.0
            };
            let row_step = card_h + gap;
            let total_rows = self.catalog.results.len().div_ceil(cols);
            let content_h = (total_rows as f32 * row_step - gap).max(0.0);
            let max_scroll = (content_h - view_h).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([top, bottom, max_scroll]);

            // One connected, continuously scrolling grid: when the scroll
            // nears the bottom and the last fetch was a full page, the next
            // page is appended in place (no Next-button page flip needed).
            // (When a full page fits the view outright, this also fires
            // immediately, so tall screens fill up without any scrolling.)
            if self.catalog.scroll >= max_scroll - row_step && self.catalog.can_append() {
                self.catalog.append_next_page(self.home);
            }

            let grid_v_start = ui.batch_mut().vertices.len();
            let results_age = self.catalog.results_born.elapsed().as_secs_f32();
            for (i, p) in self.catalog.results.iter().enumerate() {
                let row = i / cols;
                let col_idx = i % cols;
                let x = cx + col_idx as f32 * (card_w + gap);
                let y = top + row as f32 * row_step - self.catalog.scroll;

                let download = self.catalog.download_label(p);
                let is_java_mod = p.kind == crate::browser::Kind::Mod
                    && matches!(
                        p.source,
                        crate::browser::SourceId::Modrinth | crate::browser::SourceId::CurseForge
                    );
                let can_install = download.is_none()
                    && !crate::catalog::source_only_mod(p)
                    && !is_java_mod
                    && !(p.kind == crate::browser::Kind::Mod && !cfg!(target_pointer_width = "64"));

                let btn_w = if cols == 3 { 98.0 } else { 116.0 };
                let btn_h = 34.0;
                let btn_rect = if cols == 3 {
                    [x + card_w - btn_w - 14.0, y + 16.0, btn_w, btn_h]
                } else {
                    [x + card_w - btn_w - 14.0, y + card_h - btn_h - 12.0, btn_w, btn_h]
                };

                if can_install {
                    self.list_targets.push((btn_rect, Action::Install(i)));
                }
                // Draw every card that touches the view; the clip below
                // trims the straddling ones, so scrolling never pops a
                // whole row in or out at the view edges.
                if y + card_h < top || y > bottom {
                    continue;
                }

                let card_v_start = ui.batch_mut().vertices.len();
                let icon_slot = 4 + (i % 24);
                let project_icon = self.catalog.project_icon(p);

                ui.rounded(x, y, card_w, card_h, 12.0, theme::CARD());
                ui.border(x, y, card_w, card_h, 12.0, theme::BORDER_CRIMSON());

                if cols == 3 {
                    // 3-Column Resource Pack Card layout with real project logo + source pill!
                    let icon_sz = 46.0;
                    ui.rounded(x + 14.0, y + 13.0, icon_sz, icon_sz, 10.0, theme::HERO_BG());
                    let mut drew_logo = false;
                    if let Some(ic) = project_icon {
                        let fade = (ic.loaded_at.elapsed().as_secs_f32() / 0.22).clamp(0.0, 1.0);
                        let alpha = (ease_out_cubic(fade) * 255.0).round() as u8;
                        drew_logo = ui.draw_dynamic_rgba(
                            icon_slot,
                            ic.hash,
                            64,
                            64,
                            &ic.rgba,
                            x + 15.0,
                            y + 14.0,
                            icon_sz - 2.0,
                            icon_sz - 2.0,
                            alpha,
                        );
                    }
                    if !drew_logo {
                        let pulse = (self.idle_phase * 4.2 + i as f32 * 0.7).sin() * 0.5 + 0.5;
                        ui.rounded(
                            x + 14.0,
                            y + 13.0,
                            icon_sz,
                            icon_sz,
                            10.0,
                            horizon_client::ui_theme::color::lerp(
                                theme::CRIMSON_BADGE(),
                                theme::HERO_BG(),
                                pulse,
                            ),
                        );
                        draw_page_icon(
                            ui,
                            page,
                            x + 14.0 + icon_sz * 0.5,
                            y + 13.0 + icon_sz * 0.5,
                            theme::CRIMSON_BRIGHT(),
                        );
                    }
                    ui.border(
                        x + 14.0,
                        y + 13.0,
                        icon_sz,
                        icon_sz,
                        10.0,
                        theme::BORDER_CRIMSON(),
                    );

                    // Platform badge next to icon so Both mode clearly distinguishes Modrinth vs CurseForge
                    let src_tag = p.source.label();
                    let sw = ui.measure(Face::LauncherMedium, 10.5, src_tag) + 12.0;
                    if x + 66.0 + sw < btn_rect[0] - 6.0 {
                        ui.rounded(x + 66.0, y + 25.0, sw, 20.0, 6.0, theme::CRIMSON_BADGE());
                        draw_centered_text(
                            ui,
                            Face::LauncherMedium,
                            10.5,
                            [x + 66.0, y + 25.0, sw, 20.0],
                            theme::CRIMSON_BRIGHT(),
                            src_tag,
                        );
                    }

                    let title_fit = ui.fit_text(Face::LauncherMedium, 16.0, &p.title, card_w - 28.0);
                    ui.text(
                        Face::LauncherMedium,
                        16.0,
                        x + 14.0,
                        y + 66.0,
                        theme::CRIMSON_BRIGHT(),
                        &title_fit,
                    );
                    let desc_lines =
                        wrap_lines(ui, Face::LauncherRegular, 12.5, &p.summary, card_w - 28.0, 3);
                    for (li, line) in desc_lines.iter().enumerate() {
                        ui.text(
                            Face::LauncherRegular,
                            12.5,
                            x + 14.0,
                            y + 92.0 + li as f32 * 18.0,
                            theme::MUTED(),
                            line,
                        );
                    }
                } else {
                    // 2-Column Rich Mod / Shader / Pack Card layout with real project logo!
                    let icon_sz = if compact { 44.0 } else { 56.0 };
                    ui.rounded(x + 14.0, y + 14.0, icon_sz, icon_sz, 11.0, theme::HERO_BG());
                    let mut drew_logo = false;
                    if let Some(ic) = project_icon {
                        let fade = (ic.loaded_at.elapsed().as_secs_f32() / 0.22).clamp(0.0, 1.0);
                        let alpha = (ease_out_cubic(fade) * 255.0).round() as u8;
                        drew_logo = ui.draw_dynamic_rgba(
                            icon_slot,
                            ic.hash,
                            64,
                            64,
                            &ic.rgba,
                            x + 15.0,
                            y + 15.0,
                            icon_sz - 2.0,
                            icon_sz - 2.0,
                            alpha,
                        );
                    }
                    if !drew_logo {
                        let pulse = (self.idle_phase * 4.2 + i as f32 * 0.7).sin() * 0.5 + 0.5;
                        ui.rounded(
                            x + 14.0,
                            y + 14.0,
                            icon_sz,
                            icon_sz,
                            11.0,
                            horizon_client::ui_theme::color::lerp(
                                theme::CRIMSON_BADGE(),
                                theme::HERO_BG(),
                                pulse,
                            ),
                        );
                        draw_page_icon(
                            ui,
                            page,
                            x + 14.0 + icon_sz * 0.5,
                            y + 14.0 + icon_sz * 0.5,
                            theme::CRIMSON_BRIGHT(),
                        );
                    }
                    ui.border(
                        x + 14.0,
                        y + 14.0,
                        icon_sz,
                        icon_sz,
                        11.0,
                        theme::BORDER_CRIMSON(),
                    );

                    let text_x = x + 26.0 + icon_sz;
                    let dl_text = if p.downloads > 0 {
                        format!("{} dl", format_downloads(p.downloads))
                    } else {
                        String::new()
                    };
                    let dl_w = if dl_text.is_empty() {
                        0.0
                    } else {
                        ui.measure(Face::LauncherRegular, 12.0, &dl_text) + 12.0
                    };
                    let title_max = (card_w - (text_x - x) - dl_w - 14.0).max(60.0);
                    let title_fit = ui.fit_text(Face::LauncherMedium, 16.5, &p.title, title_max);
                    ui.text(Face::LauncherMedium, 16.5, text_x, y + 12.0, theme::TEXT(), &title_fit);
                    if !dl_text.is_empty() {
                        ui.text(
                            Face::LauncherRegular,
                            12.0,
                            x + card_w - dl_w - 2.0,
                            y + 15.0,
                            theme::MUTED(),
                            &dl_text,
                        );
                    }

                    // Tag pills (source + loader + version)
                    if !compact {
                        let mut tx = text_x;
                        let src_tag = p.source.label();
                        let sw = ui.measure(Face::LauncherMedium, 11.0, src_tag) + 14.0;
                        ui.rounded(tx, y + 38.0, sw, 20.0, 6.0, theme::CRIMSON_BADGE());
                        draw_centered_text(
                            ui,
                            Face::LauncherMedium,
                            11.0,
                            [tx, y + 38.0, sw, 20.0],
                            theme::CRIMSON_BRIGHT(),
                            src_tag,
                        );
                        tx += sw + 6.0;

                        for tag in p
                            .loaders
                            .iter()
                            .take(1)
                            .chain(p.game_versions.iter().take(2))
                        {
                            let tw = ui.measure(Face::LauncherRegular, 11.0, tag) + 14.0;
                            if tx + tw > x + card_w - 14.0 {
                                break;
                            }
                            ui.rounded(tx, y + 38.0, tw, 20.0, 6.0, theme::CARD_INNER());
                            draw_centered_text(
                                ui,
                                Face::LauncherRegular,
                                11.0,
                                [tx, y + 38.0, tw, 20.0],
                                theme::MUTED(),
                                tag,
                            );
                            tx += tw + 6.0;
                        }
                    }

                    let desc_y = if compact { y + 38.0 } else { y + 64.0 };
                    let desc_w = (card_w - (text_x - x) - 14.0).max(80.0);
                    let lines = wrap_lines(ui, Face::LauncherRegular, 12.5, &p.summary, desc_w, 2);
                    for (li, line) in lines.iter().enumerate() {
                        ui.text(
                            Face::LauncherRegular,
                            12.5,
                            text_x,
                            desc_y + li as f32 * 17.0,
                            theme::MUTED(),
                            line,
                        );
                    }

                    // Card tap opens own detail page (About/Screenshots/Download) per request
                    let card_rect = [x, y, card_w, card_h];
                    self.targets.push((card_rect, Action::ViewDetail(i)));
                    // Widened "View details" button now opens detail, not external
                    if !compact
                        && (p.page.starts_with("https://") || p.page.starts_with("http://"))
                        && y + card_h - btn_h - 12.0 >= top
                        && y + card_h - 12.0 <= bottom
                    {
                        styled_button(
                            ui,
                            self.targets,
                            [x + 14.0, y + card_h - btn_h - 12.0, 128.0, btn_h],
                            "View details",
                            Action::ViewDetail(i),
                            BtnStyle::Secondary,
                            true,
                        );
                    }
                }

                // Install button or status pill
                if btn_rect[1] >= top && btn_rect[1] + btn_rect[3] <= bottom {
                    if let Some(state) = download {
                        ui.rounded(
                            btn_rect[0],
                            btn_rect[1],
                            btn_rect[2],
                            btn_rect[3],
                            8.0,
                            theme::CARD_INNER(),
                        );
                        ui.border(
                            btn_rect[0],
                            btn_rect[1],
                            btn_rect[2],
                            btn_rect[3],
                            8.0,
                            theme::CRIMSON(),
                        );
                        draw_centered_text(
                            ui,
                            Face::LauncherMedium,
                            13.0,
                            btn_rect,
                            theme::TEXT(),
                            &state,
                        );
                    } else if is_java_mod {
                        draw_centered_text(
                            ui,
                            Face::LauncherRegular,
                            12.5,
                            btn_rect,
                            theme::MUTED(),
                            "Java mod",
                        );
                    } else if crate::catalog::source_only_mod(p) {
                        draw_centered_text(
                            ui,
                            Face::LauncherRegular,
                            12.5,
                            btn_rect,
                            theme::MUTED(),
                            ".hnmod: later",
                        );
                    } else if p.kind == crate::browser::Kind::Mod
                        && !cfg!(target_pointer_width = "64")
                    {
                        draw_centered_text(
                            ui,
                            Face::LauncherRegular,
                            12.0,
                            btn_rect,
                            theme::MUTED(),
                            "64-bit req.",
                        );
                    } else {
                        styled_button(
                            ui,
                            self.targets,
                            btn_rect,
                            self.catalog.install_button(p),
                            Action::Install(i),
                            BtnStyle::Primary,
                            true,
                        );
                    }
                }

                // Staggered card entrance slide + micro-scale + fade!
                let delay = (i.min(8) as f32) * 0.028;
                if results_age < delay + 0.25 {
                    let raw_t = ((results_age - delay) / 0.25).clamp(0.0, 1.0);
                    let ease = ease_out_cubic(raw_t);
                    ui.animate_vertices(
                        card_v_start,
                        0.0,
                        (1.0 - ease) * 14.0,
                        0.97 + 0.03 * ease,
                        [x + card_w * 0.5, y + card_h * 0.5],
                        ease,
                    );
                }
            }
            // Trim the straddling cards to the list view, so partially
            // scrolled rows render cut instead of vanishing wholesale.
            let bounds = [
                ui.to_batch(cx),
                ui.to_batch(top),
                ui.to_batch(cx + cw),
                ui.to_batch(bottom),
            ];
            ui.batch_mut().clip_since(grid_v_start, bounds[0], bounds[1], bounds[2], bounds[3]);
        }

        // Bottom status bar. The grid scrolls continuously and appends the
        // next page by itself, so the Previous/Next page flip buttons are
        // gone and the whole bar carries the status text.
        let fy = cy + ch - footer_h;
        ui.rounded(cx, fy, cw, footer_h, 11.0, theme::CARD());
        ui.border(cx, fy, cw, footer_h, 11.0, theme::BORDER());
        let status_text = self
            .error
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                if !self.catalog.status.is_empty() {
                    &self.catalog.status
                } else {
                    hint
                }
            });
        let fitted = ui.fit_text(Face::LauncherRegular, 13.0, status_text, (cw - 16.0).max(40.0));
        draw_centered_text(
            ui,
            Face::LauncherRegular,
            13.0,
            [cx + 8.0, fy, (cw - 16.0).max(40.0), footer_h],
            if self.error.is_some() {
                theme::CRIMSON_BRIGHT()
            } else {
                theme::MUTED()
            },
            &fitted,
        );
    }

    /// The Mods tab: every mod in this APK as one searchable, sortable
    /// list with master switches. The launcher is the source of truth: a
    /// mod turned off here neither runs nor appears in-game.
    fn draw_mods_landing(&mut self, ui: &mut Ui, area: [f32; 4], account_status: &str) {
        let _ = account_status;
        let [cx, cy, cw, ch] = area;
        ui.text(Face::LauncherMedium, 18.0, cx, cy, theme::TEXT(), "Mods");

        // Toolbar: search field + All on / All off / Sort.
        let bar_y = cy + 28.0;
        let bar_h = 40.0;
        let btn_w = 76.0;
        let sort_w = 104.0;
        let gap = 8.0;
        let s_w = (cw - btn_w * 2.0 - sort_w - gap * 3.0).max(120.0);
        let srect = [cx, bar_y + 3.0, s_w, 34.0];
        ui.rounded(srect[0], srect[1], srect[2], srect[3], 8.0, theme::CARD_INNER());
        ui.border(
            srect[0],
            srect[1],
            srect[2],
            srect[3],
            8.0,
            if self.catalog.editing == Some("mods_search") {
                theme::CRIMSON()
            } else {
                theme::BORDER()
            },
        );
        let searching = !self.catalog.mods_query.is_empty();
        let placeholder = !searching && self.catalog.editing != Some("mods_search");
        let shown = if placeholder {
            "Search mods...".to_owned()
        } else {
            self.catalog.mods_query.clone()
        };
        let qfit = ui.fit_text(Face::LauncherRegular, 13.5, &shown, s_w - 24.0);
        ui.text(
            Face::LauncherRegular,
            13.5,
            srect[0] + 12.0,
            text_vcenter(srect[1], srect[3], 13.5),
            if placeholder { theme::DIM() } else { theme::TEXT() },
            &qfit,
        );
        self.targets.push((srect, Action::Field("mods_search")));
        let mut bx = cx + s_w + gap;
        styled_button(
            ui,
            self.targets,
            [bx, bar_y + 3.0, btn_w, 34.0],
            "All on",
            Action::ModsAllOn,
            BtnStyle::Primary,
            true,
        );
        bx += btn_w + gap;
        styled_button(
            ui,
            self.targets,
            [bx, bar_y + 3.0, btn_w, 34.0],
            "All off",
            Action::ModsAllOff,
            BtnStyle::Secondary,
            true,
        );
        bx += btn_w + gap;
        let sort_label = match self.catalog.mods_sort {
            1 => "Sort: On",
            2 => "Sort: Off",
            _ => "Sort: All",
        };
        styled_button(
            ui,
            self.targets,
            [bx, bar_y + 3.0, sort_w, 34.0],
            sort_label,
            Action::ModsSort,
            BtnStyle::CrimsonOutline,
            true,
        );

        let top = bar_y + bar_h + 8.0;
        let bottom = cy + ch;
        let view_h = (bottom - top).max(0.0);
        if view_h < 1.0 {
            return;
        }

        // One row model over the three kinds, filtered and sorted.
        #[derive(Clone, Copy)]
        enum Kind {
            Replay,
            Bundled(usize),
            Woven(usize),
        }
        let bundled = horizon_client::bundled_native::BUNDLED;
        let woven = horizon_client::bundled_native::WOVEN;
        let query = self.catalog.mods_query.trim().to_lowercase();
        let hit = |title: &str, blurb: &str| {
            query.is_empty()
                || title.to_lowercase().contains(&query)
                || blurb.to_lowercase().contains(&query)
        };
        let mut rows: Vec<(Kind, bool, String)> = Vec::new();
        if hit("Volt Replay", "Records your sessions replays timeline") {
            rows.push((
                Kind::Replay,
                self.catalog.extras.volt_qol.replay_buffer,
                "Volt Replay".to_owned(),
            ));
        }
        for (i, &(id, _, _)) in bundled.iter().enumerate() {
            let title = bundled_mod_title(id);
            if hit(title, bundled_mod_blurb(id)) {
                let installed = self.catalog.bundled_mods.contains(id);
                let on = installed && !self.catalog.bundled_disabled.contains(id);
                rows.push((Kind::Bundled(i), on, title.to_owned()));
            }
        }
        for (i, &(id, _)) in woven.iter().enumerate() {
            let title = bundled_mod_title(id);
            if hit(title, bundled_mod_blurb(id)) {
                let on = !self.catalog.bundled_disabled.contains(id);
                rows.push((Kind::Woven(i), on, title.to_owned()));
            }
        }
        match self.catalog.mods_sort {
            1 => rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.2.cmp(&b.2))),
            2 => rows.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.2.cmp(&b.2))),
            _ => {}
        }

        let row_h = 64.0;
        let step = 70.0;
        let total = rows.len() as f32 * step;
        let max_scroll = (total - view_h).max(0.0);
        self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
        *self.list_view = Some([top, bottom, max_scroll]);
        if rows.is_empty() {
            ui.text(
                Face::LauncherRegular,
                13.0,
                cx,
                top + 10.0,
                theme::DIM(),
                "No mods match this search.",
            );
            return;
        }

        for (slot, &(kind, enabled, _)) in rows.iter().enumerate() {
            let ry = top + slot as f32 * step - self.catalog.scroll;
            if ry + row_h < top || ry > bottom {
                continue;
            }
            let start = ui.batch_mut().vertices.len();
            ui.rounded(cx, ry, cw, row_h, 10.0, theme::CARD());
            ui.border(cx, ry, cw, row_h, 10.0, theme::BORDER());
            let icon = [cx + 8.0, ry + 11.0, 42.0, 42.0];
            ui.rounded(icon[0], icon[1], icon[2], icon[3], 8.0, theme::HERO_GLOW_SOFT());
            ui.border(icon[0], icon[1], icon[2], icon[3], 8.0, theme::BORDER());
            let tx = cx + 58.0;
            let text_w = (cw - 178.0).max(32.0);
            // On/off dot on the icon's corner.
            let dot = if enabled { theme::ONLINE() } else { theme::DIM() };
            ui.rounded(icon[0] + icon[2] - 11.0, icon[1] + 3.0, 8.0, 8.0, 4.0, dot);
            match kind {
                Kind::Replay => {
                    draw_centered_text(ui, Face::LauncherMedium, 18.0, icon, theme::CRIMSON(), "V");
                    let title = ui.fit_text(Face::LauncherMedium, 14.0, "Volt Replay 1.0.0", text_w);
                    ui.text(Face::LauncherMedium, 14.0, tx, ry + 10.0, theme::TEXT(), &title);
                    let state = if enabled { "Recording on" } else { "Recording off" };
                    let line = format!("Volt mod · Built into this APK · {state}");
                    let line = ui.fit_text(Face::LauncherRegular, 11.0, &line, text_w);
                    ui.text(Face::LauncherRegular, 11.0, tx, ry + 31.0, theme::DIM(), &line);
                    let blurb = ui.fit_text(
                        Face::LauncherRegular,
                        11.0,
                        "Records your sessions. Watch them from Home > Replays: timeline, speeds, free camera.",
                        text_w,
                    );
                    ui.text(Face::LauncherRegular, 11.0, tx, ry + 46.0, theme::MUTED(), &blurb);
                    if !self.catalog.busy() {
                        let button = [cx + cw - 108.0, ry + 7.0, 98.0, 24.0];
                        let first = self.targets.len();
                        styled_button(
                            ui,
                            self.targets,
                            button,
                            if enabled { "Turn off" } else { "Turn on" },
                            Action::ToggleVoltReplay,
                            if enabled { BtnStyle::Secondary } else { BtnStyle::Primary },
                            true,
                        );
                        if let Some(clipped) = super::gesture::clip_target_y(button, top, bottom) {
                            self.targets[first].0 = clipped;
                        } else {
                            self.targets.truncate(first);
                        }
                        let btn2 = [cx + cw - 108.0, ry + 35.0, 98.0, 22.0];
                        let btn_on = self.catalog.extras.volt_qol.replay_button;
                        let first = self.targets.len();
                        styled_button(
                            ui,
                            self.targets,
                            btn2,
                            if btn_on { "Button: On" } else { "Button: Off" },
                            Action::ToggleReplayButton,
                            if btn_on { BtnStyle::Primary } else { BtnStyle::Secondary },
                            true,
                        );
                        if let Some(clipped) = super::gesture::clip_target_y(btn2, top, bottom) {
                            self.targets[first].0 = clipped;
                        } else {
                            self.targets.truncate(first);
                        }
                    }
                }
                Kind::Bundled(i) => {
                    let (id, version, _) = bundled[i];
                    let initial: String = bundled_mod_title(id).chars().take(1).collect();
                    draw_centered_text(ui, Face::LauncherMedium, 18.0, icon, theme::CRIMSON(), &initial);
                    let installed = self.catalog.bundled_mods.contains(id);
                    let title = format!("{} {version}", bundled_mod_title(id));
                    let title = ui.fit_text(Face::LauncherMedium, 14.0, &title, text_w);
                    ui.text(Face::LauncherMedium, 14.0, tx, ry + 10.0, theme::TEXT(), &title);
                    let state = if installed {
                        if enabled { "Installed · On in game" } else { "Installed · Off in game" }
                    } else {
                        "In this APK · not installed"
                    };
                    let line = format!("Official Horizon mod · {state}");
                    let line = ui.fit_text(Face::LauncherRegular, 11.0, &line, text_w);
                    ui.text(Face::LauncherRegular, 11.0, tx, ry + 31.0, theme::DIM(), &line);
                    let blurb = ui.fit_text(Face::LauncherRegular, 11.0, bundled_mod_blurb(id), text_w);
                    ui.text(Face::LauncherRegular, 11.0, tx, ry + 46.0, theme::MUTED(), &blurb);
                    if !self.catalog.busy() {
                        let button = [cx + cw - 108.0, ry + 17.0, 98.0, 30.0];
                        let first = self.targets.len();
                        styled_button(
                            ui,
                            self.targets,
                            button,
                            if installed { "Remove" } else { "Install" },
                            Action::ToggleBundled(i),
                            if installed { BtnStyle::Secondary } else { BtnStyle::Primary },
                            true,
                        );
                        if let Some(clipped) = super::gesture::clip_target_y(button, top, bottom) {
                            self.targets[first].0 = clipped;
                        } else {
                            self.targets.truncate(first);
                        }
                    }
                }
                Kind::Woven(i) => {
                    let (id, version) = woven[i];
                    let initial: String = bundled_mod_title(id).chars().take(1).collect();
                    draw_centered_text(ui, Face::LauncherMedium, 18.0, icon, theme::CRIMSON(), &initial);
                    let title = format!("{} {version}", bundled_mod_title(id));
                    let title = ui.fit_text(Face::LauncherMedium, 14.0, &title, text_w);
                    ui.text(Face::LauncherMedium, 14.0, tx, ry + 10.0, theme::TEXT(), &title);
                    let state = if enabled { "On in game" } else { "Off in game" };
                    let line = format!("Official Horizon mod · Built into this APK · {state}");
                    let line = ui.fit_text(Face::LauncherRegular, 11.0, &line, text_w);
                    ui.text(Face::LauncherRegular, 11.0, tx, ry + 31.0, theme::DIM(), &line);
                    let blurb = ui.fit_text(Face::LauncherRegular, 11.0, bundled_mod_blurb(id), text_w);
                    ui.text(Face::LauncherRegular, 11.0, tx, ry + 46.0, theme::MUTED(), &blurb);
                    if !self.catalog.busy() {
                        let button = [cx + cw - 108.0, ry + 17.0, 98.0, 30.0];
                        let first = self.targets.len();
                        styled_button(
                            ui,
                            self.targets,
                            button,
                            if enabled { "Turn off" } else { "Turn on" },
                            Action::ToggleWoven(i),
                            if enabled { BtnStyle::Secondary } else { BtnStyle::Primary },
                            true,
                        );
                        if let Some(clipped) = super::gesture::clip_target_y(button, top, bottom) {
                            self.targets[first].0 = clipped;
                        } else {
                            self.targets.truncate(first);
                        }
                    }
                }
            }
            let bounds = [ui.to_batch(cx), ui.to_batch(top), ui.to_batch(cx + cw), ui.to_batch(bottom)];
            ui.batch_mut().clip_since(start, bounds[0], bounds[1], bounds[2], bounds[3]);
        }
    }

    fn draw_installed_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        account_status: &str,
    ) {
        use horizon_client::content_files::Kind;
        let [cx, cy, cw, ch] = area;

        if !compact {
            // Top bar: Installed Content + Profile pill + Account chip
            ui.text(Face::LauncherMedium, 24.0, cx, cy, theme::TEXT(), "Installed Content");
            styled_button(
                ui,
                self.targets,
                [cx + cw - 360.0, cy + 2.0, 148.0, 38.0],
                &format!("Profile {}   v", self.catalog.config.profile),
                Action::Profile,
                BtnStyle::CrimsonOutline,
                false,
            );
            draw_account_chip(
                ui,
                self.targets,
                [cx + cw - 200.0, cy + 2.0, 200.0, 38.0],
                account_status,
                self.login.signed_in,
            );

            // 3-Column Modular Cards (matching 05-installed.png)
            let top = cy + 52.0;
            let footer_h = 40.0;
            let bottom = cy + ch - footer_h - 10.0;
            let card_h = (bottom - top).max(180.0);
            let gap = 12.0;
            let col_w = (cw - 2.0 * gap) / 3.0;

            let max_items = Kind::ALL
                .iter()
                .enumerate()
                .map(|(sec, k)| {
                    if self.catalog.sections[sec] {
                        self.catalog.local.iter().filter(|e| e.kind == *k).count()
                    } else {
                        0
                    }
                })
                .max()
                .unwrap_or(0);
            let inner_top = top + 98.0;
            let inner_bottom = bottom - 12.0;
            let inner_h = (inner_bottom - inner_top).max(60.0);
            let item_h = 68.0;
            let max_scroll = (max_items as f32 * item_h - inner_h).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([inner_top, inner_bottom, max_scroll]);

            for (section, kind) in Kind::ALL.into_iter().enumerate() {
                let col_x = cx + section as f32 * (col_w + gap);
                ui.rounded(col_x, top, col_w, card_h, 13.0, theme::CARD());
                ui.border(col_x, top, col_w, card_h, 13.0, theme::BORDER_CRIMSON());

                let count = self.catalog.local.iter().filter(|e| e.kind == kind).count();
                let header_label = format!("{} ({count})", kind.label());
                let hfit = ui.fit_text(Face::LauncherMedium, 16.5, &header_label, col_w - 60.0);
                ui.text(Face::LauncherMedium, 16.5, col_x + 16.0, top + 12.0, theme::TEXT(), &hfit);

                let chev = if self.catalog.sections[section] { "v" } else { ">" };
                styled_button(
                    ui,
                    self.targets,
                    [col_x + col_w - 42.0, top + 10.0, 28.0, 28.0],
                    chev,
                    Action::ToggleSection(section),
                    BtnStyle::Subtle,
                    true,
                );
                ui.rounded(col_x + 12.0, top + 46.0, col_w - 24.0, 1.0, 0.0, theme::BORDER());

                // Import + Folder action strip
                styled_button(
                    ui,
                    self.targets,
                    [col_x + 14.0, top + 56.0, 96.0, 32.0],
                    "Import",
                    Action::Import(kind),
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    [col_x + col_w - 104.0, top + 56.0, 90.0, 32.0],
                    "Folder",
                    Action::OpenFolder(kind),
                    BtnStyle::Secondary,
                    true,
                );

                if !self.catalog.sections[section] {
                    draw_centered_text(
                        ui,
                        Face::LauncherRegular,
                        13.0,
                        [col_x + 14.0, inner_top + 20.0, col_w - 28.0, 28.0],
                        theme::MUTED(),
                        "Section collapsed — tap > to expand",
                    );
                    continue;
                }

                if count == 0 {
                    // Empty dropzone box inside column (matching 05-installed.png)
                    let dz = [col_x + 14.0, inner_top, col_w - 28.0, inner_h];
                    ui.rounded(dz[0], dz[1], dz[2], dz[3], 10.0, theme::BG());
                    ui.border(dz[0], dz[1], dz[2], dz[3], 10.0, theme::BORDER());
                    let mid_y = dz[1] + dz[3] * 0.5;
                    draw_page_icon(ui, Page::Installed, dz[0] + dz[2] * 0.5, mid_y - 26.0, theme::TEXT());
                    draw_centered_text(
                        ui,
                        Face::LauncherRegular,
                        13.5,
                        [dz[0] + 8.0, mid_y - 6.0, dz[2] - 16.0, 22.0],
                        theme::TEXT(),
                        "Tap Import to add files",
                    );
                    draw_centered_text(
                        ui,
                        Face::LauncherRegular,
                        12.5,
                        [dz[0] + 8.0, mid_y + 18.0, dz[2] - 16.0, 20.0],
                        theme::MUTED(),
                        "No items installed",
                    );
                    self.targets.push((dz, Action::Import(kind)));
                } else {
                    let mut idx_in_col = 0usize;
                    for (i, item) in self
                        .catalog
                        .local
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.kind == kind)
                    {
                        let iy = inner_top + idx_in_col as f32 * item_h - self.catalog.scroll;
                        idx_in_col += 1;
                        let toggle_rect = [col_x + 18.0, iy + 32.0, 88.0, 28.0];
                        let del_rect = [col_x + col_w - 92.0, iy + 32.0, 74.0, 28.0];
                        self.list_targets.push((toggle_rect, Action::ToggleLocal(i)));
                        self.list_targets.push((del_rect, Action::DeleteLocal(i)));
                        if iy < inner_top || iy + item_h - 4.0 > inner_bottom {
                            continue;
                        }
                        ui.rounded(
                            col_x + 12.0,
                            iy,
                            col_w - 24.0,
                            item_h - 6.0,
                            9.0,
                            theme::CARD_INNER(),
                        );
                        ui.border(
                            col_x + 12.0,
                            iy,
                            col_w - 24.0,
                            item_h - 6.0,
                            9.0,
                            theme::BORDER(),
                        );
                        let tfit = ui.fit_text(Face::LauncherMedium, 13.5, &item.title, col_w - 40.0);
                        ui.text(Face::LauncherMedium, 13.5, col_x + 20.0, iy + 6.0, theme::TEXT(), &tfit);
                        if let Some(status) = &item.status {
                            let failed = status.contains("NOT loaded")
                                || status.contains("FAILED")
                                || status.contains("unreadable");
                            let color = if failed { theme::STATUS_BAD() } else { theme::MUTED() };
                            let sfit = ui.fit_text(Face::LauncherRegular, 10.5, status, col_w - 40.0);
                            ui.text(Face::LauncherRegular, 10.5, col_x + 20.0, iy + 21.0, color, &sfit);
                        }
                        if kind == Kind::Mods && !cfg!(target_pointer_width = "64") {
                            ui.text(
                                Face::LauncherRegular,
                                12.0,
                                col_x + 20.0,
                                iy + 34.0,
                                theme::MUTED(),
                                "64-bit required",
                            );
                        } else {
                            toggle(ui, self.targets, toggle_rect, item.enabled, Action::ToggleLocal(i));
                        }
                        styled_button(
                            ui,
                            self.targets,
                            del_rect,
                            "Remove",
                            Action::DeleteLocal(i),
                            BtnStyle::Secondary,
                            true,
                        );
                    }
                }
            }

            // Bottom status bar (matching 05-installed.png)
            let fy = cy + ch - footer_h;
            ui.rounded(cx, fy, cw, footer_h, 10.0, theme::CARD());
            ui.border(cx, fy, cw, footer_h, 10.0, theme::BORDER());
            let default_msg = format!(
                "Manage enabled mods, packs and shaders for Profile {}",
                self.catalog.config.profile
            );
            let msg = self
                .error
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    if !self.catalog.status.is_empty() {
                        &self.catalog.status
                    } else {
                        &default_msg
                    }
                });
            let fitted = ui.fit_text(Face::LauncherRegular, 13.5, msg, cw - 32.0);
            ui.text(
                Face::LauncherRegular,
                13.5,
                cx + 16.0,
                text_vcenter(fy, footer_h, 13.5),
                theme::TEXT(),
                &fitted,
            );
        } else {
            // Compact portrait stacked sections
            let top = cy;
            let bottom = cy + ch;
            let row_height = 96.0;
            let total = Kind::ALL
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    48.0 + if self.catalog.sections[i] {
                        self.catalog.local.iter().filter(|e| e.kind == *k).count() as f32
                            * row_height
                    } else {
                        0.0
                    }
                })
                .sum::<f32>();
            let maximum = (total - (bottom - top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, maximum);
            *self.list_view = Some([top, bottom, maximum]);
            let mut offset = 0.0;
            for (section, kind) in Kind::ALL.into_iter().enumerate() {
                let count = self.catalog.local.iter().filter(|e| e.kind == kind).count();
                let y = top + offset - self.catalog.scroll;
                let label = format!(
                    "{} {} ({count})",
                    if self.catalog.sections[section] { "-" } else { "+" },
                    kind.label()
                );
                let sec_rect = [cx, y, cw - 110.0, 40.0];
                let imp_rect = [cx + cw - 102.0, y, 102.0, 40.0];
                self.list_targets.push((sec_rect, Action::ToggleSection(section)));
                self.list_targets.push((imp_rect, Action::Import(kind)));
                if y >= top && y + 40.0 <= bottom {
                    styled_button(
                        ui,
                        self.targets,
                        sec_rect,
                        &label,
                        Action::ToggleSection(section),
                        BtnStyle::CrimsonOutline,
                        false,
                    );
                    styled_button(
                        ui,
                        self.targets,
                        imp_rect,
                        "Import",
                        Action::Import(kind),
                        BtnStyle::Primary,
                        true,
                    );
                }
                offset += 48.0;
                if !self.catalog.sections[section] {
                    continue;
                }
                for (i, item) in self
                    .catalog
                    .local
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.kind == kind)
                {
                    let y = top + offset - self.catalog.scroll;
                    offset += row_height;
                    let tog_rect = [cx + 12.0, y + 48.0, 104.0, 34.0];
                    let del_rect = [cx + cw - 112.0, y + 48.0, 100.0, 34.0];
                    self.list_targets.push((tog_rect, Action::ToggleLocal(i)));
                    self.list_targets.push((del_rect, Action::DeleteLocal(i)));
                    if y < top || y + row_height - 6.0 > bottom {
                        continue;
                    }
                    ui.rounded(cx, y, cw, row_height - 8.0, 10.0, theme::CARD());
                    ui.border(cx, y, cw, row_height - 8.0, 10.0, theme::BORDER());
                    let tfit = ui.fit_text(Face::LauncherMedium, 15.0, &item.title, cw - 24.0);
                    ui.text(Face::LauncherMedium, 15.0, cx + 12.0, y + 10.0, theme::TEXT(), &tfit);
                    if let Some(status) = &item.status {
                        let failed = status.contains("NOT loaded")
                            || status.contains("FAILED")
                            || status.contains("unreadable");
                        let color = if failed { theme::STATUS_BAD() } else { theme::MUTED() };
                        let sfit = ui.fit_text(Face::LauncherRegular, 11.5, status, cw - 24.0);
                        ui.text(Face::LauncherRegular, 11.5, cx + 12.0, y + 30.0, color, &sfit);
                    }
                    if kind == Kind::Mods && !cfg!(target_pointer_width = "64") {
                        ui.text(
                            Face::LauncherRegular,
                            13.0,
                            cx + 12.0,
                            y + 54.0,
                            theme::MUTED(),
                            "64-bit required",
                        );
                    } else {
                        toggle(ui, self.targets, tog_rect, item.enabled, Action::ToggleLocal(i));
                    }
                    styled_button(
                        ui,
                        self.targets,
                        del_rect,
                        "Remove",
                        Action::DeleteLocal(i),
                        BtnStyle::Secondary,
                        true,
                    );
                }
            }
        }
    }

    /// The dedicated Controls tab: import touch layouts, manage saved
    /// presets by name, open the Pojav-parity visual editor and toggle the
    /// dashboard preview. The whole page scrolls as one column.
    fn draw_controls_page(&mut self, ui: &mut Ui, area: [f32; 4], compact: bool) {
        let [cx, cy, cw, ch] = area;
        let pad_x = if compact { 10.0 } else { 16.0 };
        let hero_h = 148.0;
        let row_h = 52.0;
        let head_h = 30.0;
        let presets = self.catalog.control_presets.clone();
        let total_h = hero_h + 12.0 + head_h + presets.len().max(1) as f32 * row_h + 16.0;
        let top = cy;
        let bottom = cy + ch;
        let max_scroll = (total_h - ch).max(0.0);
        self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
        *self.list_view = Some([top, bottom, max_scroll]);
        let mut y = top - self.catalog.scroll;

        // ---- Hero card: active layout + the five primary actions ----
        ui.rounded(cx, y, cw, hero_h, 14.0, theme::HERO_BG());
        ui.rounded(cx + 8.0, y + 8.0, cw * 0.44, hero_h - 16.0, 12.0, theme::HERO_GLOW_SOFT());
        ui.border(cx, y, cw, hero_h, 14.0, theme::BORDER_CRIMSON());
        ui.text(Face::LauncherMedium, 19.0, cx + pad_x + 6.0, y + 12.0, theme::TEXT(), "Controls");
        let active_line = match &self.catalog.active_controls {
            Some(m) => format!("Active: {} ({}) - {} buttons", m.name, m.format, m.buttons),
            None => "Active: Volt Default touch controls".to_owned(),
        };
        let fit_a = ui.fit_text(Face::LauncherMedium, 13.0, &active_line, cw - pad_x * 2.0 - 12.0);
        ui.text(
            Face::LauncherMedium,
            13.0,
            cx + pad_x + 6.0,
            y + 40.0,
            if self.catalog.active_controls.is_some() { theme::CRIMSON_BRIGHT() } else { theme::TEXT() },
            &fit_a,
        );
        let sub = "Import Pojav, Zalith, FCL & Claw layouts 1:1, or build your own in the editor.";
        let fit_s = ui.fit_text(Face::LauncherRegular, 12.0, sub, cw - pad_x * 2.0 - 12.0);
        ui.text(Face::LauncherRegular, 12.0, cx + pad_x + 6.0, y + 61.0, theme::MUTED(), &fit_s);

        let btn_y = y + hero_h - 62.0;
        let half_w = (cw - pad_x * 2.0 - 8.0) * 0.5;
        let hero_btns_top: [([f32; 4], &str, Action, BtnStyle); 2] = [
            (
                [cx + pad_x, btn_y, half_w, 26.0],
                "Open Control Editor",
                Action::Studio(super::controls_studio::StudioAction::Open),
                BtnStyle::Primary,
            ),
            (
                [cx + pad_x + half_w + 8.0, btn_y, half_w, 26.0],
                "Import Layout File",
                Action::ImportControl,
                BtnStyle::CrimsonOutline,
            ),
        ];
        for (rect, label, action, style) in hero_btns_top {
            self.list_targets.push((rect, action));
            styled_button(ui, self.targets, rect, label, action, style, true);
        }
        let half_w = (cw - pad_x * 2.0 - 6.0) / 2.0;
        let hero_btns_bot: [([f32; 4], &str, Action); 2] = [
            (
                [cx + pad_x, btn_y + 32.0, half_w, 24.0],
                "Apply to All Profiles",
                Action::ApplyControlsAllProfiles,
            ),
            (
                [cx + pad_x + half_w + 6.0, btn_y + 32.0, half_w, 24.0],
                "Reset to Default",
                Action::ResetControls,
            ),
        ];
        for (rect, label, action) in hero_btns_bot {
            let style = if label.ends_with("ON") { BtnStyle::CrimsonOutline } else { BtnStyle::Secondary };
            self.list_targets.push((rect, action));
            styled_button(ui, self.targets, rect, label, action, style, true);
        }
        y += hero_h + 12.0;

        // ---- Saved layouts (named) ----
        ui.text(
            Face::LauncherMedium,
            14.5,
            cx + pad_x,
            y + 4.0,
            theme::TEXT(),
            &format!("Saved layouts ({})", presets.len()),
        );
        y += head_h;
        if presets.is_empty() {
            ui.text(
                Face::LauncherRegular,
                12.5,
                cx + pad_x,
                y + 8.0,
                theme::MUTED(),
                "No saved layouts yet - import a file or save one from the editor.",
            );
            return;
        }
        let active_name = self.catalog.active_controls.as_ref().map(|m| m.name.clone());
        for (i, preset) in presets.iter().enumerate() {
            if y + row_h < top - 4.0 || y > bottom + 4.0 {
                y += row_h;
                continue;
            }
            let rr = [cx, y, cw, row_h - 8.0];
            let is_active = active_name.as_deref() == Some(preset.meta.name.as_str());
            ui.rounded(rr[0], rr[1], rr[2], rr[3], 10.0, theme::CARD());
            ui.border(
                rr[0],
                rr[1],
                rr[2],
                rr[3],
                10.0,
                if is_active { theme::BORDER_CRIMSON() } else { theme::BORDER() },
            );
            let btns_w = 148.0;
            let name_max = (cw - btns_w - pad_x * 2.0 - 12.0).max(80.0);
            let fit_n = ui.fit_text(Face::LauncherMedium, 13.5, &preset.meta.name, name_max);
            ui.text(
                Face::LauncherMedium,
                13.5,
                rr[0] + pad_x,
                rr[1] + 6.0,
                if is_active { theme::CRIMSON_BRIGHT() } else { theme::TEXT() },
                &fit_n,
            );
            let detail = format!(
                "{} - {} buttons{}{}",
                preset.meta.format,
                preset.meta.buttons,
                if preset.meta.drawers > 0 {
                    format!(" - {} drawers", preset.meta.drawers)
                } else {
                    String::new()
                },
                if preset.meta.has_joystick { " - joystick" } else { "" },
            );
            let fit_d = ui.fit_text(Face::LauncherRegular, 11.0, &detail, name_max);
            ui.text(Face::LauncherRegular, 11.0, rr[0] + pad_x, rr[1] + 24.0, theme::MUTED(), &fit_d);
            let apply_r = [rr[0] + cw - btns_w, rr[1] + 8.0, 78.0, rr[3] - 16.0];
            self.list_targets.push((apply_r, Action::ChooseControlLayout(i)));
            styled_button(
                ui,
                self.targets,
                apply_r,
                if is_active { "Active" } else { "Apply" },
                Action::ChooseControlLayout(i),
                if is_active { BtnStyle::Primary } else { BtnStyle::CrimsonOutline },
                !is_active,
            );
            let del_r = [rr[0] + cw - btns_w + 84.0, rr[1] + 8.0, 54.0, rr[3] - 16.0];
            self.list_targets.push((del_r, Action::DeleteControlPreset(i)));
            styled_button(
                ui,
                self.targets,
                del_r,
                "Delete",
                Action::DeleteControlPreset(i),
                BtnStyle::Subtle,
                true,
            );
            y += row_h;
        }
    }

    fn draw_settings_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        account_status: &str,
    ) {
        let [cx, cy, cw, ch] = area;
        let c = &self.catalog.config;
        let render_scale_val = if self.catalog.editing == Some("render_scale") {
            self.catalog.render_scale_text.clone()
        } else {
            format!("{}%", c.render_scale.percent())
        };
        let server_val = if c.server.is_empty() {
            "Singleplayer".to_owned()
        } else {
            c.server.clone()
        };

        let vulkan_driver_val = match &c.custom_vulkan_driver_name {
            Some(name) if !name.trim().is_empty() => format!("Custom: {name}"),
            _ if c.custom_vulkan_driver.is_some() => "Custom .so active".to_owned(),
            _ => "Built-in Supporter".to_owned(),
        };

        let left_items: [(&str, String, Action, Option<bool>, bool); 12] = [
            ("Theme", if c.launcher_liquid { "Liquid Glass" } else { "Classic" }.to_owned(), Action::ToggleLiquid, None, false),
            ("Appearance", if c.launcher_dark { "Dark" } else { "Light" }.to_owned(), Action::ToggleAppearance, None, false),
            ("Accent color", format!("#{:06X}", c.launcher_accent), Action::CycleAccent, None, false),
            (
                "Rendering",
                c.renderer.label().to_owned(),
                Action::Renderer,
                None,
                true,
            ),
            (
                "Vulkan driver",
                vulkan_driver_val,
                Action::ImportVulkanDriver,
                None,
                false,
            ),
            (
                "Client resolution",
                render_scale_val,
                Action::RenderScale,
                None,
                true,
            ),
            (
                "Orientation",
                c.orientation.label().to_owned(),
                Action::Orientation,
                None,
                true,
            ),
            (
                "Immersive fullscreen",
                String::new(),
                Action::Immersive,
                Some(c.screen.immersive),
                false,
            ),
            (
                "Avoid camera cutout",
                String::new(),
                Action::Cutout,
                Some(c.screen.avoid_cutout),
                false,
            ),
            (
                "Aspect ratio",
                c.screen.aspect.label().to_owned(),
                Action::Aspect,
                None,
                true,
            ),
            (
                "Display refresh rate",
                c.device.display_label(),
                Action::DisplayHz,
                None,
                true,
            ),
            (
                "Performance hints",
                String::new(),
                Action::PerformanceHints,
                Some(c.device.performance_hints),
                false,
            ),
        ];

        let right_items: [(&str, String, Action, Option<bool>, bool); 14] = [
            (
                "Profile",
                format!("Profile {}", c.profile),
                Action::Profile,
                None,
                true,
            ),
            (
                "Reset GPU driver",
                if c.custom_vulkan_driver.is_some() {
                    "Switch to Built-in".to_owned()
                } else {
                    "Using Built-in".to_owned()
                },
                Action::ResetVulkanDriver,
                None,
                false,
            ),
            (
                "Input",
                c.input_host.label().to_owned(),
                Action::InputHost,
                None,
                true,
            ),
            ("Server", server_val, Action::Field("server"), None, false),
            (
                "GPU profiling",
                String::new(),
                Action::GpuProfile,
                Some(c.device.gpu_profile),
                false,
            ),
            (
                "Performance recording",
                String::new(),
                Action::RecordPerformance,
                Some(c.record_performance),
                false,
            ),
            (
                "Diagnostic report",
                "Export ZIP".to_owned(),
                Action::ExportDiagnostics,
                None,
                false,
            ),
            (
                "Raw performance data",
                "Open Reports".to_owned(),
                Action::ExportPerformance,
                None,
                false,
            ),
            (
                "Discord webhook",
                if c.discord_webhook.is_empty() {
                    "Off".to_owned()
                } else {
                    "Configured".to_owned()
                },
                Action::Field("discord_webhook"),
                None,
                false,
            ),
            // --- About ---
            (
                "About · Website",
                "volt-launcher".to_owned(),
                Action::AboutWebsite,
                None,
                false,
            ),
            (
                "About · Discord",
                "Join the server".to_owned(),
                Action::Discord,
                None,
                false,
            ),
            (
                "About · Owner",
                "entitybrian".to_owned(),
                Action::AboutOwner,
                None,
                false,
            ),
            (
                "About · Rich Presence",
                crate::discord_rpc::status_text().to_owned(),
                Action::PresenceStatus,
                None,
                false,
            ),
            (
                "Check for updates",
                format!("v{}", crate::native_ime::app_version_name()),
                Action::CheckUpdates,
                None,
                false,
            ),
        ];

        let active_ctrl_summary = if let Some(m) = &self.catalog.active_controls {
            format!(
                "Active: {} • {} ({} buttons{})",
                m.name,
                m.format,
                m.buttons,
                if m.has_joystick { ", joystick" } else { "" }
            )
        } else {
            format!(
                "Active: Default Volt Touch Controls (Profile {})",
                c.profile
            )
        };
        let saved_label = format!("Saved Layouts ({})", self.catalog.control_presets.len());

        if !compact {
            // Dedicated "Import Control" Hero Section Card at top of Settings
            let import_h = 96.0;
            ui.rounded(cx, cy, cw, import_h, 14.0, theme::HERO_BG());
            ui.rounded(
                cx + 8.0,
                cy + 8.0,
                cw * 0.46,
                import_h - 16.0,
                12.0,
                theme::HERO_GLOW_SOFT(),
            );
            ui.border(cx, cy, cw, import_h, 14.0, theme::BORDER_CRIMSON());

            let btn_grid_w = (cw * 0.45).clamp(256.0, 344.0);
            let btn_x = cx + cw - btn_grid_w - 16.0;
            let half_btn_w = (btn_grid_w - 8.0) * 0.5;
            let text_max = (btn_x - cx - 32.0).max(140.0);

            ui.text(
                Face::LauncherMedium,
                18.0,
                cx + 20.0,
                cy + 11.0,
                theme::TEXT(),
                "Controls Studio",
            );
            if text_max >= 260.0 {
                let badge_x = cx + 162.0;
                ui.rounded(badge_x, cy + 12.0, 128.0, 22.0, 6.0, theme::CRIMSON_BADGE());
                ui.border(badge_x, cy + 12.0, 128.0, 22.0, 6.0, theme::BORDER_CRIMSON());
                draw_centered_text(
                    ui,
                    Face::LauncherMedium,
                    11.0,
                    [badge_x, cy + 12.0, 128.0, 22.0],
                    theme::CRIMSON_BRIGHT(),
                    "Pojav • Zalith • FCL",
                );
            }

            let active_fit = ui.fit_text(Face::LauncherMedium, 13.5, &active_ctrl_summary, text_max);
            ui.text(
                Face::LauncherMedium,
                13.5,
                cx + 20.0,
                cy + 41.0,
                if self.catalog.active_controls.is_some() {
                    theme::CRIMSON_BRIGHT()
                } else {
                    theme::TEXT()
                },
                &active_fit,
            );
            let sub_hint =
                "Built-in Pojav, Zalith, FCL & Claw presets + visual button editor & .json/.zip import";
            let sub_fit = ui.fit_text(Face::LauncherRegular, 12.5, sub_hint, text_max);
            ui.text(
                Face::LauncherRegular,
                12.5,
                cx + 20.0,
                cy + 65.0,
                theme::MUTED(),
                &sub_fit,
            );

            let third_btn_w = (btn_grid_w - 12.0) / 3.0;
            styled_button(
                ui,
                self.targets,
                [btn_x, cy + 13.0, half_btn_w, 34.0],
                "Controls Studio",
                Action::Studio(super::controls_studio::StudioAction::Open),
                BtnStyle::Primary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x + half_btn_w + 8.0, cy + 13.0, half_btn_w, 34.0],
                "Import File",
                Action::ImportControl,
                BtnStyle::CrimsonOutline,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x, cy + 53.0, third_btn_w, 31.0],
                &saved_label,
                Action::ControlLayouts,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x + third_btn_w + 6.0, cy + 53.0, third_btn_w, 31.0],
                "Apply All",
                Action::ApplyControlsAllProfiles,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [btn_x + (third_btn_w + 6.0) * 2.0, cy + 53.0, third_btn_w, 31.0],
                "Reset",
                Action::ResetControls,
                BtnStyle::Secondary,
                true,
            );

            // Two-Column Categorized Settings Deck below Import Control
            let gap = 16.0;
            let col_w = (cw - gap) * 0.5;
            let footer_h = 52.0;
            let cards_y = cy + import_h + 10.0;
            let cards_h = (ch - import_h - 10.0 - footer_h - 10.0).max(142.0);

            ui.rounded(cx, cards_y, col_w, cards_h, 14.0, theme::CARD());
            ui.border(cx, cards_y, col_w, cards_h, 14.0, theme::BORDER());
            ui.text(
                Face::LauncherMedium,
                17.0,
                cx + 20.0,
                cards_y + 12.0,
                theme::TEXT(),
                "Display & Graphics",
            );

            let rx = cx + col_w + gap;
            ui.rounded(rx, cards_y, col_w, cards_h, 14.0, theme::CARD());
            ui.border(rx, cards_y, col_w, cards_h, 14.0, theme::BORDER());
            ui.text(
                Face::LauncherMedium,
                17.0,
                rx + 20.0,
                cards_y + 12.0,
                theme::TEXT(),
                "Profile, Input & Diagnostics",
            );

            let list_top = cards_y + 46.0;
            let list_bottom = cards_y + cards_h - 8.0;
            let row_h = 48.0;
            let max_rows = left_items.len().max(right_items.len());
            let max_scroll = (max_rows as f32 * row_h - (list_bottom - list_top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([list_top, list_bottom, max_scroll]);

            for (slice, base_x) in [(&left_items[..], cx), (&right_items[..], rx)] {
                for (i, (label, val, act, sw, dropdown)) in slice.iter().enumerate() {
                    let y = list_top + i as f32 * row_h - self.catalog.scroll;
                    let ctrl_w = (col_w * 0.50).clamp(132.0, 236.0);
                    let ctrl_rect = [base_x + col_w - ctrl_w - 16.0, y + 5.0, ctrl_w, 34.0];
                    self.list_targets.push((ctrl_rect, *act));
                    if y < list_top - 4.0 || y + row_h - 4.0 > list_bottom {
                        continue;
                    }
                    let row_rect = [base_x + 10.0, y + 1.0, col_w - 20.0, row_h - 4.0];
                    if point_in(ui.cursor, row_rect) {
                        ui.rounded(row_rect[0], row_rect[1], row_rect[2], row_rect[3], 7.0, theme::HOVER());
                        ui.rounded(row_rect[0] + 1.0, row_rect[1] + 6.0, 2.5, row_rect[3] - 12.0, 1.2, theme::CRIMSON_BRIGHT());
                    }
                    let lbl_fit =
                        ui.fit_text(Face::LauncherRegular, 14.0, label, col_w - ctrl_w - 40.0);
                    ui.text(
                        Face::LauncherRegular,
                        14.0,
                        base_x + 20.0,
                        text_vcenter(y, row_h - 4.0, 14.0),
                        if point_in(ui.cursor, row_rect) { theme::TEXT() } else { theme::MUTED() },
                        &lbl_fit,
                    );
                    if let Some(on) = sw {
                        toggle(ui, self.targets, ctrl_rect, *on, *act);
                    } else {
                        let text = if *dropdown {
                            format!("{val}   v")
                        } else {
                            val.clone()
                        };
                        styled_button(
                            ui,
                            self.targets,
                            ctrl_rect,
                            &text,
                            *act,
                            BtnStyle::Secondary,
                            false,
                        );
                    }
                    if i + 1 < slice.len() {
                        ui.rounded(
                            base_x + 20.0,
                            y + row_h - 2.0,
                            col_w - 36.0,
                            1.0,
                            0.0,
                            theme::BORDER(),
                        );
                    }
                }
            }

            // Bottom row: status on left, Diagnostics + Microsoft/Xbox + account chip inline on right
            let btn_y = cards_y + cards_h + 10.0;
            let acc_chip_w = 152.0f32.min(col_w * 0.36);
            let btns_avail = (col_w - acc_chip_w - 16.0).max(160.0);
            let half_right = (btns_avail - 8.0) * 0.5;
            styled_button(
                ui,
                self.targets,
                [rx, btn_y, half_right, 40.0],
                "Diagnostics",
                Action::Logs,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [rx + half_right + 8.0, btn_y, half_right, 40.0],
                "Xbox / MS Login",
                Action::Account,
                BtnStyle::Primary,
                true,
            );
            draw_account_chip(
                ui,
                self.targets,
                [rx + col_w - acc_chip_w, btn_y + 3.0, acc_chip_w, 34.0],
                account_status,
                self.login.signed_in,
            );

            let status = self.error.as_deref().unwrap_or(&self.catalog.status);
            if !status.is_empty() {
                let lines = wrap_lines(ui, Face::LauncherRegular, 13.0, status, col_w, 2);
                for (li, line) in lines.iter().enumerate() {
                    ui.text(
                        Face::LauncherRegular,
                        13.0,
                        cx + 4.0,
                        btn_y + 2.0 + li as f32 * 18.0,
                        if self.error.is_some() {
                            theme::CRIMSON_BRIGHT()
                        } else {
                            theme::MUTED()
                        },
                        line,
                    );
                }
            }
        } else {
            // Compact single-column Settings with "Import Control" section at top
            let footer_h = 52.0;
            let top = cy;
            let bottom = cy + ch - footer_h;
            let all_items: Vec<_> = left_items.into_iter().chain(right_items).collect();
            let import_card_h = 154.0;
            let import_block_h = import_card_h + 12.0;
            let row_h = 68.0;
            let total_h = import_block_h + all_items.len() as f32 * row_h;
            let max_scroll = (total_h - (bottom - top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([top, bottom, max_scroll]);

            let iy = top - self.catalog.scroll;
            let half_btn = (cw - 32.0 - 8.0) * 0.5;
            let third_btn = (cw - 32.0 - 12.0) / 3.0;
            let stu_btn_r = [cx + 16.0, iy + 68.0, half_btn, 36.0];
            let imp_btn_r = [cx + 16.0 + half_btn + 8.0, iy + 68.0, half_btn, 36.0];
            let sav_btn_r = [cx + 16.0, iy + 110.0, third_btn, 32.0];
            let all_btn_r = [cx + 16.0 + third_btn + 6.0, iy + 110.0, third_btn, 32.0];
            let rst_btn_r = [cx + 16.0 + (third_btn + 6.0) * 2.0, iy + 110.0, third_btn, 32.0];
            self.list_targets.push((
                stu_btn_r,
                Action::Studio(super::controls_studio::StudioAction::Open),
            ));
            self.list_targets.push((imp_btn_r, Action::ImportControl));
            self.list_targets.push((sav_btn_r, Action::ControlLayouts));
            self.list_targets
                .push((all_btn_r, Action::ApplyControlsAllProfiles));
            self.list_targets.push((rst_btn_r, Action::ResetControls));

            if iy >= top - 4.0 && iy + import_card_h <= bottom + 4.0 {
                ui.rounded(cx, iy, cw, import_card_h, 12.0, theme::HERO_BG());
                ui.border(cx, iy, cw, import_card_h, 12.0, theme::BORDER_CRIMSON());
                ui.text(
                    Face::LauncherMedium,
                    16.5,
                    cx + 16.0,
                    iy + 10.0,
                    theme::TEXT(),
                    "Controls Studio",
                );
                let act_fit =
                    ui.fit_text(Face::LauncherRegular, 12.5, &active_ctrl_summary, cw - 32.0);
                ui.text(
                    Face::LauncherRegular,
                    12.5,
                    cx + 16.0,
                    iy + 38.0,
                    if self.catalog.active_controls.is_some() {
                        theme::CRIMSON_BRIGHT()
                    } else {
                        theme::MUTED()
                    },
                    &act_fit,
                );
                styled_button(
                    ui,
                    self.targets,
                    stu_btn_r,
                    "Controls Studio",
                    Action::Studio(super::controls_studio::StudioAction::Open),
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    imp_btn_r,
                    "Import File",
                    Action::ImportControl,
                    BtnStyle::CrimsonOutline,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    sav_btn_r,
                    &saved_label,
                    Action::ControlLayouts,
                    BtnStyle::Secondary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    all_btn_r,
                    "Apply All",
                    Action::ApplyControlsAllProfiles,
                    BtnStyle::Secondary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    rst_btn_r,
                    "Reset",
                    Action::ResetControls,
                    BtnStyle::Secondary,
                    true,
                );
            }

            for (i, (label, val, act, sw, dropdown)) in all_items.iter().enumerate() {
                let y = top + import_block_h + i as f32 * row_h - self.catalog.scroll;
                let ctrl_rect = [cx, y + 24.0, cw, 38.0];
                self.list_targets.push((ctrl_rect, *act));
                if y < top || y + row_h - 4.0 > bottom {
                    continue;
                }
                ui.text(Face::LauncherRegular, 13.5, cx + 2.0, y, theme::MUTED(), label);
                if let Some(on) = sw {
                    toggle(ui, self.targets, ctrl_rect, *on, *act);
                } else {
                    let text = if *dropdown {
                        format!("{val}   v")
                    } else {
                        val.clone()
                    };
                    styled_button(
                        ui,
                        self.targets,
                        ctrl_rect,
                        &text,
                        *act,
                        BtnStyle::Secondary,
                        false,
                    );
                }
            }
            let half = (cw - 10.0) * 0.5;
            styled_button(
                ui,
                self.targets,
                [cx, cy + ch - 44.0, half, 40.0],
                "Diagnostics",
                Action::Logs,
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [cx + half + 10.0, cy + ch - 44.0, half, 40.0],
                "Microsoft account",
                Action::Account,
                BtnStyle::Primary,
                true,
            );
        }
    }

    fn draw_volt_settings_page(&mut self, ui: &mut Ui, area: [f32; 4], compact: bool) {
        use horizon_client::extras::Item;
        use horizon_client::volt_features::VoltBoostPreset;

        let [cx, cy, cw, ch] = area;
        let c = &self.catalog.config;
        let ex = &self.catalog.extras;

        let render_scale_val = if self.catalog.editing == Some("render_scale") {
            self.catalog.render_scale_text.clone()
        } else {
            format!("{}%", c.render_scale.percent())
        };
        let vulkan_driver_val = match &c.custom_vulkan_driver_name {
            Some(name) if !name.trim().is_empty() => format!("Custom: {name}"),
            _ if c.custom_vulkan_driver.is_some() => "Custom .so active".to_owned(),
            _ => "Built-in Supporter".to_owned(),
        };

        let left_items: Vec<(&str, String, Action, Option<bool>, bool)> = vec![
            (
                "Rendering engine",
                c.renderer.label().to_owned(),
                Action::Renderer,
                None,
                true,
            ),
            (
                "Vulkan driver",
                vulkan_driver_val,
                Action::ImportVulkanDriver,
                None,
                false,
            ),
            (
                "Client resolution",
                render_scale_val,
                Action::RenderScale,
                None,
                true,
            ),
            (
                "Auto-DRS dynamic scale",
                Item::VoltAutoDrs.value(ex),
                Action::VoltExtra(Item::VoltAutoDrs),
                None,
                false,
            ),
            (
                "Display refresh rate",
                c.device.display_label(),
                Action::DisplayHz,
                None,
                true,
            ),
            (
                "ADPF CPU/GPU boost",
                String::new(),
                Action::PerformanceHints,
                Some(c.device.performance_hints),
                false,
            ),
            (
                "big.LITTLE prime affinity",
                String::new(),
                Action::VoltExtra(Item::VoltThreadBoost),
                Some(ex.thread_boost),
                false,
            ),
            (
                "Volt Reflex low-latency",
                String::new(),
                Action::VoltExtra(Item::VoltReflex),
                Some(ex.volt_reflex),
                false,
            ),
            (
                "Shader & cache warmup",
                String::new(),
                Action::VoltExtra(Item::VoltPrewarm),
                Some(ex.prewarm_pipelines),
                false,
            ),
            (
                "Dynamic menu / idle FPS",
                Item::VoltDynamicFps.value(ex),
                Action::VoltExtra(Item::VoltDynamicFps),
                None,
                false,
            ),
            (
                "Menu backdrop saver",
                Item::VoltMenuBg.value(ex),
                Action::VoltExtra(Item::VoltMenuBg),
                None,
                false,
            ),
            (
                "Mobile thermal guard",
                String::new(),
                Action::VoltExtra(Item::VoltThermalGuard),
                Some(ex.thermal_guard),
                false,
            ),
            (
                "Audio mixer voice cap",
                Item::VoltAudioCap.value(ex),
                Action::VoltExtra(Item::VoltAudioCap),
                None,
                false,
            ),
            (
                "Aggressive RAM compactor",
                String::new(),
                Action::VoltExtra(Item::VoltMemorySaver),
                Some(ex.memory_saver),
                false,
            ),
            (
                "Keep screen awake",
                String::new(),
                Action::VoltExtra(Item::VoltKeepAwake),
                Some(ex.keep_screen_awake),
                false,
            ),
            (
                "Live performance HUD",
                Item::VoltPerfHud.value(ex),
                Action::VoltExtra(Item::VoltPerfHud),
                None,
                false,
            ),
            (
                "Perf HUD corner",
                Item::VoltPerfHudCorner.value(ex),
                Action::VoltExtra(Item::VoltPerfHudCorner),
                None,
                false,
            ),
        ];

        let right_items: Vec<(&str, String, Action, Option<bool>, bool)> = vec![
            (
                "Smart leaf & canopy cull",
                String::new(),
                Action::VoltExtra(Item::VoltCullLeaves),
                Some(ex.cull_leaves),
                false,
            ),
            (
                "Frustum-first chunk mesh",
                String::new(),
                Action::VoltExtra(Item::VoltFrustumMesh),
                Some(ex.frustum_priority_mesh),
                false,
            ),
            (
                "Velocity lookahead stream",
                String::new(),
                Action::VoltExtra(Item::VoltLookaheadStream),
                Some(ex.lookahead_stream),
                false,
            ),
            (
                "Adaptive water sort",
                String::new(),
                Action::VoltExtra(Item::VoltFastTranslucency),
                Some(ex.fast_translucency),
                false,
            ),
            (
                "Cave sky & cloud cull",
                String::new(),
                Action::VoltExtra(Item::VoltCaveSkyCull),
                Some(ex.cave_sky_cull),
                false,
            ),
            (
                "Chunk GPU upload cap",
                Item::VoltChunkUploadCap.value(ex),
                Action::VoltExtra(Item::VoltChunkUploadCap),
                None,
                false,
            ),
            (
                "Fast weather columns",
                String::new(),
                Action::VoltExtra(Item::VoltFastWeather),
                Some(ex.fast_weather),
                false,
            ),
            (
                "Distant horizon LOD boost",
                String::new(),
                Action::VoltExtra(Item::VoltDistantLod),
                Some(ex.distant_lod_boost),
                false,
            ),
            (
                "Max rendered entities",
                Item::VoltEntityCap.value(ex),
                Action::VoltExtra(Item::VoltEntityCap),
                None,
                false,
            ),
            (
                "Dropped item cap",
                Item::VoltItemCap.value(ex),
                Action::VoltExtra(Item::VoltItemCap),
                None,
                false,
            ),
            (
                "Block-entity cap + occlude",
                Item::VoltBlockEntityCap.value(ex),
                Action::VoltExtra(Item::VoltBlockEntityCap),
                None,
                false,
            ),
            (
                "Sign & hologram text cull",
                String::new(),
                Action::VoltExtra(Item::VoltCullSignText),
                Some(ex.cull_sign_text),
                false,
            ),
            (
                "Entity contact shadows",
                Item::VoltFastShadows.value(ex),
                Action::VoltExtra(Item::VoltFastShadows),
                None,
                false,
            ),
            (
                "Distant mob pose throttle",
                String::new(),
                Action::VoltExtra(Item::VoltDistantMobThrottle),
                Some(ex.distant_mob_throttle),
                false,
            ),
            (
                "## Frame items",
                String::new(),
                Action::VoltExtra(Item::FrameMobs),
                None,
                false,
            ),
            (
                "Mobs",
                String::new(),
                Action::VoltExtra(Item::FrameMobs),
                Some(ex.frame_mobs),
                false,
            ),
            (
                "Mob render distance",
                Item::FrameMobsDistance.value(ex),
                Action::VoltExtra(Item::FrameMobsDistance),
                None,
                false,
            ),
            (
                "Other players",
                String::new(),
                Action::VoltExtra(Item::FramePlayers),
                Some(ex.frame_players),
                false,
            ),
            (
                "Player render distance",
                Item::FramePlayersDistance.value(ex),
                Action::VoltExtra(Item::FramePlayersDistance),
                None,
                false,
            ),
            (
                "Dropped items",
                String::new(),
                Action::VoltExtra(Item::FrameDropped),
                Some(ex.frame_dropped),
                false,
            ),
            (
                "Dropped item distance",
                Item::FrameDroppedDistance.value(ex),
                Action::VoltExtra(Item::FrameDroppedDistance),
                None,
                false,
            ),
            (
                "Block entities",
                String::new(),
                Action::VoltExtra(Item::FrameBlockEntities),
                Some(ex.frame_block_entities),
                false,
            ),
            (
                "Block-entity distance",
                Item::FrameBlockEntitiesDistance.value(ex),
                Action::VoltExtra(Item::FrameBlockEntitiesDistance),
                None,
                false,
            ),
            (
                "Particles",
                String::new(),
                Action::VoltExtra(Item::FrameParticles),
                Some(ex.frame_particles),
                false,
            ),
            (
                "Particle render distance",
                Item::FrameParticlesDistance.value(ex),
                Action::VoltExtra(Item::FrameParticlesDistance),
                None,
                false,
            ),
            (
                "Multiplayer hub lag shield",
                String::new(),
                Action::VoltExtra(Item::VoltHubLagShield),
                Some(ex.hub_lag_shield),
                false,
            ),
            (
                "Mod lag shield",
                String::new(),
                Action::VoltExtra(Item::VoltModLagShield),
                Some(ex.mod_lag_shield),
                false,
            ),
            (
                "Particle budget & cutoff",
                Item::VoltParticleBudget.value(ex),
                Action::VoltExtra(Item::VoltParticleBudget),
                None,
                false,
            ),
            (
                "TNT / firework / break shield",
                String::new(),
                Action::VoltExtra(Item::VoltChaosShield),
                Some(ex.chaos_particle_shield),
                false,
            ),
            (
                "Smart 10 Hz animated atlas",
                String::new(),
                Action::VoltExtra(Item::VoltAnimThrottle),
                Some(ex.anim_throttle),
                false,
            ),
            (
                "Glint & armor-trim saver",
                Item::VoltGlintMode.value(ex),
                Action::VoltExtra(Item::VoltGlintMode),
                None,
                false,
            ),
        ];

        let rss_mib = horizon_threads::resident_ram_mib().unwrap_or(0);
        let total_mib = horizon_client::extras::detect_total_ram_mib().unwrap_or(0);
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let abi = if cfg!(target_arch = "aarch64") {
            "arm64-v8a"
        } else if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "armeabi-v7a"
        };
        let active_n = ex.active_volt_optimizations();
        let hw_summary = format!(
            "RAM: {rss_mib} / {total_mib} MiB  •  {cores} Cores ({abi})  •  {active_n} Boosts Active"
        );

        let presets = [
            (VoltBoostPreset::Off, "OFF"),
            (VoltBoostPreset::Balanced, "Balanced"),
            (VoltBoostPreset::Competitive, "PvP"),
            (VoltBoostPreset::UltraFps, "Ultra FPS"),
            (VoltBoostPreset::BatterySaver, "Battery"),
            (VoltBoostPreset::LowRam, "Low-RAM"),
        ];

        if !compact {
            let hero_h = 104.0;
            ui.rounded(cx, cy, cw, hero_h, 14.0, theme::HERO_BG());
            ui.rounded(
                cx + 8.0,
                cy + 8.0,
                cw * 0.48,
                hero_h - 16.0,
                12.0,
                theme::HERO_GLOW_SOFT(),
            );
            ui.border(cx, cy, cw, hero_h, 14.0, theme::BORDER_CRIMSON());

            ui.text(
                Face::LauncherMedium,
                18.5,
                cx + 20.0,
                cy + 11.0,
                theme::TEXT(),
                "Volt Performance & Optimization Suite",
            );
            let hw_fit = ui.fit_text(Face::LauncherRegular, 13.0, &hw_summary, (cw - 320.0).max(160.0));
            ui.text(
                Face::LauncherRegular,
                13.0,
                cx + 20.0,
                cy + 36.0,
                theme::MUTED(),
                &hw_fit,
            );

            let act_w = 132.0;
            styled_button(
                ui,
                self.targets,
                [cx + cw - act_w * 2.0 - 24.0, cy + 12.0, act_w, 34.0],
                "Auto-Tune",
                Action::VoltExtra(Item::VoltAutoOptimize),
                BtnStyle::Primary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [cx + cw - act_w - 16.0, cy + 12.0, act_w, 34.0],
                "Purge RAM",
                Action::VoltExtra(Item::VoltPurgeRam),
                BtnStyle::CrimsonOutline,
                true,
            );

            let pill_gap = 8.0;
            let pill_w = ((cw - 32.0 - pill_gap * 5.0) / 6.0).max(64.0);
            for (pi, (preset, name)) in presets.iter().enumerate() {
                let px = cx + 16.0 + pi as f32 * (pill_w + pill_gap);
                let selected = ex.volt_preset == *preset;
                styled_button(
                    ui,
                    self.targets,
                    [px, cy + 60.0, pill_w, 33.0],
                    name,
                    Action::VoltPreset(*preset),
                    if selected {
                        BtnStyle::Primary
                    } else {
                        BtnStyle::Secondary
                    },
                    true,
                );
            }

            let gap = 16.0;
            let col_w = (cw - gap) * 0.5;
            let cards_y = cy + hero_h + 10.0;
            let cards_h = (ch - hero_h - 12.0).max(150.0);

            ui.rounded(cx, cards_y, col_w, cards_h, 14.0, theme::CARD());
            ui.border(cx, cards_y, col_w, cards_h, 14.0, theme::BORDER());
            ui.text(
                Face::LauncherMedium,
                16.5,
                cx + 20.0,
                cards_y + 12.0,
                theme::TEXT(),
                "Launcher, Hardware, CPU & RAM",
            );

            let rx = cx + col_w + gap;
            ui.rounded(rx, cards_y, col_w, cards_h, 14.0, theme::CARD());
            ui.border(rx, cards_y, col_w, cards_h, 14.0, theme::BORDER());
            ui.text(
                Face::LauncherMedium,
                16.5,
                rx + 20.0,
                cards_y + 12.0,
                theme::TEXT(),
                "In-World Terrain, Entities & Particles",
            );

            let list_top = cards_y + 46.0;
            let list_bottom = cards_y + cards_h - 8.0;
            let row_h = 46.0;
            let max_rows = left_items.len().max(right_items.len());
            let max_scroll = (max_rows as f32 * row_h - (list_bottom - list_top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([list_top, list_bottom, max_scroll]);

            for (slice, base_x) in [(&left_items[..], cx), (&right_items[..], rx)] {
                for (i, (label, val, act, sw, dropdown)) in slice.iter().enumerate() {
                    let y = list_top + i as f32 * row_h - self.catalog.scroll;
                    let ctrl_w = (col_w * 0.46).clamp(124.0, 214.0);
                    let ctrl_rect = [base_x + col_w - ctrl_w - 16.0, y + 5.0, ctrl_w, 34.0];
                    // "## " rows are section headers: no control, no tap target.
                    if let Some(header) = label.strip_prefix("## ") {
                        if y < list_top - 4.0 || y + row_h - 4.0 > list_bottom {
                            continue;
                        }
                        ui.text(
                            Face::LauncherMedium,
                            15.0,
                            base_x + 18.0,
                            text_vcenter(y + 6.0, row_h - 4.0, 15.0),
                            theme::TEXT(),
                            header,
                        );
                        ui.rounded(
                            base_x + 18.0,
                            y + row_h - 2.0,
                            col_w - 34.0,
                            1.0,
                            0.0,
                            theme::BORDER(),
                        );
                        continue;
                    }
                    self.list_targets.push((ctrl_rect, *act));
                    if y < list_top - 4.0 || y + row_h - 4.0 > list_bottom {
                        continue;
                    }
                    let row_rect = [base_x + 10.0, y + 1.0, col_w - 20.0, row_h - 4.0];
                    if point_in(ui.cursor, row_rect) {
                        ui.rounded(row_rect[0], row_rect[1], row_rect[2], row_rect[3], 7.0, theme::HOVER());
                        ui.rounded(row_rect[0] + 1.0, row_rect[1] + 6.0, 2.5, row_rect[3] - 12.0, 1.2, theme::CRIMSON_BRIGHT());
                    }
                    let lbl_fit =
                        ui.fit_text(Face::LauncherRegular, 13.5, label, col_w - ctrl_w - 36.0);
                    ui.text(
                        Face::LauncherRegular,
                        13.5,
                        base_x + 18.0,
                        text_vcenter(y, row_h - 4.0, 13.5),
                        if point_in(ui.cursor, row_rect) { theme::TEXT() } else { theme::MUTED() },
                        &lbl_fit,
                    );
                    if let Some(on) = sw {
                        toggle(ui, self.targets, ctrl_rect, *on, *act);
                    } else {
                        let text = if *dropdown {
                            format!("{val}   v")
                        } else {
                            val.clone()
                        };
                        styled_button(
                            ui,
                            self.targets,
                            ctrl_rect,
                            &text,
                            *act,
                            BtnStyle::Secondary,
                            false,
                        );
                    }
                    if i + 1 < slice.len() {
                        ui.rounded(
                            base_x + 18.0,
                            y + row_h - 2.0,
                            col_w - 34.0,
                            1.0,
                            0.0,
                            theme::BORDER(),
                        );
                    }
                }
            }
        } else {
            let top = cy;
            let bottom = cy + ch;
            let all_items: Vec<_> = left_items.into_iter().chain(right_items).collect();
            let hero_h = 152.0;
            let row_h = 66.0;
            let total_h = hero_h + 12.0 + all_items.len() as f32 * row_h;
            let max_scroll = (total_h - (bottom - top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([top, bottom, max_scroll]);

            let iy = top - self.catalog.scroll;
            let half_btn = (cw - 32.0 - 8.0) * 0.5;
            let auto_r = [cx + 16.0, iy + 58.0, half_btn, 36.0];
            let purge_r = [cx + 16.0 + half_btn + 8.0, iy + 58.0, half_btn, 36.0];
            let next_preset_r = [cx + 16.0, iy + 102.0, cw - 32.0, 36.0];
            self.list_targets
                .push((auto_r, Action::VoltExtra(Item::VoltAutoOptimize)));
            self.list_targets
                .push((purge_r, Action::VoltExtra(Item::VoltPurgeRam)));
            self.list_targets
                .push((next_preset_r, Action::VoltExtra(Item::VoltPreset)));

            if iy >= top - 4.0 && iy + hero_h <= bottom + 4.0 {
                ui.rounded(cx, iy, cw, hero_h, 12.0, theme::HERO_BG());
                ui.border(cx, iy, cw, hero_h, 12.0, theme::BORDER_CRIMSON());
                ui.text(
                    Face::LauncherMedium,
                    16.5,
                    cx + 16.0,
                    iy + 10.0,
                    theme::TEXT(),
                    "Volt Settings & Boost",
                );
                let hw_fit = ui.fit_text(Face::LauncherRegular, 12.0, &hw_summary, cw - 32.0);
                ui.text(Face::LauncherRegular, 12.0, cx + 16.0, iy + 34.0, theme::MUTED(), &hw_fit);
                styled_button(ui, self.targets, auto_r, "Auto-Tune", Action::VoltExtra(Item::VoltAutoOptimize), BtnStyle::Primary, true);
                styled_button(ui, self.targets, purge_r, "Purge RAM", Action::VoltExtra(Item::VoltPurgeRam), BtnStyle::CrimsonOutline, true);
                styled_button(
                    ui,
                    self.targets,
                    next_preset_r,
                    &format!("Profile: {} (Tap to Cycle)", ex.volt_preset.label()),
                    Action::VoltExtra(Item::VoltPreset),
                    BtnStyle::Secondary,
                    true,
                );
            }

            for (i, (label, val, act, sw, dropdown)) in all_items.iter().enumerate() {
                let y = top + hero_h + 12.0 + i as f32 * row_h - self.catalog.scroll;
                let ctrl_rect = [cx, y + 24.0, cw, 36.0];
                // "## " rows are section headers: no control, no tap target.
                if let Some(header) = label.strip_prefix("## ") {
                    if y < top || y + row_h - 4.0 > bottom {
                        continue;
                    }
                    ui.text(
                        Face::LauncherMedium,
                        15.0,
                        cx + 2.0,
                        y + 20.0,
                        theme::TEXT(),
                        header,
                    );
                    continue;
                }
                self.list_targets.push((ctrl_rect, *act));
                if y < top || y + row_h - 4.0 > bottom {
                    continue;
                }
                ui.text(Face::LauncherRegular, 13.5, cx + 2.0, y, theme::MUTED(), label);
                if let Some(on) = sw {
                    toggle(ui, self.targets, ctrl_rect, *on, *act);
                } else {
                    let text = if *dropdown {
                        format!("{val}   v")
                    } else {
                        val.clone()
                    };
                    styled_button(ui, self.targets, ctrl_rect, &text, *act, BtnStyle::Secondary, false);
                }
            }
        }
    }

    fn draw_input_field(
        &mut self,
        ui: &mut Ui,
        rect: [f32; 4],
        field_name: &'static str,
        value: &str,
        placeholder: &str,
        masked: bool,
    ) {
        let [x, y, w, h] = rect;
        let focused = self.catalog.editing == Some(field_name);
        ui.rounded(x, y, w, h, 8.0, theme::CARD_INNER());
        ui.border(
            x,
            y,
            w,
            h,
            8.0,
            if focused {
                theme::CRIMSON_BRIGHT()
            } else {
                theme::BORDER()
            },
        );
        let display = if value.is_empty() {
            placeholder.to_owned()
        } else if masked {
            "*".repeat(value.chars().count())
        } else {
            value.to_owned()
        };
        let col = if value.is_empty() {
            theme::DIM()
        } else {
            theme::TEXT()
        };
        let fitted = ui.fit_text(Face::LauncherMedium, 14.5, &display, (w - 24.0).max(16.0));
        ui.text(
            Face::LauncherMedium,
            14.5,
            x + 12.0,
            text_vcenter(y, h, 14.5),
            col,
            &fitted,
        );
        self.targets.push((rect, Action::Field(field_name)));
    }

    fn draw_account_page(
        &mut self,
        ui: &mut Ui,
        area: [f32; 4],
        compact: bool,
        account_status: &str,
    ) {
        use crate::volt_accounts::{AccountProvider, PresetCape};
        let [cx, cy, cw, ch] = area;

        // Top Page Title + Status Bar
        let header_h = if compact { 34.0 } else { 38.0 };
        ui.text(
            Face::LauncherMedium,
            if compact { 19.5 } else { 24.0 },
            cx + 4.0,
            cy + 2.0,
            theme::TEXT(),
            "Account & Custom Skin Wardrobe",
        );
        if !compact {
            let chip_w = (cw * 0.34).clamp(190.0, 300.0);
            draw_account_chip(
                ui,
                self.targets,
                [cx + cw - chip_w, cy, chip_w, 36.0],
                account_status,
                self.volt_accounts
                    .active()
                    .is_some_and(|a| a.provider != AccountProvider::Offline),
            );
        }

        let body_y = cy + header_h + 8.0;
        let body_h = (ch - header_h - if compact { 8.0 } else { 28.0 }).max(180.0);

        let active_acc = self.volt_accounts.active().cloned();
        let (acc_name, acc_badge, acc_slim, acc_sync, acc_signed, skin_lbl, cape_lbl, default_uuid) =
            match &active_acc {
                Some(a) => (
                    a.name.as_str(),
                    a.provider.badge(),
                    a.slim,
                    a.sync_to_servers,
                    a.signed_skin_url.is_some() || a.skin_label.starts_with("Mojang:"),
                    a.skin_label.as_str(),
                    a.cape_label.as_str(),
                    a.uuid_u128(),
                ),
                None => (
                    "Player", "OFFLINE", false, true, false, "Default", "None", 0u128,
                ),
            };

        if compact {
            // Stacked scrollable mobile portrait layout
            let total_h = 260.0 + 320.0 + 310.0 + 24.0;
            let max_scroll = (total_h - body_h).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, max_scroll);
            *self.list_view = Some([body_y, body_y + body_h, max_scroll]);
            let s = self.catalog.scroll;

            let p_rect = [cx, body_y - s, cw, 250.0];
            if p_rect[1] + p_rect[3] > body_y && p_rect[1] < body_y + body_h {
                self.draw_wardrobe_card(
                    ui,
                    p_rect,
                    acc_name,
                    acc_badge,
                    acc_slim,
                    skin_lbl,
                    cape_lbl,
                    default_uuid,
                );
            }
            let a_rect = [cx, body_y + 262.0 - s, cw, 310.0];
            if a_rect[1] + a_rect[3] > body_y && a_rect[1] < body_y + body_h {
                self.draw_accounts_auth_card(ui, a_rect);
            }
            let c_rect = [cx, body_y + 584.0 - s, cw, 300.0];
            if c_rect[1] + c_rect[3] > body_y && c_rect[1] < body_y + body_h {
                self.draw_cosmetics_loader_card(ui, c_rect, acc_sync, acc_signed);
            }
            return;
        }

        // Wide 3-column bento layout (Left: 3D Character Wardrobe | Center: Accounts & Auth | Right: Skin & Cape Loader)
        let gap = 12.0;
        let left_w = (cw * 0.29).clamp(200.0, 290.0).round();
        let rem_w = (cw - left_w - gap * 2.0).max(320.0);
        let mid_w = (rem_w * 0.48).round();
        let right_w = rem_w - mid_w;

        self.draw_wardrobe_card(
            ui,
            [cx, body_y, left_w, body_h],
            acc_name,
            acc_badge,
            acc_slim,
            skin_lbl,
            cape_lbl,
            default_uuid,
        );
        self.draw_accounts_auth_card(ui, [cx + left_w + gap, body_y, mid_w, body_h]);
        self.draw_cosmetics_loader_card(
            ui,
            [cx + left_w + gap + mid_w + gap, body_y, right_w, body_h],
            acc_sync,
            acc_signed,
        );

        let status = self
            .volt_accounts
            .busy
            .as_deref()
            .or(self.error.as_deref())
            .or_else(|| {
                (!self.catalog.status.is_empty()).then_some(self.catalog.status.as_str())
            })
            .or(self.volt_accounts.status.as_deref());
        if let Some(status) = status {
            let fitted = ui.fit_text(Face::LauncherMedium, 12.5, status, cw);
            ui.text(
                Face::LauncherMedium,
                12.5,
                cx + 4.0,
                cy + ch - 18.0,
                if self.error.is_some() {
                    theme::CRIMSON_BRIGHT()
                } else {
                    theme::MUTED()
                },
                &fitted,
            );
        }
        let _ = PresetCape::VoltCrimson;
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_wardrobe_card(
        &mut self,
        ui: &mut Ui,
        rect: [f32; 4],
        acc_name: &str,
        acc_badge: &str,
        acc_slim: bool,
        skin_lbl: &str,
        cape_lbl: &str,
        default_uuid: u128,
    ) {
        let [x, y, w, h] = rect;
        ui.rounded(x, y, w, h, 14.0, theme::CARD());
        ui.rounded(
            x + w * 0.14,
            y + h * 0.14,
            w * 0.72,
            h * 0.58,
            16.0,
            theme::HERO_GLOW_SOFT(),
        );
        ui.border(x, y, w, h, 14.0, theme::BORDER_CRIMSON());

        let badge_w = (ui.measure(Face::LauncherMedium, 11.0, acc_badge) + 18.0).min(96.0);
        ui.rounded(x + 14.0, y + 12.0, badge_w, 22.0, 6.0, theme::CRIMSON_BADGE());
        ui.border(x + 14.0, y + 12.0, badge_w, 22.0, 6.0, theme::BORDER_CRIMSON());
        draw_centered_text(
            ui,
            Face::LauncherMedium,
            11.0,
            [x + 14.0, y + 12.0, badge_w, 22.0],
            theme::CRIMSON_BRIGHT(),
            acc_badge,
        );

        let nfit = ui.fit_text(
            Face::LauncherMedium,
            14.5,
            acc_name,
            (w - badge_w - 94.0).max(36.0),
        );
        ui.text(
            Face::LauncherMedium,
            14.5,
            x + 20.0 + badge_w,
            text_vcenter(y + 12.0, 22.0, 14.5),
            theme::TEXT(),
            &nfit,
        );

        styled_button(
            ui,
            self.targets,
            [x + w - 68.0, y + 10.0, 56.0, 26.0],
            "Flip",
            Action::ResetPreviewAngle,
            BtnStyle::Secondary,
            true,
        );

        let footer_h = 68.0;
        let model_bounds = [
            x + 10.0,
            y + 40.0,
            (w - 20.0).max(40.0),
            (h - 46.0 - footer_h).max(60.0),
        ];
        crate::skin_preview::draw_player_3d(
            ui,
            model_bounds,
            self.preview_yaw,
            self.preview_pitch,
            self.idle_phase,
            self.preview_vel_yaw,
            2,
            3,
            self.volt_accounts.active_skin_preview(),
            self.volt_accounts.active_cape_preview(),
            default_uuid,
        );
        *self.preview_rect = Some(model_bounds);

        // Classic vs Slim arm toggle + equipped summary at bottom of 3D preview card
        let arm_label = if acc_slim {
            "Model: Slim (3px)"
        } else {
            "Model: Classic (4px)"
        };
        styled_button(
            ui,
            self.targets,
            [x + 14.0, y + h - 62.0, w - 28.0, 30.0],
            arm_label,
            Action::ToggleSlimModel,
            BtnStyle::CrimsonOutline,
            true,
        );
        let summary = ui.fit_text(
            Face::LauncherRegular,
            11.5,
            &format!("Skin: {skin_lbl}  •  Cape: {cape_lbl}"),
            w - 24.0,
        );
        draw_centered_text(
            ui,
            Face::LauncherRegular,
            11.5,
            [x + 12.0, y + h - 26.0, w - 24.0, 20.0],
            theme::MUTED(),
            &summary,
        );
    }

    fn draw_accounts_auth_card(&mut self, ui: &mut Ui, rect: [f32; 4]) {
        use crate::volt_accounts::AccountProvider;
        let [x, y, w, h] = rect;
        ui.rounded(x, y, w, h, 14.0, theme::CARD());
        ui.border(x, y, w, h, 14.0, theme::BORDER_CRIMSON());

        ui.text(
            Face::LauncherMedium,
            15.5,
            x + 16.0,
            y + 12.0,
            theme::TEXT(),
            "Accounts",
        );

        // Saved accounts list (top section)
        let max_list_rows = ((h - 216.0) / 36.0).floor().clamp(2.0, 4.0) as usize;
        let list_start_y = y + 38.0;
        let active_id = self.volt_accounts.active_id.as_str();
        for (idx, acc) in self
            .volt_accounts
            .accounts
            .iter()
            .take(max_list_rows)
            .enumerate()
        {
            let ry = list_start_y + idx as f32 * 36.0;
            let selected = acc.id == active_id;
            let row_w = (w - 68.0).max(80.0);
            let row_rect = [x + 14.0, ry, row_w, 31.0];
            ui.rounded(
                row_rect[0],
                row_rect[1],
                row_rect[2],
                row_rect[3],
                8.0,
                if selected {
                    theme::CRIMSON_SOFT()
                } else {
                    theme::CARD_INNER()
                },
            );
            ui.border(
                row_rect[0],
                row_rect[1],
                row_rect[2],
                row_rect[3],
                8.0,
                if selected {
                    theme::CRIMSON()
                } else {
                    theme::BORDER()
                },
            );
            let tag = format!("[{}] {}", acc.provider.badge(), acc.name);
            let tfit = ui.fit_text(Face::LauncherMedium, 12.5, &tag, row_w - 20.0);
            ui.text(
                Face::LauncherMedium,
                12.5,
                row_rect[0] + 10.0,
                text_vcenter(row_rect[1], row_rect[3], 12.5),
                if selected {
                    theme::ON_CRIMSON()
                } else {
                    theme::TEXT()
                },
                &tfit,
            );
            self.targets.push((row_rect, Action::SelectVoltAccount(idx)));

            styled_button(
                ui,
                self.targets,
                [x + w - 48.0, ry, 34.0, 31.0],
                "Del",
                Action::RemoveVoltAccount(idx),
                BtnStyle::Subtle,
                true,
            );
        }

        // Provider Tabs: Offline | Ely.by | Microsoft
        let form_top = list_start_y + (max_list_rows.min(self.volt_accounts.accounts.len().max(1)) as f32) * 36.0 + 8.0;
        ui.rounded(x + 14.0, form_top, w - 28.0, 30.0, 8.0, theme::RAIL());
        ui.border(x + 14.0, form_top, w - 28.0, 30.0, 8.0, theme::BORDER());
        let tabs = [
            (AccountProvider::Offline, "Offline"),
            (AccountProvider::ElyBy, "Ely.by"),
            (AccountProvider::Microsoft, "Microsoft"),
        ];
        let tab_w = (w - 32.0) / 3.0;
        for (ti, (prov, label)) in tabs.into_iter().enumerate() {
            let tx = x + 16.0 + ti as f32 * tab_w;
            let trect = [tx, form_top + 2.0, tab_w - 2.0, 26.0];
            let active = self.catalog.account_tab == prov;
            if active {
                ui.rounded(trect[0], trect[1], trect[2], trect[3], 6.0, theme::CRIMSON());
            }
            draw_centered_text(
                ui,
                Face::LauncherMedium,
                12.0,
                trect,
                if active { theme::ON_CRIMSON() } else { theme::MUTED() },
                label,
            );
            self.targets.push((trect, Action::AccountProviderTab(prov)));
        }

        let fy = form_top + 38.0;
        match self.catalog.account_tab {
            AccountProvider::Offline => {
                let val = self.catalog.offline_name_input.clone();
                self.draw_input_field(
                    ui,
                    [x + 14.0, fy, w - 28.0, 34.0],
                    "offline_name",
                    &val,
                    "Offline username (1-16 chars)",
                    false,
                );
                let bw = (w - 36.0) * 0.5;
                styled_button(
                    ui,
                    self.targets,
                    [x + 14.0, fy + 42.0, bw, 34.0],
                    "Create / Switch",
                    Action::CreateOfflineAccount,
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    [x + 22.0 + bw, fy + 42.0, bw, 34.0],
                    "Rename Active",
                    Action::UpdateOfflineName,
                    BtnStyle::Secondary,
                    true,
                );
                let note = ui.fit_text(
                    Face::LauncherRegular,
                    11.5,
                    "Supports full offline skin & cape equipping + server sync.",
                    w - 28.0,
                );
                ui.text(Face::LauncherRegular, 11.5, x + 14.0, fy + 84.0, theme::MUTED(), &note);
            }
            AccountProvider::ElyBy => {
                let u_val = self.catalog.ely_user_input.clone();
                let p_val = self.catalog.ely_pass_input.clone();
                let t_val = self.catalog.ely_totp_input.clone();
                self.draw_input_field(
                    ui,
                    [x + 14.0, fy, w - 28.0, 32.0],
                    "ely_user",
                    &u_val,
                    "Ely.by email or username",
                    false,
                );
                let pw_w = (w - 36.0) * 0.68;
                let totp_w = (w - 36.0) - pw_w;
                self.draw_input_field(
                    ui,
                    [x + 14.0, fy + 38.0, pw_w, 32.0],
                    "ely_pass",
                    &p_val,
                    "Password",
                    true,
                );
                self.draw_input_field(
                    ui,
                    [x + 22.0 + pw_w, fy + 38.0, totp_w, 32.0],
                    "ely_totp",
                    &t_val,
                    "2FA",
                    false,
                );
                let sign_w = (w - 36.0) * 0.62;
                let reg_w = (w - 36.0) - sign_w;
                styled_button(
                    ui,
                    self.targets,
                    [x + 14.0, fy + 78.0, sign_w, 34.0],
                    if self.volt_accounts.busy.is_some() {
                        "Signing in..."
                    } else {
                        "Sign In (Ely.by)"
                    },
                    Action::ElySignIn,
                    BtnStyle::Primary,
                    true,
                );
                styled_button(
                    ui,
                    self.targets,
                    [x + 22.0 + sign_w, fy + 78.0, reg_w, 34.0],
                    "Register",
                    Action::ElyRegister,
                    BtnStyle::CrimsonOutline,
                    true,
                );
            }
            AccountProvider::Microsoft => {
                let lines = wrap_lines(
                    ui,
                    Face::LauncherRegular,
                    12.0,
                    "Official Microsoft / Xbox Live login for Hypixel and online-mode Java servers using device code.",
                    w - 28.0,
                    3,
                );
                for (li, line) in lines.iter().enumerate() {
                    ui.text(
                        Face::LauncherRegular,
                        12.0,
                        x + 14.0,
                        fy + li as f32 * 17.0,
                        theme::MUTED(),
                        line,
                    );
                }
                let btn_y = fy + 56.0;
                styled_button(
                    ui,
                    self.targets,
                    [x + 14.0, btn_y, w - 28.0, 36.0],
                    if self.login.signed_in {
                        "Manage / Re-authenticate Microsoft"
                    } else {
                        "Sign In with Microsoft"
                    },
                    Action::Account,
                    BtnStyle::Primary,
                    true,
                );
                if self.login.signed_in {
                    styled_button(
                        ui,
                        self.targets,
                        [x + 14.0, btn_y + 42.0, w - 28.0, 30.0],
                        "Sign Out Microsoft",
                        Action::SignOut,
                        BtnStyle::Subtle,
                        true,
                    );
                }
            }
        }
    }

    fn draw_cosmetics_loader_card(
        &mut self,
        ui: &mut Ui,
        rect: [f32; 4],
        acc_sync: bool,
        acc_signed: bool,
    ) {
        use crate::volt_accounts::PresetCape;
        let [x, y, w, h] = rect;
        ui.rounded(x, y, w, h, 14.0, theme::CARD());
        ui.border(x, y, w, h, 14.0, theme::BORDER_CRIMSON());

        ui.text(
            Face::LauncherMedium,
            15.5,
            x + 16.0,
            y + 12.0,
            theme::TEXT(),
            "Custom Skin & Cape Loader",
        );

        // Server-wide visibility toggle row
        let sync_sub = if !acc_sync {
            "Local + LAN only"
        } else if acc_signed {
            "Server Sync Ready (Signed + Ely.by + LAN)"
        } else {
            "Auto-syncs on SkinsRestorer, Ely.by & LAN"
        };
        let sync_fit = ui.fit_text(Face::LauncherRegular, 12.0, sync_sub, (w - 86.0).max(60.0));
        ui.text(Face::LauncherRegular, 12.0, x + 16.0, y + 38.0, theme::MUTED(), &sync_fit);
        toggle(
            ui,
            self.targets,
            [x + w - 62.0, y + 33.0, 48.0, 26.0],
            acc_sync,
            Action::ToggleServerSkinSync,
        );

        // Shared Username / URL input box
        let q_val = self.catalog.skin_query_input.clone();
        self.draw_input_field(
            ui,
            [x + 14.0, y + 66.0, w - 28.0, 34.0],
            "skin_query",
            &q_val,
            "Player name (Mojang/Ely.by) or PNG URL",
            false,
        );

        // Skin Loader Buttons
        ui.text(
            Face::LauncherMedium,
            12.5,
            x + 16.0,
            y + 106.0,
            theme::CRIMSON_BRIGHT(),
            "SKIN",
        );
        let half = (w - 36.0) * 0.5;
        styled_button(
            ui,
            self.targets,
            [x + 14.0, y + 124.0, half, 32.0],
            "Import PNG",
            Action::ImportSkinPng,
            BtnStyle::Primary,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 22.0 + half, y + 124.0, half, 32.0],
            "Fetch Username",
            Action::FetchSkinByUsername,
            BtnStyle::CrimsonOutline,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 14.0, y + 162.0, half, 30.0],
            "Load Skin URL",
            Action::LoadSkinUrl,
            BtnStyle::Secondary,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 22.0 + half, y + 162.0, half, 30.0],
            "Reset Skin",
            Action::ClearCustomSkin,
            BtnStyle::Subtle,
            true,
        );

        // Cape Loader + One-Tap Volt Preset Capes
        ui.text(
            Face::LauncherMedium,
            12.5,
            x + 16.0,
            y + 198.0,
            theme::CRIMSON_BRIGHT(),
            "CAPE",
        );
        let third = (w - 44.0) / 3.0;
        styled_button(
            ui,
            self.targets,
            [x + 14.0, y + 216.0, third, 30.0],
            "Import PNG",
            Action::ImportCapePng,
            BtnStyle::CrimsonOutline,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 22.0 + third, y + 216.0, third, 30.0],
            "Cape URL",
            Action::LoadCapeUrl,
            BtnStyle::Secondary,
            true,
        );
        styled_button(
            ui,
            self.targets,
            [x + 30.0 + third * 2.0, y + 216.0, third, 30.0],
            "Remove",
            Action::ClearCustomCape,
            BtnStyle::Subtle,
            true,
        );

        if y + 282.0 <= y + h {
            styled_button(
                ui,
                self.targets,
                [x + 14.0, y + 252.0, third, 30.0],
                "Volt V",
                Action::EquipPresetCape(PresetCape::VoltCrimson),
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [x + 22.0 + third, y + 252.0, third, 30.0],
                "Crown",
                Action::EquipPresetCape(PresetCape::RoyalGold),
                BtnStyle::Secondary,
                true,
            );
            styled_button(
                ui,
                self.targets,
                [x + 30.0 + third * 2.0, y + 252.0, third, 30.0],
                "Stealth",
                Action::EquipPresetCape(PresetCape::ObsidianStealth),
                BtnStyle::Secondary,
                true,
            );
        }
    }

}

impl Launcher {
    pub(super) fn is_animating(&self) -> bool {
        let page_idx = Page::ALL
            .iter()
            .position(|&p| p == self.catalog.page)
            .unwrap_or(0) as f32;
        if (self.sidebar_anim - page_idx).abs() > 0.005 {
            return true;
        }
        if self
            .page_anim
            .is_some_and(|(ts, _)| ts.elapsed().as_secs_f32() < 0.28)
        {
            return true;
        }
        if self
            .tab_anim
            .is_some_and(|(ts, _)| ts.elapsed().as_secs_f32() < 0.25)
        {
            return true;
        }
        if self
            .modal_anim
            .is_some_and(|ts| ts.elapsed().as_secs_f32() < 0.24)
        {
            return true;
        }
        if self
            .feedback
            .is_some_and(|(_, ts)| ts.elapsed().as_secs_f32() < 0.24)
        {
            return true;
        }
        if self.preview_target_yaw.is_some() || self.preview_vel_yaw.abs() > 0.4 {
            return true;
        }
        if self.worker.is_some() || self.catalog.animating_icons_or_cards() {
            return true;
        }
        has_active_toggle_anims()
    }

    fn tick_animations(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().clamp(0.0, 0.08);
        self.last_frame = now;

        // Detect page changes from any trigger (sidebar tap, character tap, controller, install completion)
        if self.catalog.page != self.prev_rendered_page {
            let old_idx = Page::ALL
                .iter()
                .position(|&p| p == self.prev_rendered_page)
                .unwrap_or(0) as f32;
            let new_idx = Page::ALL
                .iter()
                .position(|&p| p == self.catalog.page)
                .unwrap_or(0) as f32;
            let dir = if new_idx >= old_idx { 1.0 } else { -1.0 };
            self.page_anim = Some((now, dir));
            self.prev_rendered_page = self.catalog.page;
        }

        // Smoothly glide sidebar / dock active selection pill
        let target_idx = Page::ALL
            .iter()
            .position(|&p| p == self.catalog.page)
            .unwrap_or(0) as f32;
        let alpha = 1.0 - (-dt * 18.0).exp();
        self.sidebar_anim += (target_idx - self.sidebar_anim) * alpha;
        if (target_idx - self.sidebar_anim).abs() < 0.003 {
            self.sidebar_anim = target_idx;
        }

        // Retire finished home-tab switch transitions.
        if self.tab_anim.is_some_and(|(start, _)| start.elapsed().as_secs_f32() >= 0.22) {
            self.tab_anim = None;
        }

        // Detect modal pop-in transitions (Chooser / Microsoft Login / Update Prompt / Volt Boost)
        let modal_open = self.chooser.is_some()
            || self.login.visible
            || self.update_prompt.is_some()
            || self.catalog.volt_boost_modal
            || self.controls_studio.open;
        if modal_open && !self.modal_was_open {
            self.modal_anim = Some(now);
        }
        self.modal_was_open = modal_open;

        // Smooth 3D character Flip target + rotational inertia
        let dragging_preview = (self.mouse_down || self.drag.is_some())
            && self.drag.as_ref().is_some_and(|d| d.moved);
        if let Some(target_yaw) = self.preview_target_yaw {
            let mut diff = (target_yaw - self.preview_yaw + 180.0).rem_euclid(360.0) - 180.0;
            if diff.abs() < 0.4 {
                self.preview_yaw = target_yaw.rem_euclid(360.0);
                self.preview_vel_yaw = 0.0;
                self.preview_target_yaw = None;
            } else {
                let step_alpha = 1.0 - (-dt * 11.5).exp();
                let delta = diff * step_alpha;
                self.preview_yaw = (self.preview_yaw + delta).rem_euclid(360.0);
                self.preview_vel_yaw = (delta / dt.max(0.004)).clamp(-320.0, 320.0);
                diff -= delta;
                if diff.abs() < 0.4 {
                    self.preview_yaw = target_yaw.rem_euclid(360.0);
                    self.preview_target_yaw = None;
                }
            }
        } else if !dragging_preview {
            if self.preview_vel_yaw.abs() > 0.3 {
                self.preview_yaw =
                    (self.preview_yaw + self.preview_vel_yaw * dt).rem_euclid(360.0);
                self.preview_vel_yaw *= (-dt * 6.0).exp();
            } else {
                self.preview_vel_yaw = 0.0;
            }
        }
    }

    pub(super) fn draw_pages(&mut self) -> Result<()> {
        self.tick_animations();
        theme::set(
            self.catalog.config.launcher_dark,
            self.catalog.config.launcher_liquid,
            self.catalog.config.launcher_accent,
        );
        ACTIVE_FEEDBACK.with(|fb| *fb.borrow_mut() = self.feedback);
        let (Some(r), Some(gui)) = (&mut self.renderer, &self.gui) else {
            return Ok(());
        };
        if r.suspended {
            return Ok(());
        }
        self.targets.clear();
        self.list_targets.clear();
        self.list_view = None;
        self.preview_rect = None;
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        let mut ui = Ui::mobile(&gui.atlas, &mut b, (0.0, 0.0));
        // Draw the scene across the surface; controls use the visible window
        // rect so temporarily visible system bars never cover buttons.
        let full_w = ui.w;
        let full_h = ui.h;
        let scale = ui.density / ui.batch.scale;
        ui.batch.fill(&gui.atlas, 0.0, 0.0, full_w * scale, full_h * scale, theme::BG());
        draw_launcher_scene(&mut ui, full_w, full_h, self.launch_start.elapsed().as_secs_f32());
        let scene_end = ui.batch_mut().vertices.len();
        let mut offset = [0.0f32; 2];
        if let Some([left, top, right, bottom]) = crate::native_ime::launcher_visible_bounds() {
            let density = ui.density;
            let x0 = (left as f32 / density).min(full_w);
            let y0 = (top as f32 / density).min(full_h);
            let x1 = (right as f32 / density).min(full_w);
            let y1 = (bottom as f32 / density).min(full_h);
            if x1 - x0 >= 120.0 && y1 - y0 >= 120.0 {
                offset = [x0, y0];
                ui.w = x1 - x0;
                ui.h = y1 - y0;
            }
        }
        let (w, h) = (ui.w, ui.h);
        ui.cursor = (self.cursor.0 as f32 / ui.density - offset[0],
                     self.cursor.1 as f32 / ui.density - offset[1]);

        let compact = w < 760.0;
        // The lobby home draws full-bleed with its own corner controls, so
        // the dock shell / compact header stay out of its way entirely.
        let lobby = self.catalog.page == Page::Play && self.home_tab == HomeTab::Overview;
        let account_status = self
            .volt_accounts
            .active()
            .map(|a| format!("{} • {}", a.name, a.provider.badge()))
            .unwrap_or_else(|| self.login.status(&self.catalog.config.name));
        let idle_phase = self.launch_start.elapsed().as_secs_f32();
        let content_area;
        let page_v_start;
        {
            let mut ctx = DrawCtx {
                home: &self.home,
                install: &self.install,
                worker: &self.worker,
                error: &self.error,
                catalog: &mut self.catalog,
                home_tab: self.home_tab,
                targets: &mut self.targets,
                list_targets: &mut self.list_targets,
                list_view: &mut self.list_view,
                login: &self.login,
                volt_accounts: &self.volt_accounts,
                preview_yaw: self.preview_yaw,
                preview_pitch: self.preview_pitch,
                preview_vel_yaw: self.preview_vel_yaw,
                sidebar_anim: self.sidebar_anim,
                idle_phase,
                tab_anim: self.tab_anim.and_then(|(start, dir)| {
                    let el = start.elapsed().as_secs_f32();
                    (el < 0.22).then(|| {
                        let ease = ease_out_cubic(el / 0.22);
                        (dir * (1.0 - ease) * 26.0, ease)
                    })
                }),
                preview_rect: &mut self.preview_rect,
            };
            content_area = if lobby {
                [0.0, 0.0, w, h]
            } else if compact {
                ctx.draw_compact_header(&mut ui, w, &account_status);
                [6.0, 102.0, (w - 12.0).max(120.0), (h - 148.0).max(120.0)]
            } else {
                ctx.draw_shell(&mut ui, w, h, &account_status);
                // Black glassy far-left rail: reserve left gutter instead of bottom
                let left = 94.0;
                [left, 12.0, (w - left - 8.0).max(240.0), (h - 24.0).max(120.0)]
            };

            page_v_start = ui.batch_mut().vertices.len();
            let page = ctx.catalog.page;
            match page {
                Page::Play | Page::Mods => ctx.draw_instance_page(&mut ui, content_area, compact, &account_status),
                Page::Account => {
                    ctx.draw_account_page(&mut ui, content_area, compact, &account_status)
                }
                Page::Packs | Page::Shaders => {
                    ctx.draw_catalog_page(&mut ui, content_area, compact, page, &account_status)
                }
                Page::Installed => {
                    ctx.draw_installed_page(&mut ui, content_area, compact, &account_status)
                }
                Page::VoltSettings => ctx.draw_volt_settings_page(&mut ui, content_area, compact),
                Page::Controls => ctx.draw_controls_page(&mut ui, content_area, compact),
                Page::Settings => {
                    ctx.draw_settings_page(&mut ui, content_area, compact, &account_status)
                }
            }
        }
        if let Some((anim_start, dir)) = self.page_anim {
            let el = anim_start.elapsed().as_secs_f32();
            if el < 0.25 {
                let ease = ease_out_cubic(el / 0.25);
                ui.animate_vertices(
                    page_v_start,
                    0.0,
                    dir * (1.0 - ease) * 18.0,
                    0.985 + 0.015 * ease,
                    [
                        content_area[0] + content_area[2] * 0.5,
                        content_area[1] + content_area[3] * 0.5,
                    ],
                    ease,
                );
            } else {
                self.page_anim = None;
            }
        }

        if compact && !lobby {
            let status = self.error.as_deref().unwrap_or(&self.catalog.status);
            if !status.is_empty() {
                let sfit = ui.fit_text(Face::LauncherRegular, 12.0, status, (w - 196.0).max(40.0));
                ui.text(Face::LauncherRegular, 12.0, 14.0, h - 32.0, theme::MUTED(), &sfit);
            }
            styled_button(
                &mut ui,
                &mut self.targets,
                [w - 178.0, h - 38.0, 78.0, 28.0],
                "Discord",
                Action::Discord,
                BtnStyle::CrimsonOutline,
                true,
            );
            styled_button(
                &mut ui,
                &mut self.targets,
                [w - 94.0, h - 38.0, 82.0, 28.0],
                "GitHub",
                Action::Repository,
                BtnStyle::Secondary,
                true,
            );
        }

        let modal_v_start = ui.batch_mut().vertices.len();
        if self.login.visible {
            self.list_targets.clear();
            self.list_view = None;
            self.login.draw(
                &mut ui,
                &mut self.targets,
                &self.catalog.config.name,
                self.catalog.editing == Some("name"),
            );
        }
        if let Some(chooser) = &mut self.chooser {
            chooser.draw(&mut ui, &mut self.targets);
            self.list_targets = chooser.navigation(ui.w, ui.h);
            self.list_view = Some(chooser.viewport(ui.h));
        } else if !self.login.visible
            && let Some(prompt) = &mut self.update_prompt
        {
            self.targets.clear();
            self.list_targets.clear();
            self.list_view = None;
            draw_update_prompt(&mut ui, &mut self.targets, prompt);
        } else if !self.login.visible && self.controls_studio.open {
            self.targets.clear();
            self.list_targets.clear();
            self.list_view = None;
            self.controls_studio.draw(&mut ui, &mut self.targets);
        } else if !self.login.visible && self.catalog.volt_boost_modal {
            self.targets.clear();
            self.list_targets.clear();
            self.list_view = draw_volt_boost_modal(
                &mut ui,
                &mut self.targets,
                &mut self.list_targets,
                &mut self.catalog,
            );
        }
        if (self.login.visible
            || self.chooser.is_some()
            || self.update_prompt.is_some()
            || self.catalog.volt_boost_modal
            || self.controls_studio.open)
            && let Some(anim_start) = self.modal_anim
        {
            let el = anim_start.elapsed().as_secs_f32();
            if el < 0.20 {
                let ease = ease_out_cubic(el / 0.20);
                // Fade backdrop smoothly while keeping full-bleed bounds; pop-scale modal card!
                let backdrop_end = (modal_v_start + 6).min(ui.batch_mut().vertices.len());
                for v in &mut ui.batch_mut().vertices[modal_v_start..backdrop_end] {
                    let base_a = (v.color >> 24) as f32;
                    let new_a = (base_a * ease).round().clamp(0.0, 255.0) as u32;
                    v.color = (v.color & 0x00ff_ffff) | (new_a << 24);
                }
                ui.animate_vertices(
                    backdrop_end,
                    0.0,
                    (1.0 - ease) * 14.0,
                    0.92 + 0.08 * ease,
                    [w * 0.5, h * 0.5],
                    ease,
                );
            } else {
                self.modal_anim = None;
            }
        }
        if let Some([top, bottom, maximum]) = self.list_view
            && maximum > 0.0
            && bottom > top
        {
            let scroll = if self.catalog.volt_boost_modal {
                self.catalog.volt_modal_scroll
            } else {
                self.chooser
                    .as_ref()
                    .map_or(self.catalog.scroll, |c| c.scroll)
            };
            let height = bottom - top;
            let thumb = (height * height / (height + maximum)).max(20.0).min(height);
            let y = top + (height - thumb) * (scroll / maximum).clamp(0.0, 1.0);
            ui.rounded(w - 8.0, top, 4.0, height, 2.0, theme::BORDER());
            ui.rounded(w - 8.0, y, 4.0, thumb, 2.0, theme::CRIMSON());
        }
        let caret_visible = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .is_ok_and(|t| t.as_millis() % 1000 < 500);
        if caret_visible && let Some(name) = self.catalog.editing {
            let (value, font_sz, pad_x) = match name {
                "name" => (self.catalog.config.name.clone(), 15.5, 14.0),
                "server" => (self.catalog.config.server.clone(), 15.5, 12.0),
                "discord_webhook" => (self.catalog.config.discord_webhook.clone(), 13.0, 12.0),
                "render_scale" => (self.catalog.render_scale_text.clone(), 15.5, 12.0),
                "offline_name" => (self.catalog.offline_name_input.clone(), 14.5, 12.0),
                "ely_user" => (self.catalog.ely_user_input.clone(), 14.5, 12.0),
                "ely_pass" => (
                    "*".repeat(self.catalog.ely_pass_input.chars().count()),
                    14.5,
                    12.0,
                ),
                "ely_totp" => (self.catalog.ely_totp_input.clone(), 14.5, 12.0),
                "skin_query" => (self.catalog.skin_query_input.clone(), 14.5, 12.0),
                "mods_search" => (self.catalog.mods_query.clone(), 13.5, 12.0),
                _ => (self.catalog.query.clone(), 14.5, 14.0),
            };
            let face = if name == "search" {
                Face::LauncherRegular
            } else {
                Face::LauncherMedium
            };
            for ([x, y, bw, bh], action) in &self.targets {
                if *action == Action::Field(name)
                    || (name == "render_scale" && *action == Action::RenderScale)
                {
                    let text = ui.fit_text(face, font_sz, &value, (bw - 28.0).max(8.0));
                    let caret_x = x + pad_x + ui.measure(face, font_sz, &text) + 1.0;
                    ui.rounded(
                        caret_x.min(x + bw - 8.0),
                        y + (bh - 18.0) * 0.5,
                        1.5,
                        18.0,
                        0.5,
                        theme::CRIMSON_BRIGHT(),
                    );
                }
            }
        }
        if let Some(action) = self.pending_focus.take() {
            self.focus = self.targets.iter().position(|(_, a)| *a == action);
        }
        if let Some(index) = self.focus
            && let Some(([x, y, bw, bh], _)) = self.targets.get(index)
        {
            ui.border(
                *x - 2.0,
                *y - 2.0,
                *bw + 4.0,
                *bh + 4.0,
                10.0,
                theme::CRIMSON_BRIGHT(),
            );
            self.cursor = (
                f64::from((x + bw / 2.0 + offset[0]) * ui.density),
                f64::from((y + bh / 2.0 + offset[1]) * ui.density),
            );
        }
        if self.mouse_down || self.drag.as_ref().is_some_and(|drag| !drag.moved) {
            let point = [
                self.cursor.0 as f32 / ui.density - offset[0],
                self.cursor.1 as f32 / ui.density - offset[1],
            ];
            if let Some(([x, y, bw, bh], _)) = self.targets.iter().find(|(rect, _)| {
                point[0] >= rect[0]
                    && point[0] <= rect[0] + rect[2]
                    && point[1] >= rect[1]
                    && point[1] <= rect[1] + rect[3]
            }) {
                ui.rounded(
                    *x,
                    *y,
                    *bw,
                    *bh,
                    8.0,
                    horizon_client::hud::rgba(0, 155, 230, 44),
                );
            }
        }
        if !self.mouse_down
            && self.drag.is_none()
            && let Some((action, time)) = self.feedback
            && time.elapsed() < std::time::Duration::from_millis(160)
            && let Some(([x, y, bw, bh], _)) = self.targets.iter().find(|(_, a)| *a == action)
        {
            ui.rounded(
                *x,
                *y,
                *bw,
                *bh,
                8.0,
                horizon_client::hud::rgba(0, 155, 230, 44),
            );
        }
        crate::documents::draw_ui(&mut ui);
        // Geometry and touch targets share the same translation. The scene is
        // deliberately excluded so it still covers the whole Android surface.
        if offset != [0.0, 0.0] {
            let dx = ui.to_batch(offset[0]);
            let dy = ui.to_batch(offset[1]);
            ui.batch_mut().translate_since(scene_end, dx, dy);
            for (rect, _) in self.targets.iter_mut().chain(self.list_targets.iter_mut()) {
                rect[0] += offset[0];
                rect[1] += offset[1];
            }
            if let Some(rect) = &mut self.preview_rect {
                rect[0] += offset[0];
                rect[1] += offset[1];
            }
            if let Some(view) = &mut self.list_view {
                view[0] += offset[1];
                view[1] += offset[1];
            }
        }
        ui.finish();

        if !r.render(&game::loading_frame(&b))? && !b.patches.is_empty() {
            // A suspended/lost surface skips the frame, including its glyph
            // uploads. Rebuild them next time instead of caching invisible text.
            horizon_client::ui_theme::reset_atlas();
        }
        Ok(())
    }

    pub(super) fn activate(&mut self, _events: &ActiveEventLoop) -> Result<()> {
        if self.session_worker.is_some() || self.pending_play {
            return Ok(());
        }
        let Some(r) = &self.renderer else {
            return Ok(());
        };
        let Some(gui) = &self.gui else {
            return Ok(());
        };
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        let ui = Ui::mobile(&gui.atlas, &mut b, (0.0, 0.0));
        let p = self.focus.and_then(|index| self.targets.get(index)).map_or(
            [
                self.cursor.0 as f32 / ui.density,
                self.cursor.1 as f32 / ui.density,
            ],
            |([x, y, w, h], _)| [x + w / 2.0, y + h / 2.0],
        );
        let action = self
            .targets
            .iter()
            .find(|([x, y, w, h], _)| {
                (*x..*x + *w).contains(&p[0]) && (*y..*y + *h).contains(&p[1])
            })
            .map(|(_, a)| *a);
        if self.catalog.busy() || crate::documents::busy() {
            return Ok(());
        }
        let now = std::time::Instant::now();
        if let Some(act) = action
            && !matches!(act, Action::Page(_))
            && self.transition_guard_until.is_some_and(|until| now < until)
        {
            self.focus = None;
            return Ok(());
        }
        let prev_view = (
            self.catalog.page,
            self.chooser.is_some(),
            self.login.visible,
            self.catalog.volt_boost_modal,
            self.controls_studio.open,
            self.update_prompt.is_some(),
        );
        self.feedback = action.map(|action| (action, now));
        self.focus = None;
        if action.is_some() {
            crate::native_ime::play_click_sound();
        }
        match action {
            Some(Action::ToggleAppearance) => {
                self.catalog.config.launcher_dark = !self.catalog.config.launcher_dark;
                self.catalog.config.save(&self.home)?;
            }
            Some(Action::ToggleLiquid) => {
                self.catalog.config.launcher_liquid = !self.catalog.config.launcher_liquid;
                self.catalog.config.save(&self.home)?;
            }
            Some(Action::CycleAccent) => {
                const COLORS: [u32; 8] = [0x79e59c, 0x5b67f2, 0x3b82f6, 0x0e9fbd, 0xe6ab32, 0xf26c5b, 0xd755ae, 0x8b5cf6];
                let current = self.catalog.config.launcher_accent;
                let index = COLORS.iter().position(|&c| c == current).map_or(0, |i| (i + 1) % COLORS.len());
                self.catalog.config.launcher_accent = COLORS[index];
                self.catalog.config.save(&self.home)?;
            }
            Some(Action::Immersive) => {
                self.catalog.config.screen.immersive ^= true;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
            }
            Some(Action::Cutout) => {
                self.catalog.config.screen.avoid_cutout ^= true;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
            }
            Some(Action::Aspect) => self.open_chooser(chooser::Kind::Aspect),
            Some(Action::ChooseAspect(aspect)) => {
                self.catalog.config.screen.aspect = aspect;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
                self.chooser = None;
            }
            Some(Action::DisplayHz) => self.open_chooser(chooser::Kind::DisplayHz),
            Some(Action::ChooseDisplayHz(hz)) => {
                self.catalog.config.device.display_hz = hz;
                self.catalog.config.device.sanitize();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
                self.chooser = None;
            }
            Some(Action::PerformanceHints) => {
                self.catalog.config.device.performance_hints ^= true;
                self.catalog.status = "ADPF gives Android CPU frame-work hints on supported devices. It does not cap FPS or guarantee a speedup.".into();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
            }
            Some(Action::GpuProfile) => {
                self.catalog.config.device.gpu_profile ^= true;
                self.catalog.status = "Detailed GPU profiling records diagnostics and has overhead. Reports remain on this device.".into();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
            }
            Some(Action::ModSource(origin)) => {
                self.catalog.config.mod_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.page_anim = Some((std::time::Instant::now(), 0.45));
                self.catalog.browse(&self.home);
            }
            Some(Action::ShaderSource(origin)) => {
                self.catalog.config.shader_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.page_anim = Some((std::time::Instant::now(), 0.45));
                self.catalog.browse(&self.home);
            }
            Some(Action::PackSource(origin)) => {
                self.catalog.config.pack_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.page_anim = Some((std::time::Instant::now(), 0.45));
                self.catalog.browse(&self.home);
            }
            Some(Action::OpenProject(i)) => {
                if let Some(url) = self.catalog.results.get(i).map(|p| p.page.as_str()) {
                    self.catalog.status = if crate::platform::open_url(url) {
                        "Opened project page in browser.".into()
                    } else {
                        "Could not open project URL.".into()
                    };
                }
            }
            Some(Action::ViewDetail(i)) => {
                self.catalog.detail_index = Some(i);
                self.catalog.detail_tab = 0;
                self.catalog.scroll = 0.0;
            }
            Some(Action::CloseDetail) => {
                self.catalog.detail_index = None;
            }
            Some(Action::DetailTab(tab)) => {
                self.catalog.detail_tab = tab;
                self.catalog.scroll = 0.0;
            }
            Some(Action::ViewScreenshot(i)) => {
                self.catalog.viewer_shot = Some(i);
            }
            Some(Action::ViewWorld(i)) => {
                self.catalog.viewer_world = Some(i);
            }
            Some(Action::CloseViewer) => {
                self.catalog.viewer_shot = None;
                self.catalog.viewer_world = None;
            }
            Some(Action::OpenFolder(kind)) => {
                crate::platform::hide();
                let folder = match kind {
                    horizon_client::content_files::Kind::Mods => "mods",
                    horizon_client::content_files::Kind::Packs => "packs",
                    horizon_client::content_files::Kind::Shaders => "shaders",
                };
                let id = crate::document_store::folder_id(self.catalog.config.profile, folder);
                self.catalog.status = if crate::platform::open_documents(&id) {
                    "Opened in Files. Return here after copying files.".into()
                } else {
                    "Open the system Files app and choose Volt Launcher.".into()
                };
            }
            Some(Action::Import(kind)) => {
                crate::platform::hide();
                let home = self.catalog.config.game_home(&self.home);
                if let Err(e) = crate::documents::begin(kind, home) {
                    self.catalog.status = e;
                }
            }
            Some(Action::ImportControl) => {
                crate::platform::hide();
                self.chooser = None;
                let profile_home = self.catalog.config.game_home(&self.home);
                if let Err(e) = crate::documents::begin_controls(self.home.clone(), profile_home) {
                    self.catalog.status = e;
                }
            }
            Some(Action::ControlLayouts) => {
                self.catalog.refresh_controls(&self.home);
                self.open_chooser(chooser::Kind::ControlLayout);
            }
            Some(Action::ChooseControlLayout(idx)) => {
                self.chooser = None;
                if let Some(preset) = self.catalog.control_presets.get(idx).cloned() {
                    let profile_home = self.catalog.config.game_home(&self.home);
                    match horizon_client::touch::import::load_preset(&self.home, &preset.slug)
                        .and_then(|s| {
                            horizon_client::touch::import::apply_to_profile(&profile_home, &s)
                        }) {
                        Ok(()) => {
                            if matches!(
                                self.catalog.config.input_host,
                                crate::catalog::InputHost::KeyboardMouse
                                    | crate::catalog::InputHost::Controller
                            ) {
                                self.catalog.config.input_host = crate::catalog::InputHost::Auto;
                                let _ = self.catalog.config.save(&self.home);
                                crate::configure_input_mode(self.catalog.config.input_host);
                            }
                            self.catalog.refresh_controls(&self.home);
                            self.catalog.status = format!(
                                "Activated \"{}\" ({}) for Profile {}.",
                                preset.meta.name,
                                preset.meta.format,
                                self.catalog.config.profile
                            );
                        }
                        Err(e) => self.catalog.status = format!("Could not apply preset: {e}"),
                    }
                }
            }
            Some(Action::ApplyControlsAllProfiles) => {
                let profile_home = self.catalog.config.game_home(&self.home);
                let current = horizon_client::touch::Settings::load_from(
                    &profile_home.join("horizon-touch.json"),
                );
                match horizon_client::touch::import::apply_to_all_profiles(&self.home, &current) {
                    Ok(count) => {
                        self.catalog.refresh_controls(&self.home);
                        self.catalog.status = format!(
                            "Applied active touch control layout across {count} profile(s)."
                        );
                    }
                    Err(e) => self.catalog.status = format!("Apply failed: {e}"),
                }
            }
            Some(Action::ResetControls) => {
                self.chooser = None;
                let profile_home = self.catalog.config.game_home(&self.home);
                match horizon_client::touch::import::reset_profile(&profile_home) {
                    Ok(()) => {
                        self.catalog.refresh_controls(&self.home);
                        self.catalog.status = format!(
                            "Restored default Volt touch controls for Profile {}.",
                            self.catalog.config.profile
                        );
                    }
                    Err(e) => self.catalog.status = format!("Reset failed: {e}"),
                }
            }
            Some(Action::DeleteControlPreset(idx)) => {
                self.chooser = None;
                if let Some(preset) = self.catalog.control_presets.get(idx).cloned() {
                    match horizon_client::touch::import::delete_preset(&self.home, &preset.slug) {
                        Ok(()) => {
                            self.catalog.refresh_controls(&self.home);
                            self.catalog.status =
                                format!("Deleted control layout \"{}\".", preset.meta.name);
                        }
                        Err(e) => self.catalog.status = e,
                    }
                }
            }
            Some(Action::ToggleSection(i)) => {
                self.catalog.sections[i] = !self.catalog.sections[i];
                self.catalog.scroll = 0.0;
            }
            Some(Action::ToggleLocal(i)) => self.catalog.toggle_local(&self.home, i),
            Some(Action::DeleteLocal(i)) => {
                self.catalog.pending_delete = self.catalog.local.get(i).cloned();
                if let Some(item) = &self.catalog.pending_delete {
                    self.chooser = Some(super::chooser::Chooser::confirm_delete(item.file.clone()));
                }
            }
            Some(Action::ConfirmDeleteLocal) => {
                self.catalog.remove_local(&self.home);
                self.chooser = None;
            }
            Some(Action::PatchNotes) => {
                self.catalog.status = if crate::platform::open_url(
                    "https://github.com/entitybrian69-bit/Volt-launcher/releases",
                ) {
                    "Volt Launcher patch notes opened in browser."
                } else {
                    "No browser available to open the patch notes."
                }
                .into();
            }
            Some(Action::Repository) => {
                self.catalog.status = if crate::platform::open_repository() {
                    "Volt Launcher GitHub opened in browser."
                } else {
                    "No browser available to open GitHub."
                }
                .into();
            }
            Some(Action::Discord) => {
                self.catalog.status = if crate::platform::open_discord() {
                    "Volt Launcher Discord opened in browser."
                } else {
                    "No browser available to open Discord."
                }
                .into();
            }
            Some(Action::AboutWebsite) => {
                self.catalog.status = if crate::platform::open_website() {
                    "Volt Launcher website opened in browser."
                } else {
                    "No browser available to open the website."
                }
                .into();
            }
            Some(Action::AboutOwner) => {
                self.catalog.status = if crate::platform::open_owner_page() {
                    "Owner page opened in browser."
                } else {
                    "No browser available to open the owner page."
                }
                .into();
            }
            Some(Action::PresenceStatus) => {
                let status = crate::discord_rpc::status_text();
                self.catalog.status = match status.as_str() {
                    s if s.starts_with("Connected") => {
                        "Rich Presence is live through the Discord app on this phone.".into()
                    }
                    "Unavailable on this device" => {
                        "This build has no Discord presence bridge.".into()
                    }
                    // The bridge diagnosed a concrete failure: show it verbatim.
                    s if s.contains(" - ") => status.clone(),
                    _ => {
                        "Install or update the Discord app and stay signed in; presence retries every 15 seconds.".into()
                    }
                };
            }
            Some(Action::CheckUpdates) => {
                if self.manual_check_rx.is_none() {
                    self.catalog.status = "Checking for updates…".into();
                    self.manual_check_rx =
                        Some(crate::volt_updater::spawn_manual_check(self.home.clone()));
                }
            }
            Some(Action::AccountProviderTab(prov)) => {
                crate::platform::hide();
                self.catalog.editing = None;
                self.catalog.account_tab = prov;
                self.page_anim = Some((std::time::Instant::now(), 0.40));
            }
            Some(Action::SelectVoltAccount(idx)) => {
                if let Some(acc) = self.volt_accounts.accounts.get(idx).cloned() {
                    self.volt_accounts.select(&acc.id);
                    self.catalog.config.name = acc.name.clone();
                    self.catalog.offline_name_input = acc.name;
                    self.catalog.account_tab = acc.provider;
                    self.catalog.config.save(&self.home)?;
                }
            }
            Some(Action::RemoveVoltAccount(idx)) => {
                if let Some(id) = self
                    .volt_accounts
                    .accounts
                    .get(idx)
                    .map(|a| a.id.clone())
                {
                    if self.volt_accounts.remove_account(&id).is_some() {
                        self.login.sign_out();
                        self.login.visible = false;
                    }
                    if let Some(active) = self.volt_accounts.active() {
                        self.catalog.config.name = active.name.clone();
                        self.catalog.offline_name_input = active.name.clone();
                        self.catalog.config.save(&self.home)?;
                    }
                }
            }
            Some(Action::CreateOfflineAccount) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let input = self.catalog.offline_name_input.clone();
                match self.volt_accounts.create_offline(&input) {
                    Ok(()) => {
                        if let Some(acc) = self.volt_accounts.active() {
                            self.catalog.config.name = acc.name.clone();
                            self.catalog.config.save(&self.home)?;
                        }
                        self.catalog.status = "Offline account active.".into();
                    }
                    Err(e) => self.catalog.status = e,
                }
            }
            Some(Action::UpdateOfflineName) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let input = self.catalog.offline_name_input.clone();
                match self.volt_accounts.rename_active_offline(&input) {
                    Ok(()) => {
                        if let Some(acc) = self.volt_accounts.active() {
                            self.catalog.config.name = acc.name.clone();
                            self.catalog.config.save(&self.home)?;
                        }
                        self.catalog.status = "Offline username updated.".into();
                    }
                    Err(e) => self.catalog.status = e,
                }
            }
            Some(Action::ElySignIn) => {
                crate::platform::hide();
                self.catalog.editing = None;
                self.volt_accounts.begin_ely_login(
                    &self.catalog.ely_user_input,
                    &self.catalog.ely_pass_input,
                    &self.catalog.ely_totp_input,
                );
                self.catalog.ely_pass_input.clear();
                self.catalog.ely_totp_input.clear();
            }
            Some(Action::ElyRegister) => {
                let _ = crate::platform::open_ely_register();
            }
            Some(Action::ToggleSlimModel) => {
                self.volt_accounts.toggle_slim();
            }
            Some(Action::ToggleServerSkinSync) => {
                self.volt_accounts.toggle_server_sync();
            }
            Some(Action::ImportSkinPng) => {
                if let Some(id) = self.volt_accounts.active().map(|a| a.id.clone()) {
                    let dest = self.volt_accounts.skin_png_path(&id);
                    if let Err(e) = crate::documents::begin_cosmetic(false, dest) {
                        self.catalog.status = e;
                    }
                }
            }
            Some(Action::ImportCapePng) => {
                if let Some(id) = self.volt_accounts.active().map(|a| a.id.clone()) {
                    let dest = self.volt_accounts.cape_png_path(&id);
                    if let Err(e) = crate::documents::begin_cosmetic(true, dest) {
                        self.catalog.status = e;
                    }
                }
            }
            Some(Action::FetchSkinByUsername) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let q = self.catalog.skin_query_input.clone();
                self.volt_accounts.begin_fetch_skin_by_username(&q);
            }
            Some(Action::LoadSkinUrl) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let q = self.catalog.skin_query_input.clone();
                self.volt_accounts.begin_fetch_cosmetic_url(&q, false);
            }
            Some(Action::LoadCapeUrl) => {
                crate::platform::hide();
                self.catalog.editing = None;
                let q = self.catalog.skin_query_input.clone();
                self.volt_accounts.begin_fetch_cosmetic_url(&q, true);
            }
            Some(Action::EquipPresetCape(preset)) => {
                if let Err(e) = self.volt_accounts.apply_preset_cape(preset) {
                    self.catalog.status = e;
                } else {
                    // Smoothly spin character toward back so user watches their 3D cape billow into view!
                    self.preview_target_yaw = Some(162.0);
                    self.catalog.status = format!("Equipped {}!", preset.label());
                }
            }
            Some(Action::ClearCustomSkin) => {
                self.volt_accounts.clear_skin();
            }
            Some(Action::ClearCustomCape) => {
                self.volt_accounts.clear_cape();
            }
            Some(Action::ResetPreviewAngle) => {
                let normalized = (self.preview_yaw + 180.0).rem_euclid(360.0) - 180.0;
                let target = if normalized.abs() < 90.0 { 162.0 } else { 342.0 };
                self.preview_target_yaw = Some(target);
                self.preview_pitch = 6.0;
            }
            Some(Action::Account) => {
                self.login.visible = true;
            }
            Some(Action::CloseLogin) => {
                self.login.hide();
                crate::platform::hide();
                self.catalog.editing = None;
            }
            Some(Action::DeviceSignIn) => self
                .login
                .start_device(&self.catalog.config.microsoft_client_id),
            Some(Action::CopyLoginCode) => self.login.copy_code(),
            Some(Action::CancelLogin) => {
                self.login.close();
                self.login.visible = true;
            }
            Some(Action::ReviewApp) => self.login.review_app(),
            Some(Action::SignIn) => self.login.start(&self.catalog.config.microsoft_client_id),
            Some(Action::SignOut) => {
                self.login.sign_out();
                if self.catalog.page == Page::Account {
                    self.login.visible = false;
                }
                self.volt_accounts
                    .sync_msa_accounts(&crate::account::Accounts::load(), false);
            }
            Some(Action::LoginLink) => self.login.open_link(),
            Some(Action::HomeTab(tab)) => {
                if self.catalog.page == Page::Play && tab != self.home_tab {
                    let order = |t: HomeTab| match t {
                        HomeTab::Overview => 0,
                        HomeTab::Worlds => 2,
                        HomeTab::Multiplayer => 3,
                        HomeTab::Replays => 4,
                        HomeTab::Screenshots => 5,
                        HomeTab::Console => 6,
                    };
                    let dir = if order(tab) >= order(self.home_tab) { 1.0 } else { -1.0 };
                    self.tab_anim = Some((std::time::Instant::now(), dir));
                }
                if tab == HomeTab::Multiplayer {
                    let home = self.catalog.config.game_home(&self.home);
                    self.catalog.saved_servers = crate::server_list::List::load(&home).servers();
                } else if tab == HomeTab::Screenshots {
                    self.catalog.refresh_screenshots(&self.home);
                } else if tab == HomeTab::Replays {
                    self.catalog.refresh_replays(&self.home);
                }
                self.home_tab = tab;
                self.catalog.page = Page::Play;
                self.catalog.scroll = 0.0;
                self.catalog.editing = None;
                self.catalog.viewer_shot = None;
                self.catalog.viewer_world = None;
                if crate::platform::active() { crate::platform::hide(); }
            }
            Some(Action::Page(page)) => {
                if page == Page::Play { self.home_tab = HomeTab::Overview; }
                self.catalog.volt_boost_modal = false;
                self.controls_studio.open = false;
                self.catalog.editing = None;
                if crate::platform::active() {
                    crate::platform::hide();
                }
                self.catalog.page = page;
                self.catalog.scroll = 0.0;
                self.error = None;
                self.catalog.offset = 0;
                if page == Page::Mods {
                    self.catalog.query.clear();
                }
                if page == Page::Controls {
                    self.catalog.refresh_controls(&self.home);
                }
                self.catalog.browse(&self.home);
            }
            Some(Action::SaveServer) => {
                let home = self.catalog.config.game_home(&self.home);
                let address = self.catalog.config.server.trim().to_owned();
                let result = crate::server_list::List::try_load(&home).and_then(|mut list| {
                    if list.servers().iter().any(|server| server.ip.eq_ignore_ascii_case(&address)) {
                        return Err("This server is already saved.".into());
                    }
                    list.add(&address, &address)?;
                    list.save(&home)
                });
                self.catalog.status = match result {
                    Ok(()) => {
                        self.catalog.saved_servers = crate::server_list::List::load(&home).servers();
                        "Server saved to this instance.".into()
                    }
                    Err(error) => error,
                };
            }
            Some(Action::ConnectServer(index)) => {
                if let Some(server) = self.catalog.saved_servers.get(index) {
                    self.catalog.config.server = server.ip.clone();
                    self.catalog.config.replay.clear();
                    self.catalog.config.save(&self.home)?;
                    self.pending_play = true;
                    self.needs_redraw = true;
                } else {
                    self.catalog.status = "Server list changed; please choose again.".into();
                }
            }
            Some(Action::WatchReplay(index)) => {
                if let Some(replay) = self.catalog.replays.get(index) {
                    self.catalog.config.replay = replay.path.to_string_lossy().into_owned();
                    self.catalog.config.replay_export = false;
                    self.catalog.config.save(&self.home)?;
                    self.pending_play = true;
                    self.needs_redraw = true;
                } else {
                    self.catalog.status = "Replay list changed; please choose again.".into();
                }
            }
            Some(Action::ExportReplay(index)) => {
                if let Some(replay) = self.catalog.replays.get(index) {
                    // One tap: watch the replay with the MP4 export already
                    // running; the file lands in recordings/ with the other
                    // captures (Captures panel / Files app).
                    self.catalog.config.replay = replay.path.to_string_lossy().into_owned();
                    self.catalog.config.replay_export = true;
                    self.catalog.config.save(&self.home)?;
                    self.pending_play = true;
                    self.needs_redraw = true;
                } else {
                    self.catalog.status = "Replay list changed; please choose again.".into();
                }
            }
            Some(Action::ToggleVoltReplay) => {
                let on = !self.catalog.extras.volt_qol.replay_buffer;
                self.catalog.extras.volt_qol.replay_buffer = on;
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                self.catalog.extras.save(&game_dir);
                crate::documents::note_mods_changed(&game_dir);
                self.catalog.status = if on {
                    "Volt Replay on: sessions are recorded. Watch them from Home > Replays.".into()
                } else {
                    "Volt Replay off: sessions are no longer recorded.".into()
                };
            }
            Some(Action::ToggleReplayButton) => {
                let on = !self.catalog.extras.volt_qol.replay_button;
                self.catalog.extras.volt_qol.replay_button = on;
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                self.catalog.extras.save(&game_dir);
                self.catalog.status = if on {
                    "Replay button on: drag it anywhere in game; hold it to resize or fade it; tap for Start / Pause / End.".into()
                } else {
                    "Replay button off.".into()
                };
            }
            Some(Action::DeleteReplay(index)) => {
                if let Some(replay) = self.catalog.replays.get(index) {
                    self.catalog.status = match std::fs::remove_file(&replay.path) {
                        Ok(()) => "Replay deleted.".into(),
                        Err(error) => format!("Could not delete the replay: {error}"),
                    };
                    self.catalog.refresh_replays(&self.home);
                }
            }
            Some(Action::OpenCaptures) => {
                crate::platform::hide();
                let id = crate::document_store::folder_id(self.catalog.config.profile, "recordings");
                self.catalog.status = if crate::platform::open_documents(&id) {
                    "Opened the recordings folder in Files. Tap an MP4 to play it.".into()
                } else {
                    "Open the system Files app and choose Volt Launcher > Recordings.".into()
                };
            }
            Some(Action::DeleteRecording(index)) => {
                if let Some((name, _)) = self.catalog.recordings.get(index) {
                    let path = self.catalog.config.game_home(&self.home)
                        .join("recordings").join(name);
                    self.catalog.status = match std::fs::remove_file(&path) {
                        Ok(()) => "Recording deleted.".into(),
                        Err(error) => format!("Could not delete the recording: {error}"),
                    };
                    self.catalog.refresh_screenshots(&self.home);
                }
            }
            Some(Action::Play) => {
                // An ordinary Play never resumes a replay.
                if !self.catalog.config.replay.is_empty() {
                    self.catalog.config.replay.clear();
                    self.catalog.config.save(&self.home)?;
                }
                // Remember the launch for the hero subtitle; the elapsed session
                // is folded into play_seconds when the launcher next loads.
                let now_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                if now_secs > 0 {
                    self.catalog.config.last_played = now_secs;
                    self.catalog.config.session_mark = now_secs;
                    self.catalog.config.save(&self.home)?;
                }
                self.pending_play = true;
                self.needs_redraw = true;
            }
            Some(Action::Search) => {
                crate::platform::hide();
                self.catalog.offset = 0;
                self.catalog.search(&self.home);
            }
            Some(Action::FinishNameEdit) => {
                self.catalog.editing = None;
                crate::platform::hide();
                self.catalog.config.save(&self.home)?;
            }
            Some(Action::Field(name)) => {
                if name == "render_scale" {
                    self.catalog.render_scale_text =
                        self.catalog.config.render_scale.percent().to_string();
                    self.chooser = None;
                }
                self.catalog.editing = Some(name);
                let initial = match name {
                    "name" => &self.catalog.config.name,
                    "server" => &self.catalog.config.server,
                    "discord_webhook" => &self.catalog.config.discord_webhook,
                    "render_scale" => &self.catalog.render_scale_text,
                    "client_id" => &self.catalog.config.microsoft_client_id,
                    "offline_name" => &self.catalog.offline_name_input,
                    "ely_user" => &self.catalog.ely_user_input,
                    "ely_pass" => &self.catalog.ely_pass_input,
                    "ely_totp" => &self.catalog.ely_totp_input,
                    "skin_query" => &self.catalog.skin_query_input,
                    "mods_search" => &self.catalog.mods_query,
                    _ => &self.catalog.query,
                };
                crate::platform::show(initial);
            }
            Some(Action::Install(i)) => self.catalog.install(&self.home, i),
            Some(Action::Profile) => self.open_chooser(chooser::Kind::Profile),
            Some(Action::Renderer) => self.open_chooser(chooser::Kind::Renderer),
            Some(Action::RenderScale) => self.open_chooser(chooser::Kind::RenderScale),
            Some(Action::ChooseRenderScale(scale)) => {
                self.catalog.config.render_scale = scale;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::display::set_scale(scale);
            }
            Some(Action::Orientation) => self.open_chooser(chooser::Kind::Orientation),
            Some(Action::InputHost) => self.open_chooser(chooser::Kind::InputHost),
            Some(Action::ChooseInputHost(mode)) => {
                self.catalog.config.input_host = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::configure_input_mode(mode);
            }
            Some(Action::ChooseOrientation(mode)) => {
                self.catalog.config.orientation = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::platform::set_orientation(mode);
            }
            Some(Action::ChooseRenderer(mode)) => {
                self.catalog.config.renderer = mode;
                self.catalog.config.save(&self.home)?;
                horizon_client::android_runtime::configure_graphics_api(mode.graphics_api());
                self.chooser = None;
            }
            Some(Action::ImportVulkanDriver) => {
                if let Err(err) = crate::documents::begin_vulkan_driver(self.home.clone()) {
                    self.catalog.status = err;
                }
            }
            Some(Action::ResetVulkanDriver) => {
                self.catalog.config.custom_vulkan_driver = None;
                self.catalog.config.custom_vulkan_driver_name = None;
                self.catalog.config.save(&self.home)?;
                horizon_client::android_runtime::configure_custom_vulkan_driver(None);
                self.catalog.status =
                    "Switched back to Volt's built-in Vulkan Supporter driver.".into();
            }
            Some(Action::InstallUpdate) => {
                // The download runs in the user's browser on purpose: Volt never
                // asks for REQUEST_INSTALL_PACKAGES or drives PackageInstaller,
                // so Play Protect has no "can install other apps" signal to warn
                // about when players sideload the launcher.
                if let Some(prompt) = self.update_prompt.take() {
                    crate::volt_updater::mark_update_seen(&self.home, &prompt.info.tag_name);
                    self.catalog.status = if crate::platform::open_url(&prompt.info.download_url) {
                        format!(
                            "Downloading {} ({}) in your browser — open the APK when it finishes to update.",
                            prompt.info.display_version, prompt.info.abi_label
                        )
                    } else {
                        "Could not open a browser for the update download.".into()
                    };
                }
            }
            Some(Action::CancelUpdate) => {
                if let Some(prompt) = self.update_prompt.take() {
                    crate::volt_updater::mark_update_seen(&self.home, &prompt.info.tag_name);
                }
            }
            Some(Action::ChooseProfile(profile)) => {
                std::fs::create_dir_all(if profile == 0 {
                    self.home.clone()
                } else {
                    self.home
                        .join("profiles")
                        .join(format!("profile-{profile}"))
                })?;
                self.catalog.config.profile = profile;
                self.catalog.config.save(&self.home)?;
                self.catalog.refresh(&self.home);
                if self.home_tab == HomeTab::Multiplayer {
                    let home = self.catalog.config.game_home(&self.home);
                    self.catalog.saved_servers = crate::server_list::List::load(&home).servers();
                } else if self.home_tab == HomeTab::Screenshots {
                    self.catalog.refresh_screenshots(&self.home);
                }
                self.chooser = None;
            }
            Some(Action::Dismiss) => self.chooser = None,
            Some(Action::Logs) => {
                self.catalog.status = crate::diagnostics::summary(&self.home);
                if let Ok(error) = std::fs::read_to_string(self.home.join("android-error.txt")) {
                    self.catalog.status = format!(
                        "Last handled error: {}. {}",
                        crate::diagnostics::redact(&error),
                        self.catalog.status
                    );
                }
            }
            Some(Action::ExportDiagnostics) if self.diagnostic_export.is_none() => {
                let home = self.home.clone();
                let (sender, receiver) = mpsc::channel();
                self.diagnostic_export = Some(receiver);
                self.catalog.status =
                    "Preparing local diagnostic ZIP. No upload; review before sharing.".into();
                if let Err(err) = std::thread::Builder::new()
                    .name("volt-diag-export".into())
                    .stack_size(4 * 1024 * 1024)
                    .spawn(move || {
                        let result = crate::diagnostics::export(&home).map(|p| {
                            p.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned()
                        });
                        let _ = sender.send(result);
                    })
                {
                    self.diagnostic_export = None;
                    self.catalog.status = format!("Diagnostic export failed: {err}");
                }
            }
            Some(Action::ExportDiagnostics) => {}
            Some(Action::RecordPerformance) => {
                self.catalog.config.record_performance = !self.catalog.config.record_performance;
                if !self.catalog.config.record_performance {
                    crate::performance::stop();
                }
                self.catalog.config.save(&self.home)?;
                self.catalog.status = if self.catalog.config.record_performance {
                    "Recording starts on Play. Detailed timings add overhead. Return here to save the report."
                } else {
                    "Performance recording off."
                }
                .into();
            }
            Some(Action::ExportPerformance) => {
                self.catalog.status = match crate::performance::export(&self.home) {
                    Ok(()) => "Reports opened in Files. Copy or share a report from there.".into(),
                    Err(e) => e,
                };
            }
            Some(Action::OpenVoltBoost) => {
                self.catalog.volt_boost_modal = true;
                self.catalog.volt_modal_scroll = 0.0;
            }
            Some(Action::CloseVoltBoost) => {
                self.catalog.volt_boost_modal = false;
            }
            Some(Action::VoltPreset(preset)) => {
                self.catalog.extras.apply_volt_preset(preset);
                self.catalog.extras.distant_lod_boost = false;
                horizon_threads::set_thread_boost(self.catalog.extras.thread_boost);
                self.configure_screen();
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                self.catalog.extras.save(&game_dir);
                // The FPS profiles also release vanilla's VSync (Fifo) and
                // 120-FPS defaults in this instance's options.txt, which
                // otherwise cap every phone at its panel's refresh rate.
                if matches!(
                    preset,
                    horizon_client::volt_features::VoltBoostPreset::Competitive
                        | horizon_client::volt_features::VoltBoostPreset::UltraFps
                ) {
                    let options = game_dir.join("options.txt");
                    let mut vanilla = horizon_client::settings::Settings::load(&options);
                    if vanilla.vsync || vanilla.max_fps < 260 {
                        vanilla.vsync = false;
                        vanilla.max_fps = vanilla.max_fps.max(260);
                        vanilla.save(&options);
                    }
                }
                let mut gfx = horizon_client::graphics::Graphics::load(&game_dir);
                if gfx.lod_distance != 0 {
                    gfx.lod_distance = 0;
                    gfx.save(&game_dir);
                }
                self.catalog.status = format!(
                    "Volt Boost profile set to {} ({} optimizations active).",
                    preset.label(),
                    self.catalog.extras.active_volt_optimizations()
                );
            }
            Some(Action::VoltExtra(item)) => {
                item.cycle(&mut self.catalog.extras, false);
                self.catalog.extras.purge_ram_requested = false;
                horizon_threads::set_thread_boost(self.catalog.extras.thread_boost);
                self.configure_screen();
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                self.catalog.extras.save(&game_dir);
                if matches!(
                    item,
                    horizon_client::extras::Item::VoltPreset
                        | horizon_client::extras::Item::VoltAutoOptimize
                ) {
                    let mut gfx = horizon_client::graphics::Graphics::load(&game_dir);
                    if gfx.lod_distance != 0 {
                        gfx.lod_distance = 0;
                        gfx.save(&game_dir);
                    }
                }
                if item == horizon_client::extras::Item::VoltPurgeRam {
                    let rss = horizon_threads::resident_ram_mib().unwrap_or(0);
                    self.catalog.status =
                        format!("Purged unused allocator pages (Resident RAM: {rss} MiB).");
                } else if item == horizon_client::extras::Item::VoltAutoOptimize {
                    self.catalog.status = format!(
                        "Auto-tuned device to {} ({} optimizations active).",
                        self.catalog.extras.volt_preset.label(),
                        self.catalog.extras.active_volt_optimizations()
                    );
                }
            }
            Some(Action::ToggleBundled(i)) => {
                if let Some(&(id, version, _)) = horizon_client::bundled_native::BUNDLED.get(i) {
                    let game_dir = self.catalog.config.game_home(&self.home);
                    let _ = std::fs::create_dir_all(&game_dir);
                    let install = !self.catalog.bundled_mods.contains(id);
                    match horizon_client::bundled_native::set_installed(&game_dir, id, install) {
                        Ok(()) => {
                            if install {
                                self.catalog.bundled_mods.insert(id.to_owned());
                            } else {
                                self.catalog.bundled_mods.remove(id);
                            }
                            if id == "zoom" {
                                self.catalog.bundled_zoom_installed = install;
                                self.catalog.bundled_zoom_enabled = install;
                            }
                            crate::documents::note_mods_changed(&game_dir);
                            self.catalog.status = if install {
                                format!(
                                    "{} {version} installed from the APK. Toggle and configure it in-game under Mods.",
                                    bundled_mod_title(id)
                                )
                            } else {
                                format!(
                                    "{} removed from this profile; its settings are kept.",
                                    bundled_mod_title(id)
                                )
                            };
                        }
                        Err(e) => {
                            self.catalog.status =
                                format!("Could not change {}: {e}", bundled_mod_title(id));
                        }
                    }
                }
            }
            Some(Action::ToggleWoven(i)) => {
                if let Some(&(id, version)) = horizon_client::bundled_native::WOVEN.get(i) {
                    let game_dir = self.catalog.config.game_home(&self.home);
                    let _ = std::fs::create_dir_all(&game_dir);
                    let mut on = horizon_client::mod_list::ModList::load(&game_dir);
                    let enable = on.disabled.contains(id);
                    if enable {
                        on.disabled.remove(id);
                        on.enabled.insert(id.to_owned());
                    } else {
                        on.enabled.remove(id);
                        on.disabled.insert(id.to_owned());
                    }
                    if let Err(e) = on.try_save(&game_dir) {
                        self.catalog.status = format!("Could not save the mod list: {e}");
                        return Ok(());
                    }
                    self.catalog.bundled_disabled = on.disabled.clone();
                    crate::documents::note_mods_changed(&game_dir);
                    self.catalog.status = if enable {
                        format!(
                            "{} {version} is on for this profile. Configure it in-game under Mods.",
                            bundled_mod_title(id)
                        )
                    } else {
                        format!(
                            "{} {version} is off for this profile; its settings are kept.",
                            bundled_mod_title(id)
                        )
                    };
                }
            }
            Some(Action::ModsAllOn) | Some(Action::ModsAllOff) => {
                let on = action == Some(Action::ModsAllOn);
                let game_dir = self.catalog.config.game_home(&self.home);
                let _ = std::fs::create_dir_all(&game_dir);
                let mut list = horizon_client::mod_list::ModList::load(&game_dir);
                let mut flip = |id: &str| {
                    if on {
                        list.disabled.remove(id);
                        list.enabled.insert(id.to_owned());
                    } else {
                        list.enabled.remove(id);
                        list.disabled.insert(id.to_owned());
                    }
                };
                for &(id, _) in horizon_client::bundled_native::WOVEN {
                    flip(id);
                }
                for &(id, _, _) in horizon_client::bundled_native::BUNDLED {
                    if self.catalog.bundled_mods.contains(id) {
                        flip(id);
                    }
                }
                if let Err(e) = list.try_save(&game_dir) {
                    self.catalog.status = format!("Could not save the mod list: {e}");
                    return Ok(());
                }
                self.catalog.bundled_disabled = list.disabled.clone();
                crate::documents::note_mods_changed(&game_dir);
                self.catalog.bundled_zoom_enabled =
                    self.catalog.bundled_zoom_installed && !list.disabled.contains("zoom");
                self.catalog.extras.volt_qol.replay_buffer = on;
                self.catalog.extras.save(&game_dir);
                self.catalog.status = if on {
                    "All mods are on for this profile.".into()
                } else {
                    "All mods are off for this profile; their settings are kept.".into()
                };
            }
            Some(Action::ModsSort) => {
                self.catalog.mods_sort = (self.catalog.mods_sort + 1) % 3;
            }
            Some(Action::Studio(sact)) => {
                let profile_home = self.catalog.config.game_home(&self.home);
                if let Some(msg) =
                    self.controls_studio
                        .handle_action(sact, &self.home, &profile_home)
                {
                    if matches!(
                        self.catalog.config.input_host,
                        crate::catalog::InputHost::KeyboardMouse
                            | crate::catalog::InputHost::Controller
                    ) {
                        self.catalog.config.input_host = crate::catalog::InputHost::Auto;
                        let _ = self.catalog.config.save(&self.home);
                        crate::configure_input_mode(self.catalog.config.input_host);
                    }
                    self.catalog.refresh_controls(&self.home);
                    self.catalog.status = msg;
                } else if matches!(sact, super::controls_studio::StudioAction::SavePreset) {
                    self.catalog.refresh_controls(&self.home);
                }
            }
            None => {}
        }
        let next_view = (
            self.catalog.page,
            self.chooser.is_some(),
            self.login.visible,
            self.catalog.volt_boost_modal,
            self.controls_studio.open,
            self.update_prompt.is_some(),
        );
        if next_view != prev_view {
            self.targets.clear();
            self.list_targets.clear();
            self.drag = None;
            self.mouse_down = false;
            self.pending_activate = None;
            self.transition_guard_until =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(180));
        }
        Ok(())
    }

    pub(super) fn ui_density(&self) -> f32 {
        let (Some(r), Some(gui)) = (&self.renderer, &self.gui) else {
            return 1.0;
        };
        let mut batch = HudBatch::new([r.config.width, r.config.height]);
        Ui::mobile(&gui.atlas, &mut batch, (0.0, 0.0)).density
    }

    pub(super) fn touch_event(&mut self, _events: &ActiveEventLoop, t: Touch) -> Result<()> {
        self.focus = None;
        self.pending_focus = None;
        self.needs_redraw = true;
        match t.phase {
            TouchPhase::Started => {
                if self.drag.is_none() {
                    self.cursor = (t.location.x, t.location.y);
                    if self.controls_studio.open {
                        let density = self.ui_density() as f64;
                        let _ = self.controls_studio.pointer_down(
                            (t.location.x / density) as f32,
                            (t.location.y / density) as f32,
                        );
                    }
                    self.drag = Some(super::gesture::Drag::with_id(
                        t.id,
                        t.location.x,
                        t.location.y,
                    ));
                }
            }
            TouchPhase::Moved => {
                if self
                    .drag
                    .as_ref()
                    .is_none_or(|d| d.pointer_id != Some(t.id))
                {
                    return Ok(());
                }
                self.cursor = (t.location.x, t.location.y);
                let density = self.ui_density() as f64;
                if self.controls_studio.open
                    && self.controls_studio.pointer_move(
                        (t.location.x / density) as f32,
                        (t.location.y / density) as f32,
                    )
                {
                    if let Some(drag) = &mut self.drag {
                        let _ = drag.update_xy(t.location.x, t.location.y, density);
                    }
                    return Ok(());
                }
                if let Some(drag) = &mut self.drag {
                    let (dx, dy) = drag.update_xy(t.location.x, t.location.y, density);
                    if drag.moved {
                        let [ox, oy] = drag.origin_xy();
                        let ux = (ox / density) as f32;
                        let uy = (oy / density) as f32;
                        let in_preview = !self.login.visible
                            && self.chooser.is_none()
                            && self.update_prompt.is_none()
                            && !self.catalog.volt_boost_modal
                            && !self.controls_studio.open
                            && self.preview_rect.is_some_and(|[rx, ry, rw, rh]| {
                                ux >= rx && ux <= rx + rw && uy >= ry && uy <= ry + rh
                            });
                        if in_preview {
                            self.preview_target_yaw = None;
                            self.preview_yaw =
                                (self.preview_yaw + dx * 0.9).rem_euclid(360.0);
                            self.preview_vel_yaw = (dx * 42.0).clamp(-360.0, 360.0);
                            self.preview_pitch =
                                (self.preview_pitch - dy * 0.45).clamp(-28.0, 28.0);
                        } else if let Some(prompt) = &mut self.update_prompt {
                            prompt.scroll = (prompt.scroll + dy).max(0.0);
                        } else if self.catalog.volt_boost_modal {
                            self.catalog.volt_modal_scroll =
                                (self.catalog.volt_modal_scroll + dy).max(0.0);
                        } else if let Some(chooser) = &mut self.chooser {
                            chooser.scroll = (chooser.scroll + dy).max(0.0);
                        } else if !self.controls_studio.open
                            && self
                                .list_view
                                .is_some_and(|view| oy / density >= view[0] as f64)
                        {
                            self.catalog.scroll = (self.catalog.scroll + dy).max(0.0);
                        }
                    }
                }
            }
            TouchPhase::Ended => {
                if self
                    .drag
                    .as_ref()
                    .is_some_and(|d| d.pointer_id == Some(t.id))
                {
                    let studio_dragged = self.controls_studio.pointer_up();
                    self.cursor = (t.location.x, t.location.y);
                    if let Some(drag) = self.drag.take()
                        && !drag.moved
                        && !studio_dragged
                    {
                        self.request_activate();
                    }
                }
            }
            TouchPhase::Cancelled => {
                if self
                    .drag
                    .as_ref()
                    .is_some_and(|d| d.pointer_id == Some(t.id) || d.pointer_id.is_none())
                {
                    let _ = self.controls_studio.pointer_up();
                    self.drag = None;
                }
            }
        }
        Ok(())
    }

    pub(super) fn handle_escape(&mut self) {
        if let Some(prompt) = self.update_prompt.take() {
            crate::volt_updater::mark_update_seen(&self.home, &prompt.info.tag_name);
            return;
        }
        if self.login.visible {
            self.login.hide();
            crate::platform::hide();
            self.catalog.editing = None;
            return;
        }
        if self.controls_studio.open {
            if !self.controls_studio.back() {
                crate::platform::hide();
                self.catalog.refresh_controls(&self.home);
            }
            return;
        }
        if self.catalog.volt_boost_modal {
            self.catalog.volt_boost_modal = false;
            return;
        }
        if self.chooser.take().is_some() {
            return;
        }
        crate::platform::hide();
        self.catalog.editing = None;
        let _ = self.catalog.config.save(&self.home);
    }

    pub(super) fn type_key(&mut self, key: KeyEvent) {
        if key.physical_key == PhysicalKey::Code(KeyCode::Escape)
            || key.logical_key
                == winit::keyboard::Key::Named(winit::keyboard::NamedKey::BrowserBack)
        {
            self.pending_escape = true;
            return;
        }
        if self.catalog.editing.is_none() {
            return;
        }
        let out = match key.physical_key {
            PhysicalKey::Code(KeyCode::Backspace) => Some(Output::Backspace),
            PhysicalKey::Code(KeyCode::Enter) => {
                crate::platform::hide();
                Some(Output::Enter)
            }
            _ => key.text.map(|s| Output::Text(s.into())),
        };
        if let Some(out) = out {
            self.catalog.edit(out, &self.home);
        }
    }
}

fn wrap_modal_lines(ui: &Ui, lines: &[String], font_sz: f32, max_w: f32) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for raw in lines {
        let is_bullet = raw.starts_with("• ");
        let mut rest = raw.as_str();
        let mut first = true;
        while !rest.is_empty() {
            let prefix = if !first && is_bullet { "   " } else { "" };
            let avail = (max_w - if !first && is_bullet { 14.0 } else { 0.0 }).max(80.0);
            let fitted = ui.fit_text(Face::LauncherRegular, font_sz, rest, avail);
            if fitted == rest || !fitted.ends_with("...") {
                out.push((format!("{prefix}{rest}"), is_bullet && first));
                break;
            }
            let min_step = rest.chars().next().map_or(1, char::len_utf8);
            let mut cut_target = fitted
                .trim_end_matches("...")
                .len()
                .max(min_step)
                .min(rest.len());
            while cut_target < rest.len() && !rest.is_char_boundary(cut_target) {
                cut_target += 1;
            }
            let split_at = rest[..cut_target]
                .rfind(' ')
                .filter(|&i| i > 8)
                .unwrap_or(cut_target)
                .max(min_step);
            let head = rest[..split_at].trim_end();
            out.push((format!("{prefix}{head}"), is_bullet && first));
            rest = rest[split_at..].trim_start();
            first = false;
        }
    }
    out
}

fn draw_update_prompt(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    prompt: &mut crate::volt_updater::UpdatePrompt,
) {
    let (w, h) = (ui.w, ui.h);
    ui.rounded(0.0, 0.0, w, h, 0.0, theme::BACKDROP());
    let mw = (w - 44.0).clamp(300.0, 580.0);
    let mh = (h - 36.0).clamp(250.0, 396.0);
    let mx = (w - mw) * 0.5;
    let my = (h - mh) * 0.5;

    ui.rounded(mx, my, mw, mh, 16.0, theme::CARD());
    ui.rounded(mx + 6.0, my + 6.0, mw - 12.0, 62.0, 12.0, theme::HERO_BG());
    ui.border(mx, my, mw, mh, 16.0, theme::BORDER_CRIMSON());

    ui.text(
        Face::LauncherMedium,
        20.0,
        mx + 22.0,
        my + 15.0,
        theme::TEXT(),
        "Update Available",
    );

    let ver_badge_w = 82.0;
    let ver_badge_x = mx + mw - ver_badge_w - 20.0;
    ui.rounded(
        ver_badge_x,
        my + 15.0,
        ver_badge_w,
        24.0,
        7.0,
        theme::CRIMSON_BADGE(),
    );
    ui.border(
        ver_badge_x,
        my + 15.0,
        ver_badge_w,
        24.0,
        7.0,
        theme::BORDER_CRIMSON(),
    );
    draw_centered_text(
        ui,
        Face::LauncherMedium,
        12.5,
        [ver_badge_x, my + 15.0, ver_badge_w, 24.0],
        theme::CRIMSON_BRIGHT(),
        &prompt.info.display_version,
    );

    let size_mb = if prompt.info.size_bytes > 0 {
        format!(" · {:.1} MB", prompt.info.size_bytes as f64 / (1024.0 * 1024.0))
    } else {
        String::new()
    };
    let subtitle = format!(
        "Matched build for your device: {}{}",
        prompt.info.abi_label, size_mb
    );
    let sub_fit = ui.fit_text(Face::LauncherRegular, 12.5, &subtitle, mw - 44.0);
    ui.text(
        Face::LauncherRegular,
        12.5,
        mx + 22.0,
        my + 43.0,
        theme::MUTED(),
        &sub_fit,
    );

    ui.text(
        Face::LauncherMedium,
        14.0,
        mx + 22.0,
        my + 76.0,
        theme::CRIMSON_BRIGHT(),
        "What's New",
    );

    let box_x = mx + 20.0;
    let box_y = my + 98.0;
    let box_w = mw - 40.0;
    let box_h = (mh - 170.0).max(80.0);
    ui.rounded(box_x, box_y, box_w, box_h, 11.0, theme::CARD_INNER());
    ui.border(box_x, box_y, box_w, box_h, 11.0, theme::BORDER());

    let wrapped = wrap_modal_lines(ui, &prompt.info.whats_new, 13.0, box_w - 28.0);
    let line_h = 21.0;
    let max_scroll = ((wrapped.len() as f32) * line_h + 14.0 - box_h).max(0.0);
    prompt.scroll = prompt.scroll.clamp(0.0, max_scroll);

    for (idx, (line, highlight)) in wrapped.iter().enumerate() {
        let ly = box_y + 8.0 + idx as f32 * line_h - prompt.scroll;
        if ly < box_y + 3.0 || ly + 17.0 > box_y + box_h - 3.0 {
            continue;
        }
        ui.text(
            if *highlight { Face::LauncherMedium } else { Face::LauncherRegular },
            13.0,
            box_x + 14.0,
            ly,
            if *highlight { theme::TEXT() } else { theme::MUTED() },
            line,
        );
    }

    if max_scroll > 1.0 {
        let track_h = (box_h - 12.0).max(18.0);
        let thumb_h = (track_h * (box_h / (box_h + max_scroll))).clamp(18.0, track_h);
        let thumb_y =
            box_y + 6.0 + (track_h - thumb_h) * (prompt.scroll / max_scroll).clamp(0.0, 1.0);
        ui.rounded(box_x + box_w - 7.0, thumb_y, 3.0, thumb_h, 1.5, theme::CRIMSON());
    }

    let footer_y = my + mh - 56.0;
    let btn_w = 116.0;
    let install_x = mx + mw - 20.0 - btn_w;
    let cancel_x = install_x - 10.0 - btn_w;
    let hint = "Downloads in your browser — open the APK to update.";
    let hint_fit = ui.fit_text(Face::LauncherRegular, 11.5, hint, (cancel_x - mx - 32.0).max(90.0));
    ui.text(
        Face::LauncherRegular,
        11.5,
        mx + 22.0,
        footer_y + 12.0,
        theme::DIM(),
        &hint_fit,
    );
    styled_button(
        ui,
        targets,
        [cancel_x, footer_y + 4.0, btn_w, 38.0],
        "Later",
        Action::CancelUpdate,
        BtnStyle::Secondary,
        true,
    );
    styled_button(
        ui,
        targets,
        [install_x, footer_y + 4.0, btn_w, 38.0],
        "Download",
        Action::InstallUpdate,
        BtnStyle::Primary,
        true,
    );
}

fn draw_volt_boost_modal(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    list_targets: &mut Vec<([f32; 4], Action)>,
    catalog: &mut Catalog,
) -> Option<[f32; 3]> {
    use horizon_client::extras::Item;
    use horizon_client::volt_features::VoltBoostPreset;

    let (w, h) = (ui.w, ui.h);
    let scale = ui.density / ui.batch.scale;
    ui.batch
        .fill(ui.atlas, 0.0, 0.0, w * scale, h * scale, theme::BACKDROP());

    let mw = (w - 32.0).clamp(300.0, 740.0);
    let mh = (h - 28.0).clamp(240.0, 500.0);
    let mx = ((w - mw) * 0.5).round();
    let my = ((h - mh) * 0.5).round();

    ui.rounded(mx, my, mw, mh, 16.0, theme::CARD());
    ui.rounded(mx + 8.0, my + 8.0, mw * 0.55, 86.0, 12.0, theme::HERO_GLOW_SOFT());
    ui.border(mx, my, mw, mh, 16.0, theme::BORDER_CRIMSON());

    let active_n = catalog.extras.active_volt_optimizations();
    let rss_mib = horizon_threads::resident_ram_mib().unwrap_or(0);
    ui.text(
        Face::LauncherMedium,
        19.0,
        mx + 20.0,
        my + 12.0,
        theme::TEXT(),
        "Volt Boost — Performance Hub",
    );
    let sub = format!("{active_n} Optimizations Active  •  Resident RAM: {rss_mib} MiB");
    let sub_fit = ui.fit_text(Face::LauncherRegular, 12.5, &sub, (mw - 260.0).max(120.0));
    ui.text(Face::LauncherRegular, 12.5, mx + 20.0, my + 37.0, theme::MUTED(), &sub_fit);

    styled_button(
        ui,
        targets,
        [mx + mw - 236.0, my + 13.0, 106.0, 32.0],
        "Auto-Tune",
        Action::VoltExtra(Item::VoltAutoOptimize),
        BtnStyle::Primary,
        true,
    );
    styled_button(
        ui,
        targets,
        [mx + mw - 122.0, my + 13.0, 104.0, 32.0],
        "Purge RAM",
        Action::VoltExtra(Item::VoltPurgeRam),
        BtnStyle::CrimsonOutline,
        true,
    );

    let presets = [
        (VoltBoostPreset::Off, "OFF"),
        (VoltBoostPreset::Balanced, "Balanced"),
        (VoltBoostPreset::Competitive, "PvP"),
        (VoltBoostPreset::UltraFps, "Ultra"),
        (VoltBoostPreset::BatterySaver, "Battery"),
        (VoltBoostPreset::LowRam, "Low-RAM"),
    ];
    let p_gap = 6.0;
    let p_w = ((mw - 36.0 - p_gap * 5.0) / 6.0).max(42.0);
    for (pi, (preset, label)) in presets.iter().enumerate() {
        let px = mx + 18.0 + pi as f32 * (p_w + p_gap);
        styled_button(
            ui,
            targets,
            [px, my + 58.0, p_w, 30.0],
            label,
            Action::VoltPreset(*preset),
            if catalog.extras.volt_preset == *preset {
                BtnStyle::Primary
            } else {
                BtnStyle::Secondary
            },
            true,
        );
    }

    let items: [Item; 31] = [
        Item::VoltPerfHud,
        Item::VoltAutoDrs,
        Item::VoltCullLeaves,
        Item::VoltFrustumMesh,
        Item::VoltLookaheadStream,
        Item::VoltFastTranslucency,
        Item::VoltCaveSkyCull,
        Item::VoltChunkUploadCap,
        Item::VoltFastWeather,
        Item::VoltDistantLod,
        Item::VoltEntityCap,
        Item::VoltItemCap,
        Item::VoltBlockEntityCap,
        Item::VoltCullSignText,
        Item::VoltFastShadows,
        Item::VoltDistantMobThrottle,
        Item::VoltBlockTickRange,
        Item::VoltEntityTickRange,
        Item::VoltHubLagShield,
        Item::VoltModLagShield,
        Item::VoltParticleBudget,
        Item::VoltParticleFrustum,
        Item::VoltChaosShield,
        Item::VoltAnimThrottle,
        Item::VoltActiveAnimCull,
        Item::VoltGlintMode,
        Item::VoltDynamicFps,
        Item::VoltMenuBg,
        Item::VoltThermalGuard,
        Item::VoltReflex,
        Item::VoltMemorySaver,
    ];

    let list_top = my + 96.0;
    let footer_y = my + mh - 46.0;
    let list_bottom = footer_y - 8.0;
    ui.rounded(
        mx + 14.0,
        list_top - 2.0,
        mw - 28.0,
        (list_bottom - list_top + 4.0).max(40.0),
        10.0,
        theme::CARD_INNER(),
    );
    ui.border(
        mx + 14.0,
        list_top - 2.0,
        mw - 28.0,
        (list_bottom - list_top + 4.0).max(40.0),
        10.0,
        theme::BORDER(),
    );

    let two_cols = mw >= 540.0;
    let row_h = 40.0;
    let rows_count = if two_cols {
        items.len().div_ceil(2)
    } else {
        items.len()
    };
    let max_scroll = (rows_count as f32 * row_h - (list_bottom - list_top)).max(0.0);
    catalog.volt_modal_scroll = catalog.volt_modal_scroll.clamp(0.0, max_scroll);
    let col_w = if two_cols {
        (mw - 44.0) * 0.5
    } else {
        mw - 36.0
    };

    for (idx, &item) in items.iter().enumerate() {
        let (r_idx, c_idx) = if two_cols {
            (idx / 2, idx % 2)
        } else {
            (idx, 0)
        };
        let rx = mx + 20.0 + c_idx as f32 * (col_w + 8.0);
        let ry = list_top + 4.0 + r_idx as f32 * row_h - catalog.volt_modal_scroll;
        let ctrl_w = (col_w * 0.44).clamp(96.0, 148.0);
        let ctrl_rect = [rx + col_w - ctrl_w - 6.0, ry + 3.0, ctrl_w, 30.0];
        list_targets.push((ctrl_rect, Action::VoltExtra(item)));
        if ry < list_top - 2.0 || ry + row_h - 4.0 > list_bottom {
            continue;
        }
        let lbl_fit = ui.fit_text(Face::LauncherRegular, 13.0, item.caption(), col_w - ctrl_w - 14.0);
        ui.text(
            Face::LauncherRegular,
            13.0,
            rx + 4.0,
            text_vcenter(ry, row_h - 4.0, 13.0),
            theme::TEXT(),
            &lbl_fit,
        );
        if let Some(on) = item.switch(&catalog.extras) {
            toggle(ui, targets, ctrl_rect, on, Action::VoltExtra(item));
        } else {
            styled_button(
                ui,
                targets,
                ctrl_rect,
                &item.value(&catalog.extras),
                Action::VoltExtra(item),
                BtnStyle::Secondary,
                false,
            );
        }
    }

    styled_button(
        ui,
        targets,
        [mx + 18.0, footer_y + 2.0, 174.0, 36.0],
        "All Volt Settings",
        Action::Page(Page::VoltSettings),
        BtnStyle::CrimsonOutline,
        true,
    );
    styled_button(
        ui,
        targets,
        [mx + mw - 122.0, footer_y + 2.0, 104.0, 36.0],
        "Done",
        Action::CloseVoltBoost,
        BtnStyle::Primary,
        true,
    );
    Some([list_top, list_bottom, max_scroll])
}

