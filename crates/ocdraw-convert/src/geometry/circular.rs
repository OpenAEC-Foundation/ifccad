use super::blocks::{trig, PairedCurve, PairedPoint, Range};
use super::numeric::exact;
use super::{cad_plane, orthonormal_pair, stored_normal};
use crate::OcdrawToCadError;
use num_rational::BigRational;
use num_traits::Signed;
use ocdraw::ocdraw::{CoordinateFrame3, Point3};
use opencadcodec::Vector3;

type CurveComponents = ([BigRational; 3], [[Range; 3]; 2]);

fn parameter_axes(
    cosine: [BigRational; 3],
    sine: [BigRational; 3],
    start: f64,
    direction: f64,
) -> [[Range; 3]; 2] {
    let (sin_start, cos_start) = trig(start);
    let first =
        std::array::from_fn(|i| cos_start.scale(&cosine[i]).add(&sin_start.scale(&sine[i])));
    let second = std::array::from_fn(|i| {
        cos_start
            .scale(&sine[i])
            .add(&sin_start.scale(&-cosine[i].clone()))
            .scale(&exact(direction))
    });
    [first, second]
}

fn native_components(
    plane: CoordinateFrame3,
    major: f64,
    minor: f64,
    start: f64,
    direction: f64,
) -> CurveComponents {
    let origin = plane.origin();
    let x = plane.x_axis();
    let y = plane.y_axis();
    let center = [origin.x(), origin.y(), origin.z()].map(exact);
    let cosine = [x.x(), x.y(), x.z()].map(|v| exact(v) * exact(major));
    let sine = [y.x(), y.y(), y.z()].map(|v| exact(v) * exact(minor));
    (center, parameter_axes(cosine, sine, start, direction))
}

fn cad_circular_components(
    center: Vector3,
    normal: Vector3,
    radius: f64,
    start: f64,
    direction: f64,
) -> Option<CurveComponents> {
    let basis = opencadcodec::types::Matrix3::arbitrary_axis(normal);
    let world_center = basis * center;
    if ![world_center.x, world_center.y, world_center.z]
        .into_iter()
        .chain(basis.m.into_iter().flatten())
        .chain([radius, start])
        .all(f64::is_finite)
    {
        return None;
    }
    let center = [world_center.x, world_center.y, world_center.z].map(exact);
    let basis = basis.m;
    let cosine = [0, 1, 2].map(|i| exact(basis[i][0]) * exact(radius));
    let sine = [0, 1, 2].map(|i| exact(basis[i][1]) * exact(radius));
    Some((center, parameter_axes(cosine, sine, start, direction)))
}

fn cad_ellipse_components(
    ellipse: &opencadcodec::Ellipse,
    start: f64,
    direction: f64,
) -> Option<CurveComponents> {
    let basis = cad_plane(ellipse.normal)?;
    let axis = ellipse.major_axis;
    let length = axis.x.hypot(axis.y).hypot(axis.z);
    if !length.is_finite() || length <= 0.0 || !start.is_finite() {
        return None;
    }
    let x = [axis.x / length, axis.y / length, axis.z / length];
    let y = [
        basis.n[1] * x[2] - basis.n[2] * x[1],
        basis.n[2] * x[0] - basis.n[0] * x[2],
        basis.n[0] * x[1] - basis.n[1] * x[0],
    ];
    let minor_length = length * ellipse.minor_axis_ratio;
    if !minor_length.is_finite()
        || !y.into_iter().all(f64::is_finite)
        || ![
            ellipse.center.x,
            ellipse.center.y,
            ellipse.center.z,
            axis.x,
            axis.y,
            axis.z,
        ]
        .into_iter()
        .all(f64::is_finite)
    {
        return None;
    }
    let center = [ellipse.center.x, ellipse.center.y, ellipse.center.z].map(exact);
    let cosine = [axis.x, axis.y, axis.z].map(exact);
    let sine = y.map(|v| exact(v) * exact(minor_length));
    Some((center, parameter_axes(cosine, sine, start, direction)))
}

fn pair_components(source: CurveComponents, target: CurveComponents) -> PairedCurve {
    PairedCurve::new(source.0, source.1, target.0, target.1)
}

pub(crate) fn import_circle_curve(
    plane: CoordinateFrame3,
    radius: f64,
    phase: f64,
    target: &opencadcodec::Circle,
) -> Option<PairedCurve> {
    Some(pair_components(
        native_components(plane, radius, radius, 0.0, 1.0),
        cad_circular_components(target.center, target.normal, target.radius, phase, 1.0)?,
    ))
}

