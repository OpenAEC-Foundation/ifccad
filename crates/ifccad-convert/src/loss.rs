use crate::{diagnostics::diagnostic, IfccadDiagnostic};
use std::collections::BTreeSet;

pub(crate) fn native_defaults(raw: &serde_json::Value) -> serde_json::Value {
    let mut raw = raw.clone();
    for n in raw["data"].as_array_mut().expect("validated graph") {
        if let Some(attrs) = n
            .get_mut("attributes")
            .and_then(serde_json::Value::as_object_mut)
        {
            for key in ["ifccad::drawing", "ifccad::entity"] {
                if let Some(value) = attrs
                    .get_mut(key)
                    .and_then(serde_json::Value::as_object_mut)
                {
                    value
                        .entry("linePatternScale")
                        .or_insert(serde_json::json!(1.0));
                }
            }
            if let Some(drawing) = attrs
                .get_mut("ifccad::drawing")
                .and_then(serde_json::Value::as_object_mut)
            {
                drawing.entry("nextUcsId").or_insert(serde_json::json!(1));
                drawing
                    .entry("nextModelWindowId")
                    .or_insert(serde_json::json!(1));
                if drawing
                    .get("modelWindows")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|a| a.is_empty())
                {
                    drawing.remove("modelWindows");
                }
            }
            for key in [
                "ifccad::paperCanvas",
                "ifccad::viewportWorkspace",
                "ifccad::modelWindow",
            ] {
                if let Some(value) = attrs
                    .get_mut(key)
                    .and_then(serde_json::Value::as_object_mut)
                {
                    value
                        .entry("useStoredUcs")
                        .or_insert(serde_json::json!(true));
                }
            }
            if let Some(value) = attrs
                .get_mut("ifccad::geom::planarPolyline")
                .and_then(serde_json::Value::as_object_mut)
            {
                value
                    .entry("linePatternGeneration")
                    .or_insert(serde_json::json!("perSegment"));
            }
            if let Some(value) = attrs
                .get_mut("ifccad::geom::planarPolyline")
                .and_then(serde_json::Value::as_object_mut)
            {
                if value
                    .get("bulges")
                    .and_then(|v| v.as_array())
                    .is_some_and(|a| a.iter().all(|v| v.as_f64() == Some(0.)))
                {
                    value.remove("bulges");
                }
            }
            if let Some(value) = attrs
                .get_mut("ifccad::geom::spatialPolyline")
                .and_then(serde_json::Value::as_object_mut)
            {
                value
                    .entry("linePatternGeneration")
                    .or_insert(serde_json::json!("perSegment"));
            }
            if let Some(layers) = attrs
                .get_mut("ifccad::viewport")
                .and_then(|value| value.get_mut("frozenLayers"))
                .and_then(serde_json::Value::as_array_mut)
            {
                // Native layer references form a set. Compare them in the
                // writer's exact numeric order without rounding uint64 IDs.
                layers.sort_by_key(|value| {
                    value
                        .as_str()
                        .and_then(|path| path.rsplit('/').next())
                        .and_then(|id| id.parse::<u64>().ok())
                });
            }
        }
    }
    raw
}

struct Definition {
    id: u64,
    location: String,
    entities: Vec<String>,
}
struct Instance {
    location: String,
    target: u64,
    owner: Option<u64>,
}

fn propagate(defs: Vec<Definition>, instances: Vec<Instance>, issues: &mut Vec<IfccadDiagnostic>) {
    fn at(issue: &str, location: &str) -> bool {
        issue == location
            || issue
                .strip_prefix(location)
                .is_some_and(|s| s.starts_with('.'))
    }
    let mut lost: BTreeSet<_> = defs
        .iter()
        .filter(|def| {
            issues.iter().any(|d| {
                d.is_semantic_loss()
                    && (at(&d.location, &def.location)
                        || def.entities.iter().any(|e| at(&d.location, e)))
            })
        })
        .map(|d| d.id)
        .collect();
    loop {
        let parents: Vec<_> = instances
            .iter()
            .filter(|i| lost.contains(&i.target))
            .filter_map(|i| i.owner)
            .collect();
        let before = lost.len();
        lost.extend(parents);
        if lost.len() == before {
            break;
        }
    }
    for i in instances {
        if lost.contains(&i.target) {
            issues.push(diagnostic("block-content-loss", i.location,
                "referenced block definition has omitted or modified content, directly or through a nested block"));
        }
    }
}

