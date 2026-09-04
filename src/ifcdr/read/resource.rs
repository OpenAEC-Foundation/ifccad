use crate::ifcdr::{AppearanceId, Bounds2d, IfcdrLengthUnit, LayerId, Point2, ScopeId};
use crate::json_resource::LoadedJsonResource;
use crate::validated::{Validated, ValidationTarget};
use crate::ResourceId;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct LoadedIfcdrResource {
    uri: String,
    source: Arc<LoadedJsonResource>,
}

impl LoadedIfcdrResource {
    pub(crate) fn new(uri: String, source: Arc<LoadedJsonResource>) -> Self {
        Self { uri, source }
    }

    pub(crate) fn uri(&self) -> &str {
        &self.uri
    }

    pub(crate) fn source(&self) -> &Arc<LoadedJsonResource> {
        &self.source
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct IfcdrHeader {
    pub(crate) format: String,
    pub(crate) version: String,
    pub(crate) resource_id: ResourceId,
    pub(crate) unit: String,
    pub(crate) next_entity_id: u64,
}

impl IfcdrHeader {
    pub(crate) fn format(&self) -> &str {
        &self.format
    }
    pub(crate) fn version(&self) -> &str {
        &self.version
    }
    pub(crate) fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    pub(crate) fn unit(&self) -> &str {
        &self.unit
    }
    pub(crate) fn next_entity_id(&self) -> u64 {
        self.next_entity_id
    }
}

#[derive(Debug)]
pub(crate) struct ValidatedTable {
    pub(crate) row_count: usize,
    pub(crate) ids: BTreeMap<u64, usize>,
}

#[derive(Debug)]
pub(crate) struct ValidatedStream {
    pub(crate) registry_index: usize,
    pub(crate) row_count: usize,
}

#[derive(Debug)]
pub(crate) struct IfcdrValidationEvidence {
    pub(crate) header: IfcdrHeader,
    pub(crate) bounds: Bounds2d,
    pub(crate) tables: BTreeMap<String, ValidatedTable>,
    pub(crate) streams: BTreeMap<String, ValidatedStream>,
    pub(crate) entities: super::entity::EntityIndex,
}

pub(crate) type ValidatedIfcdrResource = Validated<LoadedIfcdrResource>;

/// Opaque semantic view over a strictly validated IFCDR resource.
#[derive(Clone, Copy)]
pub struct IfcdrResourceRef<'a> {
    resource: &'a ValidatedIfcdrResource,
}

impl<'a> IfcdrResourceRef<'a> {
    pub(crate) fn new(resource: &'a ValidatedIfcdrResource) -> Self {
        Self { resource }
    }

    pub fn resource_id(&self) -> &'a ResourceId {
        self.resource.header().resource_id()
    }

    pub fn unit(&self) -> IfcdrLengthUnit {
        match self.resource.header().unit() {
            "unitless" => IfcdrLengthUnit::Unitless,
            "mm" => IfcdrLengthUnit::Millimetre,
            "cm" => IfcdrLengthUnit::Centimetre,
            "m" => IfcdrLengthUnit::Metre,
            "km" => IfcdrLengthUnit::Kilometre,
            "in" => IfcdrLengthUnit::Inch,
            "ft" => IfcdrLengthUnit::Foot,
            _ => unreachable!("validated IFCDR length unit"),
        }
    }

    pub fn bounds(&self) -> Bounds2d {
        self.resource.bounds()
    }

    pub fn scopes(&self) -> impl ExactSizeIterator<Item = ScopeRef<'_>> {
        self.resource.scopes()
    }

    pub fn scope(&self, id: ScopeId) -> Option<ScopeRef<'_>> {
        self.resource.scope(id)
    }

    pub fn entities(&self, scope: ScopeId) -> super::entity::EntityIterator<'_> {
        self.resource
            .entities()
            .in_scope(scope)
            .expect("validated IFCDR scope has canonical entity order")
    }
}

impl Validated<LoadedIfcdrResource> {
    pub(crate) fn header(&self) -> &IfcdrHeader {
        &self.evidence().header
    }
    pub(crate) fn bounds(&self) -> Bounds2d {
        self.evidence().bounds
    }

