use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

fn asset(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schemas/ifcdr")
        .join(name);
    serde_json::from_slice(
        &std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    )
    .unwrap()
}

fn validator(name: &str) -> jsonschema::Validator {
    jsonschema::draft202012::new(&asset(name)).unwrap()
}

fn mapping_matches(registry: &Value, mapping: &Value) -> bool {
    let mut expected = BTreeSet::new();
    for field in registry["resource"]["fields"].as_array().unwrap() {
        expected.insert(format!("resource.{}", field["name"].as_str().unwrap()));
    }
    for category in ["tables", "streams"] {
        for group in registry[category].as_array().unwrap() {
            for field in group["fields"].as_array().unwrap() {
                expected.insert(format!(
                    "{}.{}",
                    group["name"].as_str().unwrap(),
                    field["name"].as_str().unwrap()
                ));
            }
        }
    }
    let mut actual = BTreeSet::new();
    for field in mapping["resource"]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .chain(
            ["tables", "streams"]
                .into_iter()
                .flat_map(|category| mapping[category].as_array().unwrap())
                .flat_map(|group| group["fields"].as_array().unwrap()),
        )
    {
        if !actual.insert(field["logical"].as_str().unwrap().to_owned()) {
            return false;
        }
    }
    expected == actual
}

#[test]
fn logical_and_physical_assets_validate_independently() {
    let registry = asset("registry-0.7.0.json");
    let mapping = asset("json-mapping-0.7.0.json");
    let logical = validator("registry-meta-schema-v2.json");
    let physical = validator("json-mapping-meta-schema-v1.json");
    let errors: Vec<_> = logical
        .iter_errors(&registry)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    let errors: Vec<_> = physical
        .iter_errors(&mapping)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    let mut wrong = registry.clone();
    wrong["streams"][0]["payloadKey"] = json!("lineStream");
    assert!(!logical.is_valid(&wrong));
    let mut wrong = registry;
    wrong["streams"][0]["fields"][0]["valueType"] = json!("jsonValue");
    assert!(!logical.is_valid(&wrong));
    let mut wrong = mapping;
    wrong["streams"][0]["fields"][0]["default"] = json!(true);
    assert!(
        !physical.is_valid(&wrong),
        "mapping must not redefine logical defaults"
    );
}

#[test]
fn mapping_rejects_missing_duplicate_and_unknown_logical_properties() {
    let registry = asset("registry-0.7.0.json");
    let mapping = asset("json-mapping-0.7.0.json");
    assert!(mapping_matches(&registry, &mapping));
    let mut missing = mapping.clone();
    let line = missing["streams"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|s| s["name"] == "line")
        .unwrap();
    line["fields"]
        .as_array_mut()
        .unwrap()
        .retain(|f| f["logical"] != "line.layerId");
    assert!(!mapping_matches(&registry, &missing));
    let mut duplicate = mapping.clone();
    let field = duplicate["streams"][0]["fields"][0].clone();
    duplicate["streams"][0]["fields"]
        .as_array_mut()
        .unwrap()
        .push(field);
    assert!(!mapping_matches(&registry, &duplicate));
    let mut unknown = mapping;
    unknown["streams"][0]["fields"][0]["logical"] = json!("hatch.pattern");
    assert!(!mapping_matches(&registry, &unknown));
}

#[test]
fn retained_vocabulary_has_typed_defaults_and_sequence_semantics() {
    let registry = asset("registry-0.7.0.json");
    let names = |category: &str| -> BTreeSet<String> {
        registry[category]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["name"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(
        names("streams"),
        BTreeSet::from(["line", "polyline", "entityOrder", "entityOrderEntry"].map(String::from))
    );
    assert_eq!(
        names("tables"),
        BTreeSet::from(
            [
                "scope",
                "layerBinding",
                "appearanceBinding",
                "appearanceOverride"
            ]
            .map(String::from)
        )
    );
    for kind in ["line", "polyline"] {
        let stream = registry["streams"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["name"] == kind)
            .unwrap();
        let field = stream["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == "visible")
            .unwrap();
        assert_eq!(field["default"], true);
    }
    assert_eq!(
        registry["types"]["appearanceMode"]["values"],
        json!(["ByLayer", "Explicit", "ByBlock"])
    );
    assert_eq!(registry["types"]["vertices"]["minItems"], 2);
    assert_eq!(registry["types"]["color"]["fields"][0]["name"], "rgb");
}

#[test]
fn point_pool_mapping_requires_exactly_two_coordinate_columns() {
    let schema = validator("json-mapping-meta-schema-v1.json");
    for pools in [json!([]), json!(["x"]), json!(["x", "y", "z"])] {
        let mut mapping = asset("json-mapping-0.7.0.json");
        let field = mapping["streams"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|s| s["name"] == "polyline")
            .unwrap()["fields"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|f| f["logical"] == "polyline.vertices")
            .unwrap();
        field["pools"] = pools;
        assert!(!schema.is_valid(&mapping));
    }
}
