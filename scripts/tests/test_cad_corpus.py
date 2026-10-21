import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("cad_corpus", Path(__file__).parents[1] / "cad_corpus.py")
corpus = importlib.util.module_from_spec(spec)
spec.loader.exec_module(corpus)


class CorpusTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bytes = b"AC1032test-data"
        self.item = {"id": "fixture", "sha256": corpus.sha256(self.bytes),
                     "bytes": len(self.bytes), "format": "dwg", "version": "AC1032",
                     "sourcePath": "samples/a.dwg",
                     "url": "https://raw.githubusercontent.com/owner/repo/" + "a" * 40 + "/a.dwg"}
        self.manifest = {"schemaVersion": 1, "files": [self.item]}
        self.path = self.root / "manifest.json"

    def load(self):
        self.path.write_text(json.dumps(self.manifest))
        return corpus.read_manifest(self.path)

    def test_rejects_traversal_duplicate_ids_and_unpinned_sources(self):
        self.load()
        for key, value in [("id", "../escaped"), ("url", "http://example.org/a.dwg"),
                           ("url", "https://raw.githubusercontent.com/a/b/main/a.dwg"),
                           ("bytes", -1)]:
            old = self.item[key]
            self.item[key] = value
            with self.assertRaises(ValueError):
                self.load()
            self.item[key] = old
        self.manifest["files"].append(dict(self.item))
        with self.assertRaises(ValueError):
            self.load()

    def test_mismatch_is_not_cached_and_existing_corruption_is_not_overwritten(self):
        cache = self.root / "cache"
        cache.mkdir()
        target = corpus.sample_path(cache, self.item)
        target.write_bytes(b"corrupt")
        with patch.object(corpus.urllib.request, "urlopen") as request:
            with self.assertRaises(ValueError):
                corpus.fetch(self.manifest, cache)
            request.assert_not_called()
        self.assertEqual(target.read_bytes(), b"corrupt")

    def test_version_is_checked_even_with_matching_hash(self):
        self.item["version"] = "AC1018"
        with self.assertRaises(ValueError):
            corpus.verify(self.bytes, self.item)

    def test_zero_exit_does_not_qualify_without_matching_report_identity(self):
        report = {"qualified": True, "status": "qualified-native-roundtrip",
                  "source": {"sha256": self.item["sha256"]}}
        self.assertTrue(corpus.result_row(self.item, 0, report)["qualified"])
        self.assertFalse(corpus.result_row(self.item, 1, report)["qualified"])
        report["source"]["sha256"] = "different"
        self.assertFalse(corpus.result_row(self.item, 0, report)["qualified"])
        self.assertFalse(corpus.result_row(self.item, 0, {})["qualified"])

    def test_timeout_keeps_incomplete_child_evidence_and_completes_batch(self):
        cache = self.root / "cache"
        cache.mkdir()
        corpus.sample_path(cache, self.item).write_bytes(self.bytes)
        binary = self.root / "binary"
        binary.write_bytes(b"test executable identity")
        output = self.root / "run"
        with patch.object(corpus.subprocess, "run", side_effect=subprocess.TimeoutExpired("test", 1)):
            self.assertEqual(corpus.run(self.manifest, cache, binary, output, 1), 1)
        summary = json.loads((output / "summary.json").read_text())
        self.assertTrue(summary["complete"])
        self.assertEqual(summary["qualifiedCount"], 0)
        self.assertEqual(summary["results"][0]["status"], "timeout")
        before = (output / "summary.json").read_bytes()
        with self.assertRaises(FileExistsError):
            corpus.run(self.manifest, cache, binary, output, 1)
        self.assertEqual(before, (output / "summary.json").read_bytes())

    def test_run_verifies_all_inputs_before_starting(self):
        cache = self.root / "cache"
        cache.mkdir()
        corpus.sample_path(cache, self.item).write_bytes(b"bad")
        with patch.object(corpus.subprocess, "run") as process:
            with self.assertRaises(ValueError):
                corpus.run(self.manifest, cache, self.root / "missing", self.root / "run", 1)
            process.assert_not_called()
        self.assertFalse((self.root / "run").exists())


if __name__ == "__main__":
    unittest.main()
