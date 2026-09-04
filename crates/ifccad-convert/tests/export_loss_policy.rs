use cadcodec::{CadDocument, Circle, EntityType, Line};
use ifccad::package::PackageOptions;
use ifccad::PackageId;
use ifccad_convert::{cad_document_to_package, ExportError, ExportLossPolicy, ExportOptions};

fn package_options(label: &str) -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new(format!("policy-{label}")).unwrap(),
        data_version: "1".to_owned(),
        author: "Export policy test".to_owned(),
        timestamp: "2026-09-04T10:00:00Z".to_owned(),
    }
}

#[test]
fn reject_returns_the_complete_same_loss_list_as_allow_and_no_package() {
    let mut document = CadDocument::new();
    document
        .add_entity(EntityType::Circle(Circle::new()))
        .unwrap();
    let mut nonplanar = Line::from_coords(0.0, 0.0, 2.0, 1.0, 1.0, 3.0);
    nonplanar.thickness = 1.0;
    document.add_entity(EntityType::Line(nonplanar)).unwrap();

    let allowed = cad_document_to_package(
        &document,
        package_options("allow"),
        ExportOptions {
            loss_policy: ExportLossPolicy::Allow,
        },
    )
    .unwrap_or_else(|error| panic!("allow export failed: {error}"));
    assert_eq!(allowed.diagnostics().len(), 2);

    let error = cad_document_to_package(
        &document,
        package_options("reject"),
        ExportOptions {
            loss_policy: ExportLossPolicy::Reject,
        },
    )
    .err()
    .expect("reject policy must not return a partial package");
    assert!(matches!(
        error,
        ExportError::LossRejected { diagnostics }
            if diagnostics == allowed.diagnostics()
    ));
}
