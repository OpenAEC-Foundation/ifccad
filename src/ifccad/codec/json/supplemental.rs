//! Offline local-value supplements. This is not general IFCX graph validation.
use crate::ifccad::IfccadReport;
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::OnceLock};

const ENTRIES: &[(&str, &str, &str)] = &[
    ("ifccad::drawing", "drawingExtras", "IFCCAD-ID-002"),
    ("ifccad::layout", "layoutExtras", "IFCCAD-LAYOUT-002"),
    ("ifccad::layer", "layerExtras", "IFCCAD-APPEARANCE-001"),
    ("ifccad::entity", "entityExtras", "IFCCAD-APPEARANCE-002"),
    ("ifccad::linePattern", "patternExtras", "IFCCAD-PATTERN-002"),
    (
        "ifccad::blockDefinition",
        "blockDefinitionExtras",
        "IFCCAD-BOUNDS-001",
    ),
    (
        "ifccad::blockInstance",
        "blockInstanceExtras",
        "IFCCAD-GEOMETRY-004",
    ),
    (
        "ifccad::geom::placement",
        "placementExtras",
        "IFCCAD-GEOMETRY-001",
    ),
    (
        "ifccad::geom::point",
        "geometryPointExtras",
        "IFCCAD-GEOMETRY-001",
    ),
    (
        "ifccad::geom::lineSegment",
        "geometryLineExtras",
        "IFCCAD-GEOMETRY-001",
    ),
    (
        "ifccad::geom::circle",
        "geometryCircleExtras",
        "IFCCAD-GEOMETRY-002",
    ),
    (
        "ifccad::geom::arc",
        "geometryArcExtras",
        "IFCCAD-GEOMETRY-002",
    ),
    (
        "ifccad::geom::ellipse",
        "geometryEllipseExtras",
        "IFCCAD-GEOMETRY-002",
    ),
    (
        "ifccad::geom::ellipseArc",
        "geometryEllipseArcExtras",
        "IFCCAD-GEOMETRY-002",
    ),
    (
        "ifccad::geom::planarPolyline",
        "geometryPlanarExtras",
        "IFCCAD-GEOMETRY-003",
    ),
    (
        "ifccad::geom::spatialPolyline",
        "geometrySpatialExtras",
        "IFCCAD-GEOMETRY-003",
    ),
    ("ifccad::viewport", "viewportExtras", "IFCCAD-VIEWPORT-002"),
    ("ifccad::textStyle", "textStyle", "IFCCAD-TEXT-001"),
    ("ifccad::text", "text", "IFCCAD-TEXT-002"),
    ("ifccad::mText", "mText", "IFCCAD-TEXT-003"),
    (
        "ifccad::preservation",
        "preservationExtras",
        "IFCCAD-PRESERVATION-001",
    ),
    (
        "ifccad::preservationRecord",
        "preservationRecordExtras",
        "IFCCAD-PRESERVATION-001",
    ),
    (
        "ifccad::opaqueEntity",
        "opaqueEntityExtras",
        "IFCCAD-PRESERVATION-002",
    ),
    (
        "ifccad::drawingWorkspace",
        "drawingWorkspaceExtras",
        "IFCCAD-WORKSPACE-001",
    ),
    (
        "ifccad::modelViewState",
        "modelViewStateExtras",
        "IFCCAD-WORKSPACE-001",
    ),
    (
        "ifccad::ucsDefinition",
        "ucsDefinitionExtras",
        "IFCCAD-WORKSPACE-001",
    ),
    (
        "ifccad::modelWindow",
        "modelWindowExtras",
        "IFCCAD-WORKSPACE-001",
    ),
    (
        "ifccad::paperCanvas",
        "paperCanvasExtras",
        "IFCCAD-WORKSPACE-002",
    ),
    (
        "ifccad::viewportWorkspace",
        "viewportWorkspaceExtras",
        "IFCCAD-WORKSPACE-002",
    ),
];

pub(super) fn rule_id(attribute: &str) -> &'static str {
    ENTRIES
        .iter()
        .find(|entry| entry.0 == attribute)
        .map_or("IFCCAD-WIRE-004", |entry| entry.2)
}

pub(super) fn validate_value(
    attribute: &str,
    value: &Value,
    node_path: &str,
) -> Result<(), IfccadReport> {
    let Some((_, _, rule)) = ENTRIES.iter().find(|e| e.0 == attribute) else {
        return Ok(());
    };
    static VALIDATORS: OnceLock<BTreeMap<&str, jsonschema::Validator>> = OnceLock::new();
    let validators = VALIDATORS.get_or_init(|| {
        let base: Value = serde_json::from_str(include_str!(
            "../../../../schemas/ifccad/supplemental-values-0.1.0.schema.json"
        ))
        .expect("bundled IFCCAD supplements");
        ENTRIES
            .iter()
            .map(|(attribute, entry, _)| {
                let mut schema = base.clone();
                schema["$ref"] = json!(format!("#/$defs/{entry}"));
                (
                    *attribute,
                    jsonschema::draft202012::new(&schema).expect("valid bundled IFCCAD supplement"),
                )
            })
            .collect()
    });
    let errors: Vec<_> = validators[attribute]
        .iter_errors(value)
        .map(|e| format!("{rule} {node_path}/{attribute}{}: {e}", e.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(IfccadReport { errors })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn registered_text_errors_include_rule_attribute_and_path() {
        let path = "/cad/d1/textStyle/0";
        let report = validate_value("ifccad::textStyle", &json!({"font":42}), path).unwrap_err();
        let message = report.to_string();
        assert!(message.contains("IFCCAD-TEXT-001"));
        assert!(message.contains("ifccad::textStyle"));
        assert!(message.contains(path));
        assert!(validate_value(
            "ifccad::textStyle",
            &json!({"name":"Face","font":{"family":"Face"}}),
            path
        )
        .is_ok());
        assert!(validate_value("foreign::note", &json!({"any":null}), "foreign").is_ok());
    }
}
