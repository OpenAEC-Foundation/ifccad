//! Fontless authored-text estimates and independent producer bounds claims.
use super::*;
use crate::text::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OcdrawBoundsQuality {
    Enclosing,
    Estimated,
    /// Derived incomplete coverage; never a stored box quality.
    Partial,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OcdrawScopeBoundsAssessment {
    pub scope_id: u32,
    pub bounds: Option<crate::ocdraw::Bounds3d>,
    pub quality: Option<OcdrawBoundsQuality>,
    /// Independent enclosure evidence, not the stored producer declaration.
    pub enclosure_verified: bool,
    pub text_reasons: Vec<TextExtentReason>,
}

pub(crate) fn text_estimates(
    doc: &OcdrawDocument,
) -> Result<BTreeMap<u64, TextExtentEstimate>, OcdrawValidationError> {
    let styles = doc
        .text_styles
        .iter()
        .map(|s| (s.id, &s.properties))
        .collect::<BTreeMap<_, _>>();
    let failure = |id, error: TextExtentError| {
        OcdrawValidationError::from_logical_errors(vec![super::field_validation::logical_error(
            "TEXT_EXTENT",
            format!("/entities/{id}"),
            &error.to_string(),
        )])
    };
    let mut results = BTreeMap::new();
    for t in &doc.text_entities {
        let Some(style) = styles.get(&t.style_id) else {
            continue;
        };
        let extent = estimate_text_extent(&ResolvedTextExtentInput {
            content: &t.content,
            layout: t.layout,
            placement: t.placement,
            rotation: t.rotation,
            backward: t.backward,
            upside_down: t.upside_down,
            oblique_angle: t.oblique_angle,
            thickness: t.thickness,
            vertical: style.vertical,
        })
        .map_err(|e| failure(t.id, e))?;
        results.insert(t.id, extent);
    }
    for t in &doc.mtext_entities {
        let Some(style) = styles.get(&t.style_id) else {
            continue;
        };
        let extent = estimate_mtext_extent(&ResolvedMTextExtentInput {
            content: &t.content,
            style,
            character_format: &t.character_format,
            paragraph_format: &t.paragraph_format,
            height: t.height,
            attachment: t.attachment,
            flow: t.flow,
            wrap_width: t.wrap_width,
            columns: t.columns.as_ref(),
            background: t.background.as_ref(),
            frame_stroke_width: frame_stroke_width(doc, t).map_err(|e| failure(t.id, e))?,
            placement: t.placement,
            rotation: t.rotation,
            backward: t.backward,
            upside_down: t.upside_down,
        })
        .map_err(|e| failure(t.id, e))?;
        results.insert(t.id, extent);
    }
    Ok(results)
}

fn frame_stroke_width(
    doc: &OcdrawDocument,
    t: &DrawingMTextEntity,
) -> Result<Option<f64>, TextExtentError> {
    if !t.background.as_ref().is_some_and(|b| b.frame) {
        return Ok(None);
    }
    let weight = match t.appearance.line_weight {
        AppearanceSelection::Explicit(v) => Some(v),
        AppearanceSelection::ByLayer => doc
            .layers
            .iter()
            .find(|l| l.id == t.layer_id)
            .map(|l| l.line_weight),
        AppearanceSelection::ByBlock => None,
    };
    let Some(weight) = weight else {
        return Ok(None);
    };
    if weight == 0. {
        return Ok(Some(0.));
    }
    use num_rational::BigRational as Q;
    use num_traits::ToPrimitive;
    let q = |n: i64, d: i64| Q::new(n.into(), d.into());
    let owner = doc.scopes.iter().find(|s| s.entities.contains(&t.id));
    let scale = if owner.is_some_and(|s| s.kind == DrawingScopeKind::Paper) {
        let plot = owner
            .and_then(|s| doc.layouts.iter().find(|l| l.scope_id == s.id))
            .and_then(|l| l.settings.plot_settings.as_ref());
        let Some(plot) = plot else {
            return Ok(None);
        };
        let crate::plot_kernel::PlotScale::Fixed {
            output_length,
            scope_length,
        } = plot.mapping.scale
        else {
            return Ok(None);
        };
        if !output_length.is_finite()
            || output_length <= 0.
            || !scope_length.is_finite()
            || scope_length <= 0.
        {
            return Err(TextExtentError::OutOfRange);
        }
        let metres = match plot.plot_unit {
            crate::plot_kernel::PlotUnit::Millimetre => q(1, 1000),
            crate::plot_kernel::PlotUnit::Inch => q(127, 5000),
            crate::plot_kernel::PlotUnit::Pixel => return Ok(None),
        };
        Q::from_float(scope_length).ok_or(TextExtentError::OutOfRange)?
            / (metres * Q::from_float(output_length).ok_or(TextExtentError::OutOfRange)?)
    } else {
        let Some((_, upper)) = crate::geometry_kernel::CoordinateLengthUnit::from_token(&doc.unit)
            .and_then(|u| u.coordinates_per_metre())
        else {
            return Ok(None);
        };
        upper
    };
    let exact = Q::from_float(weight).ok_or(TextExtentError::OutOfRange)? * q(1, 1000) * scale;
    let mut out = exact
        .to_f64()
        .filter(|v| v.is_finite())
        .ok_or(TextExtentError::OutOfRange)?;
    if Q::from_float(out).ok_or(TextExtentError::OutOfRange)? < exact {
        out = out.next_up();
    }
    if !out.is_finite() {
        return Err(TextExtentError::OutOfRange);
    }
    Ok(Some(out))
}

pub(crate) fn validate_bounds_quality(doc: &OcdrawDocument) -> Vec<LogicalError> {
    doc.scopes
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            s.bounds_quality.is_some()
                && (s.bounds.is_none() || s.bounds_quality == Some(OcdrawBoundsQuality::Partial))
        })
        .map(|(i, _)| {
            super::field_validation::logical_error(
                "BOUNDS_QUALITY",
                format!("/scopes/{i}/boundsQuality"),
                "stored quality requires a box and must be enclosing or estimated",
            )
        })
        .collect()
}
