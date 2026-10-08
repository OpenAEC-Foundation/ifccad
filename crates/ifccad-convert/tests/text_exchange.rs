mod common;
use common::metadata;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType, MText, Text, Vector3};
use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;

#[test]
fn simple_text_and_mtext_convert_with_styles_and_unassessed_glyphs() {
    let mut source = CadDocument::new();
    let mut text = Text::with_value("Native text 世界", Vector3::new(5., 6., 0.));
    text.height = 2.;
    source.add_entity(EntityType::Text(text)).unwrap();
    let mut mtext = MText::new();
    mtext.value = "First\\PSecond".into();
    mtext.height = 3.;
    mtext.insertion_point = Vector3::new(15., 16., 0.);
    source.add_entity(EntityType::MText(mtext)).unwrap();
    let native = cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
    assert_eq!(native.document().model.entities.len(), 2);
    assert!(!native.document().text_styles.is_empty());
    assert!(matches!(
        native.document().model.entities[0]
            .as_native()
            .unwrap()
            .kind,
        IfccadEntityKind::Text(_)
    ));
    assert!(matches!(
        native.document().model.entities[1]
            .as_native()
            .unwrap()
            .kind,
        IfccadEntityKind::MText(_)
    ));
    assert!(!native.geometry_assessment().is_complete());
    assert_eq!(native.text_assessment().entries().len(), 2);
    assert_eq!(
        native.text_assessment().entries()[0].numeric_coverage,
        IfccadTextNumericCoverage::ActiveTextAnchors
    );
    assert_eq!(
        native.text_assessment().entries()[1].numeric_coverage,
        IfccadTextNumericCoverage::MTextWcsAnchor
    );
    assert!(native
        .text_assessment()
        .entries()
        .iter()
        .all(|e| e.glyph_coverage == IfccadTextGlyphCoverage::Unassessed));
    let bytes = encode_ifccad_document(native.document()).unwrap();
    let loaded = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    let target = ifccad_document_to_cad_document(loaded.document(), Default::default()).unwrap();
    assert_eq!(target.document().entities().count(), 2);
    assert!(!target.geometry_assessment().is_complete());
    let back =
        cad_document_to_ifccad_document(target.document(), metadata(), Default::default()).unwrap();
    assert_eq!(back.document().model.entities.len(), 2);
}

#[test]
fn text_and_mtext_survive_actual_dxf_and_ac1032_dwg_readback() {
    let mut source = CadDocument::new();
    let mut text = Text::with_value("Hello 世界 \\ literal", Vector3::new(5., 6., 0.));
    text.height = 2.;
    text.rotation = 0.5;
    source.add_entity(EntityType::Text(text)).unwrap();
    let mut mtext = MText::new();
    mtext.value = "First\\PSecond\\P".into();
    mtext.height = 3.;
    mtext.insertion_point = Vector3::new(15., 16., 0.);
    source.add_entity(EntityType::MText(mtext)).unwrap();
    let original =
        cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
    let target = ifccad_document_to_cad_document(original.document(), Default::default()).unwrap();
    for dwg in [false, true] {
        let cad = if dwg {
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
        let native = cad_document_to_ifccad_document(&cad, metadata(), Default::default()).unwrap();
        assert_eq!(native.document().model.entities.len(), 2);
        for (expected, actual) in original
            .document()
            .model
            .entities
            .iter()
            .zip(&native.document().model.entities)
        {
            match (
                &expected.as_native().unwrap().kind,
                &actual.as_native().unwrap().kind,
            ) {
                (IfccadEntityKind::Text(a), IfccadEntityKind::Text(b)) => {
                    assert_eq!(a.content, b.content);
                    assert_eq!(a.layout, b.layout);
                    assert_eq!(a.placement, b.placement);
                    assert!((a.rotation - b.rotation).abs() <= f64::EPSILON);
                }
                (IfccadEntityKind::MText(a), IfccadEntityKind::MText(b)) => {
                    assert_eq!(a.content, b.content);
                    assert_eq!(a.placement, b.placement);
                    assert_eq!(a.height, b.height);
                }
                _ => panic!("text family changed"),
            }
        }
        let bytes = encode_ifccad_document(native.document()).unwrap();
        load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    }
}

#[test]
fn explicitly_disabled_annotation_is_ordinary_text_but_active_contexts_skip() {
    use opencadcodec::xdata::{ExtendedDataRecord, XDataValue as X};
    for flag in [0, 1] {
        let mut source = CadDocument::new();
        let mut text = Text::with_value("Annotation state", Vector3::ZERO);
        let mut record = ExtendedDataRecord::new("AcadAnnotative");
        record.values = vec![
            X::String("AnnotativeData".into()),
            X::ControlString("{".into()),
            X::Integer16(1),
            X::Integer16(flag),
            X::ControlString("}".into()),
        ];
        text.common.extended_data.add_record(record);
        source.add_entity(EntityType::Text(text)).unwrap();
        let native =
            cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
        assert_eq!(
            native.document().model.entities.len(),
            usize::from(flag == 0)
        );
    }
}

#[test]
fn model_paper_and_block_text_have_independent_actual_file_qualification() {
    let original = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/hello-text.ifcx"),
        Default::default(),
    )
    .unwrap();
    let target = ifccad_document_to_cad_document(original.document(), Default::default()).unwrap();
    for dwg in [false, true] {
        let cad = if dwg {
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
        let native = cad_document_to_ifccad_document(&cad, metadata(), Default::default()).unwrap();
        let back = native.document();
        assert_eq!(back.model.entities.len(), 3);
        assert_eq!(back.paper_layouts.len(), 1);
        assert_eq!(back.paper_layouts[0].entities.len(), 2);
        assert_eq!(back.blocks.len(), 1);
        assert_eq!(back.blocks[0].entities.len(), 2);
        assert_eq!(native.text_assessment().entries().len(), 6);
        assert!(!native.geometry_assessment().is_complete());
        for (a, b) in [
            &original.document().model.entities,
            &original.document().paper_layouts[0].entities,
            &original.document().blocks[0].entities,
        ]
        .into_iter()
        .zip([
            &back.model.entities,
            &back.paper_layouts[0].entities,
            &back.blocks[0].entities,
        ]) {
            for (a, b) in a.iter().zip(b) {
                match (&a.as_native().unwrap().kind, &b.as_native().unwrap().kind) {
                    (IfccadEntityKind::Text(a), IfccadEntityKind::Text(b)) => {
                        assert_eq!(a.content, b.content);
                        assert_eq!(a.placement, b.placement);
                        assert_eq!(a.layout, b.layout);
                        assert!((a.rotation - b.rotation).abs() <= f64::EPSILON);
                    }
                    (IfccadEntityKind::MText(a), IfccadEntityKind::MText(b)) => {
                        assert_eq!(a.content, b.content);
                        assert_eq!(a.placement, b.placement);
                        assert_eq!(a.height, b.height);
                    }
                    (
                        IfccadEntityKind::BlockInstance { .. },
                        IfccadEntityKind::BlockInstance { .. },
                    ) => {}
                    _ => panic!("native text owner/family changed"),
                }
            }
        }
        let encoded = encode_ifccad_document(back).unwrap();
        load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    }
}

#[test]
fn unsupported_annotation_does_not_hide_invalid_text_scalars_or_references() {
    use opencadcodec::xdata::{ExtendedDataRecord, XDataValue as X};
    for invalid_style in [false, true] {
        let mut source = CadDocument::new();
        let mut text = Text::with_value("Invalid source", Vector3::ZERO);
        if invalid_style {
            text.style = "Missing".into();
        } else {
            text.height = f64::NAN;
        }
        let mut r = ExtendedDataRecord::new("AcadAnnotative");
        r.values = vec![
            X::String("AnnotativeData".into()),
            X::ControlString("{".into()),
            X::Integer16(1),
            X::Integer16(1),
            X::ControlString("}".into()),
        ];
        text.common.extended_data.add_record(r);
        source.add_entity(EntityType::Text(text)).unwrap();
        for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            assert!(matches!(
                cad_document_to_ifccad_document(
                    &source,
                    metadata(),
                    CadToIfccadOptions {
                        loss_policy,
                        ..Default::default()
                    }
                ),
                Err(IfccadConversionError::InvalidStructure(_))
            ));
        }
    }
}

