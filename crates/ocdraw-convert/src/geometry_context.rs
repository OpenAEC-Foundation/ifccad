//! Owner/domain selection around the common geometry proof state.
use crate::{geometry_assessment::Assessment, *};
use cad_geometry_convert::{
    plot_units::{resolve_paper_tolerance, GeometryCoordinateMeaning, PaperMapping},
    ExchangeState,
};
use ocdraw::geometry_kernel::CoordinateLengthUnit;
use std::collections::BTreeMap;
pub(crate) struct GeometryContext<K> {
    pub state: ExchangeState<K, OcdrawGeometryEntitySource>,
    current: OcdrawGeometryDomain,
    owners: BTreeMap<K, OcdrawGeometryDomain>,
    meanings: BTreeMap<OcdrawGeometryDomain, GeometryCoordinateMeaning>,
    stored: BTreeMap<OcdrawGeometryDomain, Assessment>,
    tolerance: OcdrawGeometryTolerance,
}
impl<K: Copy + Ord> GeometryContext<K> {
    pub fn new(
        tolerance: OcdrawGeometryTolerance,
        unit: CoordinateLengthUnit,
    ) -> Result<Self, OcdrawToleranceError> {
        Ok(Self {
            state: ExchangeState::new(Assessment::new(tolerance, unit)?),
            current: OcdrawGeometryDomain::Drawing,
            owners: BTreeMap::new(),
            stored: BTreeMap::new(),
            meanings: BTreeMap::from([(
                OcdrawGeometryDomain::Drawing,
                GeometryCoordinateMeaning::DrawingUnit(unit),
            )]),
            tolerance,
        })
    }
    pub fn add_paper(
        &mut self,
        owner: K,
        id: u32,
        mapping: PaperMapping,
    ) -> Result<(), OcdrawPaperToleranceError> {
        let domain = OcdrawGeometryDomain::PaperLayout(id);
        let limit = resolve_paper_tolerance(self.tolerance, &mapping).map_err(|reason| {
            OcdrawPaperToleranceError {
                layout_id: id,
                reason,
            }
        })?;
        self.owners.insert(owner, domain);
        self.stored.insert(
            domain,
            Assessment::with_resolved_limit(self.tolerance, CoordinateLengthUnit::Unitless, limit),
        );
        self.meanings.insert(
            domain,
            GeometryCoordinateMeaning::PaperCoordinates { mapping },
        );
        Ok(())
    }
    pub fn refine_paper(
        &mut self,
        owner: K,
        target: PaperMapping,
    ) -> Result<(), OcdrawPaperToleranceError> {
        let domain = self.owners[&owner];
        let GeometryCoordinateMeaning::PaperCoordinates { mapping: source } =
            &self.meanings[&domain]
        else {
            unreachable!()
        };
        let OcdrawGeometryDomain::PaperLayout(id) = domain else {
            unreachable!()
        };
        let a = resolve_paper_tolerance(self.tolerance, source).map_err(|reason| {
            OcdrawPaperToleranceError {
                layout_id: id,
                reason,
            }
        })?;
        let b = resolve_paper_tolerance(self.tolerance, &target).map_err(|reason| {
            OcdrawPaperToleranceError {
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
    pub fn select(&mut self, owner: K) {
        let next = self
            .owners
            .get(&owner)
            .copied()
            .unwrap_or(OcdrawGeometryDomain::Drawing);
        self.select_domain(next);
    }
    pub fn select_drawing(&mut self) {
        self.select_domain(OcdrawGeometryDomain::Drawing);
    }
    fn select_domain(&mut self, next: OcdrawGeometryDomain) {
        if self.current != next {
            let new = self
                .stored
                .remove(&next)
                .expect("registered coordinate domain");
            let old = std::mem::replace(&mut self.state.assessment, new);
            self.stored.insert(self.current, old);
            self.current = next;
        }
    }
    pub fn failure(
        &self,
        failure: Box<cad_geometry_convert::GeometryFailure<OcdrawGeometryEntitySource>>,
    ) -> Box<OcdrawGeometryFailure> {
        Box::new(OcdrawGeometryFailure {
            domain: self.current,
            coordinate_meaning: self.meanings[&self.current].clone(),
            failure,
        })
    }
    pub fn construction_error(
        &self,
        error: cad_geometry_convert::CadGeometryError<OcdrawGeometryEntitySource>,
    ) -> OcdrawToCadError {
        match error {
            cad_geometry_convert::CadGeometryError::Cad(s) => OcdrawToCadError::Cad(s),
            cad_geometry_convert::CadGeometryError::Geometry(f) => {
                OcdrawToCadError::Geometry(self.failure(f))
            }
        }
    }
    pub fn record_geometry(
        &mut self,
        key: K,
        source: OcdrawGeometryEntitySource,
        pair: cad_geometry_convert::GeometryPair,
    ) -> Result<f64, Box<OcdrawGeometryFailure>> {
        self.state
            .record_geometry(key, source, pair)
            .map_err(|f| self.failure(f))
    }
    pub fn assess_occurrences(
        &mut self,
        members: &BTreeMap<K, Vec<K>>,
    ) -> Result<Vec<(K, f64)>, Box<OcdrawGeometryFailure>> {
        let mut results = Vec::new();
        for (owner, roots) in members {
            self.select(*owner);
            let mut evidence = self.state.assessment.clone();
            let records = self
                .state
                .assess_roots(members, roots, &mut evidence)
                .map_err(|f| self.failure(f))?;
            self.state.assessment = evidence;
            results.extend(
                records
                    .into_iter()
                    .filter(|r| r.deviation.upper() > 0.)
                    .map(|r| (r.root, r.deviation.upper())),
            );
        }
        Ok(results)
    }
    pub fn record_unassessed(&mut self, doc: &ocdraw::ocdraw::OcdrawDocument) {
        let mut sources = Vec::new();
        crate::preservation::record_unassessed_occurrences(doc, &mut sources);
        for source in sources {
            let root = match &source {
                OcdrawGeometryEntitySource::BlockOccurrence { path, .. } => {
                    path.first().unwrap_or(&source)
                }
                _ => &source,
            };
            let domain = match root {
                OcdrawGeometryEntitySource::DrawingEntity { scope_id, .. } => doc
                    .layouts
                    .iter()
                    .find(|l| {
                        l.scope_id == *scope_id
                            && l.kind == ocdraw::ocdraw::DrawingLayoutKind::Paper
                    })
                    .map(|l| OcdrawGeometryDomain::PaperLayout(l.id))
                    .unwrap_or(OcdrawGeometryDomain::Drawing),
                _ => OcdrawGeometryDomain::Drawing,
            };
            self.select_domain(domain);
            self.state.assessment.record_unassessed(source);
        }
    }
    pub fn finish(mut self) -> OcdrawGeometryAssessment {
        self.stored.insert(self.current, self.state.assessment);
        let unassessed = self
            .stored
            .values()
            .flat_map(|a| a.unassessed_sources().iter().cloned())
            .collect();
        OcdrawGeometryAssessment {
            unassessed,
            domains: self
                .stored
                .into_iter()
                .map(|(domain, evidence)| OcdrawGeometryDomainAssessment {
                    domain,
                    meaning: self.meanings.remove(&domain).unwrap(),
                    evidence,
                })
                .collect(),
        }
    }
}
impl<K> std::ops::Deref for GeometryContext<K> {
    type Target = ExchangeState<K, OcdrawGeometryEntitySource>;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl<K> std::ops::DerefMut for GeometryContext<K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
