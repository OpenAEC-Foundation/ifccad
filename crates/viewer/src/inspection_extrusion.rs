//! Authored sweep parameters in the owning geometry scope, not rendered surfaces.
use serde_json::{json, Value};

pub(crate) fn authored(graph: &Value, unit: &Value) -> Value {
    let paper_entities = graph["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|node| node["attributes"]["ifccad::layout"]["kind"] == "Paper")
        .flat_map(|node| {
            node["children"]
                .as_object()
                .into_iter()
                .flat_map(|v| v.values())
        })
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let rows = graph["data"].as_array().into_iter().flatten().filter_map(|node| {
        let attrs = &node["attributes"];
        let (kind, body) = ["point", "lineSegment", "circle", "arc", "planarPolyline"]
            .into_iter().find_map(|kind| attrs.get(format!("ifccad::geom::{kind}")) .map(|body| (kind, body)))?;
        let direction = if kind == "lineSegment" {
            body.get("extrusionDirection").cloned().unwrap_or(json!([0.,0.,1.]))
        } else {
            let frame = &attrs["ifccad::geom::placement"];
            let axis = |name: &str, default: [f64;3]| -> [f64;3] {
                frame.get(name).map(|v|std::array::from_fn(|i|v[i].as_f64().expect("validated frame"))).unwrap_or(default)
            };
            let x = axis("xAxis",[1.,0.,0.]);
            let y = axis("yAxis",[0.,1.,0.]);
            json!(std::array::from_fn::<_,3,_>(|i|x[(i+1)%3]*y[(i+2)%3]-x[(i+2)%3]*y[(i+1)%3]))
        };
        let path = node["path"].as_str().expect("validated node path");
        let is_paper = paper_entities.iter().any(|owner|path == *owner || path.strip_prefix(*owner).is_some_and(|suffix|suffix.starts_with('/')));
        Some(json!({"path":path,"thickness":body.get("thickness").cloned().unwrap_or(json!(0.)),"direction":direction,"unit":if is_paper {&Value::Null} else {unit},"coordinateDomain":"owningGeometryScope","displayEvaluation":"unassessed"}))
    }).collect::<Vec<_>>();
    json!(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paper_uses_its_own_numeric_domain_including_saved_parts() {
        let graph = json!({"data":[
            {"path":"/cad/d1/layout/2","children":{"e0":"/cad/d1/e2"},"attributes":{"ifccad::layout":{"kind":"Paper"}}},
            {"path":"/cad/d1/e2/graphics/p0","attributes":{"ifccad::geom::point":{}}},
            {"path":"/cad/d1/e20","attributes":{"ifccad::geom::lineSegment":{}}}
        ]});
        let rows = authored(&graph, &json!("m"));
        assert!(rows[0]["unit"].is_null());
        assert_eq!(rows[1]["unit"], "m");
        assert_eq!(rows[1]["thickness"], 0.0);
        assert_eq!(rows[1]["direction"], json!([0., 0., 1.]));
    }
}
