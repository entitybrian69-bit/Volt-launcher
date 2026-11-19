import json
from pathlib import Path
import tempfile
import unittest

from tools.check_licensed_world import verify


class LicensedWorldEvidenceTests(unittest.TestCase):
    def test_reject_partial_install_title_or_offline_runs(self):
        # Fixtures exercise the verifier only. They are not licensed acceptance evidence.
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            resources = home / '.local/horizon-client/test'
            resources.mkdir(parents=True)
            (resources / 'horizon-resources.json').write_text('{"minecraft":"test"}')
            for name in ('android-downloads-v2.json', 'audio-complete.json', 'asset-index.json'):
                (resources / name).write_text('{}')
            save = home / 'saves/World/level.dat'
            save.parent.mkdir(parents=True)
            save.write_bytes(b'fixture')
            good = dict(signed_in=True, world_frames=120, error=None)
            report = home / 'licensed-world.json'
            for change in [dict(world_frames=0), dict(signed_in=False), dict(error='failed')]:
                report.write_text(json.dumps(good | change))
                with self.assertRaises(ValueError):
                    verify(home, 'test')
            report.write_text(json.dumps(good))
            verify(home, 'test')
            save.unlink()
            with self.assertRaisesRegex(ValueError, 'saved'):
                verify(home, 'test')
            (resources / 'audio-complete.json').unlink()
            with self.assertRaisesRegex(ValueError, 'installation'):
                verify(home, 'test')
