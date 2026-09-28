use super::conversion::ExportContext;
use super::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportError, ExportLossReason,
};
use cadcodec::tables::VPort;
use cadcodec::{CadDocument, Handle};
use ifccad::ifcdr::{
    BackClip, BackClipMode, CoordinateFrame3, DrawingViewState, FrontClip, FrontClipMode,
    IfcdrWorkspace, IsometricPlane, ModelWindow, NormalizedRect2, Point2, Point3, ProjectionMode,
    UcsDefinition, UcsSelection, Vector3, ViewDefinition, ViewportRenderMode, WorkspaceGrid,
    WorkspaceGridStyle, WorkspaceSnap, WorkspaceSnapStyle,
};
use ifccad::package::DrawingBuilder;
use std::collections::BTreeMap;

fn loss(context: &mut ExportContext, name: &str) {
    context.diagnostics.push(ExportDiagnostic::loss(
        ExportDiagnosticSource::DocumentField {
            name: name.to_owned(),
        },
        ExportAction::Skipped,
        vec![ExportLossReason::UnsupportedSemantic {
            name: name.to_owned(),
        }],
    ));
}

fn point3(value: cadcodec::Vector3) -> Point3 {
    Point3::new(value.x, value.y, value.z)
}
fn vector3(value: cadcodec::Vector3) -> Vector3 {
    Vector3::new(value.x, value.y, value.z)
}
fn frame(
    origin: cadcodec::Vector3,
    x: cadcodec::Vector3,
    y: cadcodec::Vector3,
) -> Option<CoordinateFrame3> {
    CoordinateFrame3::try_new(point3(origin), vector3(x), vector3(y)).ok()
}
fn source_ucs(
    name: &str,
    origin: cadcodec::Vector3,
    x: cadcodec::Vector3,
    y: cadcodec::Vector3,
    names: &BTreeMap<String, u32>,
) -> Option<UcsSelection> {
    if !name.is_empty() {
        return names
            .get(&name.to_lowercase())
            .copied()
            .map(|ucs_id| UcsSelection::Named { ucs_id });
    }
    let frame = frame(origin, x, y)?;
    Some(if frame == CoordinateFrame3::default() {
        UcsSelection::World
    } else {
        UcsSelection::Unnamed { frame }
    })
}
fn vport_ucs(vport: &VPort, handles: &BTreeMap<Handle, u32>) -> Option<UcsSelection> {
    if vport.named_ucs_handle != Handle::NULL {
        return handles
            .get(&vport.named_ucs_handle)
            .copied()
            .map(|ucs_id| UcsSelection::Named { ucs_id });
    }
    source_ucs(
        "",
        vport.ucs_origin,
        vport.ucs_x_axis,
        vport.ucs_y_axis,
        &BTreeMap::new(),
    )
}

fn model_window(vport: &VPort, id: u32, stored_ucs: UcsSelection) -> Option<ModelWindow> {
    let rect = NormalizedRect2 {
        min_x: vport.lower_left.x,
        min_y: vport.lower_left.y,
        max_x: vport.upper_right.x,
        max_y: vport.upper_right.y,
    };
    let render_mode = match vport.render_mode {
        cadcodec::entities::ViewportRenderMode::Wireframe2D => ViewportRenderMode::TwoDimensional,
        cadcodec::entities::ViewportRenderMode::Wireframe3D => ViewportRenderMode::Wireframe,
        cadcodec::entities::ViewportRenderMode::HiddenLine => ViewportRenderMode::HiddenLine,
        cadcodec::entities::ViewportRenderMode::FlatShaded => {
            ViewportRenderMode::FlatShadedWithoutEdges
        }
        cadcodec::entities::ViewportRenderMode::FlatShadedWithEdges => {
            ViewportRenderMode::FlatShadedWithEdges
        }
        cadcodec::entities::ViewportRenderMode::GouraudShaded => {
            ViewportRenderMode::SmoothShadedWithoutEdges
        }
        cadcodec::entities::ViewportRenderMode::GouraudShadedWithEdges => {
            ViewportRenderMode::SmoothShadedWithEdges
        }
    };
    let grid = WorkspaceGrid {
        enabled: vport.grid_on,
        spacing: Point2::new(vport.grid_spacing.x, vport.grid_spacing.y),
        style: WorkspaceGridStyle::Lines,
        major_line_frequency: u32::try_from(vport.grid_major).ok()?,
        beyond_limits: vport.grid_flags.beyond_limits,
        adaptive: vport.grid_flags.adaptive,
        subdivision: vport.grid_flags.subdivision,
        follows_workplane: vport.grid_flags.follow_dynamic,
    };
    let snap = WorkspaceSnap {
        enabled: vport.snap_on,
        base: Point2::new(vport.snap_base.x, vport.snap_base.y),
        spacing: Point2::new(vport.snap_spacing.x, vport.snap_spacing.y),
        angle: vport.snap_rotation,
        style: if vport.snap_style {
            WorkspaceSnapStyle::Isometric
        } else {
            WorkspaceSnapStyle::Rectangular
        },
        isometric_plane: match vport.snap_isopair {
            0 => IsometricPlane::Left,
            1 => IsometricPlane::Top,
            2 => IsometricPlane::Right,
            _ => return None,
        },
    };
    Some(ModelWindow {
        model_window_id: id,
        rectangle: rect,
        view: ViewDefinition {
            center: Point2::new(vport.view_center.x, vport.view_center.y),
            target: point3(vport.view_target),
            direction: vector3(vport.view_direction),
            height: vport.view_height,
            twist: vport.view_twist,
            projection: if vport.perspective {
                ProjectionMode::Perspective
            } else {
                ProjectionMode::Orthographic
            },
            lens_length: Some(vport.lens_length),
            front_clip: FrontClip {
                mode: if vport.front_clipping {
                    if vport.front_clip_at_eye {
                        FrontClipMode::AtCamera
                    } else {
                        FrontClipMode::AtDistance
                    }
                } else {
                    FrontClipMode::Disabled
                },
                distance: Some(vport.front_clip),
            },
            back_clip: BackClip {
                mode: if vport.back_clipping {
                    BackClipMode::AtDistance
                } else {
                    BackClipMode::Disabled
                },
                distance: Some(vport.back_clip),
            },
        },
        aspect_ratio: vport.aspect_ratio,
        render_mode,
        grid,
        snap,
        stored_ucs,
        use_stored_ucs: vport.ucs_per_viewport,
    })
}