pub(crate) fn from_cad(
    doc: &opencadcodec::CadDocument,
    blocks: &[opencadcodec::Handle],
    model: &[opencadcodec::Handle],
    papers: &[&[opencadcodec::Handle]],
    issues: &mut Vec<IfccadDiagnostic>,
) {
    let mut defs = Vec::new();
    let mut instances = Vec::new();
    let owners = std::iter::once((None, model))
        .chain(papers.iter().map(|entities| (None, *entities)))
        .chain(blocks.iter().map(|h| {
            let b = doc.block_records.iter().find(|b| b.handle == *h).unwrap();
            defs.push(Definition {
                id: h.value(),
                location: format!("block/{}", b.name),
                entities: b
                    .entity_handles
                    .iter()
                    .map(|h| format!("entity/{h}"))
                    .collect(),
            });
            (Some(h.value()), b.entity_handles.as_slice())
        }));
    for (owner, entities) in owners {
        for h in entities {
            if let Some(opencadcodec::EntityType::Insert(i)) = doc.get_entity(*h) {
                instances.push(Instance {
                    owner,
                    location: format!("entity/{h}"),
                    target: doc.block_records.get(&i.block_name).unwrap().handle.value(),
                });
            }
        }
    }
    propagate(defs, instances, issues);
}

pub(crate) fn to_cad(doc: &ocdraw::ifccad::IfccadDocument, issues: &mut Vec<IfccadDiagnostic>) {
    let defs = doc
        .blocks
        .iter()
        .map(|b| Definition {
            id: b.id,
            location: format!("block/{}", b.id),
            entities: b
                .entities
                .iter()
                .map(|e| format!("entity/{}", e.id()))
                .collect(),
        })
        .collect();
    let mut instances = Vec::new();
    for (owner, entities) in std::iter::once((None, doc.model.entities.as_slice()))
        .chain(
            doc.paper_layouts
                .iter()
                .map(|p| (None, p.entities.as_slice())),
        )
        .chain(
            doc.blocks
                .iter()
                .map(|b| (Some(b.id), b.entities.as_slice())),
        )
    {
        for e in entities
            .iter()
            .filter_map(ocdraw::ifccad::IfccadEntity::as_native)
        {
            if let ocdraw::ifccad::IfccadEntityKind::BlockInstance { definition_id, .. } = e.kind {
                instances.push(Instance {
                    owner,
                    location: format!("entity/{}", e.id),
                    target: definition_id,
                });
            }
        }
    }
    propagate(defs, instances, issues);
}

/// Compare known geometry fields exactly, while permitting foreign extensions.
pub(crate) fn precision(
    raw: &serde_json::Value,
    canonical: &serde_json::Value,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    use serde_json::Value;
    fn projected(actual: &Value, expected: &Value) -> bool {
        match expected {
            Value::Object(fields) => fields
                .iter()
                .all(|(k, v)| actual.get(k).is_some_and(|a| projected(a, v))),
            Value::Array(items) => actual.as_array().is_some_and(|a| {
                a.len() == items.len() && a.iter().zip(items).all(|(a, b)| projected(a, b))
            }),
            _ => crate::source::same_graph(actual, expected),
        }
    }
    let expected: std::collections::BTreeMap<_, _> = canonical["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["path"].as_str().unwrap(), n))
        .collect();
    for node in raw["data"].as_array().unwrap() {
        let Some(path) = node["path"].as_str() else {
            continue;
        };
        let Some(reference) = expected.get(path) else {
            continue;
        };
        let Some(attrs) = reference["attributes"].as_object() else {
            continue;
        };
        for (key, value) in attrs {
            if key == "ifccad::layout" {
                for field in ["bounds", "paper"] {
                    if let Some(expected) = value.get(field) {
                        if !projected(&node["attributes"][key][field], expected) {
                            issues.push(diagnostic(
                                "precision",
                                format!("{path}.{key}.{field}"),
                                "typed layout projection changes an exact source number",
                            ));
                        }
                    }
                }
            }
            if key == "ifccad::viewport"
                && ["frame", "view"]
                    .iter()
                    .any(|field| !projected(&node["attributes"][key][field], &value[field]))
            {
                issues.push(diagnostic(
                    "precision",
                    format!("{path}.{key}"),
                    "typed viewport projection changes an exact source number",
                ));
            }
            if (key.starts_with("ifccad::geom::")
                || key == "ifccad::linePattern"
                || key == "ifccad::blockInstance"
                || key == "ifccad::blockDefinition"
                || matches!(
                    key.as_str(),
                    "ifccad::ucsDefinition"
                        | "ifccad::modelWindow"
                        | "ifccad::paperCanvas"
                        | "ifccad::viewportWorkspace"
                ))
                && !projected(&node["attributes"][key], value)
            {
                issues.push(diagnostic("precision", format!("{path}.{key}"), "typed native projection changes an exact source value; scalar approximation is rejected under both policies"));
            }
            if (key == "ifccad::drawing" || key == "ifccad::entity")
                && !projected(
                    &node["attributes"][key]["linePatternScale"],
                    &value["linePatternScale"],
                )
            {
                issues.push(diagnostic(
                    "precision",
                    format!("{path}.{key}.linePatternScale"),
                    "typed line pattern scale changes an exact source number",
                ));
            }
        }
    }
}
