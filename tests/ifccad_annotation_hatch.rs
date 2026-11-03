use ifccad::{geometry_kernel::hatch::*, *};
fn drawing() -> IfccadDocument {
    let mut d = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-hatch-solid.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.annotation_scales = vec![
        IfccadAnnotationScale {
            id: IfccadAnnotationScaleId(1),
            name: "1:20".into(),
            paper_units: 1.,
            drawing_units: 20.,
            unit_scale: false,
        },
        IfccadAnnotationScale {
            id: IfccadAnnotationScaleId(2),
            name: "1:50".into(),
            paper_units: 1.,
            drawing_units: 50.,
            unit_scale: false,
        },
    ];
    d.id_counters.next_annotation_scale_id = 3;
    d
}
fn hatch(d: &mut IfccadDocument) -> &mut IfccadHatch {
    d.model
        .entities
        .iter_mut()
        .find_map(|e| match &mut e.as_native_mut()?.kind {
            IfccadEntityKind::Hatch(h) => Some(h),
            _ => None,
        })
        .unwrap()
}
fn presentation(scale: f64) -> IfccadHatchAnnotationContext {
    IfccadHatchAnnotationContext {
        pattern: IfccadHatchContextPattern {
            families: vec![],
            scale,
            orientation_reference: [4., 5., 6.],
        },
        loops: vec![
            IfccadHatchContextLoop::Shared {
                base_loop_index: 0,
                classification: IfccadHatchLoopClassification {
                    is_annotative: true,
                    does_not_support_scale: true,
                    force_annotation_visible: true,
                    orient_to_paper: true,
                    is_annotative_block: true,
                    ..Default::default()
                },
            },
            IfccadHatchContextLoop::Local {
                classification: Default::default(),
                boundary: IfccadHatchContextBoundary {
                    geometry: HatchBoundary2::Circle {
                        center: [100., 0.],
                        radius: 5.,
                    },
                    source_entity_ids: vec![],
                },
            },
        ],
    }
}
#[test]
fn hatch_scale_and_repeated_null_view_snapshots_keep_separate_defaults_and_base() {
    let mut d = drawing();
    let h = hatch(&mut d);
    let base = h.loops.clone();
    let key = IfccadAnnotationContextKey {
        scale_id: IfccadAnnotationScaleId(1),
    };
    h.annotation = Some(Box::new(IfccadAnnotation {
        orientation: None,
        mode: IfccadAnnotationMode::Disabled,
        reference: Some(key),
        contexts: vec![IfccadAnnotationContext {
            key,
            presentation: Some(presentation(2.)),
        }],
    }));
    h.annotation_view_contexts = Some(Box::new(IfccadHatchAnnotationViewContexts {
        next_context_id: 5,
        reference: Some(IfccadHatchAnnotationViewContextId(2)),
        contexts: [(1, 3.), (2, 4.)]
            .into_iter()
            .map(|(id, scale)| IfccadHatchAnnotationViewContext {
                id: IfccadHatchAnnotationViewContextId(id),
                scale_id: IfccadAnnotationScaleId(1),
                target: None,
                presentation: presentation(scale),
                view_state: IfccadHatchContextViewState {
                    normal: [0., 0., 2.],
                    rotation: 0.25,
                    evaluate: false,
                },
            })
            .collect(),
    }));
    assert_eq!(h.loops, base);
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert_eq!(
        assess_ifccad_document_bounds(&d).unwrap().scopes[&IfccadScopeId::Layout(d.model.id)]
            .quality,
        Some(IfccadBoundsQuality::Partial)
    );
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
    let h = hatch(&mut d);
    h.annotation_view_contexts.as_mut().unwrap().contexts[1].id =
        IfccadHatchAnnotationViewContextId(1);
    assert!(validate_ifccad_document(&d).is_err());
}
#[test]
fn hatch_view_targets_are_typed_references_and_context_sources_are_not_discarded() {
    let mut d = drawing();
    hatch(&mut d).annotation_view_contexts = Some(Box::new(IfccadHatchAnnotationViewContexts {
        next_context_id: 2,
        reference: None,
        contexts: vec![IfccadHatchAnnotationViewContext {
            id: IfccadHatchAnnotationViewContextId(1),
            scale_id: IfccadAnnotationScaleId(1),
            target: Some(IfccadAnnotationViewTarget::StoredView {
                id: IfccadAnnotationViewId(0),
            }),
            presentation: presentation(1.),
            view_state: IfccadHatchContextViewState {
                normal: [0., 0., 1.],
                rotation: 0.,
                evaluate: true,
            },
        }],
    }));
    assert!(validate_ifccad_document(&d).is_err());
    d.annotation_views.push(IfccadAnnotationStoredView {
        annotation_scale_id: None,
        id: IfccadAnnotationViewId(0),
        name: "Saved target".into(),
        space: IfccadAnnotationViewSpace::Model,
        width: 20.,
        view: IfccadViewportView {
            center: [0., 0.],
            target: [0., 0., 0.],
            direction: [0., 0., 1.],
            height: 10.,
            twist: 0.,
            projection: IfccadViewportProjection::Orthographic,
            lens_length_mm: Some(50.),
            front_clip: IfccadViewportDepthClip {
                mode: IfccadViewportClipMode::Disabled,
                distance: None,
            },
            back_clip: IfccadViewportDepthClip {
                mode: IfccadViewportClipMode::Disabled,
                distance: None,
            },
        },
    });
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let bytes = encode_ifccad_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    let bytes = encode_ifccad_cbor_document(&d).unwrap();
    assert_eq!(
        load_ifccad_cbor_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    let h = hatch(&mut d);
    let p = &mut h.annotation_view_contexts.as_mut().unwrap().contexts[0].presentation;
    let IfccadHatchContextLoop::Local { boundary, .. } = &mut p.loops[1] else {
        unreachable!()
    };
    boundary.source_entity_ids = vec![u64::MAX - 1];
    assert!(validate_ifccad_document(&d).is_err());
}
