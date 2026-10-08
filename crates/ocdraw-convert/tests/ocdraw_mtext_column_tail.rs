use ocdraw::{ocdraw::*, text::*};
use ocdraw_convert::*;
use opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, LineWeight, MText,
};
use std::io::Cursor;

fn source(final_height: f64) -> CadDocument {
    let mut doc = CadDocument::new();
    let mut m = MText::new();
    m.value = "First\\PRemaining".into();
    m.common.line_weight = LineWeight::ByLayer;
    m.rectangle_width = 85.;
    m.column_data.column_type = 2;
    m.column_data.column_count = 2;
    m.column_data.width = 40.;
    m.column_data.gutter = 5.;
    m.column_data.heights = vec![60., final_height];
    doc.add_entity(EntityType::MText(m)).unwrap();
    doc
}

#[test]
fn manual_tail_is_native_auto_and_survives_strict_native_and_cad_readback() {
    for final_height in [0., -2.9442928603910867, 45.] {
        let native = cad_document_to_ocdraw_document(
            &source(final_height),
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            },
        )
        .unwrap();
        let expected = MTextColumns::DynamicManualHeight {
            column_width: 40.,
            gutter: 5.,
            column_heights: vec![
                MTextColumnHeight::Fixed { distance: 60. },
                MTextColumnHeight::Auto,
            ],
            flow_reversed: false,
        };
        assert_eq!(
            native.document().mtext_entities[0].columns,
            Some(expected.clone())
        );
        let encoded = encode_ocdraw_document(native.document()).unwrap();
        let strict = load_ocdraw_bytes(encoded.bytes()).unwrap();
        assert_eq!(strict.mtext_entities(), native.document().mtext_entities);
        let cad = ocdraw_document_to_cad_document(
            strict.document(),
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            },
        )
        .unwrap();
        for dwg in [false, true] {
            let loaded = if dwg {
                DwgReader::from_stream(Cursor::new(
                    DwgWriter::write_to_vec(cad.document()).unwrap(),
                ))
                .read()
                .unwrap()
            } else {
                DxfReader::from_reader(Cursor::new(
                    DxfWriter::new(cad.document()).write_to_vec().unwrap(),
                ))
                .unwrap()
                .read()
                .unwrap()
            };
            let native_back = cad_document_to_ocdraw_document(&loaded, Default::default()).unwrap();
            assert_eq!(
                native_back.document().mtext_entities[0].columns,
                Some(expected.clone()),
                "DWG={dwg}"
            );
            assert_eq!(
                native_back.document().mtext_entities[0].content,
                native.document().mtext_entities[0].content
            );
            let encoded = encode_ocdraw_document(native_back.document()).unwrap();
            load_ocdraw_bytes(encoded.bytes()).unwrap();
        }
    }
}

#[test]
fn native_fixed_final_cap_is_located_whole_entity_loss_in_cad() {
    let mut native = cad_document_to_ocdraw_document(&source(0.), Default::default())
        .unwrap()
        .into_document();
    let id = native.mtext_entities[0].id;
    if let Some(MTextColumns::DynamicManualHeight { column_heights, .. }) =
        &mut native.mtext_entities[0].columns
    {
        column_heights[1] = MTextColumnHeight::Fixed { distance: 45. };
    }
    recompute_ocdraw_document_bounds(&mut native).unwrap();
    let encoded = encode_ocdraw_document(&native).unwrap();
    let strict = load_ocdraw_bytes(encoded.bytes()).unwrap();
    let cad = ocdraw_document_to_cad_document(strict.document(), Default::default()).unwrap();
    assert!(!cad.entity_mapping().contains_key(&id));
    assert!(cad
        .document()
        .entities()
        .all(|e| !matches!(e, EntityType::MText(_))));
    assert!(cad
        .diagnostics()
        .iter()
        .any(|d| d.code == "TEXT_UNSUPPORTED" && d.location == format!("/entities/{id}")));
    assert!(matches!(
        ocdraw_document_to_cad_document(
            strict.document(),
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
}
