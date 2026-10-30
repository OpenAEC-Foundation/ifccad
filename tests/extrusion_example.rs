#[path = "../examples/support/extrusion_gallery.rs"]
mod extrusion;

#[test]
fn review_camera_keeps_the_accepted_extrusion_geometry() {
    let document = extrusion::explorer_document();
    assert_eq!(
        document.model.entities,
        extrusion::document().model.entities
    );
    let encoded = ifccad::encode_ifccad_document(&document).unwrap();
    ifccad::load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
}
