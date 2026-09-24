//! Occurrence-space error propagation for shared and nested block geometry.

use super::numeric::exact;
use num_rational::BigRational as Q;
use num_traits::{Signed, Zero};

#[derive(Clone, Debug)]
struct Range {
    lo: Q,
    hi: Q,
}
impl Range {
    fn point(value: Q) -> Self {
        Self {
            lo: value.clone(),
            hi: value,
        }
    }
    fn add(&self, rhs: &Self) -> Self {
        Self {
            lo: &self.lo + &rhs.lo,
            hi: &self.hi + &rhs.hi,
        }
    }
    fn scale(&self, value: &Q) -> Self {
        if value.is_negative() {
            Self {
                lo: &self.hi * value,
                hi: &self.lo * value,
            }
        } else {
            Self {
                lo: &self.lo * value,
                hi: &self.hi * value,
            }
        }
    }
    fn mul(&self, rhs: &Self) -> Self {
        let values = [
            &self.lo * &rhs.lo,
            &self.lo * &rhs.hi,
            &self.hi * &rhs.lo,
            &self.hi * &rhs.hi,
        ];
        Self {
            lo: values.iter().min().unwrap().clone(),
            hi: values.iter().max().unwrap().clone(),
        }
    }
    fn squared(&self) -> (Q, Q) {
        let a = &self.lo * &self.lo;
        let b = &self.hi * &self.hi;
        let lower = if self.lo <= Q::zero() && self.hi >= Q::zero() {
            Q::zero()
        } else {
            a.clone().min(b.clone())
        };
        (lower, a.max(b))
    }
}

/// Same pinned backend and two-neighbour enclosure as the core; exact rational
/// interval arithmetic here prevents accumulation of arithmetic enclosure error.
/// See docs/geometry/block-trigonometry.md for qualification and assumptions.
fn trig(angle: f64) -> (Range, Range) {
    if angle == 0. {
        return (Range::point(Q::zero()), Range::point(exact(1.)));
    }
    let (s, c) = fpmath::sin_cos(angle);
    let enclose = |value: f64| Range {
        lo: exact(value.next_down().next_down().max(-1.)),
        hi: exact(value.next_up().next_up().min(1.)),
    };
    (enclose(s), enclose(c))
}

#[derive(Clone)]
pub(crate) struct EvaluatedBlock {
    pub origin: [Q; 3],
    pub base: [Q; 3],
    pub rotation: f64,
    /// Matrix coefficients multiplying cos(rotation), sin(rotation), and 1.
    pub cosine: [[Q; 3]; 3],
    pub sine: [[Q; 3]; 3],
    pub constant: [[Q; 3]; 3],
}
impl EvaluatedBlock {
    pub fn native(transform: ifccad::ifcdr::BlockTransform, base: [f64; 3]) -> Self {
        let (o, u, v) = super::components(transform.placement());
        let u = u.map(exact);
        let v = v.map(exact);
        let normal = std::array::from_fn(|i| {
            let j = (i + 1) % 3;
            let k = (i + 2) % 3;
            &u[j] * &v[k] - &u[k] * &v[j]
        });
        let scale = transform.scale();
        Self::from_axes(
            o.map(exact),
            base.map(exact),
            [u, v, normal],
            [scale.x(), scale.y(), scale.z()],
            transform.rotation(),
        )
    }
    fn from_axes(
        origin: [Q; 3],
        base: [Q; 3],
        axes: [[Q; 3]; 3],
        scale: [f64; 3],
        rotation: f64,
    ) -> Self {
        Self {
            origin,
            base,
            rotation,
            cosine: std::array::from_fn(|i| {
                [
                    &axes[0][i] * exact(scale[0]),
                    &axes[1][i] * exact(scale[1]),
                    Q::zero(),
                ]
            }),
            sine: std::array::from_fn(|i| {
                [
                    &axes[1][i] * exact(scale[0]),
                    -&axes[0][i] * exact(scale[1]),
                    Q::zero(),
                ]
            }),
            constant: std::array::from_fn(|i| {
                [Q::zero(), Q::zero(), &axes[2][i] * exact(scale[2])]
            }),
        }
    }
    #[cfg(test)]
    fn identity() -> Self {
        Self {
            origin: std::array::from_fn(|_| Q::zero()),
            base: std::array::from_fn(|_| Q::zero()),
            rotation: 0.,
            cosine: std::array::from_fn(|i| {
                std::array::from_fn(|j| if i == j { exact(1.) } else { Q::zero() })
            }),
            sine: std::array::from_fn(|_| std::array::from_fn(|_| Q::zero())),
            constant: std::array::from_fn(|_| std::array::from_fn(|_| Q::zero())),
        }
    }
    fn coefficient(&self, row: usize, col: usize, sin: &Range, cos: &Range) -> Range {
        cos.scale(&self.cosine[row][col])
            .add(&sin.scale(&self.sine[row][col]))
            .add(&Range::point(self.constant[row][col].clone()))
    }
    fn difference(&self, other: &Self, row: usize, col: usize) -> Range {
        let (sin, cos) = trig(self.rotation);
        if self.rotation == other.rotation {
            // Retain correlation: equal coefficient functions cancel exactly,
            // instead of independently subtracting two trig enclosures.
            cos.scale(&(&self.cosine[row][col] - &other.cosine[row][col]))
                .add(&sin.scale(&(&self.sine[row][col] - &other.sine[row][col])))
                .add(&Range::point(
                    &self.constant[row][col] - &other.constant[row][col],
                ))
        } else {
            let (other_sin, other_cos) = trig(other.rotation);
            self.coefficient(row, col, &sin, &cos).add(
                &other
                    .coefficient(row, col, &other_sin, &other_cos)
                    .scale(&exact(-1.)),
            )
        }
    }
}

