//! Marker scalars and source basenames; native identity resolution belongs to adapters.
use opencadcodec::{
    tables::{LineTypeComplexContent, LineTypeComplexData},
    Handle,
};

#[derive(Clone, Debug, PartialEq)]
pub enum CadLinePatternMarkerContent {
    Text(String),
    Shape(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CadLinePatternMarkerValues {
    pub content: CadLinePatternMarkerContent,
    pub scale: f64,
    pub rotation: f64,
    pub absolute_rotation: bool,
    pub upright: bool,
    pub offset: [f64; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CadShxFileName {
    pub file_name: String,
    pub discarded_locator: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CadLinePatternValueError {
    #[error("invalid marker value: {0}")]
    InvalidValue(&'static str),
    #[error("unsupported marker value: {0}")]
    Unsupported(&'static str),
}

fn validate(value: &CadLinePatternMarkerValues) -> Result<(), CadLinePatternValueError> {
    if !value.scale.is_finite()
        || !value.rotation.is_finite()
        || value.offset.iter().any(|v| !v.is_finite())
    {
        return Err(CadLinePatternValueError::InvalidValue(
            "nonfinite transform",
        ));
    }
    if value.absolute_rotation && value.upright {
        return Err(CadLinePatternValueError::Unsupported(
            "conflicting absolute/upright marker rotation",
        ));
    }
    if value.scale <= 0. {
        return Err(CadLinePatternValueError::Unsupported("nonpositive scale"));
    }
    Ok(())
}

pub fn marker_values_from_cad(
    value: &LineTypeComplexData,
) -> Result<CadLinePatternMarkerValues, CadLinePatternValueError> {
    // Check nonfinite values before classifying unsupported selectors.
    let mut result = CadLinePatternMarkerValues {
        content: CadLinePatternMarkerContent::Shape(0),
        scale: value.scale,
        rotation: value.rotation,
        absolute_rotation: value.absolute_rotation,
        upright: value.upright,
        offset: value.offset,
    };
    validate(&result)?;
    if value.source_flags & !0x0f != 0 {
        return Err(CadLinePatternValueError::Unsupported(
            "unqualified complex linetype flags",
        ));
    }
    result.content = match &value.content {
        LineTypeComplexContent::Text { text } => CadLinePatternMarkerContent::Text(text.clone()),
        LineTypeComplexContent::Shape { shape_number } => CadLinePatternMarkerContent::Shape(
            u32::try_from(*shape_number)
                .map_err(|_| CadLinePatternValueError::Unsupported("negative shape selector"))?,
        ),
    };
    Ok(result)
}

pub fn marker_values_to_cad(
    value: &CadLinePatternMarkerValues,
    style_handle: Handle,
) -> Result<LineTypeComplexData, CadLinePatternValueError> {
    validate(value)?;
    if style_handle.is_null() {
        return Err(CadLinePatternValueError::InvalidValue(
            "unresolved STYLE handle",
        ));
    }
    let content = match &value.content {
        CadLinePatternMarkerContent::Text(text) => {
            LineTypeComplexContent::Text { text: text.clone() }
        }
        CadLinePatternMarkerContent::Shape(shape) => LineTypeComplexContent::Shape {
            shape_number: i16::try_from(*shape).map_err(|_| {
                CadLinePatternValueError::Unsupported("shape selector exceeds signed CAD range")
            })?,
        },
    };
    Ok(LineTypeComplexData {
        source_flags: 0,
        upright: value.upright,
        content,
        style_handle,
        scale: value.scale,
        rotation: value.rotation,
        absolute_rotation: value.absolute_rotation,
        offset: value.offset,
    })
}

pub fn shx_file_name_from_cad(value: &str) -> Result<CadShxFileName, CadLinePatternValueError> {
    let file_name = value.rsplit(['/', '\\']).next().unwrap_or("");
    if file_name.is_empty() || matches!(file_name, "." | "..") || value.contains('\0') {
        return Err(CadLinePatternValueError::InvalidValue(
            "empty or invalid SHX basename",
        ));
    }
    Ok(CadShxFileName {
        file_name: file_name.into(),
        discarded_locator: file_name != value,
    })
}
