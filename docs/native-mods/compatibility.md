# Horizon native mod compatibility work (experimental)

Horizon `.hnmod` files contain Rust source transformations; they are not
Android-loadable plugins. This is an **offline inventory and experimental
per-mod Android type-check probe**, not a general-purpose APK distribution service, security sandbox, or
compatibility certification. A separate limited Android browser and bundled
Zoom installer are described in `horizon-browser.md`; this probe does not
implement dynamic loading or arbitrary mod combinations.

## Registry snapshot

`horizon-registry-78853c19.json` lists the 72 native entries at the pinned
[horizon-mods commit 78853c19a6d48640ef500f3c8ff8988923576f3f](https://github.com/horizon-rust/horizon-mods/tree/78853c19a6d48640ef500f3c8ff8988923576f3f).
The inventory baseline labels all 144 ABI entries `not_checked`: inventory
alone validates metadata only, not bundles, compilation, runtime, accessibility,
safety or licenses. See the separately recorded probe result below. The
snapshot is not a live catalog and is not used by the launcher. The explicitly
removed Accurate Block Placement project is excluded from probing, even if
present in the upstream registry. Latest means the first listed `.hnmod`
version for game 26.3, not a semver calculation. A current revocation check is
needed before any eventual distribution; this snapshot alone cannot provide it.

To regenerate from a **separately verified checkout** of that registry commit:

```sh
python3 tools/native_mods/compat.py inventory --registry /path/to/horizon-mods \
  --registry-revision 78853c19a6d48640ef500f3c8ff8988923576f3f \
  --out docs/native-mods/horizon-registry-78853c19.json
python3 -m unittest discover -s tools/native_mods -v
```

The revision parameter records what the operator verified; the tool does not
verify the registry directory's provenance. Regenerate it deliberately and
review changes, including revocations, before trusting it.

## Recorded result

[Run 42021388714](https://github.com/entitybrian69-bit/Volt-launcher/actions/runs/42021388714)
checked official Zoom 2.0.0 (bundle SHA-256
`49fa2d7d635225c53304fbf1477606c0677eb23d94dac75b730f6c3ba065f127`)
on Volt commit `7ca0e07d1b62fa3e71368ace1a43a7e6d4b27367`:

| ABI | Result |
| --- | --- |
| arm64-v8a | `compile_passed` (type-check only) |
| armeabi-v7a | `compile_passed` (type-check only) |

This run did not link an APK or test runtime behavior or touch controls. None
of the other 71 projects has been compiled by this pipeline. Its CI artifacts
are retained for 14 days; their logs may be unavailable outside GitHub Actions.

## Probe

The `Native mod compatibility probe` workflow (manual or on changes to the
probe in an Arena branch) exercises
**only official Zoom 2.0.0** and runs both Android ABI checks independently on
fresh GitHub-hosted runners. It reconstructs the exact published bundle from
the source commit at which it was registered and compares its SHA-256 with the
pinned registry. The current upstream source snapshot produces a *different*
bundle and is not substituted. The mixin compiler is pinned to the revision in
the bundle. The workflow has read-only permissions and does not have release
secrets, signing keys, APK publishing, shared caches, or deployment access.
Reports and logs are diagnostic artifacts only, not release inputs.

To run a reviewed mod manually, use a **disposable, credential-free isolated
VM**, a clean committed Volt checkout, Android SDK/NDK, Rust 1.95.0 and both
Rust Android targets. Obtain and independently verify the registry snapshot,
corresponding bundle, and the `horizon-mixins` executable built from the exact
revision declared in the bundle. Never point this tool at a signing machine,
personal workstation with credentials, privileged runner, or shared build
cache. Environment filtering is only defense in depth: compiling a native mod
or dependency runs arbitrary code with the runner's permissions and network.

```sh
python3 tools/native_mods/compat.py probe --registry /path/to/registry \
  --id zoom --bundle /path/to/zoom-2.0.0.hnmod \
  --mixins /path/to/horizon-mixins \
  --mixins-revision 0d6b103c70633d8fd9443d8d0acd3aadaacedb06 \
  --abi arm64-v8a --output /tmp/zoom-probe --trust-native-code
# Repeat with --abi armeabi-v7a and a different output directory.
```

The probe checks the pinned archive hash and snapshot revocations, blocks
archives with unsafe paths or declared post-build commands, rejects signed
entries until author-signature verification is implemented, stages the exact
Volt commit with tracked symlinks omitted, and applies one mod in a separate
temporary workspace. If revisions differ, it only attempts a hook-based
revision rebind when the declared API and injection targets exist in Volt.
This is an experiment, **not** an assertion of source/API compatibility.
Module code and build scripts can still do anything; the restrictions do not
make a mod safe. The supplied mixin executable's identity and pinned registry
commit are the operator's responsibility. Multi-mod conflicts are not tested.

Results are `rejected`, `apply_failed`, `check_failed`, or `compile_passed`.
Failures may be infrastructure/tool failures, not necessarily incompatibility;
inspect the logs. A pass means only `cargo check` ran for that ABI, not that an
APK was linked, a mod was enabled, or a game ran on an Android device. Check
results and artifacts cannot be trusted as security evidence for unreviewed
code. A dirty tracked Volt checkout is refused. Temporary work is discarded.

## Work still needed for distribution

* Verify signatures and fresh revocations; review licenses, source, build
  scripts and transitive dependencies per mod and provide abuse handling.
* Check each published bundle and exact supported version on **both** ABIs;
  resolve source/API incompatibilities rather than silently claiming success.
* Build and test complete APKs and mod combinations, Android touch UI and
  behavior on actual devices; design real mod selection/update behavior.
* Establish independently reviewed, reproducible provenance and a separately
  controlled signing/release process. Same-package upgrades require the
  existing Volt signing certificate and increasing versionCode; untrusted mod
  builds must never receive that key. A single signed APK is not a dynamic
  plugin host and the normal updater is not a mod installer.
* Only then add an in-launcher browser/downloader that shows honest status and
  distributes approved compatible builds without requiring an external browser.

## Native build worker prototype

`tools/native_mods/build_set.py` is an experimental **offline worker**, not a
hosted service or an installer. It accepts multiple `--mod ID=PATH` entries,
checks the registry hash, revocation and manifests for each, applies the set
with the pinned Horizon mixin compiler, and attempts to compile an Android
native library. It writes an unsigned `.so`, logs, and a JSON report. One
mixin compiler revision must match every bundle; duplicate IDs, the excluded
project, and the four already-integrated mods are rejected. Successful
compilation does not imply the combination works at runtime or on touchscreens.

**Run only in a disposable, isolated, secret-free worker.** Mixins, Rust code,
Cargo build scripts and native code have full worker permissions; this script
is not a sandbox. Do not share credentials, signing keys, caches or trusted
network services with it. A build result is executable untrusted code, not an
APK. No artifact from the worker is automatically signed or published. A
production service still needs isolation, quota/abuse controls, a launcher
submission and status API, conflict and license review, a separate trusted
packaging/signing process, safe update distribution, and device validation.
Author-signed bundles remain rejected until signature verification is added.
