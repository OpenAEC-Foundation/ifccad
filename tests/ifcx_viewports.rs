use ocdraw::ifcx_cad::*;

fn viewport() -> IfcxCadViewport {
    IfcxCadViewport {
        model_id: 1,
        frame: IfcxCadViewportFrame {
            center: [100., 75.],
            width: 160.,
            height: 100.,
        },
        view: IfcxCadViewportView {
            center: [0., 0.],
            target: [0.; 3],
            direction: [0., 0., 100.],
            height: 200.,
            twist: 0.,
            projection: IfcxCadViewportProjection::Perspective,
            lens_length_mm: Some(50.),
            front_clip: IfcxCadViewportDepthClip {
                mode: IfcxCadViewportClipMode::Disabled,
                distance: None,
            },
            back_clip: IfcxCadViewportDepthClip {
                mode: IfcxCadViewportClipMode::Disabled,
                distance: None,
            },
        },
        render_mode: IfcxCadViewportRenderMode::Wireframe,
        view_enabled: true,
        view_locked: false,
        visible: true,
        paper_clip: IfcxCadViewportPaperClip {
            enabled: false,
            boundary_entity_id: None,
        },
        frozen_layers: vec![],
    }
}
fn placement(origin: [f64; 3]) -> IfcxCadPlacement {
    IfcxCadPlacement {
        origin,
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}

#[test]
fn camera_and_depth_modes_are_strict() {
    let valid = viewport();
    assert!(validate_ifcx_cad_viewport_parameters(&valid).is_ok());
    for lens in [None, Some(0.), Some(-1.), Some(f64::INFINITY)] {
        let mut v = valid.clone();
        v.view.lens_length_mm = lens;
        assert!(validate_ifcx_cad_viewport_parameters(&v).is_err());
    }
    let mut v = valid.clone();
    v.view.projection = IfcxCadViewportProjection::Orthographic;
    v.view.lens_length_mm = Some(0.);
    assert!(validate_ifcx_cad_viewport_parameters(&v).is_ok());
    for direction in [[0.; 3], [f64::MAX; 3], [f64::NAN, 0., 1.]] {
        let mut v = valid.clone();
        v.view.direction = direction;
        assert!(validate_ifcx_cad_viewport_parameters(&v).is_err());
    }
    v = valid.clone();
    v.view.front_clip.mode = IfcxCadViewportClipMode::AtCamera;
    v.view.front_clip.distance = Some(-20.);
    v.view.back_clip = IfcxCadViewportDepthClip {
        mode: IfcxCadViewportClipMode::AtDistance,
        distance: Some(99.),
    };
    assert!(validate_ifcx_cad_viewport_parameters(&v).is_ok());
    v.view.back_clip.distance = Some(100.);
    assert!(validate_ifcx_cad_viewport_parameters(&v).is_err());
    v.view.back_clip.mode = IfcxCadViewportClipMode::AtCamera;
    assert!(validate_ifcx_cad_viewport_parameters(&v).is_err());
    v.view.front_clip = IfcxCadViewportDepthClip {
        mode: IfcxCadViewportClipMode::AtDistance,
        distance: Some(-10.),
    };
    v.view.back_clip = IfcxCadViewportDepthClip {
        mode: IfcxCadViewportClipMode::AtDistance,
        distance: Some(-20.),
    };
    assert!(validate_ifcx_cad_viewport_parameters(&v).is_ok());
    v.view.front_clip.distance = None;
    assert!(validate_ifcx_cad_viewport_parameters(&v).is_err());
}

#[test]
fn circle_clip_checks_complete_curve_and_plane() {
    let frame = viewport().frame;
    let circle = |radius, origin| IfcxCadEntityKind::Circle {
        radius,
        placement: placement(origin),
    };
    assert!(validate_ifcx_cad_viewport_boundary(&frame, &circle(50., [100., 75., 0.])).is_ok());
    assert!(validate_ifcx_cad_viewport_boundary(
        &frame,
        &circle(f64::from_bits(50f64.to_bits() + 1), [100., 75., 0.])
    )
    .is_err());
    for (radius, origin) in [
        (2., [179., 75., 0.]),
        (0., [100., 75., 0.]),
        (-1., [100., 75., 0.]),
        (1., [100., 75., 1.]),
        (f64::MAX, [100., 75., 0.]),
    ] {
        assert!(validate_ifcx_cad_viewport_boundary(&frame, &circle(radius, origin)).is_err());
    }
    let mut tilted = placement([100., 75., 0.]);
    tilted.y_axis = [0., 0., 1.];
    assert!(validate_ifcx_cad_viewport_boundary(
        &frame,
        &IfcxCadEntityKind::Circle {
            radius: 1.,
            placement: tilted
        }
    )
    .is_err());
    let mut rotated = placement([100., 75., 0.]);
    rotated.x_axis = [0., 1., 0.];
    rotated.y_axis = [-1., 0., 0.];
    assert!(validate_ifcx_cad_viewport_boundary(
        &frame,
        &IfcxCadEntityKind::Circle {
            radius: 50.,
            placement: rotated
        }
    )
    .is_ok());
}

#[test]
fn active_boundary_accepts_eligible_polylines_and_circles() {
    let frame = viewport().frame;
    let poly = |vertices, closed| IfcxCadEntityKind::PlanarPolyline {
        vertices,
        closed,
        placement: placement([0.; 3]),
        line_pattern_generation: IfcxCadLinePatternGeneration::PerSegment,
    };
    assert!(validate_ifcx_cad_viewport_boundary(
        &frame,
        &poly(
            vec![
                [20., 25.],
                [180., 25.],
                [100., 75.],
                [180., 125.],
                [20., 125.]
            ],
            true
        )
    )
    .is_ok());
    assert!(validate_ifcx_cad_viewport_boundary(
        &frame,
        &poly(vec![[20., 25.], [180., 25.], [20., 125.]], false)
    )
    .is_err());
    assert!(validate_ifcx_cad_viewport_boundary(
        &frame,
        &poly(vec![[20., 25.], [20., 25.], [20., 125.]], true)
    )
    .is_err());
    assert!(validate_ifcx_cad_viewport_boundary(
        &frame,
        &poly(vec![[19., 25.], [180., 25.], [20., 125.]], true)
    )
    .is_err());
}

#[test]
fn viewport_requires_paper_and_unique_model_target() {
    let mut doc = load_ifcx_cad_bytes(
        include_bytes!("../examples/ifcx-native-cad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut entity = doc.model.entities[0].clone();
    entity.id = doc.id_counters.allocate_entity_id().unwrap();
    let mut v = viewport();
    v.model_id = doc.model.id;
    entity.kind = IfcxCadEntityKind::Viewport(v);
    doc.model.entities.push(entity.clone());
    assert!(validate_ifcx_cad_document(&doc).is_err());
    doc.model.entities.pop();
    let id = doc.id_counters.allocate_layout_id().unwrap();
    doc.paper_layouts.push(IfcxCadPaperLayout {
        id,
        name: "Sheet".into(),
        tab_index: 1,
        length_unit: "in".into(),
        paper: None,
        entities: vec![entity],
    });
    assert!(validate_ifcx_cad_document(&doc).is_ok());
    let IfcxCadEntityKind::Viewport(v) = &mut doc.paper_layouts[0].entities[0].kind else {
        unreachable!()
    };
    v.model_id = id;
    assert!(validate_ifcx_cad_document(&doc).is_err());
}

fn drawing_with_viewport() -> IfcxCadDocument {
    let mut doc = load_ifcx_cad_bytes(
        include_bytes!("../examples/ifcx-native-cad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut entity = doc.model.entities[0].clone();
    entity.id = doc.id_counters.allocate_entity_id().unwrap();
    let mut view = viewport();
    view.model_id = doc.model.id;
    entity.kind = IfcxCadEntityKind::Viewport(view);
    doc.paper_layouts.push(IfcxCadPaperLayout {
        id: doc.id_counters.allocate_layout_id().unwrap(),
        name: "Sheet".into(),
        tab_index: 1,
        length_unit: "mm".into(),
        paper: None,
        entities: vec![entity],
    });
    doc
}

#[test]
fn dormant_boundary_is_owned_and_exclusive() {
    let mut doc = drawing_with_viewport();
    let mut boundary = doc.model.entities[0].clone();
    boundary.id = doc.id_counters.allocate_entity_id().unwrap();
    let id = boundary.id;
    doc.paper_layouts[0].entities.push(boundary);
    let IfcxCadEntityKind::Viewport(v) = &mut doc.paper_layouts[0].entities[0].kind else {
        panic!()
    };
    v.paper_clip.boundary_entity_id = Some(id);
    assert!(validate_ifcx_cad_document(&doc).is_ok()); // dormant ordinary Line is allowed
    let mut copy = doc.paper_layouts[0].entities[0].clone();
    copy.id = doc.id_counters.allocate_entity_id().unwrap();
    doc.paper_layouts[0].entities.push(copy);
    assert!(validate_ifcx_cad_document(&doc).is_err());
    doc.paper_layouts[0].entities.pop();
    let boundary = doc.paper_layouts[0].entities.pop().unwrap();
    doc.model.entities.push(boundary);
    assert!(validate_ifcx_cad_document(&doc).is_err());
    doc.model.entities.pop();
    assert!(validate_ifcx_cad_document(&doc).is_err()); // dangling dormant relationship
}

#[test]
fn frozen_layers_are_a_set_with_exact_large_ids() {
    let mut doc = drawing_with_viewport();
    let mut layer = doc.layers[0].clone();
    layer.id = 9_007_199_254_740_993;
    layer.name = "High".into();
    doc.id_counters.next_layer_id = layer.id + 1;
    let low = doc.layers[0].id;
    let high = layer.id;
    doc.layers.push(layer);
    let IfcxCadEntityKind::Viewport(v) = &mut doc.paper_layouts[0].entities[0].kind else {
        panic!()
    };
    v.frozen_layers = vec![high, low];
    let encoded = encode_ifcx_cad_document(&doc).unwrap();
    let read = load_ifcx_cad_bytes(encoded.bytes(), Default::default()).unwrap();
    let IfcxCadEntityKind::Viewport(v) = &read.document().paper_layouts[0].entities[0].kind else {
        panic!()
    };
    assert_eq!(v.frozen_layers, [low, high]);
    let IfcxCadEntityKind::Viewport(v) = &mut doc.paper_layouts[0].entities[0].kind else {
        panic!()
    };
    v.frozen_layers = vec![high, high];
    assert!(validate_ifcx_cad_document(&doc).is_err());
    let IfcxCadEntityKind::Viewport(v) = &mut doc.paper_layouts[0].entities[0].kind else {
        panic!()
    };
    v.frozen_layers = vec![high + 1];
    assert!(validate_ifcx_cad_document(&doc).is_err());
}

#[test]
fn finite_camera_and_frame_components_cannot_overflow_their_enclosures() {
    let mut v = viewport();
    v.frame.center[0] = f64::MAX;
    v.frame.width = f64::MAX;
    assert!(validate_ifcx_cad_viewport_parameters(&v).is_err());
    for size in [0., -1., f64::NAN] {
        v = viewport();
        v.frame.height = size;
        assert!(validate_ifcx_cad_viewport_parameters(&v).is_err());
    }
    v = viewport();
    v.view.direction = [0., 0., f64::MAX];
    assert!(validate_ifcx_cad_viewport_parameters(&v).is_ok());
}

#[test]
fn native_copy_move_and_delete_preserve_viewport_identity_history() {
    let mut doc = load_ifcx_cad_bytes(
        include_bytes!("../examples/ifcx-native-cad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let view = doc.paper_layouts[0]
        .entities
        .iter()
        .find(|e| matches!(e.kind, IfcxCadEntityKind::Viewport(_)))
        .unwrap()
        .clone();
    let IfcxCadEntityKind::Viewport(v) = &view.kind else {
        unreachable!()
    };
    let boundary_id = v.paper_clip.boundary_entity_id.unwrap();
    let boundary = doc.paper_layouts[0]
        .entities
        .iter()
        .find(|e| e.id == boundary_id)
        .unwrap()
        .clone();
    let mut copy = view.clone();
    copy.id = doc.id_counters.allocate_entity_id().unwrap();
    let copy_id = copy.id;
    let mut copied_boundary = boundary.clone();
    copied_boundary.id = doc.id_counters.allocate_entity_id().unwrap();
    let copied_boundary_id = copied_boundary.id;
    let IfcxCadEntityKind::Viewport(v) = &mut copy.kind else {
        unreachable!()
    };
    v.paper_clip.boundary_entity_id = Some(copied_boundary_id);
    doc.paper_layouts[0]
        .entities
        .extend([copy, copied_boundary]);

    // Moving both objects changes ownership without changing their paths.
    let mut second = doc.paper_layouts[0].clone();
    second.id = doc.id_counters.allocate_layout_id().unwrap();
    second.tab_index = 2;
    second.name = "Moved viewport".into();
    second.entities = vec![view.clone(), boundary];
    doc.paper_layouts[0]
        .entities
        .retain(|e| e.id != view.id && e.id != boundary_id);
    doc.paper_layouts[0].entities.reverse();
    doc.paper_layouts.push(second);
    let encoded = encode_ifcx_cad_document(&doc).unwrap();
    let mut read = load_ifcx_cad_bytes(encoded.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(read, doc);
    assert_eq!(read.paper_layouts[1].entities[0].id, view.id);
    let IfcxCadEntityKind::Viewport(v) = &read.paper_layouts[1].entities[0].kind else {
        unreachable!()
    };
    assert_eq!(v.paper_clip.boundary_entity_id, Some(boundary_id));

    // Deleting the highest allocated pair must not reset the watermark.
    read.paper_layouts[0]
        .entities
        .retain(|e| e.id != copy_id && e.id != copied_boundary_id);
    let encoded = encode_ifcx_cad_document(&read).unwrap();
    let mut read = load_ifcx_cad_bytes(encoded.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert!(read.id_counters.allocate_entity_id().unwrap() > copied_boundary_id);
}
