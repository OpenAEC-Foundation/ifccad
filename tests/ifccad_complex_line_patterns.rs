use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn drawing() -> Value {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("../tests/fixtures/ifccad/hello-cad.ifcx")).unwrap();
    let source = "/cad/d1/symbolSource/9007199254740993";
    for node in value["data"].as_array_mut().unwrap() {
        if node["path"] == "/cad/d1" {
            node["attributes"]["ifccad::drawing"]["nextSymbolSourceId"] =
                json!(9007199254740994u64);
            node["attributes"]["ifccad::drawing"]["nextLinePatternId"] = json!(2);
            node["children"]["sourceHigh"] = json!(source);
            node["children"]["sourceZero"] = json!("/cad/d1/symbolSource/0");
            node["children"]["pattern1"] = json!("/cad/d1/linePattern/1");
        }
    }
    value["data"].as_array_mut().unwrap().extend([
        json!({"path":source,"attributes":{"ifccad::symbolSource":{"kind":"shx","fileName":"missing.shx"}}}),
        json!({"path":"/cad/d1/symbolSource/0","attributes":{"ifccad::symbolSource":{"kind":"shx","fileName":"missing.shx"}}}),
        json!({"path":"/cad/d1/linePattern/1","attributes":{"ifccad::linePattern":{"name":"Shapes","pattern":[{"length":1,"marker":{"kind":"symbol","symbolSource":source,"shapeNumber":42}}]}}})
    ]);
    value
}

#[test]
fn source_zero_and_full_width_ids_roundtrip() {
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&drawing()).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_document(loaded.document()).unwrap();
    let reopened = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), reopened.document());
}

#[test]
fn marker_paths_are_local_and_role_checked() {
    for source in [
        "/cad/d2/symbolSource/0",
        "/cad/d1/layer/0",
        "/cad/d1/symbolSource/1",
    ] {
        let mut value = drawing();
        value["data"].as_array_mut().unwrap().last_mut().unwrap()["attributes"]
            ["ifccad::linePattern"]["pattern"][0]["marker"]["symbolSource"] = json!(source);
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).is_err()
        );
    }
}

#[test]
fn source_counter_survives_deletion() {
    let mut doc = load_ifccad_bytes(&serde_json::to_vec(&drawing()).unwrap(), Default::default())
        .unwrap()
        .into_document();
    doc.line_patterns.retain(|p| p.id.0 == 0);
    doc.symbol_sources.clear();
    let encoded = encode_ifccad_document(&doc).unwrap();
    let reopened = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(
        reopened.document().id_counters.next_symbol_source_id,
        9007199254740994
    );
}

#[test]
fn duplicate_filenames_are_not_identity_collisions() {
    let doc = load_ifccad_bytes(&serde_json::to_vec(&drawing()).unwrap(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(doc.symbol_sources.len(), 2);
    assert_eq!(doc.symbol_sources[0].id, IfccadSymbolSourceId(0));
    assert_eq!(
        doc.symbol_sources[1].id,
        IfccadSymbolSourceId(9007199254740993)
    );
    assert_eq!(doc.symbol_sources[0].source, doc.symbol_sources[1].source);
}

#[test]
fn whole_attribute_composition_replaces_pattern() {
    let mut value = drawing();
    value["data"].as_array_mut().unwrap().push(json!({"path":"/cad/d1/linePattern/1","attributes":{"ifccad::linePattern":{"name":"Replacement","pattern":[3,-1]}}}));
    let bytes = serde_json::to_vec(&value).unwrap();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    let p = loaded
        .document()
        .line_patterns
        .iter()
        .find(|p| p.id.0 == 1)
        .unwrap();
    assert_eq!(p.name, "Replacement");
    assert!(p.pattern.iter().all(|e| e.marker.is_none()));
    assert!(load_ifccad_bytes(
        &bytes,
        IfccadReadOptions {
            composition_policy: IfccadCompositionPolicy::RejectConflicts
        }
    )
    .is_err());
}

#[test]
fn source_requires_membership_and_closed_role() {
    let mut value = drawing();
    value["data"][0]["children"]
        .as_object_mut()
        .unwrap()
        .remove("sourceHigh");
    assert!(load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).is_err());
    let mut value = drawing();
    let source = value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/symbolSource/0")
        .unwrap();
    source["attributes"]["ifccad::textStyle"] = json!({"name":"Wrong","font":{}});
    assert!(load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).is_err());
}

#[test]
fn unknown_marker_values_and_nulls_are_rejected() {
    for (key, val) in [
        ("uri", json!("a.shx")),
        ("scale", json!(null)),
        ("scale", json!(0)),
        ("shapeNumber", json!(42.0)),
        ("offset", json!([1])),
        ("text", json!("wrong variant")),
    ] {
        let mut value = drawing();
        value["data"].as_array_mut().unwrap().last_mut().unwrap()["attributes"]
            ["ifccad::linePattern"]["pattern"][0]["marker"][key] = val;
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).is_err(),
            "{key}"
        );
    }
}

#[test]
fn text_markers_use_independent_style_role() {
    use ocdraw::{line_pattern::*, text::*};
    let mut doc = load_ifccad_bytes(&serde_json::to_vec(&drawing()).unwrap(), Default::default())
        .unwrap()
        .into_document();
    doc.text_styles.push(IfccadTextStyle {
        id: IfccadTextStyleId(0),
        name: "Requested face".into(),
        properties: TextStyleProperties::new(FontRequest::family("Missing face")),
    });
    doc.line_patterns
        .iter_mut()
        .find(|p| p.id.0 == 1)
        .unwrap()
        .pattern
        .push(LinePatternElement {
            length: 0.,
            marker: Some(IfccadLinePatternMarker::Text {
                text: "GAS α".into(),
                text_style_id: IfccadTextStyleId(0),
                transform: LinePatternTransform {
                    rotation: LinePatternRotation::Absolute(0.25),
                    scale: 0.1,
                    offset: [-1., 2.],
                },
            }),
        });
    let encoded = encode_ifccad_document(&doc).unwrap();
    let reopened = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(doc.line_patterns, reopened.document().line_patterns);
    let mut wire: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let p = wire["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/linePattern/1")
        .unwrap();
    p["attributes"]["ifccad::linePattern"]["pattern"][1]["marker"]["textStyle"] =
        json!("/cad/d1/symbolSource/0");
    assert!(load_ifccad_bytes(&serde_json::to_vec(&wire).unwrap(), Default::default()).is_err());
}
