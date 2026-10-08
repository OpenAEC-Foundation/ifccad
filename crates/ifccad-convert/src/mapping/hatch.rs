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
    tolerance: f64,
    context: &mut crate::geometry_context::GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Option<IfccadEntityKind>, IfccadConversionError> {
    let loc = format!("entity/{}", h.common.handle);
    let p = match cad_geometry_convert::hatch::prepare_hatch_from_cad(h, tolerance) {
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
        fill: ocdraw::geometry_kernel::hatch::HatchFill::Solid,
    })))
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
    )
    .map_err(|source| IfccadConversionError::HatchPreparation {
        location: format!("entity/{id}"),
        source,
    })?;
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
