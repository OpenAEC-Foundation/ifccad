#![allow(dead_code)]
use ocdraw::ifccad::*;

/// The original exact-subset tests continue to exercise explicit rejection.
pub fn to_cad(
    source: &ValidatedIfccad,
) -> Result<ifccad_convert::IfccadToCadOutcome, ifccad_convert::IfccadConversionError> {
    ifccad_convert::ifccad_source_to_cad_document(
        source,
        ifccad_convert::IfccadToCadOptions {
            preservation: Default::default(),
            loss_policy: ifccad_convert::IfccadLossPolicy::Reject,
            geometry_tolerance: ifccad_convert::IfccadGeometryTolerance::exact(),
        },
    )
}
pub fn from_cad(
    source: &opencadcodec::CadDocument,
    metadata: ifccad_convert::IfccadTargetMetadata,
) -> Result<ifccad_convert::CadToEncodedIfccadOutcome, ifccad_convert::IfccadConversionError> {
    ifccad_convert::cad_document_to_encoded_ifccad(
        source,
        metadata,
        ifccad_convert::CadToIfccadOptions {
            hatch_join_tolerance: Default::default(),
            preservation: Default::default(),
            loss_policy: ifccad_convert::IfccadLossPolicy::Reject,
            geometry_tolerance: ifccad_convert::IfccadGeometryTolerance::exact(),
        },
    )
}

pub fn metadata() -> ifccad_convert::IfccadTargetMetadata {
    ifccad_convert::IfccadTargetMetadata {
        header: header(),
        drawing_id: 7,
    }
}

