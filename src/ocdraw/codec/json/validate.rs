mod choices;
mod geometry;
mod streams;

use crate::ocdraw::read::{diagnostic, OcdrawDiagnostic};
use serde_json::Value;

pub(crate) fn validate_physical(value: &Value, diagnostics: &mut Vec<OcdrawDiagnostic>) {
    // JSON Schema's integer type also accepts integral floating-point backings.
    // Allocation history must use exact unsigned integer decoding, never f64.
    for field in [
        "nextEntityId",
        "nextLayerId",
        "nextLayoutId",
        "nextLinePatternId",
    ] {
        let valid = value["header"][field]
            .as_u64()
            .is_some_and(|id| field == "nextEntityId" || u32::try_from(id).is_ok());
        if !valid {
            diagnostics.push(diagnostic(
                "ID_WATERMARK",
                format!("/header/{field}"),
                "allocation watermark must have an exact unsigned integer backing in its ID range",
            ));
        }
    }
    streams::validate_stream_columns(value, diagnostics);
    if !diagnostics.is_empty() {
        return;
    }
    choices::validate_choices(value, diagnostics);
    streams::validate_streams(value, diagnostics);
}
