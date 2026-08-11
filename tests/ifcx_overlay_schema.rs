use jsonschema::Registry;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const RESOURCE_OVERLAY_ID: &str = "https://schemas.ifccad.org/ifcx/ifccad-overlay-0.1.0.json";
const DRAWING_CORE_ID: &str = "https://schemas.ifccad.org/ifcx/ifccad-drawing-core-0.1.0.json";
const OVERLAY_0_2_ID: &str = "https://schemas.ifccad.org/ifcx/ifccad-overlay-0.2.0.json";

fn schema_path(file_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schemas")
        .join("ifcx")
        .join(file_name)
}

fn load_schema(file_name: &str) -> Value {
    let bytes = fs::read(schema_path(file_name)).expect("read IFCX schema");
    serde_json::from_slice(&bytes).expect("parse IFCX schema")
}

fn resource_overlay_schema() -> Value {
    load_schema("ifccad-overlay-0.1.0.json")
}

fn resource_overlay_validator() -> jsonschema::Validator {
    let schema = resource_overlay_schema();
    jsonschema::draft202012::new(&schema).expect("compile IFCX resource overlay")
}

fn drawing_core_schema() -> Value {
    load_schema("ifccad-drawing-core-0.1.0.json")
}

fn drawing_core_validator() -> jsonschema::Validator {
    let schema = drawing_core_schema();
    jsonschema::draft202012::new(&schema).expect("compile IFCX drawing core")
}

fn composite_overlay_schema() -> Value {
    load_schema("ifccad-overlay-0.2.0.json")
}

