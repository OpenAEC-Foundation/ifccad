use crate::ocdraw::logical::{
    AppearanceMode, AppearancePair, BlockDefinition, DrawingModel, Entity, Layout, NamedId,
    NamedUcs, PlotRectangles, Scope, ScopeKind, UcsChoiceCheck,
};
use crate::ocdraw::{Bounds3d, Point3};
use serde_json::Value;

fn appearance_pair(stream: &Value, property: &str, row: usize) -> AppearancePair {
    let mode = match stream[format!("{}Mode", property.strip_suffix("Id").unwrap_or(property))][row]
        .as_str()
    {
        Some("ByBlock") => AppearanceMode::ByBlock,
        Some("Explicit") => AppearanceMode::Explicit,
        _ => AppearanceMode::ByLayer,
    };
    let has_value = stream[property]
        .get(row)
        .is_some_and(|value| !value.is_null());
    AppearancePair { mode, has_value }
}

fn rect_from_json(row: &Value) -> crate::ocdraw::LayoutRect {
    crate::ocdraw::LayoutRect {
        min_x: row["minX"].as_f64().expect("schema rectangle"),
        min_y: row["minY"].as_f64().expect("schema rectangle"),
        max_x: row["maxX"].as_f64().expect("schema rectangle"),
        max_y: row["maxY"].as_f64().expect("schema rectangle"),
    }
}

fn named_ucs_ref(
    row: &Value,
    location: String,
    refs: &mut Vec<(Option<u32>, String)>,
    checks: &mut Vec<UcsChoiceCheck>,
) {
    if !row.is_object() {
        return;
    }
    checks.push(UcsChoiceCheck {
        location: location
            .strip_suffix("/ucsId")
            .unwrap_or(&location)
            .to_owned(),
        kind: row["kind"].as_str().unwrap_or("").to_owned(),
        has_id: row.get("ucsId").is_some(),
        has_frame: row.get("frame").is_some(),
        frame_valid: row
            .get("frame")
            .is_some_and(|frame| super::frame_from_json(frame).is_some()),
    });
    if row["kind"] == "Named" {
        refs.push((row["ucsId"].as_u64().map(|id| id as u32), location));
    }
}

