"""Controlled four-format transport probe using production Rust readers."""
import argparse
import gzip
import hashlib
import io
import json
import os
import platform
import re
import subprocess
import sys
import tomllib
import zlib
from pathlib import Path

CODEC_REVISION = 'd96e3fa2fe5acbeac966f1db4c01142618bf9c79'
SCHEMA = 'schemas/ifccad/ifccad-profile-0.1.0.ifcx'
CORPUS = 'benchmarks/size-exchange/corpus-v1.json'
FILES = ['drawing.ocdraw.json', 'drawing.ifcx', 'drawing.dxf', 'drawing.dwg',
         'drawing.compact.ocdraw.json', 'drawing.compact.ifcx']


def digest(data):
    return hashlib.sha256(data).hexdigest()


def decompress_verified(payload, expected):
    actual = gzip.decompress(payload)
    if actual != expected:
        raise ValueError('gzip decompression changed bytes')
    return actual


def compress_verified(payload):
    output = io.BytesIO()
    with gzip.GzipFile(fileobj=output, mode='wb', filename='', mtime=0, compresslevel=9) as stream:
        stream.write(payload)
    data = output.getvalue()
    decompress_verified(data, payload)
    return data


def compact_json(payload):
    # Python integers remain arbitrary precision; IDs never pass through floats.
    return json.dumps(json.loads(payload), ensure_ascii=False, allow_nan=False,
                      separators=(',', ':')).encode('utf-8')


def fresh_directory(path):
    path.mkdir(parents=True, exist_ok=False)


def require_same(first, second):
    if first != second:
        raise ValueError('source, diagnostics, snapshot or artifact changed between passes')


def command(args, repo):
    result = subprocess.run(args, cwd=repo, capture_output=True, text=True, encoding='utf-8')
    if result.returncode:
        raise RuntimeError(result.stderr)
    return result.stdout


def provenance(repo, practice):
    paths = command(['git', 'ls-files', '--cached', '--others', '--exclude-standard'], repo).splitlines()
    source = {}
    for name in sorted(set(paths + ['Cargo.lock'])):
        if name.startswith(('target/', '.worktrees/')) or name in (
                'benchmarks/size-exchange/results-v1.json',
                'docs/benchmarks/common-subset-size-exchange-v1.md',
                'benchmarks/size-exchange/test-dxf-results-v1.json',
                'docs/benchmarks/common-subset-test-dxf-v1.md'):
            continue
        path = repo / name
        source[name] = digest(path.read_bytes()) if path.is_file() else 'deleted'
    lock = (repo / 'Cargo.lock').read_text(encoding='utf-8')
    codec = [p for p in tomllib.loads(lock)['package'] if p['name'] == 'opencadcodec']
    expected_source = 'git+https://github.com/HakanSeven12/opencadcodec.git?rev=' + CODEC_REVISION + '#' + CODEC_REVISION
    if len(codec) != 1 or codec[0].get('source') != expected_source:
        raise ValueError('unexpected codec lock revision')
    for name in [name for name in source if name.endswith('Cargo.toml')]:
        if re.search(r'^\s*\[(patch|replace)', (repo / name).read_text(), re.MULTILINE):
            raise ValueError('codec overrides are outside accepted experiment configuration')
    return {
        'gitHead': command(['git', 'rev-parse', 'HEAD'], repo).strip(),
        'sourceManifest': source, 'codecRevision': CODEC_REVISION, 'codecSource': expected_source,
        'practice': {'filename': practice.name, 'bytes': practice.stat().st_size,
                     'sha256': digest(practice.read_bytes())} if practice else None,
        'python': platform.python_version(), 'zlib': zlib.ZLIB_RUNTIME_VERSION,
        'rustc': command(['rustc', '--version'], repo).strip(),
        'cargo': command(['cargo', '--version'], repo).strip(),
        'dirty': bool(command(['git', 'status', '--porcelain', '--untracked-files=all'], repo).strip()),
    }


def process_case(binary, repo, directory):
    for original, compact in [('drawing.ocdraw.json', 'drawing.compact.ocdraw.json'),
                              ('drawing.ifcx', 'drawing.compact.ifcx')]:
        (directory / compact).write_bytes(compact_json((directory / original).read_bytes()))
    restored = directory / 'decompressed'
    fresh_directory(restored)
    (restored / 'expected.json').write_bytes((directory / 'expected.json').read_bytes())
    artifacts = []
    for name in FILES:
        payload = (directory / name).read_bytes()
        compressed = compress_verified(payload)
        (directory / (name + '.gz')).write_bytes(compressed)
        (restored / name).write_bytes(decompress_verified(compressed, payload))
        artifacts.append({'file': name, 'rawBytes': len(payload), 'gzipBytes': len(compressed),
                          'sha256': digest(payload), 'gzipSha256': digest(compressed)})
    verification = json.loads(command([str(binary), 'verify', '--directory', str(restored)], repo))
    return {'artifacts': artifacts, 'decompressedVerification': verification,
            'snapshotSha256': digest((directory / 'expected.json').read_bytes()),
            'preparation': json.loads((directory / 'preparation.json').read_bytes())}


