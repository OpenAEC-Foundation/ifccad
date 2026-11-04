use ifccad::{dimension::*, *};
#[test]
fn dimension_sizing_keeps_viewport_annotation_and_dormant_factor_distinct() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    for sizing in [
        DimensionPresentationScale::Fixed { factor: 2. },
        DimensionPresentationScale::ByViewport,
        DimensionPresentationScale::ByAnnotationScale {
            dormant_factor: None,
        },
        DimensionPresentationScale::ByAnnotationScale {
            dormant_factor: Some(3.),
        },
    ] {
        d.dimension_styles[0].settings.presentation_scale = sizing;
        d.dimension_styles[0].annotative = true;
        let bytes = encode_ifccad_document(&d).unwrap();
        let cbor = encode_ifccad_cbor_document(&d).unwrap();
        for read in [
            load_ifccad_bytes(bytes.bytes(), Default::default())
                .unwrap()
                .into_document(),
            load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
                .unwrap()
                .into_document(),
        ] {
            assert_eq!(read.dimension_styles, d.dimension_styles);
        }
    }
    d.dimension_styles[0].settings.presentation_scale =
        DimensionPresentationScale::ByAnnotationScale {
            dormant_factor: Some(0.),
        };
    assert!(validate_ifccad_document(&d).is_err());
}

#[test]
fn dimension_contexts_match_all_nine_kinds_and_keep_xyz_dormant_layout_and_pictures() {
    let cases = [
        (
            include_bytes!("../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx")
                .as_slice(),
            IfccadDimensionContextGeometry::Linear {
                dimension_line_point: [10., 20., 30.],
            },
        ),
        (
            include_bytes!("../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx")
                .as_slice(),
            IfccadDimensionContextGeometry::Aligned {
                dimension_line_point: [10., 20., 30.],
            },
        ),
        (
            include_bytes!("../conformance/next/ifccad/valid/dimension-angular-two-lines.ifcx")
                .as_slice(),
            IfccadDimensionContextGeometry::AngularTwoLines {
                dimension_arc_point: [10., 20., 30.],
            },
        ),
        (
            include_bytes!("../conformance/next/ifccad/valid/dimension-angular-three-points.ifcx")
                .as_slice(),
            IfccadDimensionContextGeometry::AngularThreePoints {
                dimension_arc_point: [10., 20., 30.],
            },
        ),
        (
            include_bytes!("../conformance/next/ifccad/valid/dimension-radius.ifcx").as_slice(),
            IfccadDimensionContextGeometry::Radius {
                curve_point: [10., 20., 30.],
            },
        ),
        (
            include_bytes!("../conformance/next/ifccad/valid/dimension-diameter.ifcx").as_slice(),
            IfccadDimensionContextGeometry::Diameter {
                first_diameter_point: [10., 20., 30.],
                second_diameter_point: [40., 50., 60.],
            },
        ),
        (
            include_bytes!("../conformance/next/ifccad/valid/dimension-ordinate-signed.ifcx")
                .as_slice(),
            IfccadDimensionContextGeometry::Ordinate {
                feature_point: [10., 20., 30.],
                leader_end: [40., 50., 60.],
            },
        ),
        (
            include_bytes!("../conformance/next/ifccad/valid/dimension-jogged-radius.ifcx")
                .as_slice(),
            IfccadDimensionContextGeometry::JoggedRadius {
                display_center: [10., 20., 30.],
                jog_point: [40., 50., 60.],
            },
        ),
        (
            include_bytes!("../conformance/next/ifccad/valid/dimension-arc-length-full.ifcx")
                .as_slice(),
            IfccadDimensionContextGeometry::ArcLength {
                dimension_arc_point: [10., 20., 30.],
            },
        ),
    ];
    for (bytes, geometry) in cases {
        let kind = geometry.kind();
        let mut d = load_ifccad_bytes(bytes, Default::default())
            .unwrap()
            .into_document();
        d.annotation_scales.push(IfccadAnnotationScale {
            id: IfccadAnnotationScaleId(1),
            name: "1:50".into(),
            paper_units: 1.,
            drawing_units: 50.,
            unit_scale: false,
        });
        d.id_counters.next_annotation_scale_id = 2;
        let entity = d
            .model
            .entities
            .iter_mut()
            .find_map(|e| match &mut e.as_native_mut()?.kind {
                IfccadEntityKind::Dimension(v) if v.definition.kind() == kind => Some(v),
                _ => None,
            })
            .unwrap();
        let geometry = if entity.definition.kind() == DimensionKind::Aligned {
            IfccadDimensionContextGeometry::Aligned {
                dimension_line_point: [10., 20., 30.],
            }
        } else {
            geometry
        };
        let original = entity.definition.clone();
        entity.annotation = Some(Box::new(IfccadAnnotation {
            mode: IfccadAnnotationMode::ByDefinition,
            orientation: None,
            reference: None,
            contexts: vec![IfccadAnnotationContext {
                key: IfccadAnnotationContextKey {
                    scale_id: IfccadAnnotationScaleId(1),
                },
                presentation: Some(IfccadDimensionAnnotationContext {
                    text_location: [100., 200.],
                    user_text_location: false,
                    text_rotation: 0.5,
                    layout: IfccadDimensionContextLayout {
                        line_outside: IfccadStoredBoolOverride {
                            active: false,
                            value: true,
                        },
                        ..Default::default()
                    },
                    geometry,
                    graphics: Some(IfccadDimensionGraphics {
                        origin: IfccadDimensionGraphicsOrigin::SourceSnapshot,
                        parts: vec![],
                    }),
                }),
            }],
        }));
        assert_eq!(entity.definition, original);
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
        let entity = d
            .model
            .entities
            .iter_mut()
            .find_map(|e| match &mut e.as_native_mut()?.kind {
                IfccadEntityKind::Dimension(v) if v.definition.kind() == kind => Some(v),
                _ => None,
            })
            .unwrap();
        entity.annotation.as_mut().unwrap().contexts[0]
            .presentation
            .as_mut()
            .unwrap()
            .geometry = IfccadDimensionContextGeometry::Diameter {
            first_diameter_point: [1., 2., 3.],
            second_diameter_point: [4., 5., 6.],
        };
        if entity.definition.kind() != DimensionKind::Diameter {
            assert!(validate_ifccad_document(&d).is_err());
        }
    }
}

