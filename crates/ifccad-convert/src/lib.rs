#![doc = include_str!("../README.md")]

mod export;
mod import;

pub use cadcodec;
/// CadDocument-to-IFCCAD export API, including loss policy, diagnostics, and
/// source-handle-to-IFCDR-ID mappings.
pub use export::{
    cad_document_to_package, ExportAction, ExportDiagnostic, ExportDiagnosticSource,
    ExportEntityMapping, ExportError, ExportLossPolicy, ExportLossReason, ExportOptions,
    ExportOutcome, SourceStructureProblem,
};
/// Validated-IFCCAD-to-CadDocument import API, including diagnostics and
/// source-ID-to-target-handle mappings.
pub use import::{
    drawing_to_cad_document, ImportDiagnostic, ImportEntityMapping, ImportError, ImportOutcome,
};
