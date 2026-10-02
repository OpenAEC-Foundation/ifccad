mod common;
use common::*;
use ifcx_cad_convert::*;
use ocdraw::ifcx_cad::*;

fn patterns() -> IfcxCadDocument {
    let mut d = primitives();
    d.id_counters.next_line_pattern_id = 9;
    d.line_patterns.extend([
        IfcxCadLinePattern {
            id: IfcxCadLinePatternId(7),
            name: "EigenStreepPunt".into(),
            description: Some("authored".into()),
            pattern: vec![0.5, -0.25, 0., -0.25],
        },
        IfcxCadLinePattern {
            id: IfcxCadLinePatternId(8),
            name: "UnusedSolid".into(),
            description: None,
            pattern: vec![],
        },
    ]);
    d.layers[0].appearance.line_pattern = IfcxCadLinePatternId(7);
    d.model.entities[0].appearance.line_pattern = IfcxCadMode::Explicit(IfcxCadLinePatternId(7));
    d.line_pattern_scale = 2.;
    d.model.entities[0].line_pattern_scale = 0.5;
    let IfcxCadEntityKind::PlanarPolyline {
        line_pattern_generation,
        ..
    } = &mut d.model.entities[1].kind
    else {
        panic!()
    };
    *line_pattern_generation = IfcxCadLinePatternGeneration::Continuous;
    d
}

#[test]
fn ordinary_patterns_scales_and_generation_convert_without_losses() {
    let d = patterns();
    let first = to_cad(&validated(&d)).unwrap();
    assert!(first.diagnostics().is_empty());
    let lt = first.document().line_types.get("EigenStreepPunt").unwrap();
    assert_eq!(
        lt.elements.iter().map(|e| e.length).collect::<Vec<_>>(),
        vec![0.5, -0.25, 0., -0.25]
    );
    assert_eq!(first.document().header.linetype_scale, 2.);
    let h = first.mappings().entities.cad_handle(90).unwrap();
    assert_eq!(
        first
            .document()
            .get_entity(h)
            .unwrap()
            .common()
            .linetype_scale,
        0.5
    );
    assert_eq!(
        first
            .document()
            .get_entity(h)
            .unwrap()
            .common()
            .linetype_handle,
        Some(lt.handle)
    );
    let back = from_cad(first.document(), metadata()).unwrap();
    assert!(back
        .validated_ifcx()
        .document()
        .line_patterns
        .iter()
        .any(|p| p.name == "UnusedSolid" && p.pattern.is_empty()));
    assert_eq!(back.validated_ifcx().document().line_pattern_scale, 2.);
    assert_eq!(
        back.validated_ifcx().document().model.entities[0].line_pattern_scale,
        0.5
    );
    assert!(matches!(
        back.validated_ifcx().document().model.entities[1].kind,
        IfcxCadEntityKind::PlanarPolyline {
            line_pattern_generation: IfcxCadLinePatternGeneration::Continuous,
            ..
        }
    ));
}

#[test]
fn complex_patterns_fallback_once_including_unused_with_stable_references() {
    for shape in [false, true] {
        let mut c = to_cad(&validated(&patterns())).unwrap().into_document();
        let complex = cadcodec::tables::LineTypeComplexData {
            content: if shape {
                cadcodec::tables::LineTypeComplexContent::Shape { shape_number: 1 }
            } else {
                cadcodec::tables::LineTypeComplexContent::Text { text: "GAS".into() }
            },
            style_handle: c.text_styles.get("Standard").unwrap().handle,
            ..Default::default()
        };
        c.line_types.get_mut("EigenStreepPunt").unwrap().elements[0].complex =
            Some(complex.clone());
        let unused = c.line_types.get_mut("UnusedSolid").unwrap();
        unused.elements = vec![cadcodec::tables::LineTypeElement {
            length: 0.,
            complex: Some(complex),
        }];
        let first = c.clone();
        let out = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
        assert_eq!(c, first);
        let d = out.validated_ifcx().document();
        for name in ["EigenStreepPunt", "UnusedSolid"] {
            let p = d.line_patterns.iter().find(|p| p.name == name).unwrap();
            assert!(p.pattern.is_empty());
            assert_eq!(
                out.diagnostics()
                    .iter()
                    .filter(|i| i.code == "line-pattern-complex"
                        && i.location == format!("linePattern/{name}"))
                    .count(),
                1
            );
        }
        assert!(matches!(
            from_cad(&c, metadata()),
            Err(IfcxCadConversionError::Unsupported(_))
        ));
        let back = ifcx_cad_to_cad_document(out.validated_ifcx()).unwrap();
        assert!(back
            .document()
            .line_types
            .get("EigenStreepPunt")
            .unwrap()
            .elements
            .is_empty());
        assert_eq!(
            back.document()
                .model_space_entities()
                .next()
                .unwrap()
                .common()
                .linetype,
            "EigenStreepPunt"
        );
    }
}

