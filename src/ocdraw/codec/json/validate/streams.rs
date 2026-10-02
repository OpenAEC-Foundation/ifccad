use super::geometry;
use crate::ocdraw::read::{diagnostic, DrawingDiagnostic};
use serde_json::Value;
use std::collections::BTreeSet;

/// Checks row counts and column shapes without traversing declared rows.
pub(super) fn validate_stream_columns(value: &Value, diagnostics: &mut Vec<DrawingDiagnostic>) {
    for map in super::super::stream_contract::mapping()["streams"]
        .as_array()
        .expect("bundled mapping")
    {
        let payload = map["payload"]
            .as_str()
            .expect("bundled mapping")
            .strip_prefix("streams.")
            .expect("stream path");
        let Some(stream) = value["streams"].get(payload) else {
            continue;
        };
        for field in map["fields"].as_array().expect("bundled mapping") {
            if field["omission"] == "forbidden" {
                let column = field["payload"].as_str().expect("bundled mapping");
                if stream.get(column).is_none() {
                    diagnostics.push(diagnostic(
                        "STREAM_COLUMN",
                        format!("/streams/{payload}/{column}"),
                        "required stream column is missing",
                    ));
                }
            }
        }
    }
    if let Some(streams) = value["streams"].as_object() {
        for (name, stream) in streams {
            let Some(count) = stream["count"]
                .as_u64()
                .and_then(|count| usize::try_from(count).ok())
            else {
                diagnostics.push(diagnostic(
                    "STREAM_COUNT",
                    format!("/streams/{name}/count"),
                    "stream count must be an exact unsigned integer in the addressable range",
                ));
                continue;
            };
            let pooled_columns = pooled_columns(name);
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
        }
    }
}

pub(super) fn validate_streams(value: &Value, diagnostics: &mut Vec<DrawingDiagnostic>) {
    if let Some(streams) = value["streams"].as_object() {
        for (name, stream) in streams {
            let pooled_columns = pooled_columns(name);
            if !pooled_columns.is_empty() {
                let count = stream["count"].as_u64().expect("validated stream count") as usize;
                validate_vertex_pool(name, stream, pooled_columns, count, diagnostics);
            }
        }
    }
    geometry::validate_placed_geometry(value, diagnostics);
    let override_stream = &value["streams"]["viewportLayerOverrideStream"];
    let viewport_stream = &value["streams"]["viewportStream"];
    if let Some(overrides) = override_stream["layerId"].as_array() {
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
                for position in start..end {
                    if !covered.insert(position) {
                        diagnostics.push(diagnostic(
                            "VIEWPORT_LAYER",
                            format!("/streams/viewportStream/layerOverrideOffset/{row}"),
                            "viewport override ranges overlap",
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

fn pooled_columns(name: &str) -> &'static [&'static str] {
    match name {
        "planarPolylineStream" => &["x", "y", "bulge"],
        "spatialPolylineStream" => &["x", "y", "z"],
        _ => &[],
    }
}

fn validate_vertex_pool(
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
