"""Verify the shipped browser dependency graph uses the shared Document route."""
from pathlib import Path
import json
import subprocess
import sys


def development_codec(pins):
    override = pins.get('developmentOverrides', {}).get('acadrust')
    if (pins.get('localDevelopment') == 'leader-slice' and override
            and override.get('baseCommit') == pins.get('explorerCodecCommit')
            and pins.get('explorerCodecRepository') == 'https://github.com/BjornFidder/opencadcodec.git'):
        return (Path(__file__).resolve().parents[1] / override['path']).resolve().as_posix()
    if (pins.get('localDevelopment') == 'block-attributes'
            and pins.get('explorerCodecRepository') == 'https://github.com/BjornFidder/opencadcodec.git'
            and pins.get('explorerCodecCommit') == '0f20af02a1d6be744272e64dd505e8c8933c8772'):
        return pins['explorerCodecRepository'] + '?rev=' + pins['explorerCodecCommit'] + '#'
    override = pins.get('developmentOverrides', {}).get('acadrust')
    if pins.get('localDevelopment') == 'block-attributes' and override and override.get('baseCommit') == pins.get('explorerCodecCommit') and pins.get('explorerCodecRepository') == 'https://github.com/hakanaktt/acadrust.git':
        return (Path(__file__).resolve().parents[1] / override['path']).resolve().as_posix()

    if (pins.get('localDevelopment') in {'annotation-contexts', 'font-symbol-sources'}
            and pins.get('explorerCodecRepository') == 'https://github.com/BjornFidder/opencadcodec.git'
            and pins.get('explorerCodecCommit') == '4018037daf8fd3e40aa1aa6455b80232849a2756'):
        return pins['explorerCodecRepository'] + '?rev=' + pins['explorerCodecCommit'] + '#'
    return None


def main():
    root = Path(__file__).resolve().parents[1]
    result = subprocess.run(
        ['cargo', 'tree', '--locked', '--offline', '-p', 'browser',
         '--target', 'wasm32-unknown-unknown', '--edges', 'normal,build',
         '--prefix', 'none', '--format', '{p}'],
        cwd=root, capture_output=True, text=True, encoding='utf-8',
    )
    if result.returncode:
        print(result.stderr, file=sys.stderr)
        return 1
    packages = result.stdout.splitlines()
    pins = json.loads((root / 'format-explorer/ocs-source.json').read_text())
    allowed_codec = development_codec(pins)
    forbidden = [line for line in packages if line.startswith('ifccad-convert ')
                 or 'github.com/HakanSeven12/opencadcodec.git' in line
                 or (line.startswith('acadrust ') and
                     'github.com/hakanaktt/acadrust.git' not in line and not (allowed_codec and allowed_codec in line.replace("\\", "/")))]
    if forbidden:
        print('Explorer depends on the retired reference conversion route:\n' +
              '\n'.join(sorted(set(forbidden))), file=sys.stderr)
        return 1
    if not any(line.startswith('opencaddocument ') for line in packages):
        print('Explorer is missing its shared Document converter', file=sys.stderr)
        return 1
    print('Explorer uses opencaddocument; no ifccad-convert or reference codec dependency.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
