use crate::{IfccadGeometryEntitySource, IfccadGeometryOwner};
use ocdraw::ifccad::*;

type Context = (Option<u64>, Option<IfccadLinePatternId>);
pub(crate) fn record(doc: &IfccadDocument, out: &mut Vec<IfccadGeometryEntitySource>) {
    fn walk(
        doc: &IfccadDocument,
        owner: IfccadGeometryOwner,
        entities: &[IfccadEntity],
        parent: Option<Context>,
        path: &mut Vec<IfccadGeometryEntitySource>,
        out: &mut Vec<IfccadGeometryEntitySource>,
    ) {
        for e in entities {
            let Some(e) = e.as_native() else { continue };
            let layer = if matches!(owner, IfccadGeometryOwner::BlockDefinition(_))
                && doc
                    .layers
                    .iter()
                    .find(|l| l.id == e.layer_id)
                    .is_some_and(|l| l.name == "0")
            {
                parent.and_then(|p| p.0)
            } else {
                Some(e.layer_id)
            };
            let pattern = match e.appearance.line_pattern {
                IfccadMode::Explicit(id) => Some(id),
                IfccadMode::ByBlock => parent.and_then(|p| p.1),
                IfccadMode::ByLayer => layer.and_then(|id| {
                    doc.layers
                        .iter()
                        .find(|l| l.id == id)
                        .map(|l| l.appearance.line_pattern)
                }),
            };
            let identity = IfccadGeometryEntitySource::NativeEntity {
                owner,
                entity_id: e.id,
            };
            if let IfccadEntityKind::BlockInstance { definition_id, .. } = e.kind {
                if let Some(block) = doc.blocks.iter().find(|b| b.id == definition_id) {
                    path.push(identity);
                    walk(
                        doc,
                        IfccadGeometryOwner::BlockDefinition(block.id),
                        &block.entities,
                        Some((layer, pattern)),
                        path,
                        out,
                    );
                    path.pop();
                }
            } else if matches!(
                e.kind,
                IfccadEntityKind::LineSegment { .. }
                    | IfccadEntityKind::Circle { .. }
                    | IfccadEntityKind::Arc { .. }
                    | IfccadEntityKind::Ellipse { .. }
                    | IfccadEntityKind::EllipseArc { .. }
                    | IfccadEntityKind::PlanarPolyline { .. }
                    | IfccadEntityKind::SpatialPolyline { .. }
            ) && doc
                .line_patterns
                .iter()
                .filter(|p| pattern.is_none_or(|id| p.id == id))
                .any(|p| {
                    p.pattern.iter().any(|e| match &e.marker {
                        Some(IfccadLinePatternMarker::Text { text, .. }) => !text.is_empty(),
                        Some(IfccadLinePatternMarker::Symbol { .. }) => true,
                        None => false,
                    })
                })
            {
                out.push(if path.is_empty() {
                    identity
                } else {
                    IfccadGeometryEntitySource::BlockOccurrence {
                        path: path.clone(),
                        leaf: Box::new(identity),
                    }
                });
            }
        }
    }
    walk(
        doc,
        IfccadGeometryOwner::ModelLayout(doc.model.id),
        &doc.model.entities,
        None,
        &mut Vec::new(),
        out,
    );
    for p in &doc.paper_layouts {
        walk(
            doc,
            IfccadGeometryOwner::PaperLayout(p.id),
            &p.entities,
            None,
            &mut Vec::new(),
            out,
        );
    }
    for b in &doc.blocks {
        walk(
            doc,
            IfccadGeometryOwner::BlockDefinition(b.id),
            &b.entities,
            None,
            &mut Vec::new(),
            out,
        );
    }
}
