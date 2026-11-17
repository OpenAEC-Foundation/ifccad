"""Independently author external plot-style conformance cases."""

import json
from pathlib import Path

from literal_cbor import decode, encode

ROOT = Path(__file__).resolve().parent
NAME = "plot-styles"
SHA256 = "0123456789abcdef" * 4
SOURCE = "/cad/d1/plotStyleSource/1"


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


ifcx_dir = ROOT / "ifccad"
graph = json.loads((ifcx_dir / "valid-circle.ifcx").read_bytes())
nodes = {node["path"]: node for node in graph["data"]}
root = nodes["/cad/d1"]
root["attributes"]["ifccad::drawing"].update(plotStyleMode="named", nextPlotStyleSourceId=2)
root["children"]["plotStyleSource1"] = SOURCE
graph["data"].append({
    "path": SOURCE,
    "attributes": {"ifccad::plotStyleSource": {"kind": "stb", "fileName": "review.stb", "sha256": SHA256}},
})
layout = nodes["/cad/d1/layout/42"]["attributes"]["ifccad::layout"]
layout["plotStyleSource"] = SOURCE
nodes["/cad/d1/layer/4"]["attributes"]["ifccad::layer"]["plotStyle"] = {"kind": "Named", "name": "Thin"}
nodes["/cad/d1/e1000"]["attributes"]["ifccad::entity"]["plotStyle"] = {"kind": "ByBlock"}
ifcx_cases = ifcx_dir / "cases.json"
cases = json.loads(ifcx_cases.read_bytes())
for filename, valid in [("valid-plot-styles.ifcx", True), ("invalid-plot-style-source.ifcx", False)]:
    if not valid:
        layout["plotStyleSource"] = "/cad/d1/plotStyleSource/2"
    write_json(ifcx_dir / filename, graph)
    if not any(case["file"] == filename for case in cases):
        cases.append({"file": filename, "valid": valid})
write_json(ifcx_cases, cases)

cbor_dir = ROOT / "ifccad-cbor"
document = decode((cbor_dir / "valid/line-xy.ifccad.cbor").read_bytes())
drawing = document[2]
drawing[4] = 1
entity_id = drawing[5][5][0][1][0]
drawing[28] = {0: 2, 1: {1: {0: 1, 1: "review.stb", 2: SHA256}},
               2: {0: 1}, 3: {0: {0: 3, 1: "Thin"}}, 4: {entity_id: {0: 2}}}
manifest = cbor_dir / "cases.json"
cases = json.loads(manifest.read_bytes())
case = {"file": f"valid/{NAME}.ifccad.cbor", "valid": True,
        "diagnostic": f"valid/{NAME}.ifccad.cbor.diag"}
(cbor_dir / case["file"]).write_bytes(encode(document))
(cbor_dir / case["diagnostic"]).write_text(
    repr(document).replace("False", "false").replace("True", "true").replace("'", '"') + "\n",
    encoding="utf-8",
)
if not any(existing["file"] == case["file"] for existing in cases["cases"]):
    cases["cases"].append(case)
write_json(manifest, cases)