def markdown(result):
    lines = ['# Common-subset size/exchange experiment v1', '',
             'Sizes in bytes. Every accepted row has exact production readback after gzip decompression.', '',
             '| Case | OCDraw gzip | IFCX gzip | DXF gzip | DWG raw | DWG gzip |',
             '|---|---:|---:|---:|---:|---:|']
    for case in result['cases']:
        if case['status'] != 'passed':
            continue
        rows = {v['file']: v for v in case['measurement']['artifacts']}
        sizes = [rows['drawing.compact.ocdraw.json']['gzipBytes'],
                 rows['drawing.compact.ifcx']['gzipBytes'], rows['drawing.dxf']['gzipBytes'],
                 rows['drawing.dwg']['rawBytes'], rows['drawing.dwg']['gzipBytes']]
        lines.append('| ' + case['id'] + ' | ' + ' | '.join(map(str, sizes)) + ' |')
    lines += ['', 'Failed cases are excluded:', '']
    lines += ['- ' + c['id'] + ': ' + c['error'] for c in result['cases'] if c['status'] != 'passed']
    lines += ['', 'Known-profile transfer: shared schema/registry installed at the receiver.',
              f"IFCX schema alone: {result['schema']['rawBytes']} raw / {result['schema']['gzipBytes']} gzip bytes.",
              'Compression: gzip level 9, mtime 0, empty filename. Compact JSON preserves integer tokens.',
              'Detailed raw/pretty/compact sizes, losses, numeric preparation changes and provenance are in results.json.', '']
    return '\n'.join(lines)


def run_experiment(repo, run_name, practice=None, release=False):
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_-]{0,79}', run_name):
        raise ValueError('invalid run name')
    root = repo / 'target' / 'size-exchange' / run_name
    fresh_directory(root)
    before = provenance(repo, practice)
    (root / 'provenance.json').write_text(json.dumps(before, indent=2), encoding='utf-8')
    build = ['cargo', 'build', '--offline', '-p', 'viewer', '--example', 'size_exchange']
    if release:
        build.append('--release')
    command(build, repo)
    metadata = json.loads(command(['cargo', 'metadata', '--offline', '--format-version=1', '--no-deps'], repo))
    profile = 'release' if release else 'debug'
    binary = Path(metadata['target_directory']) / profile / 'examples' / ('size_exchange.exe' if os.name == 'nt' else 'size_exchange')
    passes = []
    for label in ['first', 'second']:
        require_same(before, provenance(repo, practice))
        print(f'Generating and verifying {label} pass...', flush=True)
        directory = root / label
        args = [str(binary), 'generate', '--corpus', str(repo / CORPUS), '--output', str(directory)]
        if practice:
            args += ['--practice', str(practice)]
        generation = json.loads(command(args, repo))
        for case in generation['cases']:
            evidence = directory / (case['id'] + '-preparation.json')
            if evidence.exists():
                case['initialPreparation'] = json.loads(evidence.read_bytes())
            if case['status'] == 'passed':
                try:
                    case['measurement'] = process_case(binary, repo, directory / case['id'])
                except Exception as error:
                    case['status'] = 'failed'
                    case['error'] = str(error)
        (directory / 'measurements.json').write_text(json.dumps(generation, indent=2), encoding='utf-8')
        passes.append(generation)
        require_same(before, provenance(repo, practice))
    require_same(passes[0], passes[1])
    schema = (repo / SCHEMA).read_bytes()
    result = {'version': 1, 'repeatability': 'exact', 'cargoProfile': profile, 'compression': {'gzipLevel': 9, 'mtime': 0, 'filename': ''},
              'profile': 'known-profile transfer', 'schema': {'file': SCHEMA, 'sha256': digest(schema),
              'rawBytes': len(schema), 'gzipBytes': len(compress_verified(schema))},
              'provenance': before, 'cases': passes[0]['cases']}
    result['status'] = 'passed' if all(c['status'] == 'passed' for c in result['cases']) else 'partial'
    (root / 'results.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    (root / 'report.md').write_text(markdown(result), encoding='utf-8')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', required=True)
    parser.add_argument('--practice', type=Path)
    parser.add_argument('--release', action='store_true', help='use optimized Rust tools for larger drawings')
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    result = run_experiment(repo, args.run, args.practice.resolve() if args.practice else None, args.release)
    print(f"{result['status']}; results: target/size-exchange/{args.run}/results.json")
    return 0 if result['status'] == 'passed' else 1


if __name__ == '__main__':
    sys.exit(main())
