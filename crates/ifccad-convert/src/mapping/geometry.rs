use crate::IfccadDiagnostic;
use ocdraw::ifccad::*;
use opencadcodec::{EntityType, Vector3};

pub(crate) fn v(p: [f64; 3]) -> Vector3 {
    Vector3::new(p[0], p[1], p[2])
}
pub(crate) fn p(v: Vector3) -> [f64; 3] {
    [v.x, v.y, v.z]
}
pub(crate) fn xy(origin: [f64; 3]) -> IfccadPlacement {
    IfccadPlacement {
        origin,
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
pub(crate) fn to_entity(
    kind: &IfccadEntityKind,
    key: u64,
    context: &mut crate::geometry_context::GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Option<EntityType>, crate::IfccadConversionError> {
    let source = crate::IfccadGeometryEntitySource::NativeEntity {
        owner: context.owner,
        entity_id: key,
    };
    let geometry = kind
        .as_shared_geometry()
        .map_err(crate::IfccadConversionError::CoreValidation)?
        .expect("primitive branch");
    let mut prepared = cad_geometry_convert::prepare_to_cad(geometry).map_err(|e| {
        context.preparation_failure(&source, e, crate::IfccadGeometryStage::TargetConstruction)
    })?;
    if let (
        IfccadEntityKind::PlanarPolyline {
            line_pattern_generation,
            ..
        },
        EntityType::LwPolyline(p),
    ) = (kind, &mut prepared.entity)
    {
        p.plinegen = *line_pattern_generation == IfccadLinePatternGeneration::Continuous;
    }
    if let (
        IfccadEntityKind::SpatialPolyline {
            line_pattern_generation,
            ..
        },
        EntityType::Polyline3D(p),
    ) = (kind, &mut prepared.entity)
    {
        p.flags.linetype_continuous =
            *line_pattern_generation == IfccadLinePatternGeneration::Continuous;
    }
    context.record(key, source, prepared.pair, issues)?;
    Ok(Some(prepared.entity))
}
pub(crate) fn placement(frame: ocdraw::geometry_kernel::CoordinateFrame3) -> IfccadPlacement {
    IfccadPlacement {
        origin: frame.origin().components(),
        x_axis: frame.x_axis().components(),
        y_axis: frame.y_axis().components(),
    }
}
fn generation(e: &EntityType) -> IfccadLinePatternGeneration {
    let continuous = match e {
        EntityType::LwPolyline(p) => p.plinegen,
        EntityType::Polyline2D(p) => p.flags.bits() & 128 != 0,
        EntityType::Polyline3D(p) => p.flags.to_bits() & 128 != 0,
        EntityType::Polyline(p) => p.flags.bits() & 128 != 0,
        _ => false,
    };
    if continuous {
        IfccadLinePatternGeneration::Continuous
    } else {
        IfccadLinePatternGeneration::PerSegment
    }
}
pub(crate) fn from_entity(
    e: &EntityType,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
    context: &mut crate::geometry_context::GeometryContext,
) -> Result<Option<IfccadEntityKind>, crate::IfccadConversionError> {
    if !crate::source::entities::classified(e, loc, issues) {
        return Ok(None);
    }
    let identity = crate::IfccadGeometryEntitySource::CadEntity {
        handle: e.common().handle,
        kind: e.as_entity().entity_type().into(),
    };
    let prepared = cad_geometry_convert::prepare_from_cad(e).map_err(|error| {
        context.preparation_failure(
            &identity,
            error,
            crate::IfccadGeometryStage::SourceEvaluation,
        )
    })?;
    context.record(e.common().handle.value(), identity, prepared.pair, issues)?;
    if prepared.normal_normalized {
        issues.push(crate::diagnostics::modification(
            "source-normal-normalized",
            loc,
            "CAD normal magnitude cannot be retained in a placed unit-frame primitive",
        ));
    }
    use ocdraw::geometry_kernel::OwnedGeometry as G;
    let line_pattern_generation = generation(e);
    let kind = match prepared.geometry {
        G::Line { start, end } => IfccadEntityKind::LineSegment { start, end },
        G::Point { placement: frame } => IfccadEntityKind::Point {
            placement: placement(frame),
        },
        G::Circle {
            placement: frame,
            radius,
        } => IfccadEntityKind::Circle {
            placement: placement(frame),
            radius,
        },
        G::Arc {
            placement: frame,
            radius,
            start,
            sweep,
        } => IfccadEntityKind::Arc {
            placement: placement(frame),
            radius,
            start_parameter: start,
            sweep_parameter: sweep,
        },
        G::Ellipse {
            placement: frame,
            major,
            minor,
            arc: None,
        } => IfccadEntityKind::Ellipse {
            placement: placement(frame),
            semi_major_radius: major,
            semi_minor_radius: minor,
        },
        G::Ellipse {
            placement: frame,
            major,
            minor,
            arc: Some((start, sweep)),
        } => IfccadEntityKind::EllipseArc {
            placement: placement(frame),
            semi_major_radius: major,
            semi_minor_radius: minor,
            start_parameter: start,
            sweep_parameter: sweep,
        },
        G::PlanarPolyline {
            placement: frame,
            vertices,
            closed,
        } => IfccadEntityKind::PlanarPolyline {
            placement: placement(frame),
            bulges: vertices.iter().map(|v| v[2]).collect(),
            vertices: vertices.into_iter().map(|v| [v[0], v[1]]).collect(),
            closed,
            line_pattern_generation,
        },
        G::SpatialPolyline { vertices, closed } => IfccadEntityKind::SpatialPolyline {
            vertices,
            closed,
            line_pattern_generation,
        },
    };
    Ok(Some(kind))
}
