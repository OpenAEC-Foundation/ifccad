use super::*;
use crate::diagnostics::{finite, loss};
use ocdraw::geometry_kernel::{CoordinateFrame3, Point2, Point3, Vector3};
use ocdraw::workspace_kernel::*;
use opencadcodec::entities::{Viewport, ViewportRenderMode};
use opencadcodec::{VPort, Vector3 as CadVector};

fn point(v: CadVector) -> Point3 {
    Point3::new(v.x, v.y, v.z)
}
fn vector(v: CadVector) -> Vector3 {
    Vector3::new(v.x, v.y, v.z)
}
pub fn prepare_ucs_frame_from_cad(
    origin: CadVector,
    x: CadVector,
    y: CadVector,
) -> Result<CoordinateFrame3, WorkspaceNumericError> {
    let frame = CoordinateFrame3::try_new(point(origin), vector(x), vector(y)).map_err(|e| {
        WorkspaceNumericError {
            field: "storedUcs",
            message: e.to_string(),
        }
    })?;
    if frame.origin().components() != [origin.x, origin.y, origin.z]
        || frame.x_axis().components() != [x.x, x.y, x.z]
        || frame.y_axis().components() != [y.x, y.y, y.z]
    {
        return Err(WorkspaceNumericError {
            field: "storedUcs",
            message: "frame normalization is not an exact field copy".into(),
        });
    }
    Ok(frame)
}
pub fn render_mode_from_cad(mode: ViewportRenderMode) -> WorkspaceRenderMode {
    match mode {
        ViewportRenderMode::Wireframe2D => WorkspaceRenderMode::TwoDimensional,
        ViewportRenderMode::Wireframe3D => WorkspaceRenderMode::Wireframe,
        ViewportRenderMode::HiddenLine => WorkspaceRenderMode::HiddenLine,
        ViewportRenderMode::FlatShaded => WorkspaceRenderMode::FlatShadedWithoutEdges,
        ViewportRenderMode::FlatShadedWithEdges => WorkspaceRenderMode::FlatShadedWithEdges,
        ViewportRenderMode::GouraudShaded => WorkspaceRenderMode::SmoothShadedWithoutEdges,
        ViewportRenderMode::GouraudShadedWithEdges => WorkspaceRenderMode::SmoothShadedWithEdges,
    }
}
fn frequency(value: i16) -> Result<u32, WorkspaceNumericError> {
    u32::try_from(value)
        .ok()
        .filter(|v| *v > 0)
        .ok_or_else(|| WorkspaceNumericError {
            field: "grid.majorLineFrequency",
            message: "nonpositive CAD frequency".into(),
        })
}
fn check(aids: &ViewportAidValues) -> Result<(), WorkspaceNumericError> {
    validate_grid(&aids.grid)?;
    validate_snap(&aids.snap)?;
    finite("storedUcs.elevation", [aids.ucs_elevation])?;
    Ok(())
}
pub fn prepare_model_window_from_cad(
    v: &VPort,
) -> Result<PreparedWorkspace<ModelWindowValues>, WorkspaceNumericError> {
    let aids = ViewportAidValues {
        grid: WorkspaceGrid {
            enabled: v.grid_on,
            spacing: Point2::new(v.grid_spacing.x, v.grid_spacing.y),
            style: WorkspaceGridStyle::Lines,
            major_line_frequency: frequency(v.grid_major)?,
            beyond_limits: v.grid_flags.beyond_limits,
            adaptive: v.grid_flags.adaptive,
            subdivision: v.grid_flags.subdivision,
            follows_workplane: v.grid_flags.follow_dynamic,
        },
        snap: WorkspaceSnap {
            enabled: v.snap_on,
            base: Point2::new(v.snap_base.x, v.snap_base.y),
            spacing: Point2::new(v.snap_spacing.x, v.snap_spacing.y),
            angle: v.snap_rotation,
            style: if v.snap_style {
                WorkspaceSnapStyle::Isometric
            } else {
                WorkspaceSnapStyle::Rectangular
            },
            isometric_plane: match v.snap_isopair {
                0 => WorkspaceIsometricPlane::Left,
                1 => WorkspaceIsometricPlane::Top,
                2 => WorkspaceIsometricPlane::Right,
                _ => {
                    return Err(WorkspaceNumericError {
                        field: "snap.isometricPlane",
                        message: "unknown CAD plane".into(),
                    })
                }
            },
        },
        ucs_frame: prepare_ucs_frame_from_cad(v.ucs_origin, v.ucs_x_axis, v.ucs_y_axis)?,
        ucs_elevation: v.ucs_elevation,
        use_stored_ucs: v.ucs_per_viewport,
    };
    check(&aids)?;
    let view = WorkspaceView {
        center: Point2::new(v.view_center.x, v.view_center.y),
        target: point(v.view_target),
        direction: vector(v.view_direction),
        height: v.view_height,
        twist: v.view_twist,
        projection: if v.perspective {
            WorkspaceProjection::Perspective
        } else {
            WorkspaceProjection::Orthographic
        },
        lens_length: Some(v.lens_length),
        front_clip: WorkspaceClip {
            mode: if !v.front_clipping {
                WorkspaceClipMode::Disabled
            } else if v.front_clip_at_eye {
                WorkspaceClipMode::AtCamera
            } else {
                WorkspaceClipMode::AtDistance
            },
            distance: Some(v.front_clip),
        },
        back_clip: WorkspaceClip {
            mode: if v.back_clipping {
                WorkspaceClipMode::AtDistance
            } else {
                WorkspaceClipMode::Disabled
            },
            distance: Some(v.back_clip),
        },
    };
    validate_view(&view, WorkspaceViewKind::Model)?;
    let rectangle = [
        v.lower_left.x,
        v.lower_left.y,
        v.upper_right.x,
        v.upper_right.y,
    ];
    finite("rectangle", rectangle)?;
    if !rectangle.into_iter().all(|n| (0.0..=1.).contains(&n))
        || rectangle[0] >= rectangle[2]
        || rectangle[1] >= rectangle[3]
        || !v.aspect_ratio.is_finite()
        || v.aspect_ratio <= 0.
    {
        return Err(WorkspaceNumericError {
            field: "rectangle/aspectRatio",
            message: "invalid normalized window".into(),
        });
    }
    Ok(PreparedWorkspace {
        value: ModelWindowValues {
            rectangle,
            view,
            aspect_ratio: v.aspect_ratio,
            render_mode: render_mode_from_cad(v.render_mode),
            aids,
        },
        losses: vec![],
    })
}
pub fn prepare_viewport_aids_from_cad(
    v: &Viewport,
) -> Result<PreparedWorkspace<ViewportAidValues>, WorkspaceNumericError> {
    let mut losses = vec![];
    for (field, value) in [
        ("grid.spacing.z", v.grid_spacing.z),
        ("snap.spacing.z", v.snap_spacing.z),
        ("snap.base.z", v.snap_base.z),
    ] {
        finite(field, [value])?;
        if value != 0. {
            losses.push(loss(
                field,
                "nonzero Z component has no planar drawing-aid field",
            ));
        }
    }
    let aids = ViewportAidValues {
        grid: WorkspaceGrid {
            enabled: v.status.grid_on,
            spacing: Point2::new(v.grid_spacing.x, v.grid_spacing.y),
            style: WorkspaceGridStyle::Lines,
            major_line_frequency: frequency(v.grid_major)?,
            beyond_limits: v.grid_flags.beyond_limits,
            adaptive: v.grid_flags.adaptive,
            subdivision: v.grid_flags.subdivision,
            follows_workplane: v.grid_flags.follow_dynamic,
        },
        snap: WorkspaceSnap {
            enabled: v.status.snap_on,
            base: Point2::new(v.snap_base.x, v.snap_base.y),
            spacing: Point2::new(v.snap_spacing.x, v.snap_spacing.y),
            angle: v.snap_angle,
            style: if v.status.isometric_snap {
                WorkspaceSnapStyle::Isometric
            } else {
                WorkspaceSnapStyle::Rectangular
            },
            isometric_plane: match (v.status.iso_pair_top, v.status.iso_pair_right) {
                (true, false) => WorkspaceIsometricPlane::Top,
                (false, true) => WorkspaceIsometricPlane::Right,
                _ => WorkspaceIsometricPlane::Left,
            },
        },
        ucs_frame: prepare_ucs_frame_from_cad(v.ucs_origin, v.ucs_x_axis, v.ucs_y_axis)?,
        ucs_elevation: v.elevation,
        use_stored_ucs: v.ucs_per_viewport,
    };
    check(&aids)?;
    Ok(PreparedWorkspace {
        value: aids,
        losses,
    })
}
pub fn prepare_canvas_from_cad(
    v: &Viewport,
) -> Result<PreparedWorkspace<PaperCanvasValues>, WorkspaceNumericError> {
    let aids = prepare_viewport_aids_from_cad(v)?;
    let mut losses = aids.losses;
    finite("view.center.z", [v.view_center.z])?;
    if v.view_center.z != 0. {
        losses.push(loss(
            "view.center.z",
            "nonzero DCS Z has no planar center field",
        ));
    }
    let view = WorkspaceView {
        center: Point2::new(v.view_center.x, v.view_center.y),
        target: point(v.view_target),
        direction: vector(v.view_direction),
        height: v.view_height,
        twist: v.twist_angle,
        projection: if v.status.perspective {
            WorkspaceProjection::Perspective
        } else {
            WorkspaceProjection::Orthographic
        },
        lens_length: Some(v.lens_length),
        front_clip: WorkspaceClip {
            mode: if !v.status.front_clipping {
                WorkspaceClipMode::Disabled
            } else if v.status.front_clip_not_at_eye {
                WorkspaceClipMode::AtDistance
            } else {
                WorkspaceClipMode::AtCamera
            },
            distance: Some(v.front_clip_z),
        },
        back_clip: WorkspaceClip {
            mode: if v.status.back_clipping {
                WorkspaceClipMode::AtDistance
            } else {
                WorkspaceClipMode::Disabled
            },
            distance: Some(v.back_clip_z),
        },
    };
    validate_view(&view, WorkspaceViewKind::PaperCanvas)?;
    let candidate = WorkspaceCanvasFrame {
        center: point(v.center),
        width: v.width,
        height: v.height,
    };
    finite(
        "frame",
        [v.center.x, v.center.y, v.center.z]
            .into_iter()
            .chain([v.width, v.height]),
    )?;
    let frame = if validate_canvas_frame(&candidate).is_ok() {
        Some(candidate)
    } else {
        losses.push(loss("frame", "degenerate canvas frame omitted"));
        None
    };
    Ok(PreparedWorkspace {
        value: PaperCanvasValues {
            frame,
            view,
            aids: aids.value,
        },
        losses,
    })
}
