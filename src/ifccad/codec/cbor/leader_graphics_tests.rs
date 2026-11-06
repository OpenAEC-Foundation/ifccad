use super::*;

fn roundtrip<T: CborValue + std::fmt::Debug>(value: T) -> T {
    let mut writer = CborWriter::new();
    value.encode(&mut writer).unwrap();
    let bytes = writer.finish().unwrap().bytes;
    let back = T::decode(&mut CborReader::new(&bytes, 0, "/leader".into())).unwrap();
    assert!(value.exact(&back), "{value:?} differs from {back:?}");
    back
}

#[test]
fn command_and_primitive_codecs_keep_order_binary64_and_absence() {
    use IfccadLeaderGraphicCommand as C;
    use IfccadLeaderGraphicInheritance as I;
    use IfccadLeaderGraphicPrimitive as P;
    let text = IfccadLeaderGraphicText {
        position: [-0., 1., 2.],
        normal: [0., 0., 1.],
        direction: [1., 0., 0.],
        text: "%%c 🙂".into(),
        processing: IfccadLeaderGraphicTextProcessing::Raw,
        height: 2.5,
        width_factor: 1.,
        oblique_angle: -0.,
        tracking: 1.,
        backwards: false,
        upside_down: true,
        vertical: true,
        underline: false,
        overline: true,
        font: Some(IfccadLeaderGraphicFont {
            family: String::new(),
            font_file: "simplex.shx".into(),
            big_font_file: String::new(),
            bold: None,
            italic: Some(false),
            charset: None,
            pitch_family: Some(0),
        }),
    };
    let points = [[1., 0., 0.], [0., 1., 0.], [-1., 0., 0.]];
    let mut commands = vec![
        C::StoredExtents {
            min: [-1.; 3],
            max: [1.; 3],
        },
        C::SetColor(I::Owner),
        C::SetColor(I::ByLayer),
        C::SetColor(I::ByBlock),
        C::SetColor(I::Explicit(IfccadColor::Foreground)),
        C::SetLayer(None),
        C::SetLayer(Some(u64::MAX)),
        C::SetLinePattern(I::Explicit(IfccadLinePatternId(u64::MAX))),
        C::SetSelectionMarker(u32::MAX),
        C::SetFill(false),
        C::SetLineWeight(IfccadLeaderGraphicLineWeight::Owner),
        C::SetLineWeight(IfccadLeaderGraphicLineWeight::ByLayer),
        C::SetLineWeight(IfccadLeaderGraphicLineWeight::ByBlock),
        C::SetLineWeight(IfccadLeaderGraphicLineWeight::DrawingDefault),
        C::SetLineWeight(IfccadLeaderGraphicLineWeight::Explicit(-0.)),
        C::SetLinePatternScale(1.),
        C::SetThickness(-0.),
        C::PushTransform([0.; 16]),
        C::PopTransform,
        C::CadCompatibility51Zero,
    ];
    commands.extend(
        [
            P::Point([-0.; 3]),
            P::Polyline {
                vertices: points.to_vec(),
                normal: None,
            },
            P::Polyline {
                vertices: points.to_vec(),
                normal: Some([0., 0., 1.]),
            },
            P::Polygon {
                vertices: points.to_vec(),
            },
            P::Circle {
                center: [0.; 3],
                radius: 1.,
                normal: [0., 0., 1.],
            },
            P::Circle3 { points },
            P::Arc {
                center: [0.; 3],
                radius: 1.,
                normal: [0., 0., 1.],
                start_direction: [1., 0., 0.],
                sweep_angle: -1.,
                closure: IfccadLeaderGraphicArcClosure::Sector,
            },
            P::Arc3 {
                points,
                closure: IfccadLeaderGraphicArcClosure::Chord,
            },
            P::Text(Box::new(text)),
        ]
        .into_iter()
        .map(C::Draw),
    );
    let snapshot = IfccadLeaderGraphics {
        origin: IfccadLeaderGraphicsOrigin::Generated,
        placement: IfccadPlacement {
            origin: [0.; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        commands,
    };
    let back = roundtrip(snapshot);
    let C::Draw(P::Text(text)) = back.commands.last().unwrap() else {
        panic!()
    };
    assert_eq!(text.font.as_ref().unwrap().bold, None);
    assert_eq!(text.font.as_ref().unwrap().italic, Some(false));
}

#[test]
fn command_codec_rejects_unknown_duplicate_null_and_wrong_variant_fields() {
    for bytes in [
        vec![0xa1, 0, 13],                  // unknown command
        vec![0xa2, 0, 12, 1, 0xa0],         // unit command with a body
        vec![0xa2, 0, 12, 0, 12],           // duplicate discriminator
        vec![0xa2, 0, 3, 1, 0xa1, 0, 0xf6], // SetLayer explicit null
        vec![0xa2, 0, 0, 1, 0xa0],          // Draw missing primitive
    ] {
        assert!(IfccadLeaderGraphicCommand::decode(&mut CborReader::new(
            &bytes,
            0,
            "/command".into()
        ))
        .is_err());
    }
}

#[test]
fn classic_value_codecs_keep_external_target_and_independent_context() {
    let placement = IfccadPlacement {
        origin: [-0.; 3],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    };
    let leader = IfccadLeader {
        placement: placement.clone(),
        path: IfccadLeaderPath::SplineThroughVertices(vec![[1., 2., 3.], [4., 5., 6.]]),
        annotation_kind: IfccadLeaderAnnotationKind::Tolerance,
        annotation_entity: Some(u64::MAX),
        dimension_style: IfccadDimensionStyleId(600),
        overrides: Default::default(),
        arrow: Some(IfccadLeaderArrow::None),
        arrow_enabled: false,
        hook_enabled: true,
        hook_direction: IfccadLeaderHookDirection::AlongHorizontal,
        annotation_box: Some([-0., 3.]),
        horizontal_direction: [0., 1., 0.],
        block_offset: [1.; 3],
        annotation_offset: [2.; 3],
        origin: [3.; 3],
        annotation: Some(IfccadAnnotation {
            orientation: None,
            mode: IfccadAnnotationMode::Disabled,
            reference: None,
            contexts: vec![IfccadAnnotationContext {
                key: IfccadAnnotationContextKey {
                    scale_id: IfccadAnnotationScaleId(900),
                },
                presentation: Some(IfccadLeaderAnnotationContext {
                    points: vec![[9.; 3]],
                    horizontal_direction: [1., 0., 0.],
                    annotation_enabled: true,
                    insertion_offset: [-0.; 3],
                    endpoint_projection: [8.; 3],
                }),
            }],
        }),
        graphics: None,
    };
    let back = roundtrip(leader);
    assert_eq!(back.annotation_entity, Some(u64::MAX));
    assert_eq!(back.arrow, Some(IfccadLeaderArrow::None));
    roundtrip(IfccadTolerance {
        placement,
        dimension_style: IfccadDimensionStyleId(700),
        overrides: Default::default(),
        notation: IfccadToleranceNotation::CadFeatureControlTextV1 {
            code: "{\\Fgdt;p}%%v%%v🙂\r\n".into(),
        },
        annotation: None,
        graphics: None,
    });
    roundtrip(IfccadToleranceAnnotationContext {
        location: [2., 4., 6.],
        horizontal_direction: [0., 1., 0.],
    });
    roundtrip(IfccadLeaderPath::StraightSegments(vec![]));
    roundtrip(IfccadLeaderArrow::Symbol {
        definition_id: u64::MAX,
    });
    roundtrip(IfccadLeaderArrow::DefaultClosedFilled);
}

#[test]
fn branch_value_codecs_keep_local_and_source_keys_and_reject_duplicate_overrides() {
    use std::collections::BTreeSet;
    let properties = BTreeSet::from([
        IfccadMultiLeaderLineProperty::PathKind,
        IfccadMultiLeaderLineProperty::ArrowSize,
    ]);
    let group = IfccadMultiLeaderGroup {
        key: 4,
        source_index: Some(-5),
        has_connection_point: false,
        connection_point: [-0.; 3],
        has_dogleg_direction: false,
        dogleg_direction: [0.; 3],
        dogleg_length: 5.,
        text_attachment_direction: IfccadMultiLeaderAttachmentDirection::Vertical,
        breaks: vec![],
        lines: vec![IfccadMultiLeaderLine {
            key: 9,
            source_index: None,
            settings: IfccadMultiLeaderLineSettings {
                path_kind: IfccadMultiLeaderPathKind::Invisible,
                color: IfccadMode::ByBlock,
                line_pattern: IfccadMode::ByLayer,
                line_weight: IfccadLeaderLineWeight::DrawingDefault,
                arrow: IfccadLeaderArrow::None,
                arrow_size: 0.,
            },
            overridden_properties: properties.clone(),
            vertices: vec![],
            connection: IfccadMultiLeaderConnection::ExplicitLastVertex,
            breaks: vec![IfccadMultiLeaderSegmentBreaks {
                segment_index: 123,
                intervals: vec![IfccadLeaderBreakInterval {
                    start: [1.; 3],
                    end: [2.; 3],
                }],
            }],
        }],
    };
    // Structural value encoding is separate from full drawing semantic validation.
    let back = roundtrip(group);
    assert_eq!(back.source_index, Some(-5));
    assert_eq!(back.lines[0].source_index, None);
    assert_eq!(back.lines[0].overridden_properties, properties);
    assert!(
        BTreeSet::<IfccadMultiLeaderLineProperty>::decode(&mut CborReader::new(
            &[0x82, 0, 0],
            0,
            "/overrides".into()
        ))
        .is_err()
    );
    assert!(
        BTreeSet::<IfccadMultiLeaderLineProperty>::decode(&mut CborReader::new(
            &[0x81, 6],
            0,
            "/overrides".into()
        ))
        .is_err()
    );
}

#[test]
fn stored_content_codecs_keep_inactive_controls_and_both_block_transforms() {
    let plane = IfccadPlacement {
        origin: [-0.; 3],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    };
    let state = IfccadMultiLeaderTextState {
        stored_rotation: Some(-0.375),
        boundary_height: -0.,
        automatic_height: false,
        word_break: true,
        color: IfccadMode::ByBlock,
        attachments: IfccadMultiLeaderTextAttachments {
            left: IfccadMultiLeaderTextAttachment::MiddleOfText,
            right: IfccadMultiLeaderTextAttachment::BottomOfTopLineUnderlineAll,
            top: IfccadMultiLeaderTextAttachment::CenterOfText,
            bottom: IfccadMultiLeaderTextAttachment::CenterOfTextOverline,
        },
        attachment_point: IfccadMultiLeaderTextAttachmentPoint::Right,
        alignment: IfccadMultiLeaderTextAlignment::Center,
        background: IfccadMultiLeaderTextBackground {
            enabled: false,
            mask_enabled: true,
            color: IfccadMode::ByLayer,
            scale_factor: 0.,
            opacity: IfccadMode::Explicit(0.5),
        },
    };
    roundtrip(state);
    let presentation = IfccadMultiLeaderPresentation {
        plane: plane.clone(),
        normal_reversed: true,
        content_base_point: [1., 2., 3.],
        scale_factor: 2.,
        text_height: 0.,
        arrow_size: 0.,
        landing_gap: -0.,
        next_part_id: 1,
        groups: vec![],
        content: IfccadMultiLeaderStoredContent {
            m_text: None,
            block: Some(IfccadMultiLeaderBlockContent {
                definition_id: u64::MAX,
                transform: IfccadBlockTransform {
                    placement: plane.clone(),
                    rotation: 0.5,
                    scale: [-1., 2., 3.],
                },
                color: IfccadMode::ByBlock,
                connection: IfccadMultiLeaderBlockConnection::BasePoint,
                stored_transform: Some([-0.; 16]),
            }),
            tolerance: Some(Box::new(IfccadMultiLeaderToleranceContent {
                placement: plane,
                dimension_style: IfccadDimensionStyleId(9),
                overrides: Default::default(),
                notation: IfccadToleranceNotation::CadFeatureControlTextV1 {
                    code: "%%v0.1%%vA".into(),
                },
            })),
        },
    };
    let back = roundtrip(presentation);
    assert!(back.content.m_text.is_none());
    assert!(back.content.block.is_some());
    assert!(back.content.tolerance.is_some());
    roundtrip(IfccadMultiLeaderBlockLabel {
        key: 5,
        definition_id: None,
        source_index: Some(-1),
        width: -0.,
        text: String::new(),
    });
}

#[test]
fn settings_style_and_override_codecs_keep_independent_values() {
    let mut style = IfccadMultiLeaderStyle::new(IfccadMultiLeaderStyleId(u64::MAX), "style");
    style.common.block.scale = [-0., -1., 3.];
    style.block_scale_enabled = false;
    style.description = "🙂".into();
    style.annotative = true;
    roundtrip(style);
    let mut leader = IfccadMultiLeader::new();
    leader.style_id = Some(IfccadMultiLeaderStyleId(41));
    leader.settings.common.line.arrow_size = 9.;
    leader.presentation.arrow_size = 5.;
    leader.overridden_properties.extend([
        IfccadMultiLeaderProperty::ArrowSize,
        IfccadMultiLeaderProperty::UseDefaultMText,
    ]);
    let back = roundtrip(leader);
    assert_eq!(back.presentation.arrow_size, 5.);
    assert_eq!(back.settings.common.line.arrow_size, 9.);
    assert_eq!(back.overridden_properties.len(), 2);
    assert!(
        std::collections::BTreeSet::<IfccadMultiLeaderProperty>::decode(&mut CborReader::new(
            &[0x82, 0, 0],
            0,
            "/overrides".into()
        ))
        .is_err()
    );
}
