use ifccad::*;
use serde_json::{json, Value};

// Catches native readers rejecting or silently dropping the newly supported sweep.
#[test]
fn extrusion_values_survive_strict_json_and_cbor_readback() {
    let cases = [
        ("point", json!({})),
        ("circle", json!({"radius": 2.0})),
        (
            "arc",
            json!({"radius": 2.0,"startParameter":0.0,"sweepParameter":1.0}),
        ),
        (
            "lineSegment",
            json!({"start":[1,2,3],"end":[4,6,8],"extrusionDirection":[0,0,1]}),
        ),
        (
            "planarPolyline",
            json!({"vertices":[[0,0],[2,0]],"widths":{"kind":"constant","width":0.25}}),
        ),
    ];
    for (kind, body) in cases {
        for h in [2.0_f64, -2.0, -0.0] {
            let mut graph: Value =
                serde_json::from_slice(include_bytes!("fixtures/ifccad/hello-cad.ifcx")).unwrap();
            let node = graph["data"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["attributes"].get("ifccad::geom::lineSegment").is_some())
                .unwrap();
            node["attributes"]
                .as_object_mut()
                .unwrap()
                .remove("ifccad::geom::lineSegment");
            let key = format!("ifccad::geom::{kind}");
            node["attributes"][&key] = body.clone();
            node["attributes"][&key]["thickness"] = json!(h);
            if kind != "lineSegment" {
                node["attributes"]["ifccad::geom::placement"] =
                    json!({"origin":[10,20,30],"xAxis":[1,0,0],"yAxis":[0,1,0]});
            }
            // Producer bounds from the original drawing cannot enclose the test geometry.
            for n in graph["data"].as_array_mut().unwrap() {
                for role in ["ifccad::layout", "ifccad::blockDefinition"] {
                    if let Some(v) = n["attributes"].get_mut(role).and_then(Value::as_object_mut) {
                        v.remove("bounds");
                        v.remove("boundsQuality");
                    }
                }
            }
            let input = serde_json::to_vec(&graph).unwrap();
            let d = load_ifccad_bytes(&input, Default::default())
                .unwrap_or_else(|e| panic!("{kind}/{h}: {e:?}"));
            let binary = encode_ifccad_cbor_document(d.document()).unwrap();
            let restored = load_ifccad_cbor_bytes(binary.bytes(), Default::default()).unwrap();
            let output = encode_ifccad_document(restored.document()).unwrap();
            let g: Value = serde_json::from_slice(output.bytes()).unwrap();
            let v = g["data"]
                .as_array()
                .unwrap()
                .iter()
                .find_map(|n| n["attributes"].get(&key))
                .unwrap();
            assert_eq!(
                v["thickness"].as_f64().unwrap().to_bits(),
                h.to_bits(),
                "{kind}"
            );
        }
    }
}
