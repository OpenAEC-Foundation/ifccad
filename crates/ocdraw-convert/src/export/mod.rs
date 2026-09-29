mod appearance;
mod blocks;
mod conversion;
mod coverage;
mod diagnostic;
mod entities;
mod entity_mapping;
mod layers;
mod layouts;
mod options;
mod outcome;
mod structure;
mod units;
mod workspace;

pub(crate) use appearance::{direct_entity, direct_layer};
pub(crate) use blocks::ordered_entities;
pub(crate) use blocks::{inspect_markers, inspect_references};
pub(crate) use structure::{inspect_model_space, with_recovered_model_space_handle};

pub(crate) fn direct_line_losses(line: &cadcodec::Line) -> Vec<ExportLossReason> {
    entities::line_losses(line)
}

pub(crate) fn direct_point_losses(point: &cadcodec::Point) -> Vec<ExportLossReason> {
    entities::point_losses(point)
}

pub(crate) fn direct_circle_losses(circle: &cadcodec::Circle) -> Vec<ExportLossReason> {
    entities::circle_losses(circle)
}

pub(crate) fn direct_arc_losses(arc: &cadcodec::Arc) -> Vec<ExportLossReason> {
    entities::arc_losses(arc)
}

pub(crate) fn direct_ellipse_losses(ellipse: &cadcodec::Ellipse) -> Vec<ExportLossReason> {
    entities::ellipse_losses(ellipse)
}

pub(crate) fn direct_planar_polyline_losses(
    polyline: &cadcodec::LwPolyline,
) -> Vec<ExportLossReason> {
    entities::polyline_losses(polyline)
}

pub(crate) fn direct_spatial_polyline_losses(
    polyline: &cadcodec::entities::Polyline3D,
) -> Vec<ExportLossReason> {
    entities::polyline3d_losses(polyline)
}

pub(crate) fn direct_legacy_planar_polyline_losses(
    polyline: &cadcodec::entities::Polyline2D,
) -> Vec<ExportLossReason> {
    entities::polyline2d_losses(polyline)
}

pub(crate) fn direct_generic_spatial_polyline_losses(
    polyline: &cadcodec::entities::Polyline,
) -> Vec<ExportLossReason> {
    entities::generic_polyline_losses(polyline)
}

pub(crate) fn direct_common_losses(
    common: &cadcodec::entities::EntityCommon,
) -> Vec<ExportLossReason> {
    entities::common_semantic_losses(common)
}

pub(crate) fn direct_document_losses(document: &cadcodec::CadDocument) -> Vec<ExportDiagnostic> {
    let mut context = conversion::ExportContext::default();
    context
        .mapped_workspace_ucss
        .extend(document.ucss.iter().map(|ucs| ucs.handle));
    coverage::scan_document_semantics(document, &mut context);
    context.diagnostics
}

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
    PackageBuild(#[from] ocdraw::package::PackageBuildError),
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
