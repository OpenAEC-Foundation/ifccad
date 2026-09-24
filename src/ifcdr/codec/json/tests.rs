use super::*;
use crate::ifcdr::logical::*;
use serde_json::{json, Value};

fn block_fixture() -> Value {
    json!({
        "header":{"format":"openaec.ifcdr","version":"0.9.0","resourceId":"blocks","unit":"m","nextEntityId":4},
        "scopeTable":[{"id":7,"kind":0,"bounds":null},{"id":21,"kind":2,"bounds":null}],
        "blockDefinitionTable":[{"scopeId":21,"name":"Door"}],
        "layerBindings":[{"id":0,"ifcxLayer":"layer"}],
        "appearanceBindings":[{"id":0,"ifcxAppearance":null,"colorMode":0,"opacityMode":0,"linePatternMode":0,"lineWeightMode":0,"overrideId":null}],
        "streamDirectory":{"version":"ifccad.ifcdr.streamDirectory.v1","streams":[
            {"name":"blockInstance","schema":"ifccad.ifcdr.blockInstance.v1","role":"object","count":1,"columns":["entityId","scopeId","definitionScopeId","transform","layerId","appearanceId"]},
            {"name":"entityOrder","schema":"ifccad.ifcdr.entityOrder.v1","role":"order","count":2,"columns":["scopeId","entryOffset","entryCount"],"children":["entityOrderEntry"]},
            {"name":"entityOrderEntry","schema":"ifccad.ifcdr.entityOrderEntry.v1","role":"child","count":1,"columns":["entityId"],"parent":"entityOrder"}
        ]},
        "streams":{
            "blockInstanceStream":{"count":1,"entityId":[3],"scopeId":[7],"definitionScopeId":[21],"transform":[{}],"layerId":[0],"appearanceId":[0]},
            "entityOrderStream":{"count":2,"scopeId":[7,21],"entryOffset":[0,1],"entryCount":[1,0]},
            "entityOrderEntryStream":{"count":1,"entityId":[3]}
        }
    })
}

fn viewport_fixture() -> Value {
    let mut value = block_fixture();
    value["header"]["version"] = json!("0.10.0");
    value["header"]["nextEntityId"] = json!(5);
    value["scopeTable"] = json!([
        {"id":0,"kind":0,"bounds":null},
        {"id":7,"kind":1,"bounds":{"minX":8.0,"minY":19.0,"minZ":0.0,"maxX":12.0,"maxY":21.0,"maxZ":0.0}}
    ]);
    value["blockDefinitionTable"] = json!([]);
    value["appearanceOverrides"] =
        json!([{"id":1,"color":null,"opacity":0.5,"lineWeight":null,"ifcxLinePattern":null}]);
    value["streamDirectory"]["streams"] = json!([
        {"name":"viewport","schema":"ifccad.ifcdr.viewport.v1","role":"object","count":1,"columns":["entityId","scopeId","viewScopeId","frame","view","renderMode","viewEnabled","viewLocked","paperClip","plotShadingOverride","layerId","appearanceId","layerOverrideOffset","layerOverrideCount"],"children":["viewportLayerOverride"]},
        {"name":"viewportLayerOverride","schema":"ifccad.ifcdr.viewportLayerOverride.v1","role":"child","count":1,"columns":["layerId","frozen","appearanceOverrideId"],"parent":"viewport"},
        {"name":"entityOrder","schema":"ifccad.ifcdr.entityOrder.v1","role":"order","count":2,"columns":["scopeId","entryOffset","entryCount"],"children":["entityOrderEntry"]},
        {"name":"entityOrderEntry","schema":"ifccad.ifcdr.entityOrderEntry.v1","role":"child","count":1,"columns":["entityId"],"parent":"entityOrder"}
    ]);
    value["streams"] = json!({
        "viewportStream":{"count":1,"entityId":[4],"scopeId":[7],"viewScopeId":[0],
            "frame":[{"center":{"x":10.0,"y":20.0},"width":4.0,"height":2.0}],
            "view":[{"center":{"x":0.0,"y":0.0},"target":{"x":0.0,"y":0.0,"z":0.0},"direction":{"x":0.0,"y":0.0,"z":1.0},"height":10.0,"twist":0.0,"projection":0,"frontClip":{"mode":0},"backClip":{"mode":0}}],
            "renderMode":[0],"viewEnabled":[true],"viewLocked":[false],"paperClip":[{"enabled":false}],"plotShadingOverride":[null],"layerId":[0],"appearanceId":[0],"layerOverrideOffset":[0],"layerOverrideCount":[1]},
        "viewportLayerOverrideStream":{"count":1,"layerId":[0],"frozen":[true],"appearanceOverrideId":[null]},
        "entityOrderStream":{"count":2,"scopeId":[0,7],"entryOffset":[0,0],"entryCount":[0,1]},
        "entityOrderEntryStream":{"count":1,"entityId":[4]}
    });
    value
}

