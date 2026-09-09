use ifccad::conformance::bundled_conformance_root;
use ifccad::package::load_directory_package;
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "ifccad-inline-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let source = bundled_conformance_root().join("packages/valid").join(name);
        for item in fs::read_dir(source).unwrap() {
            let item = item.unwrap();
            if item.file_type().unwrap().is_file() {
                fs::copy(item.path(), path.join(item.file_name())).unwrap();
            }
        }
        Self(path)
    }
    fn graph(&self) -> Value {
        serde_json::from_slice(&fs::read(self.0.join("package.ifcx.json")).unwrap()).unwrap()
    }
    fn save(&self, graph: &Value) {
        fs::write(
            self.0.join("package.ifcx.json"),
            serde_json::to_vec(graph).unwrap(),
        )
        .unwrap();
    }
    fn inline(&self, field: &str) {
        let mut graph = self.graph();
        for node in graph["data"].as_array_mut().unwrap() {
            if let Some(d) = node
                .get_mut("attributes")
                .and_then(|a| a.get_mut(field))
                .and_then(Value::as_object_mut)
            {
                let uri = d.remove("uri").unwrap();
                let file = self.0.join(uri.as_str().unwrap());
                d.remove("checksum");
                d.insert(
                    "content".into(),
                    serde_json::from_slice(&fs::read(&file).unwrap()).unwrap(),
                );
                fs::remove_file(file).unwrap();
            }
        }
        self.save(&graph);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn inline_drawing_loads_without_external_file() {
    let fixture = Fixture::new("minimal-no-preservation");
    fixture.inline("resource");
    let outcome = load_directory_package(&fixture.0).unwrap();
    assert!(outcome.report().is_valid(), "{:?}", outcome.report());
    let package = outcome.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    assert_eq!(drawing.representation().external_uri(), None);
    assert_eq!(
        drawing.representation().resource_id().as_str(),
        "drawing-main"
    );
    assert_eq!(
        drawing
            .representation()
            .resource()
            .entities(drawing.layouts().next().unwrap().scope().id())
            .count(),
        4
    );
}

#[test]
fn preservation_links_work_in_all_source_combinations() {
    for (drawing_inline, preservation_inline) in
        [(false, false), (true, false), (false, true), (true, true)]
    {
        let fixture = Fixture::new("source-archive");
        if drawing_inline {
            fixture.inline("resource");
        }
        if preservation_inline {
            fixture.inline("preservation");
        }
        let outcome = load_directory_package(&fixture.0).unwrap();
        assert!(
            outcome.report().is_valid(),
            "sources {drawing_inline}/{preservation_inline}: {:?}",
            outcome.report()
        );
    }
}

#[test]
fn inline_body_errors_point_inside_the_containing_document() {
    let fixture = Fixture::new("minimal-no-preservation");
    fixture.inline("resource");
    let mut graph = fixture.graph();
    graph["data"][3]["attributes"]["resource"]["content"]["header"]["version"] =
        "unsupported".into();
    fixture.save(&graph);
    let outcome = load_directory_package(&fixture.0).unwrap();
    let error = outcome
        .report()
        .iter()
        .find(|d| d.code == "IFCCAD_IFCDR_VERSION_UNSUPPORTED")
        .unwrap();
    assert_eq!(error.resource_uri.as_deref(), Some("package.ifcx.json"));
    assert_eq!(
        error.location.as_deref(),
        Some("/data/3/attributes/resource/content/header/version")
    );
    assert_eq!(error.resource_id.as_ref().unwrap().as_str(), "drawing-main");
}

#[test]
fn inline_and_external_layouts_have_equal_semantic_content() {
    use ifccad::ifcdr::IfcdrEntityRef;
    let fixture = Fixture::new("shared-layout-representation");
    let external = load_directory_package(&fixture.0).unwrap();
    fixture.inline("resource");
    let inline = load_directory_package(&fixture.0).unwrap();
    let snapshot = |package: &ifccad::package::ValidatedPackage| {
        let drawing = package.drawings().next().unwrap();
        let representation = drawing.representation();
        let resource = representation.resource();
        let scopes: Vec<_> = drawing
            .layouts()
            .map(|layout| {
                assert_eq!(layout.representation().path(), representation.path());
                let scope = layout.scope();
                let entities: Vec<_> = resource
                    .entities(scope.id())
                    .map(|entity| match entity {
                        IfcdrEntityRef::Line(line) => format!("{:?}", line),
                        IfcdrEntityRef::Polyline(poly) => format!(
                            "{:?}",
                            (
                                poly.entity_id(),
                                poly.scope_id(),
                                poly.layer_id(),
                                poly.appearance_id(),
                                poly.visible(),
                                poly.closed(),
                                poly.points().collect::<Vec<_>>()
                            )
                        ),
                    })
                    .collect();
                format!(
                    "{:?}",
                    (
                        scope.id(),
                        scope.name(),
                        scope.base(),
                        scope.kind(),
                        scope.flags(),
                        entities
                    )
                )
            })
            .collect();
        let layers: Vec<_> = representation
            .layers()
            .map(|layer| {
                format!(
                    "{:?}",
                    (
                        layer.id(),
                        layer.path(),
                        layer.name(),
                        layer.visible(),
                        layer.appearance().map(|a| (
                            a.color(),
                            a.opacity(),
                            a.line_pattern(),
                            a.line_weight()
                        ))
                    )
                )
            })
            .collect();
        format!(
            "{:?}",
            (
                resource.resource_id(),
                resource.unit(),
                resource.bounds(),
                scopes,
                layers
            )
        )
    };
    assert_eq!(
        snapshot(external.validated_package().unwrap()),
        snapshot(
            inline
                .validated_package()
                .expect("inline shared-layout proof")
        )
    );
}

#[test]
fn inline_preservation_errors_keep_their_origin() {
    let fixture = Fixture::new("source-archive");
    fixture.inline("preservation");
    let mut graph = fixture.graph();
    let index = graph["data"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["type"] == "openaec:PreservationRepresentation")
        .unwrap();
    graph["data"][index]["attributes"]["preservation"]["content"]["header"]["version"] =
        "wrong".into();
    fixture.save(&graph);
    let outcome = load_directory_package(&fixture.0).unwrap();
    assert!(outcome.validated_package().is_none());
    let expected = format!("/data/{index}/attributes/preservation/content/header/version");
    assert!(outcome
        .report()
        .iter()
        .any(|d| d.code == "IFCCAD_PACKAGE_SCHEMA_INVALID"
            && d.resource_uri.as_deref() == Some("package.ifcx.json")
            && d.location.as_deref() == Some(expected.as_str())));
}

#[test]
fn duplicate_inline_identity_is_rejected_even_for_equal_bodies() {
    let fixture = Fixture::new("minimal-no-preservation");
    fixture.inline("resource");
    let mut graph = fixture.graph();
    let mut second = graph["data"][3].clone();
    second["path"] = "second-representation".into();
    graph["data"].as_array_mut().unwrap().push(second);
    fixture.save(&graph);
    let outcome = load_directory_package(&fixture.0).unwrap();
    assert!(outcome.validated_package().is_none());
    assert!(outcome
        .report()
        .iter()
        .any(|d| d.code == "IFCCAD_PACKAGE_RESOURCE_ID_DUPLICATE"));
}

#[test]
fn duplicate_ids_do_not_misdirect_body_error_origins() {
    let fixture = Fixture::new("minimal-no-preservation");
    fixture.inline("resource");
    let mut graph = fixture.graph();
    let mut second = graph["data"][3].clone();
    second["path"] = "second-representation".into();
    second["attributes"]["resource"]["content"]["header"]["version"] = "unsupported".into();
    let index = graph["data"].as_array().unwrap().len();
    graph["data"].as_array_mut().unwrap().push(second);
    fixture.save(&graph);
    let outcome = load_directory_package(&fixture.0).unwrap();
    let error = outcome
        .report()
        .iter()
        .find(|d| d.code == "IFCCAD_IFCDR_VERSION_UNSUPPORTED")
        .unwrap();
    assert_eq!(
        error.location.as_deref(),
        Some(format!("/data/{index}/attributes/resource/content/header/version").as_str())
    );
}

#[test]
fn ambiguous_sources_do_not_attempt_to_load_the_external_alternative() {
    let fixture = Fixture::new("minimal-no-preservation");
    fixture.inline("resource");
    let mut graph = fixture.graph();
    graph["data"][3]["attributes"]["resource"]["uri"] = "missing.json".into();
    fixture.save(&graph);
    let outcome = load_directory_package(&fixture.0).unwrap();
    assert!(outcome.validated_package().is_none());
    assert!(outcome
        .report()
        .iter()
        .any(|d| d.code == "IFCCAD_PACKAGE_SCHEMA_INVALID"));
    assert!(!outcome
        .report()
        .iter()
        .any(|d| d.code == "IFCCAD_PACKAGE_RESOURCE_MISSING"));
}

#[test]
fn writer_selects_inline_storage_without_changing_default_external_output() {
    use ifccad::package::{DrawingOptions, DrawingResourceStorage, PackageBuilder, PackageOptions};
    use ifccad::{PackageId, ResourceId};
    let encode = |storage| {
        let mut builder = PackageBuilder::new(PackageOptions {
            package_id: PackageId::new("package").unwrap(),
            data_version: "1".into(),
            author: "Test".into(),
            timestamp: "2026-09-09T00:00:00Z".into(),
        })
        .unwrap();
        let mut drawing = builder
            .add_drawing(DrawingOptions {
                model_layout_name: "Model".into(),
                representation_resource_id: ResourceId::new("geometry:arbitrary/id").unwrap(),
                length_unit: ifccad::ifcdr::IfcdrLengthUnit::Metre,
            })
            .unwrap();
        if let Some(storage) = storage {
            drawing.set_resource_storage(storage);
        }
        builder.finish().unwrap()
    };
    let inline = encode(Some(DrawingResourceStorage::Inline));
    let repeated = encode(Some(DrawingResourceStorage::Inline));
    assert_eq!(
        inline.files().collect::<Vec<_>>(),
        repeated.files().collect::<Vec<_>>()
    );
    assert_eq!(inline.files().len(), 1);
    let graph: Value = serde_json::from_slice(inline.file("package.ifcx.json").unwrap()).unwrap();
    let descriptor = &graph["data"][3]["attributes"]["resource"];
    assert!(descriptor["content"].is_object());
    assert!(descriptor.get("uri").is_none());
    assert!(descriptor.get("checksum").is_none());
    let external = encode(None);
    assert_eq!(external.files().len(), 2);
    assert_eq!(
        external.files().collect::<Vec<_>>(),
        encode(Some(DrawingResourceStorage::External))
            .files()
            .collect::<Vec<_>>()
    );
    let external_body: Value =
        serde_json::from_slice(external.file("resources/drawing.ifcdr.json").unwrap()).unwrap();
    assert_eq!(descriptor["content"], external_body);
    let fixture = Fixture::new("minimal-no-preservation");
    let destination = fixture.0.join("written");
    inline.write_directory(&destination).unwrap();
    assert!(load_directory_package(destination)
        .unwrap()
        .report()
        .is_valid());
}
