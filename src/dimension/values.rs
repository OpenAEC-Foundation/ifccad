use super::DimensionPrimaryNumberSettings;
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum DimensionDefinition {
    Linear {
        first_point: [f64; 2],
        second_point: [f64; 2],
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
        first_point: [f64; 2],
        second_point: [f64; 2],
        dimension_line_point: [f64; 2],
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::present_option"
        )]
        extension_direction_angle: Option<f64>,
    },
}
impl DimensionDefinition {
    pub fn kind(&self) -> DimensionKind {
        match self {
            Self::Linear { .. } => DimensionKind::Linear,
            Self::Aligned { .. } => DimensionKind::Aligned,
        }
    }
    pub fn points(&self) -> ([f64; 2], [f64; 2]) {
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
            } => (*first_point, *second_point),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionRole {
    FirstPoint,
    SecondPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DimensionPointSelector {
    Position,
    Start,
    End,
    Center,
    Vertex { index: u32 },
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
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DimensionAlternateNumberSettings {
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
    pub runs: Vec<TextRun>,
    /// Numeric display text; empty when the authored label needs no measurement.
    pub primary: String,
    pub alternate: Option<String>,
}
