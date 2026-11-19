# Volt Launcher PC — port ledger

**Development branch, not a Windows release.** No installer yet. Do not point the
Horizon desktop updater at upstream releases or install an Android APK on Windows.

## Reference and boundaries

On 2026-11-19 the default and sole published branch of `luciferstudios/Volt-launcherPc`
was `android`. GitHub's fork parent `entitybrian69-bit/Volt-launcher:android` and
`origin/android` both resolved to `6227a1edd3251fa9fd396c3569288aaec1e5353c`:
no fork-specific PC delta at baseline. `origin` is the only configured remote.
Work is on `arena/d51c35d7-volt-launcherpc`; keep `android` as reference. The
fork has issues disabled. The `.github/upstream-workflows/*.disabled` files
are reference material, **not** a working release pipeline for Volt.

The root Cargo workspace/lock builds the game. The Android cdylib entry point
is `crates/horizon-client/src/android_main.rs`; it composes the game, launcher
and platform services. The Android UI is **not** the GPUI application in the
separate `launcher/` workspace. It uses `android/{launcher,launcher_pages,
controls_studio,chooser}.rs`, winit, the game's wgpu renderer, `HudBatch`,
`ui_theme`, actual fonts, logo and hit targets. Use those pages rather than
reimplementing them. The GPUI workspace is useful for its actual account,
browser, server-list, skins, preparation and desktop OS implementations; Android
already includes some of those files directly. See [architecture](architecture.md),
[Android feature notes](android-features.md) and `AGENTS.md` for engine ownership.

## Feature inventory / Windows acceptance map

Rows describe source capabilities, **not** claims of Windows verification.
`android/catalog.rs::Page`, `launcher_pages.rs::{Action,HomeTab}` and
`controls_studio.rs::StudioAction` are the action-level source of truth.

| Surface in Volt | Code to keep | Windows integration / proof needed |
|---|---|---|
| Play, Overview, Worlds, Multiplayer, Screenshots, Replays, Console; sidebar, chooser, patch notes, About, photo viewer | launcher pages, chooser, photo cache, skin preview; themes, liquid, accents, feedback and animations | Shared renderer host in a resizable DPI-aware winit window. Compare screenshots and keyboard/mouse/touch hit targets at multiple scales. |
| Create/switch numbered profiles, edit names, settings, saves and per-profile content | `catalog::Config`, engine `options.txt`, content and world modules | Separate writable per-user root under LocalAppData, profile 0 legacy layout, `profiles/profile-N` for others; never write beside the installed EXE. Restart and test two profiles. |
| Install/repair pinned 26.3, progress, cache cleanup, audio, languages, worldgen | `official_install`, `android_resources`, `resource_install`, `research-lock.json`, `horizon-prepare` | Replace APK asset-manager access for generated tables. Build only allowed tables; fetch verified official objects at runtime. Interrupt and resume with non-ASCII paths; no Mojang files in Git or CI artifacts. |
| Singleplayer, integrated server, worlds; Java protocol 777, server list, multiplayer, friends | `game.rs`, `horizon-{world,server,protocol,friends}`, `server_list` | Use existing desktop event loop and session handoff, join a matching online-mode server; verify save, reload, leave and return to launcher. Friends relay is not proven Android parity. |
| Accounts: Microsoft device code, pending/cancel, sign out, offline profiles, Ely.by; skins, capes, slim models, previews, server sync | `account.rs`, `android/{login,volt_accounts,skin_preview,secure_store}` | OS browser, explicit clipboard, Credential Manager for secrets; isolate provider identities. No bundled credentials or substitute for MS entitlement. Needs an approved client ID and a licensed tester for live verification. |
| Mods, installed toggle/delete, per-profile bundled and woven mods, import/catalog/detail, resource and shader packs | `catalog`, `shader_catalog`, `content_files`, `horizon-{mods,shaders}`, `tools/native_mods` | File picker/Explorer in place of SAF; keep checksum/capability checks. Validate an actual `.hmod` and packaged woven mod behavior, not just a catalog row. Android does not run Fabric/Forge Java mods or compile arbitrary `.hnmod`. |
| Controls Studio, preset import, custom keys/drawers/images, four touch schemes, keypads, keyboard/IME, mouse capture, controller and feedback | `controls_studio`, `horizon-client-input`, touch/OSK, gilrs and DualSense | Bridge winit IME, raw mouse, touchscreen and focus cancellation; test edits, presets, gamepad navigation and sleep/resume. Phone vibration/gyro require applicable hardware, not simulated callbacks. |
| Renderer selection, scale, aspect, refresh, orientation, immersive, cutout, custom phone Vulkan drivers | `display`, `gfx`, `graphics`; patched wgpu-hal and winit | Existing desktop Vulkan/DX12/WGL backends; test adapter fallback, DPI, resize, fullscreen, minimize/restore. Android driver packages are not Windows drivers. |
| Sound, volumes, subtitles, accessibility | `audio`, `sound_engine`, desktop CPAL | WASAPI playback, device switch and mixer tap against prepared sound assets. |
| Volt Settings / Boost, waypoints, minimap, mods, quick toggles, performance HUD | `volt_features`, `volt_settings_ui`, `volt_mods`, in-game `volt_hub` | Preserve real settings and presets, inspect output on GPU; OS performance hints/thermal data must be explicitly capability-dependent. |
| Replay record/watch/seek/camera, MP4 export, recorder, HUD/control layers, clip ring, captures | `game/{volt_gameplay,video_export,recorder}`, replay and renderer readback | **First implementation:** Windows Media Foundation sink for replay export. Real Windows test below. Live H.264 + AAC recording and 30 s clip saving are separate Android MediaCodec paths, **not ported by this change**. |
| Files/gallery, screenshots, reports, console and crash recovery | `gallery`, `documents`, `diagnostics`, `exit_reports`, `performance`, engine logging | Native open/save dialogs, per-profile directories, log rotation/redaction and local report export. Test locked files, failed writes and forced termination. SAF/MediaStore/tombstones are not Windows APIs. |
| Discord presence, links, update check/install | `discord_rpc`, `volt_updater`, `launcher/src/signed.rs` | Desktop IPC and safe OS links; fork-specific signed update channel. Never install APK updates. No automatic publishing during bring-up. |

