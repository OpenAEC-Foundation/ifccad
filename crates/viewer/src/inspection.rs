//! Application presentation of validated OCDraw logical geometry.
use ocdraw::ocdraw::{
    AppearanceSelection, CoordinateFrame3, DrawingColor, DrawingGeometry, EntityAppearance,
    ValidatedOcdraw,
};
use serde_json::{json, Value};

fn frame(value: CoordinateFrame3) -> Value {
    let o = value.origin();
    let x = value.x_axis();
    let y = value.y_axis();
    json!({"origin":[o.x(),o.y(),o.z()],"xAxis":[x.x(),x.y(),x.z()],"yAxis":[y.x(),y.y(),y.z()]})
}
fn color(value: &DrawingColor) -> Value {
    json!({"rgb":value.rgb,"indexed":value.indexed,"named":value.named})
}
fn choice<T>(value: &AppearanceSelection<T>, encode: impl FnOnce(&T) -> Value) -> Value {
    match value {
        AppearanceSelection::ByLayer => json!({"mode":"ByLayer"}),
        AppearanceSelection::ByBlock => json!({"mode":"ByBlock"}),
        AppearanceSelection::Explicit(v) => json!({"mode":"Explicit","value":encode(v)}),
    }
}
fn appearance(value: &EntityAppearance) -> Value {
    json!({"color":choice(&value.color,color),"opacity":choice(&value.opacity,|v|json!(v)),"lineWeight":choice(&value.line_weight,|v|json!(v)),"linePattern":choice(&value.line_pattern,|v|json!(v.0)),"linePatternScale":value.line_pattern_scale})
}
fn geometry(value: &DrawingGeometry) -> Value {
    match value {
        DrawingGeometry::Line { start, end } => json!({"type":"line","start":start,"end":end}),
        DrawingGeometry::Point { placement } => {
            json!({"type":"point","placement":frame(*placement)})
        }
        DrawingGeometry::Circle { placement, radius } => {
            json!({"type":"circle","placement":frame(*placement),"radius":radius})
        }
        DrawingGeometry::Arc {
            placement,
            radius,
            start_parameter,
            sweep_parameter,
        } => {
            json!({"type":"arc","placement":frame(*placement),"radius":radius,"startParameter":start_parameter,"sweepParameter":sweep_parameter})
        }
        DrawingGeometry::Ellipse {
            placement,
            semi_major_radius,
            semi_minor_radius,
            arc,
        } => {
            json!({"type":"ellipse","placement":frame(*placement),"semiMajorRadius":semi_major_radius,"semiMinorRadius":semi_minor_radius,"arc":arc})
        }
        DrawingGeometry::PlanarPolyline {
            placement,
            vertices,
            closed,
            line_pattern_generation,
        } => {
            json!({"type":"planarPolyline","placement":frame(*placement),"vertices":vertices,"closed":closed,"linePatternGeneration":format!("{line_pattern_generation:?}")})
        }
        DrawingGeometry::SpatialPolyline {
            vertices,
            closed,
            line_pattern_generation,
        } => {
            json!({"type":"spatialPolyline","vertices":vertices,"closed":closed,"linePatternGeneration":format!("{line_pattern_generation:?}")})
        }
        DrawingGeometry::BlockInstance {
            definition_scope_id,
            transform,
        } => {
            let s = transform.scale();
            json!({"type":"blockInstance","definitionScopeId":definition_scope_id,"placement":frame(transform.placement()),"rotation":transform.rotation(),"scale":[s.x(),s.y(),s.z()]})
        }
    }
}
pub(crate) fn entities(drawing: &ValidatedOcdraw) -> Value {
    let mut output:Vec<Value>=drawing.geometric_entities().iter().map(|e|json!({"id":e.id,"layerId":e.layer_id,"visible":e.visible,"appearance":appearance(&e.appearance),"geometry":geometry(&e.geometry)})).collect();
    for v in drawing.viewports() {
        let mut stored = serde_json::Map::new();
        for stream in drawing.as_value()["streams"]
            .as_object()
            .into_iter()
            .flat_map(|v| v.values())
        {
            if let Some(row) = stream["entityId"]
                .as_array()
                .and_then(|ids| ids.iter().position(|id| id.as_u64() == Some(v.id)))
            {
                if let Some(columns) = stream.as_object() {
                    for (key, value) in columns {
                        if let Some(value) = value.as_array().and_then(|a| a.get(row)) {
                            stored.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }
        output.push(json!({"id":v.id,"layerId":v.layer_id,"type":"viewport","viewScopeId":v.view_scope_id,"visible":v.visible,"appearance":appearance(&v.appearance),"frame":{"center":[v.frame.center.x(),v.frame.center.y()],"width":v.frame.width,"height":v.frame.height},"paperClip":{"enabled":v.paper_clip.enabled,"boundaryEntityId":v.paper_clip.boundary_entity_id},"storedFields":stored}));
    }
    json!(output)
}
