use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn wire() -> Value {
    let mut v: Value =
        serde_json::from_slice(include_bytes!("../examples/ifccad/hello-cad.ifcx")).unwrap();
    let attributes = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"]["ifccad::entity"].is_object())
        .unwrap()["attributes"]["ifccad::entity"]
        .clone();
    v["data"][0]["attributes"]["ifccad::drawing"]["nextEntityId"] = json!(1003);
    v["data"][0]["attributes"]["ifccad::drawing"]["nextLayoutId"] = json!(43);
    v["data"][0]["children"]["paper"] = json!("/cad/d1/layout/42");
    let payload = json!({
        "model":"/cad/d1/layout/1", "frame":{"center":[100,75],"width":160,"height":100},
        "view":{"center":[0,0],"target":[0,0,0],"direction":[0,0,100],"height":200,"twist":0,
                "projection":"Perspective","lensLengthMm":50,"frontClip":{"mode":"Disabled"},"backClip":{"mode":"Disabled"}},
        "renderMode":"Wireframe","viewEnabled":true,"viewLocked":false,"visible":true,
        "paperClip":{"enabled":true,"boundary":"/cad/d1/e1002"},"frozenLayers":[]
    });
    v["data"].as_array_mut().unwrap().extend([
        json!({"path":"/cad/d1/layout/42","children":{"0":"/cad/d1/e1000","1":"/cad/d1/e1002"},
            "attributes":{"ifccad::layout":{"kind":"Paper","name":"Sheet","tabIndex":1}}}),
        json!({"path":"/cad/d1/e1000","attributes":{"ifccad::entity":attributes.clone(),"ifccad::viewport":payload}}),
        json!({"path":"/cad/d1/e1002","attributes":{"ifccad::entity":attributes,"ifccad::geom::circle":{"radius":50},
            "ifccad::geom::placement":{"origin":[100,75,0],"xAxis":[1,0,0],"yAxis":[0,1,0]}}})
    ]);
    v
}
fn read(v: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(v).unwrap(), Default::default())
}
fn payload(v: &mut Value) -> &mut Value {
    &mut v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/e1000")
        .unwrap()["attributes"]["ifccad::viewport"]
}
#[test]
fn viewport_payload_roundtrips_through_production_reader() {
    let original = wire();
    let doc = read(&original).unwrap().into_document();
    let IfccadEntityKind::Viewport(view) = &doc.paper_layouts[0].entities[0].kind else {
        panic!()
    };
    assert_eq!(view.view.direction, [0., 0., 100.]);
    assert_eq!(view.paper_clip.boundary_entity_id, Some(1002));
    assert_eq!(
        doc.paper_layouts[0]
            .entities
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>(),
        [1000, 1002]
    );
    let encoded = encode_ifccad_document(&doc).unwrap();
    assert_eq!(
        load_ifccad_bytes(encoded.bytes(), Default::default())
            .unwrap()
            .document(),
        &doc
    );
}
#[test]
fn viewport_wire_rejects_unknown_null_and_conflicting_payloads() {
    assert!(read(&wire()).is_ok());
    for pointer in [
        "/extra",
        "/frame/extra",
        "/view/extra",
        "/view/frontClip/extra",
        "/paperClip/extra",
    ] {
        let mut v = wire();
        let parts: Vec<_> = pointer.split('/').skip(1).collect();
        let mut p = payload(&mut v);
        for part in &parts[..parts.len() - 1] {
            p = &mut p[*part];
        }
        p[parts[parts.len() - 1]] = json!(true);
        assert!(read(&v).is_err(), "{pointer}");
    }
    for pointer in [
        "/view/lensLengthMm",
        "/view/frontClip/distance",
        "/paperClip/boundary",
    ] {
        let mut v = wire();
        let parts: Vec<_> = pointer.split('/').skip(1).collect();
        let mut p = payload(&mut v);
        for part in &parts[..parts.len() - 1] {
            p = &mut p[*part];
        }
        p[parts[parts.len() - 1]] = Value::Null;
        assert!(read(&v).is_err(), "{pointer}");
    }
    for field in [
        "model",
        "frame",
        "view",
        "renderMode",
        "viewEnabled",
        "viewLocked",
        "visible",
        "paperClip",
        "frozenLayers",
    ] {
        let mut v = wire();
        payload(&mut v).as_object_mut().unwrap().remove(field);
        assert!(read(&v).is_err(), "{field}");
    }
    let mut v = wire();
    payload(&mut v)["model"] = json!("/cad/d2/layout/1");
    assert!(read(&v).is_err());
    let mut v = wire();
    let node = v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/e1000")
        .unwrap();
    node["attributes"]["ifccad::geom::placement"] =
        json!({"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]});
    assert!(read(&v).is_err());
}
#[test]
fn viewport_composes_as_complete_attribute() {
    let mut v = wire();
    let mut later = payload(&mut v).clone();
    later["view"]["projection"] = json!("Orthographic");
    later["view"]
        .as_object_mut()
        .unwrap()
        .remove("lensLengthMm");
    later["paperClip"] = json!({"enabled":false});
    v["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"/cad/d1/e1000","attributes":{"ifccad::viewport":later}}));
    let doc = read(&v).unwrap().into_document();
    let IfccadEntityKind::Viewport(view) = &doc.paper_layouts[0].entities[0].kind else {
        panic!()
    };
    assert_eq!(view.view.lens_length_mm, None);
    assert_eq!(view.paper_clip.boundary_entity_id, None);
    let options = IfccadReadOptions {
        composition_policy: IfccadCompositionPolicy::RejectConflicts,
    };
    assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), options).is_err());
}
