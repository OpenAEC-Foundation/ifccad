mod common;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType, Vector3};
fn metadata() -> IfccadTargetMetadata {
    IfccadTargetMetadata {
        drawing_id: 1,
        header: IfccadHeader {
            id: "spline".into(),
            data_version: "1".into(),
            author: "test".into(),
            timestamp: "2026-10-07T00:00:00Z".into(),
        },
    }
}
#[test]
fn all_spline_parameters_capture_before_native_geometry_admission() {
    let mut cad = CadDocument::new();
    let mut spline = opencadcodec::entities::Spline::new();
    spline.common.line_weight = opencadcodec::LineWeight::Default;
    spline.degree = -4;
    spline.knots = vec![f64::from_bits(0x7ff8000000000042), -0.];
    spline.fit_points = vec![Vector3::new(1., 2., 3.)];
    spline.knot_parameterization = 2;
    cad.add_entity(EntityType::Spline(spline.clone())).unwrap();
    let native = cad_document_to_ifccad_document(
        &cad,
        metadata(),
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(native.document().model.entities.len(), 1);
    let entity = native.document().model.entities[0].as_opaque().unwrap();
    assert!(entity.appearance.is_none());
    assert!(native.document().model.bounds.is_none());
    assert!(!native.geometry_assessment().is_complete());
    let p = native.document().preservation.as_ref().unwrap();
    let decoded = cad_preservation::decode_spline_snapshot(&p.records[0].payload.bytes)
        .unwrap()
        .to_source();
    assert_eq!(decoded.degree, -4);
    assert_eq!(decoded.knots[0].to_bits(), 0x7ff8000000000042);
    assert_eq!(decoded.knots[1].to_bits(), 0x8000000000000000);
    assert_eq!(decoded.knot_parameterization, 2);
    let bytes = encode_ifccad_document(native.document()).unwrap();
    assert_eq!(
        load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        native.document()
    );
    assert!(native
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.result == IfccadPreservationResult::CapturedTyped));
    let disabled = cad_document_to_ifccad_document(&cad, metadata(), Default::default()).unwrap();
    assert!(disabled.document().model.entities.is_empty());
}

#[test]
fn exact_native_common_is_optional_without_fabricated_fallbacks() {
    use opencadcodec::{Color, Handle, LineWeight};
    for (color, weight, scale, expected) in [
        (Color::from_rgb(1, 2, 3), LineWeight::ByLayer, 2., true),
        (Color::Index(3), LineWeight::ByLayer, 1., false),
        (Color::None, LineWeight::ByLayer, 1., false),
        (Color::ByLayer, LineWeight::Default, 1., false),
        (Color::ByBlock, LineWeight::ByBlock, -0., false),
    ] {
        let mut cad = CadDocument::new();
        let mut s = opencadcodec::entities::Spline::new();
        s.common.color = color;
        s.common.line_weight = weight;
        s.common.linetype_scale = scale;
        cad.add_entity(EntityType::Spline(s)).unwrap();
        let result = cad_document_to_ifccad_document(
            &cad,
            metadata(),
            CadToIfccadOptions {
                preservation: IfccadPreservationCapture::SupportedTyped,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            result.document().model.entities[0]
                .as_opaque()
                .unwrap()
                .appearance
                .is_some(),
            expected
        );
        encode_ifccad_document(result.document()).unwrap();
    }
    let mut cad = CadDocument::new();
    let mut s = opencadcodec::entities::Spline::new();
    s.common.layer_handle = Some(Handle::new(0xffff));
    cad.add_entity(EntityType::Spline(s)).unwrap();
    let result = cad_document_to_ifccad_document(
        &cad,
        metadata(),
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        result.document().model.entities[0]
            .as_opaque()
            .unwrap()
            .layer_id,
        None
    );
    let restored = ifccad_document_to_cad_document(result.document(), Default::default()).unwrap();
    assert!(!restored
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
}

#[test]
fn unsupported_valid_owners_archive_typed_source_but_membership_conflicts_remain_fatal() {
    let mut cad = ifccad_document_to_cad_document(&common::nested([0.; 3]), Default::default())
        .unwrap()
        .into_document();
    let owner = cad.block_records.get("Unused").unwrap().handle;
    let mut s = opencadcodec::entities::Spline::new();
    s.common.owner_handle = owner;
    s.fit_points = vec![Vector3::new(1., 2., 3.)];
    let handle = cad.add_entity(EntityType::Spline(s)).unwrap();
    cad.block_records.get_mut("Unused").unwrap().flags.is_xref = true;
    let options = CadToIfccadOptions {
        preservation: IfccadPreservationCapture::SupportedTyped,
        ..Default::default()
    };
    let captured = cad_document_to_ifccad_document(&cad, metadata(), options).unwrap();
    assert!(captured
        .document()
        .blocks
        .iter()
        .all(|b| b.name != "Unused"));
    let p = captured.document().preservation.as_ref().unwrap();
    assert_eq!(p.records.len(), 1);
    assert_eq!(p.records[0].subject, None);
    assert_eq!(
        cad_preservation::decode_spline_snapshot(&p.records[0].payload.bytes)
            .unwrap()
            .to_source()
            .fit_points,
        vec![Vector3::new(1., 2., 3.)]
    );
    encode_ifccad_document(captured.document()).unwrap();
    cad.get_entity_mut(handle)
        .unwrap()
        .common_mut()
        .owner_handle = cad.header.model_space_block_handle;
    assert!(cad_document_to_ifccad_document(&cad, metadata(), options).is_err());
}
