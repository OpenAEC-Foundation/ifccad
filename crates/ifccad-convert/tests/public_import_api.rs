use ifccad::conformance::bundled_conformance_root;
use ifccad::package::load_directory_package;
use ifccad_convert::{
    drawing_to_cad_document, ImportDiagnostic, ImportEntityMapping, ImportError, ImportOutcome,
};

fn assert_public_types(
    outcome: &ImportOutcome,
    diagnostics: &[ImportDiagnostic],
    mapping: &ImportEntityMapping,
) {
    let _: &ifccad_convert::cadcodec::CadDocument = outcome.document();
    let _ = (diagnostics, mapping);
}

fn assert_error_type(_: Option<&ImportError>) {}

#[test]
fn flat_import_api_converts_a_validated_drawing() {
    let root = bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation");
    let loaded = load_directory_package(root).unwrap();
    let package = loaded.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    let outcome = drawing_to_cad_document(drawing).unwrap();

    assert_public_types(&outcome, outcome.diagnostics(), outcome.entity_mapping());
    assert_error_type(None);
    assert_eq!(outcome.document().entities().count(), 4);
}
