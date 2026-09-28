use cadcodec::objects::{Layout, ObjectType};
use cadcodec::{CadDocument, Color, Handle, Layer, LineWeight, Transparency};
use ifccad::ifcdr::IfcdrLengthUnit;
use ifccad::package::{load_directory_package, LinePatternRef, PackageOptions};
use ifccad::PackageId;
use ifccad_convert::{
    cad_document_to_package, ExportAction, ExportDiagnosticSource, ExportError, ExportLossReason,
    ExportOptions, SourceStructureProblem,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempRoot(PathBuf);

impl TempRoot {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ifccad-export-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn package_options(label: &str) -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new(format!("export-{label}")).unwrap(),
        data_version: "1".to_owned(),
        author: "Export drawing test".to_owned(),
        timestamp: "2026-09-04T10:00:00Z".to_owned(),
    }
}

fn exported_unit(document: &CadDocument, label: &str) -> IfcdrLengthUnit {
    let outcome =
        cad_document_to_package(document, package_options(label), ExportOptions::default())
            .unwrap_or_else(|error| panic!("export failed: {error}"));
    let root = TempRoot::new(label);
    let package_root = root.0.join("package");
    outcome.package().write_directory(&package_root).unwrap();
    let loaded = load_directory_package(package_root).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let unit = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap()
        .representation()
        .resource()
        .unit();
    unit
}

#[test]
fn exports_active_model_view_and_unused_named_ucs() {
    use cadcodec::{Ucs, Vector2, Vector3};
    use ifccad::ifcdr::{Point2, UcsSelection};
    let mut document = CadDocument::new();
    let origin = Vector3::new(1.0, 4.0, 2.0);
    document.header.model_space_ucs_origin = origin;
    let mut unused =
        Ucs::from_origin_axes("MOVED_ORIGIN", origin, Vector3::UNIT_X, Vector3::UNIT_Y);
    unused.handle = document.allocate_handle();
    document.ucss.add(unused).unwrap();
    let active = document.vports.get_mut("*Active").unwrap();
    active.view_height = 42.0;
    active.grid_spacing = Vector2::new(0.0, 8.0);
    active.snap_spacing = Vector2::new(0.5, 0.25);
    active.snap_on = true;
    active.ucs_origin = origin;
    let outcome = cad_document_to_package(
        &document,
        package_options("workspace-model"),
        ExportOptions::default(),
    )
    .unwrap();
    let root = TempRoot::new("workspace-model");
    let target = root.0.join("package");
    outcome.package().write_directory(&target).unwrap();
    let loaded = load_directory_package(target).unwrap();
    let package = loaded
        .validated_package()
        .unwrap_or_else(|| panic!("{:?}", loaded.report()));
    let resume = package.workspace().unwrap();
    let state = resume
        .active_layout()
        .representation()
        .resource()
        .workspace()
        .unwrap();
    assert_eq!(state.ucs_definitions[0].name, "MOVED_ORIGIN");
    assert_eq!(state.model_windows[0].view.height, 42.0);
    assert_eq!(state.model_windows[0].grid.spacing, Point2::new(0.0, 8.0));
    assert_eq!(state.model_windows[0].snap.spacing, Point2::new(0.5, 0.25));
    assert!(state.model_windows[0].snap.enabled);
    assert!(matches!(
        state.drawing_view_state.unwrap().current_model_ucs,
        UcsSelection::Unnamed { .. }
    ));
    let imported = ifccad_convert::drawing_to_cad_document(resume.active_drawing()).unwrap();
    let cad = imported.document();
    assert_eq!(cad.header.model_space_ucs_origin, origin);
    assert_eq!(cad.ucss.get("MOVED_ORIGIN").unwrap().origin, origin);
    let active = cad.vports.get("*Active").unwrap();
    assert_eq!(active.view_height, 42.0);
    assert_eq!(active.grid_spacing, Vector2::new(0.0, 8.0));
    assert_eq!(active.snap_spacing, Vector2::new(0.5, 0.25));
}

