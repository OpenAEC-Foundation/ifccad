use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

fn read_json(path: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    serde_json::from_slice(
        &std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap()
}

fn field_names(record: &Value) -> Vec<&str> {
    record["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["name"].as_str().unwrap())
        .collect()
}

fn named<'a>(records: &'a Value, name: &str) -> &'a Value {
    records
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["name"] == name)
        .unwrap_or_else(|| panic!("missing {name}"))
}

#[test]
fn logical_and_json_mapping_are_closed_and_cover_the_same_fields() {
    let registry = read_json("schemas/ocdraw/registry-0.1.0.json");
    let mapping = read_json("schemas/ocdraw/json-mapping-0.1.0.json");
    for (meta_path, value) in [
        ("schemas/ocdraw/registry-meta-schema-v1.json", &registry),
        ("schemas/ocdraw/json-mapping-meta-schema-v1.json", &mapping),
    ] {
        let meta = read_json(meta_path);
        let validator = jsonschema::draft202012::new(&meta).unwrap();
        let errors: Vec<_> = validator
            .iter_errors(value)
            .map(|error| error.to_string())
            .collect();
        assert!(errors.is_empty(), "{meta_path}: {errors:?}");
    }

    let mut expected = BTreeSet::new();
    for item in registry["resource"]["fields"].as_array().unwrap() {
        expected.insert(format!("resource.{}", item["name"].as_str().unwrap()));
    }
    for category in ["tables", "streams"] {
        for group in registry[category].as_array().unwrap() {
            for item in group["fields"].as_array().unwrap() {
                expected.insert(format!(
                    "{}.{}",
                    group["name"].as_str().unwrap(),
                    item["name"].as_str().unwrap()
                ));
            }
        }
    }
    let actual: Vec<_> = mapping["resource"]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .chain(
            ["tables", "streams"]
                .into_iter()
                .flat_map(|category| mapping[category].as_array().unwrap())
                .flat_map(|group| group["fields"].as_array().unwrap()),
        )
        .map(|item| item["logical"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(actual.len(), actual.iter().collect::<BTreeSet<_>>().len());
    assert_eq!(expected, actual.into_iter().collect());
}

#[test]
fn ocdraw_registry_owns_layers_layouts_and_entity_appearance() {
    let registry = read_json("schemas/ocdraw/registry-0.1.0.json");
    let mapping = read_json("schemas/ocdraw/json-mapping-0.1.0.json");
    assert_eq!(registry["ocdrawVersion"], "0.1.0");
    assert_eq!(mapping["ocdrawVersion"], "0.1.0");
    assert_eq!(
        mapping["resource"]["constants"]["header.format"],
        "open_cad_drawing"
    );
    assert_eq!(mapping["resource"]["constants"]["header.version"], "0.1.0");

    let resource_fields = field_names(&registry["resource"]);
    for name in [
        "drawingId",
        "unit",
        "nextEntityId",
        "nextLayerId",
        "nextLayoutId",
        "plotStyleMode",
    ] {
        assert!(
            resource_fields.contains(&name),
            "missing document field {name}"
        );
    }
    assert!(!resource_fields.contains(&"resourceId"));
    let layers = field_names(named(&registry["tables"], "layer"));
    for name in [
        "id",
        "name",
        "visible",
        "frozen",
        "locked",
        "plottable",
        "frozenInNewViewports",
        "color",
        "opacity",
        "linePattern",
        "lineWeight",
    ] {
        assert!(layers.contains(&name), "missing Layer field {name}");
    }
    let layouts = field_names(named(&registry["tables"], "layout"));
    for name in ["id", "scopeId", "kind", "name", "tabIndex"] {
        assert!(layouts.contains(&name), "missing Layout field {name}");
    }
    let table_names: Vec<_> = registry["tables"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    assert!(!table_names.contains(&"layerBinding"));
    assert!(!table_names.contains(&"appearanceBinding"));
    assert!(!table_names.contains(&"appearanceOverride"));
    let line = field_names(named(&registry["streams"], "line"));
    assert!(!line.contains(&"appearanceId"));
    for name in [
        "layerId",
        "colorMode",
        "color",
        "opacityMode",
        "opacity",
        "linePatternMode",
        "linePattern",
        "lineWeightMode",
        "lineWeight",
    ] {
        assert!(
            line.contains(&name),
            "missing entity appearance field {name}"
        );
    }
}

#[test]
fn minimal_empty_drawing_matches_its_document_schema() {
    let schema = read_json("schemas/ocdraw/schema-0.1.0.json");
    let fixture = read_json("conformance/next/ocdraw/valid/empty.ocdraw.json");
    let validator = jsonschema::draft202012::new(&schema).unwrap();
    assert!(validator.is_valid(&fixture));
    assert_eq!(fixture["header"]["format"], "open_cad_drawing");
    assert_eq!(fixture["header"]["version"], "0.1.0");
    assert_eq!(fixture["header"]["unit"], "unitless");
    assert!(fixture.get("layers").is_none());
    assert!(fixture.get("drawingWorkspaceState").is_none());
    assert!(fixture.get("plotStyleMode").is_none());
    assert_eq!(fixture["layouts"].as_array().unwrap().len(), 1);
    assert_eq!(fixture["scopes"].as_array().unwrap().len(), 1);

    let mut unknown = fixture.clone();
    unknown["inventedCoreField"] = json!(true);
    assert!(!validator.is_valid(&unknown));
    let mut other_version = fixture;
    other_version["header"]["version"] = json!("0.2.0");
    assert!(!validator.is_valid(&other_version));
}
