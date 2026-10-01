//! Geometry and scope bounds rules over decoded drawing values.

use super::{
    DrawingGeometricEntity, DrawingScope, DrawingScopeKind, DrawingViewport, EntityGeometry,
    LogicalError,
};
use crate::ocdraw::geometry::{bulge_segment_bounds, circular_bounds, elliptic_bounds};
use crate::ocdraw::Point2;
use std::collections::BTreeMap;

pub(crate) fn enclosure(geometry: &EntityGeometry) -> Option<([f64; 3], [f64; 3])> {
    let (min, max) = match geometry {
        EntityGeometry::Line { start, end } => (
            std::array::from_fn(|axis| start[axis].min(end[axis])),
            std::array::from_fn(|axis| start[axis].max(end[axis])),
        ),
        EntityGeometry::Point { placement } => {
            let origin = placement.origin().components();
            (origin, origin)
        }
        EntityGeometry::Circle { placement, radius } => {
            let bounds = circular_bounds(placement.components(), *radius, None)?;
            (bounds.min().components(), bounds.max().components())
        }
        EntityGeometry::Arc {
            placement,
            radius,
            start_parameter,
            sweep_parameter,
        } => {
            let bounds = circular_bounds(
                placement.components(),
                *radius,
                Some((*start_parameter, *sweep_parameter)),
            )?;
            (bounds.min().components(), bounds.max().components())
        }
        EntityGeometry::Ellipse {
            placement,
            semi_major_radius,
            semi_minor_radius,
            arc,
        } => {
            let bounds = elliptic_bounds(
                placement.components(),
                *semi_major_radius,
                *semi_minor_radius,
                *arc,
            )?;
            (bounds.min().components(), bounds.max().components())
        }
        EntityGeometry::PlanarPolyline {
            placement,
            vertices,
            closed,
        } => {
            if vertices.len() < 2 {
                return None;
            }
            let mut min = [f64::INFINITY; 3];
            let mut max = [f64::NEG_INFINITY; 3];
            let segments = if *closed {
                vertices.len()
            } else {
                vertices.len() - 1
            };
            for index in 0..segments {
                let start = vertices[index];
                let end = vertices[(index + 1) % vertices.len()];
                let segment = bulge_segment_bounds(
                    Point2::new(start[0], start[1]),
                    Point2::new(end[0], end[1]),
                    start[2],
                )?;
                for x in [segment.min().x(), segment.max().x()] {
                    for y in [segment.min().y(), segment.max().y()] {
                        let point = placement.enclose_point(Point2::new(x, y)).ok()?;
                        for axis in 0..3 {
                            min[axis] = min[axis].min(point.min().components()[axis]);
                            max[axis] = max[axis].max(point.max().components()[axis]);
                        }
                    }
                }
            }
            (min, max)
        }
        EntityGeometry::SpatialPolyline { vertices, .. } => {
            if vertices.len() < 2 {
                return None;
            }
            let mut min = [f64::INFINITY; 3];
            let mut max = [f64::NEG_INFINITY; 3];
            for vertex in vertices {
                for (axis, value) in vertex.iter().copied().enumerate() {
                    min[axis] = min[axis].min(value);
                    max[axis] = max[axis].max(value);
                }
            }
            (min, max)
        }
        EntityGeometry::BlockInstance { .. } => return None,
    };
    min.into_iter()
        .chain(max)
        .all(f64::is_finite)
        .then_some((min, max))
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
        if owner.kind != DrawingScopeKind::Paper
            || by_id.get(&viewport.view_scope_id).map(|scope| scope.kind)
                != Some(DrawingScopeKind::Model)
        {
            errors.push(LogicalError {
                code: "VIEWPORT_SCOPE",
                location: format!("/streams/viewportStream/entityId/{index}"),
                message: "viewport must belong to paper space and view model space".into(),
            });
        }
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
    use crate::ocdraw::{
        geometry::numeric::{exact, round_down, round_up},
        Bounds3d, Point3,
    };
    if ![
        frame.center.x(),
        frame.center.y(),
        frame.width,
        frame.height,
    ]
    .into_iter()
    .all(f64::is_finite)
        || frame.width <= 0.
        || frame.height <= 0.
    {
        return None;
    }
    let x = exact(frame.center.x());
    let y = exact(frame.center.y());
    let half_width = exact(frame.width) * exact(0.5);
    let half_height = exact(frame.height) * exact(0.5);
    Some(Bounds3d {
        min: Point3::new(
            round_down(&(x.clone() - &half_width)).ok()?,
            round_down(&(y.clone() - &half_height)).ok()?,
            0.,
        ),
        max: Point3::new(
            round_up(&(x + half_width)).ok()?,
            round_up(&(y + half_height)).ok()?,
            0.,
        ),
    })
}
