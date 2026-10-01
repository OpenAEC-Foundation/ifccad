mod common;
use cadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use common::*;
use ifcx_cad_convert::*;
use std::io::Cursor;

fn exchange(c: &CadDocument, dwg: bool) -> CadDocument {
    assert_eq!(c.version, cadcodec::DxfVersion::AC1032);
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
fn roundtrip(d: &ocdraw::ifcx_cad::IfcxCadDocument, dwg: bool) {
    let first = ifcx_cad_to_cad_document(&validated(d)).unwrap();
    let file = exchange(first.document(), dwg);
    let expected = cad_document_to_ifcx_cad(first.document(), metadata()).unwrap();
    let back = cad_document_to_ifcx_cad(&file, metadata()).unwrap();
    let second = ifcx_cad_to_cad_document(back.validated_ifcx()).unwrap();
    let stable = cad_document_to_ifcx_cad(second.document(), metadata()).unwrap();
    assert_eq!(
        back.validated_ifcx().document(),
        stable.validated_ifcx().document()
    );
    assert_eq!(
        back.validated_ifcx().document().model.entities.len(),
        d.model.entities.len()
    );
    assert_eq!(back.validated_ifcx().document().length_unit, d.length_unit);
    assert_eq!(
        semantic(expected.validated_ifcx().document()),
        semantic(back.validated_ifcx().document())
    );
}
#[test]
fn dxf_primitives_exchange() {
    roundtrip(&primitives(), false);
}
#[test]
fn dwg_primitives_exchange() {
    roundtrip(&primitives(), true);
}
#[test]
fn dxf_nested_blocks_with_nonzero_base() {
    roundtrip(&nested([1., 2., 0.]), false);
}
#[test]
fn dwg_nested_blocks_with_zero_base() {
    roundtrip(&nested([0.; 3]), true);
}
#[test]
fn dwg_nonzero_base_marker_limitation_is_explicit() {
    let c = ifcx_cad_to_cad_document(&validated(&nested([1., 2., 0.])))
        .unwrap()
        .into_document();
    let file = exchange(&c, true);
    assert!(
        matches!(cad_document_to_ifcx_cad(&file,metadata()),Err(IfcxCadConversionError::InvalidStructure(s)) if s.contains("marker"))
    );
}

fn semantic(d: &ocdraw::ifcx_cad::IfcxCadDocument) -> serde_json::Value {
    use ocdraw::ifcx_cad::*;
    let entities = |values: &[IfcxCadEntity]| {
        values
            .iter()
            .map(|e| {
                let layer = &d.layers.iter().find(|l| l.id == e.layer_id).unwrap().name;
                let geometry = match &e.kind {
                    IfcxCadEntityKind::LineSegment { start, end } => {
                        serde_json::json!(["line", start, end])
                    }
                    IfcxCadEntityKind::Circle { radius, placement } => {
                        serde_json::json!(["circle", radius, placement])
                    }
                    IfcxCadEntityKind::PlanarPolyline {
                        vertices,
                        closed,
                        placement,
                    } => serde_json::json!(["polyline", vertices, closed, placement]),
                    IfcxCadEntityKind::BlockInstance {
                        definition_id,
                        transform,
                    } => serde_json::json!([
                        "insert",
                        d.blocks
                            .iter()
                            .find(|b| b.id == *definition_id)
                            .unwrap()
                            .name,
                        transform
                    ]),
                };
                serde_json::json!({"layer":layer,"appearance":e.appearance,"geometry":geometry})
            })
            .collect::<Vec<_>>()
    };
    let mut layers = d
        .layers
        .iter()
        .map(|l| (&l.name, &l.appearance))
        .collect::<Vec<_>>();
    layers.sort_by_key(|l| l.0);
    let mut blocks=d.blocks.iter().map(|b|serde_json::json!({"name":b.name,"base":b.base_point,"unit":b.insertion_unit,"entities":entities(&b.entities)})).collect::<Vec<_>>();
    blocks.sort_by_key(|v| v["name"].as_str().unwrap().to_string());
    serde_json::json!({"unit":d.length_unit,"layers":layers,"model":entities(&d.model.entities),"blocks":blocks})
}
