use ocdraw::{ifccad::*, text::*};
use serde_json::{json, Value};

#[test]
fn bundled_profile_uses_ifcx_array_item_description_syntax() {
    // IFC5-development/schema/ifcx.tsp: ArrayRestrictions.value is singular.
    // Inspect schema declarations independently of our native reader projection.
    fn check(v: &Value) {
        if v.get("dataType").and_then(Value::as_str) == Some("Array") {
            let r = v
                .get("arrayRestrictions")
                .and_then(Value::as_object)
                .expect("array restriction declaration");
            assert!(
                r.get("value").is_some_and(Value::is_object),
                "IFCX arrays require arrayRestrictions.value: {v}"
            );
            assert!(
                !r.contains_key("values"),
                "object restriction spelling cannot describe array items"
            );
        }
        match v {
            Value::Object(o) => o.values().for_each(check),
            Value::Array(a) => a.iter().for_each(check),
            _ => {}
        }
    }
    let profile: Value =
        serde_json::from_str(include_str!("../schemas/ifccad/ifccad-profile-0.1.0.ifcx")).unwrap();
    check(&profile["schemas"]);
}

fn graph() -> Value {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.text_styles.push(IfccadTextStyle {
        id: IfccadTextStyleId(0),
        name: "Unused".into(),
        properties: TextStyleProperties::new(FontRequest::family("Requested")),
    });
    serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap()
}
fn read(v: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(v).unwrap(), Default::default())
}
fn style(v: &mut Value) -> &mut Value {
    v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::textStyle").is_some())
        .unwrap()
}
#[test]
fn styles_require_local_role_membership_and_allocation_watermark() {
    let original = graph();
    assert!(read(&original).is_ok());
    let mut v = original.clone();
    v["data"][0]["attributes"]["ifccad::drawing"]
        .as_object_mut()
        .unwrap()
        .remove("nextTextStyleId");
    assert!(read(&v).is_err());
    let mut v = original.clone();
    v["data"][0]["children"]
        .as_object_mut()
        .unwrap()
        .remove("textStyle0");
    assert!(read(&v).is_err());
    let mut v = original.clone();
    style(&mut v)["path"] = json!("/cad/d2/textStyle/0");
    assert!(read(&v).is_err());
    let mut v = original.clone();
    style(&mut v)["attributes"]["ifccad::geom::circle"] = json!({"radius":1});
    assert!(read(&v).is_err());
    let mut v = original.clone();
    style(&mut v)["path"] = json!("/cad/d1/textStyle/00");
    assert!(read(&v).is_err());
}
#[test]
fn style_payload_replacement_preserves_source_and_removes_prior_overrides() {
    let mut v = graph();
    style(&mut v)["attributes"]["ifccad::textStyle"]["font"]["bold"] = json!(true);
    let replacement = json!({"path":"/cad/d1/textStyle/0","attributes":{"ifccad::textStyle":{"name":"Replacement","font":{"family":"Other"}}}});
    v["data"].as_array_mut().unwrap().push(replacement);
    let bytes = serde_json::to_vec(&v).unwrap();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    assert_eq!(loaded.graph().source_bytes(), bytes);
    assert_eq!(loaded.document().text_styles[0].name, "Replacement");
    assert_eq!(loaded.document().text_styles[0].properties.font.bold, None);
    assert!(load_ifccad_bytes(
        &bytes,
        IfccadReadOptions {
            composition_policy: IfccadCompositionPolicy::RejectConflicts
        }
    )
    .is_err());
}
#[test]
fn unknown_style_fields_and_explicit_nulls_are_rejected() {
    for key in ["font", "creationHeight", "vertical", "lastUsedHeight"] {
        let mut v = graph();
        style(&mut v)["attributes"]["ifccad::textStyle"][key] = Value::Null;
        assert!(read(&v).is_err(), "{key}");
    }
    let mut v = graph();
    style(&mut v)["attributes"]["ifccad::textStyle"]["unknown"] = json!(true);
    assert!(read(&v).is_err());
    let mut v = graph();
    style(&mut v)["attributes"]["ifccad::textStyle"]["font"]["unknown"] = json!(true);
    assert!(read(&v).is_err());
}
