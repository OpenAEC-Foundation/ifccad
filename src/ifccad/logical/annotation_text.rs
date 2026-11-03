//! Independently saved annotation presentations for existing text families.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTextAnnotationHorizontal {
    Left,
    Center,
    Right,
    Aligned,
    Middle,
    Fit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTextAnnotationContext {
    pub insertion: [f64; 2],
    pub alignment: [f64; 2],
    pub rotation: f64,
    pub horizontal: IfccadTextAnnotationHorizontal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadAnnotationColumnKind {
    Static,
    Dynamic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadMTextAnnotationColumns {
    pub kind: IfccadAnnotationColumnKind,
    pub count: u32,
    pub width: f64,
    pub gutter: f64,
    pub auto_height: bool,
    pub flow_reversed: bool,
    pub heights: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadMTextAnnotationContext {
    pub insertion: [f64; 3],
    pub x_direction: [f64; 3],
    pub attachment: crate::text::MTextAttachment,
    pub rectangle_width: f64,
    pub rectangle_height: f64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    pub columns: Option<IfccadMTextAnnotationColumns>,
}

fn invalid() -> IfccadReport {
    crate::ifccad::diagnostics::failure(
        "IFCCAD-ANNOTATION-005",
        "/ifccad::annotation/contexts",
        "invalid saved text presentation",
    )
}
impl IfccadTextAnnotationContext {
    pub(super) fn validate(&self) -> Result<(), IfccadReport> {
        if self
            .insertion
            .iter()
            .chain(&self.alignment)
            .chain([&self.rotation])
            .any(|v| !v.is_finite())
        {
            return Err(invalid());
        }
        Ok(())
    }
}
impl IfccadMTextAnnotationContext {
    pub(super) fn validate(&self) -> Result<(), IfccadReport> {
        if self
            .insertion
            .iter()
            .chain(&self.x_direction)
            .any(|v| !v.is_finite())
            || self.x_direction.iter().all(|v| *v == 0.)
            || [self.rectangle_width, self.rectangle_height]
                .iter()
                .any(|v| !v.is_finite() || *v < 0.)
        {
            return Err(invalid());
        }
        if let Some(c) = &self.columns {
            if [c.width, c.gutter]
                .iter()
                .chain(&c.heights)
                .any(|v| !v.is_finite() || *v < 0.)
                || (c.kind == IfccadAnnotationColumnKind::Dynamic
                    && !c.auto_height
                    && c.heights.len() != c.count as usize)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
