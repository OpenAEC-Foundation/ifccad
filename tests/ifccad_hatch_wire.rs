use ocdraw::ifccad::*;
#[path = "support/ifccad_hatch.rs"]
mod fixture;
use serde_json::{json, Value};
fn encoded() -> Value {
    serde_json::from_slice(encode_ifccad_document(&fixture::drawing()).unwrap().bytes()).unwrap()
}
#[test]
fn hatch_fragment_replaces_the_whole_value_instead_of_merging_loops() {
    let mut v = encoded();
    let h = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::hatch").is_some())
        .unwrap()
        .clone();
    let mut body = h["attributes"]["ifccad::hatch"].clone();
    let last = body["loops"][1].clone();
    body["loops"] = json!([last]);
    body["areaRule"] = json!("ignore");
    v["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":h["path"],"attributes":{"ifccad::hatch":body}}));
    let read = load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).unwrap();
    let e = read
        .document()
        .model
        .entities
        .iter()
        .find_map(|e| {
            if let Some(IfccadNativeEntity {
                kind: IfccadEntityKind::Hatch(h),
                ..
            }) = e.as_native()
            {
                Some(h)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(e.loops.len(), 1);
    assert_eq!(
        e.area_rule,
        ocdraw::geometry_kernel::hatch::HatchAreaRule::Ignore
    );
}
#[test]
fn nested_unknown_fields_null_sources_and_noncanonical_paths_are_rejected() {
    for mutation in 0..4 {
        let mut v = encoded();
        let h = v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::hatch").is_some())
            .unwrap();
        let body = &mut h["attributes"]["ifccad::hatch"];
        match mutation {
            0 => body["loops"][0]["boundary"]["secret"] = json!(1),
            1 => body["loops"][1]["source"] = Value::Null,
            2 => body["loops"][1]["source"] = json!("/cad/d1/e0001"),
            _ => body["fill"] = json!({"kind":"linePattern"}),
        }
        assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
    }
}
#[test]
fn hatch_reference_preserves_zero_ids_in_ifccad_domains() {
    let mut d = fixture::drawing();
    d.drawing_id = 0;
    for e in &mut d.model.entities {
        let e = e.as_native_mut().unwrap();
        if e.id == fixture::SOURCE {
            e.id = 0;
        }
        if let IfccadEntityKind::Hatch(h) = &mut e.kind {
            h.loops[1].source_entity_id = Some(0);
        }
    }
    let bytes = encode_ifccad_document(&d).unwrap();
    assert!(String::from_utf8_lossy(bytes.bytes()).contains("/cad/d0/e0"));
    assert_eq!(
        load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
}
