//! Construct independent IFCCAD text in Model, Paper and a local definition.
use ocdraw::{ifccad::*, text::*};
fn placement(origin: [f64; 3]) -> IfccadPlacement {
    IfccadPlacement {
        origin,
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ids = IfccadIdCounters::default();
    let model_id = ids.allocate_layout_id()?;
    let paper_id = ids.allocate_layout_id()?;
    let block_id = ids.allocate_block_id()?;
    let style_id = ids.allocate_text_style_id()?;
    let text = IfccadEntityKind::Text(IfccadText {
        style_id,
        placement: placement([10., 20., 0.]),
        rotation: 0.2,
        backward: false,
        upside_down: false,
        layout: TextLayout::Anchored {
            horizontal: TextHorizontalAlignment::Left,
            vertical: TextVerticalAlignment::Baseline,
            height: 2.5,
            width_factor: 1.,
        },
        oblique_angle: 0.,
        thickness: 0.,
        content: vec![TextRun {
            text: "Hello IFCCAD 世界".into(),
            ..Default::default()
        }],
    });
    let mtext = IfccadEntityKind::MText(Box::new(IfccadMText {
        style_id,
        placement: placement([10., 35., 0.]),
        rotation: 0.,
        backward: false,
        upside_down: false,
        height: 3.,
        attachment: MTextAttachment::TopLeft,
        flow: MTextFlow::Horizontal,
        wrap_width: Some(40.),
        columns: None,
        background: None,
        character_format: CharacterFormat::default(),
        paragraph_format: ParagraphFormat::default(),
        content: vec![
            MTextParagraph {
                character_format: CharacterFormat {
                    color: Some(TextColor::Explicit(IfccadColor::rgb(17, 146, 238))),
                    ..Default::default()
                },
                inlines: vec![MTextInline::Run {
                    text: "Literal text \\P".into(),
                    character_format: CharacterFormat::default(),
                }],
                ..Default::default()
            },
            MTextParagraph {
                inlines: vec![MTextInline::Run {
                    text: "Second paragraph".into(),
                    character_format: Default::default(),
                }],
                ..Default::default()
            },
            MTextParagraph::default(),
        ],
    }));
    let appearance = IfccadEntityAppearance {
        color: IfccadMode::ByLayer,
        opacity: IfccadMode::ByLayer,
        line_pattern: IfccadMode::ByLayer,
        line_weight: IfccadMode::ByLayer,
    };
    let mut entity = |kind| -> Result<IfccadEntity, IfccadIdAllocationError> {
        Ok(IfccadEntity::Native(IfccadNativeEntity {
            visible: true,
            id: ids.allocate_entity_id()?,
            layer_id: 0,
            appearance: appearance.clone(),
            line_pattern_scale: 1.,
            kind,
        }))
    };
    let mut model_entities = vec![entity(text.clone())?, entity(mtext.clone())?];
    model_entities.push(entity(IfccadEntityKind::BlockInstance {
        definition_id: block_id,
        transform: IfccadBlockTransform {
            placement: placement([100., 50., 0.]),
            rotation: 0.3,
            scale: [-1.2, 2., 1.],
        },
    })?);
    let paper_entities = vec![entity(text.clone())?, entity(mtext.clone())?];
    let block_entities = vec![entity(text)?, entity(mtext)?];
    let mut d = IfccadDocument {
        point_display: None,
        ucs_definitions: vec![],
        model_windows: vec![],
        workspace_state: None,
        model_view_state: None,
        header: IfccadHeader {
            id: "ifccad-native-text".into(),
            data_version: "0.1.0".into(),
            author: "OpenAEC".into(),
            timestamp: "2026-10-08T00:00:00Z".into(),
        },
        preservation: None,
        drawing_id: 1,
        id_counters: ids,
        length_unit: "mm".into(),
        plot_style_mode: Default::default(),
        line_pattern_scale: 1.,
        text_styles: vec![IfccadTextStyle {
            id: style_id,
            name: "Labels".into(),
            properties: TextStyleProperties::new(FontRequest {
                cad_font_name: Some("txt".into()),
                ..Default::default()
            }),
        }],
        line_patterns: vec![IfccadLinePattern {
            id: IfccadLinePatternId(0),
            name: "Continuous".into(),
            description: None,
            pattern: vec![],
        }],
        layers: vec![IfccadLayer {
            description: None,
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            id: 0,
            name: "0".into(),
            appearance: IfccadLayerAppearance {
                color: IfccadColor::rgb(255, 255, 255).with_indexed("ACI", 7),
                opacity: 1.,
                line_pattern: IfccadLinePatternId(0),
                line_weight: 0.25,
            },
        }],
        model: IfccadLayout {
            id: model_id,
            tab_index: 0,
            settings: Default::default(),
            bounds: None,
            bounds_quality: None,
            entities: model_entities,
        },
        paper_layouts: vec![IfccadPaperLayout {
            canvas: None,
            id: paper_id,
            name: "Sheet".into(),
            tab_index: 1,
            settings: Default::default(),
            bounds: None,
            bounds_quality: None,
            entities: paper_entities,
        }],
        blocks: vec![IfccadBlockDefinition {
            description: String::new(),
            anonymous: false,
            explodable: true,
            uniform_scaling: false,
            id: block_id,
            name: "LabelsBlock".into(),
            base_point: [0.; 3],
            insertion_unit: "unitless".into(),
            bounds: None,
            bounds_quality: None,
            entities: block_entities,
        }],
    };
    recompute_ifccad_document_bounds(&mut d)?;
    let encoded = encode_ifccad_document(&d)?;
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/ifccad/hello-text.ifcx".into());
    std::fs::write(path, encoded.bytes())?;
    Ok(())
}
