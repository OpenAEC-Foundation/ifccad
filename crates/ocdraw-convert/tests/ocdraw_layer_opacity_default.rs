use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::{
    CadDocument, Color, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Layer, Line,
    LineWeight, Text, Transparency, Vector3,
};
use std::io::Cursor;

fn source(transparency: Transparency) -> CadDocument {
    let mut d = CadDocument::new();
    let mut layer = Layer::new("Coloured");
    layer.color = Color::Rgb {
        r: 17,
        g: 146,
        b: 238,
    };
    layer.transparency = transparency;
    layer.handle = d.allocate_handle();
    d.layers.add(layer).unwrap();
    let mut t = Text::with_value("Coloured text", Vector3::ZERO);
    t.common.layer = "Coloured".into();
    t.common.line_weight = LineWeight::ByLayer;
    d.add_entity(EntityType::Text(t)).unwrap();
    let mut line = Line::new();
    line.end = Vector3::new(1., 1., 0.);
    line.common.layer = "Coloured".into();
    line.common.line_weight = LineWeight::ByLayer;
    d.add_entity(EntityType::Line(line)).unwrap();
    d
}

#[test]
fn default_layer_retains_colour_entities_and_unused_layers_in_strict_native_output() {
    let mut d = source(Transparency::ByLayer);
    let mut unused = Layer::new("Unused default");
    unused.transparency = Transparency::ByLayer;
    unused.handle = d.allocate_handle();
    d.layers.add(unused).unwrap();
    let out = cad_document_to_ocdraw_document(
        &d,
        CadToOcdrawOptions {
            loss_policy: OcdrawLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    let layer = out
        .document()
        .layers
        .iter()
        .find(|l| l.name == "Coloured")
        .unwrap();
    assert_eq!(layer.opacity, 1.);
    assert_eq!(layer.color.rgb, [17, 146, 238]);
    assert_eq!(
        out.document()
            .layers
            .iter()
            .find(|l| l.name == "Unused default")
            .unwrap()
            .opacity,
        1.
    );
    assert_eq!(out.document().text_entities.len(), 1);
    assert_eq!(out.entity_mapping().len(), 2);
    assert_eq!(
        out.document().text_entities[0].appearance.opacity,
        AppearanceSelection::ByLayer
    );
    let encoded = encode_ocdraw_document(out.document()).unwrap();
    let strict = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(strict.document().layers, out.document().layers);
    assert_eq!(strict.text_entities(), out.document().text_entities);
}

#[test]
fn explicit_layer_alpha_and_entity_opacity_modes_remain_distinct() {
    for transparency in [
        Transparency::ByLayer,
        Transparency::ByBlock,
        Transparency::Explicit(0),
    ] {
        let mut d = source(Transparency::Explicit(128));
        for e in d.entities_mut() {
            e.common_mut().transparency = transparency;
        }
        let out = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
        assert_eq!(
            out.document()
                .layers
                .iter()
                .find(|l| l.name == "Coloured")
                .unwrap()
                .opacity,
            127. / 255.
        );
        let expected = match transparency {
            Transparency::ByLayer => AppearanceSelection::ByLayer,
            Transparency::ByBlock => AppearanceSelection::ByBlock,
            Transparency::Explicit(_) => AppearanceSelection::Explicit(1.),
        };
        assert_eq!(out.document().text_entities[0].appearance.opacity, expected);
        let encoded = encode_ocdraw_document(out.document()).unwrap();
        load_ocdraw_bytes(encoded.bytes()).unwrap();
    }
}

#[test]
fn byblock_on_a_layer_stays_diagnosed_and_rejectable() {
    let d = source(Transparency::ByBlock);
    let out = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    assert!(!out.document().layers.iter().any(|l| l.name == "Coloured"));
    assert!(out.entity_mapping().is_empty());
    assert!(out.diagnostics().iter().any(
        |d| matches!(d.source(), CadToOcdrawDiagnosticSource::Layer { name } if name == "Coloured")
            && d.reasons()
                .contains(&CadToOcdrawLossReason::LayerTransparencyUnsupported)
    ));
    assert!(matches!(
        cad_document_to_ocdraw_document(
            &d,
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}

#[test]
fn default_layer_survives_actual_dxf_and_dwg_import_and_canonical_export() {
    let d = source(Transparency::ByLayer);
    for dwg in [false, true] {
        let loaded = if dwg {
            DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&d).unwrap()))
                .read()
                .unwrap()
        } else {
            DxfReader::from_reader(Cursor::new(DxfWriter::new(&d).write_to_vec().unwrap()))
                .unwrap()
                .read()
                .unwrap()
        };
        assert_eq!(
            loaded.layers.get("Coloured").unwrap().transparency,
            Transparency::ByLayer
        );
        let native = cad_document_to_ocdraw_document(&loaded, Default::default()).unwrap();
        let encoded = encode_ocdraw_document(native.document()).unwrap();
        let strict = load_ocdraw_bytes(encoded.bytes()).unwrap();
        let exported =
            ocdraw_document_to_cad_document(strict.document(), Default::default()).unwrap();
        assert_eq!(
            exported
                .document()
                .layers
                .get("Coloured")
                .unwrap()
                .transparency,
            Transparency::Explicit(0)
        );
        assert_eq!(native.document().text_entities.len(), 1);
        let back = if dwg {
            DwgReader::from_stream(Cursor::new(
                DwgWriter::write_to_vec(exported.document()).unwrap(),
            ))
            .read()
            .unwrap()
        } else {
            DxfReader::from_reader(Cursor::new(
                DxfWriter::new(exported.document()).write_to_vec().unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap()
        };
        let native_back = cad_document_to_ocdraw_document(&back, Default::default()).unwrap();
        let layer = native_back
            .document()
            .layers
            .iter()
            .find(|l| l.name == "Coloured")
            .unwrap();
        assert_eq!(layer.opacity, 1.);
        assert_eq!(layer.color.rgb, [17, 146, 238]);
        assert_eq!(
            native_back.document().text_entities[0].content,
            native.document().text_entities[0].content
        );
        let encoded = encode_ocdraw_document(native_back.document()).unwrap();
        load_ocdraw_bytes(encoded.bytes()).unwrap();
    }
}
