use cad_dimension::*;
use ifccad::geometry_kernel::Point3;
use opencadcodec::{
    entities::{Dimension, DimensionAligned},
    Vector3,
};
fn dim(normal: Vector3, elevation: f64) -> Dimension {
    let mut d = DimensionAligned::new(Vector3::ZERO, Vector3::new(5., 0., 0.));
    d.definition_point = Vector3::new(10., 30., 4.);
    d.base.definition_point = d.definition_point;
    d.base.normal = normal;
    d.base.text_middle_point = Vector3::new(2., 7., elevation);
    Dimension::Aligned(d)
}
#[test]
fn owner_picture_does_not_add_definition_point() {
    assert!(matches!(
        prepare_dimension_picture_frame(&dim(Vector3::UNIT_Z, 4.), [0.; 3]).unwrap(),
        CadDimensionPictureFrame::Owner
    ));
}
#[test]
fn planar_cache_frames_have_literal_tilted_negative_z_and_elevation_oracles() {
    for (normal, elevation, want) in [
        (Vector3::new(1., 0., 0.), 3., [3., 2., 7.]),
        (Vector3::new(0., 0., -1.), 5., [-2., 7., -5.]),
    ] {
        let CadDimensionPictureFrame::PlanarOcs(p) =
            prepare_dimension_picture_frame(&dim(normal, elevation), [0.; 3]).unwrap()
        else {
            panic!()
        };
        assert_eq!(
            p.try_to_scope_spatial_point(Point3::new(2., 7., 0.))
                .unwrap()
                .components(),
            want
        );
    }
}
#[test]
fn cloned_nonzero_base_and_invalid_frames_are_refused() {
    let mut d = dim(Vector3::UNIT_Z, 0.);
    assert!(matches!(
        prepare_dimension_picture_frame(&d, [1., 0., 0.]),
        Err(CadDimensionPreparationError::Unsupported { .. })
    ));
    d.base_mut().insertion_rotation = 0.5;
    assert!(prepare_dimension_picture_frame(&d, [0.; 3]).is_err());
    let d = dim(Vector3::ZERO, 0.);
    assert!(matches!(
        prepare_dimension_picture_frame(&d, [0.; 3]),
        Err(CadDimensionPreparationError::Invalid { .. })
    ));
    let d = dim(Vector3::UNIT_X, f64::NAN);
    assert!(prepare_dimension_picture_frame(&d, [0.; 3]).is_err());
}
