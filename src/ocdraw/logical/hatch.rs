use super::EntityAppearance;
use crate::geometry_kernel::hatch::{HatchAreaRule, HatchBoundary2, HatchFill};
use crate::geometry_kernel::CoordinateFrame3;

/// Resolves a creation request to authored local units; never changes document state.
pub fn resolve_ocdraw_hatch_join_tolerance(
    doc: &super::OcdrawDocument,
    scope_id: u32,
    request: crate::geometry_kernel::hatch::HatchJoinToleranceRequest,
) -> Result<f64, super::OcdrawValidationError> {
    let scope = doc
        .scopes
        .iter()
        .find(|s| s.id == scope_id)
        .ok_or_else(|| {
            super::OcdrawValidationError::from_logical_errors(vec![
                super::field_validation::logical_error(
                    "HATCH_TOLERANCE",
                    format!("/scopes/{scope_id}"),
                    "Hatch owner scope does not exist",
                ),
            ])
        })?;
    let plot = doc
        .layouts
        .iter()
        .find(|l| l.scope_id == scope_id)
        .and_then(|l| l.settings.plot_settings.as_ref());
    resolve_context(
        &doc.unit,
        scope_id,
        scope.kind == super::DrawingScopeKind::Paper,
        plot,
        request,
    )
}

pub(crate) fn resolve_context(
    unit: &str,
    scope_id: u32,
    paper: bool,
    plot: Option<&crate::plot_kernel::PlotSettings>,
    request: crate::geometry_kernel::hatch::HatchJoinToleranceRequest,
) -> Result<f64, super::OcdrawValidationError> {
    use crate::geometry_kernel::{
        hatch::resolve_hatch_join_tolerance, numeric::exact, CoordinateLengthUnit,
    };
    use crate::plot_kernel::{PlotScale, PlotUnit};
    use num_rational::BigRational;
    let error = |message: &str| {
        super::OcdrawValidationError::from_logical_errors(vec![
            super::field_validation::logical_error(
                "HATCH_TOLERANCE",
                format!("/scopes/{scope_id}"),
                message,
            ),
        ])
    };
    if paper {
        if let Some(p) = plot {
            if let PlotScale::Fixed {
                output_length,
                scope_length,
            } = p.mapping.scale
            {
                if !output_length.is_finite()
                    || !scope_length.is_finite()
                    || output_length <= 0.
                    || scope_length <= 0.
                {
                    return Err(error("invalid known fixed plot scale"));
                }
            }
        }
    }
    let metres: Option<BigRational> = if paper {
        plot.and_then(|p| {
            if let PlotScale::Fixed {
                output_length,
                scope_length,
            } = p.mapping.scale
            {
                if !output_length.is_finite()
                    || !scope_length.is_finite()
                    || output_length <= 0.0
                    || scope_length <= 0.0
                {
                    return None;
                }
                let unit = match p.plot_unit {
                    PlotUnit::Millimetre => BigRational::new(1.into(), 1000.into()),
                    PlotUnit::Inch => BigRational::new(127.into(), 5000.into()),
                    PlotUnit::Pixel => return None,
                };
                Some(unit * exact(output_length) / exact(scope_length))
            } else {
                None
            }
        })
    } else {
        CoordinateLengthUnit::from_token(unit)
            .ok_or_else(|| error("invalid drawing coordinate unit"))?
            .coordinates_per_metre()
            .map(|(lower, _)| lower.recip())
    };
    resolve_hatch_join_tolerance(request, metres.as_ref()).map_err(|e| error(&e.to_string()))
}

#[derive(Clone, Debug, PartialEq)]
pub struct OcdrawHatchLoop {
    pub boundary: HatchBoundary2,
    pub source_entity_id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingHatchEntity {
    pub id: u64,
    pub layer_id: u32,
    pub visible: bool,
    pub appearance: EntityAppearance,
    pub placement: CoordinateFrame3,
    pub loops: Vec<OcdrawHatchLoop>,
    pub area_rule: HatchAreaRule,
    pub join_tolerance: f64,
    pub fill: HatchFill,
}
