use ocdraw::ocdraw::*;

fn frame(width: f64, height: f64) -> DrawingViewportFrame {
    DrawingViewportFrame {
        center: Point2::new(0., 0.),
        width,
        height,
    }
}
fn circle(radius: f64) -> DrawingGeometry {
    DrawingGeometry::Circle {
        placement: CoordinateFrame3::default(),
        radius,
    }
}
fn poly(vertices: Vec<[f64; 3]>, closed: bool) -> DrawingGeometry {
    DrawingGeometry::PlanarPolyline {
        placement: CoordinateFrame3::default(),
        vertices,
        closed,
        line_pattern_generation: LinePatternGeneration::PerSegment,
    }
}
fn valid(f: DrawingViewportFrame, g: &DrawingGeometry) -> bool {
    validate_viewport_clip_boundary(f, g).is_ok()
}

#[test]
fn circles_accept_exact_frame_tangency_and_reject_excursions() {
    assert!(valid(frame(4., 4.), &circle(2.)));
    assert!(!valid(frame(4., 4.), &circle(2.0_f64.next_up())));
}
#[test]
fn full_ellipses_accept_tangency_and_reflection_but_not_arcs() {
    let placement = CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(-1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let mut g = DrawingGeometry::Ellipse {
        placement,
        semi_major_radius: 3.,
        semi_minor_radius: 2.,
        arc: None,
    };
    assert!(valid(frame(6., 4.), &g));
    if let DrawingGeometry::Ellipse { arc, .. } = &mut g {
        *arc = Some((0., 1.));
    }
    assert!(!valid(frame(6., 4.), &g));
}
#[test]
fn closed_two_vertex_curves_accept_both_directions() {
    for b in [1., -1.] {
        assert!(valid(
            frame(2., 2.),
            &poly(vec![[-1., 0., b], [1., 0., b]], true)
        ));
        assert!(!valid(
            frame(2., 2.),
            &poly(vec![[-1., 0., b], [1., 0., b]], false)
        ));
    }
    assert!(!valid(
        frame(2., 2.),
        &poly(vec![[-1., 0., 0.], [1., 0., 0.]], true)
    ));
}
#[test]
fn closing_bulges_and_major_arcs_cannot_escape_the_frame() {
    // Endpoints all fit; the closing semicircle reaches x=2.
    let g = poly(vec![[0., -1., 0.], [1., -1., 0.], [1., 1., 1.]], true);
    assert!(!valid(frame(2., 2.), &g));
    assert!(valid(frame(8., 8.), &g));
    for b in [2., -2.] {
        let g = poly(vec![[-1., 0., b], [1., 0., b]], true);
        assert!(!valid(frame(2., 2.), &g));
        assert!(valid(frame(8., 8.), &g));
    }
}
#[test]
fn rotated_curves_use_actual_extrema_not_transformed_local_boxes() {
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let placement = CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(s, s, 0.),
        Vector3::new(-s, s, 0.),
    )
    .unwrap();
    let mut g = poly(vec![[-1., 0., 1.], [1., 0., 1.]], true);
    if let DrawingGeometry::PlanarPolyline { placement: p, .. } = &mut g {
        *p = placement;
    }
    assert!(valid(frame(2.1, 2.1), &g));
    assert!(!valid(frame(1.9, 2.1), &g));
}

