import hashlib
import io
import tarfile
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

    def test_fork_remaps_rewrite_official_chat_paths(self):
        import tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory() as td:
            m = Path(td) / 'horizon-mixin.toml'
            m.write_text('destination = "crates/horizon-client/src/chat.rs"\n'
                         'target = "horizon_client::chat::ChatScreen::draw"\n',
                         encoding='utf-8')
            self.assertTrue(compat.apply_fork_remaps(m))
            text = m.read_text(encoding='utf-8')
            self.assertIn('crates/horizon-client-render/src/chat.rs', text)
            self.assertIn('horizon_client_render::chat::ChatScreen::draw', text)
            self.assertFalse(compat.apply_fork_remaps(m))

    def test_latest_prefers_the_newest_appended_version(self):
        entry = {'versions': [
            {'version': '1.0', 'game': '26.3', 'file': 'a-1.0.hnmod'},
            {'version': '2.0', 'game': '26.2', 'file': 'a-2.0.hnmod'},
            {'version': '1.2', 'game': '26.3', 'file': 'a-1.2.hnmod'},
        ]}
        self.assertEqual(compat.latest(entry)['version'], '1.2')

    def sign(self, archive, entry, tamper_sig=False, wrong_key_id=False):
        import base64
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
        secret = Ed25519PrivateKey.generate()
        public = secret.public_key().public_bytes_raw()
        key_id = b'\x07' * 8
        entry['key'] = base64.b64encode(b'Ed' + key_id + public).decode()
        digest = hashlib.blake2b(archive.read_bytes(), digest_size=64).digest()
        sig = secret.sign(digest)
        if tamper_sig:
            sig = bytes([sig[0] ^ 1]) + sig[1:]
        trusted = 'test 1.0 for Horizon 26.3'
        global_sig = secret.sign(sig + trusted.encode())
        record_id = (b'\x08' * 8) if wrong_key_id else key_id
        entry['versions'][0]['signature'] = (
            'untrusted comment: signature from minisign secret key\n'
            + base64.b64encode(b'ED' + record_id + sig).decode()
            + '\ntrusted comment: ' + trusted + '\n'
            + base64.b64encode(global_sig).decode())

    def test_valid_minisign_signature_accepted(self):
        try:
            import cryptography  # noqa: F401
        except ImportError:
            self.skipTest('cryptography not installed')
        archive, entry = self.bundle()
        self.sign(archive, entry)
        self.assertEqual(self.extract(archive, entry)['meta'], self.meta)

    def test_bad_minisign_signature_rejected(self):
        try:
            import cryptography  # noqa: F401
        except ImportError:
            self.skipTest('cryptography not installed')
        archive, entry = self.bundle()
        self.sign(archive, entry, tamper_sig=True)
        with self.assertRaisesRegex(compat.Rejected, 'does not verify'):
            self.extract(archive, entry)

    def test_minisign_key_id_mismatch_rejected(self):
        try:
            import cryptography  # noqa: F401
        except ImportError:
            self.skipTest('cryptography not installed')
        archive, entry = self.bundle()
        self.sign(archive, entry, wrong_key_id=True)
        with self.assertRaisesRegex(compat.Rejected, 'different key'):
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
        with self.assertRaisesRegex(compat.Rejected, r'needs native hook API 41; Volt serves \d+'):
            compat.check_hooks(data, compat.ROOT, 'c' * 40)
        data['meta']['hooks'] = 1
        with self.assertRaisesRegex(compat.Rejected, 'non-hook'):
            compat.check_hooks(data, compat.ROOT, 'c' * 40)

    def test_newer_hook_level_falls_through_on_raw_targets_probe(self):
        import json as _json
        hook = _json.loads((compat.ROOT / 'native-hooks.json').read_text())['hooks'][0]
        manifest = {'mod': {}, 'compatibility': {}, 'module': {},
                    'injection': [{'target': 'horizon_client::game::Game::attack'},
                                  {'hook': hook['name'], 'hook_version': hook['version']}]}
        data = {'meta': dict(self.meta, hooks=9999), 'manifest': manifest}
        with self.assertRaisesRegex(compat.Rejected, r'needs native hook API 9999'):
            compat.check_hooks(data, compat.ROOT, 'c' * 40)
        # The probe-only flag checks each injection instead of the level.
        self.assertTrue(compat.check_hooks(data, compat.ROOT, 'c' * 40, raw_targets=True))
        # But a named hook Volt does not serve still fails closed.
        manifest['injection'][1] = {'hook': 'definitely_not_served', 'hook_version': 1}
        with self.assertRaisesRegex(compat.Rejected, 'named hook unavailable'):
            compat.check_hooks(data, compat.ROOT, 'c' * 40, raw_targets=True)

    def test_revision_pinned_bundle_rejected_unless_raw_targets_probe(self):
        import json as _json
        hook = _json.loads((compat.ROOT / 'native-hooks.json').read_text())['hooks'][0]
        manifest = {'mod': {}, 'compatibility': {}, 'module': {},
                    'injection': [{'target': 'horizon_client::game::Game::attack'},
                                  {'hook': hook['name'], 'hook_version': hook['version']}]}
        data = {'meta': dict(self.meta), 'manifest': manifest}
        data['meta'].pop('hooks', None)
        with self.assertRaisesRegex(compat.Rejected, 'pins Horizon .* exactly without a hook-API'):
            compat.check_hooks(data, compat.ROOT, 'c' * 40)
        # The probe-only flag lets the mixin compiler judge raw targets instead.
        self.assertTrue(compat.check_hooks(data, compat.ROOT, 'c' * 40, raw_targets=True))
        # But a named hook Volt does not serve still fails closed.
        manifest['injection'][1] = {'hook': 'definitely_not_served', 'hook_version': 1}
        with self.assertRaisesRegex(compat.Rejected, 'missing from Volt'):
            compat.check_hooks(data, compat.ROOT, 'c' * 40, raw_targets=True)

    def test_source_archive_rejects_traversal_and_skips_symlinks(self):
        archive_path = self.root / 'source.tar'
        source = self.root / 'source'
        source.mkdir()
        with tarfile.open(archive_path, 'w') as tar:
            link = tarfile.TarInfo('link')
            link.type = tarfile.SYMTYPE
            link.linkname = '../outside'
            tar.addfile(link)
            info = tarfile.TarInfo('../escape')
            info.size = 3
            tar.addfile(info, io.BytesIO(b'bad'))
        with self.assertRaisesRegex(compat.Rejected, 'unsafe path'):
            compat.extract_source_archive(archive_path, source)
        self.assertFalse((self.root / 'escape').exists())
        self.assertFalse((source / 'link').exists())

    def test_post_build_rejected_even_exact_revision(self):
        data = {'meta': self.meta, 'manifest': {'post_build': [{'command': 'bad'}]}}
        with self.assertRaisesRegex(compat.Rejected, 'post-build'):
            compat.check_hooks(data, compat.ROOT, 'a' * 40)

if __name__ == '__main__':
    unittest.main()
