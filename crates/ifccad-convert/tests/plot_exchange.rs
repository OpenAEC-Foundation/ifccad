mod common;
use ifccad_convert::{
    opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter},
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
fn plot_and_medium_only_have_independent_strict_file_readback() {
    for bytes in [
        include_bytes!("../../../conformance/next/ifccad/valid/layout-plot-inch.ifcx").as_slice(),
        include_bytes!("../../../conformance/next/ifccad/valid/layout-medium-only.ifcx").as_slice(),
        include_bytes!("../../../conformance/next/ifccad/valid/layout-linetype-scaling.ifcx")
            .as_slice(),
    ] {
        let native = ocdraw::ifccad::load_ifccad_bytes(bytes, Default::default()).unwrap();
        let output =
            ifccad_document_to_cad_document(native.document(), Default::default()).unwrap();
        for dwg in [false, true] {
            let cad = exchange(output.document(), dwg);
            let restored =
                cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default())
                    .unwrap();
            let paper = &restored.validated_source().document().paper_layouts[0];
            assert_eq!(
                paper.settings,
                native.document().paper_layouts[0].settings,
                "{dwg}"
            );
        }
    }
}
#[test]
fn raster_medium_is_native_but_not_guessed_into_cad_millimetres() {
    let native = ocdraw::ifccad::load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/layout-raster.ifcx"),
        Default::default(),
    )
    .unwrap();
    let output = ifccad_document_to_cad_document(native.document(), Default::default()).unwrap();
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.code == "paper-medium"));
    assert!(ifccad_document_to_cad_document(
        native.document(),
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}
