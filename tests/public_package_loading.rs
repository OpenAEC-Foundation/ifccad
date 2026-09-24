use ifccad::conformance::bundled_conformance_root;
use ifccad::package::{
    load_directory_package, PackageHeaderRef, PackageOpenError, PackageValidationReport,
    ValidatedPackage, DIRECTORY_PACKAGE_ENTRYPOINT,
};
use ifccad::PackageId;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ifccad-public-loading-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn minimal_package() -> PathBuf {
    bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation")
}

#[test]
fn model_and_paper_layouts_share_the_drawing_resource() {
    let outcome = load_directory_package(
        bundled_conformance_root().join("packages/valid/shared-layout-representation"),
    )
    .expect("open shared representation fixture");
    assert!(outcome.report().is_valid(), "{:?}", outcome.report());
    let package = outcome.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    let representation: ifccad::package::DrawingRepresentationRef<'_> = drawing.representation();
    assert_eq!(representation.role(), "drawing");
    let layouts: Vec<_> = drawing.layouts().collect();
    assert_eq!(layouts.len(), 2);
    for (layout, expected_ids) in layouts.iter().zip([vec![1, 2, 3], vec![4]]) {
        assert_eq!(layout.representation().path(), representation.path());
        assert_eq!(
            layout.representation().resource_id(),
            representation.resource_id()
        );
        let ids: Vec<_> = representation
            .resource()
            .entities(layout.scope().id())
            .map(|entity| match entity {
                ifccad::ifcdr::IfcdrEntityRef::Point(point) => point.entity_id().get(),
                ifccad::ifcdr::IfcdrEntityRef::Circle(circle) => circle.entity_id().get(),
                ifccad::ifcdr::IfcdrEntityRef::Arc(arc) => arc.entity_id().get(),
                ifccad::ifcdr::IfcdrEntityRef::Ellipse(ellipse) => ellipse.entity_id().get(),
                ifccad::ifcdr::IfcdrEntityRef::EllipseArc(arc) => arc.entity_id().get(),
                ifccad::ifcdr::IfcdrEntityRef::Line(line) => line.entity_id().get(),
                ifccad::ifcdr::IfcdrEntityRef::PlanarPolyline(polyline) => {
                    polyline.entity_id().get()
                }
                ifccad::ifcdr::IfcdrEntityRef::SpatialPolyline(polyline) => {
                    polyline.entity_id().get()
                }
                ifccad::ifcdr::IfcdrEntityRef::BlockInstance(instance) => {
                    instance.entity_id().get()
                }
                ifccad::ifcdr::IfcdrEntityRef::Viewport(viewport) => viewport.entity_id().get(),
            })
            .collect();
        assert_eq!(ids, expected_ids);
    }
}

fn copy_minimal_package(root: &Path) -> serde_json::Value {
    let source = minimal_package();
    for entry in fs::read_dir(&source).expect("read minimal package") {
        let entry = entry.expect("minimal package entry");
        if entry.file_type().expect("minimal entry type").is_file() {
            fs::copy(entry.path(), root.join(entry.file_name()))
                .expect("copy minimal package file");
        }
    }
    serde_json::from_slice(
        &fs::read(root.join(DIRECTORY_PACKAGE_ENTRYPOINT)).expect("read copied entrypoint"),
    )
    .expect("parse copied entrypoint")
}

fn candidate_package(root: &Path) -> serde_json::Value {
    use sha2::{Digest, Sha256};
    let mut entrypoint = copy_minimal_package(root);
    let resource_path = root.join("drawing.ifcdr.json");
    let mut resource: serde_json::Value =
        serde_json::from_slice(&fs::read(&resource_path).unwrap()).unwrap();
    resource["header"]["version"] = serde_json::json!("0.11.0");
    let bytes = serde_json::to_vec_pretty(&resource).unwrap();
    fs::write(&resource_path, &bytes).unwrap();
    entrypoint["data"][3]["attributes"]["resource"]["version"] = serde_json::json!("0.11.0");
    entrypoint["data"][3]["attributes"]["resource"]["checksum"] =
        serde_json::json!(format!("sha256:{:x}", Sha256::digest(&bytes)));
    entrypoint["data"][1]["children"]["Layers"] = serde_json::json!(["layer-0", "layer-a-wall"]);
    entrypoint["data"][1]["children"]["Appearances"] =
        serde_json::json!(["appearance-default-solid", "appearance-dashed-red"]);
    entrypoint["data"][2]["attributes"]
        .as_object_mut()
        .unwrap()
        .remove("limitsCheckEnabled");
    entrypoint["data"][2]["attributes"]["limitsChecking"] = serde_json::json!(false);
    for layer_index in [4, 5] {
        let attrs = &mut entrypoint["data"][layer_index]["attributes"];
        attrs["frozen"] = serde_json::json!(false);
        attrs["locked"] = serde_json::json!(false);
        attrs["plottable"] = serde_json::json!(true);
        attrs["frozenInNewViewports"] = serde_json::json!(false);
    }
    entrypoint
}

