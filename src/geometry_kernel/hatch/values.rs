use num_rational::BigRational;

pub const DEFAULT_HATCH_JOIN_TOLERANCE: f64 = 1e-9;

/// Authored loop construction hints; these do not certify current source geometry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HatchLoopProvenance {
    pub from_source_objects: bool,
    pub derived_from_pick_point: bool,
    pub outermost_hint: bool,
    pub text_box: bool,
}

impl HatchLoopProvenance {
    /// An empty authored record has no meaning and must be omitted.
    pub fn is_empty(&self) -> bool {
        !self.from_source_objects
            && !self.derived_from_pick_point
            && !self.outermost_hint
            && !self.text_box
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HatchAreaRule {
    #[default]
    Normal,
    Outer,
    Ignore,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum HatchFill {
    #[default]
    Solid,
    LinePattern(HatchLinePattern),
    Gradient,
}

/// Predefined CAD gradient intent; this value does not evaluate a color field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HatchGradientType {
    Linear,
    Cylinder,
    InverseCylinder,
    Spherical,
    InverseSpherical,
    Hemispherical,
    InverseHemispherical,
    Curved,
    InverseCurved,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HatchGradientColorMode {
    OneColor,
    #[default]
    TwoColor,
}

/// Stored authorship settings. Inactive settings may lack a type or color pair.
/// Endpoint colors are authoritative even in one-color mode; IO never derives
/// the second color from tint. Concrete colors belong to each drawing model.
#[derive(Clone, Debug, PartialEq)]
pub struct HatchGradient<C> {
    pub gradient_type: Option<HatchGradientType>,
    pub colors: Option<[C; 2]>,
    pub rotation: f64,
    pub shift: f64,
    pub color_mode: HatchGradientColorMode,
    pub tint: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HatchLinePattern {
    pub name: Option<String>,
    pub description: Option<String>,
    pub origin: [f64; 2],
    pub rotation: f64,
    pub scale: f64,
    pub families: Vec<HatchLineFamily>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct HatchLineFamily {
    pub angle: f64,
    pub base_point: [f64; 2],
    /// Along-direction and perpendicular offsets, in that order.
    pub offset: [f64; 2],
    pub dashes: Vec<HatchDash>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HatchDash {
    Dash { length: f64 },
    Gap { length: f64 },
    Dot,
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
