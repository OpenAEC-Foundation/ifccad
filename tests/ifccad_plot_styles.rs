use ifccad::{
    encode_ifccad_cbor_document, encode_ifccad_document, load_ifccad_bytes, load_ifccad_cbor_bytes,
};
use serde_json::{json, Value};

const SOURCE: &str = "/cad/d1/plotStyleSource/1";

fn node<'a>(file: &'a mut Value, path: &str) -> &'a mut Value {
    file["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["path"] == path)
        .unwrap()
}

fn named_drawing() -> Value {
    let mut file: Value =
        serde_json::from_str(include_str!("../conformance/next/ifccad/valid-circle.ifcx")).unwrap();
    let root = node(&mut file, "/cad/d1");
    root["attributes"]["ifccad::drawing"]["plotStyleMode"] = json!("named");
    root["attributes"]["ifccad::drawing"]["nextPlotStyleSourceId"] = json!(2);
    root["children"]["plotStyleSource1"] = json!(SOURCE);
    file["data"].as_array_mut().unwrap().push(json!({
        "path":SOURCE,
        "attributes":{"ifccad::plotStyleSource":{
            "kind":"stb","fileName":"review.stb",
            "sha256":"00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
        }}
    }));
    node(&mut file, "/cad/d1/layout/42")["attributes"]["ifccad::layout"]["plotStyleSource"] =
        json!(SOURCE);
    node(&mut file, "/cad/d1/layer/4")["attributes"]["ifccad::layer"]["plotStyle"] =
        json!({"kind":"Named","name":"Thin"});
    node(&mut file, "/cad/d1/e1000")["attributes"]["ifccad::entity"]["plotStyle"] =
        json!({"kind":"Normal"});
    node(&mut file, "/cad/d1/e1001")["attributes"]["ifccad::entity"]["plotStyle"] =
        json!({"kind":"ByBlock"});
    file
}

#[test]
fn named_plot_intent_survives_native_json_and_cbor_without_the_external_file() {
    let original = named_drawing();
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&original).unwrap(), Default::default()).unwrap();
    let cbor = encode_ifccad_cbor_document(loaded.document()).unwrap();
    let loaded_cbor = load_ifccad_cbor_bytes(cbor.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), loaded_cbor.document());
    let json = encode_ifccad_document(loaded_cbor.document()).unwrap();
    let mut returned: Value = serde_json::from_slice(json.bytes()).unwrap();
    for (path, attribute, field, value) in [
        (
            "/cad/d1/layout/42",
            "ifccad::layout",
            "plotStyleSource",
            json!(SOURCE),
        ),
        (
            "/cad/d1/layer/4",
            "ifccad::layer",
            "plotStyle",
            json!({"kind":"Named","name":"Thin"}),
        ),
        (
            "/cad/d1/e1000",
            "ifccad::entity",
            "plotStyle",
            json!({"kind":"Normal"}),
        ),
        (
            "/cad/d1/e1001",
            "ifccad::entity",
            "plotStyle",
            json!({"kind":"ByBlock"}),
        ),
    ] {
        assert_eq!(
            node(&mut returned, path)["attributes"][attribute][field],
            value
        );
    }
}

#[test]
fn rejects_invalid_plot_sources_and_assignments() {
    for (path, attribute, field, value) in [
        (
            SOURCE,
            "ifccad::plotStyleSource",
            "fileName",
            json!("../review.stb"),
        ),
        (
            SOURCE,
            "ifccad::plotStyleSource",
            "sha256",
            json!("INVALID"),
        ),
        (SOURCE, "ifccad::plotStyleSource", "sha256", Value::Null),
        (
            "/cad/d1",
            "ifccad::drawing",
            "nextPlotStyleSourceId",
            json!(1),
        ),
        (
            "/cad/d1",
            "ifccad::drawing",
            "plotStyleMode",
            json!("colorDependent"),
        ),
        (
            "/cad/d1/layout/42",
            "ifccad::layout",
            "plotStyleSource",
            json!("/cad/d1/plotStyleSource/2"),
        ),
        (
            "/cad/d1/layout/42",
            "ifccad::layout",
            "plotStyleSource",
            json!("/cad/d2/plotStyleSource/1"),
        ),
        (
            "/cad/d1/layer/4",
            "ifccad::layer",
            "plotStyle",
            json!({"kind":"ByBlock"}),
        ),
        (
            "/cad/d1/e1000",
            "ifccad::entity",
            "plotStyle",
            json!({"kind":"Named","name":"nOrMaL"}),
        ),
    ] {
        let mut file = named_drawing();
        node(&mut file, path)["attributes"][attribute][field] = value;
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&file).unwrap(), Default::default()).is_err(),
            "{path}/{attribute}/{field}",
        );
    }
}