#[test]
fn viewport_0_10_physical_child_range_decodes_and_checks_partition() {
    let value = viewport_fixture();
    let decoded = decode_json("viewport.json", &value).expect("0.10 viewport physical resource");
    assert_eq!(decoded.viewports().len(), 1);
    assert_eq!(decoded.viewports()[0].layer_overrides.len(), 1);
    let (proof, errors) = validate_resource(decoded).into_parts();
    assert!(proof.is_some(), "{errors:?}");

    let mut noncontiguous = value.clone();
    noncontiguous["streams"]["viewportStream"]["layerOverrideOffset"][0] = json!(1);
    let errors = decode_json("viewport.json", &noncontiguous).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"));
}

#[test]
fn viewport_writer_roundtrips_through_production_decoder() {
    let decoded = decode_json("viewport.json", &viewport_fixture()).unwrap();
    let (proof, errors) = validate_resource(decoded).into_parts();
    let proof = proof.unwrap_or_else(|| panic!("{errors:?}"));
    let encoded = encode_json(&proof).unwrap();
    assert_eq!(encoded.value["header"]["version"], "0.11.0");
    let reread = decode_json("roundtrip.json", &encoded.value).unwrap();
    assert_eq!(reread.viewports(), proof.loaded().resource().viewports());
}

#[test]
fn viewport_0_10_child_stream_requires_exact_nonoverlapping_coverage() {
    let mut value = viewport_fixture();
    let viewport = &mut value["streams"]["viewportStream"];
    viewport["count"] = json!(2);
    let fields = [
        "entityId",
        "scopeId",
        "viewScopeId",
        "frame",
        "view",
        "renderMode",
        "viewEnabled",
        "viewLocked",
        "paperClip",
        "plotShadingOverride",
        "layerId",
        "appearanceId",
    ];
    for field in fields {
        let first = viewport[field][0].clone();
        viewport[field].as_array_mut().unwrap().push(first);
    }
    viewport["entityId"] = json!([4, 5]);
    viewport["layerOverrideOffset"] = json!([0, 0]);
    viewport["layerOverrideCount"] = json!([0, 2]);
    value["streams"]["viewportLayerOverrideStream"] =
        json!({"count":2,"layerId":[0,0],"frozen":[true,false],"appearanceOverrideId":[null,1]});
    value["streams"]["entityOrderStream"]["entryCount"] = json!([0, 2]);
    value["streams"]["entityOrderEntryStream"] = json!({"count":2,"entityId":[4,5]});
    value["header"]["nextEntityId"] = json!(6);
    for entry in value["streamDirectory"]["streams"].as_array_mut().unwrap() {
        match entry["name"].as_str().unwrap() {
            "viewport" | "viewportLayerOverride" | "entityOrderEntry" => entry["count"] = json!(2),
            _ => {}
        }
    }
    let decoded = decode_json("two-viewports.json", &value).unwrap();
    assert_eq!(decoded.viewports()[0].layer_overrides.len(), 0);
    assert_eq!(decoded.viewports()[1].layer_overrides.len(), 2);

    let mut overlap = value.clone();
    overlap["streams"]["viewportStream"]["layerOverrideCount"] = json!([1, 2]);
    assert!(decode_json("overlap.json", &overlap)
        .unwrap_err()
        .iter()
        .any(|error| error.code == "IFCCAD_IFCDR_VIEWPORT_RANGE_INVALID"));
    let mut trailing = value;
    trailing["streams"]["viewportStream"]["layerOverrideCount"] = json!([0, 1]);
    assert!(decode_json("trailing.json", &trailing)
        .unwrap_err()
        .iter()
        .any(|error| error.code == "IFCCAD_IFCDR_VIEWPORT_RANGE_INVALID"));
}

