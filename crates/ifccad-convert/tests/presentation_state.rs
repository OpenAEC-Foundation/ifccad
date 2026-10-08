mod common;
use common::*;
use ifccad_convert::*;
use opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, MText, Point, Text,
    Vector3,
};
use serde_json::{json, Value};
use std::io::Cursor;

fn source() -> CadDocument {
    let mut doc = ifccad_document_to_cad_document(&nested([0.; 3]), Default::default())
        .unwrap()
        .into_document();
    doc.add_entity(EntityType::Point(Point::at(Vector3::new(3., 4., 0.))))
        .unwrap();
    let mut text = Text::with_value("Hidden text", Vector3::new(6., 7., 0.));
    text.height = 2.;
    doc.add_entity(EntityType::Text(text)).unwrap();
    let mut mtext = MText::new();
    mtext.value = "Hidden MText".into();
    mtext.height = 3.;
    doc.add_entity(EntityType::MText(mtext)).unwrap();
    doc.add_layout("Sheet").unwrap();
    let mut viewport = opencadcodec::entities::Viewport::new();
    viewport.id = 2;
    viewport.center = Vector3::new(100., 100., 0.);
    viewport.width = 30.;
    viewport.height = 20.;
    viewport.view_height = 50.;
    viewport.view_direction = Vector3::new(0., 0., 100.);
    doc.add_entity_to_layout(EntityType::Viewport(viewport), "Sheet")
        .unwrap();
    let handles: Vec<_> = doc
        .entities()
        .filter(|e| match e {
            EntityType::Viewport(v) => v.id != 1,
            EntityType::Block(_) | EntityType::BlockEnd(_) => false,
            _ => true,
        })
        .map(|e| e.common().handle)
        .collect();
    for handle in handles {
        doc.get_entity_mut(handle).unwrap().common_mut().invisible = true;
    }
    let layer = doc.layers.get_mut("0").unwrap();
    layer.flags.off = true;
    layer.flags.frozen = true;
    layer.flags.locked = true;
    layer.flags.frozen_in_new_viewport = true;
    layer.is_plottable = false;
    layer.description = "Hidden locked nonplot layer".into();
    doc
}

fn backing(source: &CadDocument, transport: u8) -> CadDocument {
    match transport {
        0 => source.clone(),
        1 => DxfReader::from_reader(Cursor::new(DxfWriter::new(source).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap(),
        _ => DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(source).unwrap()))
            .read()
            .unwrap(),
    }
}

#[test]
fn layer_status_survives_native_and_real_cad_readback_in_both_directions() {
    let original = source();
    for transport in 0..3 {
        let source = backing(&original, transport);
        let native =
            cad_document_to_encoded_ifccad(&source, metadata(), Default::default()).unwrap();
        let raw: Value = serde_json::from_slice(native.encoded().bytes()).unwrap();
        let layer = raw["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|n| n["attributes"].get("ifccad::layer"))
            .find(|l| l["name"] == "0")
            .unwrap();
        for (key, expected) in [
            ("visible", false),
            ("frozen", true),
            ("locked", true),
            ("plottable", false),
            ("frozenInNewViewports", true),
        ] {
            assert_eq!(layer[key], json!(expected), "transport {transport}: {key}");
        }
        assert_eq!(layer["description"], "Hidden locked nonplot layer");
        assert!(
            !native.diagnostics().iter().any(|d| d.location == "layer/0"
                && (d.message.contains("flags") || d.message.contains("description"))),
            "mapped metadata remains loss"
        );
        let target = ifccad_document_to_cad_document(
            native.validated_source().document(),
            Default::default(),
        )
        .unwrap();
        let restored = backing(target.document(), transport);
        let layer = restored.layers.get("0").unwrap();
        assert!(
            layer.flags.off
                && layer.flags.frozen
                && layer.flags.locked
                && layer.flags.frozen_in_new_viewport
        );
        assert!(!layer.is_plottable);
        assert_eq!(layer.description, "Hidden locked nonplot layer");
    }
}

#[test]
fn common_hidden_state_survives_geometry_instances_text_and_viewports() {
    let original = source();
    for transport in 0..3 {
        let source = backing(&original, transport);
        let native =
            cad_document_to_encoded_ifccad(&source, metadata(), Default::default()).unwrap();
        let document = native.validated_source().document();
        assert!(document.model.entities.iter().any(|e| matches!(
            &e.as_native().unwrap().kind,
            ocdraw::ifccad::IfccadEntityKind::Text(_)
        )));
        assert!(document.model.entities.iter().any(|e| matches!(
            &e.as_native().unwrap().kind,
            ocdraw::ifccad::IfccadEntityKind::MText(_)
        )));
        assert!(document.model.entities.iter().any(|e| matches!(
            &e.as_native().unwrap().kind,
            ocdraw::ifccad::IfccadEntityKind::Point { .. }
        )));
        assert!(document.model.entities.iter().any(|e| matches!(
            &e.as_native().unwrap().kind,
            ocdraw::ifccad::IfccadEntityKind::BlockInstance { .. }
        )));
        assert!(document
            .paper_layouts
            .iter()
            .any(|p| p.entities.iter().any(|e| matches!(
                &e.as_native().unwrap().kind,
                ocdraw::ifccad::IfccadEntityKind::Viewport(_)
            ))));
        let raw: Value = serde_json::from_slice(native.encoded().bytes()).unwrap();
        for entity in raw["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|n| n["attributes"].get("ifccad::entity"))
        {
            assert_eq!(entity["visible"], json!(false), "transport {transport}");
        }
        let target = ifccad_document_to_cad_document(document, Default::default()).unwrap();
        let restored = backing(target.document(), transport);
        for entity in restored.entities().filter(|e| match e {
            EntityType::Viewport(v) => v.id != 1,
            EntityType::Block(_) | EntityType::BlockEnd(_) => false,
            _ => true,
        }) {
            assert!(
                entity.common().invisible,
                "transport {transport}: handle {}",
                entity.common().handle
            );
        }
    }
}