pub(crate) fn import_arc_curve(
    plane: CoordinateFrame3,
    radius: f64,
    start: f64,
    sweep: f64,
    target: &opencadcodec::Arc,
) -> Option<PairedCurve> {
    let drift = (exact(target.end_angle) - exact(target.start_angle) - exact(sweep.abs())).abs();
    Some(
        pair_components(
            native_components(plane, radius, radius, start, sweep.signum()),
            cad_circular_components(
                target.center,
                target.normal,
                target.radius,
                target.start_angle,
                1.0,
            )?,
        )
        .with_angular_drift(drift)
        .with_span(sweep.abs()),
    )
}

pub(crate) fn import_ellipse_curve(
    plane: CoordinateFrame3,
    major: f64,
    minor: f64,
    start: f64,
    sweep: f64,
    target: &opencadcodec::Ellipse,
) -> Option<PairedCurve> {
    let drift =
        (exact(target.end_parameter) - exact(target.start_parameter) - exact(sweep.abs())).abs();
    Some(
        pair_components(
            native_components(plane, major, minor, start, sweep.signum()),
            cad_ellipse_components(target, target.start_parameter, 1.0)?,
        )
        .with_angular_drift(drift)
        .with_span(sweep.abs()),
    )
}

pub(crate) fn export_circle_curve(
    source: &opencadcodec::Circle,
    plane: CoordinateFrame3,
) -> Option<PairedCurve> {
    Some(pair_components(
        cad_circular_components(source.center, source.normal, source.radius, 0.0, 1.0)?,
        native_components(plane, source.radius, source.radius, 0.0, 1.0),
    ))
}

pub(crate) fn export_arc_curve(
    source: &opencadcodec::Arc,
    plane: CoordinateFrame3,
    sweep: f64,
) -> Option<PairedCurve> {
    let source_sweep = exact(source.end_angle) - exact(source.start_angle);
    let source_sweep = if source_sweep.is_negative() {
        source_sweep + exact(std::f64::consts::TAU)
    } else {
        source_sweep
    };
    let drift = (source_sweep - exact(sweep)).abs();
    Some(
        pair_components(
            cad_circular_components(source.center, source.normal, source.radius, 0.0, 1.0)?,
            native_components(plane, source.radius, source.radius, 0.0, 1.0),
        )
        .with_angular_drift(drift)
        .with_span(sweep),
    )
}

pub(crate) fn export_ellipse_curve(
    source: &opencadcodec::Ellipse,
    plane: CoordinateFrame3,
    major: f64,
    minor: f64,
    sweep: f64,
) -> Option<PairedCurve> {
    let source_sweep = exact(source.end_parameter) - exact(source.start_parameter);
    let source_sweep = if source_sweep.is_negative() {
        source_sweep + exact(std::f64::consts::TAU)
    } else {
        source_sweep
    };
    let drift = (source_sweep - exact(sweep)).abs();
    Some(
        pair_components(
            cad_ellipse_components(source, 0.0, 1.0)?,
            native_components(plane, major, minor, 0.0, 1.0),
        )
        .with_angular_drift(drift)
        .with_span(sweep),
    )
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn add3(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0] + c[0], a[1] + b[1] + c[1], a[2] + b[2] + c[2]]
}
fn scaled(a: [f64; 3], value: f64) -> [f64; 3] {
    [a[0] * value, a[1] * value, a[2] * value]
}
fn vec3(a: [f64; 3]) -> Vector3 {
    Vector3::new(a[0], a[1], a[2])
}
fn ifc_vec3(a: [f64; 3]) -> ocdraw::ocdraw::Vector3 {
    ocdraw::ocdraw::Vector3::new(a[0], a[1], a[2])
}

pub(crate) fn from_cad_ocs(center: Vector3, normal: Vector3) -> Option<CoordinateFrame3> {
    let basis = cad_plane(normal)?;
    let origin = add3(
        scaled(basis.u, center.x),
        scaled(basis.v, center.y),
        scaled(basis.n, center.z),
    );
    if !origin.into_iter().all(f64::is_finite) {
        return None;
    }
    CoordinateFrame3::try_new(
        Point3::new(origin[0], origin[1], origin[2]),
        ifc_vec3(basis.u),
        ifc_vec3(basis.v),
    )
    .ok()
}

pub(crate) fn to_cad_ocs(
    placement: CoordinateFrame3,
    reverse: bool,
) -> Option<(Vector3, Vector3, f64)> {
    let mut normal = stored_normal(placement)?;
    if reverse {
        normal = Vector3::new(-normal.x, -normal.y, -normal.z);
    }
    let basis = cad_plane(normal)?;
    let origin = placement.origin();
    let center = [origin.x(), origin.y(), origin.z()];
    let target = [
        dot(basis.u, center),
        dot(basis.v, center),
        dot(basis.n, center),
    ];
    if !target.into_iter().all(f64::is_finite) {
        return None;
    }
    let x = placement.x_axis();
    let x = [x.x(), x.y(), x.z()];
    let angle = dot(basis.v, x).atan2(dot(basis.u, x));
    Some((vec3(target), normal, angle))
}

