use super::bindings::PackageBindings;
use super::codes::{IFCCAD_PACKAGE_APPEARANCE_INVALID, IFCCAD_PACKAGE_LAYER_NAME_DUPLICATE};
use super::{PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity};
use crate::ifcdr::{AppearanceId, ValidatedIfcdrResource};
use crate::ResourceId;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

/// How one appearance property obtains its value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppearanceProperty<T> {
    ByLayer,
    ByBlock,
    Explicit(T),
}

/// An RGB value with optional indexed and named identities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppearanceColorRef<'a> {
    rgb: RgbColor,
    indexed: Option<IndexedColorRef<'a>>,
    named: Option<NamedColorRef<'a>>,
}
impl<'a> AppearanceColorRef<'a> {
    pub(crate) fn new(value: &'a Value) -> Self {
        let rgb = value["rgb"].as_array().expect("validated color");
        Self {
            rgb: RgbColor([
                rgb[0].as_u64().unwrap() as u8,
                rgb[1].as_u64().unwrap() as u8,
                rgb[2].as_u64().unwrap() as u8,
            ]),
            indexed: value.get("indexedColor").map(|v| IndexedColorRef {
                system: v["system"].as_str().unwrap(),
                index: v["index"].as_u64().unwrap(),
            }),
            named: value.get("namedColor").map(|v| NamedColorRef {
                catalog: v["catalog"].as_str().unwrap(),
                name: v["name"].as_str().unwrap(),
            }),
        }
    }
    pub(crate) fn from_color(value: &'a crate::ifcdr::logical::IfcdrColor) -> Self {
        Self {
            rgb: RgbColor(value.rgb),
            indexed: value.indexed.as_ref().map(|v| IndexedColorRef {
                system: &v.system,
                index: v.index,
            }),
            named: value.named.as_ref().map(|v| NamedColorRef {
                catalog: &v.catalog,
                name: &v.name,
            }),
        }
    }
    pub fn rgb(&self) -> RgbColor {
        self.rgb
    }
    pub fn indexed(&self) -> Option<IndexedColorRef<'a>> {
        self.indexed
    }
    pub fn named(&self) -> Option<NamedColorRef<'a>> {
        self.named
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RgbColor([u8; 3]);
impl RgbColor {
    pub fn components(self) -> [u8; 3] {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IndexedColorRef<'a> {
    system: &'a str,
    index: u64,
}
impl<'a> IndexedColorRef<'a> {
    pub fn system(&self) -> &'a str {
        self.system
    }
    pub fn index(&self) -> u64 {
        self.index
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NamedColorRef<'a> {
    catalog: &'a str,
    name: &'a str,
}
impl<'a> NamedColorRef<'a> {
    pub fn catalog(&self) -> &'a str {
        self.catalog
    }
    pub fn name(&self) -> &'a str {
        self.name
    }
}
/// A line-pattern value can be a local IFCX appearance value or an IFCX identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinePatternRef<'a> {
    Name(&'a str),
    IfcxIdentity(&'a str),
}

pub(crate) use crate::ifcdr::logical::{appearance_mode, AppearanceMode};

pub(crate) fn validate_appearance_and_layer_semantics(
    nodes: &[Value],
    node_indices_by_path: &BTreeMap<String, usize>,
    resources: &BTreeMap<ResourceId, Arc<ValidatedIfcdrResource>>,
    bindings: &PackageBindings,
) -> Vec<PackageDiagnostic> {
    let mut diagnostics = Vec::new();
    for (resource_id, resource) in resources {
        let uri = resource.loaded().uri();
        validate_appearances(
            resource_id,
            uri,
            resource,
            nodes,
            node_indices_by_path,
            bindings,
            &mut diagnostics,
        );
        validate_layer_names(
            resource_id,
            uri,
            resource,
            nodes,
            node_indices_by_path,
            bindings,
            &mut diagnostics,
        );
    }
    diagnostics
}

fn validate_appearances(
    resource_id: &ResourceId,
    uri: &str,
    resource: &ValidatedIfcdrResource,
    nodes: &[Value],
    node_indices_by_path: &BTreeMap<String, usize>,
    bindings: &PackageBindings,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    // Declared links must resolve even when no current binding selects them.
    for (row, value) in resource.bindings().appearance_overrides().enumerate() {
        if let Some(path) = value.ifcx_line_pattern() {
            if !node_indices_by_path.contains_key(path) {
                diagnostics.push(PackageDiagnostic {
                    category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
                    code: IFCCAD_PACKAGE_APPEARANCE_INVALID.to_owned(),
                    severity: PackageDiagnosticSeverity::Error,
                    resource_id: Some(resource_id.clone()),
                    resource_uri: Some(uri.to_owned()),
                    location: Some(format!("/appearanceOverrides/{row}/ifcxLinePattern")),
                    context: BTreeMap::from([(
                        "target".to_owned(),
                        PackageDiagnosticContextValue::String(path.to_owned()),
                    )]),
                    message: "line-pattern override must reference an existing IFCX identity"
                        .into(),
                });
            }
        }
    }
    for (row_index, binding) in resource.bindings().appearances().enumerate() {
        let ifcx_appearance = bindings
            .ifcx_appearance_by_ifcdr_id
            .get(&(resource_id.clone(), binding.id()))
            .and_then(|path| node_indices_by_path.get(path))
            .and_then(|index| nodes.get(*index));
        let appearance_override = binding
            .override_id()
            .and_then(|id| resource.appearance_override(id));

        let override_line_pattern = appearance_override
            .as_ref()
            .and_then(|value| value.ifcx_line_pattern())
            .map(|path| {
                if node_indices_by_path.contains_key(path) {
                    Value::String(path.to_owned())
                } else {
                    Value::Null
                }
            });
        validate_property(
            uri,
            resource_id,
            row_index,
            binding.id(),
            "color",
            "colorMode",
            binding.color_mode(),
            appearance_override
                .as_ref()
                .and_then(|value| value.color())
                .map(crate::ifcdr::logical::valid_color),
            ifcx_appearance.and_then(|node| node.pointer("/attributes/color/value")),
            valid_color,
            diagnostics,
        );
        validate_property(
            uri,
            resource_id,
            row_index,
            binding.id(),
            "opacity",
            "opacityMode",
            binding.opacity_mode(),
            appearance_override
                .as_ref()
                .and_then(|value| value.opacity())
                .map(|_| true),
            ifcx_appearance.and_then(|node| node.pointer("/attributes/opacity/value")),
            valid_opacity,
            diagnostics,
        );
        validate_property(
            uri,
            resource_id,
            row_index,
            binding.id(),
            "linePattern",
            "linePatternMode",
            binding.line_pattern_mode(),
            override_line_pattern.as_ref().map(valid_line_pattern),
            ifcx_appearance.and_then(|node| node.pointer("/attributes/linePattern/value")),
            valid_line_pattern,
            diagnostics,
        );
        validate_property(
            uri,
            resource_id,
            row_index,
            binding.id(),
            "lineWeight",
            "lineWeightMode",
            binding.line_weight_mode(),
            appearance_override
                .as_ref()
                .and_then(|value| value.line_weight())
                .map(|_| true),
            ifcx_appearance.and_then(|node| node.pointer("/attributes/lineWeight/value")),
            valid_line_weight,
            diagnostics,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_property(
    uri: &str,
    resource_id: &ResourceId,
    row_index: usize,
    binding_id: AppearanceId,
    property: &str,
    mode_field: &str,
    raw_mode: u32,
    override_value: Option<bool>,
    ifcx_value: Option<&Value>,
    value_is_valid: fn(&Value) -> bool,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    let Some(mode) = appearance_mode(raw_mode) else {
        diagnostics.push(appearance_diagnostic(
            uri,
            resource_id,
            row_index,
            binding_id,
            property,
            mode_field,
            raw_mode,
            "appearance mode must be 0 (ByLayer), 1 (Explicit), or 2 (ByBlock)",
        ));
        return;
    };
    if mode == AppearanceMode::Explicit
        && !override_value
            .or_else(|| ifcx_value.map(value_is_valid))
            .unwrap_or(false)
    {
        diagnostics.push(appearance_diagnostic(
            uri,
            resource_id,
            row_index,
            binding_id,
            property,
            mode_field,
            raw_mode,
            "explicit appearance property has no valid override or IFCX value",
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn appearance_diagnostic(
    uri: &str,
    resource_id: &ResourceId,
    row_index: usize,
    binding_id: AppearanceId,
    property: &str,
    mode_field: &str,
    raw_mode: u32,
    message: &str,
) -> PackageDiagnostic {
    PackageDiagnostic {
        category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
        code: IFCCAD_PACKAGE_APPEARANCE_INVALID.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: Some(resource_id.clone()),
        resource_uri: Some(uri.to_owned()),
        location: Some(format!("/appearanceBindings/{row_index}/{mode_field}")),
        context: BTreeMap::from([
            (
                "appearanceId".to_owned(),
                PackageDiagnosticContextValue::Number(binding_id.get().into()),
            ),
            (
                "mode".to_owned(),
                PackageDiagnosticContextValue::Number(raw_mode.into()),
            ),
            (
                "property".to_owned(),
                PackageDiagnosticContextValue::String(property.to_owned()),
            ),
        ]),
        message: message.to_owned(),
    }
}

fn validate_layer_names(
    resource_id: &ResourceId,
    uri: &str,
    resource: &ValidatedIfcdrResource,
    nodes: &[Value],
    node_indices_by_path: &BTreeMap<String, usize>,
    bindings: &PackageBindings,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    let mut first_by_name = BTreeMap::<String, (u32, String)>::new();
    for (row_index, binding) in resource.bindings().layers().enumerate() {
        let Some(path) = bindings
            .ifcx_layer_by_ifcdr_id
            .get(&(resource_id.clone(), binding.id()))
        else {
            continue;
        };
        let Some(name) = node_indices_by_path
            .get(path)
            .and_then(|index| nodes.get(*index))
            .and_then(|node| node.pointer("/attributes/name"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let normalized = name.to_lowercase();
        if let Some((first_id, first_path)) = first_by_name.get(&normalized) {
            diagnostics.push(PackageDiagnostic {
                category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
                code: IFCCAD_PACKAGE_LAYER_NAME_DUPLICATE.to_owned(),
                severity: PackageDiagnosticSeverity::Error,
                resource_id: Some(resource_id.clone()),
                resource_uri: Some(uri.to_owned()),
                location: Some(format!("/layerBindings/{row_index}/ifcxLayer")),
                context: BTreeMap::from([
                    (
                        "firstLayerId".to_owned(),
                        PackageDiagnosticContextValue::Number((*first_id).into()),
                    ),
                    (
                        "firstLayerPath".to_owned(),
                        PackageDiagnosticContextValue::String(first_path.clone()),
                    ),
                    (
                        "layerId".to_owned(),
                        PackageDiagnosticContextValue::Number(binding.id().get().into()),
                    ),
                    (
                        "layerName".to_owned(),
                        PackageDiagnosticContextValue::String(name.to_owned()),
                    ),
                    (
                        "layerPath".to_owned(),
                        PackageDiagnosticContextValue::String(path.clone()),
                    ),
                ]),
                message: "layer names must be case-insensitively unique within one IFCDR resource"
                    .to_owned(),
            });
        } else {
            first_by_name.insert(normalized, (binding.id().get(), path.clone()));
        }
    }
}

fn valid_color(value: &Value) -> bool {
    crate::ifcdr::codec::json::project_color(value)
        .is_some_and(|color| crate::ifcdr::logical::valid_color(&color))
}

fn valid_opacity(value: &Value) -> bool {
    value
        .as_f64()
        .is_some_and(crate::ifcdr::logical::valid_opacity)
}

fn valid_line_pattern(value: &Value) -> bool {
    value.as_str().is_some_and(|value| !value.is_empty())
}

fn valid_line_weight(value: &Value) -> bool {
    value
        .as_f64()
        .is_some_and(crate::ifcdr::logical::valid_line_weight)
}
