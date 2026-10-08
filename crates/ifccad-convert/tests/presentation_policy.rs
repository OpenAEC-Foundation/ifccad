mod common;
#[allow(dead_code)]
mod spline_support;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{Color, EntityType, Line, LineWeight, Transparency, Vector3};

#[test]
fn source_aci_and_named_identity_survive_layer_and_entity_native_colors() {
    let mut source = cad();
    let layer = source.layers.get_mut("0").unwrap();
    layer.color = Color::Index(1);
    layer.book_name = Some("Example".into());
    layer.color_name = Some("Red".into());
    let mut line = Line::new();
    line.end = Vector3::new(1., 2., 0.);
    line.common.color = Color::Index(2);
    line.common.color_name = Some("Example$Yellow".into());
    source.add_entity(EntityType::Line(line)).unwrap();
    let output = cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
    let layer = output
        .document()
        .layers
        .iter()
        .find(|l| l.name == "0")
        .unwrap();
    assert_eq!(
        layer.appearance.color,
        IfccadColor::rgb(255, 0, 0)
            .with_indexed("ACI", 1)
            .with_named("Example", "Red")
    );
    let expected = IfccadColor::rgb(255, 255, 0)
        .with_indexed("ACI", 2)
        .with_named("Example", "Yellow");
    assert_eq!(
        output.document().model.entities[0]
            .as_native()
            .unwrap()
            .appearance
            .color,
        IfccadMode::Explicit(expected)
    );
    assert!(
        !output
            .diagnostics()
            .iter()
            .any(|d| d.location == "layer/0.book_name" || d.location == "layer/0.color_name"),
        "mapped named identity still classified as loss"
    );
    let target = ifccad_document_to_cad_document(output.document(), Default::default()).unwrap();
    let line = target
        .document()
        .model_space_entities()
        .next()
        .unwrap()
        .common();
    assert_eq!(line.color, Color::Index(2));
    assert_eq!(line.color_name.as_deref(), Some("Example$Yellow"));
}

#[test]
fn inline_aci_and_background_color_keep_identity_in_mtext_exchange() {
    use ocdraw::text::{MTextFill, MTextInline, TextColor};
    let mut source = cad();
    let mut text = opencadcodec::MText::new();
    text.height = 3.;
    text.value = "\\C1;Red".into();
    text.background_fill_flags = 1;
    text.background_color = Color::Index(2);
    source.add_entity(EntityType::MText(text)).unwrap();
    let native = cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
    let IfccadEntityKind::MText(text) = &native.document().model.entities[0]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!()
    };
    let expected = Some(TextColor::Explicit(
        IfccadColor::rgb(255, 0, 0).with_indexed("ACI", 1),
    ));
    let colors = std::iter::once(&text.character_format.color)
        .chain(text.content.iter().map(|p| &p.character_format.color))
        .chain(
            text.content
                .iter()
                .flat_map(|p| p.inlines.iter())
                .filter_map(|i| match i {
                    MTextInline::Run {
                        character_format, ..
                    } => Some(&character_format.color),
                    _ => None,
                }),
        );
    assert!(
        colors.into_iter().any(|c| c == &expected),
        "ACI identity missing from factored native character color: {text:?}"
    );
    assert!(
        matches!(&text.background.as_ref().unwrap().fill,MTextFill::Color(c) if c.indexed == Some(("ACI".into(),2)))
    );
    let target = ifccad_document_to_cad_document(native.document(), Default::default()).unwrap();
    let EntityType::MText(text) = target.document().model_space_entities().next().unwrap() else {
        panic!()
    };
    assert!(text.value.contains("\\C1;"));
    assert_eq!(text.background_color, Color::Index(2));
}

