use cadcodec::{CadDocument, Handle};
use ifccad::package::PackageOptions;
use ifccad::PackageId;
use ifccad_convert::{
    cad_document_to_package, ExportEntityMapping, ExportError, ExportLossPolicy, ExportOptions,
    ExportOutcome, SourceStructureProblem,
};

#[test]
fn default_export_policy_allows_reported_loss() {
    assert_eq!(
        ExportOptions::default().loss_policy,
        ExportLossPolicy::Allow
    );
}

#[test]
fn crate_root_exposes_the_agreed_export_function_signature() {
    let _export: fn(
        &CadDocument,
        PackageOptions,
        ExportOptions,
    ) -> Result<ExportOutcome, ExportError> = cad_document_to_package;
}

#[test]
fn empty_export_mapping_has_an_ordered_read_only_api() {
    let mapping = ExportEntityMapping::default();
    let source = Handle::new(0x20);

    assert_eq!(mapping.len(), 0);
    assert!(mapping.is_empty());
    assert_eq!(mapping.target_entity_id(source), None);
    assert_eq!(mapping.iter().collect::<Vec<_>>(), []);
}

#[test]
fn structurally_invalid_documents_are_rejected_without_a_fabricated_package() {
    let mut document = CadDocument::new();
    document.header.model_space_block_handle = Handle::NULL;

    let error = cad_document_to_package(
        &document,
        PackageOptions {
            package_id: PackageId::new("invalid-source").unwrap(),
            data_version: "1".to_owned(),
            author: "Public API test".to_owned(),
            timestamp: "2026-09-04T10:00:00Z".to_owned(),
        },
        ExportOptions::default(),
    )
    .err()
    .expect("a null model-space handle is structurally invalid");

    assert!(matches!(
        error,
        ExportError::InvalidSourceStructure { problems }
            if problems == [SourceStructureProblem::ModelSpaceBlockMissing]
    ));
}