/// Target location plus correlated source-minus-target residual. Independent
/// source/target coordinate intervals would invent error for identical maps.
#[derive(Clone)]
pub(crate) struct PairedPoint {
    target: [Range; 3],
    residual: [Range; 3],
}
impl PairedPoint {
    pub fn new(source: [Q; 3], target: [Q; 3]) -> Self {
        Self {
            residual: std::array::from_fn(|i| Range::point(&source[i] - &target[i])),
            target: target.map(Range::point),
        }
    }
    pub fn exact(point: [f64; 3]) -> Self {
        Self {
            target: point.map(|v| Range::point(exact(v))),
            residual: std::array::from_fn(|_| Range::point(Q::zero())),
        }
    }
    pub fn apply(&mut self, source: &EvaluatedBlock, target: &EvaluatedBlock) {
        let (ss, sc) = trig(source.rotation);
        let (ts, tc) = trig(target.rotation);
        let target_local: [Range; 3] =
            std::array::from_fn(|i| self.target[i].add(&Range::point(-target.base[i].clone())));
        let residual_local: [Range; 3] = std::array::from_fn(|i| {
            self.residual[i].add(&Range::point(&target.base[i] - &source.base[i]))
        });
        let residual = std::array::from_fn(|i| {
            let mut result = Range::point(&source.origin[i] - &target.origin[i]);
            for j in 0..3 {
                result = result.add(&source.difference(target, i, j).mul(&target_local[j]));
                result = result.add(&source.coefficient(i, j, &ss, &sc).mul(&residual_local[j]));
            }
            result
        });
        self.target = std::array::from_fn(|i| {
            let mut result = Range::point(target.origin[i].clone());
            for (j, point) in target_local.iter().enumerate() {
                result = result.add(&target.coefficient(i, j, &ts, &tc).mul(point));
            }
            result
        });
        self.residual = residual;
    }
    pub fn squared_deviation(&self) -> (Q, Q) {
        self.residual
            .iter()
            .map(Range::squared)
            .fold((Q::zero(), Q::zero()), |(a, b), (c, d)| (a + c, b + d))
    }
}

