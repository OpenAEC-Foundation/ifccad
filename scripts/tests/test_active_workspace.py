"""Regression tests for the active/archived workspace boundary."""
import pathlib
import subprocess
import sys
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parents[1] / 'check_active_workspace.py'

class WorkspaceBoundary(unittest.TestCase):
    def check_workspace(self, manifest, files=None):
        with tempfile.TemporaryDirectory(dir=SCRIPT.parent) as directory:
            root = pathlib.Path(directory)
            (root / 'src').mkdir()
            (root / 'src/lib.rs').write_text('', encoding='utf-8')
            (root / 'Cargo.toml').write_text(manifest + '\n[workspace]\n', encoding='utf-8')
            for name, text in (files or {}).items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text, encoding='utf-8')
            subprocess.run(['cargo', 'generate-lockfile', '--offline'], cwd=root, check=True, capture_output=True)
            return subprocess.run([sys.executable, str(SCRIPT), '--root', str(root)], text=True, capture_output=True)

    def test_active_ifccad_is_accepted(self):
        result = self.check_workspace('[package]\nname="ifccad"\nversion="0.1.0"\nedition="2021"\n')
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_legacy_directory_is_not_autodiscovered(self):
        result = self.check_workspace('[package]\nname="ifccad"\nversion="0.1.0"\n', {'legacy/ocdraw/Cargo.toml':'[package]\nname="ocdraw"\nversion="0.1.0"\n', 'legacy/ocdraw/src/lib.rs':''})
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_old_target_name_is_rejected(self):
        result = self.check_workspace('[package]\nname="ifccad"\nversion="0.1.0"\n[lib]\nname="ocdraw"\n')
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn('ocdraw', result.stderr)

    def test_old_package_is_rejected(self):
        result = self.check_workspace('[package]\nname="ocdraw"\nversion="0.1.0"\n')
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn('ocdraw', result.stderr)

    def test_transitive_legacy_path_is_rejected(self):
        result = self.check_workspace('[package]\nname="ifccad"\nversion="0.1.0"\n[dependencies]\nhelper={path="legacy/helper"}\n', {'legacy/helper/Cargo.toml':'[package]\nname="helper"\nversion="0.1.0"\n', 'legacy/helper/src/lib.rs':''})
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn('legacy', result.stderr)

    def test_compile_time_legacy_fixture_is_rejected(self):
        result = self.check_workspace('[package]\nname="ifccad"\nversion="0.1.0"\n', {'tests/leak.rs':'const X: &str = include_str!("../legacy/ocdraw/data.json");'})
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn('tests/leak.rs', result.stderr)

if __name__ == '__main__':
    unittest.main()
