//! Inspect the validated owned graph without projecting parts to CAD entities.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn authored(graph: &Value) -> Value {
    let Some(nodes) = graph["data"].as_array() else {
        return json!([]);
    };
    let index: BTreeMap<_, _> = nodes
        .iter()
        .filter_map(|n| Some((n["path"].as_str()?, n)))
        .collect();
    let mut output = Vec::new();
    for node in nodes {
        let Some((role, kind)) = [
            ("ifccad::leader", "Leader"),
            ("ifccad::multiLeader", "MultiLeader"),
            ("ifccad::tolerance", "Tolerance"),
        ]
        .into_iter()
        .find(|(role, _)| node["attributes"].get(*role).is_some()) else {
            continue;
        };
        let mut pending: Vec<_> = node["children"]
            .as_object()
            .into_iter()
            .flat_map(|c| c.values())
            .filter_map(Value::as_str)
            .collect();
        let mut visited = BTreeSet::new();
        let mut owned = Vec::new();
        while let Some(path) = pending.pop() {
            if !visited.insert(path) {
                continue;
            }
            let Some(child) = index.get(path) else {
                continue;
            };
            pending.extend(
                child["children"]
                    .as_object()
                    .into_iter()
                    .flat_map(|c| c.values())
                    .filter_map(Value::as_str),
            );
            owned.push((*child).clone());
        }
        owned.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
        let graphics=node["children"]["graphics"].as_str().and_then(|p|index.get(p)).map(|picture|{
            let commands=picture["children"].as_object().map_or(0,|c|c.len());
            json!({"state":"stored","path":picture["path"],"origin":picture["attributes"]["ifccad::leaderGraphics"]["origin"],"commandCount":commands})
        }).unwrap_or_else(||json!({"state":"absent"}));
        output.push(json!({"path":node["path"],"kind":kind,"definition":node["attributes"][role],"ownedNodes":owned,"graphics":graphics,"displayEvaluation":"unassessed","boundsEvaluation":"unavailable"}));
    }
    json!(output)
}
