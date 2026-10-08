//! Model-specific identities and coordinate-domain evidence over the shared proof engine.
pub use cad_geometry_convert::plot_units::{
    GeometryCoordinateMeaning as IfccadGeometryCoordinateMeaning,
    PaperMapping as IfccadPaperMapping,
};
pub use cad_geometry_convert::{
    DistanceInterval as IfccadDistanceInterval,
    GeometryFailureReason as IfccadGeometryFailureReason, GeometryStage as IfccadGeometryStage,
    GeometryStatus as IfccadGeometryStatus, GeometryTolerance as IfccadGeometryTolerance,
    GeometryToleranceError as IfccadToleranceError,
};
use opencadcodec::Handle;
use IfccadGeometryCoordinateMeaning as GeometryCoordinateMeaning;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IfccadGeometryDomain {
    Drawing,
    PaperLayout(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadGeometryOwner {
    ModelLayout(u64),
    PaperLayout(u64),
    BlockDefinition(u64),
}
impl IfccadGeometryOwner {
    pub(crate) fn domain(self) -> IfccadGeometryDomain {
        match self {
            Self::PaperLayout(id) => IfccadGeometryDomain::PaperLayout(id),
            _ => IfccadGeometryDomain::Drawing,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IfccadGeometryEntitySource {
    CadEntity {
        handle: Handle,
        kind: String,
    },
    NativeEntity {
        owner: IfccadGeometryOwner,
        entity_id: u64,
    },
    BlockOccurrence {
        path: Vec<IfccadGeometryEntitySource>,
        leaf: Box<IfccadGeometryEntitySource>,
    },
}
impl cad_geometry_convert::GeometrySource for IfccadGeometryEntitySource {
    fn cad_entity(handle: Handle, kind: String) -> Self {
        Self::CadEntity { handle, kind }
    }
    fn occurrence(path: Vec<Self>, leaf: Self) -> Self {
        Self::BlockOccurrence {
            path,
            leaf: Box::new(leaf),
        }
    }
}
pub(crate) type Assessment = cad_geometry_convert::GeometryAssessment<IfccadGeometryEntitySource>;
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadGeometryDomainAssessment {
    pub(crate) domain: IfccadGeometryDomain,
    pub(crate) evidence: Assessment,
    pub(crate) meaning: GeometryCoordinateMeaning,
}
impl IfccadGeometryDomainAssessment {
    pub fn domain(&self) -> IfccadGeometryDomain {
        self.domain
    }
    pub fn coordinate_meaning(&self) -> &GeometryCoordinateMeaning {
        &self.meaning
    }
    pub fn resolved_tolerance(&self) -> IfccadDistanceInterval {
        self.evidence.resolved_tolerance()
    }
    pub fn status(&self) -> IfccadGeometryStatus {
        self.evidence.status()
    }
    pub fn assessed_entities(&self) -> usize {
        self.evidence.assessed_entities()
    }
    pub fn assessed_vertices(&self) -> usize {
        self.evidence.assessed_vertices()
    }
    pub fn rounded_entities(&self) -> usize {
        self.evidence.rounded_entities()
    }
    pub fn max_deviation_upper_bound(&self) -> f64 {
        self.evidence.max_deviation_upper_bound()
    }
    pub fn worst_entity(&self) -> Option<&IfccadGeometryEntitySource> {
        self.evidence.worst_entity()
    }
    pub fn requested_tolerance(&self) -> IfccadGeometryTolerance {
        self.evidence.requested_tolerance()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadGeometryAssessment {
    pub(crate) domains: Vec<IfccadGeometryDomainAssessment>,
    pub(crate) unassessed: Vec<IfccadGeometryEntitySource>,
}
impl IfccadGeometryAssessment {
    /// Accuracy evidence covers native subsets; opaque geometry stays explicitly unassessed.
    pub fn is_complete(&self) -> bool {
        self.unassessed.is_empty()
    }
    pub fn unassessed_entities(&self) -> &[IfccadGeometryEntitySource] {
        &self.unassessed
    }
    pub(crate) fn with_unassessed(mut self, d: &ocdraw::ifccad::IfccadDocument) -> Self {
        use ocdraw::ifccad::*;
        fn walk(
            d: &IfccadDocument,
            owner: IfccadGeometryOwner,
            contents: &[IfccadEntity],
            path: &mut Vec<IfccadGeometryEntitySource>,
            out: &mut Vec<IfccadGeometryEntitySource>,
        ) {
            for e in contents {
                let identity = IfccadGeometryEntitySource::NativeEntity {
                    owner,
                    entity_id: e.id(),
                };
                if e.as_opaque().is_some()
                    || e.as_native().is_some_and(|e| {
                        matches!(&e.kind, IfccadEntityKind::MText(_))
                            || matches!(&e.kind,IfccadEntityKind::Text(t) if !t.content.is_empty())
                    })
                {
                    out.push(if path.is_empty() {
                        identity
                    } else {
                        IfccadGeometryEntitySource::BlockOccurrence {
                            path: path.clone(),
                            leaf: Box::new(identity),
                        }
                    });
                } else if let Some(IfccadNativeEntity {
                    kind: IfccadEntityKind::BlockInstance { definition_id, .. },
                    ..
                }) = e.as_native()
                {
                    if let Some(b) = d.blocks.iter().find(|b| b.id == *definition_id) {
                        path.push(identity);
                        walk(
                            d,
                            IfccadGeometryOwner::BlockDefinition(b.id),
                            &b.entities,
                            path,
                            out,
                        );
                        path.pop();
                    }
                }
            }
        }
        self.unassessed.clear();
        walk(
            d,
            IfccadGeometryOwner::ModelLayout(d.model.id),
            &d.model.entities,
            &mut vec![],
            &mut self.unassessed,
        );
        for p in &d.paper_layouts {
            walk(
                d,
                IfccadGeometryOwner::PaperLayout(p.id),
                &p.entities,
                &mut vec![],
                &mut self.unassessed,
            );
        }
        for b in &d.blocks {
            walk(
                d,
                IfccadGeometryOwner::BlockDefinition(b.id),
                &b.entities,
                &mut vec![],
                &mut self.unassessed,
            );
        }
        self
    }
    pub fn domains(&self) -> &[IfccadGeometryDomainAssessment] {
        &self.domains
    }
    pub fn status(&self) -> IfccadGeometryStatus {
        if self
            .domains
            .iter()
            .any(|d| d.status() == IfccadGeometryStatus::RoundedWithinTolerance)
        {
            IfccadGeometryStatus::RoundedWithinTolerance
        } else if self
            .domains
            .iter()
            .any(|d| d.status() == IfccadGeometryStatus::Exact)
        {
            IfccadGeometryStatus::Exact
        } else {
            IfccadGeometryStatus::Empty
        }
    }
}
#[derive(Debug, thiserror::Error)]
#[error("geometric accuracy failed in {domain:?}, coordinates {coordinate_meaning:?}: {failure:?}")]
pub struct IfccadGeometryFailure {
    pub domain: IfccadGeometryDomain,
    pub coordinate_meaning: GeometryCoordinateMeaning,
    pub failure: Box<cad_geometry_convert::GeometryFailure<IfccadGeometryEntitySource>>,
}
impl std::ops::Deref for IfccadGeometryFailure {
    type Target = cad_geometry_convert::GeometryFailure<IfccadGeometryEntitySource>;
    fn deref(&self) -> &Self::Target {
        &self.failure
    }
}
