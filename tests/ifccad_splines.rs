use ifccad::geometry_kernel::{SplineControl, SplineKnots, SplineLimit};
use ifccad::{IfccadSpline, IfccadSplineDefinition, IfccadSplineFitData, IfccadSplineTopology};
use serde_json::{json, Value};

#[test]
fn full_multiplicity_preserves_both_limits() {
    let control = SplineControl {
        degree: 1,
        knots: SplineKnots::Full(vec![0., 0., 1., 1., 2., 2.]),
        points: vec![[0., 0.], [1., 0.], [3., 0.], [4., 0.]],
        weights: None,
    };
    let prepared = control.prepare().unwrap();

    assert_eq!(
        prepared.evaluate_limit(1., SplineLimit::Left).unwrap(),
        [1., 0.]
    );
    assert_eq!(prepared.evaluate(1.).unwrap(), [3., 0.]);
    assert_eq!(prepared.evaluate(2.).unwrap(), [4., 0.]);
}

#[test]
fn rational_quarter_circle_evaluates_in_its_knot_domain() {
    let control = SplineControl {
        degree: 2,
        knots: SplineKnots::Full(vec![2., 2., 2., 6., 6., 6.]),
        points: vec![[1., 0.], [1., 1.], [0., 1.]],
        weights: Some(vec![1., std::f64::consts::FRAC_1_SQRT_2, 1.]),
    };
    let prepared = control.prepare().unwrap();
    let midpoint = prepared.evaluate(4.).unwrap();

    assert_eq!(prepared.domain(), [2., 6.]);
    for component in midpoint {
        assert!((component - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-15);
    }
    assert!(prepared.evaluate(1.).is_err());
}

#[test]
fn nonuniform_periodic_cycle_preserves_its_seam() {
    let control = SplineControl {
        degree: 3,
        knots: SplineKnots::PeriodicCycle(vec![2., 2.5, 4., 7., 9.]),
        points: vec![[0., 0.], [3., -2.], [5., 4.], [-1., 2.]],
        weights: Some(vec![1., 2., 0.75, 1.25]),
    };
    let prepared = control.prepare().unwrap();

    assert_eq!(prepared.domain(), [2., 9.]);
    let start = prepared.evaluate(2.).unwrap();
    let end = prepared.evaluate(9.).unwrap();
    for (a, b) in start.into_iter().zip(end) {
        assert!((a - b).abs() < 1e-14, "{a} differs from {b}");
    }
}

#[test]
fn fit_only_is_valid_with_no_control_basis() {
    let mut spline = IfccadSpline::new(
        IfccadSplineTopology::Open,
        IfccadSplineDefinition::FitOnly {
            degree: Some(3),
            fit: IfccadSplineFitData {
                points: vec![[0., 0., 0.], [2., 3., 1.]],
                start_tangent: Some([1., 0., 0.]),
                end_tangent: None,
                tolerance: Some(0.),
                parameterization: None,
            },
        },
    );
    spline.validate().unwrap();
    assert!(spline.control_basis().is_none());
    spline.knot_tolerance = Some(-1.);
    assert!(spline.validate().is_err());
}

#[test]
fn standalone_spline_preserves_definition_and_optional_z() {
    let mut source: Value =
        serde_json::from_slice(include_bytes!("fixtures/ifccad/hello-cad.ifcx")).unwrap();
    let entity = source["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1/e1")
        .unwrap();
    let attributes = entity["attributes"].as_object_mut().unwrap();
    attributes.remove("ifccad::geom::lineSegment");
    attributes.insert(
        "ifccad::geom::spline".into(),
        json!({
            "topology": "open",
            "definition": {
                "kind": "control",
                "basis": {
                    "degree": 1, "knotKind": "full", "knots": [0, 0, 1, 2, 2],
                    "points": [[0, 0], [1, 2, -0.0], [3, 4, 5]]
                },
                "fit": { "points": [[0, 0, 0], [3, 4, 5]], "tolerance": 0 }
            }
        }),
    );
    let input = serde_json::to_vec(&source).unwrap();
    let read = ifccad::load_ifccad_bytes(&input, Default::default()).unwrap();
    let document = read.document();
    let ifcx = ifccad::encode_ifccad_document(document).unwrap();
    let cbor = ifccad::encode_ifccad_cbor_document(document).unwrap();
    let ifcx_read = ifccad::load_ifccad_bytes(ifcx.bytes(), Default::default()).unwrap();
    let cbor_read = ifccad::load_ifccad_cbor_bytes(cbor.bytes(), Default::default()).unwrap();
    for drawing in [ifcx_read.document(), cbor_read.document()] {
        let entity = drawing.model.entities[0].as_native().unwrap();
        let ifccad::IfccadEntityKind::Spline(spline) = &entity.kind else {
            panic!("expected native spline");
        };
        let basis = spline.control_basis().unwrap();
        assert_eq!(basis.points[0][2].to_bits(), 0.0_f64.to_bits());
        assert_eq!(basis.points[1][2].to_bits(), (-0.0_f64).to_bits());
        assert_eq!(basis.points[2][2], 5.);
        assert!(matches!(
            spline.definition,
            IfccadSplineDefinition::Control { fit: Some(_), .. }
        ));
    }
}

#[test]
fn spline_clip_tests_curve_instead_of_control_hull() {
    use ifccad::geometry_kernel::{
        validate_paper_boundary, GeometryRef, GeometryValidationError, PaperFrame,
    };
    let mut control = SplineControl {
        degree: 2,
        knots: SplineKnots::Full(vec![0., 0., 0., 1., 1., 1., 2., 2., 2.]),
        points: vec![
            [0., 0., 0.],
            [2.5, 0.5, 0.],
            [0., 1., 0.],
            [0., 1., 0.],
            [-0.5, 0.5, 0.],
            [0., 0., 0.],
        ],
        weights: None,
    };
    let frame = PaperFrame {
        center: [0.25, 0.5],
        width: 2.5,
        height: 1.5,
    };
    let check = |control: &SplineControl<3>| {
        validate_paper_boundary(
            frame,
            GeometryRef::Spline {
                control,
                closed: true,
            },
        )
    };
    check(&control).unwrap();
    control.points[1][0] = 3.;
    check(&control).unwrap();
    control.points[1][0] = 3.25;
    assert_eq!(
        check(&control),
        Err(GeometryValidationError::BoundaryOutsideFrame)
    );
    control.points[1][0] = 2.5;
    control.points[3][0] = 0.25;
    assert_eq!(
        check(&control),
        Err(GeometryValidationError::UnsupportedBoundary)
    );
}

#[test]
fn hatch_spline_edge_joins_line_at_exact_rational_endpoint() {
    use ifccad::geometry_kernel::hatch::{validate_hatch_boundaries, HatchBoundary2, HatchEdge2};
    let control = SplineControl {
        degree: 2,
        knots: SplineKnots::Full(vec![0., 0., 0., 1., 1., 1.]),
        points: vec![[0., 0.], [1., 2.], [2., 0.]],
        weights: Some(vec![1., 0.75, 1.]),
    };
    let edges = HatchBoundary2::Edges(vec![
        HatchEdge2::Spline {
            control,
            fit: None,
            periodic: false,
        },
        HatchEdge2::Line {
            start: [2., 0.],
            end: [0., 0.],
        },
    ]);
    validate_hatch_boundaries([&edges], 0.).unwrap();
}
