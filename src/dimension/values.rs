use super::{DimensionFormattedFragment, DimensionPrimaryNumberSettings};
use crate::text::{MTextAttachment, TextRun};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionKind {
    Linear,
    Aligned,
    AngularTwoLines,
    AngularThreePoints,
    Radius,
    Diameter,
    Ordinate,
    ArcLength,
    JoggedRadius,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionTraversal {
    Counterclockwise,
    Clockwise,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionRaySign {
    Positive,
    Negative,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionOrdinateMode {
    Absolute,
    Signed,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DimensionArcSpan {
    Partial { radians: f64 },
    Full { traversal: DimensionTraversal },
}
impl DimensionArcSpan {
    pub fn radians(&self) -> f64 {
        match self {
            Self::Partial { radians } => *radians,
            Self::Full {
                traversal: DimensionTraversal::Counterclockwise,
            } => std::f64::consts::TAU,
            Self::Full { .. } => -std::f64::consts::TAU,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionArcLeader {
    pub start: [f64; 2],
    pub end: [f64; 2],
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum DimensionDefinition {
    Linear {
        first_point: [f64; 3],
        second_point: [f64; 3],
        direction_angle: f64,
        dimension_line_point: [f64; 2],
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::present_option"
        )]
        extension_direction_angle: Option<f64>,
    },
    Aligned {
        first_point: [f64; 3],
        second_point: [f64; 3],
        dimension_line_point: [f64; 2],
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::present_option"
        )]
        extension_direction_angle: Option<f64>,
    },
    AngularTwoLines {
        line1_start: [f64; 3],
        line1_end: [f64; 3],
        line2_start: [f64; 3],
        line2_end: [f64; 3],
        ray1_sign: DimensionRaySign,
        ray2_sign: DimensionRaySign,
        traversal: DimensionTraversal,
        dimension_arc_point: [f64; 2],
    },
    AngularThreePoints {
        vertex: [f64; 3],
        first_point: [f64; 3],
        second_point: [f64; 3],
        traversal: DimensionTraversal,
        dimension_arc_point: [f64; 2],
    },
    Radius {
        center: [f64; 3],
        curve_point: [f64; 3],
        leader_length: f64,
    },
    Diameter {
        first_diameter_point: [f64; 3],
        second_diameter_point: [f64; 3],
        leader_length: f64,
    },
    Ordinate {
        origin: [f64; 3],
        feature_point: [f64; 3],
        axis_angle: f64,
        leader_end: [f64; 2],
        value_mode: DimensionOrdinateMode,
    },
    ArcLength {
        center: [f64; 3],
        radius: f64,
        start_parameter: f64,
        sweep: DimensionArcSpan,
        dimension_arc_point: [f64; 2],
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::present_option"
        )]
        leader: Option<DimensionArcLeader>,
    },
    JoggedRadius {
        center: [f64; 3],
        curve_point: [f64; 3],
        display_center: [f64; 2],
        jog_point: [f64; 2],
        jog_angle: f64,
    },
}
impl DimensionDefinition {
    pub fn kind(&self) -> DimensionKind {
        match self {
            Self::Linear { .. } => DimensionKind::Linear,
            Self::Aligned { .. } => DimensionKind::Aligned,
            Self::AngularTwoLines { .. } => DimensionKind::AngularTwoLines,
            Self::AngularThreePoints { .. } => DimensionKind::AngularThreePoints,
            Self::Radius { .. } => DimensionKind::Radius,
            Self::Diameter { .. } => DimensionKind::Diameter,
            Self::Ordinate { .. } => DimensionKind::Ordinate,
            Self::ArcLength { .. } => DimensionKind::ArcLength,
            Self::JoggedRadius { .. } => DimensionKind::JoggedRadius,
        }
    }
    /// Convenience projection of the first two role-ordered anchors. This is
    /// the complete pair for linear/aligned definitions, not a complete
    /// definition for every kind. Use `measurement_anchors` for all roles.
    pub fn reference_points(&self) -> ([f64; 3], [f64; 3]) {
        let a = self.measurement_anchors();
        (a[0].1, a[1].1)
    }
    pub fn projected_points(&self) -> ([f64; 2], [f64; 2]) {
        let (a, b) = self.reference_points();
        ([a[0], a[1]], [b[0], b[1]])
    }
    pub fn measurement_anchors(&self) -> Vec<(DimensionRole, [f64; 3])> {
        use DimensionRole as R;
        match self {
            Self::Linear {
                first_point,
                second_point,
                ..
            }
            | Self::Aligned {
                first_point,
                second_point,
                ..
            } => vec![
                (R::FirstPoint, *first_point),
                (R::SecondPoint, *second_point),
            ],
            Self::Radius {
                center,
                curve_point,
                ..
            }
            | Self::JoggedRadius {
                center,
                curve_point,
                ..
            } => vec![(R::Center, *center), (R::CurvePoint, *curve_point)],
            Self::Diameter {
                first_diameter_point,
                second_diameter_point,
                ..
            } => vec![
                (R::FirstDiameterPoint, *first_diameter_point),
                (R::SecondDiameterPoint, *second_diameter_point),
            ],
            Self::AngularTwoLines {
                line1_start,
                line1_end,
                line2_start,
                line2_end,
                ..
            } => vec![
                (R::Line1Start, *line1_start),
                (R::Line1End, *line1_end),
                (R::Line2Start, *line2_start),
                (R::Line2End, *line2_end),
            ],
            Self::AngularThreePoints {
                vertex,
                first_point,
                second_point,
                ..
            } => vec![
                (R::Vertex, *vertex),
                (R::FirstLegPoint, *first_point),
                (R::SecondLegPoint, *second_point),
            ],
            Self::Ordinate {
                origin,
                feature_point,
                ..
            } => vec![(R::Origin, *origin), (R::FeaturePoint, *feature_point)],
            Self::ArcLength {
                center,
                radius,
                start_parameter,
                sweep,
                ..
            } => {
                let point = |angle: f64| {
                    let (s, c) = fpmath::sin_cos(angle);
                    [center[0] + radius * c, center[1] + radius * s, center[2]]
                };
                vec![
                    (R::Center, *center),
                    (R::ArcStart, point(*start_parameter)),
                    (R::ArcEnd, point(*start_parameter + sweep.radians())),
                ]
            }
        }
    }
    pub fn accepts_role(&self, role: DimensionRole) -> bool {
        self.measurement_anchors().iter().any(|(r, _)| *r == role)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionRole {
    FirstPoint,
    SecondPoint,
    Center,
    CurvePoint,
    FirstDiameterPoint,
    SecondDiameterPoint,
    Line1Start,
    Line1End,
    Line2Start,
    Line2End,
    Vertex,
    FirstLegPoint,
    SecondLegPoint,
    Origin,
    FeaturePoint,
    ArcStart,
    ArcEnd,
}
impl DimensionRole {
    pub(crate) fn field_name(self) -> &'static str {
        match self {
            Self::FirstPoint => "firstPoint",
            Self::SecondPoint => "secondPoint",
            Self::Center => "center",
            Self::CurvePoint => "curvePoint",
            Self::FirstDiameterPoint => "firstDiameterPoint",
            Self::SecondDiameterPoint => "secondDiameterPoint",
            Self::Line1Start => "line1Start",
            Self::Line1End => "line1End",
            Self::Line2Start => "line2Start",
            Self::Line2End => "line2End",
            Self::Vertex => "vertex",
            Self::FirstLegPoint => "firstLegPoint",
            Self::SecondLegPoint => "secondLegPoint",
            Self::Origin => "origin",
            Self::FeaturePoint => "featurePoint",
            Self::ArcStart => "arcStart",
            Self::ArcEnd => "arcEnd",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DimensionPointSelector {
    Position,
    Start,
    End,
    Center,
    Vertex { index: u32 },
    CirclePoint { angle: f64 },
    ArcPoint { fraction: f64 },
    SegmentPoint { index: u32, fraction: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[derive(Default)]
pub enum DimensionTextPlacement {
    #[default]
    StyleDriven,
    /// Local anchor; rotation offsets the default dimension text orientation.
    Explicit {
        position: [f64; 2],
        rotation: f64,
        attachment: MTextAttachment,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "parts",
    rename_all = "camelCase",
    deny_unknown_fields
)]
#[derive(Default)]
pub enum DimensionLabel {
    #[default]
    Generated,
    Suppressed,
    Template(Vec<DimensionLabelPart>),
}
impl DimensionLabel {
    pub fn template(parts: Vec<DimensionLabelPart>) -> Self {
        Self::Template(parts)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "runs",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DimensionLabelPart {
    Literal(Vec<TextRun>),
    Measurement,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DimensionMeasurement {
    Distance(f64),
    Angle(f64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionFactorApplication {
    AllOwners,
    PaperOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionDistanceFormat {
    Decimal,
    Fractional,
    Engineering,
    Architectural,
    Scientific,
}
/// Presentation of a numeric fraction; independent of precision and units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionFractionLayout {
    #[default]
    Inline,
    Horizontal,
    Diagonal,
}
pub(crate) fn default_fraction_scale() -> f64 {
    1.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionAngleFormat {
    DecimalDegrees,
    DegreesMinutesSeconds,
    Grads,
    Radians,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionArcSymbol {
    None,
    Before,
    Above,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DimensionZeroSuppression {
    pub leading: bool,
    pub trailing: bool,
    pub zero_feet: bool,
    pub zero_inches: bool,
}

impl Default for DimensionPrimaryNumberSettings {
    fn default() -> Self {
        Self {
            measurement_factor: 1.0,
            application: DimensionFactorApplication::AllOwners,
            format: DimensionDistanceFormat::Decimal,
            angle_format: DimensionAngleFormat::DecimalDegrees,
            precision: 2,
            decimal_separator: '.',
            rounding_increment: 0.0,
            zero_suppression: Default::default(),
            prefix: String::new(),
            suffix: String::new(),
            arc_symbol: DimensionArcSymbol::Before,
            fraction_layout: DimensionFractionLayout::Inline,
            fraction_scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DimensionAlternateNumberSettings {
    #[serde(default)]
    pub fraction_layout: DimensionFractionLayout,
    #[serde(default = "default_fraction_scale")]
    pub fraction_scale: f64,
    pub factor: f64,
    pub format: DimensionDistanceFormat,
    pub precision: u8,
    pub decimal_separator: char,
    pub rounding_increment: f64,
    pub zero_suppression: DimensionZeroSuppression,
    pub prefix: String,
    pub suffix: String,
}
impl Default for DimensionAlternateNumberSettings {
    fn default() -> Self {
        Self {
            fraction_layout: DimensionFractionLayout::Inline,
            fraction_scale: 1.0,
            factor: 1.0,
            format: DimensionDistanceFormat::Decimal,
            precision: 2,
            decimal_separator: '.',
            rounding_increment: 0.0,
            zero_suppression: Default::default(),
            prefix: String::new(),
            suffix: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[derive(Default)]
pub enum DimensionToleranceSettings {
    #[default]
    None,
    Deviations {
        upper: f64,
        lower: f64,
        precision: u8,
    },
    Limits {
        upper: f64,
        lower: f64,
        precision: u8,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DimensionNumberSettings {
    pub primary: DimensionPrimaryNumberSettings,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::present_option"
    )]
    pub alternate: Option<DimensionAlternateNumberSettings>,
    pub tolerance: DimensionToleranceSettings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DimensionFormatContext {
    pub is_paper: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DimensionFormattedLabelState {
    Visible,
    Suppressed,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DimensionFormattedLabel {
    pub state: DimensionFormattedLabelState,
    /// Structured presentation; `runs` remains its plain text projection.
    pub fragments: Vec<DimensionFormattedFragment>,
    pub runs: Vec<TextRun>,
    /// Numeric display text; empty when the authored label needs no measurement.
    pub primary: String,
    pub alternate: Option<String>,
}
