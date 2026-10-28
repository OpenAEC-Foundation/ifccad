mod common;
use ifccad::{text::*, *};
use ifccad_convert::*;
use opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, MText, Vector3,
};
use std::io::Cursor;

fn cad_readback(document: &CadDocument, dwg: bool) -> CadDocument {
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(document).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(
            DxfWriter::new(document).write_to_vec().unwrap(),
        ))
        .unwrap()
        .read()
        .unwrap()
    }
}

fn strict_native(document: &IfccadDocument) -> IfccadDocument {
    let bytes = encode_ifccad_document(document).unwrap();
    let read = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    validate_ifccad_document(read.document()).unwrap();
    let cbor = encode_ifccad_cbor_document(read.document()).unwrap();
    let back = load_ifccad_cbor_bytes(cbor.bytes(), Default::default()).unwrap();
    validate_ifccad_document(back.document()).unwrap();
    assert_eq!(back.document(), read.document());
    back.document().clone()
}

fn source(values: &[&str]) -> CadDocument {
    let mut doc = CadDocument::new();
    doc.text_styles.get_mut("Standard").unwrap().font_file = "txt.shx".into();
    for (i, value) in values.iter().enumerate() {
        let mut text = MText::new();
        text.value = (*value).into();
        text.height = 2.0;
        text.rectangle_width = 80.0;
        text.insertion_point = Vector3::new(i as f64 * 40.0, 0.0, 0.0);
        doc.add_entity(EntityType::MText(Box::new(text))).unwrap();
    }
    doc
}

fn mtext(document: &IfccadDocument, index: usize) -> &IfccadMText {
    let IfccadEntityKind::MText(text) = &document.model.entities[index].as_native().unwrap().kind
    else {
        panic!("MText kind changed")
    };
    text
}

fn assert_case(document: &IfccadDocument, index: usize, lefts: &[f64], labels: &[&str]) {
    let text = mtext(document, index);
    assert_eq!(text.content.len(), lefts.len());
    assert_eq!(text.height, 2.0);
    assert_eq!(text.wrap_width, Some(80.0));
    assert_eq!(text.placement.origin, [index as f64 * 40.0, 0.0, 0.0]);
    for (p, (&left, &label)) in text.content.iter().zip(lefts.iter().zip(labels)) {
        let effective =
            resolve_paragraph_format(&text.paragraph_format, &p.paragraph_format).unwrap();
        assert_eq!(effective.left_indent_factor, left);
        let literal = p
            .inlines
            .iter()
            .map(|inline| match inline {
                MTextInline::Run { text, .. } => text.as_str(),
                _ => panic!("Unexpected inline in synthetic case"),
            })
            .collect::<String>();
        assert_eq!(literal, label);
    }
}

fn assert_full_fields(document: &IfccadDocument) {
    let text = mtext(document, 1);
    for (p, left) in text.content.iter().zip([2.0, 5.0, 5.0]) {
        let p = resolve_paragraph_format(&text.paragraph_format, &p.paragraph_format).unwrap();
        assert_eq!(p.alignment, TextParagraphAlignment::Center);
        assert_eq!(p.first_line_indent_factor, -1.0);
        assert_eq!(p.left_indent_factor, left);
        assert_eq!(p.right_indent_factor, 3.0);
        assert_eq!(
            p.tab_stops,
            vec![
                TextTabStop {
                    position_factor: 4.0,
                    alignment: TextTabAlignment::Left
                },
                TextTabStop {
                    position_factor: 8.0,
                    alignment: TextTabAlignment::Center
                },
                TextTabStop {
                    position_factor: 12.0,
                    alignment: TextTabAlignment::Right
                },
            ]
        );
    }
    let text = mtext(document, 2);
    for p in &text.content[1..] {
        let p = resolve_paragraph_format(&text.paragraph_format, &p.paragraph_format).unwrap();
        assert_eq!(p.alignment, TextParagraphAlignment::Left);
        assert_eq!(p.first_line_indent_factor, 0.0);
        assert_eq!(p.right_indent_factor, 0.0);
        assert!(p.tab_stops.is_empty());
    }
    let text = mtext(document, 6);
    for (p, (alignment, tab)) in text.content.iter().zip([
        (TextParagraphAlignment::Center, 4.0),
        (TextParagraphAlignment::Right, 8.0),
        (TextParagraphAlignment::Right, 8.0),
        (TextParagraphAlignment::Center, 4.0),
    ]) {
        let p = resolve_paragraph_format(&text.paragraph_format, &p.paragraph_format).unwrap();
        assert_eq!(p.alignment, alignment);
        assert_eq!(
            p.tab_stops,
            vec![TextTabStop {
                position_factor: tab,
                alignment: TextTabAlignment::Left
            }]
        );
    }
}

