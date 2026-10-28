"""Manual six-case IFCCAD JSON/CBOR baseline; never invoked by tests or CI."""
import argparse
import hashlib
import json
import os
import platform
import subprocess
import time
import tomllib
import zipfile
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CORPUS = 'benchmarks/ifccad-cbor/corpus-v1.json'
RESULT = 'benchmarks/ifccad-cbor/results-v1.json'
REPORT = 'docs/benchmarks/ifccad-cbor-results-v1.md'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def dump(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write('\n')


def command(args, cwd=REPO):
    return subprocess.check_output(args, cwd=cwd, encoding='utf-8').strip()


def tracked_inputs(root):
    names = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z'], cwd=root).decode().split('\0')
    return sorted(set(n for n in names if n))


def build_inputs():
    names = [n for n in tracked_inputs(REPO)
             if n.startswith(('src/', 'crates/', 'schemas/', 'examples/'))
             or n.startswith('benchmarks/ifccad-cbor/exclusions-')
             or n in ('Cargo.toml', 'Cargo.lock', CORPUS, 'scripts/ifccad_cbor_baseline.py')]
    return sorted(set(names + ['Cargo.lock']))


def identity():
    cfg = tomllib.loads((REPO / '.cargo/config.toml').read_text())
    dep = Path(cfg['patch']['https://github.com/HakanSeven12/opencadcodec.git']['opencadcodec']['path'])
    files = {n: sha((REPO / n).read_bytes()) if (REPO / n).is_file() else None for n in build_inputs()}
    dependency = {n: sha((dep / n).read_bytes()) if (dep / n).is_file() else None for n in tracked_inputs(dep)}
    configs = {}
    for name, path in [('task', REPO / '.cargo/config.toml'), ('parent', REPO.parent.parent / '.cargo/config.toml'), ('global', Path.home() / '.cargo/config.toml')]:
        configs[name] = sha(path.read_bytes()) if path.is_file() else None
    return {'base': command(['git', 'rev-parse', 'HEAD']), 'sourceFiles': files,
            'sourceHash': sha(json.dumps(files, sort_keys=True).encode()),
            'dependencyRevision': command(['git', 'rev-parse', 'HEAD'], dep),
            'dependencyFiles': dependency, 'dependencyHash': sha(json.dumps(dependency, sort_keys=True).encode()),
            'dependencyStatus': command(['git', 'status', '--porcelain'], dep),
            'configurationHashes': configs, 'rustc': command(['rustc', '-Vv']),
            'cargo': command(['cargo', '-V']), 'profile': 'release; unchanged Cargo.toml profile',
            'target': 'x86_64-pc-windows-msvc',
            'environment': {n: os.environ.get(n) for n in ['CARGO_TARGET_DIR', 'CARGO_INCREMENTAL', 'RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTUP_TOOLCHAIN']}}


def archive_inputs(evidence, state):
    with zipfile.ZipFile(evidence / 'source-inputs.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
        for name, digest in state['sourceFiles'].items():
            if digest is not None:
                archive.write(REPO / name, name)
        archive.write(REPO / '.cargo/config.toml', '.cargo/config.toml')
        parent = REPO.parent.parent / '.cargo/config.toml'
        if parent.is_file():
            archive.write(parent, 'parent-cargo-config.toml')
    cfg = tomllib.loads((REPO / '.cargo/config.toml').read_text())
    dep = Path(cfg['patch']['https://github.com/HakanSeven12/opencadcodec.git']['opencadcodec']['path'])
    (evidence / 'selected-codec.patch').write_bytes(subprocess.check_output(['git', 'diff', '--binary', 'HEAD'], cwd=dep))
    # Tracked upstream is recoverable by revision. Preserve any additional source files.
    extra = subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', '-z'], cwd=dep).decode().split('\0')
    with zipfile.ZipFile(evidence / 'selected-codec-extra.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
        for name in extra:
            if name and (dep / name).is_file():
                archive.write(dep / name, name)


def active_builds():
    if os.name != 'nt':
        return ''
    script = "Get-Process cargo,rustc,clippy-driver,link -ErrorAction SilentlyContinue | Select-Object ProcessName,Id | ConvertTo-Json -Compress"
    return subprocess.run(['pwsh', '-NoProfile', '-Command', script], capture_output=True, text=True).stdout.strip()


def render(result, result_path=RESULT, report_path=REPORT):
    lines = ['# Initial IFCCAD JSON / base-CBOR measurement', '',
             f"Run `{result['run']}`; {result['date']}. Size is the primary result; timings are indicative.", '',
             '| Case | JSON bytes | Compact JSON bytes | CBOR bytes | CBOR / compact |',
             '| --- | ---: | ---: | ---: | ---: |']
    passed = [c for c in result['cases'] if c['status'] == 'passed']
    for case in passed:
        sizes = [a['bytes'] for a in case['artifacts']]
        lines.append(f"| {case['id']} | {sizes[0]} | {sizes[1]} | {sizes[2]} | {case['cborToCompactJson']:.3f} |")
    lines += ['', 'Median native API time, milliseconds; 1 warmup + 5 samples, no filesystem IO.', '',
              '| Case | JSON open | Compact open | CBOR open | JSON write | CBOR write |',
              '| --- | ---: | ---: | ---: | ---: | ---: |']
    for case in passed:
        values = [case['timing'][key]['medianMs'] for key in ['jsonRead', 'compactJsonRead', 'cborRead', 'jsonWrite', 'cborWrite']]
        lines.append('| ' + case['id'] + ' | ' + ' | '.join(f'{v:.3f}' for v in values) + ' |')
    lines += ['', 'Production writers include validation and mandatory readback. JSON opening also retains',
              'the IFCX source graph; CBOR returns the CAD document. Compact JSON has no dedicated',
              'production writer. These are complete route costs, not pure token-parser comparisons.', '',
              'Every accepted case passed strict production readback, full typed/numeric comparison',
              'and one untimed independent repeat. All original CAD content is not necessarily retained.', '']
    for case in result['cases']:
        if case['status'] == 'failed':
            lines.append(f"Failed `{case['id']}`: {case['error']}")
        elif case['id'].startswith('practice-'):
            prep = case['preparation']['conversion']
            if prep.get('exclusions'):
                exclusions = prep['exclusions']
                kinds = {kind: sum(e['kind'] == kind for e in exclusions) for kind in sorted({e['kind'] for e in exclusions})}
                lines += [f"Explicit benchmark exclusions: {len(exclusions)} entities; kinds `{json.dumps(kinds, sort_keys=True)}`. Full handles/errors are in the detailed results.",
                          'The original file is unchanged; all variants contain the same filtered conversion result.', '']
            lines += [f"`{case['id']}` retained {case['inventory']['storedEntities']} entities;",
                      f"diagnostic actions: `{json.dumps(prep['diagnosticActions'], sort_keys=True)}`.",
                      f"Retained kinds: `{json.dumps(case['inventory']['kinds'], sort_keys=True)}`.", '']
    lines += ['Current selected codec is locally patched; exact source/config/dependency identities,',
              'source hashes, diagnostic records, sample ranges and raw samples are retained in',
              f"[matching detailed results]({os.path.relpath(result_path, Path(report_path).parent).replace(chr(92), '/')}).",
              'Applicable source/config/input snapshots and selected dependency patch are retained',
              f"outside Git under `.superpowers/measurements/ifccad-cbor/{result['run']}/`.", '',
              'Scope/reproduction: [six-case protocol](ifccad-cbor-baseline-v1.md).',
              'No compression, DWG-size equivalence, cold-start, memory or general performance claim.',
              'Old OCDraw/common-subset reports remain unchanged. Full integration verification of',
              'the changed worktree remains pending; these are scoped measurement correctness results.', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cases', required=True, help='Explicit comma-separated selection from corpus-v1.json')
    parser.add_argument('--source-root', type=Path, help='Original IFCCAD Prototype directory')
    parser.add_argument('--run', default='cbor-base-' + datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))
    parser.add_argument('--result', default=RESULT, help='Fresh repository-relative detailed result path')
    parser.add_argument('--report', default=REPORT, help='Fresh repository-relative report path')
    parser.add_argument('--omit-test-dxf-entity', choices=['EF354'], help='Explicitly authorized single benchmark exclusion')
    parser.add_argument('--omit-test-dxf-dimension-errors', choices=['point2-proven-exceedance'], help='Authorized benchmark-only dimension geometry exclusion')
    parser.add_argument('--omit-test-dxf-entity-errors', choices=['identified-entity'], help='Authorized benchmark-only identified source entity errors')
    parser.add_argument('--exclusions', type=Path, help='Verified source-specific prior exclusion manifest')
    args = parser.parse_args()
    if not args.run or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_' for c in args.run):
        raise ValueError('run name must be a simple directory name')
    corpus = json.loads((REPO / CORPUS).read_text())
    rows = {c['id']: c for c in corpus['cases']}
    chosen = args.cases.split(',')
    if len(set(chosen)) != len(chosen) or any(c not in rows for c in chosen):
        raise ValueError('unknown, empty or duplicate case')
    for output in [args.result, args.report]:
        if Path(output).is_absolute() or not (REPO / output).resolve().is_relative_to(REPO):
            raise ValueError('output must stay within the repository')
    if (args.omit_test_dxf_entity or args.omit_test_dxf_dimension_errors or args.omit_test_dxf_entity_errors) and chosen != ['practice-test-dxf']:
        raise ValueError('explicit exclusion requires selecting only practice-test-dxf')
    if args.exclusions and not args.omit_test_dxf_entity_errors:
        raise ValueError('exclusion manifest requires identified-entity policy')
    if (REPO / args.result).exists() or (REPO / args.report).exists():
        raise ValueError('retained results exist; select a new report version before another run')
    evidence = REPO / '.superpowers/measurements/ifccad-cbor' / args.run
    evidence.mkdir(parents=True, exist_ok=False)
    scratch = REPO / 'target/ifccad-cbor-baseline' / args.run
    scratch.parent.mkdir(parents=True, exist_ok=True)
    if scratch.exists():
        raise ValueError('scratch run already exists')
    originals = {}
    source_args = []
    if args.omit_test_dxf_entity:
        source_args += ['--omit-test-dxf-entity', args.omit_test_dxf_entity]
    if args.omit_test_dxf_dimension_errors:
        source_args += ['--omit-test-dxf-dimension-errors', args.omit_test_dxf_dimension_errors]
    if args.omit_test_dxf_entity_errors:
        source_args += ['--omit-test-dxf-entity-errors', args.omit_test_dxf_entity_errors]
    for case_id in chosen:
        row = rows[case_id]
        if 'file' not in row:
            continue
        if args.source_root is None:
            raise ValueError('original source-root required for practice cases')
        path = args.source_root / row['file']
        data = path.read_bytes()
        if sha(data) != row['sha256']:
            raise ValueError(f'original hash mismatch for {case_id}')
        destination = evidence / ('foundation.dwg' if case_id == 'practice-foundation' else 'test.dxf')
        destination.write_bytes(data)
        originals[case_id] = {'file': row['file'], 'bytes': len(data), 'sha256': sha(data)}
        source_args += ['--foundation' if case_id == 'practice-foundation' else '--test-dxf', str(destination)]
    build_args = ['cargo', 'build', '--offline', '--release', '-p', 'ifccad-convert', '--example', 'cbor_baseline']
    print('Building unchanged release profile before measurement', flush=True)
    before_build = identity()
    dump(evidence / 'inputs-before-build.json', before_build)
    started = time.monotonic()
    with (evidence / 'build.log').open('w', encoding='utf-8') as log:
        subprocess.run(build_args, cwd=REPO, stdout=log, stderr=subprocess.STDOUT, check=True)
    build_seconds = time.monotonic() - started
    state = identity()
    if args.exclusions:
        manifest = json.loads(args.exclusions.read_text())
        if manifest['sourceSha256'] != originals['practice-test-dxf']['sha256']:
            raise ValueError('exclusion source hash mismatch')
        prior = manifest['provenance']
        def production_files(identity):
            return {name: digest for name, digest in identity['sourceFiles'].items()
                    if name.startswith(('src/', 'crates/', 'schemas/')) and '/examples/' not in name}
        if production_files(prior) != production_files(state):
            raise ValueError('production inputs changed since exclusion proof')
        for key in ['dependencyFiles', 'configurationHashes', 'rustc', 'cargo', 'environment', 'target', 'profile']:
            if state[key] != prior[key]:
                raise ValueError('exclusion proof configuration changed: ' + key)
        data = args.exclusions.read_bytes()
        destination = evidence / 'source-exclusions.json'
        destination.write_bytes(data)
        source_args += ['--exclusions', str(destination)]
        originals['practice-test-dxf']['exclusionManifest'] = {'sha256': sha(data), 'priorRun': manifest['evidenceRun'], 'records': len(manifest['exclusions'])}
    comparable_before = json.loads(json.dumps(before_build))
    comparable_after = json.loads(json.dumps(state))
    for comparison in [comparable_before, comparable_after]:
        comparison.pop('sourceHash')
        comparison['sourceFiles'].pop('Cargo.lock', None)
    if comparable_before != comparable_after:
        raise RuntimeError('source/config/dependency changed during build; no measurements started')
    dump(evidence / 'inputs-before.json', state)
    archive_inputs(evidence, state)
    busy = active_builds()
    for _ in range(60):
        if not busy:
            break
        print('Waiting for other Cargo/compiler processes before timing', flush=True)
        time.sleep(10)
        busy = active_builds()
    if busy:
        raise RuntimeError('other builds still active; no measurements started')
    binary = REPO / 'target/release/examples' / ('cbor_baseline.exe' if os.name == 'nt' else 'cbor_baseline')
    print('Starting explicitly selected cases: ' + args.cases, flush=True)
    with (evidence / 'run.log').open('w', encoding='utf-8') as log:
        completed = subprocess.run([str(binary), '--out', str(scratch), '--cases', args.cases, *source_args], cwd=REPO, stdout=log, stderr=subprocess.STDOUT)
    after = identity()
    dump(evidence / 'inputs-after.json', after)
    if state != after:
        raise RuntimeError('source/config/dependency/toolchain inputs changed; run is not accepted')
    result = json.loads((scratch / 'results.json').read_text())
    result.update(run=args.run, date=datetime.now(timezone.utc).isoformat(), inputs=originals,
                  provenance=state, binarySha256=sha(binary.read_bytes()), buildCommand=build_args,
                  buildSeconds=build_seconds, runnerExit=completed.returncode,
                  machine={'platform': platform.platform(), 'cpu': platform.processor(), 'logicalCpus': os.cpu_count(), 'python': platform.python_version()},
                  activeBuildsBefore='', activeBuildsAfter=active_builds())
    for case in result['cases']:
        directory = scratch / case['id']
        for name in ['preparation.json', 'source-inventory.json']:
            if (directory / name).is_file():
                case['preparation' if name == 'preparation.json' else 'sourceRead'] = json.loads((directory / name).read_text())
        for artifact in case.get('artifacts', []):
            artifact['sha256'] = sha((directory / artifact['file']).read_bytes())
    dump(evidence / 'results.json', result)
    dump(REPO / args.result, result)
    report = render(result, args.result, args.report)
    with (REPO / args.report).open('x', encoding='utf-8', newline='\n') as stream:
        stream.write(report)
    print(report, flush=True)
    return completed.returncode


if __name__ == '__main__':
    raise SystemExit(main())
