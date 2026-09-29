mod geometry;
mod streams;

use super::logical::{
    AppearanceMode, AppearancePair, BlockDefinition, DrawingModel, Entity, Layout, NamedId,
    PlotRectangles, Scope, ScopeKind, ScopeOrder, UcsChoiceCheck,
};
use super::read::{diagnostic, DrawingDiagnostic};
use serde_json::Value;
use std::collections::BTreeSet;

fn appearance_pair(stream: &Value, property: &str, row: usize) -> AppearancePair {
    let mode = match stream[format!("{property}Mode")][row].as_str() {
        Some("ByBlock") => AppearanceMode::ByBlock,
        Some("Explicit") => AppearanceMode::Explicit,
        _ => AppearanceMode::ByLayer,
    };
    let has_value = stream[property]
        .get(row)
        .is_some_and(|value| !value.is_null());
    AppearancePair { mode, has_value }
}

fn rect_from_json(row: &Value) -> crate::drawing::LayoutRect {
    crate::drawing::LayoutRect {
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
            .is_some_and(|frame| super::workspace::frame_from_json(frame).is_some()),
    });
    if row["kind"] == "Named" {
        refs.push((row["ucsId"].as_u64().map(|id| id as u32), location));
    }
}

pub(super) fn model_from_json(value: &Value) -> DrawingModel {
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
            id: row["id"].as_u64().expect("schema ID") as u32,
            has_bounds: Some(!row["bounds"].is_null()),
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
    if let Some(directory) = value["streamDirectory"]["streams"].as_array() {
        for entry in directory.iter().filter(|entry| entry["role"] == "object") {
            let Some(name) = entry["name"].as_str() else {
                continue;
            };
            let payload = format!("{name}Stream");
            let stream = &value["streams"][&payload];
            let Some(ids) = stream["entityId"].as_array() else {
                continue;
            };
            for (row, id) in ids.iter().enumerate() {
                let (Some(id), Some(scope_id), Some(layer_id)) = (
                    id.as_u64(),
                    stream["scopeId"][row].as_u64(),
                    stream["layerId"][row].as_u64(),
                ) else {
                    continue;
                };
                entities.push(Entity {
                    id,
                    scope_id: scope_id as u32,
                    layer_id: layer_id as u32,
                    definition_scope_id: stream["definitionScopeId"][row]
                        .as_u64()
                        .map(|id| id as u32),
                    appearance: ["color", "opacity", "linePattern", "lineWeight"]
                        .map(|property| appearance_pair(stream, property, row)),
                    location: format!("/streams/{payload}/entityId/{row}"),
                });
            }
        }
    }
    let mut orders = Vec::new();
    let order_stream = &value["streams"]["entityOrderStream"];
    let entries = value["streams"]["entityOrderEntryStream"]["entityId"].as_array();
    if let Some(scope_ids) = order_stream["scopeId"].as_array() {
        for (row, scope_id) in scope_ids.iter().enumerate() {
            let (Some(scope_id), Some(offset), Some(count), Some(entries)) = (
                scope_id.as_u64(),
                order_stream["entryOffset"][row]
                    .as_u64()
                    .and_then(|value| usize::try_from(value).ok()),
                order_stream["entryCount"][row]
                    .as_u64()
                    .and_then(|value| usize::try_from(value).ok()),
                entries,
            ) else {
                continue;
            };
            let ids = offset
                .checked_add(count)
                .and_then(|end| entries.get(offset..end))
                .map(|range| range.iter().filter_map(Value::as_u64).collect())
                .unwrap_or_default();
            orders.push(ScopeOrder {
                scope_id: scope_id as u32,
                entities: ids,
                location: format!("/streams/entityOrderStream/scopeId/{row}"),
            });
        }
    }
    DrawingModel {
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
        orders,
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
            .map(|row| NamedId {
                id: row["ucsId"].as_u64().expect("schema UCS ID") as u32,
                name: row["name"].as_str().expect("schema UCS name").to_owned(),
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

pub(super) fn validate_semantics(value: &Value, diagnostics: &mut Vec<DrawingDiagnostic>) {
    let model = model_from_json(value);
    diagnostics.extend(
        model
            .validate()
            .into_iter()
            .map(|error| diagnostic(error.code, error.location, error.message)),
    );
    for (index, ucs) in value["ucsDefinitions"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        if super::workspace::frame_from_json(&ucs["frame"]).is_none() {
            diagnostics.push(diagnostic(
                "UCS_FRAME",
                format!("/ucsDefinitions/{index}/frame"),
                "UCS frame is invalid",
            ));
        }
    }
    let layer_ids = model
        .layers
        .iter()
        .map(|layer| u64::from(layer.id))
        .collect::<BTreeSet<_>>();
    streams::validate_streams(value, &layer_ids, diagnostics);
}
