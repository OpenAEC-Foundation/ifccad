//! Typed authoring recipes for the existing public catalogue.
use ifccad::{text::*, *};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn plane(x: f64, y: f64) -> IfccadPlacement {
    IfccadPlacement {
        origin: [x, y, 0.],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
fn add(d: &mut IfccadDocument, kind: IfccadEntityKind) -> Result<u64> {
    let id = d.id_counters.allocate_entity_id()?;
    d.model
        .entities
        .push(IfccadEntity::Native(IfccadNativeEntity {
            id,
            layer_id: 1,
            appearance: IfccadEntityAppearance {
                color: IfccadMode::ByLayer,
                opacity: IfccadMode::ByLayer,
                line_pattern: IfccadMode::ByLayer,
                line_weight: IfccadMode::ByLayer,
            },
            visible: true,
            line_pattern_scale: 1.,
            kind,
        }));
    Ok(id)
}
pub fn append(d: &mut IfccadDocument, cells: &[(&str, f64, f64)]) -> Result<()> {
    let dimension_style = d.dimension_styles[0].id;
    let text_style = d.text_styles[0].id;
    for &(kind, x, y) in cells {
        if kind == "Leader and tolerance" {
            let target = add(
                d,
                IfccadEntityKind::Tolerance(Box::new(IfccadTolerance {
                    placement: plane(x + 42., y + 20.),
                    dimension_style,
                    overrides: Default::default(),
                    notation: IfccadToleranceNotation::CadFeatureControlTextV1 {
                        code: "{\\Fgdt;j}%%v0.1%%vA".into(),
                    },
                    annotation: None,
                    graphics: None,
                })),
            )?;
            for (offset, spline) in [(5., false), (35., true)] {
                let points = vec![[x + 3., y + offset, 0.], [x + 33., y + 20., 0.]];
                add(
                    d,
                    IfccadEntityKind::Leader(Box::new(IfccadLeader {
                        placement: plane(0., 0.),
                        path: if spline {
                            IfccadLeaderPath::SplineThroughVertices(points)
                        } else {
                            IfccadLeaderPath::StraightSegments(points)
                        },
                        annotation_kind: IfccadLeaderAnnotationKind::Tolerance,
                        annotation_entity: Some(target),
                        dimension_style,
                        overrides: Default::default(),
                        arrow: None,
                        arrow_enabled: true,
                        hook_enabled: false,
                        hook_direction: IfccadLeaderHookDirection::AlongHorizontal,
                        annotation_box: Some([0., 0.]),
                        horizontal_direction: [1., 0., 0.],
                        block_offset: [0.; 3],
                        annotation_offset: [0.; 3],
                        origin: [0.; 3],
                        annotation: None,
                        graphics: None,
                    })),
                )?;
            }
        } else if kind == "Multileader" {
            let style_id = d.id_counters.allocate_multi_leader_style_id()?;
            let mut style = IfccadMultiLeaderStyle::new(style_id, "Catalogue leaders");
            style.common.text.style_id = Some(text_style);
            style.common.text.height = 3.;
            style.common.line.color = IfccadMode::ByLayer;
            let common = style.common.clone();
            d.multi_leader_styles.push(style);
            let text = IfccadMText {
                annotation: None,
                style_id: text_style,
                placement: plane(35., 20.),
                rotation: 0.,
                backward: false,
                upside_down: false,
                height: 3.,
                attachment: MTextAttachment::TopLeft,
                flow: MTextFlow::Horizontal,
                wrap_width: Some(45.),
                columns: None,
                background: None,
                character_format: Default::default(),
                paragraph_format: Default::default(),
                content: vec![MTextParagraph {
                    inlines: vec![MTextInline::Run {
                        text: "Owned text".into(),
                        character_format: Default::default(),
                    }],
                    ..Default::default()
                }],
            };
            let state = IfccadMultiLeaderTextState {
                boundary_height: 0.,
                automatic_height: false,
                word_break: true,
                color: IfccadMode::ByLayer,
                attachments: common.text.attachments,
                attachment_point: IfccadMultiLeaderTextAttachmentPoint::Left,
                alignment: IfccadMultiLeaderTextAlignment::Left,
                background: IfccadMultiLeaderTextBackground {
                    enabled: false,
                    mask_enabled: false,
                    color: IfccadMode::ByBlock,
                    scale_factor: 1.5,
                    opacity: IfccadMode::ByLayer,
                },
                stored_rotation: None,
            };
            let lines = [5., 35.]
                .into_iter()
                .enumerate()
                .map(|(i, start)| IfccadMultiLeaderLine {
                    key: i as u64 + 2,
                    source_index: Some(901 + i as i32),
                    settings: common.line.clone(),
                    overridden_properties: Default::default(),
                    vertices: vec![[0., start, 0.], [25., 20., 0.]],
                    connection: IfccadMultiLeaderConnection::GroupConnection,
                    breaks: vec![],
                })
                .collect();
            let group = IfccadMultiLeaderGroup {
                key: 1,
                source_index: Some(47),
                has_connection_point: true,
                connection_point: [30., 20., 0.],
                has_dogleg_direction: true,
                dogleg_direction: [1., 0., 0.],
                dogleg_length: 5.,
                text_attachment_direction: IfccadMultiLeaderAttachmentDirection::Horizontal,
                lines,
                breaks: vec![],
            };
            let commands = vec![
                IfccadLeaderGraphicCommand::SetLayer(Some(1)),
                IfccadLeaderGraphicCommand::SetColor(IfccadLeaderGraphicInheritance::ByLayer),
                IfccadLeaderGraphicCommand::Draw(IfccadLeaderGraphicPrimitive::Polyline {
                    vertices: vec![
                        [x, y + 5., 0.],
                        [x + 25., y + 20., 0.],
                        [x + 35., y + 20., 0.],
                    ],
                    normal: None,
                }),
                IfccadLeaderGraphicCommand::Draw(IfccadLeaderGraphicPrimitive::Polyline {
                    vertices: vec![[x, y + 35., 0.], [x + 25., y + 20., 0.]],
                    normal: None,
                }),
                IfccadLeaderGraphicCommand::Draw(IfccadLeaderGraphicPrimitive::Text(Box::new(
                    IfccadLeaderGraphicText {
                        position: [x + 35., y + 20., 0.],
                        normal: [0., 0., 1.],
                        direction: [1., 0., 0.],
                        text: "Owned text".into(),
                        processing: IfccadLeaderGraphicTextProcessing::Interpreted,
                        height: 3.,
                        width_factor: 1.,
                        oblique_angle: 0.,
                        tracking: 1.,
                        backwards: false,
                        upside_down: false,
                        vertical: false,
                        underline: false,
                        overline: false,
                        font: None,
                    },
                ))),
            ];
            add(
                d,
                IfccadEntityKind::MultiLeader(Box::new(IfccadMultiLeader {
                    style_id: Some(style_id),
                    settings: IfccadMultiLeaderSettings {
                        common,
                        text_attachment_point: IfccadMultiLeaderTextAttachmentPoint::Left,
                        text_direction_negative: false,
                        extend_leader_to_text: false,
                    },
                    overridden_properties: Default::default(),
                    block_labels: vec![],
                    next_label_id: 1,
                    legacy_arrow_overrides: vec![],
                    presentation: IfccadMultiLeaderPresentation {
                        plane: plane(x, y),
                        normal_reversed: false,
                        content_base_point: [35., 20., 0.],
                        scale_factor: 1.,
                        text_height: 3.,
                        arrow_size: 2.5,
                        landing_gap: 1.,
                        next_part_id: 4,
                        groups: vec![group],
                        content: IfccadMultiLeaderStoredContent {
                            m_text: Some(Box::new(IfccadMultiLeaderTextContent { text, state })),
                            ..Default::default()
                        },
                    },
                    annotation: None,
                    graphics: Some(IfccadLeaderGraphics {
                        origin: IfccadLeaderGraphicsOrigin::Generated,
                        placement: plane(0., 0.),
                        commands,
                    }),
                })),
            )?;
        }
    }
    recompute_ifccad_document_bounds(d)?;
    Ok(())
}
