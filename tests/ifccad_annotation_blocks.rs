use ifccad::*;
fn drawing() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-nested-blocks.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}
#[test]
fn block_context_full_transform_is_independent_and_survives_both_encodings() {
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
    let e = d
        .model
        .entities
        .iter_mut()
        .find_map(|e| match &mut e.as_native_mut()?.kind {
            IfccadEntityKind::BlockInstance {
                annotation,
                transform,
                ..
            } => Some((annotation, transform)),
            _ => None,
        })
        .unwrap();
    let base = e.1.clone();
    let mut context = base.clone();
    context.placement.origin = [300., 400., 500.];
    context.rotation = 0.5;
    context.scale = [-2., 3., 4.];
    *e.0 = Some(Box::new(IfccadAnnotation {
        orientation: Some(IfccadAnnotationOrientation::Fixed),
        mode: IfccadAnnotationMode::ByDefinition,
        reference: Some(key),
        contexts: vec![IfccadAnnotationContext {
            key,
            presentation: Some(IfccadBlockAnnotationContext { transform: context }),
        }],
    }));
    assert_eq!(*e.1, base);
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
    let e = d
        .model
        .entities
        .iter_mut()
        .find_map(|e| match &mut e.as_native_mut()?.kind {
            IfccadEntityKind::BlockInstance { annotation, .. } => annotation.as_mut(),
            _ => None,
        })
        .unwrap();
    e.contexts[0].presentation.as_mut().unwrap().transform.scale[0] = 0.;
    assert!(validate_ifccad_document(&d).is_err());
}

#[test]
fn explicit_disabled_block_overrides_definition_and_context_obeys_uniform_scaling() {
    let mut d = drawing();
    let (entity_id, definition) = d
        .model
        .entities
        .iter()
        .find_map(|e| match &e.as_native()?.kind {
            IfccadEntityKind::BlockInstance { definition_id, .. } => Some((e.id(), *definition_id)),
            _ => None,
        })
        .unwrap();
    d.blocks
        .iter_mut()
        .find(|b| b.id == definition)
        .unwrap()
        .annotation
        .enabled = true;
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let scope = IfccadScopeId::Layout(d.model.id);
    assert!(assess_ifccad_document_bounds(&d).unwrap().scopes[&scope]
        .annotation_reasons
        .iter()
        .any(|r| r.occurrence_path == [entity_id]));
    let e = d
        .model
        .entities
        .iter_mut()
        .find(|e| e.id() == entity_id)
        .unwrap()
        .as_native_mut()
        .unwrap();
    let IfccadEntityKind::BlockInstance {
        annotation,
        transform,
        ..
    } = &mut e.kind
    else {
        unreachable!()
    };
    let context = transform.clone();
    *annotation = Some(Box::new(IfccadAnnotation {
        orientation: None,
        mode: IfccadAnnotationMode::Disabled,
        reference: None,
        contexts: vec![],
    }));
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(!assess_ifccad_document_bounds(&d).unwrap().scopes[&scope]
        .annotation_reasons
        .iter()
        .any(|r| r.occurrence_path == [entity_id]));
    d.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(1),
        name: "1:10".into(),
        paper_units: 1.,
        drawing_units: 10.,
        unit_scale: false,
    });
    d.id_counters.next_annotation_scale_id = 2;
    d.blocks
        .iter_mut()
        .find(|b| b.id == definition)
        .unwrap()
        .uniform_scaling = true;
    let e = d
        .model
        .entities
        .iter_mut()
        .find(|e| e.id() == entity_id)
        .unwrap()
        .as_native_mut()
        .unwrap();
    let IfccadEntityKind::BlockInstance {
        annotation,
        transform,
        ..
    } = &mut e.kind
    else {
        unreachable!()
    };
    transform.scale = [1.; 3];
    let mut context = context;
    context.scale = [1., 2., 1.];
    annotation
        .as_mut()
        .unwrap()
        .contexts
        .push(IfccadAnnotationContext {
            key: IfccadAnnotationContextKey {
                scale_id: IfccadAnnotationScaleId(1),
            },
            presentation: Some(IfccadBlockAnnotationContext { transform: context }),
        });
    assert!(validate_ifccad_document(&d).is_err());
}
