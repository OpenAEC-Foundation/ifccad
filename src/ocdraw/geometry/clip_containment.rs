//! Exact containment of represented paper curves, independent of CAD and tessellation.
use super::numeric::exact;
use crate::ocdraw::logical::{viewport_bounds, DrawingViewportFrame, EntityGeometry};
use crate::ocdraw::{Bounds3d, CoordinateFrame3, Point2};
use num_rational::BigRational;
use num_traits::{Signed, Zero};

fn fits_extremum(
    center: &BigRational,
    amplitude_squared: &BigRational,
    edge: f64,
    upper: bool,
) -> bool {
    let gap = if upper {
        exact(edge) - center
    } else {
        center - exact(edge)
    };
    !gap.is_negative() && amplitude_squared <= &(&gap * &gap)
}
fn paper_plane(p: CoordinateFrame3) -> bool {
    p.origin().z() == 0. && p.x_axis().z() == 0. && p.y_axis().z() == 0.
}
fn full_curve(p: CoordinateFrame3, major: f64, minor: f64, bounds: Bounds3d) -> bool {
    if !paper_plane(p) {
        return false;
    }
    let o = p.origin().components();
    let x = p.x_axis().components();
    let y = p.y_axis().components();
    (0..2).all(|axis| {
        let a = exact(major) * exact(x[axis]);
        let b = exact(minor) * exact(y[axis]);
        let r2 = &a * &a + &b * &b;
        fits_extremum(&exact(o[axis]), &r2, bounds.min().components()[axis], false)
            && fits_extremum(&exact(o[axis]), &r2, bounds.max().components()[axis], true)
    })
}
fn cross(a: &[BigRational; 2], b: &[BigRational; 2]) -> BigRational {
    &a[0] * &b[1] - &a[1] * &b[0]
}
fn on_sweep(
    start: &[BigRational; 2],
    end: &[BigRational; 2],
    ray: &[BigRational; 2],
    bulge: f64,
) -> bool {
    let sign = if bulge > 0. { exact(1.) } else { exact(-1.) };
    let left = cross(start, ray) * &sign;
    let right = cross(ray, end) * sign;
    if bulge.abs() <= 1. {
        !left.is_negative() && !right.is_negative()
    } else {
        !left.is_negative() || !right.is_negative()
    }
}
fn bulged_segment(p: CoordinateFrame3, start: [f64; 3], end: [f64; 3], bounds: Bounds3d) -> bool {
    if !p.enclosed_by(Point2::new(start[0], start[1]), bounds)
        || !p.enclosed_by(Point2::new(end[0], end[1]), bounds)
    {
        return false;
    }
    let b = start[2];
    if b == 0. {
        return true;
    }
    if !paper_plane(p) {
        return false;
    }
    let s = [exact(start[0]), exact(start[1])];
    let e = [exact(end[0]), exact(end[1])];
    let dx = &e[0] - &s[0];
    let dy = &e[1] - &s[1];
    if dx.is_zero() && dy.is_zero() {
        return false;
    }
    let factor = (exact(1.) / exact(b) - exact(b)) / exact(4.);
    let c = [
        (&s[0] + &e[0]) / exact(2.) - &dy * &factor,
        (&s[1] + &e[1]) / exact(2.) + &dx * &factor,
    ];
    let u = [&s[0] - &c[0], &s[1] - &c[1]];
    let v = [&e[0] - &c[0], &e[1] - &c[1]];
    let r2 = &u[0] * &u[0] + &u[1] * &u[1];
    let x = p.x_axis().components();
    let y = p.y_axis().components();
    let o = p.origin().components();
    (0..2).all(|axis| {
        let q = [exact(x[axis]), exact(y[axis])];
        let center = exact(o[axis]) + &q[0] * &c[0] + &q[1] * &c[1];
        let a2 = &r2 * (&q[0] * &q[0] + &q[1] * &q[1]);
        let negative = [-&q[0], -&q[1]];
        (!on_sweep(&u, &v, &q, b)
            || fits_extremum(&center, &a2, bounds.max().components()[axis], true))
            && (!on_sweep(&u, &v, &negative, b)
                || fits_extremum(&center, &a2, bounds.min().components()[axis], false))
    })
}

/// Called only after intrinsic geometry validation; nonfinite input is rejected defensively.
pub(crate) fn enclosed_by_frame(frame: DrawingViewportFrame, geometry: &EntityGeometry) -> bool {
    let Some(bounds) = viewport_bounds(frame) else {
        return false;
    };
    match geometry {
        EntityGeometry::Circle { placement, radius } if radius.is_finite() && *radius > 0. => {
            full_curve(*placement, *radius, *radius, bounds)
        }
        EntityGeometry::Ellipse {
            placement,
            semi_major_radius,
            semi_minor_radius,
            arc: None,
        } if semi_major_radius.is_finite()
            && semi_minor_radius.is_finite()
            && *semi_minor_radius > 0.
            && semi_major_radius >= semi_minor_radius =>
        {
            full_curve(*placement, *semi_major_radius, *semi_minor_radius, bounds)
        }
        EntityGeometry::PlanarPolyline {
            placement,
            vertices,
            closed: true,
            ..
        } if vertices.len() >= 2 && vertices.iter().flatten().all(|v| v.is_finite()) => {
            (0..vertices.len()).all(|i| {
                bulged_segment(
                    *placement,
                    vertices[i],
                    vertices[(i + 1) % vertices.len()],
                    bounds,
                )
            })
        }
        _ => false,
    }
}
