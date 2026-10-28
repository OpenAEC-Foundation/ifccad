mod common;
#[path = "support/foreground.rs"]
mod support;
use ifccad::text::{MTextFill, MTextInline, TextColor};
use ifccad::*;
use ifccad_convert::*;

fn assert_native(d: &IfccadDocument) {
    assert_eq!(
        d.layers
            .iter()
            .find(|l| l.name == "0")
            .unwrap()
            .appearance
            .color,
        IfccadColor::Foreground
    );
    let text = d
        .model
        .entities
        .iter()
        .find_map(|e| {
            if let IfccadEntityKind::MText(t) = &e.as_native()?.kind {
                Some(t)
            } else {
                None
            }
        })
        .unwrap();
    assert!(matches!(
        text.background.as_ref().unwrap().fill,
        MTextFill::Color(IfccadColor::Foreground)
    ));
    assert!(text.content.iter().flat_map(|p|&p.inlines).any(|i|matches!(i,MTextInline::Run {character_format,..} if matches!(character_format.color,Some(TextColor::Explicit(IfccadColor::Foreground))))));
    assert!(d.paper_layouts.iter().flat_map(|l|&l.entities).filter_map(|e|e.as_native()).any(|e|matches!(&e.kind,IfccadEntityKind::Viewport(v) if v.layer_overrides.iter().any(|r|r.color==Some(IfccadColor::Foreground)))));
}
#[test]
fn foreground_survives_dxf_and_ac1032_dwg() {
    let source = support::source();
    let native =
        cad_document_to_ifccad_document(&source, common::metadata(), Default::default()).unwrap();
    assert_native(native.document());
    let encoded = encode_ifccad_document(native.document()).unwrap();
    let strict = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    let target = ifccad_document_to_cad_document(strict.document(), Default::default()).unwrap();
    support::assert_cad_colors(target.document());
    for dwg in [false, true] {
        let source = support::readback(target.document(), dwg);
        support::assert_cad_colors(&source);
        let back = cad_document_to_ifccad_document(&source, common::metadata(), Default::default())
            .unwrap();
        assert_native(back.document());
        let encoded = encode_ifccad_document(back.document()).unwrap();
        assert_native(
            load_ifccad_bytes(encoded.bytes(), Default::default())
                .unwrap()
                .document(),
        );
    }
}
