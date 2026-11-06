import importlib.util
from pathlib import Path
import unittest
import tempfile
import subprocess

spec = importlib.util.spec_from_file_location('prepare_explorer_ocs', Path(__file__).parents[1] / 'prepare_explorer_ocs.py')
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)


class HostDependencies(unittest.TestCase):
    def setUp(self):
        self.pins = dict(documentCommit='doc', nativeCoreCommit='core', codecCommit='cad', kernelCommit='kernel')
        self.packages = [dict(name=name, source='git+https://example.test/repo#'+rev) for name, rev in [
            ('opencaddocument', 'doc'), ('ifccad', 'core'), ('acadrust', 'cad'),
            ('opencadkernel', 'kernel'), ('opencadkernel-constraints', 'kernel')]]

    def test_shared_models_and_both_kernel_packages_match(self):
        prepare.validate_packages(self.packages, self.pins)

    def test_second_codec_model_is_rejected_before_build(self):
        with self.assertRaisesRegex(SystemExit, 'acadrust'):
            prepare.validate_packages(self.packages + [dict(name='acadrust', source='git+https://example.test/repo#old')], self.pins)

    def test_mismatched_kernel_is_rejected(self):
        self.packages[-1]['source'] = 'git+https://example.test/repo#old'
        with self.assertRaisesRegex(SystemExit, 'opencadkernel-constraints'):
            prepare.validate_packages(self.packages, self.pins)


class ResourceCodecPatch(unittest.TestCase):
    def test_resource_codec_patch_is_exact_and_idempotent(self):
        pins = dict(codecRepository='https://github.com/BjornFidder/opencadcodec.git', codecCommit='4018037daf8fd3e40aa1aa6455b80232849a2756')
        patched = prepare.prepare_codec_patch('[package]\nname="host"\n', pins)
        self.assertIn('[patch."https://github.com/hakanaktt/acadrust.git"]', patched)
        self.assertIn('[patch."https://github.com/HakanSeven12/acadrust.git"]', patched)
        self.assertIn(pins['codecCommit'], patched)
        self.assertEqual(prepare.prepare_codec_patch(patched, pins), patched)

    def test_codec_repository_is_checked_alongside_revision(self):
        pins = dict(documentCommit='doc', nativeCoreCommit='core', codecCommit='cad', codecRepository='https://github.com/BjornFidder/opencadcodec.git')
        packages = [dict(name=name, source='git+https://wrong.test/repo?rev='+rev+'#'+rev) for name,rev in [('opencaddocument','doc'),('ifccad','core'),('acadrust','cad')]]
        with self.assertRaisesRegex(SystemExit, 'acadrust repository'):
            prepare.validate_packages(packages,pins)


class FrozenHostPatch(unittest.TestCase):
    def test_clean_and_already_applied_patch_succeed_but_drift_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            target = root / 'host.rs'
            target.write_text('before\n')
            patch = root / 'host.patch'
            patch.write_text('diff --git a/host.rs b/host.rs\n--- a/host.rs\n+++ b/host.rs\n@@ -1 +1 @@\n-before\n+after\n')
            prepare.apply_host_patch(root, patch)
            prepare.apply_host_patch(root, patch)
            self.assertEqual(target.read_text(), 'after\n')
            target.write_text('unrelated change\n')
            with self.assertRaisesRegex(SystemExit, 'Host source differs'):
                prepare.apply_host_patch(root, patch)


class ExplicitDevelopmentPair(unittest.TestCase):
    def test_path_package_must_match_the_selected_checkout(self):
        path = prepare.ROOT / 'test-codec-checkout'
        pins = dict(documentCommit='doc', nativeCoreCommit='core', codecCommit='cad', developmentOverrides={'acadrust':dict(path=str(path),baseCommit='cad')})
        packages = [dict(name=n,source='git+https://example.test/repo#'+r) for n,r in [('opencaddocument','doc'),('ifccad','core')]]
        local = dict(name='acadrust',source=None,manifest_path=str(path/'Cargo.toml'))
        prepare.validate_packages(packages+[local],pins)
        for bad in [dict(local,source='git+https://example.test/repo#cad'),dict(local,manifest_path=str(path/'other'/'Cargo.toml'))]:
            with self.assertRaisesRegex(SystemExit,'acadrust development'):
                prepare.validate_packages(packages+[bad],pins)
    def test_official_same_source_is_not_patched_to_itself(self):
        source='[package]\nname="host"\n'
        self.assertEqual(prepare.prepare_codec_patch(source,dict(codecRepository='https://github.com/hakanaktt/acadrust.git',codecCommit='cad')),source)
