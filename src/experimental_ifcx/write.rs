use super::{
    read_native_cad_ifcx, IfcxCadDocument, IfcxCadEntity, IfcxCadEntityKind, IfcxCadReport,
    PROFILE_URI,
};
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

fn attrs(value: Value) -> Map<String, Value> {
    value.as_object().expect("attribute object").clone()
}

fn entity_node(entity: &IfcxCadEntity, prefix: &str) -> NodeOut {
    let mut attrs = Map::new();
    attrs.insert("ifccad::entity".into(), json!({"layer":format!("{prefix}/layer/{}>",entity.layer_id),"appearance":entity.appearance}));
    match &entity.kind {
        IfcxCadEntityKind::LineSegment { start, end } => {
            attrs.insert(
                "ifccad::geom::lineSegment".into(),
                json!({"start":start,"end":end}),
            );
        }
        IfcxCadEntityKind::PlanarPolyline {
            vertices,
            closed,
            placement,
        } => {
            attrs.insert(
                "ifccad::geom::planarPolyline".into(),
                json!({"vertices":vertices,"closed":closed}),
            );
            attrs.insert("ifccad::geom::placement".into(), json!(placement));
        }
        IfcxCadEntityKind::Circle { radius, placement } => {
            attrs.insert("ifccad::geom::circle".into(), json!({"radius":radius}));
            attrs.insert("ifccad::geom::placement".into(), json!(placement));
        }
        IfcxCadEntityKind::BlockInstance {
            definition_id,
            transform,
        } => {
            attrs.insert("ifccad::blockInstance".into(), json!({"definition":format!("{prefix}/block/{definition_id}>"),"transform":transform}));
        }
    }
    NodeOut {
        path: format!("{prefix}/e{}>", entity.id),
        children: None,
        attributes: attrs,
    }
}

fn numbered_children(entities: &[IfcxCadEntity], prefix: &str) -> Map<String, Value> {
    entities
        .iter()
        .enumerate()
        .map(|(i, e)| (i.to_string(), json!(format!("{prefix}/e{}>", e.id))))
        .collect()
}

/// Serialize an IFCX alpha drawing with the versioned CAD schema import and strict-read it.
pub fn write_native_cad_ifcx(document: &IfcxCadDocument) -> Result<Vec<u8>, IfcxCadReport> {
    let prefix = format!("</cad/d{}", document.drawing_id);
    let mut drawing_children = Map::new();
    drawing_children.insert(
        "model".into(),
        json!(format!("{prefix}/layout/{}>", document.model.id)),
    );
    let mut data = Vec::new();
    for layer in &document.layers {
        drawing_children.insert(
            format!("layer{}", layer.id),
            json!(format!("{prefix}/layer/{}>", layer.id)),
        );
    }
    for block in &document.blocks {
        drawing_children.insert(
            format!("block{}", block.id),
            json!(format!("{prefix}/block/{}>", block.id)),
        );
    }
    data.push(NodeOut {
        path: format!("{prefix}>"),
        children: Some(drawing_children),
        attributes: attrs(
            json!({"ifccad::drawing":{"profileVersion":"0.1.0","lengthUnit":document.length_unit}}),
        ),
    });
    data.push(NodeOut {
        path: format!("{prefix}/layout/{}>", document.model.id),
        children: Some(numbered_children(&document.model.entities, &prefix)),
        attributes: attrs(json!({"ifccad::layout":{"kind":"Model"}})),
    });
    for layer in &document.layers {
        data.push(NodeOut {
            path: format!("{prefix}/layer/{}>", layer.id),
            children: None,
            attributes: attrs(
                json!({"ifccad::layer":{"name":layer.name,"appearance":layer.appearance}}),
            ),
        });
    }
    for block in &document.blocks {
        data.push(NodeOut {
            path: format!("{prefix}/block/{}>",block.id),
            children: Some(numbered_children(&block.entities,&prefix)),
            attributes: attrs(json!({"ifccad::blockDefinition":{"name":block.name,"basePoint":block.base_point,"insertionUnit":block.insertion_unit}})),
        });
    }
    for entity in document
        .model
        .entities
        .iter()
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
    let bytes = serde_json::to_vec_pretty(&result)
        .map_err(|e| IfcxCadReport::one(format!("IFCX serialization failed: {e}")))?;
    let loaded = read_native_cad_ifcx(&bytes)?;
    if loaded.document() != document {
        return Err(IfcxCadReport::one(
            "strict IFCX readback changed CAD semantics",
        ));
    }
    Ok(bytes)
}