fn paired(source: Point3, target: Vector3) -> (PairedPoint, BigRational) {
    let source = [source.x(), source.y(), source.z()].map(exact);
    let target = [target.x, target.y, target.z].map(exact);
    let squared = (0..3)
        .map(|i| {
            let delta = &source[i] - &target[i];
            &delta * &delta
        })
        .sum();
    (PairedPoint::new(source, target), squared)
}
fn paired_export(source: Vector3, target: Point3) -> (PairedPoint, BigRational) {
    let source = [source.x, source.y, source.z].map(exact);
    let target = [target.x(), target.y(), target.z()].map(exact);
    let squared = (0..3)
        .map(|i| {
            let delta = &source[i] - &target[i];
            &delta * &delta
        })
        .sum();
    (PairedPoint::new(source, target), squared)
}
fn source_point(
    plane: CoordinateFrame3,
    radius: f64,
    angle: f64,
) -> Result<Point3, OcdrawToCadError> {
    plane
        .try_to_scope_point(ocdraw::ocdraw::Point2::new(
            radius * angle.cos(),
            radius * angle.sin(),
        ))
        .map_err(|_| OcdrawToCadError::Cad("validated curve sample exceeds finite range".into()))
}
pub(crate) fn circle_sample_pairs(
    plane: CoordinateFrame3,
    radius: f64,
    phase: f64,
    target: &opencadcodec::Circle,
) -> Result<Vec<(PairedPoint, BigRational)>, OcdrawToCadError> {
    [
        0.0,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
        3.0 * std::f64::consts::FRAC_PI_2,
    ]
    .into_iter()
    .map(|t| {
        Ok(paired(
            source_point(plane, radius, t)?,
            target.point_at_angle_wcs(phase + t),
        ))
    })
    .collect()
}
pub(crate) fn arc_sample_pairs(
    plane: CoordinateFrame3,
    radius: f64,
    start: f64,
    sweep: f64,
    target: &opencadcodec::Arc,
) -> Result<Vec<(PairedPoint, BigRational)>, OcdrawToCadError> {
    [0.0, 0.5, 1.0]
        .into_iter()
        .map(|fraction| {
            let source = source_point(plane, radius, start + fraction * sweep)?;
            let target = target.point_at_angle_wcs(target.start_angle + fraction * sweep.abs());
            Ok(paired(source, target))
        })
        .collect()
}
pub(crate) fn export_circle_sample_pairs(
    source: &opencadcodec::Circle,
    plane: CoordinateFrame3,
) -> Option<Vec<(PairedPoint, BigRational)>> {
    [
        0.0,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
        3.0 * std::f64::consts::FRAC_PI_2,
    ]
    .into_iter()
    .map(|angle| {
        let target = plane
            .try_to_scope_point(ocdraw::ocdraw::Point2::new(
                source.radius * angle.cos(),
                source.radius * angle.sin(),
            ))
            .ok()?;
        Some(paired_export(source.point_at_angle_wcs(angle), target))
    })
    .collect()
}
pub(crate) fn export_arc_sample_pairs(
    source: &opencadcodec::Arc,
    plane: CoordinateFrame3,
    sweep: f64,
) -> Option<Vec<(PairedPoint, BigRational)>> {
    [0.0, 0.5, 1.0]
        .into_iter()
        .map(|fraction| {
            let angle = source.start_angle + fraction * sweep;
            let target = plane
                .try_to_scope_point(ocdraw::ocdraw::Point2::new(
                    source.radius * angle.cos(),
                    source.radius * angle.sin(),
                ))
                .ok()?;
            Some(paired_export(source.point_at_angle_wcs(angle), target))
        })
        .collect()
}

pub(crate) fn from_cad_ellipse(
    source: &opencadcodec::Ellipse,
) -> Option<(CoordinateFrame3, f64, f64)> {
    let basis = cad_plane(source.normal)?;
    let a = source.major_axis;
    let scale = a.x.abs().max(a.y.abs()).max(a.z.abs());
    if !scale.is_finite() || scale == 0.0 {
        return None;
    }
    let length = scale * (a.x / scale).hypot(a.y / scale).hypot(a.z / scale);
    if !length.is_finite() || length <= 0.0 {
        return None;
    }
    let x = [a.x / length, a.y / length, a.z / length];
    if dot(x, basis.n).abs() > 1e-12 {
        return None;
    }
    let y = [
        basis.n[1] * x[2] - basis.n[2] * x[1],
        basis.n[2] * x[0] - basis.n[0] * x[2],
        basis.n[0] * x[1] - basis.n[1] * x[0],
    ];
    let (x, y) = orthonormal_pair(vec3(x), vec3(y))?;
    let ratio = source.minor_axis_ratio;
    let minor = length * ratio;
    if !ratio.is_finite() || ratio <= 0.0 || ratio > 1.0 || !minor.is_finite() || minor <= 0.0 {
        return None;
    }
    let center = source.center;
    let placement = CoordinateFrame3::try_new(
        Point3::new(center.x, center.y, center.z),
        ocdraw::ocdraw::Vector3::new(x.x, x.y, x.z),
        ocdraw::ocdraw::Vector3::new(y.x, y.y, y.z),
    )
    .ok()?;
    Some((placement, length, minor))
}