pub(crate) fn from_cad_instance(
    insert: &cadcodec::entities::Insert,
    base: cadcodec::Vector3,
    assessment: &crate::ConversionGeometryAssessment,
) -> Result<
    (
        ifccad::ifcdr::BlockTransform,
        EvaluatedBlock,
        EvaluatedBlock,
    ),
    Box<crate::ConversionGeometryFailure>,
> {
    use crate::{
        ConversionEntitySource, ConversionGeometryFailureReason as Reason,
        ConversionGeometryStage as Stage,
    };
    use ifccad::ifcdr::{BlockTransform, PlanePlacement, Point3, Scale3, Vector3};
    let source = ConversionEntitySource::CadEntity {
        handle: insert.common.handle,
        kind: "INSERT".into(),
    };
    let fail = || {
        assessment.failure(
            &source,
            None,
            Stage::TargetConstruction,
            Reason::CadAxisEvaluationFailed,
        )
    };
    let position = [
        insert.insert_point.x,
        insert.insert_point.y,
        insert.insert_point.z,
    ];
    let scale = [insert.x_scale(), insert.y_scale(), insert.z_scale()];
    if !position
        .into_iter()
        .chain(scale)
        .chain([insert.rotation, base.x, base.y, base.z])
        .all(f64::is_finite)
    {
        return Err(fail());
    }
    // Evaluate the actual public CAD frame, not a separately re-normalized one.
    let m = cadcodec::types::Matrix3::arbitrary_axis(insert.normal).m;
    if !m.iter().flatten().all(|v| v.is_finite()) {
        return Err(fail());
    }
    let axes: [[Q; 3]; 3] = std::array::from_fn(|j| std::array::from_fn(|i| exact(m[i][j])));
    let origin: [Q; 3] =
        std::array::from_fn(|i| (0..3).map(|j| &axes[j][i] * exact(position[j])).sum());
    let mut rounded = [0.; 3];
    for i in 0..3 {
        rounded[i] = super::numeric::round_nearest(&origin[i]).map_err(|_| {
            assessment.failure(
                &source,
                None,
                Stage::TargetConstruction,
                Reason::TargetCoordinateOutOfRange,
            )
        })?;
    }
    let plane = PlanePlacement::try_new(
        Point3::new(rounded[0], rounded[1], rounded[2]),
        Vector3::new(m[0][0], m[1][0], m[2][0]),
        Vector3::new(m[0][1], m[1][1], m[2][1]),
    )
    .map_err(|_| fail())?;
    let transform = BlockTransform::try_new(
        plane,
        insert.rotation,
        Scale3::new(scale[0], scale[1], scale[2]),
    )
    .map_err(|_| fail())?;
    let base = [base.x, base.y, base.z].map(exact);
    let source_map =
        EvaluatedBlock::from_axes(origin, base.clone(), axes.clone(), scale, insert.rotation);
    let mut target_axes = axes;
    target_axes[2] = std::array::from_fn(|i| {
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;
        &target_axes[0][j] * &target_axes[1][k] - &target_axes[0][k] * &target_axes[1][j]
    });
    let target_map = EvaluatedBlock::from_axes(
        rounded.map(exact),
        base,
        target_axes,
        scale,
        insert.rotation,
    );
    Ok((transform, source_map, target_map))
}

pub(crate) fn polyline_pairs(
    poly: &cadcodec::LwPolyline,
    plane: ifccad::ifcdr::PlanePlacement,
) -> Vec<PairedPoint> {
    let basis = super::cad_plane(poly.normal).expect("validated CAD polyline axes");
    let (o, x, y) = super::components(plane);
    let mut pairs = poly
        .vertices
        .iter()
        .map(|vertex| {
            let a = exact(vertex.location.x);
            let b = exact(vertex.location.y);
            let source = std::array::from_fn(|i| {
                exact(basis.u[i]) * &a
                    + exact(basis.v[i]) * &b
                    + exact(basis.n[i]) * exact(poly.elevation)
            });
            let target = std::array::from_fn(|i| exact(o[i]) + exact(x[i]) * &a + exact(y[i]) * &b);
            PairedPoint::new(source, target)
        })
        .collect::<Vec<_>>();
    let count = if poly.is_closed {
        poly.vertices.len()
    } else {
        poly.vertices.len().saturating_sub(1)
    };
    for index in 0..count {
        let start = &poly.vertices[index];
        if start.bulge == 0.0 {
            continue;
        }
        let end = &poly.vertices[(index + 1) % poly.vertices.len()];
        let [a, b] = super::bulge_midpoint(
            [start.location.x, start.location.y],
            [end.location.x, end.location.y],
            start.bulge,
        );
        let source = std::array::from_fn(|i| {
            exact(basis.u[i]) * &a
                + exact(basis.v[i]) * &b
                + exact(basis.n[i]) * exact(poly.elevation)
        });
        let target = std::array::from_fn(|i| exact(o[i]) + exact(x[i]) * &a + exact(y[i]) * &b);
        pairs.push(PairedPoint::new(source, target));
    }
    pairs
}

