use super::conversion::ExportContext;
use super::{ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason};
use cadcodec::CadDocument;

pub(crate) fn scan_document_semantics(document: &CadDocument, context: &mut ExportContext) {
    if !document.header.project_name.is_empty() {
        context.diagnostics.push(ExportDiagnostic::loss(
            ExportDiagnosticSource::DocumentField {
                name: "header.project_name".to_owned(),
            },
            ExportAction::Skipped,
            vec![ExportLossReason::UnsupportedHeaderField {
                name: "project_name".to_owned(),
            }],
        ));
    }

    let baseline = CadDocument::new();
    let mut remaining_header = document.header.clone();
    let mut baseline_header = baseline.header.clone();
    remaining_header.project_name.clear();
    baseline_header.project_name.clear();
    remaining_header.insertion_units = baseline_header.insertion_units;
    remaining_header.measurement = baseline_header.measurement;
    normalize_header_bookkeeping(&mut remaining_header, &baseline_header);
    if remaining_header != baseline_header {
        context.diagnostics.push(ExportDiagnostic::loss(
            ExportDiagnosticSource::DocumentField {
                name: "header.other_semantics".to_owned(),
            },
            ExportAction::Skipped,
            vec![ExportLossReason::UnsupportedHeaderField {
                name: "other_header_semantics".to_owned(),
            }],
        ));
    }
    if document.summary_info != baseline.summary_info {
        context.diagnostics.push(ExportDiagnostic::loss(
            ExportDiagnosticSource::DocumentField {
                name: "summary_info".to_owned(),
            },
            ExportAction::Skipped,
            vec![ExportLossReason::DocumentSummaryInformation],
        ));
    }

    let custom_line_types = document
        .line_types
        .iter()
        .filter(|line_type| {
            !matches!(
                line_type.name.as_str(),
                "Continuous" | "ByLayer" | "ByBlock" | "Dashed"
            )
        })
        .count();
    record_table(context, "line_types", custom_line_types);
    record_table(
        context,
        "text_styles",
        document
            .text_styles
            .len()
            .saturating_sub(baseline.text_styles.len()),
    );
    record_table(
        context,
        "block_records",
        document
            .block_records
            .len()
            .saturating_sub(baseline.block_records.len()),
    );
    record_table(
        context,
        "dim_styles",
        document
            .dim_styles
            .len()
            .saturating_sub(baseline.dim_styles.len()),
    );
    record_table(
        context,
        "app_ids",
        document
            .app_ids
            .len()
            .saturating_sub(baseline.app_ids.len()),
    );
    record_table(context, "views", document.views.len());
    record_table(
        context,
        "vports",
        document.vports.len().saturating_sub(baseline.vports.len()),
    );
    record_table(context, "ucss", document.ucss.len());
    record_table(context, "vx_table", document.vx_table.len());

    record_collection(
        context,
        "vx_control_entries",
        document.vx_control_entries.len(),
    );
    record_collection(
        context,
        "classes",
        document
            .classes
            .len()
            .saturating_sub(baseline.classes.len()),
    );
    record_collection(
        context,
        "block_visibility_params",
        document.block_visibility_params.len(),
    );
    record_collection(context, "context_scales", document.context_scales.len());
    record_collection(
        context,
        "block_representations",
        document.block_representations.len(),
    );
    record_collection(context, "fields", document.fields.len());
    record_collection(
        context,
        "dgn_ls_definitions",
        document.dgn_ls_definitions.len(),
    );
    record_collection(
        context,
        "dgn_ls_components",
        document.dgn_ls_components.len(),
    );
    record_collection(context, "preview", usize::from(document.preview.is_some()));
    record_collection(
        context,
        "section_view_style",
        usize::from(document.section_view_style.is_some()),
    );
    record_collection(context, "view_rep_refs", document.view_rep_refs.len());
    record_collection(
        context,
        "section_view_reps",
        document.section_view_reps.len(),
    );

    let extra_objects = document
        .objects
        .len()
        .saturating_sub(baseline.objects.len());
    record_collection(context, "objects", extra_objects);
}

fn normalize_header_bookkeeping(
    header: &mut cadcodec::document::HeaderVariables,
    baseline: &cadcodec::document::HeaderVariables,
) {
    macro_rules! normalize {
        ($($field:ident),+ $(,)?) => {
            $(header.$field = baseline.$field;)+
        };
    }
    normalize!(
        handle_seed,
        ucs_ortho_ref,
        paper_ucs_ortho_ref,
        current_layer_handle,
        current_text_style_handle,
        current_linetype_handle,
        current_dimstyle_handle,
        current_multiline_style_handle,
        current_material_handle,
        dim_text_style_handle,
        dim_linetype_handle,
        dim_linetype1_handle,
        dim_linetype2_handle,
        dim_arrow_block_handle,
        dim_arrow_block1_handle,
        dim_arrow_block2_handle,
        block_control_handle,
        layer_control_handle,
        style_control_handle,
        linetype_control_handle,
        view_control_handle,
        ucs_control_handle,
        vport_control_handle,
        appid_control_handle,
        dimstyle_control_handle,
        vpent_hdr_control_handle,
        current_vx_handle,
        named_objects_dict_handle,
        acad_group_dict_handle,
        acad_mlinestyle_dict_handle,
        acad_layout_dict_handle,
        acad_plotsettings_dict_handle,
        acad_plotstylename_dict_handle,
        acad_material_dict_handle,
        acad_color_dict_handle,
        acad_visualstyle_dict_handle,
        model_space_block_handle,
        paper_space_block_handle,
        bylayer_linetype_handle,
        byblock_linetype_handle,
        continuous_linetype_handle,
    );
}

fn record_table(context: &mut ExportContext, kind: &str, count: usize) {
    if count == 0 {
        return;
    }
    context.diagnostics.push(ExportDiagnostic::loss(
        ExportDiagnosticSource::Table {
            kind: kind.to_owned(),
        },
        ExportAction::Skipped,
        vec![ExportLossReason::UnsupportedTableRecords {
            kind: kind.to_owned(),
            count,
        }],
    ));
}

fn record_collection(context: &mut ExportContext, kind: &str, count: usize) {
    if count == 0 {
        return;
    }
    context.diagnostics.push(ExportDiagnostic::loss(
        ExportDiagnosticSource::Collection {
            kind: kind.to_owned(),
            count,
        },
        ExportAction::Skipped,
        vec![ExportLossReason::UnsupportedCollection {
            kind: kind.to_owned(),
            count,
        }],
    ));
}
