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
struct Residual {
    base: V2,
    offset: V2,
    direction: V2,
}
struct PairEvidence<'a> {
    source: &'a Frame,
    target: &'a Frame,
    correlated: Option<Residual>,
}
impl PairEvidence<'_> {
    fn emit(&self, points: &mut Vec<(PairedPoint, Q)>, s: &V2, t: &V2, k: &Q, ts: &Q, tt: &Q) {
        let target = world(self.target, t);
        let p = if let Some(r) = &self.correlated {
            // Exact shared-angle identity was checked before construction.
            // Subtract parameters before multiplying the
            // common direction, retaining period, phase and segment drift.
            let local = add(
                &add(&r.base, &scale(&r.offset, k)),
                &scale(&r.direction, &(ts - tt)),
            );
            let residual = std::array::from_fn(|i| {
                Range::point(&self.source.o[i] - &self.target.o[i])
                    .add(&local[0].scale(&self.source.x[i]))
                    .add(&local[1].scale(&self.source.y[i]))
                    // S*(s-t) + (S-T)*t retains actual frame drift, including
                    // tiny changes when CAD reconstructs an oblique basis.
                    .add(&t[0].scale(&(&self.source.x[i] - &self.target.x[i])))
                    .add(&t[1].scale(&(&self.source.y[i] - &self.target.y[i])))
            });
            PairedPoint::from_residual_ranges(target, residual)
        } else {
            PairedPoint::from_ranges(world(self.source, s), target)
        };
        let d = p.squared_deviation().1;
        points.push((p, d));
    }
}
fn family_pair(
    s: &Family,
    t: &Family,
    evidence: &PairEvidence<'_>,
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
                evidence.emit(points, &sp, &tp, k, u, u);
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
                    evidence.emit(points, &sp, &tp, k, &ts, &tt);
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
        // Binary64 equality of a rounded sum would not establish a real-angle
        // identity. Compare the exact rational operands, without an epsilon.
        // The qualified double operation has separate exact-quarter-turn
        // semantics and continues through the conservative existing route.
        let correlated =
            if !(double && index == 1) && exact(a.angle) + exact(p.rotation) == exact(b.angle) {
                let normal = [c.d[1].scale(&exact(-1.)), c.d[0].clone()];
                // R(rotation)*d(relative_angle) = d(effective_angle). This
                // changes only the proof expression, never the stored offset.
                let offset = scale(
                    &add(
                        &scale(&c.d, &exact(a.offset[0])),
                        &scale(&normal, &exact(a.offset[1])),
                    ),
                    &exact(p.scale),
                );
                let (base, offset) = if export {
                    (sub(&n.base, &c.base), sub(&offset, &c.offset))
                } else {
                    (sub(&c.base, &n.base), sub(&c.offset, &offset))
                };
                Some(Residual {
                    base,
                    offset,
                    direction: c.d.clone(),
                })
            } else {
                None
            };
        if export {
            let evidence = PairEvidence {
                source: &nf,
                target: &cf,
                correlated,
            };
            family_pair(&n, &c, &evidence, &nr, &cr, &mut points)?;
        } else {
            let evidence = PairEvidence {
                source: &cf,
                target: &nf,
                correlated,
            };
            family_pair(&c, &n, &evidence, &cr, &nr, &mut points)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::blocks::EvaluatedBlock;
    use ifccad::geometry_kernel::{BlockTransform, Point3, Scale3, Vector3};
    use opencadcodec::{entities::hatch::HatchPatternLine, Vector2};

    fn fixture() -> (Hatch, HatchLinePattern, Vec<HatchBoundary2>) {
        let mut h =
            Hatch::with_pattern(opencadcodec::entities::hatch::HatchPattern::new("Explicit"));
        h.pattern_angle = 0.25;
        h.pattern_scale = 2.;
        h.pattern.lines.push(HatchPatternLine {
            angle: 1.25,
            base_point: Vector2::ZERO,
            offset: Vector2::new(1., 2.),
            dash_lengths: vec![2., -1., 0.],
        });
        let contours = vec![HatchBoundary2::Circle {
            center: [10000., 20000.],
            radius: 10.,
        }];
        let p = prepare_pattern_from_cad(&h, Default::default(), &contours)
            .unwrap()
            .pattern;
        (h, p, contours)
    }

    #[test]
    fn exact_angle_identity_does_not_remove_real_phase_period_or_offset_drift() {
        let (h, p, b) = fixture();
        for field in 0..3 {
            let mut changed = p.clone();
            match field {
                0 => changed.families[0].base_point[0] += 1e-4,
                1 => changed.families[0].offset[1] += 1e-4,
                _ => changed.families[0].dashes[0] = HatchDash::Dash { length: 1.0001 },
            }
            for export in [false, true] {
                let pair = build(&h, &changed, Default::default(), &b, export, false).unwrap();
                assert!(pair
                    .points
                    .iter()
                    .any(|(p, _)| p.squared_deviation().0 > exact(1e-18)));
            }
        }
    }

    #[test]
    fn actual_z_origin_drift_is_retained_in_both_directions() {
        let (h, p, b) = fixture();
        let plane = CoordinateFrame3::try_new(
            Point3::new(0., 0., 2e-9),
            Vector3::new(1., 0., 0.),
            Vector3::new(0., 1., 0.),
        )
        .unwrap();
        for export in [false, true] {
            let pair = build(&h, &p, plane, &b, export, false).unwrap();
            for (point, _) in pair.points {
                assert!(point.squared_deviation().0 >= exact(2e-9) * exact(2e-9));
            }
        }
    }

    #[test]
    fn unequal_angles_and_axes_keep_conservative_certificates() {
        let (h, mut p, b) = fixture();
        p.families[0].angle += 1e-4;
        for export in [false, true] {
            let pair = build(&h, &p, Default::default(), &b, export, false).unwrap();
            assert!(pair
                .points
                .iter()
                .any(|(p, _)| p.squared_deviation().0 > exact(1e-18)));
        }
        let (h, p, b) = fixture();
        let plane = CoordinateFrame3::try_new(
            Point3::new(0., 0., 0.),
            Vector3::new(0., 1., 0.),
            Vector3::new(-1., 0., 0.),
        )
        .unwrap();
        for export in [false, true] {
            let pair = build(&h, &p, plane, &b, export, false).unwrap();
            assert!(pair
                .points
                .iter()
                .any(|(p, _)| p.squared_deviation().0 > exact(1.)));
        }
    }

    #[test]
    fn rounded_angle_sum_is_not_an_exact_direction_identity() {
        let (mut h, mut p, _) = fixture();
        p.rotation = 0.2;
        p.families[0].angle = 0.1;
        h.pattern.lines[0].angle = p.rotation + p.families[0].angle;
        assert_ne!(
            exact(p.rotation) + exact(p.families[0].angle),
            exact(h.pattern.lines[0].angle)
        );
        let b = [HatchBoundary2::Circle {
            center: [1e9, 1e9],
            radius: 10.,
        }];
        // Offset/base drift is removed here so that only the direction proof
        // (including its conservative uncertainty) can make this test fail.
        p.families[0].base_point = [0., 0.];
        p.families[0].offset = [0., 1.];
        let q = prepare_pattern_to_cad(
            &p,
            Default::default(),
            opencadcodec::Vector3::UNIT_Z,
            0.,
            &b,
        )
        .unwrap();
        assert!(q
            .pair
            .points
            .iter()
            .any(|(p, _)| p.squared_deviation().1 > exact(1e-18)));
    }

    #[test]
    fn target_position_survives_unequal_and_nested_block_maps() {
        let (h, p, b) = fixture();
        let pair = build(&h, &p, Default::default(), &b, false, false).unwrap();
        let block = |scales| {
            EvaluatedBlock::native(
                BlockTransform::try_new(Default::default(), 0.3, scales).unwrap(),
                [1., 2., 3.],
            )
        };
        let source = block(Scale3::new(1.001, 1., 1.));
        let target = block(Scale3::new(1., 1., 1.));
        let outer = block(Scale3::new(-2., 3., -1.));
        for (mut point, _) in pair.points {
            point.apply(&source, &target);
            point.apply(&outer, &outer);
            assert!(point.squared_deviation().0 > exact(1.));
        }
    }
}
