use ifccad::conformance::bundled_conformance_root;
use ifccad::package::{
    load_directory_package, AssessmentCompleteness, AssessmentGapReason, PackageDiagnosticCategory,
    PackageValidationReport, PackageValidity,
};

#[test]
fn completed_drawing_and_limited_preservation_are_distinguished() {
    let root = bundled_conformance_root().join("packages/valid");
    let drawing = load_directory_package(root.join("minimal-no-preservation")).unwrap();
    assert!(drawing.report().is_valid());
    assert!(drawing.validated_package().is_some());
    assert_eq!(
        drawing.report().assessment().validity(),
        PackageValidity::Valid
    );
    assert_eq!(
        drawing.report().assessment().completeness(),
        AssessmentCompleteness::Complete
    );
    let preservation = load_directory_package(root.join("source-archive")).unwrap();
    assert!(preservation.report().is_valid());
    assert!(preservation.validated_package().is_some());
    assert_eq!(
        preservation.report().assessment().validity(),
        PackageValidity::NotFullyAssessed
    );
    assert!(preservation
        .report()
        .assessment()
        .gaps()
        .iter()
        .any(|gap| gap.reason == AssessmentGapReason::PreservationSemanticsNotAssessed));
}

#[test]
fn default_report_is_not_completion_evidence() {
    let report = PackageValidationReport::default();
    assert!(report.is_valid());
    assert_eq!(
        report.assessment().validity(),
        PackageValidity::NotFullyAssessed
    );
    assert_eq!(
        serde_json::to_value(PackageDiagnosticCategory::ExecutionBlocked).unwrap(),
        "executionBlocked"
    );
}

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
            "ifccad-assessment-{}-{}",
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

fn assert_blocked(outcome: &ifccad::package::PackageLoadOutcome, validity: PackageValidity) {
    assert!(!outcome.report().is_valid());
    assert!(outcome.validated_package().is_none());
    assert_eq!(
        outcome.report().assessment().validity(),
        validity,
        "{:?}",
        outcome.report()
    );
}

#[test]
fn unknown_version_is_not_a_proven_contract_violation() {
    for inline in [false, true] {
        let fixture = Fixture::new("minimal-no-preservation");
        let mut graph = fixture.graph();
        for node in graph["data"].as_array_mut().unwrap() {
            if let Some(d) = node
                .get_mut("attributes")
                .and_then(|a| a.get_mut("resource"))
            {
                d["version"] = "99.0.0".into();
                let path = fixture.0.join(d["uri"].as_str().unwrap());
                let mut resource: Value =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                resource["header"]["version"] = "99.0.0".into();
                let bytes = serde_json::to_vec(&resource).unwrap();
                use sha2::{Digest, Sha256};
                d["checksum"] = format!("sha256:{:x}", Sha256::digest(&bytes)).into();
                fs::write(path, bytes).unwrap();
            }
        }
        fixture.save(&graph);
        if inline {
            fixture.inline("resource");
        }
        let outcome = load_directory_package(&fixture.0).unwrap();
        assert_blocked(&outcome, PackageValidity::NotFullyAssessed);
        assert!(outcome
            .report()
            .iter()
            .all(|d| d.category == PackageDiagnosticCategory::UnsupportedContent));
        assert!(outcome
            .report()
            .assessment()
            .gaps()
            .iter()
            .all(|g| g.reason == AssessmentGapReason::UnsupportedProfile));

        let mut graph = fixture.graph();
        let duplicate = graph["data"][0].clone();
        graph["data"].as_array_mut().unwrap().push(duplicate);
        fixture.save(&graph);
        let outcome = load_directory_package(&fixture.0).unwrap();
        assert_blocked(&outcome, PackageValidity::Invalid);
        assert_eq!(
            outcome.report().assessment().completeness(),
            AssessmentCompleteness::Incomplete
        );
        assert!(outcome
            .report()
            .iter()
            .any(|d| d.code == "IFCCAD_PACKAGE_NODE_PATH_DUPLICATE"
                && d.category == PackageDiagnosticCategory::ContractViolation));
    }
}

#[test]
fn preservation_gap_uses_its_actual_source_origin() {
    for inline in [false, true] {
        let fixture = Fixture::new("source-archive");
        if inline {
            fixture.inline("preservation");
        }
        let outcome = load_directory_package(&fixture.0).unwrap();
        let gaps = outcome.report().assessment().gaps();
        assert_eq!(gaps.len(), 1);
        let gap = &gaps[0];
        assert!(gap.resource_id.is_some());
        assert_eq!(
            gap.reason,
            AssessmentGapReason::PreservationSemanticsNotAssessed
        );
        if inline {
            assert_eq!(gap.resource_uri.as_deref(), Some("package.ifcx.json"));
            let pointer = gap.location.as_deref().unwrap();
            assert!(pointer.ends_with("/attributes/preservation/content"));
            assert!(fixture.graph().pointer(pointer).is_some());
        } else {
            assert_eq!(gap.resource_uri.as_deref(), Some("preservation.ifcpr.json"));
            assert_eq!(gap.location.as_deref(), Some(""));
        }
    }
}

