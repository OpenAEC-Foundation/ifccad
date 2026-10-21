use cad_geometry_convert::geometry::filled::common_filled_surface_cover;
use cad_geometry_convert::{prepare_from_cad, prepare_to_cad, CadPreparationError};
use ifccad::geometry_kernel::*;
use opencadcodec::entities::Solid;
use opencadcodec::{EntityType, Vector3 as V};
fn cases() -> Vec<[[f64; 2]; 4]> {
    let v: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/geometry/fixtures/solid-trace-profile/cases.json"
    ))
    .unwrap();
    v["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| serde_json::from_value(v["corners"].clone()).unwrap())
        .collect()
}
#[test]
fn filled_preparation_keeps_all_finite_topologies() {
    for c in cases() {
        for h in [0., 3., -3.] {
            for flag in [false, true] {
                let v = c.map(|c| V::new(c[0], c[1], 5.));
                let mut s = Solid::new(v[0], v[1], v[2], v[3]).with_thickness(h);
                s.is_trace = flag;
                let n = prepare_from_cad(&EntityType::Solid(s)).unwrap();
                assert_eq!(n.pair.points.len(), 8);
                assert!(n.pair.curves.is_empty());
                let out = prepare_to_cad(n.geometry.as_ref()).unwrap();
                assert_eq!(out.pair.points.len(), 8);
                for (p, _) in &out.pair.points {
                    assert_eq!(
                        p.squared_deviation().1,
                        num_rational::BigRational::from_integer(0.into())
                    );
                }
                let EntityType::Solid(out) = out.entity else {
                    panic!()
                };
                assert_eq!(out.third_corner, v[2]);
                assert_eq!(out.fourth_corner, v[3]);
                assert_eq!(out.thickness, h);
            }
        }
    }
}
#[test]
fn filled_surface_cover_is_optional() {
    let c = cases();
    assert_eq!(common_filled_surface_cover(&c[2], &c[3]), None);
    assert!(common_filled_surface_cover(&c[2], &c[2]).is_some());
    assert!(common_filled_surface_cover(&[[4., 4.]; 4], &[[4., 4.]; 4]).is_none());
}
#[test]
fn filled_records_are_not_repaired() {
    let mut s = Solid::triangle(V::ZERO, V::UNIT_X, V::UNIT_Y);
    s.fourth_corner.z = 1.;
    assert!(matches!(
        prepare_from_cad(&EntityType::Solid(s.clone())),
        Err(CadPreparationError::UnsupportedGeometry)
    ));
    s.first_corner.x = f64::NAN;
    assert!(matches!(
        prepare_from_cad(&EntityType::Solid(s)),
        Err(CadPreparationError::InvalidGeometry)
    ));
}
#[test]
fn filled_pairs_use_stored_axis_cross_product() {
    let p = CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(1. + 2e-13, 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let c = [[0., 0.]; 4];
    let out = prepare_to_cad(GeometryRef::FilledQuadrilateral {
        placement: p,
        corners: &c,
        thickness: 1e8,
    })
    .unwrap();
    assert_eq!(out.pair.points.len(), 8);
    assert_ne!(
        out.pair.points[4].0.squared_deviation().1,
        num_rational::BigRational::from_integer(0.into())
    );
}
