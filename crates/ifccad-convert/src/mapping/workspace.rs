//! IFCCAD-owned identity allocation and workspace bindings.
use crate::{IfccadConversionError as Error, IfccadDiagnostic, IfccadMappings};
use cad_workspace_convert::{ModelWindowValues, PaperCanvasValues, ViewportAidValues};
use ocdraw::geometry_kernel::CoordinateFrame3;
use ocdraw::ifccad::*;
use opencadcodec::objects::ObjectType;
use opencadcodec::{CadDocument, EntityType, Handle, Vector3};
fn loss(
    issues: &mut Vec<IfccadDiagnostic>,
    location: impl Into<String>,
    message: impl Into<String>,
) {
    issues.push(crate::diagnostics::diagnostic(
        "workspace",
        location,
        message,
    ));
}
fn unnamed(frame: CoordinateFrame3) -> IfccadUcsSelection {
    if frame == CoordinateFrame3::default() {
        IfccadUcsSelection::World
    } else {
        IfccadUcsSelection::Unnamed(frame)
    }
}
fn stored(
    handle: Handle,
    frame: CoordinateFrame3,
    elevation: f64,
    document: &IfccadDocument,
    maps: &IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
    location: &str,
) -> IfccadUcsSelection {
    if !handle.is_null() {
        if let Some(id) = maps.ucss.ifccad_id(handle) {
            if document
                .ucs_definitions
                .iter()
                .any(|u| u.id.0 == id && u.frame == frame && u.elevation == elevation)
            {
                return IfccadUcsSelection::Named(IfccadUcsId(id));
            }
            loss(
                issues,
                format!("{location}.storedUcs"),
                "named definition disagrees with stored frame/elevation; retained unnamed frame",
            );
        } else {
            loss(
                issues,
                format!("{location}.storedUcs"),
                "named handle unresolved; retained unnamed frame",
            );
        }
    }
    if elevation != 0. {
        loss(
            issues,
            format!("{location}.storedUcs.elevation"),
            "unnamed/World selection cannot retain separate elevation",
        );
    }
    unnamed(frame)
}
fn field_losses(
    values: Vec<cad_workspace_convert::WorkspaceFieldLoss>,
    location: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    for v in values {
        loss(issues, format!("{location}.{}", v.field), v.message);
    }
}
pub(crate) fn from_cad(
    source: &CadDocument,
    drawing: &mut IfccadDocument,
    maps: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), Error> {
    for u in source.ucss.iter() {
        let frame =
            cad_workspace_convert::prepare_ucs_frame_from_cad(u.origin, u.x_axis, u.y_axis)?;
        if !u.elevation.is_finite() {
            return Err(cad_workspace_convert::WorkspaceNumericError {
                field: "ucs.elevation",
                message: "nonfinite elevation".into(),
            }
            .into());
        }
        let id = drawing.id_counters.allocate_ucs_id()?;
        maps.ucss.insert(id.0, u.handle);
        drawing.ucs_definitions.push(IfccadUcsDefinition {
            id,
            name: u.name.clone(),
            frame,
            elevation: u.elevation,
        });
    }
    let header = &source.header;
    let header_frame = cad_workspace_convert::prepare_ucs_frame_from_cad(
        header.model_space_ucs_origin,
        header.model_space_ucs_x_axis,
        header.model_space_ucs_y_axis,
    )?;
    let current = if header.model_space_ucs_name.is_empty() {
        Some(unnamed(header_frame))
    } else {
        let candidates: Vec<_> = drawing
            .ucs_definitions
            .iter()
            .filter(|u| u.name.eq_ignore_ascii_case(&header.model_space_ucs_name))
            .collect();
        if candidates.len() == 1 && candidates[0].frame == header_frame {
            Some(IfccadUcsSelection::Named(candidates[0].id))
        } else {
            loss(
                issues,
                "header.model_space_ucs",
                "unresolved, ambiguous or inconsistent current UCS; choice unspecified",
            );
            None
        }
    };
    for v in source
        .vports
        .iter()
        .filter(|v| v.name.eq_ignore_ascii_case("*active"))
    {
        let fields = cad_workspace_convert::prepare_model_window_from_cad(v)?;
        let value = fields.value;
        let location = format!("vport/{}", v.handle);
        field_losses(fields.losses, &location, issues);
        let stored_ucs = stored(
            v.named_ucs_handle,
            value.aids.ucs_frame,
            value.aids.ucs_elevation,
            drawing,
            maps,
            issues,
            &location,
        );
        let id = drawing.id_counters.allocate_model_window_id()?;
        maps.model_windows.insert(id.0, v.handle);
        drawing.model_windows.push(IfccadModelWindow {
            id,
            rectangle: value.rectangle,
            view: value.view,
            aspect_ratio: value.aspect_ratio,
            render_mode: value.render_mode,
            grid: value.aids.grid,
            snap: value.aids.snap,
            stored_ucs,
            use_stored_ucs: value.aids.use_stored_ucs,
        });
    }
    let active = match drawing.model_windows.as_slice() {
        [w] if !w.use_stored_ucs || current.is_none_or(|c| c == w.stored_ucs) => Some(w.id),
        _ => None,
    };
    if !drawing.model_windows.is_empty() && active.is_none() {
        loss(
            issues,
            "drawing.modelViewState.activeModelWindow",
            "current association unavailable or inconsistent; windows retained",
        );
    }
    drawing.model_view_state = Some(IfccadModelViewState {
        current_model_ucs: current,
        active_model_window_id: active,
    });
    let current_layer_id = source
        .layers
        .get(&header.current_layer_name)
        .and_then(|l| maps.layers.ifccad_id(l.handle));
    if current_layer_id.is_none() {
        loss(
            issues,
            "header.current_layer_name",
            "unresolved current layer; choice unspecified",
        );
    }
    let active_layout_id = if header.show_model_space {
        Some(drawing.model.id)
    } else {
        let papers: Vec<_> = source
            .objects
            .values()
            .filter_map(|o| match o {
                ObjectType::Layout(l)
                    if source
                        .block_records
                        .iter()
                        .any(|b| b.handle == l.block_record && b.is_paper_space()) =>
                {
                    Some(l)
                }
                _ => None,
            })
            .collect();
        if papers.len() == 1 {
            maps.layouts.ifccad_id(papers[0].handle)
        } else {
            None
        }
    };
    if active_layout_id.is_none() {
        loss(
            issues,
            "header.paper_space_block_handle",
            "active Paper layout association unavailable",
        );
    }
    drawing.workspace_state = Some(IfccadDrawingWorkspaceState {
        current_layer_id,
        active_layout_id,
    });
    for index in 0..drawing.paper_layouts.len() {
        let id = drawing.paper_layouts[index].id;
        let Some(ObjectType::Layout(layout)) = maps
            .layouts
            .cad_handle(id)
            .and_then(|h| source.objects.get(&h))
        else {
            continue;
        };
        let Some(handle) = crate::source::workspace::overall_handle(source, layout) else {
            if source.entities().any(|e|matches!(e,EntityType::Viewport(v) if v.common.owner_handle == layout.block_record)) {loss(issues,format!("layout/{}.canvas",layout.name),"overall role unavailable");}
            continue;
        };
        let Some(EntityType::Viewport(v)) = source.get_entity(handle) else {
            continue;
        };
        let fields = cad_workspace_convert::prepare_canvas_from_cad(v)?;
        let value = fields.value;
        let location = format!("entity/{handle}.canvas");
        let stored_ucs = stored(
            v.ucs_handle,
            value.aids.ucs_frame,
            value.aids.ucs_elevation,
            drawing,
            maps,
            issues,
            &location,
        );
        field_losses(fields.losses, &location, issues);
        crate::source::workspace::canvas_residual(v, issues);
        loss(
            issues,
            format!("{location}.activeContext/currentUcs"),
            "codec does not expose current Paper context/UCS association",
        );
        drawing.paper_layouts[index].canvas = Some(IfccadPaperCanvas {
            view: value.view,
            frame: value.frame,
            grid: value.aids.grid,
            snap: value.aids.snap,
            stored_ucs,
            use_stored_ucs: value.aids.use_stored_ucs,
            current_ucs: None,
            active_context: None,
        });
    }
    for index in 0..drawing.paper_layouts.len() {
        for item in 0..drawing.paper_layouts[index].entities.len() {
            let id = drawing.paper_layouts[index].entities[item].id();
            if !drawing.paper_layouts[index].entities[item].as_native().is_some_and(|e|matches!(e.kind,IfccadEntityKind::Viewport(_))) {
                continue;
            }
            let Some(EntityType::Viewport(v)) = maps
                .entities
                .cad_handle(id)
                .and_then(|h| source.get_entity(h))
            else {
                continue;
            };
            let fields = cad_workspace_convert::prepare_viewport_aids_from_cad(v)?;
            let value = fields.value;
            let location = format!("entity/{}", v.common.handle);
            let stored_ucs = stored(
                v.ucs_handle,
                value.ucs_frame,
                value.ucs_elevation,
                drawing,
                maps,
                issues,
                &location,
            );
            field_losses(fields.losses, &location, issues);
            if let IfccadEntityKind::Viewport(view) =
                &mut drawing.paper_layouts[index].entities[item].as_native_mut().expect("native viewport").kind
            {
                view.workspace = Some(Box::new(IfccadViewportWorkspace {
                    grid: value.grid,
                    snap: value.snap,
                    stored_ucs,
                    use_stored_ucs: value.use_stored_ucs,
                }));
            }
        }
    }
    Ok(())
}
fn binding(
    selection: IfccadUcsSelection,
    drawing: &IfccadDocument,
    maps: &IfccadMappings,
) -> (Handle, CoordinateFrame3, f64) {
    match selection {
        IfccadUcsSelection::World => (Handle::NULL, CoordinateFrame3::default(), 0.),
        IfccadUcsSelection::Unnamed(frame) => (Handle::NULL, frame, 0.),
        IfccadUcsSelection::Named(id) => {
            let u = drawing
                .ucs_definitions
                .iter()
                .find(|u| u.id == id)
                .expect("validated UCS");
            (
                maps.ucss.cad_handle(id.0).expect("allocated UCS"),
                u.frame,
                u.elevation,
            )
        }
    }
}
fn aids(
    grid: ocdraw::workspace_kernel::WorkspaceGrid,
    snap: ocdraw::workspace_kernel::WorkspaceSnap,
    selection: IfccadUcsSelection,
    enabled: bool,
    drawing: &IfccadDocument,
    maps: &IfccadMappings,
) -> (Handle, ViewportAidValues) {
    let (handle, ucs_frame, ucs_elevation) = binding(selection, drawing, maps);
    (
        handle,
        ViewportAidValues {
            grid,
            snap,
            ucs_frame,
            ucs_elevation,
            use_stored_ucs: enabled,
        },
    )
}
fn vector(values: [f64; 3]) -> Vector3 {
    Vector3::new(values[0], values[1], values[2])
}
pub(crate) fn to_cad(
    drawing: &IfccadDocument,
    target: &mut CadDocument,
    maps: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), Error> {
    for u in &drawing.ucs_definitions {
        let mut value = opencadcodec::Ucs::new(&u.name);
        value.handle = target.allocate_handle();
        value.origin = vector(u.frame.origin().components());
        value.x_axis = vector(u.frame.x_axis().components());
        value.y_axis = vector(u.frame.y_axis().components());
        value.elevation = u.elevation;
        maps.ucss.insert(u.id.0, value.handle);
        target
            .ucss
            .add(value)
            .map_err(|e| Error::CadConstruction(format!("UCS {}: {e}", u.id.0)))?;
    }
    if let Some(current) = drawing
        .model_view_state
        .as_ref()
        .and_then(|s| s.current_model_ucs)
    {
        let (_, frame, _) = binding(current, drawing, maps);
        target.header.model_space_ucs_name = match current {
            IfccadUcsSelection::Named(id) => drawing
                .ucs_definitions
                .iter()
                .find(|u| u.id == id)
                .unwrap()
                .name
                .clone(),
            _ => String::new(),
        };
        target.header.model_space_ucs_origin = vector(frame.origin().components());
        target.header.model_space_ucs_x_axis = vector(frame.x_axis().components());
        target.header.model_space_ucs_y_axis = vector(frame.y_axis().components());
    }
    if !drawing.model_windows.is_empty() {
        target.vports.clear();
        let active = drawing
            .model_view_state
            .as_ref()
            .and_then(|s| s.active_model_window_id);
        let mut windows: Vec<_> = drawing.model_windows.iter().collect();
        if let Some(id) = active {
            if windows.first().is_some_and(|w| w.id != id) {
                loss(
                    issues,
                    "drawing.modelWindows",
                    "CAD orders the known current window first; authored collection order changes",
                );
            }
            windows.sort_by_key(|w| w.id != id);
        } else {
            loss(
                issues,
                "drawing.modelViewState.activeModelWindow",
                "CAD cannot encode unspecified active identity separately from table order",
            );
        }
        for w in windows {
            let (handle, aids) = aids(
                w.grid,
                w.snap,
                w.stored_ucs,
                w.use_stored_ucs,
                drawing,
                maps,
            );
            let source = ModelWindowValues {
                rectangle: w.rectangle,
                view: w.view,
                aspect_ratio: w.aspect_ratio,
                render_mode: w.render_mode,
                aids,
            };
            let mut value = opencadcodec::VPort::active();
            value.handle = target.allocate_handle();
            field_losses(
                cad_workspace_convert::apply_model_window_to_cad(&mut value, &source)?,
                &format!("modelWindow/{}", w.id.0),
                issues,
            );
            value.named_ucs_handle = handle;
            maps.model_windows.insert(w.id.0, value.handle);
            target.vports.add_allow_duplicate(value);
        }
    }
    if let Some(state) = &drawing.workspace_state {
        if let Some(id) = state.current_layer_id {
            let layer = drawing
                .layers
                .iter()
                .find(|l| l.id == id)
                .expect("validated layer");
            target.header.current_layer_name = layer.name.clone();
            target.header.current_layer_handle =
                maps.layers.cad_handle(id).expect("allocated layer");
        }
        if let Some(id) = state.active_layout_id {
            target.header.show_model_space = id == drawing.model.id;
            if !target.header.show_model_space && drawing.paper_layouts.len() > 1 {
                loss(issues,format!("/cad/d{}.ifccad::drawingWorkspace.activeLayout",drawing.drawing_id),"CAD retains Model/Paper mode but does not expose the selected Paper tab among several layouts");
            }
            let settings = if id == drawing.model.id {
                &drawing.model.settings
            } else {
                &drawing
                    .paper_layouts
                    .iter()
                    .find(|p| p.id == id)
                    .expect("validated layout")
                    .settings
            };
            target.header.paper_space_linetype_scaling = settings.paper_space_linetype_scaling;
        }
    }
    for paper in &drawing.paper_layouts {
        if let Some(canvas) = &paper.canvas {
            let location = format!("layout/{}.canvas", paper.id);
            if canvas.active_context.is_some() || canvas.current_ucs.is_some() {
                loss(
                    issues,
                    format!("{location}.activeContext"),
                    "codec cannot encode current Paper context/UCS association",
                );
            }
            let overall = maps
                .layouts
                .cad_handle(paper.id)
                .and_then(|h| target.objects.get(&h))
                .and_then(|o| match o {
                    ObjectType::Layout(l) => crate::source::workspace::overall_handle(target, l),
                    _ => None,
                });
            let (handle, aids) = aids(
                canvas.grid,
                canvas.snap,
                canvas.stored_ucs,
                canvas.use_stored_ucs,
                drawing,
                maps,
            );
            if let Some(EntityType::Viewport(value)) =
                overall.and_then(|h| target.get_entity_mut(h))
            {
                let source = PaperCanvasValues {
                    frame: canvas.frame,
                    view: canvas.view,
                    aids,
                };
                field_losses(
                    cad_workspace_convert::apply_canvas_to_cad(value, &source)?,
                    &location,
                    issues,
                );
                value.ucs_handle = handle;
            } else {
                loss(
                    issues,
                    &location,
                    "no allocated overall viewport; canvas not emitted",
                );
            }
        }
        for entity in paper.entities.iter().filter_map(IfccadEntity::as_native) {
            let IfccadEntityKind::Viewport(view) = &entity.kind else {
                continue;
            };
            let Some(workspace) = &view.workspace else {
                continue;
            };
            let location = format!("entity/{}.workspace", entity.id);
            let (handle, aids) = aids(
                workspace.grid,
                workspace.snap,
                workspace.stored_ucs,
                workspace.use_stored_ucs,
                drawing,
                maps,
            );
            if let Some(EntityType::Viewport(value)) = maps
                .entities
                .cad_handle(entity.id)
                .and_then(|h| target.get_entity_mut(h))
            {
                field_losses(
                    cad_workspace_convert::apply_viewport_aids_to_cad(value, &aids)?,
                    &location,
                    issues,
                );
                value.ucs_handle = handle;
            } else {
                loss(
                    issues,
                    location,
                    "owning viewport was not constructed; workspace not emitted",
                );
            }
        }
    }
    Ok(())
}
