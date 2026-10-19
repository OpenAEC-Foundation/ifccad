use super::{field_validation::logical_error, *};
use crate::dimension::*;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

fn settings(doc: &OcdrawDocument, path: &str, v: &OcdrawDimensionSettings) -> Vec<LogicalError> {
    settings_in_context(
        path,
        v,
        |s| doc.text_styles.iter().any(|t| t.id == s),
        |b| doc.block_definitions.iter().any(|d| d.scope_id == b),
        |p| doc.line_patterns.iter().any(|d| d.id == p),
    )
}
pub(crate) fn settings_in_context(
    path: &str,
    v: &OcdrawDimensionSettings,
    text: impl Fn(OcdrawTextStyleId) -> bool,
    block: impl Fn(u32) -> bool,
    pattern: impl Fn(LinePatternId) -> bool,
) -> Vec<LogicalError> {
    let errors = RefCell::new(Vec::new());
    if let Err(e) = validate_dimension_settings(v) {
        errors.borrow_mut().push(logical_error(
            "DIMENSION_VALUE",
            format!("{path}/{}", e.field),
            &e.to_string(),
        ));
    }
    let _ = v.map_references(
        |s| {
            if !text(*s) {
                errors.borrow_mut().push(logical_error(
                    "DIMENSION_TEXT_STYLE",
                    format!("{path}/text/style"),
                    "missing drawing-local TextStyle",
                ));
            }
            *s
        },
        |b| {
            if !block(*b) {
                errors.borrow_mut().push(logical_error(
                    "DIMENSION_MARKER_BLOCK",
                    path,
                    "missing local marker block",
                ));
            }
            *b
        },
        |c| {
            if !super::field_validation::color(c) {
                errors.borrow_mut().push(logical_error(
                    "DIMENSION_COLOR",
                    path,
                    "invalid concrete color",
                ));
            }
            c.clone()
        },
        |p| {
            if !pattern(*p) {
                errors.borrow_mut().push(logical_error(
                    "DIMENSION_LINE_PATTERN",
                    path,
                    "missing local line pattern",
                ));
            }
            *p
        },
    );
    errors.into_inner()
}
pub(crate) fn entity_values(e: &DrawingDimensionEntity) -> Vec<LogicalError> {
    let mut errors = Vec::new();
    let path = format!("/entities/{}", e.id);
    for check in [
        validate_dimension_definition(&e.definition),
        validate_dimension_text_placement(&e.text_placement),
        evaluate_dimension(&e.definition).map(|_| ()),
    ] {
        if let Err(x) = check {
            errors.push(logical_error(
                "DIMENSION_VALUE",
                format!("{path}/{}", x.field),
                &x.to_string(),
            ));
        }
    }
    if !super::field_validation::appearance(&e.appearance) {
        errors.push(logical_error(
            "ENTITY_APPEARANCE",
            &path,
            "invalid Dimension appearance",
        ));
    }
    if let DimensionLabel::Template(parts) = &e.label {
        for p in parts {
            if let DimensionLabelPart::Literal(runs) = p {
                if let Err(x) = crate::text::validate_text_runs(runs) {
                    errors.push(logical_error("DIMENSION_LABEL", &path, x.message));
                }
            }
        }
    }
    errors
}
fn source_eligible(g: &EntityGeometry, s: DimensionPointSelector) -> bool {
    match (g, s) {
        (EntityGeometry::Point { .. }, DimensionPointSelector::Position) => true,
        (
            EntityGeometry::Line { .. } | EntityGeometry::Arc { .. },
            DimensionPointSelector::Start | DimensionPointSelector::End,
        ) => true,
        (
            EntityGeometry::Ellipse { arc: Some(_), .. },
            DimensionPointSelector::Start | DimensionPointSelector::End,
        ) => true,
        (
            EntityGeometry::Circle { .. }
            | EntityGeometry::Arc { .. }
            | EntityGeometry::Ellipse { .. },
            DimensionPointSelector::Center,
        ) => true,
        (
            EntityGeometry::PlanarPolyline { vertices, .. },
            DimensionPointSelector::Vertex { index },
        ) => (index as usize) < vertices.len(),
        (
            EntityGeometry::SpatialPolyline { vertices, .. },
            DimensionPointSelector::Vertex { index },
        ) => (index as usize) < vertices.len(),
        (
            EntityGeometry::PlanarPolyline {
                closed: false,
                vertices,
                ..
            },
            DimensionPointSelector::Start | DimensionPointSelector::End,
        ) => !vertices.is_empty(),
        (
            EntityGeometry::SpatialPolyline {
                closed: false,
                vertices,
                ..
            },
            DimensionPointSelector::Start | DimensionPointSelector::End,
        ) => !vertices.is_empty(),
        _ => false,
    }
}
fn marker_targets(v: &OcdrawDimensionSettings) -> Vec<u32> {
    [&v.arrows.first, &v.arrows.second]
        .into_iter()
        .filter_map(|m| match m {
            DimensionMarker::LocalBlock { block } => Some(*block),
            _ => None,
        })
        .collect()
}

