"""Apply the explicitly pinned Explorer host dependencies and frozen lockfile."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
INPUTS = ROOT / "format-explorer/ocs-inputs"


def identity():
    paths = [ROOT / "format-explorer/ocs-source.json", Path(__file__), INPUTS / "Cargo.lock", ROOT / ".github/workflows/format-explorer.yml", ROOT / "rust-toolchain.toml"]
    digest = hashlib.sha256()
    for path in paths:
        digest.update(str(path.relative_to(ROOT)).encode())
        digest.update(path.read_bytes())
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
        if len(matches) != 1 or not (matches[0]["source"] or "").endswith("#" + revision):
            raise SystemExit(f"Unexpected resolved {name} dependency")


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
    manifests = ["Cargo.toml", "crates/ocs_save/Cargo.toml", "crates/ocs_web_worker/Cargo.toml", "crates/ocs_plugin_api/Cargo.toml"]
    for name in manifests:
        path = checkout / name
        text = path.read_text()
        pattern = r'(codec\s*=\s*\{[^\n]*package\s*=\s*"opencaddocument"[^\n]*git\s*=\s*")[^"]+("[^\n]*rev\s*=\s*")[^"]+("[^\n]*\})'
        text, count = re.subn(pattern, lambda m: m[1] + pins["documentRepository"] + m[2] + pins["documentCommit"] + m[3], text)
        expected = 2 if name == "crates/ocs_plugin_api/Cargo.toml" else 1
        if count != expected:
            raise SystemExit(f"Expected {expected} Document dependencies in {name}")
        path.write_text(text)
    if "kernelCommit" in pins:
        path = checkout / "Cargo.toml"
        text, count = re.subn(
            r'(kernel(?:-constraints)?\s*=\s*\{[^\n]*rev\s*=\s*")[^"]+("[^\n]*\})',
            lambda m: m[1] + pins["kernelCommit"] + m[2], path.read_text())
        if count != 2:
            raise SystemExit("Expected both kernel dependencies in Studio root")
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
        record = {"inputIdentity": identity(), "pins": pins, "files": {p.relative_to(bundle).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}}
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
