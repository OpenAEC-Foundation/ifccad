use ocdraw::geometry_kernel::{
    geometry_bounds, validate_geometry, validate_paper_boundary, CoordinateFrame3, GeometryRef,
    PaperFrame, PlanarVertices,
};

#[test]
fn packed_and_separate_bulges_have_identical_geometry() {
    let packed = [[-1., 0., 1.], [1., 0., 1.]];
    let xy = [[-1., 0.], [1., 0.]];
    let bulges = [1., 1.];
    let contour = |vertices| GeometryRef::PlanarPolyline {
        placement: CoordinateFrame3::default(),
        vertices,
        closed: true,
    };
    let frame = PaperFrame {
        center: [0., 0.],
        width: 2.,
        height: 2.,
    };
    let a = contour(PlanarVertices::Packed(&packed));
    let b = contour(PlanarVertices::Separate {
        xy: &xy,
        bulges: &bulges,
    });
    assert_eq!(geometry_bounds(a).unwrap(), geometry_bounds(b).unwrap());
    assert!(validate_paper_boundary(frame, a).is_ok());
    assert!(validate_paper_boundary(frame, b).is_ok());
    let straight = [0., 0.];
    assert!(validate_paper_boundary(
        frame,
        contour(PlanarVertices::Separate {
            xy: &xy,
            bulges: &straight
        })
    )
    .is_err());
}

#[test]
fn ellipse_tangency_uses_entire_curve_without_an_epsilon() {
    let frame = PaperFrame {
        center: [0., 0.],
        width: 6.,
        height: 4.,
    };
    let ellipse = |minor| GeometryRef::Ellipse {
        placement: CoordinateFrame3::default(),
        major: 3.,
        minor,
        arc: None,
    };
    assert!(validate_paper_boundary(frame, ellipse(2.)).is_ok());
    assert!(validate_paper_boundary(frame, ellipse(2.0_f64.next_up())).is_err());
}

#[test]
fn dormant_final_bulge_does_not_enlarge_an_open_path() {
    let vertices = [[0., 0.], [2., 0.]];
    let bulges = [0., 7.];
    let path = GeometryRef::PlanarPolyline {
        placement: CoordinateFrame3::default(),
        vertices: PlanarVertices::Separate {
            xy: &vertices,
            bulges: &bulges,
        },
        closed: false,
    };
    let bounds = geometry_bounds(path).unwrap();
    assert_eq!(bounds.min().components(), [0., 0., 0.]);
    assert_eq!(bounds.max().components(), [2., 0., 0.]);
}

#[test]
fn invalid_parameters_fail_before_exact_arithmetic() {
    for radius in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(validate_geometry(GeometryRef::Circle {
            placement: CoordinateFrame3::default(),
            radius,
        })
        .is_err());
    }
    for sweep in [0., std::f64::consts::TAU, f64::NAN] {
        assert!(validate_geometry(GeometryRef::Arc {
            placement: CoordinateFrame3::default(),
            radius: 1.,
            start: 0.,
            sweep,
        })
        .is_err());
    }
    assert!(validate_geometry(GeometryRef::Arc {
        placement: CoordinateFrame3::default(),
        radius: 1.,
        start: 0.,
        sweep: std::f64::consts::TAU.next_down(),
    })
    .is_ok());
    let xy = [[0., 0.], [0., 0.]];
    assert!(validate_geometry(GeometryRef::PlanarPolyline {
        placement: CoordinateFrame3::default(),
        vertices: PlanarVertices::Separate {
            xy: &xy,
            bulges: &[1., 0.]
        },
        closed: false,
    })
    .is_err());
}

#[test]
fn drawing_units_and_coordinate_types_keep_existing_public_imports() {
    use ocdraw::geometry_kernel::CoordinateLengthUnit;
    use ocdraw::ocdraw::DrawingLengthUnit;
    let registry: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/ocdraw/registry-0.1.0.json")).unwrap();
    for token in registry["types"]["unit"]["values"].as_array().unwrap() {
        assert!(CoordinateLengthUnit::from_token(token.as_str().unwrap()).is_some());
    }
    assert_eq!(
        CoordinateLengthUnit::from_token("mm"),
        Some(DrawingLengthUnit::Millimetre)
    );
    assert_eq!(CoordinateLengthUnit::from_token("metre"), None);
    let legacy: ocdraw::ocdraw::CoordinateFrame3 = CoordinateFrame3::default();
    assert_eq!(legacy.origin().components(), [0.; 3]);
}

#[test]
fn the_native_model_exposes_a_borrowed_geometry_view() {
    let circle = ocdraw::ocdraw::DrawingGeometry::Circle {
        placement: CoordinateFrame3::default(),
        radius: 2.,
    };
    assert!(geometry_bounds(circle.as_shared_geometry().unwrap()).is_ok());
}
