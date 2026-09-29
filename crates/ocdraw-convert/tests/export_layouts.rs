use cadcodec::entities::EntityType;
use cadcodec::entities::Viewport;
use cadcodec::objects::ObjectType;
use cadcodec::{CadDocument, Layer, Line, LwPolyline, Vector2};
use ocdraw::ifcdr::IfcdrEntityRef;
use ocdraw::package::{load_directory_package, PackageOptions};
use ocdraw::PackageId;
use ocdraw_convert::{
    cad_document_to_package, drawing_to_cad_document, ExportAction, ExportError, ExportLossPolicy,
    ExportLossReason, ExportOptions,
};

fn options() -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new("cad-layouts").unwrap(),
        data_version: "1".into(),
        author: "test".into(),
        timestamp: "2026-09-23T00:00:00Z".into(),
    }
}

#[test]
fn paper_canvas_views_survive_strict_package_and_cad_roundtrip() {
    let mut source = CadDocument::new();
    let mut first_canvas = Viewport::new();
    first_canvas.id = 1;
    first_canvas.view_center = cadcodec::Vector3::new(7.5, 4.0, 0.0);
    first_canvas.view_height = 9.0;
    source
        .add_entity_to_layout(EntityType::Viewport(first_canvas), "Layout1")
        .unwrap();
    let mut authored = Viewport::new();
    authored.id = 2;
    source
        .add_entity_to_layout(EntityType::Viewport(authored), "Layout1")
        .unwrap();
    let second = source.add_layout("Layout2").unwrap();
    let ObjectType::Layout(second_layout) = source.objects.get(&second).unwrap() else {
        panic!()
    };
    let second_canvas = second_layout.viewport;
    let EntityType::Viewport(second_view) = source.get_entity_mut(second_canvas).unwrap() else {
        panic!()
    };
    second_view.view_center = cadcodec::Vector3::new(12.0, 6.0, 0.0);
    second_view.view_height = 11.0;

    let exported = cad_document_to_package(&source, options(), ExportOptions::default()).unwrap();
    let root = std::env::temp_dir().join(format!("ifccad-paper-canvas-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    exported.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let workspace = drawing.representation().resource().workspace().unwrap();
    assert_eq!(workspace.paper_canvases.len(), 2);
    let imported = drawing_to_cad_document(drawing).unwrap();
    for (name, center_x, height) in [("Layout1", 7.5, 9.0), ("Layout2", 12.0, 11.0)] {
        let layout = imported
            .document()
            .objects
            .values()
            .find_map(|object| match object {
                ObjectType::Layout(layout) if layout.name == name => Some(layout),
                _ => None,
            })
            .unwrap();
        let EntityType::Viewport(canvas) = imported.document().get_entity(layout.viewport).unwrap()
        else {
            panic!()
        };
        assert_eq!(canvas.id, 1, "{name}");
        assert_eq!(canvas.view_center.x, center_x, "{name}");
        assert_eq!(canvas.view_height, height, "{name}");
    }
    assert!(imported
        .document()
        .entities()
        .any(|entity| matches!(entity, EntityType::Viewport(viewport) if viewport.id == 2)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn inactive_model_tab_does_not_discard_paper_canvas_view() {
    let mut source = CadDocument::new();
    source.header.show_model_space = false;
    let sheet = source.add_layout("Sheet").unwrap();
    let ObjectType::Layout(layout) = source.objects.get(&sheet).unwrap() else {
        panic!()
    };
    let canvas_handle = layout.viewport;
    let EntityType::Viewport(canvas) = source.get_entity_mut(canvas_handle).unwrap() else {
        panic!()
    };
    canvas.view_height = 8.5;
    let exported = cad_document_to_package(&source, options(), ExportOptions::default()).unwrap();
    let root = std::env::temp_dir().join(format!("ifccad-paper-tab-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    exported.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let canvas = &drawing
        .representation()
        .resource()
        .workspace()
        .unwrap()
        .paper_canvases[0];
    assert_eq!(canvas.view.height, 8.5);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn active_paper_layout_and_unlinked_overall_viewport_roundtrip() {
    let mut source = CadDocument::new();
    source.header.show_model_space = false;
    let layout_handle = source
        .objects
        .iter()
        .find_map(|(&handle, object)| match object {
            ObjectType::Layout(layout) if layout.name == "Layout1" => Some(handle),
            _ => None,
        })
        .unwrap();
    let ObjectType::Layout(layout) = source.objects.get(&layout_handle).unwrap() else {
        panic!()
    };
    source.header.paper_space_block_handle = layout.block_record;
    let mut overall = Viewport::new();
    overall.id = 1;
    overall.view_center = cadcodec::Vector3::new(6.0, 4.5, 0.0);
    overall.view_height = 12.0;
    overall.snap_spacing = cadcodec::Vector3::ZERO;
    source
        .add_entity_to_layout(EntityType::Viewport(overall), "Layout1")
        .unwrap();
    let mut model_window = Viewport::new();
    model_window.id = 2;
    source
        .add_entity_to_layout(EntityType::Viewport(model_window), "Layout1")
        .unwrap();
    let Some(ObjectType::Layout(layout)) = source.objects.get_mut(&layout_handle) else {
        panic!()
    };
    layout.viewport = cadcodec::Handle::NULL;
    assert!(
        matches!(source.objects.get(&layout_handle), Some(ObjectType::Layout(layout)) if layout.viewport == cadcodec::Handle::NULL)
    );

    let exported = cad_document_to_package(&source, options(), ExportOptions::default()).unwrap();
    let root = std::env::temp_dir().join(format!("ifccad-active-paper-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    exported.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let package = loaded
        .validated_package()
        .unwrap_or_else(|| panic!("{:?}", loaded.report()));
    let resume = package.workspace().unwrap();
    assert_eq!(resume.active_layout().name(), "Layout1");
    let drawing = resume.active_drawing();
    let workspace = drawing.representation().resource().workspace().unwrap();
    assert_eq!(workspace.paper_canvases.len(), 1);
    assert_eq!(workspace.paper_canvases[0].view.height, 12.0);
    let imported = drawing_to_cad_document(drawing).unwrap();
    assert!(!imported.document().header.show_model_space);
    let canvas_count = imported
        .document()
        .entities()
        .filter(|entity| matches!(entity, EntityType::Viewport(viewport) if viewport.id == 1))
        .count();
    assert_eq!(canvas_count, 1);
    let model_count = imported
        .document()
        .entities()
        .filter(|entity| matches!(entity, EntityType::Viewport(viewport) if viewport.id > 1))
        .count();
    assert_eq!(model_count, 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn zeroed_paper_viewport_id_reports_unresolved_canvas() {
    let mut source = CadDocument::new();
    source.header.show_model_space = false;
    let mut viewport = Viewport::new();
    viewport.id = 0;
    source
        .add_entity_to_layout(EntityType::Viewport(viewport), "Layout1")
        .unwrap();
    let layout = source
        .objects
        .values_mut()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == "Layout1" => Some(layout),
            _ => None,
        })
        .unwrap();
    layout.min_limits = (0.0, 0.0);
    layout.max_limits = (12.0, 9.0);
    let exported = cad_document_to_package(&source, options(), ExportOptions::default()).unwrap();
    assert!(exported
        .diagnostics()
        .iter()
        .any(|diagnostic| format!("{diagnostic:?}")
            .contains("paper overall viewport identity unavailable")));
    assert!(exported
        .diagnostics()
        .iter()
        .any(|diagnostic| format!("{diagnostic:?}")
            .contains("active paper layout has no recoverable canvas")));
    let root = std::env::temp_dir().join(format!("ifccad-zeroed-paper-id-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    exported.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let package = loaded
        .validated_package()
        .unwrap_or_else(|| panic!("{:?}", loaded.report()));
    assert_eq!(package.workspace().unwrap().active_layout().name(), "Model");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn viewport_free_paper_layout_stays_viewport_free_after_dxf_and_dwg_roundtrip() {
    let mut source = CadDocument::new();
    let mut first_canvas = Viewport::new();
    first_canvas.id = 1;
    source
        .add_entity_to_layout(EntityType::Viewport(first_canvas), "Layout1")
        .unwrap();
    let handle = source.add_layout("Layout2").unwrap();
    let ObjectType::Layout(layout) = source.objects.get_mut(&handle).unwrap() else {
        panic!()
    };
    let canvas_handle = layout.viewport;
    layout.viewport = cadcodec::Handle::NULL;
    layout.viewports.clear();
    layout.min_limits = (0.0, 0.0);
    layout.max_limits = (12.0, 9.0);
    let layout2_block = layout.block_record;
    source.header.show_model_space = false;
    source.header.paper_space_block_handle = source
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == "Layout1" => Some(layout.block_record),
            _ => None,
        })
        .unwrap();
    source
        .block_records
        .iter_mut()
        .find(|record| record.handle == layout2_block)
        .unwrap()
        .entity_handles
        .retain(|handle| *handle != canvas_handle);
    source.remove_entity(canvas_handle);
    let exported = cad_document_to_package(&source, options(), ExportOptions::default()).unwrap();
    let root =
        std::env::temp_dir().join(format!("ifccad-layout-limits-view-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    exported.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let package = loaded
        .validated_package()
        .unwrap_or_else(|| panic!("{:?}", loaded.report()));
    assert_eq!(
        package.workspace().unwrap().active_layout().name(),
        "Layout1"
    );
    let drawing = package.drawings().next().unwrap();
    let layout2_scope_id = drawing
        .layouts()
        .find(|layout| layout.name() == "Layout2")
        .unwrap()
        .scope()
        .id()
        .get() as u32;
    assert!(!drawing
        .representation()
        .resource()
        .workspace()
        .unwrap()
        .paper_canvases
        .iter()
        .any(|canvas| canvas.scope_id == layout2_scope_id));
    let imported = drawing_to_cad_document(drawing).unwrap();
    let layout = imported
        .document()
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == "Layout2" => Some(layout),
            _ => None,
        })
        .unwrap();
    assert_eq!(layout.viewport, cadcodec::Handle::NULL);
    assert!(!imported.document().entities().any(|entity| matches!(entity, EntityType::Viewport(viewport) if viewport.common.owner_handle == layout.block_record)));
    let dxf = root.with_extension("dxf");
    cadcodec::DxfWriter::new(imported.document())
        .write_to_file(&dxf)
        .unwrap();
    let reread = cadcodec::DxfReader::from_file(&dxf)
        .unwrap()
        .read()
        .unwrap();
    let layout = reread
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == "Layout2" => Some(layout),
            _ => None,
        })
        .unwrap();
    assert_eq!(layout.viewport, cadcodec::Handle::NULL);
    assert!(!reread.entities().any(|entity| matches!(entity, EntityType::Viewport(viewport) if viewport.common.owner_handle == layout.block_record)));
    let dwg = root.with_extension("dwg");
    std::fs::write(
        &dwg,
        cadcodec::DwgWriter::write_to_vec(imported.document()).unwrap(),
    )
    .unwrap();
    let reread = cadcodec::DwgReader::from_file(&dwg)
        .unwrap()
        .read()
        .unwrap();
    let layout = reread
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == "Layout2" => Some(layout),
            _ => None,
        })
        .unwrap();
    assert_eq!(layout.viewport, cadcodec::Handle::NULL);
    assert!(!reread.entities().any(|entity| matches!(entity, EntityType::Viewport(viewport) if viewport.common.owner_handle == layout.block_record)));
    let _ = std::fs::remove_file(&dxf);
    let _ = std::fs::remove_file(&dwg);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn padded_dxf_plot_integer_values_keep_paper_rotation() {
    let mut source = CadDocument::new();
    let handle = source.add_layout("Rotated sheet").unwrap();
    let ObjectType::Layout(layout) = source.objects.get_mut(&handle).unwrap() else {
        panic!()
    };
    layout.paper_width = 215.9;
    layout.paper_height = 279.4;
    layout.raw_plot_settings_codes = Some(vec![
        (72, "     0".into()),
        (73, "     1".into()),
        (74, "     5".into()),
        (75, "    16".into()),
    ]);
    let exported = cad_document_to_package(&source, options(), ExportOptions::default()).unwrap();
    let root = std::env::temp_dir().join(format!("ifccad-plot-rotation-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    exported.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let imported = drawing_to_cad_document(drawing).unwrap();
    let layout = imported
        .document()
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == "Rotated sheet" => Some(layout),
            _ => None,
        })
        .unwrap();
    assert_eq!(layout.paper_width, 215.9);
    assert_eq!(layout.paper_height, 279.4);
    assert_eq!(layout.plot_rotation, 1);
    let dxf = root.with_extension("dxf");
    cadcodec::DxfWriter::new(imported.document())
        .write_to_file(&dxf)
        .unwrap();
    let reread = cadcodec::DxfReader::from_file(&dxf)
        .unwrap()
        .read()
        .unwrap();
    let written = reread
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == "Rotated sheet" => Some(layout),
            _ => None,
        })
        .unwrap();
    assert!(written
        .raw_plot_settings_codes
        .as_ref()
        .unwrap()
        .iter()
        .any(|(code, value)| *code == 73 && value.trim() == "1"));
    let _ = std::fs::remove_file(&dxf);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn dxf_roundtripped_empty_default_layout_remains_scaffolding() {
    let source = CadDocument::new();
    let path =
        std::env::temp_dir().join(format!("ifccad-layout-scaffold-{}.dxf", std::process::id()));
    cadcodec::DxfWriter::new(&source)
        .write_to_file(&path)
        .unwrap();
    let roundtripped = cadcodec::DxfReader::from_file(&path)
        .unwrap()
        .read()
        .unwrap();
    let outcome =
        cad_document_to_package(&roundtripped, options(), ExportOptions::default()).unwrap();
    let root = std::env::temp_dir().join(format!("ifccad-layout-scaffold-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    outcome.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    assert_eq!(drawing.layouts().count(), 1);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn dwg_roundtripped_empty_default_layout_remains_scaffolding() {
    let source = CadDocument::new();
    let bytes = cadcodec::DwgWriter::write_to_vec(&source).unwrap();
    let roundtripped = cadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes))
        .read()
        .unwrap();
    let outcome =
        cad_document_to_package(&roundtripped, options(), ExportOptions::default()).unwrap();
    assert!(
        outcome.diagnostics().is_empty(),
        "{:?}",
        outcome.diagnostics()
    );
}

#[test]
fn duplicate_paper_block_record_is_a_source_structure_error() {
    let mut document = CadDocument::new();
    let first = document.add_layout("Sheet One").unwrap();
    let ObjectType::Layout(existing) = document.objects.get(&first).unwrap() else {
        panic!()
    };
    let mut duplicate = existing.clone();
    duplicate.handle = document.allocate_handle();
    duplicate.name = "Sheet Two".into();
    document
        .objects
        .insert(duplicate.handle, ObjectType::Layout(duplicate));
    assert!(matches!(
        cad_document_to_package(&document, options(), ExportOptions::default()),
        Err(ExportError::InvalidSourceStructure { .. })
    ));
}

#[test]
fn unconfigured_medium_does_not_silently_drop_meaningful_plot_options() {
    let mut document = CadDocument::new();
    let handle = document.add_layout("Unsized plot").unwrap();
    let ObjectType::Layout(layout) = document.objects.get_mut(&handle).unwrap() else {
        panic!()
    };
    layout.plot_flags.print_lineweights = true;
    let outcome = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert!(outcome.diagnostics().iter().any(|diagnostic| diagnostic.reasons().iter().any(|reason| matches!(reason, ExportLossReason::UnsupportedSemantic { name } if name.contains("unconfigured plot medium")))));
}

#[test]
fn exports_a_real_paper_layout_without_the_untouched_scaffold() {
    let mut document = CadDocument::new();
    let layout_handle = document.add_layout("Sheet A").unwrap();
    let ObjectType::Layout(layout) = document.objects.get_mut(&layout_handle).unwrap() else {
        panic!("layout")
    };
    layout.paper_width = 210.0;
    layout.paper_height = 297.0;
    layout.plot_paper_units = 1;
    layout.plot_scale_type = 1;
    document
        .add_entity_to_layout(EntityType::Line(Line::new()), "Sheet A")
        .unwrap();

    let exported = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    let root = std::env::temp_dir().join(format!("ifccad-export-layouts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    exported.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let layouts = drawing.layouts().collect::<Vec<_>>();
    assert_eq!(
        layouts
            .iter()
            .map(|layout| layout.name())
            .collect::<Vec<_>>(),
        ["Model", "Sheet A"]
    );
    let representation = layouts[1].representation();
    let resource = representation.resource();
    let paper_entities = resource
        .entities(layouts[1].scope().id())
        .collect::<Vec<_>>();
    assert!(matches!(
        paper_entities.as_slice(),
        [IfcdrEntityRef::Line(_)]
    ));
    let entry: serde_json::Value =
        serde_json::from_slice(exported.package().file("package.ifcx.json").unwrap()).unwrap();
    assert_eq!(
        entry["data"][3]["attributes"]["plotSettings"]["area"]["mode"],
        "Layout"
    );
    let imported = drawing_to_cad_document(drawing).unwrap();
    assert!(imported.document().objects.values().any(|object| matches!(object, ObjectType::Layout(layout) if layout.name == "Sheet A" && layout.paper_width == 210.0 && layout.paper_height == 297.0)));
    let imported_layout = imported
        .document()
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == "Sheet A" => Some(layout),
            _ => None,
        })
        .unwrap();
    assert!(imported.document().entities().any(|entity| matches!(entity, EntityType::Line(line) if line.common.owner_handle == imported_layout.block_record)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn exports_a_rectangular_orthographic_paper_viewport() {
    let mut document = CadDocument::new();
    document.add_layout("Sheet V").unwrap();
    let mut viewport = Viewport::new();
    viewport.id = 2;
    viewport.center = cadcodec::Vector3::new(100.0, 80.0, 0.0);
    viewport.width = 60.0;
    viewport.height = 40.0;
    viewport.view_height = 20.0;
    document
        .add_entity_to_layout(EntityType::Viewport(viewport), "Sheet V")
        .unwrap();
    let exported = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    let root = std::env::temp_dir().join(format!("ifccad-export-viewport-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    exported.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let paper = drawing
        .layouts()
        .find(|layout| layout.name() == "Sheet V")
        .unwrap();
    let resource = paper.representation().resource();
    let entities = resource.entities(paper.scope().id()).collect::<Vec<_>>();
    assert!(
        matches!(entities.as_slice(), [IfcdrEntityRef::Viewport(viewport)] if viewport.frame().width == 60.0 && viewport.frame().height == 40.0)
    );
    let imported = drawing_to_cad_document(drawing).unwrap();
    assert!(imported.document().entities().any(|entity| matches!(entity, EntityType::Viewport(viewport) if viewport.id == 2 && viewport.width == 60.0 && viewport.height == 40.0)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn exports_zero_dormant_lens_on_orthographic_viewport() {
    let mut document = CadDocument::new();
    document.add_layout("Zero lens").unwrap();
    let mut viewport = Viewport::new();
    viewport.id = 2;
    viewport.lens_length = 0.0;
    document
        .add_entity_to_layout(EntityType::Viewport(viewport), "Zero lens")
        .unwrap();

    let exported = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    let root = std::env::temp_dir().join(format!("ifccad-zero-lens-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    exported.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let paper = drawing
        .layouts()
        .find(|layout| layout.name() == "Zero lens")
        .unwrap();
    let resource = paper.representation().resource();
    let entities = resource.entities(paper.scope().id()).collect::<Vec<_>>();
    assert!(
        matches!(entities.as_slice(), [IfcdrEntityRef::Viewport(viewport)]
        if viewport.view().projection == ocdraw::ifcdr::ProjectionMode::Orthographic
            && viewport.view().lens_length == Some(0.0))
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn deferred_display_area_omits_complete_plot_settings_under_allow_and_rejects() {
    let mut document = CadDocument::new();
    let handle = document.add_layout("Display sheet").unwrap();
    let ObjectType::Layout(layout) = document.objects.get_mut(&handle).unwrap() else {
        panic!()
    };
    layout.paper_width = 200.0;
    layout.paper_height = 100.0;
    layout.plot_type = 0;
    let outcome = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert!(outcome.diagnostics().iter().any(|diagnostic| diagnostic.action() == ExportAction::Skipped && diagnostic.reasons().iter().any(|reason| matches!(reason, ExportLossReason::UnsupportedSemantic { name } if name.contains("Display")))));
    let root = std::env::temp_dir().join(format!("ifccad-export-display-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    outcome.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let layout = drawing
        .layouts()
        .find(|layout| layout.name() == "Display sheet")
        .unwrap();
    assert!(layout.settings().plot_settings.is_none());
    let _ = std::fs::remove_dir_all(&root);
    assert!(matches!(
        cad_document_to_package(
            &document,
            options(),
            ExportOptions {
                loss_policy: ExportLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(ExportError::LossRejected { .. })
    ));
}

#[test]
fn active_external_plot_style_table_keeps_name_but_reports_missing_contents() {
    let mut document = CadDocument::new();
    document.header.plotstyle_mode = false;
    let handle = document.add_layout("Styled sheet").unwrap();
    let ObjectType::Layout(layout) = document.objects.get_mut(&handle).unwrap() else {
        panic!()
    };
    layout.paper_width = 297.0;
    layout.paper_height = 210.0;
    layout.plot_scale_type = 1;
    layout.plot_style_sheet = "office.ctb".into();
    layout.plot_flags.plot_plot_styles = true;
    let outcome = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert!(outcome.diagnostics().iter().any(|diagnostic| diagnostic.reasons().iter().any(|reason| matches!(reason, ExportLossReason::UnsupportedSemantic { name } if name.contains("CTB/STB")))));
    let root = std::env::temp_dir().join(format!("ifccad-export-style-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    outcome.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    assert_eq!(
        drawing.plot_style_mode(),
        ocdraw::package::PlotStyleMode::Named
    );
    let layout = drawing
        .layouts()
        .find(|layout| layout.name() == "Styled sheet")
        .unwrap();
    assert_eq!(
        layout
            .settings()
            .plot_settings
            .unwrap()
            .output
            .plot_style_table_name
            .as_deref(),
        Some("office.ctb")
    );
    assert!(
        !drawing_to_cad_document(drawing)
            .unwrap()
            .document()
            .header
            .plotstyle_mode
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn active_unresolved_viewport_clip_skips_entire_viewport() {
    let mut document = CadDocument::new();
    document.add_layout("Clipped sheet").unwrap();
    let mut viewport = Viewport::new();
    viewport.id = 2;
    viewport.clip_boundary_handle = cadcodec::Handle::new(0xabc);
    document
        .add_entity_to_layout(EntityType::Viewport(viewport), "Clipped sheet")
        .unwrap();
    let outcome = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert!(outcome.diagnostics().iter().any(|diagnostic| diagnostic.action() == ExportAction::Skipped && diagnostic.reasons().iter().any(|reason| matches!(reason, ExportLossReason::UnsupportedSemantic { name } if name.contains("clip boundary")))));
    let root = std::env::temp_dir().join(format!("ifccad-export-clip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    outcome.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let paper = drawing
        .layouts()
        .find(|layout| layout.name() == "Clipped sheet")
        .unwrap();
    assert_eq!(
        paper
            .representation()
            .resource()
            .entities(paper.scope().id())
            .count(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
    assert!(matches!(
        cad_document_to_package(
            &document,
            options(),
            ExportOptions {
                loss_policy: ExportLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(ExportError::LossRejected { .. })
    ));
}

#[test]
fn closed_straight_clip_boundary_is_preserved() {
    let mut document = CadDocument::new();
    document.add_layout("Clipped valid").unwrap();
    let mut boundary = LwPolyline::from_points(vec![
        Vector2::new(80.0, 70.0),
        Vector2::new(120.0, 70.0),
        Vector2::new(120.0, 90.0),
        Vector2::new(80.0, 90.0),
    ]);
    boundary.is_closed = true;
    let boundary_handle = document
        .add_entity_to_layout(EntityType::LwPolyline(boundary), "Clipped valid")
        .unwrap();
    let mut viewport = Viewport::new();
    viewport.id = 2;
    viewport.center = cadcodec::Vector3::new(100.0, 80.0, 0.0);
    viewport.width = 60.0;
    viewport.height = 40.0;
    viewport.clip_boundary_handle = boundary_handle;
    document
        .add_entity_to_layout(EntityType::Viewport(viewport), "Clipped valid")
        .unwrap();
    let outcome = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    let root =
        std::env::temp_dir().join(format!("ifccad-export-valid-clip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    outcome.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let paper = drawing
        .layouts()
        .find(|layout| layout.name() == "Clipped valid")
        .unwrap();
    let resource = paper.representation().resource();
    let entities = resource.entities(paper.scope().id()).collect::<Vec<_>>();
    assert!(
        matches!(entities.as_slice(), [IfcdrEntityRef::PlanarPolyline(_), IfcdrEntityRef::Viewport(viewport)] if viewport.paper_clip().enabled && viewport.paper_clip().boundary_entity_id.is_some())
    );
    let imported = drawing_to_cad_document(drawing).unwrap();
    let viewport = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::Viewport(viewport) if viewport.id != 1 => Some(viewport),
            _ => None,
        })
        .unwrap();
    assert_ne!(viewport.clip_boundary_handle, cadcodec::Handle::NULL);
    assert!(matches!(
        imported
            .document()
            .get_entity(viewport.clip_boundary_handle),
        Some(EntityType::LwPolyline(_))
    ));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn frozen_viewport_layer_maps_to_relational_override_and_back() {
    let mut document = CadDocument::new();
    document.add_layout("Frozen sheet").unwrap();
    let mut layer = Layer::new("A-WALL");
    layer.handle = document.allocate_handle();
    let layer_handle = layer.handle;
    document.layers.add(layer).unwrap();
    let mut viewport = Viewport::new();
    viewport.id = 2;
    viewport.frozen_layers.push(layer_handle);
    document
        .add_entity_to_layout(EntityType::Viewport(viewport), "Frozen sheet")
        .unwrap();
    let outcome = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    let root = std::env::temp_dir().join(format!("ifccad-export-frozen-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    outcome.package().write_directory(&root).unwrap();
    let loaded = load_directory_package(&root).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let paper = drawing
        .layouts()
        .find(|layout| layout.name() == "Frozen sheet")
        .unwrap();
    let resource = paper.representation().resource();
    let viewport = resource
        .entities(paper.scope().id())
        .find_map(|entity| match entity {
            IfcdrEntityRef::Viewport(viewport) => Some(viewport),
            _ => None,
        })
        .unwrap();
    assert_eq!(viewport.layer_overrides().len(), 1);
    assert!(viewport.layer_overrides()[0].frozen);
    let imported = drawing_to_cad_document(drawing).unwrap();
    let viewport = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::Viewport(viewport) if viewport.id != 1 => Some(viewport),
            _ => None,
        })
        .unwrap();
    let imported_layer = imported.document().layers.get("A-WALL").unwrap();
    assert!(viewport.frozen_layers.contains(&imported_layer.handle));
    let _ = std::fs::remove_dir_all(&root);
}
