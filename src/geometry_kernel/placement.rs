use super::numeric::{exact, round_down, round_nearest, round_up, Interval};
use super::values::{Bounds3d, CoordinateAxis, Point3, Vector3};
use crate::geometry_kernel::Point2;
use num_rational::BigRational;
use num_traits::Signed;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateFrameField {
    Origin,
    XAxis,
    YAxis,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaneAxis {
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum CoordinateFrameError {
    #[error("arbitrary-axis construction requires a finite nonzero normal")]
    InvalidNormal,
    #[error("non-finite {axis:?} component of plane {field:?}")]
    NonFiniteComponent {
        field: CoordinateFrameField,
        axis: CoordinateAxis,
    },
    #[error("plane axis {axis:?} squared length is outside the unit tolerance")]
    AxisLengthOutsideTolerance { axis: PlaneAxis },
    #[error("plane axes are not perpendicular within the frame tolerance")]
    AxesNotPerpendicular,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum GeometryEvaluationError {
    #[error("non-finite local {axis:?} coordinate")]
    NonFiniteLocalCoordinate { axis: CoordinateAxis },
    #[error("exact placed {axis:?} coordinate is outside the finite binary64 range")]
    CoordinateOutOfRange { axis: CoordinateAxis },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CoordinateFrameComponents {
    pub origin: Point3,
    pub x: Vector3,
    pub y: Vector3,
}

impl CoordinateFrameComponents {
    pub(crate) fn validate(self) -> Result<(), CoordinateFrameError> {
        for (field, values) in [
            (CoordinateFrameField::Origin, self.origin.components()),
            (CoordinateFrameField::XAxis, self.x.components()),
            (CoordinateFrameField::YAxis, self.y.components()),
        ] {
            for (axis, value) in CoordinateAxis::ALL.into_iter().zip(values) {
                if !value.is_finite() {
                    return Err(CoordinateFrameError::NonFiniteComponent { field, axis });
                }
            }
        }
        if self == CoordinateFrame3::default().components() {
            return Ok(());
        }
        let tolerance = exact(f64::from_bits(0x3d719799812dea11));
        let dot = |a: [f64; 3], b: [f64; 3]| -> BigRational {
            a.into_iter().zip(b).map(|(a, b)| exact(a) * exact(b)).sum()
        };
        for (axis, vector) in [(PlaneAxis::X, self.x), (PlaneAxis::Y, self.y)] {
            if (dot(vector.components(), vector.components()) - exact(1.0)).abs() > tolerance {
                return Err(CoordinateFrameError::AxisLengthOutsideTolerance { axis });
            }
        }
        if dot(self.x.components(), self.y.components()).abs() > tolerance {
            return Err(CoordinateFrameError::AxesNotPerpendicular);
        }
        Ok(())
    }
}

/// An immutable 3D coordinate frame, validated without normalizing its axes.
///
/// The default is the complete XY placement at the origin. Explicit placements
/// require an origin and both directions; validation never fills components in.
/// Axis squared lengths and their dot product are tested exactly against the
/// binary64 value nearest to `1e-12`. Evaluation returns owning-scope coordinates,
/// without applying a scope base or any external semantic placement.
///
/// ```
/// use ocdraw::ocdraw::{CoordinateFrame3, Point2, Point3, Vector3};
/// let plane = CoordinateFrame3::try_new(
///     Point3::new(10.0, 20.0, 30.0),
///     Vector3::new(0.0, 1.0, 0.0),
///     Vector3::new(-1.0, 0.0, 0.0),
/// )?;
/// assert_eq!(plane.try_to_scope_point(Point2::new(2.0, 3.0))?,
///            Point3::new(7.0, 22.0, 30.0));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordinateFrame3(CoordinateFrameComponents);

impl Default for CoordinateFrame3 {
    fn default() -> Self {
        Self(CoordinateFrameComponents {
            origin: Point3::new(0.0, 0.0, 0.0),
            x: Vector3::new(1.0, 0.0, 0.0),
            y: Vector3::new(0.0, 1.0, 0.0),
        })
    }
}

impl CoordinateFrame3 {
    pub(crate) fn from_validated_components(value: CoordinateFrameComponents) -> Self {
        Self(value)
    }
    pub(crate) fn enclosed_by(self, point: Point2, bounds: Bounds3d) -> bool {
        for i in 0..3 {
            let lo = bounds.min.components()[i];
            let hi = bounds.max.components()[i];
            if let Some(interval) = self.coordinate_interval(point, i) {
                if lo <= interval.lower && interval.upper <= hi {
                    continue;
                }
                if hi < interval.lower || interval.upper < lo {
                    return false;
                }
            }
            let value = self.exact_coordinate(point, i);
            if value < exact(lo) || value > exact(hi) {
                return false;
            }
        }
        true
    }
    pub fn try_new(origin: Point3, x: Vector3, y: Vector3) -> Result<Self, CoordinateFrameError> {
        let components = CoordinateFrameComponents { origin, x, y };
        components.validate()?;
        Ok(Self(components))
    }

    /// Constructs a frame from a finite nonzero normal using arbitrary axes.
    ///
    /// The normal is explicitly normalized here using scaled arithmetic. The
    /// origin is already in the owning scope, not in CAD OCS coordinates. This
    /// constructor does not change the strict preserved-axis behavior of
    /// [`Self::try_new`], and does not establish a CAD conversion accuracy proof.
    ///
    /// ```
    /// use ocdraw::geometry_kernel::{CoordinateFrame3, Point3, Vector3};
    /// let frame = CoordinateFrame3::try_from_normal_arbitrary_axis(
    ///     Point3::new(10., 20., 30.), Vector3::new(0., 2., 0.),
    /// )?;
    /// assert_eq!(frame.x_axis().components(), [-1., 0., 0.]);
    /// assert_eq!(frame.y_axis().components(), [0., 0., 1.]);
    /// # Ok::<(), ocdraw::geometry_kernel::CoordinateFrameError>(())
    /// ```
    pub fn try_from_normal_arbitrary_axis(
        origin: Point3,
        normal: Vector3,
    ) -> Result<Self, CoordinateFrameError> {
        fn normalized(v: [f64; 3]) -> Result<[f64; 3], CoordinateFrameError> {
            if !v.into_iter().all(f64::is_finite) {
                return Err(CoordinateFrameError::InvalidNormal);
            }
            let scale = v.into_iter().map(f64::abs).fold(0.0, f64::max);
            if scale == 0.0 {
                return Err(CoordinateFrameError::InvalidNormal);
            }
            let v = v.map(|value| value / scale);
            let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            Ok(v.map(|value| value / length))
        }
        fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        }
        let n = normalized(normal.components())?;
        let x = if n[0].abs() < 1. / 64. && n[1].abs() < 1. / 64. {
            [n[2], 0., -n[0]]
        } else {
            [-n[1], n[0], 0.]
        };
        let x = normalized(x)?;
        let y = cross(n, x);
        Self::try_new(
            origin,
            Vector3::new(x[0], x[1], x[2]),
            Vector3::new(y[0], y[1], y[2]),
        )
    }
    pub fn origin(self) -> Point3 {
        self.0.origin
    }
    pub fn x_axis(self) -> Vector3 {
        self.0.x
    }
    pub fn y_axis(self) -> Vector3 {
        self.0.y
    }
    pub(crate) fn components(self) -> CoordinateFrameComponents {
        self.0
    }

    /// Evaluate a local point, rounding each exact coordinate once to nearest/even.
    pub fn try_to_scope_point(self, point: Point2) -> Result<Point3, GeometryEvaluationError> {
        Self::validate_point(point)?;
        let mut result = [0.0; 3];
        for (i, axis) in CoordinateAxis::ALL.into_iter().enumerate() {
            // A singleton enclosure proves the exact value; a wider enclosure
            // cannot establish correctly rounded output on its own.
            result[i] = match self.coordinate_interval(point, i) {
                Some(interval) if interval.lower == interval.upper => interval.lower,
                _ => round_nearest(&self.exact_coordinate(point, i))
                    .map_err(|_| GeometryEvaluationError::CoordinateOutOfRange { axis })?,
            };
        }
        Ok(Point3::new(result[0], result[1], result[2]))
    }
    pub(crate) fn enclose_point(self, point: Point2) -> Result<Bounds3d, GeometryEvaluationError> {
        Self::validate_point(point)?;
        let mut lower = [0.0; 3];
        let mut upper = [0.0; 3];
        for (i, axis) in CoordinateAxis::ALL.into_iter().enumerate() {
            let interval = match self.coordinate_interval(point, i) {
                Some(interval) => interval,
                None => {
                    let value = self.exact_coordinate(point, i);
                    let error = |_| GeometryEvaluationError::CoordinateOutOfRange { axis };
                    Interval {
                        lower: round_down(&value).map_err(error)?,
                        upper: round_up(&value).map_err(error)?,
                    }
                }
            };
            lower[i] = interval.lower;
            upper[i] = interval.upper;
        }
        Ok(Bounds3d {
            min: Point3::new(lower[0], lower[1], lower[2]),
            max: Point3::new(upper[0], upper[1], upper[2]),
        })
    }

    fn validate_point(point: Point2) -> Result<(), GeometryEvaluationError> {
        for (axis, value) in [
            (CoordinateAxis::X, point.x()),
            (CoordinateAxis::Y, point.y()),
        ] {
            if !value.is_finite() {
                return Err(GeometryEvaluationError::NonFiniteLocalCoordinate { axis });
            }
        }
        Ok(())
    }

    fn coordinate_interval(self, point: Point2, i: usize) -> Option<Interval> {
        Interval::point(self.0.origin.components()[i])
            .add(Interval::point(point.x()).mul(Interval::point(self.0.x.components()[i]))?)?
            .add(Interval::point(point.y()).mul(Interval::point(self.0.y.components()[i]))?)
    }

    pub(crate) fn exact_coordinate(self, point: Point2, i: usize) -> BigRational {
        #[cfg(test)]
        super::numeric::metrics::EXACT_COORDINATES
            .set(super::numeric::metrics::EXACT_COORDINATES.get() + 1);
        exact(self.0.origin.components()[i])
            + exact(point.x()) * exact(self.0.x.components()[i])
            + exact(point.y()) * exact(self.0.y.components()[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_frame3_preserves_oblique_axes() {
        let x = Vector3::new(0.0, 1.0, 0.0);
        let y = Vector3::new(0.0, 0.0, 1.0);
        let frame = CoordinateFrame3::try_new(Point3::new(3.0, 4.0, 5.0), x, y).unwrap();
        assert_eq!(frame.origin(), Point3::new(3.0, 4.0, 5.0));
        assert_eq!(frame.x_axis(), x);
        assert_eq!(frame.y_axis(), y);
        assert_eq!(
            CoordinateFrame3::try_new(Point3::new(0.0, 0.0, 0.0), x, x),
            Err(CoordinateFrameError::AxesNotPerpendicular)
        );
    }

    #[test]
    fn varied_points_and_accepted_skew_have_exact_enclosures() {
        let origin = Point3::new(13.25, -97.625, 41.0);
        let x = Vector3::new(1.0, 0.0, 0.0);
        let y = Vector3::new(1e-13, 1.0, 0.0);
        let plane = CoordinateFrame3::try_new(origin, x, y).unwrap();
        assert_eq!(plane.origin(), origin);
        assert_eq!(plane.x_axis(), x);
        assert_eq!(plane.y_axis(), y);
        let mut seed = 123_u64;
        for _ in 0..128 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u = f64::from_bits(seed & 0xffdfffffffffffff);
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let v = f64::from_bits(seed & 0xffdfffffffffffff);
            let point = Point2::new(u, v);
            let bounds = plane.enclose_point(point).unwrap();
            let evaluated = plane.try_to_scope_point(point).unwrap();
            for (i, expected) in [
                exact(13.25) + exact(u) + exact(v) * exact(1e-13),
                exact(-97.625) + exact(v),
                exact(41.0),
            ]
            .into_iter()
            .enumerate()
            {
                assert!(exact(bounds.min.components()[i]) <= expected);
                assert!(expected <= exact(bounds.max.components()[i]));
                assert_eq!(evaluated.components()[i], round_nearest(&expected).unwrap());
            }
        }
    }

    #[test]
    fn squared_axis_length_uses_the_same_exact_threshold() {
        for length in [1.0 - 6e-13, 1.0 - 4e-13, 1.0, 1.0 + 4e-13, 1.0 + 6e-13] {
            let expected = (exact(length) * exact(length) - exact(1.0)).abs()
                <= exact(f64::from_bits(0x3d719799812dea11));
            let result = CoordinateFrame3::try_new(
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(length, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            );
            assert_eq!(result.is_ok(), expected);
            if let Ok(plane) = result {
                assert_eq!(plane.x_axis().x(), length);
            }
        }
    }

    #[test]
    #[ignore = "release-mode numerical performance measurement"]
    fn kernel_performance_measurement() {
        use super::super::numeric::metrics;
        use std::{hint::black_box, time::Instant};
        let half = 0.5_f64.sqrt();
        let oblique = CoordinateFrame3::try_new(
            Point3::new(123.4, -567.8, 90.1),
            Vector3::new(half, half, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        let points: Vec<_> = (0..10_000)
            .map(|i| Point2::new(i as f64 * 0.17, i as f64 * -0.31))
            .collect();
        for (name, plane) in [
            ("identity", CoordinateFrame3::default()),
            ("oblique", oblique),
        ] {
            for _ in 0..2 {
                for point in &points {
                    black_box(plane.try_to_scope_point(black_box(*point)).unwrap());
                }
            }
            let mut times = Vec::new();
            for _ in 0..5 {
                metrics::reset();
                let start = Instant::now();
                for point in &points {
                    black_box(plane.try_to_scope_point(black_box(*point)).unwrap());
                }
                times.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            times.sort_by(f64::total_cmp);
            println!(
                "{name}: points=10000 median_ms={} min_ms={} max_ms={} exact/candidate/search={:?}",
                times[2],
                times[0],
                times[4],
                metrics::counts()
            );
            if name == "identity" {
                assert_eq!(metrics::counts(), (0, 0, 0));
            }
        }
    }

    #[test]
    fn perpendicularity_boundary_is_inclusive_and_preserves_axes() {
        let tau = f64::from_bits(0x3d719799812dea11);
        let make = |s| {
            CoordinateFrame3::try_new(
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(s, 1.0, 0.0),
            )
        };
        assert!(make(tau.next_down()).is_ok());
        assert_eq!(make(tau).unwrap().y_axis().x(), tau);
        assert_eq!(
            make(tau.next_up()),
            Err(CoordinateFrameError::AxesNotPerpendicular)
        );
    }

    #[test]
    fn rejects_nonfinite_and_nonunit_axes_without_repair() {
        let o = Point3::new(0.0, 0.0, 0.0);
        let x = Vector3::new(1.0, 0.0, 0.0);
        let y = Vector3::new(0.0, 1.0, 0.0);
        assert!(matches!(
            CoordinateFrame3::try_new(Point3::new(f64::NAN, 0.0, 0.0), x, y),
            Err(CoordinateFrameError::NonFiniteComponent {
                field: CoordinateFrameField::Origin,
                ..
            })
        ));
        assert_eq!(
            CoordinateFrame3::try_new(o, Vector3::new(2.0, 0.0, 0.0), y),
            Err(CoordinateFrameError::AxisLengthOutsideTolerance { axis: PlaneAxis::X })
        );
        assert_eq!(
            CoordinateFrame3::try_new(o, x, Vector3::new(0.0, 0.0, 0.0)),
            Err(CoordinateFrameError::AxisLengthOutsideTolerance { axis: PlaneAxis::Y })
        );
    }

    #[test]
    fn maps_rotated_translated_points_and_identity() {
        let p = Point2::new(2.0, 3.0);
        assert_eq!(
            CoordinateFrame3::default().try_to_scope_point(p),
            Ok(Point3::new(2.0, 3.0, 0.0))
        );
        let plane = CoordinateFrame3::try_new(
            Point3::new(10.0, 20.0, 30.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
        )
        .unwrap();
        assert_eq!(
            plane.try_to_scope_point(p),
            Ok(Point3::new(7.0, 22.0, 30.0))
        );
        let bounds = plane.enclose_point(p).unwrap();
        assert!(bounds.min().x() <= 7.0 && bounds.max().x() >= 7.0);
        assert!(bounds.min().y() <= 22.0 && bounds.max().y() >= 22.0);
    }

    #[test]
    fn exact_evaluation_survives_intermediate_cancellation() {
        let half = 0.5_f64.sqrt();
        let plane = CoordinateFrame3::try_new(
            Point3::new(f64::MAX, 0.0, 0.0),
            Vector3::new(half, half, 0.0),
            Vector3::new(-half, half, 0.0),
        )
        .unwrap();
        // The X expression overflows in ordinary left-to-right arithmetic, but
        // its exact result is MAX. Keep Y finite with a smaller local magnitude.
        let p = Point2::new(f64::MAX * 0.5, f64::MAX * 0.5);
        let result = plane.try_to_scope_point(p).unwrap();
        assert_eq!(result.x(), f64::MAX);
        assert!(result.y().is_finite());
        assert_eq!(plane.enclose_point(p).unwrap().max().x(), f64::MAX);
    }

    #[test]
    fn tight_bounds_enclose_exact_geometry_not_rounded_point() {
        let plane = CoordinateFrame3::try_new(
            Point3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let point = Point2::new(f64::EPSILON / 4.0, 0.0);
        assert_eq!(plane.try_to_scope_point(point).unwrap().x(), 1.0);
        let tight = Bounds3d {
            min: Point3::new(1.0, 0.0, 0.0),
            max: Point3::new(1.0, 0.0, 0.0),
        };
        assert!(!plane.enclosed_by(point, tight));
        let outward = Bounds3d {
            max: Point3::new(1.0_f64.next_up(), 0.0, 0.0),
            ..tight
        };
        assert!(plane.enclosed_by(point, outward));
        // Cancellation restores an exact boundary even when interval arithmetic
        // alone cannot prove enclosure.
        let cancellation = Point2::new(-1.0, 0.0);
        let zero = Bounds3d {
            min: Point3::new(0.0, 0.0, 0.0),
            max: Point3::new(0.0, 0.0, 0.0),
        };
        assert!(plane.enclosed_by(cancellation, zero));
    }

    #[test]
    fn exact_out_of_range_and_nonfinite_input_are_explicit() {
        let plane = CoordinateFrame3::try_new(
            Point3::new(f64::MAX, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        assert_eq!(
            plane.try_to_scope_point(Point2::new(1.0, 0.0)),
            Err(GeometryEvaluationError::CoordinateOutOfRange {
                axis: CoordinateAxis::X
            })
        );
        assert_eq!(
            plane.try_to_scope_point(Point2::new(0.0, f64::INFINITY)),
            Err(GeometryEvaluationError::NonFiniteLocalCoordinate {
                axis: CoordinateAxis::Y
            })
        );
    }
}
