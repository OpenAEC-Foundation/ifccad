use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Caller-supplied IFCX header metadata; writing never invents identifiers or dates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcxCadHeader {
    pub id: String,
    pub data_version: String,
    pub author: String,
    pub timestamp: String,
}

/// A small, standalone drawing in the experimental IFCX CAD profile.
#[derive(Clone, Debug, PartialEq)]
pub struct IfcxCadDocument {
    pub header: IfcxCadHeader,
    pub drawing_id: u64,
    /// Persistent watermarks; native reading/writing never recomputes them.
    pub id_counters: super::IfcxCadIdCounters,
    pub length_unit: String,
    pub line_patterns: Vec<IfcxCadLinePattern>,
    pub line_pattern_scale: f64,
    pub layers: Vec<IfcxCadLayer>,
    pub model: IfcxCadLayout,
    /// Paper scopes form an unordered set; readback returns them in increasing ID order.
    pub paper_layouts: Vec<IfcxCadPaperLayout>,
    pub blocks: Vec<IfcxCadBlockDefinition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IfcxCadLayer {
    pub id: u64,
    pub name: String,
    pub appearance: IfcxCadLayerAppearance,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcxCadLayerAppearance {
    pub color: String,
    pub opacity: f64,
    pub line_pattern: IfcxCadLinePatternId,
    pub line_weight: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfcxCadLayout {
    pub id: u64,
    /// Vector position is the CAD draw order.
    pub entities: Vec<IfcxCadEntity>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfcxCadPaperLayout {
    pub id: u64,
    pub name: String,
    pub paper: IfcxCadPaperSize,
    /// Vector position is this paper layout's CAD draw order.
    pub entities: Vec<IfcxCadEntity>,
}

/// Sheet dimensions and the coordinate unit of direct paper-layout entities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcxCadPaperSize {
    pub width: f64,
    pub height: f64,
    pub length_unit: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfcxCadBlockDefinition {
    pub id: u64,
    pub name: String,
    pub base_point: [f64; 3],
    pub insertion_unit: String,
    pub entities: Vec<IfcxCadEntity>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "value")]
pub enum IfcxCadMode<T> {
    ByLayer,
    ByBlock,
    Explicit(T),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcxCadEntityAppearance {
    pub color: IfcxCadMode<String>,
    pub opacity: IfcxCadMode<f64>,
    pub line_pattern: IfcxCadMode<IfcxCadLinePatternId>,
    pub line_weight: IfcxCadMode<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfcxCadEntity {
    pub id: u64,
    pub layer_id: u64,
    pub appearance: IfcxCadEntityAppearance,
    pub line_pattern_scale: f64,
    pub kind: IfcxCadEntityKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcxCadPlacement {
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcxCadBlockTransform {
    pub placement: IfcxCadPlacement,
    pub rotation: f64,
    pub scale: [f64; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub enum IfcxCadEntityKind {
    LineSegment {
        start: [f64; 3],
        end: [f64; 3],
    },
    PlanarPolyline {
        vertices: Vec<[f64; 2]>,
        closed: bool,
        placement: IfcxCadPlacement,
        line_pattern_generation: IfcxCadLinePatternGeneration,
    },
    Circle {
        radius: f64,
        placement: IfcxCadPlacement,
    },
    BlockInstance {
        definition_id: u64,
        transform: IfcxCadBlockTransform,
    },
}

/// Drawing-local pattern identity; its complete IFCX path is the wire reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IfcxCadLinePatternId(pub u64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IfcxCadLinePattern {
    pub id: IfcxCadLinePatternId,
    pub name: String,
    pub description: Option<String>,
    pub pattern: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfcxCadLinePatternGeneration {
    #[default]
    PerSegment,
    Continuous,
}

/// The validated CAD projection and the composed IFCX graph, including extensions.
#[derive(Clone, Debug)]
pub struct ValidatedIfcxCad {
    pub(crate) raw: Value,
    pub(crate) document: IfcxCadDocument,
}

impl ValidatedIfcxCad {
    pub fn document(&self) -> &IfcxCadDocument {
        &self.document
    }
    pub fn raw_ifcx(&self) -> &Value {
        &self.raw
    }
}
