#![allow(dead_code)]
use ocdraw::ifcx_cad::*;

/// The original exact-subset tests continue to exercise explicit rejection.
pub fn to_cad(
    source: &ValidatedIfcxCad,
) -> Result<ifcx_cad_convert::IfcxCadToCadOutcome, ifcx_cad_convert::IfcxCadConversionError> {
    ifcx_cad_convert::ifcx_cad_source_to_cad_document(
        source,
        ifcx_cad_convert::IfcxCadToCadOptions {
            loss_policy: ifcx_cad_convert::IfcxCadLossPolicy::Reject,
        },
    )
}
pub fn from_cad(
    source: &opencadcodec::CadDocument,
    metadata: ifcx_cad_convert::IfcxCadTargetMetadata,
) -> Result<ifcx_cad_convert::CadToEncodedIfcxCadOutcome, ifcx_cad_convert::IfcxCadConversionError>
{
    ifcx_cad_convert::cad_document_to_encoded_ifcx_cad(
        source,
        metadata,
        ifcx_cad_convert::CadToIfcxCadOptions {
            loss_policy: ifcx_cad_convert::IfcxCadLossPolicy::Reject,
        },
    )
}

pub fn metadata() -> ifcx_cad_convert::IfcxCadTargetMetadata {
    ifcx_cad_convert::IfcxCadTargetMetadata {
        header: header(),
        drawing_id: 7,
    }
}

