"""Restore the complete historical converter checkpoint into a new directory."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile


def restore(destination):
    archive = Path(__file__).resolve().parent
    root = archive.parents[1]
    frozen = json.loads((archive / 'FREEZE.json').read_text(encoding='utf-8'))
    # Verify before creating anything. Raw Git bytes, not checkout line endings.
    for entry in frozen['archivedPaths']:
        if hashlib.sha256((archive / entry['path']).read_bytes()).hexdigest() != entry['sha256']:
            raise ValueError(f"archive hash mismatch: {entry['path']}")
    for key in ('lockfile', 'configuration'):
        entry = frozen[key]
        if hashlib.sha256((archive / entry['path']).read_bytes()).hexdigest() != entry['sha256']:
            raise ValueError(f'{key} hash mismatch')
    tree = subprocess.check_output(['git', 'rev-parse', frozen['freezeCommit'] + '^{tree}'], cwd=root, text=True).strip()
    if tree != frozen['sourceTree']:
        raise ValueError('frozen Git tree mismatch')
    data = subprocess.check_output(['git', '-c', 'core.autocrlf=false', '-c', 'core.eol=lf',
                                    'archive', '--format=tar', frozen['freezeCommit']], cwd=root)
    destination = destination.resolve()
    destination.mkdir(parents=True, exist_ok=False)
    with tarfile.open(fileobj=io.BytesIO(data)) as source:
        source.extractall(destination, filter='data')
    for entry in frozen['archivedPaths']:
        if hashlib.sha256((destination / entry['source']).read_bytes()).hexdigest() != entry['sha256']:
            raise ValueError(f"restored source mismatch: {entry['source']}")
    shutil.copyfile(archive / frozen['lockfile']['path'], destination / 'Cargo.lock')
    (destination / '.cargo').mkdir(exist_ok=True)
    shutil.copyfile(archive / frozen['configuration']['path'], destination / '.cargo/config.toml')
    print(f"Restored and verified {len(frozen['archivedPaths'])} reference files at {frozen['freezeCommit']}.")
    print('Prepare the pinned codec and run optional historical tests as described in the archive README.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--destination', required=True, type=Path)
    restore(parser.parse_args().destination)
