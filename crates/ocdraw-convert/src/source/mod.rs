//! CAD source structure, semantics and loss classification, independent of encoding.
mod appearance;
mod blocks;
mod coverage;
mod diagnostic;
mod entities;
mod layouts;
mod structure;
pub(crate) use crate::ExportOptions;
pub(crate) use appearance::{
    convert_entity_appearance, convert_layer_appearance, AppearanceMode, EntityAppearanceError,
    LayerAppearanceError,
};
pub(crate) use blocks::{
    inspect_markers, inspect_references, ordered_entities, with_recovered_anonymous_block_name,
};
pub use diagnostic::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason,
    SourceStructureProblem,
};
pub(crate) use layouts::{
    is_empty_reserved_paper_block, is_untouched_scaffold, overall_viewport_handle,
};
pub(crate) use structure::{inspect_model_space, with_recovered_model_space_handle};
#[derive(Default)]
struct SourceCoverage {
    diagnostics: Vec<ExportDiagnostic>,
    mapped_block_records: std::collections::BTreeSet<cadcodec::Handle>,
    mapped_workspace_vports: std::collections::BTreeSet<cadcodec::Handle>,
    mapped_workspace_ucss: std::collections::BTreeSet<cadcodec::Handle>,
}
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

pub(crate) fn direct_document_losses(
    document: &cadcodec::CadDocument,
    mapped_vports: &std::collections::BTreeSet<cadcodec::Handle>,
    mapped_blocks: impl IntoIterator<Item = cadcodec::Handle>,
) -> Vec<ExportDiagnostic> {
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
