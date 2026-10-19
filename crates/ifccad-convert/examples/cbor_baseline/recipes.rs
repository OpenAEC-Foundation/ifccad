use super::Result;
use ifccad::{geometry_kernel::hatch::*, text::*, *};
pub fn header() -> IfccadHeader {
    IfccadHeader {
        id: "cbor-baseline-v1".into(),
        data_version: "0.1.0".into(),
        author: "OpenAEC baseline".into(),
        timestamp: "2026-10-16T00:00:00Z".into(),
    }
}
fn plane(x: f64, y: f64) -> IfccadPlacement {
    IfccadPlacement {
        origin: [x, y, 0.],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
pub fn drawing(family: &str, count: usize) -> Result<IfccadDocument> {
    let mut ids = IfccadIdCounters::default();
    let pattern = ids.allocate_line_pattern_id()?;
    let mut layers = Vec::new();
    for i in 0..4 {
        layers.push(IfccadLayer {
            id: ids.allocate_layer_id()?,
            name: format!("Layer-{i}"),
            description: None,
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            appearance: IfccadLayerAppearance {
                color: IfccadColor::rgb(10, 20, 30),
                opacity: 1.,
                line_pattern: pattern,
                line_weight: 0.25,
            },
        });
    }
    let layout = ids.allocate_layout_id()?;
    let mut d = IfccadDocument {
        header: header(),
        drawing_id: 7,
        id_counters: ids,
        length_unit: "mm".into(),
        plot_style_mode: Default::default(),
        layers,
        line_patterns: vec![IfccadLinePattern {
            id: pattern,
            name: "Continuous".into(),
            description: None,
            pattern: vec![],
        }],
        line_pattern_scale: 1.,
        model: IfccadLayout {
            id: layout,
            tab_index: 0,
            bounds: None,
            bounds_quality: None,
            settings: Default::default(),
            entities: vec![],
        },
        paper_layouts: vec![],
        blocks: vec![],
        dimension_styles: vec![],
        symbol_sources: vec![],
        point_display: None,
        text_styles: vec![],
        ucs_definitions: vec![],
        model_windows: vec![],
        workspace_state: None,
        model_view_state: None,
        preservation: None,
    };
    if family == "text" {
        for name in ["Regular", "Emphasis"] {
            d.text_styles.push(IfccadTextStyle {
                id: d.id_counters.allocate_text_style_id()?,
                name: name.into(),
                properties: TextStyleProperties::new(FontRequest::family("Arial")),
            });
        }
    }
    for i in 0..count {
        let x = ((i % 100) * 32) as f64;
        let y = ((i / 100) * 32) as f64;
        let kind = match family {
            "lines" => IfccadEntityKind::LineSegment {
                start: [x, y, 0.],
                end: [x + 8., y + 4., 0.],
            },
            "polylines" => {
                let closed = i % 2 == 0;
                IfccadEntityKind::PlanarPolyline {
                    vertices: vec![[x, y], [x + 8., y], [x + 8., y + 4.], [x, y + 4.]],
                    bulges: vec![0.; 4],
                    closed,
                    placement: plane(0., 0.),
                    line_pattern_generation: Default::default(),
                }
            }
            "text" if i % 2 == 0 => IfccadEntityKind::Text(IfccadText {
                style_id: d.text_styles[i % 2].id,
                placement: plane(x, y),
                rotation: 0.,
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
                    text: format!("Fundering {i:06} \u{00b5}m"),
                    underline: i % 4 == 0,
                    ..Default::default()
                }],
            }),
            "text" => IfccadEntityKind::MText(Box::new(IfccadMText {
                style_id: d.text_styles[i % 2].id,
                placement: plane(x, y),
                rotation: 0.,
                backward: false,
                upside_down: false,
                height: 2.5,
                attachment: Default::default(),
                flow: Default::default(),
                wrap_width: Some(24.),
                columns: None,
                background: None,
                character_format: Default::default(),
                paragraph_format: Default::default(),
                content: (0..3)
                    .map(|j| MTextParagraph {
                        inlines: vec![
                            MTextInline::Run {
                                text: format!("Niveau {i:06}/{j}: "),
                                character_format: Default::default(),
                            },
                            MTextInline::Run {
                                text: "beton \u{00e9}n staal".into(),
                                character_format: CharacterFormat {
                                    underline: Some(true),
                                    ..Default::default()
                                },
                            },
                        ],
                        ..Default::default()
                    })
                    .collect(),
            })),
            "hatches" => {
                let gradient = (i % 3 == 2).then(|| HatchGradient {
                    gradient_type: Some(HatchGradientType::Linear),
                    colors: Some([
                        IfccadColor::rgb(20, 40, 60),
                        IfccadColor::rgb(180, 140, 100),
                    ]),
                    rotation: 0.,
                    shift: 0.,
                    color_mode: HatchGradientColorMode::TwoColor,
                    tint: 0.5,
                });
                let fill = match i % 3 {
                    0 => HatchFill::Solid,
                    1 => HatchFill::LinePattern(HatchLinePattern {
                        name: Some("Baseline".into()),
                        description: None,
                        origin: [0., 0.],
                        rotation: 0.,
                        scale: 1.,
                        families: vec![HatchLineFamily {
                            angle: 0.,
                            base_point: [0., 0.],
                            offset: [0., 2.],
                            dashes: vec![
                                HatchDash::Dash { length: 2. },
                                HatchDash::Gap { length: 1. },
                            ],
                        }],
                    }),
                    _ => HatchFill::Gradient,
                };
                let mut loops = vec![IfccadHatchLoop {
                    boundary: HatchBoundary2::Polyline {
                        vertices: vec![[-3., -3.], [3., -3.], [3., 3.], [-3., 3.]],
                        bulges: vec![0.; 4],
                    },
                    source_entity_ids: vec![],
                    provenance: None,
                    text_island: false,
                }];
                if i % 2 == 0 {
                    loops.push(IfccadHatchLoop {
                        boundary: HatchBoundary2::Circle {
                            center: [0., 0.],
                            radius: 1.,
                        },
                        source_entity_ids: vec![],
                        provenance: None,
                        text_island: false,
                    });
                }
                IfccadEntityKind::Hatch(IfccadHatch {
                    placement: plane(x, y),
                    gradient,
                    fill,
                    association_enabled: false,
                    loops,
                    area_rule: HatchAreaRule::Normal,
                    join_tolerance: 1e-9,
                })
            }
            _ => return Err(format!("unknown recipe {family}").into()),
        };
        d.model
            .entities
            .push(IfccadEntity::Native(IfccadNativeEntity {
                id: d.id_counters.allocate_entity_id()?,
                visible: true,
                layer_id: d.layers[i % 4].id,
                line_pattern_scale: 1.,
                appearance: IfccadEntityAppearance {
                    color: if i % 3 == 0 {
                        IfccadMode::Explicit(IfccadColor::rgb(180, 40, 60))
                    } else {
                        IfccadMode::ByLayer
                    },
                    opacity: IfccadMode::ByLayer,
                    line_pattern: IfccadMode::ByLayer,
                    line_weight: IfccadMode::ByLayer,
                },
                kind,
            }));
    }
    validate_ifccad_document(&d)?;
    Ok(d)
}