#[test]
fn block_defaults_decode_without_conflating_owner_and_definition() {
    use crate::ifcdr::{BlockScaling, BlockTransform, IfcdrLengthUnit, Point3};
    let decoded = decode_json("blocks.json", &block_fixture()).unwrap();
    assert_eq!(decoded.scopes[0].kind, IfcdrScopeKind::ModelSpace);
    assert_eq!(decoded.scopes[1].kind, IfcdrScopeKind::BlockDefinition);
    let definition = &decoded.block_definitions()[0];
    assert_eq!(definition.scope_id, 21);
    assert_eq!(definition.name, "Door");
    assert_eq!(definition.description, "");
    assert_eq!(definition.base_point, Point3::new(0., 0., 0.));
    assert_eq!(definition.insertion_unit, IfcdrLengthUnit::Unitless);
    assert_eq!(definition.scaling, BlockScaling::Any);
    assert!(definition.explodable);
    assert!(!definition.anonymous);
    let instance = decoded.block_instances().first().unwrap();
    assert_eq!(instance.entity.scope_id, 7);
    assert_eq!(instance.definition_scope_id, 21);
    assert_eq!(instance.transform, BlockTransform::default().components());
    assert!(instance.entity.visible);
}

#[test]
fn block_writer_and_reader_backings_preserve_distinct_owner_and_target_scopes() {
    use crate::ifcdr::write::{prepare_resource, IfcdrWriteEntity, IfcdrWriteInput};
    use crate::ifcdr::Point3;
    let decoded = decode_json("blocks.json", &block_fixture()).unwrap();
    let instance = *decoded.block_instances().first().unwrap();
    let mut entity = instance.entity;
    entity.entity_id = 2;
    entity.scope_id = 21;
    let prepared = prepare_resource(IfcdrWriteInput {
        resource_id: decoded.id.clone(),
        unit: decoded.unit,
        next_entity_id: decoded.next,
        scopes: decoded.scopes.clone(),
        block_definitions: decoded.block_definitions.clone(),
        layers: decoded.layers.clone(),
        appearances: decoded.appearances.clone(),
        overrides: decoded.overrides.clone(),
        entities: vec![
            IfcdrWriteEntity::BlockInstance(instance),
            IfcdrWriteEntity::Line(IfcdrLineRow {
                entity,
                start: Point3::new(2., 0., 0.),
                end: Point3::new(3., 0., 0.),
            }),
        ],
    })
    .unwrap();
    assert_eq!(prepared.order(7), Some([3].as_slice()));
    assert_eq!(prepared.order(21), Some([2].as_slice()));
    let (proof, errors) = validate_resource(prepared).into_parts();
    let proof = proof.unwrap_or_else(|| panic!("{errors:?}"));
    let encoded = encode_json(&proof).unwrap();
    let reloaded = decode_json(
        "blocks.json",
        &serde_json::from_slice(&encoded.bytes).unwrap(),
    )
    .unwrap();
    crate::ifcdr::logical::test_support::assert_resource_eq(proof.loaded().resource(), &reloaded);
    assert!(validate_resource(reloaded).validated().is_some());
}

#[test]
fn block_physical_records_reject_null_partial_records_and_unknown_enum_codes() {
    for (pointer, replacement) in [
        ("/streams/blockInstanceStream/transform/0", Value::Null),
        (
            "/streams/blockInstanceStream/transform/0",
            json!({"placement":{"origin":{"x":0,"y":0,"z":0}}}),
        ),
        (
            "/streams/blockInstanceStream/transform/0",
            json!({"scale":{"x":1,"y":1}}),
        ),
        (
            "/streams/blockInstanceStream/transform/0",
            json!({"rotation":null}),
        ),
        ("/scopeTable/0/kind", json!(3)),
    ] {
        let mut value = block_fixture();
        *value.pointer_mut(pointer).unwrap() = replacement;
        assert!(decode_json("blocks.json", &value).is_err(), "{pointer}");
    }
    for value in [json!(2), Value::Null] {
        let mut input = block_fixture();
        input["blockDefinitionTable"][0]["scaling"] = value;
        assert!(decode_json("blocks.json", &input).is_err());
    }
    let mut value = block_fixture();
    value["streams"]["blockInstanceStream"]
        .as_object_mut()
        .unwrap()
        .remove("transform");
    assert!(decode_json("blocks.json", &value).is_err());
}

