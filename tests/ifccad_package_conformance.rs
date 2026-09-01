use ifccad::conformance::{
    bundled_conformance_root, load_conformance_manifest, ConformanceOperationName,
};
use ifccad::package::{load_directory_package, PackageDiagnosticSeverity};
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
            let actual = outcome
                .report()
                .iter()
                .map(|diagnostic| {
                    json!({
                        "code": diagnostic.code,
                        "severity": severity_name(diagnostic.severity),
                    })
                })
                .collect::<Vec<Value>>();

            if actual != operation.expected.diagnostics {
                mismatches.push(format!(
                    "{}: expected {:?}, got {actual:?}",
                    case.case_id, operation.expected.diagnostics
                ));
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
