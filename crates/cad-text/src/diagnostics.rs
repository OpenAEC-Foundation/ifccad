#[derive(Debug, thiserror::Error)]
pub enum CadTextError {
    #[error("CAD text semantics are outside the qualified profile: {0}")]
    Unsupported(&'static str),
    #[error("invalid CAD text structure: {0}")]
    InvalidSource(&'static str),
    #[error("invalid native text values: {0}")]
    Values(#[from] ocdraw::text::TextValueError),
    #[error("invalid CAD text content: {0}")]
    Markup(#[from] crate::markup::TextMarkupError),
    #[error("invalid text frame: {0}")]
    Frame(#[from] ocdraw::geometry_kernel::CoordinateFrameError),
    #[error("CAD text numeric preparation failed: {0}")]
    Preparation(#[from] cad_geometry_convert::CadPreparationError),
    #[error("CAD text numeric preparation exceeds finite range")]
    OutOfRange,
}

/// Route adapters locate/classify these, and enforce their own Allow/Reject policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CadTextIssue {
    CharacterBasisNormalized,
    AuthoredFormattingNormalized,
    LineSpacingReferenceChanged,
    ParagraphBasisExpanded,
    PaddingReferenceChanged,
    InactiveColumnPropertiesOmitted,
    InactiveBackgroundPropertiesOmitted,
}
