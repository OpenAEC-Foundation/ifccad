use super::conversion::ExportContext;
use super::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportError, ExportLossReason,
    SourceStructureProblem,
};
use cadcodec::objects::ObjectType;
use cadcodec::{CadDocument, EntityType, Handle};
use ifccad::ifcdr::{BlockScaling, Point3};

pub(crate) struct ConvertedInstance {
    pub definition: Handle,
    pub source: crate::geometry::blocks::EvaluatedBlock,
    pub target: crate::geometry::blocks::EvaluatedBlock,
}

pub(crate) fn propagate_losses(document: &CadDocument, context: &mut ExportContext) {
    use std::collections::{BTreeMap, BTreeSet};
    let mut affected: BTreeMap<Handle, BTreeSet<Handle>> = BTreeMap::new();
    for diagnostic in &context.diagnostics {
        if !diagnostic.blocks_reject() {
            continue;
        }
        let owner = match diagnostic.source() {
            ExportDiagnosticSource::Entity { handle, .. } => document
                .get_entity(*handle)
                .map(|entity| entity.common().owner_handle),
            ExportDiagnosticSource::Object { handle, kind } if kind == "BLOCK_RECORD" => {
                Some(*handle)
            }
            _ => None,
        };
        if let Some(owner) = owner.filter(|owner| context.blocks.contains_key(owner)) {
            affected.entry(owner).or_default();
        }
    }
    for (&handle, root) in &context.block_instances {
        let mut todo = vec![root.definition];
        let mut visited = BTreeSet::new();
        while let Some(definition) = todo.pop() {
            if !visited.insert(definition) {
                continue;
            }
            if let Some(instances) = affected.get_mut(&definition) {
                instances.insert(handle);
            }
            if let Some(record) = document
                .block_records
                .iter()
                .find(|r| r.handle == definition)
            {
                for nested in &record.entity_handles {
                    if let Some(instance) = context.block_instances.get(nested) {
                        todo.push(instance.definition);
                    }
                }
            }
        }
    }
    for (definition, instances) in affected {
        if instances.is_empty() {
            continue;
        }
        context.diagnostics.push(ExportDiagnostic::loss(
            ExportDiagnosticSource::Object {
                handle: definition,
                kind: "BLOCK_RECORD".into(),
            },
            ExportAction::PartiallyExported,
            vec![ExportLossReason::BlockContentLoss {
                definition,
                affected_instances: instances.into_iter().collect(),
            }],
        ));
    }
}

pub(crate) fn assess_occurrences(
    document: &CadDocument,
    context: &mut ExportContext,
) -> Result<(), ExportError> {
    use crate::ConversionEntitySource;
    // Every stored occurrence is checked in its own owning coordinate scope,
    // including occurrences in unused definitions. The source DAG was checked.
    for (&root, instance) in &context.block_instances {
        let mut stack = vec![(instance.definition, vec![root])];
        while let Some((definition, path)) = stack.pop() {
            let record = document
                .block_records
                .iter()
                .find(|record| record.handle == definition)
                .expect("checked definition");
            for &handle in &record.entity_handles {
                if let Some(nested) = context.block_instances.get(&handle) {
                    let mut nested_path = path.clone();
                    nested_path.push(handle);
                    stack.push((nested.definition, nested_path));
                } else if let Some(points) = context.block_points.get(&handle) {
                    let entity = document.get_entity(handle).expect("converted primitive");
                    let source = ConversionEntitySource::BlockOccurrence {
                        path: path
                            .iter()
                            .map(|&handle| ConversionEntitySource::CadEntity {
                                handle,
                                kind: "INSERT".into(),
                            })
                            .collect(),
                        leaf: Box::new(ConversionEntitySource::CadEntity {
                            handle,
                            kind: entity.as_entity().entity_type().into(),
                        }),
                    };
                    let mut maximum = 0_f64;
                    for (index, point) in points.iter().enumerate() {
                        let mut point = point.clone();
                        for handle in path.iter().rev() {
                            let pair = &context.block_instances[handle];
                            point.apply(&pair.source, &pair.target);
                        }
                        let (lower, upper) = point.squared_deviation();
                        maximum = maximum.max(
                            context
                                .geometry
                                .as_ref()
                                .unwrap()
                                .check_interval(&source, index, &lower, &upper)?,
                        );
                    }
                    context
                        .geometry
                        .as_mut()
                        .unwrap()
                        .record(source, points.len(), maximum);
                    if maximum > 0. {
                        context.diagnostics.push(ExportDiagnostic::loss(
                            ExportDiagnosticSource::Entity {
                                handle: root,
                                kind: "INSERT".into(),
                            },
                            ExportAction::PartiallyExported,
                            vec![ExportLossReason::GeometryRoundedWithinTolerance {
                                max_deviation_upper_bound: maximum,
                            }],
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
use ifccad::package::{BlockDefinitionOptions, DrawingBuilder};

/// Allocate definitions before processing any contents or INSERT references.
pub(crate) fn add_definitions(
    document: &CadDocument,
    drawing: &mut DrawingBuilder<'_>,
    context: &mut ExportContext,
) -> Result<(), ExportError> {
    let layout_owners: std::collections::BTreeSet<_> = document
        .objects
        .values()
        .filter_map(|object| match object {
            ObjectType::Layout(layout) => Some(layout.block_record),
            _ => None,
        })
        .collect();
    for record in document.block_records.iter() {
        if record.handle == document.header.model_space_block_handle
            || record.handle == document.header.paper_space_block_handle
            || layout_owners.contains(&record.handle)
            || record.layout != Handle::NULL
        {
            continue;
        }
        // Unsupported external definitions are not reinterpreted as local ones.
        if record.flags.is_xref
            || record.flags.is_xref_overlay
            || record.flags.is_external
            || record.flags.is_xref_unloaded
            || !record.xref_path.is_empty()
        {
            continue;
        }
        if record.handle == Handle::NULL || context.blocks.contains_key(&record.handle) {
            return Err(ExportError::InvalidSourceStructure {
                problems: vec![SourceStructureProblem::InconsistentRelationship {
                    description: format!(
                        "Block {:?} has a missing or duplicate record handle",
                        record.name
                    ),
                }],
            });
        }
        let (unit, unit_loss) = super::units::map_length_unit(record.units);
        let key = drawing.add_block_definition(BlockDefinitionOptions {
            name: record.name.clone(),
            base_point: Point3::new(
                record.base_point.x,
                record.base_point.y,
                record.base_point.z,
            ),
            description: record.description.clone(),
            anonymous: record.flags.anonymous,
            insertion_unit: unit,
            explodable: record.explodable,
            scaling: if record.scale_uniformly {
                BlockScaling::Uniform
            } else {
                BlockScaling::Any
            },
        })?;
        context.blocks.insert(record.handle, key);
        let mut losses: Vec<_> = unit_loss.into_iter().collect();
        if record.flags.has_attributes
            || !record.preview_data.is_empty()
            || !record.insert_count_bytes.is_empty()
        {
            losses.push(ExportLossReason::UnsupportedSemantic {
                name: "block attribute flag, preview or insertion-count metadata".into(),
            });
        }
        if !losses.is_empty() {
            context.diagnostics.push(ExportDiagnostic::loss(
                ExportDiagnosticSource::Object {
                    handle: record.handle,
                    kind: "BLOCK_RECORD".into(),
                },
                ExportAction::PartiallyExported,
                losses,
            ));
        }
    }
    Ok(())
}

/// Compare only metadata actually exposed by both public representations.
/// An absent marker is normal for DXF and for programmatically built records.
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
