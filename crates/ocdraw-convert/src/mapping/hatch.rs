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
            definition: Box::new(HatchEntityDefinition {
                association_enabled: p.is_associative,
                gradient: p.gradient.map(super::hatch_gradient::gradient_from_cad),
                scope_id,
                layer_id,
                visible: !h.common.invisible,
                appearance,
                placement: p.placement,
                loops: p
                    .boundaries
                    .into_iter()
                    .zip(p.loop_flags)
                    .map(|(boundary, flags)| OcdrawHatchLoop {
                        provenance: flags.provenance,
                        text_island: flags.text_island,
                        boundary,
                        source_entity_ids: vec![],
                    })
                    .collect(),
                area_rule: p.area_rule,
                join_tolerance: p.join_tolerance,
                fill: p.fill,
            }),
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
) -> Result<Option<EntityType>, OcdrawToCadError> {
    let boundaries = h
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
        h.placement,
        &boundaries,
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
                "HATCH_UNSUPPORTED"
            };
            diagnostics.push(crate::to_cad::diagnostic(
                code,
                format!("/entities/{}/{field}", h.id),
                detail,
            ));
            return Ok(None);
        }
        Err(source) => {
            return Err(OcdrawToCadError::HatchPreparation {
                entity_id: h.id,
                source,
            })
        }
    };
    let flags = h
        .loops
        .iter()
        .map(|l| cad_geometry_convert::hatch::PreparedHatchLoopFlags {
            provenance: l.provenance.clone(),
            text_island: l.text_island,
        })
        .collect::<Vec<_>>();
    cad_geometry_convert::hatch::apply_loop_flags_to_cad(&mut p.hatch, &flags).map_err(
        |source| OcdrawToCadError::HatchPreparation {
            entity_id: h.id,
            source,
        },
    )?;
    if !p.losses.iter().any(|l| l.field == "gradient.frame") {
        for loss in color_losses {
            diagnostics.push(crate::to_cad::diagnostic(
                "HATCH_GRADIENT_COLOR",
                format!("/entities/{}/{}", h.id, loss.field),
                loss.detail,
            ));
        }
    }
    for loss in p.losses {
        diagnostics.push(crate::to_cad::diagnostic(
            if loss.field == "gradient.frame" {
                "HATCH_GRADIENT_FRAME_UNSUPPORTED"
            } else {
                "HATCH_PATTERN_METADATA"
            },
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
    Ok(Some(EntityType::Hatch(Box::new(p.hatch))))
}
