//! IFCCAD owner/domain adapter over shared primitive and occurrence proofs.
//! Model/definitions use drawing units; each Paper domain uses fixed output
//! mapping or unknown coordinate meaning. Conversion state is not drawing data.
use crate::{geometry_assessment::Assessment, *};
use ocdraw::geometry_kernel::CoordinateLengthUnit;
use std::collections::BTreeMap;
pub(crate) struct GeometryContext {
    pub state: cad_geometry_convert::ExchangeState<u64, IfccadGeometryEntitySource>,
    pub owner: IfccadGeometryOwner,
    stored: BTreeMap<IfccadGeometryDomain, Assessment>,
    tolerance: IfccadGeometryTolerance,
    meanings:
        BTreeMap<IfccadGeometryDomain, cad_geometry_convert::plot_units::GeometryCoordinateMeaning>,
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
            meanings: BTreeMap::from([(
                IfccadGeometryDomain::Drawing,
                cad_geometry_convert::plot_units::GeometryCoordinateMeaning::DrawingUnit(unit),
            )]),
        })
    }
    pub fn add_paper(
        &mut self,
        id: u64,
        mapping: cad_geometry_convert::plot_units::PaperMapping,
    ) -> Result<(), IfccadConversionError> {
        let limit =
            cad_geometry_convert::plot_units::resolve_paper_tolerance(self.tolerance, &mapping)
                .map_err(|reason| IfccadConversionError::PaperTolerance {
                    layout_id: id,
                    reason,
                })?;
        let assessment =
            Assessment::with_resolved_limit(self.tolerance, CoordinateLengthUnit::Unitless, limit);
        self.stored
            .insert(IfccadGeometryDomain::PaperLayout(id), assessment);
        self.meanings.insert(
            IfccadGeometryDomain::PaperLayout(id),
            cad_geometry_convert::plot_units::GeometryCoordinateMeaning::PaperCoordinates {
                mapping,
            },
        );
        Ok(())
    }
    pub fn refine_paper(
        &mut self,
        id: u64,
        target: cad_geometry_convert::plot_units::PaperMapping,
    ) -> Result<(), IfccadConversionError> {
        use cad_geometry_convert::plot_units::{
            resolve_paper_tolerance, GeometryCoordinateMeaning,
        };
        let domain = IfccadGeometryDomain::PaperLayout(id);
        let GeometryCoordinateMeaning::PaperCoordinates { mapping: source } =
            &self.meanings[&domain]
        else {
            unreachable!()
        };
        let a = resolve_paper_tolerance(self.tolerance, source).map_err(|reason| {
            IfccadConversionError::PaperTolerance {
                layout_id: id,
                reason,
            }
        })?;
        let b = resolve_paper_tolerance(self.tolerance, &target).map_err(|reason| {
            IfccadConversionError::PaperTolerance {
                layout_id: id,
                reason,
            }
        })?;
        let limit = cad_geometry_convert::units::ResolvedTolerance {
            lower: a.lower.min(b.lower),
            upper: a.upper.min(b.upper),
        };
        self.stored.insert(
            domain,
            Assessment::with_resolved_limit(self.tolerance, CoordinateLengthUnit::Unitless, limit),
        );
        self.meanings.insert(
            domain,
            GeometryCoordinateMeaning::PaperCoordinates { mapping: target },
        );
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
            coordinate_meaning: self.meanings[&self.owner.domain()].clone(),
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
                .map(|(domain, evidence)| IfccadGeometryDomainAssessment {
                    meaning: self.meanings.remove(&domain).unwrap(),
                    domain,
                    evidence,
                })
                .collect(),
        }
    }
}
