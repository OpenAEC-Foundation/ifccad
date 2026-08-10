use super::{ConformanceError, BUNDLED_CONFORMANCE_VERSION};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceCategory {
    Valid,
    Invalid,
    Lifecycle,
    Planning,
    Vector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceOperationName {
    ValidateSchema,
    ValidatePackage,
    Canonicalize,
    Fingerprint,
    EvaluateLifecycle,
    PlanExport,
    ExtractSource,
    RoundtripPackage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConformanceManifest {
    pub suite_version: String,
    pub cases: Vec<ConformanceCase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConformanceCase {
    pub case_id: String,
    pub category: ConformanceCategory,
    pub description: String,
    pub entrypoint: String,
    pub operations: Vec<ConformanceOperation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConformanceOperation {
    pub name: ConformanceOperationName,
    pub expected: ExpectedOutcome,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpectedOutcome {
    #[serde(default)]
    pub diagnostics: Vec<Value>,
    pub canonical_utf8_base64: Option<String>,
    pub normalized_json: Option<String>,
    pub fingerprint: Option<String>,
    pub lifecycle_rows: Option<Vec<Value>>,
    pub record_actions: Option<Vec<Value>>,
    pub attachment_actions: Option<Vec<Value>>,
    pub blocked: Option<bool>,
    pub superseded_targets: Option<Vec<Value>>,
    pub canonical_resource_hashes: Option<BTreeMap<String, String>>,
    pub extracted_source_base64: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawManifest {
    suite_version: String,
    cases: Vec<RawCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawCase {
    case_id: String,
    category: String,
    description: String,
    entrypoint: String,
    operations: Vec<RawOperation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOperation {
    name: String,
    expected: ExpectedOutcome,
}

pub fn parse_conformance_manifest(
    source: &str,
    origin: &Path,
) -> Result<ConformanceManifest, ConformanceError> {
    let raw: RawManifest =
        serde_json::from_str(source).map_err(|source| ConformanceError::MalformedJson {
            path: origin.to_path_buf(),
            source,
        })?;
    if raw.suite_version != BUNDLED_CONFORMANCE_VERSION {
        return Err(ConformanceError::UnsupportedSuiteVersion {
            found: raw.suite_version,
        });
    }

    let mut identifiers = BTreeSet::new();
    let mut cases = Vec::with_capacity(raw.cases.len());
    for case in raw.cases {
        if !valid_case_id(&case.case_id) {
            return Err(ConformanceError::InvalidCaseId {
                case_id: case.case_id,
            });
        }
        if !identifiers.insert(case.case_id.clone()) {
            return Err(ConformanceError::DuplicateCaseId {
                case_id: case.case_id,
            });
        }
        if case.operations.is_empty() {
            return Err(ConformanceError::EmptyOperations {
                case_id: case.case_id,
            });
        }
        validate_entrypoint_syntax(&case.case_id, &case.entrypoint)?;
        let category = parse_category(&case.case_id, &case.category)?;
        let operations = case
            .operations
            .into_iter()
            .map(|operation| {
                Ok(ConformanceOperation {
                    name: parse_operation(&case.case_id, &operation.name)?,
                    expected: operation.expected,
                })
            })
            .collect::<Result<Vec<_>, ConformanceError>>()?;
        cases.push(ConformanceCase {
            case_id: case.case_id,
            category,
            description: case.description,
            entrypoint: case.entrypoint,
            operations,
        });
    }
    Ok(ConformanceManifest {
        suite_version: raw.suite_version,
        cases,
    })
}

pub fn load_conformance_manifest(
    root: impl AsRef<Path>,
) -> Result<ConformanceManifest, ConformanceError> {
    let root = root.as_ref();
    let manifest_path = root.join("manifest.json");
    let source = fs::read_to_string(&manifest_path).map_err(|source| ConformanceError::Io {
        path: manifest_path.clone(),
        source,
    })?;
    let manifest = parse_conformance_manifest(&source, &manifest_path)?;
    let canonical_root = fs::canonicalize(root).map_err(|source| ConformanceError::Io {
        path: root.to_path_buf(),
        source,
    })?;

    for case in &manifest.cases {
        let target = root.join(&case.entrypoint);
        if !target.is_file() {
            return Err(ConformanceError::MissingEntrypoint {
                case_id: case.case_id.clone(),
                path: target,
            });
        }
        let canonical_target =
            fs::canonicalize(&target).map_err(|source| ConformanceError::Io {
                path: target.clone(),
                source,
            })?;
        if !canonical_target.starts_with(&canonical_root) {
            return Err(ConformanceError::UnsafeEntrypoint {
                case_id: case.case_id.clone(),
                entrypoint: case.entrypoint.clone(),
            });
        }
    }
    Ok(manifest)
}

fn valid_case_id(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphanumeric())
        && chars.all(|value| value.is_ascii_alphanumeric() || matches!(value, '.' | '_' | '-'))
}

fn validate_entrypoint_syntax(case_id: &str, entrypoint: &str) -> Result<(), ConformanceError> {
    let segments: Vec<_> = entrypoint.split('/').collect();
    let unsafe_path = entrypoint.is_empty()
        || entrypoint.contains('\\')
        || entrypoint.starts_with('/')
        || segments.first().is_some_and(|value| value.contains(':'))
        || segments
            .iter()
            .any(|value| value.is_empty() || *value == "." || *value == "..");
    if unsafe_path {
        return Err(ConformanceError::UnsafeEntrypoint {
            case_id: case_id.to_owned(),
            entrypoint: entrypoint.to_owned(),
        });
    }
    Ok(())
}

fn parse_category(case_id: &str, value: &str) -> Result<ConformanceCategory, ConformanceError> {
    match value {
        "valid" => Ok(ConformanceCategory::Valid),
        "invalid" => Ok(ConformanceCategory::Invalid),
        "lifecycle" => Ok(ConformanceCategory::Lifecycle),
        "planning" => Ok(ConformanceCategory::Planning),
        "vector" => Ok(ConformanceCategory::Vector),
        category => Err(ConformanceError::UnknownCategory {
            case_id: case_id.to_owned(),
            category: category.to_owned(),
        }),
    }
}

fn parse_operation(
    case_id: &str,
    value: &str,
) -> Result<ConformanceOperationName, ConformanceError> {
    match value {
        "validateSchema" => Ok(ConformanceOperationName::ValidateSchema),
        "validatePackage" => Ok(ConformanceOperationName::ValidatePackage),
        "canonicalize" => Ok(ConformanceOperationName::Canonicalize),
        "fingerprint" => Ok(ConformanceOperationName::Fingerprint),
        "evaluateLifecycle" => Ok(ConformanceOperationName::EvaluateLifecycle),
        "planExport" => Ok(ConformanceOperationName::PlanExport),
        "extractSource" => Ok(ConformanceOperationName::ExtractSource),
        "roundtripPackage" => Ok(ConformanceOperationName::RoundtripPackage),
        operation => Err(ConformanceError::UnknownOperation {
            case_id: case_id.to_owned(),
            operation: operation.to_owned(),
        }),
    }
}
