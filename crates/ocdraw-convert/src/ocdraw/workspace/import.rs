use super::{
    BackClipMode, CoordinateFrame3, FrontClipMode, IsometricPlane, ModelWindow, PaperCanvas,
    ProjectionMode, UcsSelection, ViewportRenderMode, WorkspaceGridStyle, WorkspaceSnapStyle,
};
use crate::ocdraw::import::{diagnostic, DirectImportDiagnostic};
use cadcodec::entities::EntityType;
use cadcodec::objects::ObjectType;
use cadcodec::{CadDocument, Handle, VPort, Vector2, Vector3};
use ocdraw::ocdraw::{DrawingPaperContext, ValidatedDrawing};
use std::collections::BTreeMap;
struct DiagnosticAccumulator<'a>(&'a mut Vec<DirectImportDiagnostic>);
enum ImportDiagnostic {
    WorkspaceUnsupported { reason: String },
}
impl DiagnosticAccumulator<'_> {
    fn record(&mut self, issue: ImportDiagnostic) {
        let ImportDiagnostic::WorkspaceUnsupported { reason } = issue;
        self.0.push(diagnostic("WORKSPACE", "/", reason));
    }
}
fn cad_vector(frame: [f64; 3]) -> Vector3 {
    Vector3::new(frame[0], frame[1], frame[2])
}
fn assign_frame(
    origin: &mut Vector3,
    x_axis: &mut Vector3,
    y_axis: &mut Vector3,
    frame: CoordinateFrame3,
) {
    let p = frame.origin();
    let x = frame.x_axis();
    let y = frame.y_axis();
    *origin = cad_vector([p.x(), p.y(), p.z()]);
    *x_axis = cad_vector([x.x(), x.y(), x.z()]);
    *y_axis = cad_vector([y.x(), y.y(), y.z()]);
}
fn selection(
    choice: UcsSelection,
    definitions: &BTreeMap<u32, (String, Handle, CoordinateFrame3)>,
) -> (String, Handle, CoordinateFrame3) {
    match choice {
        UcsSelection::World => (String::new(), Handle::NULL, CoordinateFrame3::default()),
        UcsSelection::Named(ucs_id) => {
            let (name, handle, frame) = &definitions[&ucs_id];
            (name.clone(), *handle, *frame)
        }
        UcsSelection::Unnamed(frame) => (String::new(), Handle::NULL, frame),
    }
}
fn to_cad_mode(mode: ViewportRenderMode) -> cadcodec::entities::ViewportRenderMode {
    use cadcodec::entities::ViewportRenderMode as Cad;
    match mode {
        ViewportRenderMode::TwoDimensional => Cad::Wireframe2D,
        ViewportRenderMode::Wireframe => Cad::Wireframe3D,
        ViewportRenderMode::HiddenLine => Cad::HiddenLine,
        ViewportRenderMode::FlatShadedWithoutEdges => Cad::FlatShaded,
        ViewportRenderMode::FlatShadedWithEdges => Cad::FlatShadedWithEdges,
        ViewportRenderMode::SmoothShadedWithoutEdges => Cad::GouraudShaded,
        ViewportRenderMode::SmoothShadedWithEdges => Cad::GouraudShadedWithEdges,
    }
}
fn apply_window(
    target: &mut VPort,
    source: &ModelWindow,
    definitions: &BTreeMap<u32, (String, Handle, CoordinateFrame3)>,
    diagnostics: &mut DiagnosticAccumulator,
) {
    target.lower_left = Vector2::new(source.rectangle[0], source.rectangle[1]);
    target.upper_right = Vector2::new(source.rectangle[2], source.rectangle[3]);
    target.view_center = Vector2::new(source.view.center.x(), source.view.center.y());
    let target_point = source.view.target;
    let direction = source.view.direction;
    target.view_target = cad_vector([target_point.x(), target_point.y(), target_point.z()]);
    target.view_direction = cad_vector([direction.x(), direction.y(), direction.z()]);
    target.view_height = source.view.height;
    target.aspect_ratio = source.aspect_ratio;
    target.view_twist = source.view.twist;
    target.perspective = source.view.projection == ProjectionMode::Perspective;
    target.lens_length = source.view.lens_length.unwrap_or(50.0);
    target.front_clipping = source.view.front_clip.mode != FrontClipMode::Disabled;
    target.front_clip_at_eye = source.view.front_clip.mode == FrontClipMode::AtCamera;
    target.front_clip = source.view.front_clip.distance.unwrap_or(0.0);
    target.back_clipping = source.view.back_clip.mode != BackClipMode::Disabled;
    target.back_clip = source.view.back_clip.distance.unwrap_or(0.0);
    target.render_mode = to_cad_mode(source.render_mode);
    target.grid_on = source.grid.enabled;
    target.grid_spacing = Vector2::new(source.grid.spacing.x(), source.grid.spacing.y());
    target.grid_major = match i16::try_from(source.grid.major_line_frequency) {
        Ok(value) => value,
        Err(_) => {
            diagnostics.record(ImportDiagnostic::WorkspaceUnsupported {
                reason: "grid major frequency exceeds CAD range".into(),
            });
            5
        }
    };
    target.grid_flags = cadcodec::entities::GridFlags {
        beyond_limits: source.grid.beyond_limits,
        adaptive: source.grid.adaptive,
        subdivision: source.grid.subdivision,
        follow_dynamic: source.grid.follows_workplane,
    };
    if source.grid.style != WorkspaceGridStyle::Lines {
        diagnostics.record(ImportDiagnostic::WorkspaceUnsupported {
            reason: "dot grid style".into(),
        });
    }
    target.snap_on = source.snap.enabled;
    target.snap_base = Vector2::new(source.snap.base.x(), source.snap.base.y());
    target.snap_spacing = Vector2::new(source.snap.spacing.x(), source.snap.spacing.y());
    target.snap_rotation = source.snap.angle;
    target.snap_style = source.snap.style == WorkspaceSnapStyle::Isometric;
    target.snap_isopair = match source.snap.isometric_plane {
        IsometricPlane::Left => 0,
        IsometricPlane::Top => 1,
        IsometricPlane::Right => 2,
    };
    let (_, handle, frame) = selection(source.stored_ucs, definitions);
    target.named_ucs_handle = handle;
    assign_frame(
        &mut target.ucs_origin,
        &mut target.ucs_x_axis,
        &mut target.ucs_y_axis,
        frame,
    );
    target.ucs_per_viewport = source.use_stored_ucs;
}

