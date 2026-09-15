mod appearance;
mod conversion;
mod coverage;
mod diagnostic;
mod entities;
mod entity_mapping;
mod layers;
mod options;
mod outcome;
mod structure;
mod units;

pub use conversion::cad_document_to_package;
pub use diagnostic::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason,
    SourceStructureProblem,
};
pub use entity_mapping::ExportEntityMapping;
pub use options::{ExportLossPolicy, ExportOptions};
pub use outcome::{ExportOutcome, ExportOutcomeParts};

use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ExportError {
    #[error(transparent)]
    InvalidGeometryTolerance(#[from] crate::ConversionToleranceError),
    #[error("geometry exceeds the requested tolerance: {failure:?}")]
    GeometryToleranceExceeded {
        failure: Box<crate::ConversionGeometryFailure>,
    },
    #[error("geometry accuracy could not be established: {failure:?}")]
    GeometryAccuracyNotEstablished {
        failure: Box<crate::ConversionGeometryFailure>,
    },

    #[error("CAD source structure is invalid")]
    InvalidSourceStructure {
        problems: Vec<SourceStructureProblem>,
    },
    #[error("export loss was rejected")]
    LossRejected { diagnostics: Vec<ExportDiagnostic> },
    #[error(transparent)]
    PackageBuild(#[from] ifccad::package::PackageBuildError),
    #[error("internal conversion invariant failed: {message}")]
    InternalInvariant { message: String },
}

impl From<Box<crate::ConversionGeometryFailure>> for ExportError {
    fn from(failure: Box<crate::ConversionGeometryFailure>) -> Self {
        if failure.reason == crate::ConversionGeometryFailureReason::ProvenExceedance {
            Self::GeometryToleranceExceeded { failure }
        } else {
            Self::GeometryAccuracyNotEstablished { failure }
        }
    }
}