#[test]
fn contextual_picture_reuses_closed_parts_and_keeps_base_picture_independent() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/dimension-graphics.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(1),
        name: "1:25".into(),
        paper_units: 1.,
        drawing_units: 25.,
        unit_scale: false,
    });
    d.id_counters.next_annotation_scale_id = 2;
    let e = d
        .model
        .entities
        .iter_mut()
        .find_map(|e| match &mut e.as_native_mut()?.kind {
            IfccadEntityKind::Dimension(v) if v.graphics.is_some() => Some(v),
            _ => None,
        })
        .unwrap();
    let base = e.graphics.clone();
    let mut picture = base.clone().unwrap();
    picture.parts.reverse();
    picture.origin = IfccadDimensionGraphicsOrigin::Generated;
    let kind = e.definition.kind();
    let geometry = if kind == DimensionKind::Aligned {
        IfccadDimensionContextGeometry::Aligned {
            dimension_line_point: [10., 20., 30.],
        }
    } else {
        IfccadDimensionContextGeometry::Linear {
            dimension_line_point: [10., 20., 30.],
        }
    };
    e.annotation = Some(Box::new(IfccadAnnotation {
        mode: IfccadAnnotationMode::Disabled,
        orientation: None,
        reference: None,
        contexts: vec![IfccadAnnotationContext {
            key: IfccadAnnotationContextKey {
                scale_id: IfccadAnnotationScaleId(1),
            },
            presentation: Some(IfccadDimensionAnnotationContext {
                text_location: [100., 200.],
                user_text_location: false,
                text_rotation: 0.25,
                layout: Default::default(),
                geometry,
                graphics: Some(picture),
            }),
        }],
    }));
    assert_eq!(e.graphics, base);
    let counters = d.id_counters;
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
    assert_eq!(d.id_counters, counters);
    let mut raw: serde_json::Value = serde_json::from_slice(json.bytes()).unwrap();
    let node = raw["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"]["ifccad::annotation"].is_object())
        .unwrap();
    node["attributes"]["ifccad::annotation"]["contexts"][0]["presentation"]["graphics"]["parts"]
        [0]["ifccad::unknownPart"] = serde_json::json!({});
    assert!(load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).is_err());
    let e = d
        .model
        .entities
        .iter_mut()
        .find_map(|e| match &mut e.as_native_mut()?.kind {
            IfccadEntityKind::Dimension(v) if v.annotation.is_some() => Some(v),
            _ => None,
        })
        .unwrap();
    e.annotation.as_mut().unwrap().contexts[0]
        .presentation
        .as_mut()
        .unwrap()
        .graphics
        .as_mut()
        .unwrap()
        .parts[0]
        .layer_id = Some(u64::MAX);
    assert!(validate_ifccad_document(&d).is_err());
}
