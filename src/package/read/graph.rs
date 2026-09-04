use super::codes::{
    IFCCAD_PACKAGE_NODE_PATH_DUPLICATE, IFCCAD_PACKAGE_NODE_REFERENCE_MISSING,
    IFCCAD_PACKAGE_NODE_REFERENCE_TYPE_MISMATCH,
};
use super::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
    DIRECTORY_PACKAGE_ENTRYPOINT,
};
use serde_json::Value;
use std::collections::BTreeMap;

pub(crate) struct IfcxGraphValidation {
    pub(crate) node_indices_by_path: BTreeMap<String, usize>,
    pub(crate) diagnostics: Vec<PackageDiagnostic>,
}

pub(crate) fn validate_ifcx_graph(ifcx: &Value) -> IfcxGraphValidation {
    let mut result = IfcxGraphValidation {
        node_indices_by_path: BTreeMap::new(),
        diagnostics: Vec::new(),
    };
    let Some(data) = ifcx.get("data").and_then(Value::as_array) else {
        return result;
    };

    for (index, node) in data.iter().enumerate() {
        let Some(path) = node.get("path").and_then(Value::as_str) else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        if let Some(first_index) = result.node_indices_by_path.get(path).copied() {
            result
                .diagnostics
                .push(duplicate_path_diagnostic(path, first_index, index));
        } else {
            result.node_indices_by_path.insert(path.to_owned(), index);
        }
    }

    validate_supported_references(data, &result.node_indices_by_path, &mut result.diagnostics);
    result
}

fn duplicate_path_diagnostic(path: &str, first_index: usize, index: usize) -> PackageDiagnostic {
    PackageDiagnostic {
        code: IFCCAD_PACKAGE_NODE_PATH_DUPLICATE.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: None,
        resource_uri: Some(DIRECTORY_PACKAGE_ENTRYPOINT.to_owned()),
        location: Some(format!("/data/{index}/path")),
        context: BTreeMap::from([
            (
                "firstLocation".to_owned(),
                PackageDiagnosticContextValue::String(format!("/data/{first_index}/path")),
            ),
            (
                "nodePath".to_owned(),
                PackageDiagnosticContextValue::String(path.to_owned()),
            ),
        ]),
        message: format!("IFCX node path {path:?} is duplicated"),
    }
}

fn validate_supported_references(
    data: &[Value],
    index: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    for (node_index, node) in data.iter().enumerate() {
        let Some(node_type) = node.get("type").and_then(Value::as_str) else {
            continue;
        };
        match node_type {
            "openaec:DrawingSet" => {
                check_array_refs(
                    data,
                    index,
                    diagnostics,
                    node,
                    node_index,
                    "/children/Drawings",
                    "openaec:Drawing",
                );
                check_array_refs(
                    data,
                    index,
                    diagnostics,
                    node,
                    node_index,
                    "/children/PreservationResources",
                    "openaec:PreservationRepresentation",
                );
            }
            "openaec:Drawing" => {
                check_array_refs(
                    data,
                    index,
                    diagnostics,
                    node,
                    node_index,
                    "/children/Layouts",
                    "openaec:DrawingLayout",
                );
                check_single_ref(
                    data,
                    index,
                    diagnostics,
                    node,
                    node_index,
                    "/children/Representation",
                    "openaec:DrawingGeometryRepresentation",
                );
            }
            "openaec:DrawingLayout" => check_single_ref(
                data,
                index,
                diagnostics,
                node,
                node_index,
                "/children/Representation",
                "openaec:DrawingGeometryRepresentation",
            ),
            "openaec:Layer" => check_single_ref(
                data,
                index,
                diagnostics,
                node,
                node_index,
                "/attributes/appearance",
                "openaec:Appearance",
            ),
            _ => {}
        }
    }
}

fn check_array_refs(
    data: &[Value],
    index: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<PackageDiagnostic>,
    node: &Value,
    node_index: usize,
    pointer: &str,
    expected_type: &str,
) {
    let Some(references) = node.pointer(pointer).and_then(Value::as_array) else {
        return;
    };
    for (reference_index, reference) in references.iter().enumerate() {
        let Some(target_path) = reference.as_str() else {
            continue;
        };
        check_target(
            data,
            index,
            diagnostics,
            target_path,
            expected_type,
            format!("/data/{node_index}{pointer}/{reference_index}"),
        );
    }
}

fn check_single_ref(
    data: &[Value],
    index: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<PackageDiagnostic>,
    node: &Value,
    node_index: usize,
    pointer: &str,
    expected_type: &str,
) {
    let Some(target_path) = node.pointer(pointer).and_then(Value::as_str) else {
        return;
    };
    check_target(
        data,
        index,
        diagnostics,
        target_path,
        expected_type,
        format!("/data/{node_index}{pointer}"),
    );
}

