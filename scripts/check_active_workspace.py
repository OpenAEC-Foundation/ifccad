"""Reject parked OCDraw code and fixtures in the active Cargo graph."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys


def audit(root):
    completed = subprocess.run(
        ['cargo', 'metadata', '--locked', '--offline', '--format-version', '1'],
        cwd=root, capture_output=True, text=True, encoding='utf-8',
    )
    if completed.returncode:
        raise RuntimeError(completed.stderr.strip())
    metadata = json.loads(completed.stdout)
    errors = []
    legacy = (root / 'legacy').resolve()
    for package in metadata['packages']:
        manifest = Path(package['manifest_path']).resolve()
        if package['name'] in ('ocdraw', 'ocdraw-convert'):
            errors.append(f"parked package in active graph: {package['name']}")
        if manifest.is_relative_to(legacy):
            errors.append(f'active dependency under legacy: {manifest.relative_to(root)}')
        if package['id'] not in metadata['workspace_members']:
            continue
        directory = manifest.parent
        files = set()
        for folder in ('src', 'tests', 'examples', 'benches'):
            files.update((directory / folder).rglob('*.rs'))
        for target in package['targets']:
            if target['name'] in ('ocdraw', 'ocdraw_convert', 'ocdraw-convert'):
                errors.append(f"parked target in active graph: {target['name']}")
            path = Path(target['src_path']).resolve()
            if path.is_relative_to(legacy):
                errors.append(f'active target under legacy: {path.relative_to(root)}')
            if path.suffix == '.rs':
                files.add(path)
        build = directory / 'build.rs'
        if build.exists():
            files.add(build)
        for path in sorted(files):
            # Include macros may be multiline; checking every literal also catches
            # runtime file reads in tests/examples that would keep legacy live.
            content = path.read_text(encoding='utf-8')
            if re.search(r'(?:legacy[/\\]+ocdraw|ocdraw-convert|(?:schemas|conformance/next|tests/fixtures|examples)[/\\]+ocdraw)', content):
                errors.append(f'active source references parked material: {path.relative_to(root).as_posix()}')
            if re.search(r'\b(?:ocdraw|ocdraw_convert)::', content):
                errors.append(f'active source references parked API: {path.relative_to(root).as_posix()}')
    return sorted(set(errors))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        errors = audit(args.root.resolve())
    except (RuntimeError, OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print('Active workspace contains IFCCAD and its companions; no parked OCDraw dependency.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
