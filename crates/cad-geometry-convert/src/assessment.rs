#[cfg(test)]
use crate::geometry::numeric::exact;
use crate::geometry::numeric::{round_down, round_up};
use crate::units::{ResolvedTolerance, ToleranceVerdict};
use crate::{GeometryTolerance, GeometryToleranceError};
use num_rational::BigRational;
use num_traits::Zero;
use ocdraw::geometry_kernel::CoordinateLengthUnit as DrawingLengthUnit;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistanceInterval {
    pub lower: f64,
    pub upper: f64,
}
impl DistanceInterval {
    pub fn lower(self) -> f64 {
        self.lower
    }
    pub fn upper(self) -> f64 {
        self.upper
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeometryStatus {
    Empty,
    Exact,
    RoundedWithinTolerance,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryStage {
    SourceEvaluation,
    TargetConstruction,
    DeviationAssessment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryFailureReason {
    ProvenExceedance,
    NumericalProofIncomplete,
    CadAxisEvaluationFailed,
    TargetCoordinateOutOfRange,
    DeviationBoundOutOfRange,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryFailure<S> {
    pub source: S,
    pub vertex_index: Option<usize>,
    pub stage: GeometryStage,
    pub requested_tolerance: GeometryTolerance,
    pub resolved_tolerance: Option<DistanceInterval>,
    pub deviation: Option<DistanceInterval>,
    pub reason: GeometryFailureReason,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryAssessment<S> {
    unassessed: Vec<S>,
    requested: GeometryTolerance,
    unit: DrawingLengthUnit,
    resolved: DistanceInterval,
    limit: ResolvedTolerance,
    entities: usize,
    vertices: usize,
    rounded: usize,
    maximum: f64,
    worst: Option<S>,
}
impl<S: Clone> GeometryAssessment<S> {
    pub fn new(
        requested: GeometryTolerance,
        unit: DrawingLengthUnit,
    ) -> Result<Self, GeometryToleranceError> {
        let limit = requested.resolve(unit)?;
        let resolved = DistanceInterval {
            lower: round_down(&limit.lower).unwrap_or(f64::MAX),
            upper: round_up(&limit.upper).unwrap_or(f64::INFINITY),
        };
        Ok(Self {
            unassessed: Vec::new(),
            requested,
            unit,
            resolved,
            limit,
            entities: 0,
            vertices: 0,
            rounded: 0,
            maximum: 0.0,
            worst: None,
        })
    }
    pub fn requested_tolerance(&self) -> GeometryTolerance {
        self.requested
    }
    pub fn drawing_unit(&self) -> DrawingLengthUnit {
        self.unit
    }
    pub fn resolved_tolerance(&self) -> DistanceInterval {
        self.resolved
    }
    pub fn status(&self) -> GeometryStatus {
        if self.entities == 0 {
            GeometryStatus::Empty
        } else if self.rounded > 0 {
            GeometryStatus::RoundedWithinTolerance
        } else {
            GeometryStatus::Exact
        }
    }
    pub fn max_deviation_upper_bound(&self) -> f64 {
        self.maximum
    }
    pub fn assessed_entities(&self) -> usize {
        self.entities
    }
    /// Retained source geometries for which no primitive proof was performed.
    pub fn unassessed_sources(&self) -> &[S] {
        &self.unassessed
    }
    /// Whether all retained sources registered by the adapter were assessed.
    pub fn is_complete(&self) -> bool {
        self.unassessed.is_empty()
    }
    pub fn record_unassessed(&mut self, source: S) {
        self.unassessed.push(source);
    }
    pub fn assessed_vertices(&self) -> usize {
        self.vertices
    }
    pub fn rounded_entities(&self) -> usize {
        self.rounded
    }
    pub fn worst_entity(&self) -> Option<&S> {
        self.worst.as_ref()
    }
    pub fn failure(
        &self,
        source: &S,
        vertex_index: Option<usize>,
        stage: GeometryStage,
        reason: GeometryFailureReason,
    ) -> Box<GeometryFailure<S>> {
        Box::new(GeometryFailure {
            source: source.clone(),
            vertex_index,
            stage,
            reason,
            requested_tolerance: self.requested,
            resolved_tolerance: Some(self.resolved),
            deviation: None,
        })
    }
    pub fn check(
        &self,
        source: &S,
        vertex: usize,
        d2: &BigRational,
    ) -> Result<f64, Box<GeometryFailure<S>>> {
        if d2.is_zero() {
            return Ok(0.0);
        }
        let deviation = crate::geometry::numeric::sqrt_interval(d2).ok_or_else(|| {
            self.failure(
                source,
                Some(vertex),
                GeometryStage::DeviationAssessment,
                GeometryFailureReason::DeviationBoundOutOfRange,
            )
        })?;
        let verdict = self.limit.check_squared(d2);
        if verdict != ToleranceVerdict::Within {
            let mut failure = self.failure(
                source,
                Some(vertex),
                GeometryStage::DeviationAssessment,
                match verdict {
                    ToleranceVerdict::Exceeds => GeometryFailureReason::ProvenExceedance,
                    ToleranceVerdict::Unresolved => GeometryFailureReason::NumericalProofIncomplete,
                    ToleranceVerdict::Within => unreachable!(),
                },
            );
            failure.deviation = Some(DistanceInterval {
                lower: deviation.0,
                upper: deviation.1,
            });
            return Err(failure);
        }
        Ok(deviation.1)
    }
    pub fn check_interval(
        &self,
        source: &S,
        vertex: usize,
        lower: &BigRational,
        upper: &BigRational,
    ) -> Result<f64, Box<GeometryFailure<S>>> {
        match self.check(source, vertex, upper) {
            Ok(bound) => Ok(bound),
            Err(mut failure) => {
                if self.limit.check_squared(lower) != ToleranceVerdict::Exceeds {
                    failure.reason = GeometryFailureReason::NumericalProofIncomplete;
                }
                if let (Some(lo), Some(hi)) = (
                    crate::geometry::numeric::sqrt_interval(lower),
                    crate::geometry::numeric::sqrt_interval(upper),
                ) {
                    failure.deviation = Some(DistanceInterval {
                        lower: lo.0,
                        upper: hi.1,
                    });
                }
                Err(failure)
            }
        }
    }
    pub fn check_curve_bound(
        &self,
        source: &S,
        lower: &BigRational,
        upper: &BigRational,
    ) -> Result<f64, Box<GeometryFailure<S>>> {
        self.check_interval(source, 0, lower, upper)
            .map_err(|mut failure| {
                failure.vertex_index = None;
                failure
            })
    }
    pub fn check_curve(
        &self,
        source: &S,
        curve: &crate::geometry::blocks::PairedCurve,
    ) -> Result<f64, Box<GeometryFailure<S>>> {
        let bound = |interval: Option<(BigRational, BigRational)>| {
            interval.ok_or_else(|| {
                self.failure(
                    source,
                    None,
                    GeometryStage::DeviationAssessment,
                    GeometryFailureReason::DeviationBoundOutOfRange,
                )
            })
        };
        let (lower, upper) = bound(curve.squared_deviation())?;
        match self.check_curve_bound(source, &lower, &upper) {
            Ok(value) => Ok(value),
            Err(mut failure)
                if failure.reason == GeometryFailureReason::NumericalProofIncomplete =>
            {
                for segments in [2, 4, 8, 16, 32] {
                    let (lower, upper) = bound(curve.refined_squared_deviation(segments))?;
                    match self.check_curve_bound(source, &lower, &upper) {
                        Ok(value) => return Ok(value),
                        Err(next) => failure = next,
                    }
                }
                Err(failure)
            }
            Err(failure) => Err(failure),
        }
    }
    pub fn record(&mut self, source: S, count: usize, bound: f64) {
        self.entities += 1;
        self.vertices += count;
        if bound > 0.0 {
            self.rounded += 1;
        }
        if bound > self.maximum {
            self.maximum = bound;
            self.worst = Some(source);
        }
    }
}
#[cfg(test)]
mod tests {
    use opencadcodec::Handle;
    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    enum TestSource {
        CadEntity { handle: Handle, kind: String },
    }

    use super::*;
    #[test]
    fn interval_outcomes_distinguish_proof_gap_from_proved_exceedance() {
        let assessment = GeometryAssessment::new(
            GeometryTolerance::drawing_units(1.).unwrap(),
            DrawingLengthUnit::Metre,
        )
        .unwrap();
        let source = TestSource::CadEntity {
            handle: Handle::NULL,
            kind: "INSERT".into(),
        };
        assert!(assessment
            .check_interval(&source, 0, &exact(0.), &exact(1.))
            .is_ok());
        let uncertain = assessment
            .check_interval(&source, 1, &exact(0.5), &exact(2.))
            .unwrap_err();
        assert_eq!(
            uncertain.reason,
            GeometryFailureReason::NumericalProofIncomplete
        );
        let outside = assessment
            .check_interval(&source, 2, &exact(2.), &exact(3.))
            .unwrap_err();
        assert_eq!(outside.reason, GeometryFailureReason::ProvenExceedance);
    }
    #[test]
    fn uncertainty_is_a_hard_accuracy_failure_in_both_conversion_directions() {
        let mut a =
            GeometryAssessment::new(GeometryTolerance::default(), DrawingLengthUnit::Parsec)
                .unwrap();
        a.limit = ResolvedTolerance {
            lower: crate::units::q(2, 1),
            upper: crate::units::q(3, 1),
        };
        let source = TestSource::CadEntity {
            handle: Handle::NULL,
            kind: "LINE".into(),
        };
        let failure = a.check(&source, 0, &crate::units::q(5, 1)).unwrap_err();
        assert_eq!(
            failure.reason,
            GeometryFailureReason::NumericalProofIncomplete
        );
    }
    #[test]
    fn rational_unit_limit_and_reported_distance_do_not_relax_acceptance() {
        let a =
            GeometryAssessment::new(GeometryTolerance::default(), DrawingLengthUnit::Inch).unwrap();
        let source = TestSource::CadEntity {
            handle: Handle::NULL,
            kind: "LINE".into(),
        };
        let limit = BigRational::new(1.into(), 25400.into());
        let squared = &limit * &limit;
        let reported = a.check(&source, 0, &squared).unwrap();
        assert!(exact(reported) * exact(reported) >= squared);
        assert!(a
            .check(
                &source,
                0,
                &(&squared + BigRational::new(1.into(), num_bigint::BigInt::from(1_u8) << 200))
            )
            .is_err());
        assert!(exact(a.resolved_tolerance().lower()) <= limit);
        assert!(exact(a.resolved_tolerance().upper()) >= limit);
    }

    #[test]
    fn exact_euclidean_tolerance_is_inclusive() {
        let a = GeometryAssessment::new(
            GeometryTolerance::drawing_units(1.0).unwrap(),
            DrawingLengthUnit::Unitless,
        )
        .unwrap();
        let source = TestSource::CadEntity {
            handle: Handle::NULL,
            kind: "LINE".into(),
        };
        assert_eq!(a.check(&source, 0, &exact(1.0)).unwrap(), 1.0);
        assert!(a.check(&source, 0, &exact(1.0_f64.next_up())).is_err());
        assert!(a
            .check(&source, 0, &(exact(0.75) * exact(0.75) * exact(2.0)))
            .is_err());
        let zero = GeometryAssessment::new(GeometryTolerance::exact(), DrawingLengthUnit::Unitless)
            .unwrap();
        assert_eq!(zero.check(&source, 0, &exact(0.0)).unwrap(), 0.0);
        assert!(zero.check(&source, 0, &exact(f64::from_bits(1))).is_err());
    }
}
