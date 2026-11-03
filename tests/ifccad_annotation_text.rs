use ifccad::*;
fn drawing() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/text-rich.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}
#[test]
fn text_snapshots_are_independent_and_preserved_in_both_encodings() {
    let mut d = drawing();
    d.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(1),
        name: "1:50".into(),
        paper_units: 1.,
        drawing_units: 50.,
        unit_scale: false,
    });
    d.id_counters.next_annotation_scale_id = 2;
    let key = IfccadAnnotationContextKey {
        scale_id: IfccadAnnotationScaleId(1),
    };
    let mut found = false;
    for e in &mut d.model.entities {
        if let IfccadEntity::Native(IfccadNativeEntity {
            kind: IfccadEntityKind::Text(t),
            ..
        }) = e
        {
            t.annotation = Some(Box::new(IfccadAnnotation {
                orientation: None,
                mode: IfccadAnnotationMode::Disabled,
                reference: Some(key),
                contexts: vec![IfccadAnnotationContext {
                    key,
                    presentation: Some(IfccadTextAnnotationContext {
                        insertion: [101., -22.],
                        alignment: [105., -21.],
                        rotation: 0.3,
                        horizontal: IfccadTextAnnotationHorizontal::Fit,
                    }),
                }],
            }));
            found = true;
        }
    }
    assert!(found);
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let json = encode_ifccad_document(&d).unwrap();
    let cbor = encode_ifccad_cbor_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(json.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    assert_eq!(
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    let mut invalid = d.clone();
    for e in &mut invalid.model.entities {
        if let IfccadEntity::Native(IfccadNativeEntity {
            kind: IfccadEntityKind::Text(t),
            ..
        }) = e
        {
            t.annotation.as_mut().unwrap().mode = IfccadAnnotationMode::ByDefinition;
        }
    }
    assert!(validate_ifccad_document(&invalid).is_err());
}
#[test]
fn mtext_snapshots_preserve_z_nonunit_direction_and_dormant_column_heights() {
    let mut d = drawing();
    d.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(1),
        name: "Saved".into(),
        paper_units: 2.,
        drawing_units: 100.,
        unit_scale: false,
    });
    d.id_counters.next_annotation_scale_id = 2;
    let key = IfccadAnnotationContextKey {
        scale_id: IfccadAnnotationScaleId(1),
    };
    let mut found = false;
    for e in &mut d.model.entities {
        if let IfccadEntity::Native(IfccadNativeEntity {
            kind: IfccadEntityKind::MText(t),
            ..
        }) = e
        {
            t.annotation = Some(Box::new(IfccadAnnotation {
                orientation: None,
                mode: IfccadAnnotationMode::Enabled,
                reference: None,
                contexts: vec![IfccadAnnotationContext {
                    key,
                    presentation: Some(IfccadMTextAnnotationContext {
                        insertion: [2., 3., 19.],
                        x_direction: [0., 2., 1.],
                        attachment: ifccad::text::MTextAttachment::BottomRight,
                        rectangle_width: 0.,
                        rectangle_height: 17.,
                        columns: Some(IfccadMTextAnnotationColumns {
                            kind: IfccadAnnotationColumnKind::Static,
                            count: 2,
                            width: 0.,
                            gutter: 0.,
                            auto_height: true,
                            flow_reversed: true,
                            heights: vec![1., 2., 3.],
                        }),
                    }),
                }],
            }));
            found = true;
        }
    }
    assert!(found);
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let json = encode_ifccad_document(&d).unwrap();
    let cbor = encode_ifccad_cbor_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(json.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    assert_eq!(
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    let source: serde_json::Value = serde_json::from_slice(json.bytes()).unwrap();
    let i = source["data"]
        .as_array()
        .unwrap()
        .iter()
        .position(|n| n["attributes"].get("ifccad::annotation").is_some())
        .unwrap();
    for field in ["reference", "contexts"] {
        let mut bad = source.clone();
        bad["data"][i]["attributes"]["ifccad::annotation"][field] = serde_json::Value::Null;
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&bad).unwrap(), Default::default()).is_err(),
            "{field}"
        );
    }
    let mut bad = source.clone();
    bad["data"][i]["attributes"]["ifccad::annotation"]["contexts"][0]["key"]["scaleId"] =
        serde_json::json!("/cad/d2/annotationScale/1");
    assert!(load_ifccad_bytes(&serde_json::to_vec(&bad).unwrap(), Default::default()).is_err());
    for e in &mut d.model.entities {
        if let IfccadEntity::Native(IfccadNativeEntity {
            kind: IfccadEntityKind::MText(t),
            ..
        }) = e
        {
            let c = t.annotation.as_mut().unwrap().contexts[0]
                .presentation
                .as_mut()
                .unwrap();
            c.x_direction = [0.; 3];
        }
    }
    assert!(validate_ifccad_document(&d).is_err());
}