#[test]
fn unsupported_orthographic_ucs_is_reported_and_reject_policy_blocks_export() {
    use cadcodec::{Ucs, Vector3};
    let mut document = CadDocument::new();
    let mut ucs = Ucs::from_origin_axes(
        "ORTHO",
        Vector3::new(1.0, 2.0, 3.0),
        Vector3::UNIT_X,
        Vector3::UNIT_Y,
    );
    ucs.handle = document.allocate_handle();
    ucs.ortho_type = 1;
    document.ucss.add(ucs).unwrap();
    let allowed = cad_document_to_package(
        &document,
        package_options("orthographic-ucs-allow"),
        ExportOptions::default(),
    )
    .unwrap();
    assert!(allowed.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        ifccad_convert::ExportDiagnostic::Loss { reasons, .. }
        if reasons.iter().any(|reason| matches!(reason,
            ExportLossReason::UnsupportedSemantic { name }
            if name.contains("orthographic")
        ))
    )));
    let rejected = cad_document_to_package(
        &document,
        package_options("orthographic-ucs-reject"),
        ExportOptions {
            loss_policy: ifccad_convert::ExportLossPolicy::Reject,
            ..ExportOptions::default()
        },
    )
    .err()
    .expect("Reject must refuse orthographic UCS loss");
    assert!(matches!(rejected, ExportError::LossRejected { .. }));
}

#[test]
fn tiled_vports_keep_both_views_and_diagnose_ambiguous_ucs_activation() {
    use cadcodec::{VPort, Vector2, Vector3};
    let mut document = CadDocument::new();
    document.header.model_space_ucs_origin = Vector3::new(1.0, 4.0, 2.0);
    let first = document.vports.get_mut("*Active").unwrap();
    first.upper_right = Vector2::new(0.5, 1.0);
    let mut second = VPort::active();
    second.handle = document.allocate_handle();
    second.lower_left = Vector2::new(0.5, 0.0);
    second.view_height = 50.0;
    document.vports.add_allow_duplicate(second);
    let outcome = cad_document_to_package(
        &document,
        package_options("tiled-ucs"),
        ExportOptions::default(),
    )
    .unwrap();
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|d| format!("{d:?}").contains("active tiled VPORT")));
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|d| format!("{d:?}").contains("differs from header current UCS")));
    let root = TempRoot::new("tiled-ucs");
    let path = root.0.join("package");
    outcome.package().write_directory(&path).unwrap();
    let loaded = load_directory_package(path).unwrap();
    let package = loaded
        .validated_package()
        .unwrap_or_else(|| panic!("{:?}", loaded.report()));
    let workspace = package
        .workspace()
        .unwrap()
        .active_layout()
        .representation()
        .resource()
        .workspace()
        .unwrap();
    assert_eq!(workspace.model_windows.len(), 2);
    assert_eq!(workspace.model_windows[1].view.height, 50.0);
    assert!(!workspace.model_windows[0].use_stored_ucs);
    assert!(matches!(
        workspace.drawing_view_state.unwrap().current_model_ucs,
        ifccad::ifcdr::UcsSelection::Unnamed { .. }
    ));
}

#[test]
fn structure_requires_one_layout_related_to_the_model_space_block() {
    let mut missing_block = CadDocument::new();
    let model_space = missing_block.header.model_space_block_handle;
    missing_block.block_records.remove("*Model_Space");
    let error = cad_document_to_package(
        &missing_block,
        package_options("missing-block"),
        ExportOptions::default(),
    )
    .err()
    .expect("missing model-space block record must fail");
    assert!(matches!(
        error,
        ExportError::InvalidSourceStructure { problems }
            if problems == [SourceStructureProblem::ModelSpaceBlockRecordMissing {
                model_space_block: model_space,
            }]
    ));

    let mut missing = CadDocument::new();
    let model_space = missing.header.model_space_block_handle;
    missing.objects.retain(
        |_, object| !matches!(object, ObjectType::Layout(layout) if layout.block_record == model_space),
    );
    let error = cad_document_to_package(
        &missing,
        package_options("missing-layout"),
        ExportOptions::default(),
    )
    .err()
    .expect("missing model layout must fail");
    assert!(matches!(
        error,
        ExportError::InvalidSourceStructure { problems }
            if problems == [SourceStructureProblem::ModelLayoutMissing {
                model_space_block: model_space,
            }]
    ));

    let mut multiple = CadDocument::new();
    let model_space = multiple.header.model_space_block_handle;
    let mut duplicate = Layout::new("Second model layout");
    duplicate.handle = Handle::new(0xFFFF);
    duplicate.block_record = model_space;
    multiple
        .objects
        .insert(duplicate.handle, ObjectType::Layout(duplicate));
    let error = cad_document_to_package(
        &multiple,
        package_options("multiple-layouts"),
        ExportOptions::default(),
    )
    .err()
    .expect("ambiguous model layouts must fail");
    assert!(matches!(
        error,
        ExportError::InvalidSourceStructure { problems }
            if problems == [SourceStructureProblem::MultipleModelLayouts {
                model_space_block: model_space,
                count: 2,
            }]
    ));
}

