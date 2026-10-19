"""Reproduce transport fixtures with Python 3.14's independent Zstd binding."""
import json
from pathlib import Path
from compression import zstd

ROOT = Path(__file__).resolve().parent
P = zstd.CompressionParameter
cases = []


def emit(name, data, *, payload=None, error=None):
    valid = error is None
    file = f"{'valid' if valid else 'invalid'}/{name}.ifccad.cbor.zst"
    path = ROOT / file
    path.parent.mkdir(exist_ok=True)
    path.write_bytes(data)
    case = {"file": file, "valid": valid}
    if payload:
        case["payload"] = payload
    if error:
        case["errorClass"] = error
    cases.append(case)


def frame(payload, level=3, checksum=True, size=True):
    return zstd.compress((ROOT / payload).read_bytes(), options={
        P.compression_level: level, P.checksum_flag: int(checksum),
        P.content_size_flag: int(size), P.dict_id_flag: 0, P.nb_workers: 0,
    })


empty = "../ifccad-cbor/valid/empty.ifccad.cbor"
signed = "../ifccad-cbor/valid/line-signed-zero.ifccad.cbor"
data = frame(empty)
emit("empty", data, payload=empty)
emit("signed-zero-level9", frame(signed, level=9), payload=signed)
emit("unknown-content-size", frame(empty, size=False), payload=empty)
emit("wrong-magic", b"\x50" + data[1:], error="Malformed")
emit("no-checksum", frame(empty, checksum=False), error="Malformed")
emit("truncated", data[:-1], error="Decompression")
emit("checksum-corrupt", data[:-1] + bytes([data[-1] ^ 128]), error="Decompression")
emit("concatenated", data + data, error="Malformed")
emit("trailing", data + b"\x00", error="Malformed")
emit("future-revision", frame("../ifccad-cbor/invalid/future-revision.ifccad.cbor"),
     error="Cbor.UnsupportedRevision")
(ROOT / "cases.json").write_text(json.dumps({
    "transport": "IFCCAD CBOR Zstd experimental 1", "mappingRevision": 2,
    "generator": "Python 3.14 compression.zstd; libzstd 1.5.7", "cases": cases,
}, indent=2) + "\n", encoding="utf-8")
