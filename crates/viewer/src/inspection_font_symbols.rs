//! Authored requests from the validated native graph, without asset lookup.
use serde_json::{json, Value};

fn request(rows: &mut Vec<Value>, path: String, role: &str, name: &str) {
    rows.push(
        json!({"requestPath":path,"fontRole":role,"requestedName":name,"status":"notChecked"}),
    );
}

fn font(rows: &mut Vec<Value>, value: &Value, path: &str) {
    for (field, role) in [
        ("family", "family"),
        ("cadFontName", "primary"),
        ("bigFontName", "bigFont"),
    ] {
        if let Some(name) = value[field].as_str() {
            request(rows, format!("{path}/{field}"), role, name);
        }
    }
}

fn mtext(rows: &mut Vec<Value>, text: &Value, base: &str) {
    font(
        rows,
        &text["characterFormat"]["font"],
        &format!("{base}/characterFormat/font"),
    );
    for (p, paragraph) in text["content"].as_array().into_iter().flatten().enumerate() {
        let prefix = format!("{base}/content/{p}");
        font(
            rows,
            &paragraph["characterFormat"]["font"],
            &format!("{prefix}/characterFormat/font"),
        );
        for (i, inline) in paragraph["inlines"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            font(
                rows,
                &inline["characterFormat"]["font"],
                &format!("{prefix}/inlines/{i}/characterFormat/font"),
            );
        }
    }
}

pub(crate) fn authored(graph: &Value) -> Vec<Value> {
    let mut rows = Vec::new();
    for node in graph["data"].as_array().into_iter().flatten() {
        let Some(path) = node["path"].as_str() else {
            continue;
        };
        let attrs = &node["attributes"];
        if let Some(style) = attrs.get("ifccad::textStyle") {
            font(
                &mut rows,
                &style["font"],
                &format!("{path}#/attributes/ifccad::textStyle/font"),
            );
        }
        if let Some(source) = attrs.get("ifccad::symbolSource") {
            let field = if source.get("cadFileName").is_some() {
                "cadFileName"
            } else {
                "fileName"
            };
            if let Some(name) = source[field].as_str() {
                request(
                    &mut rows,
                    format!("{path}#/attributes/ifccad::symbolSource/{field}"),
                    "shx",
                    name,
                );
            }
        }
        if let Some(text) = attrs.get("ifccad::mText") {
            mtext(
                &mut rows,
                text,
                &format!("{path}#/attributes/ifccad::mText"),
            );
        }
        for role in ["ifccad::attributeDefinition", "ifccad::blockAttribute"] {
            if let Some(m) = attrs.get(role).and_then(|a| a.get("mText")) {
                mtext(
                    &mut rows,
                    &m["text"],
                    &format!("{path}#/attributes/{role}/mText/text"),
                );
            }
        }
        for (context_index, context) in attrs["ifccad::annotation"]["contexts"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            for (part_index, part) in context["presentation"]["graphics"]["parts"]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                if let Some(text) = part.get("ifccad::mText") {
                    mtext(&mut rows, text, &format!("{path}#/attributes/ifccad::annotation/contexts/{context_index}/presentation/graphics/parts/{part_index}/ifccad::mText"));
                }
            }
        }
    }
    rows
}