#[test]
fn candidate_drawing_lists_close_resource_bindings() {
    let root = TestDirectory::new("candidate-drawing-lists");
    let mut entrypoint = candidate_package(root.path());
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        serde_json::to_vec(&entrypoint).unwrap(),
    )
    .unwrap();
    let valid = load_directory_package(root.path()).unwrap();
    assert!(valid.validated_package().is_some(), "{:?}", valid.report());

    entrypoint["data"][1]["children"]["Layers"] = serde_json::json!(["layer-0"]);
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        serde_json::to_vec(&entrypoint).unwrap(),
    )
    .unwrap();
    let invalid = load_directory_package(root.path()).unwrap();
    assert!(invalid.validated_package().is_none());
    assert!(invalid
        .report()
        .iter()
        .any(|d| d.code == "IFCCAD_PACKAGE_BINDING_INVALID"));
}

#[test]
fn two_drawings_can_share_layer_and_appearance_nodes_for_one_resource() {
    let root = TestDirectory::new("shared-drawing-definitions");
    let mut entrypoint = candidate_package(root.path());
    let mut second_drawing = entrypoint["data"][1].clone();
    second_drawing["path"] = serde_json::json!("drawing-other");
    second_drawing["children"]["Layouts"] = serde_json::json!(["drawing-other-layout-model"]);
    let mut second_layout = entrypoint["data"][2].clone();
    second_layout["path"] = serde_json::json!("drawing-other-layout-model");
    entrypoint["data"][0]["children"]["Drawings"] =
        serde_json::json!(["drawing-main", "drawing-other"]);
    entrypoint["data"]
        .as_array_mut()
        .unwrap()
        .push(second_drawing);
    entrypoint["data"]
        .as_array_mut()
        .unwrap()
        .push(second_layout);
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        serde_json::to_vec(&entrypoint).unwrap(),
    )
    .unwrap();

    let loaded = load_directory_package(root.path()).unwrap();
    let package = loaded
        .validated_package()
        .unwrap_or_else(|| panic!("{:?}", loaded.report()));
    let drawings = package.drawings().collect::<Vec<_>>();
    assert_eq!(drawings.len(), 2);
    for drawing in drawings {
        assert_eq!(drawing.representation().layers().count(), 2);
        assert_eq!(drawing.layouts().count(), 1);
    }
}

fn candidate_plot_settings(area_mode: &str) -> serde_json::Value {
    serde_json::json!({
        "media":{"unit":"mm","width":210.0,"height":297.0,"printableArea":{"minX":5.0,"minY":5.0,"maxX":205.0,"maxY":292.0},"rotation":"none"},
        "area":{"mode":area_mode},
        "mapping":{"scale":{"mode":"Fixed","outputLength":1.0,"scopeLength":1.0},"placement":{"mode":"Offset","reference":"Media","x":0.0,"y":0.0}},
        "output":{"shadedPlot":{"mode":"AsDisplayed","quality":{"mode":"Normal"}},"applyPlotStyles":true},
        "options":{"plotViewportBorders":false,"plotPaperSpaceLast":true,"hidePaperSpaceObjects":false,"plotLineWeights":true,"scaleLineWeights":false,"plotTransparency":false}
    })
}

#[test]
fn candidate_plot_area_uses_layout_kind_and_authored_limits() {
    let root = TestDirectory::new("candidate-plot-area");
    let mut entrypoint = candidate_package(root.path());
    entrypoint["data"][2]["attributes"]["plotSettings"] = candidate_plot_settings("Limits");
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        serde_json::to_vec(&entrypoint).unwrap(),
    )
    .unwrap();
    let missing_limits = load_directory_package(root.path()).unwrap();
    assert!(missing_limits.validated_package().is_none());
    assert!(missing_limits
        .report()
        .iter()
        .any(|d| d.code == "IFCCAD_PACKAGE_BINDING_INVALID"));

    entrypoint["data"][2]["attributes"]["limits"] =
        serde_json::json!({"minX":0.0,"minY":0.0,"maxX":100.0,"maxY":100.0});
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        serde_json::to_vec(&entrypoint).unwrap(),
    )
    .unwrap();
    let valid = load_directory_package(root.path()).unwrap();
    assert!(valid.validated_package().is_some(), "{:?}", valid.report());

    entrypoint["data"][2]["attributes"]["plotSettings"]["area"]["mode"] =
        serde_json::json!("Layout");
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        serde_json::to_vec(&entrypoint).unwrap(),
    )
    .unwrap();
    assert!(load_directory_package(root.path())
        .unwrap()
        .validated_package()
        .is_none());

    entrypoint["data"][2]["attributes"]["plotSettings"]["area"]["mode"] =
        serde_json::json!("Limits");
    entrypoint["data"][2]["attributes"]["plotSettings"]["media"]["printableArea"]["maxX"] =
        serde_json::json!(220.0);
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        serde_json::to_vec(&entrypoint).unwrap(),
    )
    .unwrap();
    assert!(load_directory_package(root.path())
        .unwrap()
        .validated_package()
        .is_none());
}

