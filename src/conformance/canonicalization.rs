use super::ConformanceError;
use crate::canonicalization::canonicalize_typed_value;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorDocument {
    vectors: Vec<VectorRow>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VectorRow {
    vector_id: String,
    input: Value,
    normalized_json: Option<String>,
    canonical_utf8_base64: Option<String>,
    #[serde(rename = "fingerprint")]
    _fingerprint: Option<String>,
    expected_error: Option<String>,
}

/// Execute every row in an IFCCAD canonicalization vector document.
///
/// Returns the number of verified rows. The first mismatch is reported with
/// its vector identifier so a damaged corpus or implementation is actionable.
pub fn verify_canonicalization_vectors(path: impl AsRef<Path>) -> Result<usize, ConformanceError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(|source| ConformanceError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let document: VectorDocument =
        serde_json::from_str(&source).map_err(|source| ConformanceError::MalformedJson {
            path: path.to_path_buf(),
            source,
        })?;

    for row in &document.vectors {
        verify_row(row)?;
    }
    Ok(document.vectors.len())
}

fn verify_row(row: &VectorRow) -> Result<(), ConformanceError> {
    match (canonicalize_typed_value(&row.input), &row.expected_error) {
        (Err(error), Some(expected)) if error.code().as_str() == expected => Ok(()),
        (Err(error), Some(expected)) => mismatch(
            row,
            format!(
                "expected error {expected}, received {}",
                error.code().as_str()
            ),
        ),
        (Ok(_), Some(expected)) => {
            mismatch(row, format!("expected error {expected}, received bytes"))
        }
        (Err(error), None) => mismatch(
            row,
            format!("expected bytes, received {}", error.code().as_str()),
        ),
        (Ok(bytes), None) => verify_expected_bytes(row, &bytes),
    }
}

fn verify_expected_bytes(row: &VectorRow, bytes: &[u8]) -> Result<(), ConformanceError> {
    let normalized =
        row.normalized_json
            .as_deref()
            .ok_or_else(|| ConformanceError::VectorMismatch {
                vector_id: row.vector_id.clone(),
                message: "valid row is missing normalizedJson".to_owned(),
            })?;
    if bytes != normalized.as_bytes() {
        return mismatch(
            row,
            "canonical UTF-8 differs from normalizedJson".to_owned(),
        );
    }

    let encoded =
        row.canonical_utf8_base64
            .as_deref()
            .ok_or_else(|| ConformanceError::VectorMismatch {
                vector_id: row.vector_id.clone(),
                message: "valid row is missing canonicalUtf8Base64".to_owned(),
            })?;
    if STANDARD.encode(bytes) != encoded {
        return mismatch(
            row,
            "canonical UTF-8 differs from canonicalUtf8Base64".to_owned(),
        );
    }
    Ok(())
}

fn mismatch(row: &VectorRow, message: String) -> Result<(), ConformanceError> {
    Err(ConformanceError::VectorMismatch {
        vector_id: row.vector_id.clone(),
        message,
    })
}
