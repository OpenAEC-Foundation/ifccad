use ifccad::{text::*, *};
use serde_json::{json, Value};

#[test]
fn bundled_profile_uses_ifcx_array_item_description_syntax() {
    // IFC5-development/schema/ifcx.tsp: ArrayRestrictions.value is singular.
    // Inspect schema declarations independently of our native reader projection.
    fn check(v: &Value, path: &str) {
        if v.get("dataType").and_then(Value::as_str) == Some("Array") {
            if v.get("arrayRestrictions").is_none() {
                // IFCX item descriptions cannot express numeric-or-marker values.
                // Only this approved wrapper delegates its closed item union.
                assert_eq!(
                    path,
                    "/schemas/ifccad::linePattern/value/objectRestrictions/values/pattern"
                );
                let supplemental: Value = serde_json::from_str(include_str!(
                    "../schemas/ifccad/supplemental-values-0.1.0.schema.json"
                ))
                .unwrap();
                let choices = supplemental["$defs"]["linePatternElement"]["oneOf"]
                    .as_array()
                    .unwrap();
                assert_eq!(choices.len(), 2);
                assert_eq!(choices[0]["type"], "number");
                assert_eq!(choices[1]["type"], "object");
                assert_eq!(choices[1]["additionalProperties"], false);
                return;
            }
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
            Value::Object(o) => o
                .iter()
                .for_each(|(key, value)| check(value, &format!("{path}/{key}"))),
            Value::Array(a) => a
                .iter()
                .enumerate()
                .for_each(|(i, value)| check(value, &format!("{path}/{i}"))),
            _ => {}
        }
    }
    let profile: Value =
        serde_json::from_str(include_str!("../schemas/ifccad/ifccad-profile-0.1.0.ifcx")).unwrap();
    check(&profile["schemas"], "/schemas");
}

fn graph() -> Value {
    let mut d = load_ifccad_bytes(
        include_bytes!("../tests/fixtures/ifccad/hello-cad.ifcx"),
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
