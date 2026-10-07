use ocdraw_convert::{
    opencadcodec::{objects::ObjectType, CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter},
    *,
};
use std::io::Cursor;
fn exchange(c: &CadDocument, dwg: bool) -> CadDocument {
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(c).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(DxfWriter::new(c).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap()
    }
}
#[test]
fn plot_fields_and_medium_only_survive_actual_cad_codecs() {
    for bytes in [
        include_bytes!("../../../conformance/next/ocdraw/valid/layout-plot-inch.ocdraw.json")
            .as_slice(),
        include_bytes!("../../../conformance/next/ocdraw/valid/layout-medium-only.ocdraw.json")
            .as_slice(),
        include_bytes!(
            "../../../conformance/next/ocdraw/valid/layout-linetype-scaling.ocdraw.json"
        )
        .as_slice(),
    ] {
        let native = ocdraw::ocdraw::load_ocdraw_bytes(bytes).unwrap();
        let output = ocdraw_source_to_cad_document(&native, Default::default()).unwrap();
        for dwg in [false, true] {
            let cad = exchange(output.document(), dwg);
            let restored = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
            let encoded = ocdraw::ocdraw::encode_ocdraw_document(restored.document()).unwrap();
            let read = ocdraw::ocdraw::load_ocdraw_bytes(encoded.bytes()).unwrap();
            let paper = read
                .document()
                .layouts
                .iter()
                .find(|l| l.name == "Sheet")
                .unwrap();
            let original = &native.document().layouts[1];
            assert_eq!(paper.settings, original.settings, "{dwg}");
            let raw = cad
                .objects
                .values()
                .find_map(|o| match o {
                    ObjectType::Layout(l) if l.name == "Sheet" => Some(l),
                    _ => None,
                })
                .unwrap();
            assert_eq!(
                raw.flags & 1 != 0,
                original.settings.paper_space_linetype_scaling
            );
        }
    }
}
#[test]
fn raster_transfer_reports_missing_calibration() {
    let native = ocdraw::ocdraw::load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/layout-raster.ocdraw.json"
    ))
    .unwrap();
    let out = ocdraw_source_to_cad_document(&native, Default::default()).unwrap();
    assert!(out.diagnostics().iter().any(|d| d.code == "LAYOUT_MEDIA"));
    assert!(matches!(
        ocdraw_source_to_cad_document(
            &native,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
}
