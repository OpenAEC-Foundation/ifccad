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

/// An RGB color preserved together with optional CAD color identities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppearanceColorRef<'a> {
    value: &'a Value,
}

impl<'a> AppearanceColorRef<'a> {
    pub(crate) fn new(value: &'a Value) -> Self {
        Self { value }
    }

    pub fn rgb(&self) -> RgbColor {
        let components = self.value["rgb"]
            .as_array()
            .expect("validated appearance RGB value");
        RgbColor([
            components[0].as_u64().expect("validated red component") as u8,
            components[1].as_u64().expect("validated green component") as u8,
            components[2].as_u64().expect("validated blue component") as u8,
        ])
    }

    pub fn indexed(&self) -> Option<IndexedColorRef<'a>> {
        self.value.get("indexedColor").map(IndexedColorRef::new)
    }

    pub fn named(&self) -> Option<NamedColorRef<'a>> {
        self.value.get("namedColor").map(NamedColorRef::new)
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
    value: &'a Value,
}

impl<'a> IndexedColorRef<'a> {
    fn new(value: &'a Value) -> Self {
        Self { value }
    }

    pub fn system(&self) -> &'a str {
        self.value["system"]
            .as_str()
            .expect("validated indexed-color system")
    }

    pub fn index(&self) -> u64 {
        self.value["index"]
            .as_u64()
            .expect("validated indexed-color index")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NamedColorRef<'a> {
    value: &'a Value,
}

impl<'a> NamedColorRef<'a> {
    fn new(value: &'a Value) -> Self {
        Self { value }
    }

    pub fn catalog(&self) -> &'a str {
        self.value["catalog"]
            .as_str()
            .expect("validated named-color catalog")
    }

    pub fn name(&self) -> &'a str {
        self.value["name"]
            .as_str()
            .expect("validated named-color name")
    }
}

/// A line-pattern value can be a local IFCX appearance value or an IFCX identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinePatternRef<'a> {
    Name(&'a str),
    IfcxIdentity(&'a str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppearanceMode {
    ByLayer,
    Explicit,
    ByBlock,
}

pub(crate) fn appearance_mode(value: u32) -> Option<AppearanceMode> {
    match value {
        0 => Some(AppearanceMode::ByLayer),
        1 => Some(AppearanceMode::Explicit),
        2 => Some(AppearanceMode::ByBlock),
        _ => None,
    }
}

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
            appearance_override.as_ref().and_then(|value| value.color()),
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
                .map(Value::from)
                .as_ref(),
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
            override_line_pattern.as_ref(),
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
                .map(Value::from)
                .as_ref(),
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
    override_value: Option<&Value>,
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
        && !override_value.or(ifcx_value).is_some_and(value_is_valid)
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
    let Some(color) = value.as_object() else {
        return false;
    };
    let valid_rgb = color
        .get("rgb")
        .and_then(Value::as_array)
        .is_some_and(|rgb| {
            rgb.len() == 3
                && rgb
                    .iter()
                    .all(|component| component.as_u64().is_some_and(|component| component <= 255))
        });
    let valid_indexed = color.get("indexedColor").is_none_or(|indexed| {
        indexed.as_object().is_some_and(|indexed| {
            indexed
                .get("system")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty())
                && indexed.get("index").and_then(Value::as_u64).is_some()
        })
    });
    let valid_named = color.get("namedColor").is_none_or(|named| {
        named.as_object().is_some_and(|named| {
            ["catalog", "name"].iter().all(|key| {
                named
                    .get(*key)
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty())
            })
        })
    });
    valid_rgb && valid_indexed && valid_named
}

fn valid_opacity(value: &Value) -> bool {
    value
        .as_f64()
        .is_some_and(|value| value.is_finite() && (0.0..=1.0).contains(&value))
}

fn valid_line_pattern(value: &Value) -> bool {
    value.as_str().is_some_and(|value| !value.is_empty())
}

fn valid_line_weight(value: &Value) -> bool {
    value
        .as_f64()
        .is_some_and(|value| value.is_finite() && value >= 0.0)
}
