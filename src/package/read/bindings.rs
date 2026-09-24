use super::codes::{IFCCAD_PACKAGE_BINDING_INVALID, IFCCAD_PACKAGE_TARGET_RESOURCE_MISSING};
use super::discovery::{ResourceDeclaration, ResourceKind};
use super::model::LoadedIfccadPackage;
use super::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
    DIRECTORY_PACKAGE_ENTRYPOINT,
};
use crate::ifcdr::{AppearanceId, LayerId, ScopeId, ValidatedIfcdrResource};
use crate::ResourceId;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub(crate) struct PackageBindings {
    pub(crate) drawing_ifcdr_by_path: BTreeMap<String, Arc<ValidatedIfcdrResource>>,
    pub(crate) preservation_ifcdr_resource_ids_by_path: BTreeMap<String, Vec<ResourceId>>,
    pub(crate) layout_by_path: BTreeMap<String, LayoutBinding>,
    pub(crate) ifcx_layer_by_ifcdr_id: BTreeMap<(ResourceId, LayerId), String>,
    pub(crate) ifcx_appearance_by_ifcdr_id: BTreeMap<(ResourceId, AppearanceId), String>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct LayoutBinding {
    pub(crate) representation_path: String,
    pub(crate) ifcdr_resource_id: ResourceId,
    pub(crate) scope_id: ScopeId,
}

pub(super) struct BindingAnalysis {
    pub(super) bindings: PackageBindings,
    pub(super) diagnostics: Vec<PackageDiagnostic>,
}

pub(super) fn analyze_resource_bindings(
    package: &LoadedIfccadPackage,
    node_indices_by_path: &BTreeMap<String, usize>,
    validated_ifcdr_resources: &BTreeMap<ResourceId, Arc<ValidatedIfcdrResource>>,
    unavailable_ifcdr_resource_ids: &BTreeSet<ResourceId>,
) -> BindingAnalysis {
    let mut result = BindingAnalysis {
        bindings: PackageBindings::default(),
        diagnostics: validate_unique_resource_kinds(&package.declarations),
    };
    let proven_ifcdr = validated_ifcdr_resources.keys().collect::<BTreeSet<_>>();
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
            "openaec:DrawingRepresentation" => {
                let Some(resource_id) = node
                    .pointer("/attributes/resource/resourceId")
                    .and_then(Value::as_str)
                    .and_then(|value| ResourceId::new(value).ok())
                else {
                    continue;
                };
                if let Some(resource) = validated_ifcdr_resources.get(&resource_id) {
                    result
                        .bindings
                        .drawing_ifcdr_by_path
                        .insert(path.to_owned(), resource.clone());
                }
            }
            "openaec:PreservationRepresentation" => {
                let Some(links) = node
                    .pointer("/attributes/preservation/linkedDrawingResourceIds")
                    .and_then(Value::as_array)
                else {
                    continue;
                };
                let mut proven = Vec::new();
                for (link_index, link) in links.iter().enumerate() {
                    let Some(resource_id) =
                        link.as_str().and_then(|value| ResourceId::new(value).ok())
                    else {
                        continue;
                    };
                    if !proven_ifcdr.contains(&resource_id) {
                        if unavailable_ifcdr_resource_ids.contains(&resource_id) {
                            continue;
                        }
                        result.diagnostics.push(target_resource_diagnostic(
                            format!(
                                "/data/{node_index}/attributes/preservation/linkedDrawingResourceIds/{link_index}"
                            ),
                            &resource_id,
                        ));
                    } else {
                        proven.push(resource_id);
                    }
                }
                result
                    .bindings
                    .preservation_ifcdr_resource_ids_by_path
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
    validate_drawing_membership(nodes, node_indices_by_path, &mut result);
    result
        .diagnostics
        .extend(super::appearance::validate_appearance_and_layer_semantics(
            nodes,
            node_indices_by_path,
            validated_ifcdr_resources,
            &result.bindings,
        ));
    result
}

fn validate_drawing_membership(
    nodes: &[Value],
    node_indices_by_path: &BTreeMap<String, usize>,
    result: &mut BindingAnalysis,
) {
    for (drawing_index, drawing) in nodes.iter().enumerate() {
        if drawing["type"] != "openaec:Drawing" {
            continue;
        }
        let Some(representation_path) = drawing
            .pointer("/children/Representation")
            .and_then(Value::as_str)
        else {
            continue;
        };
        let Some(&representation_index) = node_indices_by_path.get(representation_path) else {
            continue;
        };
        if nodes[representation_index]
            .pointer("/attributes/resource/version")
            .and_then(Value::as_str)
            .is_none_or(|version| !matches!(version, "0.10.0" | "0.11.0"))
        {
            continue;
        }
        let Some(resource) = result
            .bindings
            .drawing_ifcdr_by_path
            .get(representation_path)
        else {
            continue;
        };
        let layers = drawing
            .pointer("/children/Layers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        let appearances = drawing
            .pointer("/children/Appearances")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        let mut seen_layer_names = BTreeSet::new();
        for (position, path) in drawing
            .pointer("/children/Layers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let Some(path) = path.as_str() else { continue };
            let Some(&index) = node_indices_by_path.get(path) else {
                continue;
            };
            let Some(name) = nodes[index]
                .pointer("/attributes/name")
                .and_then(Value::as_str)
            else {
                continue;
            };
            if !seen_layer_names.insert(crate::ifcdr::names::name_key(name)) {
                result.diagnostics.push(binding_diagnostic(
                    format!("/data/{drawing_index}/children/Layers/{position}"),
                    "layer names must be unique within one Drawing",
                    BTreeMap::new(),
                ));
            }
            if let Some(appearance) = nodes[index]
                .pointer("/attributes/appearance")
                .and_then(Value::as_str)
            {
                if !appearances.contains(appearance) {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{drawing_index}/children/Appearances"),
                        "Drawing Appearances must include each listed Layer's default appearance",
                        BTreeMap::new(),
                    ));
                }
            }
        }
        let mut seen_appearance_names = BTreeSet::new();
        for (position, path) in drawing
            .pointer("/children/Appearances")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let Some(path) = path.as_str() else { continue };
            let Some(&index) = node_indices_by_path.get(path) else {
                continue;
            };
            let Some(name) = nodes[index]
                .pointer("/attributes/name")
                .and_then(Value::as_str)
            else {
                continue;
            };
            if !seen_appearance_names.insert(crate::ifcdr::names::name_key(name)) {
                result.diagnostics.push(binding_diagnostic(
                    format!("/data/{drawing_index}/children/Appearances/{position}"),
                    "appearance names must be unique within one Drawing",
                    BTreeMap::new(),
                ));
            }
        }
        let mut seen_layout_names = BTreeSet::new();
        let mut selected_scopes = BTreeSet::new();
        let mut model_count = 0;
        for (position, path) in drawing
            .pointer("/children/Layouts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let Some(path) = path.as_str() else { continue };
            let Some(&index) = node_indices_by_path.get(path) else {
                continue;
            };
            let layout = &nodes[index];
            if let Some(name) = layout.pointer("/attributes/name").and_then(Value::as_str) {
                if !seen_layout_names.insert(crate::ifcdr::names::name_key(name)) {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{drawing_index}/children/Layouts/{position}"),
                        "layout names must be unique within one Drawing",
                        BTreeMap::new(),
                    ));
                }
            }
            let kind = layout.pointer("/attributes/kind").and_then(Value::as_str);
            if kind == Some("model") {
                model_count += 1;
                if position != 0 {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{drawing_index}/children/Layouts/{position}"),
                        "model layout must be first",
                        BTreeMap::new(),
                    ));
                }
            }
            if let Some(binding) = result.bindings.layout_by_path.get(path) {
                if binding.representation_path == representation_path
                    && !selected_scopes.insert(binding.scope_id.get())
                {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{drawing_index}/children/Layouts/{position}"),
                        "each scope must have exactly one Drawing layout",
                        BTreeMap::new(),
                    ));
                }
            }
            if let Some(settings) = layout.pointer("/attributes/plotSettings") {
                let area = settings.pointer("/area/mode").and_then(Value::as_str);
                let invalid = match area {
                    Some("Limits") => {
                        kind != Some("model") || layout.pointer("/attributes/limits").is_none()
                    }
                    Some("Layout") => {
                        kind != Some("paper")
                            || settings
                                .pointer("/mapping/scale/mode")
                                .and_then(Value::as_str)
                                != Some("Fixed")
                            || settings
                                .pointer("/mapping/placement/mode")
                                .and_then(Value::as_str)
                                != Some("Offset")
                    }
                    _ => false,
                };
                if invalid {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{index}/attributes/plotSettings/area"),
                        "plot area is incompatible with layout kind or authored mapping",
                        BTreeMap::new(),
                    ));
                }
                let media = &settings["media"];
                let printable = &media["printableArea"];
                let width = media["width"].as_f64();
                let height = media["height"].as_f64();
                if !valid_rect(printable)
                    || width.zip(height).is_none_or(|(w, h)| {
                        printable["minX"].as_f64().unwrap_or(f64::INFINITY) < 0.0
                            || printable["minY"].as_f64().unwrap_or(f64::INFINITY) < 0.0
                            || printable["maxX"].as_f64().unwrap_or(f64::INFINITY) > w
                            || printable["maxY"].as_f64().unwrap_or(f64::INFINITY) > h
                    })
                {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{index}/attributes/plotSettings/media/printableArea"),
                        "printable area must be a nonempty rectangle inside the media",
                        BTreeMap::new(),
                    ));
                }
                if area == Some("Window") && !valid_rect(&settings["area"]["window"]) {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{index}/attributes/plotSettings/area/window"),
                        "plot window must be a nonempty rectangle",
                        BTreeMap::new(),
                    ));
                }
            }
            if let Some(limits) = layout.pointer("/attributes/limits") {
                if !valid_rect(limits) {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{index}/attributes/limits"),
                        "layout limits must be a nonempty rectangle",
                        BTreeMap::new(),
                    ));
                }
            }
        }
        if model_count != 1 {
            result.diagnostics.push(binding_diagnostic(
                format!("/data/{drawing_index}/children/Layouts"),
                "Drawing requires exactly one model layout",
                BTreeMap::new(),
            ));
        }
        for scope in resource.scopes() {
            if matches!(
                scope,
                crate::ifcdr::ScopeRef::ModelSpace(_) | crate::ifcdr::ScopeRef::PaperSpace(_)
            ) && !selected_scopes.contains(&scope.id().get())
            {
                result.diagnostics.push(binding_diagnostic(
                    format!("/data/{drawing_index}/children/Layouts"),
                    "every model and paper scope needs one layout",
                    BTreeMap::new(),
                ));
            }
        }
        for (row, binding) in resource.bindings().layers().enumerate() {
            if let Some(path) = binding.ifcx_layer() {
                if !layers.contains(path) {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{drawing_index}/children/Layers"),
                        "Drawing Layers must list every IFCDR layer binding target",
                        BTreeMap::from([
                            (
                                "bindingRow".into(),
                                PackageDiagnosticContextValue::Number(row.into()),
                            ),
                            (
                                "targetPath".into(),
                                PackageDiagnosticContextValue::String(path.into()),
                            ),
                        ]),
                    ));
                }
            }
        }
        for (row, binding) in resource.bindings().appearances().enumerate() {
            if let Some(path) = binding.ifcx_appearance() {
                if !appearances.contains(path) {
                    result.diagnostics.push(binding_diagnostic(
                        format!("/data/{drawing_index}/children/Appearances"),
                        "Drawing Appearances must list every IFCDR appearance binding target",
                        BTreeMap::from([
                            (
                                "bindingRow".into(),
                                PackageDiagnosticContextValue::Number(row.into()),
                            ),
                            (
                                "targetPath".into(),
                                PackageDiagnosticContextValue::String(path.into()),
                            ),
                        ]),
                    ));
                }
            }
        }
    }
}

