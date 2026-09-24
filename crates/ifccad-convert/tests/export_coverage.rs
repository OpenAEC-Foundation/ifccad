use cadcodec::{classes::DxfClassCollection, objects::ObjectType};
use cadcodec::{CadDocument, EntityType, Handle, Line, LineType, Vector3};
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
    assert!(!outcome.diagnostics().iter().any(|diagnostic| {
        diagnostic.reasons().iter().any(|reason| {
            matches!(
                reason,
                ExportLossReason::UnsupportedCollection { kind, .. }
                if kind == "inventory.additional_relationships"
            )
        })
    }));
}

#[test]
fn document_tables_and_metadata_are_covered_deterministically() {
    let mut document = CadDocument::new();
    document.header.project_name = "IFCCAD pilot".to_owned();
    document.header.text_height = 9.0;
    document.summary_info.title = "Coverage drawing".to_owned();
    document.line_types.add(LineType::new("CUSTOM")).unwrap();

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

#[test]
fn cached_geometry_extents_are_not_independent_drawing_settings() {
    let mut document = CadDocument::new();
    document
        .add_entity(EntityType::Line(Line::from_coords(
            0.0, 0.0, 0.0, 10.0, 5.0, 0.0,
        )))
        .unwrap();
    document.header.model_space_extents_min = Vector3::new(-100.0, -100.0, 0.0);
    document.header.model_space_extents_max = Vector3::new(100.0, 100.0, 0.0);
    document.header.paper_space_extents_min = Vector3::new(-1.0, -1.0, 0.0);
    document.header.paper_space_extents_max = Vector3::new(1.0, 1.0, 0.0);
    let outcome = cad_document_to_package(
        &document,
        package_options(),
        ExportOptions {
            geometry_tolerance: Default::default(),
            loss_policy: ifccad_convert::ExportLossPolicy::Reject,
        },
    )
    .unwrap();
    assert!(outcome.diagnostics().is_empty());
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "ifccad-cached-extents-{}-{unique}",
        std::process::id()
    ));
    outcome.package().write_directory(&root).unwrap();
    let loaded = ifccad::package::load_directory_package(&root).unwrap();
    assert!(loaded.report().is_empty());
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let bounds = drawing.layouts().next().unwrap().scope().bounds().unwrap();
    assert_eq!(bounds.min(), ifccad::ifcdr::Point3::new(0.0, 0.0, 0.0));
    assert_eq!(bounds.max(), ifccad::ifcdr::Point3::new(10.0, 5.0, 0.0));
}

#[test]
fn meaningful_current_defaults_and_drawing_limits_still_reject() {
    for change in [0, 1] {
        let mut document = CadDocument::new();
        if change == 0 {
            document.header.current_line_weight = 50;
        } else {
            document.header.model_space_limits_max.x += 100.0;
        }
        assert!(matches!(
            cad_document_to_package(
                &document,
                package_options(),
                ExportOptions {
                    geometry_tolerance: Default::default(),
                    loss_policy: ifccad_convert::ExportLossPolicy::Reject
                }
            ),
            Err(ifccad_convert::ExportError::LossRejected { .. })
        ));
    }
}

#[test]
fn upstream_drawing_variables_are_reported_as_unsupported_objects() {
    let mut document = CadDocument::new();
    assert!(document.set_hatch_origin([12.5, -8.25]));
    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(outcome.diagnostics().iter().any(|diagnostic| {
        diagnostic.reasons().iter().any(|reason| {
            matches!(reason, ExportLossReason::UnsupportedCollection { kind, count }
                if kind == "objects" && *count > 0)
        })
    }));
    assert!(matches!(
        cad_document_to_package(
            &document,
            package_options(),
            ExportOptions {
                geometry_tolerance: Default::default(),
                loss_policy: ifccad_convert::ExportLossPolicy::Reject
            }
        ),
        Err(ifccad_convert::ExportError::LossRejected { .. })
    ));
}

#[test]
fn layer_description_is_preserved_without_dropping_geometry() {
    let mut document = CadDocument::new();
    document.layers.get_mut("0").unwrap().description = "Draagconstructie".into();
    document.add_entity(EntityType::Line(Line::new())).unwrap();
    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert_eq!(outcome.entity_mapping().len(), 1);
    assert!(!outcome
        .diagnostics()
        .iter()
        .any(|d| d.source() == &ExportDiagnosticSource::Layer { name: "0".into() }));
    let entry: serde_json::Value =
        serde_json::from_slice(outcome.package().file("package.ifcx.json").unwrap()).unwrap();
    assert!(entry["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|node| node["type"] == "openaec:Layer"
            && node["attributes"]["description"] == "Draagconstructie"));
}

