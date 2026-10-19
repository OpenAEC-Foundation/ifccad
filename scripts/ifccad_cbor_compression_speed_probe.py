"""Manual gzip/Zstd timing of the one previously qualified foundation artifact."""
import argparse
from compression import zstd
from datetime import datetime, timezone
import gc
import gzip
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import sys
import time
import zlib


def sha(data):
    return hashlib.sha256(data).hexdigest()


def active_builds():
    if os.name != 'nt':
        return ''
    command = 'Get-Process cargo,rustc,clippy-driver,link -ErrorAction SilentlyContinue | Select-Object ProcessName,Id | ConvertTo-Json -Compress'
    return subprocess.run(['pwsh', '-NoProfile', '-Command', command],
                          capture_output=True, text=True, check=False).stdout.strip()


def summary(samples, raw_bytes):
    values = sorted(samples)
    median = statistics.median(values)
    return {'samplesNs': samples, 'medianMs': median / 1e6,
            'minMs': values[0] / 1e6, 'maxMs': values[-1] / 1e6,
            'p10Ms': values[math.ceil(len(values) * .1) - 1] / 1e6,
            'p90Ms': values[math.ceil(len(values) * .9) - 1] / 1e6,
            'rawMBPerSecond': raw_bytes * 1000 / median}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    prior_bytes = (args.baseline / 'results.json').read_bytes()
    prior = json.loads(prior_bytes)
    raw = (args.baseline / 'input.ifccad.cbor').read_bytes()
    if sha(raw) != '391fa55223f87f9821d8782b9ace5553725fe6d17a18aedd6cb4df2b915b7c68':
        raise ValueError('expected the previously qualified foundation CBOR artifact')
    if prior['input']['sha256'] != sha(raw):
        raise ValueError('prior results refer to a different input')
    if active_builds():
        raise RuntimeError('another build is active; no timing started')
    args.out.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(__file__, args.out / 'probe.py')
    (args.out / 'input.ifccad.cbor').write_bytes(raw)
    operations, rows = [], []
    for variant in prior['variants']:
        codec, level = variant['codec'], variant['level']
        encoded = (args.baseline / variant['file']).read_bytes()
        if sha(encoded) != variant['sha256']:
            raise ValueError('compressed input hash mismatch')
        if codec == 'gzip' and level == 9:
            compress = lambda data: gzip.compress(data, compresslevel=9, mtime=0)
            decompress = gzip.decompress
        elif codec == 'zstd' and level in [3, 9, 19]:
            options = {zstd.CompressionParameter.compression_level: level,
                       zstd.CompressionParameter.checksum_flag: 1,
                       zstd.CompressionParameter.content_size_flag: 1,
                       zstd.CompressionParameter.nb_workers: 0}
            compress = lambda data, opts=options: zstd.compress(data, options=opts)
            decompress = zstd.decompress
        else:
            raise ValueError('unexpected variant outside the requested scope')
        # Qualify exact compressed bytes and decompression before timing.
        if compress(raw) != encoded or decompress(encoded) != raw:
            raise ValueError('runtime does not reproduce prior qualified bytes')
        (args.out / variant['file']).write_bytes(encoded)
        row = dict(variant, compressionSamples=[], decompressionSamples=[])
        rows.append(row)
        operations.extend([(compress, raw, encoded, row['compressionSamples']),
                           (decompress, encoded, raw, row['decompressionSamples'])])
    if [(r['codec'], r['level']) for r in rows] != [('gzip', 9), ('zstd', 3), ('zstd', 9), ('zstd', 19)]:
        raise ValueError('expected precisely gzip9 and Zstd3/9/19')
    warmups, samples = 3, 31
    for round_index in range(warmups + samples):
        for step in range(len(operations)):
            operation, data, expected, collected = operations[(step + round_index) % len(operations)]
            started = time.perf_counter_ns()
            output = operation(data)
            elapsed = time.perf_counter_ns() - started
            # Equality checks and returned-buffer disposal are outside timing.
            if output != expected:
                raise ValueError('a timed invocation changed the output bytes')
            del output
            if round_index >= warmups:
                collected.append(elapsed)
    after_builds = active_builds()
    for row in rows:
        row['compression'] = summary(row.pop('compressionSamples'), len(raw))
        row['decompression'] = summary(row.pop('decompressionSamples'), len(raw))
    result = {'version': 1, 'requestedOn': '2026-10-19',
              'executedUtc': datetime.now(timezone.utc).isoformat(),
              'scope': 'foundation CBOR only; one-shot external compression/decompression in memory',
              'input': prior['input'], 'priorSizeResultsSha256': sha(prior_bytes),
              'priorSizeOptions': prior['options'],
              'runtime': {'python': sys.version, 'zstd': zstd.zstd_version,
                          'zlib': zlib.ZLIB_RUNTIME_VERSION, 'platform': platform.platform(),
                          'cpu': platform.processor(), 'logicalCpus': os.cpu_count(),
                          'gcEnabled': gc.isenabled(), 'cpuAffinity': 'not pinned',
                          'clock': vars(time.get_clock_info('perf_counter'))},
              'protocol': {'warmupsPerOperation': warmups, 'samplesPerOperation': samples,
                           'order': 'rotate eight codec/direction operations by round',
                           'includes': ['one-shot Python API, context creation, output allocation', 'codec checksum work'],
                           'excludes': ['filesystem, output comparison/disposal, CBOR parsing and validation, CAD conversion'],
                           'throughput': 'decimal MB/s based on uncompressed bytes in both directions'},
              'probeSha256': sha(Path(__file__).read_bytes()),
              'activeBuildsBefore': '', 'activeBuildsAfter': after_builds,
              'variants': rows, 'accepted': not after_builds,
              'limitations': ['one warm fixed artifact; no cold-cache or peak-memory claim',
                              'normal scheduling and power management; no general performance claim',
                              'compression remains experimental, not native IFCCAD storage']}
    # Retain exact runtime library identities from the prerequisite size run,
    # and verify that those binaries have not changed before accepting timings.
    hashes = {}
    for name in ['_zstd', 'zlib']:
        module = __import__(name)
        location = getattr(module, '__file__', None)
        if location:
            path = Path(location)
            hashes[path.name] = sha(path.read_bytes())
        else:
            hashes[name] = 'built into Python runtime'
    for path in [Path(sys.executable), Path(sys.base_prefix) / 'python314.dll']:
        if path.is_file():
            hashes[path.name] = sha(path.read_bytes())
    result['runtime']['extensionHashes'] = hashes
    if hashes != prior['runtime']['extensionHashes']:
        result['accepted'] = False
    (args.out / 'results.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8', newline='\n')
    for row in rows:
        print(f"{row['codec']} {row['level']}: compress {row['compression']['medianMs']:.4f} ms; decompress {row['decompression']['medianMs']:.4f} ms")
    if not result['accepted']:
        raise RuntimeError('timings retained but not accepted: competing build or changed runtime')


if __name__ == '__main__':
    main()
