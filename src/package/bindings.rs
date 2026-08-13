use super::codes::IFCCAD_PACKAGE_BINDING_INVALID;
use super::discovery::{ResourceDeclaration, ResourceKind};
use super::model::LoadedIfccadPackage;
use super::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
    DIRECTORY_PACKAGE_ENTRYPOINT,
};
use crate::ifcdr::ValidatedIfcdrResource;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub(crate) struct PackageBindings {
    pub(crate) geometry_ifcdr_by_path: BTreeMap<String, Arc<ValidatedIfcdrResource>>,
    pub(crate) preservation_ifcdr_uris_by_path: BTreeMap<String, Vec<String>>,
}

pub(super) struct BindingAnalysis {
    pub(super) bindings: PackageBindings,
    pub(super) diagnostics: Vec<PackageDiagnostic>,
}

pub(super) fn analyze_resource_bindings(
    package: &LoadedIfccadPackage,
    validated_ifcdr_resources: &BTreeMap<String, Arc<ValidatedIfcdrResource>>,
) -> BindingAnalysis {
    let mut result = BindingAnalysis {
        bindings: PackageBindings::default(),
        diagnostics: validate_unique_resource_kinds(&package.declarations),
    };
    let declared_ifcdr = package
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == ResourceKind::Ifcdr)
        .map(|declaration| declaration.uri.as_str())
        .collect::<BTreeSet<_>>();
    let Some(nodes) = package
        .entrypoint
        .value()
        .get("data")
        .and_then(Value::as_array)
    else {
        return result;
    };

    for (node_index, node) in nodes.iter().enumerate() {
        let Some(node_type) = node.get("type").and_then(Value::as_str) else {
            continue;
        };
        let Some(path) = node.get("path").and_then(Value::as_str) else {
            continue;
        };
        match node_type {
            "openaec:DrawingGeometryRepresentation" => {
                let Some(uri) = node
                    .pointer("/attributes/geometry/uri")
                    .and_then(Value::as_str)
                else {
                    continue;
                };
                if let Some(resource) = validated_ifcdr_resources.get(uri) {
                    result
                        .bindings
                        .geometry_ifcdr_by_path
                        .insert(path.to_owned(), resource.clone());
                }
            }
            "openaec:PreservationRepresentation" => {
                let Some(links) = node
                    .pointer("/attributes/preservation/linkedDrawingResourceUris")
                    .and_then(Value::as_array)
                else {
                    continue;
                };
                let mut proven = Vec::new();
                for (link_index, link) in links.iter().enumerate() {
                    let Some(uri) = link.as_str() else { continue };
                    if !declared_ifcdr.contains(uri) {
                        result.diagnostics.push(binding_diagnostic(
                            format!(
                                "/data/{node_index}/attributes/preservation/linkedDrawingResourceUris/{link_index}"
                            ),
                            "preservation link does not identify a declared IFCDR resource",
                            BTreeMap::from([(
                                "resourceUri".to_owned(),
                                PackageDiagnosticContextValue::String(uri.to_owned()),
                            )]),
                        ));
                    } else if validated_ifcdr_resources.contains_key(uri) {
                        proven.push(uri.to_owned());
                    }
                }
                result
                    .bindings
                    .preservation_ifcdr_uris_by_path
                    .insert(path.to_owned(), proven);
            }
            _ => {}
        }
    }
    result
}

fn validate_unique_resource_kinds(declarations: &[ResourceDeclaration]) -> Vec<PackageDiagnostic> {
    let mut first_kinds = BTreeMap::<&str, ResourceKind>::new();
    let mut diagnostics = Vec::new();
    for declaration in declarations {
        match first_kinds.get(declaration.uri.as_str()).copied() {
            None => {
                first_kinds.insert(&declaration.uri, declaration.kind);
            }
            Some(first_kind) if first_kind != declaration.kind => {
                diagnostics.push(binding_diagnostic(
                    declaration.uri_location.clone(),
                    "one resource URI cannot be both IFCDR and IFCPR",
                    BTreeMap::from([
                        (
                            "resourceUri".to_owned(),
                            PackageDiagnosticContextValue::String(declaration.uri.clone()),
                        ),
                        (
                            "firstKind".to_owned(),
                            PackageDiagnosticContextValue::String(kind_name(first_kind).to_owned()),
                        ),
                        (
                            "actualKind".to_owned(),
                            PackageDiagnosticContextValue::String(
                                kind_name(declaration.kind).to_owned(),
                            ),
                        ),
                    ]),
                ));
            }
            Some(_) => {}
        }
    }
    diagnostics
}

fn kind_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Ifcdr => "ifcdr",
        ResourceKind::Ifcpr => "ifcpr",
    }
}

fn binding_diagnostic(
    location: String,
    message: &str,
    context: BTreeMap<String, PackageDiagnosticContextValue>,
) -> PackageDiagnostic {
    PackageDiagnostic {
        code: IFCCAD_PACKAGE_BINDING_INVALID.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_uri: Some(DIRECTORY_PACKAGE_ENTRYPOINT.to_owned()),
        location: Some(location),
        context,
        message: message.to_owned(),
    }
}
