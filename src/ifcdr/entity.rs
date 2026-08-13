use super::registry::{IfcdrRegistry, StreamRole};
use super::resource::ValidatedStream;
use crate::ifcdr::resource::{LoadedIfcdrResource, ValidatedIfcdrResource};
use crate::ifcdr::streams::{Line, PolylineRef};
use crate::package::codes::{
    IFCCAD_IFCDR_ENTITY_ID_DUPLICATE, IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
    IFCCAD_IFCDR_STRUCTURE_INVALID,
};
use crate::package::{PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity};
use crate::validated::Validated;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct EntityId(NonZeroU64);

impl EntityId {
    pub(crate) fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }

    pub(crate) fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ScopeId(u32);

impl ScopeId {
    pub(crate) fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EntityLocation {
    pub(crate) stream_name: String,
    pub(crate) schema_id: String,
    pub(crate) row_index: usize,
    pub(crate) scope_id: ScopeId,
}

#[derive(Debug, Default)]
pub(crate) struct EntityIndex {
    pub(crate) by_id: BTreeMap<EntityId, EntityLocation>,
    order_by_scope: BTreeMap<ScopeId, Vec<EntityId>>,
}

impl EntityIndex {
    pub(crate) fn get(&self, id: EntityId) -> Option<&EntityLocation> {
        self.by_id.get(&id)
    }

    pub(crate) fn in_scope(
        &self,
        scope: ScopeId,
    ) -> Option<impl ExactSizeIterator<Item = EntityId> + '_> {
        self.order_by_scope
            .get(&scope)
            .map(|items| items.iter().copied())
    }

    fn order_slice(&self, scope: ScopeId) -> Option<&[EntityId]> {
        self.order_by_scope.get(&scope).map(Vec::as_slice)
    }
}

pub(crate) struct IfcdrEntities<'a> {
    resource: &'a ValidatedIfcdrResource,
}

impl<'a> IfcdrEntities<'a> {
    pub(crate) fn in_scope(&self, scope: ScopeId) -> Option<EntityIterator<'a>> {
        self.resource
            .evidence()
            .entities
            .order_slice(scope)
            .map(|ids| EntityIterator {
                resource: self.resource,
                ids: ids.iter(),
            })
    }
}

pub(crate) struct EntityIterator<'a> {
    resource: &'a ValidatedIfcdrResource,
    ids: std::slice::Iter<'a, EntityId>,
}

impl<'a> Iterator for EntityIterator<'a> {
    type Item = IfcdrEntityRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = *self.ids.next()?;
        let location = self
            .resource
            .evidence()
            .entities
            .get(id)
            .expect("ordered entity has a validated location");
        Some(match location.stream_name.as_str() {
            "line" => IfcdrEntityRef::Line(
                self.resource
                    .streams()
                    .lines()
                    .expect("validated line stream")
                    .get(location.row_index)
                    .expect("validated line row"),
            ),
            "polyline" => IfcdrEntityRef::Polyline(
                self.resource
                    .streams()
                    .polylines()
                    .expect("validated polyline stream")
                    .get(location.row_index)
                    .expect("validated polyline row"),
            ),
            _ => IfcdrEntityRef::Unmodeled(UnmodeledEntityRef {
                entity_id: id,
                location,
            }),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}

impl ExactSizeIterator for EntityIterator<'_> {}

pub(crate) enum IfcdrEntityRef<'a> {
    Line(Line),
    Polyline(PolylineRef<'a>),
    Unmodeled(UnmodeledEntityRef<'a>),
}

pub(crate) struct UnmodeledEntityRef<'a> {
    entity_id: EntityId,
    location: &'a EntityLocation,
}

impl UnmodeledEntityRef<'_> {
    pub(crate) fn entity_id(&self) -> EntityId {
        self.entity_id
    }
    pub(crate) fn scope_id(&self) -> ScopeId {
        self.location.scope_id
    }
    pub(crate) fn stream_name(&self) -> &str {
        &self.location.stream_name
    }
    pub(crate) fn schema_id(&self) -> &str {
        &self.location.schema_id
    }
    pub(crate) fn row_index(&self) -> usize {
        self.location.row_index
    }
}

impl Validated<LoadedIfcdrResource> {
    pub(crate) fn entities(&self) -> IfcdrEntities<'_> {
        IfcdrEntities { resource: self }
    }
}