Java version/JVM flags/RAM sliders are not how Volt's Rust simulation starts.
Do not advertise the ancestral desktop launcher's multi-version JVM management
as Android parity. Likewise, world import/export and all-server compatibility
are not guaranteed by the Android source.

## First milestone: native engine and replay encoder

On Windows x64 use MSVC Build Tools (C++ workload, Windows SDK), rustup with
the repo's `rust-toolchain.toml`, Git and a Windows installation with Media
Foundation (Windows N needs the Media Feature Pack). From the repo root:

```powershell
cargo test --locked -p volt-windows-media --target x86_64-pc-windows-msvc
$env:RUSTFLAGS = '-C target-feature=+crt-static'
cargo build --locked --target x86_64-pc-windows-msvc --profile fast -p horizon-client --bin horizon-client --features mods
.\target\x86_64-pc-windows-msvc\fast\horizon-client.exe --help
```

`windows-engine.yml` runs the native encode/decode + failed-reexport test,
then builds the game, checks PE imports and starts its CLI with scratch homes.
Its artifact is **engine only**. `--help` does not create a window or prove
that gameplay works. No prepared resources or accounts are provided. Windows
Explorer icon is derived from `packaging/android/res/drawable-nodpi/launcher_icon.png`.

## Next milestones / release gates

1. Get native runner proof for the locked engine, replay export and a rendered
   offscreen frame using a tester's private prepared resources. Reconcile any
   fork-specific desktop build regressions without disabling renderer/mods.
2. Factor launcher host seams (data paths, window/event loop, async workers,
   file picker, account vault, lifecycle) away from JNI. Mount the **same**
   pages on Windows. Compare every page and verify persisted navigation/settings.
3. Port generated-table packaging and verified install/repair, profile isolation,
   licensed sign-in and process launch/return; exercise fresh install and upgrade.
4. Run the inventory on Windows 10 and 11 with real input, GPU and audio;
   validate multiplayer, mod hooks, recorder/clip, shutdown and crash reports.
5. Only after these gates package `Volt Launcher.exe`, dependencies, license,
   notices and icon into a per-user installer and portable archive. Verify clean
   install, offline relaunch, uninstall without save loss, signed update and rollback.
