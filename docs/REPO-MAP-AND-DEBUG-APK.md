# Volt Launcher — repo map and debug APK

Working notes for the `arena/8fa03d72-volt-launcher` session (base `53883055b`, 2026-11-15).
Local scratch document; delete it if it should not stay in the tree.

## What this repository is

`entitybrian69-bit/Volt-launcher` is the **Volt Launcher** distribution of
**Volt Android** — an unofficial Android fork of
[Volt](https://github.com/ludovic111/volt), a from-scratch Rust + wgpu
reimplementation of **Minecraft: Java Edition 26.3**. Players install an APK, the
app downloads Mojang's original resources itself, and the engine runs natively on
the phone (no Java runtime, no JVM, no Minecraft jar at play time).

- Vanilla Java 26.3 is the specification; the PC/desktop launcher is
  GPUI/Rust (`launcher/`), the game is `crates/volt-client` (+ 32 more crates).
- Branding on top of upstream: `ae.net.Voltlauncher` (stable) and
  `ae.net.Voltlauncher.debug` ("Volt Debug"), Volt theme (Liquid Glass /
  Liquid Black), dashboard, Controls Studio, mod catalog, updater.

### Layout

| Path | What it holds |
| --- | --- |
| `crates/volt-client` | The game: `src/lib.rs` (flat modules) + `src/main.rs` → `game.rs`, `game/`; also `bin/volt-ctl` and the dedicated server |
| `crates/volt-client/src/android/` | **Volt Android launcher UI**: `launcher.rs`, `launcher_pages.rs`, `catalog.rs`, `controls_studio.rs`, `volt_updater.rs`, `volt_accounts.rs`, `login.rs`, `secure_store.rs`, `diagnostics.rs`, `native_ime.rs`, … |
| `crates/volt-client/src/game/` | Gameplay per feature: `android_input.rs`, `pad.rs`, `touch_ui`, `remote*.rs`, `lan*.rs`, `integrated.rs`, `dedicated.rs`, mods, screens |
| `crates/volt-client-render` | World data, rendering, HUD/menus/settings (~80 modules re-exported as `volt_client::*`) |
| `crates/volt-world` | Vanilla world data: NBT, regions, chunks, `datafix/`, `worldgen/`, `level/` |
| `crates/volt-core`, `volt-protocol`, `volt-assets`, `volt-command`, `volt-items`, `volt-mod*`, `volt-theme`, … | 33 crates total (see `AGENTS.md` table) |
| `crates/volt-android` | The `cdylib` `libvolt_launcher.so` wrapper (NativeActivity + Rust IME bridge) |
| `crates/volt-client/src/touch*`, `ui_theme.rs` | Touch schemes (joystick/D-pad × crosshair/tap), HUD editor, theme |
| `packaging/android/` | `AndroidManifest.xml`, `build.gradle` (namespaces/app ids/versions), `java/` (2 stock-Java classes), `res/`, `aidl/` |
| `tools/android_*.py` | `android_build.py` (Gradle + NDK + signing + audit), `android_prepare.py` (engine tables from your own MC install), `android_release.py`, `android_ctl.py`, probes |
| `tests/test_android*.py` | Release audit, source export, release schedule, public-tree checks |
| `.github/workflows/` | `android-apk.yml` (the APK pipeline), `android-source.yml`, `native-mod-probe.yml`, `pages.yml` |
| `launcher/` | Desktop GPUI launcher (separate workspace), not shipped on Android |
| `docs/` | `android.md` (build/run), `android-publication.md`, `android-features.md`, `RELEASE-NOTES-*.md`, upstream reviews |

### How the APK is produced (`android-apk.yml`)

1. **plan** — ABI matrix (default: all four), profile (`release`/`fast`), publish flag.
2. **prepare** — `tools/android_prepare.py` turns the *player's own* Minecraft 26.3
   install into allowlisted engine tables (never committed, never Mojang files).
3. **weave** — 37 registry `.hnmod` native mods + 1 vendored (`punchy`) are woven
   into the engine library with pinned sha256s.
4. **build / APK per ABI** — unit tests, Gradle 8.14, `tools/android_build.py --public`,
   deterministic signing keystore (`ANDROID_KEYSTORE_BASE64` or derived), payload audit.
5. **release** — `push` to `android`/`arena/**` or dispatch refreshes the single rolling
   **`debug`** release; `v*` tags publish the stable channel. Artifacts are also
   attached to the run (30-day retention).

APK details: `--public` release profile = opt level 3 + ThinLTO, **non-debuggable**,
paths remapped, agent-control socket off. `minSdk 28` (Android 9), `targetSdk 35`,
`versionName 1.1.0`, versionCode = `280000 + run_number*2 + abi_offset`.
The **debug channel** is a *separate app* (`ae.net.Voltlauncher.debug`, "Volt Debug")
so it never overwrites a stable install.

## Branches

`android` is the long-lived branch; everything else is `arena/*` session branches
(one per Arena session), plus a junk `check-log-branch`.

| Branch | Tip | State |
| --- | --- | --- |
| `android` | `53883055b` (2026-11-15) | **Current base.** Merge of PR #3 (`arena/e1f86b34`): "Ship Volt Android launcher updates and login fixes" — modem/authentication fixes, login packet ordering, launcher alignment, live Android mod reload, key capture, shader metadata. |
| `arena/0cae2b73-volt-launcher` | `8594f2b7c` (2026-11-15) | **PR #4, open**, `+83,883 / −4,348` over 328 files, 259 commits ahead / 72 behind `android`. Newest work: debug-channel app split, sneak reliability, joystick toggle, "Frame" items, Liquid Glass silver redesign, Liquid Black "Embers" theme, dashboard + embedded Controls Studio, 3D skins, server-join vanilla fingerprint fixes (captcha-failed kicks). **Source of the current debug APK.** |
| `arena/e1f86b34-volt-launcher` | `c443b8a30` | merged (PR #3) → `android` |
| `arena/42c008a1-volt-launcher` | `e11d9c788` (2026-11-07) | merged (PR #1): Volt branding + screenshot gallery (docs only) |
| `arena/c3d6069d-volt-launcher` | `6b225e4d6` (2026-11-13) | **PR #2, open, stale** — 4 commits, none in `android` or the PR #4 tip: dark-glass monochrome overhaul, app branding, APK rename, then it pulls in the full PR #4 branch. Likely to be superseded by PR #4 (which already carries Liquid Glass/Liquid Black). |
| `arena/58a3cdf9-volt-launcher` | `3c6a6ad50` (2026-10-31) | no PR; "Brand Android app as Volt Launcher" — early predecessor, 1,700+ lines of docs behind |
| `arena/a4df1109-volt-launcher` | `9199f02ad` (2026-11-14) | no PR; spike that reverted an extra `volt-client-render` dep that broke the weave |
| `check-log-branch` | `5f7b9689b` | throwaway CI log dump (`check.log`), tag `check-log` with a `check.log` asset |

Repo tags/releases: `v1.0.0` (stable, `e413ea1a4`), `v0.7.6-build.939d02a` (pre-release),
rolling `debug`, `check-log`.

## Debug APK — where it is and why it is not attached here

Current debug build: **Volt Launcher Debug Build (1.0.0 @ 8594f2b)**, published
2026-11-15 10:24 UTC from `8594f2b7c` (= PR #4 head), successful CI run
[43701289653](https://github.com/entitybrian69-bit/Volt-launcher/actions/runs/43701289653)
(all 12 jobs green: plan, prepare, worldgen tests, runtime proof, 4× weave, 4× build, release).

Release page: <https://github.com/entitybrian69-bit/Volt-launcher/releases/tag/debug>

| Asset | ABI | Size | sha256 |
| --- | --- | --- | --- |
| `Volt-Launcher-debug-arm64-v8a.apk` | arm64-v8a (phones) | 73,188,131 B | `50cade8f09caa8e01a7160d0dccf71f1cf6e28b507e16fed44c19613f97bd3fb` |
| `Volt-Launcher-debug-armeabi-v7a.apk` | armeabi-v7a (32-bit) | 51,299,109 B | `3af80746021f40aa57449c1970a73a9a7b7621f4b519a92c0afe155979fcd1be` |
| `Volt-Launcher-debug-x86_64.apk` | x86_64 (emulator/ChromeOS) | 80,757,536 B | `511c3c804938f5e062b5dd491042ed32ee0b921e1fcdf72008e19b12c0996813` |
| `Volt-Launcher-debug-x86.apk` | x86 | 73,433,885 B | `32d9d73ce1e9c16fd04e85bc276426cf675dca33f82c8fc14f5bd316040952e2` |

Also attached: `biomes_unseen.vmod` (44 biomes, 176 blocks, 29 items), `SHA256SUMS.txt`,
`LICENSE.txt`, `THIRD_PARTY_NOTICES.md`.

Per-run CI artifacts for `android` @ `53883055b` (37 MB arm64-v8a zip, run 43700660342)
expire after 30 days.

**Why no APK file sits in this workspace:** this sandbox's outbound policy allows only
`github.com`, `codeload.github.com`, `api.github.com`, `registry.npmjs.org`, `pypi.org`
and `files.pythonhosted.org`. GitHub serves release assets and Actions artifacts from
`release-assets.githubusercontent.com`, `objects.githubusercontent.com` and
`*.blob.core.windows.net` (302 redirects), so every download attempt fails with
`SSL_ERROR_SYSCALL`. A local build is equally impossible: no Rust toolchain, JDK,
Android SDK/NDK or Gradle, and `static.rust-lang.org`, `dl.google.com`,
`services.gradle.org`, `repo.maven.apache.org` and `piston-meta.mojang.com` are all
unreachable, plus a 21 GB / 2-core budget for a ~1 hour multi-ABI Rust build.
The APK must be downloaded from the release page above in a normal browser, or built
with `python tools/android_build.py --abi arm64-v8a --public` (needs SDK 35, NDK
28.2.13676358, JDK 17, Gradle 8.14, Rust 1.95 + `aarch64-linux-android`, and a one-time
`tools/android_prepare.py` against a licensed Minecraft 26.3 install).

### Install notes

- Enable "install unknown apps" for your browser; the debug APK is signed with the
  project's persistent debug certificate, so it updates in place only over earlier
  Volt **Debug** installs — it installs beside the stable `ae.net.Voltlauncher`.
- First launch: pick **Download Minecraft 26.3** (~500 MiB, official Mojang HTTPS,
  hash-pinned, resumable), then **Play**. Vulkan is the default renderer with a GLES
  fallback; Android 9+ (API 28); arm64 build ships the native `.vmod` runtime.
- Verify integrity with `sha256sum Volt-Launcher-debug-arm64-v8a.apk`.

## How the pieces fit (mental model)

- **Two launchers, one engine.** `launcher/` is the desktop GPUI launcher; on Android
  the launcher is *in-engine* UI from `crates/volt-client/src/android/`. The APK
  contains one `libvolt_launcher.so` and two small stock Java classes; no duplicated engine.
- **Launcher → installer → game.** The launcher downloads/verifies resources, then
  re-execs the same binary into the game loop (`game.rs`). Leaving the game returns to
  the launcher.
- **One simulation for LAN/SP/multiplayer.** Single player runs on an in-process server
  thread (`game/integrated.rs`); the window joins it like a LAN guest, so fixes serve
  all three paths. `--in-frame` (old path) is deprecated.
- **Rules that bite:** never commit Mojang files or prepared resources; `--public` for
  anything distributed; `--release-candidate`/`--public` audits are privacy checks, not
  redistribution permission; keep signing keys in `.local/android/`; upstream deployment
  workflows stay inactive on this fork.

---

## Session changes (2026-11-15, `arena/8fa03d72-volt-launcher`)

Player-facing fixes requested in the review of the dashboard screenshot.

### Release channel — never a pre-release
`.github/workflows/android-apk.yml`: `publish_release` is now true only for
`android` itself, a `v*` tag, or an explicit `workflow_dispatch` with
`publish_release`. Session branches (`arena/**`) build APKs as run artifacts but
no longer roll the `debug` release. The release job already creates/edits rolling
debug builds with `--latest=false` and never marks them pre-release (only
`docs/RELEASE-NOTES-v*.md` tags ship notes); this keeps agent branches out of the
download page entirely.

### Dashboard layout (`crates/volt-client/src/android/launcher_pages.rs`)
- **Play** sits on the bottom edge and is centred in the column between the two
  rails (was offset 22 dp up and off-centre because the right column is wider).
  Pinned by `play_pill_is_centred_between_the_rails`.
- **Discord** moved out of the crowded top-right corner into its own glass row
  under the account chip (`draw_discord_row`), clear of the destination rail.
- **Account/profile chip** moved to its own bottom-left column and never slides
  under a rail (`indicator_right_room`).
- **Right rail is now real navigation**: icon-only Worlds / Servers / Shots /
  Replays / Boost (`draw_right_rail`), visible on the dashboard *and* on every
  secondary tab and on the Packs/Shaders/Browse catalog pages (`with_right_rail`).
  Tab bodies reserve its width (`liquid_home_tab_area`), so nothing hides under it.
- **Volt Boost is a tab** (`HomeTab::VoltBoost`) drawn by the same panel as the
  dialog (`draw_volt_boost_panel`), so the launcher stays reachable.
- **Tab bodies show real pictures**: the world's `icon.png` or its newest world
  screenshot (`world_picture`), server icons, screenshot thumbs and replay
  thumbnails; tapping a picture (not just the caption) opens fullscreen, and
  every tab keeps a visible `< Home` button plus the destination rail.
- **Left rail no longer cramped** (non-Liquid shell): item height is derived from
  the available height, entries shrink to icons before any gets clipped.
- **Shader / pack / mod detail pages** (`draw_catalog_page`): About / Screenshots /
  Download with `← Back`, a `Home` button, and `← List` in the footer, so a detail
  page is never a dead end.

### Shader download failures were silent (`crates/volt-client/src/android/catalog.rs`)
Failures lived only in `status`, so a failed Volt shader left its Download tab
looking inert. Now every install path records a per-project reason
(`failed_message` + `failure_for`), including early returns that used to do
nothing (busy worker, unsupported kind), and the detail page + grid cards print
it. The Download tab shows a live percentage and a "still working" note after 20 s
(`job_is_stalled`). Worker spawn failures also surface.

### "Captcha failed" on public server joins (ported from the unmerged PR #4)
Three non-vanilla behaviours in the join fingerprint; all three also existed on
`android`:
1. `net.rs` — the client volunteered a `volt:mod_net` hello to every server
   during configuration. Vanilla sends exactly `ClientInformation` and
   `minecraft:brand`; the hello is now strictly an answer to the host's own
   announcement (LAN hosts announce first, `lan/conn.rs`), and the client stays
   silent otherwise. Regression test
   `volt_hello_only_answers_the_servers_mod_net_announcement`.
2. `game/remote.rs` — `volt:skin` only goes to hosts that announced themselves
   as Volt.
3. `game/remote.rs` — the SkinsRestorer `/skin` command no longer fires the
   instant the command tree arrives (auth plugins punish that as bot behaviour);
   it waits 3 s and the player's own movement input (`pending_skin_cmd`).

### Verification log
| run | result |
| --- | --- |
| `43714386104` | weave failed: `Option<[f32;3]>` mismatch, private `progress` |
| `43715560533` | all four ABIs + APKs green, release skipped |
| `43718256658` | same, after the rail-tap fix: green (PR #5) |

PR **#5** (`arena/8fa03d72-volt-launcher` → `android`). Compile errors surface
through check-run annotations (`gh api repos/.../actions/runs/<id>/jobs`, then
`check-runs/<job_id>/annotations`), not through logs, which are blocked here.

### Known gaps after this pass
- The **classic (non-Liquid) lobby** still has its own top-right icon squares and
  bottom bar, so it only gets part of the new layout. Liquid Glass is the shipped
  theme; the classic fixes are limited to the derived-height left rail.
- The new dashboard has **no Discord control**; Discord remains reachable from the
  classic lobby's icon row and Utilities page. Adding it to the Liquid shell's
  left rail is the natural next step.
- Shader download success against the live registry is still unproven; only the
  failure path is now guaranteed to be visible.

PR #5's own check suite (`43720770524`) finished all-green: source, engine
tables, worldgen tests, runtime proof, native weave on all four ABIs, every APK
build; `Publish GitHub Release` = skipped (no pre-release, nothing published from
a session branch). The `docs/`+memory commits after it only touch this file.

### How to get the APK for a specific commit
Artifact downloads are blocked inside the agent sandbox (the blob storage host is
not reachable), but the run page itself is public:

`https://github.com/entitybrian69-bit/Volt-launcher/actions/runs/<run-id>` →
**Artifacts** → `Volt-Launcher-<version>-<abi>`, which unzips to
`Volt-Launcher-<version>-<abi>.apk`.

Beware: artifact names (`Volt-Launcher-1.0.0-<abi>`) carry **no commit**, so an
open **Actions** page can show a rebuilt, newer listing. Navigate from the run id.

Rolling `debug` release assets are likewise unlabelled by commit in the filename;
the release body names the commit and its `SHA256SUMS.txt` pins the bytes.
Session branches publish nothing, so that release only advances when `android`
or a `v*` tag does.

### APK download reality (checked 2026-11-15)
- Artifacts are **auth-walled**: an anonymous request to a public repo's artifact
  URL bounces through `github.com/<repo>/suites/<id>/artifacts/<id>` and lands on
  a 404 sign-in wall. There is **no** public direct-download URL for an artifact.
  The zip is a `.zip` containing `Volt-Launcher-<version>-<abi>.apk`.
- The agent's token can `push` and read, but **`workflow_dispatch` is 403**
  ("Resource not accessible by integration"), so it cannot trigger the
  `publish_release: true` path that would refresh the rolling `debug` release and
  mint a sign-in-free link. Only a human can run the dispatch:
  Actions → Android APK → Run workflow → branch `arena/8fa03d72-volt-launcher`,
  ABIs `all`, profile `release`, `publish_release` checked.
- `publish_release` is true for `refs/heads/android`, `v*` tags, or that explicit
  dispatch, so session-branch runs only ever produce artifacts.

### Second round of player feedback (screenshots 2026-11-15 23:10)
Confirmed the player runs the new build: the Discord row, the bottom-left profile
chip and the red "Retry" + reason on the Wisdom Shaders card are all from the
first pass.

**Fixed, with the mechanism proven:**

1. **Right rail not on the far right.** Measured the screenshot (2412x1080): left
   rail flush at x=0, right rail's outer edge at x=2145, then **213 px of bare
   wallpaper** to the right edge. Density 1.875 (from the 8 dp inset = 15 px),
   rail 90.7 dp == `LIQUID_RIGHT_RAIL_WIDTH` (88). The layout surface was
   **2160 px, not 2412**: `launcher_visible_bounds()` returned a frame 252 px
   narrower than the screen, and `ui.w` was set from it, so `surface_right`
   (`= w`) placed the rail 252 px in. The same shrink pulled PLAY to 44.4 %
   instead of 50 % (predicted 44.39 % from the constants, matching the observed
   44.4 %). Fix: only *vertical* insets move controls now; the width stays the
   full surface, which is what the scene is already painted across. Tests
   `destination_rail_sits_on_the_true_screen_edge` and
   `play_pill_is_centred_on_the_screen` pin both.

2. **Shaders still not downloading.** The on-screen reason was the answer:
   `volt-shaders.toml: TOML parse error ... unknown field 'fog', expected
   'file' or 'name'`. `Stage`, `RawOption` and `RawPage` had
   `deny_unknown_fields`, so one published registry pack writing `fog = false`
   in its `[[post]]` blocks aborted its whole download. Unknown keys are now
   ignored (vanilla's behaviour for keys it does not read); top-level typos are
   still refused. Test `a_stage_ignores_keys_this_build_does_not_read`.

3. **Controls missing in game.** A full `Page::Controls` existed but was only
   reachable through Options > Controls. The pause menu now has a Controls
   button (vanilla `options.controls`, verified real via the language key list)
   opening it directly; the menu grid grew 24 dp for the row and the pause-menu
   test was corrected (it had a `- 6.0` fudge that only worked for the old
   height).

**Not fixed — captcha ("still says captcha failed").** No evidence yet, and the
previous guess (the mod_net hello) was evidently not the cause. What is ruled
out by reading: the login hello carries the correct offline UUID
(`OfflinePlayer:<name>` md5); the configuration phase now volunteers nothing but
`ClientInformation` + `minecraft:brand` (brand is exactly "vanilla"); no
known-packs entry; chat answers go out as a plain `serverbound_chat_message`
with `signature: None`. To pin it down rather than guess again, the next step
needs the exact kick text plus the client brand the server sees.

## Volt 1.1.0 — touch Freelook, crouch, Frame Items Loader

**Freelook did nothing on touch.** `try_freelook_turn` was only called from
`apply_mouse_look`, so the Android look path (`game.rs`, the `touch.update`
branch) turned `yaw`/`pitch` directly and the mod never saw a delta. Touch now
goes through `try_freelook_turn_degrees`, a degree-unit sibling of the
`Entity.turn`-unit entry point — the touch path feeds `camera_delta`, which has
already applied the 180-degree scale, so routing it through the 0.15 factor
would have run the camera at 15% speed. The on-screen **Look** button
(`Control::Freelook`, appended to the 14 old controls so saved layouts still
load: `deserialize_layouts` and `deserialize_visibility` accept 10/12/14/15) is
a latch in `VoltRuntime::freelook_touch_latch`, because a touch button cannot
hold a key. It shows only while the mod is on, and a real key press hands
control back to the hold/toggle binding.

**Crouch needed two taps on D-pad devices.** The sneak arm in
`TouchControls::update` only produced `Button::RightStick` on the second tap
within 500 ms, and only in `Mode::Dpad`; the first tap stored a timestamp and
returned `Role::Ignore`. Every scheme now routes through `sneak_toggles()`
(one tap, except while flying/swimming, where the button is a held descend
control). `last_sneak_tap` is gone.

**Frame Items Loader** (`VoltBoostPlusConfig::frame_items_loader`, saved as
`voltFrameItemsLoader`) skips the 1024-block `shouldRenderAtSqrDistance` check
and the frustum test in `frames.rs` for item frames only. Off by default and
deliberately excluded from `apply_volt_preset`: it draws more than vanilla, so
a Boost profile must not switch it on.

## Volt 1.1.0 follow-up — axis sensitivity, GUI framerate

**Horizontal and Vertical sensitivity are now on Volt Settings → Controls.**
`Opt::TouchSensitivityHorizontal`/`Vertical` already existed and already scaled
each axis on its own in `camera_delta` (`sensitivity * sensitivity_horizontal` on
x, `* sensitivity_vertical` on y), but the Volt Controls page only listed the
combined `TouchSensitivity`, so the per-axis controls were unreachable there.
Both are 10%–300%, applied continuously by `set_slider`, and the new
`axis_sensitivity_scales_its_own_axis_only` test pins that each one moves only
its own axis and that the lowest setting still turns the camera.

**Menus no longer clamp to a flat 60 FPS.** `framerate_limit` returned a literal
`60` for `ThrottleReason::OutOfLevelMenu`, so a screen opened over gameplay, or
any GUI at the title, ignored Max Framerate: setting 90 or 120 still showed 60
in every menu. It is now `option_limit.min(60)`, so a player who chose 30 keeps
30, and one who chose 120 keeps 120 in menus. This is what "it still lowers FPS
in all GUIs" was: not the Dynamic FPS mod, which only engages after 35 s without
input or when unfocused. The Volt idle-menu cap (after 10 s idle) also skips the
hidden-HUD state now, since hiding the HUD is not a menu.

## Volt 1.1.1 follow-up — crouch no longer raises the shield, status bar, tap-to-bind

**Crouching raised the shield.** `Game::poll_pads` gated the touch shield on
`if s.touch.shield_button { shield_held() } else { sneaking || held(RightStick) }`.
`shield_button` defaults to `false`, and `Control::Sneak` is mapped to
`Button::RightStick` (`touch::TouchControls::button`), which is also the pad's
stick click — so every crouch, toggle or stick click set `pad.touch_shield` and
`remote_use_hands` raised the off hand. Vanilla is the other way round:
`playerHasBlockingItemUseIntent` suppresses an off-hand shield while sneaking.
The decision now lives in `TouchControls::shield_raised(use_pressed, in_menu,
touch_visible)`: no raise while `sneaking`, and without a Shield button on the
HUD only the Use press (`Button::LeftTrigger`) raises. `crouching_never_raises_the_shield`
covers the full truth table. Note that `triggers[0]` is still forced to 1.0
while the shield is up, which is what `use_entity`/`use_block` check to skip the
main hand.

**The status bar corrupted menu text.** `targetSdk 35` means Android 15+ forces
edge-to-edge, so the system status bar is composited over the game surface:
`VoltActivity` extends `NativeActivity`, whose default decor no longer
reserves that strip. The "Volt Mods" title bar and the `Mouse`/`Back` header
buttons were drawn into the same rows as the clock and battery. The activity now
hides the status and navigation bars in `onCreate` and again on
`onWindowFocusChanged` (API 30+ `WindowInsetsController.hide`, older
`SYSTEM_UI_FLAG_IMMERSIVE_STICKY`), and sets
`LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES`. `setDecorFitsSystemWindows` is left
alone on purpose so the manifest's `adjustResize` still shrinks the view for the
soft keyboard.

**Tapping a bind did not set it.** `OptionsUi::key` and `bind_key` were already
correct; the problem was producing a `KeyCode` on a phone. The only taps that
carry one are the touch HUD's key grid (`touch_ui::KEYS`, `action()`), which is
collapsed until "..." is pressed and sits at `y = 3..23`, exactly under the
status bar. While a row waits, the render path now forces
`touch_ui.expanded = true`, and `Action::Key` no longer collapses the grid
during capture. `options_mouse` sends every screen tap through
`click(drag = false)`, and `android_aux_touch` resolves HUD grid taps *before*
that, so a grid key is consumed as a key rather than a mouse press. Nothing in
`OptionsUi` needed changing; the missing piece was reachability. A tap on the
virtual mouse's LMB/RMB (`Action::MouseButton`) also binds through
`options_waiting_for_key` -> `bind_mouse`, for a row that wants a mouse button.
