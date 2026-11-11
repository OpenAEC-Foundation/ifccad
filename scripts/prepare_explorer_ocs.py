"""Apply the explicitly pinned Explorer host dependencies and frozen lockfile."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
INPUTS = ROOT / "format-explorer/ocs-inputs"


def development_sources(pins, package):
    sources = {
        'ifccad': ['https://github.com/OpenAEC-Foundation/ifccad.git'],
        'opencaddocument': ['https://github.com/HakanSeven12/opencaddocument.git'],
        'acadrust': ['https://github.com/hakanaktt/acadrust.git', 'https://github.com/HakanSeven12/acadrust.git'],
    }[package]
    if package == 'acadrust':
        sources += [pins[key] for key in ['codecRepository', 'explorerCodecRepository'] if pins.get(key)]
    return sorted(set(sources))


def prepare_codec_patch(text, pins):
    repository = pins.get("codecRepository")
    if not repository or repository == "https://github.com/hakanaktt/acadrust.git" or "acadrust" in pins.get("developmentOverrides", {}):
        return text
    # Kernel still requests the former URL. Both sources must resolve to the
    # same exact codec as Document, rather than create distinct Rust models.
    for source in ("https://github.com/hakanaktt/acadrust.git",
                   "https://github.com/HakanSeven12/acadrust.git"):
        patch = (f'[patch."{source}"]\n'
                 f'acadrust = {{ git = "{repository}", rev = "{pins["codecCommit"]}" }}\n')
        pattern = r'(?m)^\[patch\."' + re.escape(source) + r'"\]\n[^\[]*'
        text = re.sub(pattern, lambda _: patch + "\n", text) if re.search(pattern, text) else text.rstrip() + "\n\n" + patch + "\n"
    return text


def apply_host_patch(checkout, patch):
    """Apply a frozen patch once; refuse a different, partially applied source."""
    def check(*flags):
        return subprocess.run(['git', 'apply', '--check', *flags, str(patch)],
                              cwd=checkout, capture_output=True).returncode == 0
    if check('--reverse'):
        return
    if not check():
        raise SystemExit('Host source differs from the selected patch')
    subprocess.run(['git', 'apply', str(patch)], cwd=checkout, check=True)


def host_source_identity(checkout):
    digest = hashlib.sha256()
    files = subprocess.check_output([
        "git", "-C", str(checkout), "ls-files", "--cached", "--others",
        "--exclude-standard", "-z"
    ]).decode("utf-8").split("\0")
    for relative in sorted(set(files + [".cargo/config.toml"])):
        path = checkout / relative
        if relative and path.is_file() and path.suffix in {
            ".rs", ".toml", ".lock", ".js", ".html", ".ftl", ".wgsl", ".ifcx",
            ".sh", ".ps1", ".py", ".json", ".css", ".svg", ".png", ".ttf", ".woff", ".woff2"
        }:
            digest.update(relative.encode("utf-8") + b"\0")
            digest.update(hashlib.sha256(path.read_bytes()).digest())
    return digest.hexdigest()


def development_identity(pins):
    rows = []
    for name, override in sorted(pins.get("developmentOverrides", {}).items()):
        source = (ROOT / override["path"]).resolve()
        files = subprocess.check_output(["git", "-C", str(source), "ls-files", "--cached", "--others", "--exclude-standard", "-z"]).decode("utf-8").split("\0")
        digest = hashlib.sha256()
        for relative in sorted(set(files + [".cargo/config.toml"])):
            path = source / relative
            if not relative or not path.is_file() or not (path.suffix in {".rs", ".toml", ".lock", ".json", ".ifcx"}):
                continue
            digest.update(relative.encode("utf-8") + b"\0")
            digest.update(hashlib.sha256(path.read_bytes()).digest())
        rows.append({"package": name, "baseCommit": override["baseCommit"], "contentSha256": digest.hexdigest()})
    return rows


def identity():
    paths = [ROOT / "format-explorer/ocs-source.json", Path(__file__), ROOT / ".github/workflows/format-explorer.yml", ROOT / "rust-toolchain.toml"]
    paths += sorted(p for p in INPUTS.rglob('*') if p.is_file())
    digest = hashlib.sha256()
    for path in paths:
        digest.update(str(path.relative_to(ROOT)).encode())
        digest.update(path.read_bytes())
    pins = json.loads(paths[0].read_text(encoding="utf-8"))
    digest.update(json.dumps(development_identity(pins), sort_keys=True).encode("utf-8"))
    return digest.hexdigest()


def validate_packages(packages, pins):
    expected = [("opencaddocument", pins["documentCommit"]),
                ("ifccad", pins["nativeCoreCommit"]),
                ("acadrust", pins["codecCommit"])]
    if "kernelCommit" in pins:
        expected += [("opencadkernel", pins["kernelCommit"]),
                     ("opencadkernel-constraints", pins["kernelCommit"])]
    for name, revision in expected:
        matches = [p for p in packages if p["name"] == name]
        override = pins.get("developmentOverrides", {}).get(name)
        if override:
            expected_path = (ROOT / override["path"] / "Cargo.toml").resolve()
            if len(matches) != 1 or matches[0].get("source") is not None or Path(matches[0].get("manifest_path", "")).resolve() != expected_path:
                raise SystemExit(f"Unexpected resolved {name} development dependency")
            continue
        if len(matches) != 1 or not (matches[0]["source"] or "").endswith("#" + revision):
            raise SystemExit(f"Unexpected resolved {name} dependency")
        if name == "acadrust" and pins.get("codecRepository"):
            if (matches[0]["source"] or "").split("?")[0] != "git+" + pins["codecRepository"]:
                raise SystemExit("Unexpected acadrust repository")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checkout", type=Path)
    parser.add_argument("--record-lock", action="store_true")
    parser.add_argument("--record-build", action="store_true")
    parser.add_argument("--verify-build", action="store_true")
    parser.add_argument("--verify-dependencies", action="store_true")
    args = parser.parse_args()
    pins = json.loads((ROOT / "format-explorer/ocs-source.json").read_text())
    checkout = args.checkout.resolve()
    head = subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip()
    if head != pins["commit"]:
        raise SystemExit("Studio checkout differs from ocs-source.json")
    if pins.get('hostPatch'):
        if Path(pins['hostPatch']).name != pins['hostPatch']:
            raise SystemExit('Host patch must be a file in ocs-inputs')
        apply_host_patch(checkout, INPUTS / pins['hostPatch'])
    if pins.get('hostPatch') == 'block-attributes.patch':
        for source, target in [('block_attributes_smoke.rs', 'src/app/block_attributes_smoke.rs'),
                               ('block-attributes.ifcx', 'tests/fixtures/ifccad-block-attributes.ifcx')]:
            (checkout / target).write_bytes((INPUTS / source).read_bytes())
        module = checkout / 'src/app/mod.rs'
        text = module.read_text(encoding='utf-8')
        if 'mod block_attributes_smoke;' not in text:
            module.write_text(text + '\n#[cfg(test)]\nmod block_attributes_smoke;\n', encoding='utf-8')
        if pins.get('localDevelopment') == 'leader-slice':
            (checkout / 'src/app/leaders_smoke.rs').write_bytes(
                (INPUTS / 'leaders_smoke.rs').read_bytes())
            text = module.read_text(encoding='utf-8')
            if 'mod leaders_smoke;' not in text:
                module.write_text(text + '\n#[cfg(test)]\nmod leaders_smoke;\n', encoding='utf-8')
    manifests = ["Cargo.toml", "crates/ocs_save/Cargo.toml", "crates/ocs_web_worker/Cargo.toml", "crates/ocs_plugin_api/Cargo.toml", "crates/ocs_acis/Cargo.toml"]
    for name in manifests:
        path = checkout / name
        text = path.read_text()
        pattern = r'((?:codec|document)\s*=\s*\{[^\n]*package\s*=\s*"opencaddocument"[^\n]*git\s*=\s*")[^"]+("[^\n]*rev\s*=\s*")[^"]+("[^\n]*\})'
        text, count = re.subn(pattern, lambda m: m[1] + pins["documentRepository"] + m[2] + pins["documentCommit"] + m[3], text)
        expected = 2 if name == "crates/ocs_plugin_api/Cargo.toml" else 1
        if count != expected:
            raise SystemExit(f"Expected {expected} Document dependencies in {name}")
        path.write_text(text)
    path = checkout / "Cargo.toml"
    path.write_text(prepare_codec_patch(path.read_text(), pins))
    # Explicit local development pairs do not masquerade as unmodified Git revisions.
    config_path = checkout / ".cargo/config.toml"
    config = config_path.read_text(encoding="utf-8")
    for package, override in pins.get("developmentOverrides", {}).items():
        source = (ROOT / override["path"]).resolve()
        actual = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
        if actual != override["baseCommit"]:
            raise SystemExit(f"Unexpected {package} development base")
        for repository in development_sources(pins, package):
            patch = f'[patch."{repository}"]\n{package} = {{ path = "{source.as_posix()}" }}\n'
            pattern = r'(?m)^\[patch\."' + re.escape(repository) + r'"\]\n[^\[]*'
            config = re.sub(pattern, lambda _: patch + "\n", config) if re.search(pattern, config) else config.rstrip() + "\n\n" + patch
    config_path.write_text(config, encoding="utf-8")
    worker = checkout / "scripts/build-web-worker.sh"
    text = worker.read_text()
    old_worker_path = "  target/wasm32-unknown-unknown/release/ocs_web_worker.wasm"
    if old_worker_path in text:
        python = "python" if sys.platform == "win32" else "python3"
        target_query = ("worker_target_dir=$(cargo metadata --locked --offline --format-version 1 --no-deps | "
                        + python + " -c \"import json, sys; sys.stdout.write(json.load(sys.stdin)['target_directory'])\")")
        text = text.replace("set -eu", "set -eu\n\n" + target_query, 1)
        text = text.replace(old_worker_path, '  "${worker_target_dir}/wasm32-unknown-unknown/release/ocs_web_worker.wasm"')
        worker.write_text(text)
    elif '"${worker_target_dir}/wasm32-unknown-unknown/release/ocs_web_worker.wasm"' not in text:
        raise SystemExit("Unexpected parser-worker artifact path")
    if "kernelCommit" in pins:
        for name, expected in [("Cargo.toml", 2), ("crates/ocs_acis/Cargo.toml", 1)]:
            path = checkout / name
            text, count = re.subn(
                r'(kernel(?:-constraints)?\s*=\s*\{[^\n]*rev\s*=\s*")[^"]+("[^\n]*\})',
                lambda m: m[1] + pins["kernelCommit"] + m[2], path.read_text())
            if count != expected:
                raise SystemExit(f"Expected {expected} kernel dependencies in {name}")
            path.write_text(text)
    lock = INPUTS / "Cargo.lock"
    if args.record_lock:
        INPUTS.mkdir(exist_ok=True)
        lock.write_bytes((checkout / "Cargo.lock").read_bytes())
    else:
        (checkout / "Cargo.lock").write_bytes(lock.read_bytes())
    if args.record_build or args.verify_dependencies:
        metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--offline", "--format-version", "1", "--filter-platform", "wasm32-unknown-unknown"], cwd=checkout, text=True, encoding="utf-8"))
        validate_packages(metadata["packages"], pins)
    if args.record_build or args.verify_build:
        bundle = checkout / "dist/app"
        record_path = bundle / "IFCCAD-BUILD.json"
        files = sorted(p for p in bundle.rglob("*") if p.is_file() and p.suffix in {".wasm", ".js", ".html"})
        if not any(p.parent.name == "worker_pkg" and p.suffix == ".wasm" for p in files):
            raise SystemExit("Missing parser worker WASM")
        if not any(p.parent == bundle and p.suffix == ".wasm" for p in files):
            raise SystemExit("Missing main WASM")
        record = {"inputIdentity": identity(), "hostSourceIdentity": host_source_identity(checkout), "pins": pins, "files": {p.relative_to(bundle).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}}
        if args.record_build:
            record["toolchain"] = subprocess.check_output(["rustc", "-Vv"], text=True).strip()
            record_path.write_text(json.dumps(record, indent=2) + "\n")
        else:
            saved = json.loads(record_path.read_text())
            saved.pop("toolchain", None)
            if saved != record:
                raise SystemExit("Cached host bundle or dependency provenance differs")
    print(identity())


if __name__ == "__main__":
    main()