fn check_target(
    data: &[Value],
    index: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<PackageDiagnostic>,
    target_path: &str,
    expected_type: &str,
    location: String,
) {
    let Some(target_index) = index.get(target_path).copied() else {
        diagnostics.push(reference_diagnostic(
            location,
            target_path,
            expected_type,
            ReferenceFailure::Missing,
        ));
        return;
    };
    let actual_type = data[target_index].get("type").and_then(Value::as_str);
    if actual_type != Some(expected_type) {
        diagnostics.push(reference_diagnostic(
            location,
            target_path,
            expected_type,
            ReferenceFailure::TypeMismatch(actual_type),
        ));
    }
}

enum ReferenceFailure<'a> {
    Missing,
    TypeMismatch(Option<&'a str>),
}

fn reference_diagnostic(
    location: String,
    target_path: &str,
    expected_type: &str,
    failure: ReferenceFailure<'_>,
) -> PackageDiagnostic {
    let mut context = BTreeMap::from([
        (
            "expectedType".to_owned(),
            PackageDiagnosticContextValue::String(expected_type.to_owned()),
        ),
        (
            "targetPath".to_owned(),
            PackageDiagnosticContextValue::String(target_path.to_owned()),
        ),
    ]);
    let (code, message) = match failure {
        ReferenceFailure::Missing => (
            IFCCAD_PACKAGE_NODE_REFERENCE_MISSING,
            format!("IFCX reference target {target_path:?} is missing; expected {expected_type:?}"),
        ),
        ReferenceFailure::TypeMismatch(actual_type) => {
            context.insert(
                "actualType".to_owned(),
                actual_type.map_or(PackageDiagnosticContextValue::Null, |value| {
                    PackageDiagnosticContextValue::String(value.to_owned())
                }),
            );
            let message = actual_type.map_or_else(
                || {
                    format!(
                        "IFCX reference {target_path:?} targets a node that has no string type; expected {expected_type:?}"
                    )
                },
                |actual_type| {
                    format!(
                        "IFCX reference {target_path:?} targets {actual_type:?}, expected {expected_type:?}"
                    )
                },
            );
            (IFCCAD_PACKAGE_NODE_REFERENCE_TYPE_MISMATCH, message)
        }
    };
    PackageDiagnostic {
        code: code.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: None,
        resource_uri: Some(DIRECTORY_PACKAGE_ENTRYPOINT.to_owned()),
        location: Some(location),
        context,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::super::codes::{
        IFCCAD_PACKAGE_NODE_PATH_DUPLICATE, IFCCAD_PACKAGE_NODE_REFERENCE_MISSING,
        IFCCAD_PACKAGE_NODE_REFERENCE_TYPE_MISMATCH,
    };
    use super::*;
    use crate::package::PackageDiagnosticContextValue;
    use serde_json::json;

    fn valid_ifcx_graph() -> serde_json::Value {
        json!({
            "data": [
                {
                    "path": "drawing-set",
                    "type": "openaec:DrawingSet",
                    "children": {
                        "Drawings": ["drawing"],
                        "PreservationResources": ["preservation"]
                    }
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
                    "children": {"Representation": "geometry"}
                },
                {
                    "path": "geometry",
                    "type": "openaec:DrawingGeometryRepresentation"
                },
                {
                    "path": "layer",
                    "type": "openaec:Layer",
                    "attributes": {"appearance": "appearance"}
                },
                {
                    "path": "appearance",
                    "type": "openaec:Appearance"
                },
                {
                    "path": "preservation",
                    "type": "openaec:PreservationRepresentation"
                },
                {
                    "path": "extension",
                    "type": "example:UnknownNode",
                    "children": {"Anything": "missing-but-uninterpreted"}
                }
            ]
        })
    }

    #[test]
    fn first_node_path_remains_indexed_when_a_duplicate_is_reported() {
        let ifcx = json!({
            "data": [
                {"path": "drawing-main", "type": "example:First"},
                {"path": "drawing-main", "type": "example:Second"}
            ]
        });

        let result = validate_ifcx_graph(&ifcx);

        assert_eq!(result.node_indices_by_path["drawing-main"], 0);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            IFCCAD_PACKAGE_NODE_PATH_DUPLICATE
        );
        assert_eq!(
            result.diagnostics[0].location.as_deref(),
            Some("/data/1/path")
        );
        assert_eq!(
            result.diagnostics[0].context.get("firstLocation"),
            Some(&PackageDiagnosticContextValue::String(
                "/data/0/path".to_owned()
            ))
        );
    }

    #[test]
    fn valid_supported_references_and_unknown_extensions_are_accepted() {
        let result = validate_ifcx_graph(&valid_ifcx_graph());

        assert!(result.diagnostics.is_empty());
        assert_eq!(result.node_indices_by_path.len(), 8);
    }

    #[test]
    fn supported_references_report_missing_targets_at_the_referring_location() {
        for (pointer, expected_type) in [
            ("/data/0/children/Drawings/0", "openaec:Drawing"),
            (
                "/data/0/children/PreservationResources/0",
                "openaec:PreservationRepresentation",
            ),
            ("/data/1/children/Layouts/0", "openaec:DrawingLayout"),
            (
                "/data/1/children/Representation",
                "openaec:DrawingGeometryRepresentation",
            ),
            (
                "/data/2/children/Representation",
                "openaec:DrawingGeometryRepresentation",
            ),
            ("/data/4/attributes/appearance", "openaec:Appearance"),
        ] {
            let mut ifcx = valid_ifcx_graph();
            *ifcx.pointer_mut(pointer).expect("reference pointer") = json!("missing");

            let result = validate_ifcx_graph(&ifcx);
            let diagnostic = result
                .diagnostics
                .iter()
                .find(|item| item.location.as_deref() == Some(pointer))
                .unwrap_or_else(|| panic!("missing diagnostic at {pointer}"));

            assert_eq!(diagnostic.code, IFCCAD_PACKAGE_NODE_REFERENCE_MISSING);
            assert_eq!(
                diagnostic.context.get("targetPath"),
                Some(&PackageDiagnosticContextValue::String("missing".to_owned()))
            );
            assert_eq!(
                diagnostic.context.get("expectedType"),
                Some(&PackageDiagnosticContextValue::String(
                    expected_type.to_owned()
                ))
            );
        }
    }

    #[test]
    fn supported_references_report_wrong_target_types() {
        for (pointer, wrong_target, expected_type, actual_type) in [
            (
                "/data/0/children/Drawings/0",
                "appearance",
                "openaec:Drawing",
                "openaec:Appearance",
            ),
            (
                "/data/0/children/PreservationResources/0",
                "appearance",
                "openaec:PreservationRepresentation",
                "openaec:Appearance",
            ),
            (
                "/data/1/children/Layouts/0",
                "appearance",
                "openaec:DrawingLayout",
                "openaec:Appearance",
            ),
            (
                "/data/1/children/Representation",
                "appearance",
                "openaec:DrawingGeometryRepresentation",
                "openaec:Appearance",
            ),
            (
                "/data/2/children/Representation",
                "appearance",
                "openaec:DrawingGeometryRepresentation",
                "openaec:Appearance",
            ),
            (
                "/data/4/attributes/appearance",
                "drawing",
                "openaec:Appearance",
                "openaec:Drawing",
            ),
        ] {
            let mut ifcx = valid_ifcx_graph();
            *ifcx.pointer_mut(pointer).expect("reference pointer") = json!(wrong_target);

            let result = validate_ifcx_graph(&ifcx);
            let diagnostic = result
                .diagnostics
                .iter()
                .find(|item| item.location.as_deref() == Some(pointer))
                .unwrap_or_else(|| panic!("missing diagnostic at {pointer}"));

            assert_eq!(diagnostic.code, IFCCAD_PACKAGE_NODE_REFERENCE_TYPE_MISMATCH);
            assert_eq!(
                diagnostic.context.get("targetPath"),
                Some(&PackageDiagnosticContextValue::String(
                    wrong_target.to_owned()
                ))
            );
            assert_eq!(
                diagnostic.context.get("expectedType"),
                Some(&PackageDiagnosticContextValue::String(
                    expected_type.to_owned()
                ))
            );
            assert_eq!(
                diagnostic.context.get("actualType"),
                Some(&PackageDiagnosticContextValue::String(
                    actual_type.to_owned()
                ))
            );
        }
    }

    #[test]
    fn existing_target_without_a_string_type_is_not_reported_as_missing() {
        let mut ifcx = valid_ifcx_graph();
        ifcx["data"][3]
            .as_object_mut()
            .expect("geometry node")
            .remove("type");

        let result = validate_ifcx_graph(&ifcx);
        let diagnostic = result
            .diagnostics
            .iter()
            .find(|item| item.location.as_deref() == Some("/data/1/children/Representation"))
            .expect("type mismatch diagnostic");

        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_NODE_REFERENCE_TYPE_MISMATCH);
        assert_eq!(
            diagnostic.context.get("actualType"),
            Some(&PackageDiagnosticContextValue::Null)
        );
        assert!(diagnostic.message.contains("has no string type"));
    }
}
