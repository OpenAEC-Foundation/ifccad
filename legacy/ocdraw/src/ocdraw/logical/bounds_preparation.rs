//! Explicit preparation of derived bounds; identity and authored records stay intact.
use super::line_pattern_bounds::{effective_context, marker_contribution, AppearanceContext};
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
    line_pattern_reasons: Vec<crate::line_pattern::LinePatternBoundsReason>,
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
    document: &'a OcdrawDocument,
    text: BTreeMap<u64, crate::text::TextExtentEstimate>,
    opaque: std::collections::BTreeSet<u64>,
    scopes: BTreeMap<u32, &'a DrawingScope>,
    entities: BTreeMap<u64, &'a DrawingGeometricEntity>,
    viewports: BTreeMap<u64, &'a DrawingViewport>,
    hatches: BTreeMap<u64, &'a DrawingHatchEntity>,
    definitions: BTreeMap<u32, &'a DrawingBlockDefinition>,
    completed: BTreeMap<(u32, Option<AppearanceContext>), ScopeExtent>,
}
impl Evaluation<'_> {
    fn scope(
        &mut self,
        id: u32,
        parent: Option<AppearanceContext>,
    ) -> Result<ScopeExtent, OcdrawValidationError> {
        let key = (id, parent);
        if let Some(bounds) = self.completed.get(&key) {
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
            if let Some(h) = self.hatches.get(entity_id) {
                let b = crate::geometry_kernel::hatch::hatch_bounds(
                    h.placement,
                    h.loops.iter().map(|l| &l.boundary),
                )
                .map_err(|e| failure(id, &e.to_string()))?;
                union(&mut bounds, (b.min().components(), b.max().components()));
                continue;
            }
            let entity = self.entities[entity_id];
            let context = effective_context(
                entity,
                parent,
                self.definitions.contains_key(&id),
                self.document,
            );
            let mut range = match entity.geometry {
                EntityGeometry::BlockInstance {
                    definition_scope_id,
                    transform,
                } => {
                    let definition = self.definitions[&definition_scope_id];
                    let local_extent = self.scope(definition_scope_id, Some(context))?;
                    extent.incomplete |= local_extent.incomplete;
                    extent.estimated |= local_extent.estimated;
                    extent.text_reasons.extend(local_extent.text_reasons);
                    extent
                        .line_pattern_reasons
                        .extend(local_extent.line_pattern_reasons);
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
            if !matches!(
                entity.geometry,
                EntityGeometry::BlockInstance { .. } | EntityGeometry::Point { .. }
            ) {
                let markers = marker_contribution(
                    context.pattern_id,
                    entity.appearance.line_pattern_scale,
                    self.document,
                )?;
                extent.incomplete |= markers.unavailable;
                extent.estimated |= markers.radius.is_some();
                extent.text_reasons.extend(markers.reasons);
                extent.line_pattern_reasons.extend(markers.bounds_reasons);
                if let Some(radius) = markers.radius {
                    range = crate::line_pattern::inflate_range(range, radius)
                        .ok_or_else(|| failure(id, "text marker estimate exceeds finite range"))?;
                }
            }
            if !range.0.into_iter().chain(range.1).all(f64::is_finite) {
                return Err(failure(id, "non-finite bounds"));
            }
            union(&mut bounds, range);
        }
        extent.range = bounds;
        extent.text_reasons.dedup();
        extent.line_pattern_reasons.dedup();
        self.completed.insert(key, extent.clone());
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
        document: doc,
        text: super::text_bounds::text_estimates(doc)?,
        opaque: doc
            .opaque_entities
            .iter()
            .map(|e| e.id)
            .chain(doc.dimension_entities.iter().map(|e| e.id))
            .collect(),
        scopes: doc.scopes.iter().map(|s| (s.id, s)).collect(),
        entities: doc.geometric_entities.iter().map(|e| (e.id, e)).collect(),
        viewports: doc.viewports.iter().map(|v| (v.id, v)).collect(),
        hatches: doc.hatch_entities.iter().map(|h| (h.id, h)).collect(),
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
            evaluation.scope(s.id, None).map(|extent| {
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
                    line_pattern_reasons: extent.line_pattern_reasons,
                }
            })
        })
        .collect()
}