#[test]
fn missing_or_conflicting_source_targets_and_invalid_values_always_fail() {
    for field in 0..6 {
        let mut c = to_cad(&validated(&patterns())).unwrap().into_document();
        let h = c.block_records.get("*Model_Space").unwrap().entity_handles[0];
        match field {
            0 => c.get_entity_mut(h).unwrap().common_mut().linetype = "missing".into(),
            1 => {
                c.get_entity_mut(h).unwrap().common_mut().linetype_handle =
                    Some(c.line_types.get("Continuous").unwrap().handle)
            }
            2 => c.get_entity_mut(h).unwrap().common_mut().linetype_scale = 0.,
            3 => c.header.linetype_scale = f64::NAN,
            4 => c.line_types.get_mut("EigenStreepPunt").unwrap().alignment = 'S',
            _ => c.line_types.get_mut("EigenStreepPunt").unwrap().elements[0].length = f64::NAN,
        }
        for policy in [IfcxCadLossPolicy::Allow, IfcxCadLossPolicy::Reject] {
            assert!(cad_document_to_ifcx_cad_with_options(
                &c,
                metadata(),
                IfcxCadConversionOptions {
                    loss_policy: policy
                }
            )
            .is_err());
        }
    }
}

#[test]
fn simple_and_named_empty_patterns_survive_real_dxf_and_dwg() {
    let first = to_cad(&validated(&patterns())).unwrap();
    for dwg in [false, true] {
        let c = if dwg {
            let bytes = cadcodec::DwgWriter::write_to_vec(first.document()).unwrap();
            cadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes))
                .read()
                .unwrap()
        } else {
            let bytes = cadcodec::DxfWriter::new(first.document())
                .write_to_vec()
                .unwrap();
            cadcodec::DxfReader::from_reader(std::io::Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap()
        };
        let back = from_cad(&c, metadata()).unwrap();
        let d = back.validated_ifcx().document();
        assert_eq!(
            d.line_patterns
                .iter()
                .find(|p| p.name == "EigenStreepPunt")
                .unwrap()
                .pattern,
            vec![0.5, -0.25, 0., -0.25]
        );
        assert!(d
            .line_patterns
            .iter()
            .find(|p| p.name == "UnusedSolid")
            .unwrap()
            .pattern
            .is_empty());
        assert_eq!(d.line_pattern_scale, 2.);
        assert_eq!(d.model.entities[0].line_pattern_scale, 0.5);
        assert!(matches!(
            d.model.entities[1].kind,
            IfcxCadEntityKind::PlanarPolyline {
                line_pattern_generation: IfcxCadLinePatternGeneration::Continuous,
                ..
            }
        ));
    }
}