    pub(crate) fn scopes(&self) -> impl ExactSizeIterator<Item = ScopeRef<'_>> {
        rows(self.loaded().source().value(), "scopeTable")
            .iter()
            .map(|row| ScopeRef {
                row: row.as_object().expect("validated scope row"),
            })
    }

    pub(crate) fn scope(&self, id: ScopeId) -> Option<ScopeRef<'_>> {
        self.scopes().find(|scope| scope.id() == id)
    }

    pub(crate) fn layer_binding(&self, id: LayerId) -> Option<LayerBindingRef<'_>> {
        self.bindings().layers().find(|binding| binding.id() == id)
    }

    pub(crate) fn appearance_binding(&self, id: AppearanceId) -> Option<AppearanceBindingRef<'_>> {
        self.bindings()
            .appearances()
            .find(|binding| binding.id() == id)
    }

    pub(crate) fn appearance_override(&self, id: u32) -> Option<AppearanceOverrideRef<'_>> {
        self.bindings()
            .appearance_overrides()
            .find(|appearance_override| appearance_override.id() == id)
    }

    pub(crate) fn bindings(&self) -> IfcdrBindings<'_> {
        IfcdrBindings { resource: self }
    }

    pub(crate) fn streams(&self) -> super::streams::IfcdrStreams<'_> {
        super::streams::IfcdrStreams::new(self)
    }

    pub(crate) fn unmodeled_streams(&self) -> impl Iterator<Item = UnmodeledStreamRef<'_>> {
        self.evidence()
            .streams
            .iter()
            .filter(|(name, _)| name.as_str() != "line" && name.as_str() != "polyline")
            .map(|(_, stream)| {
                let schema =
                    &super::registry::canonical_registry().streams()[stream.registry_index];
                UnmodeledStreamRef {
                    schema,
                    row_count: stream.row_count,
                }
            })
    }
}

pub(crate) struct UnmodeledStreamRef<'a> {
    schema: &'a super::registry::StreamSchema,
    row_count: usize,
}

impl<'a> UnmodeledStreamRef<'a> {
    pub(crate) fn name(&self) -> &'a str {
        self.schema.name()
    }
    pub(crate) fn schema_id(&self) -> &'a str {
        self.schema.schema_id()
    }
    pub(crate) fn role(&self) -> super::registry::StreamRole {
        self.schema.role()
    }
    pub(crate) fn len(&self) -> usize {
        self.row_count
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.row_count == 0
    }
}

pub struct ScopeRef<'a> {
    row: &'a Map<String, Value>,
}

impl ScopeRef<'_> {
    pub fn id(&self) -> ScopeId {
        ScopeId::new(u32_value(self.row, "id"))
    }
    pub fn name(&self) -> &str {
        string_value(self.row, "name")
    }
    pub fn base(&self) -> Point2 {
        Point2::new(
            float_value(self.row, "baseX"),
            float_value(self.row, "baseY"),
        )
    }
    pub fn kind(&self) -> u32 {
        u32_value(self.row, "kind")
    }
    pub fn flags(&self) -> u32 {
        u32_value(self.row, "flags")
    }
}

pub(crate) struct IfcdrBindings<'a> {
    resource: &'a ValidatedIfcdrResource,
}

impl<'a> IfcdrBindings<'a> {
    pub(crate) fn layers(&self) -> impl ExactSizeIterator<Item = LayerBindingRef<'a>> {
        rows(self.resource.loaded().source().value(), "layerBindings")
            .iter()
            .map(|row| LayerBindingRef {
                row: row.as_object().expect("validated layer row"),
            })
    }
    pub(crate) fn appearances(&self) -> impl ExactSizeIterator<Item = AppearanceBindingRef<'a>> {
        rows(
            self.resource.loaded().source().value(),
            "appearanceBindings",
        )
        .iter()
        .map(|row| AppearanceBindingRef {
            row: row.as_object().expect("validated appearance row"),
        })
    }
    pub(crate) fn appearance_overrides(
        &self,
    ) -> impl ExactSizeIterator<Item = AppearanceOverrideRef<'a>> {
        rows(
            self.resource.loaded().source().value(),
            "appearanceOverrides",
        )
        .iter()
        .map(|row| AppearanceOverrideRef {
            row: row.as_object().expect("validated appearance override row"),
        })
    }
}

pub(crate) struct LayerBindingRef<'a> {
    row: &'a Map<String, Value>,
}
impl LayerBindingRef<'_> {
    pub(crate) fn id(&self) -> LayerId {
        LayerId::new(u32_value(self.row, "id"))
    }
    pub(crate) fn ifcx_layer(&self) -> Option<&str> {
        self.row.get("ifcxLayer").and_then(Value::as_str)
    }
}

pub(crate) struct AppearanceBindingRef<'a> {
    row: &'a Map<String, Value>,
}
impl AppearanceBindingRef<'_> {
    pub(crate) fn id(&self) -> AppearanceId {
        AppearanceId::new(u32_value(self.row, "id"))
    }
    pub(crate) fn ifcx_appearance(&self) -> Option<&str> {
        self.row.get("ifcxAppearance").and_then(Value::as_str)
    }
    pub(crate) fn color_mode(&self) -> u32 {
        u32_value(self.row, "colorMode")
    }
    pub(crate) fn opacity_mode(&self) -> u32 {
        u32_value(self.row, "opacityMode")
    }
    pub(crate) fn line_pattern_mode(&self) -> u32 {
        u32_value(self.row, "linePatternMode")
    }
    pub(crate) fn line_weight_mode(&self) -> u32 {
        u32_value(self.row, "lineWeightMode")
    }
    pub(crate) fn override_id(&self) -> Option<u32> {
        self.row
            .get("overrideId")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
    }
}

pub(crate) struct AppearanceOverrideRef<'a> {
    row: &'a Map<String, Value>,
}

