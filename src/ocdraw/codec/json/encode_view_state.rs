use crate::ocdraw::*;
use serde_json::{json, Value};

fn point2(p: Point2) -> Value {
    json!({"x":p.x(),"y":p.y()})
}
fn point3(p: Point3) -> Value {
    json!({"x":p.x(),"y":p.y(),"z":p.z()})
}
fn vector3(p: Vector3) -> Value {
    json!({"x":p.x(),"y":p.y(),"z":p.z()})
}
fn clip(source: DrawingClip) -> Value {
    let mut value = json!({"mode":match source.mode {DrawingClipMode::Disabled=>"Disabled",DrawingClipMode::AtCamera=>"AtCamera",DrawingClipMode::AtDistance=>"AtDistance"}});
    if let Some(distance) = source.distance {
        value["distance"] = json!(distance);
    }
    value
}
pub(super) fn view(source: DrawingView) -> Value {
    let mut value = json!({"center":point2(source.center),"target":point3(source.target),"direction":vector3(source.direction),"height":source.height,"twist":source.twist,
        "projection":match source.projection {DrawingProjection::Orthographic=>"Orthographic",DrawingProjection::Perspective=>"Perspective"},"frontClip":clip(source.front_clip),"backClip":clip(source.back_clip)});
    if let Some(lens) = source.lens_length {
        value["lensLength"] = json!(lens);
    }
    value
}
fn grid(source: DrawingGrid) -> Value {
    json!({"enabled":source.enabled,"spacing":point2(source.spacing),"style":match source.style {DrawingGridStyle::Lines=>"Lines",DrawingGridStyle::Dots=>"Dots"},"majorLineFrequency":source.major_line_frequency,
    "beyondLimits":source.beyond_limits,"adaptive":source.adaptive,"subdivision":source.subdivision,"followsWorkplane":source.follows_workplane})
}
fn snap(source: DrawingSnap) -> Value {
    json!({"enabled":source.enabled,"base":point2(source.base),"spacing":point2(source.spacing),"angle":source.angle,
    "style":match source.style {DrawingSnapStyle::Rectangular=>"Rectangular",DrawingSnapStyle::Isometric=>"Isometric"},
    "isometricPlane":match source.isometric_plane {DrawingIsometricPlane::Left=>"Left",DrawingIsometricPlane::Top=>"Top",DrawingIsometricPlane::Right=>"Right"}})
}
fn ucs(source: DrawingUcsSelection) -> Value {
    match source {
        DrawingUcsSelection::World => json!({"kind":"World"}),
        DrawingUcsSelection::Named(id) => json!({"kind":"Named","ucsId":id}),
        DrawingUcsSelection::Unnamed(frame) => {
            json!({"kind":"Unnamed","frame":super::encode_geometry::placement_frame(frame)})
        }
    }
}
pub(super) fn render_mode(source: DrawingRenderMode) -> &'static str {
    match source {
        DrawingRenderMode::TwoDimensional => "TwoDimensional",
        DrawingRenderMode::Wireframe => "Wireframe",
        DrawingRenderMode::HiddenLine => "HiddenLine",
        DrawingRenderMode::FlatShadedWithoutEdges => "FlatShadedWithoutEdges",
        DrawingRenderMode::FlatShadedWithEdges => "FlatShadedWithEdges",
        DrawingRenderMode::SmoothShadedWithoutEdges => "SmoothShadedWithoutEdges",
        DrawingRenderMode::SmoothShadedWithEdges => "SmoothShadedWithEdges",
    }
}
pub(super) fn encode_saved_state(root: &mut Value, state: &DrawingSavedState) {
    if let Some(source) = state.view_state {
        root["drawingViewState"] = json!({"currentModelUcs":ucs(source.current_model_ucs),"activeModelWindowId":source.active_model_window_id});
    }
    if !state.model_windows.is_empty() {
        root["modelWindows"]=json!(state.model_windows.iter().map(|source|json!({"modelWindowId":source.id,
        "rectangle":{"minX":source.rectangle[0],"minY":source.rectangle[1],"maxX":source.rectangle[2],"maxY":source.rectangle[3]},"view":view(source.view),
        "aspectRatio":source.aspect_ratio,"renderMode":render_mode(source.render_mode),"grid":grid(source.grid),"snap":snap(source.snap),"storedUcs":ucs(source.stored_ucs),"useStoredUcs":source.use_stored_ucs})).collect::<Vec<_>>());
    }
    if !state.paper_canvases.is_empty() {
        root["paperCanvases"]=json!(state.paper_canvases.iter().map(|source|json!({"scopeId":source.scope_id,"view":view(source.view),
        "grid":grid(source.grid),"snap":snap(source.snap),"storedUcs":ucs(source.stored_ucs),"currentUcs":ucs(source.current_ucs),"activeContext":match source.active_context {DrawingPaperContext::Canvas=>json!({"kind":"Canvas"}),DrawingPaperContext::Viewport(id)=>json!({"kind":"Viewport","viewportEntityId":id})}})).collect::<Vec<_>>());
    }
    if !state.viewport_workspaces.is_empty() {
        root["viewportWorkspaces"]=json!(state.viewport_workspaces.iter().map(|source|json!({"viewportEntityId":source.viewport_entity_id,
        "grid":grid(source.grid),"snap":snap(source.snap),"storedUcs":ucs(source.stored_ucs),"useStoredUcs":source.use_stored_ucs})).collect::<Vec<_>>());
    }
}
pub(super) fn encode_viewports(root: &mut Value, rows: &[DrawingViewport]) {
    if rows.is_empty() {
        return;
    }
    let mut fields = Vec::new();
    let mut overrides = Vec::new();
    for row in rows {
        let mut paper_clip = json!({"enabled":row.paper_clip.enabled});
        if let Some(id) = row.paper_clip.boundary_entity_id {
            paper_clip["boundaryEntityId"] = json!(id);
        }
        let field = json!({"entityId":row.id,"viewScopeId":row.view_scope_id,"layerId":row.layer_id,"visible":row.visible,
            "frame":{"center":point2(row.frame.center),"width":row.frame.width,"height":row.frame.height},"view":view(row.view),"renderMode":render_mode(row.render_mode),
            "viewEnabled":row.view_enabled,"viewLocked":row.view_locked,"paperClip":paper_clip,"plotShadingOverride":row.plot_shading_override.map(super::encode_plot::encode_shading),
            "layerOverrideOffset":overrides.len(),"layerOverrideCount":row.layer_overrides.len()});
        for entry in &row.layer_overrides {
            let mut item = json!({"layerId":entry.layer_id,"frozen":entry.frozen});
            if let Some(color) = &entry.color {
                item["color"] = super::encode_color(color);
            }
            if let Some(opacity) = entry.opacity {
                item["opacity"] = json!(opacity);
            }
            if let Some(pattern) = &entry.line_pattern {
                item["linePattern"] = json!(pattern);
            }
            if let Some(weight) = entry.line_weight {
                item["lineWeight"] = json!(weight);
            }
            overrides.push(item);
        }
        // All rows author the same physical columns; optional values use nulls.
        fields.push(field);
    }
    let mut columns = serde_json::Map::new();
    columns.insert("count".into(), json!(rows.len()));
    for key in fields[0].as_object().unwrap().keys() {
        columns.insert(
            key.clone(),
            json!(fields
                .iter()
                .map(|field| field[key].clone())
                .collect::<Vec<_>>()),
        );
    }
    super::encode_geometry::appearance_columns(
        &mut columns,
        &rows.iter().map(|row| &row.appearance).collect::<Vec<_>>(),
    );
    root["streams"]["viewportStream"] = Value::Object(columns);
    if !overrides.is_empty() {
        let names = [
            "layerId",
            "frozen",
            "color",
            "opacity",
            "linePattern",
            "lineWeight",
        ];
        let mut columns = serde_json::Map::new();
        columns.insert("count".into(), json!(overrides.len()));
        for key in names {
            columns.insert(
                key.into(),
                json!(overrides
                    .iter()
                    .map(|row| row.get(key).cloned().unwrap_or(Value::Null))
                    .collect::<Vec<_>>()),
            );
        }
        root["streams"]["viewportLayerOverrideStream"] = Value::Object(columns);
    }
}
