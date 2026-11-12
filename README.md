<p align="center">
  <img src="docs/media/volt/logo.webp" width="360" alt="Volt Launcher logo: a dimensional white V on a black background">
</p>

<h1 align="center">Volt Launcher</h1>

<p align="center">
  <strong>A native Rust launcher for Minecraft: Java Edition 26.3 on Android.</strong>
</p>

<p align="center">
  <a href="docs/android.md#install-and-play"><img src="https://img.shields.io/badge/PLATFORM-ANDROID%209%2B-dc2626?style=for-the-badge&amp;logo=android&amp;logoColor=white&amp;labelColor=7f1d1d" alt="Platform: Android 9+ for modern builds"></a>
  <a href="https://github.com/entitybrian69-bit"><img src="https://img.shields.io/badge/MAINTAINED%20BY-ENTITYBRIAN-dc2626?style=for-the-badge&amp;logo=github&amp;logoColor=white&amp;labelColor=7f1d1d" alt="Maintained by Entitybrian"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/LICENSE-MIT-dc2626?style=for-the-badge&amp;labelColor=7f1d1d" alt="License: MIT"></a>
  <br>
  <a href="https://discord.gg/ED2bhF77"><img src="https://img.shields.io/badge/DISCORD-COMMUNITY%20%26%20SUGGESTIONS-dc2626?style=for-the-badge&amp;logo=discord&amp;logoColor=white&amp;labelColor=7f1d1d" alt="Discord: community and suggestions"></a>
  <a href="https://youtube.com/@team-aerix"><img src="https://img.shields.io/badge/YOUTUBE-TEAM%20AERIX-dc2626?style=for-the-badge&amp;logo=youtube&amp;logoColor=white&amp;labelColor=7f1d1d" alt="YouTube: Team Aerix"></a>
  <a href="https://entitybrian69-bit.github.io/Aerix-launcher/"><img src="https://img.shields.io/badge/AERIX%20LAUNCHER-JAVA%20LAUNCHER-dc2626?style=for-the-badge&amp;logo=githubpages&amp;logoColor=white&amp;labelColor=7f1d1d" alt="Aerix Launcher: Java launcher website"></a>
  <br>
  <a href="https://github.com/entitybrian69-bit/Volt-launcher/releases"><img src="https://img.shields.io/github/downloads/entitybrian69-bit/Volt-launcher/total?style=for-the-badge&amp;logo=github&amp;logoColor=white&amp;label=TOTAL%20DOWNLOADS&amp;labelColor=7f1d1d&amp;color=dc2626" alt="Total GitHub release-asset downloads"></a>
  <a href="https://github.com/entitybrian69-bit/Volt-launcher/releases"><img src="https://img.shields.io/github/downloads-pre/entitybrian69-bit/Volt-launcher/latest/total?sort=date&amp;style=for-the-badge&amp;logo=android&amp;logoColor=white&amp;label=THIS%20RELEASE&amp;labelColor=7f1d1d&amp;color=dc2626" alt="Downloads for the latest published release, including prereleases"></a>
  <a href="https://github.com/entitybrian69-bit/Volt-launcher/releases"><img src="https://img.shields.io/github/v/release/entitybrian69-bit/Volt-launcher?include_prereleases=&amp;sort=date&amp;style=for-the-badge&amp;logo=github&amp;logoColor=white&amp;label=LATEST%20BUILD&amp;labelColor=7f1d1d&amp;color=dc2626" alt="Latest published Volt Launcher build, including prereleases"></a>
</p>

<p align="center">
  <a href="https://github.com/entitybrian69-bit/Volt-launcher/releases"><strong>Download APK</strong></a> ·
  <a href="#screenshots">Screenshots</a> ·
  <a href="#owner-and-maintainer">Owner</a> ·
  <a href="#community-and-related-projects">Community</a> ·
  <a href="#build-from-source">Build guide</a>
</p>

