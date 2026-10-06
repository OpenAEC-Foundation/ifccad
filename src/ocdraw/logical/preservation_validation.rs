use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn valid_target(target: OcdrawPreservationTarget) -> bool {
    match target {
        OcdrawPreservationTarget::Entity(id) => id > 0,
        OcdrawPreservationTarget::Record(id) => id.0 > 0,
        _ => true,
    }
}

pub(crate) fn validate_preservation(doc: &OcdrawDocument) -> Vec<LogicalError> {
    let mut errors = Vec::new();
    let mut check = |valid: bool, code, location: String, message| {
        if !valid {
            errors.push(super::field_validation::logical_error(
                code, location, message,
            ));
        }
    };
    let Some(p) = &doc.preservation else {
        check(
            doc.opaque_entities.is_empty(),
            "PRESERVATION_LINK",
            "/opaqueEntities".into(),
            "live opaque entities require preservation records",
        );
        return errors;
    };
    check(
        p.version == 1,
        "PRESERVATION_VERSION",
        "/preservation/version".into(),
        "unsupported preservation envelope version",
    );
    let mut sources = BTreeSet::new();
    for (i, source) in p.sources.iter().enumerate() {
        check(
            !source.id.is_empty() && sources.insert(source.id.as_str()),
            "PRESERVATION_SOURCE",
            format!("/preservation/sources/{i}/id"),
            "source IDs must be nonempty and unique",
        );
        check(
            !source.provider.is_empty()
                && !source.provider_revision.is_empty()
                && source.source_version.as_ref().is_none_or(|v| !v.is_empty()),
            "PRESERVATION_FIELD",
            format!("/preservation/sources/{i}"),
            "provider identity and present source version must be nonempty",
        );
    }
    let mut ids = BTreeSet::new();
    let mut records = BTreeMap::new();
    check(
        p.next_record_id > 0,
        "PRESERVATION_ID",
        "/preservation/nextRecordId".into(),
        "record watermark must be nonzero",
    );
    for (i, record) in p.records.iter().enumerate() {
        let path = format!("/preservation/records/{i}");
        check(
            record.id.0 > 0 && ids.insert(record.id) && record.id.0 < p.next_record_id,
            "PRESERVATION_ID",
            format!("{path}/id"),
            "record IDs must be nonzero, unique and below the allocation watermark",
        );
        records.insert(record.id, record);
        check(
            sources.contains(record.source_id.as_str()),
            "PRESERVATION_SOURCE",
            format!("{path}/sourceId"),
            "source reference does not resolve",
        );
        check(
            !record.source_key.is_empty()
                && !record.payload.schema.is_empty()
                && record.payload.version > 0
                && record.subject.is_none_or(valid_target),
            "PRESERVATION_FIELD",
            path.clone(),
            "invalid record identity, subject or payload schema",
        );
        let mut slots = BTreeSet::new();
        for (j, binding) in record.bindings.iter().enumerate() {
            check(!binding.slot.is_empty() && slots.insert(binding.slot.as_str()) && !binding.source_key.is_empty()
                && valid_target(binding.target), "PRESERVATION_FIELD", format!("{path}/bindings/{j}"), "binding slots must be unique and nonempty with a valid source key and target domain");
        }
        for (j, condition) in record.conditions.iter().enumerate() {
            check(
                !condition.predicate.is_empty()
                    && condition.version > 0
                    && valid_target(condition.target),
                "PRESERVATION_FIELD",
                format!("{path}/conditions/{j}"),
                "invalid predicate identity, version or target domain",
            );
        }
    }
    let mut live_records = BTreeSet::new();
    for (i, entity) in doc.opaque_entities.iter().enumerate() {
        let record = records.get(&entity.preservation_record_id);
        check(live_records.insert(entity.preservation_record_id) && record.is_some_and(|record|
            record.category == OcdrawPreservationCategory::Entity
                && record.role == OcdrawPreservationRole::Complete
                && record.subject == Some(OcdrawPreservationTarget::Entity(entity.id))),
            "PRESERVATION_LINK", format!("/opaqueEntities/{i}/preservationRecordId"),
            "live opaque entity must have a unique complete entity record with the same entity subject");
    }
    errors
}