pub(crate) fn add_workspace(
    document: &CadDocument,
    drawing: &mut DrawingBuilder<'_>,
    context: &mut ExportContext,
) -> Result<(), ExportError> {
    if !document.header.show_model_space {
        loss(context, "active paper layout is not exposed by cadcodec");
        return Ok(());
    }
    let current_layer = match context
        .layer_keys
        .get(&document.header.current_layer_name.to_lowercase())
    {
        Some(key) => *key,
        None => {
            loss(context, "header.current_layer_name");
            return Ok(());
        }
    };
    let mut workspace = IfcdrWorkspace::default();
    let mut names = BTreeMap::new();
    let mut handles = BTreeMap::new();
    let mut mapped_ucss = Vec::new();
    let mut mapped_vports = Vec::new();
    for (index, source) in document.ucss.iter().enumerate() {
        if source.ortho_view_type != 0
            || source.ortho_type != 0
            || source.named_ucs_handle != Handle::NULL
            || source.base_ucs_handle != Handle::NULL
            || source.xref_reference
            || source.xref_dependent
        {
            loss(context, "UCS orthographic or external-reference settings");
        }
        let Some(frame) = frame(source.origin, source.x_axis, source.y_axis) else {
            loss(context, "invalid UCS frame");
            continue;
        };
        let ucs_id = u32::try_from(index + 1).map_err(|_| ExportError::InternalInvariant {
            message: "too many UCS definitions".into(),
        })?;
        names.insert(source.name.to_lowercase(), ucs_id);
        handles.insert(source.handle, ucs_id);
        workspace.ucs_definitions.push(UcsDefinition {
            ucs_id,
            name: source.name.clone(),
            frame,
            elevation: source.elevation,
        });
        mapped_ucss.push(source.handle);
    }
    let Some(current_model_ucs) = source_ucs(
        &document.header.model_space_ucs_name,
        document.header.model_space_ucs_origin,
        document.header.model_space_ucs_x_axis,
        document.header.model_space_ucs_y_axis,
        &names,
    ) else {
        loss(context, "header.model_space_ucs");
        return Ok(());
    };
    let mut active_id = None;
    for source in document
        .vports
        .iter()
        .filter(|v| v.name.eq_ignore_ascii_case("*active"))
    {
        let Some(stored_ucs) = vport_ucs(source, &handles) else {
            loss(context, "VPORT UCS");
            return Ok(());
        };
        let id = u32::try_from(workspace.model_windows.len() + 1).map_err(|_| {
            ExportError::InternalInvariant {
                message: "too many active VPORT entries".into(),
            }
        })?;
        let Some(window) = model_window(source, id, stored_ucs) else {
            loss(context, "VPORT view/grid/snap");
            return Ok(());
        };
        if active_id.is_none() {
            active_id = Some(id);
        }
        mapped_vports.push(source.handle);
        workspace.model_windows.push(window);
    }
    let Some(active_model_window_id) = active_id else {
        loss(context, "active VPORT");
        return Ok(());
    };
    if workspace.model_windows.len() > 1 {
        loss(
            context,
            "active tiled VPORT identity is not exposed by cadcodec",
        );
    }
    if let Some(active) = workspace.model_windows.first_mut() {
        if active.use_stored_ucs && active.stored_ucs != current_model_ucs {
            loss(context, "active VPORT UCS differs from header current UCS");
            active.use_stored_ucs = false;
        }
    }
    workspace.drawing_view_state = Some(DrawingViewState {
        current_model_ucs,
        active_model_window_id,
    });
    drawing.set_workspace_state(workspace, current_layer, None)?;
    context.mapped_workspace_ucss.extend(mapped_ucss);
    context.mapped_workspace_vports.extend(mapped_vports);
    Ok(())
}
