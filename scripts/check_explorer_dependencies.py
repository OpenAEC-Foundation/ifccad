"""Verify the shipped browser dependency graph uses the shared Document route."""
from pathlib import Path
import subprocess
import sys


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
    forbidden = [line for line in packages if line.startswith('ifccad-convert ')
                 or 'github.com/HakanSeven12/opencadcodec.git' in line
                 or (line.startswith('acadrust ') and
                     'github.com/HakanSeven12/acadrust.git' not in line)]
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
