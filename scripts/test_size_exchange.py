import gzip
import tempfile
import unittest
from pathlib import Path
import size_exchange as exchange


class ExchangeTests(unittest.TestCase):
    def test_gzip_is_repeatable_and_byte_exact(self):
        payload = bytes(range(256)) * 100
        first = exchange.compress_verified(payload)
        self.assertEqual(first, exchange.compress_verified(payload))
        self.assertEqual(payload, gzip.decompress(first))
        self.assertEqual(first[4:8], b'\0' * 4)
        self.assertEqual(first[3] & 8, 0)

    def test_gzip_corruption_is_rejected(self):
        data = bytearray(exchange.compress_verified(b'content'))
        data[-8] ^= 1
        with self.assertRaises((gzip.BadGzipFile, EOFError)):
            exchange.decompress_verified(bytes(data), b'content')

    def test_compact_json_keeps_u64_tokens(self):
        data = b'{ "id": 18446744073709551615, "x": 0.125 }'
        self.assertEqual(exchange.compact_json(data), b'{"id":18446744073709551615,"x":0.125}')

    def test_existing_run_directory_is_rejected(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'run'
            exchange.fresh_directory(path)
            with self.assertRaises(FileExistsError):
                exchange.fresh_directory(path)

    def test_changed_source_between_passes_is_rejected(self):
        with self.assertRaises(ValueError):
            exchange.require_same({'input': 'a'}, {'input': 'b'})

    def test_nonmatching_artifact_hashes_fail_repeatability(self):
        with self.assertRaises(ValueError):
            exchange.require_same({'file': 'a'}, {'file': 'b'})


if __name__ == '__main__':
    unittest.main()
