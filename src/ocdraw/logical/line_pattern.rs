//! Drawing-local line pattern identities and simple pattern definitions.

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
    pub pattern: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
/// Authored pattern in drawing units: positive stroke, negative gap, zero dot.
pub struct LinePatternDefinition {
    pub name: String,
    pub description: Option<String>,
    pub pattern: Vec<f64>,
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
