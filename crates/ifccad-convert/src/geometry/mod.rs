pub(crate) mod numeric;
use cadcodec::types::Matrix3;
use cadcodec::Vector3;
#[derive(Clone, Copy, Debug)]
pub(crate) struct CadPlane {
    pub u: [f64; 3],
    pub v: [f64; 3],
    pub n: [f64; 3],
}
fn normalized(v: Vector3) -> Option<Vector3> {
    if ![v.x, v.y, v.z].into_iter().all(f64::is_finite) {
        return None;
    }
    let scale = v.x.abs().max(v.y.abs()).max(v.z.abs());
    if scale == 0.0 {
        return None;
    }
    let v = Vector3::new(v.x / scale, v.y / scale, v.z / scale);
    let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    Some(Vector3::new(v.x / length, v.y / length, v.z / length))
}
pub(crate) fn cad_plane(normal: Vector3) -> Option<CadPlane> {
    let normal = normalized(normal)?;
    let m = Matrix3::arbitrary_axis(normal).m;
    let p = CadPlane {
        u: [m[0][0], m[1][0], m[2][0]],
        v: [m[0][1], m[1][1], m[2][1]],
        n: [m[0][2], m[1][2], m[2][2]],
    };
    PlanePlacement::try_new(Point3::new(0., 0., 0.), cv(p.u), cv(p.v)).ok()?;
    Some(p)
}

use crate::{
    ConversionEntitySource, ConversionGeometryAssessment, ConversionGeometryFailure,
    ConversionGeometryFailureReason as Reason, ConversionGeometryStage as Stage,
};
use cadcodec::{LwPolyline, Vector2};
use ifccad::ifcdr::{PlanePlacement, Point3, PolylineRef};
use num_rational::BigRational;
use numeric::{exact, round_nearest};
fn cv(v: [f64; 3]) -> ifccad::ifcdr::Vector3 {
    ifccad::ifcdr::Vector3::new(v[0], v[1], v[2])
}
fn components(plane: PlanePlacement) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let (o, x, y) = (plane.origin(), plane.x_axis(), plane.y_axis());
    (
        [o.x(), o.y(), o.z()],
        [x.x(), x.y(), x.z()],
        [y.x(), y.y(), y.z()],
    )
}
fn dot(a: [f64; 3], b: [f64; 3]) -> BigRational {
    a.into_iter().zip(b).map(|(a, b)| exact(a) * exact(b)).sum()
}
fn stored_normal(plane: PlanePlacement) -> Option<Vector3> {
    let (_, x, y) = components(plane);
    let mut n = [0.0; 3];
    for (i, component) in n.iter_mut().enumerate() {
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;
        *component =
            round_nearest(&(exact(x[j]) * exact(y[k]) - exact(x[k]) * exact(y[j]))).ok()?;
    }
    normalized(Vector3::new(n[0], n[1], n[2]))
}

