use crate::ocdraw::logical::{DrawingLinePattern, LinePatternId};
use serde_json::Value;

pub(crate) fn decode_line_patterns(
    value: &Value,
) -> Result<Vec<DrawingLinePattern>, crate::ocdraw::OcdrawDiagnostic> {
    value["linePatterns"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, row)| {
            Ok(DrawingLinePattern {
                id: LinePatternId(
                    row["id"]
                        .as_u64()
                        .and_then(|v| u32::try_from(v).ok())
                        .ok_or_else(|| crate::ocdraw::OcdrawDiagnostic {
                            code: "LINE_PATTERN_ENCODING",
                            location: format!("/linePatterns/{i}/id"),
                            message: "pattern ID requires an exact unsigned integer backing".into(),
                        })?,
                ),
                name: row["name"].as_str().expect("schema pattern name").into(),
                description: row["description"].as_str().map(str::to_owned),
                pattern: super::line_pattern_values::decode_pattern_elements(
                    &row["pattern"],
                    &format!("/linePatterns/{i}/pattern"),
                )?,
            })
        })
        .collect()
}
