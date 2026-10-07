use ocdraw::ifccad::*;
fn base() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}
#[test]
fn encoding_retains_absent_and_oversized_bounds() {
    let mut d = base();
    assert!(d.model.bounds.is_none());
    d.model.bounds = Some(IfccadBounds3d {
        min: [-1e9; 3],
        max: [1e9; 3],
    });
    let bytes = encode_ifccad_document(&d).unwrap();
    let readback = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(readback.document().model.bounds, d.model.bounds);
    d.model.bounds = Some(IfccadBounds3d {
        min: [0.; 3],
        max: [0.; 3],
    });
    assert!(validate_ifccad_document(&d).is_err());
}
#[test]
fn recomputation_is_atomic_and_keeps_identity() {
    let mut d = base();
    let ids = d.id_counters;
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_some());
    assert_eq!(d.id_counters, ids);
    d.model.entities[0].kind = IfccadEntityKind::Circle {
        radius: f64::MAX,
        placement: IfccadPlacement {
            origin: [f64::MAX, 0., 0.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
    };
    let before = d.clone();
    assert!(recompute_ifccad_document_bounds(&mut d).is_err());
    assert_eq!(d, before);
}
#[test]
fn separate_layout_and_block_ids_do_not_alias_during_preparation() {
    let mut d = base();
    let template = d.model.entities[0].clone();
    d.model.entities.clear();
    d.blocks.clear();
    let mut leaf = template.clone();
    leaf.id = d.id_counters.allocate_entity_id().unwrap();
    leaf.kind = IfccadEntityKind::LineSegment {
        start: [4., 0., 0.],
        end: [6., 0., 0.],
    };
    let block_id = d.model.id;
    d.id_counters.next_block_id = d.id_counters.next_block_id.max(block_id + 1);
    d.blocks.push(IfccadBlockDefinition {
        id: block_id,
        name: "Bounds block".into(),
        base_point: [4., 0., 0.],
        insertion_unit: "mm".into(),
        bounds: None,
        entities: vec![leaf],
    });
    let mut instance = template;
    instance.id = d.id_counters.allocate_entity_id().unwrap();
    instance.kind = IfccadEntityKind::BlockInstance {
        definition_id: block_id,
        transform: IfccadBlockTransform {
            placement: IfccadPlacement {
                origin: [10., 0., 0.],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            rotation: 0.,
            scale: [-2., 1., 1.],
        },
    };
    d.model.entities.push(instance);
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let local = d.blocks[0].bounds.unwrap();
    let placed = d.model.bounds.unwrap();
    assert!(local.min[0] <= 4. && local.max[0] >= 6.);
    assert!(placed.min[0] <= 6. && placed.max[0] >= 10.);
    assert!(placed.min[0] > 5. && placed.max[0] < 11.);
    validate_ifccad_document(&d).unwrap();
}

#[test]
fn bounds_cover_curve_interiors_and_exclude_dormant_bulges() {
    let mut d = base();
    let mut e = d.model.entities[0].clone();
    d.blocks.clear();
    d.model.entities.clear();
    e.kind = IfccadEntityKind::Arc {
        radius: 2.,
        start_parameter: 0.,
        sweep_parameter: std::f64::consts::PI,
        placement: IfccadPlacement {
            origin: [0.; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
    };
    d.model.entities.push(e.clone());
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let bounds = d.model.bounds.unwrap();
    assert!(bounds.max[1] >= 2.);
    assert!(bounds.min[0] <= -2. && bounds.max[0] >= 2.);
    d.model.entities[0].kind = IfccadEntityKind::PlanarPolyline {
        vertices: vec![[0., 0.], [2., 0.]],
        bulges: vec![0., 100.],
        closed: false,
        placement: IfccadPlacement {
            origin: [0.; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
    };
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let bounds = d.model.bounds.unwrap();
    assert_eq!(bounds.min, [0.; 3]);
    assert_eq!(bounds.max, [2., 0., 0.]);
}

#[test]
fn empty_owners_require_absent_bounds_and_wire_null_is_rejected() {
    let mut d = base();
    d.model.entities.clear();
    d.blocks.clear();
    d.model.bounds = Some(IfccadBounds3d {
        min: [0.; 3],
        max: [0.; 3],
    });
    assert!(validate_ifccad_document(&d).is_err());
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_none());
    let mut raw: serde_json::Value =
        serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap();
    let node = raw["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == format!("/cad/d{}/layout/{}", d.drawing_id, d.model.id))
        .unwrap();
    node["attributes"]["ifccad::layout"]["bounds"] = serde_json::Value::Null;
    assert!(load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).is_err());
}

#[test]
fn paper_bounds_use_coordinates_and_include_hidden_viewport_frames() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let before = d.paper_layouts[0].bounds;
    d.paper_layouts[0].settings.media = Some(IfccadLayoutMedia {
        unit: IfccadMediaUnit::Physical(ocdraw::geometry_kernel::CoordinateLengthUnit::Inch),
        width: 1000.,
        height: 1000.,
    });
    for e in &mut d.paper_layouts[0].entities {
        if let IfccadEntityKind::Viewport(v) = &mut e.kind {
            v.visible = false;
            v.view.target = [1e20, 2e20, 3e20];
        }
    }
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert_eq!(d.paper_layouts[0].bounds, before);
    let bounds = d.paper_layouts[0].bounds.unwrap();
    for e in &d.paper_layouts[0].entities {
        if let IfccadEntityKind::Viewport(v) = &e.kind {
            assert!(bounds.min[0] <= v.frame.center[0] - v.frame.width / 2.);
            assert!(bounds.max[1] >= v.frame.center[1] + v.frame.height / 2.);
            assert!(bounds.min[2] <= 0. && bounds.max[2] >= 0.);
        }
    }
    validate_ifccad_document(&d).unwrap();
}
