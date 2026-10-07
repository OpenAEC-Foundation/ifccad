//! Adapter-owned restoration predicates; core transports their baselines untouched.
use super::OcdrawPreservationReason;
use ocdraw::ocdraw::*;
use opencadcodec::entities::Spline;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EntityBaseline {
    preservation_record_id: u64,
    owner_scope_id: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CoordinateBaseline {
    kind: String,
    coordinate_unit: String,
}

fn scope_kind(kind: DrawingScopeKind) -> &'static str {
    match kind {
        DrawingScopeKind::Model => "Model",
        DrawingScopeKind::Paper => "Paper",
        DrawingScopeKind::Block => "Block",
    }
}

pub(crate) fn build_spline_conditions(
    record_id: OcdrawPreservationRecordId,
    entity_id: u64,
    scope_id: u32,
    kind: DrawingScopeKind,
    unit: &str,
) -> Vec<OcdrawPreservationCondition> {
    vec![
        OcdrawPreservationCondition {
            target: OcdrawPreservationTarget::Entity(entity_id),
            predicate: "openaec.ocdraw.splineEntityBinding".into(),
            version: 1,
            baseline: serde_json::to_vec(&EntityBaseline {
                preservation_record_id: record_id.0,
                owner_scope_id: scope_id,
            })
            .expect("integer baseline"),
        },
        OcdrawPreservationCondition {
            target: OcdrawPreservationTarget::Scope(scope_id),
            predicate: "openaec.ocdraw.splineCoordinateContext".into(),
            version: 1,
            baseline: serde_json::to_vec(&CoordinateBaseline {
                kind: scope_kind(kind).into(),
                coordinate_unit: unit.into(),
            })
            .expect("string baseline"),
        },
    ]
}

pub(crate) fn evaluate_spline_conditions(
    record: &OcdrawPreservationRecord,
    entity: &DrawingOpaqueEntity,
    drawing: &OcdrawDocument,
) -> Result<u32, OcdrawPreservationReason> {
    use OcdrawPreservationReason::*;
    let owners = drawing
        .scopes
        .iter()
        .filter(|s| s.entities.contains(&entity.id))
        .collect::<Vec<_>>();
    let owner = owners.first().ok_or(MissingDependency)?;
    let mut binding = false;
    let mut coordinate = false;
    for condition in &record.conditions {
        if condition.version != 1 {
            return Err(UnsupportedPredicate);
        }
        match condition.predicate.as_str() {
            "openaec.ocdraw.splineEntityBinding" => {
                if binding || condition.target != OcdrawPreservationTarget::Entity(entity.id) {
                    return Err(UnsupportedContext);
                }
                let baseline: EntityBaseline =
                    serde_json::from_slice(&condition.baseline).map_err(|_| MalformedPayload)?;
                if baseline.preservation_record_id == 0 {
                    return Err(MalformedPayload);
                }
                if baseline.preservation_record_id != record.id.0
                    || baseline.owner_scope_id != owner.id
                {
                    return Err(ChangedDependency);
                }
                binding = true;
            }
            "openaec.ocdraw.splineCoordinateContext" => {
                if coordinate || condition.target != OcdrawPreservationTarget::Scope(owner.id) {
                    return Err(ChangedDependency);
                }
                let baseline: CoordinateBaseline =
                    serde_json::from_slice(&condition.baseline).map_err(|_| MalformedPayload)?;
                if !["Model", "Paper", "Block"].contains(&baseline.kind.as_str())
                    || !crate::units::UNIT_TOKENS.contains(&baseline.coordinate_unit.as_str())
                {
                    return Err(MalformedPayload);
                }
                if baseline.kind != scope_kind(owner.kind)
                    || baseline.coordinate_unit != drawing.unit
                {
                    return Err(ChangedDependency);
                }
                coordinate = true;
            }
            "openaec.ocdraw.sourceReferenceBinding" => {
                super::validate_reference_condition(record, condition)?
            }
            _ => return Err(UnsupportedPredicate),
        }
    }
    if !binding || !coordinate {
        return Err(UnsupportedContext);
    }
    Ok(owner.id)
}

/// Extra application/private context is captured completely but cannot be replayed
/// safely until its semantics and identity requirements are qualified.
pub(crate) fn unsupported_common_context(spline: &Spline) -> bool {
    let c = &spline.common;
    c.graphic_data.is_some()
        || !c.extended_data.raw_dwg_eed.is_empty()
        || !c.reactors.is_empty()
        || c.xdictionary_handle.is_some()
        || c.color_book_handle.is_some()
        || c.full_visual_style_handle.is_some()
        || c.face_visual_style_handle.is_some()
        || c.edge_visual_style_handle.is_some()
        || c.material_flags != 0
        || c.material_handle.is_some()
        || c.shadow_flags != 0
        || c.plotstyle_flags != 0
        || c.plotstyle_handle.is_some()
        || c.has_ds_data
        || c.extended_data
            .records()
            .iter()
            .flat_map(|r| &r.values)
            .any(|v| {
                matches!(
                    v,
                    opencadcodec::xdata::XDataValue::String(_)
                        | opencadcodec::xdata::XDataValue::ControlString(_)
                        | opencadcodec::xdata::XDataValue::BinaryData(_)
                )
            })
}

pub(crate) fn record_unassessed_occurrences(
    doc: &OcdrawDocument,
    assessment: &mut Vec<crate::OcdrawGeometryEntitySource>,
) {
    let opaque = doc
        .opaque_entities
        .iter()
        .map(|e| e.id)
        .collect::<std::collections::BTreeSet<_>>();
    let instances = doc
        .geometric_entities
        .iter()
        .filter_map(|e| match e.geometry {
            DrawingGeometry::BlockInstance {
                definition_scope_id,
                ..
            } => Some((e.id, definition_scope_id)),
            _ => None,
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut stack = doc
        .scopes
        .iter()
        .flat_map(|s| {
            s.entities
                .iter()
                .map(move |id| (s.id, *id, Vec::<crate::OcdrawGeometryEntitySource>::new()))
        })
        .collect::<Vec<_>>();
    while let Some((scope, id, path)) = stack.pop() {
        let source = crate::OcdrawGeometryEntitySource::DrawingEntity {
            scope_id: scope,
            entity_id: id,
        };
        if opaque.contains(&id) {
            assessment.push(if path.is_empty() {
                source
            } else {
                crate::OcdrawGeometryEntitySource::BlockOccurrence {
                    path,
                    leaf: Box::new(source),
                }
            });
        } else if let Some(target) = instances.get(&id) {
            if let Some(definition) = doc.scopes.iter().find(|s| s.id == *target) {
                let mut next = path;
                next.push(source);
                for child in &definition.entities {
                    stack.push((*target, *child, next.clone()));
                }
            }
        }
    }
}
