use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::entities::Spline;
use opencadcodec::{CadDocument, EntityType, Vector3};

#[test]
fn every_spline_variant_is_captured_without_geometry_admission() {
    let mut source = CadDocument::new();
    for degree in [-7, 0, 2, 3, 9] {
        let mut spline = Spline::new();
        spline.degree = degree;
        spline.knots = vec![f64::from_bits(0x7ff8000000000042), -0.0, f64::INFINITY];
        spline.fit_points = vec![Vector3::new(1., 2., 3.)];
        spline.flags.periodic = true;
        source.add_entity(EntityType::Spline(spline)).unwrap();
    }
    let captured = cad_document_to_ocdraw_document(
        &source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(captured.document().opaque_entities.len(), 5);
    let records = &captured.document().preservation.as_ref().unwrap().records;
    assert_eq!(records.len(), 5);
    let payload: serde_json::Value = serde_json::from_slice(&records[0].payload.bytes).unwrap();
    assert_eq!(payload["degree"], -7);
    assert_eq!(
        payload["knots"],
        serde_json::json!(["7ff8000000000042", "8000000000000000", "7ff0000000000000"])
    );
    assert_eq!(payload["flags"]["periodic"], true);
    let bytes = encode_ocdraw_document(captured.document()).unwrap();
    drop(source);
    let loaded = load_ocdraw_bytes(bytes.bytes()).unwrap();
    assert_eq!(loaded.preservation().unwrap().records, *records);
}
