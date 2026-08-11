use super::codes::IFCCAD_PACKAGE_ENTRYPOINT_INVALID;
use super::{PackageDiagnostic, PackageDiagnosticSeverity};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ResourceKind {
    Ifcdr,
    Ifcpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceDeclaration {
    pub(crate) kind: ResourceKind,
    pub(crate) uri: String,
    pub(crate) uri_location: String,
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
        let Some((kind, resource_name)) = recognized_resource(node) else {
            continue;
        };
        let descriptor_pointer = format!("/attributes/{resource_name}");
        let uri_location = format!("/data/{index}{descriptor_pointer}/uri");
        let checksum_location = format!("/data/{index}{descriptor_pointer}/checksum");
        match node.pointer(&format!("/attributes/{resource_name}/uri")) {
            Some(serde_json::Value::String(uri)) => {
                let checksum = node
                    .pointer(&format!("{descriptor_pointer}/checksum"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                discovery.declarations.push(ResourceDeclaration {
                    kind,
                    uri: uri.clone(),
                    uri_location,
                    checksum,
                    checksum_location,
                });
            }
            _ => discovery.diagnostics.push(invalid_entrypoint(
                &uri_location,
                "recognized IFCCAD resource URI must be a string",
            )),
        }
    }

    discovery
}

fn recognized_resource(node: &serde_json::Value) -> Option<(ResourceKind, &'static str)> {
    match node.get("type").and_then(serde_json::Value::as_str) {
        Some("openaec:DrawingGeometryRepresentation") => Some((ResourceKind::Ifcdr, "geometry")),
        Some("openaec:PreservationRepresentation") => Some((ResourceKind::Ifcpr, "preservation")),
        _ => None,
    }
}

fn invalid_entrypoint(location: &str, message: &str) -> PackageDiagnostic {
    PackageDiagnostic {
        code: IFCCAD_PACKAGE_ENTRYPOINT_INVALID.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_uri: None,
        location: Some(location.to_owned()),
        context: BTreeMap::new(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::codes::IFCCAD_PACKAGE_ENTRYPOINT_INVALID;

    #[test]
    fn discovers_drawing_and_preservation_resources() {
        let ifcx = serde_json::json!({
            "data": [
                {
                    "type": "openaec:DrawingGeometryRepresentation",
                    "attributes": {
                        "geometry": { "uri": "drawing.ifcdr.json" }
                    }
                },
                {
                    "type": "openaec:PreservationRepresentation",
                    "attributes": {
                        "preservation": { "uri": "preservation.ifcpr.json" }
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
                    uri: "drawing.ifcdr.json".to_owned(),
                    uri_location: "/data/0/attributes/geometry/uri".to_owned(),
                    checksum: None,
                    checksum_location: "/data/0/attributes/geometry/checksum".to_owned(),
                },
                ResourceDeclaration {
                    kind: ResourceKind::Ifcpr,
                    uri: "preservation.ifcpr.json".to_owned(),
                    uri_location: "/data/1/attributes/preservation/uri".to_owned(),
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
                "attributes": { "geometry": {} }
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
                "type": "openaec:DrawingGeometryRepresentation",
                "attributes": {
                    "geometry": { "url": "drawing.ifcdr.json" }
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
            Some("/data/0/attributes/geometry/uri")
        );
    }

    #[test]
    fn diagnoses_non_string_uri_on_a_recognized_node() {
        let ifcx = serde_json::json!({
            "data": [{
                "type": "openaec:PreservationRepresentation",
                "attributes": {
                    "preservation": { "uri": 42 }
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
}
