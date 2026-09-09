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

#[test]
fn inline_drawing_import_preserves_the_same_cad_content() {
    let import = |name| {
        let loaded =
            load_directory_package(bundled_conformance_root().join("packages/valid").join(name))
                .unwrap();
        let outcome = drawing_to_cad_document(
            loaded
                .validated_package()
                .unwrap()
                .drawings()
                .next()
                .unwrap(),
        )
        .unwrap();
        outcome
    };
    let external = import("minimal-no-preservation");
    let inline = import("inline-drawing");
    assert_eq!(inline.document().entities().count(), 4);
    assert_eq!(
        format!("{:?}", inline.diagnostics()),
        format!("{:?}", external.diagnostics())
    );
    let project = |document: &ifccad_convert::cadcodec::CadDocument| {
        document
            .entities()
            .map(|entity| {
                use ifccad_convert::cadcodec::EntityType;
                let geometry = match entity {
                    EntityType::Line(line) => format!("{:?}", (line.start, line.end)),
                    EntityType::LwPolyline(poly) => format!(
                        "{:?}",
                        (
                            poly.is_closed,
                            poly.vertices.iter().map(|v| v.location).collect::<Vec<_>>()
                        )
                    ),
                    _ => panic!("unexpected converted entity"),
                };
                let common = entity.common();
                format!(
                    "{:?}",
                    (
                        geometry,
                        &common.layer,
                        common.invisible,
                        &common.color,
                        &common.color_name,
                        &common.linetype,
                        common.line_weight,
                        common.transparency
                    )
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(project(inline.document()), project(external.document()));
}