fn spatial_fixture() -> Value {
    let mut value = fixture();
    value.as_object_mut().unwrap().remove("bounds");
    for scope in value["scopeTable"].as_array_mut().unwrap() {
        scope["bounds"] =
            json!({"minX":-100,"minY":-100,"minZ":-100,"maxX":100,"maxY":100,"maxZ":100});
    }
    for entry in value["streamDirectory"]["streams"].as_array_mut().unwrap() {
        if entry["name"] == "line" {
            entry["schema"] = json!("ifccad.ifcdr.line.v3");
            entry["columns"].as_array_mut().unwrap().push(json!("z1"));
        }
        if entry["name"] == "planarPolyline" {
            entry["schema"] = json!("ifccad.ifcdr.planarPolyline.v1");
            entry["columns"]
                .as_array_mut()
                .unwrap()
                .push(json!("placement"));
        }
    }
    value["streams"]["lineStream"]["z1"] = json!([4, 0]);
    value["streams"]["planarPolylineStream"]["placement"] = json!([null,
        {"origin":{"x":0,"y":0,"z":5},"X":{"x":0,"y":1,"z":0},"Y":{"x":0,"y":0,"z":1}}]);
    value
}
#[test]
fn spatial_columns_and_scoped_bounds_pass_the_production_decoder() {
    let value = spatial_fixture();
    let decoded = decode_json("spatial.json", &value).unwrap();
    let (proof, errors) = validate_resource(decoded).into_parts();
    assert!(proof.is_some(), "{errors:?}");
}

fn fixture() -> Value {
    let path = crate::conformance::bundled_conformance_root()
        .join("packages/valid/minimal-no-preservation/drawing.ifcdr.json");
    let value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    value
}
#[test]
fn decodes_typed_collections_before_shared_validation() {
    let decoded = decode_json("drawing.ifcdr.json", &fixture()).unwrap();
    assert_eq!(decoded.lines().len(), 2);
    assert!(decoded.lines().get(0).unwrap().entity.visible);
    assert!(decoded.lines().get(2).is_none());
    let (proof, errors) = validate_resource(decoded).into_parts();
    assert!(proof.is_some(), "{errors:?}");
}
#[test]
fn semantic_failures_are_reported_after_physical_decoding() {
    let mut value = fixture();
    value["streams"]["lineStream"]["entityId"][0] = json!(0);
    value["scopeTable"][0]["bounds"] = Value::Null;
    let decoded = decode_json("drawing.ifcdr.json", &value).unwrap();
    let (_, errors) = validate_resource(decoded).into_parts();
    assert!(errors
        .iter()
        .any(|e| e.code == IFCCAD_IFCDR_ENTITY_ID_INVALID));
    assert!(errors.iter().any(|e| e.code == IFCCAD_IFCDR_BOUNDS_INVALID));
}
#[test]
fn broken_packing_does_not_produce_a_candidate() {
    let mut value = fixture();
    value["streams"]["lineStream"]["x1"] = json!([]);
    assert!(decode_json("drawing.ifcdr.json", &value).is_err());
    let mut value = fixture();
    value["streams"]["planarPolylineStream"]["vertexOffset"][0] = json!(u32::MAX);
    assert!(decode_json("drawing.ifcdr.json", &value).is_err());
}
#[test]
fn unsupported_version_is_explicit_without_semantic_cascades() {
    let mut value = fixture();
    value["header"]["version"] = json!("0.6.0");
    let errors = decode_json("drawing.ifcdr.json", &value).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "IFCCAD_IFCDR_VERSION_UNSUPPORTED");
}
#[test]
fn decoding_preserves_integer_identity_above_float_precision() {
    let mut value = fixture();
    value["header"]["nextEntityId"] = json!(9007199254740994u64);
    value["streams"]["lineStream"]["entityId"][0] = json!(9007199254740993u64);
    let decoded = decode_json("drawing.ifcdr.json", &value).unwrap();
    assert_eq!(
        decoded.lines().get(0).unwrap().entity.entity_id,
        9007199254740993
    );
}

