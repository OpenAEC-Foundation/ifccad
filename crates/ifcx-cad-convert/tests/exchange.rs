mod common;
use common::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;

fn exchange(c: &CadDocument, dwg: bool) -> CadDocument {
    assert_eq!(c.version, opencadcodec::DxfVersion::AC1032);
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
    let first = to_cad(&validated(d)).unwrap();
    let file = exchange(first.document(), dwg);
    let expected = from_cad(first.document(), metadata()).unwrap();
    let back = from_cad(&file, metadata()).unwrap();
    let second = to_cad(back.validated_source()).unwrap();
    let stable = from_cad(second.document(), metadata()).unwrap();
    assert_eq!(
        back.validated_source().document(),
        stable.validated_source().document()
    );
    assert_eq!(
        back.validated_source().document().model.entities.len(),
        d.model.entities.len()
    );
    assert_eq!(
        back.validated_source().document().length_unit,
        d.length_unit
    );
    assert_eq!(
        semantic(expected.validated_source().document()),
        semantic(back.validated_source().document())
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
fn dwg_nested_blocks_with_nonzero_base() {
    roundtrip(&nested([1., 2., 0.]), true);
}

#[test]
fn dxf_degree_radian_rounding_is_an_external_codec_limit() {
    use ocdraw::ifcx_cad::IfcxCadEntityKind;
    let mut d = nested([0.; 3]);
    let source_rotation = 1.570796326794893_f64;
    let IfcxCadEntityKind::BlockInstance { transform, .. } = &mut d.model.entities[0].kind else {
        panic!()
    };
    transform.rotation = source_rotation;
    let first = to_cad(&validated(&d)).unwrap();
    let h = first
        .mappings()
        .entities
        .cad_handle(d.model.entities[0].id)
        .unwrap();
    let opencadcodec::EntityType::Insert(i) = first.document().get_entity(h).unwrap() else {
        panic!()
    };
    assert_eq!(i.rotation, source_rotation);
    let file = exchange(first.document(), false);
    let restored = from_cad(&file, metadata()).unwrap();
    let IfcxCadEntityKind::BlockInstance { transform, .. } =
        &restored.validated_source().document().model.entities[0].kind
    else {
        panic!()
    };
    assert_eq!(
        transform.rotation,
        source_rotation.to_degrees().to_radians()
    );
    assert_eq!(transform.rotation.to_bits(), source_rotation.to_bits() + 1);
}

fn semantic(d: &ocdraw::ifcx_cad::IfcxCadDocument) -> serde_json::Value {
    use ocdraw::ifcx_cad::*;
    let pattern_name = |id: IfcxCadLinePatternId| {
        d.line_patterns
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .name
            .clone()
    };
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
                        line_pattern_generation,
                    } => serde_json::json!(["polyline", vertices, closed, placement, line_pattern_generation]),
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
                { let mut appearance = serde_json::to_value(&e.appearance).unwrap();
                  if let IfcxCadMode::Explicit(id) = e.appearance.line_pattern { appearance["linePattern"]["value"] = serde_json::json!(pattern_name(id)); }
                  serde_json::json!({"layer":layer,"appearance":appearance,"scale":e.line_pattern_scale,"geometry":geometry}) }
            })
            .collect::<Vec<_>>()
    };
    let mut layers = d
        .layers
        .iter()
        .map(|l| {
            let mut a = serde_json::to_value(&l.appearance).unwrap();
            a["linePattern"] = serde_json::json!(pattern_name(l.appearance.line_pattern));
            (&l.name, a)
        })
        .collect::<Vec<_>>();
    layers.sort_by_key(|l| l.0);
    let mut blocks=d.blocks.iter().map(|b|serde_json::json!({"name":b.name,"base":b.base_point,"unit":b.insertion_unit,"entities":entities(&b.entities)})).collect::<Vec<_>>();
    blocks.sort_by_key(|v| v["name"].as_str().unwrap().to_string());
    let mut patterns = d
        .line_patterns
        .iter()
        .map(|p| serde_json::json!({"name":p.name,"description":p.description,"pattern":p.pattern}))
        .collect::<Vec<_>>();
    patterns.sort_by_key(|p| p["name"].as_str().unwrap().to_string());
    serde_json::json!({"unit":d.length_unit,"patterns":patterns,"scale":d.line_pattern_scale,"layers":layers,"model":entities(&d.model.entities),"blocks":blocks})
}
