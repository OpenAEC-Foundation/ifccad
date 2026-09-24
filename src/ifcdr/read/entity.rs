use super::resource::{LoadedIfcdrResource, ValidatedIfcdrResource};
use super::streams::{Line, PlanarPolylineRef};
use crate::ifcdr::logical::{IfcdrEntityKind, IfcdrResourceAccess};
use crate::ifcdr::ScopeId;
use crate::validated::Validated;
pub struct EntityIterator<'a> {
    resource: &'a ValidatedIfcdrResource,
    ids: std::slice::Iter<'a, u64>,
}
impl<'a> EntityIterator<'a> {
    pub(crate) fn new(resource: &'a ValidatedIfcdrResource, scope: ScopeId) -> Self {
        Self {
            resource,
            ids: resource.typed().order(scope.get()).unwrap_or(&[]).iter(),
        }
    }
}
impl<'a> Iterator for EntityIterator<'a> {
    type Item = IfcdrEntityRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let id = self.ids.next()?;
        let location = self
            .resource
            .evidence()
            .data
            .evidence()
            .entities
            .get(id)
            .expect("validated ordered ID");
        Some(match location.kind {
            IfcdrEntityKind::Point => IfcdrEntityRef::Point(PointRef {
                row: &self.resource.typed().points[location.row],
            }),
            IfcdrEntityKind::Circle => IfcdrEntityRef::Circle(CircleRef {
                row: &self.resource.typed().circles[location.row],
            }),
            IfcdrEntityKind::Arc => IfcdrEntityRef::Arc(ArcRef {
                row: &self.resource.typed().arcs[location.row],
            }),
            IfcdrEntityKind::Ellipse => IfcdrEntityRef::Ellipse(EllipseRef {
                row: &self.resource.typed().ellipses[location.row],
            }),
            IfcdrEntityKind::EllipseArc => IfcdrEntityRef::EllipseArc(EllipseArcRef {
                row: &self.resource.typed().ellipse_arcs[location.row],
            }),
            IfcdrEntityKind::Viewport => IfcdrEntityRef::Viewport(ViewportRef {
                row: &self.resource.typed().viewports[location.row],
            }),
            IfcdrEntityKind::BlockInstance => IfcdrEntityRef::BlockInstance(BlockInstanceRef {
                row: self.resource.typed().block_instances[location.row],
            }),
            IfcdrEntityKind::Line => IfcdrEntityRef::Line(
                self.resource
                    .streams()
                    .lines()
                    .unwrap()
                    .get(location.row)
                    .expect("validated line"),
            ),
            IfcdrEntityKind::PlanarPolyline => IfcdrEntityRef::PlanarPolyline(
                self.resource
                    .streams()
                    .polylines()
                    .unwrap()
                    .get(location.row)
                    .expect("validated polyline"),
            ),
            IfcdrEntityKind::SpatialPolyline => {
                IfcdrEntityRef::SpatialPolyline(SpatialPolylineRef {
                    row: &self.resource.typed().spatial_polylines[location.row],
                })
            }
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}
impl ExactSizeIterator for EntityIterator<'_> {}
pub enum IfcdrEntityRef<'a> {
    Point(PointRef<'a>),
    Circle(CircleRef<'a>),
    Arc(ArcRef<'a>),
    Ellipse(EllipseRef<'a>),
    EllipseArc(EllipseArcRef<'a>),
    Line(Line),
    PlanarPolyline(PlanarPolylineRef<'a>),
    SpatialPolyline(SpatialPolylineRef<'a>),
    BlockInstance(BlockInstanceRef),
    Viewport(ViewportRef<'a>),
}
#[derive(Clone, Copy, Debug)]
pub struct SpatialPolylineRef<'a> {
    row: &'a crate::ifcdr::logical::IfcdrSpatialPolylineRow,
}
impl SpatialPolylineRef<'_> {
    pub fn entity_id(&self) -> crate::ifcdr::EntityId {
        crate::ifcdr::EntityId::new(self.row.entity.entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.entity.scope_id)
    }
    pub fn layer_id(&self) -> crate::ifcdr::LayerId {
        self.row.entity.layer_id.into()
    }
    pub fn appearance_id(&self) -> crate::ifcdr::AppearanceId {
        self.row.entity.appearance_id.into()
    }
    pub fn visible(&self) -> bool {
        self.row.entity.visible
    }
    pub fn closed(&self) -> bool {
        self.row.closed
    }
    pub fn points(&self) -> &[crate::ifcdr::Point3] {
        &self.row.points
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PointRef<'a> {
    row: &'a crate::ifcdr::logical::IfcdrPointRow,
}
impl PointRef<'_> {
    pub fn entity_id(&self) -> crate::ifcdr::EntityId {
        crate::ifcdr::EntityId::new(self.row.entity.entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.entity.scope_id)
    }
    pub fn layer_id(&self) -> crate::ifcdr::LayerId {
        self.row.entity.layer_id.into()
    }
    pub fn appearance_id(&self) -> crate::ifcdr::AppearanceId {
        self.row.entity.appearance_id.into()
    }
    pub fn visible(&self) -> bool {
        self.row.entity.visible
    }
    pub fn placement(&self) -> crate::ifcdr::PlanePlacement {
        crate::ifcdr::PlanePlacement::from_validated_components(self.row.placement)
    }
    pub fn position(&self) -> crate::ifcdr::Point3 {
        self.row.placement.origin
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CircleRef<'a> {
    row: &'a crate::ifcdr::logical::IfcdrCircleRow,
}
impl CircleRef<'_> {
    pub fn entity_id(&self) -> crate::ifcdr::EntityId {
        crate::ifcdr::EntityId::new(self.row.entity.entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.entity.scope_id)
    }
    pub fn layer_id(&self) -> crate::ifcdr::LayerId {
        self.row.entity.layer_id.into()
    }
    pub fn appearance_id(&self) -> crate::ifcdr::AppearanceId {
        self.row.entity.appearance_id.into()
    }
    pub fn visible(&self) -> bool {
        self.row.entity.visible
    }
    pub fn placement(&self) -> crate::ifcdr::PlanePlacement {
        crate::ifcdr::PlanePlacement::from_validated_components(self.row.placement)
    }
    pub fn radius(&self) -> f64 {
        self.row.radius
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ArcRef<'a> {
    row: &'a crate::ifcdr::logical::IfcdrArcRow,
}
impl ArcRef<'_> {
    pub fn entity_id(&self) -> crate::ifcdr::EntityId {
        crate::ifcdr::EntityId::new(self.row.entity.entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.entity.scope_id)
    }
    pub fn layer_id(&self) -> crate::ifcdr::LayerId {
        self.row.entity.layer_id.into()
    }
    pub fn appearance_id(&self) -> crate::ifcdr::AppearanceId {
        self.row.entity.appearance_id.into()
    }
    pub fn visible(&self) -> bool {
        self.row.entity.visible
    }
    pub fn placement(&self) -> crate::ifcdr::PlanePlacement {
        crate::ifcdr::PlanePlacement::from_validated_components(self.row.placement)
    }
    pub fn radius(&self) -> f64 {
        self.row.radius
    }
    pub fn start_parameter(&self) -> f64 {
        self.row.start_parameter
    }
    pub fn sweep_parameter(&self) -> f64 {
        self.row.sweep_parameter
    }
}
#[derive(Clone, Copy, Debug)]
pub struct EllipseRef<'a> {
    row: &'a crate::ifcdr::logical::IfcdrEllipseRow,
}
impl EllipseRef<'_> {
    pub fn entity_id(&self) -> crate::ifcdr::EntityId {
        crate::ifcdr::EntityId::new(self.row.entity.entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.entity.scope_id)
    }
    pub fn layer_id(&self) -> crate::ifcdr::LayerId {
        self.row.entity.layer_id.into()
    }
    pub fn appearance_id(&self) -> crate::ifcdr::AppearanceId {
        self.row.entity.appearance_id.into()
    }
    pub fn visible(&self) -> bool {
        self.row.entity.visible
    }
    pub fn placement(&self) -> crate::ifcdr::PlanePlacement {
        crate::ifcdr::PlanePlacement::from_validated_components(self.row.placement)
    }
    pub fn semi_major_radius(&self) -> f64 {
        self.row.semi_major_radius
    }
    pub fn semi_minor_radius(&self) -> f64 {
        self.row.semi_minor_radius
    }
}
#[derive(Clone, Copy, Debug)]
pub struct EllipseArcRef<'a> {
    row: &'a crate::ifcdr::logical::IfcdrEllipseArcRow,
}
impl EllipseArcRef<'_> {
    pub fn entity_id(&self) -> crate::ifcdr::EntityId {
        crate::ifcdr::EntityId::new(self.row.entity.entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.entity.scope_id)
    }
    pub fn layer_id(&self) -> crate::ifcdr::LayerId {
        self.row.entity.layer_id.into()
    }
    pub fn appearance_id(&self) -> crate::ifcdr::AppearanceId {
        self.row.entity.appearance_id.into()
    }
    pub fn visible(&self) -> bool {
        self.row.entity.visible
    }
    pub fn placement(&self) -> crate::ifcdr::PlanePlacement {
        crate::ifcdr::PlanePlacement::from_validated_components(self.row.placement)
    }
    pub fn semi_major_radius(&self) -> f64 {
        self.row.semi_major_radius
    }
    pub fn semi_minor_radius(&self) -> f64 {
        self.row.semi_minor_radius
    }
    pub fn start_parameter(&self) -> f64 {
        self.row.start_parameter
    }
    pub fn sweep_parameter(&self) -> f64 {
        self.row.sweep_parameter
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ViewportRef<'a> {
    row: &'a crate::ifcdr::logical::IfcdrViewportRow,
}
impl ViewportRef<'_> {
    pub fn entity_id(&self) -> crate::ifcdr::EntityId {
        crate::ifcdr::EntityId::new(self.row.entity.entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.entity.scope_id)
    }
    pub fn view_scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.view_scope_id)
    }
    pub fn layer_id(&self) -> crate::ifcdr::LayerId {
        self.row.entity.layer_id.into()
    }
    pub fn appearance_id(&self) -> crate::ifcdr::AppearanceId {
        self.row.entity.appearance_id.into()
    }
    pub fn visible(&self) -> bool {
        self.row.entity.visible
    }
    pub fn frame(&self) -> crate::ifcdr::ViewportFrame {
        self.row.frame
    }
    pub fn view(&self) -> crate::ifcdr::ViewDefinition {
        self.row.view
    }
    pub fn render_mode(&self) -> crate::ifcdr::ViewportRenderMode {
        self.row.render_mode
    }
    pub fn view_enabled(&self) -> bool {
        self.row.view_enabled
    }
    pub fn view_locked(&self) -> bool {
        self.row.view_locked
    }
    pub fn paper_clip(&self) -> crate::ifcdr::PaperClip {
        self.row.paper_clip
    }
    pub fn plot_shading_override(&self) -> Option<crate::ifcdr::ShadedPlot> {
        self.row.plot_shading_override
    }
    pub fn layer_overrides(&self) -> &[crate::ifcdr::ViewportLayerOverride] {
        &self.row.layer_overrides
    }
}
/// A validated insertion row; no implicit explosion of definition geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockInstanceRef {
    row: crate::ifcdr::logical::IfcdrBlockInstanceRow,
}
impl BlockInstanceRef {
    pub fn entity_id(&self) -> crate::ifcdr::EntityId {
        crate::ifcdr::EntityId::new(self.row.entity.entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.entity.scope_id)
    }
    pub fn definition_scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.definition_scope_id)
    }
    pub fn layer_id(&self) -> crate::ifcdr::LayerId {
        self.row.entity.layer_id.into()
    }
    pub fn appearance_id(&self) -> crate::ifcdr::AppearanceId {
        self.row.entity.appearance_id.into()
    }
    pub fn visible(&self) -> bool {
        self.row.entity.visible
    }
    pub fn transform(&self) -> crate::ifcdr::BlockTransform {
        crate::ifcdr::BlockTransform::from_validated_components(self.row.transform)
    }
}
pub(crate) struct IfcdrEntities<'a> {
    resource: &'a ValidatedIfcdrResource,
}
impl<'a> IfcdrEntities<'a> {
    pub(crate) fn in_scope(&self, scope: ScopeId) -> Option<EntityIterator<'a>> {
        self.resource.typed().order(scope.get())?;
        Some(EntityIterator::new(self.resource, scope))
    }
}
impl Validated<LoadedIfcdrResource> {
    pub(crate) fn entities(&self) -> IfcdrEntities<'_> {
        IfcdrEntities { resource: self }
    }
}
#[cfg(test)]
mod tests {
    use super::super::resource::{fixture_source, LoadedIfcdrResource};
    use super::super::validation::{validate_ifcdr, validate_value};
    use super::{IfcdrEntityRef, ScopeId};
    use crate::ifcdr::logical::IfcdrResourceAccess;

