use super::error::PackageBuildError;
use super::state::DrawingState;
use super::types::PackageOptions;
use crate::ifcdr::write::EncodedIfcdrResource;
use serde_json::{json, Map, Value};

pub(crate) const MODEL_SPACE_RESOURCE_URI: &str = "resources/model-space.ifcdr.json";

#[derive(Debug)]
pub(crate) struct NodePaths {
    pub(crate) drawing_set: String,
    pub(crate) drawing: String,
    pub(crate) layout: String,
    pub(crate) representation: String,
    pub(crate) layers: Vec<String>,
    pub(crate) appearances: Vec<String>,
}

impl NodePaths {
    pub(crate) fn for_drawing(drawing: &DrawingState) -> Result<Self, PackageBuildError> {
        let layers = numbered_paths("layer", drawing.layers.len())?;
        let appearances = numbered_paths("appearance", drawing.appearances.len())?;
        Ok(Self {
            drawing_set: "drawing-set-0".to_owned(),
            drawing: "drawing-0".to_owned(),
            layout: "layout-0".to_owned(),
            representation: "representation-0".to_owned(),
            layers,
            appearances,
        })
    }
}

pub(crate) fn assemble_ifcx(
    package_options: &PackageOptions,
    drawing: &DrawingState,
    paths: &NodePaths,
    resource: &EncodedIfcdrResource,
) -> Result<Vec<u8>, PackageBuildError> {
    let mut data = vec![
        json!({
            "path": paths.drawing_set,
            "type": "openaec:DrawingSet",
            "children": {"Drawings": [paths.drawing.clone()]}
        }),
        json!({
            "path": paths.drawing,
            "type": "openaec:Drawing",
            "children": {
                "Layouts": [paths.layout.clone()],
                "Representation": paths.representation
            }
        }),
        json!({
            "path": paths.layout,
            "type": "openaec:DrawingLayout",
            "attributes": {
                "name": drawing.options.model_layout_name,
                "kind": "model",
                "scopeId": 0
            },
            "children": {"Representation": paths.representation}
        }),
        json!({
            "path": paths.representation,
            "type": "openaec:DrawingGeometryRepresentation",
            "attributes": {
                "name": "ModelSpace",
                "geometry": {
                    "format": "openaec.ifcdr",
                    "version": "0.5.0",
                    "resourceId": resource.resource_id,
                    "uri": MODEL_SPACE_RESOURCE_URI,
                    "checksum": resource.checksum,
                    "role": "modelspace"
                }
            }
        }),
    ];

    let layer_nodes = drawing
        .layers
        .iter()
        .zip(&paths.layers)
        .map(|(layer, path)| {
            let appearance_index = layer
                .definition
                .appearance
                .local_id
                .checked_sub(2)
                .and_then(|value| usize::try_from(value).ok())
                .filter(|&index| index < paths.appearances.len())
                .ok_or_else(|| PackageBuildError::Encoding {
                    stage: "IFCX layer",
                    message: format!(
                        "layer {} references an unavailable appearance",
                        layer.definition.name
                    ),
                })?;
            Ok(json!({
                "path": path,
                "type": "openaec:Layer",
                "attributes": {
                    "name": layer.definition.name,
                    "visible": layer.definition.visible,
                    "appearance": paths.appearances[appearance_index]
                }
            }))
        })
        .collect::<Result<Vec<_>, PackageBuildError>>()?;
    data.extend(layer_nodes);
    data.extend(
        drawing
            .appearances
            .iter()
            .zip(&paths.appearances)
            .map(|(appearance, path)| {
                let definition = &appearance.definition;
                let mut color = Map::new();
                color.insert("rgb".to_owned(), json!(definition.color.rgb));
                if let Some(indexed) = &definition.color.indexed {
                    color.insert(
                        "indexedColor".to_owned(),
                        json!({"system": indexed.system, "index": indexed.index}),
                    );
                }
                if let Some(named) = &definition.color.named {
                    color.insert(
                        "namedColor".to_owned(),
                        json!({"catalog": named.catalog, "name": named.name}),
                    );
                }
                json!({
                    "path": path,
                    "type": "openaec:Appearance",
                    "attributes": {
                        "name": definition.name,
                        "color": {"mode": "explicit", "value": Value::Object(color)},
                        "opacity": {"mode": "explicit", "value": definition.opacity},
                        "linePattern": {
                            "mode": "explicit",
                            "value": definition.line_pattern.name
                        },
                        "lineWeight": {"mode": "explicit", "value": definition.line_weight}
                    }
                })
            }),
    );

    serde_json::to_vec_pretty(&json!({
        "header": {
            "id": package_options.package_id,
            "ifcxVersion": "ifcx_alpha",
            "dataVersion": package_options.data_version,
            "author": package_options.author,
            "timestamp": package_options.timestamp
        },
        "imports": [],
        "data": data
    }))
    .map_err(|error| PackageBuildError::Encoding {
        stage: "IFCX",
        message: error.to_string(),
    })
}

