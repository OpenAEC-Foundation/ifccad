use super::codes::IFCCAD_PACKAGE_SCHEMA_INVALID;
use super::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
    DIRECTORY_PACKAGE_ENTRYPOINT,
};
use jsonschema::Registry;
use serde_json::Value;
use std::collections::BTreeMap;

const RESOURCE_OVERLAY_ID: &str = "https://schemas.ifccad.org/ifcx/ifccad-overlay-0.1.0.json";
const DRAWING_CORE_ID: &str = "https://schemas.ifccad.org/ifcx/ifccad-drawing-core-0.2.0.json";
const RESOURCE_OVERLAY: &str = include_str!("../../schemas/ifcx/ifccad-overlay-0.1.0.json");
const DRAWING_CORE: &str = include_str!("../../schemas/ifcx/ifccad-drawing-core-0.2.0.json");
const COMPOSITE_OVERLAY: &str = include_str!("../../schemas/ifcx/ifccad-overlay-0.3.0.json");
const IFCPR_SCHEMA: &str = include_str!("../../schemas/ifcpr/schema-0.1.0.json");

pub(crate) fn validate_ifcx(value: &Value) -> Vec<PackageDiagnostic> {
    let resource_overlay = parse_schema(RESOURCE_OVERLAY, "IFCX resource overlay 0.1.0");
    let drawing_core = parse_schema(DRAWING_CORE, "IFCX drawing core 0.2.0");
    let composite = parse_schema(COMPOSITE_OVERLAY, "IFCX overlay 0.3.0");
    let registry = Registry::new()
        .add(RESOURCE_OVERLAY_ID, resource_overlay)
        .expect("register embedded IFCX resource overlay 0.1.0")
        .add(DRAWING_CORE_ID, drawing_core)
        .expect("register embedded IFCX drawing core 0.2.0")
        .prepare()
        .expect("prepare embedded IFCX schema registry");
    let validator = jsonschema::draft202012::options()
        .with_registry(&registry)
        .build(&composite)
        .expect("compile embedded IFCX overlay 0.3.0");
    schema_diagnostics(&validator, DIRECTORY_PACKAGE_ENTRYPOINT, value)
}

pub(crate) fn validate_ifcpr(resource_uri: &str, value: &Value) -> Vec<PackageDiagnostic> {
    let schema = parse_schema(IFCPR_SCHEMA, "IFCPR schema 0.1.0");
    let validator =
        jsonschema::draft202012::new(&schema).expect("compile embedded IFCPR schema 0.1.0");
    schema_diagnostics(&validator, resource_uri, value)
}

fn parse_schema(source: &str, name: &str) -> Value {
    serde_json::from_str(source).unwrap_or_else(|error| panic!("parse embedded {name}: {error}"))
}

