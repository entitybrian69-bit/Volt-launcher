# Horizon browser on Android

The launcher Mods tab now opens the live Horizon registry. Search by name or
author, page through results, and download from inside the launcher; use
**Volt mods** to return to the existing 50-switch catalog. `.hmod` files are
installed on the 64-bit APK through the registry's checked installer. The
32-bit build does not include the sandboxed mod runtime, so `.hmod` installation
is disabled there.

Native `.hnmod` projects require compilation into the Android game library.
The **Source** button downloads a validated source bundle into the selected
profile's `mod-browser/native-sources/`, but **does not install or run it**.
The browser reports this as source-only. Download checks the current registry's
revocations, SHA-256, size, signature when the registry supplies one, and
manifest. If the registry cannot be reached, native downloads fail instead of
using an old cached revocation list. The browsing index itself may be cached.
No downloaded Rust code is executed on the phone or in the signing pipeline.

The single prebuilt native project currently supported is **official Horizon
Zoom 2.0.0**. Its exact source bundle digest is checked against the registry
before the APK's bundled implementation is activated for that profile. Other
versions and projects remain source-only, even if the project has the same
name. An installed Zoom can be removed from either Mods view. The exact bundled
version can also be activated offline in Volt mods, as documented in
`bundled-zoom.md`.

This is **not yet a general native-mod build service or a prebuilt distribution
for every registry project**, and it cannot run arbitrary combinations. Those
require security and license review, separate architecture builds, independent
signing, conflict checks, revocation handling and runtime testing. Neither
successful download nor a registry listing proves an Android mod will work.
