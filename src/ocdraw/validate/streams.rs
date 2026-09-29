use super::geometry;
use crate::drawing::read::{diagnostic, DrawingDiagnostic};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn validate_streams(
    value: &Value,
    layer_ids: &BTreeSet<u64>,
    diagnostics: &mut Vec<DrawingDiagnostic>,
) {
    let registry: Value =
        serde_json::from_str(include_str!("../../../schemas/ocdraw/registry-0.1.0.json"))
            .expect("bundled registry");
    let mapping: Value = serde_json::from_str(include_str!(
        "../../../schemas/ocdraw/json-mapping-0.1.0.json"
    ))
    .expect("bundled mapping");
    let entries = value["streamDirectory"]["streams"]
        .as_array()
        .expect("schema directory");
    let mut declared = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let name = entry["name"].as_str().expect("schema name");
        if !declared.insert(name) {
            diagnostics.push(diagnostic(
                "STREAM_DIRECTORY",
                format!("/streamDirectory/streams/{index}/name"),
                "duplicate stream entry",
            ));
        }
        let Some(reg) = registry["streams"]
            .as_array()
            .expect("bundled registry")
            .iter()
            .find(|r| r["name"] == name)
        else {
            continue;
        };
        let Some(map) = mapping["streams"]
            .as_array()
            .expect("bundled mapping")
            .iter()
            .find(|r| r["name"] == name)
        else {
            continue;
        };
        if entry["schema"] != reg["schemaId"] || entry["role"] != reg["role"] {
            diagnostics.push(diagnostic(
                "STREAM_DIRECTORY",
                format!("/streamDirectory/streams/{index}"),
                "stream schema or role differs from registry",
            ));
        }
        let payload = map["payload"]
            .as_str()
            .expect("bundled mapping")
            .strip_prefix("streams.")
            .expect("stream path");
        let Some(stream) = value["streams"].get(payload) else {
            diagnostics.push(diagnostic(
                "STREAM_DIRECTORY",
                format!("/streams/{payload}"),
                "declared stream payload is missing",
            ));
            continue;
        };
        if entry["count"] != stream["count"] {
            diagnostics.push(diagnostic(
                "STREAM_DIRECTORY",
                format!("/streamDirectory/streams/{index}/count"),
                "directory count differs from payload",
            ));
        }
        let actual = stream
            .as_object()
            .expect("schema payload")
            .keys()
            .filter(|key| *key != "count")
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let listed = entry["columns"]
            .as_array()
            .expect("schema columns")
            .iter()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        if actual != listed {
            diagnostics.push(diagnostic(
                "STREAM_DIRECTORY",
                format!("/streamDirectory/streams/{index}/columns"),
                "directory columns differ from payload",
            ));
        }
        for field in map["fields"].as_array().expect("bundled mapping") {
            if field["omission"] == "forbidden" {
                let column = field["payload"].as_str().expect("bundled mapping");
                if !actual.contains(column) {
                    diagnostics.push(diagnostic(
                        "STREAM_COLUMN",
                        format!("/streams/{payload}/{column}"),
                        "required stream column is missing",
                    ));
                }
            }
        }
    }
    for map in mapping["streams"].as_array().expect("bundled mapping") {
        let name = map["name"].as_str().expect("bundled mapping");
        let payload = map["payload"]
            .as_str()
            .expect("bundled mapping")
            .strip_prefix("streams.")
            .expect("stream path");
        if value["streams"].get(payload).is_some() && !declared.contains(name) {
            diagnostics.push(diagnostic(
                "STREAM_DIRECTORY",
                format!("/streams/{payload}"),
                "stream payload has no directory entry",
            ));
        }
    }
    if let Some(streams) = value["streams"].as_object() {
        for (name, stream) in streams {
            let count = stream["count"].as_u64().expect("schema count") as usize;
            let pooled_columns: &[&str] = match name.as_str() {
                "planarPolylineStream" => &["x", "y", "bulge"],
                "spatialPolylineStream" => &["x", "y", "z"],
                _ => &[],
            };
            for (field, column) in stream.as_object().expect("schema stream") {
                if field != "count"
                    && !pooled_columns.contains(&field.as_str())
                    && column.as_array().is_some_and(|items| items.len() != count)
                {
                    diagnostics.push(diagnostic(
                        "COLUMN_COUNT",
                        format!("/streams/{name}/{field}"),
                        "column length differs from stream count",
                    ));
                }
            }
            if !pooled_columns.is_empty() {
                validate_vertex_pool(value, name, stream, pooled_columns, count, diagnostics);
            }
        }
    }
    validate_order_ranges(value, diagnostics);
    for (index, scope) in value["scopes"]
        .as_array()
        .expect("schema scopes")
        .iter()
        .enumerate()
    {
        if let Some(bounds) = scope["bounds"].as_object() {
            for axis in ["X", "Y", "Z"] {
                let min = bounds[&format!("min{axis}")]
                    .as_f64()
                    .expect("schema bounds");
                let max = bounds[&format!("max{axis}")]
                    .as_f64()
                    .expect("schema bounds");
                if min > max {
                    diagnostics.push(diagnostic(
                        "SCOPE_BOUNDS",
                        format!("/scopes/{index}/bounds"),
                        "scope minimum exceeds maximum",
                    ));
                }
            }
        }
    }
    if let Some(lines) = value["streams"]["lineStream"]["entityId"].as_array() {
        for row in 0..lines.len() {
            let scope_id = value["streams"]["lineStream"]["scopeId"]
                .get(row)
                .and_then(Value::as_u64);
            let Some(scope) = value["scopes"]
                .as_array()
                .expect("schema scopes")
                .iter()
                .find(|scope| scope["id"].as_u64() == scope_id)
            else {
                continue;
            };
            let Some(bounds) = scope["bounds"].as_object() else {
                continue;
            };
            for axis in ["X", "Y", "Z"] {
                for endpoint in ["1", "2"] {
                    let column = format!("{}{endpoint}", axis.to_ascii_lowercase());
                    let coordinate = value["streams"]["lineStream"][&column]
                        .get(row)
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0);
                    let min = bounds[&format!("min{axis}")]
                        .as_f64()
                        .expect("schema bounds");
                    let max = bounds[&format!("max{axis}")]
                        .as_f64()
                        .expect("schema bounds");
                    if coordinate < min || coordinate > max {
                        diagnostics.push(diagnostic(
                            "SCOPE_BOUNDS",
                            format!("/streams/lineStream/{column}/{row}"),
                            "line lies outside declared scope bounds",
                        ));
                    }
                }
            }
        }
    }
    geometry::validate_placed_geometry(value, diagnostics);
    let override_stream = &value["streams"]["viewportLayerOverrideStream"];
    let viewport_stream = &value["streams"]["viewportStream"];
    if let Some(overrides) = override_stream["layerId"].as_array() {
        for (index, id) in overrides.iter().enumerate() {
            if !id.as_u64().is_some_and(|id| layer_ids.contains(&id)) {
                diagnostics.push(diagnostic(
                    "VIEWPORT_LAYER",
                    format!("/streams/viewportLayerOverrideStream/layerId/{index}"),
                    "viewport override Layer does not resolve",
                ));
            }
        }
        let offsets = viewport_stream["layerOverrideOffset"].as_array();
        let counts = viewport_stream["layerOverrideCount"].as_array();
        let viewport_count = viewport_stream["count"].as_u64().unwrap_or(0) as usize;
        let mut covered = BTreeSet::new();
        for row in 0..viewport_count {
            let range = offsets
                .and_then(|v| v.get(row))
                .and_then(Value::as_u64)
                .zip(counts.and_then(|v| v.get(row)).and_then(Value::as_u64))
                .and_then(|(start, count)| start.checked_add(count).map(|end| (start, end)));
            if let Some((start, end)) = range.filter(|(_, end)| *end <= overrides.len() as u64) {
                let mut local_layers = BTreeSet::new();
                for position in start..end {
                    if !covered.insert(position)
                        || !overrides[position as usize]
                            .as_u64()
                            .is_some_and(|id| local_layers.insert(id))
                    {
                        diagnostics.push(diagnostic(
                            "VIEWPORT_LAYER",
                            format!("/streams/viewportStream/layerOverrideOffset/{row}"),
                            "viewport override ranges overlap or repeat a Layer",
                        ));
                    }
                }
            } else {
                diagnostics.push(diagnostic(
                    "VIEWPORT_LAYER",
                    format!("/streams/viewportStream/layerOverrideOffset/{row}"),
                    "viewport override range is missing or out of bounds",
                ));
            }
        }
        if covered.len() != overrides.len() {
            diagnostics.push(diagnostic(
                "VIEWPORT_LAYER",
                "/streams/viewportLayerOverrideStream",
                "viewport override rows must belong to a viewport",
            ));
        }
    }
}

