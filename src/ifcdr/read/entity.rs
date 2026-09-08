use super::resource::{LoadedIfcdrResource, ValidatedIfcdrResource};
use super::streams::{Line, PolylineRef};
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
            IfcdrEntityKind::Line => IfcdrEntityRef::Line(
                self.resource
                    .streams()
                    .lines()
                    .unwrap()
                    .get(location.row)
                    .expect("validated line"),
            ),
            IfcdrEntityKind::Polyline => IfcdrEntityRef::Polyline(
                self.resource
                    .streams()
                    .polylines()
                    .unwrap()
                    .get(location.row)
                    .expect("validated polyline"),
            ),
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}
impl ExactSizeIterator for EntityIterator<'_> {}
pub enum IfcdrEntityRef<'a> {
    Line(Line),
    Polyline(PolylineRef<'a>),
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
                IfcdrEntityRef::Line(line) => ("line", line.entity_id().get()),
                IfcdrEntityRef::Polyline(polyline) => ("polyline", polyline.entity_id().get()),
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
        duplicate["streams"]["polylineStream"]["entityId"][0] = serde_json::json!(1);
        let duplicate = validate_value("drawing.ifcdr.json", duplicate);
        assert!(duplicate.diagnostics().iter().any(|item| {
            item.code == "IFCCAD_IFCDR_ENTITY_ID_DUPLICATE"
                && item.location.as_deref() == Some("/streams/polylineStream/entityId/0")
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
