use ifccad::{geometry_kernel::hatch::*, *};
#[path = "support/ifccad_hatch.rs"]
mod fixture;
fn pattern() -> HatchFill {
    HatchFill::LinePattern(HatchLinePattern {
        name: Some("Literal pattern".into()),
        description: Some("".into()),
        origin: [2., 3.],
        rotation: 0.25,
        scale: 2.,
        families: vec![HatchLineFamily {
            angle: 0.5,
            base_point: [1., -2.],
            offset: [3., -4.],
            dashes: vec![
                HatchDash::Gap { length: 2. },
                HatchDash::Dot,
                HatchDash::Dash { length: 1. },
            ],
        }],
    })
}

#[test]
fn ifccad_pattern_strict_readback_keeps_source_and_whole_attribute() {
    let mut d = fixture::drawing();
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.fill = pattern();
    let bytes = encode_ifccad_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
}

#[test]
fn unknown_null_dot_length_and_missing_families_are_rejected_in_both_formats() {
    let mut i = fixture::drawing();
    let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.fill = pattern();
    for mutation in 0..6 {
        {
            let _ifccad = true;
            let bytes = { encode_ifccad_document(&i).unwrap().bytes().to_vec() };
            let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let fill = {
                &mut v["data"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::hatch").is_some())
                    .unwrap()["attributes"]["ifccad::hatch"]["fill"]
            };
            match mutation {
                0 => fill["unknown"] = serde_json::json!(1),
                1 => fill["name"] = serde_json::Value::Null,
                2 => fill["families"][0]["dashes"][1]["length"] = serde_json::json!(1.),
                3 => {
                    fill.as_object_mut().unwrap().remove("families");
                }
                4 => fill["families"][0]["offset"][1] = serde_json::json!(0.),
                _ => fill["families"][0]["dashes"] = serde_json::json!([{"kind":"dot"}]),
            }
            let bytes = serde_json::to_vec(&v).unwrap();
            assert!({ load_ifccad_bytes(&bytes, Default::default()).is_err() });
        }
    }
}

#[test]
fn absent_transform_defaults_are_distinct_from_forbidden_nulls() {
    let mut d = fixture::drawing();
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.fill = pattern();
    let bytes = encode_ifccad_document(&d).unwrap();
    let mut v: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    let fill = &mut v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::hatch").is_some())
        .unwrap()["attributes"]["ifccad::hatch"]["fill"];
    for key in ["origin", "rotation", "scale"] {
        fill.as_object_mut().unwrap().remove(key);
    }
    let loaded = load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).unwrap();
    let IfccadEntityKind::Hatch(h) = &loaded.document().model.entities[0]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!()
    };
    let HatchFill::LinePattern(p) = &h.fill else {
        panic!()
    };
    assert_eq!(p.origin, [0., 0.]);
    assert_eq!(p.rotation, 0.);
    assert_eq!(p.scale, 1.);
}
