use ifccad::{geometry_kernel::hatch::*, *};
#[path = "support/ifccad_hatch.rs"]
mod fixture;
#[test]
fn direct_definition_bits_and_presence_survive_production_storage() {
    let mut d = fixture::drawing();
    let line = HatchPatternDefinitionLine {
        angle: 0.7853981633974487,
        base_point: [89.80256121069161, -0.],
        offset_vector: [89.80256121069161, 89.80256121069145],
        dash_lengths: vec![2., -1., -0., 0.],
    };
    state(&mut d).classic_settings = Some(HatchClassicSettings {
        definition_lines: Some(vec![line.clone(), line.clone()]),
        ..Default::default()
    });
    for cbor in [false, true] {
        let encoded = if cbor {
            encode_ifccad_cbor_document(&d).unwrap().bytes().to_vec()
        } else {
            encode_ifccad_document(&d).unwrap().bytes().to_vec()
        };
        let mut back = if cbor {
            load_ifccad_cbor_bytes(&encoded, Default::default())
                .unwrap()
                .into_document()
        } else {
            load_ifccad_bytes(&encoded, Default::default())
                .unwrap()
                .into_document()
        };
        let c = state(&mut back).classic_settings.as_ref().unwrap();
        assert!(c.origin.is_none());
        for actual in c.definition_lines.as_ref().unwrap() {
            assert_eq!(actual, &line);
            assert_eq!(actual.base_point[1].to_bits(), (-0f64).to_bits());
            assert_eq!(actual.dash_lengths[2].to_bits(), (-0f64).to_bits());
        }
    }
}