fn validate_order_ranges(value: &Value, diagnostics: &mut Vec<DrawingDiagnostic>) {
    let Some(entries) = value["streams"]["entityOrderEntryStream"]["entityId"].as_array() else {
        return;
    };
    let order = &value["streams"]["entityOrderStream"];
    let count = order["count"].as_u64().unwrap_or(0) as usize;
    let mut covered = BTreeSet::new();
    for row in 0..count {
        let range = order["entryOffset"][row]
            .as_u64()
            .zip(order["entryCount"][row].as_u64())
            .and_then(|(start, count)| start.checked_add(count).map(|end| (start, end)));
        if let Some((start, end)) = range.filter(|(_, end)| *end <= entries.len() as u64) {
            for position in start..end {
                if !covered.insert(position) {
                    diagnostics.push(diagnostic(
                        "ENTITY_ORDER",
                        format!("/streams/entityOrderStream/entryOffset/{row}"),
                        "order ranges overlap",
                    ));
                }
            }
        } else {
            diagnostics.push(diagnostic(
                "ENTITY_ORDER",
                format!("/streams/entityOrderStream/entryCount/{row}"),
                "order range exceeds entry stream",
            ));
        }
    }
    if covered.len() != entries.len() {
        diagnostics.push(diagnostic(
            "ENTITY_ORDER",
            "/streams/entityOrderStream",
            "order ranges must cover all entries",
        ));
    }
}

