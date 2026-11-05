import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

import compat

class CompatibilityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'revoked.toml').write_text('')
        self.meta = {'schema': 1, 'id': 'test_mod', 'version': '1.0', 'game': '26.3',
                     'horizon': 'a' * 40, 'mixins': 'b' * 40, 'hooks': 1}
        self.mixin = ('[mod]\nid = "test_mod"\nversion = "1.0"\n'
                      '[compatibility]\nhorizon_revision = "' + 'a' * 40 + '"\n')

    def bundle(self, extra=None):
        archive = self.root / 'test.hnmod'
        with zipfile.ZipFile(archive, 'w') as z:
            z.writestr('horizon-native.json', json.dumps(self.meta))
            z.writestr('mixins/test_mod/horizon-mixin.toml', self.mixin)
            for name, contents in (extra or []):
                z.writestr(name, contents)
        entry = {'id': 'test_mod', 'versions': [{'version': '1.0', 'game': '26.3',
                   'file': 'test.hnmod', 'sha256': hashlib.sha256(archive.read_bytes()).hexdigest()}]}
        return archive, entry

    def extract(self, archive, entry):
        return compat.verified_extract(archive, entry, self.root / 'out', self.root)

    def test_valid_identity(self):
        archive, entry = self.bundle()
        self.assertEqual(self.extract(archive, entry)['meta'], self.meta)

    def test_checksum_fails_closed(self):
        archive, entry = self.bundle()
        entry['versions'][0]['sha256'] = '0' * 64
        with self.assertRaisesRegex(compat.Rejected, 'SHA-256'):
            self.extract(archive, entry)

    def test_signed_not_silently_accepted(self):
        archive, entry = self.bundle()
        entry['versions'][0]['signature'] = 'signature'
        with self.assertRaisesRegex(compat.Rejected, 'signature'):
            self.extract(archive, entry)

    def test_revoked_bundle(self):
        archive, entry = self.bundle()
        (self.root / 'revoked.toml').write_text('[[revoked]]\nid = "test_mod"\n')
        with self.assertRaisesRegex(compat.Rejected, 'revoked'):
            self.extract(archive, entry)

    def test_traversal_rejected(self):
        archive, entry = self.bundle([('mixins/test_mod/../../escape', 'bad')])
        with self.assertRaisesRegex(compat.Rejected, 'unsafe'):
            self.extract(archive, entry)
        self.assertFalse((self.root / 'escape').exists())

    def test_duplicate_rejected(self):
        archive, entry = self.bundle([('horizon-native.json', '{}')])
        with self.assertRaisesRegex(compat.Rejected, 'duplicate'):
            self.extract(archive, entry)

    def test_hook_version_and_unknown_target_rejected(self):
        data = {'meta': dict(self.meta, hooks=41), 'manifest': {'injection': [{'target': 'not_a_hook'}]}}
        with self.assertRaisesRegex(compat.Rejected, 'hook API'):
            compat.check_hooks(data, compat.ROOT, 'c' * 40)
        data['meta']['hooks'] = 1
        with self.assertRaisesRegex(compat.Rejected, 'non-hook'):
            compat.check_hooks(data, compat.ROOT, 'c' * 40)

    def test_post_build_rejected_even_exact_revision(self):
        data = {'meta': self.meta, 'manifest': {'post_build': [{'command': 'bad'}]}}
        with self.assertRaisesRegex(compat.Rejected, 'post-build'):
            compat.check_hooks(data, compat.ROOT, 'a' * 40)

if __name__ == '__main__':
    unittest.main()
