use cad_geometry_convert::geometry::blocks::EvaluatedBlock;
use cad_geometry_convert::*;
use ifccad::geometry_kernel::*;
fn map(origin: [f64; 3]) -> EvaluatedBlock {
    EvaluatedBlock::native(
        BlockTransform::try_new(
            CoordinateFrame3::try_new(
                Point3::new(origin[0], origin[1], origin[2]),
                Vector3::new(1., 0., 0.),
                Vector3::new(0., 1., 0.),
            )
            .unwrap(),
            0.,
            Scale3::new(1., 1., 1.),
        )
        .unwrap(),
        [0.; 3],
    )
}
#[test]
fn picture_pair_compares_source_to_native_and_proves_actual_placement() {
    let original = GeometryPair::points(&[[100., 9., 0.]], &[[100., 9., 0.]]).unwrap();
    for (x, want) in [(2., 0), (3., 1)] {
        // The second target is deliberately unrelated: only its native source is selected.
        let native = GeometryPair::points(&[[x, 9., 0.]], &[[999., 9., 0.]]).unwrap();
        let mut pair = GeometryPair::from_prepared_sources(&original, &native).unwrap();
        pair.apply(&map([0.; 3]), &map([98., 0., 0.]));
        assert_eq!(
            pair.points[0].0.squared_deviation().1,
            num_rational::BigRational::from_integer(want.into())
        );
    }
}
#[test]
fn picture_pair_rejects_mismatched_sample_topology() {
    let a = GeometryPair::points(&[[0.; 3]], &[[0.; 3]]).unwrap();
    let b = GeometryPair::points(&[[0.; 3]; 2], &[[0.; 3]; 2]).unwrap();
    assert!(GeometryPair::from_prepared_sources(&a, &b).is_err());
}