pub fn assert_appearance_mapping(
    source: &IfcxCadEntityAppearance,
    target: &IfcxCadEntityAppearance,
    original: &ifcx_cad_convert::IfcxCadMappings,
    restored: &ifcx_cad_convert::IfcxCadMappings,
) {
    let mut expected = source.clone();
    if let IfcxCadMode::Explicit(id) = source.line_pattern {
        let handle = original.line_patterns.cad_handle(id.0).unwrap();
        let restored_id = restored.line_patterns.ifcx_id(handle).unwrap();
        expected.line_pattern = IfcxCadMode::Explicit(IfcxCadLinePatternId(restored_id));
    }
    assert_eq!(&expected, target);
}
pub fn header() -> IfcxCadHeader {
    IfcxCadHeader {
        id: "proof".into(),
        data_version: "0.1.0".into(),
        author: "conversion test".into(),
        timestamp: "2026-10-01T00:00:00Z".into(),
    }
}
pub fn empty() -> IfcxCadDocument {
    IfcxCadDocument {
        id_counters: IfcxCadIdCounters {
            next_layer_id: 5,
            next_layout_id: 2,
            ..Default::default()
        },
        line_patterns: vec![IfcxCadLinePattern {
            id: IfcxCadLinePatternId(0),
            name: "Continuous".into(),
            description: Some("Solid line".into()),
            pattern: vec![],
        }],
        line_pattern_scale: 1.,
        header: header(),
        drawing_id: 1,
        length_unit: "mm".into(),
        layers: vec![layer(0, "0"), layer(4, "Notes")],
        model: IfcxCadLayout {
            id: 1,
            tab_index: 0,
            entities: vec![],
        },
        paper_layouts: vec![],
        blocks: vec![],
    }
}
pub fn layer(id: u64, name: &str) -> IfcxCadLayer {
    IfcxCadLayer {
        id,
        name: name.into(),
        appearance: IfcxCadLayerAppearance {
            color: "#FFFFFF".into(),
            opacity: 1.,
            line_pattern: IfcxCadLinePatternId(0),
            line_weight: 0.25,
        },
    }
}
pub fn validated(doc: &IfcxCadDocument) -> ValidatedIfcxCad {
    load_ifcx_cad_bytes(
        encode_ifcx_cad_document(doc).unwrap().bytes(),
        Default::default(),
    )
    .unwrap()
}
pub fn modes() -> IfcxCadEntityAppearance {
    IfcxCadEntityAppearance {
        color: IfcxCadMode::ByLayer,
        opacity: IfcxCadMode::ByBlock,
        line_pattern: IfcxCadMode::Explicit(IfcxCadLinePatternId(0)),
        line_weight: IfcxCadMode::Explicit(0.25),
    }
}
pub fn placement(origin: [f64; 3]) -> IfcxCadPlacement {
    IfcxCadPlacement {
        origin,
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
pub fn primitives() -> IfcxCadDocument {
    let mut doc = empty();
    doc.id_counters.next_entity_id = 91;
    doc.length_unit = "cm".into();
    doc.model.entities = vec![
        IfcxCadEntity {
            line_pattern_scale: 1.,
            id: 90,
            layer_id: 4,
            appearance: modes(),
            kind: IfcxCadEntityKind::LineSegment {
                start: [1., 2., 3.],
                end: [4., 5., 6.],
            },
        },
        IfcxCadEntity {
            line_pattern_scale: 1.,
            id: 2,
            layer_id: 0,
            appearance: IfcxCadEntityAppearance {
                color: IfcxCadMode::ByBlock,
                opacity: IfcxCadMode::Explicit(1.),
                line_pattern: IfcxCadMode::ByLayer,
                line_weight: IfcxCadMode::ByBlock,
            },
            kind: IfcxCadEntityKind::PlanarPolyline {
                line_pattern_generation: IfcxCadLinePatternGeneration::PerSegment,
                vertices: vec![[0., 0.], [2., 0.], [2., 4.]],
                closed: true,
                placement: placement([8., 16., 3.]),
            },
        },
        IfcxCadEntity {
            line_pattern_scale: 1.,
            id: 41,
            layer_id: 4,
            appearance: IfcxCadEntityAppearance {
                color: IfcxCadMode::Explicit("#123456".into()),
                opacity: IfcxCadMode::ByLayer,
                line_pattern: IfcxCadMode::ByBlock,
                line_weight: IfcxCadMode::ByLayer,
            },
            kind: IfcxCadEntityKind::Circle {
                radius: 2.,
                placement: placement([5., 6., 7.]),
            },
        },
    ];
    doc
}
#[allow(dead_code)]
pub fn cad() -> opencadcodec::CadDocument {
    ifcx_cad_convert::ifcx_cad_source_to_cad_document(&validated(&empty()), Default::default())
        .unwrap()
        .into_document()
}
pub fn nested(base: [f64; 3]) -> IfcxCadDocument {
    let mut d = empty();
    d.id_counters.next_entity_id = 9_007_199_254_740_994;
    d.id_counters.next_block_id = 10;
    let line = primitives().model.entities[0].clone();
    d.blocks = vec![
        IfcxCadBlockDefinition {
            id: 3,
            name: "Inner".into(),
            base_point: base,
            insertion_unit: "mm".into(),
            entities: vec![line],
        },
        IfcxCadBlockDefinition {
            id: 9,
            name: "Outer".into(),
            base_point: [0.; 3],
            insertion_unit: "mm".into(),
            entities: vec![
                instance(15, 3, [0.; 3]),
                IfcxCadEntity {
                    line_pattern_scale: 1.,
                    id: 80,
                    ..primitives().model.entities[0].clone()
                },
            ],
        },
        IfcxCadBlockDefinition {
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
    let IfcxCadEntityKind::BlockInstance { transform, .. } = &mut d.model.entities[0].kind else {
        unreachable!()
    };
    transform.rotation = std::f64::consts::FRAC_PI_2;
    transform.scale[0] = -2.;
    d
}
pub fn instance(id: u64, definition_id: u64, origin: [f64; 3]) -> IfcxCadEntity {
    IfcxCadEntity {
        line_pattern_scale: 1.,
        id,
        layer_id: 0,
        appearance: modes(),
        kind: IfcxCadEntityKind::BlockInstance {
            definition_id,
            transform: IfcxCadBlockTransform {
                placement: placement(origin),
                rotation: 0.,
                scale: [2., 4., 1.],
            },
        },
    }
}

pub fn native_viewport() -> IfcxCadViewport {
    IfcxCadViewport {
        model_id: 1,
        frame: IfcxCadViewportFrame {
            center: [100., 75.],
            width: 160.,
            height: 100.,
        },
        view: IfcxCadViewportView {
            center: [0., 0.],
            target: [0.; 3],
            direction: [0., 0., 100.],
            height: 200.,
            twist: 0.,
            projection: IfcxCadViewportProjection::Perspective,
            lens_length_mm: Some(50.),
            front_clip: IfcxCadViewportDepthClip {
                mode: IfcxCadViewportClipMode::Disabled,
                distance: Some(0.),
            },
            back_clip: IfcxCadViewportDepthClip {
                mode: IfcxCadViewportClipMode::Disabled,
                distance: Some(0.),
            },
        },
        render_mode: IfcxCadViewportRenderMode::Wireframe,
        view_enabled: false,
        view_locked: true,
        visible: false,
        paper_clip: IfcxCadViewportPaperClip {
            enabled: true,
            boundary_entity_id: Some(1002),
        },
        frozen_layers: vec![4],
    }
}
pub fn viewport_drawing() -> IfcxCadDocument {
    let mut d = empty();
    d.id_counters.next_entity_id = 1003;
    d.id_counters.next_layout_id = 43;
    let appearance = IfcxCadEntityAppearance {
        color: IfcxCadMode::ByLayer,
        opacity: IfcxCadMode::ByLayer,
        line_weight: IfcxCadMode::ByLayer,
        line_pattern: IfcxCadMode::ByLayer,
    };
    let entity = |id, kind| IfcxCadEntity {
        id,
        layer_id: 0,
        appearance: appearance.clone(),
        line_pattern_scale: 1.,
        kind,
    };
    d.paper_layouts.push(IfcxCadPaperLayout {
        id: 42,
        name: "Sheet".into(),
        tab_index: 1,
        length_unit: "mm".into(),
        paper: None,
        entities: vec![
            entity(
                1000,
                IfcxCadEntityKind::LineSegment {
                    start: [0.; 3],
                    end: [1., 0., 0.],
                },
            ),
            entity(1001, IfcxCadEntityKind::Viewport(native_viewport())),
            entity(
                1002,
                IfcxCadEntityKind::Circle {
                    radius: 50.,
                    placement: IfcxCadPlacement {
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