fn schema_diagnostics(
    validator: &jsonschema::Validator,
    resource_uri: &str,
    value: &Value,
) -> Vec<PackageDiagnostic> {
    validator
        .iter_errors(value)
        .map(|error| {
            let schema_location = error
                .absolute_keyword_location()
                .map(ToString::to_string)
                .unwrap_or_else(|| error.schema_path().to_string());
            let mut diagnostic_context = BTreeMap::from([
                (
                    "keyword".to_owned(),
                    PackageDiagnosticContextValue::String(error.kind().keyword().to_owned()),
                ),
                (
                    "schemaLocation".to_owned(),
                    PackageDiagnosticContextValue::String(schema_location),
                ),
            ]);
            let property = match error.kind() {
                jsonschema::error::ValidationErrorKind::AdditionalProperties { unexpected }
                    if unexpected.len() == 1 =>
                {
                    unexpected.first().cloned()
                }
                jsonschema::error::ValidationErrorKind::Required { property } => {
                    property.as_str().map(str::to_owned)
                }
                _ => None,
            };
            if let Some(property) = property {
                diagnostic_context.insert(
                    "property".to_owned(),
                    PackageDiagnosticContextValue::String(property),
                );
            }
            PackageDiagnostic {
                code: IFCCAD_PACKAGE_SCHEMA_INVALID.to_owned(),
                severity: PackageDiagnosticSeverity::Error,
                resource_uri: Some(resource_uri.to_owned()),
                location: Some(error.instance_path().to_string()),
                context: diagnostic_context,
                message: format!("schema validation failed: {}", error.masked()),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::bundled_conformance_root;
    use crate::package::codes::IFCCAD_PACKAGE_SCHEMA_INVALID;
    use serde_json::json;
    use std::fs;

    fn valid_ifcx() -> serde_json::Value {
        json!({
            "data": [
                {
                    "path": "drawing-set",
                    "type": "openaec:DrawingSet",
                    "children": {"Drawings": ["drawing"]}
                },
                {
                    "path": "drawing",
                    "type": "openaec:Drawing",
                    "children": {
                        "Representation": "geometry",
                        "Layouts": ["layout"]
                    }
                },
                {
                    "path": "layout",
                    "type": "openaec:DrawingLayout",
                    "attributes": {"name": "Model", "kind": "model", "scopeId": 0},
                    "children": {"Representation": "geometry"}
                },
                {
                    "path": "geometry",
                    "type": "openaec:DrawingGeometryRepresentation",
                    "attributes": {
                        "geometry": {
                            "format": "openaec.ifcdr",
                            "version": "0.5.0",
                            "uri": "drawing.ifcdr.json",
                            "checksum": format!("sha256:{}", "a".repeat(64)),
                            "role": "modelspace"
                        }
                    }
                }
            ]
        })
    }

    #[test]
    fn embedded_ifcx_schema_accepts_the_drawing_spine() {
        assert!(validate_ifcx(&valid_ifcx()).is_empty());
    }

    #[test]
    fn embedded_ifcx_schema_reports_the_instance_location() {
        let mut ifcx = valid_ifcx();
        ifcx["data"][2]["attributes"]["kind"] = json!("sheet");

        let diagnostics = validate_ifcx(&ifcx);

        assert!(diagnostics.iter().any(|item| {
            item.code == IFCCAD_PACKAGE_SCHEMA_INVALID
                && item.location.as_deref() == Some("/data/2/attributes/kind")
                && item.context.get("keyword")
                    == Some(&PackageDiagnosticContextValue::String("enum".to_owned()))
        }));
    }

    #[test]
    fn embedded_ifcx_schema_rejects_an_incomplete_typed_appearance() {
        let mut ifcx = valid_ifcx();
        ifcx["data"].as_array_mut().unwrap().push(json!({
            "path": "appearance-incomplete",
            "type": "openaec:Appearance",
            "attributes": {"name": "Incomplete"}
        }));

        let diagnostics = validate_ifcx(&ifcx);

        assert!(diagnostics.iter().any(|item| {
            item.code == IFCCAD_PACKAGE_SCHEMA_INVALID
                && item.location.as_deref() == Some("/data/4/attributes")
                && item.context.get("property")
                    == Some(&PackageDiagnosticContextValue::String("color".to_owned()))
        }));
    }

    #[test]
    fn embedded_ifcpr_schema_accepts_a_committed_resource() {
        let path = bundled_conformance_root()
            .join("packages")
            .join("valid")
            .join("source-archive")
            .join("preservation.ifcpr.json");
        let value = serde_json::from_slice(&fs::read(path).expect("read IFCPR resource"))
            .expect("parse IFCPR resource");

        assert!(validate_ifcpr("preservation.ifcpr.json", &value).is_empty());
    }

    #[test]
    fn embedded_ifcpr_schema_rejects_a_missing_required_property() {
        let path = bundled_conformance_root()
            .join("packages")
            .join("valid")
            .join("source-archive")
            .join("preservation.ifcpr.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(path).expect("read IFCPR resource"))
                .expect("parse IFCPR resource");
        value
            .as_object_mut()
            .expect("IFCPR object")
            .remove("header");

        let diagnostics = validate_ifcpr("preservation.ifcpr.json", &value);

        assert!(diagnostics.iter().any(|item| {
            item.code == IFCCAD_PACKAGE_SCHEMA_INVALID
                && item.resource_uri.as_deref() == Some("preservation.ifcpr.json")
                && item.context.get("property")
                    == Some(&PackageDiagnosticContextValue::String("header".to_owned()))
        }));
    }
}
