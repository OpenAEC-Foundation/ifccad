//! Saved presentations on the catalogue's existing samples, leaving labels alone.
use serde_json::{json, Value};
pub fn attach(nodes: &mut Vec<Value>, selected: &[String]) {
    let scale = |i| format!("/cad/d1/annotationScale/{i}");
    for i in 1..=2 {
        nodes[0]["children"][format!("annotationScale{i}")] = json!(scale(i));
        nodes.push(json!({"path":scale(i),"attributes":{"ifccad::annotationScale":{"name":format!("1:{i}"),"paperUnits":1.,"drawingUnits":i}}}));
    }
    nodes[0]["attributes"]["ifccad::drawing"]["nextAnnotationScaleId"] = json!(3);
    nodes[1]["attributes"]["ifccad::layout"]["annotationState"] =
        json!({"currentScale":scale(1),"visibility":"currentScaleOnly"});
    for node in nodes.iter_mut() {
        let a = &mut node["attributes"];
        if let Some(style) = a.get_mut("ifccad::dimensionStyle") {
            style["annotative"] = json!(true);
            style["settings"]["presentationScale"] = json!({"kind":"byAnnotationScale"});
        }
        if let Some(block) = a.get_mut("ifccad::blockDefinition") {
            block["annotation"] = json!({"enabled":true});
        }
        let dimension = a.get("ifccad::dimension").is_some();
        if !dimension && !selected.iter().any(|p| node["path"] == *p) {
            continue;
        }
        let a = &mut node["attributes"];
        let p = if a.get("ifccad::text").is_some() {
            json!({"insertion":[0.,0.],"alignment":[0.,0.],"rotation":0.,"horizontal":"left"})
        } else if let Some(t) = a.get("ifccad::mText") {
            json!({"insertion":[0.,0.,0.],"xDirection":[1.,0.,0.],"attachment":t["attachment"],"rectangleWidth":t["wrapWidth"],"rectangleHeight":0.})
        } else if let Some(b) = a.get("ifccad::blockInstance") {
            json!({"transform":b["transform"]})
        } else if let Some(h) = a.get("ifccad::hatch") {
            let families = h["fill"].get("families").cloned().unwrap_or(json!([]));
            json!({"pattern":{"families":families,"scale":1.,"orientationReference":[1.,0.,0.]},"loops":h["loops"].as_array().unwrap().iter().enumerate().map(|(i,_)|json!({"kind":"shared","baseLoopIndex":i,"classification":{}})).collect::<Vec<_>>()})
        } else if a.get("ifccad::dimension").is_some() {
            // The catalogue's StyleDriven labels have no authored per-scale
            // layout result. A local [0,0] would become a real saved text point
            // at the owning Dimension's world origin, not CAD's auto sentinel.
            Value::Null
        } else {
            continue;
        };
        let contexts = (1..=2)
            .map(|i| {
                if p.is_null() {
                    return json!({"key":{"scaleId":scale(i)}});
                }
                let mut snapshot = p.clone();
                if i == 2 {
                    if let Some(values) = snapshot
                        .get_mut("transform")
                        .and_then(|t| t.get_mut("scale"))
                        .and_then(Value::as_array_mut)
                    {
                        for v in values {
                            *v = json!(v.as_f64().unwrap() * 1.25);
                        }
                    }
                    if snapshot.get("pattern").is_some() {
                        snapshot["pattern"]["scale"] = json!(2.);
                    }
                }
                json!({"key":{"scaleId":scale(i)},"presentation":snapshot})
            })
            .collect::<Vec<_>>();
        node["attributes"]["ifccad::annotation"] = json!({"mode":"enabled","orientation":"fixed","reference":{"scaleId":scale(1)},"contexts":contexts});
    }
}
