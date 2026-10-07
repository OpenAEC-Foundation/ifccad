//! Shared CAD geometric preparation and hard accuracy assessment.
mod assessment;
pub mod geometry;
pub mod plot_units;
mod tolerance;
pub mod units;
pub use assessment::*;
pub use tolerance::{GeometryTolerance, GeometryToleranceError};

/// Caller-owned identities; the numeric engine never narrows native IDs.
pub trait GeometrySource: Clone {
    fn cad_entity(handle: opencadcodec::Handle, kind: String) -> Self;
    fn occurrence(path: Vec<Self>, leaf: Self) -> Self;
}
#[derive(Debug, thiserror::Error)]
pub enum CadConstructionError {
    #[error("CAD geometry construction failed: {0}")]
    Cad(String),
}
#[derive(Debug, thiserror::Error)]
pub enum CadGeometryError<S> {
    #[error("CAD geometry construction failed: {0}")]
    Cad(String),
    #[error("geometric accuracy failed: {0:?}")]
    Geometry(Box<GeometryFailure<S>>),
}
impl<S> From<Box<GeometryFailure<S>>> for CadGeometryError<S> {
    fn from(value: Box<GeometryFailure<S>>) -> Self {
        Self::Geometry(value)
    }
}
impl<S> From<CadConstructionError> for CadGeometryError<S> {
    fn from(value: CadConstructionError) -> Self {
        match value {
            CadConstructionError::Cad(message) => Self::Cad(message),
        }
    }
}

pub mod exchange;
pub use exchange::ExchangeState;
mod prepare;
pub use prepare::*;