#[test]
fn newly_exposed_table_fields_are_reported_even_on_bootstrap_records() {
    for (table, change) in [("line_types", 0), ("text_styles", 1), ("block_records", 2)] {
        let mut document = CadDocument::new();
        match change {
            0 => {
                document
                    .line_types
                    .get_mut("Continuous")
                    .unwrap()
                    .xref_block_record_handle = Handle::new(0xB01)
            }
            1 => {
                document
                    .text_styles
                    .get_mut("Standard")
                    .unwrap()
                    .xref_block_record_handle = Handle::new(0xB01)
            }
            _ => {
                document
                    .block_records
                    .get_mut("*Model_Space")
                    .unwrap()
                    .flags
                    .is_xref_unloaded = true
            }
        }
        let outcome =
            cad_document_to_package(&document, package_options(), ExportOptions::default())
                .unwrap();
        assert!(
            outcome.diagnostics().iter().any(|d| d.reasons().contains(
                &ExportLossReason::UnsupportedTableRecords {
                    kind: table.into(),
                    count: 1
                }
            )),
            "{table}"
        );
        assert_rejected(&document);
    }
}

#[test]
fn inventory_exposes_layer_extension_dictionary_relationships() {
    let mut document = CadDocument::new();
    let layer = document.layers.get("0").unwrap().handle;
    document.ensure_extension_dictionary(layer);
    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(outcome.diagnostics().iter().any(|d| d.reasons().contains(
        &ExportLossReason::UnsupportedCollection {
            kind: "inventory.additional_relationships".into(),
            count: 1
        }
    )));
    assert_rejected(&document);
}

#[test]
fn inventory_reports_extended_data_not_represented_by_the_typed_layer() {
    let mut source = CadDocument::new();
    source.layers.get_mut("0").unwrap().description = "Stored description".into();
    let bytes = cadcodec::DwgWriter::write_to_vec(&source).unwrap();
    let mut document = cadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes))
        .read()
        .unwrap();
    assert_eq!(
        document.layers.get("0").unwrap().description,
        "Stored description"
    );
    // The public typed field no longer accounts for the retained EED payload.
    document.layers.get_mut("0").unwrap().description.clear();
    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|d| d.reasons().iter().any(|r| {
            matches!(r, ExportLossReason::UnsupportedCollection { kind, count }
            if kind == "inventory.non_entity_extended_data" && *count > 0)
        })));
    assert_rejected(&document);
}

