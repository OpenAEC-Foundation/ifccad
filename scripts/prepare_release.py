"""Check/set IFCCAD release versions or prepare a deterministic source archive.

Local preparation only: this script never commits, tags, uploads or publishes.
Python 3.11+; run from any directory. See docs/releases.md.
"""
import argparse
from datetime import date
import gzip
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
import tomllib


ROOT = Path(__file__).resolve().parents[1]
def validate_version(value):
    match = re.fullmatch(r'([1-9][0-9]{3})\.([1-9][0-9]?)\.(0|[1-9][0-9]*)', value)
    if not match:
        raise ValueError('expected YYYY.W.N without leading zeroes or suffix')
    year, week, _ = map(int, match.groups())
    date.fromisocalendar(year, week, 1)
    return value


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8'))


def software_versions(root):
    cargo = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))
    return [cargo['workspace']['package']['version'],
            read_json(root / 'format-explorer/package.json')['version'],
            read_json(root / 'release/manifest.json')['softwareVersion']]


def set_version(root, value):
    validate_version(value)
    path = root / 'Cargo.toml'
    cargo, count = re.subn(r'(\[workspace\.package\]\s*\nversion = ")[^"]+(")',
                            lambda m: m.group(1) + value + m.group(2),
                            path.read_text(encoding='utf-8'))
    if count != 1:
        raise ValueError('expected one workspace.package version')
    updates = {path: cargo}
    for relative, key in [('format-explorer/package.json', 'version'),
                          ('release/manifest.json', 'softwareVersion')]:
        target = root / relative
        data = read_json(target)
        data[key] = value
        updates[target] = json.dumps(data, indent=2, ensure_ascii=False) + '\n'
    for target, contents in updates.items():
        target.write_text(contents, encoding='utf-8', newline='\n')


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args]).decode('utf-8').strip()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def check(root):
    versions = software_versions(root)
    value = validate_version(versions[0])
    if versions != [value] * len(versions):
        raise ValueError(f'software version drift: {versions}')
    cargo = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))
    names = set()
    for relative in ['Cargo.toml'] + [f'{p}/Cargo.toml' for p in cargo['workspace']['members']]:
        package = tomllib.loads((root / relative).read_text(encoding='utf-8'))['package']
        names.add(package['name'])
        if package['version'] != {'workspace': True}:
            raise ValueError(f'{relative} must inherit workspace version')
        if relative != 'Cargo.toml' and package.get('publish') is not False:
            raise ValueError(f'{relative} must remain source-only')
    manifest = read_json(root / 'release/manifest.json')
    revision = manifest['cborMappingRevision']
    mapping = read_json(root / f'schemas/ifccad/cbor-mapping-{revision}.json')
    if mapping['mappingRevision'] != revision:
        raise ValueError('CBOR mapping revision drift')
    profile = manifest['profileVersion']
    module = root / f'schemas/ifccad/ifccad-profile-{profile}.ifcx'
    if not module.is_file():
        raise ValueError('missing profile module')
    if read_json(module)['header']['id'] != manifest['profileImport']:
        raise ValueError('profile import drift')
    for relative, literal in [
        ('src/ifccad/codec/json/validate.rs', f'drawing.profile_version != "{profile}"'),
        ('src/ifccad/codec/json/encode.rs', f'"profileVersion":"{profile}"'),
        ('src/ifccad/mod.rs', f'PROFILE_URI: &str = "{manifest["profileImport"]}"'),
        ('src/ifccad/codec/cbor/mod.rs', f'revision != {revision}'),
    ]:
        if literal not in (root / relative).read_text(encoding='utf-8'):
            raise ValueError(f'format/reader drift: {relative}')
    if not (root / f"schemas/ifccad/cbor-zstd-transport-{manifest['zstdTransportRevision']}.md").is_file():
        raise ValueError('missing Zstd transport contract')
    lock = tomllib.loads((root / 'Cargo.lock').read_text(encoding='utf-8'))
    locked_names = set()
    for package in lock['package']:
        if package['name'] in names:
            locked_names.add(package['name'])
            if package['version'] != value or package.get('source') is not None:
                raise ValueError(f"lockfile version/source drift: {package['name']}")
    if locked_names != names:
        raise ValueError('lockfile is missing workspace packages')
    return value


