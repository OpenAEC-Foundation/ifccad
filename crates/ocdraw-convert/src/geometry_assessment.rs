pub use cad_geometry_convert::{
    DistanceInterval as OcdrawDistanceInterval,
    GeometryFailureReason as OcdrawGeometryFailureReason, GeometryStage as OcdrawGeometryStage,
    GeometryStatus as OcdrawGeometryStatus,
};
use opencadcodec::Handle;
pub type OcdrawGeometryAssessment =
    cad_geometry_convert::GeometryAssessment<OcdrawGeometryEntitySource>;
pub type OcdrawGeometryFailure = cad_geometry_convert::GeometryFailure<OcdrawGeometryEntitySource>;
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
impl From<cad_geometry_convert::CadGeometryError<OcdrawGeometryEntitySource>>
    for crate::OcdrawToCadError
{
    fn from(error: cad_geometry_convert::CadGeometryError<OcdrawGeometryEntitySource>) -> Self {
        match error {
            cad_geometry_convert::CadGeometryError::Cad(message) => Self::Cad(message),
            cad_geometry_convert::CadGeometryError::Geometry(error) => Self::Geometry(error),
        }
    }
}
