//! IFCCAD owner/domain adapter over shared primitive and occurrence proofs.
//! Model/definitions use drawing units; each Paper domain uses fixed output
//! mapping or unknown coordinate meaning. Conversion state is not drawing data.
use crate::{geometry_assessment::Assessment, *};
use ifccad::geometry_kernel::CoordinateLengthUnit;
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GeometryNodeKey {
    Entity(u64),
    BlockDefinition(u64),
    DimensionGraphicsGroup(u64),
    DimensionGraphicsPart { dimension: u64, index: usize },
    DimensionGraphicsOccurrence(u64),
}
pub(crate) struct GeometryContext {
    pub state: cad_geometry_convert::ExchangeState<GeometryNodeKey, IfccadGeometryEntitySource>,
    pub owner: IfccadGeometryOwner,
    pub preparation_error: Option<cad_geometry_convert::CadPreparationError>,
    pub pending_hatch_recipes: Vec<crate::to_cad::hatch_recipes::PendingHatchRecipe>,
    stored: BTreeMap<IfccadGeometryDomain, Assessment>,
    pub graphics_members: BTreeMap<GeometryNodeKey, Vec<GeometryNodeKey>>,
    pub graphics_owners: BTreeMap<u64, IfccadGeometryOwner>,
    pub buffered_pairs: Option<Vec<cad_geometry_convert::GeometryPair>>,
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
            preparation_error: None,
            pending_hatch_recipes: Vec::new(),
            stored: BTreeMap::new(),
            graphics_members: BTreeMap::new(),
            graphics_owners: BTreeMap::new(),
            buffered_pairs: None,
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
        &mut self,
        source: &IfccadGeometryEntitySource,
        error: cad_geometry_convert::CadPreparationError,
        stage: IfccadGeometryStage,
    ) -> IfccadConversionError {
        self.preparation_error = Some(error);
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
        if let Some(buffer) = &mut self.buffered_pairs {
            buffer.push(pair);
            return Ok(());
        }
        self.record_key(GeometryNodeKey::Entity(key), source, pair, issues)
    }
    pub fn record_key(
        &mut self,
        key: GeometryNodeKey,
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
        self.assess_roots_impl(members, roots, issues, true)
    }
    pub fn trial_roots(
        &mut self,
        members: &BTreeMap<u64, Vec<u64>>,
        roots: &[u64],
    ) -> Result<(), IfccadConversionError> {
        self.assess_roots_impl(members, roots, &mut Vec::new(), false)
    }
    fn assess_roots_impl(
        &mut self,
        members: &BTreeMap<u64, Vec<u64>>,
        roots: &[u64],
        issues: &mut Vec<IfccadDiagnostic>,
        commit: bool,
    ) -> Result<(), IfccadConversionError> {
        let mut members = self.tag_members(members);
        members.extend(self.graphics_members.clone());
        let roots = roots
            .iter()
            .map(|id| {
                if self.graphics_owners.contains_key(id) {
                    GeometryNodeKey::DimensionGraphicsOccurrence(*id)
                } else {
                    GeometryNodeKey::Entity(*id)
                }
            })
            .collect::<Vec<_>>();
        let mut assessment = self.state.assessment.clone();
        let records = self
            .state
            .assess_roots(&members, &roots, &mut assessment)
            .map_err(|e| self.numerical_failure(e))?;
        if !commit {
            return Ok(());
        }
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
    pub fn install_graphics_group(
        &mut self,
        id: u64,
        source: IfccadGeometryEntitySource,
        parts: Vec<GeometryNodeKey>,
        source_map: cad_geometry_convert::geometry::blocks::EvaluatedBlock,
        target_map: cad_geometry_convert::geometry::blocks::EvaluatedBlock,
    ) {
        let root = GeometryNodeKey::DimensionGraphicsOccurrence(id);
        let group = GeometryNodeKey::DimensionGraphicsGroup(id);
        self.state.register_identity(root, source);
        self.state
            .record_instance_parts(root, group, source_map, target_map);
        self.graphics_members.insert(group, parts);
        self.graphics_owners.insert(id, self.owner);
    }
    pub fn register_identity(&mut self, key: u64, source: IfccadGeometryEntitySource) {
        self.state
            .register_identity(GeometryNodeKey::Entity(key), source);
    }
    pub fn record_instance_parts(
        &mut self,
        key: u64,
        definition: u64,
        source: cad_geometry_convert::geometry::blocks::EvaluatedBlock,
        target: cad_geometry_convert::geometry::blocks::EvaluatedBlock,
    ) {
        self.state.record_instance_parts(
            GeometryNodeKey::Entity(key),
            GeometryNodeKey::BlockDefinition(definition),
            source,
            target,
        );
    }
    pub fn discard_entity(&mut self, id: u64) {
        self.state.discard_entity(GeometryNodeKey::Entity(id));
        if let Some(parts) = self
            .graphics_members
            .remove(&GeometryNodeKey::DimensionGraphicsGroup(id))
        {
            for key in parts {
                self.state.discard_entity(key);
            }
        }
        self.state
            .discard_entity(GeometryNodeKey::DimensionGraphicsOccurrence(id));
        self.graphics_owners.remove(&id);
    }
    fn tag_members(
        &self,
        members: &BTreeMap<u64, Vec<u64>>,
    ) -> BTreeMap<GeometryNodeKey, Vec<GeometryNodeKey>> {
        members
            .iter()
            .map(|(id, list)| {
                (
                    GeometryNodeKey::BlockDefinition(*id),
                    list.iter()
                        .flat_map(|e| {
                            std::iter::once(GeometryNodeKey::Entity(*e)).chain(
                                self.graphics_owners
                                    .contains_key(e)
                                    .then_some(GeometryNodeKey::DimensionGraphicsOccurrence(*e)),
                            )
                        })
                        .collect(),
                )
            })
            .collect()
    }
    pub fn affected_instances(
        &self,
        members: &BTreeMap<u64, Vec<u64>>,
        affected: &std::collections::BTreeSet<u64>,
    ) -> BTreeMap<u64, Vec<u64>> {
        self.state
            .affected_instances(
                &self.tag_members(members),
                &affected
                    .iter()
                    .map(|id| GeometryNodeKey::BlockDefinition(*id))
                    .collect(),
            )
            .into_iter()
            .filter_map(|(key, roots)| {
                let GeometryNodeKey::BlockDefinition(id) = key else {
                    return None;
                };
                Some((
                    id,
                    roots
                        .into_iter()
                        .filter_map(|r| match r {
                            GeometryNodeKey::Entity(id)
                            | GeometryNodeKey::DimensionGraphicsOccurrence(id) => Some(id),
                            _ => None,
                        })
                        .collect(),
                ))
            })
            .collect()
    }
    pub fn finish(mut self) -> IfccadGeometryAssessment {
        self.stored
            .insert(self.owner.domain(), self.state.assessment);
        IfccadGeometryAssessment {
            filled_surfaces: vec![],
            unassessed: vec![],
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

#[cfg(test)]
mod graphics_identity_tests {
    use super::*;
    #[test]
    fn part_indices_do_not_overwrite_definition_entities_or_markers() {
        let mut c = GeometryContext::new(IfccadGeometryTolerance::exact(), "unitless", 1).unwrap();
        let entity = IfccadGeometryEntitySource::NativeEntity {
            owner: c.owner,
            entity_id: 1,
        };
        c.record(
            1,
            entity.clone(),
            cad_geometry_convert::GeometryPair::points(&[[0.; 3]], &[[0.; 3]]).unwrap(),
            &mut vec![],
        )
        .unwrap();
        for index in [0, 1] {
            let source = IfccadGeometryEntitySource::DimensionGraphicsPart {
                owner: c.owner,
                dimension: Box::new(entity.clone()),
                part_index: index,
                source: Box::new(IfccadGeometryEntitySource::CadEntity {
                    handle: opencadcodec::Handle::new(1),
                    kind: "POINT".into(),
                }),
            };
            c.record_key(
                GeometryNodeKey::DimensionGraphicsPart {
                    dimension: 1,
                    index,
                },
                source.clone(),
                cad_geometry_convert::GeometryPair::points(&[[2.; 3]], &[[2.; 3]]).unwrap(),
                &mut vec![],
            )
            .unwrap();
            assert_eq!(
                c.state.identity(GeometryNodeKey::DimensionGraphicsPart {
                    dimension: 1,
                    index
                }),
                &source
            );
        }
        assert_eq!(c.state.identity(GeometryNodeKey::Entity(1)), &entity);
        assert_eq!(c.state.assessment.assessed_entities(), 3);
    }
}

#[cfg(test)]
mod graphics_occurrence_tests {
    use super::*;
    use crate::dimension_graphics_context::PictureMap;
    use ifccad::*;
    #[test]
    fn picture_coordinates_have_independent_hard_proof_after_owner_placement() {
        for (x, passes) in [(2., true), (3., false)] {
            let mut c =
                GeometryContext::new(IfccadGeometryTolerance::exact(), "unitless", 1).unwrap();
            let dimension = IfccadGeometryEntitySource::NativeEntity {
                owner: c.owner,
                entity_id: 1,
            };
            let source = IfccadGeometryEntitySource::DimensionGraphicsPart {
                owner: c.owner,
                dimension: Box::new(dimension.clone()),
                part_index: 0,
                source: Box::new(IfccadGeometryEntitySource::CadEntity {
                    handle: opencadcodec::Handle::new(1),
                    kind: "LINE".into(),
                }),
            };
            let key = GeometryNodeKey::DimensionGraphicsPart {
                dimension: 1,
                index: 0,
            };
            c.state.register_geometry(
                key,
                source,
                cad_geometry_convert::GeometryPair::points(&[[100., 9., 0.]], &[[x, 9., 0.]])
                    .unwrap(),
            );
            let p = IfccadPlacement {
                origin: [98., 0., 0.],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            };
            c.install_graphics_group(
                1,
                dimension,
                vec![key],
                PictureMap::identity().evaluated(),
                PictureMap::native(p.coordinate_frame().unwrap()).evaluated(),
            );
            let result = c.assess_roots(&BTreeMap::new(), &[1], &mut vec![]);
            assert_eq!(result.is_ok(), passes);
            if let Err(IfccadConversionError::Geometry(e)) = result {
                assert!(matches!(
                    e.source,
                    IfccadGeometryEntitySource::BlockOccurrence { .. }
                ));
            }
        }
    }
}