fn valid_rect(rect: &Value) -> bool {
    let Some((min_x, max_x)) = rect["minX"].as_f64().zip(rect["maxX"].as_f64()) else {
        return false;
    };
    let Some((min_y, max_y)) = rect["minY"].as_f64().zip(rect["maxY"].as_f64()) else {
        return false;
    };
    [min_x, max_x, min_y, max_y].into_iter().all(f64::is_finite) && min_x < max_x && min_y < max_y
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
            .drawing_ifcdr_by_path
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
        let kind_matches = matches!(
            (
                node.pointer("/attributes/kind").and_then(Value::as_str),
                resource.scope(scope_id)
            ),
            (Some("model"), Some(crate::ifcdr::ScopeRef::ModelSpace(_)))
                | (Some("paper"), Some(crate::ifcdr::ScopeRef::PaperSpace(_)))
        );
        if !kind_matches {
            result.diagnostics.push(binding_diagnostic(
                format!("/data/{node_index}/attributes/scopeId"),
                "layout kind must match its selected model or paper scope",
                BTreeMap::new(),
            ));
            continue;
        }
        result.bindings.layout_by_path.insert(
            path.to_owned(),
            LayoutBinding {
                representation_path: representation_path.to_owned(),
                ifcdr_resource_id: resource.header().resource_id().clone(),
                scope_id,
            },
        );
    }
}