#[test]
fn paragraph_carry_resets_and_scopes_survive_actual_cad_and_strict_native_readback() {
    let cases: &[(&str, &[f64], &[&str])] = &[
        (r"\pl1;A\PB\PC", &[1.0, 1.0, 1.0], &["A", "B", "C"]),
        (
            r"\pqc,i-1,l2,r3,t4,c8,r12;A\P\pl5;B\PC",
            &[2.0, 5.0, 5.0],
            &["A", "B", "C"],
        ),
        (
            r"\pqc,i-1,l2,r3,t4;A\P\pi0,l0,r0,ql,t;B\PC",
            &[2.0, 0.0, 0.0],
            &["A", "B", "C"],
        ),
        (
            r"\pl1;A\P{\pl2;B\P{\pl3;C}\PD}\PE",
            &[1.0, 2.0, 3.0, 2.0, 1.0],
            &["A", "B", "C", "D", "E"],
        ),
        (
            r"\pl1;A\P{\pl2;B\P}C\PD",
            &[1.0, 2.0, 1.0, 1.0],
            &["A", "B", "C", "D"],
        ),
        (r"\pl1;A\P\PB\P", &[1.0, 1.0, 1.0, 1.0], &["A", "", "B", ""]),
        (
            r"\pqc,l1,t4;A\P{\pqr,l2,t8;B\PC}\PD",
            &[1.0, 2.0, 2.0, 1.0],
            &["A", "B", "C", "D"],
        ),
    ];
    let source = source(&cases.iter().map(|c| c.0).collect::<Vec<_>>());
    let original = serde_json::to_value(&source).unwrap();
    for dwg in [false, true] {
        let read = cad_readback(&source, dwg);
        let imported =
            cad_document_to_ifccad_document(&read, common::metadata(), Default::default()).unwrap();
        assert!(imported
            .diagnostics()
            .iter()
            .any(|d| d.message.contains("AuthoredFormattingNormalized")));
        assert!(matches!(
            cad_document_to_ifccad_document(
                &read,
                common::metadata(),
                CadToIfccadOptions {
                    loss_policy: IfccadLossPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(IfccadConversionError::Unsupported(diagnostics)) if diagnostics.iter().any(|d| d.code == "text-dependency" && d.message == "AuthoredFormattingNormalized")
        ));
        let native = strict_native(imported.document());
        let target = ifccad_document_to_cad_document(&native, Default::default()).unwrap();
        let output = cad_readback(target.document(), dwg);
        let restored =
            cad_document_to_ifccad_document(&output, common::metadata(), Default::default())
                .unwrap();
        let restored = strict_native(restored.document());
        for doc in [&native, &restored] {
            assert_eq!(doc.model.entities.len(), cases.len());
            for (index, (_, lefts, labels)) in cases.iter().enumerate() {
                assert_case(doc, index, lefts, labels);
            }
            assert_full_fields(doc);
        }
    }
    assert_eq!(serde_json::to_value(&source).unwrap(), original);
}

#[test]
fn native_entity_basis_and_default_paragraphs_end_previous_cad_overrides() {
    let source = source(&[r"A\PB\PC"]);
    let imported =
        cad_document_to_ifccad_document(&source, common::metadata(), Default::default()).unwrap();
    let mut document = imported.document().clone();
    let IfccadEntityKind::MText(text) =
        &mut document.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    text.paragraph_format.left_indent_factor = Some(0.5);
    text.content[0].paragraph_format = ParagraphFormat {
        alignment: Some(TextParagraphAlignment::Right),
        first_line_indent_factor: Some(-1.0),
        left_indent_factor: Some(2.0),
        right_indent_factor: Some(3.0),
        tab_stops: Some(vec![TextTabStop {
            position_factor: 4.0,
            alignment: TextTabAlignment::Center,
        }]),
        ..Default::default()
    };
    let native = strict_native(&document);
    let target = ifccad_document_to_cad_document(&native, Default::default()).unwrap();
    assert!(target
        .diagnostics()
        .iter()
        .any(|d| d.message.contains("AuthoredFormattingNormalized")));
    for dwg in [false, true] {
        let read = cad_readback(target.document(), dwg);
        let imported =
            cad_document_to_ifccad_document(&read, common::metadata(), Default::default()).unwrap();
        let restored = strict_native(imported.document());
        assert_case(&restored, 0, &[2.0, 0.5, 0.5], &["A", "B", "C"]);
        let text = mtext(&restored, 0);
        for p in &text.content[1..] {
            let p = resolve_paragraph_format(&text.paragraph_format, &p.paragraph_format).unwrap();
            assert_eq!(p.alignment, TextParagraphAlignment::Left);
            assert_eq!(p.first_line_indent_factor, 0.0);
            assert_eq!(p.right_indent_factor, 0.0);
            assert!(p.tab_stops.is_empty());
        }
    }
    assert_eq!(native, document);
}
