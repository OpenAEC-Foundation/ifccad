//! Validation of block transforms over typed geometric records.

use super::{
    DrawingBlockDefinition, DrawingGeometricEntity, DrawingScope, EntityGeometry, LogicalError,
};
use crate::ocdraw::{
    geometry::{numeric::Interval, PreparedBlockTransform},
    Point3,
};
use std::collections::BTreeMap;

pub(crate) fn validate_blocks(
    entities: &[DrawingGeometricEntity],
    definitions: &[DrawingBlockDefinition],
    scopes: &[DrawingScope],
) -> Vec<LogicalError> {
    let owners = super::owner_index(scopes);
    let definitions = definitions
        .iter()
        .map(|d| (d.scope_id, d))
        .collect::<BTreeMap<_, _>>();
    let scope_bounds = scopes
        .iter()
        .map(|s| (s.id, s.bounds))
        .collect::<BTreeMap<_, _>>();
    let mut members = BTreeMap::<u32, Vec<&DrawingGeometricEntity>>::new();
    for entity in entities {
        if let Some(owner) = owners.get(&entity.id) {
            members.entry(*owner).or_default().push(entity);
        }
    }
    let mut errors = Vec::new();
    for root in entities {
        let EntityGeometry::BlockInstance {
            definition_scope_id,
            transform,
        } = root.geometry
        else {
            continue;
        };
        let Some(definition) = definitions.get(&definition_scope_id) else {
            continue;
        };
        if definition.uniform_scaling && !transform.scale().is_uniform() {
            errors.push(LogicalError {
                code: "BLOCK_SCALE",
                location: format!("/entities/{}", root.id),
                message: "block definition requires uniform scaling".into(),
            });
            continue;
        }
        let base = definition.base_point;
        let Some(prepared) =
            PreparedBlockTransform::new(transform, Point3::new(base[0], base[1], base[2]))
        else {
            errors.push(error(root.id, "block transform cannot be evaluated"));
            continue;
        };
        let Some(Some(bounds)) = owners
            .get(&root.id)
            .and_then(|owner| scope_bounds.get(owner))
        else {
            continue;
        };
        let min = bounds.min().components();
        let max = bounds.max().components();
        let mut transforms = vec![prepared];
        let mut stack = vec![(definition_scope_id, vec![0usize], vec![definition_scope_id])];
        let mut invalid = false;
        while let Some((scope, path, ancestors)) = stack.pop() {
            let leaves = members.get(&scope).map(Vec::as_slice).unwrap_or(&[]);
            for leaf in leaves {
                if let EntityGeometry::BlockInstance {
                    definition_scope_id,
                    transform,
                } = leaf.geometry
                {
                    if ancestors.contains(&definition_scope_id) {
                        invalid = true;
                        break;
                    }
                    let Some(definition) = definitions.get(&definition_scope_id) else {
                        invalid = true;
                        break;
                    };
                    let base = definition.base_point;
                    let Some(prepared) = PreparedBlockTransform::new(
                        transform,
                        Point3::new(base[0], base[1], base[2]),
                    ) else {
                        invalid = true;
                        break;
                    };
                    let mut next = path.clone();
                    next.push(transforms.len());
                    transforms.push(prepared);
                    let mut ancestors = ancestors.clone();
                    ancestors.push(definition_scope_id);
                    stack.push((definition_scope_id, next, ancestors));
                } else {
                    let Some(points) = leaf_points(&leaf.geometry) else {
                        invalid = true;
                        break;
                    };
                    for point in points {
                        let mut point = Some(point);
                        for &index in path.iter().rev() {
                            point = point.and_then(|p| transforms[index].apply_intervals(p));
                        }
                        if point.is_none_or(|p| {
                            (0..3)
                                .any(|axis| p[axis].lower < min[axis] || p[axis].upper > max[axis])
                        }) {
                            invalid = true;
                            break;
                        }
                    }
                    if invalid {
                        break;
                    }
                }
            }
            if invalid {
                break;
            }
        }
        if invalid {
            errors.push(error(
                root.id,
                "owning scope bounds do not enclose the block occurrence",
            ));
        }
    }
    errors
}
fn error(id: u64, message: &str) -> LogicalError {
    LogicalError {
        code: "BLOCK_BOUNDS",
        location: format!("/entities/{id}"),
        message: message.into(),
    }
}

fn leaf_points(geometry: &EntityGeometry) -> Option<Vec<[Interval; 3]>> {
    let mut points = Vec::new();
    match geometry {
        EntityGeometry::Line { start, end } => {
            points.extend([start.map(Interval::point), end.map(Interval::point)])
        }
        EntityGeometry::Point { placement } => {
            points.push(placement.origin().components().map(Interval::point))
        }
        EntityGeometry::SpatialPolyline { vertices, .. } => {
            points.extend(vertices.iter().map(|p| p.map(Interval::point)))
        }
        EntityGeometry::PlanarPolyline {
            placement,
            vertices,
            closed,
            ..
        } => {
            let mut local = vertices
                .iter()
                .map(|p| crate::ocdraw::Point2::new(p[0], p[1]))
                .collect::<Vec<_>>();
            let segments = if *closed {
                vertices.len()
            } else {
                vertices.len().saturating_sub(1)
            };
            for index in 0..segments {
                let start = vertices[index];
                let end = vertices[(index + 1) % vertices.len()];
                if start[2] == 0.0 {
                    continue;
                }
                let bounds = crate::ocdraw::geometry::bulge_segment_bounds(
                    crate::ocdraw::Point2::new(start[0], start[1]),
                    crate::ocdraw::Point2::new(end[0], end[1]),
                    start[2],
                )?;
                for x in [bounds.min().x(), bounds.max().x()] {
                    for y in [bounds.min().y(), bounds.max().y()] {
                        local.push(crate::ocdraw::Point2::new(x, y));
                    }
                }
            }
            for point in local {
                let bounds = placement.enclose_point(point).ok()?;
                points.push(std::array::from_fn(|axis| Interval {
                    lower: bounds.min().components()[axis],
                    upper: bounds.max().components()[axis],
                }));
            }
        }
        EntityGeometry::Circle { .. }
        | EntityGeometry::Arc { .. }
        | EntityGeometry::Ellipse { .. } => {
            let (lower, upper) = super::enclosure(geometry)?;
            for mask in 0..8 {
                points.push(std::array::from_fn(|axis| {
                    Interval::point(if mask & (1 << axis) == 0 {
                        lower[axis]
                    } else {
                        upper[axis]
                    })
                }));
            }
        }
        EntityGeometry::BlockInstance { .. } => return None,
    }
    Some(points)
}