impl<'a> AppearanceOverrideRef<'a> {
    pub(crate) fn id(&self) -> u32 {
        u32_value(self.row, "id")
    }
    pub(crate) fn color(&self) -> Option<&'a Value> {
        self.row.get("color").filter(|value| !value.is_null())
    }
    pub(crate) fn opacity(&self) -> Option<f64> {
        self.row.get("opacity").and_then(Value::as_f64)
    }
    pub(crate) fn ifcx_line_pattern(&self) -> Option<&'a str> {
        self.row.get("ifcxLinePattern").and_then(Value::as_str)
    }
    pub(crate) fn line_weight(&self) -> Option<f64> {
        self.row.get("lineWeight").and_then(Value::as_f64)
    }
}

fn rows<'a>(root: &'a Value, key: &str) -> &'a [Value] {
    root.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn u32_value(row: &Map<String, Value>, key: &str) -> u32 {
    row[key]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .expect("validated uint32")
}
fn float_value(row: &Map<String, Value>, key: &str) -> f64 {
    row[key].as_f64().expect("validated float64")
}
fn string_value<'a>(row: &'a Map<String, Value>, key: &str) -> &'a str {
    row[key].as_str().expect("validated string")
}

impl ValidationTarget for LoadedIfcdrResource {
    type Context = super::registry::IfcdrRegistry;
    type Evidence = IfcdrValidationEvidence;
    type Diagnostic = crate::diagnostic::PackageDiagnostic;

    fn build_evidence(
        &self,
        context: &Self::Context,
    ) -> crate::validated::EvidenceOutcome<Self::Evidence, Self::Diagnostic> {
        super::validation::build_evidence(self, context)
    }
}

#[cfg(test)]
pub(crate) fn fixture_source() -> Arc<LoadedJsonResource> {
    use crate::conformance::bundled_conformance_root;

    let path = bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation")
        .join("drawing.ifcdr.json");
    let bytes = std::fs::read(&path).expect("read IFCDR fixture");
    let value = serde_json::from_slice(&bytes).expect("parse IFCDR fixture");
    Arc::new(LoadedJsonResource::new(
        "drawing.ifcdr.json".to_owned(),
        path,
        bytes,
        value,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::validation::validate_ifcdr;
    use super::*;

    #[test]
    fn exposes_typed_header_bounds_scopes_and_bindings() {
        let outcome = validate_ifcdr(LoadedIfcdrResource::new(
            "drawing.ifcdr.json".to_owned(),
            fixture_source(),
        ));
        let resource = outcome.validated().unwrap();

        assert_eq!(resource.header().format(), "openaec.ifcdr");
        assert_eq!(resource.header().version(), "0.5.0");
        assert_eq!(
            resource.header().resource_id().as_str(),
            "geometry-modelspace-main"
        );
        assert_eq!(resource.header().unit(), "m");
        assert_eq!(resource.header().next_entity_id(), 5);
        assert_eq!(resource.bounds().min(), Point2::new(0.0, 0.0));
        assert_eq!(resource.bounds().max(), Point2::new(30.0, 15.0));
        let scopes = resource.scopes().collect::<Vec<_>>();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].id().get(), 0);
        assert_eq!(scopes[0].name(), "ModelSpace");
        assert_eq!(resource.bindings().layers().count(), 2);
        assert_eq!(resource.bindings().appearances().count(), 4);
        assert_eq!(
            resource.scope(ScopeId::new(0)).expect("scope").name(),
            "ModelSpace"
        );
        assert_eq!(
            resource
                .layer_binding(LayerId::new(1))
                .expect("layer binding")
                .ifcx_layer(),
            Some("layer-a-wall")
        );
        assert_eq!(
            resource
                .appearance_binding(AppearanceId::new(2))
                .expect("appearance binding")
                .ifcx_appearance(),
            Some("appearance-default-solid")
        );
        assert_eq!(
            resource
                .unmodeled_streams()
                .map(|stream| stream.name())
                .collect::<Vec<_>>(),
            ["entityOrder", "entityOrderEntry"]
        );
    }

    #[test]
    fn public_resource_view_exposes_semantic_units_and_entity_order() {
        let outcome = validate_ifcdr(LoadedIfcdrResource::new(
            "drawing.ifcdr.json".to_owned(),
            fixture_source(),
        ));
        let resource = outcome.validated().expect("valid IFCDR");
        let view = IfcdrResourceRef::new(resource);
        let scope = view.scopes().next().expect("model scope");

        assert_eq!(view.resource_id().as_str(), "geometry-modelspace-main");
        assert_eq!(view.unit(), IfcdrLengthUnit::Metre);
        assert_eq!(scope.name(), "ModelSpace");
        assert_eq!(
            view.entities(scope.id())
                .map(|entity| match entity {
                    crate::ifcdr::IfcdrEntityRef::Line(line) => line.entity_id().get(),
                    crate::ifcdr::IfcdrEntityRef::Polyline(polyline) => {
                        polyline.entity_id().get()
                    }
                    crate::ifcdr::IfcdrEntityRef::Unmodeled(entity) => {
                        entity.entity_id().get()
                    }
                })
                .collect::<Vec<_>>(),
            [1, 2, 3, 4]
        );
    }
}