pub(crate) fn to_cad_instance(
    native: ifccad::ifcdr::BlockInstanceRef,
    definition: ifccad::ifcdr::BlockDefinitionRef,
    source: crate::ConversionEntitySource,
    assessment: &crate::ConversionGeometryAssessment,
) -> Result<
    (
        cadcodec::entities::Insert,
        EvaluatedBlock,
        EvaluatedBlock,
        bool,
    ),
    crate::ImportError,
> {
    use crate::{ConversionGeometryFailureReason as Reason, ConversionGeometryStage as Stage};
    let transform = native.transform();
    let plane = transform.placement();
    let fail = || {
        assessment.failure(
            &source,
            None,
            Stage::TargetConstruction,
            Reason::CadAxisEvaluationFailed,
        )
    };
    let normal = super::stored_normal(plane).ok_or_else(fail)?;
    let matrix = cadcodec::types::Matrix3::arbitrary_axis(normal).m;
    let axes: [[f64; 3]; 3] = std::array::from_fn(|j| std::array::from_fn(|i| matrix[i][j]));
    if !axes.iter().flatten().all(|v| v.is_finite()) {
        return Err(fail().into());
    }
    let (o, u, v) = super::components(plane);
    let mut position = [0.; 3];
    for i in 0..3 {
        position[i] = super::numeric::round_nearest(&super::dot(axes[i], o)).map_err(|_| {
            assessment.failure(
                &source,
                None,
                Stage::TargetConstruction,
                Reason::TargetCoordinateOutOfRange,
            )
        })?;
    }
    let changed = u != axes[0] || v != axes[1];
    let offset = if changed {
        let x = super::numeric::round_nearest(&super::dot(u, axes[0])).map_err(|_| fail())?;
        let y = super::numeric::round_nearest(&super::dot(u, axes[1])).map_err(|_| fail())?;
        y.atan2(x)
    } else {
        0.
    };
    let mut target = cadcodec::entities::Insert::new(
        definition.name(),
        cadcodec::Vector3::new(position[0], position[1], position[2]),
    );
    target.normal = normal;
    target.rotation = transform.rotation() + offset;
    if !target.rotation.is_finite() {
        return Err(fail().into());
    }
    let scale = transform.scale();
    target.set_x_scale(scale.x());
    target.set_y_scale(scale.y());
    target.set_z_scale(scale.z());
    if [target.x_scale(), target.y_scale(), target.z_scale()] != [scale.x(), scale.y(), scale.z()] {
        return Err(crate::ImportError::BlockTargetLimitation { entity_id:Some(native.entity_id()),message:format!("CAD scale setters changed {:?} to {:?}; values below magnitude 1e-12 cannot be represented by this codec API",[scale.x(),scale.y(),scale.z()],[target.x_scale(),target.y_scale(),target.z_scale()]) });
    }
    let base = definition.base_point();
    let source_map = EvaluatedBlock::native(transform, [base.x(), base.y(), base.z()]);
    let (_, target_map, _) = from_cad_instance(
        &target,
        cadcodec::Vector3::new(base.x(), base.y(), base.z()),
        assessment,
    )
    .map_err(|mut failure| {
        failure.source = source;
        failure
    })?;
    Ok((target, source_map, target_map, changed))
}

