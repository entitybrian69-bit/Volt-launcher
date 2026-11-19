#!/usr/bin/env python3
"""Check a private, fresh-home Volt Play run without logging account or world details."""
import json
from pathlib import Path
import sys


def verify(home, version):
    resources = home / '.local' / 'horizon-client' / version
    for name in ('android-downloads-v2.json', 'audio-complete.json', 'asset-index.json'):
        if not (resources / name).is_file():
            raise ValueError('Fresh official installation did not finish')
    if json.loads((resources / 'horizon-resources.json').read_bytes()).get('minecraft') != version:
        raise ValueError('Installed engine tables do not match the pinned version')
    report = json.loads((home / 'licensed-world.json').read_bytes())
    if ('error' not in report or report['error'] is not None or report.get('signed_in') is not True
            or type(report.get('world_frames')) is not int or report['world_frames'] < 120):
        raise ValueError('No successful signed-in world render (120 presented world frames required)')
    if not any(p.is_file() and p.stat().st_size for p in (home / 'saves').glob('*/level.dat')):
        raise ValueError('No saved singleplayer world in the fresh default profile')


if __name__ == '__main__':
    pin = json.loads((Path(__file__).resolve().parents[1] / 'research-lock.json').read_bytes())
    try:
        verify(Path(sys.argv[1]), pin['minecraft']['version'])
    except (OSError, ValueError) as err:
        # Keep paths (e.g. the world name) and the private report out of CI logs.
        raise SystemExit(f'Licensed world evidence failed ({type(err).__name__})') from None
    print('Official install markers, signed-in session, rendered world frames and saved world verified.')
