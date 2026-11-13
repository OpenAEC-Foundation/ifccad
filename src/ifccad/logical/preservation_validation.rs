use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate_preservation(d: &IfccadDocument) -> Result<(), IfccadReport> {
    let entities = d
        .model
        .entities
        .iter()
        .chain(d.paper_layouts.iter().flat_map(|p| &p.entities))
        .chain(d.blocks.iter().flat_map(|b| &b.entities))
        .filter_map(|entity| entity.as_opaque().map(|opaque| (opaque.id, opaque)))
        .collect::<BTreeMap<_, _>>();
    let fail = |s: &str| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-PRESERVATION-002",
            &format!("/cad/d{}/preservation", d.drawing_id),
            s,
        )
    };
    let Some(p) = &d.preservation else {
        return if !entities.is_empty() {
            Err(fail("opaque entity requires preservation"))
        } else {
            Ok(())
        };
    };
    if p.version != 2 {
        return Err(fail("unsupported envelope version"));
    }
    let mut sources = BTreeSet::new();
    for s in &p.sources {
        if s.id.is_empty()
            || s.provider.is_empty()
            || s.provider_revision.is_empty()
            || s.source_version.as_ref().is_some_and(String::is_empty)
            || !sources.insert(&s.id)
        {
            return Err(fail("invalid or duplicate source context"));
        }
    }
    let mut records = BTreeMap::new();
    let mut identities = BTreeSet::new();
    for r in &p.records {
        if r.id.0 == 0
            || records.insert(r.id, r).is_some()
            || !sources.contains(&r.source_id)
            || r.source_key.is_empty()
            || !identities.insert((&r.source_id, &r.source_key))
            || r.payload
                .as_ref()
                .is_some_and(|payload| payload.schema.is_empty() || payload.version == 0)
        {
            return Err(fail("invalid record identity/source/payload"));
        }
        if let Some(target) = r.subject {
            let IfccadPreservationTarget::Entity(id) = target else {
                return Err(fail("live subject must be an entity"));
            };
            let e = entities.get(&id);
            if !e.is_some_and(|e| e.preservation_record_id == r.id)
                || r.role != IfccadPreservationRole::Complete
                || r.category != IfccadPreservationCategory::Entity
            {
                return Err(fail(
                    "live subject must match a complete opaque entity record",
                ));
            }
        }
        let mut slots = BTreeSet::new();
        for b in &r.bindings {
            if b.slot.is_empty()
                || b.source_key.is_empty()
                || !slots.insert(&b.slot)
                || matches!(
                    b.target,
                    IfccadPreservationTarget::Record(IfccadPreservationRecordId(0))
                )
            {
                return Err(fail("invalid binding"));
            }
        }
        if r.conditions.iter().any(|c| {
            c.predicate.is_empty()
                || c.version == 0
                || matches!(
                    c.target,
                    IfccadPreservationTarget::Record(IfccadPreservationRecordId(0))
                )
        }) {
            return Err(fail("invalid condition"));
        }
    }
    for r in &p.records {
        let error = |message: &str| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-PRESERVATION-002",
                &format!("/cad/d{}/preservation/r{}", d.drawing_id, r.id.0),
                message,
            )
        };
        let shared = r.role == IfccadPreservationRole::Shared;
        if shared != (r.category == IfccadPreservationCategory::Shared)
            || (shared && (r.subject.is_some() || r.storage_backing.is_some()))
        {
            return Err(error("invalid shared storage record"));
        }
        if r.payload.is_some() == r.storage_backing.is_some() {
            return Err(error("record requires exactly one storage form"));
        }
        if let Some(id) = r.storage_backing {
            if r.subject.is_none()
                || !records.get(&id).is_some_and(|backing| {
                    backing.role == IfccadPreservationRole::Shared
                        && backing.source_id == r.source_id
                })
            {
                return Err(error(
                    "storage backing must resolve to a shared record in the same source",
                ));
            }
        }
    }
    for e in entities.values() {
        if !records.get(&e.preservation_record_id).is_some_and(|r| {
            r.subject == Some(IfccadPreservationTarget::Entity(e.id))
                && r.role == IfccadPreservationRole::Complete
                && r.category == IfccadPreservationCategory::Entity
        }) {
            return Err(fail("unresolved opaque record or subject"));
        }
    }
    // Bindings and predicate targets are soft links: unresolved targets and cycles
    // remain transportable; adapters evaluate their eligibility on each restore.
    Ok(())
}
