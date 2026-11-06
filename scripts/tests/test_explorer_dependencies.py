import importlib.util
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('check_explorer_dependencies',Path(__file__).parents[1]/'check_explorer_dependencies.py')
guard=importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)

class ResourceDevelopmentPair(unittest.TestCase):
    def test_attribute_pair_uses_the_current_canonical_codec_repository(self):
        pins=dict(localDevelopment='block-attributes',explorerCodecRepository='https://github.com/hakanaktt/acadrust.git',explorerCodecCommit='a'*40,developmentOverrides={'acadrust':dict(baseCommit='a'*40,path='../block-attributes-codec')})
        self.assertEqual(guard.development_codec(pins),(Path(guard.__file__).resolve().parents[1]/'../block-attributes-codec').resolve().as_posix())
        for bad in [dict(pins,explorerCodecCommit='b'*40),dict(pins,explorerCodecRepository='https://wrong.test/repo')]:
            self.assertIsNone(guard.development_codec(bad))

    def test_resource_pair_is_explicitly_allowed(self):
        pins=dict(localDevelopment='font-symbol-sources',explorerCodecRepository='https://github.com/BjornFidder/opencadcodec.git',explorerCodecCommit='4018037daf8fd3e40aa1aa6455b80232849a2756')
        self.assertIsNotNone(guard.development_codec(pins))
        for field,value in [('localDevelopment','anything'),('explorerCodecRepository','https://wrong.test/repo'),('explorerCodecCommit','0'*40)]:
            with self.subTest(field=field):
                bad={**pins,field:value}
                self.assertIsNone(guard.development_codec(bad))