#[test]
fn structure_copies_the_related_model_layout_name_exactly() {
    let mut document = CadDocument::new();
    let model_space = document.header.model_space_block_handle;
    for object in document.objects.values_mut() {
        if let ObjectType::Layout(layout) = object {
            if layout.block_record == model_space {
                layout.name = "Exact model layout name".to_owned();
            }
        }
    }

    let outcome = cad_document_to_package(
        &document,
        package_options("layout-name"),
        ExportOptions::default(),
    )
    .unwrap_or_else(|error| panic!("export failed: {error}"));
    let root = TempRoot::new("layout-name");
    let package_root = root.0.join("package");
    outcome.package().write_directory(&package_root).unwrap();
    let loaded = load_directory_package(package_root).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    let layout = drawing.layouts().next().unwrap();
    assert_eq!(layout.name(), "Exact model layout name");
}

#[test]
fn units_map_exactly_without_coordinate_rescaling() {
    let cases = [
        (0, IfcdrLengthUnit::Unitless),
        (4, IfcdrLengthUnit::Millimetre),
        (5, IfcdrLengthUnit::Centimetre),
        (6, IfcdrLengthUnit::Metre),
        (7, IfcdrLengthUnit::Kilometre),
        (1, IfcdrLengthUnit::Inch),
        (2, IfcdrLengthUnit::Foot),
    ];

    for (source, expected) in cases {
        let mut document = CadDocument::new();
        document.header.insertion_units = source;
        assert_eq!(
            exported_unit(&document, &format!("unit-{source}")),
            expected
        );
    }
}

#[test]
fn units_report_unsupported_values_and_fall_back_to_unitless() {
    let mut document = CadDocument::new();
    document.header.insertion_units = 25;
    let outcome = cad_document_to_package(
        &document,
        package_options("unsupported-unit"),
        ExportOptions::default(),
    )
    .unwrap_or_else(|error| panic!("export failed: {error}"));

    assert_eq!(outcome.diagnostics().len(), 1);
    let diagnostic = &outcome.diagnostics()[0];
    assert_eq!(
        diagnostic.source(),
        &ExportDiagnosticSource::DocumentField {
            name: "header.insertion_units".to_owned(),
        }
    );
    assert_eq!(diagnostic.action(), ExportAction::PartiallyExported);
    assert_eq!(
        diagnostic.reasons(),
        [ExportLossReason::UnsupportedUnit { code: 25 }]
    );
    assert!(diagnostic.is_loss());
    assert_eq!(
        exported_unit(&document, "unsupported-unit-read"),
        IfcdrLengthUnit::Unitless
    );
}

