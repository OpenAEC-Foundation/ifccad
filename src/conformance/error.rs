use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConformanceError {
    #[error("failed to read IFCCAD conformance file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("malformed IFCCAD conformance JSON in {path}: {source}")]
    MalformedJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("unsupported IFCCAD conformance suite version {found}")]
    UnsupportedSuiteVersion { found: String },
    #[error("invalid IFCCAD conformance case ID {case_id}")]
    InvalidCaseId { case_id: String },
    #[error("duplicate IFCCAD conformance case ID {case_id}")]
    DuplicateCaseId { case_id: String },
    #[error("case {case_id} uses unknown category {category}")]
    UnknownCategory { case_id: String, category: String },
    #[error("case {case_id} uses unknown operation {operation}")]
    UnknownOperation { case_id: String, operation: String },
    #[error("case {case_id} declares no operations")]
    EmptyOperations { case_id: String },
    #[error("case {case_id} uses unsafe entrypoint {entrypoint}")]
    UnsafeEntrypoint { case_id: String, entrypoint: String },
    #[error("case {case_id} entrypoint is missing: {path}")]
    MissingEntrypoint { case_id: String, path: PathBuf },
    #[error("canonicalization vector {vector_id} does not match: {message}")]
    VectorMismatch { vector_id: String, message: String },
}
