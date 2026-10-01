mod geometry;
mod streams;

use crate::ocdraw::read::DrawingDiagnostic;
use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) fn validate_physical(value: &Value, diagnostics: &mut Vec<DrawingDiagnostic>) {
    let layer_ids = value["layers"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|layer| layer["id"].as_u64())
        .collect::<BTreeSet<_>>();
    streams::validate_streams(value, &layer_ids, diagnostics);
}
