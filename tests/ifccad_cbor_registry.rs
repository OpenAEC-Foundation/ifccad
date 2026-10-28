use serde_json::Value;
use std::collections::BTreeSet;

fn registry() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/ifccad/cbor-mapping-2.json");
    serde_json::from_slice(&std::fs::read(path).expect("CBOR register must exist")).unwrap()
}

#[test]
fn registry_pins_root_document_and_current_families() {
    let r = registry();
    assert_eq!(r["format"], "IFCCAD-CBOR");
    assert_eq!(r["mappingRevision"], 2);
    let types = r["types"].as_object().unwrap();
    assert_eq!(
        types["IfccadDocument"]["fields"].as_array().unwrap().len(),
        20
    );
    assert_eq!(
        types["IfccadIdCounters"]["fields"]
            .as_array()
            .unwrap()
            .len(),
        11
    );
    let kinds = types["IfccadEntityKind"]["variants"].as_array().unwrap();
    assert_eq!(kinds.len(), 16);
    assert_eq!(
        kinds.iter().find(|v| v["name"] == "Dimension").unwrap()["code"],
        13
    );
    assert_eq!(
        types["DimensionKind"]["variants"].as_array().unwrap().len(),
        9
    );
    assert_eq!(
        types["DimensionDefinition"]["variants"]
            .as_array()
            .unwrap()
            .len(),
        9
    );
    for variant in types["DimensionDefinition"]["variants"].as_array().unwrap() {
        for field in variant["fields"].as_array().unwrap() {
            match field["name"].as_str().unwrap() {
                "first_point" | "second_point" => assert_eq!(field["type"], "[f64;3]"),
                "dimension_line_point" => assert_eq!(field["type"], "[f64;2]"),
                _ => {}
            }
        }
    }
}

#[test]
fn registry_has_local_unique_codes_and_resolved_types() {
    let r = registry();
    let types = r["types"].as_object().unwrap();
    for (name, ty) in types {
        for collection in ["fields", "variants"] {
            if let Some(fields) = ty[collection].as_array() {
                let mut codes = BTreeSet::new();
                for field in fields {
                    assert!(
                        codes.insert(field["code"].as_u64().expect(name)),
                        "duplicate in {name}"
                    );
                    if collection == "fields" {
                        assert!(field["presence"].is_string(), "{name}");
                    }
                }
            }
        }
        for dependency in ty["dependencies"].as_array().unwrap() {
            assert!(
                types.contains_key(dependency.as_str().unwrap()),
                "unresolved in {name}: {dependency}"
            );
        }
    }
    for name in ["IfccadPlacement", "IfccadLayer", "IfccadHeader"] {
        assert_eq!(types[name]["fields"][0]["code"], 0);
    }
}

#[test]
fn main_extensions_have_explicit_codes_defaults_and_retired_source_code() {
    let r = registry();
    let t = &r["types"];
    for name in [
        "DimensionPrimaryNumberSettings",
        "DimensionAlternateNumberSettings",
    ] {
        let fields = t[name]["fields"].as_array().unwrap();
        let layout = fields
            .iter()
            .find(|f| f["name"] == "fraction_layout")
            .unwrap();
        let scale = fields
            .iter()
            .find(|f| f["name"] == "fraction_scale")
            .unwrap();
        assert_eq!(layout["default"], "Inline");
        assert_eq!(scale["default"], 1.0);
        assert_eq!(layout["presence"], "default");
    }
    let fields = t["IfccadHatchLoop"]["fields"].as_array().unwrap();
    assert!(!fields.iter().any(|f| f["code"] == 1));
    assert_eq!(
        fields
            .iter()
            .find(|f| f["name"] == "source_entity_ids")
            .unwrap()["code"],
        2
    );
    assert_eq!(
        t["HatchLoopProvenance"]["fields"].as_array().unwrap().len(),
        4
    );
    assert_eq!(
        t["DimensionFractionLayout"]["variants"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}
