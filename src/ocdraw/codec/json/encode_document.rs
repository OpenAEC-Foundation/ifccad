use super::{
    encode_color, encode_plot_settings, encode_point_display, encode_rect, encode_ucs_definition,
    object_columns, polyline_columns,
};
use crate::ocdraw::logical::{DrawingGeometricEntity, EntityGeometry};
use crate::ocdraw::{DrawingLayoutKind, DrawingScopeKind, OcdrawDocument, PlotStyleMode};
use serde_json::{json, Map, Value};

pub(crate) fn encode_document(doc: &OcdrawDocument) -> Value {
    let layer_values = doc
        .layers
        .iter()
        .map(|layer| {
            let id = layer.id;
            let mut value = json!({
                "id": id, "name": layer.name, "visible": layer.visible, "frozen": layer.frozen,
                "locked": layer.locked, "plottable": layer.plottable,
                "frozenInNewViewports": layer.frozen_in_new_viewports,
                "color": encode_color(&layer.color), "opacity": layer.opacity,
                "linePatternId": layer.line_pattern_id, "lineWeight": layer.line_weight,
            });
            if let Some(description) = &layer.description {
                value["description"] = json!(description);
            }
            value
        })
        .collect::<Vec<_>>();
    let scopes=doc.scopes.iter().map(|s| {
        let bounds=s.bounds.map(|b| json!({"minX":b.min().x(),"minY":b.min().y(),"minZ":b.min().z(),"maxX":b.max().x(),"maxY":b.max().y(),"maxZ":b.max().z()}));
        json!({"id":s.id,"kind":match s.kind {DrawingScopeKind::Model=>0,DrawingScopeKind::Paper=>1,DrawingScopeKind::Block=>2},"bounds":bounds,"entities":s.entities})
    }).collect::<Vec<_>>();
    let mut layouts=doc.layouts.iter().map(|l|json!({"id":l.id,"scopeId":l.scope_id,"kind":match l.kind {DrawingLayoutKind::Model=>"model",DrawingLayoutKind::Paper=>"paper"},"name":l.name,"tabIndex":l.tab_index})).collect::<Vec<_>>();
    for (layout, source) in layouts.iter_mut().zip(&doc.layouts) {
        let settings = &source.settings;
        if let Some(limits) = settings.limits {
            layout["limits"] = encode_rect(limits).expect("validated limits");
        }
        if settings.limits_checking {
            layout["limitsChecking"] = json!(true);
        }
        if !settings.paper_space_linetype_scaling {
            layout["paperSpaceLinetypeScaling"] = json!(false);
        }
        if let Some(plot) = &settings.plot_settings {
            layout["plotSettings"] = encode_plot_settings(plot).expect("validated plot settings");
        }
    }
    let block_definitions = doc
            .block_definitions
            .iter()
            .map(|definition| {
                let mut item = json!({"scopeId":definition.scope_id,"name":definition.name});
                if definition.base_point != [0.0; 3] {
                    item["basePoint"] = json!({"x": definition.base_point[0], "y": definition.base_point[1], "z": definition.base_point[2]});
                }
                if !definition.description.is_empty() {
                    item["description"] = json!(definition.description);
                }
                if definition.anonymous {
                    item["anonymous"] = json!(true);
                }
                if definition.insertion_unit != "unitless" {
                    item["insertionUnit"] = json!(definition.insertion_unit);
                }
                if !definition.explodable {
                    item["explodable"] = json!(false);
                }
                if definition.uniform_scaling {
                    item["scaling"] = json!("Uniform");
                }
                item
            })
            .collect::<Vec<_>>();
    let mut value = json!({
        "header": {"format":"open_cad_drawing", "version":"0.1.0", "drawingId": doc.drawing_id,
            "unit": doc.unit, "nextEntityId": doc.next_entity_id,
            "nextLinePatternId": doc.next_line_pattern_id, "nextLayerId": doc.next_layer_id, "nextLayoutId": doc.next_layout_id},
        "layouts": layouts,
        "scopes": scopes,
        "streams": {}
    });
    if !doc.line_patterns.is_empty() {
        value["linePatterns"] = json!(doc
            .line_patterns
            .iter()
            .map(|row| {
                let id = row.id;
                let mut v = json!({"id":id,"name":row.name,"pattern":row.pattern});
                if let Some(d) = &row.description {
                    v["description"] = json!(d);
                }
                v
            })
            .collect::<Vec<_>>());
    }
    if doc.line_pattern_scale != 1.0 {
        value["linePatternScale"] = json!(doc.line_pattern_scale);
    }
    if !layer_values.is_empty() {
        value["layers"] = json!(layer_values);
    }
    if !block_definitions.is_empty() {
        value["blockDefinitions"] = json!(block_definitions);
    }
    if !doc.ucs_definitions.is_empty() {
        value["ucsDefinitions"] = Value::Array(
            doc.ucs_definitions
                .iter()
                .map(|ucs| encode_ucs_definition(&ucs.definition, ucs.id))
                .collect(),
        );
    }
    if doc.plot_style_mode == PlotStyleMode::Named {
        value["plotStyleMode"] = json!("named");
    }
    if let Some(display) = doc.point_display {
        value["pointDisplay"] = encode_point_display(display);
    }
    let mut workspace = Map::new();
    if let Some(id) = doc.workspace_state.and_then(|s| s.current_layer_id) {
        workspace.insert("currentLayerId".into(), json!(id));
    }
    if let Some(id) = doc.workspace_state.and_then(|s| s.active_layout_id) {
        workspace.insert("activeLayoutId".into(), json!(id));
    }
    if !workspace.is_empty() {
        value["drawingWorkspaceState"] = Value::Object(workspace);
    }
    super::encode_view_state::encode_document_state(&mut value, doc);
    if !doc.geometric_entities.is_empty() || !doc.viewports.is_empty() {
        let mut groups = std::collections::BTreeMap::<&str, Vec<&DrawingGeometricEntity>>::new();
        for object in &doc.geometric_entities {
            let kind = match object.geometry {
                EntityGeometry::Line { .. } => "line",
                EntityGeometry::Point { .. } => "point",
                EntityGeometry::Circle { .. } => "circle",
                EntityGeometry::Arc { .. } => "arc",
                EntityGeometry::Ellipse { arc: None, .. } => "ellipse",
                EntityGeometry::Ellipse { arc: Some(_), .. } => "ellipseArc",
                EntityGeometry::PlanarPolyline { .. } => "planarPolyline",
                EntityGeometry::SpatialPolyline { .. } => "spatialPolyline",
                EntityGeometry::BlockInstance { .. } => "blockInstance",
            };
            groups.entry(kind).or_default().push(object);
        }
        for (kind, rows) in groups {
            let columns = if matches!(kind, "planarPolyline" | "spatialPolyline") {
                polyline_columns(&rows, kind)
            } else {
                object_columns(&rows)
            };
            let payload = match kind {
                "line" => "lineStream",
                "point" => "pointStream",
                "circle" => "circleStream",
                "arc" => "arcStream",
                "ellipse" => "ellipseStream",
                "ellipseArc" => "ellipseArcStream",
                "planarPolyline" => "planarPolylineStream",
                "spatialPolyline" => "spatialPolylineStream",
                "blockInstance" => "blockInstanceStream",
                _ => unreachable!("only typed object kinds enter the doc"),
            };
            value["streams"][payload] = Value::Object(columns);
        }
        super::encode_view_state::encode_viewports(&mut value, &doc.viewports);
    }
    value
}

pub(crate) fn encode_document_bytes(doc: &OcdrawDocument) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec_pretty(&encode_document(doc))
}
