use super::{IfccadDocument, IfccadEntity, IfccadEntityKind, PROFILE_URI};
use crate::ifccad::IfccadPlotStyleMode;
use serde::Serialize;
use serde_json::{json, Map, Value};

// Struct field order is the presentation order in the example and writer output.
#[derive(Serialize)]
struct NodeOut {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Map<String, Value>>,
    attributes: Map<String, Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HeaderOut<'a> {
    id: &'a str,
    ifcx_version: &'static str,
    data_version: &'a str,
    author: &'a str,
    timestamp: &'a str,
}

#[derive(Serialize)]
struct FileOut<'a> {
    header: HeaderOut<'a>,
    imports: Vec<Value>,
    schemas: Value,
    data: Vec<NodeOut>,
}

fn with_bounds(mut value: Value, bounds: Option<super::IfccadBounds3d>) -> Value {
    if let Some(bounds) = bounds {
        value["bounds"] = json!(bounds);
    }
    value
}
fn attrs(value: Value) -> Map<String, Value> {
    value.as_object().expect("attribute object").clone()
}

fn entity_node(entity: &IfccadEntity, prefix: &str) -> NodeOut {
    let mut attrs = Map::new();
    attrs.insert("ifccad::entity".into(), json!({"layer":format!("{prefix}/layer/{}",entity.layer_id),"appearance":super::wire::entity(&entity.appearance,prefix),"linePatternScale":entity.line_pattern_scale}));
    match &entity.kind {
        IfccadEntityKind::Viewport(viewport) => {
            attrs.insert(
                "ifccad::viewport".into(),
                super::viewports::encode_viewport(viewport, prefix),
            );
        }
        IfccadEntityKind::BlockInstance {
            definition_id,
            transform,
        } => {
            attrs.insert("ifccad::blockInstance".into(), json!({"definition":format!("{prefix}/block/{definition_id}"),"transform":transform}));
        }
        kind => {
            super::geometry::encode_kind(kind, &mut attrs);
        }
    }
    NodeOut {
        path: format!("{prefix}/e{}", entity.id),
        children: None,
        attributes: attrs,
    }
}

fn numbered_children(entities: &[IfccadEntity], prefix: &str) -> Map<String, Value> {
    entities
        .iter()
        .enumerate()
        .map(|(i, e)| (i.to_string(), json!(format!("{prefix}/e{}", e.id))))
        .collect()
}

/// Encode a fresh CAD-profile file and check it with the production reader.
/// Foreign IFCX content and original fragment history are not part of this
/// document and are not merged into the output.
pub(crate) fn encode_bytes(document: &IfccadDocument) -> Result<Vec<u8>, serde_json::Error> {
    let mut paper_layouts: Vec<_> = document.paper_layouts.iter().collect();
    paper_layouts.sort_by_key(|layout| layout.tab_index);
    let prefix = format!("/cad/d{}", document.drawing_id);
    let mut drawing_children = Map::new();
    drawing_children.insert(
        "model".into(),
        json!(format!("{prefix}/layout/{}", document.model.id)),
    );
    let mut data = Vec::new();
    for layout in &paper_layouts {
        drawing_children.insert(
            format!("paper{}", layout.id),
            json!(format!("{prefix}/layout/{}", layout.id)),
        );
    }
    for layer in &document.layers {
        drawing_children.insert(
            format!("layer{}", layer.id),
            json!(format!("{prefix}/layer/{}", layer.id)),
        );
    }
    for p in &document.line_patterns {
        drawing_children.insert(
            format!("linePattern{}", p.id.0),
            json!(format!("{prefix}/linePattern/{}", p.id.0)),
        );
    }
    for block in &document.blocks {
        drawing_children.insert(
            format!("block{}", block.id),
            json!(format!("{prefix}/block/{}", block.id)),
        );
    }
    data.push(NodeOut {
        path: prefix.clone(),
        children: Some(drawing_children),
        attributes: attrs(json!({"ifccad::drawing":{
            "profileVersion":"0.1.0","lengthUnit":document.length_unit,
            "linePatternScale":document.line_pattern_scale,

            "nextEntityId":document.id_counters.next_entity_id,
            "nextLayerId":document.id_counters.next_layer_id,
            "nextLayoutId":document.id_counters.next_layout_id,
            "nextBlockId":document.id_counters.next_block_id,
            "nextLinePatternId":document.id_counters.next_line_pattern_id,
        }})),
    });
    if document.plot_style_mode == IfccadPlotStyleMode::Named {
        data.last_mut().unwrap().attributes["ifccad::drawing"]["plotStyleMode"] = json!("named");
    }
    data.push(NodeOut {
        path: format!("{prefix}/layout/{}", document.model.id),
        children: Some(numbered_children(&document.model.entities, &prefix)),
        attributes: attrs(
            json!({"ifccad::layout":with_bounds(super::layout::with_output(json!({"kind":"Model","tabIndex":document.model.tab_index}),&document.model.settings),document.model.bounds)}),
        ),
    });
    for layout in &paper_layouts {
        let value = super::layout::with_output(
            json!({"kind":"Paper","name":layout.name,"tabIndex":layout.tab_index}),
            &layout.settings,
        );
        data.push(NodeOut {
            path: format!("{prefix}/layout/{}", layout.id),
            children: Some(numbered_children(&layout.entities, &prefix)),
            attributes: attrs(json!({"ifccad::layout":with_bounds(value,layout.bounds)})),
        });
    }
    for p in &document.line_patterns {
        let mut value = json!({"name":p.name,"pattern":p.pattern});
        if let Some(description) = &p.description {
            value["description"] = json!(description);
        }
        data.push(NodeOut {
            path: format!("{prefix}/linePattern/{}", p.id.0),
            children: None,
            attributes: attrs(json!({"ifccad::linePattern":value})),
        });
    }
    for layer in &document.layers {
        data.push(NodeOut {
            path: format!("{prefix}/layer/{}", layer.id),
            children: None,
            attributes: attrs(
                json!({"ifccad::layer":{"name":layer.name,"appearance":super::wire::layer(&layer.appearance,&prefix)}}),
            ),
        });
    }
    for block in &document.blocks {
        data.push(NodeOut {
            path: format!("{prefix}/block/{}",block.id),
            children: Some(numbered_children(&block.entities,&prefix)),
            attributes: attrs(json!({"ifccad::blockDefinition":with_bounds(json!({"name":block.name,"basePoint":block.base_point,"insertionUnit":block.insertion_unit}),block.bounds)})),
        });
    }
    for entity in document
        .model
        .entities
        .iter()
        .chain(
            paper_layouts
                .iter()
                .flat_map(|layout| layout.entities.iter()),
        )
        .chain(document.blocks.iter().flat_map(|b| b.entities.iter()))
    {
        data.push(entity_node(entity, &prefix));
    }
    let result = FileOut {
        header: HeaderOut {
            id: &document.header.id,
            ifcx_version: "ifcx_alpha",
            data_version: &document.header.data_version,
            author: &document.header.author,
            timestamp: &document.header.timestamp,
        },
        imports: vec![json!({"uri":PROFILE_URI})],
        schemas: json!({}),
        data,
    };
    serde_json::to_vec_pretty(&result)
}
