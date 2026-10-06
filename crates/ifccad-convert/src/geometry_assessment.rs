//! Model-specific identities and coordinate-domain evidence over the shared proof engine.
pub use cad_geometry_convert::{
    DistanceInterval as IfccadDistanceInterval,
    GeometryFailureReason as IfccadGeometryFailureReason, GeometryStage as IfccadGeometryStage,
    GeometryStatus as IfccadGeometryStatus, GeometryTolerance as IfccadGeometryTolerance,
    GeometryToleranceError as IfccadToleranceError,
};
use ocdraw::geometry_kernel::CoordinateLengthUnit;
use opencadcodec::Handle;

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
}
impl IfccadGeometryDomainAssessment {
    pub fn domain(&self) -> IfccadGeometryDomain {
        self.domain
    }
    pub fn coordinate_unit(&self) -> CoordinateLengthUnit {
        self.evidence.drawing_unit()
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
}
impl IfccadGeometryAssessment {
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
#[error("geometric accuracy failed in {domain:?}, coordinate unit {unit:?}: {failure:?}")]
pub struct IfccadGeometryFailure {
    pub domain: IfccadGeometryDomain,
    pub unit: CoordinateLengthUnit,
    pub failure: Box<cad_geometry_convert::GeometryFailure<IfccadGeometryEntitySource>>,
}
impl std::ops::Deref for IfccadGeometryFailure {
    type Target = cad_geometry_convert::GeometryFailure<IfccadGeometryEntitySource>;
    fn deref(&self) -> &Self::Target {
        &self.failure
    }
}
