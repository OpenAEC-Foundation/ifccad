use ocdraw::{
    geometry_kernel::CoordinateLengthUnit as Unit,
    ocdraw::*,
    plot_kernel::{LayoutMedia, MediaUnit},
};
#[test]
fn medium_only_model_and_paper_have_strict_native_roundtrip() {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("media-only", "unitless")).unwrap();
    let paper = b.add_paper_layout("A3").unwrap();
    for id in [0, paper] {
        b.set_layout_settings(
            id,
            LayoutSettings {
                media: Some(LayoutMedia {
                    unit: MediaUnit::Physical(Unit::Millimetre),
                    width: 420.,
                    height: 297.,
                }),
                ..Default::default()
            },
        )
        .unwrap();
    }
    let doc = b.build_document().unwrap();
    let encoded = encode_ocdraw_document(&doc).unwrap();
    let read = load_ocdraw_bytes(encoded.bytes()).unwrap();
    for l in &read.document().layouts {
        assert_eq!(l.settings.media.as_ref().unwrap().width, 420.);
        assert!(l.settings.plot_settings.is_none());
    }
    assert_eq!(read.document().next_layout_id, doc.next_layout_id);
    assert_eq!(read.document().scopes.len(), doc.scopes.len());
}