pub(super) fn validate_entities(
    uri: &str,
    root: &Map<String, Value>,
    registry: &IfcdrRegistry,
    streams: &BTreeMap<String, ValidatedStream>,
    next_entity_id: u64,
) -> (EntityIndex, Vec<PackageDiagnostic>) {
    let mut diagnostics = Vec::new();
    let mut index = EntityIndex::default();
    let Some(payloads) = root.get("streams").and_then(Value::as_object) else {
        return (index, diagnostics);
    };
    let scope_ids = root
        .get("scopeTable")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("id").and_then(Value::as_u64))
        .filter_map(|id| u32::try_from(id).ok())
        .map(ScopeId::new)
        .collect::<BTreeSet<_>>();
    let mut directly_ordered = BTreeMap::<ScopeId, BTreeSet<EntityId>>::new();

    for stream in registry
        .streams()
        .iter()
        .filter(|stream| stream.role() == StreamRole::Object && streams.contains_key(stream.name()))
    {
        let Some(payload) = payloads
            .get(stream.payload_key())
            .and_then(Value::as_object)
        else {
            continue;
        };
        let Some(ids) = payload.get("entityId").and_then(Value::as_array) else {
            continue;
        };
        let Some(scopes) = payload.get("scopeId").and_then(Value::as_array) else {
            continue;
        };
        for row in 0..ids.len().min(scopes.len()) {
            let id_pointer = format!("/streams/{}/entityId/{row}", escape(stream.payload_key()));
            let Some(raw_id) = ids[row].as_u64() else {
                continue;
            };
            let Some(id) = EntityId::new(raw_id) else {
                diagnostics.push(diagnostic(
                    uri,
                    IFCCAD_IFCDR_STRUCTURE_INVALID,
                    &id_pointer,
                    "entity ID must be greater than zero",
                    BTreeMap::new(),
                ));
                continue;
            };
            let Some(scope) = scopes[row]
                .as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .map(ScopeId::new)
            else {
                continue;
            };
            if !scope_ids.contains(&scope) {
                diagnostics.push(diagnostic(
                    uri,
                    IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                    &format!("/streams/{}/scopeId/{row}", escape(stream.payload_key())),
                    "entity scope does not exist",
                    BTreeMap::new(),
                ));
            }
            let location = EntityLocation {
                stream_name: stream.name().to_owned(),
                schema_id: stream.schema_id().to_owned(),
                row_index: row,
                scope_id: scope,
            };
            if let Some(first) = index.by_id.insert(id, location) {
                diagnostics.push(diagnostic(
                    uri,
                    IFCCAD_IFCDR_ENTITY_ID_DUPLICATE,
                    &id_pointer,
                    "entity ID occurs in more than one object row",
                    BTreeMap::from([
                        (
                            "entityId".to_owned(),
                            PackageDiagnosticContextValue::Number(raw_id.into()),
                        ),
                        (
                            "firstStream".to_owned(),
                            PackageDiagnosticContextValue::String(first.stream_name),
                        ),
                    ]),
                ));
            }
            let structural_text = stream.name() == "text"
                && payload
                    .get("ownerKind")
                    .and_then(Value::as_array)
                    .and_then(|items| items.get(row))
                    .and_then(Value::as_u64)
                    .is_some_and(|owner_kind| owner_kind != 0);
            if !structural_text {
                directly_ordered.entry(scope).or_default().insert(id);
            }
        }
    }

    if index
        .by_id
        .keys()
        .next_back()
        .is_some_and(|largest| next_entity_id <= largest.get())
    {
        diagnostics.push(diagnostic(
            uri,
            IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
            "/header/nextEntityId",
            "nextEntityId must be greater than every entity ID",
            BTreeMap::new(),
        ));
    }

    let order_rows = payloads.get("entityOrderStream").and_then(Value::as_object);
    let order_entries = payloads
        .get("entityOrderEntryStream")
        .and_then(Value::as_object)
        .and_then(|payload| payload.get("entityId"))
        .and_then(Value::as_array);
    if let (Some(rows), Some(entries)) = (order_rows, order_entries) {
        let scopes = rows.get("scopeId").and_then(Value::as_array);
        let offsets = rows.get("entryOffset").and_then(Value::as_array);
        let counts = rows.get("entryCount").and_then(Value::as_array);
        if let (Some(scopes), Some(offsets), Some(counts)) = (scopes, offsets, counts) {
            let mut seen_scopes = BTreeSet::new();
            for row in 0..scopes.len().min(offsets.len()).min(counts.len()) {
                let Some(scope) = scopes[row]
                    .as_u64()
                    .and_then(|value| u32::try_from(value).ok())
                    .map(ScopeId::new)
                else {
                    continue;
                };
                if !seen_scopes.insert(scope) {
                    diagnostics.push(diagnostic(
                        uri,
                        IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                        &format!("/streams/entityOrderStream/scopeId/{row}"),
                        "scope has more than one entity-order row",
                        BTreeMap::new(),
                    ));
                    continue;
                }
                let Some(start) = offsets[row].as_u64().and_then(|v| usize::try_from(v).ok())
                else {
                    continue;
                };
                let Some(count) = counts[row].as_u64().and_then(|v| usize::try_from(v).ok()) else {
                    continue;
                };
                let Some(end) = start.checked_add(count) else {
                    continue;
                };
                if end > entries.len() {
                    diagnostics.push(diagnostic(
                        uri,
                        IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                        &format!("/streams/entityOrderStream/entryCount/{row}"),
                        "entity-order range exceeds its entry stream",
                        BTreeMap::new(),
                    ));
                    continue;
                }
                let mut ordered = Vec::with_capacity(count);
                let mut seen_entities = BTreeSet::new();
                for (entry_index, entry_value) in entries.iter().enumerate().take(end).skip(start) {
                    let pointer = format!("/streams/entityOrderEntryStream/entityId/{entry_index}");
                    let Some(id) = entry_value.as_u64().and_then(EntityId::new) else {
                        continue;
                    };
                    let valid_location = index.get(id);
                    if valid_location.is_none() {
                        diagnostics.push(diagnostic(
                            uri,
                            IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                            &pointer,
                            "entity-order entry references a missing entity",
                            BTreeMap::new(),
                        ));
                        continue;
                    }
                    let location = valid_location.expect("checked entity location");
                    if location.scope_id != scope {
                        diagnostics.push(diagnostic(
                            uri,
                            IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                            &pointer,
                            "entity-order entry belongs to another scope",
                            BTreeMap::new(),
                        ));
                    }
                    if !seen_entities.insert(id) {
                        diagnostics.push(diagnostic(
                            uri,
                            IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                            &pointer,
                            "entity occurs more than once in scope order",
                            BTreeMap::new(),
                        ));
                    }
                    if !directly_ordered
                        .get(&scope)
                        .is_some_and(|set| set.contains(&id))
                    {
                        diagnostics.push(diagnostic(
                            uri,
                            IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                            &pointer,
                            "structural child cannot occur directly in scope order",
                            BTreeMap::new(),
                        ));
                    }
                    ordered.push(id);
                }
                if directly_ordered.get(&scope).cloned().unwrap_or_default() != seen_entities {
                    diagnostics.push(diagnostic(
                        uri,
                        IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                        &format!("/streams/entityOrderStream/entryCount/{row}"),
                        "scope order does not cover every directly drawable entity exactly once",
                        BTreeMap::new(),
                    ));
                }
                index.order_by_scope.insert(scope, ordered);
            }
            for scope in scope_ids {
                if !seen_scopes.contains(&scope) {
                    diagnostics.push(diagnostic(
                        uri,
                        IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                        "/streams/entityOrderStream/scopeId",
                        "scope is missing an entity-order row",
                        BTreeMap::new(),
                    ));
                }
            }
        }
    }
    (index, diagnostics)
}