#[test]
fn manual_columns_use_auto_tail_in_ifccad_and_actual_file_readback() {
    use ocdraw::text::{MTextColumnHeight, MTextColumns};
    for tail in [0., -123.] {
        let mut source = CadDocument::new();
        let mut t = MText::new();
        t.value = "First\\PSecond".into();
        t.height = 2.;
        t.rectangle_width = 81.;
        t.rectangle_height = Some(30.);
        t.column_data = opencadcodec::entities::MTextColumnData {
            column_type: 2,
            column_count: 2,
            flow_reversed: false,
            auto_height: false,
            width: 40.,
            gutter: 1.,
            heights: vec![30., tail],
        };
        source.add_entity(EntityType::MText(t)).unwrap();
        let native =
            cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
        let IfccadEntityKind::MText(t) = &native.document().model.entities[0]
            .as_native()
            .unwrap()
            .kind
        else {
            unreachable!()
        };
        assert!(
            matches!(&t.columns,Some(MTextColumns::DynamicManualHeight{column_heights,..}) if column_heights==&vec![MTextColumnHeight::Fixed{distance:30.},MTextColumnHeight::Auto])
        );
        let target =
            ifccad_document_to_cad_document(native.document(), Default::default()).unwrap();
        for dwg in [false, true] {
            let cad = if dwg {
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
            let restored =
                cad_document_to_ifccad_document(&cad, metadata(), Default::default()).unwrap();
            let IfccadEntityKind::MText(r) = &restored.document().model.entities[0]
                .as_native()
                .unwrap()
                .kind
            else {
                unreachable!()
            };
            assert_eq!(r.columns, t.columns);
        }
    }
}

#[test]
fn unsupported_style_roles_do_not_hide_nonfinite_ordinary_metrics() {
    let mut source = CadDocument::new();
    let style = source.text_styles.get_mut("Standard").unwrap();
    style.annotative = true;
    style.width_factor = f64::NAN;
    assert!(matches!(
        cad_document_to_ifccad_document(&source, metadata(), Default::default()),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
}

#[test]
fn inline_palette_identity_loss_is_explicit_in_the_rgb_profile() {
    let mut source = CadDocument::new();
    let mut t = MText::new();
    t.value = "\\C65;Green".into();
    source.add_entity(EntityType::MText(t)).unwrap();
    let native = cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
    assert_eq!(native.document().model.entities.len(), 1);
    assert!(native
        .diagnostics()
        .iter()
        .any(|d| d.code == "text-color-index" && d.is_semantic_loss()));
}
