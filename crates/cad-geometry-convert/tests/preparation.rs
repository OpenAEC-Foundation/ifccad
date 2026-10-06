use cad_geometry_convert::{prepare_from_cad, prepare_to_cad};
use ocdraw::geometry_kernel::{CoordinateFrame3, GeometryRef, OwnedGeometry};
use opencadcodec::{Circle, EntityType, Vector3};

#[test]
fn cad_circle_prepares_pure_geometry_without_a_format_document() {
    let circle = Circle::from_center_radius(Vector3::new(3., 4., 5.), 2.);
    let prepared = prepare_from_cad(&EntityType::Circle(circle)).unwrap();
    let OwnedGeometry::Circle { placement, radius } = prepared.geometry else {
        panic!("circle kind")
    };
    assert_eq!(placement.origin().components(), [3., 4., 5.]);
    assert_eq!(radius, 2.);
}

#[test]
fn nonfinite_cad_geometry_returns_a_preparation_error() {
    let circle = Circle::from_center_radius(Vector3::new(f64::INFINITY, 0., 0.), 2.);
    assert!(prepare_from_cad(&EntityType::Circle(circle)).is_err());
}

#[test]
fn signed_native_arcs_have_a_supported_cad_parameterization() {
    let prepared = prepare_to_cad(GeometryRef::Arc {
        placement: CoordinateFrame3::default(),
        radius: 2.,
        start: 0.25,
        sweep: -1.,
    })
    .unwrap();
    let EntityType::Arc(arc) = prepared.entity else {
        panic!("arc kind")
    };
    assert!(arc.end_angle > arc.start_angle);
    assert_eq!(arc.end_angle - arc.start_angle, 1.);
    assert_eq!(arc.normal, Vector3::new(0., 0., -1.));
}

#[test]
fn bulged_contours_include_full_segment_curve_evidence() {
    let vertices = [[-1., 0., 1.], [1., 0., 1.]];
    let prepared = prepare_to_cad(GeometryRef::PlanarPolyline {
        placement: CoordinateFrame3::default(),
        vertices: ocdraw::geometry_kernel::PlanarVertices::Packed(&vertices),
        closed: true,
    })
    .unwrap();
    assert_eq!(prepared.pair.curves.len(), 2);
}

#[test]
fn malformed_curve_parameters_do_not_panic_during_preparation() {
    let mut ellipse = opencadcodec::Ellipse::new();
    ellipse.end_parameter = f64::NAN;
    let mut polyline = opencadcodec::LwPolyline::new();
    polyline.elevation = f64::INFINITY;
    let arc = opencadcodec::Arc::from_center_radius_angles(Vector3::new(0., 0., 0.), 1., 0., 0.);
    for entity in [
        EntityType::Ellipse(ellipse),
        EntityType::LwPolyline(polyline),
        EntityType::Arc(arc),
    ] {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prepare_from_cad(&entity)));
        assert!(result.is_ok(), "invalid scalar must return a typed error");
        assert!(result.unwrap().is_err());
    }
}

#[test]
fn rounded_away_curve_parameters_cannot_be_emitted_as_degenerate_cad_curves() {
    let plane = CoordinateFrame3::default();
    let distant = CoordinateFrame3::try_new(
        ocdraw::geometry_kernel::Point3::new(1e20, 0., 0.),
        ocdraw::geometry_kernel::Vector3::new(1., 0., 0.),
        ocdraw::geometry_kernel::Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let vertices = [[0., 0., 1.], [0.0001, 0., 0.]];
    for g in [
        GeometryRef::Arc {
            placement: plane,
            radius: 1.,
            start: 1.,
            sweep: 1e-30,
        },
        GeometryRef::Ellipse {
            placement: plane,
            major: 2.,
            minor: 1.,
            arc: Some((1., 1e-30)),
        },
        GeometryRef::Ellipse {
            placement: plane,
            major: 1e20,
            minor: 1e-320,
            arc: None,
        },
        GeometryRef::PlanarPolyline {
            placement: distant,
            vertices: ocdraw::geometry_kernel::PlanarVertices::Packed(&vertices),
            closed: false,
        },
    ] {
        assert!(
            prepare_to_cad(g).is_err(),
            "curve parameters must remain valid in the target representation"
        );
    }
}
