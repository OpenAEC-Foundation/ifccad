use crate::*;
use ifccad::*;
use opencadcodec::{entities::Hatch, EntityType};
fn combine(pairs: Vec<cad_geometry_convert::GeometryPair>) -> cad_geometry_convert::GeometryPair {
    let mut p = cad_geometry_convert::GeometryPair {
        points: vec![],
        curves: vec![],
    };
    for q in pairs {
        p.points.extend(q.points);
        p.curves.extend(q.curves);
    }
    p
}
pub(crate) fn from_cad(
    h: &Hatch,
    source: &opencadcodec::CadDocument,
    tolerance: f64,
    context: &mut crate::geometry_context::GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Option<IfccadEntityKind>, IfccadConversionError> {
    let loc = format!("entity/{}", h.common.handle);
    let p = match cad_geometry_convert::hatch::prepare_hatch_from_cad_with_pattern_context(
        h,
        tolerance,
        continuous_user_linetype(h, source),
    ) {
        Ok(p) => p,
        Err(cad_geometry_convert::hatch::CadHatchPreparationError::Unsupported {
            field,
            detail,
        }) => {
            issues.push(crate::diagnostics::diagnostic(
                "hatch-unsupported",
                format!("{loc}/{field}"),
                detail,
            ));
            return Ok(None);
        }
        Err(source) => {
            return Err(IfccadConversionError::HatchPreparation {
                location: loc,
                source,
            })
        }
    };
    for loss in p.losses {
        issues.push(crate::diagnostics::diagnostic(
            editing_loss_code(loss.field),
            format!("{loc}/{}", loss.field),
            loss.detail,
        ));
    }
    context.record(
        h.common.handle.value(),
        IfccadGeometryEntitySource::CadEntity {
            handle: h.common.handle,
            kind: "HATCH".into(),
        },
        combine(p.pairs),
        issues,
    )?;
    Ok(Some(IfccadEntityKind::Hatch(IfccadHatch {
        classic_settings: p.classic_settings,
        boundary_construction: p.boundary_construction,
        association_enabled: p.is_associative,
        gradient: p.gradient.map(super::hatch_gradient::gradient_from_cad),
        placement: super::geometry::placement(p.placement),
        loops: p
            .boundaries
            .into_iter()
            .zip(p.loop_flags)
            .map(|(boundary, flags)| IfccadHatchLoop {
                provenance: flags.provenance,
                text_island: flags.text_island,
                boundary,
                source_entity_ids: vec![],
            })
            .collect(),
        area_rule: p.area_rule,
        join_tolerance: p.join_tolerance,
        fill: p.fill,
    })))
}

pub(crate) fn continuous_user_linetype(h: &Hatch, source: &opencadcodec::CadDocument) -> bool {
    let c = &h.common;
    let name = if c.linetype.is_empty() || c.linetype.eq_ignore_ascii_case("ByLayer") {
        // Layer 0 in a local block inherits the insertion layer, including
        // nested occurrences. A fixed local table lookup cannot qualify it.
        if c.layer == "0"
            && source
                .block_records
                .iter()
                .any(|b| b.handle == c.owner_handle && !b.is_model_space() && !b.is_paper_space())
        {
            return false;
        }
        let Some(layer) = source.layers.get(&c.layer) else {
            return false;
        };
        layer.line_type.as_str()
    } else {
        c.linetype.as_str()
    };
    if name.eq_ignore_ascii_case("ByBlock") || name.eq_ignore_ascii_case("ByLayer") {
        return false;
    }
    source.line_types.get(name).is_some_and(|p| {
        p.elements.is_empty()
            && p.pattern_length == 0.
            && !p.xref_dependent
            && p.xref_block_record_handle.is_null()
    })
}
pub(crate) fn to_cad(
    h: &IfccadHatch,
    id: u64,
    context: &mut crate::geometry_context::GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
    continuous_linetype: bool,
) -> Result<Option<EntityType>, IfccadConversionError> {
    let b = h
        .loops
        .iter()
        .map(|l| l.boundary.clone())
        .collect::<Vec<_>>();
    let (gradient, color_losses) = h
        .gradient
        .as_ref()
        .map(|g| {
            let (g, losses) = super::hatch_gradient::gradient_to_cad(g);
            (Some(g), losses)
        })
        .unwrap_or_default();
    let mut p = match cad_geometry_convert::hatch::prepare_hatch_to_cad(
        h.placement
            .coordinate_frame()
            .map_err(IfccadConversionError::CoreValidation)?,
        &b,
        h.area_rule,
        h.join_tolerance,
        &h.fill,
        gradient.as_ref(),
    ) {
        Ok(p) => p,
        Err(cad_geometry_convert::hatch::CadHatchPreparationError::Unsupported {
            field,
            detail,
        }) => {
            let code = if field == "gradient.frame" {
                "HATCH_GRADIENT_FRAME_UNSUPPORTED"
            } else {
                "hatch-unsupported"
            };
            issues.push(crate::diagnostics::diagnostic(
                code,
                format!("entity/{id}/{field}"),
                detail,
            ));
            return Ok(None);
        }
        Err(source) => {
            return Err(IfccadConversionError::HatchPreparation {
                location: format!("entity/{id}"),
                source,
            })
        }
    };
    let candidates = cad_geometry_convert::hatch::prepare_hatch_editing_candidates(
        p,
        h.placement
            .coordinate_frame()
            .map_err(IfccadConversionError::CoreValidation)?,
        &h.fill,
        h.classic_settings.as_ref(),
        h.boundary_construction.as_ref(),
        continuous_linetype,
    )
    .map_err(|source| IfccadConversionError::HatchPreparation {
        location: format!("entity/{id}"),
        source,
    })?;
    let mut pending =
        if let (Some(preferred), ifccad::geometry_kernel::hatch::HatchFill::LinePattern(_)) =
            (candidates.preferred.as_ref(), &h.fill)
        {
            Some(crate::to_cad::hatch_recipes::PendingHatchRecipe {
                entity_id: id,
                owner: context.owner,
                preferred: preferred.clone(),
                fallback: candidates.fallback.clone(),
            })
        } else {
            None
        };
    p = candidates.preferred.unwrap_or(candidates.fallback);
    let flags = h
        .loops
        .iter()
        .map(|l| cad_geometry_convert::hatch::PreparedHatchLoopFlags {
            provenance: l.provenance.clone(),
            text_island: l.text_island,
        })
        .collect::<Vec<_>>();
    cad_geometry_convert::hatch::apply_loop_flags_to_cad(&mut p.hatch, &flags).map_err(
        |source| IfccadConversionError::HatchPreparation {
            location: format!("entity/{id}"),
            source,
        },
    )?;
    if !p.losses.iter().any(|l| l.field == "gradient.frame") {
        for loss in color_losses {
            issues.push(crate::diagnostics::diagnostic(
                "HATCH_GRADIENT_COLOR",
                format!("entity/{id}/{}", loss.field),
                loss.detail,
            ));
        }
    }
    for loss in p.losses {
        issues.push(crate::diagnostics::diagnostic(
            if loss.field == "gradient.frame" {
                "HATCH_GRADIENT_FRAME_UNSUPPORTED"
            } else if loss.field.starts_with("classicSettings")
                || loss.field.starts_with("boundaryConstruction")
            {
                editing_loss_code(loss.field)
            } else {
                "hatch-pattern-metadata"
            },
            format!("entity/{id}/{}", loss.field),
            loss.detail,
        ));
    }
    if let Some(mut pending) = pending.take() {
        pending.preferred.hatch.paths = p.hatch.paths.clone();
        pending.fallback.hatch.paths = p.hatch.paths.clone();
        context.pending_hatch_recipes.push(pending);
    } else {
        context.record(
            id,
            IfccadGeometryEntitySource::NativeEntity {
                owner: context.owner,
                entity_id: id,
            },
            combine(p.pairs),
            issues,
        )?;
    }
    if h.join_tolerance != ifccad::geometry_kernel::hatch::DEFAULT_HATCH_JOIN_TOLERANCE {
        issues.push(crate::diagnostics::diagnostic(
            "hatch-join-policy",
            format!("entity/{id}/joinTolerance"),
            "CAD does not persist the native Hatch join tolerance",
        ));
    }
    Ok(Some(EntityType::Hatch(Box::new(p.hatch))))
}
fn editing_loss_code(field: &str) -> &'static str {
    if field.starts_with("classicSettings") {
        "hatch-classic-settings-loss"
    } else if field.starts_with("boundaryConstruction") {
        "hatch-boundary-construction-loss"
    } else {
        "hatch-editing-state"
    }
}
