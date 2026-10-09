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
    source: &opencadcodec::CadDocument,
    scope_id: u32,
    layer_id: u32,
    appearance: EntityAppearance,
    tolerance: f64,
    state: &mut super::geometry::ExchangeState<Handle>,
    partial: &mut Vec<CadToOcdrawLossReason>,
) -> Result<Option<crate::from_cad::prepared_entities::PreparedCadEntityValue>, CadToOcdrawError> {
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
                fill: p.fill,
            },
            handles: p.source_handles,
            associative: p.is_associative,
        },
    ))
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
        &h.fill,
    )
    .map_err(|e| OcdrawToCadError::Cad(format!("Hatch {}: {e}", h.id)))?;
    for loss in p.losses {
        diagnostics.push(crate::to_cad::diagnostic(
            "HATCH_PATTERN_METADATA",
            format!("/entities/{}/{}", h.id, loss.field),
            loss.detail,
        ));
    }
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
            format!("Hatch geometry deviation is at most {maximum} coordinate units"),
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
