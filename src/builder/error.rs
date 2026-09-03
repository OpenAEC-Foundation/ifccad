#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BuildError {
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
    #[error("{kind} ID or count range is exhausted")]
    RangeExhausted { kind: &'static str },
    #[error("{stage} encoding failed: {message}")]
    Encoding {
        stage: &'static str,
        message: String,
    },
}