    #[test]
    fn indexes_all_minimal_entities_in_scope_order() {
        let outcome = validate_ifcdr(LoadedIfcdrResource::new(
            "drawing.ifcdr.json".to_owned(),
            fixture_source(),
        ));
        let validated = outcome.validated().expect("valid IFCDR");
        assert_eq!(validated.typed().order(0).unwrap(), [1, 2, 3, 4]);
        assert!(validated.typed().order(99).is_none());
    }

    #[test]
    fn uniform_iterator_dispatches_lines_and_polylines_in_stored_order() {
        let outcome = validate_ifcdr(LoadedIfcdrResource::new(
            "drawing.ifcdr.json".to_owned(),
            fixture_source(),
        ));
        let resource = outcome.validated().unwrap();
        let kinds_and_ids = resource
            .entities()
            .in_scope(ScopeId::new(0))
            .unwrap()
            .map(|entity| match entity {
                IfcdrEntityRef::Point(point) => ("point", point.entity_id().get()),
                IfcdrEntityRef::Circle(circle) => ("circle", circle.entity_id().get()),
                IfcdrEntityRef::Arc(arc) => ("arc", arc.entity_id().get()),
                IfcdrEntityRef::Ellipse(ellipse) => ("ellipse", ellipse.entity_id().get()),
                IfcdrEntityRef::EllipseArc(arc) => ("ellipseArc", arc.entity_id().get()),
                IfcdrEntityRef::Line(line) => ("line", line.entity_id().get()),
                IfcdrEntityRef::PlanarPolyline(polyline) => {
                    ("polyline", polyline.entity_id().get())
                }
                IfcdrEntityRef::SpatialPolyline(polyline) => {
                    ("spatialPolyline", polyline.entity_id().get())
                }
                IfcdrEntityRef::BlockInstance(instance) => {
                    ("blockInstance", instance.entity_id().get())
                }
                IfcdrEntityRef::Viewport(viewport) => ("viewport", viewport.entity_id().get()),
            })
            .collect::<Vec<_>>();

        assert_eq!(
            kinds_and_ids,
            [("line", 1), ("line", 2), ("polyline", 3), ("polyline", 4)]
        );
        assert!(resource.entities().in_scope(ScopeId::new(99)).is_none());
    }