fn apply_canvas(
    target: &mut cadcodec::entities::Viewport,
    source: &PaperCanvas,
    definitions: &BTreeMap<u32, (String, Handle, CoordinateFrame3)>,
    diagnostics: &mut DiagnosticAccumulator,
) {
    target.view_center = Vector3::new(source.view.center.x(), source.view.center.y(), 0.0);
    let point = source.view.target;
    let direction = source.view.direction;
    target.view_target = cad_vector([point.x(), point.y(), point.z()]);
    target.view_direction = cad_vector([direction.x(), direction.y(), direction.z()]);
    target.view_height = source.view.height;
    target.twist_angle = source.view.twist;
    target.lens_length = source.view.lens_length.unwrap_or(50.0);
    target.status.front_clipping = source.view.front_clip.mode != FrontClipMode::Disabled;
    target.status.front_clip_not_at_eye = source.view.front_clip.mode == FrontClipMode::AtDistance;
    target.front_clip_z = source.view.front_clip.distance.unwrap_or(0.0);
    target.status.back_clipping = source.view.back_clip.mode != BackClipMode::Disabled;
    target.back_clip_z = source.view.back_clip.distance.unwrap_or(0.0);
    target.status.grid_on = source.grid.enabled;
    target.grid_spacing = Vector3::new(source.grid.spacing.x(), source.grid.spacing.y(), 0.0);
    target.grid_major = match i16::try_from(source.grid.major_line_frequency) {
        Ok(value) => value,
        Err(_) => {
            diagnostics.record(ImportDiagnostic::WorkspaceUnsupported {
                reason: "paper grid major frequency exceeds CAD range".into(),
            });
            5
        }
    };
    target.grid_flags = cadcodec::entities::GridFlags {
        beyond_limits: source.grid.beyond_limits,
        adaptive: source.grid.adaptive,
        subdivision: source.grid.subdivision,
        follow_dynamic: source.grid.follows_workplane,
    };
    if source.grid.style != WorkspaceGridStyle::Lines {
        diagnostics.record(ImportDiagnostic::WorkspaceUnsupported {
            reason: "paper dot grid style".into(),
        });
    }
    target.status.snap_on = source.snap.enabled;
    target.snap_base = Vector3::new(source.snap.base.x(), source.snap.base.y(), 0.0);
    target.snap_spacing = Vector3::new(source.snap.spacing.x(), source.snap.spacing.y(), 0.0);
    target.snap_angle = source.snap.angle;
    target.status.isometric_snap = source.snap.style == WorkspaceSnapStyle::Isometric;
    target.status.iso_pair_top = source.snap.isometric_plane == IsometricPlane::Top;
    target.status.iso_pair_right = source.snap.isometric_plane == IsometricPlane::Right;
    let (_, handle, frame) = selection(source.stored_ucs, definitions);
    target.ucs_handle = handle;
    assign_frame(
        &mut target.ucs_origin,
        &mut target.ucs_x_axis,
        &mut target.ucs_y_axis,
        frame,
    );
}

