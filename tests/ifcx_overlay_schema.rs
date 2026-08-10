use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const OVERLAY_ID: &str =
    "https://schemas.ifccad.org/ifcx/ifccad-overlay-0.1.0.json";

fn overlay_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schemas")
        .join("ifcx")
        .join("ifccad-overlay-0.1.0.json")
}

fn overlay_schema() -> Value {
    let bytes = fs::read(overlay_path()).expect("read active IFCX overlay");
    serde_json::from_slice(&bytes).expect("parse active IFCX overlay")
}

fn overlay_validator() -> jsonschema::Validator {
    let schema = overlay_schema();
    jsonschema::draft202012::new(&schema).expect("compile active IFCX overlay")
}

fn checksum() -> String {
    format!("sha256:{}", "a".repeat(64))
}

fn geometry_node() -> Value {
    json!({
        "path": "drawing-1/geometry",
        "type": "openaec:DrawingGeometryRepresentation",
        "attributes": {
            "geometry": {
                "format": "openaec.ifcdr",
                "version": "0.5.0",
                "uri": "resources/drawing.ifcdr.json",
                "checksum": checksum(),
                "role": "paperspace"
            },
            "futureAttribute": true
        },
        "children": [],
        "futureNodeProperty": true
    })
}

fn preservation_node() -> Value {
    json!({
        "path": "drawing-1/preservation",
        "type": "openaec:PreservationRepresentation",
        "attributes": {
            "preservation": {
                "format": "openaec.ifcpr",
                "version": "0.1.0",
                "uri": "resources/preservation.ifcpr.json",
                "checksum": checksum(),
                "sourceDocumentId": "source-drawing-1",
                "linkedDrawingResourceUris": [
                    "resources/drawing.ifcdr.json"
                ]
            }
        }
    })
}

fn remove_property(value: &mut Value, pointer: &str) {
    let (parent, property) = pointer.rsplit_once('/').expect("property pointer");
    value
        .pointer_mut(parent)
        .expect("property parent")
        .as_object_mut()
        .expect("property parent object")
        .remove(property)
        .expect("property exists");
}

#[test]
fn active_overlay_is_a_valid_draft_2020_12_schema() {
    let schema = overlay_schema();
    assert_eq!(schema["$id"], OVERLAY_ID);
    jsonschema::draft202012::meta::validate(&schema)
        .expect("active IFCX overlay must satisfy the Draft 2020-12 meta-schema");
}

#[test]
fn overlay_requires_only_the_minimal_ifcx_envelope() {
    let validator = overlay_validator();

    assert!(validator.is_valid(&json!({"data": []})));
    assert!(validator.is_valid(&json!({
        "header": {"schemaIdentifiers": ["future-ifcx"]},
        "imports": [],
        "futureRootProperty": true,
        "data": [{
            "type": "example:UnrelatedNode",
            "futureNodeProperty": {"anything": true}
        }]
    })));
    assert!(!validator.is_valid(&json!({})));
    assert!(!validator.is_valid(&json!({"data": {}})));
    assert!(!validator.is_valid(&json!({"data": [42]})));
}

#[test]
fn overlay_accepts_both_recognized_resource_nodes() {
    let validator = overlay_validator();
    assert!(validator.is_valid(&json!({
        "data": [geometry_node(), preservation_node()]
    })));
}

#[test]
fn geometry_descriptor_requires_the_owned_contract() {
    let validator = overlay_validator();

    for pointer in [
        "/path",
        "/attributes/geometry/format",
        "/attributes/geometry/version",
        "/attributes/geometry/uri",
        "/attributes/geometry/checksum",
        "/attributes/geometry/role",
    ] {
        let mut node = geometry_node();
        remove_property(&mut node, pointer);
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted geometry node after removing {pointer}"
        );
    }

    for (pointer, value) in [
        ("/attributes/geometry/format", json!("example:other")),
        ("/attributes/geometry/version", json!("0.6.0")),
        ("/attributes/geometry/checksum", json!("sha256:ABC")),
        ("/attributes/geometry/role", json!("")),
    ] {
        let mut node = geometry_node();
        *node.pointer_mut(pointer).expect("geometry test pointer") = value;
        assert!(!validator.is_valid(&json!({"data": [node]})));
    }
}

#[test]
fn preservation_descriptor_requires_the_owned_contract() {
    let validator = overlay_validator();

    for pointer in [
        "/path",
        "/attributes/preservation/format",
        "/attributes/preservation/version",
        "/attributes/preservation/uri",
        "/attributes/preservation/checksum",
        "/attributes/preservation/sourceDocumentId",
        "/attributes/preservation/linkedDrawingResourceUris",
    ] {
        let mut node = preservation_node();
        remove_property(&mut node, pointer);
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted preservation node after removing {pointer}"
        );
    }

    for (pointer, value) in [
        (
            "/attributes/preservation/format",
            json!("example:other"),
        ),
        ("/attributes/preservation/version", json!("0.2.0")),
        ("/attributes/preservation/checksum", json!("sha256:ABC")),
        ("/attributes/preservation/sourceDocumentId", json!("")),
    ] {
        let mut node = preservation_node();
        *node
            .pointer_mut(pointer)
            .expect("preservation test pointer") = value;
        assert!(!validator.is_valid(&json!({"data": [node]})));
    }
}
