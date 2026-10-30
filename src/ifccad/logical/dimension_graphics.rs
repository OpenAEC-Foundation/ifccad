//! Owned worked-out picture values; no native identity or freshness state.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadDimensionGraphicsOrigin {
    SourceSnapshot,
    Generated,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfccadDimensionGraphicsPurpose {
    #[default]
    Display,
    ReferencePoint,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadDimensionGraphics {
    pub origin: IfccadDimensionGraphicsOrigin,
    pub parts: Vec<IfccadDimensionGraphicsPart>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadDimensionGraphicsPart {
    pub layer_id: Option<u64>,
    pub kind: IfccadDimensionGraphicsKind,
    pub appearance: IfccadEntityAppearance,
    pub visible: bool,
    pub line_pattern_scale: f64,
    pub purpose: IfccadDimensionGraphicsPurpose,
}
#[derive(Clone, Debug, PartialEq)]
pub enum IfccadDimensionGraphicsKind {
    Solid(IfccadFilledQuadrilateral),
    Trace(IfccadFilledQuadrilateral),
    Hatch(Box<super::IfccadHatch>),
    Text(super::IfccadText),
    MText(Box<super::IfccadMText>),
    Point {
        placement: IfccadPlacement,
        thickness: f64,
    },
    Arc {
        radius: f64,
        start_parameter: f64,
        sweep_parameter: f64,
        placement: IfccadPlacement,
        thickness: f64,
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
        thickness: f64,
        extrusion_direction: [f64; 3],
    },
    PlanarPolyline {
        widths: IfccadPlanarPolylineWidths,
        vertices: Vec<[f64; 2]>,
        bulges: Vec<f64>,
        closed: bool,
        placement: IfccadPlacement,
        line_pattern_generation: IfccadLinePatternGeneration,
        thickness: f64,
    },
    SpatialPolyline {
        vertices: Vec<[f64; 3]>,
        closed: bool,
        line_pattern_generation: IfccadLinePatternGeneration,
    },
    Circle {
        radius: f64,
        placement: IfccadPlacement,
        thickness: f64,
    },
    BlockInstance {
        definition_id: u64,
        transform: IfccadBlockTransform,
    },
}

impl From<IfccadDimensionGraphicsKind> for IfccadEntityKind {
    fn from(v: IfccadDimensionGraphicsKind) -> Self {
        match v {
            IfccadDimensionGraphicsKind::Solid(v) => Self::Solid(v),
            IfccadDimensionGraphicsKind::Trace(v) => Self::Trace(v),
            IfccadDimensionGraphicsKind::Hatch(v) => Self::Hatch(*v),
            IfccadDimensionGraphicsKind::Text(v) => Self::Text(v),
            IfccadDimensionGraphicsKind::MText(v) => Self::MText(v),
            IfccadDimensionGraphicsKind::Point {
                placement,
                thickness,
                ..
            } => Self::Point {
                placement,
                thickness,
            },
            IfccadDimensionGraphicsKind::Arc {
                radius,
                start_parameter,
                sweep_parameter,
                placement,
                thickness,
                ..
            } => Self::Arc {
                radius,
                start_parameter,
                sweep_parameter,
                placement,
                thickness,
            },
            IfccadDimensionGraphicsKind::Ellipse {
                semi_major_radius,
                semi_minor_radius,
                placement,
            } => Self::Ellipse {
                semi_major_radius,
                semi_minor_radius,
                placement,
            },
            IfccadDimensionGraphicsKind::EllipseArc {
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
                placement,
            } => Self::EllipseArc {
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
                placement,
            },
            IfccadDimensionGraphicsKind::LineSegment {
                start,
                end,
                thickness,
                extrusion_direction,
                ..
            } => Self::LineSegment {
                start,
                end,
                thickness,
                extrusion_direction,
            },
            IfccadDimensionGraphicsKind::PlanarPolyline {
                widths,
                vertices,
                bulges,
                closed,
                placement,
                line_pattern_generation,
                thickness,
                ..
            } => Self::PlanarPolyline {
                widths,
                vertices,
                bulges,
                closed,
                placement,
                line_pattern_generation,
                thickness,
            },
            IfccadDimensionGraphicsKind::SpatialPolyline {
                vertices,
                closed,
                line_pattern_generation,
            } => Self::SpatialPolyline {
                vertices,
                closed,
                line_pattern_generation,
            },
            IfccadDimensionGraphicsKind::Circle {
                radius,
                placement,
                thickness,
                ..
            } => Self::Circle {
                radius,
                placement,
                thickness,
            },
            IfccadDimensionGraphicsKind::BlockInstance {
                definition_id,
                transform,
            } => Self::BlockInstance {
                definition_id,
                transform,
            },
        }
    }
}

impl TryFrom<IfccadEntityKind> for IfccadDimensionGraphicsKind {
    type Error = IfccadReport;
    fn try_from(v: IfccadEntityKind) -> Result<Self, Self::Error> {
        Ok(match v {
            IfccadEntityKind::Solid(v) => Self::Solid(v),
            IfccadEntityKind::Trace(v) => Self::Trace(v),
            IfccadEntityKind::Hatch(v) => Self::Hatch(Box::new(v)),
            IfccadEntityKind::Text(v) => Self::Text(v),
            IfccadEntityKind::MText(v) => Self::MText(v),
            IfccadEntityKind::Point {
                placement,
                thickness,
                ..
            } => Self::Point {
                placement,
                thickness,
            },
            IfccadEntityKind::Arc {
                radius,
                start_parameter,
                sweep_parameter,
                placement,
                thickness,
                ..
            } => Self::Arc {
                radius,
                start_parameter,
                sweep_parameter,
                placement,
                thickness,
            },
            IfccadEntityKind::Ellipse {
                semi_major_radius,
                semi_minor_radius,
                placement,
            } => Self::Ellipse {
                semi_major_radius,
                semi_minor_radius,
                placement,
            },
            IfccadEntityKind::EllipseArc {
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
                placement,
            } => Self::EllipseArc {
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
                placement,
            },
            IfccadEntityKind::LineSegment {
                start,
                end,
                thickness,
                extrusion_direction,
                ..
            } => Self::LineSegment {
                start,
                end,
                thickness,
                extrusion_direction,
            },
            IfccadEntityKind::PlanarPolyline {
                widths,
                vertices,
                bulges,
                closed,
                placement,
                line_pattern_generation,
                thickness,
                ..
            } => Self::PlanarPolyline {
                widths,
                vertices,
                bulges,
                closed,
                placement,
                line_pattern_generation,
                thickness,
            },
            IfccadEntityKind::SpatialPolyline {
                vertices,
                closed,
                line_pattern_generation,
            } => Self::SpatialPolyline {
                vertices,
                closed,
                line_pattern_generation,
            },
            IfccadEntityKind::Circle {
                radius,
                placement,
                thickness,
                ..
            } => Self::Circle {
                radius,
                placement,
                thickness,
            },
            IfccadEntityKind::BlockInstance {
                definition_id,
                transform,
            } => Self::BlockInstance {
                definition_id,
                transform,
            },
            _ => {
                return Err(crate::ifccad::diagnostics::failure(
                    "IFCCAD-DIMENSION-GRAPHICS-001",
                    "/graphics",
                    "unsupported graphics kind",
                ))
            }
        })
    }
}
