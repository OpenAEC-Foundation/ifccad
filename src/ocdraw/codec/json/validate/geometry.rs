//! Checks needed to construct typed placements before semantic validation.
use crate::ocdraw::read::{diagnostic, DrawingDiagnostic};
use serde_json::Value;

pub(super) fn validate_placed_geometry(value: &Value, diagnostics: &mut Vec<DrawingDiagnostic>) {
    for payload in [
        "pointStream",
        "circleStream",
        "arcStream",
        "ellipseStream",
        "ellipseArcStream",
        "planarPolylineStream",
    ] {
        let stream = &value["streams"][payload];
        let Some(placements) = stream["placement"].as_array() else {
            continue;
        };
        for (row, frame) in placements.iter().enumerate() {
            if !frame.is_null() && super::super::frame_from_json(frame).is_none() {
                diagnostics.push(diagnostic(
                    "ENTITY_GEOMETRY",
                    format!("/streams/{payload}/placement/{row}"),
                    "invalid entity placement",
                ));
            }
        }
    }
}
