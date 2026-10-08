use ocdraw::geometry_kernel::hatch::{
    hatch_bounds, validate_hatch_boundaries, HatchBoundary2, HatchEdge2, HatchFailureReason,
    DEFAULT_HATCH_JOIN_TOLERANCE,
};
use ocdraw::geometry_kernel::{CoordinateFrame3, Point3, Vector3};

#[test]
fn circle_enclosure_is_exact_for_identity_plane() {
    let boundary = HatchBoundary2::Circle {
        center: [2.0, 3.0],
        radius: 1.0,
    };
    let bounds = hatch_bounds(CoordinateFrame3::default(), [&boundary]).unwrap();
    assert_eq!(bounds.min().components(), [1.0, 2.0, 0.0]);
    assert_eq!(bounds.max().components(), [3.0, 4.0, 0.0]);
}

#[test]
fn rotated_ellipse_and_tilted_plane_enclose_full_curves() {
    let boundary = HatchBoundary2::Ellipse {
        center: [1.0, 2.0],
        x_axis: [0.0, 1.0],
        semi_major_radius: 5.0,
        semi_minor_radius: 2.0,
    };
    let bounds = hatch_bounds(CoordinateFrame3::default(), [&boundary]).unwrap();
    assert_eq!(bounds.min().components(), [-1.0, -3.0, 0.0]);
    assert_eq!(bounds.max().components(), [3.0, 7.0, 0.0]);
    let plane = CoordinateFrame3::try_new(
        Point3::new(10.0, 20.0, 30.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    let circle = HatchBoundary2::Circle {
        center: [2.0, 3.0],
        radius: 1.0,
    };
    let bounds = hatch_bounds(plane, [&circle]).unwrap();
    for (actual, expected) in bounds
        .min()
        .components()
        .into_iter()
        .zip([10.0, 22.0, 31.0])
    {
        assert!(actual <= expected && expected - actual < 1e-12);
    }
    for (actual, expected) in bounds
        .max()
        .components()
        .into_iter()
        .zip([10.0, 24.0, 33.0])
    {
        assert!(actual >= expected && actual - expected < 1e-12);
    }
}

fn chain_with_gap(gap: f64) -> HatchBoundary2 {
    HatchBoundary2::Edges(vec![
        HatchEdge2::Line {
            start: [-1.0, 0.0],
            end: [0.0, gap],
        },
        HatchEdge2::Line {
            start: [0.0, 0.0],
            end: [1.0, 0.0],
        },
        HatchEdge2::Line {
            start: [1.0, 0.0],
            end: [-1.0, 0.0],
        },
    ])
}

#[test]
fn joins_are_inclusive_and_locate_the_rejected_gap() {
    validate_hatch_boundaries([&chain_with_gap(0.125)], 0.125).unwrap();
    validate_hatch_boundaries([&chain_with_gap(0.0)], 0.0).unwrap();
    let error = validate_hatch_boundaries([&chain_with_gap(0.25)], 0.125).unwrap_err();
    assert_eq!(error.loop_index, Some(0));
    assert_eq!(error.edge_index, Some(0));
    assert!(matches!(
        error.reason,
        HatchFailureReason::GapExceeded { .. }
    ));
}

fn arc_with_rounded_chord() -> HatchBoundary2 {
    HatchBoundary2::Edges(vec![
        HatchEdge2::CircularArc {
            center: [0.0, 0.0],
            radius: 1.0,
            start_parameter: 0.0,
            sweep_parameter: 1.0,
        },
        HatchEdge2::Line {
            start: [0.5403023058681398, 0.8414709848078965],
            end: [1.0, 0.0],
        },
    ])
}

#[test]
fn curved_join_without_proof_is_distinct_from_proven_exceedance() {
    let boundary = arc_with_rounded_chord();
    let error = validate_hatch_boundaries([&boundary], 0.0).unwrap_err();
    assert_eq!(error.reason, HatchFailureReason::JoinProofIncomplete);
    validate_hatch_boundaries([&boundary], DEFAULT_HATCH_JOIN_TOLERANCE).unwrap();
}

#[test]
fn shared_analytic_endpoint_expressions_close_in_exact_mode() {
    let boundary = HatchBoundary2::Edges(vec![
        HatchEdge2::CircularArc {
            center: [0.0, 0.0],
            radius: 1.0,
            start_parameter: 0.0,
            sweep_parameter: 1.0,
        },
        HatchEdge2::CircularArc {
            center: [0.0, 0.0],
            radius: 1.0,
            start_parameter: 1.0,
            sweep_parameter: -1.0,
        },
    ]);
    validate_hatch_boundaries([&boundary], 0.0).unwrap();
}

#[test]
fn crossing_contours_are_not_rejected_by_a_deferred_audit() {
    let boundary = HatchBoundary2::Polyline {
        vertices: vec![[0.0, 0.0], [2.0, 2.0], [0.0, 2.0], [2.0, 0.0]],
        bulges: vec![0.0; 4],
    };
    validate_hatch_boundaries([&boundary], 0.0).unwrap();
    let bounds = hatch_bounds(CoordinateFrame3::default(), [&boundary]).unwrap();
    assert_eq!(bounds.min().components(), [0.0, 0.0, 0.0]);
    assert_eq!(bounds.max().components(), [2.0, 2.0, 0.0]);
}

#[test]
fn bulged_two_vertex_contour_encloses_both_semicircles() {
    let boundary = HatchBoundary2::Polyline {
        vertices: vec![[0.0, 0.0], [2.0, 0.0]],
        bulges: vec![1.0, 1.0],
    };
    validate_hatch_boundaries([&boundary], 0.0).unwrap();
    let bounds = hatch_bounds(CoordinateFrame3::default(), [&boundary]).unwrap();
    assert_eq!(bounds.min().components(), [0.0, -1.0, 0.0]);
    assert_eq!(bounds.max().components(), [2.0, 1.0, 0.0]);
}

#[test]
fn invalid_parameters_empty_contours_and_tolerances_fail() {
    assert!(validate_hatch_boundaries([], 0.0).is_err());
    let invalid = [
        HatchBoundary2::Circle {
            center: [0.0, 0.0],
            radius: -1.0,
        },
        HatchBoundary2::Ellipse {
            center: [0.0, 0.0],
            x_axis: [2.0, 0.0],
            semi_major_radius: 2.0,
            semi_minor_radius: 1.0,
        },
        HatchBoundary2::Polyline {
            vertices: vec![[0.0, 0.0], [0.0, 0.0]],
            bulges: vec![1.0, 1.0],
        },
        HatchBoundary2::Edges(vec![]),
    ];
    for b in &invalid {
        assert!(validate_hatch_boundaries([b], 0.0).is_err());
    }
    let b = HatchBoundary2::Circle {
        center: [0.0, 0.0],
        radius: 1.0,
    };
    for limit in [-1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            validate_hatch_boundaries([&b], limit).unwrap_err().reason,
            HatchFailureReason::InvalidTolerance,
        );
    }
}

#[test]
fn a_closed_quarter_circle_does_not_require_bounds_for_the_other_quadrants() {
    let boundary = HatchBoundary2::Edges(vec![
        HatchEdge2::CircularArc {
            center: [0.0, 0.0],
            radius: 1.0,
            start_parameter: 0.0,
            sweep_parameter: std::f64::consts::FRAC_PI_2,
        },
        HatchEdge2::Line {
            start: [0.0, 1.0],
            end: [0.0, 0.0],
        },
        HatchEdge2::Line {
            start: [0.0, 0.0],
            end: [1.0, 0.0],
        },
    ]);
    validate_hatch_boundaries([&boundary], DEFAULT_HATCH_JOIN_TOLERANCE).unwrap();
    let bounds = hatch_bounds(CoordinateFrame3::default(), [&boundary]).unwrap();
    assert_eq!(bounds.min().components(), [0.0, 0.0, 0.0]);
    assert_eq!(bounds.max().components(), [1.0, 1.0, 0.0]);
}

#[test]
fn mixed_circle_ellipse_and_line_edges_use_their_actual_parameterizations() {
    let b = HatchBoundary2::Edges(vec![
        HatchEdge2::CircularArc {
            center: [0.0, 0.0],
            radius: 1.0,
            start_parameter: 0.0,
            sweep_parameter: std::f64::consts::FRAC_PI_2,
        },
        HatchEdge2::EllipticArc {
            center: [0.0, 0.0],
            x_axis: [1.0, 0.0],
            semi_major_radius: 2.0,
            semi_minor_radius: 1.0,
            start_parameter: std::f64::consts::FRAC_PI_2,
            sweep_parameter: std::f64::consts::FRAC_PI_2,
        },
        HatchEdge2::Line {
            start: [-2.0, 0.0],
            end: [1.0, 0.0],
        },
    ]);
    validate_hatch_boundaries([&b], DEFAULT_HATCH_JOIN_TOLERANCE).unwrap();
    let bounds = hatch_bounds(CoordinateFrame3::default(), [&b]).unwrap();
    assert_eq!(bounds.min().x(), -2.0);
    assert!(bounds.min().y() > -1e-12 && bounds.min().y() <= 0.0);
    assert_eq!(bounds.max().x(), 1.0);
    assert_eq!(bounds.max().y(), 1.0);
}

#[test]
fn invalid_partial_sweeps_and_nonfinite_vertices_fail_before_arithmetic() {
    for sweep in [0.0, -0.0, std::f64::consts::TAU, f64::NAN, f64::INFINITY] {
        let b = HatchBoundary2::Edges(vec![HatchEdge2::CircularArc {
            center: [0.0, 0.0],
            radius: 1.0,
            start_parameter: 0.0,
            sweep_parameter: sweep,
        }]);
        assert_eq!(
            validate_hatch_boundaries([&b], 1.0).unwrap_err().reason,
            HatchFailureReason::InvalidParameters
        );
    }
    let b = HatchBoundary2::Polyline {
        vertices: vec![[0.0, 0.0], [f64::NAN, 0.0], [1.0, 1.0]],
        bulges: vec![0.0; 3],
    };
    assert_eq!(
        validate_hatch_boundaries([&b], 0.0).unwrap_err().reason,
        HatchFailureReason::InvalidParameters
    );
}
