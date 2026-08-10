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