pub fn assert_appearance_mapping(
    source: &IfccadEntityAppearance,
    target: &IfccadEntityAppearance,
    original: &ifccad_convert::IfccadMappings,
    restored: &ifccad_convert::IfccadMappings,
) {
    let mut expected = source.clone();
    if let IfccadMode::Explicit(id) = source.line_pattern {
        let handle = original.line_patterns.cad_handle(id.0).unwrap();
        let restored_id = restored.line_patterns.ifccad_id(handle).unwrap();
        expected.line_pattern = IfccadMode::Explicit(IfccadLinePatternId(restored_id));
    }
    assert_eq!(&expected, target);
}
pub fn header() -> IfccadHeader {
    IfccadHeader {
        id: "proof".into(),
        data_version: "0.1.0".into(),
        author: "conversion test".into(),
        timestamp: "2026-10-01T00:00:00Z".into(),
    }
}
pub fn empty() -> IfccadDocument {
    IfccadDocument {
        text_styles: Vec::new(),
        preservation: None,
        ucs_definitions: vec![],
        model_windows: vec![],
        workspace_state: None,
        model_view_state: None,

        plot_style_mode: Default::default(),
        id_counters: IfccadIdCounters {
            next_preservation_record_id: 1,
            next_layer_id: 5,
            next_layout_id: 2,
            ..Default::default()
        },
        line_patterns: vec![IfccadLinePattern {
            id: IfccadLinePatternId(0),
            name: "Continuous".into(),
            description: Some("Solid line".into()),
            pattern: vec![],
        }],
        line_pattern_scale: 1.,
        header: header(),
        drawing_id: 1,
        length_unit: "mm".into(),
        layers: vec![layer(0, "0"), layer(4, "Notes")],
        model: IfccadLayout {
            bounds_quality: None,
            settings: ocdraw::ifccad::IfccadLayoutSettings {
                media: None,
                ..Default::default()
            },
            bounds: None,
            id: 1,
            tab_index: 0,
            entities: vec![],
        },
        paper_layouts: vec![],
        blocks: vec![],
    }
}
pub fn layer(id: u64, name: &str) -> IfccadLayer {
    IfccadLayer {
        id,
        name: name.into(),
        appearance: IfccadLayerAppearance {
            color: "#FFFFFF".into(),
            opacity: 1.,
            line_pattern: IfccadLinePatternId(0),
            line_weight: 0.25,
        },
    }
}
pub fn validated(doc: &IfccadDocument) -> ValidatedIfccad {
    load_ifccad_bytes(
        encode_ifccad_document(doc).unwrap().bytes(),
        Default::default(),
    )
    .unwrap()
}
pub fn modes() -> IfccadEntityAppearance {
    IfccadEntityAppearance {
        color: IfccadMode::ByLayer,
        opacity: IfccadMode::ByBlock,
        line_pattern: IfccadMode::Explicit(IfccadLinePatternId(0)),
        line_weight: IfccadMode::Explicit(0.25),
    }
}
pub fn placement(origin: [f64; 3]) -> IfccadPlacement {
    IfccadPlacement {
        origin,
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
pub fn primitives() -> IfccadDocument {
    let mut doc = empty();
    doc.id_counters.next_entity_id = 91;
    doc.length_unit = "cm".into();
    doc.model.entities = vec![
        IfccadEntity::Native(IfccadNativeEntity {
            line_pattern_scale: 1.,
            id: 90,
            layer_id: 4,
            appearance: modes(),
            kind: IfccadEntityKind::LineSegment {
                start: [1., 2., 3.],
                end: [4., 5., 6.],
            },
        }),
        IfccadEntity::Native(IfccadNativeEntity {
            line_pattern_scale: 1.,
            id: 2,
            layer_id: 0,
            appearance: IfccadEntityAppearance {
                color: IfccadMode::ByBlock,
                opacity: IfccadMode::Explicit(1.),
                line_pattern: IfccadMode::ByLayer,
                line_weight: IfccadMode::ByBlock,
            },
            kind: {
                let vertices: Vec<[f64; 2]> = vec![[0., 0.], [2., 0.], [2., 4.]];
                IfccadEntityKind::PlanarPolyline {
                    line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
                    bulges: vec![0.; vertices.len()],
                    vertices,
                    closed: true,
                    placement: placement([8., 16., 3.]),
                }
            },
        }),
        IfccadEntity::Native(IfccadNativeEntity {
            line_pattern_scale: 1.,
            id: 41,
            layer_id: 4,
            appearance: IfccadEntityAppearance {
                color: IfccadMode::Explicit("#123456".into()),
                opacity: IfccadMode::ByLayer,
                line_pattern: IfccadMode::ByBlock,
                line_weight: IfccadMode::ByLayer,
            },
            kind: IfccadEntityKind::Circle {
                radius: 2.,
                placement: placement([5., 6., 7.]),
            },
        }),
    ];
    doc
}
#[allow(dead_code)]
pub fn cad() -> opencadcodec::CadDocument {
    ifccad_convert::ifccad_source_to_cad_document(&validated(&empty()), Default::default())
        .unwrap()
        .into_document()
}
pub fn nested(base: [f64; 3]) -> IfccadDocument {
    let mut d = empty();
    d.id_counters.next_entity_id = 9_007_199_254_740_994;
    d.id_counters.next_block_id = 10;
    let line = primitives().model.entities[0].clone();
    d.blocks = vec![
        IfccadBlockDefinition {
            bounds_quality: None,
            bounds: None,
            id: 3,
            name: "Inner".into(),
            base_point: base,
            insertion_unit: "mm".into(),
            entities: vec![line],
        },
        IfccadBlockDefinition {
            bounds_quality: None,
            id: 9,
            bounds: None,
            name: "Outer".into(),
            base_point: [0.; 3],
            insertion_unit: "mm".into(),
            entities: vec![
                instance(15, 3, [0.; 3]),
                IfccadEntity::Native(IfccadNativeEntity {
                    line_pattern_scale: 1.,
                    id: 80,
                    ..primitives().model.entities[0].as_native().unwrap().clone()
                }),
            ],
        },
        IfccadBlockDefinition {
            bounds_quality: None,
            bounds: None,
            id: 4,
            name: "Unused".into(),
            base_point: [0.; 3],
            insertion_unit: "unitless".into(),
            entities: vec![],
        },
    ];
    d.blocks.sort_by_key(|b| b.id);
    d.model.entities = vec![
        instance(9007199254740993, 9, [16., 32., 0.]),
        instance(2, 9, [64., 0., 0.]),
    ];
    let IfccadEntityKind::BlockInstance { transform, .. } =
        &mut d.model.entities[0].as_native_mut().unwrap().kind
    else {
        unreachable!()
    };
    transform.rotation = std::f64::consts::FRAC_PI_2;
    transform.scale[0] = -2.;
    d
}
pub fn instance(id: u64, definition_id: u64, origin: [f64; 3]) -> IfccadEntity {
    IfccadEntity::Native(IfccadNativeEntity {
        line_pattern_scale: 1.,
        id,
        layer_id: 0,
        appearance: modes(),
        kind: IfccadEntityKind::BlockInstance {
            definition_id,
            transform: IfccadBlockTransform {
                placement: placement(origin),
                rotation: 0.,
                scale: [2., 4., 1.],
            },
        },
    })
}

pub fn native_viewport() -> IfccadViewport {
    IfccadViewport {
        workspace: None,

        model_id: 1,
        frame: IfccadViewportFrame {
            center: [100., 75.],
            width: 160.,
            height: 100.,
        },
        view: IfccadViewportView {
            center: [0., 0.],
            target: [0.; 3],
            direction: [0., 0., 100.],
            height: 200.,
            twist: 0.,
            projection: IfccadViewportProjection::Perspective,
            lens_length_mm: Some(50.),
            front_clip: IfccadViewportDepthClip {
                mode: IfccadViewportClipMode::Disabled,
                distance: Some(0.),
            },
            back_clip: IfccadViewportDepthClip {
                mode: IfccadViewportClipMode::Disabled,
                distance: Some(0.),
            },
        },
        render_mode: IfccadViewportRenderMode::Wireframe,
        view_enabled: false,
        view_locked: true,
        visible: false,
        paper_clip: IfccadViewportPaperClip {
            enabled: true,
            boundary_entity_id: Some(1002),
        },
        frozen_layers: vec![4],
    }
}
pub fn viewport_drawing() -> IfccadDocument {
    let mut d = empty();
    d.id_counters.next_entity_id = 1003;
    d.id_counters.next_layout_id = 43;
    let appearance = IfccadEntityAppearance {
        color: IfccadMode::ByLayer,
        opacity: IfccadMode::ByLayer,
        line_weight: IfccadMode::ByLayer,
        line_pattern: IfccadMode::ByLayer,
    };
    let entity = |id, kind| {
        IfccadEntity::Native(IfccadNativeEntity {
            id,
            layer_id: 0,
            appearance: appearance.clone(),
            line_pattern_scale: 1.,
            kind,
        })
    };
    d.paper_layouts.push(IfccadPaperLayout {
        bounds_quality: None,
        canvas: None,

        settings: paper_settings(ocdraw::plot_kernel::PlotUnit::Millimetre, 1.),
        bounds: None,
        id: 42,
        name: "Sheet".into(),
        tab_index: 1,
        entities: vec![
            entity(
                1000,
                IfccadEntityKind::LineSegment {
                    start: [0.; 3],
                    end: [1., 0., 0.],
                },
            ),
            entity(1001, IfccadEntityKind::Viewport(native_viewport())),
            entity(
                1002,
                IfccadEntityKind::Circle {
                    radius: 50.,
                    placement: IfccadPlacement {
                        origin: [100., 75., 0.],
                        x_axis: [1., 0., 0.],
                        y_axis: [0., 1., 0.],
                    },
                },
            ),
        ],
    });
    d
}

/// Explicit physical output for viewport accuracy fixtures; dimensions never rescale geometry.
pub fn paper_settings(
    plot_unit: ocdraw::plot_kernel::PlotUnit,
    output_length: f64,
) -> IfccadLayoutSettings {
    use ocdraw::plot_kernel::*;
    IfccadLayoutSettings {
        media: Some(LayoutMedia {
            unit: MediaUnit::Physical(ocdraw::geometry_kernel::CoordinateLengthUnit::Millimetre),
            width: 1000.,
            height: 1000.,
        }),
        plot_settings: Some(PlotSettings {
            plot_unit,
            page: PlotPage {
                printable_area: PlotRect {
                    min_x: 0.,
                    min_y: 0.,
                    max_x: 1000.,
                    max_y: 1000.,
                },
                rotation: PlotRotation::None,
                device_name: None,
                media_name: None,
            },
            area: PlotArea::Layout,
            mapping: PlotMapping {
                scale: PlotScale::Fixed {
                    output_length,
                    scope_length: 1.,
                },
                placement: PlotPlacement::Offset {
                    reference: PlotOffsetReference::Media,
                    x: 0.,
                    y: 0.,
                },
            },
            output: PlotOutput {
                shaded_plot: ShadedPlot {
                    mode: ShadedPlotMode::AsDisplayed,
                    quality: ShadedPlotQuality {
                        mode: ShadedPlotQualityMode::Normal,
                        dpi: None,
                    },
                },
                apply_plot_styles: false,
                plot_style_table_name: None,
            },
            options: PlotOptions {
                plot_viewport_borders: false,
                plot_paper_space_last: false,
                hide_paper_space_objects: false,
                plot_line_weights: false,
                scale_line_weights: false,
                plot_transparency: false,
            },
        }),
        ..Default::default()
    }
}
