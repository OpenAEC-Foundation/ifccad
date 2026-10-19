use super::field_validation::logical_error;
use super::*;
use crate::geometry_kernel::hatch::{hatch_bounds, validate_hatch_boundaries};
use std::collections::BTreeMap;

pub(crate) fn hatch_gradient_color_valid(color: &DrawingColor) -> bool {
    color.concrete_rgb().is_some() && super::field_validation::color(color)
}

pub(crate) fn validate_hatch_parts(
    doc: &OcdrawDocument,
    phase: ValidationPhase,
) -> Vec<LogicalError> {
    let owners: BTreeMap<_, _> = doc
        .scopes
        .iter()
        .flat_map(|s| s.entities.iter().map(move |id| (*id, s.id)))
        .collect();
    let sources: std::collections::BTreeSet<_> = doc
        .geometric_entities
        .iter()
        .filter(|e| e.geometry.is_hatch_boundary_source())
        .map(|e| e.id)
        .chain(doc.text_entities.iter().map(|e| e.id))
        .chain(doc.mtext_entities.iter().map(|e| e.id))
        .chain(doc.viewports.iter().map(|e| e.id))
        .collect();
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
        if let Err(e) = crate::geometry_kernel::hatch::validate_hatch_gradient(
            matches!(h.fill, crate::geometry_kernel::hatch::HatchFill::Gradient),
            h.gradient.as_ref(),
            hatch_gradient_color_valid,
        ) {
            errors.push(logical_error(
                "HATCH_GRADIENT",
                format!("{path}/gradient/{}", e.field),
                &e.to_string(),
            ));
        }
        for (j, l) in h.loops.iter().enumerate() {
            if l.provenance.as_ref().is_some_and(|p| p.is_empty()) {
                errors.push(logical_error(
                    "HATCH_PROVENANCE",
                    format!("{path}/loops/{j}/provenance"),
                    "empty provenance must be omitted",
                ));
            }
            for (k, id) in l.source_entity_ids.iter().enumerate() {
                let good = sources.contains(id) && owners.get(id) == owners.get(&h.id);
                if !good {
                    errors.push(logical_error(
                        "HATCH_SOURCE",
                        format!("{path}/loops/{j}/sourceEntityIds/{k}"),
                        "source must be a supported boundary source entity in the same owner",
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
