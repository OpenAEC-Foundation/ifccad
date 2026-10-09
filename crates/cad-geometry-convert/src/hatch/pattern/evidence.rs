use super::super::evidence::Frame;
use super::math::*;
use super::*;
use crate::geometry::{
    blocks::{PairedPoint, Range},
    numeric::exact,
};
use num_rational::BigRational as Q;
use num_traits::{Signed, Zero};
struct Family {
    base: V2,
    offset: V2,
    d: V2,
    lengths: Vec<Q>,
}
fn native(p: &HatchLinePattern, f: &HatchLineFamily) -> Family {
    let d = direction(f.angle);
    let n = [d[1].scale(&exact(-1.)), d[0].clone()];
    Family {
        base: add(
            &point(p.origin),
            &scale(&rotate(&point(f.base_point), p.rotation), &exact(p.scale)),
        ),
        offset: scale(
            &rotate(
                &add(
                    &scale(&d, &exact(f.offset[0])),
                    &scale(&n, &exact(f.offset[1])),
                ),
                p.rotation,
            ),
            &exact(p.scale),
        ),
        d: rotate(&d, p.rotation),
        lengths: f
            .dashes
            .iter()
            .map(|v| match *v {
                HatchDash::Dot => Q::zero(),
                HatchDash::Dash { length } | HatchDash::Gap { length } => {
                    exact(length) * exact(p.scale)
                }
            })
            .collect(),
    }
}
fn cad(f: &opencadcodec::entities::hatch::HatchPatternLine) -> Family {
    Family {
        base: point([f.base_point.x, f.base_point.y]),
        offset: point([f.offset.x, f.offset.y]),
        d: direction(f.angle),
        lengths: f.dash_lengths.iter().map(|v| exact(v.abs())).collect(),
    }
}
fn rect(boundaries: &[HatchBoundary2]) -> Result<Vec<V2>, CadHatchPreparationError> {
    let b = hatch_bounds(CoordinateFrame3::default(), boundaries)?;
    let lo = b.min();
    let hi = b.max();
    Ok(vec![
        point([lo.x(), lo.y()]),
        point([lo.x(), hi.y()]),
        point([hi.x(), lo.y()]),
        point([hi.x(), hi.y()]),
    ])
}
fn cad_rect(h: &Hatch, fallback: &[HatchBoundary2]) -> Result<Vec<V2>, CadHatchPreparationError> {
    if h.paths.is_empty() {
        return rect(fallback);
    }
    let b = h
        .paths
        .iter()
        .map(super::super::from_cad::boundary)
        .collect::<Result<Vec<_>, _>>()?;
    rect(&b)
}
fn range_union(values: impl IntoIterator<Item = Range>) -> Range {
    let mut it = values.into_iter();
    let first = it.next().expect("nonempty certificate domain");
    let (l, h) = first.limits();
    let (mut lo, mut hi) = (l.clone(), h.clone());
    for r in it {
        let (l, h) = r.limits();
        lo = lo.min(l.clone());
        hi = hi.max(h.clone());
    }
    Range::hull(lo, hi)
}
fn line_indices(f: &Family, r: &[V2]) -> Result<(Q, Q), CadHatchPreparationError> {
    let spacing = cross(&f.d, &f.offset);
    let v = r
        .iter()
        .map(|p| divide(&cross(&f.d, &sub(p, &f.base)), &spacing))
        .collect::<Result<Vec<_>, _>>()?;
    let span = range_union(v);
    let (l, h) = span.limits();
    Ok((
        Q::from_integer(l.floor().to_integer()) - exact(1.),
        Q::from_integer(h.ceil().to_integer()) + exact(1.),
    ))
}
fn along(f: &Family, r: &[V2], k: &(Q, Q)) -> Range {
    range_union(r.iter().flat_map(|p| {
        [&k.0, &k.1].map(|k| dot(&f.d, &sub(&sub(p, &f.base), &scale(&f.offset, k))))
    }))
}
fn world(frame: &Frame, p: &V2) -> [Range; 3] {
    std::array::from_fn(|i| {
        Range::point(frame.o[i].clone())
            .add(&p[0].scale(&frame.x[i]))
            .add(&p[1].scale(&frame.y[i]))
    })
}
fn emit(points: &mut Vec<(PairedPoint, Q)>, sf: &Frame, tf: &Frame, s: &V2, t: &V2) {
    let p = PairedPoint::from_ranges(world(sf, s), world(tf, t));
    let d = p.squared_deviation().1;
    points.push((p, d));
}
fn family_pair(
    s: &Family,
    t: &Family,
    sf: &Frame,
    tf: &Frame,
    srect: &[V2],
    trect: &[V2],
    points: &mut Vec<(PairedPoint, Q)>,
) -> Result<(), CadHatchPreparationError> {
    // For fixed source/target coefficients, position residual is affine in
    // k and t (continuous), or k/m/u (periodic). The norm is convex, hence its
    // maximum over the enclosing box occurs at a corner. Interval coefficient
    // evaluation encloses every corner, without enumerating integer repeats.
    // The union includes lines/segments potentially intersecting either contour.
    if s.lengths.len() != t.lengths.len() {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    let sk = line_indices(s, srect)?;
    let tk = line_indices(t, trect)?;
    let k = (sk.0.min(tk.0), sk.1.max(tk.1));
    let sl = along(s, srect, &k);
    let tl = along(t, trect, &k);
    if s.lengths.is_empty() {
        let a = range_union([sl, tl]);
        let (l, h) = a.limits();
        for k in [&k.0, &k.1] {
            for u in [l, h] {
                let sp = add(&add(&s.base, &scale(&s.offset, k)), &scale(&s.d, u));
                let tp = add(&add(&t.base, &scale(&t.offset, k)), &scale(&t.d, u));
                emit(points, sf, tf, &sp, &tp);
            }
        }
        return Ok(());
    }
    let ps: Q = s.lengths.iter().cloned().sum();
    let pt: Q = t.lengths.iter().cloned().sum();
    if ps <= Q::zero() || pt <= Q::zero() {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    let sm = divide(&sl, &Range::point(ps.clone()))?;
    let tm = divide(&tl, &Range::point(pt.clone()))?;
    let m = range_union([sm, tm]);
    let (ml, mh) = m.limits();
    let mlo = Q::from_integer(ml.floor().to_integer()) - exact(1.);
    let mhi = Q::from_integer(mh.ceil().to_integer()) + exact(1.);
    let (mut cs, mut ct) = (Q::zero(), Q::zero());
    for (ls, lt) in s.lengths.iter().zip(&t.lengths) {
        for k in [&k.0, &k.1] {
            for m in [&mlo, &mhi] {
                for u in [Q::zero(), exact(1.)] {
                    let ts = m * &ps + &cs + &u * ls;
                    let tt = m * &pt + &ct + &u * lt;
                    let sp = add(&add(&s.base, &scale(&s.offset, k)), &scale(&s.d, &ts));
                    let tp = add(&add(&t.base, &scale(&t.offset, k)), &scale(&t.d, &tt));
                    emit(points, sf, tf, &sp, &tp);
                }
            }
        }
        cs += ls;
        ct += lt;
    }
    Ok(())
}
fn build(
    h: &Hatch,
    p: &HatchLinePattern,
    plane: CoordinateFrame3,
    boundaries: &[HatchBoundary2],
    export: bool,
    double: bool,
) -> Result<GeometryPair, CadHatchPreparationError> {
    if h.pattern.lines.len() != p.families.len() {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    let nf = Frame::native(plane);
    let cf = Frame::cad(h);
    let nr = rect(boundaries)?;
    let cr = cad_rect(h, boundaries)?;
    let mut points = vec![];
    for (index, (a, b)) in p.families.iter().zip(&h.pattern.lines).enumerate() {
        let n = native(p, a);
        let mut c = cad(b);
        if double && index == 1 {
            // The active double operation is an exact quarter-turn of the
            // first source direction, not the rounded binary64 PI/2 field.
            c.d = point([0., 1.]);
        }
        if export {
            family_pair(&n, &c, &nf, &cf, &nr, &cr, &mut points)?;
        } else {
            family_pair(&c, &n, &cf, &nf, &cr, &nr, &mut points)?;
        }
    }
    Ok(GeometryPair {
        points,
        curves: vec![],
    })
}
pub(super) fn from_cad(
    h: &Hatch,
    p: &HatchLinePattern,
    plane: CoordinateFrame3,
    b: &[HatchBoundary2],
) -> Result<GeometryPair, CadHatchPreparationError> {
    build(h, p, plane, b, false, false)
}
pub(super) fn to_cad(
    p: &HatchLinePattern,
    h: &Hatch,
    plane: CoordinateFrame3,
    b: &[HatchBoundary2],
) -> Result<GeometryPair, CadHatchPreparationError> {
    build(h, p, plane, b, true, false)
}

pub(crate) fn double_evidence(
    h: &Hatch,
    p: &HatchLinePattern,
    plane: CoordinateFrame3,
    b: &[HatchBoundary2],
) -> Result<GeometryPair, CadHatchPreparationError> {
    build(h, p, plane, b, false, true)
}
