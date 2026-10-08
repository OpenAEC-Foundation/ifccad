use crate::*;
use ocdraw::ocdraw::*;
use opencadcodec::{entities::Hatch, EntityType, Handle};

fn combine(pairs: Vec<cad_geometry_convert::GeometryPair>) -> cad_geometry_convert::GeometryPair {
    let mut out = cad_geometry_convert::GeometryPair {
        points: vec![],
        curves: vec![],
    };
    for p in pairs {
        out.points.extend(p.points);
        out.curves.extend(p.curves);
    }
    out
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn from_cad(
    h: &Hatch,
    scope_id: u32,
    layer_id: u32,
    appearance: EntityAppearance,
    tolerance: f64,
    state: &mut super::geometry::ExchangeState<Handle>,
    partial: &mut Vec<CadToOcdrawLossReason>,
) -> Result<Option<crate::from_cad::prepared_entities::PreparedCadEntityValue>, CadToOcdrawError> {
    let p = match cad_geometry_convert::hatch::prepare_hatch_from_cad(h, tolerance) {
        Ok(p) => p,
        Err(cad_geometry_convert::hatch::CadHatchPreparationError::Unsupported {
            field,
            detail,
        }) => {
            partial.push(CadToOcdrawLossReason::UnsupportedSemantic {
                name: format!("Hatch {field}: {detail}"),
            });
            return Ok(None);
        }
        Err(source) => {
            return Err(CadToOcdrawError::HatchPreparation {
                handle: h.common.handle,
                source,
            })
        }
    };
    partial.extend(
        p.losses
            .into_iter()
            .map(|l| CadToOcdrawLossReason::UnsupportedSemantic {
                name: format!("Hatch {}: {}", l.field, l.detail),
            }),
    );
    let maximum = state
        .record_geometry(
            h.common.handle,
            OcdrawGeometryEntitySource::CadEntity {
                handle: h.common.handle,
                kind: "HATCH".into(),
            },
            combine(p.pairs),
        )
        .map_err(CadToOcdrawError::Geometry)?;
    if maximum > 0. {
        partial.push(CadToOcdrawLossReason::GeometryRoundedWithinTolerance {
            max_deviation_upper_bound: maximum,
        });
    }
    Ok(Some(
        crate::from_cad::prepared_entities::PreparedCadEntityValue::Hatch {
            definition: HatchEntityDefinition {
                scope_id,
                layer_id,
                visible: !h.common.invisible,
                appearance,
                placement: p.placement,
                loops: p
                    .boundaries
                    .into_iter()
                    .map(|boundary| OcdrawHatchLoop {
                        boundary,
                        source_entity_id: None,
                    })
                    .collect(),
                area_rule: p.area_rule,
                join_tolerance: p.join_tolerance,
                fill: ocdraw::geometry_kernel::hatch::HatchFill::Solid,
            },
            handles: p.source_handles,
            associative: p.is_associative,
        },
    ))
}
pub(crate) fn to_cad(
    h: &DrawingHatchEntity,
    owner: u32,
    state: &mut super::geometry::ExchangeState<u64>,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) -> Result<EntityType, OcdrawToCadError> {
    let boundaries = h
        .loops
        .iter()
        .map(|l| l.boundary.clone())
        .collect::<Vec<_>>();
    let p = cad_geometry_convert::hatch::prepare_hatch_to_cad(
        h.placement,
        &boundaries,
        h.area_rule,
        h.join_tolerance,
    )
    .map_err(|e| OcdrawToCadError::Cad(format!("Hatch {}: {e}", h.id)))?;
    let maximum = state
        .record_geometry(
            h.id,
            OcdrawGeometryEntitySource::DrawingEntity {
                scope_id: owner,
                entity_id: h.id,
            },
            combine(p.pairs),
        )
        .map_err(OcdrawToCadError::Geometry)?;
    if maximum > 0. {
        diagnostics.push(crate::to_cad::diagnostic(
            "GEOMETRY_ROUNDED_WITHIN_TOLERANCE",
            format!("/entities/{}", h.id),
            format!("Hatch contour deviation is at most {maximum} coordinate units"),
        ));
    }
    if h.join_tolerance != ocdraw::geometry_kernel::hatch::DEFAULT_HATCH_JOIN_TOLERANCE {
        diagnostics.push(crate::to_cad::diagnostic(
            "HATCH_JOIN_POLICY",
            format!("/entities/{}/joinTolerance", h.id),
            "CAD does not persist the native Hatch join tolerance",
        ));
    }
    Ok(EntityType::Hatch(p.hatch))
}