    #[test]
    fn rejects_zero_and_cross_stream_duplicate_entity_ids() {
        let source = fixture_source();
        let mut zero = source.value().clone();
        zero["streams"]["lineStream"]["entityId"][0] = serde_json::json!(0);
        let zero = validate_value("drawing.ifcdr.json", zero);
        assert!(zero.diagnostics().iter().any(|item| {
            item.code == "IFCCAD_IFCDR_ENTITY_ID_INVALID"
                && item.location.as_deref() == Some("/streams/lineStream/entityId/0")
        }));

        let mut duplicate = source.value().clone();
        duplicate["streams"]["planarPolylineStream"]["entityId"][0] = serde_json::json!(1);
        let duplicate = validate_value("drawing.ifcdr.json", duplicate);
        assert!(duplicate.diagnostics().iter().any(|item| {
            item.code == "IFCCAD_IFCDR_ENTITY_ID_DUPLICATE"
                && item.location.as_deref() == Some("/streams/planarPolylineStream/entityId/0")
        }));
    }

    #[test]
    fn rejects_incomplete_or_wrong_scope_entity_order() {
        let source = fixture_source();
        let mut incomplete = source.value().clone();
        incomplete["streams"]["entityOrderStream"]["entryCount"][0] = serde_json::json!(3);
        let incomplete = validate_value("drawing.ifcdr.json", incomplete);
        assert!(incomplete
            .diagnostics()
            .iter()
            .any(|item| { item.code == "IFCCAD_IFCDR_ENTITY_ORDER_INVALID" }));

        let mut missing = source.value().clone();
        missing["streams"]["entityOrderEntryStream"]["entityId"][3] = serde_json::json!(99);
        let missing = validate_value("drawing.ifcdr.json", missing);
        assert!(missing.diagnostics().iter().any(|item| {
            item.code == "IFCCAD_IFCDR_ENTITY_ORDER_INVALID"
                && item.location.as_deref() == Some("/streams/entityOrderEntryStream/entityId/3")
        }));
    }

