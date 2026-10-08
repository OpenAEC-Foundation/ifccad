use crate::geometry_kernel::hatch::{HatchBoundary2 as B, HatchEdge2 as E};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum BoundaryValue {
    Polyline {
        vertices: Vec<[f64; 2]>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bulges: Option<Vec<f64>>,
    },
    Circle {
        center: [f64; 2],
        radius: f64,
    },
    Ellipse {
        center: [f64; 2],
        #[serde(rename = "xAxis")]
        x_axis: [f64; 2],
        #[serde(rename = "semiMajorRadius")]
        semi_major_radius: f64,
        #[serde(rename = "semiMinorRadius")]
        semi_minor_radius: f64,
    },
    Edges {
        edges: Vec<EdgeValue>,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum EdgeValue {
    Line {
        start: [f64; 2],
        end: [f64; 2],
    },
    CircularArc {
        center: [f64; 2],
        radius: f64,
        #[serde(rename = "startParameter")]
        start_parameter: f64,
        #[serde(rename = "sweepParameter")]
        sweep_parameter: f64,
    },
    EllipticArc {
        center: [f64; 2],
        #[serde(rename = "xAxis")]
        x_axis: [f64; 2],
        #[serde(rename = "semiMajorRadius")]
        semi_major_radius: f64,
        #[serde(rename = "semiMinorRadius")]
        semi_minor_radius: f64,
        #[serde(rename = "startParameter")]
        start_parameter: f64,
        #[serde(rename = "sweepParameter")]
        sweep_parameter: f64,
    },
}
impl From<B> for BoundaryValue {
    fn from(b: B) -> Self {
        match b {
            B::Polyline { vertices, bulges } => Self::Polyline {
                vertices,
                bulges: Some(bulges),
            },
            B::Circle { center, radius } => Self::Circle { center, radius },
            B::Ellipse {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
            } => Self::Ellipse {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
            },
            B::Edges(edges) => Self::Edges {
                edges: edges.into_iter().map(EdgeValue::from).collect(),
            },
        }
    }
}
impl From<BoundaryValue> for B {
    fn from(b: BoundaryValue) -> Self {
        match b {
            BoundaryValue::Polyline { vertices, bulges } => {
                let bulges = bulges.unwrap_or_else(|| vec![0.0; vertices.len()]);
                Self::Polyline { vertices, bulges }
            }
            BoundaryValue::Circle { center, radius } => Self::Circle { center, radius },
            BoundaryValue::Ellipse {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
            } => Self::Ellipse {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
            },
            BoundaryValue::Edges { edges } => Self::Edges(edges.into_iter().map(E::from).collect()),
        }
    }
}
impl From<E> for EdgeValue {
    fn from(e: E) -> Self {
        match e {
            E::Line { start, end } => Self::Line { start, end },
            E::CircularArc {
                center,
                radius,
                start_parameter,
                sweep_parameter,
            } => Self::CircularArc {
                center,
                radius,
                start_parameter,
                sweep_parameter,
            },
            E::EllipticArc {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
            } => Self::EllipticArc {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
            },
        }
    }
}
impl From<EdgeValue> for E {
    fn from(e: EdgeValue) -> Self {
        match e {
            EdgeValue::Line { start, end } => Self::Line { start, end },
            EdgeValue::CircularArc {
                center,
                radius,
                start_parameter,
                sweep_parameter,
            } => Self::CircularArc {
                center,
                radius,
                start_parameter,
                sweep_parameter,
            },
            EdgeValue::EllipticArc {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
            } => Self::EllipticArc {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
            },
        }
    }
}
