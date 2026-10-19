//! Drawing-local line pattern identities and ordered pattern definitions.
use super::{OcdrawTextStyleId, SymbolSourceId};
use crate::line_pattern::{LinePatternElement, LinePatternTransform};

#[derive(Clone, Debug, PartialEq)]
pub enum DrawingLinePatternMarker {
    Text {
        text: String,
        text_style_id: OcdrawTextStyleId,
        transform: LinePatternTransform,
    },
    Symbol {
        symbol_source_id: SymbolSourceId,
        shape_number: u32,
        transform: LinePatternTransform,
    },
}
pub type DrawingLinePatternElement = LinePatternElement<DrawingLinePatternMarker>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize)]
#[serde(transparent)]
/// Drawing-local identity, independent of table order and CAD handles.
pub struct LinePatternId(pub u32);

#[derive(Clone, Debug, PartialEq)]
/// Validated named simple pattern; an empty sequence means continuous.
pub struct DrawingLinePattern {
    pub id: LinePatternId,
    pub name: String,
    pub description: Option<String>,
    pub pattern: Vec<DrawingLinePatternElement>,
}

#[derive(Clone, Debug, PartialEq)]
/// Authored pattern in drawing units: positive stroke, negative gap, zero dot.
pub struct LinePatternDefinition {
    pub name: String,
    pub description: Option<String>,
    pub pattern: Vec<DrawingLinePatternElement>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Whether a polyline restarts its pattern at each edge or across the whole path.
pub enum LinePatternGeneration {
    #[default]
    PerSegment,
    Continuous,
}

impl LinePatternGeneration {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::PerSegment => "perSegment",
            Self::Continuous => "continuous",
        }
    }
    pub(crate) fn from_token(value: Option<&str>) -> Self {
        match value {
            Some("continuous") => Self::Continuous,
            _ => Self::PerSegment,
        }
    }
}
