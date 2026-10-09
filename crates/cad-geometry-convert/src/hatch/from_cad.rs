use super::*;
use crate::geometry::numeric::exact;
use num_traits::Signed;
use ocdraw::geometry_kernel::{
    hatch::{validate_hatch_boundaries, HatchEdge2},
    Point3, Vector3,
};
use opencadcodec::entities::hatch::*;
use std::f64::consts::TAU;

fn xy(p: opencadcodec::Vector2) -> [f64; 2] {
    [p.x, p.y]
}
// HATCH wire angles increase in their stored orientation. Clockwise
// coordinates evaluate at negative mathematical angles, unlike standalone ARC.
pub(super) fn sweep(start: f64, end: f64, ccw: bool) -> Result<f64, CadHatchPreparationError> {
    if !start.is_finite() || !end.is_finite() {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    let delta = exact(end) - exact(start);
    let d = end - start;
    if !d.is_finite() || d == 0. || delta.abs() > exact(TAU) {
        return Err(unsupported(
            "paths.edges.angle",
            "ambiguous zero or multiple-turn arc",
        ));
    }
    if delta.abs() == exact(TAU) {
        return Ok(if ccw { TAU } else { -TAU });
    }
    if d.abs() == TAU {
        return Err(unsupported(
            "paths.edges.angle",
            "rounded difference cannot certify a full turn",
        ));
    }
    Ok(if ccw {
        d.rem_euclid(TAU)
    } else {
        -d.rem_euclid(TAU)
    })
}
fn ellipse(e: &EllipticArcEdge) -> Result<([f64; 2], f64, f64), CadHatchPreparationError> {
    let a = e.major_axis_endpoint.x.hypot(e.major_axis_endpoint.y);
    let b = a * e.minor_axis_ratio;
    if !a.is_finite() || a <= 0. || !b.is_finite() || b <= 0. || b > a {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    Ok((
        [e.major_axis_endpoint.x / a, e.major_axis_endpoint.y / a],
        a,
        b,
    ))
}
pub(super) fn boundary(p: &BoundaryPath) -> Result<HatchBoundary2, CadHatchPreparationError> {
    if let [BoundaryEdge::Polyline(e)] = p.edges.as_slice() {
        if !p.flags.is_polyline() {
            return Err(unsupported(
                "paths.flags",
                "polyline edge contradicts its encoding flag",
            ));
        }
        if !e.is_closed {
            return Err(unsupported(
                "paths.edges.polyline.is_closed",
                "open boundary is not implicitly closed",
            ));
        }
        return Ok(HatchBoundary2::Polyline {
            vertices: e.vertices.iter().map(|p| [p.x, p.y]).collect(),
            bulges: e.vertices.iter().map(|p| p.z).collect(),
        });
    }
    if p.flags.is_polyline() {
        return Err(unsupported(
            "paths.flags",
            "polyline flag without one complete polyline",
        ));
    }
    if let [BoundaryEdge::CircularArc(e)] = p.edges.as_slice() {
        if sweep(e.start_angle, e.end_angle, e.counter_clockwise)?.abs() == TAU {
            return Ok(HatchBoundary2::Circle {
                center: xy(e.center),
                radius: e.radius,
            });
        }
    }
    if let [BoundaryEdge::EllipticArc(e)] = p.edges.as_slice() {
        if sweep(e.start_angle, e.end_angle, e.counter_clockwise)?.abs() == TAU {
            let (x_axis, semi_major_radius, semi_minor_radius) = ellipse(e)?;
            return Ok(HatchBoundary2::Ellipse {
                center: xy(e.center),
                x_axis,
                semi_major_radius,
                semi_minor_radius,
            });
        }
    }
    let edges = p
        .edges
        .iter()
        .map(|e| {
            Ok(match e {
                BoundaryEdge::Line(e) => HatchEdge2::Line {
                    start: xy(e.start),
                    end: xy(e.end),
                },
                BoundaryEdge::CircularArc(e) => HatchEdge2::CircularArc {
                    center: xy(e.center),
                    radius: e.radius,
                    start_parameter: if e.counter_clockwise {
                        e.start_angle
                    } else {
                        -e.start_angle
                    },
                    sweep_parameter: sweep(e.start_angle, e.end_angle, e.counter_clockwise)?,
                },
                BoundaryEdge::EllipticArc(e) => {
                    let (x_axis, semi_major_radius, semi_minor_radius) = ellipse(e)?;
                    HatchEdge2::EllipticArc {
                        center: xy(e.center),
                        x_axis,
                        semi_major_radius,
                        semi_minor_radius,
                        start_parameter: if e.counter_clockwise {
                            e.start_angle
                        } else {
                            -e.start_angle
                        },
                        sweep_parameter: sweep(e.start_angle, e.end_angle, e.counter_clockwise)?,
                    }
                }
                BoundaryEdge::Spline(_) => {
                    return Err(unsupported(
                        "paths.edges.spline",
                        "spline boundaries await native spline support",
                    ))
                }
                BoundaryEdge::Polyline(_) => {
                    return Err(unsupported(
                        "paths.edges.polyline",
                        "polyline embedded in edge list",
                    ))
                }
            })
        })
        .collect::<Result<Vec<_>, CadHatchPreparationError>>()?;
    Ok(HatchBoundary2::Edges(edges))
}

pub fn prepare_hatch_from_cad(
    h: &Hatch,
    join_tolerance: f64,
) -> Result<PreparedNativeHatch, CadHatchPreparationError> {
    let losses = super::audit::audit(h)?;
    if !h.elevation.is_finite()
        || ![h.normal.x, h.normal.y, h.normal.z]
            .into_iter()
            .all(f64::is_finite)
    {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    if h.normal.x == 0. && h.normal.y == 0. && h.normal.z == 0. {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    let basis = opencadcodec::types::Matrix3::arbitrary_axis(h.normal).m;
    let u = [basis[0][0], basis[1][0], basis[2][0]];
    let v = [basis[0][1], basis[1][1], basis[2][1]];
    let n = [basis[0][2], basis[1][2], basis[2][2]];
    if !basis.iter().flatten().all(|x| x.is_finite()) {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    let origin = n
        .map(|n| crate::geometry::numeric::round_nearest(&(exact(n) * exact(h.elevation))))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| CadPreparationError::OutOfRange)?;
    let placement = CoordinateFrame3::try_new(
        Point3::new(origin[0], origin[1], origin[2]),
        Vector3::new(u[0], u[1], u[2]),
        Vector3::new(v[0], v[1], v[2]),
    )
    .map_err(|_| CadPreparationError::InvalidGeometry)?;
    let boundaries = h
        .paths
        .iter()
        .map(boundary)
        .collect::<Result<Vec<_>, _>>()?;
    validate_hatch_boundaries(&boundaries, join_tolerance)?;
    let pairs = super::evidence::pairs_from_cad(h, placement, &boundaries)?;
    Ok(PreparedNativeHatch {
        placement,
        boundaries,
        area_rule: match h.style {
            HatchStyleType::Normal => HatchAreaRule::Normal,
            HatchStyleType::Outer => HatchAreaRule::Outer,
            HatchStyleType::Ignore => HatchAreaRule::Ignore,
        },
        join_tolerance,
        source_handles: h.paths.iter().map(|p| p.boundary_handles.clone()).collect(),
        is_associative: h.is_associative,
        pairs,
        losses,
    })
}
