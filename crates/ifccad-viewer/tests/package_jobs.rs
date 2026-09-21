use ifccad_viewer::inspect_package;
use std::path::PathBuf;
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../conformance/next/packages")
        .join(name)
}
#[test]
fn production_assessment_and_raw_documents_survive() {
    let r = inspect_package(&fixture("valid/unrepresented-packed"));
    assert_eq!(r["validation"]["strictAvailable"], true);
    assert_eq!(
        r["validation"]["report"]["assessment"]["completeness"],
        "incomplete"
    );
    assert!(r["presentation"]["documents"].as_array().unwrap().len() >= 3);
    let original =
        std::fs::read_to_string(fixture("valid/unrepresented-packed").join("package.ifcx.json"))
            .unwrap();
    assert_eq!(r["presentation"]["documents"][0]["text"], original);
}
#[test]
fn invalid_and_unsupported_keep_reports_without_graph() {
    for name in [
        "invalid/package-missing-resource",
        "invalid/unsupported-ifcdr-version",
    ] {
        let r = inspect_package(&fixture(name));
        assert_eq!(r["validation"]["strictAvailable"], false, "{r}");
        assert!(r["presentation"].is_null());
        assert!(!r["validation"]["report"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}
#[test]
fn inline_and_multiple_drawings_load() {
    for name in ["valid/inline-both", "valid/multi-drawing-projections"] {
        let r = inspect_package(&fixture(name));
        assert_eq!(r["validation"]["strictAvailable"], true, "{r}");
    }
}

#[test]
fn invalid_known_content_is_not_misreported_as_only_unsupported() {
    let r = inspect_package(&fixture("invalid/schema-wrong-scalar"));
    assert_eq!(
        r["validation"]["report"]["assessment"]["validity"],
        "invalid"
    );
    assert!(r["presentation"].is_null());
}
