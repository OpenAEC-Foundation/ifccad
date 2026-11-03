use super::*;
use crate::geometry_kernel::hatch::*;
fn invalid(path: &str, detail: impl ToString) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-ANNOTATION-006", path, detail.to_string())
}
pub(super) fn boundaries<'a>(
    base: &'a IfccadHatch,
    p: &'a IfccadHatchAnnotationContext,
) -> impl Iterator<Item = &'a HatchBoundary2> {
    p.loops.iter().filter_map(move |l| match l {
        IfccadHatchContextLoop::Shared {
            base_loop_index, ..
        } => base
            .loops
            .get(*base_loop_index as usize)
            .map(|l| &l.boundary),
        IfccadHatchContextLoop::Local { boundary, .. } => Some(&boundary.geometry),
    })
}
fn presentation(
    base: &IfccadHatch,
    p: &IfccadHatchAnnotationContext,
    path: &str,
) -> Result<(), IfccadReport> {
    if !p.pattern.scale.is_finite()
        || p.pattern.scale <= 0.
        || p.pattern
            .orientation_reference
            .iter()
            .any(|v| !v.is_finite())
    {
        return Err(invalid(path, "invalid stored Hatch context pattern"));
    }
    if !p.pattern.families.is_empty() {
        validate_hatch_fill(&HatchFill::LinePattern(HatchLinePattern {
            name: None,
            description: None,
            origin: [0.; 2],
            rotation: 0.,
            scale: p.pattern.scale,
            families: p.pattern.families.clone(),
        }))
        .map_err(|e| invalid(path, e))?;
    }
    for (i, l) in p.loops.iter().enumerate() {
        let path = format!("{path}/loops/{i}");
        match l {
            IfccadHatchContextLoop::Shared {
                base_loop_index, ..
            } if *base_loop_index as usize >= base.loops.len() => {
                return Err(invalid(&path, "unresolved base Hatch loop index"))
            }
            IfccadHatchContextLoop::Local {
                classification,
                boundary,
            } if classification.polyline
                != matches!(boundary.geometry, HatchBoundary2::Polyline { .. }) =>
            {
                return Err(invalid(
                    &path,
                    "local Hatch polyline flag must match encoded boundary alternative",
                ))
            }
            _ => (),
        }
    }
    if !p.loops.is_empty() {
        validate_hatch_boundaries(boundaries(base, p), base.join_tolerance)
            .map_err(|e| invalid(path, e))?;
    }
    Ok(())
}
pub(super) fn validate(
    d: &IfccadDocument,
    h: &IfccadHatch,
    path: &str,
) -> Result<(), IfccadReport> {
    if let Some(a) = &h.annotation {
        a.validate_memberships(&d.annotation_scales, false)?;
        for c in &a.contexts {
            if let Some(p) = &c.presentation {
                presentation(
                    h,
                    p,
                    &format!(
                        "{path}/ifccad::annotation/contexts/scale{}",
                        c.key.scale_id.0
                    ),
                )?;
            }
        }
    }
    if let Some(a) = &h.annotation_view_contexts {
        let path = format!("{path}/ifccad::hatchAnnotationViews");
        let mut ids = std::collections::BTreeSet::new();
        if a.next_context_id == 0 {
            return Err(invalid(&path, "positive Hatch view watermark required"));
        }
        for c in &a.contexts {
            let path = format!("{path}/contexts/{}", c.id.0);
            if c.id.0 == 0
                || c.id.0 >= a.next_context_id
                || !ids.insert(c.id)
                || !d.annotation_scales.iter().any(|s| s.id == c.scale_id)
            {
                return Err(invalid(
                    &path,
                    "invalid Hatch view identity, scale reference or watermark",
                ));
            }
            let target = match c.target {
                None => true,
                Some(IfccadAnnotationViewTarget::PaperViewport { entity_id }) => {
                    d.paper_layouts.iter().flat_map(|p| &p.entities).any(|e| {
                        e.id() == entity_id
                            && e.as_native()
                                .is_some_and(|e| matches!(e.kind, IfccadEntityKind::Viewport(_)))
                    })
                }
                Some(IfccadAnnotationViewTarget::ModelWindow { id }) => {
                    d.model_windows.iter().any(|w| w.id == id)
                }
                Some(IfccadAnnotationViewTarget::StoredView { id }) => {
                    d.annotation_views.iter().any(|v| v.id == id)
                }
            };
            if !target {
                return Err(invalid(
                    &path,
                    "unresolved or incompatible Hatch view target",
                ));
            }
            if c.view_state.normal.iter().any(|v| !v.is_finite())
                || c.view_state.normal.iter().all(|v| *v == 0.)
                || !c.view_state.rotation.is_finite()
            {
                return Err(invalid(&path, "invalid Hatch view projection state"));
            }
            presentation(h, &c.presentation, &format!("{path}/presentation"))?;
        }
        if a.reference.is_some_and(|id| !ids.contains(&id)) {
            return Err(invalid(&path, "unresolved independent Hatch view default"));
        }
    }
    Ok(())
}
pub(super) fn source_groups(h: &IfccadHatch) -> Vec<(String, &[u64])> {
    let mut out = h
        .loops
        .iter()
        .enumerate()
        .map(|(i, l)| (format!("loops/{i}/sources"), l.source_entity_ids.as_slice()))
        .collect::<Vec<_>>();
    for (prefix, p) in h
        .annotation
        .iter()
        .flat_map(|a| a.contexts.iter())
        .filter_map(|c| {
            c.presentation.as_ref().map(|p| {
                (
                    format!("ifccad::annotation/contexts/scale{}", c.key.scale_id.0),
                    p,
                )
            })
        })
        .chain(
            h.annotation_view_contexts
                .iter()
                .flat_map(|a| a.contexts.iter())
                .map(|c| {
                    (
                        format!("ifccad::hatchAnnotationViews/contexts/{}", c.id.0),
                        &c.presentation,
                    )
                }),
        )
    {
        for (i, l) in p.loops.iter().enumerate() {
            if let IfccadHatchContextLoop::Local { boundary, .. } = l {
                out.push((
                    format!("{prefix}/loops/{i}/boundary/sourceEntityIds"),
                    &boundary.source_entity_ids,
                ));
            }
        }
    }
    out
}