#[test]
fn encoder_preserves_validated_identity_bounds_and_unused_overrides() {
    let mut decoded = decode_json("drawing.ifcdr.json", &fixture()).unwrap();
    decoded.next = 100;
    decoded.overrides.push(IfcdrAppearanceOverride {
        id: 8,
        color: Some(IfcdrColor {
            rgb: [8, 9, 10],
            indexed: Some(IfcdrIndexedColor {
                system: "ACI".into(),
                index: 99,
            }),
            named: Some(IfcdrNamedColor {
                catalog: "catalog".into(),
                name: "name".into(),
            }),
        }),
        opacity: Some(0.5),
        line_weight: Some(0.25),
        ifcx_line_pattern: None,
    });
    let (proof, errors) = validate_resource(decoded).into_parts();
    assert!(errors.is_empty());
    let proof = proof.unwrap();
    let encoded = encode_json(&proof).unwrap();
    assert_eq!(encoded.bytes, encode_json(&proof).unwrap().bytes);
    let value: Value = serde_json::from_slice(&encoded.bytes).unwrap();
    assert_eq!(value["header"]["nextEntityId"], 100);
    assert_eq!(
        value["appearanceOverrides"][0]["color"]["namedColor"]["name"],
        "name"
    );
    let decoded = decode_json("drawing.ifcdr.json", &value).unwrap();
    assert_eq!(
        decoded.overrides[0]
            .color
            .as_ref()
            .unwrap()
            .indexed
            .as_ref()
            .unwrap()
            .index,
        99
    );
    crate::ifcdr::logical::test_support::assert_resource_eq(proof.loaded().resource(), &decoded);
    assert!(validate_resource(decoded).validated().is_some());
}

#[test]
fn resource_encoder_preserves_multiple_scopes_and_cross_kind_order() {
    use crate::ifcdr::Bounds3d;
    let mut input = decode_json("drawing.ifcdr.json", &fixture()).unwrap();
    input.scopes.push(IfcdrScope {
        bounds: None,
        id: 7,
        kind: IfcdrScopeKind::PaperSpace,
    });
    input.polylines.entity.scopes[1] = 7;
    input.orders = vec![
        IfcdrScopeOrder {
            scope_id: 0,
            entities: vec![3, 1, 2],
        },
        IfcdrScopeOrder {
            scope_id: 7,
            entities: vec![4],
        },
    ];
    input.next = 100;
    input.scopes[0].bounds = Some(Bounds3d {
        min: crate::ifcdr::Point3::new(-100., -100., 0.),
        max: crate::ifcdr::Point3::new(100., 100., 0.),
    });
    input.scopes[1].bounds = input.scopes[0].bounds;
    let (proof, errors) = validate_resource(input).into_parts();
    assert!(errors.is_empty(), "{errors:?}");
    let proof = proof.unwrap();
    let encoded = encode_json(&proof).unwrap();
    let output = decode_json(
        "drawing.ifcdr.json",
        &serde_json::from_slice(&encoded.bytes).unwrap(),
    )
    .unwrap();
    assert_eq!(output.orders[0].entities, [3, 1, 2]);
    assert_eq!(output.orders[1].entities, [4]);
    assert_eq!(output.scopes[1].kind, IfcdrScopeKind::PaperSpace);
    assert_eq!(
        output.scopes[0].bounds,
        proof.loaded().resource().scopes[0].bounds
    );
    assert!(validate_resource(output).validated().is_some());
}

#[test]
fn closed_color_records_reject_unknown_fields_without_losing_metadata() {
    let mut input = fixture();
    input["appearanceOverrides"] = json!([{"id":8,"color":{"rgb":[1,2,3],"indexedColor":{"system":"ACI","index":8,"unknown":true}},"opacity":null,"ifcxLinePattern":null,"lineWeight":null}]);
    let errors = decode_json("drawing.ifcdr.json", &input).unwrap_err();
    assert!(errors.iter().any(
        |e| e.location.as_deref() == Some("/appearanceOverrides/0/color/indexedColor/unknown")
    ));
    input["appearanceOverrides"][0]["color"] = json!({"rgb":[256,0,0]});
    let errors = decode_json("drawing.ifcdr.json", &input).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.code == IFCCAD_IFCDR_APPEARANCE_INVALID));
}

#[test]
fn closed_directory_rejects_unknown_metadata() {
    let mut input = fixture();
    input["streamDirectory"]["custom"] = json!(true);
    assert!(decode_json("drawing.ifcdr.json", &input).is_err());
}

