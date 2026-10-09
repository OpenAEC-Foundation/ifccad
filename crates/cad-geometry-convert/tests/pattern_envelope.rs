use cad_geometry_convert::geometry::{
    blocks::{EvaluatedBlock, PairedPoint, Range},
    numeric::exact,
};
use ocdraw::geometry_kernel::{BlockTransform, CoordinateFrame3, Scale3};
#[test]
fn ranged_certificate_points_propagate_through_occurrences() {
    let mut p = PairedPoint::from_ranges(
        [
            Range::point(exact(1.)),
            Range::point(exact(0.)),
            Range::point(exact(0.)),
        ],
        [
            Range::point(exact(0.)),
            Range::point(exact(0.)),
            Range::point(exact(0.)),
        ],
    );
    assert_eq!(p.squared_deviation().1, exact(1.));
    let t =
        BlockTransform::try_new(CoordinateFrame3::default(), 0., Scale3::new(2., 3., 1.)).unwrap();
    let e = EvaluatedBlock::native(t, [0., 0., 0.]);
    p.apply(&e, &e);
    assert_eq!(p.squared_deviation().1, exact(4.));
}
