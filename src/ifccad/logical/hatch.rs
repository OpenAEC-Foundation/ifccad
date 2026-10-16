use super::*;
use crate::geometry_kernel::hatch::{
    validate_hatch_boundaries, HatchAreaRule, HatchBoundary2, HatchFill,
};

/// Resolves only the selected coordinate domain; it does not edit stored limits.
pub fn resolve_ifccad_hatch_join_tolerance(
    d: &IfccadDocument,
    owner: IfccadScopeId,
    request: crate::geometry_kernel::hatch::HatchJoinToleranceRequest,
) -> Result<f64, IfccadReport> {
    use crate::geometry_kernel::{
        hatch::resolve_hatch_join_tolerance, numeric::exact, CoordinateLengthUnit,
    };
    use crate::plot_kernel::{PlotScale, PlotUnit};
    use num_rational::BigRational;
    let error = |m: &str| IfccadReport::one(format!("{owner:?}: Hatch tolerance: {m}"));
    let settings = match owner {
        IfccadScopeId::Layout(id) if id == d.model.id => None,
        IfccadScopeId::Layout(id) => Some(
            &d.paper_layouts
                .iter()
                .find(|p| p.id == id)
                .ok_or_else(|| error("owner layout missing"))?
                .settings,
        ),
        IfccadScopeId::BlockDefinition(id) => {
            if !d.blocks.iter().any(|b| b.id == id) {
                return Err(error("owner definition missing"));
            }
            None
        }
    };
    if let Some(p) = settings.and_then(|s| s.plot_settings.as_ref()) {
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
                return Err(error("invalid known fixed plot scale"));
            }
        }
    }
    let metres: Option<BigRational> = if let Some(settings) = settings {
        settings.plot_settings.as_ref().and_then(|p| {
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
        CoordinateLengthUnit::from_token(&d.length_unit)
            .ok_or_else(|| error("invalid drawing unit"))?
            .coordinates_per_metre()
            .map(|(lower, _)| lower.recip())
    };
    resolve_hatch_join_tolerance(request, metres.as_ref()).map_err(|e| error(&e.to_string()))
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadHatchLoop {
    pub boundary: HatchBoundary2,
    pub source_entity_ids: Vec<u64>,
    pub provenance: Option<crate::geometry_kernel::hatch::HatchLoopProvenance>,
    /// Special text-loop fill classification, independent of provenance and sources.
    pub text_island: bool,
}
pub type IfccadHatchGradient = crate::geometry_kernel::hatch::HatchGradient<IfccadColor>;

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadHatch {
    pub association_enabled: bool,
    pub gradient: Option<IfccadHatchGradient>,
    pub placement: IfccadPlacement,
    pub loops: Vec<IfccadHatchLoop>,
    pub area_rule: HatchAreaRule,
    pub join_tolerance: f64,
    pub fill: HatchFill,
}
pub(crate) fn validate_hatch(h: &IfccadHatch, path: &str) -> Result<(), IfccadReport> {
    for (i, l) in h.loops.iter().enumerate() {
        if l.provenance.as_ref().is_some_and(|p| p.is_empty()) {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-HATCH-001",
                &format!("{path}/loops/{i}/provenance"),
                "empty provenance must be omitted",
            ));
        }
    }
    h.placement
        .coordinate_frame()
        .map_err(|e| crate::ifccad::diagnostics::context(e, "IFCCAD-HATCH-001", path))?;
    crate::geometry_kernel::hatch::validate_hatch_fill(&h.fill).map_err(|e| {
        crate::ifccad::diagnostics::failure("IFCCAD-HATCH-001", path, e.to_string())
    })?;
    crate::geometry_kernel::hatch::validate_hatch_gradient(
        matches!(h.fill, HatchFill::Gradient),
        h.gradient.as_ref(),
        |color| color.concrete_rgb().is_some() && color.is_valid(),
    )
    .map_err(|e| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-HATCH-005",
            &format!("{path}/gradient/{}", e.field),
            e.to_string(),
        )
    })?;
    validate_hatch_boundaries(h.loops.iter().map(|l| &l.boundary), h.join_tolerance)
        .map_err(|e| crate::ifccad::diagnostics::failure("IFCCAD-HATCH-003", path, e.to_string()))
}
fn owned(d: &IfccadDocument) -> impl Iterator<Item = (IfccadScopeId, &IfccadEntity)> {
    d.model
        .entities
        .iter()
        .map(|e| (IfccadScopeId::Layout(d.model.id), e))
        .chain(d.paper_layouts.iter().flat_map(|p| {
            p.entities
                .iter()
                .map(move |e| (IfccadScopeId::Layout(p.id), e))
        }))
        .chain(d.blocks.iter().flat_map(|b| {
            b.entities
                .iter()
                .map(move |e| (IfccadScopeId::BlockDefinition(b.id), e))
        }))
}
pub(crate) fn validate_hatch_sources(d: &IfccadDocument) -> Result<(), IfccadReport> {
    let entities: std::collections::BTreeMap<_, _> =
        owned(d).map(|(owner, e)| (e.id(), (owner, e))).collect();
    for (owner, e) in owned(d) {
        let Some(e) = e.as_native() else { continue };
        let IfccadEntityKind::Hatch(h) = &e.kind else {
            continue;
        };
        for (i, l) in h.loops.iter().enumerate() {
            for (j, id) in l.source_entity_ids.iter().enumerate() {
                let valid = entities.get(id).is_some_and(|(target_owner, target)| {
                    *target_owner == owner
                        && target
                            .as_native()
                            .is_some_and(|target| target.kind.is_hatch_boundary_source())
                });
                if !valid {
                    return Err(crate::ifccad::diagnostics::failure(
                        "IFCCAD-HATCH-002",
                        &format!("/cad/d{}/e{}/loops/{i}/sources/{j}", d.drawing_id, e.id),
                        "missing or unsupported same-owner boundary source entity",
                    ));
                }
            }
        }
    }
    Ok(())
}