#[test]
fn resource_preparation_needs_only_resource_data() {
    use crate::ifcdr::write::{prepare_resource, IfcdrWriteEntity, IfcdrWriteInput};
    let decoded = decode_json("drawing.ifcdr.json", &fixture()).unwrap();
    let line = decoded.lines().get(0).unwrap();
    let other_line = decoded.lines().get(1).unwrap();
    let mut scopes = decoded.scopes.clone();
    scopes.push(IfcdrScope {
        bounds: None,
        id: 7,
        kind: IfcdrScopeKind::PaperSpace,
    });
    let polyline = decoded.polylines();
    let polyline = polyline.get(0).unwrap();
    let mut poly_entity = polyline.entity();
    poly_entity.scope_id = 7;
    let input = IfcdrWriteInput {
        block_definitions: vec![],
        resource_id: decoded.id.clone(),
        unit: decoded.unit,
        next_entity_id: 100,
        scopes,
        layers: decoded.layers.clone(),
        appearances: decoded.appearances.clone(),
        overrides: decoded.overrides.clone(),
        entities: vec![
            IfcdrWriteEntity::Line(line),
            IfcdrWriteEntity::PlanarPolyline {
                entity: poly_entity,
                placement: polyline.placement(),
                closed: polyline.closed(),
                bulges: (0..polyline.vertex_count())
                    .map(|i| polyline.bulge(i).unwrap())
                    .collect(),
                points: (0..polyline.vertex_count())
                    .map(|i| polyline.vertex(i).unwrap())
                    .collect(),
            },
            IfcdrWriteEntity::Line(other_line),
        ],
    };
    let prepared = prepare_resource(input).unwrap();
    assert_eq!(prepared.unit(), decoded.unit);
    assert_eq!(
        prepared.order(0),
        Some([line.entity.entity_id, other_line.entity.entity_id].as_slice())
    );
    assert_eq!(prepared.order(7), Some([poly_entity.entity_id].as_slice()));
    let (proof, errors) = validate_resource(prepared).into_parts();
    let proof = proof.unwrap_or_else(|| panic!("{errors:?}"));
    let encoded = encode_json(&proof).unwrap();
    let reloaded = decode_json(
        "resource.json",
        &serde_json::from_slice(&encoded.bytes).unwrap(),
    )
    .unwrap();
    crate::ifcdr::logical::test_support::assert_resource_eq(proof.loaded().resource(), &reloaded);
}

#[test]
fn point_pool_ranges_allow_sharing_and_unused_points() {
    let mut input = fixture();
    input["streams"]["planarPolylineStream"]["vertexOffset"] = json!([1, 1]);
    input["streams"]["planarPolylineStream"]["vertexCount"] = json!([2, 3]);
    // Unselected points are storage only, even when outside declared bounds.
    input["streams"]["planarPolylineStream"]["x"][6] = json!(10000.);
    let decoded = decode_json("resource.json", &input).unwrap();
    let polylines = decoded.polylines();
    assert_eq!(
        polylines.get(0).unwrap().vertex(0),
        polylines.get(1).unwrap().vertex(0)
    );
    let (proof, errors) = validate_resource(decoded).into_parts();
    let proof = proof.unwrap_or_else(|| panic!("{errors:?}"));
    let encoded = encode_json(&proof).unwrap();
    let output = decode_json(
        "resource.json",
        &serde_json::from_slice(&encoded.bytes).unwrap(),
    )
    .unwrap();
    crate::ifcdr::logical::test_support::assert_resource_eq(proof.loaded().resource(), &output);
}

#[test]
fn point_pool_range_rejects_invalid_lengths_and_indices() {
    for (field, value) in [
        ("vertexOffset", json!([-1, 4])),
        ("vertexOffset", json!([0.5, 4])),
        ("vertexOffset", json!([u32::MAX, 4])),
        ("vertexCount", json!([8, 3])),
        ("vertexCount", json!([4294967296u64, 3])),
        ("y", json!([0.])),
    ] {
        let mut input = fixture();
        input["streams"]["planarPolylineStream"][field] = value;
        assert!(decode_json("resource.json", &input).is_err(), "{field}");
    }
}

