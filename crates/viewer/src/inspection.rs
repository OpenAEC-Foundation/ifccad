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
    for h in drawing.hatch_entities() {
        let stream = &drawing.as_value()["streams"]["hatchStream"];
        let row = stream["entityId"]
            .as_array()
            .unwrap()
            .iter()
            .position(|v| v.as_u64() == Some(h.id))
            .unwrap();
        let mut loops = stream["loops"][row].clone();
        for l in loops.as_array_mut().unwrap() {
            if let Some(id) = l["sourceEntityId"].as_u64() {
                l["sourceEntityId"] = json!(id.to_string());
            }
        }
        let rule = match h.area_rule {
            ocdraw::geometry_kernel::hatch::HatchAreaRule::Normal => "normal",
            ocdraw::geometry_kernel::hatch::HatchAreaRule::Outer => "outer",
            ocdraw::geometry_kernel::hatch::HatchAreaRule::Ignore => "ignore",
        };
        output.push(json!({"id":h.id.to_string(),"layerId":h.layer_id,"visible":h.visible,"appearance":appearance(&h.appearance),"geometry":{"type":"hatch","placement":frame(h.placement),"areaRule":rule,"joinTolerance":h.join_tolerance,"loops":loops,"fill":stream["fill"][row].clone(),"fillEvaluation":"unassessed"}}));
    }
    for t in drawing.text_entities() {
        let mut geometry = text_geometry(drawing, "textStream", t.id, "text", t.placement);
        geometry["rotation"] = json!(t.rotation);
        geometry["backward"] = json!(t.backward);
        geometry["upsideDown"] = json!(t.upside_down);
        geometry["obliqueAngle"] = json!(t.oblique_angle);
        geometry["thickness"] = json!(t.thickness);
        output.push(json!({"id":t.id.to_string(),"layerId":t.layer_id,"styleId":t.style_id.0,"visible":t.visible,"appearance":appearance(&t.appearance),"geometry":geometry}));
    }
    for t in drawing.mtext_entities() {
        let mut geometry = text_geometry(drawing, "mTextStream", t.id, "mText", t.placement);
        geometry["height"] = json!(t.height);
        geometry["rotation"] = json!(t.rotation);
        geometry["backward"] = json!(t.backward);
        geometry["upsideDown"] = json!(t.upside_down);
        output.push(json!({"id":t.id.to_string(),"layerId":t.layer_id,"styleId":t.style_id.0,"visible":t.visible,"appearance":appearance(&t.appearance),"geometry":geometry}));
    }
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
        output.push(json!({"id":v.id,"layerId":v.layer_id,"type":"viewport","viewScopeId":v.view_scope_id,"visible":v.visible,"appearance":appearance(&v.appearance),"frame":{"center":[v.frame.center.x(),v.frame.center.y()],"width":v.frame.width,"height":v.frame.height},"paperClip":{"enabled":v.paper_clip.enabled,"boundaryEntityId":v.paper_clip.boundary_entity_id},"plotShadingOverride":v.plot_shading_override.map(|mode|format!("{mode:?}")),"layerOverrides":v.layer_overrides.iter().map(|row|json!({"layerId":row.layer_id,"frozen":row.frozen,"color":row.color.as_ref().map(color),"opacity":row.opacity,"linePatternId":row.line_pattern_id.map(|id|id.0),"lineWeight":row.line_weight})).collect::<Vec<_>>(),"storedFields":stored}));
    }
    json!(output)
}

fn text_geometry(
    drawing: &ValidatedOcdraw,
    stream_name: &str,
    id: u64,
    kind: &str,
    placement: CoordinateFrame3,
) -> Value {
    let stream = &drawing.as_value()["streams"][stream_name];
    let row = stream["entityId"]
        .as_array()
        .and_then(|ids| ids.iter().position(|v| v.as_u64() == Some(id)))
        .expect("validated text row");
    let mut value = json!({"type":kind,"placement":frame(placement)});
    for field in [
        "layout",
        "content",
        "attachment",
        "flow",
        "wrapWidth",
        "columns",
        "background",
        "characterFormat",
        "paragraphFormat",
    ] {
        if let Some(v) = stream[field].get(row).filter(|v| !v.is_null()) {
            value[field] = v.clone();
        }
    }
    value
}
