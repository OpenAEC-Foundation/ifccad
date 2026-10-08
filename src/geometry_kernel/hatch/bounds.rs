use super::{
    endpoints::{curve_rays, ArcRays, Range},
    validation::parameters,
    HatchBoundary2, HatchEdge2, HatchFailureReason as Reason, HatchValidationError as Error,
};
use crate::geometry_kernel::{
    numeric::{exact, round_down, round_up},
    Bounds3d, CoordinateFrame3, Point3,
};
use num_rational::BigRational as Q;
use num_traits::{Signed, Zero};
use std::cmp::Ordering;

/// Certified coordinate projections of stored contours; no filled-area evaluation.
pub fn hatch_bounds<'a>(
    plane: CoordinateFrame3,
    boundaries: impl IntoIterator<Item = &'a HatchBoundary2>,
) -> Result<Bounds3d, Error> {
    let mut result = None;
    for (index, boundary) in boundaries.into_iter().enumerate() {
        parameters(boundary, index)?;
        let mut add = |value: Option<Bounds3d>, edge: Option<usize>| -> Result<(), Error> {
            let b = value.ok_or_else(|| Error::at(index, edge, Reason::EnclosureOutOfRange))?;
            result = Some(match result {
                None => b,
                Some(a) => union(a, b),
            });
            Ok(())
        };
        match boundary {
            HatchBoundary2::Circle { center, radius } => add(
                conic(plane, *center, [1.0, 0.0], *radius, *radius, None),
                None,
            )?,
            HatchBoundary2::Ellipse {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
            } => add(
                conic(
                    plane,
                    *center,
                    *x_axis,
                    *semi_major_radius,
                    *semi_minor_radius,
                    None,
                ),
                None,
            )?,
            HatchBoundary2::Polyline { vertices, bulges } => {
                for (i, s) in vertices.iter().enumerate() {
                    add(
                        bulged(plane, *s, vertices[(i + 1) % vertices.len()], bulges[i]),
                        Some(i),
                    )?;
                }
            }
            HatchBoundary2::Edges(edges) => {
                for (i, e) in edges.iter().enumerate() {
                    match e {
                        HatchEdge2::Line { start, end } => {
                            add(straight(plane, *start, *end), Some(i))?
                        }
                        _ => {
                            let c = e.curve().expect("arc");
                            let rays = curve_rays(c).map_err(|r| Error::at(index, Some(i), r))?;
                            add(
                                conic(plane, c.center, c.x_axis, c.major, c.minor, Some(&rays)),
                                Some(i),
                            )?;
                        }
                    }
                }
            }
        }
    }
    result.ok_or(Error {
        loop_index: None,
        edge_index: None,
        reason: Reason::InvalidParameters,
    })
}
fn make_box(v: [(f64, f64); 3]) -> Bounds3d {
    Bounds3d::new(
        Point3::new(v[0].0, v[1].0, v[2].0),
        Point3::new(v[0].1, v[1].1, v[2].1),
    )
}
fn union(a: Bounds3d, b: Bounds3d) -> Bounds3d {
    let al = a.min().components();
    let ah = a.max().components();
    let bl = b.min().components();
    let bh = b.max().components();
    make_box(std::array::from_fn(|i| {
        (al[i].min(bl[i]), ah[i].max(bh[i]))
    }))
}
// Compare against center +/- sqrt(square) before rounding either term;
// huge supporting-circle centers may cancel into finite extrema.
fn algebraic(center: &Q, square: &Q, positive: bool) -> Option<(f64, f64)> {
    if square.is_negative() {
        return None;
    }
    if square.is_zero() {
        return Some((round_down(center).ok()?, round_up(center).ok()?));
    }
    let compare = |v: f64| {
        let d = exact(v) - center;
        if positive {
            if d.is_negative() {
                Ordering::Less
            } else {
                (&d * &d).cmp(square)
            }
        } else if d > Q::zero() {
            Ordering::Greater
        } else {
            (&d * &d).cmp(square).reverse()
        }
    };
    if compare(-f64::MAX) == Ordering::Greater || compare(f64::MAX) == Ordering::Less {
        return None;
    }
    let key = |v: f64| {
        let b = v.to_bits();
        if b >> 63 != 0 {
            !b
        } else {
            b ^ (1_u64 << 63)
        }
    };
    let value = |k: u64| f64::from_bits(if k >> 63 != 0 { k ^ (1_u64 << 63) } else { !k });
    let mut lo = key(-f64::MAX);
    let mut hi = key(f64::MAX);
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        match compare(value(mid)) {
            Ordering::Equal => return Some((value(mid), value(mid))),
            Ordering::Less => lo = mid,
            Ordering::Greater => hi = mid,
        }
    }
    let a = value(lo);
    let b = value(hi);
    if compare(a) == Ordering::Equal {
        Some((a, a))
    } else if compare(b) == Ordering::Equal {
        Some((b, b))
    } else {
        Some((a, b))
    }
}
pub(super) fn sqrt_bracket(square: &Q) -> Option<(f64, f64)> {
    algebraic(&Q::zero(), square, true)
}
fn axis_bounds(center: Q, a: Q, b: Q, square: Q, rays: Option<&ArcRays>) -> Option<(f64, f64)> {
    let full_lo = algebraic(&center, &square, false).map(|x| x.0);
    let full_hi = algebraic(&center, &square, true).map(|x| x.1);
    let Some(rays) = rays else {
        return Some((full_lo?, full_hi?));
    };
    let c = Range {
        lo: center.clone(),
        hi: center,
    };
    let ends =
        [&rays.start, &rays.end].map(|v| c.add(&v[0].scale(a.clone())).add(&v[1].scale(b.clone())));
    let mut lo = round_down(&ends[0].lo.clone().min(ends[1].lo.clone()))
        .ok()
        .or(full_lo)?;
    let mut hi = round_up(&ends[0].hi.clone().max(ends[1].hi.clone()))
        .ok()
        .or(full_hi)?;
    if let Some(x) = full_lo {
        lo = lo.max(x);
    }
    if let Some(x) = full_hi {
        hi = hi.min(x);
    }
    if rays.could_contain([-a.clone(), -b.clone()]) {
        lo = lo.min(full_lo?);
    }
    if rays.could_contain([a, b]) {
        hi = hi.max(full_hi?);
    }
    (lo <= hi).then_some((lo, hi))
}
fn weights(plane: CoordinateFrame3, i: usize) -> (Q, Q, Q) {
    let p = plane.components();
    (
        exact(p.origin.components()[i]),
        exact(p.x.components()[i]),
        exact(p.y.components()[i]),
    )
}
fn projection(plane: CoordinateFrame3, p: [f64; 2], i: usize) -> Q {
    let (o, x, y) = weights(plane, i);
    o + x * exact(p[0]) + y * exact(p[1])
}
fn straight(plane: CoordinateFrame3, s: [f64; 2], e: [f64; 2]) -> Option<Bounds3d> {
    let mut v = [(0.0, 0.0); 3];
    for (i, slot) in v.iter_mut().enumerate() {
        let a = projection(plane, s, i);
        let b = projection(plane, e, i);
        *slot = (
            round_down(&a.clone().min(b.clone())).ok()?,
            round_up(&a.max(b)).ok()?,
        );
    }
    Some(make_box(v))
}
fn conic(
    plane: CoordinateFrame3,
    c: [f64; 2],
    u: [f64; 2],
    major: f64,
    minor: f64,
    rays: Option<&ArcRays>,
) -> Option<Bounds3d> {
    let mut v = [(0.0, 0.0); 3];
    for (i, slot) in v.iter_mut().enumerate() {
        let (o, x, y) = weights(plane, i);
        let center = o + &x * exact(c[0]) + &y * exact(c[1]);
        let a = exact(major) * (&x * exact(u[0]) + &y * exact(u[1]));
        let b = exact(minor) * (-x * exact(u[1]) + y * exact(u[0]));
        let square = &a * &a + &b * &b;
        *slot = axis_bounds(center, a, b, square, rays)?;
    }
    Some(make_box(v))
}
fn bulged(plane: CoordinateFrame3, start: [f64; 2], end: [f64; 2], bulge: f64) -> Option<Bounds3d> {
    if bulge == 0.0 {
        return straight(plane, start, end);
    }
    let s = start.map(exact);
    let e = end.map(exact);
    let dx = &e[0] - &s[0];
    let dy = &e[1] - &s[1];
    let b = exact(bulge);
    let factor = (exact(1.0) / &b - &b) / exact(4.0);
    let center = [
        (&s[0] + &e[0]) / exact(2.0) - &dy * &factor,
        (&s[1] + &e[1]) / exact(2.0) + dx * factor,
    ];
    let rs = std::array::from_fn::<_, 2, _>(|i| &s[i] - &center[i]);
    let re = std::array::from_fn::<_, 2, _>(|i| &e[i] - &center[i]);
    let radius_square = &rs[0] * &rs[0] + &rs[1] * &rs[1];
    let rays = ArcRays {
        start: rs.map(|x| Range {
            lo: x.clone(),
            hi: x,
        }),
        end: re.map(|x| Range {
            lo: x.clone(),
            hi: x,
        }),
        positive: bulge > 0.0,
        minor: bulge.abs() <= 1.0,
    };
    let mut v = [(0.0, 0.0); 3];
    for (i, slot) in v.iter_mut().enumerate() {
        let (o, x, y) = weights(plane, i);
        let c = o + &x * &center[0] + &y * &center[1];
        let square = &radius_square * (&x * &x + &y * &y);
        *slot = axis_bounds(c, x, y, square, Some(&rays))?;
    }
    Some(make_box(v))
}
pub(super) fn local_bounds(boundary: &HatchBoundary2, index: usize) -> Result<[f64; 4], Error> {
    let b = hatch_bounds(CoordinateFrame3::default(), [boundary]).map_err(|mut e| {
        e.loop_index = Some(index);
        e
    })?;
    let lo = b.min().components();
    let hi = b.max().components();
    Ok([lo[0], lo[1], hi[0], hi[1]])
}
