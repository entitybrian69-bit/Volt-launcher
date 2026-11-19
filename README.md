<div align="center">

<img src="docs/media/volt/logo.png" alt="Volt Launcher" width="300">

# VOLT LAUNCHER

### A native Rust Minecraft launcher and client for Android

**Android 9+ · Vulkan · Minecraft Java 26.3 · Touch controls · Sandboxed `.hmod` mods**

[![Release](https://img.shields.io/github/v/release/entitybrian69-bit/Volt-launcher?color=ff2d2d&label=release&style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/releases)
[![Downloads](https://img.shields.io/github/downloads/entitybrian69-bit/Volt-launcher/total?color=ff2d2d&label=downloads&style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/releases)
[![Stars](https://img.shields.io/github/stars/entitybrian69-bit/Volt-launcher?color=ff2d2d&label=stars&style=for-the-badge)](https://github.com/entitybrian69-bit/Volt-launcher/stargazers)
[![Discord](https://img.shields.io/badge/discord-join-5865f2?logo=discord&logoColor=white&style=for-the-badge)](https://discord.gg/WsBxGsKyhU)

</div>

---

<div align="center">

**Windows PC port:** [Migration status, feature inventory and developer build](docs/windows-port.md). [Windows 11 administrator setup and private sign-in test](docs/windows-acceptance.md). No Windows installer is available yet.

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
- Native launcher, verified resource downloads, resource packs and `.hmod` import.
- Vulkan rendering modes and resolution scaling; Android keyboard, clipboard,
  AAudio sound and phone vibration through the existing game settings.
- Account status and Android Keystore storage; the current sign-in configuration
  is still for development. [Feature support and remaining work](docs/android-features.md).

The engine targets Minecraft Java **26.3**. Sandboxed `.hmod`
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


**Volt Launcher is unofficial.** It is not affiliated with, endorsed by or
approved by Mojang or Microsoft. It contains no Minecraft code or assets: you
need your own copy of Java Edition. Mojang textures are covered by the
[Minecraft EULA](https://www.minecraft.net/en-us/eula), not by this project's
licence; original work is [MIT](LICENSE) and dependencies keep their own terms
([notices](THIRD_PARTY_NOTICES.md)). Game files, prepared resources, accounts,
APKs and signing keys stay outside Git.

</sub>

## 📱 Screenshots

<sub>Captured on a Poco M8 Pro · 2412 × 1080</sub>

| | |
| :---: | :---: |
| [<img src="docs/media/volt/launcher-1.png" alt="Volt Launcher home screen: Play button, instance, renderer and Volt Boost settings">](docs/media/volt/launcher-1.png)<br><sub><b>Home</b> — Play, instance and renderer at a glance</sub> | [<img src="docs/media/volt/launcher-5.png" alt="Volt Launcher home screen with the instance panel">](docs/media/volt/launcher-5.png)<br><sub><b>Home</b> — the instance panel</sub> |
| [<img src="docs/media/volt/launcher-2.png" alt="Volt Launcher Mods page listing Horizon Zoom, FPS Overlay, Keystrokes, Minimap, JEI and Horizon Physics">](docs/media/volt/launcher-2.png)<br><sub><b>Mods</b> — install or switch off per profile</sub> | [<img src="docs/media/volt/launcher-6.png" alt="Volt Launcher Mods page showing which official mods are built into the APK">](docs/media/volt/launcher-6.png)<br><sub><b>Mods</b> — what is built into the APK</sub> |

[<img src="docs/media/volt/launcher-4.png" alt="Volt Launcher Choose profile dialog with Profile 0, Profile 1 and Create profile">](docs/media/volt/launcher-4.png)

<sub><b>Profiles</b> — separate saves, mods and packs</sub>

</div>

---

<sub>
