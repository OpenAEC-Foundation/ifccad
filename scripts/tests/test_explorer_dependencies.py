import importlib.util
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('check_explorer_dependencies',Path(__file__).parents[1]/'check_explorer_dependencies.py')
guard=importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)

class ResourceDevelopmentPair(unittest.TestCase):
    def test_resource_pair_is_explicitly_allowed(self):
        pins=dict(localDevelopment='font-symbol-sources',explorerCodecRepository='https://github.com/BjornFidder/opencadcodec.git',explorerCodecCommit='4018037daf8fd3e40aa1aa6455b80232849a2756')
        self.assertIsNotNone(guard.development_codec(pins))
        for field,value in [('localDevelopment','anything'),('explorerCodecRepository','https://wrong.test/repo'),('explorerCodecCommit','0'*40)]:
            with self.subTest(field=field):
                bad={**pins,field:value}
                self.assertIsNone(guard.development_codec(bad))