#[test]
fn direct_definition_members_are_required_closed_and_not_nullable() {
    let bytes = encode_ifccad_document(&fixture::drawing()).unwrap();
    for line in [
        serde_json::json!({"basePoint":[0,0],"offsetVector":[1,0]}),
        serde_json::json!({"angle":0,"offsetVector":[1,0]}),
        serde_json::json!({"angle":0,"basePoint":[0,0]}),
        serde_json::json!({"angle":0,"basePoint":[0,0],"offsetVector":[1,0],"dashLengths":null}),
        serde_json::json!({"angle":0,"basePoint":[0,0],"offsetVector":[1,0],"dashes":[]}),
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
        let node = value["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::hatch").is_some())
            .unwrap();
        node["attributes"]["ifccad::hatch"]["classicSettings"] =
            serde_json::json!({"definitionLines":[line]});
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).is_err()
        );
    }
    let mut d = fixture::drawing();
    state(&mut d).classic_settings = Some(HatchClassicSettings {
        definition_lines: Some(vec![HatchPatternDefinitionLine {
            angle: 0.,
            base_point: [0., 0.],
            offset_vector: [1., 0.],
            dash_lengths: vec![],
        }]),
        ..Default::default()
    });
    let encoded = encode_ifccad_document(&d).unwrap();
    let text = String::from_utf8(encoded.bytes().to_vec()).unwrap();
    let duplicated = text.replacen("\"angle\": 0.0", "\"angle\": 0.0, \"angle\": 1.0", 1);
    assert_ne!(duplicated, text);
    assert!(load_ifccad_bytes(duplicated.as_bytes(), Default::default()).is_err());
}
fn state(d: &mut IfccadDocument) -> &mut IfccadHatch {
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h
}
#[test]
fn authored_empty_and_zero_presence_survives_both_native_encodings() {
    let mut d = fixture::drawing();
    state(&mut d).classic_settings = Some(HatchClassicSettings {
        preferred_fill: Some(HatchClassicFill::Solid),
        creation_type: Some(HatchCreationType::UserDefined),
        name: Some(String::new()),
        description: Some(String::new()),
        origin: Some([0., -0.]),
        rotation: 8. * std::f64::consts::PI,
        definition_lines: Some(vec![]),
        ..Default::default()
    });
    state(&mut d).boundary_construction = Some(HatchBoundaryConstruction {
        seed_points: vec![[20., -2.], [20., -2.]],
        pixel_size: 0.25,
    });
    for cbor in [false, true] {
        let back = if cbor {
            let bytes = encode_ifccad_cbor_document(&d).unwrap();
            load_ifccad_cbor_bytes(bytes.bytes(), Default::default())
                .unwrap()
                .into_document()
        } else {
            let bytes = encode_ifccad_document(&d).unwrap();
            load_ifccad_bytes(bytes.bytes(), Default::default())
                .unwrap()
                .into_document()
        };
        assert_eq!(back, d);
        let mut copy = back;
        assert_eq!(
            state(&mut copy)
                .classic_settings
                .as_ref()
                .unwrap()
                .origin
                .unwrap()[1]
                .to_bits(),
            (-0.0_f64).to_bits()
        );
    }
    #[cfg(feature = "ifccad-cbor-zstd")]
    {
        let encoded = encode_ifccad_cbor_zstd_document(&d, Default::default()).unwrap();
        assert_eq!(
            load_ifccad_cbor_zstd_bytes(encoded.bytes(), Default::default())
                .unwrap()
                .document(),
            &d
        );
    }
}
#[test]
fn closed_editing_wire_rejects_null_unknown_and_default_records() {
    let d = fixture::drawing();
    let bytes = encode_ifccad_document(&d).unwrap();
    for (key, bad) in [
        ("classicSettings", serde_json::json!(null)),
        ("classicSettings", serde_json::json!({})),
        (
            "classicSettings",
            serde_json::json!({"name":"","unknown":true}),
        ),
        ("classicSettings", serde_json::json!({"families":[]})),
        (
            "classicSettings",
            serde_json::json!({"definitionLines":null}),
        ),
        (
            "classicSettings",
            serde_json::json!({"definitionLines":[{"angle":0,"basePoint":[0,0],"offsetVector":[1,0],"unknown":1}]}),
        ),
        ("classicSettings", serde_json::json!({"origin":null})),
        ("boundaryConstruction", serde_json::json!(null)),
        (
            "boundaryConstruction",
            serde_json::json!({"seedPoints":[],"pixelSize":0}),
        ),
        ("boundaryConstruction", serde_json::json!({"pixelSize":-1})),
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
        let node = value["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::hatch").is_some())
            .unwrap();
        node["attributes"]["ifccad::hatch"][key] = bad;
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).is_err(),
            "{key}: {value}"
        );
    }
}
#[test]
fn editing_records_survive_fill_changes_and_whole_attribute_replacement() {
    let mut d = fixture::drawing();
    let c = HatchClassicSettings {
        definition_lines: Some(vec![]),
        ..Default::default()
    };
    state(&mut d).classic_settings = Some(c.clone());
    state(&mut d).boundary_construction = Some(HatchBoundaryConstruction {
        seed_points: vec![],
        pixel_size: 0.25,
    });
    for fill in [
        HatchFill::Solid,
        HatchFill::LinePattern(HatchLinePattern {
            name: None,
            description: None,
            origin: [0., 0.],
            rotation: 0.,
            scale: 1.,
            families: vec![HatchLineFamily {
                angle: 0.,
                base_point: [0., 0.],
                offset: [0., 1.],
                dashes: vec![],
            }],
        }),
        HatchFill::Gradient,
    ] {
        let h = state(&mut d);
        h.fill = fill;
        h.gradient = Some(HatchGradient {
            gradient_type: Some(HatchGradientType::Linear),
            colors: Some([IfccadColor::rgb(10, 20, 30), IfccadColor::rgb(40, 50, 60)]),
            rotation: 0.,
            shift: 0.,
            color_mode: HatchGradientColorMode::TwoColor,
            tint: 0.,
        });
        let encoded = encode_ifccad_document(&d).unwrap();
        let back = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
        assert_eq!(back.document(), &d);
        let binary = encode_ifccad_cbor_document(&d).unwrap();
        let back = load_ifccad_cbor_bytes(binary.bytes(), Default::default()).unwrap();
        assert_eq!(back.document(), &d);
    }
    let encoded = encode_ifccad_document(&d).unwrap();
    let mut graph: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let node = graph["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::hatch").is_some())
        .unwrap();
    let mut replacement = node["attributes"]["ifccad::hatch"].clone();
    replacement
        .as_object_mut()
        .unwrap()
        .remove("classicSettings");
    replacement
        .as_object_mut()
        .unwrap()
        .remove("boundaryConstruction");
    let later = serde_json::json!({"path":node["path"],"attributes":{"ifccad::hatch":replacement}});
    graph["data"].as_array_mut().unwrap().push(later);
    let back = load_ifccad_bytes(&serde_json::to_vec(&graph).unwrap(), Default::default()).unwrap();
    let mut doc = back.into_document();
    assert!(state(&mut doc).classic_settings.is_none());
    assert!(state(&mut doc).boundary_construction.is_none());
}
