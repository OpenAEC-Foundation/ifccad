"""Independently author a mixed spline and literal CBOR revision 8 record."""
import json
from pathlib import Path

from literal_cbor import decode, encode

root = Path(__file__).resolve().parent
points = [[0., 0.], [1., 2., -0.], [3., 4., 5.]]
knots = [0., 0., 1., 2., 2.]
fit_points = [points[0], points[-1]]

ifcx_root = root / "ifccad"
document = json.loads(
    (root.parents[1] / "tests/fixtures/ifccad/hello-cad.ifcx").read_text(encoding="utf-8")
)
entity = next(row for row in document["data"] if row["path"] == "/cad/d1/e1")
del entity["attributes"]["ifccad::geom::lineSegment"]
entity["attributes"]["ifccad::geom::spline"] = {
    "topology": "open",
    "definition": {
        "kind": "control",
        "basis": {"degree": 1, "knotKind": "full", "knots": knots, "points": points},
        "fit": {"points": fit_points, "tolerance": 0.},
    },
}

def write_json(path, value):
    path.write_bytes((json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode("utf-8"))

def index(path, entry):
    value = json.loads(path.read_text(encoding="utf-8"))
    cases = value if isinstance(value, list) else value["cases"]
    if all(case["file"] != entry["file"] for case in cases):
        cases.append(entry)
    write_json(path, value)

ifcx_name = "valid/spline-mixed.ifcx"
write_json(ifcx_root / ifcx_name, document)
index(ifcx_root / "cases.json", {"file": ifcx_name, "valid": True})

cbor_root = root / "ifccad-cbor"
binary = decode((cbor_root / "valid/line-signed-zero.ifccad.cbor").read_bytes())
assert binary[1] == 8
binary[2][5][5][0][1][2] = 20
binary[2][5][5][0][1][3] = {
    0: {0: {0: 0, 1: {0: {0: 1, 1: {0: 0, 1: knots}, 2: points},
                       1: {0: fit_points, 3: 0.}}}, 1: 0}
}
cbor_name = "valid/spline-mixed.ifccad.cbor"
(cbor_root / cbor_name).write_bytes(encode(binary))
diagnostic = cbor_name + ".diag"
(cbor_root / diagnostic).write_text(
    "Literal mixed spline, kind 20, with a compact point and explicit negative zero.\n"
    + repr(binary) + "\n", encoding="utf-8"
)
index(cbor_root / "cases.json", {"file": cbor_name, "valid": True, "diagnostic": diagnostic})
