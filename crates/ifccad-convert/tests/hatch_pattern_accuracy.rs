mod common;
#[path = "../../../tests/support/far_hatch_pattern.rs"]
mod far;
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
use std::io::Cursor;

#[test]
fn far_pattern_preserves_native_semantics_through_strict_readback_and_real_cad_files() {
    let mut source = CadDocument::new();
    source
        .add_entity(EntityType::Hatch(Box::new(far::hatch())))
        .unwrap();
    let encoded = cad_document_to_encoded_ifccad(
        &source,
        common::metadata(),
        CadToIfccadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .expect("classic editing state is represented and geometry is certified");
    assert!(!encoded
        .diagnostics()
        .iter()
        .any(|d| d.code == "hatch-editing-state"));
    assert!(encoded
        .diagnostics()
        .iter()
        .any(|d| d.code == "GeometryRoundedWithinTolerance"));
    let native = encoded.validated_source().document();
    let expected = native.model.entities[0].as_native().unwrap().kind.clone();
    let IfccadEntityKind::Hatch(hatch) = &expected else {
        panic!()
    };
    let classic = hatch.classic_settings.as_ref().unwrap();
    assert_eq!(classic.creation_type, Some(HatchCreationType::Custom));
    assert_eq!(classic.rotation, std::f64::consts::FRAC_PI_2);
    assert_eq!(classic.scale, 0.5);
    let bytes = encode_ifccad_cbor_document(native).unwrap();
    let loaded = load_ifccad_cbor_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(
        loaded.document().model.entities[0]
            .as_native()
            .unwrap()
            .kind,
        expected
    );
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let cad = ifccad_document_to_cad_document(
            loaded.document(),
            IfccadToCadOptions {
                loss_policy: policy,
                ..Default::default()
            },
        );
        let cad = cad.unwrap();
        assert!(!cad
            .diagnostics()
            .iter()
            .any(|d| d.code == "hatch-classic-settings-loss"));
        let opencadcodec::EntityType::Hatch(actual) = cad
            .document()
            .entities()
            .find(|e| matches!(e, opencadcodec::EntityType::Hatch(_)))
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(actual.pattern.lines, far::hatch().pattern.lines);
        assert_eq!(
            actual.stored_pattern_origin(),
            far::hatch().stored_pattern_origin()
        );
        for format in 0..3 {
            let returned = match format {
                0 | 1 => DxfReader::from_reader(Cursor::new(
                    if format == 0 {
                        DxfWriter::new(cad.document())
                    } else {
                        DxfWriter::new_binary(cad.document())
                    }
                    .write_to_vec()
                    .unwrap(),
                ))
                .unwrap()
                .read()
                .unwrap(),
                _ => DwgReader::from_stream(Cursor::new(
                    DwgWriter::write_to_vec(cad.document()).unwrap(),
                ))
                .read()
                .unwrap(),
            };
            let back =
                cad_document_to_encoded_ifccad(&returned, common::metadata(), Default::default())
                    .unwrap();
            let back = back.validated_source().document();
            validate_ifccad_document(back).unwrap();
            let IfccadEntityKind::Hatch(a) = &expected else {
                panic!()
            };
            let IfccadEntityKind::Hatch(b) = &back.model.entities[0].as_native().unwrap().kind
            else {
                panic!()
            };
            assert_eq!(a.loops, b.loops);
            assert_eq!(a.placement, b.placement);
            let ifccad::geometry_kernel::hatch::HatchFill::LinePattern(a) = &a.fill else {
                panic!()
            };
            let ifccad::geometry_kernel::hatch::HatchFill::LinePattern(b) = &b.fill else {
                panic!()
            };
            assert_eq!(
                (a.origin, a.rotation, a.scale, &a.name, &a.description),
                (b.origin, b.rotation, b.scale, &b.name, &b.description)
            );
            assert_eq!(a.families.len(), b.families.len());
            for (a, b) in a.families.iter().zip(&b.families) {
                assert_eq!(a.dashes, b.dashes);
                assert_eq!(a.angle, b.angle);
                for (x, y) in a.base_point.iter().zip(b.base_point) {
                    assert!((x - y).abs() < 1e-9);
                }
                for (x, y) in a.offset.iter().zip(b.offset) {
                    assert!((x - y).abs() < 1e-9);
                }
            }
        }
    }
}

#[test]
fn unproved_pattern_remains_a_hard_error_under_both_loss_policies() {
    let mut h = far::hatch();
    // Deliberately magnify unavoidable stored base/offset rounding. Allow is
    // permission for diagnosed semantic loss, not for an unproved geometry.
    if let opencadcodec::entities::hatch::BoundaryEdge::CircularArc(c) = &mut h.paths[0].edges[0] {
        c.center.x *= 1e6;
        c.center.y *= 1e6;
    }
    let mut source = CadDocument::new();
    source.add_entity(EntityType::Hatch(Box::new(h))).unwrap();
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(matches!(
            cad_document_to_encoded_ifccad(
                &source,
                common::metadata(),
                CadToIfccadOptions {
                    loss_policy: policy,
                    ..Default::default()
                }
            ),
            Err(IfccadConversionError::Geometry(_))
        ));
    }
}
