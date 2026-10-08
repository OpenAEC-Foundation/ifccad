mod common;
use common::*;
use ifccad_convert::*;
use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Point, Vector3};
use serde_json::{json, Value};
use std::io::Cursor;

#[test]
fn cad_point_symbols_and_three_size_choices_survive_native_dxf_dwg() {
    for (glyph, glyph_bits) in [
        ("dot", 0),
        ("hidden", 1),
        ("plus", 2),
        ("cross", 3),
        ("shortLine", 4),
    ] {
        for (circle, square, bits) in [
            (false, false, 0),
            (true, false, 32),
            (false, true, 64),
            (true, true, 96),
        ] {
            for (size, expected_size) in [
                (0.0, json!({"kind":"defaultFivePercent"})),
                (-5.0, json!({"kind":"viewportPercent","value":5.0})),
                (2.5, json!({"kind":"absolute","value":2.5})),
            ] {
                let mut source = cad();
                source.header.point_display_mode = glyph_bits + bits;
                source.header.point_display_size = size;
                source
                    .add_entity(EntityType::Point(Point::at(Vector3::new(1., 2., 3.))))
                    .unwrap();
                let native =
                    cad_document_to_encoded_ifccad(&source, metadata(), Default::default())
                        .unwrap();
                let raw: Value = serde_json::from_slice(native.encoded().bytes()).unwrap();
                let drawing = raw["data"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find_map(|n| n["attributes"].get("ifccad::drawing"))
                    .unwrap();
                assert_eq!(
                    drawing["pointDisplay"],
                    json!({"form":{"glyph":glyph,"circle":circle,"square":square},"size":expected_size})
                );
                assert!(!native
                    .diagnostics()
                    .iter()
                    .any(|d| d.location.contains("point_display")
                        || d.location.contains("pointDisplay")));
                let target = ifccad_document_to_cad_document(
                    native.validated_source().document(),
                    Default::default(),
                )
                .unwrap();
                for dwg in [false, true] {
                    let restored = if dwg {
                        DwgReader::from_stream(Cursor::new(
                            DwgWriter::write_to_vec(target.document()).unwrap(),
                        ))
                        .read()
                        .unwrap()
                    } else {
                        DxfReader::from_reader(Cursor::new(
                            DxfWriter::new(target.document()).write_to_vec().unwrap(),
                        ))
                        .unwrap()
                        .read()
                        .unwrap()
                    };
                    assert_eq!(restored.header.point_display_mode, glyph_bits + bits);
                    assert_eq!(restored.header.point_display_size, size);
                    let restored_native =
                        cad_document_to_encoded_ifccad(&restored, metadata(), Default::default())
                            .unwrap();
                    assert_eq!(
                        restored_native
                            .validated_source()
                            .document()
                            .model
                            .entities
                            .len(),
                        1
                    );
                }
            }
        }
    }
}

#[test]
fn unsupported_source_point_settings_are_diagnosed_without_dropping_points() {
    for (mode, size) in [
        (5, 0.0),
        (8, 0.0),
        (128, 0.0),
        (-1, 0.0),
        (0, f64::NAN),
        (0, f64::INFINITY),
    ] {
        let mut source = cad();
        source.header.point_display_mode = mode;
        source.header.point_display_size = size;
        source
            .add_entity(EntityType::Point(Point::at(Vector3::new(1., 2., 3.))))
            .unwrap();
        let native =
            cad_document_to_encoded_ifccad(&source, metadata(), Default::default()).unwrap();
        assert_eq!(native.validated_source().document().model.entities.len(), 1);
        assert!(native
            .diagnostics()
            .iter()
            .any(|d| d.code == "point-display" && d.is_semantic_loss()));
        assert!(cad_document_to_encoded_ifccad(
            &source,
            metadata(),
            CadToIfccadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}
