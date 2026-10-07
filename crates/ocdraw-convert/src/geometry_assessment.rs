pub use cad_geometry_convert::plot_units::{
    GeometryCoordinateMeaning as OcdrawGeometryCoordinateMeaning,
    PaperMapping as OcdrawPaperMapping,
};
pub use cad_geometry_convert::{
    DistanceInterval as OcdrawDistanceInterval,
    GeometryFailureReason as OcdrawGeometryFailureReason, GeometryStage as OcdrawGeometryStage,
    GeometryStatus as OcdrawGeometryStatus,
};
use opencadcodec::Handle;
use OcdrawGeometryCoordinateMeaning as GeometryCoordinateMeaning;
pub(crate) type Assessment = cad_geometry_convert::GeometryAssessment<OcdrawGeometryEntitySource>;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OcdrawGeometryDomain {
    Drawing,
    PaperLayout(u32),
}
#[derive(Clone, Debug, PartialEq)]
pub struct OcdrawGeometryDomainAssessment {
    pub(crate) domain: OcdrawGeometryDomain,
    pub(crate) meaning: GeometryCoordinateMeaning,
    pub(crate) evidence: Assessment,
}
impl OcdrawGeometryDomainAssessment {
    pub fn domain(&self) -> OcdrawGeometryDomain {
        self.domain
    }
    pub fn coordinate_meaning(&self) -> &GeometryCoordinateMeaning {
        &self.meaning
    }
    pub fn resolved_tolerance(&self) -> OcdrawDistanceInterval {
        self.evidence.resolved_tolerance()
    }
    pub fn requested_tolerance(&self) -> crate::OcdrawGeometryTolerance {
        self.evidence.requested_tolerance()
    }
    pub fn status(&self) -> OcdrawGeometryStatus {
        self.evidence.status()
    }
    pub fn max_deviation_upper_bound(&self) -> f64 {
        self.evidence.max_deviation_upper_bound()
    }
    pub fn worst_entity(&self) -> Option<&OcdrawGeometryEntitySource> {
        self.evidence.worst_entity()
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
}
#[derive(Clone, Debug, PartialEq)]
pub struct OcdrawGeometryAssessment {
    pub(crate) domains: Vec<OcdrawGeometryDomainAssessment>,
    pub(crate) unassessed: Vec<OcdrawGeometryEntitySource>,
}
impl OcdrawGeometryAssessment {
    pub fn domains(&self) -> &[OcdrawGeometryDomainAssessment] {
        &self.domains
    }
    pub fn is_complete(&self) -> bool {
        self.unassessed.is_empty()
    }
    pub fn unassessed_sources(&self) -> &[OcdrawGeometryEntitySource] {
        &self.unassessed
    }
    pub fn status(&self) -> OcdrawGeometryStatus {
        if self
            .domains
            .iter()
            .any(|d| d.status() == OcdrawGeometryStatus::RoundedWithinTolerance)
        {
            OcdrawGeometryStatus::RoundedWithinTolerance
        } else if self
            .domains
            .iter()
            .any(|d| d.status() == OcdrawGeometryStatus::Exact)
        {
            OcdrawGeometryStatus::Exact
        } else {
            OcdrawGeometryStatus::Empty
        }
    }
    pub fn assessed_entities(&self) -> usize {
        self.domains.iter().map(|d| d.assessed_entities()).sum()
    }
    pub fn assessed_vertices(&self) -> usize {
        self.domains.iter().map(|d| d.assessed_vertices()).sum()
    }
    pub fn rounded_entities(&self) -> usize {
        self.domains.iter().map(|d| d.rounded_entities()).sum()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct OcdrawGeometryFailure {
    pub domain: OcdrawGeometryDomain,
    pub coordinate_meaning: GeometryCoordinateMeaning,
    pub failure: Box<cad_geometry_convert::GeometryFailure<OcdrawGeometryEntitySource>>,
}
impl std::ops::Deref for OcdrawGeometryFailure {
    type Target = cad_geometry_convert::GeometryFailure<OcdrawGeometryEntitySource>;
    fn deref(&self) -> &Self::Target {
        &self.failure
    }
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

impl cad_geometry_convert::GeometrySource for OcdrawGeometryEntitySource {
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
impl From<cad_geometry_convert::CadConstructionError> for crate::OcdrawToCadError {
    fn from(error: cad_geometry_convert::CadConstructionError) -> Self {
        match error {
            cad_geometry_convert::CadConstructionError::Cad(message) => Self::Cad(message),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("Paper layout {layout_id} tolerance cannot be resolved: {reason}")]
pub struct OcdrawPaperToleranceError {
    pub layout_id: u32,
    #[source]
    pub reason: crate::OcdrawToleranceError,
}
