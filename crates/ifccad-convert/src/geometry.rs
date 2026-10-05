use crate::{diagnostics::diagnostic, IfccadDiagnostic};
use num_rational::BigRational;
use ocdraw::ifccad::*;
use opencadcodec::{EntityType, Vector2, Vector3};

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
pub(crate) fn canonical(f: &IfccadPlacement, loc: &str, issues: &mut Vec<IfccadDiagnostic>) {
    if f.x_axis != [1., 0., 0.] || f.y_axis != [0., 1., 0.] {
        issues.push(diagnostic(
            "placement",
            loc,
            "this slice supports only standard XY frames",
        ));
    }
}
fn add(a: f64, b: f64, loc: &str, issues: &mut Vec<IfccadDiagnostic>) -> f64 {
    let result = a + b;
    let exact = BigRational::from_float(a)
        .zip(BigRational::from_float(b))
        .map(|(a, b)| a + b);
    if exact.is_none() || exact != BigRational::from_float(result) {
        issues.push(diagnostic(
            "rounding",
            loc,
            "translated coordinate is not an exact finite binary64 sum",
        ));
    }
    result
}
pub(crate) fn to_entity(
    kind: &IfccadEntityKind,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Option<EntityType> {
    let before = issues.len();
    let result = match kind {
        IfccadEntityKind::LineSegment { start, end } => {
            EntityType::Line(opencadcodec::Line::from_points(v(*start), v(*end)))
        }
        IfccadEntityKind::Circle { radius, placement } => {
            canonical(placement, loc, issues);
            EntityType::Circle(opencadcodec::Circle::from_center_radius(
                v(placement.origin),
                *radius,
            ))
        }
        IfccadEntityKind::PlanarPolyline {
            vertices,
            closed,
            placement,
            line_pattern_generation,
        } => {
            canonical(placement, loc, issues);
            let vertices = vertices
                .iter()
                .map(|p| {
                    Vector2::new(
                        add(p[0], placement.origin[0], loc, issues),
                        add(p[1], placement.origin[1], loc, issues),
                    )
                })
                .collect();
            let mut poly = opencadcodec::entities::LwPolyline::from_points(vertices);
            poly.elevation = placement.origin[2];
            poly.is_closed = *closed;
            poly.plinegen = *line_pattern_generation == IfccadLinePatternGeneration::Continuous;
            EntityType::LwPolyline(poly)
        }
        IfccadEntityKind::Viewport(_) => unreachable!("viewports use staged entity conversion"),
        IfccadEntityKind::BlockInstance { .. } => {
            issues.push(diagnostic("blocks", loc, "block conversion pending"));
            return None;
        }
    };
    (issues.len() == before).then_some(result)
}
pub(crate) fn from_entity(
    entity: &EntityType,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Option<IfccadEntityKind> {
    let unsupported = |issues: &mut Vec<_>, msg| issues.push(diagnostic("geometry", loc, msg));
    let before = issues.len();
    let result = match entity {
        EntityType::Line(l) => {
            crate::source::residual(
                l,
                &opencadcodec::Line::new(),
                &["common", "start", "end"],
                loc,
                issues,
            );
            IfccadEntityKind::LineSegment {
                start: p(l.start),
                end: p(l.end),
            }
        }
        EntityType::Circle(c) => {
            crate::source::residual(
                c,
                &opencadcodec::Circle::new(),
                &["common", "center", "radius"],
                loc,
                issues,
            );
            IfccadEntityKind::Circle {
                radius: c.radius,
                placement: xy(p(c.center)),
            }
        }
        EntityType::LwPolyline(l) => {
            crate::source::residual(
                l,
                &opencadcodec::entities::LwPolyline::new(),
                &["common", "vertices", "is_closed", "elevation", "plinegen"],
                loc,
                issues,
            );
            for vertex in &l.vertices {
                crate::source::residual(
                    vertex,
                    &opencadcodec::entities::LwVertex::new(vertex.location),
                    &["location"],
                    loc,
                    issues,
                );
            }
            IfccadEntityKind::PlanarPolyline {
                line_pattern_generation: if l.plinegen {
                    IfccadLinePatternGeneration::Continuous
                } else {
                    IfccadLinePatternGeneration::PerSegment
                },
                vertices: l
                    .vertices
                    .iter()
                    .map(|v| [v.location.x, v.location.y])
                    .collect(),
                closed: l.is_closed,
                placement: xy([0., 0., l.elevation]),
            }
        }
        _ => {
            unsupported(issues, "unsupported entity family");
            return None;
        }
    };
    (issues.len() == before).then_some(result)
}

/// Invalid scalars in known geometry remain errors even when another field
/// would cause the entire entity to be skipped.
pub(crate) fn validate_source(entity: &EntityType) -> Result<(), crate::IfccadConversionError> {
    let finite = |v: Vector3| p(v).iter().all(|n| n.is_finite());
    let valid = match entity {
        EntityType::Line(l) => {
            finite(l.start) && finite(l.end) && finite(l.normal) && l.thickness.is_finite()
        }
        EntityType::Circle(c) => {
            finite(c.center)
                && finite(c.normal)
                && c.radius.is_finite()
                && c.radius > 0.
                && c.thickness.is_finite()
        }
        EntityType::LwPolyline(l) => {
            finite(l.normal)
                && l.elevation.is_finite()
                && l.thickness.is_finite()
                && l.constant_width.is_finite()
                && l.vertices.len() >= 2
                && l.vertices.iter().all(|v| {
                    [
                        v.location.x,
                        v.location.y,
                        v.bulge,
                        v.start_width,
                        v.end_width,
                    ]
                    .iter()
                    .all(|n| n.is_finite())
                })
        }
        EntityType::Insert(i) => {
            finite(i.insert_point)
                && finite(i.normal)
                && i.rotation.is_finite()
                && [i.x_scale(), i.y_scale(), i.z_scale()]
                    .iter()
                    .all(|s| s.is_finite() && *s != 0.)
        }
        _ => true,
    };
    if !valid {
        return Err(crate::IfccadConversionError::InvalidStructure(format!(
            "invalid geometry scalars at entity/{}",
            entity.common().handle
        )));
    }
    Ok(())
}
