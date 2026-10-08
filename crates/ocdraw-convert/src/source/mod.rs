//! CAD source structure, semantics and loss classification, independent of encoding.
mod appearance;
mod blocks;
mod coverage;
pub(crate) mod workspace;

mod entities;
mod layouts;
mod structure;
pub use crate::diagnostics::{
    CadSourceStructureProblem, CadToOcdrawAction, CadToOcdrawDiagnostic,
    CadToOcdrawDiagnosticSource, CadToOcdrawLossReason,
};
pub(crate) use crate::CadToOcdrawOptions;
pub(crate) use appearance::{
    convert_entity_appearance, convert_layer_appearance, AppearanceMode, EntityAppearanceError,
    LayerAppearanceError,
};
pub(crate) use blocks::{inspect_markers, inspect_references, ordered_entities};
pub(crate) use layouts::{
    is_empty_reserved_paper_block, is_untouched_scaffold, overall_viewport_handle,
};
pub(crate) use structure::{inspect_model_space, with_recovered_model_space_handle};
#[derive(Default)]
struct SourceCoverage {
    diagnostics: Vec<CadToOcdrawDiagnostic>,
    mapped_block_records: std::collections::BTreeSet<opencadcodec::Handle>,
    mapped_workspace_vports: std::collections::BTreeSet<opencadcodec::Handle>,
    mapped_workspace_ucss: std::collections::BTreeSet<opencadcodec::Handle>,
}
pub(crate) fn direct_line_losses(line: &opencadcodec::Line) -> Vec<CadToOcdrawLossReason> {
    entities::line_losses(line)
}

pub(crate) fn direct_point_losses(point: &opencadcodec::Point) -> Vec<CadToOcdrawLossReason> {
    entities::point_losses(point)
}

pub(crate) fn direct_circle_losses(circle: &opencadcodec::Circle) -> Vec<CadToOcdrawLossReason> {
    entities::circle_losses(circle)
}

pub(crate) fn direct_arc_losses(arc: &opencadcodec::Arc) -> Vec<CadToOcdrawLossReason> {
    entities::arc_losses(arc)
}

pub(crate) fn direct_ellipse_losses(ellipse: &opencadcodec::Ellipse) -> Vec<CadToOcdrawLossReason> {
    entities::ellipse_losses(ellipse)
}

pub(crate) fn direct_planar_polyline_losses(
    polyline: &opencadcodec::LwPolyline,
) -> Vec<CadToOcdrawLossReason> {
    entities::polyline_losses(polyline)
}

pub(crate) fn direct_spatial_polyline_losses(
    polyline: &opencadcodec::entities::Polyline3D,
) -> Vec<CadToOcdrawLossReason> {
    entities::polyline3d_losses(polyline)
}

pub(crate) fn direct_legacy_planar_polyline_losses(
    polyline: &opencadcodec::entities::Polyline2D,
) -> Vec<CadToOcdrawLossReason> {
    entities::polyline2d_losses(polyline)
}

pub(crate) fn direct_generic_spatial_polyline_losses(
    polyline: &opencadcodec::entities::Polyline,
) -> Vec<CadToOcdrawLossReason> {
    entities::generic_polyline_losses(polyline)
}

pub(crate) fn direct_common_losses(
    common: &opencadcodec::entities::EntityCommon,
) -> Vec<CadToOcdrawLossReason> {
    entities::common_semantic_losses(common)
}

pub(crate) fn direct_document_losses(
    document: &opencadcodec::CadDocument,
    mapped_vports: &std::collections::BTreeSet<opencadcodec::Handle>,
    mapped_blocks: impl IntoIterator<Item = opencadcodec::Handle>,
) -> Vec<CadToOcdrawDiagnostic> {
    let mut context = SourceCoverage::default();
    context
        .mapped_workspace_ucss
        .extend(document.ucss.iter().map(|ucs| ucs.handle));
    context
        .mapped_workspace_vports
        .extend(mapped_vports.iter().copied());
    context.mapped_block_records.extend(mapped_blocks);
    coverage::scan_document_semantics(document, &mut context);
    context.diagnostics
}
