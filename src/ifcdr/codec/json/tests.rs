use super::*;
use crate::ifcdr::logical::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let path = crate::conformance::bundled_conformance_root()
        .join("packages/valid/minimal-no-preservation/drawing.ifcdr.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    value["header"]["version"] = json!("0.7.0");
    for entry in value["streamDirectory"]["streams"].as_array_mut().unwrap() {
        if entry["name"] == "polyline" {
            entry["schema"] = json!("ifccad.ifcdr.polyline.v3");
        }
    }
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
    value["bounds"] = Value::Null;
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
    value["streams"]["polylineStream"]["vertexOffset"][0] = json!(u32::MAX);
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
    use crate::ifcdr::{Bounds2d, Point2};
    let mut input = decode_json("drawing.ifcdr.json", &fixture()).unwrap();
    input.scopes.push(IfcdrScope {
        id: 7,
        kind: 1,
        name: "Paper".into(),
        base: Point2::new(1000., 2000.),
        flags: 9,
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
    input.bounds = Some(Bounds2d {
        min: Point2::new(-100., -100.),
        max: Point2::new(100., 100.),
    });
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
    assert_eq!(output.scopes[1].base, Point2::new(1000., 2000.));
    assert_eq!(output.scopes[1].flags, 9);
    assert_eq!(output.bounds, proof.loaded().resource().bounds);
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
        id: 7,
        kind: 1,
        name: "Other".into(),
        base: crate::ifcdr::Point2::new(100., 200.),
        flags: 0,
    });
    let polyline = decoded.polylines();
    let polyline = polyline.get(0).unwrap();
    let mut poly_entity = polyline.entity();
    poly_entity.scope_id = 7;
    let input = IfcdrWriteInput {
        resource_id: decoded.id.clone(),
        unit: decoded.unit,
        next_entity_id: 100,
        scopes,
        layers: decoded.layers.clone(),
        appearances: decoded.appearances.clone(),
        overrides: decoded.overrides.clone(),
        entities: vec![
            IfcdrWriteEntity::Line(line),
            IfcdrWriteEntity::Polyline {
                entity: poly_entity,
                closed: polyline.closed(),
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
    input["streams"]["polylineStream"]["vertexOffset"] = json!([1, 1]);
    input["streams"]["polylineStream"]["vertexCount"] = json!([2, 3]);
    // Unselected points are storage only, even when outside declared bounds.
    input["streams"]["polylineStream"]["x"][6] = json!(10000.);
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
        input["streams"]["polylineStream"][field] = value;
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
