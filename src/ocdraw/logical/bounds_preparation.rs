//! Explicit preparation of derived bounds; identity and authored records stay intact.
use super::*;
use crate::ocdraw::{
    geometry::{numeric::Interval, PreparedBlockTransform},
    Bounds3d, Point3,
};
use std::collections::BTreeMap;

type Range = ([f64; 3], [f64; 3]);
fn union(bounds: &mut Option<Range>, (min, max): Range) {
    match bounds {
        Some((lo, hi)) => {
            for i in 0..3 {
                lo[i] = lo[i].min(min[i]);
                hi[i] = hi[i].max(max[i]);
            }
        }
        None => *bounds = Some((min, max)),
    }
}
fn failure(id: u32, message: &str) -> OcdrawValidationError {
    OcdrawValidationError::from_logical_errors(vec![super::field_validation::logical_error(
        "SCOPE_BOUNDS",
        format!("/scopes/{id}/bounds"),
        message,
    )])
}

struct Evaluation<'a> {
    opaque: std::collections::BTreeSet<u64>,
    scopes: BTreeMap<u32, &'a DrawingScope>,
    entities: BTreeMap<u64, &'a DrawingGeometricEntity>,
    viewports: BTreeMap<u64, &'a DrawingViewport>,
    definitions: BTreeMap<u32, &'a DrawingBlockDefinition>,
    completed: BTreeMap<u32, (bool, Option<Range>)>,
}
impl Evaluation<'_> {
    fn scope(&mut self, id: u32) -> Result<(bool, Option<Range>), OcdrawValidationError> {
        if let Some(bounds) = self.completed.get(&id) {
            return Ok(*bounds);
        }
        let scope = self.scopes[&id];
        let mut bounds = None;
        let mut complete = true;
        for entity_id in &scope.entities {
            if self.opaque.contains(entity_id) {
                complete = false;
                continue;
            }
            if let Some(viewport) = self.viewports.get(entity_id) {
                let b = viewport_bounds(viewport.frame)
                    .ok_or_else(|| failure(id, "invalid viewport enclosure"))?;
                union(&mut bounds, (b.min().components(), b.max().components()));
                continue;
            }
            let entity = self.entities[entity_id];
            let range = match entity.geometry {
                EntityGeometry::BlockInstance {
                    definition_scope_id,
                    transform,
                } => {
                    let definition = self.definitions[&definition_scope_id];
                    let (local_complete, local) = self.scope(definition_scope_id)?;
                    complete &= local_complete;
                    // Validate native matrix preparation without inventing a
                    // point or complete enclosure for opaque child geometry.
                    let base = definition.base_point;
                    let prepared = PreparedBlockTransform::new(
                        transform,
                        Point3::new(base[0], base[1], base[2]),
                    )
                    .ok_or_else(|| failure(id, "block transform cannot be evaluated"))?;
                    if let Some((min, max)) = local {
                        let projected = prepared
                            .apply_intervals(std::array::from_fn(|i| Interval {
                                lower: min[i],
                                upper: max[i],
                            }))
                            .ok_or_else(|| failure(id, "block bounds are out of range"))?;
                        (projected.map(|v| v.lower), projected.map(|v| v.upper))
                    } else if local_complete {
                        // Preserve the existing writer's point enclosure for an empty instance.
                        let origin = transform.placement().origin().components();
                        (origin, origin)
                    } else {
                        continue;
                    }
                }
                _ => enclosure(&entity.geometry)
                    .ok_or_else(|| failure(id, "geometry cannot be enclosed"))?,
            };
            if !range.0.into_iter().chain(range.1).all(f64::is_finite) {
                return Err(failure(id, "non-finite bounds"));
            }
            union(&mut bounds, range);
        }
        self.completed.insert(id, (complete, bounds));
        Ok((complete, bounds))
    }
}

/// Recomputes scope bounds explicitly. Failure leaves all supplied bounds intact.
pub fn recompute_ocdraw_document_bounds(
    doc: &mut OcdrawDocument,
) -> Result<(), OcdrawValidationError> {
    let errors = validate_logical_document(doc, ValidationPhase::BeforeBounds);
    if !errors.is_empty() {
        return Err(OcdrawValidationError::from_logical_errors(errors));
    }
    let bounds = evaluate_document_bounds(doc)?;
    for (scope, bounds) in doc.scopes.iter_mut().zip(bounds) {
        scope.bounds = bounds;
    }
    Ok(())
}

/// Evaluates native subsets even when opaque content prevents a complete enclosure.
pub(crate) fn evaluate_document_bounds(
    doc: &OcdrawDocument,
) -> Result<Vec<Option<Bounds3d>>, OcdrawValidationError> {
    let mut evaluation = Evaluation {
        opaque: doc.opaque_entities.iter().map(|e| e.id).collect(),
        scopes: doc.scopes.iter().map(|s| (s.id, s)).collect(),
        entities: doc.geometric_entities.iter().map(|e| (e.id, e)).collect(),
        viewports: doc.viewports.iter().map(|v| (v.id, v)).collect(),
        definitions: doc
            .block_definitions
            .iter()
            .map(|b| (b.scope_id, b))
            .collect(),
        completed: BTreeMap::new(),
    };
    doc.scopes
        .iter()
        .map(|s| {
            evaluation.scope(s.id).map(|(complete, range)| {
                if !complete {
                    return None;
                }
                range.map(|(min, max)| {
                    Bounds3d::new(
                        Point3::new(min[0], min[1], min[2]),
                        Point3::new(max[0], max[1], max[2]),
                    )
                })
            })
        })
        .collect()
}