#[test]
fn layers_preserve_order_empty_entries_visibility_and_exact_appearance() {
    let mut document = CadDocument::new();
    let layer_0 = document.layers.get_mut("0").unwrap();
    layer_0.color = Color::Index(1);
    layer_0.color_name = Some("Traffic red".to_owned());
    layer_0.book_name = Some("RAL".to_owned());
    layer_0.line_weight = LineWeight::W0_25;

    let mut rgb = Layer::with_color("Empty RGB", Color::from_rgb(10, 20, 30));
    rgb.flags.frozen = true;
    rgb.line_type = "Dashed".to_owned();
    rgb.line_weight = LineWeight::W0_18;
    rgb.transparency = Transparency::Explicit(128);
    document.layers.add(rgb.clone()).unwrap();

    let mut off = rgb;
    off.name = "Off copy".to_owned();
    off.flags.frozen = false;
    off.flags.off = true;
    off.flags.locked = true;
    off.material = Handle::new(0xABC);
    document.layers.add(off).unwrap();

    let outcome = cad_document_to_package(
        &document,
        package_options("layers"),
        ExportOptions::default(),
    )
    .unwrap_or_else(|error| panic!("export failed: {error}"));
    assert_eq!(outcome.diagnostics().len(), 1);
    let diagnostic = &outcome.diagnostics()[0];
    assert_eq!(
        diagnostic.source(),
        &ExportDiagnosticSource::Layer {
            name: "Off copy".to_owned(),
        }
    );
    assert_eq!(diagnostic.action(), ExportAction::PartiallyExported);
    assert_eq!(
        diagnostic.reasons(),
        [ExportLossReason::MaterialReference {
            handle: Handle::new(0xABC),
        },]
    );

    let root = TempRoot::new("layers");
    let package_root = root.0.join("package");
    outcome.package().write_directory(&package_root).unwrap();
    let loaded = load_directory_package(package_root).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    let layers = drawing.representation().layers().collect::<Vec<_>>();
    assert_eq!(
        layers.iter().map(|layer| layer.name()).collect::<Vec<_>>(),
        ["0", "Empty RGB", "Off copy"]
    );
    assert!(layers[0].visible());
    assert!(layers[1].visible());
    assert!(layers[1].frozen());
    assert!(!layers[2].visible());
    assert!(layers[2].locked());

    let indexed = layers[0].appearance().unwrap();
    assert_eq!(indexed.color().rgb().components(), [255, 0, 0]);
    let aci = indexed.color().indexed().unwrap();
    assert_eq!((aci.system(), aci.index()), ("ACI", 1));
    let named = indexed.color().named().unwrap();
    assert_eq!((named.catalog(), named.name()), ("RAL", "Traffic red"));
    assert_eq!(indexed.line_weight(), 0.25);

    let true_color = layers[1].appearance().unwrap();
    assert_eq!(true_color.color().rgb().components(), [10, 20, 30]);
    assert!(true_color.color().indexed().is_none());
    assert!(true_color.color().named().is_none());
    assert_eq!(true_color.line_pattern(), LinePatternRef::Name("Dashed"));
    assert_eq!(true_color.line_weight(), 0.18);
    assert_eq!(true_color.opacity(), 1.0 - 128.0 / 255.0);
    assert_eq!(layers[2].appearance().unwrap().name(), true_color.name());
}

#[test]
fn a_layer_with_an_unrepresentable_required_appearance_is_skipped() {
    let mut document = CadDocument::new();
    let mut unsupported = Layer::new("Inherited layer color");
    unsupported.color = Color::ByLayer;
    unsupported.transparency = Transparency::ByLayer;
    unsupported.line_type.clear();
    unsupported.line_weight = LineWeight::ByLayer;
    document.layers.add(unsupported).unwrap();

    let outcome = cad_document_to_package(
        &document,
        package_options("skipped-layer"),
        ExportOptions::default(),
    )
    .unwrap_or_else(|error| panic!("export failed: {error}"));
    assert_eq!(outcome.diagnostics().len(), 1);
    let diagnostic = &outcome.diagnostics()[0];
    assert_eq!(diagnostic.action(), ExportAction::Skipped);
    assert_eq!(
        diagnostic.source(),
        &ExportDiagnosticSource::Layer {
            name: "Inherited layer color".to_owned(),
        }
    );
    assert_eq!(
        diagnostic.reasons(),
        [
            ExportLossReason::LayerColorUnsupported {
                color: "ByLayer".to_owned(),
            },
            ExportLossReason::LayerTransparencyUnsupported,
            ExportLossReason::LayerLinePatternMissing,
            ExportLossReason::LayerLineWeightUnsupported { value: -1 },
        ]
    );

    let root = TempRoot::new("skipped-layer");
    let package_root = root.0.join("package");
    outcome.package().write_directory(&package_root).unwrap();
    let loaded = load_directory_package(package_root).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    assert_eq!(
        drawing
            .representation()
            .layers()
            .map(|layer| layer.name())
            .collect::<Vec<_>>(),
        ["0"]
    );
}
