//! Source geometric field inventory; acceptance policy stays independent of numerical proofs.
use crate::mapping::geometry::p;
use crate::{diagnostics::diagnostic, source::residual, IfccadDiagnostic};
use opencadcodec::{EntityType, Vector3};
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

/// Invalid scalars in known geometry remain errors even when another field
/// would cause the entire entity to be skipped.
pub(crate) fn validate_source(entity: &EntityType) -> Result<(), crate::IfccadConversionError> {
    let finite = |v: Vector3| p(v).iter().all(|n| n.is_finite());
    let valid = match entity {
        EntityType::Line(l) => {
            finite(l.start) && finite(l.end) && finite(l.normal) && l.thickness.is_finite()
        }
        EntityType::Point(p) => {
            finite(p.location)
                && finite(p.normal)
                && p.x_axis_angle.is_finite()
                && p.thickness.is_finite()
        }
        EntityType::Arc(p) => {
            finite(p.center)
                && finite(p.normal)
                && p.radius.is_finite()
                && p.radius > 0.
                && p.start_angle.is_finite()
                && p.end_angle.is_finite()
                && p.thickness.is_finite()
        }
        EntityType::Ellipse(p) => {
            finite(p.center)
                && finite(p.normal)
                && finite(p.major_axis)
                && p.minor_axis_ratio.is_finite()
                && p.minor_axis_ratio > 0.
                && p.minor_axis_ratio <= 1.
                && p.start_parameter.is_finite()
                && p.end_parameter.is_finite()
        }
        EntityType::Polyline2D(p) => {
            finite(p.normal)
                && [p.elevation, p.thickness, p.start_width, p.end_width]
                    .into_iter()
                    .all(f64::is_finite)
                && p.vertices.len() >= 2
                && p.vertices.iter().all(|v| {
                    finite(v.location)
                        && [v.start_width, v.end_width, v.bulge, v.curve_tangent]
                            .into_iter()
                            .all(f64::is_finite)
                })
        }
        EntityType::Polyline3D(p) => {
            finite(p.normal)
                && [p.elevation, p.default_start_width, p.default_end_width]
                    .into_iter()
                    .all(f64::is_finite)
                && p.vertices.len() >= 2
                && p.vertices.iter().all(|v| finite(v.position))
        }
        EntityType::Polyline(p) => {
            p.vertices.len() >= 2 && p.vertices.iter().all(|v| finite(v.location))
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
