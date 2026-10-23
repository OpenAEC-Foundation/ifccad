//! Stored upright-marker mode qualification; glyph rendering stays unassessed.
use ifccad::{line_pattern::LinePatternRotation, IfccadDocument, IfccadLinePatternMarker};
use ifccad_convert::opencadcodec::{CadDocument, DxfVersion};
use serde_json::{json, Value};

const LOSS: &str = "TARGET_CODEC_LINE_PATTERN_UPRIGHT_LOSS";

pub(crate) fn preflight(document: &IfccadDocument) -> Value {
    let mut locations = Vec::new();
    for pattern in &document.line_patterns {
        for (index, element) in pattern.pattern.iter().enumerate() {
            let transform = match &element.marker {
                Some(IfccadLinePatternMarker::Text { transform, .. })
                | Some(IfccadLinePatternMarker::Symbol { transform, .. }) => transform,
                None => continue,
            };
            if matches!(transform.rotation, LinePatternRotation::Upright(_)) {
                locations.push(format!(
                    "linePattern/{}/pattern/{index}/marker/rotation",
                    pattern.id.0
                ));
            }
        }
    }
    let mut entries = Vec::new();
    for format in ["dxf", "dwg"] {
        for version in crate::cad::VERSIONS {
            let restrictions = if format == "dxf" && version < DxfVersion::AC1027 {
                locations.iter().map(|location| json!({
                    "code": LOSS, "location": location, "feature": "linePatternUpright",
                    "format": format, "version": version.as_str(), "policyEffect": "loss",
                    "message": "The selected codec writes upright line-pattern markers as relative rotation in DXF before AC1027"
                })).collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            entries.push(
                json!({"format":format,"version":version.as_str(),"restrictions":restrictions}),
            );
        }
    }
    json!(entries)
}

pub(crate) fn losses(expected: &CadDocument, actual: &CadDocument) -> Vec<Value> {
    let mut losses = Vec::new();
    for pattern in expected.line_types.iter() {
        for (index, element) in pattern.elements.iter().enumerate() {
            if !element.complex.as_ref().is_some_and(|m| m.upright) {
                continue;
            }
            let returned = actual
                .line_types
                .get(&pattern.name)
                .and_then(|p| p.elements.get(index))
                .and_then(|e| e.complex.as_ref());
            let mode = returned.map(|m| match (m.upright, m.absolute_rotation) {
                (true, false) => "upright",
                (false, false) => "relative",
                (false, true) => "absolute",
                (true, true) => "conflicting",
            });
            if mode != Some("upright") {
                losses.push(json!({
                    "code":LOSS,
                    "location":format!("linePattern/{}/elements/{index}/rotation",pattern.name),
                    "phase":"cadExchange","action":"Modified",
                    "expected":"upright","actual":mode,
                    "message":"The written CAD file did not retain upright line-pattern rotation"
                }));
            }
        }
    }
    losses
}
