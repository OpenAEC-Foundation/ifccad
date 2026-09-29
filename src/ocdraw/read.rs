use super::validate::validate_semantics;
use serde_json::Value;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingLoadStatus {
    Valid,
    Invalid,
    UnsupportedVersion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawingDiagnostic {
    pub code: &'static str,
    pub location: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct ValidatedDrawing {
    value: Value,
}

impl ValidatedDrawing {
    pub fn drawing_id(&self) -> &str {
        self.value["header"]["drawingId"]
            .as_str()
            .expect("validated")
    }

    pub fn layers(&self) -> &[Value] {
        self.value["layers"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn layouts(&self) -> &[Value] {
        self.value["layouts"].as_array().expect("validated")
    }

    pub fn plot_style_mode(&self) -> &str {
        self.value["plotStyleMode"]
            .as_str()
            .unwrap_or("colorDependent")
    }

    pub fn as_value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub struct DrawingLoadOutcome {
    status: DrawingLoadStatus,
    diagnostics: Vec<DrawingDiagnostic>,
    drawing: Option<ValidatedDrawing>,
}

impl DrawingLoadOutcome {
    pub fn status(&self) -> DrawingLoadStatus {
        self.status
    }

    pub fn diagnostics(&self) -> &[DrawingDiagnostic] {
        &self.diagnostics
    }

    pub fn validated_drawing(&self) -> Option<&ValidatedDrawing> {
        self.drawing.as_ref()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DrawingOpenError {
    #[error("could not read drawing: {0}")]
    Io(#[from] std::io::Error),
}

pub fn load_drawing_file(path: impl AsRef<Path>) -> Result<DrawingLoadOutcome, DrawingOpenError> {
    Ok(load_drawing_bytes(&std::fs::read(path)?))
}

pub(super) fn diagnostic(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> DrawingDiagnostic {
    DrawingDiagnostic {
        code,
        location: location.into(),
        message: message.into(),
    }
}

fn invalid(diagnostics: Vec<DrawingDiagnostic>) -> DrawingLoadOutcome {
    DrawingLoadOutcome {
        status: DrawingLoadStatus::Invalid,
        diagnostics,
        drawing: None,
    }
}

pub fn load_drawing_bytes(bytes: &[u8]) -> DrawingLoadOutcome {
    let value: Value = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(error) => return invalid(vec![diagnostic("INVALID_JSON", "", error.to_string())]),
    };
    if value["header"]["format"] != "open_cad_drawing" {
        return invalid(vec![diagnostic(
            "FORMAT",
            "/header/format",
            "expected open_cad_drawing",
        )]);
    }
    let Some(version) = value["header"]["version"]
        .as_str()
        .filter(|version| !version.is_empty())
    else {
        return invalid(vec![diagnostic(
            "VERSION",
            "/header/version",
            "drawing version is missing or malformed",
        )]);
    };
    if version != "0.1.0" {
        return DrawingLoadOutcome {
            status: DrawingLoadStatus::UnsupportedVersion,
            diagnostics: vec![diagnostic(
                "UNSUPPORTED_VERSION",
                "/header/version",
                "unsupported drawing version",
            )],
            drawing: None,
        };
    }
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    let schema = SCHEMA.get_or_init(|| {
        serde_json::from_str(include_str!("../../schemas/ocdraw/schema-0.1.0.json"))
            .expect("bundled schema")
    });
    let validator = jsonschema::draft202012::new(schema).expect("bundled schema is valid");
    let mut diagnostics = validator
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
        return invalid(diagnostics);
    }
    validate_semantics(&value, &mut diagnostics);
    if !diagnostics.is_empty() {
        return invalid(diagnostics);
    }
    DrawingLoadOutcome {
        status: DrawingLoadStatus::Valid,
        diagnostics,
        drawing: Some(ValidatedDrawing { value }),
    }
}
