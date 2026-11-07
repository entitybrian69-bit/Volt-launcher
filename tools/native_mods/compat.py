#!/usr/bin/env python3
"""Offline Horizon native-mod inventory and isolated Volt Android compile probe.

This is a development/review tool, not an installer or a security sandbox.
See docs/native-mods/compatibility.md before using --trust-native-code.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import tarfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[2]
ABIS = ("arm64-v8a", "armeabi-v7a")
ID = re.compile(r"[a-z][a-z0-9_]{0,63}\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")
REV = re.compile(r"[0-9a-f]{40}\Z")
MAX_ARCHIVE = 16 * 1024 * 1024
MAX_EXPANDED = 64 * 1024 * 1024
MAX_FILES = 2048
EXCLUDED = {"accurate_block_placement"}

class Rejected(ValueError):
    pass

def read_entry(registry: Path, mod_id: str) -> dict:
    if not ID.fullmatch(mod_id):
        raise Rejected("invalid mod id")
    path = registry / "mods" / f"{mod_id}.toml"
    entry = tomllib.loads(path.read_text(encoding="utf-8"))
    if entry.get("id") != mod_id or entry.get("kind") != "native-mod":
        raise Rejected("entry is not the requested native mod")
    return entry

def latest(entry: dict) -> dict | None:
    """The newest 26.3 bundle. Most registry entries append new versions
    last, but some list them newest-first (skins_3d), so position alone
    lies: the published date decides, the version's numbers break ties,
    and the list position only settles entries carrying neither."""
    matching = [v for v in entry.get("versions", [])
                if v.get("game") == "26.3" and str(v.get("file", "")).endswith(".hnmod")]
    if not matching:
        return None
    def key(item):
        index, v = item
        numbers = [int(n) for n in re.findall(r"\d+", str(v.get("version", "")))]
        return (str(v.get("published", "")), numbers, index)
    return max(enumerate(matching), key=key)[1]

def inventory(registry: Path) -> dict:
    projects = []
    for path in sorted((registry / "mods").glob("*.toml")):
        raw = tomllib.loads(path.read_text(encoding="utf-8"))
        if raw.get("kind") != "native-mod":
            continue
        mod_id = raw.get("id", "")
        if not ID.fullmatch(mod_id) or path.stem != mod_id:
            raise Rejected(f"invalid registry entry {path.name}")
        ver = latest(raw)
        revoked = False
        if ver and ver.get("sha256"):
            try:
                check_revoked(registry, mod_id, ver["sha256"])
            except Rejected as e:
                if str(e) != "bundle is revoked in pinned registry":
                    raise
                revoked = True
        projects.append({"id": mod_id, "name": raw.get("name", mod_id),
                         "license": raw.get("license"), "source": raw.get("source"),
                         "version": ver.get("version") if ver else None,
                         "sha256": ver.get("sha256") if ver else None,
                         "arm64-v8a": "not_checked", "armeabi-v7a": "not_checked",
                         "policy": "excluded" if mod_id in EXCLUDED else "revoked" if revoked else "review_required"})
    return {"schema": 1, "game": "26.3", "projects": projects}

def check_revoked(registry: Path, mod_id: str, digest: str) -> None:
    path = registry / "revoked.toml"
    if not path.is_file():
        raise Rejected("pinned registry revocation list is missing")
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    for item in data.get("revoked", []):
        if item.get("id") == mod_id and (not item.get("sha256") or item["sha256"] == digest):
            raise Rejected("bundle is revoked in pinned registry")


def verify_minisign(data: bytes, key_b64: str, signature: str) -> None:
    """Verify a minisign signature block over `data` with the author key.

    Supports the two minisign algorithms: `Ed` (signature over the raw file)
    and `ED` (signature over the file's Blake2b-512 digest). The trusted
    comment's global signature is verified too. Fails closed on anything
    unexpected, including a missing cryptography backend.
    """
    import base64
    try:
        from cryptography.exceptions import InvalidSignature
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    except ImportError:
        raise Rejected("signed bundle: minisign verification needs the 'cryptography' package")
    try:
        raw_key = base64.b64decode(key_b64.strip(), validate=True)
    except (ValueError, TypeError):
        raise Rejected("signed bundle: author key is not valid base64")
    if len(raw_key) != 42 or raw_key[:2] != b"Ed":
        raise Rejected("signed bundle: unsupported author key format")
    key_id, public = raw_key[2:10], raw_key[10:]
    lines = [line.strip() for line in signature.strip().splitlines() if line.strip()]
    payloads = [line for line in lines if not line.startswith(("untrusted comment:", "trusted comment:"))]
    trusted = [line[len("trusted comment:"):].strip() for line in lines
               if line.startswith("trusted comment:")]
    if len(payloads) != 2 or len(trusted) != 1:
        raise Rejected("signed bundle: malformed minisign signature block")
    try:
        record = base64.b64decode(payloads[0], validate=True)
        global_sig = base64.b64decode(payloads[1], validate=True)
    except (ValueError, TypeError):
        raise Rejected("signed bundle: signature is not valid base64")
    if len(record) != 74 or len(global_sig) != 64:
        raise Rejected("signed bundle: malformed minisign signature record")
    alg, sig_key_id, sig = record[:2], record[2:10], record[10:]
    if sig_key_id != key_id:
        raise Rejected("signed bundle: signature was made with a different key")
    if alg == b"ED":
        message = hashlib.blake2b(data, digest_size=64).digest()
    elif alg == b"Ed":
        message = data
    else:
        raise Rejected("signed bundle: unsupported minisign algorithm")
    verifier = Ed25519PublicKey.from_public_bytes(public)
    try:
        verifier.verify(sig, message)
        verifier.verify(global_sig, sig + trusted[0].encode())
    except InvalidSignature:
        raise Rejected("signed bundle: minisign signature does not verify")

def verified_extract(archive: Path, entry: dict, dest: Path, registry: Path) -> dict:
    # The archive must be a published 26.3 bundle of this entry: the version
    # record is located by the archive's own digest rather than by list
    # position, because this fork pins exact versions deliberately and the
    # registry orders [[versions]] inconsistently (some entries append new
    # versions last, newer ones list them first). Revocation and the
    # author's signature still come from the pinned registry snapshot.
    if archive.stat().st_size > MAX_ARCHIVE:
        raise Rejected("archive too large")
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    ver = next((v for v in entry.get("versions", [])
                if v.get("game") == "26.3" and str(v.get("file", "")).endswith(".hnmod")
                and SHA.fullmatch(str(v.get("sha256", ""))) and v["sha256"] == digest), None)
    if ver is None:
        raise Rejected("bundle SHA-256 does not match any pinned registry version")
    check_revoked(registry, entry["id"], digest)
    # Signed entries must not silently lose author verification: when the
    # registry publishes an author key or a signature, both must be present
    # and the minisign signature must verify over the exact archive bytes.
    signature = str(ver.get("signature") or "")
    key = str(entry.get("key") or "")
    if signature or key:
        if not signature or not key:
            raise Rejected("signed bundle: signature and author key must both be present")
        verify_minisign(archive.read_bytes(), key, signature)
    seen = set()
    total = 0
    with zipfile.ZipFile(archive) as z:
        infos = z.infolist()
        if len(infos) > MAX_FILES:
            raise Rejected("too many ZIP entries")
        for info in infos:
            name = info.filename
            p = PurePosixPath(name)
            mode = info.external_attr >> 16
            if (not name or "\\" in name or p.is_absolute() or any(x in (".", "..") for x in p.parts)
                    or name in seen or stat.S_ISLNK(mode) or info.flag_bits & 1):
                raise Rejected("unsafe, duplicate, symlink or encrypted ZIP entry")
            seen.add(name)
            total += info.file_size
            if total > MAX_EXPANDED:
                raise Rejected("expanded bundle too large")
            if info.is_dir():
                continue
            if not (name == "horizon-native.json" or (len(p.parts) >= 3
                    and p.parts[:2] == ("mixins", entry["id"]))):
                raise Rejected("unexpected bundle path")
            out = dest.joinpath(*p.parts)
            out.parent.mkdir(parents=True, exist_ok=True)
            with z.open(info) as src, out.open("xb") as dst:
                while chunk := src.read(64 * 1024):
                    dst.write(chunk)
    meta_path = dest / "horizon-native.json"
    manifest_path = dest / "mixins" / entry["id"] / "horizon-mixin.toml"
    if not meta_path.is_file() or not manifest_path.is_file():
        raise Rejected("bundle is missing native or mixin manifest")
    meta = json.loads(meta_path.read_text(encoding="utf-8"))
    if (meta.get("schema") != 1 or meta.get("id") != entry["id"]
            or meta.get("version") != ver.get("version") or meta.get("game") != "26.3"
            or not REV.fullmatch(str(meta.get("horizon", "")))
            or not REV.fullmatch(str(meta.get("mixins", "")))):
        raise Rejected("native manifest does not match the registry")
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("mod", {}).get("id") != meta["id"] or manifest.get("mod", {}).get("version") != meta["version"]:
        raise Rejected("mixin identity differs from bundle")
    if manifest.get("compatibility", {}).get("horizon_revision") != meta["horizon"]:
        raise Rejected("mixin and bundle Horizon revisions differ")
    return {"meta": meta, "manifest": manifest, "sha256": digest, "manifest_path": manifest_path}

# This fork keeps the official chat screen in the render crate
# (crates/horizon-client-render/src/chat.rs, re-exported as
# horizon_client::chat): bundles built against the official layout are
# remapped to it when their revision is rebound.
FORK_REMAPS = (
    ("crates/horizon-client/src/chat.rs", "crates/horizon-client-render/src/chat.rs"),
    ("horizon_client::chat::", "horizon_client_render::chat::"),
)


def apply_fork_remaps(manifest_path: Path) -> bool:
    """Rewrite official source paths this fork keeps elsewhere; True when changed."""
    text = original = manifest_path.read_text(encoding="utf-8")
    for old_path, new_path in FORK_REMAPS:
        text = text.replace(old_path, new_path)
    if text != original:
        manifest_path.write_text(text, encoding="utf-8")
        return True
    return False


# Upstream API drift this fork cannot absorb behind a signature (a field
# whose type moved on under an assignment in reviewed bundle source): the
# staged source is adapted textually, each edit pinned to exactly one
# occurrence so any other bundle content fails loudly.
FORK_SOURCE_SHIMS = {
    "worldedit": (
        ("runtime.rs",
         "stack.data_mut().custom_name = Some(WAND_NAME.to_owned());",
         "stack.data_mut().custom_name ="
         " Some(horizon_world::nbt::Tag::String(WAND_NAME.into()));"),
        ("runtime.rs",
         "stack.data().and_then(|data| data.custom_name.as_deref()) == Some(WAND_NAME)",
         "stack.data().and_then(|data| data.custom_name_plain()).as_deref() == Some(WAND_NAME)"),
    ),
    "biomecompass": (
        ("item.rs",
         "d.custom_name = Some(bc::ITEM_NAME.to_owned());",
         "d.custom_name ="
         " Some(horizon_world::nbt::Tag::String(bc::ITEM_NAME.into()));"),
    ),
    "carryblock": (
        # The fork's Slots (official parity) grew a Native-menu variant after
        # the bundle's pinned era; a native mod's menu opens no world block.
        ("native.rs",
         "Slots::Ender => Vec::new(),",
         "Slots::Ender | Slots::Native => Vec::new(),"),
    ),
    "horizonmatica": (
        # ui::slider lost its tenth (dragging) argument when the fork
        # restored the upstream signature; the drag-aware track moved to
        # ui::slider_dragging.
        ("menus_game.rs",
         "ui::slider(self.batch, &game.gui.atlas, r[0], r[1], r[2],"
         " &Text::literal(label), shown, hover, true, dragging);",
         "ui::slider_dragging(self.batch, &game.gui.atlas, r[0], r[1], r[2],"
         " &Text::literal(label), shown, hover, true, dragging);"),
    ),
    "skins_3d": (
        # Upstream moved NativeMod from game::native_hooks to a mod_api
        # module after this fork's era; the struct itself is identical.
        ("horizon-mixin.toml",
         'ty = "&mut Vec<crate::mod_api::NativeMod>"',
         'ty = "&mut Vec<crate::game::native_hooks::NativeMod>"'),
        ("hooks/list.rs",
         "fn skins3d_listed(mods: &mut Vec<crate::mod_api::NativeMod>) {",
         "fn skins3d_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {"),
        ("glue.rs",
         "pub(super) fn listed(mods: &mut Vec<horizon_client::mod_api::NativeMod>) {\n"
         "    mods.push(horizon_client::mod_api::NativeMod {",
         # game.rs is compiled inside the horizon-android crate, so the
         # woven glue reaches NativeMod via crate::, not the client lib.
         "pub(super) fn listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {\n"
         "    mods.push(crate::game::native_hooks::NativeMod {"),
        # native_camera (the extracted world frame's camera) postdates this
        # fork; the mod only needs a position for its reach cull, and the
        # player's eye is that camera for every drawn player model.
        ("glue.rs",
         "let camera = game.native_camera().map(|c| c.position);",
         "let camera = game.eye_position();"),
        # Polygon grew a cull_back field upstream after this fork's era.
        ("cache.rs",
         "part.polygons = built.into_iter().map(|(normal, vertices)|"
         " Polygon { normal, vertices, cull_back: false }).collect();",
         "part.polygons = built.into_iter().map(|(normal, vertices)|"
         " Polygon { normal, vertices }).collect();"),
    ),
}


def apply_fork_source_shims(mod_id: str, mixins_dir: Path) -> int:
    """Adapt a staged bundle's sources to this fork; the count of edits."""
    edited = 0
    for name, old, new in FORK_SOURCE_SHIMS.get(mod_id, ()):
        path = mixins_dir / name
        text = path.read_text(encoding="utf-8")
        if text.count(old) != 1:
            raise Rejected(f"{mod_id}: expected exactly one shim site in {name}")
        path.write_text(text.replace(old, new, 1), encoding="utf-8")
        edited += 1
    return edited


def propagate_bundle_crates(stage: Path) -> list[str]:
    """List a bundle crate woven into horizon-client in horizon-android too.

    horizon-android compiles horizon-client's sources as its own lib (its
    [lib] path points into ../horizon-client/src), so a [[crate]] dependency
    the mixin compiler writes into horizon-client's [dependencies] must also
    be one of horizon-android's. Returns the packages it propagated.
    """
    client = stage / "crates" / "horizon-client" / "Cargo.toml"
    android = stage / "crates" / "horizon-android" / "Cargo.toml"
    if not client.is_file() or not android.is_file():
        return []
    android_text = android.read_text(encoding="utf-8")
    added = []
    for line in client.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        package = stripped.split("=", 1)[0].strip()
        if ("native-crates/" in stripped and "path" in stripped and package
                and f"\n{package} =" not in "\n" + android_text):
            added.append(stripped)
    if not added:
        return []
    marker = "[dependencies]"
    if marker not in android_text:
        raise Rejected("horizon-android has no [dependencies] table to extend")
    android_text = android_text.replace(marker, marker + "\n" + "\n".join(added), 1)
    android.write_text(android_text, encoding="utf-8")
    return [entry.split("=", 1)[0].strip() for entry in added]


def check_hooks(data: dict, root: Path, volt_rev: str, raw_targets: bool = False) -> bool:
    """Return whether the staged manifest needs safe hook-only revision rebinding."""
    meta, manifest = data["meta"], data["manifest"]
    if manifest.get("post_build") or manifest.get("post-build"):
        raise Rejected("post-build commands are not supported")
    if meta["horizon"] == volt_rev:
        return False
    allowed = {"mod", "compatibility", "injection", "module"}
    if raw_targets:
        # Experimental probe path: a bundle may vendor whole crates; the mixin
        # compiler stages them and the compiler's checks decide the rest.
        allowed.add("crate")
    if set(manifest) - allowed:
        raise Rejected("non-hook transformations need exact pinned Horizon source")
    hooks = meta.get("hooks")
    source = (root / "crates/horizon-client/src/game/native_hooks.rs").read_text()
    found = re.search(r"const NATIVE_HOOKS_API: u32 = (\d+);", source)
    if not found:
        raise Rejected("Volt source does not state its native hook API version")
    if not isinstance(hooks, int) or isinstance(hooks, bool) or hooks < 1:
        injections = manifest.get("injection", [])
        raw = [str(inj.get("target")) for inj in injections if "target" in inj]
        named = [(inj.get("hook"), inj.get("hook_version")) for inj in injections if "target" not in inj]
        available = json.loads((root / "native-hooks.json").read_text())["hooks"]
        names = {(item["name"], item["version"]) for item in available}
        missing = sorted({f"{h}/{v}" for (h, v) in named if (h, v) not in names})
        if raw_targets and not missing and injections:
            # Experimental probe-only path: rebind the revision and let the
            # mixin compiler's strict AST matching decide whether each raw
            # target still exists unchanged in Volt. Never used for signing.
            return True
        raise Rejected(
            f"bundle pins Horizon {meta.get('horizon', '')[:12]} exactly without a hook-API "
            f"declaration; {len(raw)} raw target injections (first: {raw[:6]}), "
            f"{len(named)} named hook injections, missing from Volt: {missing[:25]}"
        )
    served = int(found.group(1))
    if hooks > served and not raw_targets:
        raise Rejected(
            f"needs native hook API {hooks}; Volt serves {served} "
            "(the mod targets a newer Horizon than this fork has ported)"
        )
    # Experimental probe-only path (raw_targets): a declared level above
    # Volt's falls through to the per-injection checks below. Named hooks
    # must exist in native-hooks.json; raw targets are left to the mixin
    # compiler's strict AST matching. Never used for signing.
    available = json.loads((root / "native-hooks.json").read_text())["hooks"]
    targets = {item["target"] for item in available}
    names = {(item["name"], item["version"]) for item in available}
    injections = manifest.get("injection", [])
    if not injections:
        raise Rejected("no explicit hook injections; exact Volt revision required")
    for inj in injections:
        if "target" in inj:
            if inj["target"] not in targets and not raw_targets:
                raise Rejected("non-hook target needs its exact pinned Horizon source")
        elif (inj.get("hook"), inj.get("hook_version")) not in names:
            raise Rejected("named hook unavailable")
    return True

def extract_source_archive(archive: Path, source: Path) -> None:
    """Extract a git archive without symlinks or paths outside the workspace."""
    with tarfile.open(archive) as tar:
        for member in tar:
            path = PurePosixPath(member.name)
            if not member.name or "\\" in member.name or path.is_absolute() or ".." in path.parts:
                raise Rejected("unsafe path in Volt source archive")
            if member.issym():
                continue
            if not (member.isfile() or member.isdir()):
                raise Rejected("unsupported member in Volt source archive")
            out = source.joinpath(*path.parts)
            if member.isdir():
                out.mkdir(parents=True, exist_ok=True)
                continue
            out.parent.mkdir(parents=True, exist_ok=True)
            src = tar.extractfile(member)
            if src is None:
                raise Rejected("unreadable Volt source archive member")
            with src, out.open("xb") as dst:
                shutil.copyfileobj(src, dst)
            out.chmod(member.mode & 0o777)

def sanitized_env() -> dict:
    # The job must still run in an ephemeral, secret-free runner. Removing env
    # credentials is defense in depth, not isolation or a security sandbox.
    allowed = ("HOME", "PATH", "USER", "TMPDIR", "LANG", "LC_ALL", "CARGO_HOME", "RUSTUP_HOME",
               "ANDROID_HOME", "ANDROID_SDK_ROOT", "JAVA_HOME", "CARGO_BUILD_JOBS")
    return {k: v for k, v in os.environ.items() if k in allowed}

def run_probe(args) -> dict:
    if not args.trust_native_code:
        raise Rejected("explicit --trust-native-code is required; native builds run arbitrary code")
    if args.id in EXCLUDED:
        raise Rejected("mod excluded by Volt policy")
    entry = read_entry(args.registry, args.id)
    rev = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    if subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], cwd=ROOT, text=True).strip():
        raise Rejected("Volt tracked source is dirty; use a clean pinned checkout")
    report = {"schema": 1, "id": args.id, "volt_commit": rev, "abi": args.abi,
              "status": "not_checked", "runtime_tested": False}
    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="volt-native-probe-") as td:
        work = Path(td)
        data = verified_extract(args.bundle, entry, work / "extracted", args.registry)
        meta = data["meta"]
        report.update(bundle_sha256=data["sha256"], version=meta["version"],
                      horizon_revision=meta["horizon"], mixins_revision=meta["mixins"])
        if args.mixins_revision != meta["mixins"]:
            raise Rejected("mixin tool revision must match the bundle")
        rebind = check_hooks(data, ROOT, rev, raw_targets=getattr(args, "experimental_raw_targets", False))
        report["raw_target_rebind"] = bool(getattr(args, "experimental_raw_targets", False) and rebind)
        if rebind:
            manifest_path = data["manifest_path"]
            text = manifest_path.read_text(encoding="utf-8")
            original = f'horizon_revision = "{meta["horizon"]}"'
            if text.count(original) != 1:
                raise Rejected("cannot safely rebind revision in staged manifest")
            manifest_path.write_text(text.replace(original, f'horizon_revision = "{rev}"'), encoding="utf-8")
            apply_fork_remaps(manifest_path)
            apply_fork_source_shims(entry["id"], manifest_path.parent)
        report["hook_revision_rebound"] = rebind
        # Mixins rejects symlinks. Archive the exact commit instead of copying a
        # working tree, omit tracked symlinks and label the source revision.
        source = work / "source"
        source.mkdir()
        archive = work / "source.tar"
        with archive.open("wb") as f:
            subprocess.run(["git", "archive", rev], cwd=ROOT, stdout=f, check=True)
        extract_source_archive(archive, source)
        (source / ".horizon-revision").write_text(rev + "\n", encoding="utf-8")
        stage = work / "stage"
        env = sanitized_env()
        env["CARGO_TARGET_DIR"] = str(work / "target")
        mixin = subprocess.run([str(args.mixins.resolve()), "apply", "--workspace", str(source),
                                "--mods", str(work / "extracted" / "mixins"), "--out", str(stage)],
                               capture_output=True, text=True, env=env)
        (args.output / "apply.log").write_text(mixin.stdout + "\n" + mixin.stderr, encoding="utf-8")
        if mixin.returncode:
            report["status"] = "apply_failed"
        else:
            propagated = propagate_bundle_crates(stage)
            if propagated:
                report["propagated_crates"] = propagated
            check = subprocess.run([sys.executable, str(stage / "tools/android_build.py"),
                                    "--abi", args.abi, "--check"], cwd=stage,
                                   capture_output=True, text=True, env=env)
            (args.output / "android-check.log").write_text(check.stdout + "\n" + check.stderr, encoding="utf-8")
            report["status"] = "compile_passed" if check.returncode == 0 else "check_failed"
    return report

