//! Assemble curated overview drawings without changing the focused fixtures.
use ocdraw::ocdraw::*;
use std::{collections::BTreeMap, error::Error};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn ocdraw(mut model: OcdrawDocument, workspace: OcdrawDocument) -> Result<OcdrawDocument> {
    let source_layout = workspace
        .layouts
        .iter()
        .find(|l| l.kind == DrawingLayoutKind::Paper)
        .ok_or("Paper layout required")?;
    let source_scope = workspace
        .scopes
        .iter()
        .find(|s| s.id == source_layout.scope_id)
        .ok_or("Paper scope required")?;
    let scope_id = model.scopes.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let mut ids = BTreeMap::new();
    for id in &source_scope.entities {
        ids.insert(*id, model.next_entity_id);
        model.next_entity_id += 1;
    }
    let mut layer = workspace.layers[0].clone();
    layer.id = model.next_layer_id;
    model.next_layer_id += 1;
    layer.name = "Paper".into();
    let layer_id = layer.id;
    model.layers.push(layer);
    let placement = CoordinateFrame3::try_new(
        Point3::new(100., 75., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )?;
    for mut entity in workspace
        .geometric_entities
        .iter()
        .filter(|e| ids.contains_key(&e.id))
        .cloned()
    {
        entity.id = ids[&entity.id];
        entity.layer_id = layer_id;
        let DrawingGeometry::PlanarPolyline {
            placement: frame,
            vertices,
            ..
        } = &mut entity.geometry
        else {
            return Err("planar Paper clip required".into());
        };
        *frame = placement;
        for vertex in vertices {
            vertex[0] *= 2.5;
            vertex[1] *= 2.5;
        }
        model.geometric_entities.push(entity);
    }
    for mut viewport in workspace
        .viewports
        .iter()
        .filter(|v| ids.contains_key(&v.id))
        .cloned()
    {
        let old_id = viewport.id;
        viewport.id = ids[&old_id];
        viewport.layer_id = layer_id;
        viewport.frame.center = Point2::new(100., 75.);
        viewport.frame.width *= 2.5;
        viewport.frame.height *= 2.5;
        viewport.view.target = Point3::new(90., 40., 0.);
        viewport.view.height = 160.;
        viewport.paper_clip.boundary_entity_id =
            viewport.paper_clip.boundary_entity_id.map(|id| ids[&id]);
        if let Some(mut state) = workspace
            .viewport_workspaces
            .iter()
            .find(|s| s.viewport_entity_id == old_id)
            .cloned()
        {
            state.viewport_entity_id = viewport.id;
            state.grid.enabled = false;
            model.viewport_workspaces.push(state);
        }
        model.viewports.push(viewport);
    }
    let mut layout = source_layout.clone();
    layout.id = model.next_layout_id;
    model.next_layout_id += 1;
    layout.scope_id = scope_id;
    layout.name = "A3 detail sheet".into();
    layout.settings.media = Some(ocdraw::plot_kernel::LayoutMedia {
        unit: ocdraw::plot_kernel::MediaUnit::from_token("mm").unwrap(),
        width: 420.,
        height: 297.,
    });
    model.layouts.push(layout);
    model.scopes.push(DrawingScope {
        id: scope_id,
        kind: DrawingScopeKind::Paper,
        bounds: None,
        bounds_quality: None,
        entities: source_scope.entities.iter().map(|id| ids[id]).collect(),
    });
    model.drawing_id = "explorer-ocdraw-overview".into();
    let window = &mut model.model_windows[0];
    window.id = 1;
    window.view.target = Point3::new(90., 40., 0.);
    window.view.height = 160.;
    window.grid.enabled = false;
    model.view_state.as_mut().unwrap().active_model_window_id = Some(1);
    recompute_ocdraw_document_bounds(&mut model)?;
    validate_ocdraw_document(&model)?;
    Ok(model)
}

pub fn ifccad(
    mut drawing: ocdraw::ifccad::IfccadDocument,
    state: ocdraw::ifccad::IfccadDocument,
) -> Result<ocdraw::ifccad::IfccadDocument> {
    use ocdraw::ifccad::*;
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
