use ocdraw::ifcx_cad::*;
use serde_json::{json, Value};

fn wire() -> Value {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("../examples/ifcx-native-cad/hello-cad.ifcx"))
            .unwrap();
    for node in value["data"].as_array_mut().unwrap() {
        if node["attributes"]["ifccad::layout"]["kind"] == "Model" {
            node["attributes"]["ifccad::layout"]["tabIndex"] = json!(0);
        }
    }
    value
}
fn read(value: &Value) -> Result<ValidatedIfcxCad, IfcxCadReadError> {
    load_ifcx_cad_bytes(&serde_json::to_vec(value).unwrap(), Default::default())
}
fn papers() -> Value {
    let mut value = wire();
    value["data"][0]["attributes"]["ifccad::drawing"]["nextLayoutId"] = json!(91);
    for (id, tab, name) in [(90, 1, "First"), (3, 2, "Second")] {
        value["data"][0]["children"][format!("paper{id}")] = json!(format!("/cad/d1/layout/{id}"));
        value["data"].as_array_mut().unwrap().push(json!({
            "path": format!("/cad/d1/layout/{id}"), "children": {},
            "attributes": {"ifccad::layout": {"kind":"Paper", "name":name,
                "tabIndex":tab, "lengthUnit":"unitless"}}
        }));
    }
    value
}
#[test]
fn tab_index_is_required_without_legacy_synthesis() {
    let mut value = wire();
    for node in value["data"].as_array_mut().unwrap() {
        if let Some(layout) = node["attributes"].get_mut("ifccad::layout") {
            layout.as_object_mut().unwrap().remove("tabIndex");
        }
    }
    assert!(read(&value).is_err());
    for field in ["name", "lengthUnit", "paper"] {
        let mut value = wire();
        let model = value["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"]["ifccad::layout"]["kind"] == "Model")
            .unwrap();
        model["attributes"]["ifccad::layout"][field] = Value::Null;
        assert!(read(&value).is_err());
    }
}
#[test]
fn unsized_papers_follow_tabs_not_ids_or_json_order() {
    let mut value = papers();
    value["data"].as_array_mut().unwrap().reverse();
    let loaded = read(&value).unwrap();
    let doc = loaded.document();
    assert_eq!(
        doc.paper_layouts.iter().map(|p| p.id).collect::<Vec<_>>(),
        [90, 3]
    );
    let encoded = encode_ifcx_cad_document(doc).unwrap();
    assert_eq!(
        load_ifcx_cad_bytes(encoded.bytes(), Default::default())
            .unwrap()
            .document(),
        doc
    );
    let output: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    for node in output["data"].as_array().unwrap() {
        if node["attributes"]["ifccad::layout"]["kind"] == "Paper" {
            assert!(node["attributes"]["ifccad::layout"].get("paper").is_none());
        }
    }
}
#[test]
fn names_tabs_units_and_optional_media_are_strict() {
    let valid = papers();
    assert!(read(&valid).is_ok());
    for (field, invalid) in [
        ("tabIndex", json!(0)),
        ("tabIndex", json!(1)),
        ("tabIndex", json!(3)),
        ("name", json!("MODEL")),
        ("lengthUnit", json!("nonsense")),
        ("paper", json!({"width":0,"height":210,"lengthUnit":"mm"})),
        ("paper", json!({"width":297,"lengthUnit":"mm"})),
        (
            "paper",
            json!({"width":297,"height":210,"lengthUnit":"unitless"}),
        ),
        ("unknown", json!(1)),
        ("paper", Value::Null),
        (
            "paper",
            json!({"width":297,"height":210,"lengthUnit":"mm","unknown":1}),
        ),
    ] {
        let mut value = valid.clone();
        value["data"].as_array_mut().unwrap().last_mut().unwrap()["attributes"]["ifccad::layout"]
            [field] = invalid;
        assert!(read(&value).is_err(), "{field}");
    }
    let mut value = valid;
    let nodes = value["data"].as_array_mut().unwrap();
    let count = nodes.len();
    nodes[count - 2]["attributes"]["ifccad::layout"]["name"] = json!("Straße");
    nodes[count - 1]["attributes"]["ifccad::layout"]["name"] = json!("STRASSE");
    assert!(read(&value).is_err());
}

#[test]
fn editing_tab_order_preserves_units_identity_and_allocation() {
    let mut document = read(&papers()).unwrap().into_document();
    document.paper_layouts[0].length_unit = "in".into();
    document.paper_layouts[0].paper = Some(IfcxCadPaperSize {
        width: 297.,
        height: 210.,
        length_unit: "mm".into(),
    });
    let counters = document.id_counters;
    document.paper_layouts[0].tab_index = 2;
    document.paper_layouts[1].tab_index = 1;
    let encoded = encode_ifcx_cad_document(&document).unwrap();
    let loaded = load_ifcx_cad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document().id_counters, counters);
    assert_eq!(loaded.document().paper_layouts[1].id, 90);
    assert_eq!(loaded.document().paper_layouts[1].length_unit, "in");
    assert_eq!(
        loaded.document().paper_layouts[1]
            .paper
            .as_ref()
            .unwrap()
            .width,
        297.
    );
    for invalid in [0., -1., f64::INFINITY, f64::NAN] {
        document.paper_layouts[0].paper.as_mut().unwrap().width = invalid;
        assert!(validate_ifcx_cad_document(&document).is_err());
    }
    document.model.tab_index = 1;
    assert!(validate_ifcx_cad_document(&document).is_err());
}
