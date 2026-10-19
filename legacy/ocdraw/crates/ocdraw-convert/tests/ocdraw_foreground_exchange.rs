#[path = "support/foreground.rs"]
mod support;
use ocdraw::ocdraw::*;
use ocdraw::text::{MTextFill, MTextInline, TextColor};
use ocdraw_convert::*;

fn assert_native(d: &OcdrawDocument) {
    assert_eq!(
        d.layers.iter().find(|l| l.name == "0").unwrap().color,
        DrawingColor::Foreground
    );
    let text = &d.mtext_entities[0];
    assert!(matches!(
        text.background.as_ref().unwrap().fill,
        MTextFill::Color(DrawingColor::Foreground)
    ));
    assert!(text.content.iter().flat_map(|p|&p.inlines).any(|i|matches!(i,MTextInline::Run {character_format,..} if matches!(character_format.color,Some(TextColor::Explicit(DrawingColor::Foreground))))));
    assert!(d.viewports.iter().any(|v| v
        .layer_overrides
        .iter()
        .any(|r| r.color == Some(DrawingColor::Foreground))));
}
#[test]
fn foreground_survives_dxf_and_ac1032_dwg() {
    let source = support::source();
    let native = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
    assert_native(native.document());
    let encoded = encode_ocdraw_document(native.document()).unwrap();
    let strict = load_ocdraw_bytes(encoded.bytes()).unwrap();
    let target = ocdraw_document_to_cad_document(strict.document(), Default::default()).unwrap();
    support::assert_cad_colors(target.document());
    for dwg in [false, true] {
        let source = support::readback(target.document(), dwg);
        support::assert_cad_colors(&source);
        let back = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
        assert_native(back.document());
        let encoded = encode_ocdraw_document(back.document()).unwrap();
        assert_native(load_ocdraw_bytes(encoded.bytes()).unwrap().document());
    }
}
