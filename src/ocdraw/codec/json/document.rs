//! JSON document recognition and a deliberately explicit raw encoding view.

use crate::ocdraw::read::{diagnostic, OcdrawDiagnostic, OcdrawReadStatus};
use crate::ocdraw::PlotStyleMode;
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub(crate) struct JsonEncodedDrawing(Value);

impl JsonEncodedDrawing {
    pub(crate) fn value(&self) -> &Value {
        &self.0
    }

    pub(crate) fn header(&self) -> (String, String, PlotStyleMode) {
        let value = self.value();
        let drawing_id = value["header"]["drawingId"]
            .as_str()
            .expect("validated drawing ID")
            .to_owned();
        let unit = value["header"]["unit"]
            .as_str()
            .expect("validated drawing unit")
            .to_owned();
        let plot_style_mode = match value["plotStyleMode"].as_str() {
            Some("named") => PlotStyleMode::Named,
            _ => PlotStyleMode::ColorDependent,
        };
        (drawing_id, unit, plot_style_mode)
    }
}

pub(crate) fn parse_document(
    bytes: &[u8],
) -> Result<JsonEncodedDrawing, (OcdrawReadStatus, Vec<OcdrawDiagnostic>)> {
    let value: Value = serde_json::from_slice(bytes).map_err(|error| {
        (
            OcdrawReadStatus::Invalid,
            vec![diagnostic("INVALID_JSON", "", error.to_string())],
        )
    })?;
    if value["header"]["format"] != "open_cad_drawing" {
        return Err((
            OcdrawReadStatus::Invalid,
            vec![diagnostic(
                "FORMAT",
                "/header/format",
                "expected open_cad_drawing",
            )],
        ));
    }
    let Some(version) = value["header"]["version"]
        .as_str()
        .filter(|version| !version.is_empty())
    else {
        return Err((
            OcdrawReadStatus::Invalid,
            vec![diagnostic(
                "VERSION",
                "/header/version",
                "drawing version is missing or malformed",
            )],
        ));
    };
    if version != "0.1.0" {
        return Err((
            OcdrawReadStatus::UnsupportedVersion,
            vec![diagnostic(
                "UNSUPPORTED_VERSION",
                "/header/version",
                "unsupported drawing version",
            )],
        ));
    }
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    if let Some(preservation) = value.get("preservation") {
        if let Some(version) = preservation["version"]
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .filter(|v| *v > 0)
        {
            if version != 1 {
                return Err((
                    OcdrawReadStatus::UnsupportedVersion,
                    vec![diagnostic(
                        "PRESERVATION_VERSION",
                        "/preservation/version",
                        "unsupported preservation envelope version",
                    )],
                ));
            }
        }
    }
    let schema = SCHEMA.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../schemas/ocdraw/schema-0.1.0.json"))
            .expect("bundled schema")
    });
    let validator = jsonschema::draft202012::new(schema).expect("bundled schema is valid");
    let diagnostics = validator
        .iter_errors(&value)
        .map(|error| {
            diagnostic(
                "SCHEMA",
                error.instance_path().to_string(),
                error.to_string(),
            )
        })
        .collect::<Vec<_>>();
    if !diagnostics.is_empty() {
        return Err((OcdrawReadStatus::Invalid, diagnostics));
    }
    Ok(JsonEncodedDrawing(value))
}
