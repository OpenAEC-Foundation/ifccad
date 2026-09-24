use ifccad::conformance::{
    bundled_conformance_root, load_conformance_manifest, ConformanceCategory,
    ConformanceOperationName,
};
use ifccad::ifcdr::{FrontClipMode, IfcdrEntityRef, ProjectionMode};
use ifccad::package::{load_directory_package, PackageDiagnosticSeverity};
use ifccad::package::{PlotArea, PlotRect, PlotScale};
use serde_json::{json, Value};
use std::collections::BTreeSet;

// Four cases require semantic IFCPR validation; the projection case requires
// interpretation of the conformance package.json resource index.
const DEFERRED_VALIDATE_PACKAGE_CASES: [&str; 5] = [
    "invalid.blob-digest-mismatch",
    "invalid.payload-range-invalid",
    "invalid.record-reference-missing",
    "invalid.dependency-cycle",
    "invalid.projection-resource-missing",
];

#[test]
fn supported_bundled_validate_package_cases_match_their_diagnostic_contract() {
    let root = bundled_conformance_root();
    let manifest = load_conformance_manifest(&root).expect("load bundled conformance manifest");
    let mut mismatches = Vec::new();
    let mut deferred_cases_found = BTreeSet::new();

    for case in &manifest.cases {
        for operation in &case.operations {
            if operation.name != ConformanceOperationName::ValidatePackage {
                continue;
            }
            if DEFERRED_VALIDATE_PACKAGE_CASES.contains(&case.case_id.as_str()) {
                deferred_cases_found.insert(case.case_id.as_str());
                continue;
            }

            let entrypoint = root.join(&case.entrypoint);
            let package_root = entrypoint.parent().expect("package entrypoint parent");
            let outcome = load_directory_package(package_root)
                .unwrap_or_else(|error| panic!("{} could not be inspected: {error}", case.case_id));
            match case.category {
                ConformanceCategory::Valid => assert!(
                    outcome.validated_package().is_some(),
                    "{} did not expose a strict view: {:#?}",
                    case.case_id,
                    outcome.report()
                ),
                ConformanceCategory::Invalid => assert!(
                    outcome.validated_package().is_none(),
                    "{} exposed a strict view",
                    case.case_id
                ),
                _ => unreachable!("validatePackage has an unexpected category"),
            }
            let actual = outcome
                .report()
                .iter()
                .enumerate()
                .map(|(index, diagnostic)| {
                    let mut value = json!({
                        "code": diagnostic.code,
                        "severity": severity_name(diagnostic.severity),
                    });
                    if operation
                        .expected
                        .diagnostics
                        .get(index)
                        .and_then(|d| d.get("category"))
                        .is_some()
                    {
                        value["category"] = serde_json::to_value(diagnostic.category).unwrap();
                    }
                    value
                })
                .collect::<Vec<Value>>();

            if actual != operation.expected.diagnostics {
                mismatches.push(format!(
                    "{}: expected {:?}, got {actual:?}",
                    case.case_id, operation.expected.diagnostics
                ));
            }
            if let Some(expected) = &operation.expected.package_assessment {
                assert_eq!(
                    serde_json::to_value(outcome.report().assessment()).unwrap(),
                    serde_json::to_value(expected).unwrap(),
                    "{} assessment",
                    case.case_id
                );
            }
        }
    }

    assert!(
        mismatches.is_empty(),
        "validatePackage manifest mismatches:\n{}",
        mismatches.join("\n")
    );
    assert_eq!(
        deferred_cases_found,
        DEFERRED_VALIDATE_PACKAGE_CASES.into_iter().collect(),
        "the explicit deferral list must identify existing validatePackage cases"
    );
}

fn severity_name(severity: PackageDiagnosticSeverity) -> &'static str {
    match severity {
        PackageDiagnosticSeverity::Error => "error",
        PackageDiagnosticSeverity::Warning => "warning",
        PackageDiagnosticSeverity::Info => "info",
    }
}

#[test]
fn candidate_plot_modes_have_distinct_strict_reader_values() {
    let root = bundled_conformance_root().join("packages/valid");
    for (name, layout_index, expected_area) in [
        ("plot-extents", 1, PlotArea::Extents),
        ("plot-model-limits", 0, PlotArea::Limits),
        (
            "plot-window-fit",
            2,
            PlotArea::Window(PlotRect {
                min_x: 1.0,
                min_y: 2.0,
                max_x: 5.0,
                max_y: 8.0,
            }),
        ),
        ("two-paper-layouts", 1, PlotArea::Layout),
    ] {
        let outcome = load_directory_package(root.join(name)).expect(name);
        let package = outcome.validated_package().expect(name);
        let drawing = package.drawings().next().expect(name);
        let layout = drawing.layouts().nth(layout_index).expect(name);
        let plot = layout.settings().plot_settings.expect(name);
        assert_eq!(plot.area, expected_area, "{name}");
        if name == "plot-window-fit" {
            assert_eq!(plot.mapping.scale, PlotScale::FitToArea);
        }
        if name == "plot-model-limits" {
            assert_eq!(layout.settings().limits.unwrap().max_x, 10.0);
        }
    }
}

#[test]
fn candidate_viewport_variants_preserve_child_and_optional_semantics() {
    let root = bundled_conformance_root().join("packages/valid");
    for name in [
        "viewport-active-rectangular-clip",
        "viewport-disabled-boundary-reference",
        "viewport-perspective-at-camera",
        "viewport-visible-default",
        "viewport-omitted-lens",
    ] {
        let outcome = load_directory_package(root.join(name)).expect(name);
        let package = outcome.validated_package().expect(name);
        let drawing = package.drawings().next().expect(name);
        let paper = drawing.layouts().nth(1).expect(name);
        let resource = paper.representation().resource();
        let entities = resource.entities(paper.scope().id()).collect::<Vec<_>>();
        let IfcdrEntityRef::Viewport(first) = &entities[0] else {
            panic!("{name}: first paper entity is not a viewport")
        };
        let IfcdrEntityRef::Viewport(second) = &entities[2] else {
            panic!("{name}: third paper entity is not a viewport")
        };
        assert_eq!(first.layer_overrides().len(), 2, "{name}");
        assert_eq!(second.layer_overrides().len(), 0, "{name}");
        match name {
            "viewport-active-rectangular-clip" => {
                assert!(second.paper_clip().enabled);
                assert_eq!(second.paper_clip().boundary_entity_id, Some(2));
                assert!(matches!(&entities[1], IfcdrEntityRef::PlanarPolyline(p) if p.closed()));
            }
            "viewport-disabled-boundary-reference" => {
                assert!(!second.paper_clip().enabled);
                assert_eq!(second.paper_clip().boundary_entity_id, Some(2));
                assert!(matches!(&entities[1], IfcdrEntityRef::PlanarPolyline(p) if !p.closed()));
            }
            "viewport-perspective-at-camera" => {
                assert_eq!(first.view().projection, ProjectionMode::Perspective);
                assert_eq!(first.view().front_clip.mode, FrontClipMode::AtCamera);
                assert_eq!(first.view().back_clip.distance, Some(50.0));
            }
            "viewport-visible-default" => {
                assert!(first.visible());
                assert!(second.visible());
                assert_eq!(first.plot_shading_override(), None);
            }
            "viewport-omitted-lens" => {
                assert_eq!(first.view().lens_length, None);
                assert_eq!(second.view().lens_length, Some(35.0));
            }
            _ => unreachable!(),
        }
    }
}