fn diagnostic(
    uri: &str,
    code: &str,
    location: &str,
    message: &str,
    context: BTreeMap<String, PackageDiagnosticContextValue>,
) -> PackageDiagnostic {
    PackageDiagnostic {
        code: code.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_uri: Some(uri.to_owned()),
        location: Some(location.to_owned()),
        context,
        message: message.to_owned(),
    }
}

fn escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::{EntityId, IfcdrEntityRef, ScopeId};
    use crate::ifcdr::resource::fixture_source;
    use crate::ifcdr::resource::LoadedIfcdrResource;
    use crate::ifcdr::validation::{validate_ifcdr, validate_value};

    #[test]
    fn indexes_all_minimal_entities_in_scope_order() {
        let outcome = validate_ifcdr(LoadedIfcdrResource::new(
            "drawing.ifcdr.json".to_owned(),
            fixture_source(),
        ));
        let validated = outcome.validated().expect("valid IFCDR");
        let evidence = validated.evidence();

        assert_eq!(
            evidence
                .entities
                .in_scope(ScopeId::new(0))
                .unwrap()
                .map(EntityId::get)
                .collect::<Vec<_>>(),
            [1, 2, 3, 4]
        );
        assert!(evidence.entities.in_scope(ScopeId::new(99)).is_none());
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
                IfcdrEntityRef::Unmodeled(item) => ("unmodeled", item.entity_id().get()),
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
            item.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
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
}