pub(crate) fn validate_dimensions(doc: &OcdrawDocument) -> Vec<LogicalError> {
    let mut errors = Vec::new();
    let mut styles = BTreeMap::new();
    let mut names = BTreeSet::new();
    for (i, s) in doc.dimension_styles.iter().enumerate() {
        let path = format!("/dimensionStyles/{i}");
        if styles.insert(s.id, s).is_some() {
            errors.push(logical_error(
                "DIMENSION_STYLE_ID",
                &path,
                "duplicate style ID",
            ));
        }
        if s.name.is_empty()
            || s.name.contains('\0')
            || !names.insert(crate::ocdraw::names::name_key(&s.name))
        {
            errors.push(logical_error(
                "DIMENSION_STYLE_NAME",
                &path,
                "style names must be nonempty, NUL-free and casefold-unique",
            ));
        }
        errors.extend(settings(doc, &format!("{path}/settings"), &s.settings));
        for (kind, overrides) in &s.type_overrides {
            let mut effective = s.settings.clone();
            overrides.apply(&mut effective);
            errors.extend(settings(
                doc,
                &format!("{path}/typeOverrides/{kind:?}"),
                &effective,
            ));
        }
    }
    if doc.next_dimension_style_id == 0 || styles.keys().any(|s| s.0 >= doc.next_dimension_style_id)
    {
        errors.push(logical_error(
            "DIMENSION_STYLE_WATERMARK",
            "/header/nextDimensionStyleId",
            "watermark must be positive and exceed all live IDs",
        ));
    }
    let owners = doc
        .scopes
        .iter()
        .flat_map(|s| s.entities.iter().map(move |e| (*e, s.id)))
        .collect::<BTreeMap<_, _>>();
    let sources = doc
        .geometric_entities
        .iter()
        .map(|e| (e.id, &e.geometry))
        .collect::<BTreeMap<_, _>>();
    let mut block_edges = BTreeMap::<u32, Vec<u32>>::new();
    for e in &doc.geometric_entities {
        if let EntityGeometry::BlockInstance {
            definition_scope_id,
            ..
        } = e.geometry
        {
            if let Some(owner) = owners.get(&e.id) {
                block_edges
                    .entry(*owner)
                    .or_default()
                    .push(definition_scope_id);
            }
        }
    }
    for e in &doc.dimension_entities {
        let path = format!("/entities/{}", e.id);
        errors.extend(entity_values(e));
        if let Some(style) = styles.get(&e.style_id) {
            let effective = resolve_dimension_settings(
                &style.settings,
                e.definition.kind(),
                &style.type_overrides,
                &e.overrides,
            );
            errors.extend(settings(doc, &format!("{path}/overrides"), &effective));
            if let Some(owner) = owners.get(&e.id) {
                block_edges
                    .entry(*owner)
                    .or_default()
                    .extend(marker_targets(&effective));
                block_edges
                    .entry(*owner)
                    .or_default()
                    .extend(marker_targets(&style.settings));
                for over in style.type_overrides.values() {
                    let mut v = style.settings.clone();
                    over.apply(&mut v);
                    block_edges
                        .entry(*owner)
                        .or_default()
                        .extend(marker_targets(&v));
                }
            }
        } else {
            errors.push(logical_error(
                "DIMENSION_STYLE_REFERENCE",
                &path,
                "missing drawing-local DimensionStyle",
            ));
        }
        let mut roles = BTreeSet::new();
        for (i, a) in e.associations.iter().enumerate() {
            let location = format!("{path}/associations/{i}");
            if !roles.insert(a.role) {
                errors.push(logical_error(
                    "DIMENSION_ASSOCIATION_ROLE",
                    &location,
                    "duplicate point role",
                ));
            }
            if !sources
                .get(&a.source_entity_id)
                .is_some_and(|g| source_eligible(g, a.selector))
                || !owners.contains_key(&a.source_entity_id)
                || owners.get(&a.source_entity_id) != owners.get(&e.id)
            {
                errors.push(logical_error(
                    "DIMENSION_SOURCE_REFERENCE",
                    location,
                    "source must resolve to eligible geometry in the same owner",
                ));
            }
        }
    }
    for root in block_edges.keys() {
        let mut stack = vec![(*root, vec![*root])];
        while let Some((node, path)) = stack.pop() {
            for target in block_edges.get(&node).into_iter().flatten() {
                if path.contains(target) {
                    errors.push(logical_error(
                        "DIMENSION_BLOCK_CYCLE",
                        format!("/scopes/{root}"),
                        "block/render dependency cycle",
                    ));
                    stack.clear();
                    break;
                }
                let mut next = path.clone();
                next.push(*target);
                stack.push((*target, next));
            }
        }
    }
    errors
}
