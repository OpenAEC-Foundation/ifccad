use ifccad_convert::{ifccad_document_to_cad_document, IfccadToCadOptions};
#[path = "../../../tests/support/ifccad_hatch.rs"]
mod fixture;
#[test]
fn native_ifccad_hatch_materializes_contours_and_association() {
    let d = fixture::drawing();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ifccad_document_to_cad_document(&d, IfccadToCadOptions::default())
    }))
    .expect("recognized native Hatch must not panic in conversion");
    let out = outcome.unwrap();
    assert_eq!(
        out.document()
            .entities()
            .filter(|e| matches!(e, opencadcodec::EntityType::Hatch(_)))
            .count(),
        1
    );
    assert!(!out.geometry_assessment().is_complete());
}
