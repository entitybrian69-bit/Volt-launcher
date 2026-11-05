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
    return next((v for v in entry.get("versions", [])
                 if v.get("game") == "26.3" and str(v.get("file", "")).endswith(".hnmod")), None)

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


def verified_extract(archive: Path, entry: dict, dest: Path, registry: Path) -> dict:
    ver = latest(entry)
    if not ver or not SHA.fullmatch(str(ver.get("sha256", ""))):
        raise Rejected("no SHA-256-pinned 26.3 native bundle")
    if archive.stat().st_size > MAX_ARCHIVE:
        raise Rejected("archive too large")
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    if digest != ver["sha256"]:
        raise Rejected("bundle SHA-256 does not match pinned registry entry")
    check_revoked(registry, entry["id"], digest)
    # Signed entries must not silently lose author verification. They need an
    # independent minisign-verification path before they can be built here.
    if ver.get("signature") or entry.get("key"):
        raise Rejected("signed bundle: author signature verification not implemented")
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

def check_hooks(data: dict, root: Path, volt_rev: str) -> bool:
    """Return whether the staged manifest needs safe hook-only revision rebinding."""
    meta, manifest = data["meta"], data["manifest"]
    if manifest.get("post_build") or manifest.get("post-build"):
        raise Rejected("post-build commands are not supported")
    if meta["horizon"] == volt_rev:
        return False
    if set(manifest) - {"mod", "compatibility", "injection", "module"}:
        raise Rejected("non-hook transformations need exact pinned Horizon source")
    hooks = meta.get("hooks")
    source = (root / "crates/horizon-client/src/game/native_hooks.rs").read_text()
    found = re.search(r"const NATIVE_HOOKS_API: u32 = (\d+);", source)
    if not isinstance(hooks, int) or isinstance(hooks, bool) or not found or hooks < 1 or hooks > int(found.group(1)):
        raise Rejected("hook API unavailable or above Volt's version")
    available = json.loads((root / "native-hooks.json").read_text())["hooks"]
    targets = {item["target"] for item in available}
    names = {(item["name"], item["version"]) for item in available}
    injections = manifest.get("injection", [])
    if not injections:
        raise Rejected("no explicit hook injections; exact Volt revision required")
    for inj in injections:
        if "target" in inj:
            if inj["target"] not in targets:
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
        rebind = check_hooks(data, ROOT, rev)
        if rebind:
            manifest_path = data["manifest_path"]
            text = manifest_path.read_text(encoding="utf-8")
            original = f'horizon_revision = "{meta["horizon"]}"'
            if text.count(original) != 1:
                raise Rejected("cannot safely rebind revision in staged manifest")
            manifest_path.write_text(text.replace(original, f'horizon_revision = "{rev}"'), encoding="utf-8")
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
