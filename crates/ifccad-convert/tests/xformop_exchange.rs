use ifccad::*;
use ifccad_convert::{
    cad_document_to_encoded_ifccad, ifccad_document_to_cad_document, ifccad_source_to_cad_document,
    CadToIfccadOptions, IfccadLossPolicy, IfccadTargetMetadata, IfccadToCadOptions,
};
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use serde_json::{json, Value};
use std::io::Cursor;

const NATIVE: &[u8] =
    include_bytes!("../../../examples/experiments/ifcx-xformop/native-placement.ifcx");
const MATRIX: &[u8] =
    include_bytes!("../../../examples/experiments/ifcx-xformop/standard-xformop.ifcx");

fn semantics(document: &IfccadDocument) -> Value {
    let entities = |values: &[IfccadEntity]| -> Vec<Value> {
        values.iter().map(|entity| {
            let entity = entity.as_native().unwrap();
            let layer = &document.layers.iter().find(|l|l.id == entity.layer_id).unwrap().name;
            let geometry = match &entity.kind {
                IfccadEntityKind::LineSegment {start,end} => json!(["line",start,end]),
                IfccadEntityKind::PlanarPolyline {vertices,bulges,closed,placement,line_pattern_generation} =>
                    json!(["path",vertices,bulges,closed,placement,line_pattern_generation]),
                IfccadEntityKind::BlockInstance {definition_id,transform} =>
                    json!(["instance",document.blocks.iter().find(|b|b.id == *definition_id).unwrap().name,transform]),
                _ => panic!("unexpected family in this bounded example"),
            };
            json!({"layer":layer,"appearance":entity.appearance,"patternScale":entity.line_pattern_scale,"geometry":geometry})
        }).collect()
    };
    let mut blocks: Vec<_> = document.blocks.iter().map(|block|
        json!({"name":block.name,"base":block.base_point,"unit":block.insertion_unit,"entities":entities(&block.entities)})).collect();
    blocks.sort_by_key(|value| value["name"].as_str().unwrap().to_owned());
    json!({"unit":document.length_unit,"model":entities(&document.model.entities),"blocks":blocks})
}

fn exchange(document: &IfccadDocument, dwg: bool) -> Value {
    let options = IfccadToCadOptions {
        loss_policy: IfccadLossPolicy::Reject,
        ..Default::default()
    };
    let converted = ifccad_document_to_cad_document(document, options).unwrap();
    let cad: CadDocument = if dwg {
        DwgReader::from_stream(Cursor::new(
            DwgWriter::write_to_vec(converted.document()).unwrap(),
        ))
        .read()
        .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(
            DxfWriter::new(converted.document()).write_to_vec().unwrap(),
        ))
        .unwrap()
        .read()
        .unwrap()
    };
    let native = cad_document_to_encoded_ifccad(
        &cad,
        IfccadTargetMetadata {
            header: document.header.clone(),
            drawing_id: document.drawing_id,
        },
        CadToIfccadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    let readback = load_ifccad_bytes(native.encoded().bytes(), Default::default()).unwrap();
    semantics(readback.document())
}

#[test]
fn xformop_projected_drawing_has_the_same_dxf_and_dwg_exchange_as_native_placement() {
    let native = load_ifccad_bytes(NATIVE, Default::default()).unwrap();
    let matrix = load_ifccad_xformop_bytes(MATRIX, Default::default()).unwrap();
    for dwg in [false, true] {
        assert_eq!(
            exchange(matrix.document(), dwg),
            exchange(native.document(), dwg)
        );
    }
}

#[test]
fn original_matrix_source_is_not_silently_treated_as_an_ordinary_native_graph() {
    let matrix = load_ifccad_xformop_bytes(MATRIX, Default::default()).unwrap();
    let error = match ifccad_source_to_cad_document(
        &matrix,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    ) {
        Err(error) => error,
        Ok(_) => panic!("expected explicit graph-loss rejection"),
    };
    let ifccad_convert::IfccadConversionError::Unsupported(diagnostics) = error else {
        panic!("expected explicit graph-loss diagnostics, got {error}");
    };
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "foreign-ifcx"));
    // The explicitly obtained native document is the qualified conversion input.
    assert!(ifccad_document_to_cad_document(matrix.document(), Default::default()).is_ok());
}