fn validate_ifcdr_identity_bindings(
    nodes: &[Value],
    node_indices_by_path: &BTreeMap<String, usize>,
    validated_ifcdr_resources: &BTreeMap<ResourceId, Arc<ValidatedIfcdrResource>>,
    result: &mut BindingAnalysis,
) {
    for (resource_id, resource) in validated_ifcdr_resources {
        for (row_index, binding) in resource.bindings().layers().enumerate() {
            let Some(path) = binding.ifcx_layer() else {
                continue;
            };
            if validate_ifcx_identity(
                nodes,
                node_indices_by_path,
                resource.loaded().uri(),
                format!("/layerBindings/{row_index}/ifcxLayer"),
                path,
                "openaec:Layer",
                &mut result.diagnostics,
            ) {
                result
                    .bindings
                    .ifcx_layer_by_ifcdr_id
                    .insert((resource_id.clone(), binding.id()), path.to_owned());
            }
        }
        for (row_index, binding) in resource.bindings().appearances().enumerate() {
            let Some(path) = binding.ifcx_appearance() else {
                continue;
            };
            if validate_ifcx_identity(
                nodes,
                node_indices_by_path,
                resource.loaded().uri(),
                format!("/appearanceBindings/{row_index}/ifcxAppearance"),
                path,
                "openaec:Appearance",
                &mut result.diagnostics,
            ) {
                result
                    .bindings
                    .ifcx_appearance_by_ifcdr_id
                    .insert((resource_id.clone(), binding.id()), path.to_owned());
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
        category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
        code: IFCCAD_PACKAGE_BINDING_INVALID.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: None,
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
    let mut first_kinds = BTreeMap::<&super::source::ResourceSourceKey, ResourceKind>::new();
    let mut diagnostics = Vec::new();
    let mut source_order = declarations.iter().collect::<Vec<_>>();
    source_order.sort_by_key(|declaration| declaration_source_index(declaration));
    for declaration in source_order {
        match first_kinds.get(&declaration.source).copied() {
            None => {
                first_kinds.insert(&declaration.source, declaration.kind);
            }
            Some(first_kind) if first_kind != declaration.kind => {
                diagnostics.push(binding_diagnostic(
                    declaration.source_location.clone(),
                    "one resource URI cannot be both IFCDR and IFCPR",
                    BTreeMap::from([
                        (
                            "resourceUri".to_owned(),
                            PackageDiagnosticContextValue::String(format!(
                                "{:?}",
                                declaration.source
                            )),
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

fn declaration_source_index(declaration: &ResourceDeclaration) -> usize {
    declaration
        .source_location
        .strip_prefix("/data/")
        .and_then(|suffix| suffix.split('/').next())
        .and_then(|index| index.parse().ok())
        .expect("resource declarations retain their IFCX data location")
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
        category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
        code: IFCCAD_PACKAGE_BINDING_INVALID.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: None,
        resource_uri: Some(DIRECTORY_PACKAGE_ENTRYPOINT.to_owned()),
        location: Some(location),
        context,
        message: message.to_owned(),
    }
}

fn target_resource_diagnostic(location: String, resource_id: &ResourceId) -> PackageDiagnostic {
    PackageDiagnostic {
        category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
        code: IFCCAD_PACKAGE_TARGET_RESOURCE_MISSING.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: Some(resource_id.clone()),
        resource_uri: Some(DIRECTORY_PACKAGE_ENTRYPOINT.to_owned()),
        location: Some(location),
        context: BTreeMap::from([(
            "resourceId".to_owned(),
            PackageDiagnosticContextValue::String(resource_id.to_string()),
        )]),
        message: "preservation link does not identify a validated IFCDR resource".to_owned(),
    }
}
