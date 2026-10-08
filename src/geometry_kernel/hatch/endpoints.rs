use super::{bounds::sqrt_bracket, Curve, EndpointKey, HatchEdge2, HatchFailureReason as Reason};
use crate::geometry_kernel::{numeric::exact, trig::sin_cos_interval};
use num_rational::BigRational as Q;
use num_traits::Zero;

#[derive(Clone)]
pub(super) struct Range {
    pub(super) lo: Q,
    pub(super) hi: Q,
}
impl Range {
    pub(super) fn point(x: f64) -> Self {
        let q = exact(x);
        Self {
            lo: q.clone(),
            hi: q,
        }
    }
    pub(super) fn add(&self, b: &Self) -> Self {
        Self {
            lo: &self.lo + &b.lo,
            hi: &self.hi + &b.hi,
        }
    }
    pub(super) fn sub(&self, b: &Self) -> Self {
        Self {
            lo: &self.lo - &b.hi,
            hi: &self.hi - &b.lo,
        }
    }
    pub(super) fn mul(&self, b: &Self) -> Self {
        let vals = [
            &self.lo * &b.lo,
            &self.lo * &b.hi,
            &self.hi * &b.lo,
            &self.hi * &b.hi,
        ];
        Self {
            lo: vals.iter().min().unwrap().clone(),
            hi: vals.iter().max().unwrap().clone(),
        }
    }
    pub(super) fn scale(&self, b: Q) -> Self {
        self.mul(&Self {
            lo: b.clone(),
            hi: b,
        })
    }
    fn square(&self) -> Self {
        let a = &self.lo * &self.lo;
        let b = &self.hi * &self.hi;
        let lo = if self.lo <= Q::zero() && self.hi >= Q::zero() {
            Q::zero()
        } else {
            a.clone().min(b.clone())
        };
        Self { lo, hi: a.max(b) }
    }
}
struct Endpoint {
    key: EndpointKey,
    point: [Range; 2],
}
pub(super) fn trig(theta: f64) -> Result<(Range, Range), Reason> {
    let (s, c) = sin_cos_interval(theta).ok_or(Reason::JoinProofIncomplete)?;
    Ok((
        Range {
            lo: exact(s.lower),
            hi: exact(s.upper),
        },
        Range {
            lo: exact(c.lower),
            hi: exact(c.upper),
        },
    ))
}
fn curve_endpoint(c: Curve, end: bool) -> Result<Endpoint, Reason> {
    let (mut sine, mut cosine) = trig(c.start)?;
    let mut angle = exact(c.start);
    if end {
        let (sw, cw) = trig(c.sweep)?;
        let s = sine.mul(&cw).add(&cosine.mul(&sw));
        let co = cosine.mul(&cw).sub(&sine.mul(&sw));
        sine = s;
        cosine = co;
        angle += exact(c.sweep);
    }
    let a = exact(c.major);
    let b = exact(c.minor);
    let u = c.x_axis;
    let v = [-u[1], u[0]];
    let point = std::array::from_fn(|i| {
        Range::point(c.center[i])
            .add(&cosine.scale(&a * exact(u[i])))
            .add(&sine.scale(&b * exact(v[i])))
    });
    Ok(Endpoint {
        key: EndpointKey::Curve {
            center: c.center,
            x_axis: c.x_axis,
            major: c.major,
            minor: c.minor,
            angle,
        },
        point,
    })
}
fn endpoint(edge: &HatchEdge2, end: bool) -> Result<Endpoint, Reason> {
    if let HatchEdge2::Line { start, end: finish } = edge {
        let p = if end { *finish } else { *start };
        Ok(Endpoint {
            key: EndpointKey::Point(p),
            point: p.map(Range::point),
        })
    } else {
        curve_endpoint(edge.curve().expect("arc"), end)
    }
}
pub(super) fn check_join(a: &HatchEdge2, b: &HatchEdge2, limit: f64) -> Result<(), Reason> {
    let a = endpoint(a, true)?;
    let b = endpoint(b, false)?;
    // Same mathematical expression is a proof even when interval widths are nonzero.
    if a.key == b.key {
        return Ok(());
    }
    let d2 = a.point[0]
        .sub(&b.point[0])
        .square()
        .add(&a.point[1].sub(&b.point[1]).square());
    let t = exact(limit);
    let t2 = &t * &t;
    if d2.hi <= t2 {
        return Ok(());
    }
    if d2.lo > t2 {
        let lower = sqrt_bracket(&d2.lo).map_or(f64::MAX, |x| x.0);
        let upper = sqrt_bracket(&d2.hi).map_or(f64::INFINITY, |x| x.1);
        Err(Reason::GapExceeded {
            lower,
            upper,
            limit,
        })
    } else {
        Err(Reason::JoinProofIncomplete)
    }
}

pub(super) struct ArcRays {
    pub start: [Range; 2],
    pub end: [Range; 2],
    pub positive: bool,
    pub minor: bool,
}
impl ArcRays {
    pub fn could_contain(&self, ray: [Q; 2]) -> bool {
        let cross = |a: &[Range; 2], b: &[Range; 2]| a[0].mul(&b[1]).sub(&a[1].mul(&b[0]));
        let ray = ray.map(|x| Range {
            lo: x.clone(),
            hi: x,
        });
        let sign = exact(if self.positive { 1.0 } else { -1.0 });
        let left = cross(&self.start, &ray).scale(sign.clone());
        let right = cross(&ray, &self.end).scale(sign);
        if self.minor {
            left.hi >= Q::zero() && right.hi >= Q::zero()
        } else {
            left.hi >= Q::zero() || right.hi >= Q::zero()
        }
    }
}
pub(super) fn curve_rays(c: Curve) -> Result<ArcRays, Reason> {
    let (s, co) = trig(c.start)?;
    let (sw, cw) = trig(c.sweep)?;
    Ok(ArcRays {
        start: [co.clone(), s.clone()],
        end: [co.mul(&cw).sub(&s.mul(&sw)), s.mul(&cw).add(&co.mul(&sw))],
        positive: c.sweep > 0.0,
        // No binary64 lies between PI and the true pi, so this selects the
        // mathematical minor/major half for any valid binary64 sweep.
        minor: c.sweep.abs() <= std::f64::consts::PI,
    })
}
