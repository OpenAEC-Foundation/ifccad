#![doc = include_str!("../README.md")]

mod export;
mod import;

pub use cadcodec;
pub use export::{
    cad_document_to_package, ExportAction, ExportDiagnostic, ExportDiagnosticSource,
    ExportEntityMapping, ExportError, ExportLossPolicy, ExportLossReason, ExportOptions,
    ExportOutcome, SourceStructureProblem,
};
pub use import::{
    drawing_to_cad_document, ImportDiagnostic, ImportEntityMapping, ImportError, ImportOutcome,
};
