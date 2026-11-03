import importlib.util
from pathlib import Path
import unittest

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
        self.assertIn('[patch."https://github.com/HakanSeven12/acadrust.git"]', patched)
        self.assertIn(pins['codecCommit'], patched)
        self.assertEqual(prepare.prepare_codec_patch(patched, pins), patched)

    def test_codec_repository_is_checked_alongside_revision(self):
        pins = dict(documentCommit='doc', nativeCoreCommit='core', codecCommit='cad', codecRepository='https://github.com/BjornFidder/opencadcodec.git')
        packages = [dict(name=name, source='git+https://wrong.test/repo?rev='+rev+'#'+rev) for name,rev in [('opencaddocument','doc'),('ifccad','core'),('acadrust','cad')]]
        with self.assertRaisesRegex(SystemExit, 'acadrust repository'):
            prepare.validate_packages(packages,pins)
