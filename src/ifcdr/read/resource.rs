use super::decoded::DecodedIfcdrResource;
use crate::ifcdr::logical::*;
use crate::ifcdr::{AppearanceId, Bounds2d, IfcdrLengthUnit, LayerId, Point2, ScopeId};
use crate::json_resource::LoadedJsonResource;
use crate::validated::{Validated, ValidationTarget};
use crate::ResourceId;
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
#[derive(Debug)]
pub(crate) struct IfcdrValidationEvidence {
    pub data: ValidatedIfcdr<DecodedIfcdrResource>,
}
/// Reader result retaining the original source alongside its validated typed
/// interpretation. Evidence owns `ValidatedIfcdr<DecodedIfcdrResource>`;
/// entity views use that backing, not the source JSON. Package-level links
/// are checked separately. This is the source-aware counterpart of the
/// shared, storage-generic `ValidatedIfcdr<R>` proof.
pub(crate) type ValidatedIfcdrResource = Validated<LoadedIfcdrResource>;
impl ValidationTarget for LoadedIfcdrResource {
    type Context = ();
    type Evidence = IfcdrValidationEvidence;
    type Diagnostic = crate::diagnostic::PackageDiagnostic;
    fn build_evidence(
        &self,
        _: &(),
    ) -> crate::validated::EvidenceOutcome<Self::Evidence, Self::Diagnostic> {
        super::validation::build_evidence(self)
    }
}
pub(crate) struct IfcdrHeader<'a>(&'a DecodedIfcdrResource);
impl<'a> IfcdrHeader<'a> {
    pub(crate) fn resource_id(&self) -> &'a ResourceId {
        &self.0.id
    }
    pub(crate) fn next_entity_id(&self) -> u64 {
        self.0.next
    }
    pub(crate) fn format(&self) -> &str {
        "openaec.ifcdr"
    }
    pub(crate) fn version(&self) -> &str {
        "0.7.0"
    }
    pub(crate) fn unit(&self) -> &str {
        super::super::codec::json::unit_name(self.0.unit)
    }
}
#[derive(Clone, Copy)]
pub struct IfcdrResourceRef<'a> {
    resource: &'a ValidatedIfcdrResource,
}
impl<'a> IfcdrResourceRef<'a> {
    pub(crate) fn new(resource: &'a ValidatedIfcdrResource) -> Self {
        Self { resource }
    }
    pub fn resource_id(&self) -> &'a ResourceId {
        &self.resource.typed().id
    }
    pub fn unit(&self) -> IfcdrLengthUnit {
        self.resource.typed().unit
    }
    /// Geometric bounds, absent for an empty resource.
    pub fn bounds(&self) -> Option<Bounds2d> {
        self.resource.bounds()
    }
    pub fn scopes(&self) -> impl ExactSizeIterator<Item = ScopeRef<'_>> {
        self.resource.scopes()
    }
    pub fn scope(&self, id: ScopeId) -> Option<ScopeRef<'_>> {
        self.resource.scope(id)
    }
    pub fn entities(&self, scope: ScopeId) -> super::entity::EntityIterator<'_> {
        super::entity::EntityIterator::new(self.resource, scope)
    }
}
impl Validated<LoadedIfcdrResource> {
    pub(crate) fn typed(&self) -> &DecodedIfcdrResource {
        self.evidence().data.loaded().resource()
    }
    pub(crate) fn header(&self) -> IfcdrHeader<'_> {
        IfcdrHeader(self.typed())
    }
    pub(crate) fn bounds(&self) -> Option<Bounds2d> {
        self.typed().bounds
    }
    pub(crate) fn scopes(&self) -> impl ExactSizeIterator<Item = ScopeRef<'_>> {
        self.typed().scopes.iter().map(|row| ScopeRef { row })
    }
    pub(crate) fn scope(&self, id: ScopeId) -> Option<ScopeRef<'_>> {
        self.scopes().find(|s| s.id() == id)
    }
    pub(crate) fn bindings(&self) -> IfcdrBindings<'_> {
        IfcdrBindings { resource: self }
    }
    pub(crate) fn streams(&self) -> super::streams::IfcdrStreams<'_> {
        super::streams::IfcdrStreams::new(self)
    }
    pub(crate) fn layer_binding(&self, id: LayerId) -> Option<LayerBindingRef<'_>> {
        self.bindings().layers().find(|v| v.id() == id)
    }
    pub(crate) fn appearance_binding(&self, id: AppearanceId) -> Option<AppearanceBindingRef<'_>> {
        self.bindings().appearances().find(|v| v.id() == id)
    }
    pub(crate) fn appearance_override(&self, id: u32) -> Option<AppearanceOverrideRef<'_>> {
        self.bindings()
            .appearance_overrides()
            .find(|v| v.id() == id)
    }
}
pub(crate) struct IfcdrBindings<'a> {
    resource: &'a ValidatedIfcdrResource,
}
impl<'a> IfcdrBindings<'a> {
    pub(crate) fn layers(&self) -> impl ExactSizeIterator<Item = LayerBindingRef<'a>> {
        self.resource
            .typed()
            .layers
            .iter()
            .map(|row| LayerBindingRef { row })
    }
    pub(crate) fn appearances(&self) -> impl ExactSizeIterator<Item = AppearanceBindingRef<'a>> {
        self.resource
            .typed()
            .appearances
            .iter()
            .map(|row| AppearanceBindingRef { row })
    }
    pub(crate) fn appearance_overrides(
        &self,
    ) -> impl ExactSizeIterator<Item = AppearanceOverrideRef<'a>> {
        self.resource
            .typed()
            .overrides
            .iter()
            .map(|row| AppearanceOverrideRef { row })
    }
}
pub struct ScopeRef<'a> {
    row: &'a IfcdrScope,
}
impl<'a> ScopeRef<'a> {
    pub fn id(&self) -> ScopeId {
        ScopeId::new(self.row.id)
    }
    pub fn name(&self) -> &str {
        &self.row.name
    }
    pub fn base(&self) -> Point2 {
        self.row.base
    }
    pub fn kind(&self) -> u32 {
        self.row.kind
    }
    pub fn flags(&self) -> u32 {
        self.row.flags
    }
}
pub(crate) struct LayerBindingRef<'a> {
    row: &'a IfcdrLayerBinding,
}
impl<'a> LayerBindingRef<'a> {
    pub(crate) fn id(&self) -> LayerId {
        LayerId::new(self.row.id)
    }
    pub(crate) fn ifcx_layer(&self) -> Option<&'a str> {
        Some(&self.row.ifcx_layer)
    }
}
pub(crate) struct AppearanceBindingRef<'a> {
    row: &'a IfcdrAppearanceBinding,
}
impl<'a> AppearanceBindingRef<'a> {
    pub(crate) fn id(&self) -> AppearanceId {
        AppearanceId::new(self.row.id)
    }
    pub(crate) fn ifcx_appearance(&self) -> Option<&'a str> {
        self.row.ifcx_appearance.as_deref()
    }
    pub(crate) fn override_id(&self) -> Option<u32> {
        self.row.override_id
    }
    pub(crate) fn color_mode(&self) -> u32 {
        self.row.modes[0]
    }
    pub(crate) fn opacity_mode(&self) -> u32 {
        self.row.modes[1]
    }
    pub(crate) fn line_pattern_mode(&self) -> u32 {
        self.row.modes[2]
    }
    pub(crate) fn line_weight_mode(&self) -> u32 {
        self.row.modes[3]
    }
}
pub(crate) struct AppearanceOverrideRef<'a> {
    row: &'a IfcdrAppearanceOverride,
}
impl<'a> AppearanceOverrideRef<'a> {
    pub(crate) fn id(&self) -> u32 {
        self.row.id
    }
    pub(crate) fn color(&self) -> Option<&'a IfcdrColor> {
        self.row.color.as_ref()
    }
    pub(crate) fn opacity(&self) -> Option<f64> {
        self.row.opacity
    }
    pub(crate) fn line_weight(&self) -> Option<f64> {
        self.row.line_weight
    }
    pub(crate) fn ifcx_line_pattern(&self) -> Option<&'a str> {
        self.row.ifcx_line_pattern.as_deref()
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
        assert_eq!(resource.header().version(), "0.7.0");
        assert_eq!(resource.header().resource_id().as_str(), "drawing-main");
        assert_eq!(resource.header().unit(), "m");
        assert_eq!(resource.header().next_entity_id(), 5);
        assert_eq!(resource.bounds().unwrap().min(), Point2::new(0.0, 0.0));
        assert_eq!(resource.bounds().unwrap().max(), Point2::new(30.0, 15.0));
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

        assert_eq!(view.resource_id().as_str(), "drawing-main");
        assert_eq!(view.unit(), IfcdrLengthUnit::Metre);
        assert_eq!(scope.name(), "ModelSpace");
        assert_eq!(
            view.entities(scope.id())
                .map(|entity| match entity {
                    crate::ifcdr::IfcdrEntityRef::Line(line) => line.entity_id().get(),
                    crate::ifcdr::IfcdrEntityRef::Polyline(polyline) => {
                        polyline.entity_id().get()
                    }
                })
                .collect::<Vec<_>>(),
            [1, 2, 3, 4]
        );
    }
}
