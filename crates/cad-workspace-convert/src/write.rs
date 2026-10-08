use super::*;
use crate::diagnostics::{finite, loss};
use ocdraw::geometry_kernel::CoordinateFrame3;
use ocdraw::workspace_kernel::*;
use opencadcodec::entities::{GridFlags, Viewport, ViewportRenderMode};
use opencadcodec::{VPort, Vector2, Vector3};

fn vector(values: [f64; 3]) -> Vector3 {
    Vector3::new(values[0], values[1], values[2])
}
fn check_aids(value: &ViewportAidValues) -> Result<Vec<WorkspaceFieldLoss>, WorkspaceNumericError> {
    validate_grid(&value.grid)?;
    validate_snap(&value.snap)?;
    finite("storedUcs.elevation", [value.ucs_elevation])?;
    let mut losses = Vec::new();
    if value.grid.style != WorkspaceGridStyle::Lines {
        losses.push(loss(
            "grid.style",
            "CAD public model supports only line grids",
        ));
    }
    if i16::try_from(value.grid.major_line_frequency).is_err() {
        losses.push(loss(
            "grid.majorLineFrequency",
            "CAD range exceeded; substituted frequency 5",
        ));
    }
    Ok(losses)
}
fn flags(grid: &WorkspaceGrid) -> GridFlags {
    GridFlags {
        beyond_limits: grid.beyond_limits,
        adaptive: grid.adaptive,
        subdivision: grid.subdivision,
        follow_dynamic: grid.follows_workplane,
    }
}
fn frame(frame: CoordinateFrame3) -> (Vector3, Vector3, Vector3) {
    (
        vector(frame.origin().components()),
        vector(frame.x_axis().components()),
        vector(frame.y_axis().components()),
    )
}
pub fn render_mode_to_cad(mode: WorkspaceRenderMode) -> ViewportRenderMode {
    match mode {
        WorkspaceRenderMode::TwoDimensional => ViewportRenderMode::Wireframe2D,
        WorkspaceRenderMode::Wireframe => ViewportRenderMode::Wireframe3D,
        WorkspaceRenderMode::HiddenLine => ViewportRenderMode::HiddenLine,
        WorkspaceRenderMode::FlatShadedWithoutEdges => ViewportRenderMode::FlatShaded,
        WorkspaceRenderMode::FlatShadedWithEdges => ViewportRenderMode::FlatShadedWithEdges,
        WorkspaceRenderMode::SmoothShadedWithoutEdges => ViewportRenderMode::GouraudShaded,
        WorkspaceRenderMode::SmoothShadedWithEdges => ViewportRenderMode::GouraudShadedWithEdges,
    }
}
pub fn apply_viewport_aids_to_cad(
    target: &mut Viewport,
    source: &ViewportAidValues,
) -> Result<Vec<WorkspaceFieldLoss>, WorkspaceNumericError> {
    let mut losses = check_aids(source)?;
    // The public field exists, but the pinned VIEWPORT DXF/DWG routes omit it.
    // Retain it in the runtime while rejecting unsupported file-fidelity claims.
    for (field, enabled) in [
        ("grid.beyondLimits", source.grid.beyond_limits),
        ("grid.adaptive", source.grid.adaptive),
        ("grid.subdivision", source.grid.subdivision),
        ("grid.followsWorkplane", source.grid.follows_workplane),
    ] {
        if enabled {
            losses.push(loss(field,"retained in CadDocument; pinned VIEWPORT DXF/DWG routes do not retain this grid behavior"));
        }
    }
    target.status.grid_on = source.grid.enabled;
    target.grid_spacing = Vector3::new(source.grid.spacing.x(), source.grid.spacing.y(), 0.);
    target.grid_major = i16::try_from(source.grid.major_line_frequency).unwrap_or(5);
    target.grid_flags = flags(&source.grid);
    target.status.snap_on = source.snap.enabled;
    target.snap_base = Vector3::new(source.snap.base.x(), source.snap.base.y(), 0.);
    target.snap_spacing = Vector3::new(source.snap.spacing.x(), source.snap.spacing.y(), 0.);
    target.snap_angle = source.snap.angle;
    target.status.isometric_snap = source.snap.style == WorkspaceSnapStyle::Isometric;
    target.status.iso_pair_top = source.snap.isometric_plane == WorkspaceIsometricPlane::Top;
    target.status.iso_pair_right = source.snap.isometric_plane == WorkspaceIsometricPlane::Right;
    (target.ucs_origin, target.ucs_x_axis, target.ucs_y_axis) = frame(source.ucs_frame);
    target.elevation = source.ucs_elevation;
    target.ucs_per_viewport = source.use_stored_ucs;
    Ok(losses)
}
pub fn apply_canvas_to_cad(
    target: &mut Viewport,
    source: &PaperCanvasValues,
) -> Result<Vec<WorkspaceFieldLoss>, WorkspaceNumericError> {
    validate_view(&source.view, WorkspaceViewKind::PaperCanvas)?;
    if let Some(value) = source.frame {
        validate_canvas_frame(&value)?;
    }
    let losses = apply_viewport_aids_to_cad(target, &source.aids)?;
    if let Some(value) = source.frame {
        target.center = vector(value.center.components());
        target.width = value.width;
        target.height = value.height;
    }
    target.view_center = Vector3::new(source.view.center.x(), source.view.center.y(), 0.);
    target.view_target = vector(source.view.target.components());
    target.view_direction = vector(source.view.direction.components());
    target.view_height = source.view.height;
    target.twist_angle = source.view.twist;
    target.status.perspective = source.view.projection == WorkspaceProjection::Perspective;
    target.lens_length = source.view.lens_length.unwrap_or(50.);
    target.status.front_clipping = source.view.front_clip.mode != WorkspaceClipMode::Disabled;
    target.status.front_clip_not_at_eye =
        source.view.front_clip.mode == WorkspaceClipMode::AtDistance;
    target.front_clip_z = source.view.front_clip.distance.unwrap_or(0.);
    target.status.back_clipping = source.view.back_clip.mode != WorkspaceClipMode::Disabled;
    target.back_clip_z = source.view.back_clip.distance.unwrap_or(0.);
    Ok(losses)
}
pub fn apply_model_window_to_cad(
    target: &mut VPort,
    source: &ModelWindowValues,
) -> Result<Vec<WorkspaceFieldLoss>, WorkspaceNumericError> {
    validate_view(&source.view, WorkspaceViewKind::Model)?;
    finite("rectangle", source.rectangle)?;
    if !source
        .rectangle
        .into_iter()
        .all(|n| (0.0..=1.).contains(&n))
        || source.rectangle[0] >= source.rectangle[2]
        || source.rectangle[1] >= source.rectangle[3]
        || !source.aspect_ratio.is_finite()
        || source.aspect_ratio <= 0.
    {
        return Err(WorkspaceNumericError {
            field: "rectangle/aspectRatio",
            message: "invalid normalized window".into(),
        });
    }
    let losses = check_aids(&source.aids)?;
    target.lower_left = Vector2::new(source.rectangle[0], source.rectangle[1]);
    target.upper_right = Vector2::new(source.rectangle[2], source.rectangle[3]);
    target.view_center = Vector2::new(source.view.center.x(), source.view.center.y());
    target.view_target = vector(source.view.target.components());
    target.view_direction = vector(source.view.direction.components());
    target.view_height = source.view.height;
    target.view_twist = source.view.twist;
    target.aspect_ratio = source.aspect_ratio;
    target.perspective = source.view.projection == WorkspaceProjection::Perspective;
    target.lens_length = source.view.lens_length.unwrap_or(50.);
    target.front_clipping = source.view.front_clip.mode != WorkspaceClipMode::Disabled;
    target.front_clip_at_eye = source.view.front_clip.mode == WorkspaceClipMode::AtCamera;
    target.front_clip = source.view.front_clip.distance.unwrap_or(0.);
    target.back_clipping = source.view.back_clip.mode != WorkspaceClipMode::Disabled;
    target.back_clip = source.view.back_clip.distance.unwrap_or(0.);
    target.render_mode = render_mode_to_cad(source.render_mode);
    target.grid_on = source.aids.grid.enabled;
    target.grid_spacing = Vector2::new(source.aids.grid.spacing.x(), source.aids.grid.spacing.y());
    target.grid_major = i16::try_from(source.aids.grid.major_line_frequency).unwrap_or(5);
    target.grid_flags = flags(&source.aids.grid);
    target.snap_on = source.aids.snap.enabled;
    target.snap_rotation = source.aids.snap.angle;
    target.snap_base = Vector2::new(source.aids.snap.base.x(), source.aids.snap.base.y());
    target.snap_spacing = Vector2::new(source.aids.snap.spacing.x(), source.aids.snap.spacing.y());
    target.snap_style = source.aids.snap.style == WorkspaceSnapStyle::Isometric;
    target.snap_isopair = match source.aids.snap.isometric_plane {
        WorkspaceIsometricPlane::Left => 0,
        WorkspaceIsometricPlane::Top => 1,
        WorkspaceIsometricPlane::Right => 2,
    };
    (target.ucs_origin, target.ucs_x_axis, target.ucs_y_axis) = frame(source.aids.ucs_frame);
    target.ucs_elevation = source.aids.ucs_elevation;
    target.ucs_per_viewport = source.aids.use_stored_ucs;
    Ok(losses)
}
