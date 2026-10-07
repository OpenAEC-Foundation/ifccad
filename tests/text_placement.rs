use ocdraw::geometry_kernel::{CoordinateFrame3, Point2, Point3, Vector3};

#[test]
fn arbitrary_axis_constructor_has_independent_axis_reference_cases() {
    for (normal, expected_x, expected_y) in [
        ([0., 0., 1.], [1., 0., 0.], [0., 1., 0.]),
        ([0., 0., -1.], [-1., 0., 0.], [0., 1., 0.]),
        ([0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]),
        ([0., -1., 0.], [1., 0., 0.], [0., 0., 1.]),
        ([1., 0., 0.], [0., 1., 0.], [0., 0., 1.]),
        ([-1., 0., 0.], [0., -1., 0.], [0., 0., 1.]),
    ] {
        let frame = CoordinateFrame3::try_from_normal_arbitrary_axis(
            Point3::new(10., 20., 30.),
            vector(normal),
        )
        .unwrap();
        assert_eq!(frame.origin().components(), [10., 20., 30.]);
        assert_eq!(frame.x_axis().components(), expected_x);
        assert_eq!(frame.y_axis().components(), expected_y);
    }
    let tilted = CoordinateFrame3::try_from_normal_arbitrary_axis(
        Point3::new(-3., 5., 4.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    assert_eq!(
        tilted
            .try_to_scope_point(Point2::new(2., 3.))
            .unwrap()
            .components(),
        [-5., 5., 7.]
    );
}

#[test]
fn arbitrary_axis_normalization_handles_extreme_finite_inputs() {
    for scale in [f64::from_bits(1), 1e-250, 2., 1e250, f64::MAX] {
        let frame = CoordinateFrame3::try_from_normal_arbitrary_axis(
            Point3::new(0., 0., 0.),
            Vector3::new(0., 0., scale),
        )
        .unwrap();
        assert_eq!(frame, CoordinateFrame3::default());
        let oblique = CoordinateFrame3::try_from_normal_arbitrary_axis(
            Point3::new(0., 0., 0.),
            Vector3::new(scale, scale, scale),
        )
        .unwrap();
        assert!(
            CoordinateFrame3::try_new(oblique.origin(), oblique.x_axis(), oblique.y_axis()).is_ok()
        );
    }
    for normal in [[0., 0., 0.], [f64::NAN, 0., 1.], [0., f64::INFINITY, 1.]] {
        assert!(CoordinateFrame3::try_from_normal_arbitrary_axis(
            Point3::new(0., 0., 0.),
            vector(normal)
        )
        .is_err());
    }
    assert!(CoordinateFrame3::try_from_normal_arbitrary_axis(
        Point3::new(f64::INFINITY, 0., 0.),
        Vector3::new(0., 0., 1.)
    )
    .is_err());
}

#[test]
fn arbitrary_axis_branch_uses_normalized_normal_and_strict_threshold() {
    let t = 1.0_f64 / 64.;
    // Scaled normalization of the adjacent-below source value rounds its X
    // component onto 1/64. The branch applies to that constructed normal,
    // not to the original parameter. Use independent reference outcomes.
    for (x, near_pole) in [
        (t - 1e-12, true),
        (t.next_down(), false),
        (t, false),
        (t.next_up(), false),
        (t + 1e-12, false),
    ] {
        let z = (1. - x * x).sqrt();
        let frame = CoordinateFrame3::try_from_normal_arbitrary_axis(
            Point3::new(0., 0., 0.),
            Vector3::new(x, 0., z),
        )
        .unwrap();
        let expected = if near_pole { [z, 0., -x] } else { [0., 1., 0.] };
        for (actual, expected) in frame.x_axis().components().into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1e-14,
                "source x={x:?}, actual={actual:?}, expected={expected:?}"
            );
        }
    }
    // Reading/construction with explicit axes still retains their exact bits.
    let supplied_x = Vector3::new(1.0_f64.next_up(), 0., 0.);
    let frame = CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        supplied_x,
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    assert_eq!(frame.x_axis(), supplied_x);
}

fn vector(v: [f64; 3]) -> Vector3 {
    Vector3::new(v[0], v[1], v[2])
}

#[test]
fn frame_constructor_preserves_existing_block_axis_construction() {
    use ocdraw::geometry_kernel::{BlockTransform, Scale3};
    for normal in [[1., 2., 3.], [0.1, -0.7, 2.], [1e-250, 3e-250, 4e-250]] {
        let origin = Point3::new(10., 20., 30.);
        let old = BlockTransform::from_normal(origin, vector(normal), 0.7, Scale3::default())
            .unwrap()
            .placement();
        let frame =
            CoordinateFrame3::try_from_normal_arbitrary_axis(origin, vector(normal)).unwrap();
        assert_eq!(
            frame.x_axis().components().map(f64::to_bits),
            old.x_axis().components().map(f64::to_bits)
        );
        assert_eq!(
            frame.y_axis().components().map(f64::to_bits),
            old.y_axis().components().map(f64::to_bits)
        );
    }
}
