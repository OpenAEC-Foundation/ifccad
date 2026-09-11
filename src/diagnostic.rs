use crate::ResourceId;
use serde::Serialize;
use std::collections::BTreeMap;

/// Kind of evidence, independently of diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PackageDiagnosticCategory {
    /// A rule of a known applicable contract was violated.
    ContractViolation,
    /// The reader does not implement the requested profile or content.
    UnsupportedContent,
    /// Required input or execution capacity was unavailable.
    ExecutionBlocked,
}

/// Severity of a structured IFCCAD package diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageDiagnosticSeverity {
    /// A rule violation or unsupported content blocks strict validation.
    Error,
    /// The package is usable, but deserves attention.
    Warning,
    /// Informational validation output.
    Info,
}

/// Scalar value carried in a diagnostic's machine-readable context.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PackageDiagnosticContextValue {
    /// JSON `null`.
    Null,
    /// A boolean value.
    Boolean(bool),
    /// A finite JSON number.
    Number(serde_json::Number),
    /// A string value.
    String(String),
}

/// Structured, language-neutral IFCCAD package diagnostic.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDiagnostic {
    /// What the diagnostic establishes, independently of its severity.
    pub category: PackageDiagnosticCategory,
    /// Stable machine-readable diagnostic code.
    pub code: String,
    /// Diagnostic severity.
    pub severity: PackageDiagnosticSeverity,
    /// Logical package resource identity, when known.
    pub resource_id: Option<ResourceId>,
    /// Package-relative resource URI, when the issue belongs to a resource.
    pub resource_uri: Option<String>,
    /// RFC 6901 JSON Pointer, when a location can be identified.
    pub location: Option<String>,
    /// Ordered scalar details used to identify and explain the issue.
    pub context: BTreeMap<String, PackageDiagnosticContextValue>,
    /// Non-normative explanation intended for people.
    pub message: String,
}
