use super::*;
use crate::dimension::*;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};
fn fail(path: &str, message: impl ToString) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-DIMENSION-001", path, message.to_string())
}
fn settings(
    d: &IfccadDocument,
    path: &str,
    s: &IfccadDimensionSettings,
) -> Result<(), IfccadReport> {
    validate_dimension_settings(s).map_err(|e| fail(&format!("{path}/{}", e.field), e))?;
    let error = RefCell::new(None);
    let _ = s.map_references(
        |id| {
            if !d.text_styles.iter().any(|s| s.id == *id) {
                *error.borrow_mut() = Some(fail(path, "missing TextStyle"));
            }
            *id
        },
        |id| {
            if !d.blocks.iter().any(|b| b.id == *id) {
                *error.borrow_mut() = Some(fail(path, "missing local marker block"));
            }
            *id
        },
        |c| {
            if !c.is_valid() {
                *error.borrow_mut() = Some(fail(path, "invalid concrete color"));
            }
            c.clone()
        },
        |id| {
            if !d.line_patterns.iter().any(|p| p.id == *id) {
                *error.borrow_mut() = Some(fail(path, "missing line pattern"));
            }
            *id
        },
    );
    if let Some(e) = error.into_inner() {
        Err(e)
    } else {
        Ok(())
    }
}
fn eligible(kind: &IfccadEntityKind, s: DimensionPointSelector) -> bool {
    match (kind, s) {
        (IfccadEntityKind::Point { .. }, DimensionPointSelector::Position) => true,
        (
            IfccadEntityKind::LineSegment { .. }
            | IfccadEntityKind::Arc { .. }
            | IfccadEntityKind::EllipseArc { .. },
            DimensionPointSelector::Start | DimensionPointSelector::End,
        ) => true,
        (
            IfccadEntityKind::Circle { .. }
            | IfccadEntityKind::Arc { .. }
            | IfccadEntityKind::Ellipse { .. }
            | IfccadEntityKind::EllipseArc { .. },
            DimensionPointSelector::Center,
        ) => true,
        (
            IfccadEntityKind::PlanarPolyline { vertices, .. },
            DimensionPointSelector::Vertex { index },
        ) => (index as usize) < vertices.len(),
        (
            IfccadEntityKind::SpatialPolyline { vertices, .. },
            DimensionPointSelector::Vertex { index },
        ) => (index as usize) < vertices.len(),
        (
            IfccadEntityKind::PlanarPolyline {
                closed: false,
                vertices,
                ..
            },
            DimensionPointSelector::Start | DimensionPointSelector::End,
        ) => !vertices.is_empty(),
        (
            IfccadEntityKind::SpatialPolyline {
                closed: false,
                vertices,
                ..
            },
            DimensionPointSelector::Start | DimensionPointSelector::End,
        ) => !vertices.is_empty(),
        _ => false,
    }
}
fn markers(v: &IfccadDimensionSettings) -> Vec<u64> {
    [&v.arrows.first, &v.arrows.second]
        .into_iter()
        .filter_map(|m| match m {
            DimensionMarker::LocalBlock { block } => Some(*block),
            _ => None,
        })
        .collect()
}
pub(super) fn validate_dimensions(d: &IfccadDocument) -> Result<(), IfccadReport> {
    let prefix = format!("/cad/d{}", d.drawing_id);
    let mut styles = BTreeMap::new();
    let mut names = BTreeSet::new();
    for s in &d.dimension_styles {
        let path = format!("{prefix}/dimensionStyle/{}", s.id.0);
        if styles.insert(s.id, s).is_some() {
            return Err(fail(&path, "duplicate DimensionStyle ID"));
        }
        if s.name.is_empty()
            || s.name.contains('\0')
            || !names.insert(crate::names::name_key(&s.name))
        {
            return Err(fail(&path, "invalid/colliding DimensionStyle name"));
        }
        settings(d, &path, &s.settings)?;
        for (kind, over) in &s.type_overrides {
            let mut v = s.settings.clone();
            over.apply(&mut v);
            settings(d, &format!("{path}/typeOverrides/{kind:?}"), &v)?;
        }
    }
    let owned = std::iter::once((IfccadScopeId::Layout(d.model.id), &d.model.entities))
        .chain(
            d.paper_layouts
                .iter()
                .map(|p| (IfccadScopeId::Layout(p.id), &p.entities)),
        )
        .chain(
            d.blocks
                .iter()
                .map(|b| (IfccadScopeId::BlockDefinition(b.id), &b.entities)),
        )
        .flat_map(|(owner, list)| list.iter().map(move |e| (e.id(), (owner, e))))
        .collect::<BTreeMap<_, _>>();
    let mut edges = BTreeMap::<u64, Vec<u64>>::new();
    for (owner, list) in d.blocks.iter().map(|b| (b.id, &b.entities)) {
        for e in list.iter().filter_map(IfccadEntity::as_native) {
            if let IfccadEntityKind::BlockInstance { definition_id, .. } = e.kind {
                edges.entry(owner).or_default().push(definition_id);
            }
        }
    }
    for (id, (owner, e)) in &owned {
        let Some(e) = e.as_native() else { continue };
        let IfccadEntityKind::Dimension(v) = &e.kind else {
            continue;
        };
        let path = format!("{prefix}/e{id}/ifccad::dimension");
        let frame = v.placement.coordinate_frame()?;
        validate_dimension_definition(&v.definition).map_err(|e| fail(&path, e))?;
        let (a, b) = v.definition.reference_points();
        for p in [a, b] {
            frame
                .try_to_scope_spatial_point(crate::geometry_kernel::Point3::new(p[0], p[1], p[2]))
                .map_err(|e| fail(&path, e))?;
        }
        validate_dimension_text_placement(&v.text_placement).map_err(|e| fail(&path, e))?;
        evaluate_dimension(&v.definition).map_err(|e| fail(&path, e))?;
        if let DimensionLabel::Template(parts) = &v.label {
            for p in parts {
                if let DimensionLabelPart::Literal(runs) = p {
                    crate::text::validate_text_runs(runs).map_err(|e| fail(&path, e.message))?;
                }
            }
        }
        let style = styles
            .get(&v.style_id)
            .ok_or_else(|| fail(&path, "missing local DimensionStyle"))?;
        let effective = resolve_dimension_settings(
            &style.settings,
            v.definition.kind(),
            &style.type_overrides,
            &v.overrides,
        );
        settings(d, &path, &effective)?;
        if let IfccadScopeId::BlockDefinition(block) = owner {
            edges.entry(*block).or_default().extend(markers(&effective));
            edges
                .entry(*block)
                .or_default()
                .extend(markers(&style.settings));
            for over in style.type_overrides.values() {
                let mut s = style.settings.clone();
                over.apply(&mut s);
                edges.entry(*block).or_default().extend(markers(&s));
            }
        }
        let mut roles = BTreeSet::new();
        for (i, a) in v.associations.iter().enumerate() {
            let where_ = format!("{path}/associations/{i}");
            if !roles.insert(a.role) {
                return Err(fail(&where_, "duplicate point role"));
            }
            let (source_owner, source) = owned
                .get(&a.source_entity_id)
                .ok_or_else(|| fail(&where_, "missing source entity"))?;
            if source_owner != owner
                || !source
                    .as_native()
                    .is_some_and(|s| eligible(&s.kind, a.selector))
            {
                return Err(fail(&where_, "source is ineligible or in another owner"));
            }
        }
    }
    for root in edges.keys() {
        let mut stack = vec![(*root, vec![*root])];
        while let Some((node, path)) = stack.pop() {
            for next in edges.get(&node).into_iter().flatten() {
                if path.contains(next) {
                    return Err(fail(
                        &format!("{prefix}/block/{root}"),
                        "block/render dependency cycle",
                    ));
                }
                let mut p = path.clone();
                p.push(*next);
                stack.push((*next, p));
            }
        }
    }
    Ok(())
}
