use ifccad::*;

fn context(
    id: u64,
    target: Option<IfccadAnnotationViewTarget>,
    scale: f64,
) -> IfccadHatchAnnotationViewContext {
    IfccadHatchAnnotationViewContext {
        id: IfccadHatchAnnotationViewContextId(id),
        scale_id: IfccadAnnotationScaleId(3),
        target,
        presentation: IfccadHatchAnnotationContext {
            pattern: IfccadHatchContextPattern {
                families: vec![],
                scale,
                orientation_reference: [0.0; 3],
            },
            loops: vec![],
        },
        view_state: IfccadHatchContextViewState {
            normal: [0.0, 0.0, 1.0],
            rotation: 0.0,
            evaluate: false,
        },
    }
}

#[test]
fn null_target_snapshots_keep_their_own_identity_and_default() {
    let collection = IfccadHatchAnnotationViewContexts {
        next_context_id: 9,
        reference: Some(IfccadHatchAnnotationViewContextId(7)),
        contexts: vec![context(7, None, 1.0), context(8, None, 2.0)],
    };
    let chosen = select_ifccad_hatch_annotation_view_context(
        Some(&collection),
        true,
        Some(IfccadHatchAnnotationViewRequest::ById(
            IfccadHatchAnnotationViewContextId(8),
        )),
    );
    assert!(
        matches!(chosen, IfccadHatchAnnotationViewSelection::Available(value) if value.presentation.pattern.scale == 2.0)
    );
    let active = select_ifccad_hatch_annotation_view_context(
        Some(&collection),
        true,
        Some(IfccadHatchAnnotationViewRequest::ByTarget {
            scale_id: IfccadAnnotationScaleId(3),
            target: IfccadAnnotationViewTarget::PaperViewport { entity_id: 10 },
        }),
    );
    assert!(matches!(
        active,
        IfccadHatchAnnotationViewSelection::Missing
    ));
    assert_eq!(
        collection.reference,
        Some(IfccadHatchAnnotationViewContextId(7))
    );
    assert_eq!(collection.contexts.len(), 2);
}

#[test]
fn equal_target_selectors_report_ambiguity_instead_of_choosing_or_deduplicating() {
    let target = IfccadAnnotationViewTarget::PaperViewport { entity_id: 10 };
    let collection = IfccadHatchAnnotationViewContexts {
        next_context_id: 9,
        reference: None,
        contexts: vec![context(7, Some(target), 1.0), context(8, Some(target), 1.0)],
    };
    assert!(matches!(
        select_ifccad_hatch_annotation_view_context(
            Some(&collection),
            true,
            Some(IfccadHatchAnnotationViewRequest::ByTarget {
                scale_id: IfccadAnnotationScaleId(3),
                target,
            })
        ),
        IfccadHatchAnnotationViewSelection::Ambiguous
    ));
}

#[test]
fn view_id_allocation_preserves_watermarks_and_is_atomic_at_exhaustion() {
    let mut collection = IfccadHatchAnnotationViewContexts {
        next_context_id: (1u64 << 53) + 1,
        reference: None,
        contexts: vec![],
    };
    let allocated = collection
        .allocate_context_id()
        .expect("reserve exact local ID");
    assert_eq!(allocated.0, (1u64 << 53) + 1);
    assert_eq!(collection.next_context_id, allocated.0 + 1);
    collection.next_context_id = u64::MAX;
    assert!(collection.allocate_context_id().is_err());
    assert_eq!(collection.next_context_id, u64::MAX);
    collection.next_context_id = 0;
    assert!(collection.allocate_context_id().is_err());
    assert_eq!(collection.next_context_id, 0);
}
#[test]
fn stored_view_registry_keeps_unused_camera_identity_and_allocation_history() {
    let mut d = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let id = IfccadAnnotationViewId((1u64 << 53) + 1);
    d.annotation_views.push(IfccadAnnotationStoredView {
        annotation_scale_id: None,
        id,
        name: "Saved camera".into(),
        space: IfccadAnnotationViewSpace::Model,
        width: 20.,
        view: IfccadViewportView {
            center: [4., 5.],
            target: [1., 2., 3.],
            direction: [0., 0., 2.],
            height: 10.,
            twist: 0.5,
            projection: IfccadViewportProjection::Orthographic,
            lens_length_mm: Some(50.),
            front_clip: IfccadViewportDepthClip {
                mode: IfccadViewportClipMode::Disabled,
                distance: Some(7.),
            },
            back_clip: IfccadViewportDepthClip {
                mode: IfccadViewportClipMode::Disabled,
                distance: None,
            },
        },
    });
    d.id_counters.next_annotation_view_id = id.0 + 4;
    let reserved = d.id_counters.allocate_annotation_view_id().unwrap();
    assert_eq!(reserved.0, id.0 + 4);
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
        assert_eq!(r.annotation_views, d.annotation_views);
        assert_eq!(r.id_counters.next_annotation_view_id, id.0 + 5);
    }
    d.id_counters.next_annotation_view_id = id.0;
    assert!(validate_ifccad_document(&d).is_err());
}

#[test]
fn saved_view_center_is_two_dimensional_not_an_extra_cad_z_field() {
    let mut d = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let view: IfccadViewportView = serde_json::from_value(serde_json::json!({"center":[0.,0.],"target":[0.,0.,0.],"direction":[0.,0.,1.],"height":10.,"twist":0.,"projection":"Orthographic","frontClip":{"mode":"Disabled"},"backClip":{"mode":"Disabled"}})).unwrap();
    d.annotation_views.push(IfccadAnnotationStoredView {
        annotation_scale_id: None,
        id: IfccadAnnotationViewId(0),
        name: "Camera".into(),
        space: IfccadAnnotationViewSpace::Model,
        width: 10.,
        view,
    });
    let bytes = encode_ifccad_document(&d).unwrap();
    let mut raw: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    let nodes = raw["data"].as_array_mut().unwrap();
    let node = nodes
        .iter_mut()
        .find(|n| n["attributes"]["ifccad::annotationView"].is_object())
        .unwrap();
    node["attributes"]["ifccad::annotationView"]["centerZ"] = serde_json::json!(3.);
    assert!(load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).is_err());
}