pub(crate) fn decode_model(value: &Value) -> DrawingModel {
    let mut named_ucs_refs = Vec::new();
    let mut ucs_choices = Vec::new();
    named_ucs_ref(
        &value["drawingViewState"]["currentModelUcs"],
        "/drawingViewState/currentModelUcs/ucsId".into(),
        &mut named_ucs_refs,
        &mut ucs_choices,
    );
    for (index, window) in value["modelWindows"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        named_ucs_ref(
            &window["storedUcs"],
            format!("/modelWindows/{index}/storedUcs/ucsId"),
            &mut named_ucs_refs,
            &mut ucs_choices,
        );
    }
    for (index, canvas) in value["paperCanvases"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        named_ucs_ref(
            &canvas["storedUcs"],
            format!("/paperCanvases/{index}/storedUcs/ucsId"),
            &mut named_ucs_refs,
            &mut ucs_choices,
        );
        named_ucs_ref(
            &canvas["currentUcs"],
            format!("/paperCanvases/{index}/currentUcs/ucsId"),
            &mut named_ucs_refs,
            &mut ucs_choices,
        );
    }
    for (index, workspace) in value["viewportWorkspaces"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        named_ucs_ref(
            &workspace["storedUcs"],
            format!("/viewportWorkspaces/{index}/storedUcs/ucsId"),
            &mut named_ucs_refs,
            &mut ucs_choices,
        );
    }
    let layers = value["layers"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| NamedId {
            id: row["id"].as_u64().expect("schema ID") as u32,
            name: row["name"].as_str().expect("schema name").to_owned(),
        })
        .collect();
    let layouts = value["layouts"]
        .as_array()
        .expect("schema layouts")
        .iter()
        .map(|row| Layout {
            id: row["id"].as_u64().expect("schema ID") as u32,
            name: row["name"].as_str().expect("schema name").to_owned(),
            scope_id: row["scopeId"].as_u64().expect("schema scope") as u32,
            kind: if row["kind"] == "model" {
                ScopeKind::Model
            } else {
                ScopeKind::Paper
            },
            tab_index: row["tabIndex"].as_u64().expect("schema tab") as u32,
            limits: row["limits"]
                .as_object()
                .map(|_| rect_from_json(&row["limits"])),
            plot_rectangles: row["plotSettings"].as_object().map(|_| PlotRectangles {
                printable_area: rect_from_json(&row["plotSettings"]["media"]["printableArea"]),
                window: row["plotSettings"]["area"]["window"]
                    .as_object()
                    .map(|_| rect_from_json(&row["plotSettings"]["area"]["window"])),
            }),
        })
        .collect();
    let scopes = value["scopes"]
        .as_array()
        .expect("schema scopes")
        .iter()
        .map(|row| Scope {
            entities: row["entities"]
                .as_array()
                .expect("schema entity list")
                .iter()
                .map(|id| id.as_u64().expect("schema entity ID"))
                .collect(),
            id: row["id"].as_u64().expect("schema ID") as u32,
            has_bounds: Some(!row["bounds"].is_null()),
            bounds: row["bounds"].as_object().map(|bounds| Bounds3d {
                min: Point3::new(
                    bounds["minX"].as_f64().expect("schema bounds"),
                    bounds["minY"].as_f64().expect("schema bounds"),
                    bounds["minZ"].as_f64().expect("schema bounds"),
                ),
                max: Point3::new(
                    bounds["maxX"].as_f64().expect("schema bounds"),
                    bounds["maxY"].as_f64().expect("schema bounds"),
                    bounds["maxZ"].as_f64().expect("schema bounds"),
                ),
            }),
            kind: match row["kind"].as_u64().expect("schema kind") {
                0 => ScopeKind::Model,
                1 => ScopeKind::Paper,
                2 => ScopeKind::Block,
                _ => unreachable!("schema kind"),
            },
        })
        .collect();
    let blocks = value["blockDefinitions"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| BlockDefinition {
            scope_id: row["scopeId"].as_u64().expect("schema scope") as u32,
            name: row["name"].as_str().expect("schema name").to_owned(),
        })
        .collect();
    let mut entities = Vec::new();
    for (payload, stream) in super::stream_contract::object_streams(value) {
        let Some(ids) = stream["entityId"].as_array() else {
            continue;
        };
        for (row, id) in ids.iter().enumerate() {
            let (Some(id), Some(layer_id)) = (id.as_u64(), stream["layerId"][row].as_u64()) else {
                continue;
            };
            entities.push(Entity {
                id,
                layer_id: layer_id as u32,
                definition_scope_id: stream["definitionScopeId"][row]
                    .as_u64()
                    .map(|id| id as u32),
                appearance: ["color", "opacity", "linePatternId", "lineWeight"]
                    .map(|property| appearance_pair(stream, property, row)),
                location: format!("/streams/{payload}/entityId/{row}"),
            });
        }
    }
    DrawingModel {
        line_patterns: super::decode_line_patterns(value),
        next_line_pattern_id: value["header"]["nextLinePatternId"]
            .as_u64()
            .expect("schema watermark") as u32,
        line_pattern_refs: {
            let mut refs = Vec::new();
            for (i, row) in value["layers"].as_array().into_iter().flatten().enumerate() {
                refs.push((
                    crate::ocdraw::LinePatternId(
                        row["linePatternId"].as_u64().expect("schema pattern ID") as u32,
                    ),
                    format!("/layers/{i}/linePatternId"),
                ));
            }
            for (name, stream) in value["streams"].as_object().into_iter().flatten() {
                for (i, id) in stream["linePatternId"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    if let Some(id) = id.as_u64() {
                        refs.push((
                            crate::ocdraw::LinePatternId(id as u32),
                            format!("/streams/{name}/linePatternId/{i}"),
                        ));
                    }
                }
            }
            refs
        },
        line_pattern_scales: {
            let mut scales = vec![(
                value["linePatternScale"].as_f64().unwrap_or(1.0),
                "/linePatternScale".into(),
            )];
            for (name, stream) in value["streams"].as_object().into_iter().flatten() {
                for (i, v) in stream["linePatternScale"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    scales.push((
                        v.as_f64().unwrap_or(1.0),
                        format!("/streams/{name}/linePatternScale/{i}"),
                    ));
                }
            }
            scales
        },
        next_entity_id: value["header"]["nextEntityId"]
            .as_u64()
            .expect("schema watermark"),
        next_layer_id: value["header"]["nextLayerId"]
            .as_u64()
            .expect("schema watermark") as u32,
        next_layout_id: value["header"]["nextLayoutId"]
            .as_u64()
            .expect("schema watermark") as u32,
        layers,
        layouts,
        scopes,
        blocks,
        entities,
        current_layer_id: value["drawingWorkspaceState"]["currentLayerId"]
            .as_u64()
            .map(|id| id as u32),
        active_layout_id: value["drawingWorkspaceState"]["activeLayoutId"]
            .as_u64()
            .map(|id| id as u32),
        ucs_definitions: value["ucsDefinitions"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|row| NamedUcs {
                id: row["ucsId"].as_u64().expect("schema UCS ID") as u32,
                name: row["name"].as_str().expect("schema UCS name").to_owned(),
                frame: super::frame_from_json(&row["frame"]),
            })
            .collect(),
        model_window_ids: value["modelWindows"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|row| {
                row["modelWindowId"]
                    .as_u64()
                    .expect("schema model window ID") as u32
            })
            .collect(),
        active_model_window_id: value["drawingViewState"]["activeModelWindowId"]
            .as_u64()
            .map(|id| id as u32),
        named_ucs_refs,
        ucs_choices,
    }
}
