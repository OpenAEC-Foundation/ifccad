use super::codes::IFCCAD_PACKAGE_ENTRYPOINT_INVALID;
use super::source::ResourceSourceKey;
use super::{PackageDiagnostic, PackageDiagnosticSeverity};
use crate::ResourceId;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ResourceKind {
    Ifcdr,
    Ifcpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceDeclaration {
    pub(crate) kind: ResourceKind,
    pub(crate) resource_id: ResourceId,
    pub(crate) resource_id_location: String,
    pub(crate) source: ResourceSourceKey,
    pub(crate) source_location: String,
    pub(crate) checksum: Option<String>,
    pub(crate) checksum_location: String,
}

pub(crate) struct ResourceDiscovery {
    pub(crate) declarations: Vec<ResourceDeclaration>,
    pub(crate) diagnostics: Vec<PackageDiagnostic>,
}

pub(crate) fn discover_resources(ifcx: &serde_json::Value) -> ResourceDiscovery {
    let mut discovery = ResourceDiscovery {
        declarations: Vec::new(),
        diagnostics: Vec::new(),
    };
    let Some(data) = ifcx.get("data").and_then(serde_json::Value::as_array) else {
        discovery
            .diagnostics
            .push(invalid_entrypoint("/data", "IFCX data must be an array"));
        return discovery;
    };

    for (index, node) in data.iter().enumerate() {
        if node.get("type").and_then(serde_json::Value::as_str)
            == Some("openaec:DrawingGeometryRepresentation")
        {
            discovery.diagnostics.push(PackageDiagnostic {
                category: crate::diagnostic::PackageDiagnosticCategory::UnsupportedContent,
                code: super::codes::IFCCAD_PACKAGE_VOCABULARY_UNSUPPORTED.into(),
                severity: PackageDiagnosticSeverity::Error,
                resource_id: None,
                resource_uri: Some(super::DIRECTORY_PACKAGE_ENTRYPOINT.into()),
                location: Some(format!("/data/{index}/type")),
                context: BTreeMap::from([(
                    "replacementType".into(),
                    super::PackageDiagnosticContextValue::String("openaec:DrawingRepresentation".into()),
                )]),
                message: "retired DrawingGeometryRepresentation vocabulary is unsupported; use DrawingRepresentation with attributes.resource".into(),
            });
            continue;
        }
        let Some((kind, resource_name)) = recognized_resource(node) else {
            continue;
        };
        let descriptor_pointer = format!("/attributes/{resource_name}");
        let resource_id_location = format!("/data/{index}{descriptor_pointer}/resourceId");
        let uri_location = format!("/data/{index}{descriptor_pointer}/uri");
        let checksum_location = format!("/data/{index}{descriptor_pointer}/checksum");
        let resource_id = match node.pointer(&format!("/attributes/{resource_name}/resourceId")) {
            Some(serde_json::Value::String(value)) => match ResourceId::new(value.clone()) {
                Ok(resource_id) => resource_id,
                Err(_) => continue,
            },
            _ => continue,
        };
        let descriptor = &node["attributes"][resource_name];
        let source = match (descriptor.get("uri"), descriptor.get("content")) {
            (Some(serde_json::Value::String(uri)), None) => {
                ResourceSourceKey::External(uri.clone())
            }
            (None, Some(value)) if value.is_object() && descriptor.get("checksum").is_none() => {
                ResourceSourceKey::Inline {
                    document_uri: super::DIRECTORY_PACKAGE_ENTRYPOINT.into(),
                    pointer: format!("/data/{index}{descriptor_pointer}/content"),
                }
            }
            (None, None) | (Some(_), None) => {
                discovery.diagnostics.push(invalid_entrypoint(
                    &uri_location,
                    "recognized IFCCAD resource requires a source",
                ));
                continue;
            }
            _ => continue,
        };
        let source_location = match &source {
            ResourceSourceKey::External(_) => uri_location,
            ResourceSourceKey::Inline { pointer, .. } => pointer.clone(),
        };
        discovery.declarations.push(ResourceDeclaration {
            kind,
            resource_id,
            resource_id_location,
            source,
            source_location,
            checksum: descriptor
                .get("checksum")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            checksum_location,
        });
    }

    discovery
}

fn recognized_resource(node: &serde_json::Value) -> Option<(ResourceKind, &'static str)> {
    match node.get("type").and_then(serde_json::Value::as_str) {
        Some("openaec:DrawingRepresentation") => Some((ResourceKind::Ifcdr, "resource")),
        Some("openaec:PreservationRepresentation") => Some((ResourceKind::Ifcpr, "preservation")),
        _ => None,
    }
}

fn invalid_entrypoint(location: &str, message: &str) -> PackageDiagnostic {
    PackageDiagnostic {
        category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
        code: IFCCAD_PACKAGE_ENTRYPOINT_INVALID.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: None,
        resource_uri: None,
        location: Some(location.to_owned()),
        context: BTreeMap::new(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::codes::IFCCAD_PACKAGE_ENTRYPOINT_INVALID;
    use super::*;

    #[test]
    fn retired_vocabulary_is_explicitly_unsupported() {
        let result = discover_resources(&serde_json::json!({"data":[{
            "path":"retired", "type":"openaec:DrawingGeometryRepresentation"
        }]}));
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            "IFCCAD_PACKAGE_VOCABULARY_UNSUPPORTED"
        );
        assert_eq!(
            result.diagnostics[0].location.as_deref(),
            Some("/data/0/type")
        );
        assert!(
            discover_resources(&serde_json::json!({"data":[{"type":"example:Extension"}]}))
                .diagnostics
                .is_empty()
        );
    }

    #[test]
    fn discovers_drawing_and_preservation_resources() {
        let ifcx = serde_json::json!({
            "data": [
                {
                    "type": "openaec:DrawingRepresentation",
                    "attributes": {
                        "resource": {
                            "resourceId": "geometry-main",
                            "uri": "drawing.ifcdr.json"
                        }
                    }
                },
                {
                    "type": "openaec:PreservationRepresentation",
                    "attributes": {
                        "preservation": {
                            "resourceId": "preservation-source",
                            "uri": "preservation.ifcpr.json"
                        }
                    }
                }
            ]
        });

        let discovery = discover_resources(&ifcx);

        assert!(discovery.diagnostics.is_empty());
        assert_eq!(
            discovery.declarations,
            vec![
                ResourceDeclaration {
                    kind: ResourceKind::Ifcdr,
                    resource_id: crate::ResourceId::new("geometry-main").unwrap(),
                    resource_id_location: "/data/0/attributes/resource/resourceId".to_owned(),
                    source: ResourceSourceKey::External("drawing.ifcdr.json".to_owned()),
                    source_location: "/data/0/attributes/resource/uri".to_owned(),
                    checksum: None,
                    checksum_location: "/data/0/attributes/resource/checksum".to_owned(),
                },
                ResourceDeclaration {
                    kind: ResourceKind::Ifcpr,
                    resource_id: crate::ResourceId::new("preservation-source").unwrap(),
                    resource_id_location: "/data/1/attributes/preservation/resourceId".to_owned(),
                    source: ResourceSourceKey::External("preservation.ifcpr.json".to_owned()),
                    source_location: "/data/1/attributes/preservation/uri".to_owned(),
                    checksum: None,
                    checksum_location: "/data/1/attributes/preservation/checksum".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn allows_unrelated_ifcx_node_types() {
        let ifcx = serde_json::json!({
            "data": [{
                "type": "example:UnrelatedNode",
                "attributes": { "resource": {} }
            }]
        });

        let discovery = discover_resources(&ifcx);

        assert!(discovery.declarations.is_empty());
        assert!(discovery.diagnostics.is_empty());
    }

    #[test]
    fn diagnoses_missing_uri_on_a_recognized_node() {
        let ifcx = serde_json::json!({
            "data": [{
                "type": "openaec:DrawingRepresentation",
                "attributes": {
                    "resource": {
                        "resourceId": "geometry-main",
                        "url": "drawing.ifcdr.json"
                    }
                }
            }]
        });

        let discovery = discover_resources(&ifcx);

        assert!(discovery.declarations.is_empty());
        assert_eq!(discovery.diagnostics.len(), 1);
        let diagnostic = &discovery.diagnostics[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_ENTRYPOINT_INVALID);
        assert_eq!(diagnostic.resource_uri, None);
        assert_eq!(
            diagnostic.location.as_deref(),
            Some("/data/0/attributes/resource/uri")
        );
    }

    #[test]
    fn diagnoses_non_string_uri_on_a_recognized_node() {
        let ifcx = serde_json::json!({
            "data": [{
                "type": "openaec:PreservationRepresentation",
                "attributes": {
                    "preservation": {
                        "resourceId": "preservation-source",
                        "uri": 42
                    }
                }
            }]
        });

        let discovery = discover_resources(&ifcx);

        assert!(discovery.declarations.is_empty());
        assert_eq!(discovery.diagnostics.len(), 1);
        let diagnostic = &discovery.diagnostics[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_ENTRYPOINT_INVALID);
        assert_eq!(diagnostic.resource_uri, None);
        assert_eq!(
            diagnostic.location.as_deref(),
            Some("/data/0/attributes/preservation/uri")
        );
    }

    #[test]
    fn diagnoses_missing_data_array() {
        let discovery = discover_resources(&serde_json::json!({}));

        assert!(discovery.declarations.is_empty());
        assert_eq!(discovery.diagnostics.len(), 1);
        let diagnostic = &discovery.diagnostics[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_ENTRYPOINT_INVALID);
        assert_eq!(diagnostic.resource_uri, None);
        assert_eq!(diagnostic.location.as_deref(), Some("/data"));
    }

    #[test]
    fn does_not_infer_missing_resource_id_from_uri() {
        let ifcx = serde_json::json!({
            "data": [{
                "type": "openaec:DrawingRepresentation",
                "attributes": {
                    "resource": { "uri": "drawing.ifcdr.json" }
                }
            }]
        });

        let discovery = discover_resources(&ifcx);

        assert!(discovery.declarations.is_empty());
        assert!(discovery.diagnostics.is_empty());
    }
}
