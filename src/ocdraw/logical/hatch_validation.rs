use super::field_validation::logical_error;
use super::*;
use crate::geometry_kernel::hatch::{hatch_bounds, validate_hatch_boundaries};
use std::collections::BTreeMap;

pub(crate) fn validate_hatch_parts(
    doc: &OcdrawDocument,
    phase: ValidationPhase,
) -> Vec<LogicalError> {
    let owners: BTreeMap<_, _> = doc
        .scopes
        .iter()
        .flat_map(|s| s.entities.iter().map(move |id| (*id, s.id)))
        .collect();
    let sources: BTreeMap<_, _> = doc.geometric_entities.iter().map(|e| (e.id, e)).collect();
    let scopes: BTreeMap<_, _> = doc.scopes.iter().map(|s| (s.id, s)).collect();
    let mut errors = Vec::new();
    for (i, h) in doc.hatch_entities.iter().enumerate() {
        let path = format!("/hatchEntities/{i}");
        if let Err(e) =
            validate_hatch_boundaries(h.loops.iter().map(|l| &l.boundary), h.join_tolerance)
        {
            errors.push(logical_error(
                "HATCH_GEOMETRY",
                format!("{path}/loops"),
                &e.to_string(),
            ));
            continue;
        }
        if let Err(e) = crate::geometry_kernel::hatch::validate_hatch_fill(&h.fill) {
            errors.push(logical_error(
                "HATCH_FILL",
                format!("{path}/fill"),
                &e.to_string(),
            ));
        }
        for (j, l) in h.loops.iter().enumerate() {
            if let Some(id) = l.source_entity_id {
                let good = sources.get(&id).is_some_and(|e| {
                    matches!(
                        e.geometry,
                        EntityGeometry::Circle { .. }
                            | EntityGeometry::Ellipse { arc: None, .. }
                            | EntityGeometry::PlanarPolyline { closed: true, .. }
                    )
                }) && owners.get(&id) == owners.get(&h.id);
                if !good {
                    errors.push(logical_error(
                        "HATCH_SOURCE",
                        format!("{path}/loops/{j}/sourceEntityId"),
                        "source must be a full closed supported entity in the same owner",
                    ));
                }
            }
        }
        if phase == ValidationPhase::Complete {
            if let Some(owner) = owners.get(&h.id).and_then(|id| scopes.get(id)) {
                if let Some(supplied) = owner.bounds {
                    match hatch_bounds(h.placement, h.loops.iter().map(|l| &l.boundary)) {
                        Ok(b) => {
                            let lo = supplied.min().components();
                            let hi = supplied.max().components();
                            let bl = b.min().components();
                            let bh = b.max().components();
                            if (0..3).any(|k| lo[k] > bl[k] || hi[k] < bh[k]) {
                                errors.push(logical_error(
                                    "HATCH_BOUNDS",
                                    format!("{path}/bounds"),
                                    "scope bounds do not contain the certified Hatch enclosure",
                                ));
                            }
                        }
                        Err(e) => errors.push(logical_error(
                            "HATCH_GEOMETRY",
                            format!("{path}/placement"),
                            &e.to_string(),
                        )),
                    }
                }
            }
        }
    }
    errors
}