fn assert_rejected(document: &CadDocument) {
    assert!(matches!(
        cad_document_to_package(
            document,
            package_options(),
            ExportOptions {
                loss_policy: ifccad_convert::ExportLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(ifccad_convert::ExportError::LossRejected { .. })
    ));
}

#[test]
fn changed_bootstrap_content_is_not_hidden_by_unchanged_collection_lengths() {
    let mut document = CadDocument::new();
    document.dim_styles.get_mut("Standard").unwrap().dimdle = 2.5;
    let group_dictionary = document
        .objects
        .get_mut(&document.header.acad_group_dict_handle)
        .unwrap();
    let ObjectType::Dictionary(group_dictionary) = group_dictionary else {
        panic!("bootstrap group dictionary")
    };
    group_dictionary.duplicate_cloning = 2;

    let mut classes = DxfClassCollection::new();
    for mut class in document.classes.iter().cloned() {
        if class.dxf_name == "LAYOUT" {
            class.application_name = "Changed application".into();
        }
        classes.push_preserving(class);
    }
    document.classes = classes;

    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    for (kind, count) in [("dim_styles", 1), ("objects", 1), ("classes", 1)] {
        assert!(
            outcome
                .diagnostics()
                .iter()
                .any(
                    |diagnostic| diagnostic.reasons().iter().any(|reason| match reason {
                        ExportLossReason::UnsupportedTableRecords {
                            kind: actual,
                            count: n,
                        }
                        | ExportLossReason::UnsupportedCollection {
                            kind: actual,
                            count: n,
                        } => actual == kind && *n == count,
                        _ => false,
                    })
                ),
            "missing {kind} loss: {:?}",
            outcome.diagnostics()
        );
    }
    assert_rejected(&document);
}

#[test]
fn decoded_side_view_does_not_create_independent_semantic_loss() {
    let mut document = CadDocument::new();
    document
        .context_scales
        .insert(Handle::new(0x901), Handle::new(0x902));
    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(outcome.diagnostics().is_empty());
}

#[test]
fn renumbered_bootstrap_dictionary_is_still_the_same_scaffold() {
    let mut document = CadDocument::new();
    let old_handle = document.header.acad_group_dict_handle;
    let new_handle = document.allocate_handle();
    let mut group = document.objects.remove(&old_handle).unwrap();
    let ObjectType::Dictionary(dictionary) = &mut group else {
        panic!("bootstrap group dictionary")
    };
    dictionary.handle = new_handle;
    document.objects.insert(new_handle, group);
    document.header.acad_group_dict_handle = new_handle;
    let ObjectType::Dictionary(root) = document
        .objects
        .get_mut(&document.header.named_objects_dict_handle)
        .unwrap()
    else {
        panic!("bootstrap root dictionary")
    };
    let entry = root
        .entries
        .iter_mut()
        .find(|(key, _)| key == "ACAD_GROUP")
        .unwrap();
    entry.1 = new_handle;

    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(
        outcome.diagnostics().is_empty(),
        "{:?}",
        outcome.diagnostics()
    );
}

#[test]
fn resolved_entity_reactor_to_table_record_is_reported_once() {
    let mut document = CadDocument::new();
    let layer_handle = document.layers.get("0").unwrap().handle;
    let entity_handle = document.add_entity(EntityType::Line(Line::new())).unwrap();
    document
        .get_entity_mut(entity_handle)
        .unwrap()
        .common_mut()
        .reactors
        .push(layer_handle);

    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(outcome.diagnostics().iter().any(|diagnostic| {
        diagnostic
            .reasons()
            .contains(&ExportLossReason::EntityReactors)
    }));
    assert!(!outcome.diagnostics().iter().any(|diagnostic| {
        diagnostic.reasons().iter().any(|reason| {
            matches!(
                reason,
                ExportLossReason::UnsupportedCollection { kind, .. }
                if kind == "inventory.additional_relationships"
            )
        })
    }));
}

#[test]
fn duplicate_standard_class_is_additional_source_content() {
    let mut document = CadDocument::new();
    let standard = document.classes.get_by_name("LAYOUT").unwrap().clone();
    document.classes.push_preserving(standard);
    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.reasons().contains(
            &ExportLossReason::UnsupportedCollection {
                kind: "classes".into(),
                count: 1,
            }
        )));
    assert_rejected(&document);
}

#[test]
fn duplicate_standard_table_record_is_additional_source_content() {
    let mut document = CadDocument::new();
    let mut standard = document.dim_styles.get("Standard").unwrap().clone();
    standard.handle = document.allocate_handle();
    document.dim_styles.add_allow_duplicate(standard);
    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.reasons().contains(
            &ExportLossReason::UnsupportedTableRecords {
                kind: "dim_styles".into(),
                count: 1,
            }
        )));
    assert_rejected(&document);
}

#[test]
fn modified_dashed_linetype_definition_is_not_hidden_by_its_supported_name() {
    let mut document = CadDocument::new();
    let mut dashed = LineType::dashed();
    dashed.handle = document.allocate_handle();
    dashed.description = "Custom dash sequence".into();
    document.line_types.add(dashed).unwrap();
    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.reasons().contains(
            &ExportLossReason::UnsupportedTableRecords {
                kind: "line_types".into(),
                count: 1,
            }
        )));
    assert_rejected(&document);
}

#[test]
fn extra_layout_dictionary_alias_is_not_treated_as_an_exported_layout() {
    let mut document = CadDocument::new();
    let layout_handle = document
        .objects
        .iter()
        .find_map(|(handle, object)| match object {
            ObjectType::Layout(layout) if layout.name == "Layout1" => Some(*handle),
            _ => None,
        })
        .unwrap();
    let ObjectType::Dictionary(dictionary) = document
        .objects
        .get_mut(&document.header.acad_layout_dict_handle)
        .unwrap()
    else {
        panic!("layout dictionary")
    };
    dictionary.add_entry("Alias", layout_handle);
    let outcome =
        cad_document_to_package(&document, package_options(), ExportOptions::default()).unwrap();
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.reasons().contains(
            &ExportLossReason::UnsupportedCollection {
                kind: "objects".into(),
                count: 1,
            }
        )));
    assert_rejected(&document);
}
