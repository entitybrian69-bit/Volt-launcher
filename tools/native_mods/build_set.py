#!/usr/bin/env python3
"""Compile a reviewed set of Horizon native bundles into an Android library.

Run ONLY in a disposable, isolated worker with no credentials, shared caches,
network access to private services, or signing keys. This is not a sandbox,
APK signer, public build service, or Android installer.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import compat

TARGETS = {"arm64-v8a": "aarch64-linux-android", "armeabi-v7a": "armv7-linux-androideabi"}
# Volt already has handwritten integrations for these exact module names.
# Applying them a second time can produce duplicate Rust modules/hooks.
BUILTIN = {"zoom", "fps_overlay", "keystrokes", "minimap"}
MAX_MODS = 32


def selection(value: str) -> tuple[str, Path]:
    mod_id, sep, path = value.partition("=")
    if not sep or not compat.ID.fullmatch(mod_id) or not path:
        raise argparse.ArgumentTypeError("expected a registry id and bundle path as ID=PATH")
    return mod_id, Path(path)


def prepare_set(registry: Path, selections: list[tuple[str, Path]], mixins_revision: str,
                root: Path, volt_revision: str, work: Path,
                raw_targets: bool = False) -> list[dict]:
    if not selections or len(selections) > MAX_MODS:
        raise compat.Rejected(f"select 1 to {MAX_MODS} native mods")
    ids = [mod_id for mod_id, _ in selections]
    if len(set(ids)) != len(ids):
        raise compat.Rejected("duplicate mod id")
    if not compat.REV.fullmatch(mixins_revision) or not compat.REV.fullmatch(volt_revision):
        raise compat.Rejected("invalid source or mixin revision")
    results = []
    mixins_dir = work / "mixins"
    mixins_dir.mkdir(parents=True)
    for mod_id, bundle in sorted(selections):
        if not compat.ID.fullmatch(mod_id):
            raise compat.Rejected("invalid mod id")
        if mod_id in compat.EXCLUDED or mod_id in BUILTIN:
            raise compat.Rejected(f"{mod_id}: excluded or already bundled in Volt")
        entry = compat.read_entry(registry, mod_id)
        extracted = work / "extracted" / mod_id
        data = compat.verified_extract(bundle, entry, extracted, registry)
        meta = data["meta"]
        if meta["mixins"] != mixins_revision:
            raise compat.Rejected(f"{mod_id}: mixin compiler revision differs")
        rebind = compat.check_hooks(data, root, volt_revision, raw_targets=raw_targets)
        if rebind:
            manifest_path = data["manifest_path"]
            text = manifest_path.read_text(encoding="utf-8")
            old = f'horizon_revision = "{meta["horizon"]}"'
            if text.count(old) != 1:
                raise compat.Rejected(f"{mod_id}: cannot safely rebind hook revision")
            manifest_path.write_text(text.replace(old, f'horizon_revision = "{volt_revision}"'), encoding="utf-8")
        shutil.copytree(extracted / "mixins" / mod_id, mixins_dir / mod_id)
        results.append({"id": mod_id, "version": meta["version"], "sha256": data["sha256"],
                        "horizon_revision": meta["horizon"], "hook_revision_rebound": rebind,
                        "raw_target_rebind": bool(raw_targets and rebind)})
    return results


def run(args: argparse.Namespace) -> dict:
    if not args.trust_native_code:
        raise compat.Rejected("--trust-native-code is required; native builds execute arbitrary code")
    rev = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=compat.ROOT, text=True).strip()
    if subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], cwd=compat.ROOT, text=True).strip():
        raise compat.Rejected("Volt tracked source is dirty; use a clean pinned checkout")
    report = {"schema": 1, "volt_commit": rev, "mixins_revision": args.mixins_revision,
              "abi": args.abi, "status": "not_built", "runtime_tested": False,
              "signed": False}
    args.output.mkdir(parents=True, exist_ok=True)
    # Never leave a stale success artifact when a subsequent request fails.
    library = args.output / "libhorizon_android.so"
    library.unlink(missing_ok=True)
    with tempfile.TemporaryDirectory(prefix="volt-native-set-") as tmp:
        work = Path(tmp)
        report["mods"] = prepare_set(args.registry, args.mod, args.mixins_revision, compat.ROOT, rev, work,
                                     raw_targets=getattr(args, "experimental_raw_targets", False))
        source = work / "source"
        source.mkdir()
        archive = work / "source.tar"
        with archive.open("wb") as f:
            subprocess.run(["git", "archive", rev], cwd=compat.ROOT, stdout=f, check=True)
        compat.extract_source_archive(archive, source)
        (source / ".horizon-revision").write_text(rev + "\n", encoding="utf-8")
        stage = work / "stage"
        env = compat.sanitized_env()
        env["CARGO_TARGET_DIR"] = str(work / "target")
        apply = subprocess.run([str(args.mixins.resolve()), "apply", "--workspace", str(source),
                                "--mods", str(work / "mixins"), "--out", str(stage)],
                               capture_output=True, text=True, env=env)
        (args.output / "apply.log").write_text(apply.stdout + "\n" + apply.stderr, encoding="utf-8")
        if apply.returncode:
            report["status"] = "apply_failed"
            return report
        build = subprocess.run([sys.executable, str(stage / "tools/android_build.py"), "--abi", args.abi,
                                "--public", "--rust-only"], cwd=stage, capture_output=True, text=True, env=env)
        (args.output / "build.log").write_text(build.stdout + "\n" + build.stderr, encoding="utf-8")
        if build.returncode:
            report["status"] = "build_failed"
            return report
        result = work / "target" / TARGETS[args.abi] / "release" / "libhorizon_android.so"
        if not result.is_file() or result.stat().st_size == 0:
            raise compat.Rejected("build returned success without an Android library")
        shutil.copyfile(result, library)
        report["library_sha256"] = hashlib.sha256(library.read_bytes()).hexdigest()
        report["status"] = "library_built"
        return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", type=Path, required=True)
    parser.add_argument("--mod", action="append", type=selection, required=True, help="registry ID=local verified bundle path; repeat for combinations")
    parser.add_argument("--mixins", type=Path, required=True, help="trusted pinned horizon-mixins executable")
    parser.add_argument("--mixins-revision", required=True)
    parser.add_argument("--abi", choices=tuple(TARGETS), required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--trust-native-code", action="store_true")
    parser.add_argument("--experimental-raw-targets", action="store_true",
                        help="probe-only: rebind revision-pinned bundles and let the mixin "
                             "compiler judge raw targets; output stays unsigned")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    for name in ("report.json", "apply.log", "build.log", "libhorizon_android.so"):
        (args.output / name).unlink(missing_ok=True)
    try:
        report = run(args)
    except (compat.Rejected, OSError, ValueError, subprocess.CalledProcessError) as error:
        report = {"schema": 1, "abi": args.abi, "status": "rejected", "reason": str(error),
                  "runtime_tested": False, "signed": False}
        (args.output / "libhorizon_android.so").unlink(missing_ok=True)
    (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "library_built" else 1

if __name__ == "__main__":
    raise SystemExit(main())