fn validate_vertex_pool(
    drawing: &Value,
    name: &str,
    stream: &Value,
    pools: &[&str],
    count: usize,
    diagnostics: &mut Vec<DrawingDiagnostic>,
) {
    let lengths = pools
        .iter()
        .map(|field| stream[*field].as_array().map(Vec::len))
        .collect::<Vec<_>>();
    let Some(length) = lengths.first().copied().flatten() else {
        diagnostics.push(diagnostic(
            "VERTEX_POOL",
            format!("/streams/{name}"),
            "vertex pool is missing",
        ));
        return;
    };
    if lengths.iter().any(|item| *item != Some(length)) {
        diagnostics.push(diagnostic(
            "VERTEX_POOL",
            format!("/streams/{name}"),
            "vertex pool columns have different lengths",
        ));
        return;
    }
    let mut next = 0usize;
    for row in 0..count {
        let offset = stream["vertexOffset"].get(row).and_then(Value::as_u64);
        let vertices = stream["vertexCount"].get(row).and_then(Value::as_u64);
        let range = offset.zip(vertices).and_then(|(offset, vertices)| {
            offset
                .checked_add(vertices)
                .map(|end| (offset, vertices, end))
        });
        if let Some((offset, vertices, end)) = range {
            if offset == next as u64 && vertices >= 2 && end <= length as u64 {
                next = end as usize;
                if name == "planarPolylineStream"
                    && stream["closed"].get(row).and_then(Value::as_bool) == Some(false)
                    && stream["bulge"][next - 1].as_f64() != Some(0.0)
                {
                    diagnostics.push(diagnostic(
                        "VERTEX_POOL",
                        format!("/streams/{name}/bulge/{}", next - 1),
                        "open polyline cannot have a trailing bulge",
                    ));
                }
                geometry::validate_polyline_geometry(
                    drawing,
                    name,
                    stream,
                    row,
                    offset as usize,
                    vertices as usize,
                    diagnostics,
                );
                continue;
            }
        }
        diagnostics.push(diagnostic(
            "VERTEX_POOL",
            format!("/streams/{name}/vertexOffset/{row}"),
            "vertex ranges must be consecutive, contain at least two vertices, and fit the pool",
        ));
    }
    if next != length {
        diagnostics.push(diagnostic(
            "VERTEX_POOL",
            format!("/streams/{name}"),
            "vertex ranges must cover the complete pool",
        ));
    }
}
