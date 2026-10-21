//! Ordered CAD corner descriptors. Surface recognition never gates validity.
use super::numeric::{exact, round_down, round_up};
use super::{Bounds3d, CoordinateFrame3, GeometryValidationError as Error, Point3};
use num_rational::BigRational;
use num_traits::{Signed, Zero};

pub(super) fn validate(corners: &[[f64; 2]; 4], thickness: f64) -> Result<(), Error> {
    if corners.iter().flatten().all(|v| v.is_finite()) && thickness.is_finite() {
        Ok(())
    } else {
        Err(Error::InvalidGeometry)
    }
}

pub(super) fn bounds(
    p: CoordinateFrame3,
    corners: &[[f64; 2]; 4],
    h: f64,
) -> Result<Bounds3d, Error> {
    let o = p.origin().components().map(exact);
    let u = p.x_axis().components().map(exact);
    let v = p.y_axis().components().map(exact);
    let n: [BigRational; 3] = std::array::from_fn(|i| {
        &u[(i + 1) % 3] * &v[(i + 2) % 3] - &u[(i + 2) % 3] * &v[(i + 1) % 3]
    });
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for c in corners {
        for top in [false, true] {
            for i in 0..3 {
                let q = &o[i]
                    + &u[i] * exact(c[0])
                    + &v[i] * exact(c[1])
                    + if top {
                        &n[i] * exact(h)
                    } else {
                        BigRational::zero()
                    };
                lo[i] = lo[i].min(round_down(&q).map_err(|_| Error::EnclosureOutOfRange)?);
                hi[i] = hi[i].max(round_up(&q).map_err(|_| Error::EnclosureOutOfRange)?);
            }
        }
    }
    Ok(Bounds3d::new(
        Point3::new(lo[0], lo[1], lo[2]),
        Point3::new(hi[0], hi[1], hi[2]),
    ))
}

/// A recognized simple surface in original zero-based corner slots.
/// This carries no host-rendering or native-validity guarantee.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilledQuadrilateralSurfaceCover {
    Triangle {
        boundary: [usize; 3],
    },
    /// Diagonals 0–3 and 1–2, respectively, in boundary order 0,1,3,2.
    Quadrilateral {
        admissible_diagonals: [bool; 2],
    },
}

fn orient(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> BigRational {
    (exact(b[0]) - exact(a[0])) * (exact(c[1]) - exact(a[1]))
        - (exact(b[1]) - exact(a[1])) * (exact(c[0]) - exact(a[0]))
}
fn on(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    orient(a, b, c).is_zero() && (0..2).all(|i| c[i] >= a[i].min(b[i]) && c[i] <= a[i].max(b[i]))
}
fn intersects(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let ab = [orient(a, b, c), orient(a, b, d)];
    let cd = [orient(c, d, a), orient(c, d, b)];
    on(a, b, c)
        || on(a, b, d)
        || on(c, d, a)
        || on(c, d, b)
        || (ab[0].signum() != ab[1].signum()
            && !ab.iter().any(Zero::is_zero)
            && cd[0].signum() != cd[1].signum()
            && !cd.iter().any(Zero::is_zero))
}

/// Recognizes a common-index simple-polygon cover without modifying corners.
/// Crossing, retracing and collapsed descriptors remain valid but return None.
pub fn filled_quadrilateral_surface_cover(
    corners: &[[f64; 2]; 4],
) -> Option<FilledQuadrilateralSurfaceCover> {
    use FilledQuadrilateralSurfaceCover::*;
    if !corners.iter().flatten().all(|v| v.is_finite()) {
        return None;
    }
    let mut ids = Vec::new();
    for i in [0, 1, 3, 2] {
        if ids.last().is_none_or(|&j| corners[j] != corners[i]) {
            ids.push(i);
        }
    }
    if ids.len() > 1 && corners[ids[0]] == corners[*ids.last()?] {
        ids.pop();
    }
    if ids.len() == 3 {
        let boundary = [ids[0], ids[1], ids[2]];
        return (!orient(corners[ids[0]], corners[ids[1]], corners[ids[2]]).is_zero())
            .then_some(Triangle { boundary });
    }
    if ids.len() != 4 {
        return None;
    }
    let [a, b, c, d] = [corners[0], corners[1], corners[3], corners[2]];
    if intersects(a, b, c, d) || intersects(b, c, d, a) {
        return None;
    }
    let area = orient(a, b, c) + orient(a, c, d);
    if area.is_zero() {
        return None;
    }
    let same = |x: BigRational| x.is_zero() || x.signum() == area.signum();
    let admissible_diagonals = [
        same(orient(a, b, c)) && same(orient(a, c, d)),
        same(orient(a, b, d)) && same(orient(b, c, d)),
    ];
    Some(Quadrilateral {
        admissible_diagonals,
    })
}
