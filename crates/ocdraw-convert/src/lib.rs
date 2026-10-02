#![doc = include_str!("../README.md")]

mod ocdraw;
pub use ocdraw::{
    cad_document_to_drawing, cad_document_to_drawing_with_id, cad_document_to_ocdraw_document,
    cad_document_to_ocdraw_document_with_id, ocdraw_document_to_cad_document,
    ocdraw_to_cad_document, DirectExportError, DirectExportOutcome, DirectImportDiagnostic,
    DirectImportError, DirectImportOutcome, OcdrawDocumentExportOutcome,
};

pub use cadcodec;
mod source;
pub use options::{ExportLossPolicy, ExportOptions};
pub use source::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason,
    SourceStructureProblem,
};

mod options;
mod units;
pub use options::{
    ConversionGeometryTolerance, ConversionLossPolicy, ConversionToleranceError, ImportOptions,
};

mod geometry;
mod geometry_assessment;
mod point_display;
pub use geometry_assessment::*;
