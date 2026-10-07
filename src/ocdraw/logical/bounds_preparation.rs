//! Explicit preparation of derived bounds; identity and authored records stay intact.
use super::*;
use crate::ocdraw::{
    geometry::{numeric::Interval, PreparedBlockTransform},
    Bounds3d, Point3,
};
use std::collections::BTreeMap;

type Range = ([f64; 3], [f64; 3]);
#[derive(Clone, Default)]
struct ScopeExtent {
    incomplete: bool,
    estimated: bool,
    range: Option<Range>,
    text_reasons: Vec<crate::text::TextExtentReason>,
}
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
    text: BTreeMap<u64, crate::text::TextExtentEstimate>,
    opaque: std::collections::BTreeSet<u64>,
    scopes: BTreeMap<u32, &'a DrawingScope>,
    entities: BTreeMap<u64, &'a DrawingGeometricEntity>,
    viewports: BTreeMap<u64, &'a DrawingViewport>,
    definitions: BTreeMap<u32, &'a DrawingBlockDefinition>,
    completed: BTreeMap<u32, ScopeExtent>,
}
impl Evaluation<'_> {
    fn scope(&mut self, id: u32) -> Result<ScopeExtent, OcdrawValidationError> {
        if let Some(bounds) = self.completed.get(&id) {
            return Ok(bounds.clone());
        }
        let scope = self.scopes[&id];
        let mut bounds = None;
        let mut extent = ScopeExtent::default();
        for entity_id in &scope.entities {
            if self.opaque.contains(entity_id) {
                extent.incomplete = true;
                continue;
            }
            if let Some(text) = self.text.get(entity_id) {
                extent.incomplete |= text.status == crate::text::TextExtentStatus::Unavailable;
                extent.estimated |= text.status != crate::text::TextExtentStatus::Empty;
                extent.text_reasons.extend(text.reasons.iter().copied());
                if let Some(b) = text.bounds {
                    union(&mut bounds, (b.min().components(), b.max().components()));
                }
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
                    let local_extent = self.scope(definition_scope_id)?;
                    extent.incomplete |= local_extent.incomplete;
                    extent.estimated |= local_extent.estimated;
                    extent.text_reasons.extend(local_extent.text_reasons);
                    // Validate native matrix preparation without inventing a
                    // point or complete enclosure for opaque child geometry.
                    let base = definition.base_point;
                    let prepared = PreparedBlockTransform::new(
                        transform,
                        Point3::new(base[0], base[1], base[2]),
                    )
                    .ok_or_else(|| failure(id, "block transform cannot be evaluated"))?;
                    if let Some((min, max)) = local_extent.range {
                        let projected = prepared
                            .apply_intervals(std::array::from_fn(|i| Interval {
                                lower: min[i],
                                upper: max[i],
                            }))
                            .ok_or_else(|| failure(id, "block bounds are out of range"))?;
                        (projected.map(|v| v.lower), projected.map(|v| v.upper))
                    } else if !local_extent.incomplete {
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
        extent.range = bounds;
        extent.text_reasons.dedup();
        self.completed.insert(id, extent.clone());
        Ok(extent)
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
    for (scope, assessment) in doc.scopes.iter_mut().zip(bounds) {
        scope.bounds = assessment.bounds;
        scope.bounds_quality = if assessment.bounds.is_some()
            && assessment.quality == Some(OcdrawBoundsQuality::Estimated)
        {
            assessment.quality
        } else {
            None
        };
    }
    Ok(())
}

/// Evaluates native subsets even when opaque content prevents a complete enclosure.
pub fn assess_ocdraw_document_bounds(
    doc: &OcdrawDocument,
) -> Result<Vec<OcdrawScopeBoundsAssessment>, OcdrawValidationError> {
    let errors = validate_logical_document(doc, ValidationPhase::BeforeBounds);
    if !errors.is_empty() {
        return Err(OcdrawValidationError::from_logical_errors(errors));
    }
    evaluate_document_bounds(doc)
}

pub(crate) fn evaluate_document_bounds(
    doc: &OcdrawDocument,
) -> Result<Vec<OcdrawScopeBoundsAssessment>, OcdrawValidationError> {
    let mut evaluation = Evaluation {
        text: super::text_bounds::text_estimates(doc)?,
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
            evaluation.scope(s.id).map(|extent| {
                let bounds = if extent.incomplete {
                    None
                } else {
                    extent.range.map(|(min, max)| {
                        Bounds3d::new(
                            Point3::new(min[0], min[1], min[2]),
                            Point3::new(max[0], max[1], max[2]),
                        )
                    })
                };
                OcdrawScopeBoundsAssessment {
                    scope_id: s.id,
                    bounds,
                    quality: if extent.incomplete {
                        Some(OcdrawBoundsQuality::Partial)
                    } else if bounds.is_none() {
                        None
                    } else if extent.estimated {
                        Some(OcdrawBoundsQuality::Estimated)
                    } else {
                        Some(OcdrawBoundsQuality::Enclosing)
                    },
                    enclosure_verified: !extent.incomplete && !extent.estimated,
                    text_reasons: extent.text_reasons,
                }
            })
        })
        .collect()
}
