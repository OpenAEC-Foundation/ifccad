mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{EntityType, Vector3};

#[test]
fn cad_families_map_to_independent_native_kinds_and_prepare_bounds() {
    let mut c = cad();
    let mut point = opencadcodec::Point::from_coords(1., 2., 3.);
    point.normal = Vector3::new(0., 1., 0.);
    let arc =
        opencadcodec::Arc::from_center_radius_angles(Vector3::new(0., 0., 0.), 2., 0.25, 1.25);
    let mut ellipse = opencadcodec::Ellipse::new();
    ellipse.center = Vector3::new(3., 4., 0.);
    ellipse.major_axis = Vector3::new(3., 4., 0.);
    ellipse.minor_axis_ratio = 0.5;
    let mut poly = opencadcodec::LwPolyline::from_points(vec![
        opencadcodec::Vector2::new(-1., 0.),
        opencadcodec::Vector2::new(1., 0.),
    ]);
    poly.is_closed = true;
    poly.vertices[0].bulge = 1.;
    poly.vertices[1].bulge = 1.;
    let spatial = opencadcodec::entities::Polyline3D::from_points(vec![
        Vector3::new(0., 0., 0.),
        Vector3::new(1., 2., 3.),
    ]);
    for e in [
        EntityType::Point(point),
        EntityType::Arc(arc),
        EntityType::Ellipse(ellipse),
        EntityType::LwPolyline(poly),
        EntityType::Polyline3D(spatial),
    ] {
        c.add_entity(e).unwrap();
    }
    let out = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    assert!(d.model.bounds.is_some());
    for family in 0..5 {
        assert!(d.model.entities.iter().any(|e| matches!(
            (&e.as_native().unwrap().kind, family),
            (IfccadEntityKind::Point { .. }, 0)
                | (IfccadEntityKind::Arc { .. }, 1)
                | (IfccadEntityKind::Ellipse { .. }, 2)
                | (IfccadEntityKind::PlanarPolyline { .. }, 3)
                | (IfccadEntityKind::SpatialPolyline { .. }, 4)
        )));
    }
    validate_ifccad_document(d).unwrap();
}
#[test]
fn ellipse_rounding_is_accepted_by_coordinate_default_but_exact_remains_strict() {
    let mut c = cad();
    c.header.insertion_units = 0;
    let mut ellipse = opencadcodec::Ellipse::new();
    ellipse.major_axis = Vector3::new(3., 4., 0.);
    ellipse.minor_axis_ratio = 0.5;
    c.add_entity(EntityType::Ellipse(ellipse)).unwrap();
    let exact = cad_document_to_ifccad_document(
        &c,
        metadata(),
        CadToIfccadOptions {
            geometry_tolerance: IfccadGeometryTolerance::exact(),
            ..Default::default()
        },
    );
    assert!(matches!(exact, Err(IfccadConversionError::Geometry(_))));
    assert!(cad_document_to_ifccad_document(&c, metadata(), Default::default()).is_ok());
    let out = cad_document_to_ifccad_document(
        &c,
        metadata(),
        CadToIfccadOptions {
            geometry_tolerance: IfccadGeometryTolerance::drawing_units(1e-8).unwrap(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        out.geometry_assessment().status(),
        IfccadGeometryStatus::RoundedWithinTolerance
    );
}

#[test]
fn amplified_nested_curve_uncertainty_returns_no_partial_document() {
    let native = nested([0.; 3]);
    let mut c = ifccad_document_to_cad_document(&native, Default::default())
        .unwrap()
        .into_document();
    let handle = c.block_records.get("Inner").unwrap().entity_handles[0];
    let mut ellipse = opencadcodec::Ellipse::new();
    ellipse.common = c.get_entity(handle).unwrap().common().clone();
    ellipse.major_axis = Vector3::new(3., 4., 0.);
    ellipse.minor_axis_ratio = 0.5;
    *c.get_entity_mut(handle).unwrap() = EntityType::Ellipse(ellipse);
    assert!(cad_document_to_ifccad_document(&c, metadata(), Default::default()).is_ok());
    let roots = c
        .block_records
        .get("*Model_Space")
        .unwrap()
        .entity_handles
        .clone();
    for handle in roots {
        if let Some(EntityType::Insert(i)) = c.get_entity_mut(handle) {
            i.set_x_scale(1e15);
            i.set_y_scale(1e15);
            i.set_z_scale(1e15);
        }
    }
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let result = cad_document_to_ifccad_document(
            &c,
            metadata(),
            CadToIfccadOptions {
                loss_policy,
                ..Default::default()
            },
        );
        let Err(IfccadConversionError::Geometry(failure)) = result else {
            panic!("amplified residual must fail the hard limit")
        };
        assert!(
            matches!(&failure.failure.source,IfccadGeometryEntitySource::BlockOccurrence{path,..} if path.len()>=2)
        );
    }
}
