use ifccad::*;
#[test]
fn drawing_model_paper_saved_view_and_viewport_annotation_state_is_independent_and_exact() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/workspace-state.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let a = IfccadAnnotationScaleId((1u64 << 53) + 1);
    let b = IfccadAnnotationScaleId(a.0 + 1);
    d.annotation_scales = vec![
        IfccadAnnotationScale {
            id: a,
            name: "1:20".into(),
            paper_units: 1.,
            drawing_units: 20.,
            unit_scale: false,
        },
        IfccadAnnotationScale {
            id: b,
            name: "1:50".into(),
            paper_units: 1.,
            drawing_units: 50.,
            unit_scale: false,
        },
    ];
    d.id_counters.next_annotation_scale_id = b.0 + 3;
    d.annotation_settings = Some(IfccadDrawingAnnotationSettings {
        insert_as_annotative: Some(false),
        model_linetype_scaling: Some(IfccadModelLinetypeAnnotationMode::ByAnnotationScale),
    });
    d.model.annotation_state = Some(IfccadLayoutAnnotationState {
        current_scale_id: Some(a),
        visibility: Some(IfccadAnnotationVisibility::CurrentScaleOnly),
    });
    let mut second = d.paper_layouts[0].clone();
    second.id = d.id_counters.allocate_layout_id().unwrap();
    second.name = "Second sheet".into();
    second.tab_index = 2;
    second.entities.clear();
    second.canvas = None;
    d.paper_layouts[0].annotation_state = Some(IfccadLayoutAnnotationState {
        current_scale_id: Some(b),
        visibility: Some(IfccadAnnotationVisibility::All),
    });
    second.annotation_state = Some(IfccadLayoutAnnotationState {
        current_scale_id: None,
        visibility: Some(IfccadAnnotationVisibility::CurrentScaleOnly),
    });
    d.paper_layouts.push(second);
    let viewport = d.paper_layouts[0]
        .entities
        .iter_mut()
        .find_map(|e| match &mut e.as_native_mut()?.kind {
            IfccadEntityKind::Viewport(v) => Some(v),
            _ => None,
        })
        .unwrap();
    viewport.annotation_scale_id = Some(a);
    let view = viewport.view.clone();
    let width = viewport.frame.width;
    d.annotation_views.push(IfccadAnnotationStoredView {
        annotation_scale_id: Some(b),
        id: IfccadAnnotationViewId(0),
        name: "Saved annotation camera".into(),
        space: IfccadAnnotationViewSpace::Model,
        width,
        view,
    });

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
    invalid.annotation_views[0].annotation_scale_id = Some(IfccadAnnotationScaleId(42));
    assert!(validate_ifccad_document(&invalid).is_err());
    let mut raw: serde_json::Value = serde_json::from_slice(json.bytes()).unwrap();
    let node = raw["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"]["ifccad::drawing"].is_object())
        .unwrap();
    node["attributes"]["ifccad::drawing"]["annotationSettings"]["insertAsAnnotative"] =
        serde_json::Value::Null;
    assert!(load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).is_err());
}
