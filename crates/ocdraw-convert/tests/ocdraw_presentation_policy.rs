use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::{CadDocument, Color, EntityType, Line, LineWeight, Transparency, Vector3};

fn native() -> OcdrawDocument {
    let mut source = CadDocument::new();
    source.layers.get_mut("0").unwrap().color = Color::from_rgb(255, 255, 255);
    let mut line = Line::new();
    line.end = Vector3::new(1., 2., 0.);
    source.add_entity(EntityType::Line(line)).unwrap();
    cad_document_to_ocdraw_document(&source, Default::default())
        .unwrap()
        .into_document()
}

#[test]
fn incompatible_aci_never_changes_rgb_and_reject_refuses_identity_loss() {
    let mut drawing = native();
    drawing
        .layers
        .iter_mut()
        .find(|l| l.name == "0")
        .unwrap()
        .color = DrawingColor::rgb(12, 34, 56).with_indexed("ACI", 1);
    let output = ocdraw_document_to_cad_document(&drawing, Default::default()).unwrap();
    assert_eq!(
        output.document().layers.get("0").unwrap().color,
        Color::from_rgb(12, 34, 56)
    );
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.location.contains("color")));
    assert!(ocdraw_document_to_cad_document(
        &drawing,
        OcdrawToCadOptions {
            loss_policy: OcdrawLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn every_cad_transparency_byte_stays_exact_in_layer_and_entity_roundtrips() {
    for byte in 0u8..=255 {
        let mut source = CadDocument::new();
        let layer = source.layers.get_mut("0").unwrap();
        layer.color = Color::from_rgb(255, 255, 255);
        layer.transparency = Transparency::Explicit(byte);
        let mut line = Line::new();
        line.end = Vector3::new(1., 2., 0.);
        line.common.transparency = Transparency::Explicit(byte);
        source.add_entity(EntityType::Line(line)).unwrap();
        let native = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
        let output =
            ocdraw_document_to_cad_document(native.document(), Default::default()).unwrap();
        assert_eq!(
            output.document().layers.get("0").unwrap().transparency,
            Transparency::Explicit(byte)
        );
        assert_eq!(
            output
                .document()
                .model_space_entities()
                .next()
                .unwrap()
                .common()
                .transparency,
            Transparency::Explicit(byte)
        );
        assert!(!output
            .diagnostics()
            .iter()
            .any(|d| d.location.contains("opacity")));
    }
}

#[test]
fn authored_opacity_and_weight_use_shared_quantization_with_located_loss() {
    for (opacity, byte) in [(0.5, 128), (0.999, 1)] {
        let mut drawing = native();
        let layer = drawing.layers.iter_mut().find(|l| l.name == "0").unwrap();
        layer.opacity = opacity;
        layer.line_weight = 0.225;
        let appearance = &mut drawing.geometric_entities[0].appearance;
        appearance.opacity = AppearanceSelection::Explicit(opacity);
        appearance.line_weight = AppearanceSelection::Explicit(0.225);
        let output = ocdraw_document_to_cad_document(&drawing, Default::default()).unwrap();
        let layer = output.document().layers.get("0").unwrap();
        assert_eq!(layer.transparency, Transparency::Explicit(byte));
        assert_eq!(layer.line_weight, LineWeight::Value(20));
        let entity = output
            .document()
            .model_space_entities()
            .next()
            .unwrap()
            .common();
        assert_eq!(entity.transparency, Transparency::Explicit(byte));
        assert_eq!(entity.line_weight, LineWeight::Value(20));
        assert!(output
            .diagnostics()
            .iter()
            .any(|d| d.location.contains("opacity")));
        assert!(output
            .diagnostics()
            .iter()
            .any(|d| d.location.contains("lineWeight")));
        assert!(ocdraw_document_to_cad_document(
            &drawing,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}

#[test]
fn layer_catalog_delimiter_is_diagnosed_before_portable_cad_projection() {
    let mut drawing = native();
    drawing.layers[0].color = DrawingColor::rgb(255, 0, 0).with_named("Book$Series", "Red");
    let output = ocdraw_document_to_cad_document(&drawing, Default::default()).unwrap();
    let layer = output
        .document()
        .layers
        .get(&drawing.layers[0].name)
        .unwrap();
    assert_eq!(layer.color.rgb(), Some((255, 0, 0)));
    assert!(layer.book_name.is_none() && layer.color_name.is_none());
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.location.contains("color")));
    assert!(ocdraw_document_to_cad_document(
        &drawing,
        OcdrawToCadOptions {
            loss_policy: OcdrawLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}