#[test]
fn saved_context_positions_are_proven_even_when_glyph_enclosure_is_unassessed() {
    let mut d = drawing();
    d.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(1),
        name: "Saved".into(),
        paper_units: 1.,
        drawing_units: 20.,
        unit_scale: false,
    });
    d.id_counters.next_annotation_scale_id = 2;
    for e in &mut d.model.entities {
        if let IfccadEntity::Native(IfccadNativeEntity {
            kind: IfccadEntityKind::Text(t),
            ..
        }) = e
        {
            t.annotation = Some(Box::new(IfccadAnnotation {
                orientation: None,
                mode: IfccadAnnotationMode::Disabled,
                reference: None,
                contexts: vec![IfccadAnnotationContext {
                    key: IfccadAnnotationContextKey {
                        scale_id: IfccadAnnotationScaleId(1),
                    },
                    presentation: Some(IfccadTextAnnotationContext {
                        insertion: [1e8, 2.],
                        alignment: [1e8 + 100., 3.],
                        rotation: 0.,
                        horizontal: IfccadTextAnnotationHorizontal::Left,
                    }),
                }],
            }));
        }
    }
    let scope =
        &assess_ifccad_document_bounds(&d).unwrap().scopes[&IfccadScopeId::Layout(d.model.id)];
    assert_eq!(scope.quality, Some(IfccadBoundsQuality::Partial));
    assert!(!scope.annotation_reasons.is_empty());
    d.model.bounds = Some(IfccadBounds3d {
        min: [-1000.; 3],
        max: [1000.; 3],
    });
    d.model.bounds_quality = Some(IfccadBoundsQuality::Estimated);
    assert!(
        validate_ifccad_document(&d).is_err(),
        "known dormant positions must not be hidden by unassessed glyphs"
    );
}
#[test]
fn inactive_definition_orientation_survives_without_annotating_existing_text() {
    let mut d = drawing();
    let policy = IfccadAnnotationDefinition {
        enabled: false,
        orientation: IfccadAnnotationOrientation::MatchLayout,
    };
    for s in &mut d.text_styles {
        s.annotation = policy;
    }
    for b in &mut d.blocks {
        b.annotation = IfccadAnnotationDefinition {
            enabled: true,
            ..policy
        };
    }
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let json = encode_ifccad_document(&d).unwrap();
    let cbor = encode_ifccad_cbor_document(&d).unwrap();
    for r in [
        load_ifccad_bytes(json.bytes(), Default::default())
            .unwrap()
            .into_document(),
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .into_document(),
    ] {
        assert_eq!(r.text_styles, d.text_styles);
        assert_eq!(r.blocks, d.blocks);
        for e in &r.model.entities {
            match &e.as_native().unwrap().kind {
                IfccadEntityKind::Text(t) => assert!(t.annotation.is_none()),
                IfccadEntityKind::MText(t) => assert!(t.annotation.is_none()),
                _ => (),
            }
        }
    }
}
#[test]
fn annotative_block_definition_cannot_claim_a_complete_occurrence_enclosure() {
    let mut d = drawing();
    for b in &mut d.blocks {
        b.annotation.enabled = true;
    }
    let a = assess_ifccad_document_bounds(&d).unwrap();
    let model = &a.scopes[&IfccadScopeId::Layout(d.model.id)];
    assert_eq!(model.quality, Some(IfccadBoundsQuality::Partial));
    assert!(model
        .annotation_reasons
        .iter()
        .any(|r| r.reason == IfccadAnnotationBoundsReason::DefinitionSizingUnassessed));
}
#[test]
fn object_orientation_is_independent_of_style_and_disabled_membership() {
    let mut d = drawing();
    for s in &mut d.text_styles {
        s.annotation.orientation = IfccadAnnotationOrientation::Fixed;
    }
    for e in &mut d.model.entities {
        if let IfccadEntityKind::Text(t) = &mut e.as_native_mut().unwrap().kind {
            t.annotation = Some(Box::new(IfccadAnnotation {
                orientation: None,
                mode: IfccadAnnotationMode::Disabled,
                reference: None,
                contexts: vec![],
            }));
            t.annotation.as_mut().unwrap().orientation =
                Some(IfccadAnnotationOrientation::MatchLayout);
        }
    }
    let json = encode_ifccad_document(&d).unwrap();
    let cbor = encode_ifccad_cbor_document(&d).unwrap();
    for r in [
        load_ifccad_bytes(json.bytes(), Default::default())
            .unwrap()
            .into_document(),
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .into_document(),
    ] {
        assert_eq!(r, d);
    }
}
