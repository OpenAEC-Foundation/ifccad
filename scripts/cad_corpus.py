"""Fetch pinned public CAD samples or run explicit binary roundtrip qualification.

No network access during `run`, no automatic benchmark, no source exclusions.
Downloaded drawings and generated reports stay outside the tracked repository.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import urllib.request
from urllib.parse import urlparse


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def read_manifest(path):
    manifest = json.loads(path.read_text(encoding="utf-8"))
    if manifest.get("schemaVersion") != 1 or not manifest.get("files"):
        raise ValueError("expected a nonempty schemaVersion 1 manifest")
    seen = set()
    for item in manifest["files"]:
        key = item["id"]
        if not re.fullmatch(r"[a-zA-Z0-9_-]{1,80}", key) or key in seen:
            raise ValueError("invalid or duplicate corpus ID")
        seen.add(key)
        if not re.fullmatch(r"[a-f0-9]{64}", item["sha256"]):
            raise ValueError("invalid SHA-256")
        if type(item["bytes"]) is not int or not 0 < item["bytes"] <= 64 * 1024 * 1024:
            raise ValueError("sample must be between 1 byte and 64 MiB")
        if item["format"] not in ("dwg", "dxf"):
            raise ValueError("unsupported sample format")
        url = urlparse(item["url"])
        if url.scheme != "https" or url.netloc != "raw.githubusercontent.com":
            raise ValueError("samples require pinned raw.githubusercontent.com HTTPS URLs")
        parts = url.path.split("/")
        if len(parts) < 5 or not re.fullmatch(r"[a-f0-9]{40}", parts[3]):
            raise ValueError("sample URL must pin a full Git commit")
    return manifest


def sample_path(cache, item):
    return cache / (item["id"] + "." + item["format"])


def verify(data, item):
    if len(data) != item["bytes"] or sha256(data) != item["sha256"]:
        raise ValueError("sample size or SHA-256 mismatch: " + item["id"])
    if item["format"] == "dwg" and data[:6].decode("ascii") != item["version"]:
        raise ValueError("DWG version mismatch: " + item["id"])


def fetch(manifest, cache):
    cache.mkdir(parents=True, exist_ok=True)
    for item in manifest["files"]:
        target = sample_path(cache, item)
        if target.is_symlink():
            raise ValueError("sample cache entry is a symlink")
        if target.exists():
            verify(target.read_bytes(), item)
            continue
        with urllib.request.urlopen(item["url"], timeout=45) as response:
            if urlparse(response.url).scheme != "https":
                raise ValueError("sample redirected away from HTTPS")
            data = response.read(item["bytes"] + 1)
        verify(data, item)
        with target.open("xb") as stream:
            stream.write(data)
        print("Fetched " + item["id"], flush=True)


def result_row(item, exit_code, report):
    qualified = (exit_code == 0 and report.get("qualified") is True
                 and report.get("status") == "qualified-native-roundtrip"
                 and report.get("source", {}).get("sha256") == item["sha256"])
    return {"id": item["id"], "sourcePath": item["sourcePath"],
            "version": item["version"], "exitCode": exit_code,
            "qualified": qualified, "status": report.get("status", "missing-report"),
            "phase": report.get("phase"),
            "lossCounts": {p: report.get(p, {}).get("lossCount")
                           for p in ("import", "export", "reimport")},
            "error": report.get("error")}


def save_json(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    with temporary.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")
    temporary.replace(path)


def run(manifest, cache, binary, output, timeout):
    # Verify the entire local corpus before starting any child process.
    for item in manifest["files"]:
        path = sample_path(cache, item)
        if path.is_symlink():
            raise ValueError("sample cache entry is a symlink")
        verify(path.read_bytes(), item)
    binary = binary.resolve(strict=True)
    output.mkdir()  # A report directory is never reused or overwritten.
    summary = {"schemaVersion": 1, "complete": False,
               "universalLosslessnessClaim": False, "benchmark": False,
               "binarySha256": sha256(binary.read_bytes()),
               "manifest": manifest, "timeoutSeconds": timeout, "results": []}
    save_json(output / "summary.json", summary)
    for item in manifest["files"]:
        with (output / (item["id"] + ".log")).open("xb") as log:
            try:
                child = subprocess.run([str(binary), str(sample_path(cache, item).resolve()),
                                        str((output / item["id"]).resolve())],
                                       stdout=log, stderr=subprocess.STDOUT, timeout=timeout,
                                       check=False)
                code = child.returncode
                path = output / item["id"] / "report.json"
                try:
                    report = json.loads(path.read_text(encoding="utf-8"))
                except (OSError, ValueError):
                    report = {"status": "missing-or-invalid-report"}
            except subprocess.TimeoutExpired:
                code, report = None, {"status": "timeout"}
        summary["results"].append(result_row(item, code, report))
        save_json(output / "summary.json", summary)
        print(item["id"] + ": " + summary["results"][-1]["status"], flush=True)
    summary["complete"] = True
    summary["qualifiedCount"] = sum(row["qualified"] for row in summary["results"])
    save_json(output / "summary.json", summary)
    return 0 if summary["qualifiedCount"] == len(summary["results"]) else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("fetch", "run"))
    parser.add_argument("--manifest", type=Path, default=Path(__file__).resolve().parents[1]
                        / "tests/corpus/public-dwg-v1.json")
    parser.add_argument("--cache", type=Path, required=True)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--timeout", type=float, default=120)
    args = parser.parse_args()
    if not 0 < args.timeout < float("inf"):
        parser.error("timeout must be finite and positive")
    if args.action == "run" and (args.binary is None or args.output is None):
        parser.error("run needs --binary and --output")
    try:
        manifest = read_manifest(args.manifest)
        if args.action == "fetch":
            fetch(manifest, args.cache)
            return 0
        return run(manifest, args.cache, args.binary, args.output, args.timeout)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
