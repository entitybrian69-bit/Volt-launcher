# HMOD Doctor

Volt includes a small command-line inspector for Horizon .hmod packages. It uses the same
horizon-mods package and content validators as the launcher/game code; it does not start
Minecraft or modify installed files.

## Build and run

From the Volt repository root:

    cargo run --manifest-path launcher/Cargo.toml --bin hmod-doctor -- --dir ./horizon-mods

Inspect one package:

    cargo run --manifest-path launcher/Cargo.toml --bin hmod-doctor -- --dir ./example.hmod

Machine-readable output:

    cargo run --manifest-path launcher/Cargo.toml --bin hmod-doctor -- --dir ./horizon-mods --json

## What it checks

- Reads each package manifest and checks that a code mod actually contains mod.wasm.
- Distinguishes WASM code mods, content-only mods and hybrid packages.
- Runs the Horizon content validator on packages declaring registries.
- Reports API version, side, declared capabilities and validation errors.
- Detects duplicate mod IDs in the scanned folder.
- Returns a non-zero exit status if any package is invalid or IDs collide.

The tool is diagnostic only. It does not enable mods, execute WASM, download files, or change worlds.
