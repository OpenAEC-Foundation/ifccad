//! Independently stored Dimension context geometry and layout hints.
use super::*;
use crate::dimension::DimensionKind;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadStoredBoolOverride {
    pub active: bool,
    pub value: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadDimensionContextLayout {
    pub line_outside: IfccadStoredBoolOverride,
    pub suppress_arrows_outside: IfccadStoredBoolOverride,
    pub fit_switch: IfccadStoredBoolOverride,
    pub force_text_inside: IfccadStoredBoolOverride,
    pub text_movement_switch: IfccadStoredBoolOverride,
    pub second_arrow_present: bool,
    pub flip_first: bool,
    pub flip_second: bool,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum IfccadDimensionContextGeometry {
    Linear {
        dimension_line_point: [f64; 3],
    },
    Aligned {
        dimension_line_point: [f64; 3],
    },
    AngularTwoLines {
        dimension_arc_point: [f64; 3],
    },
    AngularThreePoints {
        dimension_arc_point: [f64; 3],
    },
    Radius {
        curve_point: [f64; 3],
    },
    Diameter {
        first_diameter_point: [f64; 3],
        second_diameter_point: [f64; 3],
    },
    Ordinate {
        feature_point: [f64; 3],
        leader_end: [f64; 3],
    },
    ArcLength {
        dimension_arc_point: [f64; 3],
    },
    JoggedRadius {
        display_center: [f64; 3],
        jog_point: [f64; 3],
    },
}
impl IfccadDimensionContextGeometry {
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
    pub(crate) fn points(&self) -> impl Iterator<Item = [f64; 3]> + '_ {
        let (a, b) = match self {
            Self::Linear {
                dimension_line_point,
            }
            | Self::Aligned {
                dimension_line_point,
            } => (dimension_line_point, None),
            Self::AngularTwoLines {
                dimension_arc_point,
            }
            | Self::AngularThreePoints {
                dimension_arc_point,
            }
            | Self::ArcLength {
                dimension_arc_point,
            } => (dimension_arc_point, None),
            Self::Radius { curve_point } => (curve_point, None),
            Self::Diameter {
                first_diameter_point,
                second_diameter_point,
            } => (first_diameter_point, Some(second_diameter_point)),
            Self::Ordinate {
                feature_point,
                leader_end,
            } => (feature_point, Some(leader_end)),
            Self::JoggedRadius {
                display_center,
                jog_point,
            } => (display_center, Some(jog_point)),
        };
        std::iter::once(*a).chain(b.copied())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadDimensionAnnotationContext {
    pub text_location: [f64; 2],
    pub user_text_location: bool,
    pub text_rotation: f64,
    pub layout: IfccadDimensionContextLayout,
    pub geometry: IfccadDimensionContextGeometry,
    pub graphics: Option<IfccadDimensionGraphics>,
}
pub(super) fn validate(
    d: &IfccadDocument,
    v: &IfccadDimension,
    path: &str,
) -> Result<(), IfccadReport> {
    let Some(a) = &v.annotation else {
        return Ok(());
    };
    a.validate_memberships(&d.annotation_scales, true)?;
    for (i, p) in a
        .contexts
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.presentation.as_ref().map(|p| (i, p)))
    {
        if p.geometry.kind() != v.definition.kind()
            || p.geometry
                .points()
                .flatten()
                .chain(p.text_location)
                .chain([p.text_rotation])
                .any(|n| !n.is_finite())
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-ANNOTATION-006",
                &format!("{path}/ifccad::annotation/contexts/{i}/presentation"),
                "invalid or kind-mismatched saved Dimension presentation",
            ));
        }
    }
    Ok(())
}
