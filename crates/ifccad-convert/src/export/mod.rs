mod conversion;
mod diagnostic;
mod entity_mapping;
mod options;
mod outcome;

pub use conversion::cad_document_to_package;
pub use diagnostic::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason,
    SourceStructureProblem,
};
pub use entity_mapping::ExportEntityMapping;
pub use options::{ExportLossPolicy, ExportOptions};
pub use outcome::ExportOutcome;

use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ExportError {
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
