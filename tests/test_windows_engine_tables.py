import json
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import windows_engine_tables as tables


class PublicEngineTables(unittest.TestCase):
    def test_only_generated_tables_survive_and_version_is_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'source'
            for name in tables.REQUIRED | {'assets/minecraft/private.png', 'client.jar'}:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(json.dumps({'minecraft': tables.VERSION}))
            out = root.parent / 'engine-data.zip'
            tables.build(root, out)
            with zipfile.ZipFile(out) as archive:
                self.assertEqual(tables.REQUIRED, set(archive.namelist()))
            previous = out.read_bytes()
            (root / 'horizon-resources.json').write_text('{"minecraft":"wrong"}')
            with self.assertRaisesRegex(ValueError, 'version'):
                tables.build(root, out)
            self.assertEqual(previous, out.read_bytes())
            with zipfile.ZipFile(out, 'a') as archive:
                archive.writestr('assets/private.png', b'not distributable')
            with self.assertRaisesRegex(ValueError, 'member'):
                tables.verify(out)
