use ifccad::*;
use serde_json::{json, Value};

pub fn build() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut graph: Value = serde_json::from_slice(include_bytes!(
        "../../conformance/next/ifccad/valid/annotation-text.ifcx"
    ))?;
    graph["header"]["id"] = json!("font-symbol-source-review");
    graph["header"]["author"] = json!("IFCCAD review");
    // CAD requires Standard; author it explicitly rather than testing an added default.
    for node in graph["data"].as_array_mut().unwrap() {
        if node["path"] == "/cad/d1/textStyle/1" {
            node["attributes"]["ifccad::textStyle"]["name"] = json!("Standard");
        }
    }
    let text = graph["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == "/cad/d1/e1")
        .unwrap()
        .clone();
    let hello: Value =
        serde_json::from_slice(include_bytes!("../../tests/fixtures/ifccad/hello-cad.ifcx"))?;
    let line = hello["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == "/cad/d1/e1")
        .unwrap()
        .clone();
    let mut additions = Vec::new();
    let mut children = Vec::new();
    let mut entities = Vec::new();
    for (i, (name, bold, italic, pitch)) in [
        ("01 Arial regular", false, false, 34),
        ("02 Arial bold", true, false, 34),
        ("03 Arial italic", false, true, 34),
        ("04 Arial bold italic", true, true, 34),
        ("05 Consolas", false, false, 49),
    ]
    .into_iter()
    .enumerate()
    {
        let id = 10 + i;
        let path = format!("/cad/d1/textStyle/{id}");
        additions.push(json!({"path":path,"attributes":{"ifccad::textStyle":{
            "name":name,"font":{"family":if pitch==49 {"Consolas"}else{"Arial"},
                "cadFontName":if pitch==49 {"consola.ttf"}else{"arial.ttf"},
                "bold":bold,"italic":italic,"charset":0,"pitch":pitch}
        }}}));
        children.push((format!("reviewStyle{id}"), path.clone()));
        let entity_id = 100 + i;
        let entity_path = format!("/cad/d1/e{entity_id}");
        let mut node = text.clone();
        node["path"] = json!(entity_path);
        node["attributes"]
            .as_object_mut()
            .unwrap()
            .remove("ifccad::annotation");
        node["attributes"]["ifccad::geom::placement"]["origin"] =
            json!([0., -20. - i as f64 * 12., 0.]);
        let body = &mut node["attributes"]["ifccad::text"];
        body["style"] = json!(path);
        body["rotation"] = json!(0.);
        body["layout"]["height"] = json!(3.);
        body["content"] = json!([{"text":name}]);
        additions.push(node);
        entities.push((format!("review{entity_id}"), entity_path));
    }
    for (i, request) in [
        "ltypeshp.shx",
        "ltypeshp.shx",
        r"..\project\ltypeshp.shx",
        r"C:\IFCCAD-review\ltypeshp.shx",
        r"C:ltypeshp.shx",
        r"\\review-server\fonts\ltypeshp.shx",
    ]
    .into_iter()
    .enumerate()
    {
        let id = 10 + i;
        let path = format!("/cad/d1/symbolSource/{id}");
        let mut source = json!({"kind":"shx","fileName":"ltypeshp.shx"});
        if i >= 2 {
            source["cadFileName"] = json!(request);
        }
        additions.push(json!({"path":path,"attributes":{"ifccad::symbolSource":source}}));
        children.push((format!("reviewSource{id}"), path.clone()));
        if i == 1 {
            continue;
        } // Independent equal-named source remains intentionally unused.
        let pattern = format!("/cad/d1/linePattern/{id}");
        additions.push(json!({"path":pattern,"attributes":{"ifccad::linePattern":{"name":format!("SHX request {i}"),"pattern":[10.,{"length":0.,"marker":{"kind":"symbol","symbolSource":path,"shapeNumber":130}},-5.]}}}));
        children.push((format!("reviewPattern{id}"), pattern.clone()));
        let entity_id = 200 + i;
        let entity_path = format!("/cad/d1/e{entity_id}");
        let mut node = line.clone();
        node["path"] = json!(entity_path);
        node["attributes"]["ifccad::entity"]["layer"] = json!("/cad/d1/layer/0");
        node["attributes"]["ifccad::entity"]["appearance"]["linePattern"] =
            json!({"mode":"Explicit","value":pattern});
        node["attributes"]["ifccad::geom::lineSegment"] =
            json!({"start":[0.,-100.-i as f64*10.,0.],"end":[120.,-100.-i as f64*10.,0.]});
        additions.push(node);
        entities.push((format!("review{entity_id}"), entity_path));
    }
    let mut scoped = graph["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == "/cad/d1/e2")
        .unwrap()
        .clone();
    scoped["path"] = json!("/cad/d1/e300");
    scoped["attributes"]
        .as_object_mut()
        .unwrap()
        .remove("ifccad::annotation");
    scoped["attributes"]["ifccad::geom::placement"]["origin"] = json!([0., -175., 0.]);
    scoped["attributes"]["ifccad::mText"] = json!({
        "style":"/cad/d1/textStyle/10","height":3.,"attachment":"topLeft","flow":"horizontal","wrapWidth":120.,
        "characterFormat":{"font":{"family":"Arial","bold":false,"italic":false,"charset":0,"pitch":34}},
        "content":[{"inlines":[{"kind":"run","text":"06 MText: base Arial"}]},
            {"characterFormat":{"font":{"family":"Consolas","bold":false,"italic":false,"charset":0,"pitch":49}},
             "inlines":[{"kind":"run","text":"Paragraph Consolas / "},
                {"kind":"run","text":"inline Arial bold","characterFormat":{"font":{"family":"Arial","bold":true,"italic":false,"charset":0,"pitch":34}}},
                {"kind":"stack","stackKind":"fraction","upper":"1","lower":"2","characterFormat":{"font":{"family":"Arial","bold":false,"italic":true,"charset":0,"pitch":34}}}]}]
    });
    additions.push(scoped);
    entities.push(("review300".into(), "/cad/d1/e300".into()));
    for node in graph["data"].as_array_mut().unwrap() {
        if node["path"] == "/cad/d1" {
            for (key, path) in &children {
                node["children"][key] = json!(path);
            }
            let d = &mut node["attributes"]["ifccad::drawing"];
            d["nextTextStyleId"] = json!(20);
            d["nextSymbolSourceId"] = json!(20);
            d["nextLinePatternId"] = json!(20);
            d["nextEntityId"] = json!(1000);
        }
        if node["path"] == "/cad/d1/layout/1" {
            let start = node["children"].as_object().unwrap().len();
            for (index, (_, path)) in entities.iter().enumerate() {
                node["children"][(start + index).to_string()] = json!(path);
            }
            let layout = node["attributes"]["ifccad::layout"]
                .as_object_mut()
                .unwrap();
            layout.remove("bounds");
            layout.remove("boundsQuality");
        }
    }
    graph["data"].as_array_mut().unwrap().extend(additions);
    let document = load_ifccad_bytes(&serde_json::to_vec(&graph)?, Default::default())?;
    Ok(encode_ifccad_document(document.document())?.into_bytes())
}
