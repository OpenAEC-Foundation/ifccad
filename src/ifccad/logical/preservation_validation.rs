use super::*;
use std::collections::BTreeSet;

pub(super) fn validate_preservation(d: &IfccadDocument) -> Result<(), IfccadReport> {
    let entities: Vec<_> = d
        .model
        .entities
        .iter()
        .chain(d.paper_layouts.iter().flat_map(|p| &p.entities))
        .chain(d.blocks.iter().flat_map(|b| &b.entities))
        .collect();
    let fail = |s: &str| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-PRESERVATION-002",
            &format!("/cad/d{}/preservation", d.drawing_id),
            s,
        )
    };
    let Some(p) = &d.preservation else {
        return if entities.iter().any(|e| e.as_opaque().is_some()) {
            Err(fail("opaque entity requires preservation"))
        } else {
            Ok(())
        };
    };
    if p.version != 1 {
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
    let mut records = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for r in &p.records {
        if r.id.0 == 0
            || !records.insert(r.id)
            || !sources.contains(&r.source_id)
            || r.source_key.is_empty()
            || !identities.insert((&r.source_id, &r.source_key))
            || r.payload.schema.is_empty()
            || r.payload.version == 0
        {
            return Err(fail("invalid record identity/source/payload"));
        }
        if let Some(target) = r.subject {
            let IfccadPreservationTarget::Entity(id) = target else {
                return Err(fail("live subject must be an entity"));
            };
            let e = entities
                .iter()
                .find(|e| e.id() == id)
                .and_then(|e| e.as_opaque());
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
    for e in entities.iter().filter_map(|e| e.as_opaque()) {
        if !p.records.iter().any(|r| {
            r.id == e.preservation_record_id
                && r.subject == Some(IfccadPreservationTarget::Entity(e.id))
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
