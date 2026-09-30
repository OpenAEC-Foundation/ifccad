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
    pub length_unit: String,
    pub layers: Vec<IfcxCadLayer>,
    pub model: IfcxCadLayout,
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
    pub line_pattern: String,
    pub line_weight: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfcxCadLayout {
    pub id: u64,
    /// Vector position is the CAD draw order.
    pub entities: Vec<IfcxCadEntity>,
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
    pub line_pattern: IfcxCadMode<String>,
    pub line_weight: IfcxCadMode<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfcxCadEntity {
    pub id: u64,
    pub layer_id: u64,
    pub appearance: IfcxCadEntityAppearance,
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
