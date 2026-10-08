use cad_geometry_convert::hatch::*;
use ocdraw::geometry_kernel::hatch::{HatchAreaRule, HatchBoundary2};
use opencadcodec::entities::hatch::*;
use opencadcodec::{Handle, Vector2};

fn circle() -> Hatch {
    let mut h = Hatch::solid();
    let mut p = BoundaryPath::new();
    p.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::new(2., 3.),
        radius: 4.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    p.boundary_handles = vec![Handle::new(37), Handle::new(38)];
    h.paths.push(p);
    h
}

#[test]
fn solid_keeps_loops_handles_and_curve_evidence() {
    let mut h = circle();
    h.paths.push(h.paths[0].clone());
    h.style = HatchStyleType::Ignore;
    let p = prepare_hatch_from_cad(&h, 1e-9).unwrap();
    assert_eq!(p.boundaries.len(), 2);
    assert_eq!(p.source_handles[0], h.paths[0].boundary_handles);
    assert_eq!(p.area_rule, HatchAreaRule::Ignore);
    assert!(matches!(p.boundaries[0], HatchBoundary2::Circle { .. }));
    assert_eq!(p.pairs.iter().map(|p| p.curves.len()).sum::<usize>(), 2);
    let q =
        prepare_hatch_to_cad(p.placement, &p.boundaries, p.area_rule, p.join_tolerance).unwrap();
    assert_eq!(q.hatch.paths.len(), 2);
    assert_eq!(q.hatch.style, h.style);
    assert_eq!(q.pairs.iter().map(|p| p.curves.len()).sum::<usize>(), 2);
}

#[test]
fn active_gradient_and_mpolygon_are_not_solid_fallbacks() {
    let mut h = circle();
    h.gradient_color.enabled = true;
    assert!(matches!(
        prepare_hatch_from_cad(&h, 1e-9),
        Err(CadHatchPreparationError::Unsupported {
            field: "gradient_color",
            ..
        })
    ));
    h.gradient_color.enabled = false;
    h.is_mpolygon = true;
    assert!(matches!(
        prepare_hatch_from_cad(&h, 1e-9),
        Err(CadHatchPreparationError::Unsupported {
            field: "is_mpolygon",
            ..
        })
    ));
}

#[test]
fn no_epsilon_full_turn_or_open_polyline_repair() {
    let mut h = circle();
    if let BoundaryEdge::CircularArc(a) = &mut h.paths[0].edges[0] {
        a.end_angle = std::f64::consts::TAU - 1e-5;
    }
    assert!(prepare_hatch_from_cad(&h, 0.).is_err());
    h.paths[0].edges = vec![BoundaryEdge::Polyline(PolylineEdge::new(
        vec![
            Vector2::new(0., 0.),
            Vector2::new(1., 0.),
            Vector2::new(0., 1.),
        ],
        false,
    ))];
    h.paths[0].flags = BoundaryPathFlags::POLYLINE;
    assert!(matches!(
        prepare_hatch_from_cad(&h, 1e-9),
        Err(CadHatchPreparationError::Unsupported {
            field: "paths.edges.polyline.is_closed",
            ..
        })
    ));
}

#[test]
fn rotated_plane_full_circle_has_equivalent_curve_correspondence() {
    use cad_geometry_convert::geometry::numeric::exact;
    use ocdraw::geometry_kernel::{CoordinateFrame3, Point3, Vector3};
    let p = CoordinateFrame3::try_new(
        Point3::new(7., -2., 3.),
        Vector3::new(0., 1., 0.),
        Vector3::new(-1., 0., 0.),
    )
    .unwrap();
    let b = vec![HatchBoundary2::Circle {
        center: [2., 3.],
        radius: 4.,
    }];
    let out = prepare_hatch_to_cad(p, &b, HatchAreaRule::Normal, 1e-9).unwrap();
    assert!(out.pairs[0].curves[0].squared_deviation().unwrap().1 < exact(1e-24));
}

#[test]
fn ellipse_bulges_and_reversed_edge_sweeps_keep_all_contours() {
    let mut h = circle();
    let mut e = BoundaryPath::new();
    e.add_edge(BoundaryEdge::EllipticArc(EllipticArcEdge {
        center: Vector2::new(-2., 1.),
        major_axis_endpoint: Vector2::new(3., 4.),
        minor_axis_ratio: 0.5,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: false,
    }));
    h.paths.push(e);
    let mut p = BoundaryPath::new();
    let mut b = PolylineEdge::new(vec![Vector2::new(0., 0.), Vector2::new(2., 0.)], true);
    b.vertices[0].z = 1.;
    b.vertices[1].z = 1.;
    p.add_edge(BoundaryEdge::Polyline(b));
    h.paths.push(p);
    let mut p = BoundaryPath::new();
    for (s, e) in [(std::f64::consts::PI, 0.), (0., -std::f64::consts::PI)] {
        p.add_edge(BoundaryEdge::CircularArc(CircularArcEdge {
            center: Vector2::ZERO,
            radius: 2.,
            start_angle: s,
            end_angle: e,
            counter_clockwise: false,
        }));
    }
    h.paths.push(p);
    let p = prepare_hatch_from_cad(&h, 1e-9).unwrap();
    assert_eq!(p.boundaries.len(), 4);
    let q =
        prepare_hatch_to_cad(p.placement, &p.boundaries, p.area_rule, p.join_tolerance).unwrap();
    let r = prepare_hatch_from_cad(&q.hatch, 1e-9).unwrap();
    assert_eq!(r.boundaries, p.boundaries);
    assert_eq!(q.pairs.iter().map(|p| p.curves.len()).sum::<usize>(), 6);
}

