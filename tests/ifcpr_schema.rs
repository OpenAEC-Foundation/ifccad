use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn schema() -> Value {
    let bytes = fs::read(repository_root().join("schemas/ifcpr/schema-0.2.0.json"))
        .expect("read IFCPR 0.2.0 schema");
    serde_json::from_slice(&bytes).expect("parse IFCPR 0.2.0 schema")
}

fn migrated_fixture() -> Value {
    let path = repository_root()
        .join("conformance/1.0.0/packages/valid/unrepresented-packed/preservation.ifcpr.json");
    let mut fixture: Value = serde_json::from_slice(&fs::read(path).expect("read IFCPR fixture"))
        .expect("parse IFCPR fixture");
    fixture["header"]["version"] = serde_json::json!("0.2.0");
    fixture["header"].as_object_mut().unwrap().remove("uri");
    fixture["linkedDrawingResources"][0] = serde_json::json!("geometry-modelspace-main");
    fixture["projectionBindings"][0]["modelTargets"][0]["resourceId"] =
        serde_json::json!("geometry-modelspace-main");
    fixture
}

#[test]
fn ifcpr_0_2_uses_logical_resource_ids_without_header_uri() {
    let schema = schema();
    jsonschema::draft202012::meta::validate(&schema)
        .expect("IFCPR 0.2.0 must satisfy the Draft 2020-12 meta-schema");
    let validator = jsonschema::draft202012::new(&schema).expect("compile IFCPR 0.2.0 schema");
    let migrated = migrated_fixture();

    assert!(validator.is_valid(&migrated));
    assert!(migrated["header"].get("uri").is_none());
    assert_eq!(
        migrated["header"]["resourceId"],
        "preservation-golden-source"
    );
    assert_eq!(
        migrated["linkedDrawingResources"][0],
        "geometry-modelspace-main"
    );
}

#[test]
fn ifcpr_0_2_requires_header_resource_id() {
    let schema = schema();
    let validator = jsonschema::draft202012::new(&schema).expect("compile IFCPR 0.2.0 schema");
    let mut missing = migrated_fixture();
    missing["header"]
        .as_object_mut()
        .unwrap()
        .remove("resourceId");

    assert!(!validator.is_valid(&missing));
}

#[test]
fn ifcpr_0_2_does_not_apply_uri_syntax_to_resource_ids() {
    let schema = schema();
    let validator = jsonschema::draft202012::new(&schema).expect("compile IFCPR 0.2.0 schema");
    let mut resource = migrated_fixture();
    resource["linkedDrawingResources"][0] = serde_json::json!("identity:with/slashes");

    assert!(validator.is_valid(&resource));
}
