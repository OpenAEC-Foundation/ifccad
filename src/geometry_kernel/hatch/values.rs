use num_rational::BigRational;

pub const DEFAULT_HATCH_JOIN_TOLERANCE: f64 = 1e-9;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HatchAreaRule {
    #[default]
    Normal,
    Outer,
    Ignore,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HatchFill {
    #[default]
    Solid,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HatchBoundary2 {
    Polyline {
        vertices: Vec<[f64; 2]>,
        bulges: Vec<f64>,
    },
    Circle {
        center: [f64; 2],
        radius: f64,
    },
    Ellipse {
        center: [f64; 2],
        x_axis: [f64; 2],
        semi_major_radius: f64,
        semi_minor_radius: f64,
    },
    Edges(Vec<HatchEdge2>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum HatchEdge2 {
    Line {
        start: [f64; 2],
        end: [f64; 2],
    },
    CircularArc {
        center: [f64; 2],
        radius: f64,
        start_parameter: f64,
        sweep_parameter: f64,
    },
    EllipticArc {
        center: [f64; 2],
        x_axis: [f64; 2],
        semi_major_radius: f64,
        semi_minor_radius: f64,
        start_parameter: f64,
        sweep_parameter: f64,
    },
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[error("Hatch loop {loop_index:?}, edge {edge_index:?}: {reason}")]
pub struct HatchValidationError {
    pub loop_index: Option<usize>,
    pub edge_index: Option<usize>,
    pub reason: HatchFailureReason,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum HatchFailureReason {
    #[error("invalid boundary shape or primitive parameters")]
    InvalidParameters,
    #[error("join tolerance must be finite and nonnegative")]
    InvalidTolerance,
    #[error("join distance is in [{lower}, {upper}], exceeding {limit}")]
    GapExceeded { lower: f64, upper: f64, limit: f64 },
    #[error("available numerical evidence cannot prove or disprove the join")]
    JoinProofIncomplete,
    #[error("geometry has no finite conservative enclosure")]
    EnclosureOutOfRange,
}

impl HatchValidationError {
    pub(super) fn at(
        loop_index: usize,
        edge_index: Option<usize>,
        reason: HatchFailureReason,
    ) -> Self {
        Self {
            loop_index: Some(loop_index),
            edge_index,
            reason,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Curve {
    pub center: [f64; 2],
    pub x_axis: [f64; 2],
    pub major: f64,
    pub minor: f64,
    pub start: f64,
    pub sweep: f64,
}

impl HatchEdge2 {
    pub(super) fn curve(&self) -> Option<Curve> {
        match *self {
            Self::Line { .. } => None,
            Self::CircularArc {
                center,
                radius,
                start_parameter,
                sweep_parameter,
            } => Some(Curve {
                center,
                x_axis: [1.0, 0.0],
                major: radius,
                minor: radius,
                start: start_parameter,
                sweep: sweep_parameter,
            }),
            Self::EllipticArc {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
            } => Some(Curve {
                center,
                x_axis,
                major: semi_major_radius,
                minor: semi_minor_radius,
                start: start_parameter,
                sweep: sweep_parameter,
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum EndpointKey {
    Point([f64; 2]),
    Curve {
        center: [f64; 2],
        x_axis: [f64; 2],
        major: f64,
        minor: f64,
        angle: BigRational,
    },
}
