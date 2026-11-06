use serde_json::Value;
const SOURCE: &[u8] = include_bytes!(
    "../../../conformance/next/ifccad/valid/attributes-ownership-and-strong-link.ifcx"
);
#[test]
fn attribute_inspection_retains_child_owner_order_and_explicit_links() {
    let r = viewer::document_open_ifccad("attributes.ifcx", SOURCE);
    assert!(r["failure"].is_null(), "{r}");
    let a = &r["presentation"]["blockAttributes"];
    assert_eq!(a["values"].as_array().unwrap().len(), 1);
    let row = &a["values"][0];
    assert_eq!(row["ownerPath"], "/cad/d1/e9007199254740992");
    assert_eq!(row["order"], 0);
    assert_eq!(row["value"]["tag"], "RENAMED");
    assert!(row["value"]["definition"].is_string());
    let m = &r["preview"]["viewerSelection"];
    let child = &m["attributeOwners"][0];
    assert_eq!(child["path"], "/cad/d1/e9007199254740993");
    assert_eq!(child["ownerPath"], row["ownerPath"]);
    let parent = m["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["path"] == row["ownerPath"])
        .unwrap();
    assert_eq!(child["handle"], parent["handle"]);
    assert!(!m["entities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["path"] == child["path"]));
}
#[test]
fn fields_and_active_empty_mtext_are_inspected_without_evaluation() {
    for (source, field) in [
        (
            include_bytes!("../../../conformance/next/ifccad/valid/attributes-field-graph.ifcx")
                .as_slice(),
            "fields",
        ),
        (
            include_bytes!(
                "../../../conformance/next/ifccad/valid/attributes-empty-active-mtext.ifcx"
            )
            .as_slice(),
            "mText",
        ),
    ] {
        let r = viewer::inspect_ifccad_bytes("attributes.ifcx", source);
        let value = &r["presentation"]["blockAttributes"]["values"][0]["value"];
        assert_ne!(value[field], Value::Null);
    }
}
