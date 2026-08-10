use ifccad::conformance::{
    parse_conformance_manifest, ConformanceCategory, ConformanceError, ConformanceOperationName,
};
use std::path::Path;

fn manifest(case_body: &str) -> String {
    format!(r#"{{"suiteVersion":"1.0.0","cases":[{case_body}]}}"#)
}

fn valid_case() -> &'static str {
    r#"{"caseId":"valid.one","category":"valid","description":"valid case","entrypoint":"packages/valid/one/package.json","operations":[{"name":"validatePackage","expected":{"diagnostics":[]}}]}"#
}

#[test]
fn parses_known_manifest_vocabulary() {
    let parsed = parse_conformance_manifest(&manifest(valid_case()), Path::new("manifest.json"))
        .expect("valid manifest");

    assert_eq!(parsed.suite_version, "1.0.0");
    assert_eq!(parsed.cases.len(), 1);
    assert_eq!(parsed.cases[0].category, ConformanceCategory::Valid);
    assert_eq!(
        parsed.cases[0].operations[0].name,
        ConformanceOperationName::ValidatePackage,
    );
}

#[test]
fn rejects_unsupported_version() {
    let source = format!(r#"{{"suiteVersion":"2.0.0","cases":[{}]}}"#, valid_case());
    assert!(matches!(
        parse_conformance_manifest(&source, Path::new("manifest.json")),
        Err(ConformanceError::UnsupportedSuiteVersion { found }) if found == "2.0.0"
    ));
}

#[test]
fn rejects_duplicate_case_ids() {
    let source = format!(
        r#"{{"suiteVersion":"1.0.0","cases":[{},{}]}}"#,
        valid_case(),
        valid_case(),
    );
    assert!(matches!(
        parse_conformance_manifest(&source, Path::new("manifest.json")),
        Err(ConformanceError::DuplicateCaseId { case_id }) if case_id == "valid.one"
    ));
}

#[test]
fn rejects_unknown_category_operation_and_fields() {
    let unknown_category = manifest(&valid_case().replace("\"valid\"", "\"other\""));
    assert!(matches!(
        parse_conformance_manifest(&unknown_category, Path::new("manifest.json")),
        Err(ConformanceError::UnknownCategory { .. })
    ));

    let unknown_operation = manifest(&valid_case().replace("validatePackage", "unknownOperation"));
    assert!(matches!(
        parse_conformance_manifest(&unknown_operation, Path::new("manifest.json")),
        Err(ConformanceError::UnknownOperation { .. })
    ));

    let unknown_field = manifest(&valid_case().replace(
        "\"description\":\"valid case\"",
        "\"description\":\"valid case\",\"typo\":true",
    ));
    assert!(matches!(
        parse_conformance_manifest(&unknown_field, Path::new("manifest.json")),
        Err(ConformanceError::MalformedJson { .. })
    ));
}

#[test]
fn rejects_empty_operation_list() {
    let source = manifest(&valid_case().replace(
        r#"[{"name":"validatePackage","expected":{"diagnostics":[]}}]"#,
        "[]",
    ));
    assert!(matches!(
        parse_conformance_manifest(&source, Path::new("manifest.json")),
        Err(ConformanceError::EmptyOperations { case_id }) if case_id == "valid.one"
    ));
}
