use cadcodec::{CadDocument, EntityType, Handle, Line, LineType};
use ifccad::package::PackageOptions;
use ifccad::PackageId;
use ifccad_convert::{
    cad_document_to_package, ExportAction, ExportDiagnosticSource, ExportLossReason, ExportOptions,
};

fn package_options() -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new("coverage-test").unwrap(),
        data_version: "1".to_owned(),
        author: "Coverage test".to_owned(),
        timestamp: "2026-09-04T10:00:00Z".to_owned(),
    }
}

#[test]
fn supported_entity_common_semantics_are_emitted_and_attached_semantics_are_reported() {
    let mut document = CadDocument::new();
    let handle = document
        .add_entity(EntityType::Line(Line::from_coords(
            0.0, 0.0, 0.0, 1.0, 1.0, 0.0,
        )))
        .unwrap();
    let common = document.get_entity_mut(handle).unwrap().common_mut();
    common.linetype_scale = 2.0;
    common.linetype_handle = Some(Handle::new(0x801));
    common.graphic_data = Some(vec![1, 2, 3]);
    common.reactors.push(Handle::new(0x802));
    common.xdictionary_handle = Some(Handle::new(0x803));
    common.color_book_handle = Some(Handle::new(0x804));
    common.full_visual_style_handle = Some(Handle::new(0x805));
    common.face_visual_style_handle = Some(Handle::new(0x806));
    common.edge_visual_style_handle = Some(Handle::new(0x807));
    common.material_flags = 3;
    common.material_handle = Some(Handle::new(0x808));
    common.shadow_flags = 1;
    common.plotstyle_flags = 3;
    common.plotstyle_handle = Some(Handle::new(0x809));

    let outcome = cad_document_to_package(&document, package_options(), ExportOptions::default())
        .unwrap_or_else(|error| panic!("export failed: {error}"));
    assert_eq!(outcome.entity_mapping().len(), 1);
    let diagnostic = outcome
        .diagnostics()
        .iter()
        .find(|diagnostic| {
            diagnostic.source()
                == &ExportDiagnosticSource::Entity {
                    handle,
                    kind: "LINE".to_owned(),
                }
        })
        .expect("entity attachment loss diagnostic");
    assert_eq!(diagnostic.action(), ExportAction::PartiallyExported);
    assert_eq!(
        diagnostic.reasons(),
        [
            ExportLossReason::EntityLinetypeScale,
            ExportLossReason::EntityLinetypeHandle,
            ExportLossReason::EntityGraphicData,
            ExportLossReason::EntityReactors,
            ExportLossReason::EntityExtensionDictionary,
            ExportLossReason::EntityColorBookReference,
            ExportLossReason::EntityFullVisualStyle,
            ExportLossReason::EntityFaceVisualStyle,
            ExportLossReason::EntityEdgeVisualStyle,
            ExportLossReason::EntityMaterial,
            ExportLossReason::EntityShadowFlags,
            ExportLossReason::EntityPlotStyle,
        ]
    );
}

#[test]
fn document_tables_metadata_and_public_side_views_are_covered_deterministically() {
    let mut document = CadDocument::new();
    document.header.project_name = "IFCCAD pilot".to_owned();
    document.header.text_height = 9.0;
    document.summary_info.title = "Coverage drawing".to_owned();
    document.line_types.add(LineType::new("CUSTOM")).unwrap();
    document
        .context_scales
        .insert(Handle::new(0x901), Handle::new(0x902));

    let outcome = cad_document_to_package(&document, package_options(), ExportOptions::default())
        .unwrap_or_else(|error| panic!("export failed: {error}"));
    let projected = outcome
        .diagnostics()
        .iter()
        .map(|diagnostic| (diagnostic.source().clone(), diagnostic.reasons().to_vec()))
        .collect::<Vec<_>>();
    assert_eq!(
        projected,
        [
            (
                ExportDiagnosticSource::DocumentField {
                    name: "header.project_name".to_owned(),
                },
                vec![ExportLossReason::UnsupportedHeaderField {
                    name: "project_name".to_owned(),
                }],
            ),
            (
                ExportDiagnosticSource::DocumentField {
                    name: "header.other_semantics".to_owned(),
                },
                vec![ExportLossReason::UnsupportedHeaderField {
                    name: "other_header_semantics".to_owned(),
                }],
            ),
            (
                ExportDiagnosticSource::DocumentField {
                    name: "summary_info".to_owned(),
                },
                vec![ExportLossReason::DocumentSummaryInformation],
            ),
            (
                ExportDiagnosticSource::Table {
                    kind: "line_types".to_owned(),
                },
                vec![ExportLossReason::UnsupportedTableRecords {
                    kind: "line_types".to_owned(),
                    count: 1,
                }],
            ),
            (
                ExportDiagnosticSource::Collection {
                    kind: "context_scales".to_owned(),
                    count: 1,
                },
                vec![ExportLossReason::UnsupportedCollection {
                    kind: "context_scales".to_owned(),
                    count: 1,
                }],
            ),
        ]
    );
}

#[test]
fn bare_header_handle_values_do_not_count_as_semantic_loss() {
    let mut document = CadDocument::new();
    document.header.handle_seed = 0xFFFF;
    document.header.layer_control_handle = Handle::new(0xA01);
    document.header.named_objects_dict_handle = Handle::new(0xA02);
    document.header.current_layer_handle = Handle::new(0xA03);

    let outcome = cad_document_to_package(&document, package_options(), ExportOptions::default())
        .unwrap_or_else(|error| panic!("export failed: {error}"));
    assert!(outcome.diagnostics().is_empty());
}
