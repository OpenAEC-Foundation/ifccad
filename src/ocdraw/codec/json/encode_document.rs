use super::{
    encode_color, encode_plot_settings, encode_point_display, encode_rect, encode_ucs_definition,
    object_columns, polyline_columns,
};
use crate::ocdraw::logical::DrawingEntityRecord;
use crate::ocdraw::write::{DrawingBuilder, PlotStyleMode};
use serde_json::{json, Map, Value};

pub(crate) fn encode_document(
    builder: &DrawingBuilder,
    bounds: &[Option<([f64; 3], [f64; 3])>],
) -> Value {
    let first_block_scope = builder.paper_layouts.len() + 1;
    let layer_values = builder
        .layers
        .iter()
        .enumerate()
        .map(|(id, layer)| {
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
    let bounds_json = |bounds: Option<([f64; 3], [f64; 3])>| {
        bounds.map(|(min,max)| json!({"minX":min[0],"minY":min[1],"minZ":min[2],"maxX":max[0],"maxY":max[1],"maxZ":max[2]}))
    };
    let mut layouts = vec![
        json!({"id":0,"scopeId":0,"kind":"model","name":builder.model_layout_name,"tabIndex":0}),
    ];
    let mut scopes = vec![json!({"id":0,"kind":0,"bounds":bounds_json(bounds[0])})];
    for (index, name) in builder.paper_layouts.iter().enumerate() {
        let id = index + 1;
        layouts.push(json!({"id":id,"scopeId":id,"kind":"paper","name":name,"tabIndex":id}));
        scopes.push(json!({"id":id,"kind":1,"bounds":bounds_json(bounds[id])}));
    }
    for (index, layout) in layouts.iter_mut().enumerate() {
        let settings = &builder.layout_settings[index];
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
    let block_definitions = builder
            .block_definitions
            .iter()
            .enumerate()
            .map(|(index, definition)| {
                let id = first_block_scope + index;
                scopes.push(json!({"id":id,"kind":2,"bounds":bounds_json(bounds[id])}));
                let mut item = json!({"scopeId":id,"name":definition.name});
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
        "header": {"format":"open_cad_drawing", "version":"0.1.0", "drawingId": builder.options.drawing_id,
            "unit": builder.options.unit, "nextEntityId": builder.next_entity_id,
            "nextLinePatternId": builder.line_patterns.len(), "nextLayerId": builder.layers.len(), "nextLayoutId": layouts.len()},
        "layouts": layouts,
        "scopes": scopes,
        "streams": {}
    });
    if !builder.line_patterns.is_empty() {
        value["linePatterns"] = json!(builder
            .line_patterns
            .iter()
            .enumerate()
            .map(|(id, row)| {
                let mut v = json!({"id":id,"name":row.name,"pattern":row.pattern});
                if let Some(d) = &row.description {
                    v["description"] = json!(d);
                }
                v
            })
            .collect::<Vec<_>>());
    }
    if builder.line_pattern_scale != 1.0 {
        value["linePatternScale"] = json!(builder.line_pattern_scale);
    }
    if !layer_values.is_empty() {
        value["layers"] = json!(layer_values);
    }
    if !block_definitions.is_empty() {
        value["blockDefinitions"] = json!(block_definitions);
    }
    if !builder.ucs_definitions.is_empty() {
        value["ucsDefinitions"] = Value::Array(
            builder
                .ucs_definitions
                .iter()
                .enumerate()
                .map(|(id, ucs)| encode_ucs_definition(ucs, id as u32))
                .collect(),
        );
    }
    if builder.plot_style_mode == PlotStyleMode::Named {
        value["plotStyleMode"] = json!("named");
    }
    if let Some(display) = builder.point_display {
        value["pointDisplay"] = encode_point_display(display);
    }
    let mut workspace = Map::new();
    if let Some(id) = builder.current_layer {
        workspace.insert("currentLayerId".into(), json!(id));
    }
    if let Some(id) = builder.active_layout {
        workspace.insert("activeLayoutId".into(), json!(id));
    }
    if !workspace.is_empty() {
        value["drawingWorkspaceState"] = Value::Object(workspace);
    }
    super::encode_view_state::encode_saved_state(&mut value, &builder.saved_state);
    if !builder.objects.is_empty() || !builder.viewports.is_empty() {
        let mut groups = std::collections::BTreeMap::<&str, Vec<&DrawingEntityRecord>>::new();
        for object in &builder.objects {
            groups.entry(object.kind).or_default().push(object);
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
                _ => unreachable!("only typed object kinds enter the builder"),
            };
            value["streams"][payload] = Value::Object(columns);
        }
        super::encode_view_state::encode_viewports(&mut value, &builder.viewports);
    }
    for scope in value["scopes"].as_array_mut().expect("scope records") {
        let id = scope["id"].as_u64().expect("scope ID") as u32;
        scope["entities"] = json!(builder.scope_entities.get(&id).cloned().unwrap_or_default());
    }
    value
}

pub(crate) fn encode_document_bytes(
    builder: &DrawingBuilder,
    bounds: &[Option<([f64; 3], [f64; 3])>],
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec_pretty(&encode_document(builder, bounds))
}
