use super::SourceStructureProblem;
use cadcodec::{CadDocument, EntityType, Handle};
use std::borrow::Cow;

pub(crate) fn with_recovered_anonymous_block_name(document: &CadDocument) -> Cow<'_, CadDocument> {
    let mut candidate = None;
    for record in document.block_records.iter() {
        let Some(EntityType::Block(marker)) = document.get_entity(record.block_entity_handle)
        else {
            continue;
        };
        if marker.name == record.name {
            continue;
        }
        let Some(ordinal) = record
            .name
            .strip_prefix("*U")
            .and_then(|suffix| suffix.parse::<u32>().ok())
        else {
            return Cow::Borrowed(document);
        };
        if !record.flags.anonymous
            || record.flags.is_xref
            || record.flags.is_xref_overlay
            || record.flags.is_external
            || record.flags.is_xref_unloaded
            || !record.xref_path.is_empty()
            || format!("*U{ordinal}") != record.name
            || ordinal
                .checked_add(1)
                .is_none_or(|next| marker.name != format!("*U{next}"))
            || marker.common.handle != record.block_entity_handle
            || marker.common.owner_handle != record.handle
            || marker.base_point != record.base_point
            || document.block_records.contains(&marker.name)
            || candidate.is_some()
        {
            return Cow::Borrowed(document);
        }
        candidate = Some((record.name.as_str(), marker.name.as_str()));
    }
    let Some((old_name, new_name)) = candidate else {
        return Cow::Borrowed(document);
    };
    if document.entities().any(|entity| {
        matches!(entity, EntityType::Insert(insert) if insert.block_name.eq_ignore_ascii_case(new_name))
    }) {
        return Cow::Borrowed(document);
    }
    let insert_handles: Vec<_> = document
        .entities()
        .filter_map(|entity| match entity {
            EntityType::Insert(insert) if insert.block_name.eq_ignore_ascii_case(old_name) => {
                Some(insert.common.handle)
            }
            _ => None,
        })
        .collect();
    let mut recovered = document.clone();
    if recovered.block_records.rename(old_name, new_name).is_err() {
        return Cow::Borrowed(document);
    }
    for handle in insert_handles {
        let Some(EntityType::Insert(insert)) = recovered.get_entity_mut(handle) else {
            return Cow::Borrowed(document);
        };
        insert.block_name = new_name.to_owned();
    }
    Cow::Owned(recovered)
}

pub(crate) fn inspect_markers(document: &CadDocument) -> Vec<SourceStructureProblem> {
    let mut problems = Vec::new();
    for record in document.block_records.iter() {
        let Some(entity) = document.get_entity(record.block_entity_handle) else {
            continue;
        };
        let EntityType::Block(marker) = entity else {
            problems.push(SourceStructureProblem::InconsistentRelationship {
                description: format!(
                    "Block {:?} references a non-BLOCK begin marker",
                    record.name
                ),
            });
            continue;
        };
        if marker.common.owner_handle != record.handle || marker.name != record.name {
            problems.push(SourceStructureProblem::InconsistentRelationship {
                description: format!(
                    "Block {:?} begin marker has conflicting name or ownership",
                    record.name
                ),
            });
        }
        if marker.base_point != record.base_point {
            problems.push(SourceStructureProblem::InconsistentRelationship {
                description: format!(
                    "Block {:?} has conflicting base points: record {:?}, marker {:?}. Known DWG reader limitation: https://github.com/HakanSeven12/cadcodec/issues/52; no automatic correction applied",
                    record.name, record.base_point, marker.base_point,
                ),
            });
        }
    }
    problems
}

