//! Encoding grammar that must be checked before decoding tagged values.
use crate::ocdraw::read::{diagnostic, OcdrawDiagnostic};
use serde_json::Value;

pub(super) fn validate_choices(value: &Value, diagnostics: &mut Vec<OcdrawDiagnostic>) {
    let mut selections = Vec::new();
    if let Some(s) = value["drawingViewState"].get("currentModelUcs") {
        selections.push((s, "/drawingViewState/currentModelUcs".into()));
    }
    for (table, fields) in [
        ("modelWindows", &["storedUcs"][..]),
        ("paperCanvases", &["storedUcs", "currentUcs"][..]),
        ("viewportWorkspaces", &["storedUcs"][..]),
    ] {
        for (i, row) in value[table].as_array().into_iter().flatten().enumerate() {
            for field in fields {
                if let Some(s) = row.get(*field) {
                    selections.push((s, format!("/{table}/{i}/{field}")));
                }
            }
        }
    }
    for (s, location) in selections {
        if s["kind"] == "Named" && s.get("ucsId").is_none() {
            diagnostics.push(diagnostic(
                "UCS_REF",
                format!("{location}/ucsId"),
                "named UCS reference is missing",
            ));
        }
        let id = s.get("ucsId").is_some();
        let frame = s.get("frame");
        let valid = match s["kind"].as_str() {
            Some("World") => !id && frame.is_none(),
            Some("Named") => id && frame.is_none(),
            Some("Unnamed") => {
                !id && frame.is_some_and(|v| super::super::frame_from_json(v).is_some())
            }
            _ => false,
        };
        if !valid {
            diagnostics.push(diagnostic(
                "UCS_CHOICE",
                location,
                "UCS choice must contain exactly its selected value and a valid frame",
            ));
        }
    }
    for (name, stream) in super::super::stream_contract::object_streams(value) {
        let count = stream["count"].as_u64().unwrap_or(0) as usize;
        for row in 0..count {
            for (mode, field) in [
                ("colorMode", "color"),
                ("opacityMode", "opacity"),
                ("linePatternMode", "linePatternId"),
                ("lineWeightMode", "lineWeight"),
            ] {
                let explicit = stream[mode][row].as_str() == Some("Explicit");
                let has = stream[field].get(row).is_some_and(|v| !v.is_null());
                if explicit != has {
                    diagnostics.push(diagnostic(
                        "APPEARANCE_PAIR",
                        format!("/streams/{name}/{mode}/{row}"),
                        "appearance mode/value pair is inconsistent",
                    ));
                }
            }
        }
    }
}
