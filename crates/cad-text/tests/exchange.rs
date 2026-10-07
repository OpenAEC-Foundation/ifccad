//! CAD codec qualification only; native document schemas/routes are separate.
use cad_text::*;
use ocdraw::{geometry_kernel::CoordinateFrame3, text::*};
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
use std::io::Cursor;

#[test]
fn simple_text_and_mtext_content_survive_actual_dxf_and_dwg() {
    let runs = [TextRun {
        text: r"D-01 {literal} \P %<literal>%".into(),
        underline: true,
        ..Default::default()
    }];
    let paragraphs = [
        MTextParagraph {
            inlines: vec![
                MTextInline::Run {
                    text: "Header".into(),
                    character_format: CharacterFormat {
                        underline: Some(true),
                        ..Default::default()
                    },
                },
                MTextInline::Tab,
                MTextInline::Run {
                    text: "12,5".into(),
                    character_format: Default::default(),
                },
            ],
            ..Default::default()
        },
        MTextParagraph::default(),
    ];
    let c = CharacterFormat::default();
    let p = ParagraphFormat::default();
    let text = prepare_text_to_cad(&TextCadInput {
        style_name: "Standard",
        placement: CoordinateFrame3::default(),
        rotation: 0.,
        backward: false,
        upside_down: false,
        layout: TextLayout::Anchored {
            horizontal: TextHorizontalAlignment::Left,
            vertical: TextVerticalAlignment::Baseline,
            height: 2.5,
            width_factor: 1.,
        },
        oblique_angle: 0.,
        thickness: 0.,
        content: &runs,
    })
    .unwrap();
    let mtext = prepare_mtext_to_cad(&MTextCadInput {
        style_name: "Standard",
        placement: CoordinateFrame3::default(),
        rotation: 0.,
        backward: false,
        upside_down: false,
        height: 2.5,
        attachment: MTextAttachment::TopLeft,
        flow: MTextFlow::Horizontal,
        wrap_width: Some(80.),
        columns: None,
        background: None,
        content: &paragraphs,
        character_format: &c,
        paragraph_format: &p,
    })
    .unwrap();
    let mut doc = CadDocument::new();
    doc.add_entity(text.entity).unwrap();
    doc.add_entity(mtext.entity).unwrap();
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
        let texts = loaded
            .entities()
            .filter_map(|entity| {
                if let EntityType::Text(text) = entity {
                    Some(text)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(texts.len(), 1);
        assert_eq!(
            prepare_text_from_cad(texts[0]).unwrap().content,
            runs,
            "format dwg={dwg}"
        );
        let mtexts = loaded
            .entities()
            .filter_map(|entity| {
                if let EntityType::MText(text) = entity {
                    Some(text)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(mtexts.len(), 1);
        let restored = prepare_mtext_from_cad(mtexts[0]).unwrap();
        assert_eq!(restored.content.content.len(), 2);
        assert!(restored.content.content[1].inlines.is_empty());
        assert!(restored.content.content[0]
            .inlines
            .iter()
            .any(|inline| matches!(inline, MTextInline::Tab)));
        let literals = restored.content.content[0]
            .inlines
            .iter()
            .filter_map(|inline| {
                if let MTextInline::Run { text, .. } = inline {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(literals, vec!["Header", "12,5"]);
    }
}
