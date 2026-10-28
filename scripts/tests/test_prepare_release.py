import importlib.util
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / 'prepare_release.py'
SPEC = importlib.util.spec_from_file_location('prepare_release', SCRIPT)
release = importlib.util.module_from_spec(SPEC)


class ReleaseTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        SPEC.loader.exec_module(release)

    def test_iso_week_year_and_canonical_spelling(self):
        self.assertEqual(release.validate_version('2026.44.0'), '2026.44.0')
        self.assertEqual(release.validate_version('2026.53.2'), '2026.53.2')
        for value in ('2026.03.0', '2025.53.0', '2026.0.0', '2026.54.0',
                      '2026.44.01', '2026.44.0-alpha', '0.1.0'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                release.validate_version(value)

    def test_version_update_keeps_file_identifiers_separate(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'release').mkdir()
            (root / 'format-explorer').mkdir()
            (root / 'Cargo.toml').write_text(
                '[workspace.package]\nversion = "0.1.0"\n', encoding='utf-8')
            (root / 'format-explorer/package.json').write_text(
                '{"version":"0.1.0"}', encoding='utf-8')
            (root / 'release/manifest.json').write_text(
                '{"softwareVersion":"0.1.0","profileVersion":"0.1.0",'
                '"cborMappingRevision":1,"zstdTransportRevision":1}', encoding='utf-8')
            release.set_version(root, '2026.44.0')
            import json
            manifest = json.loads((root / 'release/manifest.json').read_text())
            self.assertEqual(manifest['softwareVersion'], '2026.44.0')
            self.assertEqual(manifest['profileVersion'], '0.1.0')
            self.assertEqual(manifest['cborMappingRevision'], 1)
            self.assertEqual(manifest['zstdTransportRevision'], 1)
            self.assertEqual(release.software_versions(root), ['2026.44.0'] * 3)

    def test_archive_is_deterministic_and_refuses_overwrite(self):
        with tempfile.TemporaryDirectory() as directory:
            first, second = (Path(directory) / name for name in ('a.tar.gz', 'b.tar.gz'))
            files = {'ifccad/LICENSE': b'license\n', 'ifccad/src/lib.rs': b'// source\n'}
            release.write_archive(first, files)
            release.write_archive(second, dict(reversed(list(files.items()))))
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with self.assertRaises(FileExistsError):
                release.write_archive(first, files)

    def test_archive_rejects_unsafe_members(self):
        with tempfile.TemporaryDirectory() as directory:
            for name in ('../escape', '/absolute', 'C:/absolute', 'a/../../escape', 'a\\b'):
                with self.subTest(name=name), self.assertRaises(ValueError):
                    release.write_archive(Path(directory) / 'bad.tar.gz', {name: b'x'})

    def test_native_package_rejects_cad_dependencies(self):
        import io
        import tarfile
        with tempfile.TemporaryDirectory() as directory:
            package = Path(directory) / 'native.crate'
            data = (b'[package]\nname="ifccad"\nversion="2026.44.0"\n'
                    b'[dependencies.acadrust]\ngit="https://example.invalid/codec"\n')
            with tarfile.open(package, 'w:gz') as archive:
                info = tarfile.TarInfo('ifccad-2026.44.0/Cargo.toml')
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))
            with self.assertRaisesRegex(ValueError, 'native core'):
                release.core_package_files(package, '2026.44.0')

    def test_native_package_rejects_absolute_path_after_prefix(self):
        import io
        import tarfile
        with tempfile.TemporaryDirectory() as directory:
            package = Path(directory) / 'bad.crate'
            with tarfile.open(package, 'w:gz') as archive:
                info = tarfile.TarInfo('ifccad-2026.44.0//escape')
                info.size = 1
                archive.addfile(info, io.BytesIO(b'x'))
            with self.assertRaisesRegex(ValueError, 'unsafe'):
                release.core_package_files(package, '2026.44.0')

    def test_native_package_rejects_shared_document_runtime(self):
        import io
        import tarfile
        with tempfile.TemporaryDirectory() as directory:
            for direct in (True, False):
                with self.subTest(direct=direct):
                    package = Path(directory) / 'document.crate'
                    manifest = '[package]\nname="ifccad"\nversion="2026.44.0"\n'
                    if direct:
                        manifest += '[dependencies]\nopencaddocument="0.1"\n'
                    files = {'Cargo.toml': manifest.encode(),
                             'Cargo.lock': b'version=4\n[[package]]\nname="opencaddocument"\nversion="0.1.0"\n'}
                    with tarfile.open(package, 'w:gz') as archive:
                        for name, data in files.items():
                            info = tarfile.TarInfo('ifccad-2026.44.0/' + name)
                            info.size = len(data)
                            archive.addfile(info, io.BytesIO(data))
                    with self.assertRaisesRegex(ValueError, 'native core'):
                        release.core_package_files(package, '2026.44.0')


if __name__ == '__main__':
    unittest.main()