#[test]
fn child_ranges_require_contiguous_complete_coverage() {
    for (offset, count) in [(1, 3), (0, 3)] {
        let mut input = fixture();
        input["streams"]["entityOrderStream"]["entryOffset"] = json!([offset]);
        input["streams"]["entityOrderStream"]["entryCount"] = json!([count]);
        assert!(decode_json("resource.json", &input).is_err());
    }
    let mut input = fixture();
    input["streams"]["entityOrderStream"] = json!({
        "count": 2, "scopeId": [0, 7], "entryOffset": [0, 2], "entryCount": [2, 2]
    });
    for entry in input["streamDirectory"]["streams"].as_array_mut().unwrap() {
        if entry["name"] == "entityOrder" {
            entry["count"] = json!(2);
        }
    }
    // Physical coverage is valid; scope membership is checked later.
    assert!(decode_json("resource.json", &input).is_ok());
    for offsets in [json!([0, 1]), json!([0, 3]), json!([2, 0])] {
        input["streams"]["entityOrderStream"]["entryOffset"] = offsets;
        assert!(decode_json("resource.json", &input).is_err());
    }
}

#[test]
fn spatial_marker_and_complete_frame_rules_are_physical_constraints() {
    for bad in [
        json!(null),
        json!([]),
        json!([null]),
        json!([null, {}]),
        json!([null,{"origin":{"x":0,"y":0},"X":{"x":1,"y":0,"z":0},"Y":{"x":0,"y":1,"z":0}}]),
    ] {
        let mut value = spatial_fixture();
        value["streams"]["planarPolylineStream"]["placement"] = bad;
        assert!(decode_json("bad.json", &value).is_err());
    }
    let mut value = spatial_fixture();
    value["streams"]["planarPolylineStream"]["placement"][1]["unexpected"] = json!(1);
    assert!(decode_json("bad.json", &value).is_err());
    let mut value = spatial_fixture();
    value["streams"]["lineStream"]["z1"][0] = Value::Null;
    assert!(decode_json("bad.json", &value).is_err());
    let mut value = spatial_fixture();
    value["bounds"] = Value::Null;
    assert!(decode_json("bad.json", &value).is_err());
}
#[test]
fn frame_validity_and_exact_enclosure_are_shared_semantic_rules() {
    let mut value = spatial_fixture();
    value["streams"]["planarPolylineStream"]["placement"][1]["X"]["y"] = json!(2);
    let decoded = decode_json("bad.json", &value).unwrap();
    let (_, errors) = validate_resource(decoded).into_parts();
    assert!(errors
        .iter()
        .any(|d| d.property == "placement" && d.code == IFCCAD_IFCDR_GEOMETRY_INVALID));
    let mut value = spatial_fixture();
    value["scopeTable"][0]["bounds"]["maxZ"] = json!(3.0);
    let (_, errors) = validate_resource(decode_json("bad.json", &value).unwrap()).into_parts();
    assert!(errors
        .iter()
        .any(|d| d.collection == "scope" && d.code == IFCCAD_IFCDR_BOUNDS_INVALID));
}
#[test]
fn explicit_identity_frames_and_zero_z_are_canonically_omitted() {
    let mut value = spatial_fixture();
    let identity =
        json!({"origin":{"x":0,"y":0,"z":0},"X":{"x":1,"y":0,"z":0},"Y":{"x":0,"y":1,"z":0}});
    value["streams"]["planarPolylineStream"]["placement"] = json!([identity.clone(), identity]);
    value["streams"]["lineStream"]["z1"] = json!([0, 0]);
    let (proof, errors) =
        validate_resource(decode_json("valid.json", &value).unwrap()).into_parts();
    assert!(errors.is_empty());
    let proof = proof.unwrap();
    let encoded = encode_json(&proof).unwrap();
    let decoded = decode_json("canonical.json", &encoded.value).unwrap();
    crate::ifcdr::logical::test_support::assert_resource_eq(proof.loaded().resource(), &decoded);
    let (canonical, errors) = validate_resource(decoded).into_parts();
    assert!(errors.is_empty());
    assert_eq!(
        encoded.bytes,
        encode_json(&canonical.unwrap()).unwrap().bytes
    );
    assert!(encoded.value["streams"]["planarPolylineStream"]
        .get("placement")
        .is_none());
    assert!(encoded.value["streams"]["lineStream"].get("z1").is_none());
    assert!(encoded.value["streams"]["lineStream"].get("z2").is_none());
}