fn assert_public_types(
    package: &ValidatedPackage,
    header: PackageHeaderRef<'_>,
    report: &PackageValidationReport,
) {
    let _: &PackageId = header.package_id();
    let _: &str = header.ifcx_version();
    let _: &str = header.data_version();
    let _: &str = header.author();
    let _: &str = header.timestamp();
    let _ = (package, report);
}

#[test]
fn package_identity_is_distinct_and_preserves_caller_text() {
    let id = PackageId::new("  package:building/main  ").expect("package ID");

    assert_eq!(id.as_str(), "  package:building/main  ");
    assert_eq!(id.to_string(), "  package:building/main  ");
    assert_eq!(PackageId::new(""), Err(ifccad::InvalidPackageId));
}

#[test]
fn valid_directory_exposes_a_strict_package_and_report() {
    let outcome = load_directory_package(minimal_package()).expect("open valid package");

    assert!(outcome.report().is_valid());
    let package = outcome
        .validated_package()
        .expect("valid package has strict proof");
    let header = package.header();
    assert_public_types(package, header, outcome.report());
    assert_eq!(
        header.package_id().as_str(),
        "ifccad/examples/golden-minimal/golden_minimal.ifcx.json"
    );
    assert_eq!(header.ifcx_version(), "ifcx_alpha");
    assert_eq!(header.data_version(), "0.5.0");
    assert_eq!(header.author(), "IFC-CAD prototype");
    assert_eq!(header.timestamp(), "2026-07-06T00:00:00Z");
    assert_eq!(DIRECTORY_PACKAGE_ENTRYPOINT, "package.ifcx.json");

    assert!(load_directory_package(minimal_package())
        .unwrap()
        .into_validated_package()
        .is_some());
    let (package, report) = load_directory_package(minimal_package())
        .unwrap()
        .into_parts();
    assert!(package.is_some());
    assert!(report.is_valid());
}

#[test]
fn incomplete_header_is_inspectable_without_a_strict_proof() {
    let root = TestDirectory::new("incomplete-header");
    let mut entrypoint = copy_minimal_package(root.path());
    entrypoint["header"].as_object_mut().unwrap().remove("id");
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        serde_json::to_vec(&entrypoint).expect("serialize incomplete entrypoint"),
    )
    .expect("write incomplete entrypoint");

    let outcome = load_directory_package(root.path()).expect("inspect incomplete header");

    assert!(outcome.validated_package().is_none());
    assert!(outcome.report().iter().any(|diagnostic| {
        diagnostic.code == "IFCCAD_PACKAGE_SCHEMA_INVALID"
            && diagnostic.location.as_deref() == Some("/header")
            && diagnostic.context.get("property")
                == Some(&ifccad::package::PackageDiagnosticContextValue::String(
                    "id".to_owned(),
                ))
    }));
}

#[test]
fn schema_failure_is_an_inspectable_outcome_without_a_strict_package() {
    let root = TestDirectory::new("schema-error");
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        br#"{"data":[{"path":"layout","type":"openaec:DrawingLayout","attributes":{"name":"Model","kind":"sheet","scopeId":0},"children":{"Representation":"missing"}}]}"#,
    )
    .expect("write invalid entrypoint");

    let outcome = load_directory_package(root.path()).expect("inspect invalid package");

    assert!(!outcome.report().is_valid());
    assert!(outcome.validated_package().is_none());
    assert!(outcome
        .report()
        .iter()
        .any(|diagnostic| diagnostic.code == "IFCCAD_PACKAGE_SCHEMA_INVALID"));
}

#[test]
fn file_root_is_a_public_open_error() {
    let root = TestDirectory::new("file-root");
    let file = root.path().join("not-a-directory.ifccad");
    fs::write(&file, b"not a directory").expect("write test file");

    let error = load_directory_package(&file).expect_err("file root must fail");

    assert!(matches!(
        error,
        PackageOpenError::RootNotDirectory { path } if path == file
    ));
}
