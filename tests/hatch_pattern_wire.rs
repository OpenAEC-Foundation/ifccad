use ocdraw::{geometry_kernel::hatch::*, ifccad::*, ocdraw::*};
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
fn ocdraw_pattern_strict_readback_preserves_phase_after_contour_edit() {
    let mut d = load_ocdraw_bytes(include_bytes!(
        "../examples/ocdraw/hello-hatch-solid.ocdraw.json"
    ))
    .unwrap()
    .into_document();
    d.hatch_entities[0].fill = pattern();
    let bytes = encode_ocdraw_document(&d).unwrap();
    assert_eq!(
        load_ocdraw_bytes(bytes.bytes()).unwrap().hatch_entities(),
        d.hatch_entities.as_slice()
    );
    d.hatch_entities[0].loops[0].boundary = HatchBoundary2::Circle {
        center: [10., 0.],
        radius: 3.,
    };
    recompute_ocdraw_document_bounds(&mut d).unwrap();
    let bytes = encode_ocdraw_document(&d).unwrap();
    assert_eq!(
        load_ocdraw_bytes(bytes.bytes()).unwrap().hatch_entities()[0].fill,
        pattern()
    );
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
    let mut o = load_ocdraw_bytes(include_bytes!(
        "../examples/ocdraw/hello-hatch-solid.ocdraw.json"
    ))
    .unwrap()
    .into_document();
    o.hatch_entities[0].fill = pattern();
    let mut i = fixture::drawing();
    let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.fill = pattern();
    for mutation in 0..6 {
        for ifccad in [false, true] {
            let bytes = if ifccad {
                encode_ifccad_document(&i).unwrap().bytes().to_vec()
            } else {
                encode_ocdraw_document(&o).unwrap().bytes().to_vec()
            };
            let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let fill = if ifccad {
                &mut v["data"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::hatch").is_some())
                    .unwrap()["attributes"]["ifccad::hatch"]["fill"]
            } else {
                &mut v["streams"]["hatchStream"]["fill"][0]
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
            assert!(if ifccad {
                load_ifccad_bytes(&bytes, Default::default()).is_err()
            } else {
                load_ocdraw_bytes(&bytes).is_err()
            });
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