pub(crate) fn to_cad_ellipse(
    placement: CoordinateFrame3,
    major: f64,
    minor: f64,
    reverse: bool,
) -> Option<opencadcodec::Ellipse> {
    let mut normal = stored_normal(placement)?;
    if reverse {
        normal = Vector3::new(-normal.x, -normal.y, -normal.z);
    }
    let x = placement.x_axis();
    let axis = Vector3::new(major * x.x(), major * x.y(), major * x.z());
    let center = placement.origin();
    let mut target = opencadcodec::Ellipse::from_center_axes(
        Vector3::new(center.x(), center.y(), center.z()),
        axis,
        minor / major,
    );
    target.normal = normal;
    Some(target)
}

fn ellipse_point(source: &opencadcodec::Ellipse, angle: f64) -> Option<Vector3> {
    let basis = cad_plane(source.normal)?;
    let axis = source.major_axis;
    let length = axis.x.hypot(axis.y).hypot(axis.z);
    let x = [axis.x / length, axis.y / length, axis.z / length];
    let y = [
        basis.n[1] * x[2] - basis.n[2] * x[1],
        basis.n[2] * x[0] - basis.n[0] * x[2],
        basis.n[0] * x[1] - basis.n[1] * x[0],
    ];
    let major = scaled([axis.x, axis.y, axis.z], angle.cos());
    let minor = scaled(y, length * source.minor_axis_ratio * angle.sin());
    let center = [source.center.x, source.center.y, source.center.z];
    Some(vec3(add3(center, major, minor)))
}

pub(crate) fn ellipse_sample_pairs(
    plane: CoordinateFrame3,
    major: f64,
    minor: f64,
    start: f64,
    sweep: f64,
    target: &opencadcodec::Ellipse,
) -> Option<Vec<(PairedPoint, BigRational)>> {
    [0.0, 0.5, 1.0]
        .into_iter()
        .map(|fraction| {
            let source_angle = start + fraction * sweep;
            let target_angle = target.start_parameter + fraction * sweep.abs();
            let source = plane
                .try_to_scope_point(ocdraw::ocdraw::Point2::new(
                    major * source_angle.cos(),
                    minor * source_angle.sin(),
                ))
                .ok()?;
            Some(paired(source, ellipse_point(target, target_angle)?))
        })
        .collect()
}

pub(crate) fn export_ellipse_sample_pairs(
    source: &opencadcodec::Ellipse,
    plane: CoordinateFrame3,
    major: f64,
    minor: f64,
    start: f64,
    sweep: f64,
) -> Option<Vec<(PairedPoint, BigRational)>> {
    [0.0, 0.5, 1.0]
        .into_iter()
        .map(|fraction| {
            let angle = start + fraction * sweep;
            let target = plane
                .try_to_scope_point(ocdraw::ocdraw::Point2::new(
                    major * angle.cos(),
                    minor * angle.sin(),
                ))
                .ok()?;
            Some(paired_export(ellipse_point(source, angle)?, target))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_ellipse_certifies_minor_axis_between_old_samples() {
        let placement = CoordinateFrame3::default();
        let target = opencadcodec::Ellipse::from_center_axes(
            Vector3::ZERO,
            Vector3::new(4.0, 0.0, 0.0),
            0.45,
        );
        let samples =
            ellipse_sample_pairs(placement, 4.0, 2.0, 0.0, std::f64::consts::TAU, &target).unwrap();
        assert!(samples.iter().all(|(_, squared)| squared < &exact(1e-20)));
        let curve =
            import_ellipse_curve(placement, 4.0, 2.0, 0.0, std::f64::consts::TAU, &target).unwrap();
        let (_, upper) = curve.squared_deviation().unwrap();
        assert!(upper > exact(0.039));
    }
    #[test]
    fn flipped_arc_frame_keeps_start_point_and_reverses_orientation() {
        let placement = CoordinateFrame3::default();
        let (_, normal, angle) = to_cad_ocs(placement, true).unwrap();
        let basis = cad_plane(normal).unwrap();
        let start = add3(
            scaled(basis.u, angle.cos()),
            scaled(basis.v, angle.sin()),
            [0.0; 3],
        );
        assert!((start[0] - 1.0).abs() < 1e-14);
        assert!(normal.z < 0.0);
    }
}
