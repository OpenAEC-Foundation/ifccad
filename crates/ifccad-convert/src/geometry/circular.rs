use super::blocks::PairedPoint;
use super::numeric::exact;
use super::{cad_plane, orthonormal_pair, stored_normal};
use crate::ImportError;
use cadcodec::Vector3;
use ifccad::ifcdr::{PlanePlacement, Point3};
use num_rational::BigRational;

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
fn ifc_vec3(a: [f64; 3]) -> ifccad::ifcdr::Vector3 {
    ifccad::ifcdr::Vector3::new(a[0], a[1], a[2])
}

pub(crate) fn from_cad_ocs(center: Vector3, normal: Vector3) -> Option<PlanePlacement> {
    let basis = cad_plane(normal)?;
    let origin = add3(
        scaled(basis.u, center.x),
        scaled(basis.v, center.y),
        scaled(basis.n, center.z),
    );
    if !origin.into_iter().all(f64::is_finite) {
        return None;
    }
    PlanePlacement::try_new(
        Point3::new(origin[0], origin[1], origin[2]),
        ifc_vec3(basis.u),
        ifc_vec3(basis.v),
    )
    .ok()
}

pub(crate) fn to_cad_ocs(
    placement: PlanePlacement,
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
fn source_point(plane: PlanePlacement, radius: f64, angle: f64) -> Result<Point3, ImportError> {
    plane
        .try_to_scope_point(ifccad::ifcdr::Point2::new(
            radius * angle.cos(),
            radius * angle.sin(),
        ))
        .map_err(|_| ImportError::InternalInvariant {
            message: "validated curve sample exceeds finite range".into(),
        })
}
pub(crate) fn circle_sample_pairs(
    plane: PlanePlacement,
    radius: f64,
    phase: f64,
    target: &cadcodec::Circle,
) -> Result<Vec<(PairedPoint, BigRational)>, ImportError> {
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
    plane: PlanePlacement,
    radius: f64,
    start: f64,
    sweep: f64,
    target: &cadcodec::Arc,
) -> Result<Vec<(PairedPoint, BigRational)>, ImportError> {
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
    source: &cadcodec::Circle,
    plane: PlanePlacement,
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
            .try_to_scope_point(ifccad::ifcdr::Point2::new(
                source.radius * angle.cos(),
                source.radius * angle.sin(),
            ))
            .ok()?;
        Some(paired_export(source.point_at_angle_wcs(angle), target))
    })
    .collect()
}
pub(crate) fn export_arc_sample_pairs(
    source: &cadcodec::Arc,
    plane: PlanePlacement,
    sweep: f64,
) -> Option<Vec<(PairedPoint, BigRational)>> {
    [0.0, 0.5, 1.0]
        .into_iter()
        .map(|fraction| {
            let angle = source.start_angle + fraction * sweep;
            let target = plane
                .try_to_scope_point(ifccad::ifcdr::Point2::new(
                    source.radius * angle.cos(),
                    source.radius * angle.sin(),
                ))
                .ok()?;
            Some(paired_export(source.point_at_angle_wcs(angle), target))
        })
        .collect()
}

pub(crate) fn from_cad_ellipse(source: &cadcodec::Ellipse) -> Option<(PlanePlacement, f64, f64)> {
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
    let placement = PlanePlacement::try_new(
        Point3::new(center.x, center.y, center.z),
        ifccad::ifcdr::Vector3::new(x.x, x.y, x.z),
        ifccad::ifcdr::Vector3::new(y.x, y.y, y.z),
    )
    .ok()?;
    Some((placement, length, minor))
}

pub(crate) fn to_cad_ellipse(
    placement: PlanePlacement,
    major: f64,
    minor: f64,
    reverse: bool,
) -> Option<cadcodec::Ellipse> {
    let mut normal = stored_normal(placement)?;
    if reverse {
        normal = Vector3::new(-normal.x, -normal.y, -normal.z);
    }
    let x = placement.x_axis();
    let axis = Vector3::new(major * x.x(), major * x.y(), major * x.z());
    let center = placement.origin();
    let mut target = cadcodec::Ellipse::from_center_axes(
        Vector3::new(center.x(), center.y(), center.z()),
        axis,
        minor / major,
    );
    target.normal = normal;
    Some(target)
}

fn ellipse_point(source: &cadcodec::Ellipse, angle: f64) -> Option<Vector3> {
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
    plane: PlanePlacement,
    major: f64,
    minor: f64,
    start: f64,
    sweep: f64,
    target: &cadcodec::Ellipse,
) -> Option<Vec<(PairedPoint, BigRational)>> {
    [0.0, 0.5, 1.0]
        .into_iter()
        .map(|fraction| {
            let source_angle = start + fraction * sweep;
            let target_angle = target.start_parameter + fraction * sweep.abs();
            let source = plane
                .try_to_scope_point(ifccad::ifcdr::Point2::new(
                    major * source_angle.cos(),
                    minor * source_angle.sin(),
                ))
                .ok()?;
            Some(paired(source, ellipse_point(target, target_angle)?))
        })
        .collect()
}

pub(crate) fn export_ellipse_sample_pairs(
    source: &cadcodec::Ellipse,
    plane: PlanePlacement,
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
                .try_to_scope_point(ifccad::ifcdr::Point2::new(
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
    fn flipped_arc_frame_keeps_start_point_and_reverses_orientation() {
        let placement = PlanePlacement::default();
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
