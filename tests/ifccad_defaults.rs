use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_slice(include_bytes!("fixtures/ifccad/hello-cad.ifcx")).unwrap()
}
fn attr_mut<'a>(source: &'a mut Value, key: &str) -> &'a mut Value {
    &mut source["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["attributes"].get(key).is_some())
        .unwrap()["attributes"][key]
}
fn read(source: &Value) -> IfccadDocument {
    load_ifccad_bytes(&serde_json::to_vec(source).unwrap(), Default::default())
        .unwrap()
        .into_document()
}
fn invalid(source: &Value) -> bool {
    load_ifccad_bytes(&serde_json::to_vec(source).unwrap(), Default::default()).is_err()
}

#[test]
fn line_xy_and_xyz_zero_have_equal_documents() {
    let mut expanded = fixture();
    attr_mut(&mut expanded, "ifccad::geom::lineSegment")["start"] = json!([10., 20., 0.]);
    let expected = read(&expanded);
    attr_mut(&mut expanded, "ifccad::geom::lineSegment")["start"] = json!([10., 20.]);
    assert_eq!(read(&expanded), expected);
}

#[test]
fn line_explicit_z_preserves_signed_zero_and_tiny_values() {
    for z in [-0.0_f64, 1e-100] {
        let mut source = fixture();
        attr_mut(&mut source, "ifccad::geom::lineSegment")["start"] = json!([10., 20., z]);
        let doc = read(&source);
        let IfccadEntityKind::LineSegment { start, .. } =
            &doc.model.entities[0].as_native().unwrap().kind
        else {
            panic!()
        };
        assert_eq!(start[2].to_bits(), z.to_bits());
    }
}

#[test]
fn native_appearance_defaults_are_independent() {
    for appearance in [None, Some(json!({}))] {
        let source = fixture();
        let expected = read(&source);
        let mut compact = source;
        let entity = attr_mut(&mut compact, "ifccad::entity")
            .as_object_mut()
            .unwrap();
        entity.remove("appearance");
        if let Some(appearance) = appearance {
            entity.insert("appearance".into(), appearance);
        }
        assert_eq!(read(&compact), expected);
    }
    let mut source = fixture();
    attr_mut(&mut source, "ifccad::entity")["appearance"] = json!({
        "color": {"mode":"Explicit","value":{"rgb":[255,0,0]}},
        "opacity": {"mode":"ByBlock"}
    });
    let doc = read(&source);
    let a = &doc.model.entities[0].as_native().unwrap().appearance;
    assert!(matches!(a.color, IfccadMode::Explicit(_)));
    assert!(matches!(a.opacity, IfccadMode::ByBlock));
    assert!(matches!(a.line_pattern, IfccadMode::ByLayer));
    assert!(matches!(a.line_weight, IfccadMode::ByLayer));
}

#[test]
fn block_transform_and_layer_defaults_expand() {
    let mut source = fixture();
    let t = &mut attr_mut(&mut source, "ifccad::blockInstance")["transform"];
    t["rotation"] = json!(0.);
    t["scale"] = json!([1., 1., 1.]);
    let expected = read(&source);
    let t = attr_mut(&mut source, "ifccad::blockInstance")["transform"]
        .as_object_mut()
        .unwrap();
    t.remove("rotation");
    t.remove("scale");
    attr_mut(&mut source, "ifccad::layer")["appearance"]
        .as_object_mut()
        .unwrap()
        .remove("opacity");
    assert_eq!(read(&source), expected);
}

