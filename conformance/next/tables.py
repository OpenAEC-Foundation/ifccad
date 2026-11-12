"""Author independent IFCX and CBOR fixtures for the native table raster."""
import copy
import json
from pathlib import Path

from literal_cbor import decode, encode

ROOT = Path(__file__).resolve().parent
IFCX = ROOT / "ifccad"
CBOR = ROOT / "ifccad-cbor"


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def add_cases(path, cases, collection=None):
    manifest = json.loads(path.read_text(encoding="utf-8"))
    existing = manifest if collection is None else manifest[collection]
    known = {case["file"] for case in cases}
    existing[:] = [case for case in existing if case["file"] not in known] + cases
    write_json(path, manifest)


table = {
    "columns": [{"width": 10.0}, {"width": 20.0}],
    "rows": [{
        "height": 4.0,
        "cells": [
            {"contents": [{"kind": "value", "value": {"kind": "text", "value": "Title"}}]},
            {"contents": [{"kind": "value", "value": {"kind": "integer", "value": 7}}]},
        ],
    }],
    "mergedRanges": [{"topRow": 0, "leftColumn": 0, "bottomRow": 0, "rightColumn": 1}],
}
file = json.loads((IFCX / "valid-circle.ifcx").read_text(encoding="utf-8"))
node = next(node for node in file["data"] if node["path"] == "/cad/d1/e1000")
attrs = node["attributes"]
node["attributes"] = {
    "ifccad::entity": attrs["ifccad::entity"],
    "ifccad::geom::placement": {
        "origin": [0.0, 0.0, 0.0], "xAxis": [1.0, 0.0, 0.0], "yAxis": [0.0, 1.0, 0.0],
    },
    "ifccad::table": table,
}
write_json(IFCX / "valid/table-grid.ifcx", file)
invalid = copy.deepcopy(file)
invalid_node = next(node for node in invalid["data"] if node["path"] == "/cad/d1/e1000")
invalid_node["attributes"]["ifccad::table"]["mergedRanges"][0]["rightColumn"] = 2
write_json(IFCX / "invalid/table-merge-outside-grid.ifcx", invalid)
add_cases(IFCX / "cases.json", [
    {"file": "valid/table-grid.ifcx", "valid": True},
    {"file": "invalid/table-merge-outside-grid.ifcx", "valid": False},
])

envelope = decode((CBOR / "valid/line-xy.ifccad.cbor").read_bytes())
entity = envelope[2][5][5][0][1]
entity[2] = 21
body = {
    0: {0: [0.0, 0.0, 0.0], 1: [1.0, 0.0, 0.0], 2: [0.0, 1.0, 0.0]},
    1: {
        0: [{0: 10.0}, {0: 20.0}],
        1: [{0: 4.0, 1: [{0: [{0: 0, 1: {0: 0, 1: "Title"}}]}, {0: [{0: 0, 1: {0: 1, 1: 7}}]}]}],
        2: [{0: 0, 1: 0, 2: 0, 3: 1}],
    },
}
entity[3] = {0: body}
cases = []
for name, valid in [("valid/table-grid", True), ("invalid/table-merge-outside-grid", False)]:
    if not valid:
        body[1][2][0][3] = 2
    file = f"{name}.ifccad.cbor"
    (CBOR / file).write_bytes(encode(envelope))
    diag = file + ".diag"
    (CBOR / diag).write_text(repr(envelope) + "\n", encoding="utf-8")
    case = {"file": file, "valid": valid, "diagnostic": diag}
    if not valid:
        case["errorClass"] = "InvalidDocument"
    cases.append(case)
add_cases(CBOR / "cases.json", cases, "cases")
