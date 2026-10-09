use super::*;
use crate::geometry::{
    blocks::{trig, PairedCurve, PairedPoint, Range},
    numeric::exact,
};
use num_rational::BigRational as Q;
use num_traits::{Signed, Zero};
use ocdraw::geometry_kernel::hatch::HatchEdge2;
use opencadcodec::entities::hatch::*;

#[derive(Clone)]
pub(super) struct Frame {
    pub o: [Q; 3],
    pub x: [Q; 3],
    pub y: [Q; 3],
}
impl Frame {
    pub fn native(p: CoordinateFrame3) -> Self {
        let (o, x, y) = (p.origin(), p.x_axis(), p.y_axis());
        Self {
            o: [o.x(), o.y(), o.z()].map(exact),
            x: [x.x(), x.y(), x.z()].map(exact),
            y: [y.x(), y.y(), y.z()].map(exact),
        }
    }
    pub fn cad(h: &Hatch) -> Self {
        let m = opencadcodec::types::Matrix3::arbitrary_axis(h.normal).m;
        Self {
            o: [0, 1, 2].map(|i| exact(m[i][2]) * exact(h.elevation)),
            x: [0, 1, 2].map(|i| exact(m[i][0])),
            y: [0, 1, 2].map(|i| exact(m[i][1])),
        }
    }
    pub fn point(&self, p: [Q; 2]) -> [Q; 3] {
        std::array::from_fn(|i| &self.o[i] + &self.x[i] * &p[0] + &self.y[i] * &p[1])
    }
    fn vector(&self, p: [Q; 2]) -> [Q; 3] {
        std::array::from_fn(|i| &self.x[i] * &p[0] + &self.y[i] * &p[1])
    }
}
struct Curves {
    points: Vec<[Q; 3]>,
    curves: Vec<([Q; 3], [[Range; 3]; 2], Q)>,
}
impl Curves {
    fn new() -> Self {
        Self {
            points: vec![],
            curves: vec![],
        }
    }
    fn point(&mut self, f: &Frame, p: [f64; 2]) {
        self.points.push(f.point(p.map(exact)));
    }
    #[allow(clippy::too_many_arguments)]
    fn curve(&mut self, f: &Frame, c: [Q; 2], a: [Q; 2], b: [Q; 2], start: f64, sweep: f64) {
        let a = f.vector(a);
        let b = f.vector(b);
        let (s, co) = trig(start);
        let axis0 = std::array::from_fn(|i| co.scale(&a[i]).add(&s.scale(&b[i])));
        let axis1 = std::array::from_fn(|i| {
            co.scale(&b[i])
                .add(&s.scale(&-a[i].clone()))
                .scale(&exact(sweep.signum()))
        });
        self.curves
            .push((f.point(c), [axis0, axis1], exact(sweep.abs())));
    }
    fn polyline(&mut self, f: &Frame, v: &[[f64; 2]], bulges: &[f64]) {
        for (i, &p) in v.iter().enumerate() {
            self.point(f, p);
            let b = bulges[i];
            if b == 0. {
                continue;
            }
            let a = p.map(exact);
            let z = v[(i + 1) % v.len()].map(exact);
            let dx = &z[0] - &a[0];
            let dy = &z[1] - &a[1];
            let b = exact(b);
            let t = (exact(1.) - &b * &b) / (exact(4.) * &b);
            let c = [
                (&a[0] + &z[0]) / exact(2.) - &dy * &t,
                (&a[1] + &z[1]) / exact(2.) + &dx * &t,
            ];
            let radial = [&a[0] - &c[0], &a[1] - &c[1]];
            self.curve(
                f,
                c,
                radial.clone(),
                [-radial[1].clone(), radial[0].clone()],
                0.,
                if b > Q::zero() {
                    std::f64::consts::TAU
                } else {
                    -std::f64::consts::TAU
                },
            );
        }
    }
}
fn native(f: &Frame, b: &HatchBoundary2) -> Curves {
    let mut out = Curves::new();
    let mut ellipse = |c: [f64; 2], x: [f64; 2], a: f64, b: f64, start: f64, sweep: f64| {
        out.curve(
            f,
            c.map(exact),
            [exact(x[0]) * exact(a), exact(x[1]) * exact(a)],
            [-exact(x[1]) * exact(b), exact(x[0]) * exact(b)],
            start,
            sweep,
        )
    };
    match b {
        HatchBoundary2::Circle { center, radius } => ellipse(
            *center,
            [1., 0.],
            *radius,
            *radius,
            0.,
            std::f64::consts::TAU,
        ),
        HatchBoundary2::Ellipse {
            center,
            x_axis,
            semi_major_radius,
            semi_minor_radius,
        } => ellipse(
            *center,
            *x_axis,
            *semi_major_radius,
            *semi_minor_radius,
            0.,
            std::f64::consts::TAU,
        ),
        HatchBoundary2::Polyline { vertices, bulges } => out.polyline(f, vertices, bulges),
        HatchBoundary2::Edges(e) => {
            for e in e {
                match *e {
                    HatchEdge2::Line { start, end } => {
                        out.point(f, start);
                        out.point(f, end);
                    }
                    HatchEdge2::CircularArc {
                        center,
                        radius,
                        start_parameter,
                        sweep_parameter,
                    } => out.curve(
                        f,
                        center.map(exact),
                        [exact(radius), Q::zero()],
                        [Q::zero(), exact(radius)],
                        start_parameter,
                        sweep_parameter,
                    ),
                    HatchEdge2::EllipticArc {
                        center,
                        x_axis,
                        semi_major_radius: a,
                        semi_minor_radius: b,
                        start_parameter,
                        sweep_parameter,
                    } => out.curve(
                        f,
                        center.map(exact),
                        [exact(x_axis[0]) * exact(a), exact(x_axis[1]) * exact(a)],
                        [-exact(x_axis[1]) * exact(b), exact(x_axis[0]) * exact(b)],
                        start_parameter,
                        sweep_parameter,
                    ),
                }
            }
        }
    }
    out
}
fn cad(f: &Frame, p: &BoundaryPath, circle_phase: f64) -> Result<Curves, CadHatchPreparationError> {
    let mut out = Curves::new();
    for e in &p.edges {
        match e {
            BoundaryEdge::Line(e) => {
                out.point(f, [e.start.x, e.start.y]);
                out.point(f, [e.end.x, e.end.y]);
            }
            BoundaryEdge::Polyline(e) => out.polyline(
                f,
                &e.vertices.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>(),
                &e.vertices.iter().map(|p| p.z).collect::<Vec<_>>(),
            ),
            BoundaryEdge::CircularArc(e) => {
                let sweep =
                    super::from_cad::sweep(e.start_angle, e.end_angle, e.counter_clockwise)?;
                let full = sweep.abs() == std::f64::consts::TAU;
                out.curve(
                    f,
                    [e.center.x, e.center.y].map(exact),
                    [exact(e.radius), Q::zero()],
                    [Q::zero(), exact(e.radius)],
                    if full {
                        circle_phase
                    } else if e.counter_clockwise {
                        e.start_angle
                    } else {
                        -e.start_angle
                    },
                    if full { std::f64::consts::TAU } else { sweep },
                );
                out.curves.last_mut().unwrap().2 =
                    exact_cad_span(e.start_angle, e.end_angle, e.counter_clockwise);
            }
            BoundaryEdge::EllipticArc(e) => {
                let sweep =
                    super::from_cad::sweep(e.start_angle, e.end_angle, e.counter_clockwise)?;
                let full = sweep.abs() == std::f64::consts::TAU;
                let a = [e.major_axis_endpoint.x, e.major_axis_endpoint.y].map(exact);
                let ratio = exact(e.minor_axis_ratio);
                out.curve(
                    f,
                    [e.center.x, e.center.y].map(exact),
                    a.clone(),
                    [-&a[1] * &ratio, &a[0] * &ratio],
                    if full {
                        0.
                    } else if e.counter_clockwise {
                        e.start_angle
                    } else {
                        -e.start_angle
                    },
                    if full { std::f64::consts::TAU } else { sweep },
                );
                out.curves.last_mut().unwrap().2 =
                    exact_cad_span(e.start_angle, e.end_angle, e.counter_clockwise);
            }
            BoundaryEdge::Spline(_) => {
                return Err(unsupported(
                    "paths.edges.spline",
                    "unsupported curve evidence",
                ))
            }
        }
    }
    Ok(out)
}
fn exact_cad_span(start: f64, end: f64, _ccw: bool) -> Q {
    let d = exact(end) - exact(start);
    let tau = exact(std::f64::consts::TAU);
    if d.abs() == tau {
        tau
    } else if d > Q::zero() {
        d
    } else {
        d + tau
    }
}
pub(super) fn pairs_from_cad(
    h: &Hatch,
    p: CoordinateFrame3,
    b: &[HatchBoundary2],
) -> Result<Vec<GeometryPair>, CadHatchPreparationError> {
    pairs(h, p, b, false)
}
pub(super) fn pairs_to_cad(
    h: &Hatch,
    p: CoordinateFrame3,
    b: &[HatchBoundary2],
) -> Result<Vec<GeometryPair>, CadHatchPreparationError> {
    pairs(h, p, b, true)
}
fn pairs(
    h: &Hatch,
    p: CoordinateFrame3,
    b: &[HatchBoundary2],
    export: bool,
) -> Result<Vec<GeometryPair>, CadHatchPreparationError> {
    let cf = Frame::cad(h);
    let nf = Frame::native(p);
    b.iter()
        .zip(&h.paths)
        .map(|(b, path)| {
            let n = native(&nf, b);
            // A full circle has no authored start. Choose a correspondence
            // aligned with native X, without changing the stored CAD contour.
            let phase = if matches!(b, HatchBoundary2::Circle { .. }) {
                let x: Q = nf.x.iter().zip(&cf.x).map(|(a, b)| a * b).sum();
                let y: Q = nf.x.iter().zip(&cf.y).map(|(a, b)| a * b).sum();
                let x = crate::geometry::numeric::round_nearest(&x)
                    .map_err(|_| CadPreparationError::OutOfRange)?;
                let y = crate::geometry::numeric::round_nearest(&y)
                    .map_err(|_| CadPreparationError::OutOfRange)?;
                y.atan2(x)
            } else {
                0.
            };
            let c = cad(&cf, path, phase)?;
            let (s, t) = if export { (n, c) } else { (c, n) };
            if s.points.len() != t.points.len() || s.curves.len() != t.curves.len() {
                return Err(CadPreparationError::InvalidGeometry.into());
            }
            let points = s
                .points
                .into_iter()
                .zip(t.points)
                .map(|(s, t)| {
                    let p = PairedPoint::new(s, t);
                    let d = p.squared_deviation().1;
                    (p, d)
                })
                .collect();
            let curves = s
                .curves
                .into_iter()
                .zip(t.curves)
                .map(|((sc, sa, ss), (tc, ta, ts))| {
                    PairedCurve::new(sc, sa, tc, ta).with_angular_drift((ss - ts).abs())
                })
                .collect();
            Ok(GeometryPair { points, curves })
        })
        .collect()
}
