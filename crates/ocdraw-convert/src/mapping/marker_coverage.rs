use crate::OcdrawGeometryEntitySource;
use ocdraw::ocdraw::*;

/// Glyph evidence is independent of the numerical proof for the host curve.
pub(crate) fn record(doc: &OcdrawDocument, out: &mut Vec<OcdrawGeometryEntitySource>) {
    let complex = |id: Option<LinePatternId>| {
        doc.line_patterns
            .iter()
            .filter(|p| id.is_none_or(|id| p.id == id))
            .any(|p| {
                p.pattern.iter().any(|e| match &e.marker {
                    Some(DrawingLinePatternMarker::Text { text, .. }) => !text.is_empty(),
                    Some(DrawingLinePatternMarker::Symbol { .. }) => true,
                    None => false,
                })
            })
    };
    let mut stack = doc
        .scopes
        .iter()
        .flat_map(|s| {
            s.entities.iter().map(move |id| {
                (
                    s.id,
                    *id,
                    None::<(Option<u32>, Option<LinePatternId>)>,
                    Vec::new(),
                )
            })
        })
        .collect::<Vec<_>>();
    while let Some((scope, id, parent, path)) = stack.pop() {
        let Some(entity) = doc.geometric_entities.iter().find(|e| e.id == id) else {
            continue;
        };
        let in_definition = doc.block_definitions.iter().any(|b| b.scope_id == scope);
        let layer = if in_definition
            && doc
                .layers
                .iter()
                .find(|l| l.id == entity.layer_id)
                .is_some_and(|l| l.name == "0")
        {
            parent.and_then(|p| p.0)
        } else {
            Some(entity.layer_id)
        };
        let pattern = match entity.appearance.line_pattern {
            AppearanceSelection::Explicit(id) => Some(id),
            AppearanceSelection::ByBlock => parent.and_then(|p| p.1),
            AppearanceSelection::ByLayer => layer.and_then(|id| {
                doc.layers
                    .iter()
                    .find(|l| l.id == id)
                    .map(|l| l.line_pattern_id)
            }),
        };
        let source = OcdrawGeometryEntitySource::DrawingEntity {
            scope_id: scope,
            entity_id: id,
        };
        if let DrawingGeometry::BlockInstance {
            definition_scope_id,
            ..
        } = entity.geometry
        {
            let mut next = path;
            next.push(source);
            if let Some(definition) = doc.scopes.iter().find(|s| s.id == definition_scope_id) {
                for child in &definition.entities {
                    stack.push((
                        definition_scope_id,
                        *child,
                        Some((layer, pattern)),
                        next.clone(),
                    ));
                }
            }
        } else if !matches!(entity.geometry, DrawingGeometry::Point { .. }) && complex(pattern) {
            out.push(if path.is_empty() {
                source
            } else {
                OcdrawGeometryEntitySource::BlockOccurrence {
                    path,
                    leaf: Box::new(source),
                }
            });
        }
    }
}
