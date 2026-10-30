use ifccad::*;
#[path = "../examples/support/extrusion_gallery.rs"]
mod gallery;
#[test]
fn thick_width_modes_and_graphics_are_exact_native_state() {
    let mut d = gallery::document();
    for e in &mut d.model.entities {
        if let IfccadEntityKind::Dimension(v) = &mut e.as_native_mut().unwrap().kind {
            if let IfccadDimensionGraphicsKind::LineSegment { thickness, .. } =
                &mut v.graphics.as_mut().unwrap().parts[0].kind
            {
                *thickness = -0.
            }
        }
    }
    let j = encode_ifccad_document(&d).unwrap();
    let b = encode_ifccad_cbor_document(&d).unwrap();
    for out in [
        load_ifccad_bytes(j.bytes(), Default::default())
            .unwrap()
            .into_document(),
        load_ifccad_cbor_bytes(b.bytes(), Default::default())
            .unwrap()
            .into_document(),
    ] {
        assert_eq!(d, out);
        for e in &out.model.entities {
            if let IfccadEntityKind::Dimension(v) = &e.as_native().unwrap().kind {
                let IfccadDimensionGraphicsKind::LineSegment { thickness, .. } =
                    &v.graphics.as_ref().unwrap().parts[0].kind
                else {
                    panic!()
                };
                assert_eq!(thickness.to_bits(), (-0f64).to_bits());
            }
        }
    }
}
#[test]
fn extrusion_bounds_overflow_does_not_modify_stored_boxes() {
    let mut d = gallery::document();
    let mut e = d.model.entities[0].clone();
    e.as_native_mut().unwrap().kind = IfccadEntityKind::LineSegment {
        start: [f64::MAX, 0., 0.],
        end: [f64::MAX, 1., 0.],
        thickness: f64::MAX,
        extrusion_direction: [1., 0., 0.],
    };
    d.model.entities = vec![e];
    d.model.bounds = None;
    let before = d.clone();
    assert!(encode_ifccad_document(&d).is_err());
    assert!(recompute_ifccad_document_bounds(&mut d).is_err());
    assert_eq!(d, before);
}

#[test]
fn nested_nonuniform_mirrored_instances_transform_the_complete_sweep() {
    let mut d = gallery::document();
    let template = d.model.entities[0].clone();
    d.model.entities.clear();
    let mut leaf = template.clone();
    leaf.as_native_mut().unwrap().id = d.id_counters.allocate_entity_id().unwrap();
    leaf.as_native_mut().unwrap().kind = IfccadEntityKind::LineSegment {
        start: [1., 2., 3.],
        end: [4., 6., 8.],
        thickness: 2.,
        extrusion_direction: [0., 0., 1.],
    };
    let inner = d.id_counters.allocate_block_id().unwrap();
    let outer = d.id_counters.allocate_block_id().unwrap();
    let block = |id, name: &str, entities| IfccadBlockDefinition {
        id,
        name: name.into(),
        description: String::new(),
        anonymous: false,
        explodable: true,
        uniform_scaling: false,
        bounds_quality: None,
        bounds: None,
        base_point: [0.; 3],
        insertion_unit: "mm".into(),
        entities,
    };
    d.blocks.push(block(inner, "Sweep inner", vec![leaf]));
    let frame = |origin| IfccadPlacement {
        origin,
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    };
    let mut instance = template.clone();
    instance.as_native_mut().unwrap().id = d.id_counters.allocate_entity_id().unwrap();
    instance.as_native_mut().unwrap().kind = IfccadEntityKind::BlockInstance {
        definition_id: inner,
        transform: IfccadBlockTransform {
            placement: frame([0.; 3]),
            rotation: 0.,
            scale: [-2., 3., 4.],
        },
    };
    d.blocks.push(block(outer, "Sweep outer", vec![instance]));
    let mut instance = template;
    instance.as_native_mut().unwrap().id = d.id_counters.allocate_entity_id().unwrap();
    instance.as_native_mut().unwrap().kind = IfccadEntityKind::BlockInstance {
        definition_id: outer,
        transform: IfccadBlockTransform {
            placement: frame([10., 20., 30.]),
            rotation: 0.,
            scale: [1., 1., 1.],
        },
    };
    d.model.entities.push(instance);
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let b = d.model.bounds.unwrap();
    for (actual, want) in b.min.into_iter().zip([2., 26., 42.]) {
        assert!(actual <= want && want - actual < 1e-10);
    }
    for (actual, want) in b.max.into_iter().zip([8., 38., 70.]) {
        assert!(actual >= want && actual - want < 1e-10);
    }
}