#[test]
fn malformed_defaultable_fields_remain_invalid() {
    for bad in [
        Value::Null,
        json!([1.]),
        json!([1., 2., 3., 4.]),
        json!([1., "2"]),
    ] {
        let mut v = fixture();
        attr_mut(&mut v, "ifccad::geom::lineSegment")["start"] = bad;
        assert!(invalid(&v));
    }
    for bad in [
        Value::Null,
        json!({"extra":0}),
        json!({"color":{"mode":"Explicit"}}),
        json!({"opacity":{"mode":"ByLayer","value":1}}),
        json!({"opacity":null}),
    ] {
        let mut v = fixture();
        attr_mut(&mut v, "ifccad::entity")["appearance"] = bad;
        assert!(invalid(&v));
    }
    for (key, bad) in [
        ("rotation", Value::Null),
        ("scale", Value::Null),
        ("scale", json!([1., 1.])),
        ("scale", json!([0., 1., 1.])),
        ("placement", Value::Null),
    ] {
        let mut v = fixture();
        attr_mut(&mut v, "ifccad::blockInstance")["transform"][key] = bad;
        assert!(invalid(&v));
    }
    let mut v = fixture();
    attr_mut(&mut v, "ifccad::layer")["appearance"]["opacity"] = Value::Null;
    assert!(invalid(&v));
}

#[test]
fn later_entity_replacement_applies_defaults_after_composition() {
    let mut source = fixture();
    attr_mut(&mut source, "ifccad::entity")["appearance"]["color"] =
        json!({"mode":"Explicit","value":{"rgb":[255,0,0]}});
    source["data"].as_array_mut().unwrap().push(
        json!({"path":"/cad/d1/e1","attributes":{"ifccad::entity":{"layer":"/cad/d1/layer/0"}}}),
    );
    let bytes = serde_json::to_vec(&source).unwrap();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    assert!(matches!(
        loaded.document().model.entities[0]
            .as_native()
            .unwrap()
            .appearance
            .color,
        IfccadMode::ByLayer
    ));
    assert_eq!(loaded.graph().source_bytes(), bytes);
    assert!(load_ifccad_bytes(
        &bytes,
        IfccadReadOptions {
            composition_policy: IfccadCompositionPolicy::RejectConflicts
        }
    )
    .is_err());
}

#[test]
fn polyline_absence_means_open_and_preserves_dormant_bulge() {
    let mut source = fixture();
    let p = attr_mut(&mut source, "ifccad::geom::planarPolyline");
    p["closed"] = json!(false);
    let count = p["vertices"].as_array().unwrap().len();
    let mut bulges = vec![0.; count];
    *bulges.last_mut().unwrap() = 0.5;
    p["bulges"] = json!(bulges);
    let expected = read(&source);
    attr_mut(&mut source, "ifccad::geom::planarPolyline")
        .as_object_mut()
        .unwrap()
        .remove("closed");
    assert_eq!(read(&source), expected);
    attr_mut(&mut source, "ifccad::geom::planarPolyline")["closed"] = Value::Null;
    assert!(invalid(&source));
}

#[test]
fn compact_and_expanded_candidates_have_equal_native_semantics() {
    let expanded: Value = serde_json::from_slice(include_bytes!(
        "../conformance/next/ifccad/valid/defaults-expanded.ifcx"
    ))
    .unwrap();
    let compact: Value = serde_json::from_slice(include_bytes!(
        "../conformance/next/ifccad/valid/defaults-compact.ifcx"
    ))
    .unwrap();
    assert_eq!(read(&compact), read(&expanded));
}

#[test]
fn spatial_closure_defaults_without_defaulting_its_vertices() {
    let mut v = fixture();
    let node = v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| {
            n["attributes"]
                .get("ifccad::geom::planarPolyline")
                .is_some()
        })
        .unwrap();
    let attrs = node["attributes"].as_object_mut().unwrap();
    attrs.remove("ifccad::geom::planarPolyline");
    attrs.remove("ifccad::geom::placement");
    attrs.insert(
        "ifccad::geom::spatialPolyline".into(),
        json!({"vertices":[[0.,0.,0.],[1.,2.,3.]],"closed":false}),
    );
    let expected = read(&v);
    attr_mut(&mut v, "ifccad::geom::spatialPolyline")
        .as_object_mut()
        .unwrap()
        .remove("closed");
    assert_eq!(read(&v), expected);
    attr_mut(&mut v, "ifccad::geom::spatialPolyline")["vertices"][0] = json!([0., 0.]);
    assert!(invalid(&v));
}