fn composite_overlay_validator() -> jsonschema::Validator {
    let registry = Registry::new()
        .add(RESOURCE_OVERLAY_ID, resource_overlay_schema())
        .expect("add IFCX resource overlay to local registry")
        .add(DRAWING_CORE_ID, drawing_core_schema())
        .expect("add IFCX drawing core to local registry")
        .prepare()
        .expect("prepare local IFCX schema registry");
    let schema = composite_overlay_schema();
    jsonschema::draft202012::options()
        .with_registry(&registry)
        .build(&schema)
        .expect("compile composite IFCX overlay from local registry")
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

fn drawing_set_node() -> Value {
    json!({
        "path": "drawing-set-main",
        "type": "openaec:DrawingSet",
        "attributes": {
            "futureAttribute": true
        },
        "children": {
            "Drawings": ["drawing-main"],
            "PreservationResources": ["drawing-1/preservation"],
            "FutureRelationship": ["future-node"]
        },
        "futureNodeProperty": true
    })
}

fn drawing_node() -> Value {
    json!({
        "path": "drawing-main",
        "type": "openaec:Drawing",
        "attributes": {
            "futureAttribute": true
        },
        "children": {
            "Representation": "drawing-1/geometry",
            "Layouts": ["layout-model"],
            "Workspace": "workspace-main"
        },
        "futureNodeProperty": true
    })
}

fn drawing_layout_node() -> Value {
    json!({
        "path": "layout-model",
        "type": "openaec:DrawingLayout",
        "attributes": {
            "name": "Model",
            "kind": "model",
            "scopeId": 0,
            "tabOrder": 0,
            "futureAttribute": true
        },
        "children": {
            "Representation": "drawing-1/geometry",
            "MainViewState": "stored-view-main"
        },
        "futureNodeProperty": true
    })
}

fn layer_node() -> Value {
    json!({
        "path": "layer-wall",
        "type": "openaec:Layer",
        "attributes": {
            "name": "A-WALL",
            "visible": true,
            "appearance": "appearance-wall",
            "futureAttribute": true
        },
        "children": {
            "FutureRelationship": "future-node"
        },
        "futureNodeProperty": true
    })
}

fn appearance_node() -> Value {
    json!({
        "path": "appearance-wall",
        "type": "openaec:Appearance",
        "attributes": {
            "futureStyleBinding": {
                "color": "#ffffff",
                "opacity": 0.8
            }
        },
        "children": {
            "FutureRelationship": "future-node"
        },
        "futureNodeProperty": true
    })
}

fn composite_document() -> Value {
    json!({
        "header": {"schemaIdentifiers": ["future-ifcx"]},
        "futureRootProperty": true,
        "data": [
            drawing_set_node(),
            drawing_node(),
            drawing_layout_node(),
            geometry_node(),
            layer_node(),
            appearance_node(),
            preservation_node(),
            {"type": "example:UnknownNode", "anything": true}
        ]
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
    let schema = resource_overlay_schema();
    assert_eq!(schema["$id"], RESOURCE_OVERLAY_ID);
    jsonschema::draft202012::meta::validate(&schema)
        .expect("active IFCX overlay must satisfy the Draft 2020-12 meta-schema");
}

#[test]
fn overlay_requires_only_the_minimal_ifcx_envelope() {
    let validator = resource_overlay_validator();

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
    let validator = resource_overlay_validator();
    assert!(validator.is_valid(&json!({
        "data": [geometry_node(), preservation_node()]
    })));
}

#[test]
fn geometry_descriptor_requires_the_owned_contract() {
    let validator = resource_overlay_validator();

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
    let validator = resource_overlay_validator();

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
        ("/attributes/preservation/format", json!("example:other")),
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

#[test]
fn package_resource_uris_match_the_rust_syntax_boundary() {
    let validator = resource_overlay_validator();

    for uri in [
        "drawing.ifcdr.json",
        "resources/drawing.ifcdr.json",
        "blobs/0123456789abcdef",
    ] {
        let mut node = geometry_node();
        *node
            .pointer_mut("/attributes/geometry/uri")
            .expect("geometry URI") = json!(uri);
        assert!(
            validator.is_valid(&json!({"data": [node]})),
            "rejected {uri:?}"
        );
    }

    for uri in [
        "",
        "/absolute.json",
        "C:/absolute.json",
        "scheme:value.json",
        r"dir\file.json",
        "dir//file.json",
        "./file.json",
        "dir/../file.json",
        "dir/.",
        "trailing/",
        "nul\0byte.json",
    ] {
        let mut node = geometry_node();
        *node
            .pointer_mut("/attributes/geometry/uri")
            .expect("geometry URI") = json!(uri);
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted {uri:?}"
        );
    }
}

#[test]
fn owned_descriptors_are_closed_but_surrounding_ifcx_content_is_open() {
    let validator = resource_overlay_validator();
    let mut geometry = geometry_node();
    geometry["attributes"]["geometry"]["futureField"] = json!(true);
    assert!(!validator.is_valid(&json!({"data": [geometry]})));

    let mut preservation = preservation_node();
    preservation["attributes"]["preservation"]["futureField"] = json!(true);
    assert!(!validator.is_valid(&json!({"data": [preservation]})));
}

#[test]
fn preservation_links_are_non_empty_unique_uris_with_the_correct_name() {
    let validator = resource_overlay_validator();

    let mut empty = preservation_node();
    empty["attributes"]["preservation"]["linkedDrawingResourceUris"] = json!([]);
    assert!(!validator.is_valid(&json!({"data": [empty]})));

    let mut duplicate = preservation_node();
    duplicate["attributes"]["preservation"]["linkedDrawingResourceUris"] = json!([
        "resources/drawing.ifcdr.json",
        "resources/drawing.ifcdr.json"
    ]);
    assert!(!validator.is_valid(&json!({"data": [duplicate]})));

    let mut historical_name = preservation_node();
    let descriptor = historical_name["attributes"]["preservation"]
        .as_object_mut()
        .expect("preservation descriptor");
    let links = descriptor
        .remove("linkedDrawingResourceUris")
        .expect("current link property");
    descriptor.insert("linkedDrawingResourceIds".to_owned(), links);
    assert!(!validator.is_valid(&json!({"data": [historical_name]})));
}

#[test]
fn drawing_core_is_a_valid_draft_2020_12_schema() {
    let schema = drawing_core_schema();
    assert_eq!(schema["$id"], DRAWING_CORE_ID);
    jsonschema::draft202012::meta::validate(&schema)
        .expect("IFCX drawing core must satisfy the Draft 2020-12 meta-schema");
}

#[test]
fn drawing_core_accepts_the_open_minimal_spine() {
    let validator = drawing_core_validator();
    assert!(validator.is_valid(&json!({
        "header": {"futureHeaderProperty": true},
        "futureRootProperty": true,
        "data": [
            drawing_set_node(),
            drawing_node(),
            drawing_layout_node(),
            {"type": "example:UnknownNode", "anything": true}
        ]
    })));
}

#[test]
fn drawing_set_requires_only_its_local_spine() {
    let validator = drawing_core_validator();

    for pointer in ["/path", "/children", "/children/Drawings"] {
        let mut node = drawing_set_node();
        remove_property(&mut node, pointer);
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted DrawingSet without {pointer}"
        );
    }

    let mut empty = drawing_set_node();
    empty["children"]["Drawings"] = json!([]);
    assert!(validator.is_valid(&json!({"data": [empty]})));

    for (pointer, value) in [
        ("/path", json!("")),
        (
            "/children/Drawings",
            json!(["drawing-main", "drawing-main"]),
        ),
        ("/children/Drawings", json!([42])),
        ("/children/PreservationResources", json!([""])),
        ("/attributes", json!(false)),
    ] {
        let mut node = drawing_set_node();
        *node.pointer_mut(pointer).expect("DrawingSet test pointer") = value;
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted invalid DrawingSet value at {pointer}"
        );
    }
}

#[test]
fn drawing_requires_representation_and_nonempty_layouts() {
    let validator = drawing_core_validator();

    for pointer in [
        "/path",
        "/children",
        "/children/Representation",
        "/children/Layouts",
    ] {
        let mut node = drawing_node();
        remove_property(&mut node, pointer);
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted Drawing without {pointer}"
        );
    }

    for (pointer, value) in [
        ("/path", json!("")),
        ("/children/Representation", json!(42)),
        ("/children/Layouts", json!([])),
        ("/children/Layouts", json!(["layout-model", "layout-model"])),
        ("/children/Layouts", json!([false])),
        ("/attributes", json!([])),
    ] {
        let mut node = drawing_node();
        *node.pointer_mut(pointer).expect("Drawing test pointer") = value;
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted invalid Drawing value at {pointer}"
        );
    }
}

