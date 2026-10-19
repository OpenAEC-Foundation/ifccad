use ifccad::{encode_ifccad_document, load_ifccad_bytes, load_ifccad_xformop_bytes};
use serde_json::{json, Value};

const NATIVE: &[u8] = include_bytes!("../examples/experiments/ifcx-xformop/native-placement.ifcx");
const USD_URI: &str = "https://ifcx.dev/@openusd.org/usd@v1.ifcx";

fn matrix_example() -> Value {
    let mut value: Value = serde_json::from_slice(NATIVE).unwrap();
    value["imports"]
        .as_array_mut()
        .unwrap()
        .push(json!({"uri": USD_URI}));
    for node in value["data"].as_array_mut().unwrap() {
        let attrs = node["attributes"].as_object_mut().unwrap();
        if let Some(placement) = attrs.remove("ifccad::geom::placement") {
            let o = &placement["origin"];
            let x = &placement["xAxis"];
            let y = &placement["yAxis"];
            let component = |v: &Value, i: usize| v[i].as_f64().unwrap();
            let z: Vec<f64> = (0..3)
                .map(|i| {
                    let (j, k) = ((i + 1) % 3, (i + 2) % 3);
                    component(x, j) * component(y, k) - component(x, k) * component(y, j)
                })
                .collect();
            attrs.insert(
                "usd::xformop".into(),
                json!({"transform": [
                    [x[0], x[1], x[2], 0], [y[0], y[1], y[2], 0],
                    [z[0], z[1], z[2], 0], [o[0], o[1], o[2], 1]
                ]}),
            );
        }
    }
    value
}

#[test]
fn matrix_placements_project_to_the_unchanged_native_document() {
    let native = load_ifccad_bytes(NATIVE, Default::default()).unwrap();
    let bytes = serde_json::to_vec(&matrix_example()).unwrap();
    assert!(load_ifccad_bytes(&bytes, Default::default()).is_err());
    let matrix = load_ifccad_xformop_bytes(&bytes, Default::default()).unwrap();
    assert_eq!(matrix.document(), native.document());
    assert_eq!(matrix.graph().source_bytes(), bytes);
    let source_node = matrix.graph().composed_ifcx()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["path"] == "/cad/d1/e1")
        .unwrap();
    assert!(source_node["attributes"].get("usd::xformop").is_some());
    assert!(source_node["attributes"]
        .get("ifccad::geom::placement")
        .is_none());
    let encoded = encode_ifccad_document(matrix.document()).unwrap();
    let back = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(back.document(), native.document());
}

fn node<'a>(value: &'a mut Value, path: &str) -> &'a mut Value {
    value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["path"] == path)
        .unwrap()
}

fn rejected(value: Value, message: &str) {
    let error = load_ifccad_xformop_bytes(&serde_json::to_vec(&value).unwrap(), Default::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains(message), "{error}");
}

#[test]
fn all_supported_placed_geometry_families_use_shared_native_validation() {
    for (key, payload) in [
        ("ifccad::geom::point", json!({})),
        ("ifccad::geom::circle", json!({"radius": 2})),
        (
            "ifccad::geom::arc",
            json!({"radius": 2,"startParameter":0,"sweepParameter":1}),
        ),
        (
            "ifccad::geom::ellipse",
            json!({"semiMajorRadius":3,"semiMinorRadius":2}),
        ),
        (
            "ifccad::geom::ellipseArc",
            json!({"semiMajorRadius":3,"semiMinorRadius":2,"startParameter":0,"sweepParameter":1}),
        ),
    ] {
        let mut native: Value = serde_json::from_slice(NATIVE).unwrap();
        let attrs = node(&mut native, "/cad/d1/e1")["attributes"]
            .as_object_mut()
            .unwrap();
        attrs.remove("ifccad::geom::planarPolyline");
        attrs.insert(key.into(), payload.clone());
        let expected =
            load_ifccad_bytes(&serde_json::to_vec(&native).unwrap(), Default::default()).unwrap();
        let mut matrix = matrix_example();
        let attrs = node(&mut matrix, "/cad/d1/e1")["attributes"]
            .as_object_mut()
            .unwrap();
        attrs.remove("ifccad::geom::planarPolyline");
        attrs.insert(key.into(), payload);
        let actual =
            load_ifccad_xformop_bytes(&serde_json::to_vec(&matrix).unwrap(), Default::default())
                .unwrap();
        assert_eq!(actual.document(), expected.document(), "{key}");
    }
    let mut matrix = matrix_example();
    let attrs = node(&mut matrix, "/cad/d1/e1")["attributes"]
        .as_object_mut()
        .unwrap();
    attrs.remove("ifccad::geom::planarPolyline");
    attrs.insert("ifccad::geom::circle".into(), json!({"radius":-2}));
    rejected(matrix, "IFCCAD-GEOMETRY-002");
}

