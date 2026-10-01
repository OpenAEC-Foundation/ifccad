use crate::{outcome::diagnostic, IfcxCadDiagnostic};
use cadcodec::{EntityType, Vector2, Vector3};
use num_rational::BigRational;
use ocdraw::ifcx_cad::*;

pub(crate) fn v(p: [f64; 3]) -> Vector3 {
    Vector3::new(p[0], p[1], p[2])
}
pub(crate) fn p(v: Vector3) -> [f64; 3] {
    [v.x, v.y, v.z]
}
pub(crate) fn xy(origin: [f64; 3]) -> IfcxCadPlacement {
    IfcxCadPlacement {
        origin,
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
pub(crate) fn canonical(f: &IfcxCadPlacement, loc: &str, issues: &mut Vec<IfcxCadDiagnostic>) {
    if f.x_axis != [1., 0., 0.] || f.y_axis != [0., 1., 0.] {
        issues.push(diagnostic(
            "placement",
            loc,
            "this slice supports only standard XY frames",
        ));
    }
}
fn add(a: f64, b: f64, loc: &str, issues: &mut Vec<IfcxCadDiagnostic>) -> f64 {
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
    kind: &IfcxCadEntityKind,
    loc: &str,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> Option<EntityType> {
    Some(match kind {
        IfcxCadEntityKind::LineSegment { start, end } => {
            EntityType::Line(cadcodec::Line::from_points(v(*start), v(*end)))
        }
        IfcxCadEntityKind::Circle { radius, placement } => {
            canonical(placement, loc, issues);
            EntityType::Circle(cadcodec::Circle::from_center_radius(
                v(placement.origin),
                *radius,
            ))
        }
        IfcxCadEntityKind::PlanarPolyline {
            vertices,
            closed,
            placement,
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
            let mut poly = cadcodec::entities::LwPolyline::from_points(vertices);
            poly.elevation = placement.origin[2];
            poly.is_closed = *closed;
            EntityType::LwPolyline(poly)
        }
        IfcxCadEntityKind::BlockInstance { .. } => {
            issues.push(diagnostic("blocks", loc, "block conversion pending"));
            return None;
        }
    })
}
pub(crate) fn from_entity(
    entity: &EntityType,
    loc: &str,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> Option<IfcxCadEntityKind> {
    let unsupported = |issues: &mut Vec<_>, msg| issues.push(diagnostic("geometry", loc, msg));
    Some(match entity {
        EntityType::Line(l) => {
            crate::source::residual(
                l,
                &cadcodec::Line::new(),
                &["common", "start", "end"],
                loc,
                issues,
            );
            IfcxCadEntityKind::LineSegment {
                start: p(l.start),
                end: p(l.end),
            }
        }
        EntityType::Circle(c) => {
            crate::source::residual(
                c,
                &cadcodec::Circle::new(),
                &["common", "center", "radius"],
                loc,
                issues,
            );
            IfcxCadEntityKind::Circle {
                radius: c.radius,
                placement: xy(p(c.center)),
            }
        }
        EntityType::LwPolyline(l) => {
            crate::source::residual(
                l,
                &cadcodec::entities::LwPolyline::new(),
                &["common", "vertices", "is_closed", "elevation"],
                loc,
                issues,
            );
            for vertex in &l.vertices {
                crate::source::residual(
                    vertex,
                    &cadcodec::entities::LwVertex::new(vertex.location),
                    &["location"],
                    loc,
                    issues,
                );
            }
            IfcxCadEntityKind::PlanarPolyline {
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
    })
}