// Fixed exact operands belong to one polyline. Keep arbitrary-axis construction
// and final nearest rounding unchanged; only avoid rebuilding rational operands.
struct PreparedProjection {
    origin: [BigRational; 3],
    x: [BigRational; 3],
    y: [BigRational; 3],
    target_origin: [BigRational; 3],
    u: [BigRational; 3],
    v: [BigRational; 3],
    coefficients: [[BigRational; 3]; 2],
}
impl PreparedProjection {
    fn new(o: [f64; 3], x: [f64; 3], y: [f64; 3], basis: CadPlane, elevation: f64) -> Self {
        let (origin, x, y) = (o.map(exact), x.map(exact), y.map(exact));
        let (u, v) = (basis.u.map(exact), basis.v.map(exact));
        let e = exact(elevation);
        let target_origin = basis.n.map(|n| &e * exact(n));
        let coefficients = [&u, &v].map(|axis| {
            [&origin, &x, &y].map(|operand| axis.iter().zip(operand).map(|(a, b)| a * b).sum())
        });
        Self {
            origin,
            x,
            y,
            target_origin,
            u,
            v,
            coefficients,
        }
    }
    fn project(
        &self,
        point: [f64; 2],
    ) -> Result<([f64; 2], BigRational), numeric::NumericRangeError> {
        let local = point.map(exact);
        let mut uv = [0.; 2];
        for (value, c) in uv.iter_mut().zip(&self.coefficients) {
            *value = round_nearest(&(&c[0] + &local[0] * &c[1] + &local[1] * &c[2]))?;
        }
        let target = uv.map(exact);
        let d2 = (0..3)
            .map(|i| {
                let before = &self.origin[i] + &local[0] * &self.x[i] + &local[1] * &self.y[i];
                let after =
                    &self.target_origin[i] + &target[0] * &self.u[i] + &target[1] * &self.v[i];
                let delta = after - before;
                &delta * &delta
            })
            .sum();
        Ok((uv, d2))
    }
}
pub(crate) fn from_cad(
    poly: &LwPolyline,
    assessment: &mut ConversionGeometryAssessment,
) -> Result<(PlanePlacement, f64, bool), Box<ConversionGeometryFailure>> {
    let source = ConversionEntitySource::CadEntity {
        handle: poly.common.handle,
        kind: "LWPOLYLINE".into(),
    };
    let fail = |stage, reason| assessment.failure(&source, None, stage, reason);
    let basis = cad_plane(poly.normal)
        .ok_or_else(|| fail(Stage::SourceEvaluation, Reason::CadAxisEvaluationFailed))?;
    let mut o = [0.0; 3];
    for (i, component) in o.iter_mut().enumerate() {
        *component = round_nearest(&(exact(poly.elevation) * exact(basis.n[i]))).map_err(|_| {
            fail(
                Stage::TargetConstruction,
                Reason::TargetCoordinateOutOfRange,
            )
        })?;
    }
    let plane = PlanePlacement::try_new(Point3::new(o[0], o[1], o[2]), cv(basis.u), cv(basis.v))
        .map_err(|_| fail(Stage::SourceEvaluation, Reason::CadAxisEvaluationFailed))?;
    // The validated frame bounds each direction component near one. Ordinary
    // magnitudes are provably in range; only extreme inputs need exact point
    // evaluation here. A target-range failure must not become an entity skip.
    let large_origin = o.into_iter().any(|v| v.abs() > f64::MAX / 8.0);
    for (index, vertex) in poly.vertices.iter().enumerate() {
        if large_origin
            || vertex.location.x.abs() > f64::MAX / 8.0
            || vertex.location.y.abs() > f64::MAX / 8.0
        {
            plane
                .try_to_scope_point(ifccad::ifcdr::Point2::new(
                    vertex.location.x,
                    vertex.location.y,
                ))
                .map_err(|_| {
                    assessment.failure(
                        &source,
                        Some(index),
                        Stage::TargetConstruction,
                        Reason::TargetCoordinateOutOfRange,
                    )
                })?;
        }
    }
    // Identical local vertices and axes make the residual a constant origin shift.
    let d2 = (0..3)
        .map(|i| {
            let d = exact(o[i]) - exact(poly.elevation) * exact(basis.n[i]);
            &d * &d
        })
        .sum();
    let bound = assessment.check(&source, 0, &d2)?;
    let normal_changed = stored_normal(plane) != Some(poly.normal);
    assessment.record(source, poly.vertices.len(), bound);
    Ok((plane, bound, normal_changed))
}
pub(crate) fn to_cad(
    poly: PolylineRef<'_>,
    source: ConversionEntitySource,
    assessment: &mut ConversionGeometryAssessment,
) -> Result<(LwPolyline, f64, bool), Box<ConversionGeometryFailure>> {
    let plane = poly.placement();
    let (o, x, y) = components(plane);
    let normal = stored_normal(plane).ok_or_else(|| {
        assessment.failure(
            &source,
            None,
            Stage::TargetConstruction,
            Reason::CadAxisEvaluationFailed,
        )
    })?;
    let basis = cad_plane(normal).ok_or_else(|| {
        assessment.failure(
            &source,
            None,
            Stage::TargetConstruction,
            Reason::CadAxisEvaluationFailed,
        )
    })?;
    let elevation = round_nearest(&dot(basis.n, o)).map_err(|_| {
        assessment.failure(
            &source,
            None,
            Stage::TargetConstruction,
            Reason::TargetCoordinateOutOfRange,
        )
    })?;
    let direct = x == basis.u
        && y == basis.v
        && (0..3).all(|i| exact(o[i]) == exact(elevation) * exact(basis.n[i]));
    let prepared = (!direct).then(|| PreparedProjection::new(o, x, y, basis, elevation));
    let mut points = Vec::with_capacity(poly.local_points().len());
    let mut maximum = 0.0_f64;
    for (index, p) in poly.local_points().enumerate() {
        let mut uv = [p.x(), p.y()];
        if let Some(prepared) = &prepared {
            let (projected, d2) = prepared.project(uv).map_err(|_| {
                assessment.failure(
                    &source,
                    Some(index),
                    Stage::TargetConstruction,
                    Reason::TargetCoordinateOutOfRange,
                )
            })?;
            uv = projected;
            maximum = maximum.max(assessment.check(&source, index, &d2)?);
        }
        points.push(Vector2::new(uv[0], uv[1]));
    }
    assessment.record(source, points.len(), maximum);
    let mut target = LwPolyline::from_points(points);
    target.normal = normal;
    target.elevation = elevation;
    target.is_closed = poly.closed();
    Ok((target, maximum, !direct))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prepared_projection_rounds_once_and_measures_the_unrounded_residual() {
        let basis = CadPlane {
            u: [1., 0., 0.],
            v: [0., 1., 0.],
            n: [0., 0., 1.],
        };
        let large = 9007199254740992.; // 2^53: adjacent positive floats differ by two.
        let p = PreparedProjection::new([large, 0., 7.], basis.u, basis.v, basis, 7.);
        for (input, expected) in [([1., 2.], [large, 2.]), ([3., -4.], [large + 4., -4.])] {
            let (uv, squared_error) = p.project(input).unwrap();
            assert_eq!(uv, expected);
            assert_eq!(squared_error, BigRational::from_integer(1.into()));
        }
        let p = PreparedProjection::new([large, 0., 7.], [-1., 0., 0.], [0., -1., 0.], basis, 7.);
        let (uv, squared_error) = p.project([large, 3.]).unwrap();
        assert_eq!(uv, [0., -3.]);
        assert_eq!(squared_error, BigRational::from_integer(0.into()));
        let p = PreparedProjection::new([f64::MAX, 0., 0.], basis.u, basis.v, basis, 0.);
        assert!(p.project([f64::MAX, 0.]).is_err());
    }
    #[test]
    fn arbitrary_axis_threshold_uses_actual_normal_and_strict_branch() {
        let threshold = 1.0_f64 / 64.0;
        let mut seen = [false; 3];
        for offset in -4_i64..=4 {
            let a = f64::from_bits((threshold.to_bits() as i64 + offset) as u64);
            let p = cad_plane(Vector3::new(a, 0., (1. - a * a).sqrt())).unwrap();
            let ordering = p.n[0].total_cmp(&threshold);
            seen[match ordering {
                std::cmp::Ordering::Less => 0,
                std::cmp::Ordering::Equal => 1,
                std::cmp::Ordering::Greater => 2,
            }] = true;
            if p.n[0] < threshold {
                assert_eq!(p.u[1], 0.);
                assert!(p.u[0] > 0. && p.u[2] < 0.);
            } else {
                assert_eq!(p.u, [0., 1., 0.]);
            }
        }
        assert_eq!(seen, [true; 3]);
        assert_eq!(
            cad_plane(Vector3::new(0., 0., f64::from_bits(1)))
                .unwrap()
                .u,
            [1., 0., 0.]
        );
    }

    #[test]
    fn cad_axes_match_independent_axis_cases_and_reject_invalid_normals() {
        for (n, u, v) in [
            (Vector3::UNIT_Z, [1., 0., 0.], [0., 1., 0.]),
            (Vector3::new(0., 0., -1.), [-1., 0., 0.], [0., 1., 0.]),
            (Vector3::UNIT_X, [0., 1., 0.], [0., 0., 1.]),
            (Vector3::new(0., 0., f64::MAX), [1., 0., 0.], [0., 1., 0.]),
        ] {
            let p = cad_plane(n).unwrap();
            assert_eq!(p.u, u);
            assert_eq!(p.v, v);
        }
        assert!(cad_plane(Vector3::new(0., 0., 0.)).is_none());
        assert!(cad_plane(Vector3::new(f64::NAN, 0., 1.)).is_none());
    }
}
