use super::ConformanceError;
use crate::canonicalization::fingerprint_typed_value;
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
    fingerprint: String,
}

/// Execute every row in an IFCCAD fingerprint vector document.
///
/// Returns the number of verified rows. The first mismatch is reported with
/// its vector identifier so a damaged corpus or implementation is actionable.
pub fn verify_fingerprint_vectors(path: impl AsRef<Path>) -> Result<usize, ConformanceError> {
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
        let actual = fingerprint_typed_value(&row.input).map_err(|error| {
            ConformanceError::VectorMismatch {
                vector_id: row.vector_id.clone(),
                message: format!("expected fingerprint, received {}", error.code().as_str()),
            }
        })?;
        if actual != row.fingerprint {
            return Err(ConformanceError::VectorMismatch {
                vector_id: row.vector_id.clone(),
                message: format!("expected {}, received {actual}", row.fingerprint),
            });
        }
    }

    Ok(document.vectors.len())
}