#[test]
fn missing_input_and_malformed_input_have_different_evidence() {
    let fixture = Fixture::new("minimal-no-preservation");
    fs::remove_file(fixture.0.join("package.ifcx.json")).unwrap();
    let missing = load_directory_package(&fixture.0).unwrap();
    assert_blocked(&missing, PackageValidity::NotFullyAssessed);
    assert_eq!(
        missing.report().diagnostics()[0].category,
        PackageDiagnosticCategory::ExecutionBlocked
    );
    assert_eq!(
        missing.report().assessment().gaps()[0].reason,
        AssessmentGapReason::UnavailableInput
    );
    fs::write(fixture.0.join("package.ifcx.json"), b"{").unwrap();
    let malformed = load_directory_package(&fixture.0).unwrap();
    assert_blocked(&malformed, PackageValidity::Invalid);
    assert_eq!(
        malformed.report().assessment().completeness(),
        AssessmentCompleteness::Incomplete
    );
    assert_eq!(
        malformed.report().diagnostics()[0].category,
        PackageDiagnosticCategory::ContractViolation
    );
    let error = load_directory_package(fixture.0.join("package.ifcx.json"))
        .err()
        .unwrap();
    assert_eq!(
        error.category(),
        PackageDiagnosticCategory::ExecutionBlocked
    );
}

#[test]
fn unrelated_ifcx_extensions_do_not_create_assessment_gaps() {
    let fixture = Fixture::new("minimal-no-preservation");
    let mut graph = fixture.graph();
    graph["data"].as_array_mut().unwrap().push(serde_json::json!({"path": "extension", "type": "other:Something", "attributes": {"anything": 1}}));
    fixture.save(&graph);
    let outcome = load_directory_package(&fixture.0).unwrap();
    assert_eq!(
        outcome.report().assessment().validity(),
        PackageValidity::Valid
    );
}

#[test]
fn known_contract_error_can_have_complete_assessment() {
    let fixture = Fixture::new("minimal-no-preservation");
    let mut graph = fixture.graph();
    for node in graph["data"].as_array_mut().unwrap() {
        if let Some(d) = node
            .get_mut("attributes")
            .and_then(|a| a.get_mut("resource"))
        {
            d["checksum"] = format!("sha256:{}", "0".repeat(64)).into();
        }
    }
    fixture.save(&graph);
    let outcome = load_directory_package(&fixture.0).unwrap();
    assert_blocked(&outcome, PackageValidity::Invalid);
    assert_eq!(
        outcome.report().assessment().completeness(),
        AssessmentCompleteness::Complete
    );
    assert!(outcome.report().assessment().gaps().is_empty());
    assert_eq!(
        outcome.report().diagnostics()[0].category,
        PackageDiagnosticCategory::ContractViolation
    );
}

#[test]
fn repeated_preservation_declarations_do_not_duplicate_coverage_gaps() {
    let fixture = Fixture::new("source-archive");
    let mut graph = fixture.graph();
    let mut declaration = graph["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "openaec:PreservationRepresentation")
        .unwrap()
        .clone();
    declaration["path"] = "preservation-copy".into();
    graph["data"].as_array_mut().unwrap().push(declaration);
    fixture.save(&graph);
    let outcome = load_directory_package(&fixture.0).unwrap();
    assert!(outcome.report().is_valid(), "{:?}", outcome.report());
    assert_eq!(outcome.report().assessment().gaps().len(), 1);
    let first = serde_json::to_value(outcome.report()).unwrap();
    assert_eq!(
        first,
        serde_json::to_value(load_directory_package(&fixture.0).unwrap().report()).unwrap()
    );
}

#[test]
fn known_geometry_reference_and_unit_rules_remain_contract_errors() {
    for (pointer, value, code) in [
        (
            "/streams/lineStream/x2/0",
            serde_json::json!(1000000.0),
            "IFCCAD_IFCDR_BOUNDS_INVALID",
        ),
        (
            "/streams/lineStream/layerId/0",
            serde_json::json!(999),
            "IFCCAD_IFCDR_REFERENCE_MISSING",
        ),
        (
            "/header/unit",
            serde_json::json!("unknown"),
            "IFCCAD_IFCDR_UNIT_UNSUPPORTED",
        ),
    ] {
        let fixture = Fixture::new("minimal-no-preservation");
        fixture.inline("resource");
        let mut graph = fixture.graph();
        let body = graph["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find_map(|node| node.pointer_mut("/attributes/resource/content"))
            .unwrap();
        *body.pointer_mut(pointer).unwrap() = value;
        fixture.save(&graph);
        let outcome = load_directory_package(&fixture.0).unwrap();
        assert_blocked(&outcome, PackageValidity::Invalid);
        assert!(
            outcome
                .report()
                .iter()
                .any(|d| d.code == code
                    && d.category == PackageDiagnosticCategory::ContractViolation),
            "{:?}",
            outcome.report()
        );
        assert!(outcome.report().assessment().gaps().iter().any(|g| g.reason
            == AssessmentGapReason::ContentNotAssessable
            && g.location
                .as_deref()
                .is_some_and(|p| p.ends_with("/resource/content"))));
    }
}