#[test]
fn drawing_layout_requires_stable_local_properties() {
    let validator = drawing_core_validator();

    for pointer in [
        "/path",
        "/attributes",
        "/attributes/name",
        "/attributes/kind",
        "/attributes/scopeId",
        "/children",
        "/children/Representation",
    ] {
        let mut node = drawing_layout_node();
        remove_property(&mut node, pointer);
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted DrawingLayout without {pointer}"
        );
    }

    for (pointer, value) in [
        ("/path", json!("")),
        ("/attributes/name", json!("")),
        ("/attributes/kind", json!("sheet")),
        ("/attributes/scopeId", json!(-1)),
        ("/attributes/scopeId", json!(1.5)),
        ("/attributes/scopeId", json!(true)),
        ("/attributes/scopeId", json!("0")),
        ("/children/Representation", Value::Null),
    ] {
        let mut node = drawing_layout_node();
        *node
            .pointer_mut(pointer)
            .expect("DrawingLayout test pointer") = value;
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted invalid DrawingLayout value at {pointer}"
        );
    }
}

#[test]
fn drawing_core_accepts_layer_and_open_appearance_nodes() {
    let validator = drawing_core_validator();
    assert!(validator.is_valid(&json!({"data": [layer_node(), appearance_node()]})));

    let mut empty = appearance_node();
    empty["attributes"] = json!({});
    assert!(validator.is_valid(&json!({"data": [empty]})));
}

#[test]
fn layer_requires_only_name_and_visibility() {
    let validator = drawing_core_validator();

    for pointer in [
        "/path",
        "/attributes",
        "/attributes/name",
        "/attributes/visible",
    ] {
        let mut node = layer_node();
        remove_property(&mut node, pointer);
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted Layer without {pointer}"
        );
    }

    for (pointer, value) in [
        ("/path", json!("")),
        ("/attributes/name", json!("")),
        ("/attributes/visible", json!("true")),
        ("/attributes/appearance", json!("")),
        ("/attributes/appearance", json!(42)),
        ("/children", json!([])),
    ] {
        let mut node = layer_node();
        *node.pointer_mut(pointer).expect("Layer test pointer") = value;
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted invalid Layer value at {pointer}"
        );
    }
}

#[test]
fn appearance_requires_an_open_attributes_object() {
    let validator = drawing_core_validator();

    for pointer in ["/path", "/attributes"] {
        let mut node = appearance_node();
        remove_property(&mut node, pointer);
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted Appearance without {pointer}"
        );
    }

    for (pointer, value) in [
        ("/path", json!("")),
        ("/attributes", json!("future")),
        ("/children", json!(false)),
    ] {
        let mut node = appearance_node();
        *node.pointer_mut(pointer).expect("Appearance test pointer") = value;
        assert!(
            !validator.is_valid(&json!({"data": [node]})),
            "accepted invalid Appearance value at {pointer}"
        );
    }
}

#[test]
fn composite_overlay_is_valid_and_compiles_offline() {
    let schema = composite_overlay_schema();
    assert_eq!(schema["$id"], OVERLAY_0_2_ID);
    jsonschema::draft202012::meta::validate(&schema)
        .expect("composite IFCX overlay must satisfy the Draft 2020-12 meta-schema");

    assert!(composite_overlay_validator().is_valid(&composite_document()));
}

#[test]
fn composite_overlay_enforces_resource_and_drawing_constraints() {
    let validator = composite_overlay_validator();

    let mut malformed_resource = composite_document();
    malformed_resource["data"][3]["attributes"]["geometry"]["format"] = json!("example:other");
    assert!(!validator.is_valid(&malformed_resource));

    let mut malformed_drawing = composite_document();
    malformed_drawing["data"][2]["attributes"]["kind"] = json!("sheet");
    assert!(!validator.is_valid(&malformed_drawing));
}
