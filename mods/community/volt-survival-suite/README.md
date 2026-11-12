# Volt Survival Suite

A client-side Horizon WebAssembly mod maintained with Volt. It is a real code HMOD,
not a resource pack or content-only archive.

## Features

- Live top-left HUD: XYZ coordinates, cardinal facing, health, hunger, dimension and world clock.
- Target-block readout using Horizon's raycast snapshot (up to six blocks).
- /kit toggles the HUD and saves the choice in per-mod storage.
- /where prints exact coordinates and dimension.
- /inspect prints the block ID and coordinates under the crosshair.
- /time prints the current world day and time.

The mod deliberately uses the non-experimental API 0.1 capabilities
game-read, hud, chat and storage. It does not request inventory access,
teleportation, network access or filesystem access outside its own storage.

## Build

The source is in mods/community/volt-survival-suite. Run its host tests with:

    cargo test --manifest-path mods/community/volt-survival-suite/Cargo.toml

The GitHub Actions workflow builds for wasm32-unknown-unknown, componentizes the
WASM using wasm-tools, packages horizon-mod.toml + mod.wasm into a ZIP-compatible
.hmod, validates the component/archive, and uploads the installable artifact.

## Install

After the workflow succeeds, download the volt-survival-suite-hmod artifact from
the workflow run, extract volt_survival_suite-0.1.0.hmod, then import it through
Volt/Horizon Android's Mods > Import file or HTTPS import flow. On a supported
64-bit build, enable it under Options > Mods and restart the game.

## Limitations

This mod reads the local player snapshot and the block currently hit by the
crosshair. It does not inspect inventory, equipped armor, biome, entities or
FPS because this project intentionally stays within the capabilities requested
in its manifest. The HMOD is built by CI; a successful source commit alone is
not evidence that a device has run it.
