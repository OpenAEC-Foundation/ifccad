use ifccad::geometry_kernel::*;
#[test]
fn line_sweep_keeps_scope_endpoints() {
    for (h, lo, hi) in [
        (2., [1., 2., 3.], [4., 6., 10.]),
        (-2., [1., 2., 1.], [4., 6., 8.]),
    ] {
        let g = GeometryRef::Line {
            start: [1., 2., 3.],
            end: [4., 6., 8.],
            thickness: h,
            extrusion_direction: [0., 0., 1.],
        };
        let base = geometry_centerline_bounds(g).unwrap();
        assert_eq!(base.max().components(), [4., 6., 8.]);
        let b = geometry_bounds(g).unwrap();
        assert_eq!(b.min().components(), lo);
        assert_eq!(b.max().components(), hi);
    }
}
#[test]
fn point_sweep_uses_frame_normal() {
    let placement = CoordinateFrame3::try_new(
        Point3::new(10., 20., 30.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 0., 1.),
    )
    .unwrap();
    let b = geometry_bounds(GeometryRef::Point {
        placement,
        thickness: 2.,
    })
    .unwrap();
    assert_eq!(b.min().components(), [10., 18., 30.]);
    assert_eq!(b.max().components(), [10., 20., 30.]);
}
#[test]
fn invalid_sweep_parameters_are_rejected() {
    for (h, d) in [
        (f64::INFINITY, [0., 0., 1.]),
        (0., [0., 0., 0.]),
        (0., [0., 0., 2.]),
    ] {
        assert!(validate_geometry(GeometryRef::Line {
            start: [0.; 3],
            end: [1.; 3],
            thickness: h,
            extrusion_direction: d
        })
        .is_err());
    }
}

#[test]
fn thick_paper_clip_uses_only_base_contour() {
    let g = GeometryRef::Circle {
        placement: CoordinateFrame3::default(),
        radius: 2.,
        thickness: 200.,
    };
    let frame = PaperFrame {
        center: [0., 0.],
        width: 10.,
        height: 10.,
    };
    validate_paper_boundary(frame, g).unwrap();
    assert_eq!(geometry_bounds(g).unwrap().max().z(), 200.);
    assert!(validate_paper_boundary(
        PaperFrame {
            center: [20., 0.],
            ..frame
        },
        g
    )
    .is_err());
}