pub(crate) fn inspect_references(document: &CadDocument) -> Vec<SourceStructureProblem> {
    use std::collections::BTreeMap;
    let mut problems = Vec::new();
    let records: BTreeMap<_, _> = document
        .block_records
        .iter()
        .map(|record| (record.handle, record))
        .collect();
    if records.len() != document.block_records.len() || records.contains_key(&Handle::NULL) {
        problems.push(SourceStructureProblem::InconsistentRelationship {
            description:
                "Block records must have distinct, non-null handles, including layout records"
                    .into(),
        });
        return problems;
    }
    let mut membership = BTreeMap::<Handle, usize>::new();
    for record in records.values() {
        for &handle in &record.entity_handles {
            *membership.entry(handle).or_default() += 1;
            match document.get_entity(handle) {
                None => problems.push(SourceStructureProblem::InconsistentRelationship {
                    description: format!(
                        "Block {:?} owns a missing entity {:?}",
                        record.name, handle
                    ),
                }),
                Some(entity)
                    if records.contains_key(&entity.common().owner_handle)
                        && entity.common().owner_handle != record.handle =>
                {
                    problems.push(SourceStructureProblem::InconsistentRelationship {
                        description: format!(
                            "Block {:?} lists entity {:?} owned by another record",
                            record.name, handle
                        ),
                    })
                }
                _ => {}
            }
        }
    }
    for entity in document.entities() {
        if matches!(
            entity,
            EntityType::Block(_) | EntityType::BlockEnd(_) | EntityType::AttributeEntity(_)
        ) {
            continue;
        }
        if records.contains_key(&entity.common().owner_handle)
            && membership.get(&entity.common().handle) != Some(&1)
        {
            problems.push(SourceStructureProblem::InconsistentRelationship {
                description: format!(
                    "Entity {:?} must occur exactly once in its owner's block contents",
                    entity.common().handle
                ),
            });
        }
    }
    let mut pending: BTreeMap<Handle, usize> = document
        .block_records
        .iter()
        .map(|record| (record.handle, 0))
        .collect();
    let mut parents: BTreeMap<Handle, Vec<Handle>> = BTreeMap::new();
    for entity in document.entities() {
        let EntityType::Insert(insert) = entity else {
            continue;
        };
        let Some(target) = document.block_records.get(&insert.block_name) else {
            problems.push(SourceStructureProblem::InconsistentRelationship {
                description: format!(
                    "INSERT {:?} references missing block {:?}",
                    insert.common.handle, insert.block_name
                ),
            });
            continue;
        };
        if let Some(count) = pending.get_mut(&insert.common.owner_handle) {
            *count += 1;
            parents
                .entry(target.handle)
                .or_default()
                .push(insert.common.owner_handle);
        }
    }
    let mut ready: Vec<_> = pending
        .iter()
        .filter_map(|(&handle, &count)| (count == 0).then_some(handle))
        .collect();
    let mut visited = 0;
    while let Some(handle) = ready.pop() {
        visited += 1;
        for owner in parents.get(&handle).into_iter().flatten() {
            let count = pending.get_mut(owner).expect("recorded owner");
            *count -= 1;
            if *count == 0 {
                ready.push(*owner);
            }
        }
    }
    if visited != pending.len() {
        problems.push(SourceStructureProblem::InconsistentRelationship {
            description: "Block reference graph contains a cycle (including unused definitions)"
                .into(),
        });
    }
    problems
}

/// Preserve inter-owner encounter order while ordering each owner's entities by
/// its explicit member list, rather than the document's flat storage indices.
pub(crate) fn ordered_entities(document: &CadDocument) -> Vec<&EntityType> {
    let mut members: std::collections::BTreeMap<_, _> = document
        .block_records
        .iter()
        .map(|r| (r.handle, r.entity_handles.iter()))
        .collect();
    document
        .entities()
        .map(|entity| {
            if matches!(
                entity,
                EntityType::Block(_) | EntityType::BlockEnd(_) | EntityType::AttributeEntity(_)
            ) {
                return entity;
            }
            members
                .get_mut(&entity.common().owner_handle)
                .and_then(Iterator::next)
                .and_then(|&handle| document.get_entity(handle))
                .unwrap_or(entity)
        })
        .collect()
}