#[test]
fn malformed_affine_and_unrepresentable_frames_are_rejected_without_repairs() {
    for matrix in [
        json!([]),
        json!([[1, 0, 0, 0]]),
        json!([[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, null]]),
    ] {
        let mut value = matrix_example();
        node(&mut value, "/cad/d1/e1")["attributes"]["usd::xformop"]["transform"] = matrix;
        rejected(value, "invalid matrix value");
    }
    for (component, replacement, message) in [
        ((3, 3), 2., "affine"),
        ((0, 3), 1., "affine"),
        ((0, 1), 2., "unit tolerance"),
        ((1, 0), 0., "unit tolerance"),
        ((2, 2), -1., "Z row"),
    ] {
        let mut value = matrix_example();
        node(&mut value, "/cad/d1/e1")["attributes"]["usd::xformop"]["transform"][component.0]
            [component.1] = json!(replacement);
        rejected(value, message);
    }
    let mut value = matrix_example();
    node(&mut value, "/cad/d1/e1")["attributes"]["usd::xformop"]["transform"] = json!([
        [0.6, 0.8, 0, 0],
        [-0.8, 0.6, 0, 0],
        [0, 0, 1, 0],
        [0, 0, 0, 1]
    ]);
    // The rounded third row is plausible, but not the exact cross product of
    // these stored binary64 axes. The experiment must expose this boundary.
    rejected(value, "Z row");
}

#[test]
fn ambiguous_or_unsupported_cad_placements_are_not_silently_ignored() {
    let mut value = matrix_example();
    node(&mut value, "/cad/d1/e1")["attributes"]["ifccad::geom::placement"] = json!({
        "origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]
    });
    rejected(value, "both present");
    for path in [
        "/cad/d1/e3",
        "/cad/d1/e5",
        "/cad/d1/layout/1",
        "/cad/d1/block/1",
    ] {
        let mut value = matrix_example();
        node(&mut value, path)["attributes"]["usd::xformop"] = json!({"transform":[
            [1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]
        ]});
        rejected(value, "unsupported for this CAD role");
    }
}

#[test]
fn import_and_inline_schema_conflicts_are_checked() {
    let mut value = matrix_example();
    value["imports"]
        .as_array_mut()
        .unwrap()
        .retain(|item| item["uri"] != USD_URI);
    rejected(value, "missing reviewed USD schema import");
    let mut value = matrix_example();
    value["schemas"]["usd::xformop"] = json!({"value":{"dataType":"String"}});
    rejected(value, "conflicting inline");
    let mut value = matrix_example();
    node(&mut value, "/cad/d1/e1")["attributes"]["usd::xformop"]["extra"] = json!(1);
    rejected(value, "invalid matrix value");
}

#[test]
fn composition_precedes_mapping_and_foreign_nodes_keep_their_source() {
    let mut value = matrix_example();
    value["data"].as_array_mut().unwrap().push(json!({
        "path":"/cad/d1/e1", "attributes":{"usd::xformop":{"transform":[
            [0,1,0,0],[-1,0,0,0],[0,0,1,0],[101,200,10,1]
        ]}}
    }));
    value["data"].as_array_mut().unwrap().push(json!({
        "path":"foreign", "attributes":{
            "ifccad::geom::circle":{"radius":2},
            "usd::xformop":{"transform":"outside this CAD experiment"}
        }
    }));
    let matrix =
        load_ifccad_xformop_bytes(&serde_json::to_vec(&value).unwrap(), Default::default())
            .unwrap();
    let mut expected = load_ifccad_bytes(NATIVE, Default::default())
        .unwrap()
        .into_document();
    let ifccad::IfccadEntityKind::PlanarPolyline { placement, .. } =
        &mut expected.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!("expected path")
    };
    placement.origin[0] = 101.;
    assert_eq!(matrix.document(), &expected);
    assert!(matrix.graph().composed_ifcx()["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["path"] == "foreign"));
}

#[test]
fn native_files_still_load_through_the_explicit_experimental_route() {
    assert_eq!(
        load_ifccad_xformop_bytes(NATIVE, Default::default())
            .unwrap()
            .document(),
        load_ifccad_bytes(NATIVE, Default::default())
            .unwrap()
            .document()
    );
}

#[test]
fn authored_matrix_example_keeps_the_native_drawing_structure() {
    let bytes = include_bytes!("../examples/experiments/ifcx-xformop/standard-xformop.ifcx");
    let matrix = load_ifccad_xformop_bytes(bytes, Default::default()).unwrap();
    assert_eq!(
        matrix.document(),
        load_ifccad_bytes(NATIVE, Default::default())
            .unwrap()
            .document()
    );
}
