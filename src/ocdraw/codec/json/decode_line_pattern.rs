use crate::ocdraw::logical::{DrawingLinePattern, LinePatternId};
use serde_json::Value;

pub(crate) fn decode_line_patterns(value: &Value) -> Vec<DrawingLinePattern> {
    value["linePatterns"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| DrawingLinePattern {
            id: LinePatternId(row["id"].as_u64().expect("schema pattern ID") as u32),
            name: row["name"].as_str().expect("schema pattern name").into(),
            description: row["description"].as_str().map(str::to_owned),
            pattern: row["pattern"]
                .as_array()
                .expect("schema pattern")
                .iter()
                .map(|v| v.as_f64().expect("schema length"))
                .collect(),
        })
        .collect()
}
