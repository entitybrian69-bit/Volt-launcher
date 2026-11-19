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
| Volt Settings / Boost, waypoints, minimap, mods, quick toggles, performance HUD | `volt_features`, `volt_settings_ui`, `volt_mods`, in-game `volt_hub` | Windows memory adapter supplies physical RAM for auto-optimize and current working set for the HUD. Verify presets and output on GPU; OS performance hints/thermal data must be explicitly capability-dependent. |
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
checks Volt Boost's memory counters, then builds the game, checks PE imports and starts its CLI with scratch homes.
Its artifact is **engine only**. `--help` does not create a window or prove
that gameplay works. No prepared resources or accounts are provided. Windows
Explorer icon is derived from `packaging/android/res/drawable-nodpi/launcher_icon.png`.

### Verification evidence

[Native x64 run, 637d899](https://github.com/luciferstudios/Volt-launcherPc/actions/runs/44535445487):
Media Foundation encoded eight frames, read the H.264 track back as NV12, preserved
presentation timestamps and kept the previous file after an invalid re-export.
This is evidence for the codec and filesystem adapter, **not** in-game capture,
launcher UI, authentication or gameplay proof.

Run `44536044523` on commit `fff1c74`: the x64 engine build, system-DLL check,
`horizon-client.exe --help`, native media and memory-counter tests passed. The CI
artifact is explicitly engine-only. There is no Windows 11 launcher startup result.

## Next milestones / release gates

1. Exercise the built engine beyond CLI startup: capture a rendered offscreen frame
   using a tester's private prepared resources. Reconcile any
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

### Candidate packaging and Windows 11 verification

`packaging/windows/build-candidate.ps1` and `verify-candidate.ps1` are release-engineering
scripts, **not a published build**. They have no publish step. They require the actual
Windows Volt launcher; an engine-only build or the separate Horizon launcher is not a
candidate. Do not use `tools/package_native_client.py`: its archives include private game files.

Once the port is ready, assemble a **fresh, audited distribution folder** from the exact
release commit: `Volt Launcher.exe`, `horizon-client.exe`, license, notices, and all approved
public runtime files. Inspect nested archives for private data and restricted game resources.
Run the following with PowerShell 7.2+, Python 3 and Inno Setup 6.3+ (`ISCC.exe` on PATH):

```powershell
./packaging/windows/build-candidate.ps1 -Version $version -Payload $stage -OutputDirectory $newOut
```

Copy the pristine staged files and both candidates to a clean, disposable **Windows 11
x64 desktop** VM, logged in with no existing Volt install. Use a normal user and no
development dependencies on PATH. Run:

```powershell
./packaging/windows/verify-candidate.ps1 -Payload $stage -CandidateDirectory $newOut
```

This checks extracted bytes and installed payload files against staging, starts each
launcher with no CLI shortcut, requires a responsive `Volt Launcher` window, checks
clean shutdown and uninstall, and hashes the exact installer and ZIP. A window title
is not evidence of a correctly rendered home page, account ownership or game launch:
complete the UI and licensed-game acceptance tests above **before** publishing. Retain
`SHA256SUMS`, the commit, OS build and test evidence; verify re-downloaded release
assets against those same hashes before sharing links. A `windows-latest` engine CI
job is not this desktop acceptance test.

### Native launcher integration milestone (not a release)

`cargo build --profile fast --locked --target x86_64-pc-windows-msvc -p volt-launcher`
creates **volt-launcher.exe**, distinct from the game. Its winit/wgpu host uses the
same Liquid Glass home drawing, hit rectangles and device-code panel as Android.
Set `HORIZON_MSA_CLIENT_ID` to your approved public Microsoft client ID; no
credentials are bundled. PC account metadata lives under `%LOCALAPPDATA%\Volt Launcher`
(or `HORIZON_LAUNCHER_HOME`); secrets use a separate Credential Manager service.
Place the mods-enabled `horizon-client.exe` and allowlisted generated
`engine-data.zip` from `tools/android_build.py:engine_data(public=True)` beside the
GUI for installation. Never substitute a private developer resource bundle.

Play checks ownership off the UI thread, installs official resources per player,
and supplies a short-lived one-use session file to the engine. The Settings action
persists the engine graphics API. Other catalog/profile/gallery/Controls Studio
routes are **not yet ported**; a rendered frame is not proof of account, install,
gameplay or full navigation parity. The Windows workflow attempts an actual presented
home frame on its hosted Windows runner. For a Windows 11 x64 client result,
dispatch with `windows11: true` on a fork-managed signed-in interactive runner
labelled `volt-win11-x64-interactive`. The job checks OS edition/build/architecture;
Windows Server evidence does not satisfy this gate. No verified player package yet.
