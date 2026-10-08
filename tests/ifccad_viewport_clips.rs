use ocdraw::ifccad::*;
fn frame() -> IfccadViewportFrame {
    IfccadViewportFrame {
        center: [0., 0.],
        width: 6.,
        height: 4.,
    }
}
fn placement() -> IfccadPlacement {
    IfccadPlacement {
        origin: [0.; 3],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
#[test]
fn new_clip_families_accept_tangency_and_reject_interior_escape() {
    let ellipse = |minor| IfccadEntityKind::Ellipse {
        semi_major_radius: 3.,
        semi_minor_radius: minor,
        placement: placement(),
    };
    assert!(validate_ifccad_viewport_boundary(&frame(), &ellipse(2.)).is_ok());
    assert!(validate_ifccad_viewport_boundary(&frame(), &ellipse(2.0_f64.next_up())).is_err());
    let frame = IfccadViewportFrame {
        center: [0., 0.],
        width: 2.,
        height: 2.,
    };
    let poly = |bulges| IfccadEntityKind::PlanarPolyline {
        vertices: vec![[-1., 0.], [1., 0.]],
        bulges,
        closed: true,
        placement: placement(),
        line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
    };
    assert!(validate_ifccad_viewport_boundary(&frame, &poly(vec![1., 1.])).is_ok());
    assert!(validate_ifccad_viewport_boundary(&frame, &poly(vec![-1., -1.])).is_ok());
    assert!(validate_ifccad_viewport_boundary(&frame, &poly(vec![0., 0.])).is_err());
    assert!(validate_ifccad_viewport_boundary(&frame, &poly(vec![2., 0.])).is_err());
}
#[test]
fn expanded_boundaries_survive_production_native_readback() {
    let original = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    for active in [false, true] {
        let mut d = original.clone();
        let boundary_id = d
            .paper_layouts
            .iter_mut()
            .flat_map(|p| &mut p.entities)
            .find_map(|e| {
                if let IfccadEntityKind::Viewport(v) = &mut e.as_native_mut().unwrap().kind {
                    v.paper_clip.enabled = active;
                    v.frame = frame();
                    v.paper_clip.boundary_entity_id
                } else {
                    None
                }
            })
            .unwrap();
        let boundary = d
            .paper_layouts
            .iter_mut()
            .flat_map(|p| &mut p.entities)
            .find(|e| e.id() == boundary_id)
            .unwrap();
        boundary.as_native_mut().unwrap().kind = IfccadEntityKind::Ellipse {
            semi_major_radius: 3.,
            semi_minor_radius: 2.,
            placement: placement(),
        };
        recompute_ifccad_document_bounds(&mut d).unwrap();
        let bytes = encode_ifccad_document(&d).unwrap();
        assert_eq!(
            load_ifccad_bytes(bytes.bytes(), Default::default())
                .unwrap()
                .document(),
            &d
        );
    }
}

#[test]
fn dormant_ineligible_shapes_and_forward_references_keep_identity() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let boundary_id = d.paper_layouts[0]
        .entities
        .iter_mut()
        .find_map(|e| {
            if let IfccadEntityKind::Viewport(v) = &mut e.as_native_mut().unwrap().kind {
                v.paper_clip.enabled = false;
                v.paper_clip.boundary_entity_id
            } else {
                None
            }
        })
        .unwrap();
    let i = d.paper_layouts[0]
        .entities
        .iter()
        .position(|e| e.id() == boundary_id)
        .unwrap();
    let mut boundary = d.paper_layouts[0].entities.remove(i);
    boundary.as_native_mut().unwrap().kind = IfccadEntityKind::SpatialPolyline {
        vertices: vec![[0., 0., 2.], [1., 2., 3.]],
        closed: false,
        line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
    };
    d.paper_layouts[0].entities.push(boundary);
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let bytes = encode_ifccad_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    for e in &mut d.paper_layouts[0].entities {
        if let IfccadEntityKind::Viewport(v) = &mut e.as_native_mut().unwrap().kind {
            if v.paper_clip.boundary_entity_id == Some(boundary_id) {
                v.paper_clip.enabled = true;
            }
        }
    }
    assert!(validate_ifccad_document(&d).is_err());
}