#[test]
fn origin_only_placement_preserves_scope_position_and_standard_axes() {
    let mut value = spatial_fixture();
    value["header"]["version"] = json!("0.11.0");
    value["streams"]["planarPolylineStream"]["placement"][1] =
        json!({"origin":{"x":4.0,"y":5.0,"z":6.0}});
    let decoded = decode_json("origin-only.json", &value).unwrap();
    let polylines = decoded.polylines();
    let polyline = polylines.get(1).unwrap();
    let placement = polyline.placement();
    assert_eq!(placement.origin.x(), 4.0);
    assert_eq!(placement.origin.y(), 5.0);
    assert_eq!(placement.origin.z(), 6.0);
    assert_eq!(placement.x.x(), 1.0);
    assert_eq!(placement.x.y(), 0.0);
    assert_eq!(placement.y.x(), 0.0);
    assert_eq!(placement.y.y(), 1.0);
}

#[test]
fn origin_only_placement_rejects_partial_axis_pair() {
    let mut value = spatial_fixture();
    value["header"]["version"] = json!("0.11.0");
    value["streams"]["planarPolylineStream"]["placement"][1] = json!({
        "origin":{"x":4.0,"y":5.0,"z":6.0},
        "X":{"x":1.0,"y":0.0,"z":0.0}
    });
    let errors = decode_json("partial-axes.json", &value).unwrap_err();
    assert!(errors.iter().any(|error| {
        error.location.as_deref() == Some("/streams/planarPolylineStream/placement/1")
            && error.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
    }));
}

#[test]
fn origin_only_encoding_does_not_change_block_transform_placement() {
    let mut value = block_fixture();
    value["header"]["version"] = json!("0.11.0");
    value["streams"]["blockInstanceStream"]["transform"][0]["placement"] =
        json!({"origin":{"x":4.0,"y":5.0,"z":6.0}});
    assert!(decode_json("block-origin-only.json", &value).is_err());
}

#[test]
fn writer_uses_origin_only_placement_for_exact_standard_axes() {
    let mut value = spatial_fixture();
    value["header"]["version"] = json!("0.11.0");
    value["streams"]["planarPolylineStream"]["placement"][1] =
        json!({"origin":{"x":4.0,"y":5.0,"z":6.0}});
    let decoded = decode_json("writer-origin.json", &value).unwrap();
    let (proof, errors) = validate_resource(decoded).into_parts();
    let proof = proof.unwrap_or_else(|| panic!("{errors:?}"));
    let encoded = encode_json(&proof).unwrap();
    assert_eq!(encoded.value["header"]["version"], "0.11.0");
    assert_eq!(
        encoded.value["streams"]["planarPolylineStream"]["placement"][1],
        json!({"origin":{"x":4.0,"y":5.0,"z":6.0}})
    );
    assert!(decode_json("writer-origin-roundtrip.json", &encoded.value).is_ok());
}

#[test]
fn point_stream_has_distinct_identity_and_origin() {
    let mut value = spatial_fixture();
    value["header"]["version"] = json!("0.11.0");
    value["header"]["nextEntityId"] = json!(6);
    value["streamDirectory"]["streams"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "name":"point", "schema":"ifccad.ifcdr.point.v1", "role":"object", "count":1,
            "columns":["entityId","scopeId","placement","layerId","appearanceId"]
        }));
    value["streams"]["pointStream"] = json!({
        "count":1,"entityId":[5],"scopeId":[0],
        "placement":[{"origin":{"x":4.0,"y":5.0,"z":6.0}}],
        "layerId":[0],"appearanceId":[0]
    });
    value["streams"]["entityOrderStream"]["entryCount"][0] = json!(5);
    value["streams"]["entityOrderEntryStream"]["count"] = json!(5);
    value["streams"]["entityOrderEntryStream"]["entityId"]
        .as_array_mut()
        .unwrap()
        .push(json!(5));
    for entry in value["streamDirectory"]["streams"].as_array_mut().unwrap() {
        if entry["name"] == "entityOrderEntry" {
            entry["count"] = json!(5);
        }
    }
    let decoded = decode_json("point.json", &value).unwrap();
    assert_eq!(decoded.points().len(), 1);
    assert_eq!(decoded.points()[0].placement.origin.x(), 4.0);
    assert_eq!(decoded.points()[0].placement.origin.z(), 6.0);
    let (proof, errors) = validate_resource(decoded).into_parts();
    assert!(proof.is_some(), "{errors:?}");
    let encoded = encode_json(&proof.unwrap()).unwrap();
    assert!(decode_json("point-roundtrip.json", &encoded.value).is_ok());
}
