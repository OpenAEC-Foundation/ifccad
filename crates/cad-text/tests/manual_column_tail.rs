use cad_text::*;
use ocdraw::{geometry_kernel::CoordinateFrame3, text::*};
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, MText};
use std::io::Cursor;

fn source(heights: Vec<f64>) -> MText {
    let mut source = MText::new();
    source.value = "First\\NRemaining".into();
    source.column_data.column_type = 2;
    source.column_data.column_count = heights.len() as i32;
    source.column_data.width = 40.;
    source.column_data.gutter = 5.;
    source.rectangle_width = heights.len() as f64 * 40. + (heights.len() - 1) as f64 * 5.;
    source.column_data.heights = heights;
    source
}

fn expected(prefix: bool) -> MTextColumns {
    let mut heights = Vec::new();
    if prefix {
        heights.push(MTextColumnHeight::Fixed { distance: 60. });
    }
    heights.push(MTextColumnHeight::Auto);
    MTextColumns::DynamicManualHeight {
        column_width: 40.,
        gutter: 5.,
        column_heights: heights,
        flow_reversed: false,
    }
}

fn export(columns: &MTextColumns) -> Result<PreparedCadText, CadTextError> {
    let parsed = parse_mtext("First\\PRemaining", Default::default()).unwrap();
    prepare_mtext_to_cad(&MTextCadInput {
        style_name: "Standard",
        placement: CoordinateFrame3::default(),
        rotation: 0.,
        backward: false,
        upside_down: false,
        height: 1.,
        attachment: MTextAttachment::TopLeft,
        flow: MTextFlow::Horizontal,
        wrap_width: None,
        columns: Some(columns),
        background: None,
        content: &parsed.content,
        character_format: &parsed.character_format,
        paragraph_format: &parsed.paragraph_format,
    })
}

#[test]
fn manual_final_column_is_auto_for_zero_negative_and_positive_stored_heights() {
    // Independent source evidence: Sample_AC1032 DWG has a negative last
    // height; its matching DXF has zero. ezdxf's layout ignores the final
    // stored height in all manual-column states, including positive values.
    for final_height in [0., -2.9442928603910867, 45.] {
        for prefix in [false, true] {
            let mut heights = if prefix { vec![60.] } else { vec![] };
            heights.push(final_height);
            let mut cad = source(heights);
            cad.extents_height = 999.;
            let parsed = prepare_mtext_from_cad(&cad).unwrap();
            assert_eq!(parsed.columns, Some(expected(prefix)));
            assert!(parsed.glyph_geometry_unassessed);
        }
    }
}

#[test]
fn invalid_prefix_count_and_nonfinite_final_heights_stay_structural_errors() {
    for heights in [
        vec![-3., 0.],
        vec![0., 0.],
        vec![f64::NAN, 0.],
        vec![60., f64::NAN],
        vec![60., f64::INFINITY],
        vec![60., f64::NEG_INFINITY],
    ] {
        assert!(matches!(
            prepare_mtext_from_cad(&source(heights)),
            Err(CadTextError::InvalidSource(_))
        ));
    }
    let mut cad = source(vec![60., 0.]);
    cad.column_data.column_count = 3;
    cad.rectangle_width = 130.;
    assert!(matches!(
        prepare_mtext_from_cad(&cad),
        Err(CadTextError::InvalidSource(_))
    ));
}

#[test]
fn native_fixed_final_height_is_not_silently_exported_as_an_auto_tail() {
    let mut columns = expected(true);
    if let MTextColumns::DynamicManualHeight { column_heights, .. } = &mut columns {
        column_heights[1] = MTextColumnHeight::Fixed { distance: 45. };
    }
    assert!(matches!(
        export(&columns),
        Err(CadTextError::Unsupported(_))
    ));
}

#[test]
fn native_auto_tail_survives_actual_dxf_and_ac1032_dwg() {
    for prefix in [false, true] {
        let columns = expected(prefix);
        let prepared = export(&columns).unwrap();
        let EntityType::MText(mtext) = &prepared.entity else {
            panic!()
        };
        assert_eq!(mtext.column_data.heights.last(), Some(&0.));
        let mut doc = CadDocument::new();
        doc.add_entity(prepared.entity).unwrap();
        for dwg in [false, true] {
            let loaded = if dwg {
                DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&doc).unwrap()))
                    .read()
                    .unwrap()
            } else {
                DxfReader::from_reader(Cursor::new(DxfWriter::new(&doc).write_to_vec().unwrap()))
                    .unwrap()
                    .read()
                    .unwrap()
            };
            let mtext = loaded
                .entities()
                .find_map(|e| {
                    if let EntityType::MText(t) = e {
                        Some(t)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(
                prepare_mtext_from_cad(mtext).unwrap().columns,
                Some(columns.clone()),
                "DWG={dwg}"
            );
        }
    }
}

#[test]
fn independently_reconstructed_r2018_embedded_column_fixture_has_an_auto_tail() {
    let bytes = include_bytes!("fixtures/mtext_manual_auto_tail_ac1032.dxf");
    let cad = DxfReader::from_reader(Cursor::new(bytes.as_slice()))
        .unwrap()
        .read()
        .unwrap();
    let mtext = cad
        .entities()
        .find_map(|e| {
            if let EntityType::MText(t) = e {
                Some(t)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(mtext.column_data.heights, vec![0.]);
    assert_eq!(
        prepare_mtext_from_cad(mtext).unwrap().columns,
        Some(MTextColumns::DynamicManualHeight {
            column_width: 15.74141424498748,
            gutter: 110.2231670381195,
            column_heights: vec![MTextColumnHeight::Auto],
            flow_reversed: false,
        })
    );
}
