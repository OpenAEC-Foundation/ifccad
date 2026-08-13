use super::codes::IFCCAD_PACKAGE_BINDING_INVALID;
use super::discovery::{ResourceDeclaration, ResourceKind};
use super::model::LoadedIfccadPackage;
use super::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
    DIRECTORY_PACKAGE_ENTRYPOINT,
};
use crate::ifcdr::{AppearanceId, LayerId, ScopeId, ValidatedIfcdrResource};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub(crate) struct PackageBindings {
    pub(crate) geometry_ifcdr_by_path: BTreeMap<String, Arc<ValidatedIfcdrResource>>,
    pub(crate) preservation_ifcdr_uris_by_path: BTreeMap<String, Vec<String>>,
    pub(crate) layout_by_path: BTreeMap<String, LayoutBinding>,
    pub(crate) ifcx_layer_by_ifcdr_id: BTreeMap<(String, LayerId), String>,
    pub(crate) ifcx_appearance_by_ifcdr_id: BTreeMap<(String, AppearanceId), String>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct LayoutBinding {
    pub(crate) representation_path: String,
    pub(crate) ifcdr_uri: String,
    pub(crate) scope_id: ScopeId,
}

pub(super) struct BindingAnalysis {
    pub(super) bindings: PackageBindings,
    pub(super) diagnostics: Vec<PackageDiagnostic>,
}

pub(super) fn analyze_resource_bindings(
    package: &LoadedIfccadPackage,
    node_indices_by_path: &BTreeMap<String, usize>,
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

    validate_layout_bindings(nodes, &mut result);
    validate_ifcdr_identity_bindings(
        nodes,
        node_indices_by_path,
        validated_ifcdr_resources,
        &mut result,
    );
    result
}

fn validate_layout_bindings(nodes: &[Value], result: &mut BindingAnalysis) {
    for (node_index, node) in nodes.iter().enumerate() {
        if node.get("type").and_then(Value::as_str) != Some("openaec:DrawingLayout") {
            continue;
        }
        let Some(path) = node.get("path").and_then(Value::as_str) else {
            continue;
        };
        let Some(representation_path) = node
            .pointer("/children/Representation")
            .and_then(Value::as_str)
        else {
            continue;
        };
        let Some(scope_id) = node
            .pointer("/attributes/scopeId")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .map(ScopeId::new)
        else {
            continue;
        };
        let Some(resource) = result
            .bindings
            .geometry_ifcdr_by_path
            .get(representation_path)
        else {
            continue;
        };
        if resource.scope(scope_id).is_none() {
            result.diagnostics.push(binding_diagnostic(
                format!("/data/{node_index}/attributes/scopeId"),
                "layout scope does not exist in its IFCDR resource",
                BTreeMap::from([
                    (
                        "representationPath".to_owned(),
                        PackageDiagnosticContextValue::String(representation_path.to_owned()),
                    ),
                    (
                        "scopeId".to_owned(),
                        PackageDiagnosticContextValue::Number(scope_id.get().into()),
                    ),
                ]),
            ));
            continue;
        }
        result.bindings.layout_by_path.insert(
            path.to_owned(),
            LayoutBinding {
                representation_path: representation_path.to_owned(),
                ifcdr_uri: resource.loaded().uri().to_owned(),
                scope_id,
            },
        );
    }
}

fn validate_ifcdr_identity_bindings(
    nodes: &[Value],
    node_indices_by_path: &BTreeMap<String, usize>,
    validated_ifcdr_resources: &BTreeMap<String, Arc<ValidatedIfcdrResource>>,
    result: &mut BindingAnalysis,
) {
    for (uri, resource) in validated_ifcdr_resources {
        for (row_index, binding) in resource.bindings().layers().enumerate() {
            let Some(path) = binding.ifcx_layer() else {
                continue;
            };
            if validate_ifcx_identity(
                nodes,
                node_indices_by_path,
                uri,
                format!("/layerBindings/{row_index}/ifcxLayer"),
                path,
                "openaec:Layer",
                &mut result.diagnostics,
            ) {
                result
                    .bindings
                    .ifcx_layer_by_ifcdr_id
                    .insert((uri.clone(), binding.id()), path.to_owned());
            }
        }
        for (row_index, binding) in resource.bindings().appearances().enumerate() {
            let Some(path) = binding.ifcx_appearance() else {
                continue;
            };
            if validate_ifcx_identity(
                nodes,
                node_indices_by_path,
                uri,
                format!("/appearanceBindings/{row_index}/ifcxAppearance"),
                path,
                "openaec:Appearance",
                &mut result.diagnostics,
            ) {
                result
                    .bindings
                    .ifcx_appearance_by_ifcdr_id
                    .insert((uri.clone(), binding.id()), path.to_owned());
            }
        }
    }
}

fn validate_ifcx_identity(
    nodes: &[Value],
    node_indices_by_path: &BTreeMap<String, usize>,
    resource_uri: &str,
    location: String,
    target_path: &str,
    expected_type: &str,
    diagnostics: &mut Vec<PackageDiagnostic>,
) -> bool {
    let actual_type = node_indices_by_path
        .get(target_path)
        .and_then(|index| nodes.get(*index))
        .and_then(|node| node.get("type"))
        .and_then(Value::as_str);
    if actual_type == Some(expected_type) {
        return true;
    }
    diagnostics.push(PackageDiagnostic {
        code: IFCCAD_PACKAGE_BINDING_INVALID.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_uri: Some(resource_uri.to_owned()),
        location: Some(location),
        context: BTreeMap::from([
            (
                "actualType".to_owned(),
                actual_type.map_or(PackageDiagnosticContextValue::Null, |value| {
                    PackageDiagnosticContextValue::String(value.to_owned())
                }),
            ),
            (
                "expectedType".to_owned(),
                PackageDiagnosticContextValue::String(expected_type.to_owned()),
            ),
            (
                "targetPath".to_owned(),
                PackageDiagnosticContextValue::String(target_path.to_owned()),
            ),
        ]),
        message: "IFCDR identity binding does not identify the expected IFCX node type".to_owned(),
    });
    false
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
