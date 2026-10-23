//! Segment-band enclosures, with deliberately partial evidence for general CAD joins.
use super::numeric::{exact, round_down, round_up};
use super::{
    Bounds3d, CoordinateFrame3, GeometryRef, GeometryValidationError as Error, Point2, Point3,
};
use num_rational::BigRational as Q;
use num_traits::{Signed, ToPrimitive, Zero};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolylineWidthBoundsReason {
    CoincidentWideSegment,
    UnassessedJoin,
    UnassessedPatternInteraction,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineWidthDisplayBounds {
    pub bounds: Option<Bounds3d>,
    pub proven_segment_bounds: Vec<Bounds3d>,
    pub reasons: Vec<PolylineWidthBoundsReason>,
}
fn union(a: &mut Option<Bounds3d>, b: Bounds3d) {
    if let Some(a) = a {
        let lo: [f64; 3] =
            std::array::from_fn(|i| a.min().components()[i].min(b.min().components()[i]));
        let hi: [f64; 3] =
            std::array::from_fn(|i| a.max().components()[i].max(b.max().components()[i]));
        *a = Bounds3d::new(
            Point3::new(lo[0], lo[1], lo[2]),
            Point3::new(hi[0], hi[1], hi[2]),
        );
    } else {
        *a = Some(b)
    }
}
// Certify a unit-normal component through exact squared comparisons. The f64
// square root is only a candidate; even underflowed candidates enter exact search.
fn unit_component(value: &Q, length_squared: &Q) -> (Q, Q) {
    if value.is_zero() {
        return (Q::zero(), Q::zero());
    }
    let square = value * value / length_squared;
    let candidate = square.to_f64().unwrap_or(0.).sqrt();
    let squared = |v: f64| exact(v) * exact(v);
    let (lo, hi) = if squared(candidate) == square {
        (candidate, candidate)
    } else if squared(candidate) < square && squared(candidate.next_up()) >= square {
        (candidate, candidate.next_up())
    } else if candidate > 0.
        && squared(candidate.next_down()) <= square
        && squared(candidate) > square
    {
        (candidate.next_down(), candidate)
    } else {
        let (mut lo, mut hi) = (0u64, 1f64.to_bits());
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if squared(f64::from_bits(mid)) <= square {
                lo = mid
            } else {
                hi = mid
            }
        }
        let lo = f64::from_bits(lo);
        if squared(lo) == square {
            (lo, lo)
        } else {
            (lo, f64::from_bits(hi))
        }
    };
    if value.is_negative() {
        (-exact(hi), -exact(lo))
    } else {
        (exact(lo), exact(hi))
    }
}
fn placed_box(p: CoordinateFrame3, lo: [f64; 2], hi: [f64; 2]) -> Result<Bounds3d, Error> {
    let mut result = None;
    for x in [lo[0], hi[0]] {
        for y in [lo[1], hi[1]] {
            union(
                &mut result,
                p.enclose_point(Point2::new(x, y))
                    .map_err(|_| Error::EnclosureOutOfRange)?,
            );
        }
    }
    result.ok_or(Error::EnclosureOutOfRange)
}
fn tangent(a: [f64; 3], b: [f64; 3], at_end: bool) -> [Q; 2] {
    let dx = exact(b[0]) - exact(a[0]);
    let dy = exact(b[1]) - exact(a[1]);
    let bulge = exact(a[2]);
    let f = exact(1.) - &bulge * &bulge;
    let s = if at_end {
        exact(2.) * bulge
    } else {
        exact(-2.) * bulge
    };
    [&f * &dx - &s * &dy, &f * &dy + &s * &dx]
}
fn continuous(a: [Q; 2], b: [Q; 2]) -> bool {
    &a[0] * &b[1] - &a[1] * &b[0] == Q::zero() && &a[0] * &b[0] + &a[1] * &b[1] > Q::zero()
}
pub fn polyline_width_display_bounds(
    g: GeometryRef<'_>,
) -> Result<PolylineWidthDisplayBounds, Error> {
    let centerline = super::geometry_centerline_bounds(g)?;
    let GeometryRef::PlanarPolyline {
        placement,
        vertices,
        widths,
        closed,
    } = g
    else {
        return Ok(PolylineWidthDisplayBounds {
            bounds: Some(centerline),
            proven_segment_bounds: vec![centerline],
            reasons: vec![],
        });
    };
    let n = vertices.len();
    let count = if closed { n } else { n - 1 };
    let pairs = widths
        .effective_widths(n)
        .map_err(|_| Error::InvalidGeometry)?;
    let wide = |i: usize| pairs[i].iter().any(|v| *v != 0.);
    if !(0..count).any(wide) {
        return Ok(PolylineWidthDisplayBounds {
            bounds: Some(centerline),
            proven_segment_bounds: vec![centerline],
            reasons: vec![],
        });
    }
    let mut result = None;
    let mut proven = Vec::new();
    let mut reasons = Vec::new();
    for (i, pair) in pairs.iter().enumerate().take(count) {
        let a = vertices.get(i);
        let b = vertices.get((i + 1) % n);
        let same = a[0] == b[0] && a[1] == b[1];
        let bounds = if same || !wide(i) || a[2] != 0. {
            if same
                && wide(i)
                && !reasons.contains(&PolylineWidthBoundsReason::CoincidentWideSegment)
            {
                reasons.push(PolylineWidthBoundsReason::CoincidentWideSegment);
            }
            let local =
                super::bulge_segment_bounds(Point2::new(a[0], a[1]), Point2::new(b[0], b[1]), a[2])
                    .ok_or(Error::EnclosureOutOfRange)?;
            let half = if same {
                Q::zero()
            } else {
                exact(pair[0].max(pair[1])) / exact(2.)
            };
            let lo = [
                round_down(&(exact(local.min().x()) - &half)),
                round_down(&(exact(local.min().y()) - &half)),
            ];
            let hi = [
                round_up(&(exact(local.max().x()) + &half)),
                round_up(&(exact(local.max().y()) + &half)),
            ];
            placed_box(
                placement,
                [
                    lo[0].map_err(|_| Error::EnclosureOutOfRange)?,
                    lo[1].map_err(|_| Error::EnclosureOutOfRange)?,
                ],
                [
                    hi[0].map_err(|_| Error::EnclosureOutOfRange)?,
                    hi[1].map_err(|_| Error::EnclosureOutOfRange)?,
                ],
            )?
        } else {
            let d = [exact(b[0]) - exact(a[0]), exact(b[1]) - exact(a[1])];
            let squared = &d[0] * &d[0] + &d[1] * &d[1];
            let normal = [
                unit_component(&-&d[1], &squared),
                unit_component(&d[0], &squared),
            ];
            let mut segment = None;
            for (endpoint, xy) in [(0, [a[0], a[1]]), (1, [b[0], b[1]])] {
                for sign in [-0.5, 0.5] {
                    let scale = exact(pair[endpoint]) * exact(sign);
                    let mut lo = [0.; 2];
                    let mut hi = [0.; 2];
                    for k in 0..2 {
                        let x = &normal[k].0 * &scale;
                        let y = &normal[k].1 * &scale;
                        lo[k] = round_down(&(exact(xy[k]) + x.clone().min(y.clone())))
                            .map_err(|_| Error::EnclosureOutOfRange)?;
                        hi[k] = round_up(&(exact(xy[k]) + x.max(y)))
                            .map_err(|_| Error::EnclosureOutOfRange)?;
                    }
                    union(&mut segment, placed_box(placement, lo, hi)?);
                }
            }
            segment.ok_or(Error::EnclosureOutOfRange)?
        };
        union(&mut result, bounds);
        proven.push(bounds);
    }
    for j in if closed { 0..n } else { 1..n - 1 } {
        let i = (j + n - 1) % n;
        if (wide(i) || wide(j))
            && (pairs[i][1] != pairs[j][0]
                || !continuous(
                    tangent(vertices.get(i), vertices.get(j), true),
                    tangent(vertices.get(j), vertices.get((j + 1) % n), false),
                ))
            && !reasons.contains(&PolylineWidthBoundsReason::UnassessedJoin)
        {
            reasons.push(PolylineWidthBoundsReason::UnassessedJoin);
        }
    }
    Ok(PolylineWidthDisplayBounds {
        bounds: if reasons.is_empty() { result } else { None },
        proven_segment_bounds: proven,
        reasons,
    })
}
