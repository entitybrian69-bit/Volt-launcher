import argparse
import io
import tarfile
from unittest.mock import patch
from types import SimpleNamespace
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

import build_set
import compat


class BuildSetTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.registry = self.root / 'registry'
        (self.registry / 'mods').mkdir(parents=True)
        (self.registry / 'revoked.toml').write_text('')
        self.work = self.root / 'work'
        self.work.mkdir()
        self.revision = 'a' * 40
        self.compiler = 'b' * 40

    def bundle(self, mod_id, *, compiler=None, revoked=False):
        metadata = {'schema': 1, 'id': mod_id, 'version': '2.0', 'game': '26.3',
                    'horizon': self.revision, 'mixins': compiler or self.compiler, 'hooks': 1}
        path = self.root / f'{mod_id}.hnmod'
        with zipfile.ZipFile(path, 'w') as z:
            z.writestr('horizon-native.json', json.dumps(metadata))
            z.writestr(f'mixins/{mod_id}/horizon-mixin.toml',
                       f'[mod]\nid = "{mod_id}"\nversion = "2.0"\n'
                       f'[compatibility]\nhorizon_revision = "{self.revision}"\n')
            z.writestr(f'mixins/{mod_id}/runtime.rs', '// test')
        sha = hashlib.sha256(path.read_bytes()).hexdigest()
        (self.registry / 'mods' / f'{mod_id}.toml').write_text(
            f'id = "{mod_id}"\nkind = "native-mod"\n[[versions]]\nversion = "2.0"\n'
            f'game = "26.3"\nfile = "{mod_id}.hnmod"\nsha256 = "{sha}"\n')
        if revoked:
            (self.registry / 'revoked.toml').write_text(f'[[revoked]]\nid = "{mod_id}"\n')
        return path

    def prepare(self, selections):
        return build_set.prepare_set(self.registry, selections, self.compiler,
                                     compat.ROOT, self.revision, self.work)

    def test_multiple_verified_bundles_are_staged_together(self):
        one, two = self.bundle('one'), self.bundle('two')
        result = self.prepare([('two', two), ('one', one)])
        self.assertEqual([m['id'] for m in result], ['one', 'two'])
        self.assertTrue((self.work / 'mixins/0/one/runtime.rs').is_file())
        self.assertTrue((self.work / 'mixins/0/two/runtime.rs').is_file())
        self.assertFalse(any(m['hook_revision_rebound'] for m in result))

    def test_duplicate_and_builtin_rejected(self):
        with self.assertRaisesRegex(compat.Rejected, 'duplicate'):
            self.prepare([('one', Path('unused')), ('one', Path('unused'))])
        with self.assertRaisesRegex(compat.Rejected, 'already bundled'):
            self.prepare([('zoom', Path('unused'))])

    def test_revoked_and_wrong_compiler_rejected(self):
        bundle = self.bundle('one', revoked=True)
        with self.assertRaisesRegex(compat.Rejected, 'revoked'):
            self.prepare([('one', bundle)])
        (self.registry / 'revoked.toml').write_text('')
        bundle = self.bundle('one', compiler='c' * 40)
        # Each build request has its own empty temporary directory.
        import shutil
        shutil.rmtree(self.work)
        self.work.mkdir()
        with self.assertRaisesRegex(compat.Rejected, 'compiler revision'):
            self.prepare([('one', bundle)])

    def test_full_worker_flow_and_no_stale_success(self):
        one, two = self.bundle('one'), self.bundle('two')
        output = self.root / 'output'
        output.mkdir()
        args = SimpleNamespace(trust_native_code=True, registry=self.registry,
                               mod=[('one', one), ('two', two)], mixins_revision=self.compiler,
                               mixins=self.root / 'mixin', abi='arm64-v8a', output=output)
        calls = []
        def fake_check_output(command, **kwargs):
            return self.revision + '\n' if 'rev-parse' in command else ''
        def fake_run(command, **kwargs):
            calls.append(command)
            if command[:2] == ['git', 'archive']:
                with tarfile.open(fileobj=kwargs['stdout'], mode='w') as archive:
                    data = b'placeholder'
                    info = tarfile.TarInfo('README')
                    info.size = len(data)
                    archive.addfile(info, io.BytesIO(data))
                return SimpleNamespace(returncode=0)
            if 'apply' in command:
                mixins = Path(command[command.index('--mods') + 1])
                self.assertTrue((mixins / 'one').is_dir() and (mixins / 'two').is_dir())
                self.assertEqual(mixins.name, '0')
                Path(command[command.index('--out') + 1]).mkdir()
            else:
                lib = Path(kwargs['env']['CARGO_TARGET_DIR']) / build_set.TARGETS[args.abi] / 'release/libhorizon_android.so'
                lib.parent.mkdir(parents=True)
                lib.write_bytes(b'example library')
            return SimpleNamespace(returncode=0, stdout='', stderr='')
        with patch.object(build_set.subprocess, 'check_output', side_effect=fake_check_output), \
             patch.object(build_set.subprocess, 'run', side_effect=fake_run):
            report = build_set.run(args)
        self.assertEqual(report['status'], 'library_built')
        self.assertFalse(report['runtime_tested'])
        self.assertFalse(report['signed'])
        self.assertEqual(report['library_sha256'], hashlib.sha256(b'example library').hexdigest())
        self.assertEqual([x['id'] for x in report['mods']], ['one', 'two'])
        self.assertIn('--rust-only', calls[-1])
        self.assertIn('--public', calls[-1])

    def test_bundles_with_different_pinned_compilers_weave_in_passes(self):
        one = self.bundle('one')
        two = self.bundle('two', compiler='d' * 40)
        output = self.root / 'output-two-pass'
        output.mkdir()
        args = SimpleNamespace(trust_native_code=True, registry=self.registry,
                               mod=[('one', one), ('two', two)],
                               mixins_revision=[self.compiler, 'd' * 40],
                               mixins=[self.root / 'mixin-a', self.root / 'mixin-b'],
                               abi='arm64-v8a', output=output)
        applies = []
        def fake_check_output(command, **kwargs):
            return self.revision + '\n' if 'rev-parse' in command else ''
        def fake_run(command, **kwargs):
            if command[:2] == ['git', 'archive']:
                with tarfile.open(fileobj=kwargs['stdout'], mode='w') as archive:
                    data = b'placeholder'
                    info = tarfile.TarInfo('README')
                    info.size = len(data)
                    archive.addfile(info, io.BytesIO(data))
                return SimpleNamespace(returncode=0)
            if 'apply' in command:
                mixins = Path(command[command.index('--mods') + 1])
                applies.append((Path(command[0]).name, mixins.name,
                                sorted(p.name for p in mixins.iterdir()),
                                Path(command[command.index('--workspace') + 1]).name))
                Path(command[command.index('--out') + 1]).mkdir()
            else:
                lib = Path(kwargs['env']['CARGO_TARGET_DIR']) / build_set.TARGETS[args.abi] / 'release/libhorizon_android.so'
                lib.parent.mkdir(parents=True, exist_ok=True)
                lib.write_bytes(b'two pass library')
            return SimpleNamespace(returncode=0, stdout='', stderr='')
        with patch.object(build_set.subprocess, 'check_output', side_effect=fake_check_output), \
             patch.object(build_set.subprocess, 'run', side_effect=fake_run):
            report = build_set.run(args)
        self.assertEqual(report['status'], 'library_built')
        # Pass 0 weaves 'one' with its own compiler on the pristine source;
        # pass 1 weaves 'two' with its compiler on pass 0's output workspace.
        self.assertEqual(applies, [('mixin-a', '0', ['one'], 'source'),
                                   ('mixin-b', '1', ['two'], 'stage-0')])
        self.assertEqual([m['mixins_revision'] for m in report['mods']],
                         [self.compiler, 'd' * 40])

    def test_checksum_mismatch_rejected(self):
        bundle = self.bundle('one')
        with bundle.open('ab') as f:
            f.write(b'tampered')
        with self.assertRaisesRegex(compat.Rejected, 'SHA-256'):
            self.prepare([('one', bundle)])

    def test_selection_parser(self):
        self.assertEqual(build_set.selection('test_mod=/tmp/test.hnmod'), ('test_mod', Path('/tmp/test.hnmod')))
        for invalid in ('../test=x', 'test', 'test='):
            with self.assertRaises(argparse.ArgumentTypeError):
                build_set.selection(invalid)


if __name__ == '__main__':
    unittest.main()
