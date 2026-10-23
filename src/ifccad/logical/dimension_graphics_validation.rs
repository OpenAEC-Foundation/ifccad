use super::*;
use std::collections::BTreeSet;
fn fail(n: u8, path: &str, message: impl ToString) -> IfccadReport {
    crate::ifccad::diagnostics::failure(
        &format!("IFCCAD-DIMENSION-GRAPHICS-{n:03}"),
        path,
        message.to_string(),
    )
}
fn hatch(h: &IfccadHatch, path: &str) -> Result<(), IfccadReport> {
    if h.association_enabled
        || h.gradient.is_some()
        || !matches!(h.fill, crate::geometry_kernel::hatch::HatchFill::Solid)
        || h.loops.iter().any(|l| !l.source_entity_ids.is_empty())
    {
        return Err(fail(
            1,
            path,
            "graphics Hatch must be Solid, unassociated and source-free",
        ));
    }
    super::hatch::validate_hatch(h, path)
}
fn marker(
    d: &IfccadDocument,
    id: u64,
    path: &str,
    active: &mut BTreeSet<u64>,
    done: &mut BTreeSet<u64>,
) -> Result<(), IfccadReport> {
    if done.contains(&id) {
        return Ok(());
    }
    if !active.insert(id) {
        return Err(fail(2, path, "graphics marker cycle"));
    }
    let b = d
        .blocks
        .iter()
        .find(|b| b.id == id)
        .ok_or_else(|| fail(2, path, "missing local graphics marker"))?;
    for e in &b.entities {
        let e = e
            .as_native()
            .ok_or_else(|| fail(1, path, "opaque marker contents are not eligible"))?;
        match &e.kind {
            IfccadEntityKind::Dimension(_) | IfccadEntityKind::Viewport(_) => {
                return Err(fail(1, path, "ineligible marker kind"))
            }
            IfccadEntityKind::Hatch(h) => hatch(h, path)?,
            IfccadEntityKind::BlockInstance { definition_id, .. } => {
                marker(d, *definition_id, path, active, done)?
            }
            _ => (),
        }
    }
    active.remove(&id);
    done.insert(id);
    Ok(())
}
pub(super) fn validate(d: &IfccadDocument) -> Result<(), IfccadReport> {
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    for e in d
        .model
        .entities
        .iter()
        .chain(d.paper_layouts.iter().flat_map(|p| &p.entities))
        .chain(d.blocks.iter().flat_map(|b| &b.entities))
        .filter_map(IfccadEntity::as_native)
    {
        let IfccadEntityKind::Dimension(v) = &e.kind else {
            continue;
        };
        let Some(g) = &v.graphics else { continue };
        for (i, p) in g.parts.iter().enumerate() {
            let path = format!("/cad/d{}/e{}/graphics/p{i}", d.drawing_id, e.id);
            if p.layer_id
                .is_some_and(|id| !d.layers.iter().any(|l| l.id == id))
            {
                return Err(fail(2, &path, "missing part layer"));
            }
            super::document_validation::entity_appearance(&p.appearance, &path)?;
            if let IfccadMode::Explicit(id) = p.appearance.line_pattern {
                if !d.line_patterns.iter().any(|l| l.id == id) {
                    return Err(fail(2, &path, "missing part line pattern"));
                }
            }
            super::patterns::scale(p.line_pattern_scale, &path)?;
            if p.purpose == IfccadDimensionGraphicsPurpose::ReferencePoint
                && !matches!(p.kind, IfccadDimensionGraphicsKind::Point { .. })
            {
                return Err(fail(1, &path, "ReferencePoint requires Point"));
            }
            if let IfccadDimensionGraphicsKind::PlanarPolyline {
                widths, vertices, ..
            } = &p.kind
            {
                widths.validate(vertices.len()).map_err(|e| {
                    crate::ifccad::diagnostics::failure(
                        "IFCCAD-POLYLINE-WIDTH-001",
                        &path,
                        e.to_string(),
                    )
                })?;
            }
            if let Some(geometry) = p.kind.as_shared_geometry()? {
                crate::geometry_kernel::validate_geometry(geometry)
                    .map_err(|e| fail(1, &path, e))?;
            }
            match &p.kind {
                IfccadDimensionGraphicsKind::BlockInstance {
                    definition_id,
                    transform,
                } => {
                    marker(d, *definition_id, &path, &mut active, &mut done)?;
                    super::document_validation::validate_block_instance(
                        d,
                        *definition_id,
                        transform,
                        &path,
                    )?;
                }
                IfccadDimensionGraphicsKind::Hatch(h) => hatch(h, &path)?,
                IfccadDimensionGraphicsKind::Text(t) => {
                    super::text::validate_value(d, super::text::TextValue::Text(t), &path)?
                }
                IfccadDimensionGraphicsKind::MText(t) => {
                    super::text::validate_value(d, super::text::TextValue::MText(t), &path)?
                }
                _ => (),
            }
        }
    }
    Ok(())
}