    #[test]
    fn rejects_missing_or_noncontiguous_scope_order() {
        let source = fixture_source();
        let mut noncontiguous = source.value().clone();
        noncontiguous["streams"]["entityOrderStream"]["entryOffset"][0] = serde_json::json!(1);
        noncontiguous["streams"]["entityOrderStream"]["entryCount"][0] = serde_json::json!(3);
        let noncontiguous = validate_value("drawing.ifcdr.json", noncontiguous);
        assert!(noncontiguous.diagnostics().iter().any(|item| {
            item.code == "IFCCAD_IFCDR_ENTITY_ORDER_INVALID"
                && item.location.as_deref() == Some("/streams/entityOrderStream/entryOffset/0")
        }));

        let mut absent = source.value().clone();
        absent["streamDirectory"]["streams"]
            .as_array_mut()
            .unwrap()
            .retain(|entry| {
                !matches!(
                    entry["name"].as_str(),
                    Some("entityOrder" | "entityOrderEntry")
                )
            });
        absent["streams"]
            .as_object_mut()
            .unwrap()
            .remove("entityOrderStream");
        absent["streams"]
            .as_object_mut()
            .unwrap()
            .remove("entityOrderEntryStream");
        let absent = validate_value("drawing.ifcdr.json", absent);
        assert!(absent.diagnostics().iter().any(|item| {
            item.code == "IFCCAD_IFCDR_ENTITY_ORDER_INVALID"
                && item.location.as_deref() == Some("/streams/entityOrderStream")
        }));
    }
}
