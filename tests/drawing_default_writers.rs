use ifccad as ifc;
use serde_json::{json, Value};

fn ifccad() -> ifc::IfccadDocument {
    ifc::load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}

fn attr<'a>(v: &'a Value, key: &str) -> &'a Value {
    &v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get(key).is_some())
        .unwrap()["attributes"][key]
}

#[test]
fn ifccad_writer_omits_native_defaults() {
    let mut doc = ifccad();
    for e in &mut doc.model.entities {
        if let ifc::IfccadEntityKind::BlockInstance { transform, .. } =
            &mut e.as_native_mut().unwrap().kind
        {
            transform.rotation = 0.;
            transform.scale = [1.; 3];
        }
    }
    let encoded = ifc::encode_ifccad_document(&doc).unwrap();
    let v: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let e = attr(&v, "ifccad::entity");
    assert!(e.get("appearance").is_none());
    assert!(e.get("linePatternScale").is_none());
    assert!(attr(&v, "ifccad::drawing")
        .get("linePatternScale")
        .is_none());
    assert!(attr(&v, "ifccad::layer")["appearance"]
        .get("opacity")
        .is_none());
    let t = &attr(&v, "ifccad::blockInstance")["transform"];
    assert!(t.get("rotation").is_none());
    assert!(t.get("scale").is_none());
    assert!(t.get("placement").is_some());
    assert_eq!(
        attr(&v, "ifccad::geom::lineSegment")["start"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        ifc::load_ifccad_bytes(encoded.bytes(), Default::default())
            .unwrap()
            .document(),
        &doc
    );
}

#[test]
fn ifccad_writer_retains_nondefaults_and_signed_zero() {
    let mut doc = ifccad();
    let e = doc.model.entities[0].as_native_mut().unwrap();
    e.visible = false;
    e.appearance.opacity = ifc::IfccadMode::ByBlock;
    e.line_pattern_scale = 2.;
    let ifc::IfccadEntityKind::LineSegment { start, end } = &mut e.kind else {
        panic!()
    };
    start[2] = -0.;
    end[2] = 1e-100;
    doc.layers[0].appearance.opacity = 0.;
    let encoded = ifc::encode_ifccad_document(&doc).unwrap();
    let v: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(
        attr(&v, "ifccad::entity")["appearance"],
        json!({"opacity":{"mode":"ByBlock"}})
    );
    assert_eq!(
        attr(&v, "ifccad::layer")["appearance"]["opacity"],
        json!(0.)
    );
    let loaded = ifc::load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    let ifc::IfccadEntityKind::LineSegment { start, end } = &loaded.document().model.entities[0]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!()
    };
    assert_eq!(start[2].to_bits(), (-0.0_f64).to_bits());
    assert_eq!(end[2].to_bits(), (1e-100_f64).to_bits());
}

#[test]
fn block_rotation_negative_zero_survives_both_writers() {
    let mut doc = ifccad();
    let e = doc
        .model
        .entities
        .iter_mut()
        .find(|e| {
            matches!(
                e.as_native().unwrap().kind,
                ifc::IfccadEntityKind::BlockInstance { .. }
            )
        })
        .unwrap();
    let ifc::IfccadEntityKind::BlockInstance { transform, .. } =
        &mut e.as_native_mut().unwrap().kind
    else {
        panic!()
    };
    transform.rotation = -0.0;
    let encoded = ifc::encode_ifccad_document(&doc).unwrap();
    let loaded = ifc::load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    let e = loaded
        .document()
        .model
        .entities
        .iter()
        .find(|e| {
            matches!(
                e.as_native().unwrap().kind,
                ifc::IfccadEntityKind::BlockInstance { .. }
            )
        })
        .unwrap();
    let ifc::IfccadEntityKind::BlockInstance { transform, .. } = &e.as_native().unwrap().kind
    else {
        panic!()
    };
    assert_eq!(transform.rotation.to_bits(), (-0.0_f64).to_bits());
}
