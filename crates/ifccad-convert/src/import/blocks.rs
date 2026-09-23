use crate::ImportError;
use cadcodec::entities::{Block, BlockEnd};
use cadcodec::{BlockRecord, CadDocument, EntityType, Handle};
use ifccad::ifcdr::{BlockScaling, IfcdrResourceRef, ScopeId, ScopeRef};
use std::collections::BTreeMap;

pub(crate) struct ConvertedInstance {
    pub definition: ScopeId,
    pub owner_scope: ScopeId,
    pub source: crate::geometry::blocks::EvaluatedBlock,
    pub target: crate::geometry::blocks::EvaluatedBlock,
}

pub(crate) fn assess_occurrences(
    resource: IfcdrResourceRef,
    instances: &BTreeMap<ifccad::ifcdr::EntityId, ConvertedInstance>,
    points: &BTreeMap<ifccad::ifcdr::EntityId, Vec<crate::geometry::blocks::PairedPoint>>,
    geometry: &mut crate::ConversionGeometryAssessment,
    diagnostics: &mut super::diagnostic::DiagnosticAccumulator,
) -> Result<(), ImportError> {
    use crate::ConversionEntitySource;
    use ifccad::ifcdr::IfcdrEntityRef;
    for (&root, instance) in instances {
        let mut stack = vec![(instance.definition, vec![root])];
        while let Some((scope, path)) = stack.pop() {
            for entity in resource.entities(scope) {
                let id = match entity {
                    IfcdrEntityRef::Line(e) => e.entity_id(),
                    IfcdrEntityRef::Polyline(e) => e.entity_id(),
                    IfcdrEntityRef::BlockInstance(e) => e.entity_id(),
                    IfcdrEntityRef::Viewport(_) => {
                        unreachable!("validated block scope has no viewport")
                    }
                };
                if let Some(nested) = instances.get(&id) {
                    let mut path = path.clone();
                    path.push(id);
                    stack.push((nested.definition, path));
                } else if let Some(points) = points.get(&id) {
                    let source = ConversionEntitySource::BlockOccurrence {
                        path: path
                            .iter()
                            .map(|id| ConversionEntitySource::IfcdrEntity {
                                resource_id: resource.resource_id().clone(),
                                scope_id: instances[id].owner_scope,
                                entity_id: *id,
                            })
                            .collect(),
                        leaf: Box::new(ConversionEntitySource::IfcdrEntity {
                            resource_id: resource.resource_id().clone(),
                            scope_id: scope,
                            entity_id: id,
                        }),
                    };
                    let mut maximum = 0_f64;
                    for (index, point) in points.iter().enumerate() {
                        let mut point = point.clone();
                        for id in path.iter().rev() {
                            let pair = &instances[id];
                            point.apply(&pair.source, &pair.target);
                        }
                        let (lower, upper) = point.squared_deviation();
                        maximum =
                            maximum.max(geometry.check_interval(&source, index, &lower, &upper)?);
                    }
                    geometry.record(source.clone(), points.len(), maximum);
                    if maximum > 0. {
                        diagnostics.record(
                            crate::ImportDiagnostic::GeometryRoundedWithinTolerance {
                                source,
                                max_deviation_upper_bound: maximum,
                            },
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn allocate(
    document: &mut CadDocument,
    resource: IfcdrResourceRef<'_>,
    model: ScopeId,
) -> Result<BTreeMap<ScopeId, Handle>, ImportError> {
    let mut mapping = BTreeMap::from([(model, document.header.model_space_block_handle)]);
    for scope in resource.scopes() {
        let ScopeRef::BlockDefinition(definition) = scope else {
            continue;
        };
        let mut record = BlockRecord::new(definition.name());
        record.handle = document.allocate_handle();
        record.block_entity_handle = document.allocate_handle();
        record.block_end_handle = document.allocate_handle();
        let base = definition.base_point();
        record.base_point = cadcodec::Vector3::new(base.x(), base.y(), base.z());
        record.description = definition.description().into();
        record.flags.anonymous = definition.anonymous();
        record.units = crate::units::cad_code(definition.insertion_unit());
        record.explodable = definition.explodable();
        record.scale_uniformly = definition.scaling() == BlockScaling::Uniform;
        let mut marker = Block::new(definition.name(), record.base_point);
        marker.description = record.description.clone();
        marker.common.handle = record.block_entity_handle;
        marker.common.owner_handle = record.handle;
        let mut end = BlockEnd::new();
        end.common.handle = record.block_end_handle;
        end.common.owner_handle = record.handle;
        let handle = record.handle;
        document.block_records.add(record).map_err(|message| {
            ImportError::BlockTargetLimitation {
                entity_id: None,
                message,
            }
        })?;
        for entity in [EntityType::Block(marker), EntityType::BlockEnd(end)] {
            document
                .add_entity(entity)
                .map_err(|error| ImportError::BlockTargetLimitation {
                    entity_id: None,
                    message: error.to_string(),
                })?;
        }
        mapping.insert(definition.scope_id(), handle);
    }
    Ok(mapping)
}
