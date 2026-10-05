use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn drawing() -> Value {
    let mut v: Value =
        serde_json::from_slice(include_bytes!("../examples/ifccad/hello-cad.ifcx")).unwrap();
    for n in v["data"].as_array_mut().unwrap() {
        if n["path"] == "/cad/d1" {
            n["attributes"]["ifccad::drawing"]["nextLinePatternId"] = json!(3);
            n["children"]["linePattern0"] = json!("/cad/d1/linePattern/0");
            n["children"]["linePattern2"] = json!("/cad/d1/linePattern/2");
        }
        if let Some(a) = n
            .get_mut("attributes")
            .and_then(|a| a.get_mut("ifccad::layer"))
            .and_then(|a| a.get_mut("appearance"))
            .and_then(Value::as_object_mut)
        {
            a.insert("linePattern".into(), json!("/cad/d1/linePattern/2"));
        }
    }
    v["data"].as_array_mut().unwrap().extend([
        json!({"path":"/cad/d1/linePattern/0","attributes":{"ifccad::linePattern":{"name":"Continuous","pattern":[]}}}),
        json!({"path":"/cad/d1/linePattern/2","attributes":{"ifccad::linePattern":{"name":"DashDot","description":"streep-punt","pattern":[6,-2,0,-2]}}}),
    ]);
    v
}
fn read(v: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(v).unwrap(), Default::default())
}
fn node<'a>(v: &'a mut Value, path: &str) -> &'a mut Value {
    v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == path)
        .unwrap()
}

#[test]
fn named_pattern_nodes_and_unused_definitions_survive_production_readback() {
    let source = read(&drawing()).unwrap();
    let bytes = encode_ifccad_document(source.document()).unwrap();
    let mut value: Value = serde_json::from_slice(bytes.bytes()).unwrap();
    assert_eq!(
        node(&mut value, "/cad/d1/linePattern/2")["attributes"]["ifccad::linePattern"]["pattern"],
        json!([6.0, -2.0, 0.0, -2.0])
    );
    assert_eq!(
        node(&mut value, "/cad/d1/layer/0")["attributes"]["ifccad::layer"]["appearance"]
            ["linePattern"],
        json!("/cad/d1/linePattern/2")
    );
    assert!(value["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["path"] == "/cad/d1/linePattern/0"));
    assert_eq!(read(&value).unwrap().document(), source.document());
}

#[test]
fn composition_replaces_a_complete_pattern_before_validation() {
    let mut v = drawing();
    v["data"].as_array_mut().unwrap().push(json!({"path":"/cad/d1/linePattern/2","attributes":{"ifccad::linePattern":{"name":"DashDot","pattern":[0.5,-0.25]}}}));
    let source = read(&v).unwrap();
    let mut back: Value =
        serde_json::from_slice(encode_ifccad_document(source.document()).unwrap().bytes()).unwrap();
    assert_eq!(
        node(&mut back, "/cad/d1/linePattern/2")["attributes"]["ifccad::linePattern"]["pattern"],
        json!([0.5, -0.25])
    );
    assert!(load_ifccad_bytes(
        &serde_json::to_vec(&v).unwrap(),
        IfccadReadOptions {
            composition_policy: IfccadCompositionPolicy::RejectConflicts
        }
    )
    .is_err());
}

#[test]
fn invalid_definitions_references_and_scales_fail() {
    for pattern in [
        json!([1]),
        json!([-1, 2]),
        json!([0, 0]),
        json!([1e308, 1e308]),
    ] {
        let mut v = drawing();
        node(&mut v, "/cad/d1/linePattern/2")["attributes"]["ifccad::linePattern"]["pattern"] =
            pattern;
        assert!(read(&v).is_err());
    }
    for name in ["Continuous", "ByLayer", "ByBlock", ""] {
        let mut v = drawing();
        node(&mut v, "/cad/d1/linePattern/2")["attributes"]["ifccad::linePattern"]["name"] =
            json!(name);
        assert!(read(&v).is_err());
    }
    for path in [
        "Continuous",
        "/cad/d2/linePattern/2",
        "/cad/d1/layer/0",
        "/cad/d1/linePattern/99",
    ] {
        let mut v = drawing();
        node(&mut v, "/cad/d1/layer/0")["attributes"]["ifccad::layer"]["appearance"]
            ["linePattern"] = json!(path);
        assert!(read(&v).is_err());
    }
    let mut v = drawing();
    node(&mut v, "/cad/d1")["attributes"]["ifccad::drawing"]["linePatternScale"] = json!(0);
    assert!(read(&v).is_err());
}

#[test]
fn definition_names_use_full_unicode_case_folding_and_ids_are_u64() {
    let mut d = read(&drawing()).unwrap().document().clone();
    d.id_counters.next_line_pattern_id = u64::MAX;
    d.line_patterns.push(IfccadLinePattern {
        id: IfccadLinePatternId(u64::MAX - 1),
        name: "Straße".into(),
        description: None,
        pattern: vec![],
    });
    let bytes = encode_ifccad_document(&d).unwrap();
    assert!(load_ifccad_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .document()
        .line_patterns
        .iter()
        .any(|p| p.id.0 == u64::MAX - 1));
    d.line_patterns.push(IfccadLinePattern {
        id: IfccadLinePatternId(42),
        name: "STRASSE".into(),
        description: None,
        pattern: vec![],
    });
    assert!(encode_ifccad_document(&d).is_err());
    d.line_patterns.pop();
    d.line_patterns.push(d.line_patterns[0].clone());
    assert!(encode_ifccad_document(&d).is_err());
}

#[test]
fn definitions_require_drawing_ownership_and_entities_keep_explicit_references() {
    let mut v = drawing();
    node(&mut v, "/cad/d1")["children"]
        .as_object_mut()
        .unwrap()
        .remove("linePattern2");
    assert!(read(&v).is_err());
    let source = read(&drawing()).unwrap();
    let mut d = source.document().clone();
    d.model.entities[0].appearance.line_pattern = IfccadMode::Explicit(IfccadLinePatternId(2));
    d.model.entities[0].line_pattern_scale = 0.125;
    assert_eq!(
        load_ifccad_bytes(
            encode_ifccad_document(&d).unwrap().bytes(),
            Default::default()
        )
        .unwrap()
        .document(),
        &d
    );
    for value in [0., -1., f64::NAN, f64::INFINITY] {
        d.model.entities[0].line_pattern_scale = value;
        assert!(encode_ifccad_document(&d).is_err());
    }
}

#[test]
fn dedicated_line_pattern_example_is_strictly_readable() {
    let source = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-line-patterns.ifcx"),
        Default::default(),
    )
    .unwrap();
    assert_eq!(source.document().line_pattern_scale, 2.);
    assert_eq!(source.document().line_patterns.len(), 3);
    assert_eq!(
        load_ifccad_bytes(
            encode_ifccad_document(source.document()).unwrap().bytes(),
            Default::default()
        )
        .unwrap()
        .document(),
        source.document()
    );
}
