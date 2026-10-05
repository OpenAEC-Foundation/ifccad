//! Resolve viewport dependencies on converted geometry before ordered emission.
use crate::source::{
    CadToOcdrawAction, CadToOcdrawDiagnostic, CadToOcdrawDiagnosticSource, CadToOcdrawLossReason,
};
use ocdraw::ocdraw::*;
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct PreparedCadEntity {
    pub handle: Handle,
    pub value: PreparedCadEntityValue,
}
pub(super) enum PreparedCadEntityValue {
    Geometry(GeometricEntityDefinition),
    Viewport {
        definition: ViewportDefinition,
        boundary: Handle,
    },
}
pub(super) fn append(
    mut drawing: OcdrawBuilder,
    prepared: Vec<PreparedCadEntity>,
    source: &CadDocument,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> Result<(OcdrawDocument, BTreeMap<Handle, u64>), OcdrawBuildError> {
    let geometry = prepared
        .iter()
        .filter_map(|e| match &e.value {
            PreparedCadEntityValue::Geometry(g) => Some((e.handle, g)),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let mut claims = BTreeMap::<Handle, usize>::new();
    let overall = source
        .objects
        .values()
        .filter_map(|object| {
            if let opencadcodec::objects::ObjectType::Layout(layout) = object {
                crate::source::overall_viewport_handle(source, layout)
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    for entity in source.entities() {
        if let EntityType::Viewport(v) = entity {
            if v.clip_boundary_handle != Handle::NULL && !overall.contains(&v.common.handle) {
                *claims.entry(v.clip_boundary_handle).or_default() += 1;
            }
        }
    }
    let mut skipped = BTreeSet::new();
    for entity in &prepared {
        let PreparedCadEntityValue::Viewport {
            definition,
            boundary,
        } = &entity.value
        else {
            continue;
        };
        if *boundary == Handle::NULL && !definition.paper_clip.enabled {
            continue;
        }
        let failure = if *boundary == Handle::NULL {
            Some("active clip has no boundary reference".to_owned())
        } else if claims[boundary] > 1 {
            Some("clip boundary is claimed by multiple viewports".to_owned())
        } else if let Some(g) = geometry.get(boundary) {
            if g.scope_id != definition.scope_id {
                Some("clip boundary belongs to another scope".into())
            } else if definition.paper_clip.enabled {
                validate_viewport_clip_boundary(definition.frame, &g.geometry)
                    .err()
                    .map(|e| {
                        e.diagnostics()
                            .iter()
                            .map(|d| d.message.as_str())
                            .collect::<Vec<_>>()
                            .join("; ")
                    })
            } else {
                None
            }
        } else {
            Some("clip boundary was not converted to supported geometry".into())
        };
        if let Some(message) = failure {
            skipped.insert(entity.handle);
            let reason = if source.get_entity(*boundary).is_none() {
                CadToOcdrawLossReason::MissingTarget {
                    kind: "viewport clip boundary".into(),
                    identifier: boundary.to_string(),
                }
            } else {
                CadToOcdrawLossReason::UnsupportedSemantic {
                    name: format!("viewport clip boundary: {message}"),
                }
            };
            diagnostics.push(CadToOcdrawDiagnostic::loss(
                CadToOcdrawDiagnosticSource::Entity {
                    handle: entity.handle,
                    kind: "VIEWPORT".into(),
                },
                CadToOcdrawAction::Skipped,
                vec![reason],
            ));
        }
    }
    let mut mapping = BTreeMap::new();
    let mut pending = Vec::new();
    for entity in prepared {
        if skipped.contains(&entity.handle) {
            continue;
        }
        let id = match entity.value {
            PreparedCadEntityValue::Geometry(g) => drawing.add_geometric_entity(g)?,
            PreparedCadEntityValue::Viewport {
                mut definition,
                boundary,
            } => {
                let enabled = definition.paper_clip.enabled;
                // These private drafts never escape; bind the complete mapping below.
                definition.paper_clip = DrawingPaperClip {
                    enabled: false,
                    boundary_entity_id: None,
                };
                let id = drawing.add_viewport(definition)?;
                if boundary != Handle::NULL {
                    pending.push((id, (boundary, enabled)));
                }
                id
            }
        };
        mapping.insert(entity.handle, id);
    }
    let mut document = drawing.build_document()?;
    let bindings = pending.into_iter().collect::<BTreeMap<_, _>>();
    for v in &mut document.viewports {
        if let Some((boundary, enabled)) = bindings.get(&v.id) {
            v.paper_clip = DrawingPaperClip {
                enabled: *enabled,
                boundary_entity_id: Some(mapping[boundary]),
            };
        }
    }
    validate_ocdraw_document(&document)
        .map_err(|e| OcdrawBuildError::Invalid(format!("{:?}", e.diagnostics())))?;
    Ok((document, mapping))
}
