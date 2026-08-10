use std::fmt;

/// Stable code identifying why an IFCCAD value cannot be canonicalized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalizationErrorCode {
    TypeInvalid,
    TypeUnsupported,
    BoolInvalid,
    IntegerInvalid,
    FloatInvalid,
    StringInvalid,
    EnumInvalid,
    SequenceInvalid,
    MappingInvalid,
    MappingKeyInvalid,
    MappingKeyDuplicate,
    RecordInvalid,
    RecordFieldInvalid,
}

impl CanonicalizationErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TypeInvalid => "VECTOR_TYPE_INVALID",
            Self::TypeUnsupported => "VECTOR_TYPE_UNSUPPORTED",
            Self::BoolInvalid => "VECTOR_BOOL_INVALID",
            Self::IntegerInvalid => "VECTOR_INTEGER_INVALID",
            Self::FloatInvalid => "VECTOR_FLOAT_INVALID",
            Self::StringInvalid => "VECTOR_STRING_INVALID",
            Self::EnumInvalid => "VECTOR_ENUM_INVALID",
            Self::SequenceInvalid => "VECTOR_SEQUENCE_INVALID",
            Self::MappingInvalid => "VECTOR_MAPPING_INVALID",
            Self::MappingKeyInvalid => "VECTOR_MAPPING_KEY_INVALID",
            Self::MappingKeyDuplicate => "VECTOR_MAPPING_KEY_DUPLICATE",
            Self::RecordInvalid => "VECTOR_RECORD_INVALID",
            Self::RecordFieldInvalid => "VECTOR_RECORD_FIELD_INVALID",
        }
    }
}

/// Failure to convert an IFCCAD value to its canonical representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalizationError {
    code: CanonicalizationErrorCode,
    message: &'static str,
}

impl CanonicalizationError {
    pub(crate) const fn new(code: CanonicalizationErrorCode, message: &'static str) -> Self {
        Self { code, message }
    }

    pub const fn code(&self) -> CanonicalizationErrorCode {
        self.code
    }
}

impl fmt::Display for CanonicalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for CanonicalizationError {}
