use super::IfccadViewport;
use serde::{Deserialize, Serialize};

/// Caller-supplied IFCX header metadata; writing never invents identifiers or dates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfccadHeader {
    pub id: String,
    pub data_version: String,
    pub author: String,
    pub timestamp: String,
}

/// Owned, editable CAD projection in the experimental IFCX profile.
///
/// This document does not contain foreign nodes or original IFCX fragments.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadDocument {
    pub header: IfccadHeader,
    pub drawing_id: u64,
    /// Persistent watermarks; native reading/writing never recomputes them.
    pub id_counters: super::IfccadIdCounters,
    pub length_unit: String,
    pub plot_style_mode: super::IfccadPlotStyleMode,
    pub line_patterns: Vec<IfccadLinePattern>,
    pub line_pattern_scale: f64,
    pub layers: Vec<IfccadLayer>,
    pub model: IfccadLayout,
    /// Collection position is incidental; tab_index defines layout-tab order.
    pub paper_layouts: Vec<IfccadPaperLayout>,
    pub blocks: Vec<IfccadBlockDefinition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IfccadLayer {
    pub id: u64,
    pub name: String,
    pub appearance: IfccadLayerAppearance,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfccadLayerAppearance {
    pub color: String,
    pub opacity: f64,
    pub line_pattern: IfccadLinePatternId,
    pub line_weight: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadLayout {
    pub settings: super::IfccadLayoutSettings,
    pub id: u64,
    pub tab_index: u32,
    pub bounds: Option<IfccadBounds3d>,
    /// Vector position is the CAD draw order.
    pub entities: Vec<IfccadEntity>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadPaperLayout {
    pub id: u64,
    pub name: String,
    pub tab_index: u32,
    pub settings: super::IfccadLayoutSettings,
    pub bounds: Option<IfccadBounds3d>,
    /// Vector position is this paper layout's CAD draw order.
    pub entities: Vec<IfccadEntity>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadBlockDefinition {
    pub id: u64,
    pub name: String,
    pub base_point: [f64; 3],
    pub insertion_unit: String,
    pub bounds: Option<IfccadBounds3d>,
    pub entities: Vec<IfccadEntity>,
}

/// Optional conservative XYZ enclosure in the owning numerical coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IfccadBounds3d {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "value")]
pub enum IfccadMode<T> {
    ByLayer,
    ByBlock,
    Explicit(T),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfccadEntityAppearance {
    pub color: IfccadMode<String>,
    pub opacity: IfccadMode<f64>,
    pub line_pattern: IfccadMode<IfccadLinePatternId>,
    pub line_weight: IfccadMode<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadEntity {
    pub id: u64,
    pub layer_id: u64,
    pub appearance: IfccadEntityAppearance,
    pub line_pattern_scale: f64,
    pub kind: IfccadEntityKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadPlacement {
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfccadBlockTransform {
    pub placement: IfccadPlacement,
    pub rotation: f64,
    pub scale: [f64; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub enum IfccadEntityKind {
    Viewport(IfccadViewport),
    Point {
        placement: IfccadPlacement,
    },
    Arc {
        radius: f64,
        start_parameter: f64,
        sweep_parameter: f64,
        placement: IfccadPlacement,
    },
    Ellipse {
        semi_major_radius: f64,
        semi_minor_radius: f64,
        placement: IfccadPlacement,
    },
    EllipseArc {
        semi_major_radius: f64,
        semi_minor_radius: f64,
        start_parameter: f64,
        sweep_parameter: f64,
        placement: IfccadPlacement,
    },
    LineSegment {
        start: [f64; 3],
        end: [f64; 3],
    },
    PlanarPolyline {
        vertices: Vec<[f64; 2]>,
        bulges: Vec<f64>,
        closed: bool,
        placement: IfccadPlacement,
        line_pattern_generation: IfccadLinePatternGeneration,
    },
    SpatialPolyline {
        vertices: Vec<[f64; 3]>,
        closed: bool,
        line_pattern_generation: IfccadLinePatternGeneration,
    },
    Circle {
        radius: f64,
        placement: IfccadPlacement,
    },
    BlockInstance {
        definition_id: u64,
        transform: IfccadBlockTransform,
    },
}

/// Drawing-local pattern identity; its complete IFCX path is the wire reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IfccadLinePatternId(pub u64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IfccadLinePattern {
    pub id: IfccadLinePatternId,
    pub name: String,
    pub description: Option<String>,
    pub pattern: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadLinePatternGeneration {
    #[default]
    PerSegment,
    Continuous,
}
