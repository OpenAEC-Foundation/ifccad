"""Manual compression-size probe for one previously qualified CBOR artifact."""
import argparse
from compression import zstd
from datetime import datetime, timezone
import gzip
import hashlib
import json
from pathlib import Path
import platform
import shutil
import sys
import zlib


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', required=True, type=Path)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--out', required=True, type=Path, help='Fresh evidence directory')
    args = parser.parse_args()
    data = args.input.read_bytes()
    if sha(data) != args.sha256:
        raise ValueError('input does not match the previously qualified artifact')
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / 'input.ifccad.cbor').write_bytes(data)
    shutil.copyfile(__file__, args.out / 'probe.py')
    variants = [('gzip', 9, gzip.compress(data, compresslevel=9, mtime=0))]
    for level in [3, 9, 19]:
        options = {zstd.CompressionParameter.compression_level: level,
                   zstd.CompressionParameter.checksum_flag: 1,
                   zstd.CompressionParameter.content_size_flag: 1,
                   zstd.CompressionParameter.nb_workers: 0}
        variants.append(('zstd', level, zstd.compress(data, options=options)))
    rows = []
    for codec, level, encoded in variants:
        decoded = gzip.decompress(encoded) if codec == 'gzip' else zstd.decompress(encoded)
        if decoded != data:
            raise ValueError('decompressed bytes differ from the qualified CBOR input')
        name = f'foundation.level-{level}.ifccad.cbor.' + ('gz' if codec == 'gzip' else 'zst')
        (args.out / name).write_bytes(encoded)
        rows.append({'codec': codec, 'level': level, 'file': name, 'bytes': len(encoded),
                     'sha256': sha(encoded), 'decompression': 'byte-exact',
                     'decodedSha256': sha(decoded), 'ratioToRaw': len(encoded) / len(data)})
    extensions = {}
    for name in ['_zstd', 'zlib']:
        module = __import__(name)
        location = getattr(module, '__file__', None)
        if location:
            path = Path(location)
            extensions[path.name] = sha(path.read_bytes())
        else:
            extensions[name] = 'built into Python runtime'
    for path in [Path(sys.executable), Path(sys.base_prefix) / 'python314.dll']:
        if path.is_file():
            extensions[path.name] = sha(path.read_bytes())
    result = {'version': 1, 'requestedOn': '2026-10-19',
              'executedUtc': datetime.now(timezone.utc).isoformat(),
              'scope': 'foundation CBOR only; external compression size probe; no timing',
              'input': {'bytes': len(data), 'sha256': sha(data),
                        'baselineRun': 'cbor-small-20261016-v1',
                        'baselineCase': 'practice-foundation',
                        'qualification': 'previous production strict-readback qualification; all decompressions reproduce exactly those bytes'},
              'runtime': {'python': sys.version, 'platform': platform.platform(),
                          'zstd': zstd.zstd_version, 'zlibBuild': zlib.ZLIB_VERSION,
                          'zlibRuntime': zlib.ZLIB_RUNTIME_VERSION, 'extensionHashes': extensions},
              'options': {'zstd': {'levels': [3, 9, 19], 'checksum': True,
                                   'contentSize': True, 'workers': 0, 'dictionary': None},
                          'gzip': {'level': 9, 'mtime': 0, 'filename': ''}},
              'probeSha256': sha(Path(__file__).read_bytes()), 'variants': rows,
              'limitations': ['No native compressed IFCCAD implementation',
                              'No new CAD conversion, corpus rerun or speed comparison',
                              'One fixed qualified artifact; no general compression claim']}
    text = json.dumps(result, indent=2) + '\n'
    (args.out / 'results.json').write_text(text, encoding='utf-8', newline='\n')
    print(text)


if __name__ == '__main__':
    main()