fn numbered_paths(prefix: &str, count: usize) -> Result<Vec<String>, PackageBuildError> {
    u32::try_from(count).map_err(|_| PackageBuildError::RangeExhausted {
        kind: "IFCX node path",
    })?;
    Ok((0..count)
        .map(|index| format!("{prefix}-{index}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::ifcdr::IfcdrLengthUnit;
    use crate::package::{
        AppearanceColor, AppearanceDefinition, DrawingOptions, LayerDefinition,
        LinePatternDefinition, PackageBuilder, PackageOptions,
    };
    use crate::{PackageId, ResourceId};
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    #[test]
    fn assembles_minimal_drawing_graph_with_external_geometry() {
        let mut package = PackageBuilder::new(PackageOptions {
            package_id: PackageId::new("building-a").unwrap(),
            data_version: "17".to_owned(),
            author: "Example application".to_owned(),
            timestamp: "2026-09-03T10:00:00.125+00:00".to_owned(),
        })
        .unwrap();
        let mut drawing = package
            .add_drawing(DrawingOptions {
                model_layout_name: "Model layout".to_owned(),
                representation_resource_id: ResourceId::new("geometry-modelspace-main").unwrap(),
                length_unit: IfcdrLengthUnit::Millimetre,
            })
            .unwrap();
        let style = drawing
            .appearances()
            .add(AppearanceDefinition {
                name: "Wall style".to_owned(),
                color: AppearanceColor::rgb(255, 0, 0)
                    .with_indexed("ACI", 1)
                    .with_named("RAL", "Traffic red"),
                opacity: 0.75,
                line_pattern: LinePatternDefinition::named("continuous"),
                line_weight: 0.25,
            })
            .unwrap();
        drawing
            .layers()
            .add(LayerDefinition {
                name: "A-WALL".to_owned(),
                visible: false,
                appearance: style,
            })
            .unwrap();

        let encoded = package.finish().unwrap();
        let root: Value =
            serde_json::from_slice(encoded.file("package.ifcx.json").unwrap()).unwrap();

        assert_eq!(root["header"]["id"], "building-a");
        assert_eq!(root["header"]["ifcxVersion"], "ifcx_alpha");
        assert_eq!(root["header"]["dataVersion"], "17");
        assert_eq!(root["header"]["author"], "Example application");
        assert_eq!(root["header"]["timestamp"], "2026-09-03T10:00:00.125Z");
        assert_eq!(root["imports"], serde_json::json!([]));
        assert_eq!(
            root["data"]
                .as_array()
                .unwrap()
                .iter()
                .map(|node| node["path"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "drawing-set-0",
                "drawing-0",
                "layout-0",
                "representation-0",
                "layer-0",
                "appearance-0"
            ]
        );
        assert_eq!(
            root["data"][0]["children"]["Drawings"],
            serde_json::json!(["drawing-0"])
        );
        assert_eq!(
            root["data"][1]["children"]["Layouts"],
            serde_json::json!(["layout-0"])
        );
        assert_eq!(root["data"][2]["attributes"]["name"], "Model layout");
        assert_eq!(root["data"][2]["attributes"]["kind"], "model");
        assert_eq!(root["data"][2]["attributes"]["scopeId"], 0);
        assert!(root["data"][1].get("name").is_none());
        assert_eq!(root["data"][3]["attributes"]["name"], "ModelSpace");
        let geometry = &root["data"][3]["attributes"]["geometry"];
        assert_eq!(geometry["format"], "openaec.ifcdr");
        assert_eq!(geometry["version"], "0.5.0");
        assert_eq!(geometry["role"], "modelspace");
        assert_eq!(geometry["resourceId"], "geometry-modelspace-main");
        assert_eq!(geometry["uri"], "resources/model-space.ifcdr.json");
        let ifcdr = encoded.file("resources/model-space.ifcdr.json").unwrap();
        assert_eq!(
            geometry["checksum"],
            format!("sha256:{:x}", Sha256::digest(ifcdr))
        );
        assert_eq!(root["data"][4]["attributes"]["appearance"], "appearance-0");
        assert_eq!(
            root["data"][5]["attributes"]["color"]["value"]["rgb"],
            serde_json::json!([255, 0, 0])
        );
        assert_eq!(
            root["data"][5]["attributes"]["color"]["value"]["indexedColor"]["system"],
            "ACI"
        );
        assert_eq!(
            root["data"][5]["attributes"]["color"]["value"]["namedColor"]["name"],
            "Traffic red"
        );
    }
}
