//! OCDraw record/identity adapter over shared primitive preparation and proofs.
use crate::{
    OcdrawGeometryEntitySource as Source, OcdrawGeometryFailure,
    OcdrawGeometryFailureReason as Reason, OcdrawGeometryStage as Stage,
};
use ocdraw::geometry_kernel::OwnedGeometry;
use ocdraw::ocdraw::{DrawingGeometry, LinePatternGeneration};
use opencadcodec::{EntityType, Handle};
use std::collections::BTreeMap;
pub(crate) type ExchangeState<K> = crate::geometry_context::GeometryContext<K>;

fn preparation_failure<K: Copy + Ord>(
    state: &ExchangeState<K>,
    identity: &Source,
    error: cad_geometry_convert::CadPreparationError,
    stage: Stage,
) -> Box<OcdrawGeometryFailure> {
    let (stage, reason) = match error {
        cad_geometry_convert::CadPreparationError::Numerical { stage, reason } => (stage, reason),
        cad_geometry_convert::CadPreparationError::OutOfRange => {
            (stage, Reason::TargetCoordinateOutOfRange)
        }
        _ => (stage, Reason::CadAxisEvaluationFailed),
    };
    state.failure(state.assessment.failure(identity, None, stage, reason))
}
fn generation(entity: &EntityType) -> LinePatternGeneration {
    let continuous = match entity {
        EntityType::LwPolyline(p) => p.plinegen,
        EntityType::Polyline2D(p) => p.flags.bits() & 128 != 0,
        EntityType::Polyline3D(p) => p.flags.to_bits() & 128 != 0,
        EntityType::Polyline(p) => p.flags.bits() & 128 != 0,
        _ => false,
    };
    if continuous {
        LinePatternGeneration::Continuous
    } else {
        LinePatternGeneration::PerSegment
    }
}
fn native(
    geometry: OwnedGeometry,
    line_pattern_generation: LinePatternGeneration,
) -> DrawingGeometry {
    match geometry {
        OwnedGeometry::Line { start, end } => DrawingGeometry::Line { start, end },
        OwnedGeometry::Point { placement } => DrawingGeometry::Point { placement },
        OwnedGeometry::Circle { placement, radius } => {
            DrawingGeometry::Circle { placement, radius }
        }
        OwnedGeometry::Arc {
            placement,
            radius,
            start,
            sweep,
        } => DrawingGeometry::Arc {
            placement,
            radius,
            start_parameter: start,
            sweep_parameter: sweep,
        },
        OwnedGeometry::Ellipse {
            placement,
            major,
            minor,
            arc,
        } => DrawingGeometry::Ellipse {
            placement,
            semi_major_radius: major,
            semi_minor_radius: minor,
            arc,
        },
        OwnedGeometry::PlanarPolyline {
            placement,
            vertices,
            closed,
        } => DrawingGeometry::PlanarPolyline {
            placement,
            vertices,
            closed,
            line_pattern_generation,
        },
        OwnedGeometry::SpatialPolyline { vertices, closed } => DrawingGeometry::SpatialPolyline {
            vertices,
            closed,
            line_pattern_generation,
        },
    }
}
pub(crate) fn from_cad(
    entity: &EntityType,
    definitions: &BTreeMap<Handle, u32>,
    document: &opencadcodec::CadDocument,
    state: &mut ExchangeState<Handle>,
) -> Result<(DrawingGeometry, f64, bool), Box<OcdrawGeometryFailure>> {
    let key = entity.common().handle;
    let identity = Source::CadEntity {
        handle: key,
        kind: entity.as_entity().entity_type().into(),
    };
    if let EntityType::Insert(insert) = entity {
        let record = document
            .block_records
            .get(&insert.block_name)
            .expect("validated block target");
        let (transform, source, target) =
            cad_geometry_convert::geometry::blocks::from_cad_instance(
                insert,
                record.base_point,
                &state.assessment,
            )
            .map_err(|f| state.failure(f))?;
        let normalized = cad_geometry_convert::geometry::stored_normal(transform.placement())
            != Some(insert.normal);
        state.register_identity(key, identity);
        state.record_instance_parts(key, record.handle, source, target);
        return Ok((
            DrawingGeometry::BlockInstance {
                definition_scope_id: definitions[&record.handle],
                transform,
            },
            0.,
            normalized,
        ));
    }
    let prepared = cad_geometry_convert::prepare_from_cad(entity)
        .map_err(|e| preparation_failure(state, &identity, e, Stage::SourceEvaluation))?;
    let maximum = state.record_geometry(key, identity, prepared.pair)?;
    Ok((
        native(prepared.geometry, generation(entity)),
        maximum,
        prepared.normal_normalized,
    ))
}
pub(crate) fn to_cad(
    source: &ocdraw::ocdraw::DrawingGeometricEntity,
    owner_scope_id: u32,
    blocks: &BTreeMap<u64, ocdraw::ocdraw::DrawingBlockDefinition>,
    state: &mut ExchangeState<u64>,
) -> Result<(EntityType, f64, bool), crate::OcdrawToCadError> {
    let key = source.id();
    let identity = Source::DrawingEntity {
        scope_id: owner_scope_id,
        entity_id: key,
    };
    if let DrawingGeometry::BlockInstance {
        definition_scope_id,
        transform,
    } = source.geometry()
    {
        let definition = &blocks[&u64::from(*definition_scope_id)];
        let (target, source_map, target_map, changed) =
            cad_geometry_convert::geometry::blocks::to_cad_instance_parts(
                *transform,
                &definition.name,
                definition.base_point,
                identity.clone(),
                &state.assessment,
            )
            .map_err(|e| state.construction_error(e))?;
        state.register_identity(key, identity);
        state.record_instance_parts(key, u64::from(*definition_scope_id), source_map, target_map);
        return Ok((EntityType::Insert(target), 0., changed));
    }
    let g = source
        .geometry()
        .as_shared_geometry()
        .expect("primitive branch");
    let mut prepared = cad_geometry_convert::prepare_to_cad(g)
        .map_err(|e| preparation_failure(state, &identity, e, Stage::TargetConstruction))?;
    match (source.geometry(), &mut prepared.entity) {
        (
            DrawingGeometry::PlanarPolyline {
                line_pattern_generation,
                ..
            },
            EntityType::LwPolyline(p),
        ) => p.plinegen = *line_pattern_generation == LinePatternGeneration::Continuous,
        (
            DrawingGeometry::SpatialPolyline {
                line_pattern_generation,
                ..
            },
            EntityType::Polyline3D(p),
        ) => {
            p.flags.linetype_continuous =
                *line_pattern_generation == LinePatternGeneration::Continuous
        }
        _ => {}
    }
    let maximum = state.record_geometry(key, identity, prepared.pair)?;
    Ok((prepared.entity, maximum, prepared.parameterization_changed))
}
