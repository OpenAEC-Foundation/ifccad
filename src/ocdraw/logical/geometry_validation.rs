//! Geometry and scope bounds rules over decoded drawing values.

use super::{DrawingGeometricEntity, DrawingScope, DrawingViewport, EntityGeometry, LogicalError};
use std::collections::BTreeMap;

pub(crate) fn geometry_ref(
    geometry: &EntityGeometry,
) -> Option<crate::geometry_kernel::GeometryRef<'_>> {
    use crate::geometry_kernel::{GeometryRef as G, PlanarVertices};
    Some(match geometry {
        EntityGeometry::Line { start, end } => G::Line {
            start: *start,
            end: *end,
        },
        EntityGeometry::Point { placement } => G::Point {
            placement: *placement,
        },
        EntityGeometry::Circle { placement, radius } => G::Circle {
            placement: *placement,
            radius: *radius,
        },
        EntityGeometry::Arc {
            placement,
            radius,
            start_parameter,
            sweep_parameter,
        } => G::Arc {
            placement: *placement,
            radius: *radius,
            start: *start_parameter,
            sweep: *sweep_parameter,
        },
        EntityGeometry::Ellipse {
            placement,
            semi_major_radius,
            semi_minor_radius,
            arc,
        } => G::Ellipse {
            placement: *placement,
            major: *semi_major_radius,
            minor: *semi_minor_radius,
            arc: *arc,
        },
        EntityGeometry::PlanarPolyline {
            placement,
            vertices,
            closed,
            ..
        } => G::PlanarPolyline {
            placement: *placement,
            vertices: PlanarVertices::Packed(vertices),
            closed: *closed,
        },
        EntityGeometry::SpatialPolyline {
            vertices, closed, ..
        } => G::SpatialPolyline {
            vertices,
            closed: *closed,
        },
        EntityGeometry::BlockInstance { .. } => return None,
    })
}

pub(crate) fn enclosure(geometry: &EntityGeometry) -> Option<([f64; 3], [f64; 3])> {
    let bounds = crate::geometry_kernel::geometry_bounds(geometry_ref(geometry)?).ok()?;
    Some((bounds.min().components(), bounds.max().components()))
}

pub(crate) fn validate_geometry_bounds(
    entities: &[DrawingGeometricEntity],
    scopes: &[DrawingScope],
) -> Vec<LogicalError> {
    let owners = super::owner_index(scopes);
    let scope_bounds = scopes
        .iter()
        .map(|scope| (scope.id, scope.bounds))
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    for (index, entity) in entities.iter().enumerate() {
        let Some(Some(bounds)) = owners
            .get(&entity.id)
            .and_then(|owner| scope_bounds.get(owner))
        else {
            continue;
        };
        let Some((min, max)) = enclosure(&entity.geometry) else {
            if !matches!(entity.geometry, EntityGeometry::BlockInstance { .. }) {
                errors.push(LogicalError {
                    code: "ENTITY_GEOMETRY",
                    location: format!("/entities/{index}"),
                    message: "entity geometry cannot be evaluated".into(),
                });
            }
            continue;
        };
        let scope_min = bounds.min().components();
        let scope_max = bounds.max().components();
        if min
            .into_iter()
            .zip(max)
            .zip(scope_min.into_iter().zip(scope_max))
            .any(|((min, max), (scope_min, scope_max))| min < scope_min || max > scope_max)
        {
            errors.push(LogicalError {
                code: "SCOPE_BOUNDS",
                location: format!("/entities/{index}"),
                message: "entity lies outside declared scope bounds".into(),
            });
        }
    }
    errors
}

pub(crate) fn validate_viewport_bounds(
    viewports: &[DrawingViewport],
    scopes: &[DrawingScope],
) -> Vec<LogicalError> {
    let owners = super::owner_index(scopes);
    let by_id = scopes
        .iter()
        .map(|scope| (scope.id, scope))
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    for (index, viewport) in viewports.iter().enumerate() {
        let Some(owner) = owners.get(&viewport.id).and_then(|owner| by_id.get(owner)) else {
            continue;
        };
        let Some(frame) = viewport_bounds(viewport.frame) else {
            errors.push(LogicalError {
                code: "VIEWPORT_FRAME",
                location: format!("/streams/viewportStream/frame/{index}"),
                message: "viewport frame must have positive finite dimensions and finite enclosure"
                    .into(),
            });
            continue;
        };
        let Some(bounds) = owner.bounds else {
            continue;
        };
        let min = bounds.min().components();
        let max = bounds.max().components();
        let frame_min_x = frame.min().x();
        let frame_max_x = frame.max().x();
        let frame_min_y = frame.min().y();
        let frame_max_y = frame.max().y();
        if frame_min_x < min[0]
            || frame_max_x > max[0]
            || frame_min_y < min[1]
            || frame_max_y > max[1]
            || 0.0 < min[2]
            || 0.0 > max[2]
        {
            errors.push(LogicalError {
                code: "SCOPE_BOUNDS",
                location: format!("/streams/viewportStream/frame/{index}"),
                message: "viewport lies outside declared paper scope bounds".into(),
            });
        }
    }
    errors
}

pub(crate) fn viewport_bounds(
    frame: super::DrawingViewportFrame,
) -> Option<crate::ocdraw::Bounds3d> {
    crate::geometry_kernel::paper_frame_bounds(crate::geometry_kernel::PaperFrame {
        center: [frame.center.x(), frame.center.y()],
        width: frame.width,
        height: frame.height,
    })
    .ok()
}
