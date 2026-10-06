//! Source geometric field inventory; acceptance policy stays independent of numerical proofs.
use crate::{diagnostics::diagnostic, source::residual, IfccadDiagnostic};
use opencadcodec::EntityType;
pub(crate) fn classified(e: &EntityType, loc: &str, issues: &mut Vec<IfccadDiagnostic>) -> bool {
    let before = issues.len();
    match e {
        EntityType::Line(p) => residual(
            p,
            &opencadcodec::Line::new(),
            &["common", "start", "end"],
            loc,
            issues,
        ),
        EntityType::Point(p) => residual(
            p,
            &opencadcodec::Point::new(),
            &["common", "location", "normal", "x_axis_angle"],
            loc,
            issues,
        ),
        EntityType::Circle(p) => residual(
            p,
            &opencadcodec::Circle::new(),
            &["common", "center", "radius", "normal"],
            loc,
            issues,
        ),
        EntityType::Arc(p) => {
            let sweep = (p.end_angle - p.start_angle).rem_euclid(std::f64::consts::TAU);
            if sweep == 0. || (p.end_angle - p.start_angle).abs() >= std::f64::consts::TAU {
                issues.push(diagnostic(
                    "geometry",
                    loc,
                    "zero/full-turn source ARC has no native arc representation",
                ));
            }
            residual(
                p,
                &opencadcodec::Arc::new(),
                &[
                    "common",
                    "center",
                    "radius",
                    "normal",
                    "start_angle",
                    "end_angle",
                ],
                loc,
                issues,
            );
        }
        EntityType::Ellipse(p) => {
            let difference = p.end_parameter - p.start_parameter;
            if difference == 0. || difference.abs() > std::f64::consts::TAU {
                issues.push(diagnostic(
                    "geometry",
                    loc,
                    "source elliptic span is not one supported full/partial curve",
                ));
            }
            residual(
                p,
                &opencadcodec::Ellipse::new(),
                &[
                    "common",
                    "center",
                    "major_axis",
                    "minor_axis_ratio",
                    "normal",
                    "start_parameter",
                    "end_parameter",
                ],
                loc,
                issues,
            );
        }
        EntityType::LwPolyline(p) => {
            residual(
                p,
                &opencadcodec::LwPolyline::new(),
                &[
                    "common",
                    "vertices",
                    "normal",
                    "elevation",
                    "is_closed",
                    "plinegen",
                ],
                loc,
                issues,
            );
            for v in &p.vertices {
                residual(
                    v,
                    &opencadcodec::entities::LwVertex::new(v.location),
                    &["location", "bulge"],
                    loc,
                    issues,
                );
            }
        }
        EntityType::Polyline2D(p) => {
            if p.flags.bits() & !129 != 0 || p.vertices.iter().any(|v| v.location.z != 0.) {
                issues.push(diagnostic(
                    "geometry",
                    loc,
                    "fitted/mesh/nonplanar classic polyline is unsupported",
                ));
            }
            let mut expected = opencadcodec::entities::Polyline2D::new();
            expected.flags = p.flags;
            residual(
                p,
                &expected,
                &["common", "vertices", "normal", "elevation"],
                loc,
                issues,
            );
            for v in &p.vertices {
                residual(
                    v,
                    &opencadcodec::entities::Vertex2D::new(v.location),
                    &["location", "bulge"],
                    loc,
                    issues,
                );
            }
        }
        EntityType::Polyline3D(p) => {
            if p.flags.to_bits() & !(1 | 8 | 128) != 0 {
                issues.push(diagnostic(
                    "geometry",
                    loc,
                    "fitted/mesh spatial polyline is unsupported",
                ));
            }
            let mut expected = opencadcodec::entities::Polyline3D::new();
            expected.flags = p.flags;
            residual(p, &expected, &["common", "vertices"], loc, issues);
            for v in &p.vertices {
                let mut expected = opencadcodec::entities::Vertex3DPolyline::new(v.position);
                if v.flags & !32 == 0 {
                    expected.flags = v.flags;
                }
                if v.layer.is_empty() || v.layer == "0" {
                    expected.layer = v.layer.clone();
                }
                residual(v, &expected, &["position", "handle"], loc, issues);
            }
        }
        EntityType::Polyline(p) => {
            if p.flags.bits() & !(1 | 8 | 128) != 0 {
                issues.push(diagnostic(
                    "geometry",
                    loc,
                    "fitted/mesh generic polyline is unsupported",
                ));
            }
            let mut expected = opencadcodec::Polyline::new();
            expected.flags = p.flags;
            residual(p, &expected, &["common", "vertices"], loc, issues);
            for v in &p.vertices {
                let mut expected = opencadcodec::entities::Vertex3D::new(v.location);
                // Constructor-default vertex flags are geometry infrastructure.
                expected.flags = if v.flags.bits() & !32 == 0 {
                    v.flags
                } else {
                    opencadcodec::entities::VertexFlags::new()
                };
                residual(v, &expected, &["location"], loc, issues);
            }
        }
        _ => issues.push(diagnostic("geometry", loc, "unsupported entity family")),
    }
    issues.len() == before
}
