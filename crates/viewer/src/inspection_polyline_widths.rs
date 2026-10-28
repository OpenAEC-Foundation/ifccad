//! Authored native widths, without a CAD export qualification.
use ifccad::IfccadPlanarPolylineWidths;
use serde_json::{json, Value};

pub(crate) fn authored(graph: &Value) -> Value {
    let entries = graph["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|node| {
            let body = node["attributes"].get("ifccad::geom::planarPolyline")?;
            let widths: IfccadPlanarPolylineWidths = body
                .get("widths")
                .map(|value| serde_json::from_value(value.clone()).expect("validated widths"))
                .unwrap_or_default();
            let count = body["vertices"]
                .as_array()
                .expect("validated vertices")
                .len();
            Some(json!({
                "path":node["path"],"widths":widths,
                "effectiveWidths":widths.effective_widths(count).expect("validated widths"),
                "displayEvaluation":"unassessed"
            }))
        })
        .collect::<Vec<_>>();
    json!(entries)
}