pub(in crate::ocdraw) fn apply(
    drawing: &ValidatedDrawing,
    document: &mut CadDocument,
    scope_layouts: &BTreeMap<u64, Option<String>>,
    diagnostics: &mut Vec<DirectImportDiagnostic>,
) {
    let mut diagnostics = DiagnosticAccumulator(diagnostics);
    let definitions = drawing
        .ucs_definitions()
        .iter()
        .map(|source| {
            (
                source.id,
                (
                    source.definition.name.clone(),
                    document
                        .ucss
                        .get(&source.definition.name)
                        .expect("converted UCS")
                        .handle,
                    source.definition.frame,
                ),
            )
        })
        .collect();
    if let Some(state) = drawing.view_state() {
        let (name, _, frame) = selection(state.current_model_ucs, &definitions);
        document.header.model_space_ucs_name = name;
        assign_frame(
            &mut document.header.model_space_ucs_origin,
            &mut document.header.model_space_ucs_x_axis,
            &mut document.header.model_space_ucs_y_axis,
            frame,
        );
    }
    if !drawing.model_windows().is_empty() {
        document.vports.clear();
        let mut windows = drawing.model_windows().iter().collect::<Vec<_>>();
        if let Some(state) = drawing.view_state() {
            windows.sort_by_key(|w| w.id != state.active_model_window_id);
        }
        for source in windows {
            let mut target = VPort::active();
            target.handle = document.allocate_handle();
            apply_window(&mut target, source, &definitions, &mut diagnostics);
            document.vports.add_allow_duplicate(target);
        }
    }
    for source in drawing.paper_canvases() {
        if source.active_context != DrawingPaperContext::Canvas {
            diagnostics.record(ImportDiagnostic::WorkspaceUnsupported {
                reason: "active paper viewport context".into(),
            });
        }
        let Some(Some(name)) = scope_layouts.get(&u64::from(source.scope_id)) else {
            continue;
        };
        let handle = document.objects.values().find_map(|object| match object {
            ObjectType::Layout(layout) if &layout.name == name => Some(layout.viewport),
            _ => None,
        });
        if let Some(EntityType::Viewport(target)) = handle.and_then(|h| document.get_entity_mut(h))
        {
            apply_canvas(target, source, &definitions, &mut diagnostics);
        } else {
            diagnostics.record(ImportDiagnostic::WorkspaceUnsupported {
                reason: format!("paper canvas for {name} has no CAD viewport"),
            });
        }
    }
    if !drawing.viewport_workspaces().is_empty() {
        diagnostics.record(ImportDiagnostic::WorkspaceUnsupported {
            reason: "paper viewport workspace state".into(),
        });
    }
}
