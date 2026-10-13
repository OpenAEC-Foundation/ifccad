//! Independent native marker references and occurrence appearance.
use super::*;
use crate::{line_pattern::*, text::TextExtentReason};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AppearanceContext {
    pub layer_id: Option<u64>,
    pub pattern_id: Option<IfccadLinePatternId>,
}
#[derive(Default)]
pub(crate) struct MarkerContribution {
    pub radius: Option<f64>,
    pub unavailable: bool,
    pub reasons: Vec<TextExtentReason>,
    pub bounds_reasons: Vec<LinePatternBoundsReason>,
}
pub(crate) fn effective_context(
    entity: &IfccadNativeEntity,
    parent: Option<AppearanceContext>,
    in_definition: bool,
    doc: &IfccadDocument,
) -> AppearanceContext {
    let zero = doc
        .layers
        .iter()
        .find(|l| l.id == entity.layer_id)
        .is_some_and(|l| l.name == "0");
    let layer_id = if in_definition && zero {
        parent.and_then(|p| p.layer_id)
    } else {
        Some(entity.layer_id)
    };
    let pattern_id = match entity.appearance.line_pattern {
        IfccadMode::Explicit(id) => Some(id),
        IfccadMode::ByBlock => parent.and_then(|p| p.pattern_id),
        IfccadMode::ByLayer => layer_id.and_then(|id| {
            doc.layers
                .iter()
                .find(|l| l.id == id)
                .map(|l| l.appearance.line_pattern)
        }),
    };
    AppearanceContext {
        layer_id,
        pattern_id,
    }
}
pub(crate) fn marker_contribution(
    id: Option<IfccadLinePatternId>,
    entity_scale: f64,
    doc: &IfccadDocument,
) -> Result<MarkerContribution, IfccadReport> {
    let mut result = MarkerContribution::default();
    let Some(id) = id else {
        if doc
            .line_patterns
            .iter()
            .any(|p| p.pattern.iter().any(|e| e.marker.is_some()))
        {
            result.unavailable = true;
            result
                .bounds_reasons
                .push(LinePatternBoundsReason::AppearanceContextUnavailable);
        }
        return Ok(result);
    };
    let pattern = doc
        .line_patterns
        .iter()
        .find(|p| p.id == id)
        .expect("validated pattern");
    let failure = |message: &str| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-PATTERN-003",
            &format!("/cad/d{}/linePattern/{}", doc.drawing_id, id.0),
            message,
        )
    };
    for element in &pattern.pattern {
        match &element.marker {
            None => {}
            Some(IfccadLinePatternMarker::Symbol { .. }) => {
                result.unavailable = true;
                result
                    .bounds_reasons
                    .push(LinePatternBoundsReason::SymbolFootprintUnavailable);
            }
            Some(IfccadLinePatternMarker::Text {
                text,
                text_style_id,
                transform,
            }) => {
                if text.is_empty() {
                    continue;
                }
                let scale = crate::geometry_kernel::numeric::Interval::point(entity_scale)
                    .mul(crate::geometry_kernel::numeric::Interval::point(
                        doc.line_pattern_scale,
                    ))
                    .ok_or_else(|| failure("pattern scale product exceeds finite range"))?
                    .upper;
                let style = doc
                    .text_styles
                    .iter()
                    .find(|s| s.id == *text_style_id)
                    .expect("validated style");
                let estimate =
                    estimate_line_pattern_text_extent(&ResolvedLinePatternTextExtentInput {
                        text,
                        fixed_style_height: style.properties.creation_height,
                        marker_scale: transform.scale,
                        pattern_scale: scale,
                        offset: transform.offset,
                    })
                    .map_err(|e| failure(&e.to_string()))?;
                if let Some(radius) = estimate.radius {
                    result.radius = Some(result.radius.unwrap_or(0.).max(radius));
                } else {
                    result.unavailable = true;
                }
                result.reasons.extend(estimate.reasons);
                result
                    .bounds_reasons
                    .push(LinePatternBoundsReason::TextEstimated);
            }
        }
    }
    result.bounds_reasons.dedup();
    result.reasons.dedup();
    Ok(result)
}
