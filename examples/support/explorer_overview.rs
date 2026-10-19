//! Assemble curated overview drawings without changing the focused fixtures.
use ifccad::geometry_kernel::Point3;
use std::error::Error;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn ifccad(
    mut drawing: ifccad::IfccadDocument,
    state: ifccad::IfccadDocument,
) -> Result<ifccad::IfccadDocument> {
    use ifccad::*;
    drawing.ucs_definitions = state.ucs_definitions;
    drawing.ucs_definitions[0].id = IfccadUcsId(1);
    drawing.model_windows = state.model_windows;
    drawing.model_windows[0].id = IfccadModelWindowId(1);
    drawing.model_windows[0].view.target = Point3::new(90., 40., 0.);
    drawing.model_windows[0].view.height = 160.;
    drawing.model_windows[0].grid.enabled = false;
    drawing.model_view_state = Some(IfccadModelViewState {
        current_model_ucs: Some(IfccadUcsSelection::Named(IfccadUcsId(1))),
        active_model_window_id: Some(IfccadModelWindowId(1)),
    });
    drawing.workspace_state = state.workspace_state;
    drawing.id_counters.next_ucs_id = 2;
    drawing.id_counters.next_model_window_id = 2;
    let workspace = state
        .paper_layouts
        .iter()
        .flat_map(|p| &p.entities)
        .filter_map(IfccadEntity::as_native)
        .find_map(|entity| match &entity.kind {
            IfccadEntityKind::Viewport(viewport) => viewport.workspace.clone(),
            _ => None,
        });
    for entity in drawing
        .paper_layouts
        .iter_mut()
        .flat_map(|p| &mut p.entities)
        .filter_map(IfccadEntity::as_native_mut)
    {
        if let IfccadEntityKind::Viewport(viewport) = &mut entity.kind {
            viewport.workspace = workspace.clone();
            if let Some(state) = &mut viewport.workspace {
                state.grid.enabled = false;
            }
        }
    }
    recompute_ifccad_document_bounds(&mut drawing)?;
    validate_ifccad_document(&drawing)?;
    Ok(drawing)
}
