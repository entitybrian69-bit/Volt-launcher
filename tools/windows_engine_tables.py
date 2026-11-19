#!/usr/bin/env python3
"""Build or audit the generated-table input to Volt's official resource installer.

Never package the prepared resource directory itself: it contains game assets.
Use the same explicit allowlist as a public Android build.
"""
import argparse
import json
from pathlib import Path, PurePosixPath
import stat
import zipfile

from android_build import generated_engine_file

ROOT = Path(__file__).resolve().parents[1]
VERSION = json.loads((ROOT / 'research-lock.json').read_text())['minecraft']['version']
REQUIRED = {'horizon-resources.json', 'worldgen/blocks/structure-blocks.json',
            'facts.json', 'registries.json', 'items.json', 'model-states.json'}


def verify(path):
    with zipfile.ZipFile(path) as archive:
        seen = set()
        total = 0
        for member in archive.infolist():
            name = member.filename
            total += member.file_size
            if (total > 8 * 1024**3 or member.file_size == 0 or name in seen or '\\' in name
                    or PurePosixPath(name).as_posix() != name or not generated_engine_file(name)
                    or stat.S_ISLNK(member.external_attr >> 16)):
                raise ValueError(f'Invalid generated-table member: {name}')
            seen.add(name)
        if REQUIRED - seen:
            raise ValueError(f'Missing required tables: {sorted(REQUIRED - seen)}')
        if archive.getinfo('horizon-resources.json').file_size > 16 * 1024**2:
            raise ValueError('Oversized engine manifest')
        manifest = json.loads(archive.read('horizon-resources.json'))
        if manifest.get('minecraft') != VERSION:
            raise ValueError('Minecraft version differs from research-lock.json')
        if archive.testzip() is not None:
            raise ValueError('Corrupt generated-table archive')
    print(f'Checked {len(seen)} allowlisted tables for Minecraft {VERSION}.')


def build(source, output):
    source = source.resolve(strict=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    part = output.with_suffix('.partial')
    try:
        with zipfile.ZipFile(part, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=1) as archive:
            for path in sorted(source.rglob('*')):
                name = path.relative_to(source).as_posix()
                if not generated_engine_file(name):
                    continue
                if path.resolve(strict=True) != path or not path.is_file():
                    raise ValueError(f'Not a regular generated table: {name}')
                archive.write(path, name)
        verify(part)
        part.replace(output)
    finally:
        part.unlink(missing_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('--from-directory', type=Path, help='Prepared, version-specific resource directory')
    args = parser.parse_args()
    if args.from_directory:
        build(args.from_directory, args.archive)
    else:
        verify(args.archive)
