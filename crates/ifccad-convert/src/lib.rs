#![doc = include_str!("../README.md")]

mod assessment;
mod export;
mod import;
pub use assessment::{TransferAssessment, TransferConclusion, TransferCoverage, TransferScope};

pub use cadcodec;
/// CadDocument-to-IFCCAD export API, including loss policy, diagnostics, and
/// source-handle-to-IFCDR-ID mappings.
pub use export::{
    cad_document_to_package, ExportAction, ExportDiagnostic, ExportDiagnosticSource,
    ExportEntityMapping, ExportError, ExportLossPolicy, ExportLossReason, ExportOptions,
    ExportOutcome, ExportOutcomeParts, SourceStructureProblem,
};
/// Validated-IFCCAD-to-CadDocument import API, including diagnostics and
/// source-ID-to-target-handle mappings.
pub use import::{
    drawing_to_cad_document, drawing_to_cad_document_with_options, ImportDiagnostic,
    ImportEntityMapping, ImportError, ImportOutcome, ImportOutcomeParts,
};

mod options;
mod units;
pub use options::{
    ConversionGeometryTolerance, ConversionLossPolicy, ConversionToleranceError, ImportOptions,
};

mod geometry;
mod geometry_assessment;
pub use geometry_assessment::*;
