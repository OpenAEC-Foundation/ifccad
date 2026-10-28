//! Located source tokens; not a native text tree or a source-preservation record.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkupLimits {
    pub max_bytes: usize,
    pub max_depth: usize,
    pub max_tokens: usize,
}
impl Default for MarkupLimits {
    fn default() -> Self {
        Self {
            max_bytes: 16 * 1024 * 1024,
            max_depth: 256,
            max_tokens: 1_000_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextMarkupErrorCode {
    MalformedCode,
    UnsupportedCode,
    InvalidNumber,
    InvalidUnicode,
    UnbalancedScope,
    DynamicField,
    ResourceLimit,
    StructuralControl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextMarkupError {
    pub code: TextMarkupErrorCode,
    pub offset: usize,
    pub message: &'static str,
}
impl TextMarkupError {
    pub(super) fn new(code: TextMarkupErrorCode, offset: usize, message: &'static str) -> Self {
        Self {
            code,
            offset,
            message,
        }
    }
}
impl std::fmt::Display for TextMarkupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} at byte {}: {}",
            self.code, self.offset, self.message
        )
    }
}
impl std::error::Error for TextMarkupError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkupDecoration {
    Underline,
    Overline,
    StrikeThrough,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkupScalarProperty {
    Height,
    Width,
    Tracking,
    ObliqueDegrees,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretControl {
    LineFeed,
    CarriageReturn,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MarkupTokenKind {
    Literal(String),
    ScopeStart,
    ScopeEnd,
    ParagraphBreak,
    ColumnBreak,
    Tab,
    /// Keep lexical spelling distinct until the conversion profile qualifies it.
    CaretControl(CaretControl),
    RawLineFeed,
    RawCarriageReturn,
    RawCarriageReturnLineFeed,
    Decoration {
        decoration: MarkupDecoration,
        enabled: bool,
        toggle: bool,
    },
    Scalar {
        property: MarkupScalarProperty,
        value: f64,
        relative: bool,
    },
    IndexedColor(u16),
    PackedBgrColor(u32),
    LineAlignment(u8),
    Font {
        body: String,
        shx: bool,
    },
    /// Source grammar retained for typed normalization, never a native payload.
    ParagraphProperties(String),
    Stack(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocatedMarkupToken {
    pub kind: MarkupTokenKind,
    pub range: std::ops::Range<usize>,
}

pub(super) fn scalar(
    property: MarkupScalarProperty,
    body: &str,
    offset: usize,
) -> Result<MarkupTokenKind, TextMarkupError> {
    let relative = body.ends_with('x');
    if relative && property == MarkupScalarProperty::ObliqueDegrees {
        return Err(TextMarkupError::new(
            TextMarkupErrorCode::MalformedCode,
            offset,
            "oblique angle cannot use a relative suffix",
        ));
    }
    let number = if relative {
        &body[..body.len() - 1]
    } else {
        body
    };
    let value = number.parse::<f64>().map_err(|_| {
        TextMarkupError::new(
            TextMarkupErrorCode::InvalidNumber,
            offset,
            "invalid scalar number",
        )
    })?;
    if !value.is_finite()
        || (property != MarkupScalarProperty::ObliqueDegrees && value <= 0.0)
        || (property == MarkupScalarProperty::ObliqueDegrees && value.abs() >= 90.0)
    {
        return Err(TextMarkupError::new(
            TextMarkupErrorCode::InvalidNumber,
            offset,
            "invalid finite scalar range",
        ));
    }
    Ok(MarkupTokenKind::Scalar {
        property,
        value,
        relative,
    })
}