pub(crate) fn import_polyline_pairs(
    native: ifccad::ifcdr::PlanarPolylineRef,
    target: &cadcodec::LwPolyline,
) -> Vec<PairedPoint> {
    let (o, u, v) = super::components(native.placement());
    let basis = super::cad_plane(target.normal).expect("constructed CAD axes");
    let mut pairs = native
        .local_points()
        .zip(&target.vertices)
        .map(|(point, vertex)| {
            let source = std::array::from_fn(|i| {
                exact(o[i]) + exact(u[i]) * exact(point.x()) + exact(v[i]) * exact(point.y())
            });
            let target = std::array::from_fn(|i| {
                exact(basis.u[i]) * exact(vertex.location.x)
                    + exact(basis.v[i]) * exact(vertex.location.y)
                    + exact(basis.n[i]) * exact(target.elevation)
            });
            PairedPoint::new(source, target)
        })
        .collect::<Vec<_>>();
    let local = native.local_points().collect::<Vec<_>>();
    let count = if native.closed() {
        local.len()
    } else {
        local.len().saturating_sub(1)
    };
    for index in 0..count {
        let bulge = native.bulge(index).expect("validated bulge");
        if bulge == 0.0 {
            continue;
        }
        let start = local[index];
        let end = local[(index + 1) % local.len()];
        let [a, b] = super::bulge_midpoint([start.x(), start.y()], [end.x(), end.y()], bulge);
        let source = std::array::from_fn(|i| exact(o[i]) + exact(u[i]) * &a + exact(v[i]) * &b);
        let cad_start = target.vertices[index].location;
        let cad_end = target.vertices[(index + 1) % target.vertices.len()].location;
        let [c, d] =
            super::bulge_midpoint([cad_start.x, cad_start.y], [cad_end.x, cad_end.y], bulge);
        let target_point = std::array::from_fn(|i| {
            exact(basis.u[i]) * &c
                + exact(basis.v[i]) * &d
                + exact(basis.n[i]) * exact(target.elevation)
        });
        pairs.push(PairedPoint::new(source, target_point));
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::numeric::exact;

    #[test]
    fn converter_trig_encloses_the_independent_core_oracle() {
        let reference: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tests/data/block-trig-reference.json"
        ))
        .unwrap();
        let value = |bits: &serde_json::Value| {
            f64::from_bits(u64::from_str_radix(bits.as_str().unwrap(), 16).unwrap())
        };
        for case in reference["cases"].as_array().unwrap() {
            let (sin, cos) = trig(value(&case["angle_bits"]));
            for (key, range) in [("sin", sin), ("cos", cos)] {
                assert!(range.lo <= exact(value(&case[key][0])));
                assert!(range.hi >= exact(value(&case[key][1])));
            }
        }
    }

    #[test]
    fn outer_scale_amplifies_inner_translation_error() {
        let mut source = EvaluatedBlock::identity();
        source.origin[0] = exact(1e-9);
        let target = EvaluatedBlock::identity();
        let mut point = PairedPoint::exact([0., 0., 0.]);
        point.apply(&source, &target);
        let mut outer = EvaluatedBlock::identity();
        outer.cosine[0][0] = exact(1e9);
        point.apply(&outer, &outer);
        let (lower, upper) = point.squared_deviation();
        let expected = exact(1e-9) * exact(1e9);
        assert_eq!(lower, &expected * &expected);
        assert_eq!(upper, lower);
    }

    #[test]
    fn equal_rotated_maps_cancel_without_artificial_error() {
        let mut transform = EvaluatedBlock::identity();
        transform.rotation = 0.7;
        transform.base[0] = exact(2.);
        let mut point = PairedPoint::exact([3., 4., 5.]);
        for _ in 0..4 {
            point.apply(&transform, &transform);
        }
        let (lower, upper) = point.squared_deviation();
        assert_eq!(lower, exact(0.));
        assert_eq!(upper, exact(0.));
    }
}