#[test]
fn missing_continuous_is_not_native_synthesis_but_diagnosed_target_scaffolding() {
    let mut d = patterns();
    d.line_patterns.retain(|p| p.name != "Continuous");
    for l in &mut d.layers {
        l.appearance.line_pattern = IfcxCadLinePatternId(7);
    }
    let native = validated(&d);
    assert!(!native
        .document()
        .line_patterns
        .iter()
        .any(|p| p.name == "Continuous"));
    assert!(matches!(
        to_cad(&native),
        Err(IfcxCadConversionError::Unsupported(_))
    ));
    let output = ifcx_cad_to_cad_document(&native).unwrap();
    assert_eq!(
        output
            .diagnostics()
            .iter()
            .filter(|d| d.code == "line-pattern-scaffold")
            .count(),
        1
    );
    assert!(output
        .document()
        .line_types
        .iter()
        .any(|p| p.handle == output.document().header.continuous_linetype_handle));
    for dwg in [false, true] {
        let c = if dwg {
            cadcodec::DwgReader::from_stream(std::io::Cursor::new(
                cadcodec::DwgWriter::write_to_vec(output.document()).unwrap(),
            ))
            .read()
            .unwrap()
        } else {
            cadcodec::DxfReader::from_reader(std::io::Cursor::new(
                cadcodec::DxfWriter::new(output.document())
                    .write_to_vec()
                    .unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap()
        };
        from_cad(&c, metadata()).unwrap();
    }
}

#[test]
fn optional_native_defaults_are_lossless_but_large_integer_pattern_values_are_not() {
    let mut raw: serde_json::Value =
        serde_json::from_slice(&write_native_cad_ifcx(&patterns()).unwrap()).unwrap();
    for n in raw["data"].as_array_mut().unwrap() {
        for key in ["ifccad::drawing", "ifccad::entity"] {
            if let Some(a) = n["attributes"]
                .get_mut(key)
                .and_then(serde_json::Value::as_object_mut)
            {
                a.remove("linePatternScale");
            }
        }
        if let Some(a) = n["attributes"]
            .get_mut("ifccad::geom::planarPolyline")
            .and_then(serde_json::Value::as_object_mut)
        {
            a.remove("linePatternGeneration");
        }
    }
    let bytes = serde_json::to_vec(&raw).unwrap();
    let native = read_native_cad_ifcx(&bytes).unwrap();
    assert!(to_cad(&native).unwrap().diagnostics().is_empty());
    for target in ["ifccad::drawing", "ifccad::entity", "ifccad::linePattern"] {
        let mut changed = raw.clone();
        let n = changed["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get(target).is_some())
            .unwrap();
        if target == "ifccad::linePattern" {
            // Select a non-reserved definition so the core can validate it.
            n["attributes"][target]["name"] = serde_json::json!("Huge");
            n["attributes"][target]["pattern"] = serde_json::json!([9_007_199_254_740_993_u64, -1]);
        } else {
            n["attributes"][target]["linePatternScale"] =
                serde_json::json!(9_007_199_254_740_993_u64);
        }
        let native = read_native_cad_ifcx(&serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(ifcx_cad_to_cad_document(&native).is_err(), "{target}");
    }
}

#[test]
fn native_unicode_names_and_target_lookup_collisions_are_checked() {
    let mut d = patterns();
    d.line_patterns[1].name = "I".into();
    d.line_patterns[2].name = "ı".into();
    let native = validated(&d);
    assert!(matches!(
        ifcx_cad_to_cad_document(&native),
        Err(IfcxCadConversionError::InvalidStructure(_))
    ));
}

#[test]
fn declared_period_normalization_is_policy_controlled_and_nonfinite_complex_values_are_fatal() {
    let mut c = to_cad(&validated(&patterns())).unwrap().into_document();
    c.line_types
        .get_mut("EigenStreepPunt")
        .unwrap()
        .pattern_length = 99.;
    let output = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
    assert_eq!(
        output
            .diagnostics()
            .iter()
            .filter(|d| d.code == "line-pattern-period")
            .count(),
        1
    );
    assert!(matches!(
        from_cad(&c, metadata()),
        Err(IfcxCadConversionError::Unsupported(_))
    ));
    c.line_types.get_mut("EigenStreepPunt").unwrap().elements[0].complex =
        Some(cadcodec::tables::LineTypeComplexData {
            rotation: f64::NAN,
            ..Default::default()
        });
    assert!(matches!(
        cad_document_to_ifcx_cad(&c, metadata()),
        Err(IfcxCadConversionError::InvalidStructure(_))
    ));
}

#[test]
fn shared_nested_definitions_keep_pattern_modes_per_occurrence() {
    let mut d = nested([0.; 3]);
    d.line_patterns = patterns().line_patterns;
    d.id_counters.next_line_pattern_id = 9;
    d.layers[0].appearance.line_pattern = IfcxCadLinePatternId(7);
    d.model.entities[0].appearance.line_pattern = IfcxCadMode::Explicit(IfcxCadLinePatternId(7));
    d.model.entities[1].appearance.line_pattern = IfcxCadMode::Explicit(IfcxCadLinePatternId(8));
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Inner")
        .unwrap()
        .entities[0]
        .appearance
        .line_pattern = IfcxCadMode::ByBlock;
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Outer")
        .unwrap()
        .entities[0]
        .appearance
        .line_pattern = IfcxCadMode::ByLayer;
    let c = to_cad(&validated(&d)).unwrap();
    let back = from_cad(c.document(), metadata()).unwrap();
    let restored = back.validated_ifcx().document();
    for (index, name) in ["EigenStreepPunt", "UnusedSolid"].into_iter().enumerate() {
        let IfcxCadMode::Explicit(id) = restored.model.entities[index].appearance.line_pattern
        else {
            panic!()
        };
        assert_eq!(
            restored
                .line_patterns
                .iter()
                .find(|p| p.id == id)
                .unwrap()
                .name,
            name
        );
    }
    assert_eq!(
        restored
            .blocks
            .iter()
            .find(|b| b.name == "Inner")
            .unwrap()
            .entities[0]
            .appearance
            .line_pattern,
        IfcxCadMode::ByBlock
    );
    assert_eq!(
        restored
            .blocks
            .iter()
            .find(|b| b.name == "Outer")
            .unwrap()
            .entities[0]
            .appearance
            .line_pattern,
        IfcxCadMode::ByLayer
    );
}