def write_archive(output, files):
    for name in files:
        path = PurePosixPath(name)
        if path.is_absolute() or '..' in path.parts or '\\' in name or ':' in name:
            raise ValueError(f'unsafe archive member: {name}')
    # Exclusive creation prevents replacing a previous candidate accidentally.
    with output.open('xb') as raw, gzip.GzipFile(fileobj=raw, mode='wb', filename='', mtime=0) as zipped:
        with tarfile.open(fileobj=zipped, mode='w', format=tarfile.PAX_FORMAT) as archive:
            for name, data in sorted(files.items()):
                info = tarfile.TarInfo(name)
                info.size = len(data)
                info.mode = 0o644
                info.mtime = 0
                archive.addfile(info, io.BytesIO(data))


def core_package_files(package, version, allow_unused_patch=False):
    prefix = f'ifccad-{version}/'
    files = {}
    with tarfile.open(package, 'r:gz') as archive:
        for member in archive:
            path = PurePosixPath(member.name)
            if (not member.isfile() or not member.name.startswith(prefix)
                    or path.as_posix() != member.name or '..' in path.parts
                    or '\\' in member.name or ':' in member.name):
                raise ValueError(f'unsafe core package member: {member.name}')
            relative = member.name[len(prefix):]
            if relative in files:
                raise ValueError('duplicate core package member')
            files[relative] = archive.extractfile(member).read()
    manifest = tomllib.loads(files['Cargo.toml'].decode())
    if (manifest['package']['name'] != 'ifccad'
            or manifest['package']['version'] != version or 'workspace' in manifest):
        raise ValueError('expected a normalized standalone native core package')
    for kind in ('dependencies', 'dev-dependencies', 'build-dependencies'):
        for name, spec in manifest.get(kind, {}).items():
            if (name.startswith('cad-') or name in ('acadrust', 'opencadcodec', 'ifccad-convert', 'viewer', 'browser')
                    or isinstance(spec, dict) and ('path' in spec or 'git' in spec)):
                raise ValueError(f'non-registry dependency in native core: {name}')
    for name in files:
        if name.startswith(('crates/', 'format-explorer/', 'legacy/', '.superpowers/', '.cargo/')):
            raise ValueError(f'non-core source in native core package: {name}')
    if 'Cargo.lock' not in files:
        raise ValueError('native core package must include its lockfile')
    lock = tomllib.loads(files['Cargo.lock'].decode())
    if lock.get('patch') and not allow_unused_patch:
        raise ValueError('workspace patch metadata in native core lockfile')
    for package in lock.get('package', []):
        if package['name'].startswith('cad-') or package['name'] in ('acadrust', 'opencadcodec', 'ifccad-convert', 'viewer', 'browser'):
            raise ValueError('CAD/application dependency in native core lockfile')
    return files


def package_core(root, staging):
    value = check(root)
    # Resolve the repository's existing workspace unchanged, then isolate the
    # normalized native package so repository codec configuration cannot leak.
    subprocess.run(['cargo', 'package', '-p', 'ifccad', '--locked', '--offline',
                    '--allow-dirty', '--no-verify'], cwd=root, check=True)
    files = core_package_files(root / f'target/package/ifccad-{value}.crate', value,
                               allow_unused_patch=True)
    staging.mkdir(parents=True, exist_ok=False)
    for name, data in files.items():
        if name in ('Cargo.toml.orig', '.cargo_vcs_info.json'):
            continue
        target = staging / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    manifest = staging / 'Cargo.toml'
    with manifest.open('a', encoding='utf-8', newline='\n') as output:
        output.write('\n[workspace]\n')
    before = tomllib.loads((staging / 'Cargo.lock').read_text())['package']
    # Cargo reads config from cwd/ancestors. The drive root intentionally avoids
    # the repository configuration; the exact Rust toolchain is explicit.
    command = ['cargo', '+1.98.0']
    subprocess.run(command + ['metadata', '--offline', '--format-version', '1',
                   '--manifest-path', str(manifest)], cwd=root.anchor,
                   check=True, stdout=subprocess.DEVNULL)
    after = tomllib.loads((staging / 'Cargo.lock').read_text())['package']
    if before != after:
        raise ValueError('native isolation unexpectedly changed resolved registry dependencies')
    subprocess.run(command + ['package', '--locked', '--offline', '--allow-dirty',
                   '--manifest-path', str(manifest)], cwd=root.anchor, check=True)
    package = staging / f'target/package/ifccad-{value}.crate'
    core_package_files(package, value)
    destination = root / f'target/package/ifccad-{value}-native.crate'
    shutil.copyfile(package, destination)
    print(f'Native-only verified package: {destination}')


