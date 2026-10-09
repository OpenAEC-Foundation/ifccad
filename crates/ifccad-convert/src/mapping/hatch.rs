use crate::*;
use ocdraw::ifccad::*;
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
            "hatch-editing-state",
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
        placement: super::geometry::placement(p.placement),
        loops: p
            .boundaries
            .into_iter()
            .map(|boundary| IfccadHatchLoop {
                boundary,
                source_entity_id: None,
            })
            .collect(),
        area_rule: p.area_rule,
        join_tolerance: p.join_tolerance,
        fill: p.fill,
    })))
}

fn continuous_user_linetype(h: &Hatch, source: &opencadcodec::CadDocument) -> bool {
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
) -> Result<Option<EntityType>, IfccadConversionError> {
    let b = h
        .loops
        .iter()
        .map(|l| l.boundary.clone())
        .collect::<Vec<_>>();
    let p = cad_geometry_convert::hatch::prepare_hatch_to_cad(
        h.placement
            .coordinate_frame()
            .map_err(IfccadConversionError::CoreValidation)?,
        &b,
        h.area_rule,
        h.join_tolerance,
        &h.fill,
    )
    .map_err(|source| IfccadConversionError::HatchPreparation {
        location: format!("entity/{id}"),
        source,
    })?;
    for loss in p.losses {
        issues.push(crate::diagnostics::diagnostic(
            "hatch-pattern-metadata",
            format!("entity/{id}/{}", loss.field),
            loss.detail,
        ));
    }
    context.record(
        id,
        IfccadGeometryEntitySource::NativeEntity {
            owner: context.owner,
            entity_id: id,
        },
        combine(p.pairs),
        issues,
    )?;
    if h.join_tolerance != ocdraw::geometry_kernel::hatch::DEFAULT_HATCH_JOIN_TOLERANCE {
        issues.push(crate::diagnostics::diagnostic(
            "hatch-join-policy",
            format!("entity/{id}/joinTolerance"),
            "CAD does not persist the native Hatch join tolerance",
        ));
    }
    Ok(Some(EntityType::Hatch(p.hatch)))
}
