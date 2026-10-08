use super::evidence::Frame;
use super::*;
use crate::geometry::{
    cad_plane,
    numeric::{exact, round_nearest},
    stored_normal,
};
use num_rational::BigRational as Q;
use ocdraw::geometry_kernel::hatch::{validate_hatch_boundaries, HatchEdge2};
use opencadcodec::entities::hatch::*;
use opencadcodec::{Vector2, Vector3};
use std::f64::consts::TAU;

struct Projection {
    frame: Frame,
    u: [Q; 3],
    v: [Q; 3],
}
impl Projection {
    fn point(&self, p: [f64; 2]) -> Result<Vector2, CadHatchPreparationError> {
        let w = self.frame.point(p.map(exact));
        self.round([
            self.u.iter().zip(&w).map(|(a, b)| a * b).sum(),
            self.v.iter().zip(&w).map(|(a, b)| a * b).sum(),
        ])
    }
    fn vector(&self, p: [f64; 2]) -> Result<Vector2, CadHatchPreparationError> {
        let w: [Q; 3] = std::array::from_fn(|i| {
            &self.frame.x[i] * exact(p[0]) + &self.frame.y[i] * exact(p[1])
        });
        self.round([
            self.u.iter().zip(&w).map(|(a, b)| a * b).sum(),
            self.v.iter().zip(&w).map(|(a, b)| a * b).sum(),
        ])
    }
    fn round(&self, p: [Q; 2]) -> Result<Vector2, CadHatchPreparationError> {
        Ok(Vector2::new(
            round_nearest(&p[0]).map_err(|_| CadPreparationError::OutOfRange)?,
            round_nearest(&p[1]).map_err(|_| CadPreparationError::OutOfRange)?,
        ))
    }
    fn arc(
        &self,
        c: [f64; 2],
        r: f64,
        start: f64,
        sweep: f64,
    ) -> Result<BoundaryEdge, CadHatchPreparationError> {
        let axis = self.vector([1., 0.])?;
        let phase = axis.y.atan2(axis.x);
        let start = start + phase;
        let end = start + sweep;
        if !start.is_finite() || !end.is_finite() {
            return Err(CadPreparationError::OutOfRange.into());
        }
        Ok(BoundaryEdge::CircularArc(CircularArcEdge {
            center: self.point(c)?,
            radius: r,
            start_angle: start,
            end_angle: end,
            counter_clockwise: sweep > 0.,
        }))
    }
    #[allow(clippy::too_many_arguments)]
    fn ellipse(
        &self,
        c: [f64; 2],
        x: [f64; 2],
        a: f64,
        b: f64,
        start: f64,
        sweep: f64,
    ) -> Result<BoundaryEdge, CadHatchPreparationError> {
        let major = self.vector([x[0] * a, x[1] * a])?;
        if !major.x.is_finite()
            || !major.y.is_finite()
            || !start.is_finite()
            || !(start + sweep).is_finite()
        {
            return Err(CadPreparationError::OutOfRange.into());
        }
        Ok(BoundaryEdge::EllipticArc(EllipticArcEdge {
            center: self.point(c)?,
            major_axis_endpoint: major,
            minor_axis_ratio: b / a,
            start_angle: start,
            end_angle: start + sweep,
            counter_clockwise: sweep > 0.,
        }))
    }
    fn boundary(&self, b: &HatchBoundary2) -> Result<BoundaryPath, CadHatchPreparationError> {
        let mut p = BoundaryPath::new();
        match b {
            HatchBoundary2::Polyline { vertices, bulges } => {
                let vertices = vertices
                    .iter()
                    .zip(bulges)
                    .map(|(&v, &b)| {
                        let v = self.point(v)?;
                        Ok(Vector3::new(v.x, v.y, b))
                    })
                    .collect::<Result<Vec<_>, CadHatchPreparationError>>()?;
                p.add_edge(BoundaryEdge::Polyline(PolylineEdge {
                    vertices,
                    is_closed: true,
                }));
            }
            HatchBoundary2::Circle { center, radius } => {
                // full circles have no phase; use the codec's canonical full turn.
                p.add_edge(BoundaryEdge::CircularArc(CircularArcEdge {
                    center: self.point(*center)?,
                    radius: *radius,
                    start_angle: 0.,
                    end_angle: TAU,
                    counter_clockwise: true,
                }));
            }
            HatchBoundary2::Ellipse {
                center,
                x_axis,
                semi_major_radius: a,
                semi_minor_radius: b,
            } => p.add_edge(self.ellipse(*center, *x_axis, *a, *b, 0., TAU)?),
            HatchBoundary2::Edges(e) => {
                for e in e {
                    p.add_edge(match *e {
                        HatchEdge2::Line { start, end } => BoundaryEdge::Line(LineEdge {
                            start: self.point(start)?,
                            end: self.point(end)?,
                        }),
                        HatchEdge2::CircularArc {
                            center,
                            radius,
                            start_parameter,
                            sweep_parameter,
                        } => self.arc(center, radius, start_parameter, sweep_parameter)?,
                        HatchEdge2::EllipticArc {
                            center,
                            x_axis,
                            semi_major_radius: a,
                            semi_minor_radius: b,
                            start_parameter,
                            sweep_parameter,
                        } => {
                            self.ellipse(center, x_axis, a, b, start_parameter, sweep_parameter)?
                        }
                    });
                }
            }
        }
        Ok(p)
    }
}
pub fn prepare_hatch_to_cad(
    placement: CoordinateFrame3,
    boundaries: &[HatchBoundary2],
    area_rule: HatchAreaRule,
    join_tolerance: f64,
) -> Result<PreparedCadHatch, CadHatchPreparationError> {
    validate_hatch_boundaries(boundaries, join_tolerance)?;
    let normal = stored_normal(placement).ok_or(CadPreparationError::InvalidGeometry)?;
    let basis = cad_plane(normal).ok_or(CadPreparationError::InvalidGeometry)?;
    let frame = Frame::native(placement);
    let elevation: Q = basis
        .n
        .into_iter()
        .zip(&frame.o)
        .map(|(a, b)| exact(a) * b)
        .sum();
    let elevation = round_nearest(&elevation).map_err(|_| CadPreparationError::OutOfRange)?;
    let projection = Projection {
        frame,
        u: basis.u.map(exact),
        v: basis.v.map(exact),
    };
    let mut hatch = Hatch::solid();
    hatch.normal = normal;
    hatch.elevation = elevation;
    hatch.style = match area_rule {
        HatchAreaRule::Normal => HatchStyleType::Normal,
        HatchAreaRule::Outer => HatchStyleType::Outer,
        HatchAreaRule::Ignore => HatchStyleType::Ignore,
    };
    hatch.paths = boundaries
        .iter()
        .map(|b| projection.boundary(b))
        .collect::<Result<Vec<_>, _>>()?;
    let pairs = super::evidence::pairs_to_cad(&hatch, placement, boundaries)?;
    Ok(PreparedCadHatch { hatch, pairs })
}
