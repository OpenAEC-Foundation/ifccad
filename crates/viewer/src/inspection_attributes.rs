//! Stored attribute records and their owners; no evaluation or tag matching.
use serde_json::{json, Value};
pub(crate) fn authored(graph: &Value) -> Value {
    let nodes = graph["data"].as_array().expect("validated graph");
    let mut definitions = vec![];
    let mut values = vec![];
    for node in nodes {
        for (role, rows) in [
            ("ifccad::attributeDefinition", &mut definitions),
            ("ifccad::blockAttribute", &mut values),
        ] {
            if let Some(v) = node["attributes"].get(role) {
                let owner = nodes.iter().find_map(|n| {
                    n["children"]
                        .as_object()?
                        .iter()
                        .find(|(k, p)| k.parse::<u64>().is_ok() && **p == node["path"])
                        .map(|(order, _)| (n, order))
                });
                let (owner, order) = owner.expect("validated ownership");
                rows.push(json!({"path":node["path"],"ownerPath":owner["path"],"order":order.parse::<u64>().expect("numeric child order"),"value":v,"common":node["attributes"]["ifccad::entity"],"placement":node["attributes"]["ifccad::geom::placement"]}));
            }
        }
    }
    json!({"definitions":definitions,"values":values,"evaluation":"notPerformed"})
}