def bundle(root, output, candidate=False):
    value = check(root)
    status = git(root, 'status', '--porcelain', '--untracked-files=all')
    if status and not candidate:
        raise ValueError('final source bundle requires a clean checkout; use --candidate for local review')
    additions = git(root, 'ls-files', '--others', '--exclude-standard', '-z').split('\0')
    allowed = read_json(root / 'release/manifest.json')['candidateFiles']
    unexpected = set(filter(None, additions)) - set(allowed)
    if unexpected:
        raise ValueError(f'unreviewed untracked files: {sorted(unexpected)}')
    package = root / f'target/package/ifccad-{value}-native.crate'
    files = core_package_files(package, value)
    listing = subprocess.check_output(
        ['cargo', 'package', '-p', 'ifccad', '--list', '--locked', '--allow-dirty'],
        cwd=root, stderr=subprocess.PIPE, text=True)
    generated = {'.cargo_vcs_info.json'}
    if ({name.replace('\\', '/') for name in listing.splitlines()} - generated
            != set(files) - generated):
        raise ValueError('core package file list is stale; rerun cargo package')
    # Refuse a stale package after source edits. Cargo normalizes these metadata
    # files; all ordinary source files must match the inspected working tree.
    for relative, data in files.items():
        if relative in ('Cargo.toml', 'Cargo.toml.orig', 'Cargo.lock', '.cargo_vcs_info.json'):
            continue
        source = root / ('Cargo.toml' if relative == 'Cargo.toml.orig' else relative)
        if not source.is_file() or source.read_bytes() != data:
            raise ValueError(f'core package is stale: {relative}; rerun cargo package')
    inventory = {
        'softwareVersion': value, 'alpha': True, 'candidate': candidate,
        'baseCommit': git(root, 'rev-parse', 'HEAD'),
        'scope': 'native ifccad core only',
        'crateSha256': sha(package.read_bytes()),
        'formats': {k: v for k, v in read_json(root / 'release/manifest.json').items()
                    if k in ('profileVersion', 'profileImport', 'cborMappingRevision', 'zstdTransportRevision')},
        'files': {name: sha(data) for name, data in sorted(files.items())},
    }
    files['SOURCE-MANIFEST.json'] = (json.dumps(inventory, indent=2) + '\n').encode()
    prefix = f'ifccad-{value}/'
    write_archive(output, {prefix + name: data for name, data in files.items()})
    digest = sha(output.read_bytes())
    print(f'{digest}  {output.name}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    commands.add_parser('check')
    version = commands.add_parser('set-version')
    version.add_argument('version')
    native = commands.add_parser('package')
    native.add_argument('--staging', type=Path, required=True,
                        help='Fresh local directory for standalone native verification')
    archive = commands.add_parser('bundle')
    archive.add_argument('--output', type=Path, required=True)
    archive.add_argument('--candidate', action='store_true')
    args = parser.parse_args()
    try:
        if args.command == 'set-version':
            set_version(ROOT, args.version)
            print('Version set; refresh Cargo.lock with cargo metadata --offline before checking.')
        elif args.command == 'check':
            print(f'Consistent IFCCAD alpha release {check(ROOT)}')
        elif args.command == 'package':
            package_core(ROOT, args.staging.resolve())
        else:
            bundle(ROOT, args.output.resolve(), args.candidate)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    main()
