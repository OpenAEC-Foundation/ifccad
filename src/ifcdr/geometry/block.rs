use super::numeric::Interval;
use super::placement::PlanePlacementComponents;
use super::trig::sin_cos_interval;
use super::{CoordinateAxis, PlanePlacement, PlanePlacementError, Point3, Vector3};
use thiserror::Error;

/// Three signed scale factors. [`BlockTransform::try_new`] validates them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Scale3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub const fn x(self) -> f64 {
        self.x
    }
    pub const fn y(self) -> f64 {
        self.y
    }
    pub const fn z(self) -> f64 {
        self.z
    }
    /// Exact signed equality, without a tolerance or absolute-value conversion.
    pub fn is_uniform(self) -> bool {
        self.x == self.y && self.y == self.z
    }
    pub(crate) fn components(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

impl Default for Scale3 {
    fn default() -> Self {
        Self::new(1., 1., 1.)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum BlockTransformError {
    #[error("block rotation is not finite")]
    NonFiniteRotation,
    #[error("block scale {axis:?} is not finite")]
    NonFiniteScale { axis: CoordinateAxis },
    #[error("block scale {axis:?} is zero")]
    ZeroScale { axis: CoordinateAxis },
    #[error("block normal must be finite and nonzero")]
    InvalidNormal,
    #[error("invalid block placement: {0}")]
    InvalidPlacement(#[from] PlanePlacementError),
}

/// Raw backing values, validated by the same predicates as the public type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BlockTransformComponents {
    pub placement: PlanePlacementComponents,
    pub rotation: f64,
    pub scale: Scale3,
}

impl BlockTransformComponents {
    pub(crate) fn validate(self) -> Result<(), BlockTransformError> {
        self.placement.validate()?;
        if !self.rotation.is_finite() {
            return Err(BlockTransformError::NonFiniteRotation);
        }
        for (axis, value) in CoordinateAxis::ALL.into_iter().zip(self.scale.components()) {
            if !value.is_finite() {
                return Err(BlockTransformError::NonFiniteScale { axis });
            }
            if value == 0. {
                return Err(BlockTransformError::ZeroScale { axis });
            }
        }
        Ok(())
    }
}

/// A block insertion's placement, separate rotation in radians, and signed scale.
///
/// Coordinates first subtract the definition's base point, then scale in local
/// XYZ, rotate in the placement's XY plane, and enter the owning scope. The third
/// axis is the exact cross product of the stored X and Y directions. Validation
/// never normalizes an angle or frame; very small nonzero scales are valid.
///
/// ```
/// use ifccad::ifcdr::{BlockTransform, Point3, Vector3, Scale3};
/// let insert = BlockTransform::from_normal(
///     Point3::new(10., 20., 0.), Vector3::new(0., 0., 1.),
///     std::f64::consts::FRAC_PI_2, Scale3::new(2., 2., 2.),
/// )?;
/// assert_eq!(insert.placement().x_axis(), Vector3::new(1., 0., 0.));
/// assert_eq!(insert.rotation(), std::f64::consts::FRAC_PI_2);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockTransform(BlockTransformComponents);

impl Default for BlockTransform {
    fn default() -> Self {
        Self(BlockTransformComponents {
            placement: PlanePlacement::default().components(),
            rotation: 0.,
            scale: Scale3::default(),
        })
    }
}

impl BlockTransform {
    pub fn try_new(
        placement: PlanePlacement,
        rotation: f64,
        scale: Scale3,
    ) -> Result<Self, BlockTransformError> {
        let value = BlockTransformComponents {
            placement: placement.components(),
            rotation,
            scale,
        };
        value.validate()?;
        Ok(Self(value))
    }
    pub fn placement(self) -> PlanePlacement {
        PlanePlacement::from_validated_components(self.0.placement)
    }
    pub fn rotation(self) -> f64 {
        self.0.rotation
    }
    pub fn scale(self) -> Scale3 {
        self.0.scale
    }
    pub(crate) fn components(self) -> BlockTransformComponents {
        self.0
    }
    pub(crate) fn from_validated_components(value: BlockTransformComponents) -> Self {
        Self(value)
    }

    /// Construct a neutral arbitrary-axis placement from a finite nonzero normal.
    ///
    /// After robust normalization, when both `abs(nx)` and `abs(ny)` are strictly
    /// below `1/64`, X is the normalized world-Y cross normal. Otherwise X is the
    /// normalized world-Z cross normal. Y is normal cross X. The supplied
    /// rotation is retained separately. This convention is not a format validity
    /// requirement; [`Self::try_new`] accepts any valid plane placement.
    pub fn from_normal(
        origin: Point3,
        normal: Vector3,
        rotation: f64,
        scale: Scale3,
    ) -> Result<Self, BlockTransformError> {
        let n = normalized(normal.components()).ok_or(BlockTransformError::InvalidNormal)?;
        let x = if n[0].abs() < 1. / 64. && n[1].abs() < 1. / 64. {
            [n[2], 0., -n[0]]
        } else {
            [-n[1], n[0], 0.]
        };
        let x = normalized(x).ok_or(BlockTransformError::InvalidNormal)?;
        let y = [
            n[1] * x[2] - n[2] * x[1],
            n[2] * x[0] - n[0] * x[2],
            n[0] * x[1] - n[1] * x[0],
        ];
        let placement = PlanePlacement::try_new(
            origin,
            Vector3::new(x[0], x[1], x[2]),
            Vector3::new(y[0], y[1], y[2]),
        )?;
        Self::try_new(placement, rotation, scale)
    }
}

fn normalized(v: [f64; 3]) -> Option<[f64; 3]> {
    if !v.into_iter().all(f64::is_finite) {
        return None;
    }
    let scale = v.into_iter().map(f64::abs).fold(0., f64::max);
    if scale == 0. {
        return None;
    }
    let v = v.map(|c| c / scale);
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    Some(v.map(|c| c / length))
}

fn neg(v: Interval) -> Interval {
    Interval {
        lower: -v.upper,
        upper: -v.lower,
    }
}
fn sub(a: Interval, b: Interval) -> Option<Interval> {
    // Equal singleton operands cancel exactly. Equal non-singleton intervals
    // do not: their independent values can differ.
    if a.lower == a.upper && a == b {
        return Some(Interval::point(0.));
    }
    a.add(neg(b))
}

/// Evaluation cache only: never a serialized property or public affine type.
/// Failure to produce a finite enclosure is a proof gap, not an invalid frame.
pub(crate) struct PreparedBlockTransform {
    origin: [Interval; 3],
    base: [Interval; 3],
    columns: [[Interval; 3]; 3],
}

impl PreparedBlockTransform {
    pub(crate) fn new(transform: BlockTransform, base_point: Point3) -> Option<Self> {
        if !base_point.components().into_iter().all(f64::is_finite) {
            return None;
        }
        let p = transform.placement().components();
        let (u, v) = (
            p.x.components().map(Interval::point),
            p.y.components().map(Interval::point),
        );
        let (s, c) = sin_cos_interval(transform.rotation())?;
        let scale = transform.scale().components().map(Interval::point);
        let mut columns = [[Interval::point(0.); 3]; 3];
        for i in 0..3 {
            let (j, k) = ((i + 1) % 3, (i + 2) % 3);
            let n = sub(u[j].mul(v[k])?, u[k].mul(v[j])?)?;
            columns[0][i] = c.mul(u[i])?.add(s.mul(v[i])?)?.mul(scale[0])?;
            columns[1][i] = neg(s).mul(u[i])?.add(c.mul(v[i])?)?.mul(scale[1])?;
            columns[2][i] = n.mul(scale[2])?;
        }
        Some(Self {
            origin: p.origin.components().map(Interval::point),
            base: base_point.components().map(Interval::point),
            columns,
        })
    }
    pub(crate) fn apply(&self, point: Point3) -> Option<[Interval; 3]> {
        if !point.components().into_iter().all(f64::is_finite) {
            return None;
        }
        self.apply_intervals(point.components().map(Interval::point))
    }
    pub(crate) fn apply_intervals(&self, point: [Interval; 3]) -> Option<[Interval; 3]> {
        if point
            .iter()
            .any(|p| !p.lower.is_finite() || !p.upper.is_finite() || p.lower > p.upper)
        {
            return None;
        }
        let mut delta = [Interval::point(0.); 3];
        for i in 0..3 {
            delta[i] = sub(point[i], self.base[i])?;
        }
        let mut result = self.origin;
        for (i, coordinate) in result.iter_mut().enumerate() {
            for (column, d) in self.columns.iter().zip(delta) {
                *coordinate = coordinate.add(column[i].mul(d)?)?;
            }
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ifcdr::geometry::numeric::{exact, Interval};
    use crate::ifcdr::{CoordinateAxis, PlanePlacement, Point3, Vector3};

    fn origin() -> Point3 {
        Point3::new(0., 0., 0.)
    }
    fn frame(o: Point3, u: Vector3, v: Vector3) -> PlanePlacement {
        PlanePlacement::try_new(o, u, v).unwrap()
    }
    fn encloses(actual: [Interval; 3], expected: [f64; 3]) {
        for (a, e) in actual.into_iter().zip(expected) {
            assert!(a.lower <= e && e <= a.upper, "{a:?} excludes {e}");
            assert!(a.upper - a.lower < 1e-10, "unexpectedly loose {a:?}");
        }
    }

    #[test]
    fn base_point_is_subtracted_before_placement() {
        let placement = frame(
            Point3::new(0., 1., 0.),
            Vector3::new(1., 0., 0.),
            Vector3::new(0., 1., 0.),
        );
        let transform = BlockTransform::try_new(placement, 0., Scale3::default()).unwrap();
        let prepared = PreparedBlockTransform::new(transform, Point3::new(2., 0., 0.)).unwrap();
        assert_eq!(
            prepared.apply(Point3::new(2., 0., 0.)).unwrap(),
            [0., 1., 0.].map(Interval::point)
        );
        encloses(prepared.apply(origin()).unwrap(), [-2., 1., 0.]);
    }

    #[test]
    fn identity_is_exact_even_at_finite_range_edges() {
        let transform = BlockTransform::default();
        assert_eq!(transform.placement(), PlanePlacement::default());
        assert_eq!(transform.rotation(), 0.);
        assert_eq!(transform.scale(), Scale3::new(1., 1., 1.));
        let p = Point3::new(f64::MAX, -f64::MAX, f64::from_bits(1));
        let prepared = PreparedBlockTransform::new(transform, origin()).unwrap();
        assert_eq!(
            prepared.apply(p).unwrap(),
            p.components().map(Interval::point)
        );
    }

    #[test]
    fn signed_uniformity_and_small_scales_are_native_semantics() {
        assert!(Scale3::new(-2., -2., -2.).is_uniform());
        assert!(!Scale3::new(-2., 2., 2.).is_uniform());
        assert!(!Scale3::new(1., 1., 1.0_f64.next_up()).is_uniform());
        for x in [f64::from_bits(1), -f64::from_bits(1)] {
            let scale = Scale3::new(x, x, x);
            assert_eq!([scale.x(), scale.y(), scale.z()], [x; 3]);
            assert!(BlockTransform::try_new(PlanePlacement::default(), 0., scale).is_ok());
        }
    }

    #[test]
    fn invalid_rotation_and_each_invalid_scale_component_are_diagnosed() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                BlockTransform::try_new(PlanePlacement::default(), value, Scale3::default()),
                Err(BlockTransformError::NonFiniteRotation)
            );
        }
        for (i, axis) in CoordinateAxis::ALL.into_iter().enumerate() {
            for value in [0., -0., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut xyz = [1.; 3];
                xyz[i] = value;
                let error = if value == 0. {
                    BlockTransformError::ZeroScale { axis }
                } else {
                    BlockTransformError::NonFiniteScale { axis }
                };
                assert_eq!(
                    BlockTransform::try_new(
                        PlanePlacement::default(),
                        0.,
                        Scale3::new(xyz[0], xyz[1], xyz[2])
                    ),
                    Err(error)
                );
            }
        }
    }

    #[test]
    fn non_neutral_placement_and_unnormalized_rotation_are_retained() {
        let p = frame(
            origin(),
            Vector3::new(0., 1., 0.),
            Vector3::new(-1., 0., 0.),
        );
        let theta = 20. * std::f64::consts::TAU;
        let t = BlockTransform::try_new(p, theta, Scale3::default()).unwrap();
        assert_eq!(t.placement(), p);
        assert_eq!(t.rotation().to_bits(), theta.to_bits());
        let zero_rotation = BlockTransform::try_new(p, 0., Scale3::default()).unwrap();
        encloses(
            PreparedBlockTransform::new(zero_rotation, origin())
                .unwrap()
                .apply(Point3::new(2., 3., 4.))
                .unwrap(),
            [-3., 2., 4.],
        );
    }

    #[test]
    fn scaling_precedes_rotation_in_a_tilted_plane() {
        let p = frame(
            Point3::new(1., 2., 3.),
            Vector3::new(1., 0., 0.),
            Vector3::new(0., 0., 1.),
        );
        let t = BlockTransform::try_new(p, std::f64::consts::FRAC_PI_2, Scale3::new(2., 3., -4.))
            .unwrap();
        // R*S*(1,2,3) = (-6,2,-12), N=U cross V=(0,-1,0).
        encloses(
            PreparedBlockTransform::new(t, origin())
                .unwrap()
                .apply(Point3::new(1., 2., 3.))
                .unwrap(),
            [-5., 14., 5.],
        );
    }

    #[test]
    fn stored_axis_cross_product_is_not_renormalized() {
        let a = 1.0_f64.next_up();
        let p = frame(origin(), Vector3::new(a, 0., 0.), Vector3::new(0., a, 0.));
        let t = BlockTransform::try_new(p, 0., Scale3::default()).unwrap();
        let z = PreparedBlockTransform::new(t, origin())
            .unwrap()
            .apply(Point3::new(0., 0., 1.))
            .unwrap()[2];
        let exact_z = exact(a) * exact(a);
        assert!(exact(z.lower) <= exact_z && exact_z <= exact(z.upper));
        assert!(z.lower > 1.);
    }

    #[test]
    fn nested_nonuniform_transforms_keep_shear_without_decomposition() {
        let inner = BlockTransform::try_new(
            PlanePlacement::default(),
            std::f64::consts::FRAC_PI_4,
            Scale3::new(2., 1., 1.),
        )
        .unwrap();
        let outer = BlockTransform::try_new(
            PlanePlacement::default(),
            -std::f64::consts::FRAC_PI_4,
            Scale3::new(3., 1., 1.),
        )
        .unwrap();
        let inner = PreparedBlockTransform::new(inner, origin()).unwrap();
        let outer = PreparedBlockTransform::new(outer, origin()).unwrap();
        let u = outer
            .apply_intervals(inner.apply(Point3::new(1., 0., 0.)).unwrap())
            .unwrap();
        let v = outer
            .apply_intervals(inner.apply(Point3::new(0., 1., 0.)).unwrap())
            .unwrap();
        encloses(u, [4., -2., 0.]);
        encloses(v, [-1., 2., 0.]);
        // The resulting columns are non-orthogonal, so a single placement/scale
        // decomposition must not be substituted for this composition.
        assert!(u[0].lower * v[0].upper + u[1].upper * v[1].lower < -7.);
    }

    #[test]
    fn underflow_encloses_and_overflow_is_a_proof_gap() {
        let tiny = f64::from_bits(1);
        let t = BlockTransform::try_new(PlanePlacement::default(), 0., Scale3::new(tiny, 1., 1.))
            .unwrap();
        let x = PreparedBlockTransform::new(t, origin())
            .unwrap()
            .apply(Point3::new(0.5, 0., 0.))
            .unwrap()[0];
        let true_x = exact(tiny) * exact(0.5);
        assert!(exact(x.lower) <= true_x && true_x <= exact(x.upper));
        let t = BlockTransform::try_new(PlanePlacement::default(), 0., Scale3::new(2., 1., 1.))
            .unwrap();
        assert!(PreparedBlockTransform::new(t, origin())
            .unwrap()
            .apply(Point3::new(f64::MAX, 0., 0.))
            .is_none());
        assert!(PreparedBlockTransform::new(t, Point3::new(f64::NAN, 0., 0.)).is_none());
        assert!(PreparedBlockTransform::new(t, origin())
            .unwrap()
            .apply(Point3::new(0., f64::INFINITY, 0.))
            .is_none());
    }

    #[test]
    fn arbitrary_axis_construction_is_robust_and_keeps_rotation_separate() {
        let origin = Point3::new(1., 2., 3.);
        for n in [
            Vector3::new(0., 0., 1.),
            Vector3::new(0., 0., -1.),
            Vector3::new(1., 2., 3.),
            Vector3::new(f64::MAX, 0., f64::MAX),
            Vector3::new(f64::from_bits(1), 0., f64::from_bits(1)),
        ] {
            let t = BlockTransform::from_normal(origin, n, 7., Scale3::new(-1., 2., 3.)).unwrap();
            assert_eq!(t.placement().origin(), origin);
            assert_eq!(t.rotation(), 7.);
            assert_eq!(t.scale(), Scale3::new(-1., 2., 3.));
        }
        let up =
            BlockTransform::from_normal(origin, Vector3::new(0., 0., 1.), 0., Scale3::default())
                .unwrap();
        assert_eq!(up.placement().x_axis(), Vector3::new(1., 0., 0.));
        assert_eq!(up.placement().y_axis(), Vector3::new(0., 1., 0.));
        let down =
            BlockTransform::from_normal(origin, Vector3::new(0., 0., -1.), 0., Scale3::default())
                .unwrap();
        assert_eq!(down.placement().x_axis(), Vector3::new(-1., 0., 0.));
        assert_eq!(down.placement().y_axis(), Vector3::new(0., 1., 0.));
        for n in [
            Vector3::new(0., 0., 0.),
            Vector3::new(f64::NAN, 1., 1.),
            Vector3::new(1., f64::INFINITY, 1.),
            Vector3::new(1., 1., f64::NEG_INFINITY),
        ] {
            assert_eq!(
                BlockTransform::from_normal(origin, n, 0., Scale3::default()),
                Err(BlockTransformError::InvalidNormal)
            );
        }
        assert!(matches!(
            BlockTransform::from_normal(
                Point3::new(f64::NAN, 0., 0.),
                Vector3::new(0., 0., 1.),
                0.,
                Scale3::default()
            ),
            Err(BlockTransformError::InvalidPlacement(_))
        ));
    }

    #[test]
    fn arbitrary_axis_branch_uses_both_normal_components() {
        let construct = |normal| {
            BlockTransform::from_normal(origin(), normal, 0., Scale3::default())
                .unwrap()
                .placement()
        };
        let below = construct(Vector3::new(1. / 128., 0., 1.));
        assert!(below.x_axis().x() > 0.99);
        assert_eq!(below.x_axis().y(), 0.);
        assert!(below.x_axis().z() < 0.);
        let above_x = construct(Vector3::new(1. / 32., 0., 1.));
        assert_eq!(above_x.x_axis(), Vector3::new(0., 1., 0.));
        let above_y = construct(Vector3::new(0., 1. / 32., 1.));
        assert_eq!(above_y.x_axis(), Vector3::new(-1., 0., 0.));
    }

    #[test]
    fn polyline_placement_is_evaluated_before_block_base_and_transform() {
        let local_plane = frame(
            Point3::new(5., 6., 7.),
            Vector3::new(0., 1., 0.),
            Vector3::new(-1., 0., 0.),
        );
        let local_bounds = local_plane
            .enclose_point(crate::ifcdr::Point2::new(2., 3.))
            .unwrap();
        let local = std::array::from_fn(|i| Interval {
            lower: local_bounds.min().components()[i],
            upper: local_bounds.max().components()[i],
        });
        let transform = BlockTransform::try_new(
            frame(
                Point3::new(1., 0., 0.),
                Vector3::new(1., 0., 0.),
                Vector3::new(0., 1., 0.),
            ),
            0.,
            Scale3::new(2., 3., 4.),
        )
        .unwrap();
        let prepared = PreparedBlockTransform::new(transform, Point3::new(2., 0., 0.)).unwrap();
        encloses(prepared.apply_intervals(local).unwrap(), [1., 24., 28.]);
    }

    #[test]
    fn raw_backing_and_public_constructor_share_transform_predicates() {
        let transform = BlockTransform::default();
        assert_eq!(
            BlockTransform::from_validated_components(transform.components()),
            transform
        );
        let mut raw = transform.components();
        raw.placement.x = Vector3::new(0., 0., 0.);
        assert!(matches!(
            raw.validate(),
            Err(BlockTransformError::InvalidPlacement(_))
        ));
        raw = transform.components();
        raw.rotation = f64::NAN;
        assert_eq!(raw.validate(), Err(BlockTransformError::NonFiniteRotation));
        raw = transform.components();
        raw.scale = Scale3::new(1., 0., 1.);
        assert_eq!(
            raw.validate(),
            Err(BlockTransformError::ZeroScale {
                axis: CoordinateAxis::Y
            })
        );
    }
}
