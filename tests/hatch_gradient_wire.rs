use ocdraw::{geometry_kernel::hatch::*, ifccad::*, ocdraw::*};
#[path = "support/ifccad_hatch.rs"]
mod fixture;

fn state<C>(a: C, b: C) -> HatchGradient<C> {
    HatchGradient {
        gradient_type: Some(HatchGradientType::InverseSpherical),
        colors: Some([a, b]),
        rotation: 0.75,
        shift: 0.5,
        color_mode: HatchGradientColorMode::OneColor,
        tint: 0.8,
    }
}
fn ocdraw_drawing() -> OcdrawDocument {
    let mut d = load_ocdraw_bytes(include_bytes!(
        "../tests/fixtures/ocdraw/hello-hatch-solid.ocdraw.json"
    ))
    .unwrap()
    .into_document();
    d.hatch_entities[0].fill = HatchFill::Gradient;
    d.hatch_entities[0].gradient = Some(state(
        DrawingColor::rgb(220, 80, 30),
        DrawingColor::rgb(30, 60, 150),
    ));
    d
}
fn ifccad_drawing() -> IfccadDocument {
    let mut d = fixture::drawing();
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.fill = HatchFill::Gradient;
    h.gradient = Some(state(
        IfccadColor::rgb(220, 80, 30),
        IfccadColor::rgb(30, 60, 150),
    ));
    d
}

#[test]
fn both_native_routes_retain_active_parameters_and_partial_inactive_settings() {
    let mut o = ocdraw_drawing();
    let mut i = ifccad_drawing();
    for active in [true, false] {
        if !active {
            o.hatch_entities[0].fill = HatchFill::Solid;
            let g = o.hatch_entities[0].gradient.as_mut().unwrap();
            g.gradient_type = None;
            g.colors = None;
            let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind
            else {
                panic!()
            };
            h.fill = HatchFill::Solid;
            let g = h.gradient.as_mut().unwrap();
            g.gradient_type = None;
            g.colors = None;
        }
        let bytes = encode_ocdraw_document(&o).unwrap();
        assert_eq!(
            load_ocdraw_bytes(bytes.bytes()).unwrap().hatch_entities(),
            o.hatch_entities.as_slice()
        );
        let bytes = encode_ifccad_document(&i).unwrap();
        assert_eq!(
            load_ifccad_bytes(bytes.bytes(), Default::default())
                .unwrap()
                .document(),
            &i
        );
    }
}

#[test]
fn native_gradient_color_identity_keeps_full_u64_indices_and_named_metadata() {
    let mut o = ocdraw_drawing();
    o.hatch_entities[0]
        .gradient
        .as_mut()
        .unwrap()
        .colors
        .as_mut()
        .unwrap()[0] = DrawingColor::rgb(220, 80, 30)
        .with_indexed("Custom", u64::MAX)
        .unwrap()
        .with_named("Book", "Ink")
        .unwrap();
    let b = encode_ocdraw_document(&o).unwrap();
    assert_eq!(
        load_ocdraw_bytes(b.bytes()).unwrap().hatch_entities()[0].gradient,
        o.hatch_entities[0].gradient
    );
    let mut i = ifccad_drawing();
    let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.gradient.as_mut().unwrap().colors.as_mut().unwrap()[0] = IfccadColor::rgb(220, 80, 30)
        .with_indexed("Custom", u64::MAX)
        .unwrap()
        .with_named("Book", "Ink")
        .unwrap();
    let expected = h.gradient.clone();
    let b = encode_ifccad_document(&i).unwrap();
    let loaded = load_ifccad_bytes(b.bytes(), Default::default()).unwrap();
    let IfccadEntityKind::Hatch(h) = &loaded.document().model.entities[0]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!()
    };
    assert_eq!(h.gradient, expected);
}

#[test]
fn ocdraw_optional_state_column_keeps_absent_rows_and_entity_order() {
    let mut d = ocdraw_drawing();
    let mut solid = d.hatch_entities[0].clone();
    solid.id = d.next_entity_id;
    d.next_entity_id += 1;
    solid.fill = HatchFill::Solid;
    solid.gradient = None;
    let owner = d
        .scopes
        .iter_mut()
        .find(|s| s.entities.contains(&d.hatch_entities[0].id))
        .unwrap();
    owner.entities.push(solid.id);
    d.hatch_entities.push(solid);
    let bytes = encode_ocdraw_document(&d).unwrap();
    let v: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    assert!(v["streams"]["hatchStream"]["gradient"][1].is_null());
    assert_eq!(
        load_ocdraw_bytes(bytes.bytes()).unwrap().hatch_entities(),
        d.hatch_entities.as_slice()
    );
}

