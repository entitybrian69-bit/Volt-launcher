#!/usr/bin/env python3
"""Pack the vendored native-mod bundles committed under vendored/ into the
deterministic .hnmod archives their registry entries pin.

A vendored mod is one whose reviewed source tree lives in this repository
(vendored/<id>/ holding horizon-native.json and mixins/<id>/...) with a
registry-style entry beside it (vendored/<id>.toml). This script rebuilds
the bundle byte-for-byte (sorted paths, fixed timestamps, stored entries),
checks its SHA-256 against the entry's pin, and places both the bundle and
the entry where the ordinary verified weave pipeline looks:

    pack_vendored.py --bundles <dir for .hnmod files> --registry <snapshot>

Every downstream check still runs: compat.verified_extract re-hashes the
archive against the entry, walks the ZIP safety rules, and the mixin
compiler judges the injections. Nothing here bypasses validation; it only
feeds the pipeline a bundle built from reviewed, committed sources.
"""
from __future__ import annotations

import argparse
import hashlib
import shutil
import sys
import tomllib
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
VENDORED = HERE / "vendored"
# Any fixed date works; this one marks the day vendored weaving shipped.
STAMP = (2026, 11, 9, 0, 0, 0)


def build_bundle(source: Path, out: Path) -> str:
    """Deterministically zip `source` (a vendored mod dir) to `out`."""
    files = sorted(p for p in source.rglob("*") if p.is_file())
    if not (source / "horizon-native.json").is_file():
        raise SystemExit(f"{source}: vendored bundle lacks horizon-native.json")
    with zipfile.ZipFile(out, "w", zipfile.ZIP_STORED) as z:
        for p in files:
            rel = p.relative_to(source).as_posix()
            info = zipfile.ZipInfo(rel, date_time=STAMP)
            info.external_attr = 0o644 << 16
            z.writestr(info, p.read_bytes())
    return hashlib.sha256(out.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundles", type=Path, required=True,
                        help="directory to write the .hnmod files into")
    parser.add_argument("--registry", type=Path, default=None,
                        help="registry snapshot to copy the entries into (mods/ subdir)")
    args = parser.parse_args()
    args.bundles.mkdir(parents=True, exist_ok=True)
    entries = sorted(VENDORED.glob("*.toml"))
    if not entries:
        raise SystemExit("no vendored entries found")
    for entry_path in entries:
        entry = tomllib.loads(entry_path.read_text(encoding="utf-8"))
        mod_id = entry.get("id", "")
        source = VENDORED / mod_id
        versions = [v for v in entry.get("versions", []) if v.get("game") == "26.3"]
        if entry.get("kind") != "native-mod" or not source.is_dir() or len(versions) != 1:
            raise SystemExit(f"{entry_path.name}: not a packable vendored native mod")
        ver = versions[0]
        out = args.bundles / str(ver["file"])
        digest = build_bundle(source, out)
        if digest != ver.get("sha256"):
            out.unlink(missing_ok=True)
            raise SystemExit(
                f"{mod_id}: packed bundle sha256 {digest} does not match the "
                f"pinned {ver.get('sha256')} - update vendored/{mod_id}.toml "
                "only after reviewing the source change")
        print(f"{mod_id}: packed {out.name} sha256 {digest}")
        if args.registry is not None:
            dest = args.registry / "mods" / entry_path.name
            if dest.exists():
                raise SystemExit(
                    f"{mod_id}: the registry snapshot already carries mods/{entry_path.name}; "
                    "a vendored entry must not shadow a published one")
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(entry_path, dest)
            print(f"{mod_id}: entry installed into the registry snapshot")
    return 0


if __name__ == "__main__":
    sys.exit(main())
