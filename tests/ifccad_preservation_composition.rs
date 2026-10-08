use ocdraw::ifccad::*;
use serde_json::{json, Value};
#[path = "support/ifccad_preservation.rs"]
mod support;
fn wire() -> Value {
    serde_json::from_slice(
        encode_ifccad_document(&support::with_opaque())
            .unwrap()
            .bytes(),
    )
    .unwrap()
}
fn read(v: &Value, policy: IfccadCompositionPolicy) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(
        &serde_json::to_vec(v).unwrap(),
        IfccadReadOptions {
            composition_policy: policy,
        },
    )
}
#[test]
fn record_replacement_is_atomic_and_source_graph_stays_immutable() {
    let mut v = wire();
    let original = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::preservationRecord").is_some())
        .unwrap()
        .clone();
    let mut record = original["attributes"]["ifccad::preservationRecord"].clone();
    record["payload"]["bytes"] = json!("AQID");
    record["conditions"] = json!([{ "target":{"role":"record","path":original["path"]}, "predicate":"future","version":19,"baseline":"BAU=" }]);
    v["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":original["path"],"attributes":{"ifccad::preservationRecord":record}}));
    let loaded = read(&v, IfccadCompositionPolicy::LaterWins).unwrap();
    let r = &loaded.document().preservation.as_ref().unwrap().records[0];
    assert_eq!(r.payload.bytes, vec![1, 2, 3]);
    assert_eq!(r.conditions[0].baseline, vec![4, 5]);
    assert!(read(&v, IfccadCompositionPolicy::RejectConflicts).is_err());
    let (graph, mut edited) = loaded.into_parts();
    let source = graph.source_bytes().to_vec();
    edited.model.entities.reverse();
    encode_ifccad_document(&edited).unwrap();
    assert_eq!(graph.source_bytes(), source);
    v["data"].as_array_mut().unwrap().last_mut().unwrap()["attributes"]
        ["ifccad::preservationRecord"] = json!({"payload":{"bytes":"AQID"}});
    assert!(read(&v, IfccadCompositionPolicy::LaterWins).is_err());
}
#[test]
fn opaque_replacement_removes_old_optional_native_state() {
    let mut v = wire();
    let d = support::with_opaque();
    let e = d.model.entities.last().unwrap();
    let path = format!("/cad/d{}/e{}", d.drawing_id, e.id());
    let node = v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == path)
        .unwrap();
    node["attributes"]["ifccad::opaqueEntity"]["nativeLayer"] =
        json!(format!("/cad/d{}/layer/{}", d.drawing_id, d.layers[0].id));
    let record_path = format!(
        "/cad/d{}/preservation/r{}",
        d.drawing_id,
        e.as_opaque().unwrap().preservation_record_id.0
    );
    v["data"].as_array_mut().unwrap().push(json!({"path":path,"attributes":{"ifccad::opaqueEntity":{"visible":false,"preservationRecord":record_path}}}));
    let loaded = read(&v, IfccadCompositionPolicy::LaterWins).unwrap();
    let e = loaded
        .document()
        .model
        .entities
        .last()
        .unwrap()
        .as_opaque()
        .unwrap();
    assert_eq!(e.layer_id, None);
    assert!(!e.visible);
}
