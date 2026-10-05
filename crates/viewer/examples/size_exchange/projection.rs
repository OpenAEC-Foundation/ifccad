use super::Result;
use ocdraw::ifccad::*;
use ocdraw_convert::opencadcodec::CadDocument;
use serde_json::{json, Value};

pub fn metadata() -> ifccad_convert::IfccadTargetMetadata {
    ifccad_convert::IfccadTargetMetadata {
        drawing_id: 1,
        header: IfccadHeader {
            id: "size-exchange".into(),
            data_version: "1".into(),
            author: "OpenAEC size experiment".into(),
            timestamp: "2026-10-02T00:00:00Z".into(),
        },
    }
}
pub fn snapshot(document: &CadDocument) -> Result<Value> {
    let native = ifccad_convert::cad_document_to_ifccad_document(
        document,
        metadata(),
        ifccad_convert::CadToIfccadOptions {
            loss_policy: ifccad_convert::IfccadLossPolicy::Reject,
        },
    )?;
    semantic(native.document())
}
pub fn semantic(d: &IfccadDocument) -> Result<Value> {
    validate_ifccad_document(d).map_err(|e| format!("{e:?}"))?;
    if !d.paper_layouts.is_empty() {
        return Err("paper outside current comparison contract".into());
    }
    let pattern_name = |id| {
        d.line_patterns
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .name
            .clone()
    };
    let entities = |values: &[IfccadEntity]| {
        values.iter().map(|e| {
            let layer = &d.layers.iter().find(|l| l.id == e.layer_id).unwrap().name;
            let geometry = match &e.kind {
                IfccadEntityKind::Viewport(_) => unreachable!("validated viewports belong to paper layouts, which this comparison rejects"),
                IfccadEntityKind::LineSegment { start, end } => json!({"kind":"line", "start":start,"end":end}),
                IfccadEntityKind::Circle { radius, placement } => json!({"kind":"circle","radius":radius,"placement":placement}),
                IfccadEntityKind::PlanarPolyline { vertices, closed, placement, line_pattern_generation } => json!({"kind":"polyline","vertices":vertices,"closed":closed,"placement":placement,"generation":line_pattern_generation}),
                IfccadEntityKind::BlockInstance { definition_id, transform } => json!({"kind":"insert","definition":d.blocks.iter().find(|b| b.id == *definition_id).unwrap().name,"transform":transform})
            };
            let mut appearance = serde_json::to_value(&e.appearance).unwrap();
            if let IfccadMode::Explicit(id) = e.appearance.line_pattern { appearance["linePattern"]["value"] = json!(pattern_name(id)); }
            json!({"layer":layer,"appearance":appearance,"scale":e.line_pattern_scale,"geometry":geometry})
        }).collect::<Vec<_>>()
    };
    let mut layers = d
        .layers
        .iter()
        .map(|l| {
            let mut a = serde_json::to_value(&l.appearance).unwrap();
            a["linePattern"] = json!(pattern_name(l.appearance.line_pattern));
            json!({"name":l.name,"appearance":a})
        })
        .collect::<Vec<_>>();
    let mut patterns = d
        .line_patterns
        .iter()
        .map(|p| json!({"name":p.name,"description":p.description,"pattern":p.pattern}))
        .collect::<Vec<_>>();
    let mut blocks = d.blocks.iter().map(|b| json!({"name":b.name,"base":b.base_point,"unit":b.insertion_unit,"entities":entities(&b.entities)})).collect::<Vec<_>>();
    for values in [&mut layers, &mut patterns, &mut blocks] {
        values.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    }
    Ok(
        json!({"unit":d.length_unit,"scale":d.line_pattern_scale,"layers":layers,"patterns":patterns,"model":entities(&d.model.entities),"blocks":blocks}),
    )
}
pub fn differences(expected: &Value, actual: &Value) -> Vec<Value> {
    fn visit(a: &Value, b: &Value, path: &str, out: &mut Vec<Value>) {
        if a == b {
            return;
        }
        match (a, b) {
            (Value::Object(a), Value::Object(b)) if a.keys().eq(b.keys()) => {
                for (key, value) in a {
                    visit(
                        value,
                        &b[key],
                        &format!("{path}/{}", key.replace('~', "~0").replace('/', "~1")),
                        out,
                    );
                }
            }
            (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    visit(a, b, &format!("{path}/{i}"), out);
                }
            }
            _ => {
                out.push(json!({"path":path,"expected":a,"actual":b,"numeric":a.is_number() && b.is_number()}));
            }
        }
    }
    let mut out = vec![];
    visit(expected, actual, "", &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::super::corpus::{generate, Case};
    use super::*;
    fn drawing(family: &str) -> IfccadDocument {
        let cad = generate(&Case {
            id: "test".into(),
            family: family.into(),
            count: 3,
        })
        .unwrap();
        ifccad_convert::cad_document_to_ifccad_document(&cad, metadata(), Default::default())
            .unwrap()
            .into_document()
    }
    #[test]
    fn fresh_ids_do_not_change_semantics() {
        let a = drawing("blocks");
        let mut b = a.clone();
        for layer in &mut b.layers {
            layer.id += 100;
        }
        for pattern in &mut b.line_patterns {
            pattern.id.0 += 100;
        }
        for layer in &mut b.layers {
            layer.appearance.line_pattern.0 += 100;
        }
        for block in &mut b.blocks {
            block.id += 100;
        }
        for entity in b
            .model
            .entities
            .iter_mut()
            .chain(b.blocks.iter_mut().flat_map(|b| &mut b.entities))
        {
            entity.id += 100;
            entity.layer_id += 100;
            if let IfccadMode::Explicit(id) = &mut entity.appearance.line_pattern {
                id.0 += 100;
            }
            if let IfccadEntityKind::BlockInstance { definition_id, .. } = &mut entity.kind {
                *definition_id += 100;
            }
        }
        b.id_counters.next_entity_id += 100;
        b.id_counters.next_layer_id += 100;
        b.id_counters.next_line_pattern_id += 100;
        b.id_counters.next_block_id += 100;
        assert_eq!(semantic(&a).unwrap(), semantic(&b).unwrap());
    }
    #[test]
    fn changed_order_and_block_target_are_not_equal() {
        let a = drawing("blocks");
        let mut b = a.clone();
        b.model.entities.swap(0, 1);
        assert_ne!(semantic(&a).unwrap(), semantic(&b).unwrap());
        let a = drawing("blocks");
        let mut b = a.clone();
        b.blocks.last_mut().unwrap().base_point[0] += 1.;
        assert_ne!(semantic(&a).unwrap(), semantic(&b).unwrap());
        b = a.clone();
        if let IfccadEntityKind::BlockInstance { definition_id, .. } = &mut b.model.entities[0].kind
        {
            *definition_id = b.blocks[0].id;
        }
        assert_ne!(semantic(&a).unwrap(), semantic(&b).unwrap());
    }
    #[test]
    fn unused_patterns_and_definitions_are_compared() {
        let a = drawing("patterns");
        let mut b = a.clone();
        b.line_patterns.last_mut().unwrap().description = Some("changed unused description".into());
        assert_ne!(semantic(&a).unwrap(), semantic(&b).unwrap());
    }
    #[test]
    fn unknown_or_residual_semantics_fail_snapshot() {
        let mut cad = generate(&Case {
            id: "test".into(),
            family: "line".into(),
            count: 1,
        })
        .unwrap();
        assert!(snapshot(&cad).is_ok());
        cad.header.project_name = "do not hide this".into();
        assert!(snapshot(&cad).is_err());
    }
    #[test]
    fn codec_numeric_difference_is_located_and_not_ignored() {
        let diffs = differences(
            &json!({"rotation":1.570796326794893}),
            &json!({"rotation":1.5707963267948932}),
        );
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0]["path"], "/rotation");
        assert_eq!(diffs[0]["numeric"], true);
        assert_eq!(
            differences(&json!([1, 2]), &json!([1, 2, 3]))[0]["numeric"],
            false
        );
    }
}
