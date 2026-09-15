#[cfg(test)]
use crate::geometry::numeric::exact;
use crate::geometry::numeric::{round_down, round_up};
use crate::{ConversionGeometryTolerance, ConversionToleranceError};
use cadcodec::Handle;
use ifccad::{
    ifcdr::{EntityId, IfcdrLengthUnit, ScopeId},
    ResourceId,
};
use num_rational::BigRational;
use num_traits::Zero;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConversionDistanceInterval {
    pub(crate) lower: f64,
    pub(crate) upper: f64,
}
impl ConversionDistanceInterval {
    pub fn lower(self) -> f64 {
        self.lower
    }
    pub fn upper(self) -> f64 {
        self.upper
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversionGeometryStatus {
    Empty,
    Exact,
    RoundedWithinTolerance,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConversionEntitySource {
    CadEntity {
        handle: Handle,
        kind: String,
    },
    IfcdrEntity {
        resource_id: ResourceId,
        scope_id: ScopeId,
        entity_id: EntityId,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionGeometryStage {
    SourceEvaluation,
    TargetConstruction,
    DeviationAssessment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionGeometryFailureReason {
    ProvenExceedance,
    CadAxisEvaluationFailed,
    TargetCoordinateOutOfRange,
    DeviationBoundOutOfRange,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConversionGeometryFailure {
    pub source: ConversionEntitySource,
    pub vertex_index: Option<usize>,
    pub stage: ConversionGeometryStage,
    pub requested_tolerance: ConversionGeometryTolerance,
    pub resolved_tolerance: Option<ConversionDistanceInterval>,
    pub deviation: Option<ConversionDistanceInterval>,
    pub reason: ConversionGeometryFailureReason,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConversionGeometryAssessment {
    requested: ConversionGeometryTolerance,
    unit: IfcdrLengthUnit,
    resolved: ConversionDistanceInterval,
    limit: BigRational,
    entities: usize,
    vertices: usize,
    rounded: usize,
    maximum: f64,
    worst: Option<ConversionEntitySource>,
}
impl ConversionGeometryAssessment {
    pub(crate) fn new(
        requested: ConversionGeometryTolerance,
        unit: IfcdrLengthUnit,
    ) -> Result<Self, ConversionToleranceError> {
        let limit = requested.resolve(unit)?;
        let resolved = ConversionDistanceInterval {
            lower: round_down(&limit).unwrap_or(f64::MAX),
            upper: round_up(&limit).unwrap_or(f64::INFINITY),
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
    pub fn requested_tolerance(&self) -> ConversionGeometryTolerance {
        self.requested
    }
    pub fn drawing_unit(&self) -> IfcdrLengthUnit {
        self.unit
    }
    pub fn resolved_tolerance(&self) -> ConversionDistanceInterval {
        self.resolved
    }
    pub fn status(&self) -> ConversionGeometryStatus {
        if self.entities == 0 {
            ConversionGeometryStatus::Empty
        } else if self.rounded > 0 {
            ConversionGeometryStatus::RoundedWithinTolerance
        } else {
            ConversionGeometryStatus::Exact
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
    pub fn worst_entity(&self) -> Option<&ConversionEntitySource> {
        self.worst.as_ref()
    }
    pub(crate) fn failure(
        &self,
        source: &ConversionEntitySource,
        vertex_index: Option<usize>,
        stage: ConversionGeometryStage,
        reason: ConversionGeometryFailureReason,
    ) -> Box<ConversionGeometryFailure> {
        Box::new(ConversionGeometryFailure {
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
        source: &ConversionEntitySource,
        vertex: usize,
        d2: &BigRational,
    ) -> Result<f64, Box<ConversionGeometryFailure>> {
        if d2.is_zero() {
            return Ok(0.0);
        }
        let deviation = crate::geometry::numeric::sqrt_interval(d2).ok_or_else(|| {
            self.failure(
                source,
                Some(vertex),
                ConversionGeometryStage::DeviationAssessment,
                ConversionGeometryFailureReason::DeviationBoundOutOfRange,
            )
        })?;
        if d2 > &(&self.limit * &self.limit) {
            let mut failure = self.failure(
                source,
                Some(vertex),
                ConversionGeometryStage::DeviationAssessment,
                ConversionGeometryFailureReason::ProvenExceedance,
            );
            failure.deviation = Some(ConversionDistanceInterval {
                lower: deviation.0,
                upper: deviation.1,
            });
            return Err(failure);
        }
        Ok(deviation.1)
    }
    pub(crate) fn record(&mut self, source: ConversionEntitySource, count: usize, bound: f64) {
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
    fn rational_unit_limit_and_reported_distance_do_not_relax_acceptance() {
        let a = ConversionGeometryAssessment::new(
            ConversionGeometryTolerance::default(),
            IfcdrLengthUnit::Inch,
        )
        .unwrap();
        let source = ConversionEntitySource::CadEntity {
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
        let a = ConversionGeometryAssessment::new(
            ConversionGeometryTolerance::drawing_units(1.0).unwrap(),
            IfcdrLengthUnit::Unitless,
        )
        .unwrap();
        let source = ConversionEntitySource::CadEntity {
            handle: Handle::NULL,
            kind: "LINE".into(),
        };
        assert_eq!(a.check(&source, 0, &exact(1.0)).unwrap(), 1.0);
        assert!(a.check(&source, 0, &exact(1.0_f64.next_up())).is_err());
        assert!(a
            .check(&source, 0, &(exact(0.75) * exact(0.75) * exact(2.0)))
            .is_err());
        let zero = ConversionGeometryAssessment::new(
            ConversionGeometryTolerance::exact(),
            IfcdrLengthUnit::Unitless,
        )
        .unwrap();
        assert_eq!(zero.check(&source, 0, &exact(0.0)).unwrap(), 0.0);
        assert!(zero.check(&source, 0, &exact(f64::from_bits(1))).is_err());
    }
}
