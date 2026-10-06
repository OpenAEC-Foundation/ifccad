//! Temporary IFCCAD conversion state over the shared geometric proof engine.
//! Tracks the active native owner and one assessment per coordinate domain:
//! Model/definitions use drawing units; each Paper layout has its own unit.
//! OCDraw currently uses the shared ExchangeState with one drawing assessment
//! directly. Native geometry records and wire payloads do not store this state.
use crate::{geometry_assessment::Assessment, *};
use ocdraw::geometry_kernel::CoordinateLengthUnit;
use std::collections::BTreeMap;
pub(crate) struct GeometryContext {
    pub state: cad_geometry_convert::ExchangeState<u64, IfccadGeometryEntitySource>,
    pub owner: IfccadGeometryOwner,
    stored: BTreeMap<IfccadGeometryDomain, Assessment>,
    tolerance: IfccadGeometryTolerance,
}
impl GeometryContext {
    pub fn new(
        tolerance: IfccadGeometryTolerance,
        unit: &str,
        model_id: u64,
    ) -> Result<Self, IfccadConversionError> {
        let unit =
            CoordinateLengthUnit::from_token(unit).expect("validated/selected coordinate unit");
        let assessment =
            Assessment::new(tolerance, unit).map_err(IfccadConversionError::Tolerance)?;
        Ok(Self {
            state: cad_geometry_convert::ExchangeState::new(assessment),
            owner: IfccadGeometryOwner::ModelLayout(model_id),
            stored: BTreeMap::new(),
            tolerance,
        })
    }
    pub fn add_paper(&mut self, id: u64, unit: &str) -> Result<(), IfccadConversionError> {
        let unit = CoordinateLengthUnit::from_token(unit).expect("validated/selected Paper unit");
        let assessment =
            Assessment::new(self.tolerance, unit).map_err(IfccadConversionError::Tolerance)?;
        self.stored
            .insert(IfccadGeometryDomain::PaperLayout(id), assessment);
        Ok(())
    }
    pub fn select(&mut self, owner: IfccadGeometryOwner) {
        let old = self.owner.domain();
        let next = owner.domain();
        if old != next {
            let assessment = self
                .stored
                .remove(&next)
                .expect("registered coordinate domain");
            let previous = std::mem::replace(&mut self.state.assessment, assessment);
            self.stored.insert(old, previous);
        }
        self.owner = owner;
    }
    pub fn numerical_failure(
        &self,
        failure: Box<cad_geometry_convert::GeometryFailure<IfccadGeometryEntitySource>>,
    ) -> IfccadConversionError {
        IfccadConversionError::Geometry(Box::new(IfccadGeometryFailure {
            domain: self.owner.domain(),
            unit: self.state.assessment.drawing_unit(),
            failure,
        }))
    }
    pub fn preparation_failure(
        &self,
        source: &IfccadGeometryEntitySource,
        error: cad_geometry_convert::CadPreparationError,
        stage: IfccadGeometryStage,
    ) -> IfccadConversionError {
        let (stage, reason) = match error {
            cad_geometry_convert::CadPreparationError::Numerical { stage, reason } => {
                (stage, reason)
            }
            cad_geometry_convert::CadPreparationError::OutOfRange => (
                stage,
                IfccadGeometryFailureReason::TargetCoordinateOutOfRange,
            ),
            _ => (stage, IfccadGeometryFailureReason::CadAxisEvaluationFailed),
        };
        self.numerical_failure(self.state.assessment.failure(source, None, stage, reason))
    }
    pub fn record(
        &mut self,
        key: u64,
        source: IfccadGeometryEntitySource,
        pair: cad_geometry_convert::GeometryPair,
        issues: &mut Vec<IfccadDiagnostic>,
    ) -> Result<(), IfccadConversionError> {
        let bound = self
            .state
            .record_geometry(key, source.clone(), pair)
            .map_err(|e| self.numerical_failure(e))?;
        if bound > 0. {
            issues.push(crate::diagnostics::geometry_rounding(
                &source,
                self.owner.domain(),
                bound,
            ));
        }
        Ok(())
    }
    pub fn assess_roots(
        &mut self,
        members: &BTreeMap<u64, Vec<u64>>,
        roots: &[u64],
        issues: &mut Vec<IfccadDiagnostic>,
    ) -> Result<(), IfccadConversionError> {
        let mut assessment = self.state.assessment.clone();
        let records = self
            .state
            .assess_roots(members, roots, &mut assessment)
            .map_err(|e| self.numerical_failure(e))?;
        self.state.assessment = assessment;
        for record in records {
            if record.deviation.upper() > 0. {
                let source = IfccadGeometryEntitySource::BlockOccurrence {
                    path: record
                        .path
                        .iter()
                        .map(|key| self.state.identity(*key).clone())
                        .collect(),
                    leaf: Box::new(record.leaf),
                };
                issues.push(crate::diagnostics::geometry_rounding(
                    &source,
                    self.owner.domain(),
                    record.deviation.upper(),
                ));
            }
        }
        Ok(())
    }
    pub fn finish(mut self) -> IfccadGeometryAssessment {
        self.stored
            .insert(self.owner.domain(), self.state.assessment);
        IfccadGeometryAssessment {
            domains: self
                .stored
                .into_iter()
                .map(|(domain, evidence)| IfccadGeometryDomainAssessment { domain, evidence })
                .collect(),
        }
    }
}