def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    subs = p.add_subparsers(dest="command", required=True)
    inv = subs.add_parser("inventory", help="list native registry entries; does not execute code")
    inv.add_argument("--registry", type=Path, required=True)
    inv.add_argument("--out", type=Path, required=True)
    inv.add_argument("--registry-revision", required=True, help="verified registry snapshot commit")
    probe = subs.add_parser("probe", help="apply one reviewed bundle and cross-check one ABI")
    probe.add_argument("--registry", type=Path, required=True)
    probe.add_argument("--id", required=True)
    probe.add_argument("--bundle", type=Path, required=True)
    probe.add_argument("--mixins", type=Path, required=True)
    probe.add_argument("--mixins-revision", required=True)
    probe.add_argument("--abi", choices=ABIS, required=True)
    probe.add_argument("--output", type=Path, required=True)
    probe.add_argument("--trust-native-code", action="store_true")
    probe.add_argument("--experimental-raw-targets", action="store_true",
                       help="probe-only: rebind revision-pinned bundles and let the mixin "
                            "compiler judge raw targets; output stays unsigned")
    args = p.parse_args()
    try:
        if args.command == "inventory":
            if not REV.fullmatch(args.registry_revision):
                raise Rejected("registry revision must be a full commit hash")
            data = inventory(args.registry)
            data["registry_revision"] = args.registry_revision
            args.out.parent.mkdir(parents=True, exist_ok=True)
            args.out.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
            print(f"Listed {len(data['projects'])} native projects; none marked compatible.")
        else:
            data = run_probe(args)
            (args.output / "report.json").write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
            print(json.dumps(data, indent=2))
            return 0 if data["status"] == "compile_passed" else 1
    except (Rejected, OSError, ValueError, zipfile.BadZipFile, subprocess.CalledProcessError) as e:
        if args.command == "probe":
            args.output.mkdir(parents=True, exist_ok=True)
            (args.output / "report.json").write_text(json.dumps({"schema": 1, "id": args.id, "abi": args.abi,
                "status": "rejected", "reason": str(e), "runtime_tested": False}, indent=2) + "\n", encoding="utf-8")
        print(f"Rejected: {e}", file=sys.stderr)
        return 2
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
