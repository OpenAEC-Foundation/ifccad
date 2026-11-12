//! Ordinary entity relationships and shared CAD block-reference closure.
use super::*;

pub(super) fn validate(document: &IfccadDocument) -> Result<(), IfccadReport> {
    validate_ifccad_multi_leader_styles(
        document,
        &document.multi_leader_styles,
        document.id_counters.next_multi_leader_style_id,
    )?;
    for style in &document.dimension_styles {
        if let IfccadLeaderArrow::Symbol { definition_id } = style.leader_arrow {
            if !document
                .blocks
                .iter()
                .any(|block| block.id == definition_id)
            {
                return Err(crate::ifccad::diagnostics::failure(
                    "IFCCAD-LEADER-001",
                    "/dimensionStyle/leaderArrow",
                    "unresolved leader arrow block",
                ));
            }
        }
    }
    let owned = std::iter::once((
        IfccadScopeId::Layout(document.model.id),
        &document.model.entities,
    ))
    .chain(
        document
            .paper_layouts
            .iter()
            .map(|layout| (IfccadScopeId::Layout(layout.id), &layout.entities)),
    )
    .chain(
        document
            .blocks
            .iter()
            .map(|block| (IfccadScopeId::BlockDefinition(block.id), &block.entities)),
    )
    .flat_map(|(owner, entities)| {
        entities
            .iter()
            .map(move |entity| (entity.id(), (owner, entity)))
    })
    .collect::<std::collections::BTreeMap<_, _>>();
    for (id, (owner, entity)) in &owned {
        let Some(entity) = entity.as_native() else {
            continue;
        };
        let path = format!("/cad/d{}/e{id}", document.drawing_id);
        let result = match &entity.kind {
            IfccadEntityKind::Leader(leader) => {
                leader.validate_values(document).map_err(|report| {
                    crate::ifccad::diagnostics::context(report, "IFCCAD-LEADER-001", &path)
                })?;
                if let Some(target) = leader.annotation_entity {
                    let valid = owned.get(&target).is_some_and(|(target_owner, entity)| {
                        *target_owner == *owner
                            && entity.as_native().is_some_and(|target| {
                                matches!(
                                    (&leader.annotation_kind, &target.kind),
                                    (
                                        IfccadLeaderAnnotationKind::MText,
                                        IfccadEntityKind::MText(_)
                                    ) | (
                                        IfccadLeaderAnnotationKind::Block,
                                        IfccadEntityKind::BlockInstance { .. }
                                    ) | (
                                        IfccadLeaderAnnotationKind::Tolerance,
                                        IfccadEntityKind::Tolerance(_)
                                    )
                                )
                            })
                    });
                    if !valid {
                        return Err(crate::ifccad::diagnostics::failure("IFCCAD-LEADER-002",&format!("{path}/annotationEntity"),"annotation target must resolve to the declared ordinary kind in the same owner"));
                    }
                }
                Ok(())
            }
            IfccadEntityKind::MultiLeader(leader) => {
                leader.validate_values(document, &document.multi_leader_styles)
            }
            IfccadEntityKind::Tolerance(tolerance) => tolerance.validate_values(document),
            _ => Ok(()),
        };
        result.map_err(|report| {
            crate::ifccad::diagnostics::context(report, "IFCCAD-LEADER-001", &path)
        })?;
    }
    Ok(())
}

fn arrow(arrow: &IfccadLeaderArrow, out: &mut Vec<u64>) {
    if let IfccadLeaderArrow::Symbol { definition_id } = arrow {
        out.push(*definition_id);
    }
}
fn common(settings: &IfccadMultiLeaderCommonSettings, out: &mut Vec<u64>) {
    arrow(&settings.line.arrow, out);
    out.extend(settings.block.definition_id);
}
fn presentation(value: &IfccadMultiLeaderPresentation, out: &mut Vec<u64>) {
    if let Some(block) = &value.content.block {
        out.push(block.definition_id);
    }
    for group in &value.groups {
        for line in &group.lines {
            arrow(&line.settings.arrow, out);
        }
    }
}
fn dimension_markers(settings: &IfccadDimensionSettings, out: &mut Vec<u64>) {
    for marker in [&settings.arrows.first, &settings.arrows.second] {
        if let crate::dimension::DimensionMarker::LocalBlock { block } = marker {
            out.push(*block);
        }
    }
}
fn picture(picture: &IfccadDimensionGraphics, out: &mut Vec<u64>) {
    for part in &picture.parts {
        if let IfccadDimensionGraphicsKind::BlockInstance { definition_id, .. } = &part.kind {
            out.push(*definition_id);
        }
    }
}
/// All stored block dependencies, including inactive values and saved pictures.
/// Shared by cycle checks so a mixed Dimension/Leader/INSERT cycle cannot escape.
pub(super) fn referenced_blocks(kind: &IfccadEntityKind, document: &IfccadDocument) -> Vec<u64> {
    let mut out = vec![];
    match kind {
        IfccadEntityKind::BlockInstance { definition_id, .. } => out.push(*definition_id),
        IfccadEntityKind::Table(table) => {
            out.extend(table.grid.referenced_blocks());
            if let Some(graphics) = &table.graphics {
                picture(graphics, &mut out);
            }
        }
        IfccadEntityKind::Leader(leader) => {
            if let Some(style) = document
                .dimension_styles
                .iter()
                .find(|style| style.id == leader.dimension_style)
            {
                arrow(&style.leader_arrow, &mut out);
            }
            if let Some(value) = &leader.arrow {
                arrow(value, &mut out);
            }
        }
        IfccadEntityKind::MultiLeader(leader) => {
            common(&leader.settings.common, &mut out);
            presentation(&leader.presentation, &mut out);
            if let Some(style) = leader.style_id.and_then(|id| {
                document
                    .multi_leader_styles
                    .iter()
                    .find(|style| style.id == id)
            }) {
                common(&style.common, &mut out);
            }
            if let Some(annotation) = &leader.annotation {
                for context in &annotation.contexts {
                    if let Some(value) = &context.presentation {
                        presentation(value, &mut out);
                    }
                }
            }
            for value in &leader.legacy_arrow_overrides {
                arrow(&value.arrow, &mut out);
            }
        }
        IfccadEntityKind::Dimension(value) => {
            if let Some(style) = document
                .dimension_styles
                .iter()
                .find(|style| style.id == value.style_id)
            {
                arrow(&style.leader_arrow, &mut out);
                dimension_markers(&style.settings, &mut out);
                for overrides in style.type_overrides.values() {
                    let mut settings = style.settings.clone();
                    overrides.apply(&mut settings);
                    dimension_markers(&settings, &mut out);
                }
                let settings = crate::dimension::resolve_dimension_settings(
                    &style.settings,
                    value.definition.kind(),
                    &style.type_overrides,
                    &value.overrides,
                );
                dimension_markers(&settings, &mut out);
            }
            if let Some(graphics) = &value.graphics {
                picture(graphics, &mut out);
            }
            if let Some(annotation) = &value.annotation {
                for context in &annotation.contexts {
                    if let Some(graphics) = context
                        .presentation
                        .as_ref()
                        .and_then(|value| value.graphics.as_ref())
                    {
                        picture(graphics, &mut out);
                    }
                }
            }
        }
        _ => {}
    }
    out
}