Volt Launcher is an unofficial, touch-first Android launcher and native Rust
client for **Minecraft: Java Edition 26.3**, built on
[Horizon](https://github.com/ludovic111/horizon) and Horizon Android. Manage
profiles, install verified game resources and play without a Java runtime or
PC resource preparation on your phone.

> Volt is under active development. Check the release notes before installing;
> graphics-driver, account and server compatibility can vary by device and build.

## Download and play

1. Open [**Volt Launcher releases**](https://github.com/entitybrian69-bit/Volt-launcher/releases)
   and choose the APK for your device architecture.
2. Install the APK, open Volt and download the Minecraft 26.3 resources.
3. Wait for verification to finish, choose your profile and press **Play**.

Modern 64-bit builds target **Android 9+**. Experimental 32-bit builds have a
different minimum version and do not include the native `.hmod` runtime; see the
[architecture and compatibility table](docs/android.md#install-and-play).
The download badges count GitHub release assets across published builds,
including prereleases, rather than unique players.

## Developer tools and community mods

- **[Volt Survival Suite](mods/community/volt-survival-suite/README.md):** a real sandboxed WebAssembly HMOD with a survival HUD, target-block inspector, saved HUD toggle, and gameplay commands. [Build workflow](.github/workflows/build-volt-survival-suite.yml).
- **[HMOD Doctor](docs/hmod-doctor.md):** a CLI utility that inspects HMOD manifests, checks code/content package structure, validates content registries and detects duplicate IDs.

## What Volt includes

- **Native Rust gameplay** targeting Minecraft Java 26.3, with Vulkan and
  compatibility rendering paths.
- **Touch controls:** joystick or D-pad movement, tap or crosshair aiming,
  editable button layouts and custom key buttons.
- **Profiles and worlds:** separate settings and saves, with verified resource
  downloads reused after an interrupted installation.
- **Java multiplayer:** connect to matching-version servers; compatibility
  depends on the server and account configuration.
- **Add-ons:** resource packs and sandboxed `.hmod` mods on 64-bit builds.
  Fabric, Forge and other Java mod JARs do not run in this client.
- **Device integration:** resolution scaling, Android keyboard and clipboard,
  sound, vibration and accessibility settings.

See [Android feature support](docs/android-features.md) for implementation
status and remaining platform work.

## Owner and maintainer

<p align="center">
  <a href="https://github.com/entitybrian69-bit"><img src="docs/media/volt/owner.jpg" width="220" alt="Owner portrait supplied for Entitybrian"></a>
  <br>
  <strong>Entitybrian</strong>
  <br>
  Writes, tests and publishes Volt Launcher.
  <br>
  <a href="https://github.com/entitybrian69-bit">GitHub profile</a>
</p>

## Screenshots

*Screenshots supplied by the project owner. The interface and available options
can vary between builds.*

| **Home and profiles** | **Mods** |
| --- | --- |
| [![Volt Launcher home screen with a selected profile, Play button and launcher shortcuts](docs/media/volt/home.jpg)](docs/media/volt/home.jpg) | [![Volt Launcher mods list with install, remove and enable controls](docs/media/volt/mods.jpg)](docs/media/volt/mods.jpg) |

| **Resource packs** | **Shader packs** |
| --- | --- |
| [![Resource-pack browser with search, source filters and import controls](docs/media/volt/resource-packs.jpg)](docs/media/volt/resource-packs.jpg) | [![Shader-pack browser showing pack cards and compatibility information](docs/media/volt/shader-packs.jpg)](docs/media/volt/shader-packs.jpg) |

| **Performance settings** | **Accounts and skin wardrobe** |
| --- | --- |
| [![Performance presets, rendering controls and in-world optimization settings](docs/media/volt/performance.jpg)](docs/media/volt/performance.jpg) | [![Account controls and custom skin and cape wardrobe](docs/media/volt/accounts-skins.jpg)](docs/media/volt/accounts-skins.jpg) |

**Controls Studio and settings**

[![Controls Studio with layout import, appearance, graphics, input profile and diagnostic settings](docs/media/volt/controls-settings.jpg)](docs/media/volt/controls-settings.jpg)

## Community and related projects

- [**Discord — community and suggestions**](https://discord.gg/ED2bhF77)
- [**YouTube — Team Aerix**](https://youtube.com/@team-aerix)
- [**Aerix Launcher — Java launcher**](https://entitybrian69-bit.github.io/Aerix-launcher/)
- [**Volt Launcher on GitHub**](https://github.com/entitybrian69-bit/Volt-launcher)

Looking for a Java launcher? **Aerix Launcher** is the separate Java-launcher
project. Volt is the native Rust Android launcher in this repository.

## Build from source

The [Android build guide](docs/android.md#build-developers-only) covers the
SDK/NDK setup, resource preparation and APK build. Public builds download
original Minecraft resources and sounds from Mojang on the phone; game files,
prepared resources, accounts and signing keys stay outside Git.

For a focused contribution or a reproducible Android bug report, start with
[CONTRIBUTING_ANDROID.md](CONTRIBUTING_ANDROID.md) and the
[issue tracker](https://github.com/entitybrian69-bit/Volt-launcher/issues).
Include your phone, Android version, renderer, resolution scale and build
version with graphics or performance reports.

## Credits and license

Volt Launcher builds on Horizon Android and Horizon while preserving upstream
authorship and history. See the [upstream integration record](docs/android-upstream.json),
the original [MIT license](LICENSE) and [third-party notices](THIRD_PARTY_NOTICES.md).
The logo, owner portrait and app screenshots above were supplied by the project
owner for this README.

Minecraft code and assets belong to Mojang and Microsoft and are not covered by
the repository's MIT license. This project is not affiliated with or endorsed
by Mojang or Microsoft.

<details>
<summary>Upstream Horizon documentation (desktop)</summary>

The following is the upstream desktop project description. Its download links,
CI badges and parity claims describe upstream Horizon, not validation of this
Android fork.

<p align="center">
  <img src="docs/media/banner.webp" alt="The Horizon website: the game you know, rebuilt in Rust" width="100%">
</p>

<p align="center">
  <a href="https://github.com/horizon-rust/horizon/actions/workflows/ci.yml"><img src="https://github.com/horizon-rust/horizon/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <img src="https://img.shields.io/badge/Minecraft-Java%2026.3-5D9C3F" alt="Minecraft Java 26.3">
  <img src="https://img.shields.io/badge/Rust-stable-B7410E?logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/vanilla%20parity-100%25-F0795A" alt="Vanilla parity 100%">
  <a href="#project-status"><img src="https://img.shields.io/badge/player--visible%20gaps-833%20open-F0795A" alt="Player-visible gaps open"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-2A2470" alt="Linux, macOS and Windows">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-3B2F7E" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://playhorizon.dev"><b>Website</b></a> ·
  <a href="#download"><b>Download</b></a> ·
  <a href="https://playhorizon.dev/docs/"><b>Documentation</b></a> ·
  <a href="https://github.com/horizon-rust/horizon/releases"><b>Release notes</b></a> ·
  <a href="ROADMAP.md"><b>Roadmap</b></a> ·
  <a href="#project-status"><b>Project status</b></a> ·
  <a href="#contributing"><b>Contributing</b></a>
</p>

# Horizon

Minecraft: Java Edition 26.3, rewritten in Rust. Same worlds, same rules, same terrain, checked against the Java game by more than 300 differential harnesses. Free, open source and unofficial.

> Not an official Minecraft product, nor approved by or associated with Mojang or Microsoft. Horizon contains no Minecraft code, assets or data: you need your own copy of Java Edition 26.3.

## Download

Get **the Horizon launcher** from [playhorizon.dev](https://playhorizon.dev) or [the launcher releases](https://github.com/horizon-rust/horizon/releases?q=launcher-v&expanded=true) (Linux x86-64, macOS on Apple Silicon, Windows x64). It installs and updates the game, prepares its files from your copy of Minecraft, and updates itself.

**You need** a Microsoft account that owns Java Edition ([why](docs/microsoft-account.md)) and a GPU with Vulkan or OpenGL 3.3 (Linux), Vulkan or DirectX 12 (Windows), or an Apple Silicon Mac. An existing Minecraft install (official launcher, Prism, MultiMC...) is reused; without one, Play downloads 26.3 from Mojang for your account. Step by step: [Install and first launch](https://playhorizon.dev/docs/install).

<table>
  <tr>
    <td width="50%"><img src="docs/media/launcher-play.webp" alt="The launcher's Home page: the selected instance ready to play, the version picker, Join a friend and the instance's tabs"></td>
    <td width="50%"><img src="docs/media/launcher-settings.webp" alt="The launcher's Settings page: Microsoft account, offline name, updates, appearance and accent colour"></td>
  </tr>
</table>

## What works today

- **The game:** vanilla world generation, mobs, villagers, raids, redstone, every command, the HUD and the recipe book, on vanilla saves. Worlds from 1.8 to 26.2 are upgraded. [Status](#project-status).
- **Extras:** new menus, graphics presets, distant terrain up to 1024 chunks, controllers, zoom, minimap and more, off by default where they change the vanilla look. [Settings](https://playhorizon.dev/docs/settings).
- **Packs:** resource packs, and data packs with functions, predicates, item modifiers and world generation.
- **Multiplayer:** invite friends with a code, Open to LAN, join servers (online mode included). [Multiplayer](https://playhorizon.dev/docs/multiplayer).
- **Mods:** content mods (blocks, items, data) and native Rust mods, built by the launcher with nothing to install. Java mod jars do not run. [Mods](https://playhorizon.dev/docs/mods).

In the Rust client, with vanilla assets and no mods or shaders:

<table>
  <tr>
    <td colspan="2"><img src="docs/media/sunset.webp" alt="Sunset over a savanna river valley and a plains village"></td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/media/survival.webp" alt="Survival at dusk: a zombie walks up, with the hotbar, health, hunger and experience"></td>
    <td width="50%"><img src="docs/media/settings.webp" alt="Vanilla's Video Settings screen with Horizon's own buttons at the top: Horizon Graphics and Advanced Graphics"></td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/media/village.webp" alt="A plains village among savanna hills, with farms, villagers and an iron golem"></td>
    <td width="50%"><img src="docs/media/nether.webp" alt="A crimson forest in the Nether with a zombified piglin"></td>
  </tr>
</table>

## Host a server

The launcher's **Servers** page creates, starts, backs up and updates a server, and invites friends over Tailscale. On a VPS, `horizon-server` installs with one command and needs no Java. [Host a server](https://playhorizon.dev/docs/hosting) ([reference](docs/hosting.md)).

## Modpacks

The launcher imports Modrinth and CurseForge packs and plays the mods Horizon can carry over: today, content mods made of data and assets, not Java code. [Modpacks](https://playhorizon.dev/docs/modpacks).

## Agent control

AI agents and scripts can play Horizon. `horizon-ctl`, shipped with the game, reads its state as JSON and acts like a player; `horizon-ctl mcp` serves the same commands over MCP.

```sh
horizon-ctl launch --world Survival --seed 5 --wait   # finds the version the launcher installed
horizon-ctl state                                       # position, health, food, target, screen...
horizon-ctl find-block birch_log                        # the nearest logs
horizon-ctl goto -0.5 65 -2.5 --near 2.5                # walks there along a path
horizon-ctl break -1 65 -3                              # mines it; replies with the drops
horizon-ctl open-inventory && horizon-ctl click-slot 0  # menus: slots, buttons by label, text fields
```

Commands block until done and reply with the result; `horizon-ctl help` lists them all. [Agent control](docs/native-client.md#agent-control-horizon-ctl-and-mcp).

## Build from source

Needs Rust 1.90+, Python 3.10+, JDK 25, and on Linux `libasound2-dev libudev-dev`. `horizon-prepare` builds the game's resources from **your own copy** of Java Edition 26.3 (checked against `research-lock.json`) and never downloads it:

1. **Client:** play 26.3 once in the official launcher. Horizon reads `~/.minecraft` (or `--minecraft-dir` / `HORIZON_MINECRAFT_DIR`).
2. **Server jar:** the 26.3 server jar (`downloads.server.url` in `versions/26.3/26.3.json`), passed once with `--server-jar` or `HORIZON_SERVER_JAR`.

```sh
git clone https://github.com/horizon-rust/horizon.git
cd horizon
cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar   # needs JDK 25
cargo run --release -p horizon-prepare -- sounds   # optional: sounds from your launcher install
cargo run --release -p horizon-client
```

Prepared files land in the gitignored `.local/`: never commit or share them. Options, controls and architecture: [the native client guide](docs/native-client.md); all developer docs: [docs/README.md](docs/README.md).

## Roadmap

What is in progress, what comes next and where to help: [ROADMAP.md](ROADMAP.md), or [playhorizon.dev/hub](https://playhorizon.dev/hub) with the work board and what shipped.

## Contributing

Issues and pull requests are welcome: read [CONTRIBUTING.md](CONTRIBUTING.md) (vanilla first, small pull requests). Questions: [Discord](https://discord.gg/WsBxGsKyhU). Before a pull request, run the self-check and paste its output:

```sh
git config core.hooksPath .githooks
python3 tools/pr_ready.py
```

It runs rustfmt, clippy, the touched crates' tests, the Python tests, the public-tree check and the vanilla harnesses your change affects. Never commit game files, decompiled source or prepared resources.

## Project status

**Player-visible gaps: 833 open** of the 833 the [port ledger](tools/port_ledger.py) found, reading every vanilla 26.3 method against Horizon's code: vanilla behaviour a player would notice that Horizon doesn't have yet. By area: Screens, HUD and client packets [197](https://github.com/horizon-rust/horizon/issues/3518); Entity and block rendering, models [129](https://github.com/horizon-rust/horizon/issues/3519); Animals [110](https://github.com/horizon-rust/horizon/issues/3512); Server and players on the server [106](https://github.com/horizon-rust/horizon/issues/3520); Blocks, redstone and fluids [86](https://github.com/horizon-rust/horizon/issues/3516); Entities, projectiles, vehicles and decorations [64](https://github.com/horizon-rust/horizon/issues/3514); Monsters and bosses [53](https://github.com/horizon-rust/horizon/issues/3513); Mob AI, villagers and raids [35](https://github.com/horizon-rust/horizon/issues/3521); World, world generation and saves [32](https://github.com/horizon-rust/horizon/issues/3517); Items, enchantments and inventories [21](https://github.com/horizon-rust/horizon/issues/3515). This number goes down as gaps close (`tools/parity_gaps.py`); the harness results below say that what is tested matches.

The table is the project's own audit: every subsystem matches vanilla in every harness that compares it. 100% means 100% of what the harnesses compare, not that nothing else differs.

### Java parity: 100%

Scores: 0 (absent), 0.25 (started), 0.5 (playable, major features missing), 0.75 (mostly complete) or 1 (complete and verified by differential tests); the percentage is their average. Last audited 2026-10-13: all 281 harnesses pass on Linux and macOS. The [port ledger](tools/port_ledger.py) adds a method-by-method view of what is ported.

| Subsystem | Score | Main gaps |
| --- | ---: | --- |
| Window, input and game loop | 1 | None: game loop, options, camera, auto-jump, toggle sprint/sneak and frame limiter match |
| Assets and resources | 1 | None: item models match over 196,290 stacks; the resource pack screen over 3,000 sessions |
| Network protocol and multiplayer | 1 | None: joins vanilla servers; the dedicated server matches vanilla's files, login, configuration, play and command tree; LAN discovery over 462 cases (offline mode only) |
| Chunk storage and palettes | 1 | None: client and saves share paletted containers, identical to vanilla over 266 scenarios |
| Section meshing | 1 | None: 112 scenes and over a million quads identical to vanilla |
| Rendering | 1 | None: entity models, block entity renderers (407 cases) and animations (296), HUD overlays (2,200 states), the recipe book (600), player list, subtitles and world border match |
| Lighting | 1 | None: saved and relit light matches vanilla, slabs and stairs included |
| World generation | 1 | None: 2,352 whole chunks over 225 structure sites, in all three dimensions, identical to vanilla |
| Region files and saves | 1 | None: saves round-trip with vanilla; worlds from 1.8 to 26.2 upgrade as vanilla's DataFixers do |
| Entities and physics | 1 | None: attributes, spawn rules, natural spawning (15,500 cases), raids and patrols (1,900), `finalizeSpawn` (24,000) and mob AI (92 scenarios) match |
| Interaction and inventory | 1 | None: menu clicks over 8,000 sequences; recipe book placement (6,000), buttons (1,200), saved book (1,500), loom and crafter screens (1,600) |
| Menus, HUD and chat | 1 | None: every command (758 over 39,685 inputs), scoreboards (81,000 operations), the player list (872,000 draw calls), chat and its suggestions (3,000 sessions) match |
| Audio | 1 | None: the sound engine, sound events, music, situational music, ambience and subtitles match vanilla |
| Block behaviours and ticking | 1 | None: block updates, random ticks, redstone, dispensers (6,000 cases), crafters (4,000), container spills (4,000), copper golems and the sculk family (435 scenarios) match |

### Known issues

Skins served over https show the default skin unless already cached; pack icons show the unknown-pack texture in game; item tooltips lack attribute lines.

## Licence

[MIT](LICENSE), for the project's original work; dependencies keep their own licences ([notices](THIRD_PARTY_NOTICES.md)). Minecraft is a trademark of Mojang Synergies AB; its code and assets belong to Mojang and Microsoft under the [Minecraft EULA](https://www.minecraft.net/en-us/eula).

## Author

Made by [Ludovic Marie](https://lu4ovic.com) ([@ludovic111](https://github.com/ludovic111)).

</details>
