use cad_workspace_convert::*;
use ocdraw::geometry_kernel::Point2;
use ocdraw::workspace_kernel::*;
use opencadcodec::entities::Viewport;
use opencadcodec::{VPort, Vector3};

#[test]
fn cad_field_adapters_keep_dormant_values() {
    let mut source = Viewport::new();
    source.status.snap_on = false;
    source.snap_spacing = Vector3::ZERO;
    source.snap_base = Vector3::new(4., 5., 0.);
    source.snap_angle = 0.25;
    source.ucs_per_viewport = false;
    source.ucs_origin = Vector3::new(1., 2., 0.);
    let fields = prepare_viewport_aids_from_cad(&source).unwrap();
    assert!(!fields.value.snap.enabled);
    assert_eq!(fields.value.snap.spacing, Point2::new(0., 0.));
    let mut target = Viewport::new();
    target.center = Vector3::new(100., 200., 0.);
    target.status.non_rectangular_clipping = true;
    target.off_screen = true;
    let before = target.clone();
    assert!(apply_viewport_aids_to_cad(&mut target, &fields.value)
        .unwrap()
        .is_empty());
    assert_eq!(target.center, before.center);
    assert_eq!(target.view_height, before.view_height);
    assert!(target.status.non_rectangular_clipping && target.off_screen);
    assert_eq!(target.snap_spacing, Vector3::ZERO);
    assert_eq!(target.ucs_origin, source.ucs_origin);
    assert!(!target.ucs_per_viewport);
}
#[test]
fn canvas_and_viewport_adapters_use_different_coordinate_roles() {
    let mut source = Viewport::new();
    source.id = 1;
    source.center = Vector3::new(100., -50., 3.);
    source.width = 10.;
    source.height = 8.;
    let canvas = prepare_canvas_from_cad(&source).unwrap();
    assert_eq!(canvas.value.frame.unwrap().center.z(), 3.);
    let aids = prepare_viewport_aids_from_cad(&source).unwrap();
    assert_eq!(aids.value.grid.spacing, canvas.value.aids.grid.spacing);
    let mut target = Viewport::new();
    apply_canvas_to_cad(&mut target, &canvas.value).unwrap();
    assert_eq!(target.center, source.center);
}
#[test]
fn all_isometric_pair_combinations_are_explicit() {
    for (top, right, plane) in [
        (false, false, WorkspaceIsometricPlane::Left),
        (true, false, WorkspaceIsometricPlane::Top),
        (false, true, WorkspaceIsometricPlane::Right),
        (true, true, WorkspaceIsometricPlane::Left),
    ] {
        let mut source = Viewport::new();
        source.status.iso_pair_top = top;
        source.status.iso_pair_right = right;
        let fields = prepare_viewport_aids_from_cad(&source).unwrap();
        assert_eq!(fields.value.snap.isometric_plane, plane);
        let mut target = Viewport::new();
        apply_viewport_aids_to_cad(&mut target, &fields.value).unwrap();
        assert_eq!(
            prepare_viewport_aids_from_cad(&target)
                .unwrap()
                .value
                .snap
                .isometric_plane,
            plane
        );
    }
}
#[test]
fn grid_substitution_has_exact_field_evidence() {
    let mut fields = prepare_model_window_from_cad(&VPort::active())
        .unwrap()
        .value;
    fields.aids.grid.style = WorkspaceGridStyle::Dots;
    fields.aids.grid.major_line_frequency = u32::MAX;
    let mut target = VPort::active();
    let issues = apply_model_window_to_cad(&mut target, &fields).unwrap();
    assert!(issues.iter().any(|i| i.field == "grid.style"));
    assert!(issues.iter().any(|i| i.field == "grid.majorLineFrequency"));
    assert_eq!(target.grid_major, 5);
}
#[test]
fn workspace_nonfinite_scalars_are_errors() {
    let mut source = Viewport::new();
    source.grid_spacing.z = f64::NAN;
    assert!(prepare_viewport_aids_from_cad(&source).is_err());
    source = Viewport::new();
    source.snap_spacing = Vector3::ZERO;
    source.status.snap_on = true;
    assert!(prepare_viewport_aids_from_cad(&source).is_err());
    let mut fields = prepare_model_window_from_cad(&VPort::active())
        .unwrap()
        .value;
    fields.view.height = f64::INFINITY;
    assert!(apply_model_window_to_cad(&mut VPort::active(), &fields).is_err());
}

#[test]
fn viewport_grid_behavior_has_explicit_file_codec_limitations() {
    let mut fields = prepare_model_window_from_cad(&VPort::active())
        .unwrap()
        .value;
    fields.aids.grid.beyond_limits = true;
    fields.aids.grid.adaptive = true;
    fields.aids.grid.subdivision = true;
    fields.aids.grid.follows_workplane = true;
    let issues = apply_viewport_aids_to_cad(&mut Viewport::new(), &fields.aids).unwrap();
    for field in [
        "grid.beyondLimits",
        "grid.adaptive",
        "grid.subdivision",
        "grid.followsWorkplane",
    ] {
        assert!(issues.iter().any(|i| i.field == field));
    }
    assert!(apply_model_window_to_cad(&mut VPort::active(), &fields)
        .unwrap()
        .is_empty());
}
