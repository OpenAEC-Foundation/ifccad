use super::{IfcdrEncodeInput, IfcdrEntityInput};
use crate::ifcdr::{Bounds2d, IfcdrLengthUnit, Point2};
use crate::ResourceId;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub(crate) struct EncodedIfcdrResource {
    pub(crate) resource_id: ResourceId,
    pub(crate) bytes: Vec<u8>,
    pub(crate) checksum: String,
    pub(crate) bounds: Bounds2d,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum IfcdrEncodeError {
    #[error("{kind} ID or count range is exhausted")]
    RangeExhausted { kind: &'static str },
    #[error("invalid IFCDR encoder input: {message}")]
    InvalidInput { message: String },
    #[error("IFCDR serialization failed: {message}")]
    Serialization { message: String },
}

pub(crate) fn encode(
    input: &IfcdrEncodeInput<'_>,
) -> Result<EncodedIfcdrResource, IfcdrEncodeError> {
    let bounds = bounds(input.entities);
    let next_entity_id = u64::try_from(input.entities.len())
        .ok()
        .and_then(|count| count.checked_add(1))
        .ok_or(IfcdrEncodeError::RangeExhausted {
            kind: "next entity",
        })?;
    let entity_ids = input
        .entities
        .iter()
        .map(|entity| match entity {
            IfcdrEntityInput::Line { entity_id, .. }
            | IfcdrEntityInput::Polyline { entity_id, .. } => entity_id.get(),
        })
        .collect::<Vec<_>>();
    let entity_count =
        u32::try_from(entity_ids.len()).map_err(|_| IfcdrEncodeError::RangeExhausted {
            kind: "entity order",
        })?;

    let mut appearance_bindings = vec![
        json!({
            "id": 0,
            "ifcxAppearance": null,
            "colorMode": 0,
            "opacityMode": 0,
            "linePatternMode": 0,
            "lineWeightMode": 0,
            "overrideId": null
        }),
        json!({
            "id": 1,
            "ifcxAppearance": null,
            "colorMode": 2,
            "opacityMode": 2,
            "linePatternMode": 2,
            "lineWeightMode": 2,
            "overrideId": null
        }),
    ];
    appearance_bindings.extend(input.appearances.iter().map(|appearance| {
        json!({
            "id": appearance.id.get(),
            "ifcxAppearance": appearance.ifcx_path,
            "colorMode": 1,
            "opacityMode": 1,
            "linePatternMode": 1,
            "lineWeightMode": 1,
            "overrideId": null
        })
    }));
    let layer_bindings = input
        .layers
        .iter()
        .map(|layer| json!({"id": layer.id.get(), "ifcxLayer": layer.ifcx_path}))
        .collect::<Vec<_>>();

    let mut directory_entries = Vec::new();
    let mut streams = Map::new();

    let mut line_entity_ids = Vec::new();
    let mut line_scope_ids = Vec::new();
    let mut line_x1 = Vec::new();
    let mut line_y1 = Vec::new();
    let mut line_x2 = Vec::new();
    let mut line_y2 = Vec::new();
    let mut line_layer_ids = Vec::new();
    let mut line_appearance_ids = Vec::new();
    let mut line_visible = Vec::new();

    let mut polyline_entity_ids = Vec::new();
    let mut polyline_scope_ids = Vec::new();
    let mut polyline_offsets = Vec::new();
    let mut polyline_counts = Vec::new();
    let mut polyline_closed = Vec::new();
    let mut polyline_x = Vec::new();
    let mut polyline_y = Vec::new();
    let mut polyline_layer_ids = Vec::new();
    let mut polyline_appearance_ids = Vec::new();
    let mut polyline_visible = Vec::new();

    for entity in input.entities {
        match entity {
            IfcdrEntityInput::Line {
                entity_id,
                start,
                end,
                layer_id,
                appearance_id,
                visible,
            } => {
                line_entity_ids.push(entity_id.get());
                line_scope_ids.push(input.scope.id.get());
                line_x1.push(start.x());
                line_y1.push(start.y());
                line_x2.push(end.x());
                line_y2.push(end.y());
                line_layer_ids.push(layer_id.get());
                line_appearance_ids.push(appearance_id.get());
                line_visible.push(*visible);
            }
            IfcdrEntityInput::Polyline {
                entity_id,
                points,
                closed,
                layer_id,
                appearance_id,
                visible,
            } => {
                let offset = u32::try_from(polyline_x.len()).map_err(|_| {
                    IfcdrEncodeError::RangeExhausted {
                        kind: "polyline vertex offset",
                    }
                })?;
                let count =
                    u32::try_from(points.len()).map_err(|_| IfcdrEncodeError::RangeExhausted {
                        kind: "polyline vertex count",
                    })?;
                polyline_entity_ids.push(entity_id.get());
                polyline_scope_ids.push(input.scope.id.get());
                polyline_offsets.push(offset);
                polyline_counts.push(count);
                polyline_closed.push(*closed);
                for point in *points {
                    polyline_x.push(point.x());
                    polyline_y.push(point.y());
                }
                polyline_layer_ids.push(layer_id.get());
                polyline_appearance_ids.push(appearance_id.get());
                polyline_visible.push(*visible);
            }
        }
    }

    if !line_entity_ids.is_empty() {
        let count =
            u32::try_from(line_entity_ids.len()).map_err(|_| IfcdrEncodeError::RangeExhausted {
                kind: "line row count",
            })?;
        directory_entries.push(json!({
            "name": "line",
            "schema": "ifccad.ifcdr.line.v2",
            "role": "object",
            "count": count,
            "columns": [
                "entityId", "scopeId", "x1", "y1", "x2", "y2",
                "layerId", "appearanceId", "visible"
            ]
        }));
        streams.insert(
            "lineStream".to_owned(),
            json!({
                "count": count,
                "entityId": line_entity_ids,
                "scopeId": line_scope_ids,
                "x1": line_x1,
                "y1": line_y1,
                "x2": line_x2,
                "y2": line_y2,
                "layerId": line_layer_ids,
                "appearanceId": line_appearance_ids,
                "visible": line_visible
            }),
        );
    }

    if !polyline_entity_ids.is_empty() {
        let count = u32::try_from(polyline_entity_ids.len()).map_err(|_| {
            IfcdrEncodeError::RangeExhausted {
                kind: "polyline row count",
            }
        })?;
        directory_entries.push(json!({
            "name": "polyline",
            "schema": "ifccad.ifcdr.polyline.v2",
            "role": "object",
            "count": count,
            "columns": [
                "entityId", "scopeId", "vertexOffset", "vertexCount", "closed",
                "x", "y", "layerId", "appearanceId", "visible"
            ]
        }));
        streams.insert(
            "polylineStream".to_owned(),
            json!({
                "count": count,
                "entityId": polyline_entity_ids,
                "scopeId": polyline_scope_ids,
                "vertexOffset": polyline_offsets,
                "vertexCount": polyline_counts,
                "closed": polyline_closed,
                "x": polyline_x,
                "y": polyline_y,
                "layerId": polyline_layer_ids,
                "appearanceId": polyline_appearance_ids,
                "visible": polyline_visible
            }),
        );
    }

    directory_entries.extend([
        json!({
            "name": "entityOrder",
            "schema": "ifccad.ifcdr.entityOrder.v1",
            "role": "order",
            "count": 1,
            "columns": ["scopeId", "entryOffset", "entryCount"],
            "children": ["entityOrderEntry"]
        }),
        json!({
            "name": "entityOrderEntry",
            "schema": "ifccad.ifcdr.entityOrderEntry.v1",
            "role": "child",
            "count": entity_count,
            "columns": ["entityId"],
            "parent": "entityOrder"
        }),
    ]);
    streams.insert(
        "entityOrderStream".to_owned(),
        json!({
            "count": 1,
            "scopeId": [input.scope.id.get()],
            "entryOffset": [0],
            "entryCount": [entity_count]
        }),
    );
    streams.insert(
        "entityOrderEntryStream".to_owned(),
        json!({
            "count": entity_count,
            "entityId": entity_ids
        }),
    );

    let root = json!({
        "header": {
            "format": "openaec.ifcdr",
            "version": "0.5.0",
            "resourceId": input.resource_id,
            "unit": unit_name(input.unit),
            "nextEntityId": next_entity_id
        },
        "bounds": {
            "minX": bounds.min.x(),
            "minY": bounds.min.y(),
            "maxX": bounds.max.x(),
            "maxY": bounds.max.y()
        },
        "scopeTable": [{
            "id": input.scope.id.get(),
            "kind": input.scope.kind,
            "name": input.scope.name,
            "baseX": input.scope.base.x(),
            "baseY": input.scope.base.y(),
            "flags": input.scope.flags
        }],
        "appearanceBindings": appearance_bindings,
        "layerBindings": layer_bindings,
        "namedUcsBindings": [],
        "dimensionOverrideTable": [],
        "streamDirectory": {
            "version": "ifccad.ifcdr.streamDirectory.v1",
            "streams": directory_entries
        },
        "streams": Value::Object(streams)
    });
    let bytes =
        serde_json::to_vec_pretty(&root).map_err(|error| IfcdrEncodeError::Serialization {
            message: error.to_string(),
        })?;
    let checksum = format!("sha256:{:x}", Sha256::digest(&bytes));
    Ok(EncodedIfcdrResource {
        resource_id: input.resource_id.clone(),
        bytes,
        checksum,
        bounds,
    })
}

fn bounds(entities: &[IfcdrEntityInput<'_>]) -> Bounds2d {
    let mut points = entities.iter().flat_map(|entity| match entity {
        IfcdrEntityInput::Line { start, end, .. } => vec![*start, *end],
        IfcdrEntityInput::Polyline { points, .. } => points.to_vec(),
    });
    let Some(first) = points.next() else {
        return Bounds2d {
            min: Point2::new(0.0, 0.0),
            max: Point2::new(0.0, 0.0),
        };
    };
    let mut min_x = first.x();
    let mut min_y = first.y();
    let mut max_x = first.x();
    let mut max_y = first.y();
    for point in points {
        min_x = min_x.min(point.x());
        min_y = min_y.min(point.y());
        max_x = max_x.max(point.x());
        max_y = max_y.max(point.y());
    }
    Bounds2d {
        min: Point2::new(min_x, min_y),
        max: Point2::new(max_x, max_y),
    }
}

fn unit_name(unit: IfcdrLengthUnit) -> &'static str {
    match unit {
        IfcdrLengthUnit::Unitless => "unitless",
        IfcdrLengthUnit::Millimetre => "mm",
        IfcdrLengthUnit::Centimetre => "cm",
        IfcdrLengthUnit::Metre => "m",
        IfcdrLengthUnit::Kilometre => "km",
        IfcdrLengthUnit::Inch => "in",
        IfcdrLengthUnit::Foot => "ft",
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::{
        AppearanceColor, AppearanceDefinition, EntityAppearance, IfccadPackageBuilder,
        LayerDefinition, LineDefinition, LinePatternDefinition, PackageOptions, PolylineDefinition,
    };
    use crate::ifcdr::{IfcdrLengthUnit, Point2};
    use crate::{PackageId, ResourceId};
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    fn empty_builder() -> IfccadPackageBuilder {
        IfccadPackageBuilder::new(PackageOptions {
            package_id: PackageId::new("empty-package").unwrap(),
            data_version: "1".to_owned(),
            author: "writer test".to_owned(),
            timestamp: "2026-09-03T10:00:00Z".to_owned(),
            model_layout_name: "Model".to_owned(),
            representation_resource_id: ResourceId::new("empty-modelspace").unwrap(),
            length_unit: IfcdrLengthUnit::Millimetre,
        })
        .unwrap()
    }

    fn mixed_builder() -> IfccadPackageBuilder {
        let mut builder = empty_builder();
        let solid = builder
            .appearances()
            .add(AppearanceDefinition {
                name: "Solid".to_owned(),
                color: AppearanceColor::rgb(1, 2, 3),
                opacity: 1.0,
                line_pattern: LinePatternDefinition::named("continuous"),
                line_weight: 0.25,
            })
            .unwrap();
        let dashed = builder
            .appearances()
            .add(AppearanceDefinition {
                name: "Dashed".to_owned(),
                color: AppearanceColor::rgb(250, 20, 10),
                opacity: 0.5,
                line_pattern: LinePatternDefinition::named("dashed"),
                line_weight: 0.18,
            })
            .unwrap();
        let layer_0 = builder
            .layers()
            .add(LayerDefinition {
                name: "0".to_owned(),
                visible: true,
                appearance: solid,
            })
            .unwrap();
        let layer_1 = builder
            .layers()
            .add(LayerDefinition {
                name: "A-WALL".to_owned(),
                visible: false,
                appearance: dashed,
            })
            .unwrap();
        builder
            .model_space()
            .add_line(LineDefinition {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(10.0, 5.0),
                layer: layer_0,
                appearance: EntityAppearance::ByLayer,
                visible: true,
            })
            .unwrap();
        builder
            .model_space()
            .add_polyline(PolylineDefinition {
                points: vec![Point2::new(-2.0, 3.0), Point2::new(4.0, -5.0)],
                closed: false,
                layer: layer_1,
                appearance: EntityAppearance::Explicit(solid),
                visible: false,
            })
            .unwrap();
        builder
            .model_space()
            .add_line(LineDefinition {
                start: Point2::new(1.0, 2.0),
                end: Point2::new(3.0, 4.0),
                layer: layer_1,
                appearance: EntityAppearance::ByBlock,
                visible: false,
            })
            .unwrap();
        builder
            .model_space()
            .add_polyline(PolylineDefinition {
                points: vec![
                    Point2::new(2.0, 2.0),
                    Point2::new(8.0, 8.0),
                    Point2::new(5.0, -1.0),
                ],
                closed: true,
                layer: layer_0,
                appearance: EntityAppearance::Explicit(dashed),
                visible: true,
            })
            .unwrap();
        builder
    }

    #[test]
    fn encodes_empty_model_space_with_explicit_empty_order() {
        let package = empty_builder().finish().unwrap();
        let bytes = package.file("resources/model-space.ifcdr.json").unwrap();
        let root: Value = serde_json::from_slice(bytes).unwrap();

        assert_eq!(root["header"]["format"], "openaec.ifcdr");
        assert_eq!(root["header"]["version"], "0.5.0");
        assert_eq!(root["header"]["resourceId"], "empty-modelspace");
        assert_eq!(root["header"]["unit"], "mm");
        assert_eq!(root["header"]["nextEntityId"], 1);
        assert_eq!(
            root["bounds"],
            serde_json::json!({"minX": 0.0, "minY": 0.0, "maxX": 0.0, "maxY": 0.0})
        );
        assert_eq!(
            root["scopeTable"],
            serde_json::json!([{
                "id": 0,
                "kind": 0,
                "name": "ModelSpace",
                "baseX": 0.0,
                "baseY": 0.0,
                "flags": 0
            }])
        );
        assert_eq!(root["appearanceBindings"].as_array().unwrap().len(), 2);
        assert_eq!(root["appearanceBindings"][0]["id"], 0);
        assert_eq!(root["appearanceBindings"][0]["colorMode"], 0);
        assert_eq!(root["appearanceBindings"][1]["id"], 1);
        assert_eq!(root["appearanceBindings"][1]["colorMode"], 2);
        assert_eq!(root["dimensionOverrideTable"], serde_json::json!([]));
        assert_eq!(root["namedUcsBindings"], serde_json::json!([]));
        assert!(root.get("lineStream").is_none());
        assert!(root["streams"].get("lineStream").is_none());
        assert!(root["streams"].get("polylineStream").is_none());
        assert_eq!(root["streams"]["entityOrderStream"]["count"], 1);
        assert_eq!(
            root["streams"]["entityOrderStream"]["entryCount"],
            serde_json::json!([0])
        );
        assert_eq!(root["streams"]["entityOrderEntryStream"]["count"], 0);
        assert_eq!(
            root["streams"]["entityOrderEntryStream"]["entityId"],
            serde_json::json!([])
        );
        assert_eq!(
            root["streamDirectory"]["streams"].as_array().unwrap().len(),
            2
        );
    }

    #[test]
    fn encodes_mixed_entities_with_typed_rows_global_order_and_bounds() {
        let package = mixed_builder().finish().unwrap();
        let bytes = package.file("resources/model-space.ifcdr.json").unwrap();
        let root: Value = serde_json::from_slice(bytes).unwrap();

        assert_eq!(root["header"]["resourceId"], "empty-modelspace");
        assert_eq!(root["bounds"]["minX"], -2.0);
        assert_eq!(root["bounds"]["minY"], -5.0);
        assert_eq!(root["bounds"]["maxX"], 10.0);
        assert_eq!(root["bounds"]["maxY"], 8.0);
        assert_eq!(root["header"]["nextEntityId"], 5);
        assert_eq!(
            root["appearanceBindings"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["id"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
        assert_eq!(
            root["streams"]["lineStream"]["entityId"],
            serde_json::json!([1, 3])
        );
        assert_eq!(
            root["streams"]["lineStream"]["appearanceId"],
            serde_json::json!([0, 1])
        );
        assert_eq!(
            root["streams"]["lineStream"]["visible"],
            serde_json::json!([true, false])
        );
        assert_eq!(
            root["streams"]["polylineStream"]["entityId"],
            serde_json::json!([2, 4])
        );
        assert_eq!(
            root["streams"]["polylineStream"]["vertexOffset"],
            serde_json::json!([0, 2])
        );
        assert_eq!(
            root["streams"]["polylineStream"]["vertexCount"],
            serde_json::json!([2, 3])
        );
        assert_eq!(
            root["streams"]["polylineStream"]["closed"],
            serde_json::json!([false, true])
        );
        assert_eq!(
            root["streams"]["polylineStream"]["appearanceId"],
            serde_json::json!([2, 3])
        );
        assert_eq!(
            root["streams"]["polylineStream"]["visible"],
            serde_json::json!([false, true])
        );
        assert_eq!(
            root["streams"]["entityOrderEntryStream"]["entityId"],
            serde_json::json!([1, 2, 3, 4])
        );
        let directory = root["streamDirectory"]["streams"].as_array().unwrap();
        assert_eq!(directory.len(), 4);
        assert_eq!(directory[0]["name"], "line");
        assert_eq!(directory[0]["schema"], "ifccad.ifcdr.line.v2");
        assert_eq!(
            directory[0]["columns"].as_array().unwrap().last().unwrap(),
            "visible"
        );
        assert_eq!(directory[1]["name"], "polyline");
        assert_eq!(
            directory[2]["children"],
            serde_json::json!(["entityOrderEntry"])
        );
        assert_eq!(directory[3]["parent"], "entityOrder");
        let checksum = format!("sha256:{:x}", Sha256::digest(bytes));
        let entrypoint = package.file("package.ifcx.json").unwrap();
        let ifcx: Value = serde_json::from_slice(entrypoint).unwrap();
        assert!(ifcx["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| {
                node["attributes"]["geometry"]["checksum"].as_str() == Some(checksum.as_str())
            }));
    }
}
