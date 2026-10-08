//! Bind native workspace values to already allocated CAD objects.
use crate::{OcdrawToCadDiagnostic, OcdrawToCadError};
use cad_workspace_convert::{ModelWindowValues, PaperCanvasValues, ViewportAidValues};
use ocdraw::ocdraw::*;
use opencadcodec::objects::ObjectType;
use opencadcodec::{CadDocument, EntityType, Handle, VPort, Vector3};
use std::collections::BTreeMap;
type Definitions = BTreeMap<u32, (String, Handle, CoordinateFrame3, f64)>;
fn selection(
    choice: DrawingUcsSelection,
    definitions: &Definitions,
) -> (String, Handle, CoordinateFrame3, f64) {
    match choice {
        DrawingUcsSelection::World => {
            (String::new(), Handle::NULL, CoordinateFrame3::default(), 0.)
        }
        DrawingUcsSelection::Named(id) => definitions[&id].clone(),
        DrawingUcsSelection::Unnamed(frame) => (String::new(), Handle::NULL, frame, 0.),
    }
}
fn aids(
    grid: DrawingGrid,
    snap: DrawingSnap,
    ucs: DrawingUcsSelection,
    enabled: bool,
    definitions: &Definitions,
) -> (Handle, ViewportAidValues) {
    let (_, handle, ucs_frame, ucs_elevation) = selection(ucs, definitions);
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
fn issues(
    values: Vec<cad_workspace_convert::WorkspaceFieldLoss>,
    location: &str,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) {
    for issue in values {
        diagnostics.push(crate::to_cad::diagnostic(
            "WORKSPACE",
            format!("{location}/{}", issue.field),
            issue.message,
        ));
    }
}
pub(crate) fn apply(
    drawing: &OcdrawDocument,
    document: &mut CadDocument,
    layouts: &BTreeMap<u64, Option<String>>,
    entities: &BTreeMap<u64, Handle>,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) -> Result<(), OcdrawToCadError> {
    let definitions: Definitions = drawing
        .ucs_definitions
        .iter()
        .map(|u| {
            (
                u.id,
                (
                    u.definition.name.clone(),
                    document
                        .ucss
                        .get(&u.definition.name)
                        .expect("allocated UCS")
                        .handle,
                    u.definition.frame,
                    u.definition.elevation,
                ),
            )
        })
        .collect();
    if let Some(current) = drawing.view_state.and_then(|s| s.current_model_ucs) {
        let (name, _, frame, _) = selection(current, &definitions);
        let vector = |v: [f64; 3]| Vector3::new(v[0], v[1], v[2]);
        document.header.model_space_ucs_name = name;
        document.header.model_space_ucs_origin = vector(frame.origin().components());
        document.header.model_space_ucs_x_axis = vector(frame.x_axis().components());
        document.header.model_space_ucs_y_axis = vector(frame.y_axis().components());
    }
    if !drawing.model_windows.is_empty() {
        document.vports.clear();
        let active = drawing.view_state.and_then(|s| s.active_model_window_id);
        let mut windows: Vec<_> = drawing.model_windows.iter().collect();
        if let Some(id) = active {
            if windows.first().is_some_and(|w| w.id != id) {
                diagnostics.push(crate::to_cad::diagnostic(
                    "WORKSPACE",
                    "/modelWindows",
                    "CAD orders the known current window first; authored collection order changes",
                ));
            }
            windows.sort_by_key(|w| w.id != id);
        } else {
            diagnostics.push(crate::to_cad::diagnostic(
                "WORKSPACE",
                "/viewState/activeModelWindowId",
                "CAD cannot encode an unspecified active window separately from table order",
            ));
        }
        for w in windows {
            let (handle, aids) = aids(w.grid, w.snap, w.stored_ucs, w.use_stored_ucs, &definitions);
            let source = ModelWindowValues {
                rectangle: w.rectangle,
                view: w.view,
                aspect_ratio: w.aspect_ratio,
                render_mode: w.render_mode,
                aids,
            };
            let mut target = VPort::active();
            target.handle = document.allocate_handle();
            issues(
                cad_workspace_convert::apply_model_window_to_cad(&mut target, &source)?,
                &format!("/modelWindows/{}", w.id),
                diagnostics,
            );
            target.named_ucs_handle = handle;
            document.vports.add_allow_duplicate(target);
        }
    }
    for canvas in &drawing.paper_canvases {
        let location = format!("/paperCanvases/{}", canvas.scope_id);
        if canvas.active_context.is_some() || canvas.current_ucs.is_some() {
            diagnostics.push(crate::to_cad::diagnostic(
                "WORKSPACE",
                format!("{location}/activeContext"),
                "codec cannot encode the current Paper context/UCS association",
            ));
        }
        let Some(Some(name)) = layouts.get(&u64::from(canvas.scope_id)) else {
            continue;
        };
        let handle = document.objects.values().find_map(|o| match o {
            ObjectType::Layout(l) if &l.name == name => Some(l.viewport),
            _ => None,
        });
        let Some(EntityType::Viewport(target)) = handle.and_then(|h| document.get_entity_mut(h))
        else {
            diagnostics.push(crate::to_cad::diagnostic(
                "WORKSPACE",
                &location,
                "no allocated overall canvas viewport",
            ));
            continue;
        };
        let (handle, aids) = aids(
            canvas.grid,
            canvas.snap,
            canvas.stored_ucs,
            canvas.use_stored_ucs,
            &definitions,
        );
        let source = PaperCanvasValues {
            frame: canvas.frame,
            view: canvas.view,
            aids,
        };
        issues(
            cad_workspace_convert::apply_canvas_to_cad(target, &source)?,
            &location,
            diagnostics,
        );
        target.ucs_handle = handle;
    }
    for w in &drawing.viewport_workspaces {
        let location = format!("/viewportWorkspaces/{}", w.viewport_entity_id);
        let Some(EntityType::Viewport(target)) = entities
            .get(&w.viewport_entity_id)
            .and_then(|h| document.get_entity_mut(*h))
        else {
            diagnostics.push(crate::to_cad::diagnostic(
                "WORKSPACE",
                &location,
                "owning viewport was not constructed; workspace not emitted",
            ));
            continue;
        };
        let (handle, source) = aids(w.grid, w.snap, w.stored_ucs, w.use_stored_ucs, &definitions);
        issues(
            cad_workspace_convert::apply_viewport_aids_to_cad(target, &source)?,
            &location,
            diagnostics,
        );
        target.ucs_handle = handle;
    }
    Ok(())
}
