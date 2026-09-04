use crate::ResourceId;
use serde::Serialize;
use std::collections::BTreeMap;

/// Severity of a structured IFCCAD package diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageDiagnosticSeverity {
    /// The package violates a required rule.
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
