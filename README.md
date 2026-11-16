<div align="center">

<img src="docs/media/volt/logo.png" alt="Volt Launcher" width="300">

# VOLT LAUNCHER

### A native Rust Minecraft launcher and client for Android

**Android 9+ · Vulkan · Minecraft Java 26.3 · Touch controls · Sandboxed `.vmod` mods**

[![Release](https://img.shields.io/github/v/release/entitybrian69-bit/Volt-launcher?color=ff2d2d&label=release&style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/releases)
[![Downloads](https://img.shields.io/github/downloads/entitybrian69-bit/Volt-launcher/total?color=ff2d2d&label=downloads&style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/releases)
[![Stars](https://img.shields.io/github/stars/entitybrian69-bit/Volt-launcher?color=ff2d2d&label=stars&style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/stargazers)
[![Discord](https://img.shields.io/badge/discord-join-5865f2?logo=discord&logoColor=white&style=for-the-badge)](https://discord.gg/WsBxGsKyhU)

</div>

---

<div align="center">

## ⚡ Download

| Android device | Download |
| :--- | :---: |
| **arm64-v8a** — almost every phone since 2016 | [![arm64-v8a](https://img.shields.io/badge/⬇%20arm64--v8a-73%20MB-ff2d2d?style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/releases/download/debug/Volt-Launcher-debug-arm64-v8a.apk) |
| **armeabi-v7a** — older 32-bit phones | [![armeabi-v7a](https://img.shields.io/badge/⬇%20armeabi--v7a-51%20MB-ff2d2d?style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/releases/download/debug/Volt-Launcher-debug-armeabi-v7a.apk) |
| **x86_64** — emulators, Chromebooks | [![x86_64](https://img.shields.io/badge/⬇%20x86__64-81%20MB-ff2d2d?style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/releases/download/debug/Volt-Launcher-debug-x86_64.apk) |
| **x86** — older emulators | [![x86](https://img.shields.io/badge/⬇%20x86-74%20MB-ff2d2d?style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/releases/download/debug/Volt-Launcher-debug-x86.apk) |

<sub>The rolling debug build installs as <b>Volt Debug</b>, so it sits beside a stable install. [All releases →](https://github.com/entitybrian69-bit/Volt-launcher/releases)</sub>

</div>

---

<div align="center">

## 📊 Volt by the numbers

<table>
  <tr>
    <td align="center" bgcolor="#ff2d2d">
      <a href="https://github.com/entitybrian69-bit/Volt-launcher/releases"><img src="https://img.shields.io/badge/205-APK_DOWNLOADS-white?style=for-the-badge" alt="205 APK downloads"></a>
      <br><sub>DOWNLOADS</sub>
    </td>
    <td align="center" bgcolor="#ff2d2d">
      <a href="https://github.com/entitybrian69-bit/Volt-launcher/releases/tag/v1.1.1"><img src="https://img.shields.io/badge/v1.1.1-RELEASE-white?style=for-the-badge" alt="v1.1.1"></a>
      <br><sub>RELEASE</sub>
    </td>
    <td align="center" bgcolor="#ff2d2d">
      <img src="https://img.shields.io/badge/2026--10--31-REPO_CREATED-white?style=for-the-badge" alt="Repository created 31 October 2026">
      <br><sub>MADE ON</sub>
    </td>
    <td align="center" bgcolor="#ff2d2d">
      <a href="https://discord.gg/WsBxGsKyhU"><img src="https://img.shields.io/badge/DISCORD-JOIN-white?style=for-the-badge&logo=discord&logoColor=5865f2" alt="Join the Discord"></a>
      <br><sub>COMMUNITY</sub>
    </td>
  </tr>
</table>

</div>

---

<div align="center">

<picture>
  <img src="docs/media/volt/volt-animated.svg" alt="Volt Launcher — play, mod, shade, go" width="900">
</picture>

</div>

## What works

- Four touch schemes: joystick or D-pad, each with tap or crosshair aiming.
- Editable button positions, sizes and opacity; handedness, circle size, optional
  360-degree movement, Auto Sprint and Sprint-button visibility.
- Java multiplayer, including the Hypixel Arcade lobby shown in **In game** below.
- Native launcher, verified resource downloads, resource packs and `.vmod` import.
- Vulkan rendering modes and resolution scaling; Android keyboard, clipboard,
  AAudio sound and phone vibration through the existing game settings.
- Account status and Android Keystore storage; the current sign-in configuration
  is still for development. [Feature support and remaining work](docs/android-features.md).

The engine targets Minecraft Java **26.3**. Sandboxed `.vmod`
code mods run on 64-bit APKs. Source-only `.hnmod` mods require a separately
compiled client and are not supported on Android yet. Fabric/Forge Java code
does not run in this client. Server and anticheat
compatibility requires testing on the actual server.



---

## 👤 Owner

<table>
  <tr>
    <td width="230" valign="top">
      <img src="docs/media/volt/owner.png" alt="Entitybrian, owner of Volt Launcher" width="210">
    </td>
    <td valign="top">
      <h3>Entitybrian</h3>
      <p>Creator and maintainer of <b>Volt Launcher</b>. Built the native Rust launcher, the
      mod browser, the profile system and the on-device build pipeline.</p>
      <p>
        <a href="https://github.com/entitybrian69-bit"><img src="https://img.shields.io/badge/GitHub-entitybrian69--bit-181717?style=for-the-badge&logo=github&logoColor=white" alt="GitHub"></a>
        <a href="https://discord.gg/WsBxGsKyhU"><img src="https://img.shields.io/badge/Discord-join%20the%20server-5865f2?style=for-the-badge&logo=discord&logoColor=white" alt="Discord"></a>
      </p>
      <blockquote>
        <p><b>Questions, bug reports and build announcements all go in the
        <a href="https://discord.gg/WsBxGsKyhU">Discord</a>.</b> That is the fastest place to
        reach me, and where new APKs are posted first.</p>
      </blockquote>
    </td>
  </tr>
</table>

---

<div align="center">

## 📱 Screenshots

<sub>Captured on a Poco M8 Pro · 2412 × 1080</sub>

| | |
| :---: | :---: |
| [<img src="docs/media/volt/launcher-1.png" alt="Volt Launcher home screen: Play button, instance, renderer and Volt Boost settings">](docs/media/volt/launcher-1.png)<br><sub><b>Home</b> — Play, instance and renderer at a glance</sub> | [<img src="docs/media/volt/launcher-5.png" alt="Volt Launcher home screen with the instance panel">](docs/media/volt/launcher-5.png)<br><sub><b>Home</b> — the instance panel</sub> |
| [<img src="docs/media/volt/launcher-2.png" alt="Volt Launcher Mods page listing Volt Zoom, FPS Overlay, Keystrokes, Minimap, JEI and Volt Physics">](docs/media/volt/launcher-2.png)<br><sub><b>Mods</b> — install or switch off per profile</sub> | [<img src="docs/media/volt/launcher-6.png" alt="Volt Launcher Mods page showing which official mods are built into the APK">](docs/media/volt/launcher-6.png)<br><sub><b>Mods</b> — what is built into the APK</sub> |

[<img src="docs/media/volt/launcher-4.png" alt="Volt Launcher Choose profile dialog with Profile 0, Profile 1 and Create profile">](docs/media/volt/launcher-4.png)

<sub><b>Profiles</b> — separate saves, mods and packs</sub>

</div>

---

<sub>

**Volt Launcher is unofficial.** It is not affiliated with, endorsed by or
approved by Mojang or Microsoft. It contains no Minecraft code or assets: you
need your own copy of Java Edition. Mojang textures are covered by the
[Minecraft EULA](https://www.minecraft.net/en-us/eula), not by this project's
licence; original work is [MIT](LICENSE) and dependencies keep their own terms
([notices](THIRD_PARTY_NOTICES.md)). Game files, prepared resources, accounts,
APKs and signing keys stay outside Git.

</sub>

<details>
<summary>Upstream Volt documentation (desktop)</summary>

The following is the upstream desktop project description. Its download links,
CI badges and parity claims describe upstream Volt, not validation of this
Android fork.

<p align="center">
  <img src="docs/media/banner.webp" alt="The Volt website: the game you know, rebuilt in Rust" width="100%">
</p>

<p align="center">
  <a href="https://github.com/volt-rust/volt/actions/workflows/ci.yml"><img src="https://github.com/volt-rust/volt/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <img src="https://img.shields.io/badge/Minecraft-Java%2026.3-5D9C3F" alt="Minecraft Java 26.3">
  <img src="https://img.shields.io/badge/Rust-stable-B7410E?logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/vanilla%20parity-100%25-F0795A" alt="Vanilla parity 100%">
  <a href="#project-status"><img src="https://img.shields.io/badge/player--visible%20gaps-833%20open-F0795A" alt="Player-visible gaps open"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-2A2470" alt="Linux, macOS and Windows">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-3B2F7E" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://playvolt.dev"><b>Website</b></a> ·
  <a href="#download"><b>Download</b></a> ·
  <a href="https://playvolt.dev/docs/"><b>Documentation</b></a> ·
  <a href="https://github.com/volt-rust/volt/releases"><b>Release notes</b></a> ·
  <a href="ROADMAP.md"><b>Roadmap</b></a> ·
  <a href="#project-status"><b>Project status</b></a> ·
  <a href="#contributing"><b>Contributing</b></a>
</p>

# Volt

Minecraft: Java Edition 26.3, rewritten in Rust. Same worlds, same rules, same terrain, checked against the Java game by more than 300 differential harnesses. Free, open source and unofficial.

> Not an official Minecraft product, nor approved by or associated with Mojang or Microsoft. Volt contains no Minecraft code, assets or data: you need your own copy of Java Edition 26.3.

## Download

Get **the Volt launcher** from [playvolt.dev](https://playvolt.dev) or [the launcher releases](https://github.com/volt-rust/volt/releases?q=launcher-v&expanded=true) (Linux x86-64, macOS on Apple Silicon, Windows x64). It installs and updates the game, prepares its files from your copy of Minecraft, and updates itself.

**You need** a Microsoft account that owns Java Edition ([why](docs/microsoft-account.md)) and a GPU with Vulkan or OpenGL 3.3 (Linux), Vulkan or DirectX 12 (Windows), or an Apple Silicon Mac. An existing Minecraft install (official launcher, Prism, MultiMC...) is reused; without one, Play downloads 26.3 from Mojang for your account. Step by step: [Install and first launch](https://playvolt.dev/docs/install).

<table>
  <tr>
    <td width="50%"><img src="docs/media/launcher-play.webp" alt="The launcher's Home page: the selected instance ready to play, the version picker, Join a friend and the instance's tabs"></td>
    <td width="50%"><img src="docs/media/launcher-settings.webp" alt="The launcher's Settings page: Microsoft account, offline name, updates, appearance and accent colour"></td>
  </tr>
</table>

## What works today

- **The game:** vanilla world generation, mobs, villagers, raids, redstone, every command, the HUD and the recipe book, on vanilla saves. Worlds from 1.8 to 26.2 are upgraded. [Status](#project-status).
- **Extras:** new menus, graphics presets, distant terrain up to 1024 chunks, controllers, zoom, minimap and more, off by default where they change the vanilla look. [Settings](https://playvolt.dev/docs/settings).
- **Packs:** resource packs, and data packs with functions, predicates, item modifiers and world generation.
- **Multiplayer:** invite friends with a code, Open to LAN, join servers (online mode included). [Multiplayer](https://playvolt.dev/docs/multiplayer).
- **Mods:** content mods (blocks, items, data) and native Rust mods, built by the launcher with nothing to install. Java mod jars do not run. [Mods](https://playvolt.dev/docs/mods).

In the Rust client, with vanilla assets and no mods or shaders:

<table>
  <tr>
    <td colspan="2"><img src="docs/media/sunset.webp" alt="Sunset over a savanna river valley and a plains village"></td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/media/survival.webp" alt="Survival at dusk: a zombie walks up, with the hotbar, health, hunger and experience"></td>
    <td width="50%"><img src="docs/media/settings.webp" alt="Vanilla's Video Settings screen with Volt's own buttons at the top: Volt Graphics and Advanced Graphics"></td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/media/village.webp" alt="A plains village among savanna hills, with farms, villagers and an iron golem"></td>
    <td width="50%"><img src="docs/media/nether.webp" alt="A crimson forest in the Nether with a zombified piglin"></td>
  </tr>
</table>

## Host a server

The launcher's **Servers** page creates, starts, backs up and updates a server, and invites friends over Tailscale. On a VPS, `volt-server` installs with one command and needs no Java. [Host a server](https://playvolt.dev/docs/hosting) ([reference](docs/hosting.md)).

## Modpacks

The launcher imports Modrinth and CurseForge packs and plays the mods Volt can carry over: today, content mods made of data and assets, not Java code. [Modpacks](https://playvolt.dev/docs/modpacks).

## Agent control

AI agents and scripts can play Volt. `volt-ctl`, shipped with the game, reads its state as JSON and acts like a player; `volt-ctl mcp` serves the same commands over MCP.

```sh
volt-ctl launch --world Survival --seed 5 --wait   # finds the version the launcher installed
volt-ctl state                                       # position, health, food, target, screen...
volt-ctl find-block birch_log                        # the nearest logs
volt-ctl goto -0.5 65 -2.5 --near 2.5                # walks there along a path
volt-ctl break -1 65 -3                              # mines it; replies with the drops
volt-ctl open-inventory && volt-ctl click-slot 0  # menus: slots, buttons by label, text fields
```

Commands block until done and reply with the result; `volt-ctl help` lists them all. [Agent control](docs/native-client.md#agent-control-volt-ctl-and-mcp).

## Build from source

Needs Rust 1.90+, Python 3.10+, JDK 25, and on Linux `libasound2-dev libudev-dev`. `volt-prepare` builds the game's resources from **your own copy** of Java Edition 26.3 (checked against `research-lock.json`) and never downloads it:

1. **Client:** play 26.3 once in the official launcher. Volt reads `~/.minecraft` (or `--minecraft-dir` / `VOLT_MINECRAFT_DIR`).
2. **Server jar:** the 26.3 server jar (`downloads.server.url` in `versions/26.3/26.3.json`), passed once with `--server-jar` or `VOLT_SERVER_JAR`.

```sh
git clone https://github.com/volt-rust/volt.git
cd volt
cargo run --release -p volt-prepare -- --prepare --server-jar ~/Downloads/server.jar   # needs JDK 25
cargo run --release -p volt-prepare -- sounds   # optional: sounds from your launcher install
cargo run --release -p volt-client
```

Prepared files land in the gitignored `.local/`: never commit or share them. Options, controls and architecture: [the native client guide](docs/native-client.md); all developer docs: [docs/README.md](docs/README.md).

## Roadmap

What is in progress, what comes next and where to help: [ROADMAP.md](ROADMAP.md), or [playvolt.dev/hub](https://playvolt.dev/hub) with the work board and what shipped.

## Contributing

Issues and pull requests are welcome: read [CONTRIBUTING.md](CONTRIBUTING.md) (vanilla first, small pull requests). Questions: [Discord](https://discord.gg/HcTeMZxe5E). Before a pull request, run the self-check and paste its output:

```sh
git config core.hooksPath .githooks
python3 tools/pr_ready.py
```

It runs rustfmt, clippy, the touched crates' tests, the Python tests, the public-tree check and the vanilla harnesses your change affects. Never commit game files, decompiled source or prepared resources.

## Project status

**Player-visible gaps: 833 open** of the 833 the [port ledger](tools/port_ledger.py) found, reading every vanilla 26.3 method against Volt's code: vanilla behaviour a player would notice that Volt doesn't have yet. By area: Screens, HUD and client packets [197](https://github.com/volt-rust/volt/issues/3518); Entity and block rendering, models [129](https://github.com/volt-rust/volt/issues/3519); Animals [110](https://github.com/volt-rust/volt/issues/3512); Server and players on the server [106](https://github.com/volt-rust/volt/issues/3520); Blocks, redstone and fluids [86](https://github.com/volt-rust/volt/issues/3516); Entities, projectiles, vehicles and decorations [64](https://github.com/volt-rust/volt/issues/3514); Monsters and bosses [53](https://github.com/volt-rust/volt/issues/3513); Mob AI, villagers and raids [35](https://github.com/volt-rust/volt/issues/3521); World, world generation and saves [32](https://github.com/volt-rust/volt/issues/3517); Items, enchantments and inventories [21](https://github.com/volt-rust/volt/issues/3515). This number goes down as gaps close (`tools/parity_gaps.py`); the harness results below say that what is tested matches.

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