#[test]
fn closed_wire_states_reject_null_unknown_incomplete_and_invalid_values() {
    let o = ocdraw_drawing();
    let i = ifccad_drawing();
    for ifccad in [false, true] {
        let bytes = if ifccad {
            encode_ifccad_document(&i).unwrap().bytes().to_vec()
        } else {
            encode_ocdraw_document(&o).unwrap().bytes().to_vec()
        };
        for mutation in 0..10 {
            let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let g = if ifccad {
                &mut v["data"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::hatch").is_some())
                    .unwrap()["attributes"]["ifccad::hatch"]["gradient"]
            } else {
                &mut v["streams"]["hatchStream"]["gradient"][0]
            };
            match mutation {
                0 => *g = serde_json::Value::Null,
                1 => g["unknown"] = serde_json::json!(1),
                2 => g["type"] = serde_json::json!("unknown"),
                3 => {
                    g.as_object_mut().unwrap().remove("type");
                }
                4 => g["colors"] = serde_json::json!([{"rgb":[1,2,3]}]),
                5 => g["shift"] = serde_json::json!(1.1),
                6 => g["tint"] = serde_json::json!(-0.1),
                7 => g["rotation"] = serde_json::Value::Null,
                8 => g["colorMode"] = serde_json::json!("auto"),
                _ => g["colors"][0]["namedColor"] = serde_json::json!({"catalog":"","name":"bad"}),
            }
            let b = serde_json::to_vec(&v).unwrap();
            assert!(
                if ifccad {
                    load_ifccad_bytes(&b, Default::default()).is_err()
                } else {
                    load_ocdraw_bytes(&b).is_err()
                },
                "route {ifccad}, mutation {mutation}"
            );
        }
    }
}

#[test]
fn gradient_endpoints_reject_symbolic_foreground_in_authored_and_wire_data() {
    for active in [true, false] {
        let mut o = ocdraw_drawing();
        let mut i = ifccad_drawing();
        if !active {
            o.hatch_entities[0].fill = HatchFill::Solid;
            let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind
            else {
                panic!()
            };
            h.fill = HatchFill::Solid;
        }
        for ifccad in [false, true] {
            let bytes = if ifccad {
                encode_ifccad_document(&i).unwrap().bytes().to_vec()
            } else {
                encode_ocdraw_document(&o).unwrap().bytes().to_vec()
            };
            let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let g = if ifccad {
                &mut v["data"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::hatch").is_some())
                    .unwrap()["attributes"]["ifccad::hatch"]["gradient"]
            } else {
                &mut v["streams"]["hatchStream"]["gradient"][0]
            };
            g["colors"][0] = serde_json::json!({"kind":"Foreground"});
            let b = serde_json::to_vec(&v).unwrap();
            assert!(
                if ifccad {
                    load_ifccad_bytes(&b, Default::default()).is_err()
                } else {
                    load_ocdraw_bytes(&b).is_err()
                },
                "route {ifccad}, active {active}"
            );
        }
        o.hatch_entities[0]
            .gradient
            .as_mut()
            .unwrap()
            .colors
            .as_mut()
            .unwrap()[0] = DrawingColor::Foreground;
        assert!(encode_ocdraw_document(&o).is_err());
        let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        h.gradient.as_mut().unwrap().colors.as_mut().unwrap()[0] = IfccadColor::Foreground;
        assert!(encode_ifccad_document(&i).is_err());
    }
}

#[test]
fn builder_rejects_symbolic_endpoint_before_allocating_the_hatch() {
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("gradient", "mm")).unwrap();
    let pattern = builder
        .add_line_pattern(LinePatternDefinition {
            name: "Continuous".into(),
            description: None,
            pattern: vec![],
        })
        .unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            pattern,
        ))
        .unwrap();
    let mut definition = HatchEntityDefinition {
        association_enabled: false,
        scope_id: 0,
        layer_id: layer,
        visible: true,
        appearance: Default::default(),
        placement: Default::default(),
        loops: vec![OcdrawHatchLoop {
            provenance: None,
            text_island: false,
            boundary: HatchBoundary2::Circle {
                center: [0., 0.],
                radius: 1.,
            },
            source_entity_ids: vec![],
        }],
        area_rule: HatchAreaRule::Normal,
        join_tolerance: DEFAULT_HATCH_JOIN_TOLERANCE,
        fill: HatchFill::Gradient,
        gradient: Some(state(DrawingColor::Foreground, DrawingColor::rgb(1, 2, 3))),
    };
    assert!(builder.add_hatch(definition.clone()).is_err());
    definition
        .gradient
        .as_mut()
        .unwrap()
        .colors
        .as_mut()
        .unwrap()[0] = DrawingColor::rgb(3, 2, 1);
    builder.add_hatch(definition).unwrap();
    let document = builder.build_document().unwrap();
    assert_eq!(document.hatch_entities.len(), 1);
    assert_eq!(document.hatch_entities[0].id, 1);
}
