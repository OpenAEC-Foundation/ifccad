#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
/// Failure while validating or encoding a package under construction.
pub enum PackageBuildError {
    #[error("a package requires one drawing before it can be finished")]
    DrawingMissing,
    #[error("this package already has a drawing")]
    DrawingAlreadyDefined,
    #[error("{field} must not be empty")]
    EmptyValue { field: &'static str },
    #[error("timestamp must be valid RFC 3339 with Z or +00:00")]
    InvalidTimestamp,
    #[error("opacity must be finite and between 0 and 1")]
    InvalidOpacity,
    #[error("line weight must be finite and non-negative")]
    InvalidLineWeight,
    #[error("coordinate values must be finite")]
    NonFiniteCoordinate,
    #[error("a polyline requires at least two points")]
    PolylineTooShort,
    #[error("layer name already exists (case-insensitive): {name}")]
    DuplicateLayerName { name: String },
    #[error("layer key belongs to another builder")]
    ForeignLayerKey,
    #[error("appearance key belongs to another builder")]
    ForeignAppearanceKey,
    #[error("an explicit appearance mode requires an appearance definition")]
    AppearanceDefinitionMissing,
    #[error("{kind} ID or count range is exhausted")]
    RangeExhausted { kind: &'static str },
    #[error("{stage} encoding failed: {message}")]
    Encoding {
        stage: &'static str,
        message: String,
    },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
/// Failure while publishing an encoded package directory.
pub enum PackageWriteError {
    #[error("target package already exists: {path}", path = .path.display())]
    TargetExists { path: std::path::PathBuf },
    #[error("target package must have a final path component: {path}", path = .path.display())]
    InvalidTarget { path: std::path::PathBuf },
    #[error("invalid package artifact path: {path}")]
    InvalidArtifactPath { path: String },
    #[error("could not {operation} at {path}: {source}", path = .path.display())]
    Io {
        operation: &'static str,
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}
