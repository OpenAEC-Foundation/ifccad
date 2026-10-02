#[cfg(test)]
use crate::geometry::numeric::exact;
use crate::geometry::numeric::{round_down, round_up};
use crate::units::{ResolvedTolerance, ToleranceVerdict};
use crate::{OcdrawGeometryTolerance, OcdrawToleranceError};
use num_rational::BigRational;
use num_traits::Zero;
use ocdraw::ocdraw::DrawingLengthUnit;
use opencadcodec::Handle;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OcdrawDistanceInterval {
    pub(crate) lower: f64,
    pub(crate) upper: f64,
}
impl OcdrawDistanceInterval {
    pub fn lower(self) -> f64 {
        self.lower
    }
    pub fn upper(self) -> f64 {
        self.upper
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OcdrawGeometryStatus {
    Empty,
    Exact,
    RoundedWithinTolerance,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OcdrawGeometryEntitySource {
    BlockOccurrence {
        path: Vec<OcdrawGeometryEntitySource>,
        leaf: Box<OcdrawGeometryEntitySource>,
    },
    CadEntity {
        handle: Handle,
        kind: String,
    },
    DrawingEntity {
        scope_id: u32,
        entity_id: u64,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OcdrawGeometryStage {
    SourceEvaluation,
    TargetConstruction,
    DeviationAssessment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OcdrawGeometryFailureReason {
    ProvenExceedance,
    NumericalProofIncomplete,
    CadAxisEvaluationFailed,
    TargetCoordinateOutOfRange,
    DeviationBoundOutOfRange,
}
#[derive(Clone, Debug, PartialEq)]
pub struct OcdrawGeometryFailure {
    pub source: OcdrawGeometryEntitySource,
    pub vertex_index: Option<usize>,
    pub stage: OcdrawGeometryStage,
    pub requested_tolerance: OcdrawGeometryTolerance,
    pub resolved_tolerance: Option<OcdrawDistanceInterval>,
    pub deviation: Option<OcdrawDistanceInterval>,
    pub reason: OcdrawGeometryFailureReason,
}
#[derive(Clone, Debug, PartialEq)]
pub struct OcdrawGeometryAssessment {
    requested: OcdrawGeometryTolerance,
    unit: DrawingLengthUnit,
    resolved: OcdrawDistanceInterval,
    limit: ResolvedTolerance,
    entities: usize,
    vertices: usize,
    rounded: usize,
    maximum: f64,
    worst: Option<OcdrawGeometryEntitySource>,
}
impl OcdrawGeometryAssessment {
    pub(crate) fn new(
        requested: OcdrawGeometryTolerance,
        unit: DrawingLengthUnit,
    ) -> Result<Self, OcdrawToleranceError> {
        let limit = requested.resolve(unit)?;
        let resolved = OcdrawDistanceInterval {
            lower: round_down(&limit.lower).unwrap_or(f64::MAX),
            upper: round_up(&limit.upper).unwrap_or(f64::INFINITY),
        };
        Ok(Self {
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
    pub fn requested_tolerance(&self) -> OcdrawGeometryTolerance {
        self.requested
    }
    pub fn drawing_unit(&self) -> DrawingLengthUnit {
        self.unit
    }
    pub fn resolved_tolerance(&self) -> OcdrawDistanceInterval {
        self.resolved
    }
    pub fn status(&self) -> OcdrawGeometryStatus {
        if self.entities == 0 {
            OcdrawGeometryStatus::Empty
        } else if self.rounded > 0 {
            OcdrawGeometryStatus::RoundedWithinTolerance
        } else {
            OcdrawGeometryStatus::Exact
        }
    }
    pub fn max_deviation_upper_bound(&self) -> f64 {
        self.maximum
    }
    pub fn assessed_entities(&self) -> usize {
        self.entities
    }
    pub fn assessed_vertices(&self) -> usize {
        self.vertices
    }
    pub fn rounded_entities(&self) -> usize {
        self.rounded
    }
    pub fn worst_entity(&self) -> Option<&OcdrawGeometryEntitySource> {
        self.worst.as_ref()
    }
    pub(crate) fn failure(
        &self,
        source: &OcdrawGeometryEntitySource,
        vertex_index: Option<usize>,
        stage: OcdrawGeometryStage,
        reason: OcdrawGeometryFailureReason,
    ) -> Box<OcdrawGeometryFailure> {
        Box::new(OcdrawGeometryFailure {
            source: source.clone(),
            vertex_index,
            stage,
            reason,
            requested_tolerance: self.requested,
            resolved_tolerance: Some(self.resolved),
            deviation: None,
        })
    }
    pub(crate) fn check(
        &self,
        source: &OcdrawGeometryEntitySource,
        vertex: usize,
        d2: &BigRational,
    ) -> Result<f64, Box<OcdrawGeometryFailure>> {
        if d2.is_zero() {
            return Ok(0.0);
        }
        let deviation = crate::geometry::numeric::sqrt_interval(d2).ok_or_else(|| {
            self.failure(
                source,
                Some(vertex),
                OcdrawGeometryStage::DeviationAssessment,
                OcdrawGeometryFailureReason::DeviationBoundOutOfRange,
            )
        })?;
        let verdict = self.limit.check_squared(d2);
        if verdict != ToleranceVerdict::Within {
            let mut failure = self.failure(
                source,
                Some(vertex),
                OcdrawGeometryStage::DeviationAssessment,
                match verdict {
                    ToleranceVerdict::Exceeds => OcdrawGeometryFailureReason::ProvenExceedance,
                    ToleranceVerdict::Unresolved => {
                        OcdrawGeometryFailureReason::NumericalProofIncomplete
                    }
                    ToleranceVerdict::Within => unreachable!(),
                },
            );
            failure.deviation = Some(OcdrawDistanceInterval {
                lower: deviation.0,
                upper: deviation.1,
            });
            return Err(failure);
        }
        Ok(deviation.1)
    }
    pub(crate) fn check_interval(
        &self,
        source: &OcdrawGeometryEntitySource,
        vertex: usize,
        lower: &BigRational,
        upper: &BigRational,
    ) -> Result<f64, Box<OcdrawGeometryFailure>> {
        match self.check(source, vertex, upper) {
            Ok(bound) => Ok(bound),
            Err(mut failure) => {
                if self.limit.check_squared(lower) != ToleranceVerdict::Exceeds {
                    failure.reason = OcdrawGeometryFailureReason::NumericalProofIncomplete;
                }
                if let (Some(lo), Some(hi)) = (
                    crate::geometry::numeric::sqrt_interval(lower),
                    crate::geometry::numeric::sqrt_interval(upper),
                ) {
                    failure.deviation = Some(OcdrawDistanceInterval {
                        lower: lo.0,
                        upper: hi.1,
                    });
                }
                Err(failure)
            }
        }
    }
    pub(crate) fn check_curve_bound(
        &self,
        source: &OcdrawGeometryEntitySource,
        lower: &BigRational,
        upper: &BigRational,
    ) -> Result<f64, Box<OcdrawGeometryFailure>> {
        self.check_interval(source, 0, lower, upper)
            .map_err(|mut failure| {
                failure.vertex_index = None;
                failure
            })
    }
    pub(crate) fn check_curve(
        &self,
        source: &OcdrawGeometryEntitySource,
        curve: &crate::geometry::blocks::PairedCurve,
    ) -> Result<f64, Box<OcdrawGeometryFailure>> {
        let bound = |interval: Option<(BigRational, BigRational)>| {
            interval.ok_or_else(|| {
                self.failure(
                    source,
                    None,
                    OcdrawGeometryStage::DeviationAssessment,
                    OcdrawGeometryFailureReason::DeviationBoundOutOfRange,
                )
            })
        };
        let (lower, upper) = bound(curve.squared_deviation())?;
        match self.check_curve_bound(source, &lower, &upper) {
            Ok(value) => Ok(value),
            Err(mut failure)
                if failure.reason == OcdrawGeometryFailureReason::NumericalProofIncomplete =>
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
    pub(crate) fn record(&mut self, source: OcdrawGeometryEntitySource, count: usize, bound: f64) {
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
    use super::*;
    #[test]
    fn interval_outcomes_distinguish_proof_gap_from_proved_exceedance() {
        let assessment = OcdrawGeometryAssessment::new(
            OcdrawGeometryTolerance::drawing_units(1.).unwrap(),
            DrawingLengthUnit::Metre,
        )
        .unwrap();
        let source = OcdrawGeometryEntitySource::CadEntity {
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
            OcdrawGeometryFailureReason::NumericalProofIncomplete
        );
        let outside = assessment
            .check_interval(&source, 2, &exact(2.), &exact(3.))
            .unwrap_err();
        assert_eq!(
            outside.reason,
            OcdrawGeometryFailureReason::ProvenExceedance
        );
        assert!(matches!(
            crate::CadToOcdrawError::from(uncertain.clone()),
            crate::CadToOcdrawError::Geometry(..)
        ));
        assert!(matches!(
            crate::OcdrawToCadError::from(uncertain),
            crate::OcdrawToCadError::Geometry(..)
        ));
    }
    #[test]
    fn uncertainty_is_a_hard_accuracy_failure_in_both_conversion_directions() {
        let mut a = OcdrawGeometryAssessment::new(
            OcdrawGeometryTolerance::default(),
            DrawingLengthUnit::Parsec,
        )
        .unwrap();
        a.limit = ResolvedTolerance {
            lower: crate::units::q(2, 1),
            upper: crate::units::q(3, 1),
        };
        let source = OcdrawGeometryEntitySource::CadEntity {
            handle: Handle::NULL,
            kind: "LINE".into(),
        };
        let failure = a.check(&source, 0, &crate::units::q(5, 1)).unwrap_err();
        assert_eq!(
            failure.reason,
            OcdrawGeometryFailureReason::NumericalProofIncomplete
        );
        assert!(matches!(
            crate::OcdrawToCadError::from(failure.clone()),
            crate::OcdrawToCadError::Geometry(..)
        ));
        assert!(matches!(
            crate::CadToOcdrawError::from(failure),
            crate::CadToOcdrawError::Geometry(..)
        ));
    }
    #[test]
    fn rational_unit_limit_and_reported_distance_do_not_relax_acceptance() {
        let a = OcdrawGeometryAssessment::new(
            OcdrawGeometryTolerance::default(),
            DrawingLengthUnit::Inch,
        )
        .unwrap();
        let source = OcdrawGeometryEntitySource::CadEntity {
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
        let a = OcdrawGeometryAssessment::new(
            OcdrawGeometryTolerance::drawing_units(1.0).unwrap(),
            DrawingLengthUnit::Unitless,
        )
        .unwrap();
        let source = OcdrawGeometryEntitySource::CadEntity {
            handle: Handle::NULL,
            kind: "LINE".into(),
        };
        assert_eq!(a.check(&source, 0, &exact(1.0)).unwrap(), 1.0);
        assert!(a.check(&source, 0, &exact(1.0_f64.next_up())).is_err());
        assert!(a
            .check(&source, 0, &(exact(0.75) * exact(0.75) * exact(2.0)))
            .is_err());
        let zero = OcdrawGeometryAssessment::new(
            OcdrawGeometryTolerance::exact(),
            DrawingLengthUnit::Unitless,
        )
        .unwrap();
        assert_eq!(zero.check(&source, 0, &exact(0.0)).unwrap(), 0.0);
        assert!(zero.check(&source, 0, &exact(f64::from_bits(1))).is_err());
    }
}
