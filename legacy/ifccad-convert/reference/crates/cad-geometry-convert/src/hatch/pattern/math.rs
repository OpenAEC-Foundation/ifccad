use crate::geometry::{
    blocks::{trig, Range},
    numeric::{exact, round_nearest},
};
use crate::CadPreparationError;
use num_rational::BigRational as Q;
use num_traits::Zero;
pub(super) type V2 = [Range; 2];
pub(super) fn point(p: [f64; 2]) -> V2 {
    p.map(|x| Range::point(exact(x)))
}
pub(super) fn add(a: &V2, b: &V2) -> V2 {
    std::array::from_fn(|i| a[i].add(&b[i]))
}
pub(super) fn sub(a: &V2, b: &V2) -> V2 {
    add(a, &scale(b, &exact(-1.)))
}
pub(super) fn scale(a: &V2, k: &Q) -> V2 {
    a.clone().map(|x| x.scale(k))
}
pub(super) fn direction(angle: f64) -> V2 {
    let (s, c) = trig(angle);
    [c, s]
}
pub(super) fn rotate(v: &V2, angle: f64) -> V2 {
    let (s, c) = trig(angle);
    [
        v[0].mul(&c).add(&v[1].mul(&s).scale(&exact(-1.))),
        v[0].mul(&s).add(&v[1].mul(&c)),
    ]
}
pub(super) fn dot(a: &V2, b: &V2) -> Range {
    a[0].mul(&b[0]).add(&a[1].mul(&b[1]))
}
pub(super) fn cross(a: &V2, b: &V2) -> Range {
    a[0].mul(&b[1]).add(&a[1].mul(&b[0]).scale(&exact(-1.)))
}
pub(super) fn divide(a: &Range, b: &Range) -> Result<Range, CadPreparationError> {
    let (al, ah) = a.limits();
    let (bl, bh) = b.limits();
    if bl <= &Q::zero() && bh >= &Q::zero() {
        return Err(CadPreparationError::UnsupportedGeometry);
    }
    let v = [al / bl, al / bh, ah / bl, ah / bh];
    Ok(Range::hull(
        v.iter().min().unwrap().clone(),
        v.iter().max().unwrap().clone(),
    ))
}
pub(super) fn nearest(r: &Range) -> Result<f64, CadPreparationError> {
    let (l, h) = r.limits();
    round_nearest(&((l + h) / exact(2.))).map_err(|_| CadPreparationError::OutOfRange)
}
pub(super) fn materialize(v: &V2) -> Result<[f64; 2], CadPreparationError> {
    Ok([nearest(&v[0])?, nearest(&v[1])?])
}
