mod common;
use ifccad::{dimension::DimensionAppearanceChoice, *};
use ifccad_convert::*;
use opencadcodec::{CadDocument, Handle};
use std::io::Cursor;

#[test]
fn byblock_dimension_pattern_survives_dwg_and_dxf_production_readers() {
    let mut cad = CadDocument::new();
    cad.text_styles.get_mut("Standard").unwrap().font_file = "txt.shx".into();
    let handle = cad.line_types.get("ByBlock").unwrap().handle;
    let style = cad.dim_styles.get_mut("Standard").unwrap();
    style.dimltex_handle = handle;
    style.dimltex1_handle = handle;
    style.dimltex2_handle = handle;
    let dimension = opencadcodec::entities::DimensionLinear::new(
        opencadcodec::Vector3::ZERO,
        opencadcodec::Vector3::new(3.0, 0.0, 0.0),
    );
    cad.add_entity(opencadcodec::EntityType::Dimension(Box::new(
        opencadcodec::entities::Dimension::Linear(dimension),
    )))
    .unwrap();
    let native =
        cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let target =
        ifccad_source_to_cad_document(native.validated_source(), Default::default()).unwrap();
    for dwg in [false, true] {
        let bytes = if dwg {
            opencadcodec::DwgWriter::write_to_vec(target.document()).unwrap()
        } else {
            opencadcodec::DxfWriter::new(target.document())
                .write_to_vec()
                .unwrap()
        };
        let read = if dwg {
            opencadcodec::DwgReader::from_stream(Cursor::new(bytes))
                .read()
                .unwrap()
        } else {
            opencadcodec::DxfReader::from_reader(Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap()
        };
        let returned =
            cad_document_to_encoded_ifccad(&read, common::metadata(), Default::default()).unwrap();
        let strict = load_ifccad_bytes(returned.encoded().bytes(), Default::default()).unwrap();
        validate_ifccad_document(strict.document()).unwrap();
        assert_eq!(strict.document().model.entities.len(), 1);
        let settings = &strict.document().dimension_styles[0].settings;
        assert_eq!(
            settings.dimension_line.appearance.line_pattern,
            DimensionAppearanceChoice::InheritEntity
        );
        assert_eq!(
            settings.extension_lines.first.appearance.line_pattern,
            DimensionAppearanceChoice::InheritEntity
        );
        assert_eq!(
            settings.extension_lines.second.appearance.line_pattern,
            DimensionAppearanceChoice::InheritEntity
        );
    }
}

#[test]
fn unused_dimension_styles_keep_canonical_pattern_selectors_through_strict_readback() {
    for name in ["ByBlock", "ByLayer", "Continuous"] {
        let mut cad = CadDocument::new();
        cad.text_styles.get_mut("Standard").unwrap().font_file = "txt.shx".into();
        let handle = cad.line_types.get(name).unwrap().handle;
        let style = cad.dim_styles.get_mut("Standard").unwrap();
        style.dimltex_handle = handle;
        style.dimltex1_handle = handle;
        style.dimltex2_handle = handle;
        let before = serde_json::to_value(&cad).unwrap();
        let out =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        let read = load_ifccad_bytes(out.encoded().bytes(), Default::default()).unwrap();
        validate_ifccad_document(read.document()).unwrap();
        let settings = &read.document().dimension_styles[0].settings;
        let expected = match name {
            "ByBlock" => DimensionAppearanceChoice::InheritEntity,
            "ByLayer" => DimensionAppearanceChoice::ByLayer,
            _ => DimensionAppearanceChoice::Explicit(read.document().line_patterns[0].id),
        };
        assert_eq!(settings.dimension_line.appearance.line_pattern, expected);
        assert_eq!(
            settings.extension_lines.first.appearance.line_pattern,
            expected
        );
        assert_eq!(
            settings.extension_lines.second.appearance.line_pattern,
            expected
        );
        assert_eq!(serde_json::to_value(&cad).unwrap(), before);
        if name == "ByLayer" {
            let target = ifccad_source_to_cad_document(&read, Default::default()).unwrap();
            assert!(target
                .diagnostics()
                .iter()
                .any(|d| d.code == "dimension-metadata"
                    && d.location.ends_with("component/linePattern")
                    && d.is_loss()));
            assert!(matches!(
                ifccad_source_to_cad_document(
                    &read,
                    IfccadToCadOptions {
                        loss_policy: IfccadLossPolicy::Reject,
                        ..Default::default()
                    }
                ),
                Err(IfccadConversionError::Unsupported(_))
            ));
        }
    }
}

#[test]
fn dangling_and_wrong_kind_dimension_pattern_handles_remain_fatal() {
    for field in 0..3 {
        for wrong_kind in [false, true] {
            let mut cad = CadDocument::new();
            cad.text_styles.get_mut("Standard").unwrap().font_file = "txt.shx".into();
            let handle = if wrong_kind {
                cad.layers.get("0").unwrap().handle
            } else {
                Handle::new(99999)
            };
            let style = cad.dim_styles.get_mut("Standard").unwrap();
            match field {
                0 => style.dimltex_handle = handle,
                1 => style.dimltex1_handle = handle,
                _ => style.dimltex2_handle = handle,
            }
            for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
                assert!(cad_document_to_encoded_ifccad(
                    &cad,
                    common::metadata(),
                    CadToIfccadOptions {
                        loss_policy,
                        ..Default::default()
                    }
                )
                .is_err());
            }
        }
    }
}
