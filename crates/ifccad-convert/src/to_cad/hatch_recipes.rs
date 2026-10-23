//! Optional recipe choice is conversion state, never a validity cache.
use crate::{
    geometry_context::{GeometryContext, GeometryNodeKey},
    *,
};
use cad_geometry_convert::{hatch::PreparedCadHatch, GeometryPair};
use opencadcodec::{CadDocument, EntityType};
use std::collections::BTreeMap;

pub(crate) struct PendingHatchRecipe {
    pub entity_id: u64,
    pub owner: IfccadGeometryOwner,
    pub preferred: PreparedCadHatch,
    pub fallback: PreparedCadHatch,
}
pub(crate) fn combine(pairs: &[GeometryPair]) -> GeometryPair {
    GeometryPair {
        points: pairs.iter().flat_map(|p| p.points.clone()).collect(),
        curves: pairs.iter().flat_map(|p| p.curves.clone()).collect(),
    }
}
fn source(p: &PendingHatchRecipe) -> IfccadGeometryEntitySource {
    IfccadGeometryEntitySource::NativeEntity {
        owner: p.owner,
        entity_id: p.entity_id,
    }
}
fn leaf(s: &IfccadGeometryEntitySource) -> Option<u64> {
    match s {
        IfccadGeometryEntitySource::NativeEntity { entity_id, .. } => Some(*entity_id),
        IfccadGeometryEntitySource::BlockOccurrence { leaf: s, .. } => leaf(s),
        _ => None,
    }
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_hatch_recipes(
    pending: Vec<PendingHatchRecipe>,
    document: &mut CadDocument,
    mappings: &IfccadMappings,
    members: &BTreeMap<u64, Vec<u64>>,
    roots: &[(IfccadGeometryOwner, Vec<u64>)],
    geometry: &mut GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), IfccadConversionError> {
    let mut preferred = vec![true; pending.len()];
    // Register actual operands without adding any trial to the final report.
    for p in &pending {
        geometry.state.register_geometry(
            GeometryNodeKey::Entity(p.entity_id),
            source(p),
            combine(&p.preferred.pairs),
        );
    }
    loop {
        let trial = (|| {
            for (p, use_recipe) in pending.iter().zip(&preferred) {
                geometry.select(p.owner);
                let candidate = if *use_recipe {
                    &p.preferred
                } else {
                    &p.fallback
                };
                let pair = combine(&candidate.pairs);
                for (index, (point, _)) in pair.points.iter().enumerate() {
                    let (lo, hi) = point.squared_deviation();
                    geometry
                        .state
                        .assessment
                        .check_interval(&source(p), index, &lo, &hi)
                        .map_err(|e| geometry.numerical_failure(e))?;
                }
                for curve in &pair.curves {
                    geometry
                        .state
                        .assessment
                        .check_curve(&source(p), curve)
                        .map_err(|e| geometry.numerical_failure(e))?;
                }
            }
            for (owner, list) in roots {
                geometry.select(*owner);
                geometry.trial_roots(members, list)?;
            }
            Ok(())
        })();
        match trial {
            Ok(()) => break,
            Err(IfccadConversionError::Geometry(failure)) => {
                let index = leaf(&failure.source)
                    .and_then(|id| pending.iter().position(|p| p.entity_id == id))
                    .filter(|i| preferred[*i]);
                let Some(index) = index else {
                    return Err(IfccadConversionError::Geometry(failure));
                };
                let p = &pending[index];
                preferred[index] = false;
                issues.push(crate::diagnostics::diagnostic("hatch-classic-settings-loss",format!("entity/{}/classicSettings/recipe",p.entity_id),format!("Original recipe was not qualified within the selected tolerance ({:?}); the checked active-fill recipe is used",failure.reason)));
                geometry.state.register_geometry(
                    GeometryNodeKey::Entity(p.entity_id),
                    source(p),
                    combine(&p.fallback.pairs),
                );
            }
            Err(e) => return Err(e),
        }
    }
    for (p, use_recipe) in pending.into_iter().zip(preferred) {
        let identity = source(&p);
        let selected = if use_recipe { p.preferred } else { p.fallback };
        let handle = mappings.entities.cad_handle(p.entity_id).ok_or_else(|| {
            IfccadConversionError::InvalidStructure("pending Hatch was not allocated".into())
        })?;
        let Some(EntityType::Hatch(target)) = document.get_entity_mut(handle) else {
            return Err(IfccadConversionError::InvalidStructure(
                "pending Hatch target is absent".into(),
            ));
        };
        // Preserve allocated common identity, contours, source/reactor links,
        // loop flags, construction and every unrelated extended-data record.
        let origin = selected.hatch.stored_pattern_origin();
        target.common =
            cad_geometry_convert::hatch::pattern::residual_common(target).map_err(|source| {
                IfccadConversionError::HatchPreparation {
                    location: format!("entity/{}", p.entity_id),
                    source,
                }
            })?;
        if let Some(o) = origin {
            target.record_pattern_origin(o);
        }
        target.pattern = selected.hatch.pattern;
        target.pattern_type = selected.hatch.pattern_type;
        target.pattern_angle = selected.hatch.pattern_angle;
        target.pattern_scale = selected.hatch.pattern_scale;
        target.is_double = selected.hatch.is_double;
        geometry.select(p.owner);
        geometry.record(p.entity_id, identity, combine(&selected.pairs), issues)?;
    }
    Ok(())
}
