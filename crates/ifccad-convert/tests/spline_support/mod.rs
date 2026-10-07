use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, Color, EntityType, LineWeight, Transparency, Vector3};
pub fn captured() -> IfccadDocument {
    let mut cad = CadDocument::new();
    cad.header.insertion_units = 4;
    let layer = cad.layers.get_mut("0").unwrap();
    layer.color = Color::from_rgb(12, 34, 56);
    layer.line_weight = LineWeight::Value(25);
    layer.transparency = Transparency::Explicit(0);
    let mut s = opencadcodec::entities::Spline::new();
    s.degree = 3;
    s.knots = vec![0., 0., 0., 0., 1., 1., 1., 1.];
    s.control_points = vec![
        Vector3::new(0., 0., 0.),
        Vector3::new(2., 4., 0.),
        Vector3::new(4., 4., 0.),
        Vector3::new(6., 0., 0.),
    ];
    s.common.line_weight = LineWeight::ByLayer;
    cad.add_entity(EntityType::Spline(s)).unwrap();
    let metadata = IfccadTargetMetadata {
        drawing_id: 1,
        header: IfccadHeader {
            id: "spline".into(),
            data_version: "1".into(),
            author: "test".into(),
            timestamp: "2026-10-07T00:00:00Z".into(),
        },
    };
    let result = cad_document_to_ifccad_document(
        &cad,
        metadata,
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    let bytes = encode_ifccad_document(result.document()).unwrap();
    load_ifccad_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .into_document()
}

#[allow(dead_code)]
pub fn spline() -> opencadcodec::entities::Spline {
    let mut s = opencadcodec::entities::Spline::new();
    s.degree = 3;
    s.knots = vec![0., 0., 0., 0., 1., 1., 1., 1.];
    s.control_points = vec![
        Vector3::new(0., 0., 0.),
        Vector3::new(2., 4., 0.),
        Vector3::new(4., 4., 0.),
        Vector3::new(6., 0., 0.),
    ];
    s.common.line_weight = LineWeight::ByLayer;
    s
}
