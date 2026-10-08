use ocdraw::geometry_kernel::{Point2, Point3, Vector3};
use ocdraw::workspace_kernel::*;

fn snap(enabled: bool, x: f64, y: f64) -> WorkspaceSnap {
    WorkspaceSnap {
        enabled,
        base: Point2::new(4., 5.),
        spacing: Point2::new(x, y),
        angle: std::f64::consts::FRAC_PI_2,
        style: WorkspaceSnapStyle::Isometric,
        isometric_plane: WorkspaceIsometricPlane::Left,
    }
}
fn grid() -> WorkspaceGrid {
    WorkspaceGrid {
        enabled: false,
        spacing: Point2::new(0., 3.),
        style: WorkspaceGridStyle::Dots,
        major_line_frequency: 1,
        beyond_limits: true,
        adaptive: false,
        subdivision: true,
        follows_workplane: false,
    }
}
fn view() -> WorkspaceView {
    WorkspaceView {
        center: Point2::new(3., 4.),
        target: Point3::new(0., 0., 0.),
        direction: Vector3::new(0., 0., 1.),
        height: 20.,
        twist: 0.,
        projection: WorkspaceProjection::Orthographic,
        lens_length: None,
        front_clip: WorkspaceClip {
            mode: WorkspaceClipMode::Disabled,
            distance: Some(0.),
        },
        back_clip: WorkspaceClip {
            mode: WorkspaceClipMode::Disabled,
            distance: None,
        },
    }
}
#[test]
fn disabled_zero_snap_is_valid_but_enabled_zero_is_not() {
    for (x, y) in [(0., 0.), (0., 3.), (2., 0.), (2., 3.)] {
        assert!(validate_snap(&snap(false, x, y)).is_ok());
        assert_eq!(validate_snap(&snap(true, x, y)).is_ok(), x > 0. && y > 0.);
    }
    assert!(validate_grid(&grid()).is_ok());
    let mut invalid = grid();
    invalid.major_line_frequency = 0;
    assert!(validate_grid(&invalid).is_err());
    invalid.major_line_frequency = u32::MAX;
    assert!(validate_grid(&invalid).is_ok());
}
#[test]
fn negative_or_nonfinite_dormant_values_are_invalid() {
    for value in [-1., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(validate_snap(&snap(false, value, 2.)).is_err());
        let mut invalid = grid();
        invalid.spacing = Point2::new(value, 3.);
        assert!(validate_grid(&invalid).is_err());
    }
    let mut invalid = snap(false, 0., 0.);
    invalid.angle = f64::NAN;
    assert!(validate_snap(&invalid).is_err());
}
#[test]
fn canvas_frame_is_positive_but_not_normalized() {
    let mut frame = WorkspaceCanvasFrame {
        center: Point3::new(100., -50., 3.),
        width: 10.,
        height: 8.,
    };
    assert!(validate_canvas_frame(&frame).is_ok());
    frame.width = 0.;
    assert!(validate_canvas_frame(&frame).is_err());
    frame.width = 10.;
    frame.center = Point3::new(f64::INFINITY, 0., 0.);
    assert!(validate_canvas_frame(&frame).is_err());
}
#[test]
fn view_predicates_keep_model_and_canvas_rules_distinct() {
    let mut v = view();
    assert!(validate_view(&v, WorkspaceViewKind::Model).is_ok());
    assert!(validate_view(&v, WorkspaceViewKind::PaperCanvas).is_ok());
    v.projection = WorkspaceProjection::Perspective;
    v.lens_length = Some(50.);
    assert!(validate_view(&v, WorkspaceViewKind::Model).is_ok());
    assert!(validate_view(&v, WorkspaceViewKind::PaperCanvas).is_err());
    v.lens_length = Some(0.);
    assert!(validate_view(&v, WorkspaceViewKind::Model).is_err());
    v = view();
    v.direction = Vector3::new(0., 0., 0.);
    assert!(validate_view(&v, WorkspaceViewKind::Model).is_err());
    v = view();
    v.front_clip = WorkspaceClip {
        mode: WorkspaceClipMode::AtDistance,
        distance: Some(2.),
    };
    v.back_clip = WorkspaceClip {
        mode: WorkspaceClipMode::AtDistance,
        distance: Some(3.),
    };
    assert!(validate_view(&v, WorkspaceViewKind::Model).is_err());
}

#[test]
fn camera_clip_order_uses_exact_stored_direction() {
    let mut v = view();
    v.direction = Vector3::new(1., 2_f64.powi(-27), 0.);
    v.front_clip = WorkspaceClip {
        mode: WorkspaceClipMode::AtCamera,
        distance: None,
    };
    v.back_clip = WorkspaceClip {
        mode: WorkspaceClipMode::AtDistance,
        distance: Some(1.),
    };
    assert!(validate_view(&v, WorkspaceViewKind::Model).is_ok());
}