#[test]
fn incompatible_aci_never_changes_rgb_and_reject_refuses_identity_loss() {
    let mut native = primitives();
    native.layers[0].appearance.color = IfccadColor::rgb(12, 34, 56).with_indexed("ACI", 1);
    let output = ifccad_document_to_cad_document(&native, Default::default()).unwrap();
    assert_eq!(
        output.document().layers.get("0").unwrap().color,
        Color::from_rgb(12, 34, 56)
    );
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.location.contains("color") && d.is_semantic_loss()));
    assert!(ifccad_document_to_cad_document(
        &native,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn every_cad_transparency_byte_stays_exact_in_layer_and_entity_roundtrips() {
    for byte in 0u8..=255 {
        let mut source = cad();
        source.layers.get_mut("0").unwrap().transparency = Transparency::Explicit(byte);
        let mut line = Line::new();
        line.end = Vector3::new(1., 2., 0.);
        line.common.transparency = Transparency::Explicit(byte);
        source.add_entity(EntityType::Line(line)).unwrap();
        let native =
            cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
        let target =
            ifccad_document_to_cad_document(native.document(), Default::default()).unwrap();
        assert_eq!(
            target.document().layers.get("0").unwrap().transparency,
            Transparency::Explicit(byte)
        );
        assert_eq!(
            target
                .document()
                .model_space_entities()
                .next()
                .unwrap()
                .common()
                .transparency,
            Transparency::Explicit(byte)
        );
        assert!(!target
            .diagnostics()
            .iter()
            .any(|d| d.location.contains("opacity")));
    }
}

#[test]
fn authored_opacity_and_weight_use_shared_quantization_with_located_loss() {
    for (opacity, byte) in [(0.5, 128), (0.999, 1)] {
        let mut native = primitives();
        native.layers[0].appearance.opacity = opacity;
        native.layers[0].appearance.line_weight = 0.225;
        let entity = native.model.entities[0].as_native_mut().unwrap();
        entity.appearance.opacity = IfccadMode::Explicit(opacity);
        entity.appearance.line_weight = IfccadMode::Explicit(0.225);
        let output = ifccad_document_to_cad_document(&native, Default::default()).unwrap();
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
            .any(|d| d.location.contains("opacity") && d.is_semantic_loss()));
        assert!(output
            .diagnostics()
            .iter()
            .any(|d| d.location.contains("line_weight") && d.is_semantic_loss()));
        assert!(ifccad_document_to_cad_document(
            &native,
            IfccadToCadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}

#[test]
fn unsupported_required_color_does_not_invent_white_or_dangling_layer_refs() {
    let mut source = cad();
    source.layers.get_mut("Notes").unwrap().color = Color::None;
    let mut good = Line::new();
    good.end = Vector3::new(1., 2., 0.);
    source.add_entity(EntityType::Line(good)).unwrap();
    let mut missing_layer = Line::new();
    missing_layer.common.layer = "Notes".into();
    missing_layer.end = Vector3::new(3., 4., 0.);
    let missing = source.add_entity(EntityType::Line(missing_layer)).unwrap();
    let mut no_color = Line::new();
    no_color.common.color = Color::None;
    no_color.end = Vector3::new(5., 6., 0.);
    let unsupported = source.add_entity(EntityType::Line(no_color)).unwrap();
    let output = cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
    assert!(output.document().layers.iter().all(|l| l.name != "Notes"));
    assert_eq!(output.document().model.entities.len(), 1);
    assert!(output.mappings().entities.ifccad_id(missing).is_none());
    assert!(output.mappings().entities.ifccad_id(unsupported).is_none());
    validate_ifccad_document(output.document()).unwrap();
}

#[test]
fn qualified_indexed_spline_common_stays_editable_without_rewriting_snapshot_bytes() {
    let mut source = cad();
    let mut spline = spline_support::spline();
    spline.common.color = Color::Index(1);
    spline.common.color_name = Some("Example$Red".into());
    source.add_entity(EntityType::Spline(spline)).unwrap();
    let captured = cad_document_to_ifccad_document(
        &source,
        metadata(),
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    let mut document = captured.document().clone();
    let payload = document.preservation.as_ref().unwrap().records[0]
        .payload
        .clone();
    let appearance = document.model.entities[0]
        .as_opaque_mut()
        .unwrap()
        .appearance
        .as_mut()
        .expect("qualified ACI common appearance unavailable");
    assert_eq!(
        appearance.appearance.color,
        IfccadMode::Explicit(
            IfccadColor::rgb(255, 0, 0)
                .with_indexed("ACI", 1)
                .with_named("Example", "Red")
        )
    );
    appearance.appearance.opacity = IfccadMode::Explicit(0.999);
    let output = ifccad_document_to_cad_document(&document, Default::default()).unwrap();
    let EntityType::Spline(spline) = output.document().model_space_entities().next().unwrap()
    else {
        panic!()
    };
    assert_eq!(spline.common.color, Color::Index(1));
    assert_eq!(spline.common.color_name.as_deref(), Some("Example$Red"));
    assert_eq!(spline.common.transparency, Transparency::Explicit(1));
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.location.contains("opacity") && d.is_semantic_loss()));
    assert_eq!(
        document.preservation.as_ref().unwrap().records[0].payload,
        payload
    );
}

#[test]
fn layer_catalog_delimiter_is_diagnosed_before_portable_cad_projection() {
    let mut drawing = common::empty();
    drawing.layers[0].appearance.color =
        IfccadColor::rgb(255, 0, 0).with_named("Book$Series", "Red");
    let output = ifccad_document_to_cad_document(&drawing, Default::default()).unwrap();
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
    assert!(ifccad_document_to_cad_document(
        &drawing,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}