#[test]
fn minor_bulges_only_require_their_directed_sweep_to_fit() {
    for b in [0.5, -0.5] {
        let g = poly(vec![[-1., 0., b], [1., 0., b]], true);
        // The two minor arcs form a lens with extents x=+-1, y=+-0.5.
        // Their whole generating circles extend to x=+-1.25.
        assert!(valid(frame(2., 1.), &g));
        assert!(!valid(frame(2., 1.0_f64.next_down()), &g));
    }
}
#[test]
fn malformed_boundaries_return_diagnostics_without_panicking() {
    for g in [
        circle(f64::NAN),
        circle(f64::INFINITY),
        circle(-1.),
        poly(vec![[0., 0., 1.], [0., 0., 1.]], true),
        poly(vec![[0., 0., 0.], [1., f64::NAN, 0.], [2., 0., 0.]], true),
        DrawingGeometry::Line {
            start: [0.; 3],
            end: [1.; 3],
        },
    ] {
        let e = validate_viewport_clip_boundary(frame(4., 4.), &g).unwrap_err();
        assert!(e.diagnostics().iter().any(|d| d.code == "VIEWPORT_CLIP"));
    }
    assert!(!valid(frame(f64::INFINITY, 4.), &circle(1.)));
}
#[test]
fn plane_and_vertex_rules_remain_exact() {
    let placement = CoordinateFrame3::try_new(
        Point3::new(0., 0., 1.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    assert!(!valid(
        frame(4., 4.),
        &DrawingGeometry::Circle {
            placement,
            radius: 1.
        }
    ));
    let tilted = CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 0., 1.),
    )
    .unwrap();
    let mut g = poly(vec![[-1., 0., 0.], [0., 0., 0.], [1., 0., 0.]], true);
    if let DrawingGeometry::PlanarPolyline { placement, .. } = &mut g {
        *placement = tilted;
    }
    assert!(valid(frame(4., 4.), &g));
    assert!(!valid(
        frame(4., 4.),
        &poly(vec![[0., 0., 0.], [-0., 0., 0.], [1., 1., 0.]], true)
    ));
    assert!(valid(
        frame(4., 4.),
        &poly(
            vec![[-1., -1., 0.], [1., 1., 0.], [-1., 1., 0.], [1., -1., 0.]],
            true
        )
    ));
}

fn document(g: DrawingGeometry) -> OcdrawDocument {
    let mut d = load_ocdraw_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/paper-viewport.ocdraw.json"
    ))
    .unwrap()
    .into_document();
    d.geometric_entities.push(DrawingGeometricEntity {
        id: 2,
        layer_id: 0,
        visible: false,
        appearance: EntityAppearance::default(),
        geometry: g,
    });
    d.scopes[1].entities.push(2);
    d.scopes[1].bounds = Some(Bounds3d::new(
        Point3::new(-10., -10., 0.),
        Point3::new(10., 10., 0.),
    ));
    d.next_entity_id = 3;
    d.viewports[0].paper_clip = DrawingPaperClip {
        enabled: true,
        boundary_entity_id: Some(2),
    };
    d
}
#[test]
fn new_clip_families_survive_authored_and_encoded_validation() {
    for g in [
        circle(2.),
        DrawingGeometry::Ellipse {
            placement: CoordinateFrame3::default(),
            semi_major_radius: 3.,
            semi_minor_radius: 2.,
            arc: None,
        },
        poly(vec![[-1., 0., 1.], [1., 0., 1.]], true),
    ] {
        let d = document(g);
        validate_ocdraw_document(&d).unwrap();
        let encoded = encode_ocdraw_document(&d).unwrap();
        let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap().into_document();
        assert_eq!(loaded.geometric_entities, d.geometric_entities);
        assert_eq!(loaded.scopes[1].entities, vec![1, 2]);
        assert_eq!(loaded.viewports[0].paper_clip, d.viewports[0].paper_clip);
    }
}
#[test]
fn relationship_rules_and_dormant_state_survive_readback() {
    let mut d = document(DrawingGeometry::Line {
        start: [0.; 3],
        end: [1., 1., 0.],
    });
    d.viewports[0].paper_clip.enabled = false;
    let encoded = encode_ocdraw_document(&d).unwrap();
    assert!(
        !load_ocdraw_bytes(encoded.bytes()).unwrap().viewports()[0]
            .paper_clip
            .enabled
    );
    d.viewports[0].paper_clip.boundary_entity_id = Some(99);
    assert!(validate_ocdraw_document(&d).is_err());
    let mut d = document(circle(1.));
    d.scopes[1].entities.pop();
    d.scopes[0].entities.push(2);
    assert!(validate_ocdraw_document(&d).is_err());
    let mut d = document(circle(1.));
    let mut v = d.viewports[0].clone();
    v.id = 3;
    d.viewports.push(v);
    d.scopes[1].entities.push(3);
    d.next_entity_id = 4;
    assert!(validate_ocdraw_document(&d).is_err());
}