#[test]
fn flags_are_audited_without_first_loop_role_inference() {
    let mut h = circle();
    h.paths.push(h.paths[0].clone());
    h.paths[1].flags = BoundaryPathFlags::OUTERMOST;
    let p = prepare_hatch_from_cad(&h, 1e-9).unwrap();
    assert_eq!(p.boundaries.len(), 2);
    assert!(p.losses.iter().any(|l| l.field == "paths.flags"));
    let q =
        prepare_hatch_to_cad(p.placement, &p.boundaries, p.area_rule, p.join_tolerance).unwrap();
    assert!(q
        .hatch
        .paths
        .iter()
        .all(|p| p.flags == BoundaryPathFlags::DEFAULT));
    h.paths[1].flags = BoundaryPathFlags::TEXT_ISLAND;
    assert!(matches!(
        prepare_hatch_from_cad(&h, 1e-9),
        Err(CadHatchPreparationError::Unsupported {
            field: "paths.flags",
            ..
        })
    ));
    h.paths[1].flags = BoundaryPathFlags::from_bits(512);
    assert!(prepare_hatch_from_cad(&h, 1e-9).is_err());
}
#[test]
fn spline_and_inactive_construction_fields_receive_explicit_classification() {
    let mut h = circle();
    h.seed_points.push(Vector2::ZERO);
    h.pixel_size = 0.25;
    h.pattern_scale = 2.;
    h.gradient_color.angle = 0.3;
    let p = prepare_hatch_from_cad(&h, 1e-9).unwrap();
    for field in [
        "seed_points",
        "pixel_size",
        "pattern_context",
        "gradient_color",
    ] {
        assert!(p.losses.iter().any(|l| l.field == field));
    }
    h.paths[0].edges = vec![BoundaryEdge::Spline(SplineEdge {
        degree: 3,
        rational: false,
        periodic: false,
        knots: vec![],
        control_points: vec![],
        fit_points: vec![],
        start_tangent: Vector2::ZERO,
        end_tangent: Vector2::ZERO,
    })];
    assert!(matches!(
        prepare_hatch_from_cad(&h, 1e-9),
        Err(CadHatchPreparationError::Unsupported {
            field: "paths.edges.spline",
            ..
        })
    ));
}
#[test]
fn unqualified_annotation_or_extension_context_is_not_an_ordinary_solid() {
    let mut h = circle();
    h.common.xdictionary_handle = Some(Handle::new(99));
    assert!(matches!(
        prepare_hatch_from_cad(&h, 1e-9),
        Err(CadHatchPreparationError::Unsupported {
            field: "common.xdictionary_handle",
            ..
        })
    ));
    h.common.xdictionary_handle = None;
    h.common
        .extended_data
        .add_record(opencadcodec::xdata::ExtendedDataRecord::new(
            "AcadAnnotative",
        ));
    assert!(matches!(
        prepare_hatch_from_cad(&h, 1e-9),
        Err(CadHatchPreparationError::Unsupported {
            field: "common.extended_data",
            ..
        })
    ));
}
#[test]
fn unrepresentable_ellipse_ratio_does_not_emit_degenerate_cad_geometry() {
    let boundaries = vec![HatchBoundary2::Ellipse {
        center: [0., 0.],
        x_axis: [1., 0.],
        semi_major_radius: 1e300,
        semi_minor_radius: 1e-300,
    }];
    assert!(
        prepare_hatch_to_cad(Default::default(), &boundaries, HatchAreaRule::Normal, 1e-9).is_err()
    );
}
#[test]
fn native_partial_arc_is_not_rounded_into_a_full_cad_turn() {
    use ocdraw::geometry_kernel::hatch::HatchEdge2;
    let boundaries = vec![HatchBoundary2::Edges(vec![HatchEdge2::CircularArc {
        center: [0., 0.],
        radius: 1.,
        start_parameter: 2.,
        sweep_parameter: std::f64::consts::TAU.next_down(),
    }])];
    assert!(
        prepare_hatch_to_cad(Default::default(), &boundaries, HatchAreaRule::Normal, 1e-9).is_err()
    );
}
