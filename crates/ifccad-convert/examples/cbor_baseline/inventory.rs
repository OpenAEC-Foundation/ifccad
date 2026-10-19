use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType};
use serde_json::{json, Value};
use std::collections::BTreeMap;
fn add(m: &mut BTreeMap<String, usize>, name: &str, n: usize) {
    *m.entry(name.into()).or_default() += n;
}
pub fn native(d: &IfccadDocument) -> Value {
    let mut kinds = BTreeMap::new();
    let mut owners = BTreeMap::new();
    let mut vertices = 0;
    let mut loops = 0;
    let mut text_chars = 0;
    let mut sources = 0;
    for (owner, entities) in std::iter::once(("Model".into(), &d.model.entities))
        .chain(
            d.paper_layouts
                .iter()
                .map(|p| (format!("Paper:{}:{}", p.id, p.name), &p.entities)),
        )
        .chain(
            d.blocks
                .iter()
                .map(|b| (format!("Block:{}:{}", b.id, b.name), &b.entities)),
        )
    {
        add(&mut owners, &owner, entities.len());
        for entity in entities {
            let name = match entity.as_native().map(|e| &e.kind) {
                None => "Opaque",
                Some(IfccadEntityKind::LineSegment { .. }) => "Line",
                Some(IfccadEntityKind::Point { .. }) => "Point",
                Some(IfccadEntityKind::Circle { .. }) => "Circle",
                Some(IfccadEntityKind::Arc { .. }) => "Arc",
                Some(IfccadEntityKind::Ellipse { .. }) => "Ellipse",
                Some(IfccadEntityKind::EllipseArc { .. }) => "EllipseArc",
                Some(IfccadEntityKind::PlanarPolyline { vertices: v, .. }) => {
                    vertices += v.len();
                    "PlanarPolyline"
                }
                Some(IfccadEntityKind::SpatialPolyline { vertices: v, .. }) => {
                    vertices += v.len();
                    "SpatialPolyline"
                }
                Some(IfccadEntityKind::BlockInstance { .. }) => "BlockInstance",
                Some(IfccadEntityKind::Viewport(_)) => "Viewport",
                Some(IfccadEntityKind::Dimension(_)) => "Dimension",
                Some(IfccadEntityKind::Text(t)) => {
                    text_chars += t
                        .content
                        .iter()
                        .map(|r| r.text.chars().count())
                        .sum::<usize>();
                    "Text"
                }
                Some(IfccadEntityKind::MText(t)) => {
                    for p in &t.content {
                        for i in &p.inlines {
                            if let ocdraw::text::MTextInline::Run { text, .. } = i {
                                text_chars += text.chars().count();
                            }
                        }
                    }
                    "MText"
                }
                Some(IfccadEntityKind::Hatch(h)) => {
                    loops += h.loops.len();
                    sources += h
                        .loops
                        .iter()
                        .map(|l| l.source_entity_ids.len())
                        .sum::<usize>();
                    match h.fill {
                        ocdraw::geometry_kernel::hatch::HatchFill::Solid => "HatchSolid",
                        ocdraw::geometry_kernel::hatch::HatchFill::LinePattern(_) => "HatchPattern",
                        ocdraw::geometry_kernel::hatch::HatchFill::Gradient => "HatchGradient",
                    }
                }
            };
            add(&mut kinds, name, 1);
        }
    }
    let payload = d.preservation.as_ref().map_or(0, |p| {
        p.records
            .iter()
            .map(|r| r.payload.bytes.len())
            .sum::<usize>()
    });
    let baseline = d.preservation.as_ref().map_or(0, |p| {
        p.records
            .iter()
            .flat_map(|r| &r.conditions)
            .map(|c| c.baseline.len())
            .sum::<usize>()
    });
    json!({"kinds":kinds,"owners":owners,"storedEntities":owners.values().sum::<usize>(),"polylineVertices":vertices,"hatchLoops":loops,"hatchSourceReferences":sources,"textCharacters":text_chars,"layers":d.layers.len(),"blocks":d.blocks.len(),"paperLayouts":d.paper_layouts.len(),"textStyles":d.text_styles.len(),"dimensionStyles":d.dimension_styles.len(),"linePatterns":d.line_patterns.len(),"lengthUnit":d.length_unit,"preservationPayloadBytes":payload,"preservationBaselineBytes":baseline})
}
pub fn cad(d: &CadDocument) -> Value {
    let mut kinds = BTreeMap::new();
    let mut owners = BTreeMap::new();
    let mut vertices = 0;
    for e in d.entities() {
        add(&mut kinds, e.as_entity().entity_type(), 1);
        let owner = d
            .block_records
            .iter()
            .find(|b| b.handle == e.common().owner_handle)
            .map(|b| b.name.as_str())
            .unwrap_or("unresolved");
        add(&mut owners, owner, 1);
        if let EntityType::LwPolyline(p) = e {
            vertices += p.vertices.len();
        }
    }
    json!({"kinds":kinds,"owners":owners,"storedEntities":owners.values().sum::<usize>(),"polylineVertices":vertices,"blocksIncludingScaffolds":d.block_records.len(),"layers":d.layers.len(),"lineTypes":d.line_types.len(),"excludesStructuralBlockMarkers":true})
}
